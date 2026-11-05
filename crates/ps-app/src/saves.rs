//! Save and load on the app side (design plan §6.4, WP26).
//!
//! - Save asks the sim thread for its state at the hour on screen without stopping it
//!   (`Session::save_ron`) and writes `<captures>/saves/<stem>-dayDDD-HHh/` holding
//!   `world.ron` (the full state: scenario, catalog, plants, soils, action log) and
//!   `save.ron` (tick, state hash, model version, date).
//! - Load reads `world.ron`, rebuilds the world, checks that its state hash is the one
//!   the save recorded (the WP26 done-when, on screen) and starts a new run from it.
//! - The replay scrubber's markers come from the snapshot's action log.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use bevy::prelude::{Commands, Res, ResMut};
use ps_ui::{CaptureControl, Charts, Inspector, Marker, SaveLoad, SaveRow, TimeControl, Tools};
use ps_view::Selection;
use ps_world::{ActionKind, World};
use serde::{Deserialize, Serialize};

use crate::capture::{CaptureConfig, capture_name};
use crate::sim_source::SimSource;
use crate::tools::RunSettings;

/// What a save records beside the state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Save")]
pub struct SaveInfo {
    /// The sim tick saved.
    pub tick: u64,
    /// The full-state hash at that tick.
    pub hash: u64,
    /// Model version the state was made with.
    pub model_version: String,
    /// Date and hour, `YYYY-MM-DD HH:00`.
    pub date: String,
    /// Sim day (0 = the scenario's first day).
    pub day: u32,
}

/// The folder saves go in.
#[must_use]
pub fn saves_dir(captures: &Path) -> PathBuf {
    captures.join("saves")
}

/// Writes a save of `saved` into a new folder under `dir`, named `name` (made unique).
///
/// # Errors
/// I/O errors.
pub fn write_save(dir: &Path, name: &str, world_ron: &str, info: &SaveInfo) -> Result<PathBuf> {
    let mut out = dir.join(name);
    let mut n = 2;
    while out.exists() {
        out = dir.join(format!("{name}-{n}"));
        n += 1;
    }
    std::fs::create_dir_all(&out).with_context(|| format!("creating {}", out.display()))?;
    std::fs::write(out.join("world.ron"), world_ron)?;
    let meta = ron::ser::to_string_pretty(info, ron::ser::PrettyConfig::default())?;
    std::fs::write(out.join("save.ron"), meta)?;
    Ok(out)
}

/// The saves under `dir`, by folder name.
#[must_use]
pub fn list_saves(dir: &Path) -> Vec<SaveRow> {
    let mut rows: Vec<SaveRow> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("world.ron").is_file())
        .map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let summary = read_info(&e.path()).map_or_else(
                |_| "no save.ron".to_owned(),
                |i| format!("day {}, {} · model {}", i.day, i.date, i.model_version),
            );
            SaveRow { name, summary }
        })
        .collect();
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    rows
}

fn read_info(dir: &Path) -> Result<SaveInfo> {
    let text = std::fs::read_to_string(dir.join("save.ron"))?;
    Ok(ron::from_str(&text)?)
}

/// Rebuilds the world in save folder `dir` and checks its state hash against the one
/// the save recorded. Returns the world and whether the hash matched (`None` when the
/// save has no record).
///
/// # Errors
/// I/O or snapshot errors.
pub fn load_save(dir: &Path) -> Result<(World, Option<bool>)> {
    let text = std::fs::read_to_string(dir.join("world.ron"))
        .with_context(|| format!("reading {}", dir.join("world.ron").display()))?;
    let world = World::from_snapshot_ron(&text).context("reading the saved world")?;
    let matched = read_info(dir).ok().map(|i| i.hash == world.state_hash());
    Ok((world, matched))
}

/// Starts a run from a save folder at the saved hour.
///
/// # Errors
/// Loading or starting the sim thread.
pub fn start_from(
    dir: &Path,
    speed: f64,
    paused: bool,
) -> Result<(SimSource, ps_world::Scenario, String)> {
    let (world, matched) = load_save(dir)?;
    let scenario = world.state().scenario.clone();
    let tick = world.state().tick;
    let day = (tick as f64 + f64::from(scenario.start.hour)) / 24.0;
    let note = match matched {
        Some(true) => format!("Loaded day {day:.0}: the state hash matches the save."),
        Some(false) => format!(
            "Loaded day {day:.0}, but its state hash differs from the save (another model version?)."
        ),
        None => format!("Loaded day {day:.0}."),
    };
    // A start day of 0 steps nothing: the run continues from the saved hour.
    let src = SimSource::start(world, &scenario, 0.0, speed, paused)?;
    Ok((src, scenario, note))
}

/// A short label for an action on the scrubber.
#[must_use]
pub fn action_label(kind: &ActionKind) -> String {
    let debug = format!("{kind:?}");
    let name = debug
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or("Action");
    let mut s = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push(' ');
            s.extend(c.to_lowercase());
        } else {
            s.push(c);
        }
    }
    match kind {
        ActionKind::Water { mm, .. } => format!("{s} {mm:.0} mm"),
        _ => s,
    }
}

/// Saves, lists and loads on request, and keeps the scrubber's markers current.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn save_load(
    mut commands: Commands,
    settings: Res<RunSettings>,
    config: Res<CaptureConfig>,
    src: Res<SimSource>,
    mut saves: ResMut<SaveLoad>,
    mut time: ResMut<TimeControl>,
    mut capture: ResMut<CaptureControl>,
    mut selection: ResMut<Selection>,
    mut inspector: ResMut<Inspector>,
    mut charts: ResMut<Charts>,
    mut tools: ResMut<Tools>,
) {
    let snap = src.snapshot();
    // The scenario's own events (a drought from flowering, a shade cloth at day 20)
    // and the player's actions.
    let events = &src.template().scenario.events;
    if time.markers.len() != events.len() + snap.actions.len() {
        let day = |tick: u64| ((tick as f64 - src.day_start_hour()) / 24.0).floor();
        time.markers = events
            .iter()
            .map(|a| (a, "scenario"))
            .chain(snap.actions.iter().map(|a| (a, "you")))
            .map(|(a, who)| Marker {
                hour: a.tick as f64,
                label: format!("Day {:.0}: {} ({who})", day(a.tick), action_label(&a.kind)),
            })
            .collect();
    }
    let dir = saves_dir(&config.dir);
    if saves.save {
        saves.save = false;
        capture.status = match save(&src, &config, &dir) {
            Ok(path) => format!("Saved {}", path.display()),
            Err(e) => format!("Save failed: {e:#}"),
        };
    }
    if saves.refresh {
        saves.refresh = false;
        saves.saves = list_saves(&dir);
    }
    if let Some(name) = saves.load.take() {
        match start_from(&dir.join(&name), settings.speed, false) {
            Ok((new, _, note)) => {
                commands.insert_resource(new);
                selection.picked = None;
                *inspector = Inspector::default();
                charts.rows = usize::MAX;
                tools.seed_library.clear();
                time.markers.clear();
                saves.status.clone_from(&note);
                capture.status = note;
            }
            Err(e) => {
                saves.open = true;
                saves.status = format!("Could not load {name}: {e:#}");
            }
        }
    }
}

fn save(src: &SimSource, config: &CaptureConfig, dir: &Path) -> Result<PathBuf> {
    let saved = src.save()?;
    let snap = src.snapshot();
    let day = (saved.tick as f64 - src.day_start_hour()) / 24.0;
    let hour_of_day = (((day - day.floor()) * 24.0).round() as u32) % 24;
    let d = snap.date;
    let info = SaveInfo {
        tick: saved.tick,
        hash: saved.hash,
        model_version: ps_world::MODEL_VERSION.to_owned(),
        date: format!("{}-{:02}-{:02} {:02}:00", d.year, d.month, d.day, d.hour),
        day: day.max(0.0).floor() as u32,
    };
    write_save(
        dir,
        &capture_name(&config.stem, day, hour_of_day),
        &saved.ron,
        &info,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_save_loads_back_to_its_hash() {
        let data = crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"));
        let (mut w, _) =
            crate::sim_source::load_world(&data, Path::new(crate::sim_source::DEFAULT_SCENARIO))
                .unwrap_or_else(|e| unreachable!("{e:#}"));
        w.run_days(3).unwrap_or_else(|e| unreachable!("{e}"));
        let ron = w.snapshot_ron().unwrap_or_else(|e| unreachable!("{e}"));
        let info = SaveInfo {
            tick: w.state().tick,
            hash: w.state_hash(),
            model_version: ps_world::MODEL_VERSION.to_owned(),
            date: "2000-05-18 06:00".into(),
            day: 3,
        };
        let dir = std::env::temp_dir().join(format!("ps-saves-{}", std::process::id()));
        let a = write_save(&dir, "pot-day003-06h", &ron, &info)
            .unwrap_or_else(|e| unreachable!("{e:#}"));
        let b = write_save(&dir, "pot-day003-06h", &ron, &info)
            .unwrap_or_else(|e| unreachable!("{e:#}"));
        assert_ne!(a, b, "a second save of the same hour gets its own folder");
        let rows = list_saves(&dir);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].summary.starts_with("day 3, 2000-05-18"));
        let (loaded, matched) = load_save(&a).unwrap_or_else(|e| unreachable!("{e:#}"));
        assert_eq!(matched, Some(true));
        assert_eq!(loaded.state().tick, w.state().tick);
        // A tampered record reads as a mismatch.
        let bad = SaveInfo {
            hash: info.hash ^ 1,
            ..info
        };
        let c = write_save(&dir, "bad", &ron, &bad).unwrap_or_else(|e| unreachable!("{e:#}"));
        assert_eq!(
            load_save(&c).unwrap_or_else(|e| unreachable!("{e:#}")).1,
            Some(false)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn actions_get_short_labels() {
        assert_eq!(
            action_label(&ActionKind::Water { soil: 0, mm: 12.0 }),
            "Water 12 mm"
        );
        assert_eq!(
            action_label(&ActionKind::Harvest { plant: 0, organ: 4 }),
            "Harvest"
        );
        assert_eq!(
            action_label(&ActionKind::SetRadiationFraction {
                soil: 0,
                fraction: 0.5
            }),
            "Set radiation fraction"
        );
    }
}
