//! WP16 done-when: the live session's hash equals the headless replay's hash.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ps_session::{Session, SessionOptions, TimeCommand};
use ps_world::{ActionKind, Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn load(name: &str, days: u32) -> (Scenario, Catalog) {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.duration_days = days;
    let c = Catalog::load(&data(), &s).unwrap();
    (s, c)
}

fn spawn(s: &Scenario, c: &Catalog) -> Session {
    let w = World::new(s.clone(), c.clone()).unwrap();
    Session::spawn(w, SessionOptions::default()).unwrap()
}

fn step(session: &Session, hours: f64) -> u64 {
    session.time(TimeCommand::Step(hours));
    session.wait_idle().unwrap()
}

fn seek(session: &Session, hour: f64) -> u64 {
    session.time(TimeCommand::Seek(hour));
    session.wait_idle().unwrap()
}

/// The done-when: a session driven by every kind of time command, with actions taken
/// while stepping, while running on the wall clock and after scrubbing back (a branch),
/// gives the hash of a headless replay of its log.
#[test]
fn live_session_hash_equals_headless_replay() {
    let (s, c) = load("two_pot_sun_shade.ron", 12);
    let session = spawn(&s, &c);
    assert_eq!(step(&session, 48.0), 48);
    session
        .act_and_wait(ActionKind::Water { soil: 0, mm: 30.0 })
        .unwrap();
    // Run on the wall clock and act at whatever tick the sim has reached.
    session.time(TimeCommand::SetSpeed(400.0));
    std::thread::sleep(Duration::from_millis(60));
    let at = session
        .act_and_wait(ActionKind::SetRadiationFraction {
            soil: 1,
            fraction: 0.3,
        })
        .unwrap();
    assert!(at >= 48, "{at}");
    session.time(TimeCommand::TogglePause);
    let paused_at = session.wait_idle().unwrap();
    // Scrub back past the shade change and act there: a new branch.
    assert_eq!(seek(&session, 30.5), 30);
    let snap = session.latest();
    assert_eq!(snap.tick, 30);
    assert!(snap.latest_tick >= paused_at);
    assert_eq!(snap.actions.len(), 2);
    assert_eq!(
        session
            .act_and_wait(ActionKind::Water { soil: 1, mm: 15.0 })
            .unwrap(),
        30
    );
    let snap = session.latest();
    assert_eq!(snap.latest_tick, 30, "the recorded future is discarded");
    assert_eq!(snap.actions.len(), 1);
    // To the end.
    assert_eq!(step(&session, 1e9), 12 * 24);
    let snap = session.latest();
    assert!(snap.paused);
    let live = session.stop().unwrap();

    let log = live.state().log.clone();
    assert_eq!(log.len(), 1);
    let replay = World::replay(s, c, &log, live.tick()).unwrap();
    assert_eq!(replay.state().hashes, live.state().hashes);
    assert_eq!(replay.state_hash(), live.state_hash());
    assert_eq!(snap.day_hash, live.state().hashes.last().copied());
}

/// The same with no scrubbing: actions at three ticks, the replay matches.
#[test]
fn actions_while_stepping_replay_to_the_same_hash() {
    let (s, c) = load("single_pot.ron", 10);
    let session = spawn(&s, &c);
    step(&session, 30.0);
    session
        .act_and_wait(ActionKind::Water { soil: 0, mm: 12.0 })
        .unwrap();
    step(&session, 1.0);
    session
        .act_and_wait(ActionKind::SetRadiationFraction {
            soil: 0,
            fraction: 0.6,
        })
        .unwrap();
    step(&session, 100.0);
    session
        .act_and_wait(ActionKind::Water { soil: 0, mm: 5.0 })
        .unwrap();
    step(&session, 1e9);
    let live = session.stop().unwrap();
    let ticks: Vec<u64> = live.state().log.iter().map(|a| a.tick).collect();
    assert_eq!(ticks, [30, 31, 131]);
    let replay = World::replay(s, c, &live.state().log, live.tick()).unwrap();
    assert_eq!(replay.state_hash(), live.state_hash());
}

/// A session with no actions is the headless run.
#[test]
fn no_actions_matches_the_headless_run() {
    let (s, c) = load("single_pot.ron", 8);
    let session = spawn(&s, &c);
    step(&session, 1e9);
    let live = session.stop().unwrap();
    let mut headless = World::new(s, c).unwrap();
    headless.run_to_end().unwrap();
    assert_eq!(live.state().hashes, headless.state().hashes);
    assert_eq!(live.state_hash(), headless.state_hash());
}

/// Scrubbing shows the same bits the session computed, backward and forward.
#[test]
fn seek_shows_the_same_bits() {
    let (s, c) = load("two_pot_sun_shade.ron", 6);
    let session = spawn(&s, &c);
    step(&session, 50.0);
    session
        .act_and_wait(ActionKind::Water { soil: 1, mm: 20.0 })
        .unwrap();
    step(&session, 70.0);
    let at_120 = session.latest();
    let log = at_120.actions.to_vec();
    for target in [70_u64, 3, 0, 119, 120, 49, 50, 51] {
        #[allow(clippy::cast_precision_loss)]
        let got = seek(&session, target as f64);
        assert_eq!(got, target);
        let snap = session.latest();
        let want = World::replay(s.clone(), c.clone(), &log, target).unwrap();
        assert_eq!(snap.plants, want.state().plants, "plants at {target}");
        assert_eq!(snap.soils, want.state().soils, "soils at {target}");
        assert_eq!(snap.latest_tick, 120);
    }
    // Seeking past the latest tick stops at it.
    assert_eq!(seek(&session, 1e6), 120);
    assert_eq!(session.latest().plants, at_120.plants);
    drop(session);
}

/// A bad action is refused and never reaches the log; the session carries on.
#[test]
fn bad_action_is_rejected() {
    let (s, c) = load("single_pot.ron", 3);
    let session = spawn(&s, &c);
    step(&session, 5.0);
    assert!(
        session
            .act_and_wait(ActionKind::Water { soil: 9, mm: 1.0 })
            .is_err()
    );
    assert!(
        session
            .act_and_wait(ActionKind::Water {
                soil: 0,
                mm: f64::NAN
            })
            .is_err()
    );
    session.act(ActionKind::SetRadiationFraction {
        soil: 0,
        fraction: 2.0,
    });
    assert_eq!(step(&session, 1.0), 6);
    let snap = session.latest();
    assert!(snap.rejected.is_some());
    assert!(snap.actions.is_empty());
    assert!(snap.error.is_none());
    let live = session.stop().unwrap();
    assert!(live.state().log.is_empty());
}

/// The wall clock drives time while running, and pausing stops it.
#[test]
fn running_follows_the_speed_and_pause_stops_it() {
    let (s, c) = load("single_pot.ron", 30);
    let session = spawn(&s, &c);
    session.time(TimeCommand::SetSpeed(40.0));
    std::thread::sleep(Duration::from_millis(400));
    session.time(TimeCommand::TogglePause);
    let t = session.wait_idle().unwrap();
    // 16 hours are due; allow a slow machine and a late wake-up.
    assert!((1..=40).contains(&t), "{t}");
    std::thread::sleep(Duration::from_millis(100));
    let snap = session.latest();
    assert_eq!(snap.tick, t);
    assert!(snap.paused);
    #[allow(clippy::cast_precision_loss)]
    let tf = t as f64;
    assert!((snap.hour - tf).abs() < 1e-12);
}
