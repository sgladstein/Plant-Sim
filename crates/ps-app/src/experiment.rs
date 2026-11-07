//! Experiment mode on the app side (design plan §2.5, §5.6, §6.5; WP17).
//!
//! - Experiment cards are data files under `data/experiments/*.ron`, so a new question
//!   needs no code.
//! - [`scenario_for`] turns the dialog's [`ExperimentSetup`] into a scenario: two to four
//!   pots in a row, sown the same day with the same species, one variable differing.
//!   Climbing species get a 2 m stake in every pot.
//! - Starting writes a lab notebook (`notebook.ron`, with the scenario next to it) under
//!   `<captures>/experiments/<name>/`. Record results writes the daily and organ CSVs for
//!   the day on screen into the same folder and adds them to the notebook.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use bevy::prelude::{Commands, Res, ResMut};
use ps_ui::{CardRow, Charts, Experiment, ExperimentSetup, Inspector, Running, Variable};
use ps_view::Selection;
use ps_world::{Catalog, Irrigation, Scenario, SeedChoice, SoilSpec, Sowing, SupportSpec, World};
use serde::{Deserialize, Serialize};

use crate::capture::{CaptureConfig, state_on_screen, write_csv};
use crate::sim_source::{DEFAULT_SCENARIO, SimSource};
use crate::tools::{RunSettings, weather_choices, weather_source};

/// Pot spacing along the row, m.
const SPACING_M: f64 = 0.6;

/// An experiment card file (design plan §6.5).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename = "ExperimentCard")]
struct CardFile {
    id: String,
    prompt: String,
    #[serde(default = "bush_bean")]
    species: String,
    variable: VariableFile,
    levels: Vec<f64>,
    #[serde(default)]
    latitude_deg: Option<f64>,
    #[serde(default)]
    watch: Vec<String>,
}

fn bush_bean() -> String {
    "bush_bean".into()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum VariableFile {
    Light,
    WaterAmount,
    WaterInterval,
    PotSize,
}

impl From<VariableFile> for Variable {
    fn from(v: VariableFile) -> Self {
        match v {
            VariableFile::Light => Self::Light,
            VariableFile::WaterAmount => Self::WaterAmount,
            VariableFile::WaterInterval => Self::WaterInterval,
            VariableFile::PotSize => Self::PotSize,
        }
    }
}

/// The experiment cards under `data/experiments`, sorted by file name. A card that
/// does not parse is skipped with a warning.
#[must_use]
pub fn load_cards(data: &Path) -> Vec<CardRow> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(data.join("experiments"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "ron"))
        .collect();
    files.sort();
    files
        .iter()
        .filter_map(|p| match read_card(p) {
            Ok(c) => Some(c),
            Err(e) => {
                bevy::log::warn!("skipping experiment card {}: {e:#}", p.display());
                None
            }
        })
        .collect()
}

fn read_card(path: &Path) -> Result<CardRow> {
    let text = std::fs::read_to_string(path)?;
    let c: CardFile = ron::from_str(&text)?;
    let variable = Variable::from(c.variable);
    let (lo, hi) = variable.range();
    anyhow::ensure!(
        (2..=4).contains(&c.levels.len()),
        "a card needs 2 to 4 levels"
    );
    anyhow::ensure!(
        c.levels.iter().all(|l| (lo..=hi).contains(l)),
        "levels must lie in {lo} to {hi} {}",
        variable.unit()
    );
    Ok(CardRow {
        id: c.id,
        prompt: c.prompt,
        variable,
        levels: c.levels,
        species: c.species,
        latitude_deg: c.latitude_deg,
        watch: c.watch,
    })
}

/// Species ids (file stems under `data/species`), sorted.
#[must_use]
pub fn species_ids(data: &Path) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(data.join("species"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            (p.extension().is_some_and(|x| x == "ron"))
                .then(|| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .flatten()
        })
        .collect();
    out.sort();
    out
}

/// Fills the dialog's lists.
pub fn init(ex: &mut Experiment, data: &Path) {
    ex.cards = load_cards(data);
    ex.species = species_ids(data);
    ex.weathers = weather_choices(data);
}

/// The value pot `i` gets for `v`: its level if `v` is the variable, else the fixed one.
fn value(s: &ExperimentSetup, v: Variable, i: usize) -> f64 {
    if v == s.variable {
        s.levels.get(i).copied().unwrap_or_else(|| v.fixed())
    } else {
        v.fixed()
    }
}

/// The scenario an experiment setup describes, built on the two-pot scenario (its
/// start date, CO2 and variation), with `stakes` true for a climbing species.
///
/// # Errors
/// I/O, parse or validation errors.
pub fn scenario_for(s: &ExperimentSetup, data: &Path, stakes: bool) -> Result<Scenario> {
    let text = std::fs::read_to_string(data.join(DEFAULT_SCENARIO))
        .with_context(|| format!("reading {DEFAULT_SCENARIO}"))?;
    let mut sc = Scenario::from_ron(&text).context(DEFAULT_SCENARIO)?;
    sc.world_seed = s.world_seed;
    sc.latitude_deg = s.latitude_deg;
    sc.weather = weather_source(&s.weather);
    if !matches!(sc.weather, ps_world::WeatherSource::Preset(_)) {
        sc.heat_wave = None;
    }
    sc.duration_days = s.days;
    // The two-pot scenario's loam.
    let texture = match sc.soils.first() {
        Some(SoilSpec::Pot { texture, .. }) => *texture,
        _ => anyhow::bail!("{DEFAULT_SCENARIO} should start with a pot"),
    };
    sc.soils.clear();
    sc.radiation_fraction.clear();
    sc.irrigation.clear();
    sc.sowings.clear();
    sc.supports.clear();
    sc.obstacles.clear();
    sc.events.clear();
    for i in 0..s.levels.len() {
        let id = u32::try_from(i).context("too many pots")?;
        let size = value(s, Variable::PotSize, i);
        sc.soils.push(SoilSpec::Pot {
            id,
            diameter_cm: size,
            depth_cm: size,
            texture,
            position_m: (SPACING_M * i as f64, 0.0),
            drainage: ps_world::PotDrainage::default(),
        });
        sc.radiation_fraction
            .insert(id, value(s, Variable::Light, i) / 100.0);
        sc.irrigation.insert(
            id,
            Irrigation::Schedule {
                every_days: value(s, Variable::WaterInterval, i).round().max(1.0) as u32,
                mm: value(s, Variable::WaterAmount, i),
            },
        );
        sc.sowings.push(Sowing {
            soil: id,
            species: s.species.clone(),
            seed: SeedChoice::Default,
            depth_cm: 3.0,
            offset_cm: None,
        });
        if stakes {
            sc.supports.push(SupportSpec {
                soil: id,
                offset_cm: (0.3 * size, 0.0),
                height_cm: 200.0,
                diameter_cm: 1.5,
            });
        }
    }
    sc.validate().context("checking the experiment")?;
    Ok(sc)
}

/// The world for a setup: built once, then again with stakes if the species climbs.
///
/// # Errors
/// I/O, parse, validation or world-building errors.
pub fn world_for(s: &ExperimentSetup, data: &Path) -> Result<(World, Scenario)> {
    let sc = scenario_for(s, data, false)?;
    let catalog = Catalog::load(data, &sc).context("loading species and weather")?;
    let world = World::new(sc.clone(), catalog).context("building the world")?;
    let climbs = world
        .state()
        .plants
        .iter()
        .any(|p| p.plant.params.indeterminate);
    if !climbs {
        return Ok((world, sc));
    }
    let sc = scenario_for(s, data, true)?;
    let catalog = Catalog::load(data, &sc).context("loading species and weather")?;
    let world = World::new(sc.clone(), catalog).context("building the world")?;
    Ok((world, sc))
}

/// The lab notebook entry (design plan §5.6).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename = "Notebook")]
pub struct Notebook {
    /// The card it came from, if any.
    pub card: Option<String>,
    /// The question.
    pub question: String,
    /// What differs between the pots (`light`, `water_mm` and so on).
    pub variable: String,
    /// Unit of the levels.
    pub unit: String,
    /// One level per pot, in pot (soil id) order.
    pub levels: Vec<f64>,
    /// Species id.
    pub species: String,
    /// Model version the run used.
    pub model_version: String,
    /// World seed.
    pub world_seed: u64,
    /// Start date, `YYYY-MM-DD`.
    pub start: String,
    /// Days the scenario runs.
    pub duration_days: u32,
    /// Latitude, degrees north.
    pub latitude_deg: f64,
    /// Weather preset or file.
    pub weather: String,
    /// Recorded results: (sim day, folder with daily.csv and organs.csv).
    pub results: Vec<(u32, String)>,
}

impl Notebook {
    fn new(s: &ExperimentSetup, sc: &Scenario) -> Self {
        Self {
            card: s.card.clone(),
            question: s.prompt.clone(),
            variable: s.variable.key().to_owned(),
            unit: s.variable.unit().to_owned(),
            levels: s.levels.clone(),
            species: s.species.clone(),
            model_version: sc.model_version.clone(),
            world_seed: sc.world_seed,
            start: format!(
                "{:04}-{:02}-{:02}",
                sc.start.year, sc.start.month, sc.start.day
            ),
            duration_days: sc.duration_days,
            latitude_deg: sc.latitude_deg,
            weather: s.weather.clone(),
            results: Vec::new(),
        }
    }

    fn write(&self, dir: &Path) -> Result<PathBuf> {
        let path = dir.join("notebook.ron");
        let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }
}

/// A folder name for an experiment: the card id or the variable, the species and the
/// world seed, made unique with a counter.
fn folder(root: &Path, s: &ExperimentSetup) -> PathBuf {
    let stem = format!(
        "{}-{}-seed{}",
        s.card.as_deref().unwrap_or(s.variable.key()),
        s.species,
        s.world_seed
    );
    let mut dir = root.join(&stem);
    let mut n = 2;
    while dir.exists() {
        dir = root.join(format!("{stem}-{n}"));
        n += 1;
    }
    dir
}

/// Starts an experiment: builds the world, writes the notebook and the scenario next
/// to it under `<captures>/experiments/`, and starts the sim `start_day` days in.
///
/// # Errors
/// Building the world, writing the notebook or starting the sim thread.
pub fn launch(
    s: &ExperimentSetup,
    data: &Path,
    captures: &Path,
    start_day: f64,
    speed: f64,
    paused: bool,
) -> Result<(SimSource, Running, OpenNotebook, Scenario)> {
    anyhow::ensure!(
        (2..=4).contains(&s.levels.len()),
        "an experiment has 2 to 4 pots"
    );
    let (world, sc) = world_for(s, data)?;
    let dir = folder(&captures.join("experiments"), s);
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let notebook = Notebook::new(s, &sc);
    let path = notebook.write(&dir)?;
    let scenario_text = ron::ser::to_string_pretty(&sc, ron::ser::PrettyConfig::default())?;
    std::fs::write(dir.join("scenario.ron"), scenario_text)?;
    let mut src = SimSource::start(world, &sc, start_day, speed, paused)?;
    let pots: Vec<String> = s.levels.iter().map(|l| s.variable.label(*l)).collect();
    for (i, label) in pots.iter().enumerate() {
        src.relabel(
            u32::try_from(i).unwrap_or(u32::MAX),
            format!("Pot {i}: {label}"),
        );
    }
    let running = Running {
        prompt: s.prompt.clone(),
        variable: s.variable.name().to_owned(),
        pots,
        notebook: path.display().to_string(),
        watch: s.watch.clone(),
    };
    Ok((src, running, OpenNotebook { dir, notebook }, sc))
}

/// The setup of the card `id` under `data/experiments`.
///
/// # Errors
/// No card has that id.
pub fn card_setup(data: &Path, id: &str) -> Result<ExperimentSetup> {
    let cards = load_cards(data);
    let card = cards
        .iter()
        .find(|c| c.id == id)
        .with_context(|| format!("no experiment card {id:?} in {}", data.display()))?;
    Ok(ExperimentSetup::default().from_card(card))
}

/// The open experiment's notebook, kept so results can be added to it.
#[derive(Clone, Debug, bevy::prelude::Resource)]
pub struct OpenNotebook {
    dir: PathBuf,
    notebook: Notebook,
}

/// Starts an experiment when the dialog asks, and records results when the panel asks.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn experiment(
    mut commands: Commands,
    settings: Res<RunSettings>,
    config: Res<CaptureConfig>,
    src: Option<Res<SimSource>>,
    open_notebook: Option<ResMut<OpenNotebook>>,
    mut ex: ResMut<Experiment>,
    mut selection: ResMut<Selection>,
    mut inspector: ResMut<Inspector>,
    mut charts: ResMut<Charts>,
    mut tools: ResMut<ps_ui::Tools>,
) {
    if ex.species.is_empty() {
        init(&mut ex, &settings.data);
    }
    if ex.start {
        ex.start = false;
        match launch(
            &ex.setup,
            &settings.data,
            &config.dir,
            0.0,
            settings.speed,
            false,
        ) {
            Ok((src, running, notebook, _)) => {
                commands.insert_resource(src);
                commands.insert_resource(notebook);
                ex.status = String::new();
                ex.running = Some(running);
                selection.picked = None;
                *inspector = Inspector::default();
                charts.rows = usize::MAX;
                charts.open = true;
                tools.seed_library.clear();
            }
            Err(e) => {
                ex.open = true;
                ex.status = format!("Could not start: {e:#}");
            }
        }
        return;
    }
    if !ex.record {
        return;
    }
    ex.record = false;
    let (Some(src), Some(mut nb)) = (src, open_notebook) else {
        ex.status = "No experiment is running.".into();
        return;
    };
    let state = state_on_screen(&src);
    let day = state.history.last().map_or(0, |r| r.day);
    let out = nb.dir.join(format!("day{day:03}"));
    let result = write_csv(&out, &state).and_then(|()| {
        let rel = out
            .strip_prefix(&nb.dir)
            .unwrap_or(&out)
            .display()
            .to_string();
        nb.notebook.results.retain(|(d, _)| *d != day);
        nb.notebook.results.push((day, rel));
        nb.notebook.write(&nb.dir)
    });
    ex.status = match result {
        Ok(_) => format!("Recorded day {day} in {}", out.display()),
        Err(e) => format!("Could not record: {e:#}"),
    };
}

/// Forgets the experiment when a plain new run replaces it.
#[allow(clippy::needless_pass_by_value)]
pub fn end_on_new_run(
    mut commands: Commands,
    tools: Res<ps_ui::Tools>,
    mut ex: ResMut<Experiment>,
) {
    if tools.new_run.start && ex.running.is_some() {
        ex.running = None;
        commands.remove_resource::<OpenNotebook>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> PathBuf {
        crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"))
    }

    #[test]
    fn every_card_in_data_loads() {
        let data = data();
        let n = std::fs::read_dir(data.join("experiments"))
            .unwrap_or_else(|e| unreachable!("{e}"))
            .count();
        let cards = load_cards(&data);
        assert_eq!(cards.len(), n, "a card failed to parse");
        let titles: Vec<String> =
            crate::sim_source::charts_from(&[], &std::collections::BTreeMap::new())
                .into_iter()
                .map(|c| c.title)
                .collect();
        for c in &cards {
            for w in &c.watch {
                assert!(titles.contains(w), "{}: no chart {w:?}", c.id);
            }
        }
        assert!(cards.len() >= 3);
        let species = species_ids(&data);
        for c in &cards {
            assert!(species.contains(&c.species), "{}", c.id);
            let setup = ExperimentSetup::default().from_card(c);
            scenario_for(&setup, &data, false).unwrap_or_else(|e| unreachable!("{e:#}"));
        }
    }

    #[test]
    fn only_the_variable_differs_between_pots() {
        let data = data();
        let s = ExperimentSetup {
            variable: Variable::WaterAmount,
            levels: vec![20.0, 10.0, 3.0],
            ..ExperimentSetup::default()
        };
        let sc = scenario_for(&s, &data, false).unwrap_or_else(|e| unreachable!("{e:#}"));
        assert_eq!(sc.soils.len(), 3);
        assert_eq!(sc.sowings.len(), 3);
        let mm: Vec<f64> = sc
            .irrigation
            .values()
            .map(|i| match i {
                Irrigation::Schedule { mm, every_days: 2 } => *mm,
                other => unreachable!("{other:?}"),
            })
            .collect();
        assert_eq!(mm, [20.0, 10.0, 3.0]);
        assert!(
            sc.radiation_fraction
                .values()
                .all(|f| (*f - 1.0).abs() < 1e-12)
        );
        assert!(sc.supports.is_empty());
    }

    #[test]
    fn a_climbing_species_gets_stakes() {
        let data = data();
        let s = ExperimentSetup {
            species: "pole_bean".into(),
            variable: Variable::Light,
            levels: vec![100.0, 50.0],
            days: 30,
            ..ExperimentSetup::default()
        };
        let (world, sc) = world_for(&s, &data).unwrap_or_else(|e| unreachable!("{e:#}"));
        assert_eq!(sc.supports.len(), 2);
        assert_eq!(world.state().plants.len(), 2);
        let bush = ExperimentSetup {
            days: 30,
            ..ExperimentSetup::default()
        };
        let (_, sc) = world_for(&bush, &data).unwrap_or_else(|e| unreachable!("{e:#}"));
        assert!(sc.supports.is_empty());
    }

    #[test]
    fn the_notebook_pins_model_seed_and_levels() {
        let data = data();
        let s = ExperimentSetup::default();
        let sc = scenario_for(&s, &data, false).unwrap_or_else(|e| unreachable!("{e:#}"));
        let nb = Notebook::new(&s, &sc);
        let text = ron::ser::to_string_pretty(&nb, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|e| unreachable!("{e}"));
        assert!(text.contains(&format!("model_version: \"{}\"", sc.model_version)));
        assert!(text.contains("world_seed: 42"));
        assert!(text.contains("variable: \"light\""));
        let dir = std::env::temp_dir().join(format!("ps-ex-{}", std::process::id()));
        let a = folder(&dir, &s);
        std::fs::create_dir_all(&a).unwrap_or_else(|e| unreachable!("{e}"));
        let b = folder(&dir, &s);
        assert_ne!(a, b);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
