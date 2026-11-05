//! The garden tools on the sim side (WP25): turns the UI's [`PlayerAction`]s into sim
//! actions, fills the seed library from the snapshot, and starts a new run from the
//! new-run dialog's choices (scenario, weather preset or file, latitude, sowing date).

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use bevy::prelude::{Commands, Res, ResMut, Resource};
use ps_ui::{Charts, Inspector, NewRun, PlayerAction, SeedRow, TimeControl, Tools, days_in};
use ps_view::Selection;
use ps_weather::presets::Preset;
use ps_world::{
    ActionKind, Catalog, ObstacleSpec, Scenario, Start, SupportSpec, WeatherSource, World,
};

use crate::sim_source::SimSource;

/// What a new run needs from the command line: where the data is and how fast to run.
#[derive(Clone, Debug, Resource)]
pub struct RunSettings {
    /// The data directory.
    pub data: PathBuf,
    /// Speed to run a new run at, sim hours per second.
    pub speed: f64,
}

/// Scenario files under `data/scenarios`, relative to the data directory, sorted.
#[must_use]
pub fn scenario_files(data: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut dirs = vec![data.join("scenarios")];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                dirs.push(p);
            } else if p.extension().is_some_and(|x| x == "ron")
                && let Ok(rel) = p.strip_prefix(data)
            {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out.sort();
    out
}

fn is_csv(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("csv"))
}

/// Weather choices: the presets by name, then hourly files under `data/weather`.
#[must_use]
pub fn weather_choices(data: &Path) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(data.join("weather"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            is_csv(&name).then(|| format!("weather/{name}"))
        })
        .collect();
    files.sort();
    Preset::ALL
        .iter()
        .map(|p| p.name().to_owned())
        .chain(files)
        .collect()
}

/// The weather source a choice names: a file path or a preset.
#[must_use]
pub fn weather_source(choice: &str) -> WeatherSource {
    if is_csv(choice) {
        WeatherSource::File(choice.to_owned())
    } else {
        WeatherSource::Preset(choice.to_owned())
    }
}

/// Fills the dialog's lists and the scenario's own latitude and date.
pub fn init_new_run(run: &mut NewRun, data: &Path, scenario: &Scenario, current: &Path) {
    run.scenarios = scenario_files(data);
    run.weathers = weather_choices(data);
    let current = current.to_string_lossy().replace('\\', "/");
    run.scenario = run
        .scenarios
        .iter()
        .position(|s| current.ends_with(s.as_str()))
        .unwrap_or(0);
    defaults_from(run, scenario);
}

fn defaults_from(run: &mut NewRun, scenario: &Scenario) {
    run.latitude_deg = scenario.latitude_deg;
    run.month = scenario.start.month;
    run.day = scenario.start.day;
    run.weather = None;
}

/// The scenario the dialog describes: the chosen file with its latitude, sowing date and
/// weather replaced. A heat wave is dropped when the weather is a file (it needs a
/// preset).
///
/// # Errors
/// I/O, parse or validation errors.
pub fn chosen_scenario(run: &NewRun, data: &Path) -> Result<Scenario> {
    let rel = run
        .scenarios
        .get(run.scenario)
        .context("no scenario chosen")?;
    let text = std::fs::read_to_string(data.join(rel)).with_context(|| format!("reading {rel}"))?;
    let mut s = Scenario::from_ron(&text).with_context(|| rel.clone())?;
    s.latitude_deg = run.latitude_deg;
    s.start = Start {
        month: run.month.clamp(1, 12),
        day: run.day.clamp(1, days_in(run.month.clamp(1, 12))),
        ..s.start
    };
    if let Some(w) = run.weather.and_then(|i| run.weathers.get(i)) {
        s.weather = weather_source(w);
        if !matches!(s.weather, WeatherSource::Preset(_)) {
            s.heat_wave = None;
        }
    }
    s.validate().context("checking the new run")?;
    Ok(s)
}

/// The sim action a player action asks for, for the plant and soil on screen.
fn to_action(src: &SimSource, a: &PlayerAction) -> Option<ActionKind> {
    let organ = |i: usize| u32::try_from(i).ok();
    Some(match *a {
        PlayerAction::Harvest { pot, organ: i } => ActionKind::Harvest {
            plant: src.plant_of_pot(pot)?,
            organ: organ(i)?,
        },
        PlayerAction::Prune { pot, organ: i } => ActionKind::Prune {
            plant: src.plant_of_pot(pot)?,
            organ: organ(i)?,
        },
        PlayerAction::Water { pot, mm } => ActionKind::Water {
            soil: src.soil_of_pot(pot)?,
            mm,
        },
        PlayerAction::AddSupport {
            pot,
            offset_cm,
            height_cm,
            diameter_cm,
        } => ActionKind::AddSupport(SupportSpec {
            soil: src.soil_of_pot(pot)?,
            offset_cm: (offset_cm[0], offset_cm[1]),
            height_cm,
            diameter_cm,
        }),
        PlayerAction::PlaceObstacle {
            pot,
            min_cm,
            max_cm,
            transmittance,
        } => ActionKind::PlaceObstacle(ObstacleSpec {
            soil: src.soil_of_pot(pot)?,
            min_cm: (min_cm[0], min_cm[1], min_cm[2]),
            max_cm: (max_cm[0], max_cm[1], max_cm[2]),
            transmittance,
        }),
    })
}

/// Sends queued player actions to the sim and refreshes the seed library.
#[allow(clippy::needless_pass_by_value)]
pub fn apply_tools(src: Res<SimSource>, mut tools: ResMut<Tools>) {
    tools.enabled = true;
    for a in std::mem::take(&mut tools.queue) {
        if let Some(kind) = to_action(&src, &a) {
            src.act(kind);
        }
    }
    let snap = src.snapshot();
    if tools.seed_library.len() != snap.seed_library.len() {
        tools.seed_library = snap
            .seed_library
            .iter()
            .map(|lot| SeedRow {
                day: lot.day,
                source: snap
                    .plants
                    .iter()
                    .position(|p| p.id == lot.plant)
                    .map_or_else(
                        || format!("Plant id {}", lot.plant),
                        |i| format!("Plant {}", i + 1),
                    ),
                species: lot.species.clone(),
                seeds: lot.seeds,
                mass_g: lot.seed_mass_g,
                mature: lot.mature,
            })
            .collect();
    }
}

/// Reloads the dialog's defaults when its scenario changes, and starts the new run when
/// asked. A failure leaves the current run going and says why in the weather line.
#[allow(clippy::needless_pass_by_value)]
pub fn new_run(
    mut commands: Commands,
    settings: Res<RunSettings>,
    mut tools: ResMut<Tools>,
    mut selection: ResMut<Selection>,
    mut inspector: ResMut<Inspector>,
    mut charts: ResMut<Charts>,
    mut time_ui: ResMut<TimeControl>,
) {
    let run = &mut tools.new_run;
    if run.reload_defaults {
        run.reload_defaults = false;
        if let Some(rel) = run.scenarios.get(run.scenario)
            && let Ok(text) = std::fs::read_to_string(settings.data.join(rel))
            && let Ok(s) = Scenario::from_ron(&text)
        {
            defaults_from(run, &s);
        }
    }
    if !run.start {
        return;
    }
    run.start = false;
    match start(run, &settings) {
        Ok(src) => {
            commands.insert_resource(src);
            selection.picked = None;
            *inspector = Inspector::default();
            charts.rows = usize::MAX;
            tools.seed_library.clear();
        }
        Err(e) => {
            run.open = true;
            time_ui.weather = format!("New run failed: {e:#}");
        }
    }
}

fn start(run: &NewRun, settings: &RunSettings) -> Result<SimSource> {
    let scenario = chosen_scenario(run, &settings.data)?;
    let catalog =
        Catalog::load(&settings.data, &scenario).context("loading species and weather")?;
    let world = World::new(scenario.clone(), catalog).context("building the world")?;
    SimSource::start(world, &scenario, 0.0, settings.speed, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> PathBuf {
        crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"))
    }

    #[test]
    fn the_dialog_lists_scenarios_presets_and_files() {
        let data = data();
        let scenarios = scenario_files(&data);
        assert!(
            scenarios
                .iter()
                .any(|s| s == crate::sim_source::DEFAULT_SCENARIO)
        );
        assert!(
            scenarios
                .iter()
                .any(|s| s == "scenarios/m2/pole_row_trellis.ron")
        );
        let weathers = weather_choices(&data);
        assert_eq!(weathers[0], "temperate_summer");
        assert!(
            weathers
                .iter()
                .any(|w| w == "weather/sample_hourly_45n.csv")
        );
        assert_eq!(
            weather_source("weather/sample_hourly_45n.csv"),
            WeatherSource::File("weather/sample_hourly_45n.csv".into())
        );
    }

    #[test]
    fn a_new_run_takes_the_chosen_latitude_date_and_weather() {
        let data = data();
        let path = data.join(crate::sim_source::DEFAULT_SCENARIO);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| unreachable!("{e}"));
        let base = Scenario::from_ron(&text).unwrap_or_else(|e| unreachable!("{e}"));
        let mut run = NewRun::default();
        init_new_run(
            &mut run,
            &data,
            &base,
            Path::new(crate::sim_source::DEFAULT_SCENARIO),
        );
        assert_eq!(
            run.scenarios[run.scenario],
            crate::sim_source::DEFAULT_SCENARIO
        );
        assert_eq!((run.month, run.day), (base.start.month, base.start.day));
        run.latitude_deg = 52.5;
        run.month = 6;
        run.day = 31; // clamped to 30
        run.weather = run.weathers.iter().position(|w| w == "overcast");
        let s = chosen_scenario(&run, &data).unwrap_or_else(|e| unreachable!("{e:#}"));
        assert!((s.latitude_deg - 52.5).abs() < 1e-12);
        assert_eq!((s.start.month, s.start.day), (6, 30));
        assert_eq!(s.weather, WeatherSource::Preset("overcast".into()));
        let catalog = Catalog::load(&data, &s).unwrap_or_else(|e| unreachable!("{e:#}"));
        World::new(s, catalog).unwrap_or_else(|e| unreachable!("{e:#}"));
    }
}
