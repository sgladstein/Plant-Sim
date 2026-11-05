//! The sim thread (design plan §11.9, WP16).
//!
//! [`Session::spawn`] moves a [`World`] onto its own std thread. The thread steps the
//! world at the requested speed and publishes a [`Snapshot`] through a double buffer
//! ([`SnapshotBuffer`]); the renderer reads the latest one without waiting for the sim.
//! The UI sends [`TimeCommand`]s and player actions back over a channel. Actions go
//! through the world's action log only, at the sim's current tick, so a headless replay
//! of the log reaches the session's hash (`tests/session.rs`).
//!
//! Wall-clock time decides only *when* hours are stepped, never what a step computes,
//! and every action lands in the log with the tick it was applied at. That is why the
//! wall clock is allowed here and nowhere in the sim crates below.
//!
//! Scrubbing: the thread keeps a copy of the state at every midnight. A `Seek` into the
//! past restores the nearest copy at or before the target and replays the log forward,
//! so the scrubber shows the same bits the session computed. Running from there replays
//! the recorded timeline, and the world becomes live again when it passes the latest
//! tick. An action taken while behind the latest tick starts a new branch: the recorded
//! future (later actions and midnight copies) is discarded, as in a save loaded and
//! changed. Decision 0016.

mod snapshot;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant; // determinism: allow (pacing only; see the module docs)

use ps_weather::DateHour;
use ps_world::{Action, ActionKind, DayRow, World, WorldError, WorldState};

pub use snapshot::{Snapshot, SnapshotBuffer};

/// A request to the owner of sim time. Mirrors `ps_ui::TimeCommand` one to one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeCommand {
    /// Pause or resume. Pausing also cancels a step in progress.
    TogglePause,
    /// Run at this many sim hours per real second (and resume).
    SetSpeed(f64),
    /// Advance this many sim hours (rounded, at least one) as fast as possible, then
    /// pause.
    Step(f64),
    /// Show this sim hour, from 0 to the latest hour reached (rounded down).
    Seek(f64),
}

/// Errors from the session.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// The thread could not be started.
    #[error("starting the sim thread: {0}")]
    Spawn(String),
    /// The sim thread panicked or is gone.
    #[error("the sim thread stopped unexpectedly")]
    Lost,
    /// The world reported an error.
    #[error(transparent)]
    World(#[from] WorldError),
}

/// How a session starts.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SessionOptions {
    /// Starting speed, sim hours per real second.
    pub speed: f64,
    /// Start paused.
    pub paused: bool,
    /// Keep a copy of the state every this many midnights, for seeking (at least 1).
    pub keyframe_every_days: u32,
    /// Shortest real time between two snapshots while the sim is busy.
    pub publish_interval: Duration,
    /// Longest real time the sim may fall behind the requested speed before it drops
    /// the backlog instead of racing to catch up.
    pub max_lag: Duration,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            speed: 24.0,
            paused: true,
            keyframe_every_days: 1,
            publish_interval: Duration::from_millis(8),
            max_lag: Duration::from_millis(250),
        }
    }
}

/// The answer to an action: the tick it was logged at, or why it was rejected.
type ActResult = Result<u64, String>;

enum Command {
    Time(TimeCommand),
    Act(ActionKind, Option<Sender<ActResult>>),
    WhenIdle(Sender<u64>),
    Stop,
}

/// The handle the app keeps: sends commands, reads snapshots, stops the thread.
#[derive(Debug)]
pub struct Session {
    tx: Sender<Command>,
    buffer: Arc<SnapshotBuffer>,
    thread: Option<JoinHandle<World>>,
}

impl Session {
    /// Moves `world` onto a new thread named `ps-sim` and starts it.
    ///
    /// # Errors
    /// [`SessionError::Spawn`] if the thread cannot be started.
    pub fn spawn(world: World, options: SessionOptions) -> Result<Self, SessionError> {
        let (tx, rx) = mpsc::channel();
        let mut runner = Runner::new(world, options);
        let buffer = Arc::new(SnapshotBuffer::new(runner.snapshot()));
        let out = Arc::clone(&buffer);
        let thread = std::thread::Builder::new()
            .name("ps-sim".into())
            .spawn(move || runner.run(&rx, &out))
            .map_err(|e| SessionError::Spawn(e.to_string()))?;
        Ok(Self {
            tx,
            buffer,
            thread: Some(thread),
        })
    }

    /// Sends a time command. Ignored if the thread is gone.
    pub fn time(&self, c: TimeCommand) {
        let _ = self.tx.send(Command::Time(c));
    }

    /// Queues a player action for the sim's current tick. Ignored if the thread is gone;
    /// a rejection shows in [`Snapshot::rejected`].
    pub fn act(&self, kind: ActionKind) {
        let _ = self.tx.send(Command::Act(kind, None));
    }

    /// Applies a player action and waits for the answer: the tick it was logged at, or
    /// why it was rejected.
    ///
    /// # Errors
    /// The rejection reason, or that the thread is gone.
    pub fn act_and_wait(&self, kind: ActionKind) -> Result<u64, String> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Command::Act(kind, Some(tx)))
            .map_err(|_| "the sim thread is gone".to_string())?;
        rx.recv()
            .map_err(|_| "the sim thread is gone".to_string())?
    }

    /// Waits until earlier commands are handled and no step or seek is in progress, and
    /// returns the world's tick then. A running session (not paused) counts as idle.
    ///
    /// # Errors
    /// [`SessionError::Lost`] if the thread is gone.
    pub fn wait_idle(&self) -> Result<u64, SessionError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Command::WhenIdle(tx))
            .map_err(|_| SessionError::Lost)?;
        rx.recv().map_err(|_| SessionError::Lost)
    }

    /// The latest snapshot, without waiting for the sim.
    #[must_use]
    pub fn latest(&self) -> Arc<Snapshot> {
        self.buffer.latest()
    }

    /// The buffer itself, for a reader that lives elsewhere (for example a Bevy
    /// resource).
    #[must_use]
    pub fn buffer(&self) -> Arc<SnapshotBuffer> {
        Arc::clone(&self.buffer)
    }

    /// Stops the thread and returns the world as it stands (for saving, or for the
    /// replay check).
    ///
    /// # Errors
    /// [`SessionError::Lost`] if the thread panicked.
    pub fn stop(mut self) -> Result<World, SessionError> {
        let _ = self.tx.send(Command::Stop);
        self.thread
            .take()
            .ok_or(SessionError::Lost)?
            .join()
            .map_err(|_| SessionError::Lost)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(t) = self.thread.take() {
            let _ = self.tx.send(Command::Stop);
            let _ = t.join();
        }
    }
}

/// The sim thread's state.
struct Runner {
    world: World,
    options: SessionOptions,
    /// The timeline's action log (the world's log after a restore may hold only a
    /// prefix until the rest is rescheduled).
    log: Vec<Action>,
    /// State copies at midnights, by tick (tick 0 always present).
    keyframes: BTreeMap<u64, WorldState>,
    latest: u64,
    end: u64,
    speed: f64,
    paused: bool,
    /// Sim hours owed by the wall clock while running.
    debt: f64,
    /// Tick a `Step` or `Seek` is heading for, and whether to pause on arrival.
    target: Option<Target>,
    waiters: Vec<Sender<u64>>,
    replies: Vec<(Sender<ActResult>, ActResult)>,
    rejected: Option<String>,
    error: Option<String>,
    seq: u64,
    history: Arc<[DayRow]>,
    actions: Arc<[Action]>,
    /// Rate window: start and hours stepped since.
    rate_start: Instant, // determinism: allow (pacing only)
    rate_hours: u64,
    achieved: f64,
}

impl Runner {
    fn new(world: World, options: SessionOptions) -> Self {
        let mut keyframes = BTreeMap::new();
        keyframes.insert(world.tick(), world.state().clone());
        let log = world.state().log.clone();
        Self {
            latest: world.tick(),
            end: world.end_tick(),
            speed: sane_speed(options.speed).unwrap_or(24.0),
            paused: options.paused,
            history: world.state().history.clone().into(),
            actions: log.clone().into(),
            log,
            keyframes,
            world,
            options,
            debt: 0.0,
            target: None,
            waiters: Vec::new(),
            replies: Vec::new(),
            rejected: None,
            error: None,
            seq: 0,
            rate_start: Instant::now(), // determinism: allow (pacing only)
            rate_hours: 0,
            achieved: 0.0,
        }
    }

    fn idle(&self) -> bool {
        self.target.is_none()
    }

    /// True when the wall clock should drive steps.
    fn running(&self) -> bool {
        !self.paused && self.error.is_none() && self.world.tick() < self.end
    }

    fn run(mut self, rx: &Receiver<Command>, out: &SnapshotBuffer) -> World {
        let mut last = Instant::now(); // determinism: allow (pacing only)
        let mut last_publish = last;
        let mut dirty = false;
        loop {
            // 1. Commands. Block when there is nothing to step; otherwise drain.
            let first = if self.idle() && !self.running() {
                if dirty {
                    self.publish(out);
                    dirty = false;
                }
                self.answer_waiters();
                match rx.recv() {
                    Ok(c) => Some(c),
                    Err(_) => return self.world,
                }
            } else if self.idle() && self.debt < 1.0 {
                // Running, but the next hour is not due yet: sleep until it is (or a
                // frame's worth, so the fractional hour keeps moving).
                let until_due = (1.0 - self.debt) / self.speed;
                let wait = Duration::from_secs_f64(until_due.clamp(0.0, 1.0))
                    .min(self.options.publish_interval);
                match rx.recv_timeout(wait) {
                    Ok(c) => Some(c),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => return self.world,
                }
            } else {
                None
            };
            let was_idle_stopped = self.idle() && !self.running();
            let mut next = first;
            loop {
                let Some(c) = next.take().or_else(|| rx.try_recv().ok()) else {
                    break;
                };
                if matches!(c, Command::Stop) {
                    return self.world;
                }
                self.handle(c);
                dirty = true;
            }
            let now = Instant::now(); // determinism: allow (pacing only)
            if was_idle_stopped {
                // Time spent paused is not owed.
                last = now;
            }
            // 2. Steps: toward a target as fast as possible, or as the clock owes.
            let budget = self.options.publish_interval;
            if let Some(t) = self.target {
                while self.world.tick() < t.tick && now.elapsed() < budget && self.error.is_none() {
                    self.step();
                }
                if self.world.tick() >= t.tick || self.error.is_some() {
                    self.target = None;
                    self.paused |= t.pause;
                    self.debt = 0.0;
                }
                dirty = true;
            } else if self.running() {
                let dt = now.duration_since(last).as_secs_f64();
                let cap = (self.speed * self.options.max_lag.as_secs_f64()).max(1.0);
                self.debt = (self.debt + self.speed * dt).min(cap);
                while self.debt >= 1.0 && self.running() && now.elapsed() < budget {
                    self.step();
                    self.debt -= 1.0;
                }
                if self.world.tick() >= self.end {
                    self.paused = true;
                    self.debt = 0.0;
                }
                dirty = true;
            }
            last = now;
            self.measure();
            // 3. Publish, at most once per interval while busy, and before telling a
            // waiter the session is idle.
            let waiting = (self.idle() && !self.waiters.is_empty()) || !self.replies.is_empty();
            if dirty && (waiting || last_publish.elapsed() >= self.options.publish_interval) {
                self.publish(out);
                last_publish = Instant::now(); // determinism: allow (pacing only)
                dirty = false;
            }
            if self.idle() {
                self.answer_waiters();
            }
        }
    }

    fn measure(&mut self) {
        let secs = self.rate_start.elapsed().as_secs_f64();
        if secs >= 0.5 {
            #[allow(clippy::cast_precision_loss)]
            let hours = self.rate_hours as f64;
            self.achieved = hours / secs;
            self.rate_hours = 0;
            self.rate_start = Instant::now(); // determinism: allow (pacing only)
        }
    }

    fn answer_waiters(&mut self) {
        let tick = self.world.tick();
        for w in self.waiters.drain(..) {
            let _ = w.send(tick);
        }
    }

    fn handle(&mut self, c: Command) {
        match c {
            Command::Time(t) => self.time(t),
            Command::Act(kind, reply) => {
                let r = self.act(kind);
                self.rejected = r.as_ref().err().cloned();
                if let Some(reply) = reply {
                    // Answered after the next publish, like `WhenIdle`.
                    self.replies.push((reply, r));
                }
            }
            // Answered after the next publish, so the waiter sees a current snapshot.
            Command::WhenIdle(w) => self.waiters.push(w),
            Command::Stop => {}
        }
    }

    fn time(&mut self, t: TimeCommand) {
        match t {
            TimeCommand::TogglePause => {
                if self.target.take().is_some() {
                    self.paused = true;
                } else {
                    self.paused = !self.paused;
                }
                self.debt = 0.0;
            }
            TimeCommand::SetSpeed(s) => {
                if let Some(s) = sane_speed(s) {
                    self.speed = s;
                    self.paused = false;
                    self.target = None;
                }
            }
            TimeCommand::Step(h) => {
                if !h.is_finite() || self.error.is_some() {
                    return;
                }
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let n = h.round().clamp(1.0, 1e12) as u64;
                let from = self.target.map_or(self.world.tick(), |t| t.tick);
                self.target = Some(Target {
                    tick: from.saturating_add(n).min(self.end),
                    pause: true,
                });
                self.debt = 0.0;
            }
            TimeCommand::Seek(h) => {
                if !h.is_finite() {
                    return;
                }
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let to = (h.floor().clamp(0.0, 1e12) as u64).min(self.latest);
                self.seek(to);
            }
        }
    }

    fn seek(&mut self, to: u64) {
        self.debt = 0.0;
        self.target = None;
        if to >= self.world.tick() && self.error.is_none() {
            // Forward within the recorded timeline: step there, keeping the pause state.
            if to > self.world.tick() {
                self.target = Some(Target {
                    tick: to,
                    pause: false,
                });
            }
            return;
        }
        match self.rebuilt(to, &self.log) {
            Ok(w) => {
                self.world = w;
                self.error = None;
            }
            Err(e) => self.fail(&e),
        }
        self.refresh_cached();
    }

    /// A world at `to` rebuilt from the nearest midnight copy at or before it and `log`.
    fn rebuilt(&self, to: u64, log: &[Action]) -> Result<World, WorldError> {
        let Some((_, key)) = self.keyframes.range(..=to).next_back() else {
            return Err(WorldError::Snapshot(format!(
                "no state copy before tick {to}"
            )));
        };
        let mut state = key.clone();
        // Unapplied entries are not part of the state (or its hash); the timeline's
        // log supplies them.
        state.log.truncate(state.applied);
        let applied = state.applied;
        let mut w = World::from_state(state)?;
        for a in log.iter().skip(applied) {
            w.schedule(a.clone())?;
        }
        w.run_until(to)?;
        Ok(w)
    }

    fn act(&mut self, kind: ActionKind) -> Result<u64, String> {
        if let Some(e) = &self.error {
            return Err(format!("the world stopped: {e}"));
        }
        let tick = self.world.tick();
        if tick >= self.end {
            return Err("the scenario has ended".into());
        }
        let branch = tick < self.latest;
        // A branch keeps the actions already logged for the current tick (the player
        // is standing on that hour and sees their markers) and drops later ones.
        let branched = if branch {
            let log: Vec<Action> = self
                .log
                .iter()
                .filter(|a| a.tick <= tick)
                .cloned()
                .collect();
            let world = self.rebuilt(tick, &log).map_err(|e| e.to_string())?;
            Some((world, log))
        } else {
            None
        };
        // Try the action on a copy for one hour, so a bad action never enters the log.
        let mut probe = branched
            .as_ref()
            .map_or_else(|| self.world.clone(), |(w, _)| w.clone());
        probe
            .apply(kind.clone())
            .map_err(|e| format!("rejected: {e}"))?;
        probe
            .step_hour()
            .map_err(|e| format!("the hour after this action failed: {e}"))?;
        drop(probe);
        if let Some((world, log)) = branched {
            self.world = world;
            self.log = log;
            self.keyframes.retain(|k, _| *k <= tick);
            self.latest = tick;
        }
        self.world
            .apply(kind.clone())
            .map_err(|e| format!("rejected: {e}"))?;
        self.log.push(Action { tick, kind });
        self.refresh_cached();
        Ok(tick)
    }

    fn step(&mut self) {
        if let Err(e) = self.world.step_hour() {
            self.fail(&e);
            return;
        }
        self.rate_hours += 1;
        let tick = self.world.tick();
        self.latest = self.latest.max(tick);
        let clock = u64::from(self.world.state().scenario.start.hour) + tick;
        let every = u64::from(self.options.keyframe_every_days.max(1));
        if clock.is_multiple_of(24) && (clock / 24).is_multiple_of(every) {
            let state = self.world.state();
            self.keyframes.entry(tick).or_insert_with(|| state.clone());
        }
        if self.history.len() != self.world.state().history.len() {
            self.history = self.world.state().history.clone().into();
        }
    }

    fn fail(&mut self, e: &WorldError) {
        self.error = Some(e.to_string());
        self.paused = true;
        self.target = None;
        self.debt = 0.0;
    }

    fn refresh_cached(&mut self) {
        self.history = self.world.state().history.clone().into();
        self.actions = self.log.clone().into();
    }

    fn snapshot(&mut self) -> Snapshot {
        self.seq += 1;
        let st = self.world.state();
        let tick = self.world.tick();
        let start = self.world.weather().start;
        let tick_i = i64::try_from(tick).unwrap_or(i64::MAX);
        #[allow(clippy::cast_precision_loss)]
        let base = tick as f64;
        let frac = if self.running() && self.idle() {
            self.debt.clamp(0.0, 0.999)
        } else {
            0.0
        };
        Snapshot {
            seq: self.seq,
            tick,
            hour: base + frac,
            latest_tick: self.latest,
            end_tick: self.end,
            speed: self.speed,
            achieved_speed: self.achieved,
            paused: !self.running(),
            busy: !self.idle(),
            date: DateHour::from_epoch_hours(start.epoch_hours().saturating_add(tick_i)),
            weather: self.world.weather_now().copied(),
            soils: st.soils.clone(),
            plants: st.plants.clone(),
            history: Arc::clone(&self.history),
            actions: Arc::clone(&self.actions),
            seed_library: st.seed_library.clone(),
            added_supports: st.added_supports.clone(),
            added_obstacles: st.added_obstacles.clone(),
            day_hash: st.hashes.last().copied(),
            rejected: self.rejected.clone(),
            error: self.error.clone(),
        }
    }

    fn publish(&mut self, out: &SnapshotBuffer) {
        let s = self.snapshot();
        drop(out.publish(s));
        for (reply, r) in self.replies.drain(..) {
            let _ = reply.send(r);
        }
    }
}

/// Where a `Step` or a forward `Seek` is heading.
#[derive(Copy, Clone, Debug)]
struct Target {
    tick: u64,
    /// Pause on arrival (a step does; a seek keeps the pause state).
    pause: bool,
}

fn sane_speed(s: f64) -> Option<f64> {
    (s.is_finite() && s > 0.0).then_some(s.min(1e6))
}
