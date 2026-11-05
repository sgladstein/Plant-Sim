//! Capture (design plan §7 WP19): screenshots, CSV export and time-lapse frames.
//!
//! - The top bar's Screenshot button (or the P key) saves the window as
//!   `captures/<scenario>-day<D>-<HH>h.png`.
//! - Export CSV writes the daily table and the organ table, the same files `ps-cli run`
//!   writes (ps-export), for the state on screen, into
//!   `captures/<scenario>-day<D>-<HH>h/`.
//! - `--timelapse DIR` renders frames offline: the sim steps to each frame's hour while
//!   paused, the scene settles, and the window is saved as `DIR/frame_00000.png` and on.
//!   Frames cover the daylight hours (`--timelapse-hours`, default 6-19) of each day
//!   from `--day` to `--to-day`, one frame per sim hour, so any machine renders the same
//!   clip. Join them with, for example,
//!   `ffmpeg -framerate 24 -i DIR/frame_%05d.png -pix_fmt yuv420p clip.mp4`.
//!
//! The seed library waits for a harvest action in the sim (M0 has none).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context as _, Result};
use bevy::prelude::{
    AppExit, ButtonInput, Commands, KeyCode, MessageWriter, On, Res, ResMut, Resource, info,
};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use ps_ui::{CaptureControl, TimeCommand, TimeControl};
use ps_view::Ready;
use ps_world::WorldState;

use crate::sim_source::SimSource;

/// Where captures go and what they are named after.
#[derive(Clone, Debug, Resource)]
pub struct CaptureConfig {
    /// Output directory.
    pub dir: PathBuf,
    /// File name stem, usually the scenario's.
    pub stem: String,
}

/// `<stem>-day<D>-<HH>h` for the time on screen.
#[must_use]
pub fn capture_name(stem: &str, day: f64, hour_of_day: u32) -> String {
    format!("{stem}-day{:03.0}-{hour_of_day:02}h", day.max(0.0).floor())
}

/// The sim ticks a time-lapse visits: every hour in `hours` (inclusive, hour of day) of
/// each day from `from_day` to `to_day`, where day 0 starts at sim hour
/// `day_start_hour`, clamped to `0..=end_tick`.
#[must_use]
pub fn timelapse_ticks(
    from_day: f64,
    to_day: f64,
    hours: (u32, u32),
    day_start_hour: f64,
    end_tick: u64,
) -> Vec<u64> {
    let (h0, h1) = (hours.0.min(23), hours.1.min(23));
    let mut ticks = Vec::new();
    let first = from_day.max(0.0).floor() as u64;
    let last = to_day.max(0.0).floor() as u64;
    for day in first..=last {
        for h in h0..=h1 {
            let t = (day * 24 + u64::from(h)) as f64 + day_start_hour;
            let start = from_day * 24.0 + day_start_hour;
            if t < 0.0 || t < start.floor() {
                continue;
            }
            let t = t as u64;
            if t > end_tick {
                return ticks;
            }
            ticks.push(t);
        }
    }
    ticks
}

/// The world state on screen, for the CSV export: the starting state with the
/// snapshot's soils, plants, daily rows and action log.
pub(crate) fn state_on_screen(src: &SimSource) -> WorldState {
    let snap = src.snapshot();
    let mut state = src.template().clone();
    state.tick = snap.tick;
    state.soils.clone_from(&snap.soils);
    state.plants.clone_from(&snap.plants);
    state.history = snap.history.to_vec();
    state.log = snap.actions.to_vec();
    state
}

pub(crate) fn write_csv(dir: &Path, state: &WorldState) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let daily = dir.join("daily.csv");
    std::fs::write(&daily, ps_export::daily_csv(state))
        .with_context(|| format!("writing {}", daily.display()))?;
    let organs = dir.join("organs.csv");
    std::fs::write(&organs, ps_export::organs_csv(state))
        .with_context(|| format!("writing {}", organs.display()))?;
    Ok(())
}

fn now_name(config: &CaptureConfig, time: &TimeControl, day_start_hour: f64) -> String {
    let day = (time.hour - day_start_hour) / 24.0;
    let hour_of_day = ((day - day.floor()) * 24.0).round() as u32 % 24;
    capture_name(&config.stem, day, hour_of_day)
}

/// Handles the Screenshot and Export CSV buttons and the P key.
#[allow(clippy::needless_pass_by_value)]
pub fn capture_requests(
    mut commands: Commands,
    config: Res<CaptureConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    src: Option<Res<SimSource>>,
    time: Res<TimeControl>,
    mut control: ResMut<CaptureControl>,
) {
    let day_start = src.as_ref().map_or(0.0, |s| s.day_start_hour());
    let name = now_name(&config, &time, day_start);
    if std::mem::take(&mut control.screenshot) || keys.just_pressed(KeyCode::KeyP) {
        let path = config.dir.join(format!("{name}.png"));
        match std::fs::create_dir_all(&config.dir) {
            Ok(()) => {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path.clone()));
                control.status = format!("Saved {}", path.display());
            }
            Err(e) => control.status = format!("Could not create {}: {e}", config.dir.display()),
        }
    }
    if std::mem::take(&mut control.export_csv) {
        control.status = match &src {
            Some(src) => {
                let dir = config.dir.join(&name);
                match write_csv(&dir, &state_on_screen(src)) {
                    Ok(()) => format!("Wrote daily.csv and organs.csv to {}", dir.display()),
                    Err(e) => format!("Export failed: {e:#}"),
                }
            }
            None => "Export needs the simulation (not the --demo plants)".to_owned(),
        };
    }
}

/// Frames to wait after the sim arrives before saving, so meshes and shadows settle.
const SETTLE_FRAMES: u32 = 4;

#[derive(Debug, Default)]
enum Phase {
    #[default]
    Request,
    Waiting {
        target: u64,
        settled: u32,
    },
    Saving(Arc<AtomicBool>),
}

/// An offline time-lapse in progress.
#[derive(Debug, Resource)]
pub struct Timelapse {
    /// Output directory.
    pub dir: PathBuf,
    /// The sim tick of each frame.
    pub ticks: Vec<u64>,
    next: usize,
    phase: Phase,
}

impl Timelapse {
    /// A time-lapse of `ticks` into `dir`.
    #[must_use]
    pub fn new(dir: PathBuf, ticks: Vec<u64>) -> Self {
        Self {
            dir,
            ticks,
            next: 0,
            phase: Phase::Request,
        }
    }
}

/// Steps the sim to each frame's tick, waits for the scene, saves the frame.
#[allow(clippy::needless_pass_by_value)]
pub fn timelapse(
    mut commands: Commands,
    ready: Res<Ready>,
    src: Res<SimSource>,
    mut time: ResMut<TimeControl>,
    mut lapse: ResMut<Timelapse>,
    mut exit: MessageWriter<AppExit>,
) {
    if !ready.ready {
        return;
    }
    let Some(&target) = lapse.ticks.get(lapse.next) else {
        info!(
            "time-lapse: wrote {} frames to {}",
            lapse.ticks.len(),
            lapse.dir.display()
        );
        exit.write(AppExit::Success);
        return;
    };
    let snap = src.snapshot();
    match &mut lapse.phase {
        Phase::Request => {
            if time.busy {
                return;
            }
            if snap.tick < target {
                time.commands
                    .push(TimeCommand::Step((target - snap.tick) as f64));
            } else if snap.tick > target {
                time.commands.push(TimeCommand::Seek(target as f64));
            }
            lapse.phase = Phase::Waiting { target, settled: 0 };
        }
        Phase::Waiting { target, settled } => {
            if snap.tick == *target && !snap.busy && !time.busy {
                *settled += 1;
            } else {
                *settled = 0;
            }
            if *settled >= SETTLE_FRAMES {
                let path = lapse.dir.join(format!("frame_{:05}.png", lapse.next));
                let saved = Arc::new(AtomicBool::new(false));
                let flag = Arc::clone(&saved);
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path))
                    .observe(move |_: On<ScreenshotCaptured>| {
                        flag.store(true, Ordering::Relaxed);
                    });
                lapse.phase = Phase::Saving(saved);
            }
        }
        Phase::Saving(saved) => {
            if saved.load(Ordering::Relaxed) {
                lapse.next += 1;
                if lapse.next.is_multiple_of(50) {
                    info!("time-lapse: {} of {} frames", lapse.next, lapse.ticks.len());
                }
                lapse.phase = Phase::Request;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_carry_day_and_hour() {
        assert_eq!(capture_name("two_pot", 45.5, 12), "two_pot-day045-12h");
        assert_eq!(capture_name("x", -1.0, 0), "x-day000-00h");
    }

    #[test]
    fn timelapse_visits_daylight_hours_in_order() {
        // Day 0 starts at sim hour −6 (a scenario starting at 06:00).
        let t = timelapse_ticks(1.0, 2.0, (6, 19), -6.0, 10_000);
        assert_eq!(t.len(), 28);
        assert_eq!(t[0], 24); // day 1, 06:00
        assert_eq!(t[13], 24 + 13); // day 1, 19:00
        assert_eq!(t[14], 48);
        assert!(t.windows(2).all(|w| w[0] < w[1]));
        // Clamped at the end of the scenario.
        let t = timelapse_ticks(1.0, 5.0, (6, 19), -6.0, 30);
        assert_eq!(*t.last().unwrap_or(&0), 30);
    }
}
