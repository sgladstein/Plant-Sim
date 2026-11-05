//! The world (design plan §3.2, §3.14, §6.2, §6.3; WP9): weather, soils and plants
//! stepped on the §3.2 schedule, an append-only action log, snapshots, and the
//! full-state hash.
//!
//! The source of truth is the scenario plus the action log; a snapshot is a cache that
//! resumes to the same bits. Model card: docs/models/world-and-schedule.md.

pub mod hash;
pub mod scenario;

use std::collections::BTreeMap;
use std::path::Path;

use ps_arch::{ArchError, DayReport, HourCtx, Plant};
use ps_soil::{SoilError, SoilGrid, SoilHour, WaterLedger};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, KgWater, Meters, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, HourWeather, Preset, WeatherSeries};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub use hash::hash_of;
pub use scenario::{
    Action, ActionKind, Irrigation, Scenario, SeedChoice, SoilSpec, Sowing, Start, SupportSpec,
    WeatherSource,
};

/// Model version: bumped on any change to an equation, a default parameter or the
/// step order (CLAUDE.md). Saves record it; replays from another version warn.
pub const MODEL_VERSION: &str = "0.7.0";

/// Most fertilizer one action may add, g N per m² (100 g m⁻² is 1000 kg N ha⁻¹, several
/// times a heavy season's dose; WP24 review).
pub const MAX_FERTILIZER_G_PER_M2: f64 = 100.0;

/// Air pressure used for gas exchange, kPa (sea level; no site elevation in M0).
const PATM_KPA: f64 = 101.325;
/// Clock hour of scheduled irrigation.
const IRRIGATION_HOUR: u64 = 6;
/// Initial soil temperature, °C, before the soil's air history fills.
const SOIL_T_INITIAL: f64 = 15.0;
/// Soil grid cell size in pots, m (§3.6).
const POT_CELL: f64 = 0.02;
/// Soil cell edge in plots, m (§3.6).
const PLOT_CELL: f64 = 0.05;
/// Canopy extinction for radiation reaching the soil surface.
const SOIL_EXTINCTION: f64 = 0.5;

/// Errors from the world.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum WorldError {
    /// The scenario is unusable.
    #[error("scenario: {0}")]
    Scenario(String),
    /// A species is missing or unusable.
    #[error("species: {0}")]
    Species(String),
    /// The weather could not be built.
    #[error("weather: {0}")]
    Weather(String),
    /// A soil could not be built.
    #[error(transparent)]
    Soil(#[from] SoilError),
    /// A plant rejected a step.
    #[error(transparent)]
    Arch(#[from] ArchError),
    /// An action cannot be applied.
    #[error("action at tick {tick}: {why}")]
    Action {
        /// The action's tick.
        tick: u64,
        /// Why.
        why: String,
    },
    /// A snapshot could not be written or read.
    #[error("snapshot: {0}")]
    Snapshot(String),
    /// A file could not be read.
    #[error("{path}: {why}")]
    Io {
        /// The file.
        path: String,
        /// The error.
        why: String,
    },
}

/// Everything a world reads besides the scenario: species files and the weather file's
/// text. Kept in snapshots so they are self-contained.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    /// Species by id.
    pub species: BTreeMap<String, Species>,
    /// Text of the scenario's weather file, if it names one.
    pub weather_text: Option<String>,
}

impl Catalog {
    /// Loads what `scenario` needs from a data directory (`species/<id>.ron` and the
    /// weather file).
    ///
    /// # Errors
    /// [`WorldError::Io`] or [`WorldError::Species`].
    pub fn load(data_dir: &Path, scenario: &Scenario) -> Result<Self, WorldError> {
        let read = |p: &Path| {
            std::fs::read_to_string(p).map_err(|e| WorldError::Io {
                path: p.display().to_string(),
                why: e.to_string(),
            })
        };
        let mut species = BTreeMap::new();
        for s in &scenario.sowings {
            if species.contains_key(&s.species) {
                continue;
            }
            let path = data_dir.join("species").join(format!("{}.ron", s.species));
            let sp = Species::from_ron(&read(&path)?)
                .map_err(|e| WorldError::Species(format!("{}: {e}", s.species)))?;
            species.insert(s.species.clone(), sp);
        }
        let weather_text = match &scenario.weather {
            WeatherSource::Preset(_) => None,
            WeatherSource::File(f) | WeatherSource::DailyFile(f) => Some(read(&data_dir.join(f))?),
        };
        Ok(Self {
            species,
            weather_text,
        })
    }
}

/// One soil and its management.
fn one() -> f64 {
    1.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoilEntry {
    /// Scenario id.
    pub id: u32,
    /// The grid.
    pub grid: SoilGrid,
    /// Share of radiation reaching this soil and its plants.
    pub radiation_fraction: f64,
    /// Share of rain reaching this soil (0 under a rain shelter).
    #[serde(default = "one")]
    pub rain_fraction: f64,
    /// Whether this soil's plants share one light grid (a plot; decision 0019).
    #[serde(default)]
    pub shared_light: bool,
    /// Automatic watering.
    pub irrigation: Irrigation,
    /// Water applied by actions, waiting for the next hourly step, mm.
    pub pending_mm: f64,
    /// Water fluxes since the start.
    pub ledger: WaterLedger,
}

/// One plant and where it grows.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlantEntry {
    /// Plant id (index in the world's plant list; its random-stream entity).
    pub id: u32,
    /// Index of its soil in the world's soil list.
    pub soil: usize,
    /// Species id.
    pub species: String,
    /// The plant.
    pub plant: Plant,
}

/// One daily summary row (§3.2 daily step 7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DayRow {
    /// Sim day (0 = the scenario's first day).
    pub day: u32,
    /// Plant id.
    pub plant: u32,
    /// The plant's report.
    pub report: DayReport,
    /// The day's weather and soil as this plant met them.
    pub env: DayEnv,
}

/// The day's weather and soil water around one plant, for the daily table.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DayEnv {
    /// Lowest hourly air temperature, °C.
    pub t_min_c: f64,
    /// Highest hourly air temperature, °C.
    pub t_max_c: f64,
    /// Global radiation reaching the plant's soil (after shade cloth), MJ m⁻².
    pub radiation_mj: f64,
    /// Rain, mm.
    pub rain_mm: f64,
    /// Root-weighted fraction of transpirable soil water at the end of the day.
    pub ftsw: f64,
    /// Water in the plant's soil, kg.
    pub soil_water_kg: f64,
    /// Mineral (nitrate plus ammonium) nitrogen in the plant's soil, g N (WP24).
    #[serde(default)]
    pub soil_mineral_n_g: f64,
}

/// The world's complete state: what snapshots store and the hash covers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    /// Model version that produced the state.
    pub model_version: String,
    /// The starting scenario.
    pub scenario: Scenario,
    /// Species and weather inputs.
    pub catalog: Catalog,
    /// The next hour to step.
    pub tick: u64,
    /// Soils in scenario order.
    pub soils: Vec<SoilEntry>,
    /// Plants by id.
    pub plants: Vec<PlantEntry>,
    /// The action log, sorted by tick (stable for equal ticks).
    pub log: Vec<Action>,
    /// Actions in the log already applied.
    pub applied: usize,
    /// Scenario events already applied.
    pub scripted: usize,
    /// Daily rows, in day then plant order.
    pub history: Vec<DayRow>,
    /// Full-state hash after each daily step.
    pub hashes: Vec<u64>,
    /// Log entries already covered by a daily hash.
    pub hashed_log: usize,
    /// History rows already covered by a daily hash.
    pub hashed_rows: usize,
}

impl WorldState {
    /// Full-state hash: every field except actions scheduled for later ticks, which
    /// are intentions rather than state (a replay knows its whole log from the start).
    #[must_use]
    pub fn hash(&self) -> u64 {
        hash_of(&(
            &self.model_version,
            &self.scenario,
            &self.catalog,
            self.tick,
            &self.soils,
            &self.plants,
            &self.log[..self.applied],
            &self.history,
            &self.hashes,
        ))
    }

    /// The daily hash: the previous day's hash (or, on day 0, the model version,
    /// scenario and catalog) chained with the current soils and plants and what was
    /// added since the previous daily hash (applied actions and history rows). The
    /// chain covers the whole history at a cost that does not grow with the day.
    #[must_use]
    pub fn day_hash(&self) -> u64 {
        let prev = self
            .hashes
            .last()
            .copied()
            .unwrap_or_else(|| hash_of(&(&self.model_version, &self.scenario, &self.catalog)));
        hash_of(&(
            prev,
            self.tick,
            &self.soils,
            &self.plants,
            &self.log[self.hashed_log..self.applied],
            &self.history[self.hashed_rows..],
        ))
    }
}

/// A daily hash being computed on a snapshot (see [`World::run_until`]).
type PendingHash = Option<std::sync::mpsc::Receiver<u64>>;

/// A world: state plus the weather series rebuilt from it.
#[derive(Clone, Debug)]
pub struct World {
    state: WorldState,
    weather: WeatherSeries,
}

fn weather_for(s: &Scenario, c: &Catalog) -> Result<WeatherSeries, WorldError> {
    let start = DateHour::new(s.start.year, s.start.month, s.start.day, s.start.hour)
        .ok_or_else(|| WorldError::Scenario("invalid start date".into()))?;
    let lat = Degrees(s.latitude_deg);
    // Two extra days: a start after midnight reaches into one more calendar day, and
    // the last daily step needs the next day's sun path.
    let days = s.duration_days + 2;
    match &s.weather {
        WeatherSource::Preset(name) => {
            let p = Preset::from_name(name)
                .ok_or_else(|| WorldError::Weather(format!("unknown preset {name:?}")))?;
            let mut params = p.params();
            if s.heat_wave.is_some() {
                params.heat_wave = s.heat_wave;
            }
            let daily =
                ps_weather::presets::generate_daily(&params, lat, start, days + 1, s.world_seed);
            Ok(WeatherSeries::from_daily(lat, start, daily))
        }
        WeatherSource::File(f) | WeatherSource::DailyFile(f) => {
            let text = c
                .weather_text
                .as_deref()
                .ok_or_else(|| WorldError::Weather(format!("{f} not in the catalog")))?;
            let r = if matches!(s.weather, WeatherSource::File(_)) {
                ps_weather::csv::read_hourly(text, lat, start)
            } else {
                ps_weather::csv::read_daily(text, lat, start)
            };
            r.map_err(|e| WorldError::Weather(format!("{f}: {e}")))
        }
    }
}

fn soil_grid(spec: &SoilSpec) -> Result<SoilGrid, WorldError> {
    match spec {
        SoilSpec::Pot {
            diameter_cm,
            depth_cm,
            texture,
            ..
        } => Ok(SoilGrid::new_pot(
            Meters(diameter_cm * 0.01),
            Meters(depth_cm * 0.01),
            Meters(POT_CELL),
            *texture,
            Celsius(SOIL_T_INITIAL),
        )?),
        SoilSpec::Plot {
            width_cm,
            length_cm,
            depth_cm,
            texture,
            ..
        } => {
            let cells = |cm: f64| {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let n = libm::round(cm * 0.01 / PLOT_CELL).max(1.0) as usize;
                n
            };
            Ok(SoilGrid::new_box(
                cells(*width_cm),
                cells(*length_cm),
                cells(*depth_cm),
                Meters(PLOT_CELL),
                *texture,
                Celsius(SOIL_T_INITIAL),
            )?)
        }
    }
}

/// Weather as the plants and soil of one container see it: radiation scaled by the
/// container's radiation fraction.
fn shaded(h: &HourWeather, f: f64) -> HourWeather {
    HourWeather {
        global: WPerM2(h.global.0 * f),
        direct: WPerM2(h.direct.0 * f),
        diffuse: WPerM2(h.diffuse.0 * f),
        ..*h
    }
}

impl World {
    /// Builds a world at tick 0 and sows the scenario's seeds.
    ///
    /// # Errors
    /// Any [`WorldError`] for an inconsistent scenario or catalog.
    pub fn new(scenario: Scenario, catalog: Catalog) -> Result<Self, WorldError> {
        scenario.validate()?;
        if scenario.model_version != MODEL_VERSION {
            tracing::warn!(
                scenario = %scenario.model_version,
                build = MODEL_VERSION,
                "the scenario was written for another model version; results may differ"
            );
        }
        let weather = weather_for(&scenario, &catalog)?;
        let mut soils = Vec::new();
        for spec in &scenario.soils {
            let id = spec.id();
            let mut grid = soil_grid(spec)?;
            grid.set_nitrogen(&scenario.nitrogen.get(&id).copied().unwrap_or_default());
            soils.push(SoilEntry {
                id,
                grid,
                shared_light: spec.shared_light(),
                radiation_fraction: scenario.radiation_fraction.get(&id).copied().unwrap_or(1.0),
                rain_fraction: scenario.rain_fraction.get(&id).copied().unwrap_or(1.0),
                irrigation: scenario
                    .irrigation
                    .get(&id)
                    .cloned()
                    .unwrap_or(Irrigation::None),
                pending_mm: 0.0,
                ledger: WaterLedger::default(),
            });
        }
        let sowings = scenario.sowings.clone();
        let events = scenario.events.clone();
        let mut w = Self {
            state: WorldState {
                model_version: MODEL_VERSION.to_string(),
                scenario,
                catalog,
                tick: 0,
                soils,
                plants: Vec::new(),
                log: Vec::new(),
                applied: 0,
                scripted: 0,
                history: Vec::new(),
                hashes: Vec::new(),
                hashed_log: 0,
                hashed_rows: 0,
            },
            weather,
        };
        for s in &sowings {
            w.sow(s)?;
        }
        // The static part of the action check; a future prune can still fail when due.
        for e in &events {
            w.check(e.tick, &e.kind, false)?;
        }
        Ok(w)
    }

    /// Resumes from a saved state.
    ///
    /// # Errors
    /// [`WorldError::Weather`] if the weather cannot be rebuilt.
    pub fn from_state(state: WorldState) -> Result<Self, WorldError> {
        if state.model_version != MODEL_VERSION {
            tracing::warn!(
                saved = %state.model_version,
                build = MODEL_VERSION,
                "the save comes from another model version; it will not continue bit for bit"
            );
        }
        let weather = weather_for(&state.scenario, &state.catalog)?;
        Ok(Self { state, weather })
    }

    /// Replays a recorded action log from the scenario up to `tick`.
    ///
    /// # Errors
    /// Any [`WorldError`] from building or stepping.
    pub fn replay(
        scenario: Scenario,
        catalog: Catalog,
        log: &[Action],
        tick: u64,
    ) -> Result<Self, WorldError> {
        let mut w = Self::new(scenario, catalog)?;
        for a in log {
            w.schedule(a.clone())?;
        }
        w.run_until(tick)?;
        Ok(w)
    }

    /// The complete state, read-only.
    #[must_use]
    pub fn state(&self) -> &WorldState {
        &self.state
    }

    /// The next hour to step.
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.state.tick
    }

    /// The weather series the world steps through (rebuilt from the scenario).
    #[must_use]
    pub fn weather(&self) -> &WeatherSeries {
        &self.weather
    }

    /// The weather of the hour the world steps next, as open sky (before any soil's
    /// radiation fraction), or `None` past the series.
    #[must_use]
    pub fn weather_now(&self) -> Option<&HourWeather> {
        self.weather.at(Hours(self.state.tick))
    }

    /// The last tick the scenario covers.
    #[must_use]
    pub fn end_tick(&self) -> u64 {
        u64::from(self.state.scenario.duration_days) * 24
    }

    /// Full-state hash now.
    #[must_use]
    pub fn state_hash(&self) -> u64 {
        self.state.hash()
    }

    /// Hash of one plant's own state (for the "extra plant" determinism check).
    #[must_use]
    pub fn plant_hash(&self, plant: u32) -> Option<u64> {
        self.state
            .plants
            .get(plant as usize)
            .map(|p| hash_of(&p.plant))
    }

    /// The snapshot: the whole state as RON text.
    ///
    /// # Errors
    /// [`WorldError::Snapshot`] if serialization fails.
    pub fn snapshot_ron(&self) -> Result<String, WorldError> {
        ron::ser::to_string(&self.state).map_err(|e| WorldError::Snapshot(e.to_string()))
    }

    /// Resumes from snapshot text.
    ///
    /// # Errors
    /// [`WorldError::Snapshot`] for a malformed snapshot.
    pub fn from_snapshot_ron(text: &str) -> Result<Self, WorldError> {
        let state: WorldState =
            ron::from_str(text).map_err(|e| WorldError::Snapshot(e.to_string()))?;
        Self::from_state(state)
    }

    /// Applies an action at the current tick (appends it to the log).
    ///
    /// # Errors
    /// [`WorldError::Action`] if the action is invalid.
    pub fn apply(&mut self, kind: ActionKind) -> Result<(), WorldError> {
        let tick = self.state.tick;
        self.check(tick, &kind, true)?;
        self.schedule(Action { tick, kind })
    }

    /// Checks an action against the world now; `now` adds the checks that need the
    /// current state (a pruned organ must exist and be alive). Rejected actions never
    /// reach the log.
    fn check(&self, tick: u64, kind: &ActionKind, now: bool) -> Result<(), WorldError> {
        let err = |why: String| Err(WorldError::Action { tick, why });
        let soil_known = |soil: u32| self.soil_index(soil).is_some();
        match kind {
            ActionKind::Sow(s) => {
                if !soil_known(s.soil) {
                    return err(format!("unknown soil {}", s.soil));
                }
            }
            ActionKind::Water { soil, mm } => {
                if !soil_known(*soil) {
                    return err(format!("unknown soil {soil}"));
                }
                if !(mm.is_finite() && *mm >= 0.0) {
                    return err(format!("water {mm} mm"));
                }
            }
            ActionKind::SetRadiationFraction { soil, fraction } => {
                if !soil_known(*soil) {
                    return err(format!("unknown soil {soil}"));
                }
                if !(0.0..=1.0).contains(fraction) {
                    return err(format!("radiation fraction {fraction}"));
                }
            }
            ActionKind::SetIrrigation { soil, .. } => {
                if !soil_known(*soil) {
                    return err(format!("unknown soil {soil}"));
                }
            }
            ActionKind::Fertilize { soil, g_per_m2, .. } => {
                if !soil_known(*soil) {
                    return err(format!("unknown soil {soil}"));
                }
                if !(0.0..=MAX_FERTILIZER_G_PER_M2).contains(g_per_m2) {
                    return err(format!(
                        "fertilizer {g_per_m2} g N per m2, outside 0 to {MAX_FERTILIZER_G_PER_M2}"
                    ));
                }
            }
            ActionKind::Prune { plant, organ } => {
                if now {
                    let Some(p) = self.state.plants.get(*plant as usize) else {
                        return err(format!("unknown plant {plant}"));
                    };
                    if let Err(e) = p.plant.check_prune(ps_graph::OrganId(*organ)) {
                        return err(e.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Adds an action to the log for a current or future tick.
    ///
    /// # Errors
    /// [`WorldError::Action`] for a past tick.
    pub fn schedule(&mut self, a: Action) -> Result<(), WorldError> {
        if a.tick < self.state.tick {
            return Err(WorldError::Action {
                tick: a.tick,
                why: format!("tick is before the world's tick {}", self.state.tick),
            });
        }
        self.check(a.tick, &a.kind, false)?;
        // Keep the log sorted by tick, stable for equal ticks.
        let at = self.state.log.partition_point(|x| x.tick <= a.tick);
        self.state.log.insert(at.max(self.state.applied), a);
        Ok(())
    }

    /// Steps until the world's tick reaches `tick`.
    ///
    /// # Errors
    /// Any [`WorldError`] from a step.
    pub fn run_until(&mut self, tick: u64) -> Result<(), WorldError> {
        // The whole run executes on a rayon worker, so each parallel section starts on
        // this thread instead of waiting for a sleeping worker to pick it up.
        // Each day's hash runs on a snapshot while the next day's hours step (it is
        // waited for at the next daily step and before returning), so the hash, which
        // is one sequential chain, leaves the critical path. Same value either way.
        rayon::scope(|s| {
            let mut pending: PendingHash = None;
            let mut result = Ok(());
            while self.state.tick < tick {
                result = self.step_hour_on_worker(Some((s, &mut pending)));
                if result.is_err() {
                    break;
                }
            }
            self.finish_hash(&mut pending);
            result
        })
    }

    /// Steps `days` whole days.
    ///
    /// # Errors
    /// Any [`WorldError`] from a step.
    pub fn run_days(&mut self, days: u32) -> Result<(), WorldError> {
        self.run_until(self.state.tick + u64::from(days) * 24)
    }

    /// Runs to the scenario's end.
    ///
    /// # Errors
    /// Any [`WorldError`] from a step.
    pub fn run_to_end(&mut self) -> Result<(), WorldError> {
        self.run_until(self.end_tick())
    }

    fn soil_index(&self, id: u32) -> Option<usize> {
        self.state.soils.iter().position(|s| s.id == id)
    }

    fn sow(&mut self, s: &Sowing) -> Result<(), WorldError> {
        let tick = self.state.tick;
        let err = |why: String| WorldError::Action { tick, why };
        let soil = self
            .soil_index(s.soil)
            .ok_or_else(|| err(format!("unknown soil {}", s.soil)))?;
        let species = self
            .state
            .catalog
            .species
            .get(&s.species)
            .ok_or_else(|| WorldError::Species(format!("{} not in the catalog", s.species)))?;
        let id =
            u32::try_from(self.state.plants.len()).map_err(|_| err("too many plants".into()))?;
        let genotype = match s.seed {
            SeedChoice::Default => species.default_genotype(),
            SeedChoice::Sample => {
                species.sample_genotype(self.state.scenario.world_seed, u64::from(id))
            }
        };
        let traits = species
            .phenotype(&genotype)
            .traits()
            .map_err(|e| WorldError::Species(e.to_string()))?;
        let grid = &self.state.soils[soil].grid;
        let (nx, ny, _) = grid.dims();
        let c = grid.cell_size().0;
        let (dx, dy) = s.offset_cm.unwrap_or((0.0, 0.0));
        #[allow(clippy::cast_precision_loss)]
        let at = [
            nx as f64 * c * 0.5 + dx * 0.01,
            ny as f64 * c * 0.5 + dy * 0.01,
            -s.depth_cm * 0.01,
        ];
        if ps_arch::geometry::soil_cell(grid, at).is_none() {
            return Err(err(format!(
                "sowing point {at:?} is outside soil {}",
                s.soil
            )));
        }
        let mut plant = Plant::sow(
            species,
            &traits,
            self.state.scenario.world_seed,
            u64::from(id),
            at,
            Hours(tick),
        )?;
        #[allow(clippy::cast_precision_loss)]
        let centre = [nx as f64 * c * 0.5, ny as f64 * c * 0.5];
        plant.supports = self
            .state
            .scenario
            .supports
            .iter()
            .filter(|p| p.soil == s.soil)
            .map(|p| ps_arch::Support {
                base: [
                    centre[0] + p.offset_cm.0 * 0.01,
                    centre[1] + p.offset_cm.1 * 0.01,
                    0.0,
                ],
                height: p.height_cm * 0.01,
                diameter: p.diameter_cm * 0.01,
            })
            .collect();
        self.state.plants.push(PlantEntry {
            id,
            soil,
            species: s.species.clone(),
            plant,
        });
        Ok(())
    }

    fn apply_due(&mut self) -> Result<(), WorldError> {
        let now = self.state.tick;
        while let Some(a) = self.state.scenario.events.get(self.state.scripted).cloned() {
            if a.tick > now {
                break;
            }
            self.state.scripted += 1;
            self.apply_kind(a.tick, a.kind)?;
        }
        while let Some(a) = self.state.log.get(self.state.applied).cloned() {
            if a.tick > now {
                break;
            }
            self.state.applied += 1;
            self.apply_kind(a.tick, a.kind)?;
        }
        Ok(())
    }

    fn apply_kind(&mut self, tick: u64, kind: ActionKind) -> Result<(), WorldError> {
        let err = |why: String| WorldError::Action { tick, why };
        match kind {
            ActionKind::Sow(s) => self.sow(&s)?,
            ActionKind::Water { soil, mm } => {
                let i = self
                    .soil_index(soil)
                    .ok_or_else(|| err(format!("unknown soil {soil}")))?;
                if !(mm.is_finite() && mm >= 0.0) {
                    return Err(err(format!("water {mm} mm")));
                }
                self.state.soils[i].pending_mm += mm;
            }
            ActionKind::SetRadiationFraction { soil, fraction } => {
                let i = self
                    .soil_index(soil)
                    .ok_or_else(|| err(format!("unknown soil {soil}")))?;
                if !(0.0..=1.0).contains(&fraction) {
                    return Err(err(format!("radiation fraction {fraction}")));
                }
                self.state.soils[i].radiation_fraction = fraction;
            }
            ActionKind::SetIrrigation { soil, irrigation } => {
                let i = self
                    .soil_index(soil)
                    .ok_or_else(|| err(format!("unknown soil {soil}")))?;
                self.state.soils[i].irrigation = irrigation;
            }
            ActionKind::Fertilize {
                soil,
                form,
                g_per_m2,
            } => {
                let i = self
                    .soil_index(soil)
                    .ok_or_else(|| err(format!("unknown soil {soil}")))?;
                if !(0.0..=MAX_FERTILIZER_G_PER_M2).contains(&g_per_m2) {
                    return Err(err(format!(
                        "fertilizer {g_per_m2} g N per m2, outside 0 to {MAX_FERTILIZER_G_PER_M2}"
                    )));
                }
                self.state.soils[i].grid.fertilize(form, g_per_m2);
            }
            ActionKind::Prune { plant, organ } => {
                let p = self
                    .state
                    .plants
                    .get_mut(plant as usize)
                    .ok_or_else(|| err(format!("unknown plant {plant}")))?;
                p.plant
                    .prune(ps_graph::OrganId(organ), Hours(tick))
                    .map_err(|e| err(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Water to add this hour from the schedule, mm.
    fn irrigation_mm(s: &SoilEntry, clock_hour: u64, day: u64) -> f64 {
        let due = |every: u32| {
            clock_hour == IRRIGATION_HOUR && day.is_multiple_of(u64::from(every.max(1)))
        };
        match s.irrigation {
            Irrigation::None => 0.0,
            Irrigation::Schedule { every_days, mm } => {
                if due(every_days) {
                    mm
                } else {
                    0.0
                }
            }
            Irrigation::ToFieldCapacity { every_days } => {
                if due(every_days) {
                    let g = &s.grid;
                    let fc = g.texture().hydraulics().field_capacity * g.cell_volume() * 1000.0;
                    let deficit: f64 = (0..g.len())
                        .filter(|i| g.is_active(*i))
                        .map(|i| (fc - g.water(i).0).max(0.0))
                        .sum();
                    // kg over the surface area in m² is mm.
                    deficit / g.surface_area().max(1e-12)
                } else {
                    0.0
                }
            }
        }
    }

    /// One hourly step (§3.2), then the daily step after the day's last hour.
    ///
    /// # Errors
    /// Any [`WorldError`] from a soil or a plant.
    pub fn step_hour(&mut self) -> Result<(), WorldError> {
        rayon::scope(|_| self.step_hour_on_worker(None))
    }

    /// [`World::step_hour`], run on the calling thread (a rayon worker). With `defer`,
    /// the daily hash is computed by a task on that scope (see [`World::run_until`]).
    fn step_hour_on_worker(
        &mut self,
        defer: Option<(&rayon::Scope<'_>, &mut PendingHash)>,
    ) -> Result<(), WorldError> {
        self.apply_due()?;
        let tick = self.state.tick;
        let start_hour = u64::from(self.state.scenario.start.hour);
        let clock = start_hour + tick;
        let (day, hour) = (clock / 24, clock % 24);
        // 1, 2. Weather and sun.
        let wx = *self
            .weather
            .at(Hours(tick))
            .ok_or_else(|| WorldError::Weather(format!("series ends before tick {tick}")))?;
        let co2 = Ppm(self.state.scenario.co2_ppm);
        let hour_index = usize::try_from(hour).unwrap_or(0);
        let n_soils = self.state.soils.len();
        for si in 0..n_soils {
            let f = self.state.soils[si].radiation_fraction;
            let local = shaded(&wx, f);
            let ctx = HourCtx {
                weather: &local,
                hour: hour_index,
                tick: Hours(tick),
                co2,
                patm: KPa(PATM_KPA),
            };
            // Each plant's hourly work depends only on itself and the shared soil, so
            // the plants of a soil run in parallel; collect keeps plant order, so the
            // result does not depend on the thread count.
            let WorldState { plants, soils, .. } = &mut self.state;
            let mut members: Vec<&mut PlantEntry> =
                plants.iter_mut().filter(|e| e.soil == si).collect();
            // 3, 5, 6. Light, water status, gas exchange: each plant's demand, from
            // the soil before this hour's water moves.
            let soil = &soils[si].grid;
            members
                .iter_mut()
                .map(|e| e.plant.update_water_status(soil))
                .collect::<Result<Vec<()>, _>>()?;
            // Plants with the same leaf parameters and stress bits share one class
            // table (the hour's weather is the soil's): the table is a pure function of
            // those, so sharing it gives the same bits as building one per plant.
            let mut keys: Vec<([u64; 14], u64)> = Vec::new();
            let mut owners: Vec<usize> = Vec::new();
            let slot: Vec<usize> = members
                .iter()
                .enumerate()
                .map(|(m, e)| {
                    let key = e.plant.class_table_key();
                    keys.iter().position(|k| *k == key).unwrap_or_else(|| {
                        keys.push(key);
                        owners.push(m);
                        keys.len() - 1
                    })
                })
                .collect();
            let tables: Vec<_> = owners
                .iter()
                .map(|&m| members[m].plant.class_table(&ctx))
                .collect();
            let (demand, areas): (Vec<KgWater>, Vec<f64>) = members
                .par_iter_mut()
                .zip(&slot)
                .map(|(e, &k)| {
                    let d = e.plant.water_demand_with(&ctx, &tables[k]);
                    (d, e.plant.leaf_area())
                })
                .unzip();
            // 4. Soil water: rain, irrigation and action water; radiation under the canopy.
            let leaf_area: f64 = areas.iter().copied().sum();
            let irr = Self::irrigation_mm(&soils[si], hour, day);
            let entry = &mut soils[si];
            let lai = leaf_area / entry.grid.surface_area().max(1e-12);
            let water_in = wx.rain.0 * entry.rain_fraction + irr + entry.pending_mm;
            entry.pending_mm = 0.0;
            let led = entry.grid.step_hour(&SoilHour {
                water_in: Millimeters(water_in),
                surface_radiation: WPerM2(local.global.0 * libm::exp(-SOIL_EXTINCTION * lai)),
                t_air: wx.t_air,
            });
            entry.ledger.accumulate(&led);
            // 7. Joint uptake for every plant in this soil.
            let requests: Vec<ps_soil::PlantUptake<'_>> = members
                .iter()
                .zip(&demand)
                .map(|(e, d)| e.plant.uptake_request(*d))
                .collect();
            let joint = soils[si].grid.take_up(&requests)?;
            soils[si].ledger.accumulate(&joint.ledger);
            // 8 to 12. Carbon.
            let soil = &soils[si].grid;
            members
                .par_iter_mut()
                .enumerate()
                .map(|(k, e)| e.plant.grow_hour(&ctx, soil, joint.plants.get(k)))
                .collect::<Result<Vec<_>, _>>()?;
        }
        self.state.tick += 1;
        if hour == 23 {
            self.step_day(defer)?;
        }
        Ok(())
    }

    /// The sun direction for each clock hour of the day starting at `tick` (the
    /// next sim-midnight), or `None` below the horizon.
    fn sun_path(&self, tick: u64) -> [Option<[f64; 3]>; 24] {
        let mut out = [None; 24];
        for (k, s) in out.iter_mut().enumerate() {
            if let Some(h) = self.weather.at(Hours(tick + k as u64)) {
                *s = h.sun.is_up().then(|| h.sun.direction());
            }
        }
        out
    }

    /// The daily step (§3.2) for every plant, then the day's rows and hash.
    fn step_day(
        &mut self,
        mut defer: Option<(&rayon::Scope<'_>, &mut PendingHash)>,
    ) -> Result<(), WorldError> {
        if let Some((_, pending)) = defer.as_mut() {
            self.finish_hash(pending);
        }
        let now = self.state.tick;
        let day = u32::try_from(self.state.hashes.len()).unwrap_or(u32::MAX);
        let path = self.sun_path(now);
        let (mut t_min, mut t_max, mut global, mut rain) = (f64::MAX, f64::MIN, 0.0, 0.0);
        for t in now.saturating_sub(24)..now {
            if let Some(h) = self.weather.at(Hours(t)) {
                t_min = t_min.min(h.t_air.0);
                t_max = t_max.max(h.t_air.0);
                global += h.global.0 * 3600.0 * 1e-6;
                rain += h.rain.0;
            }
        }
        self.daily_nitrogen(Hours(now))?;
        let reports = self.daily_plant_steps(&path, Hours(now))?;
        for (p, report) in reports.into_iter().enumerate() {
            let si = self.state.plants[p].soil;
            let entry = &self.state.soils[si];
            let soil = &entry.grid;
            let roots = &self.state.plants[p].plant.roots;
            let ftsw = if roots.cells().is_empty() {
                // No roots yet: the mean over the soil's cells.
                let active: Vec<usize> = (0..soil.len()).filter(|&i| soil.is_active(i)).collect();
                #[allow(clippy::cast_precision_loss)] // cell counts are far below 2^52
                let n = active.len().max(1) as f64;
                active.iter().map(|&i| soil.cell_ftsw(i)).sum::<f64>() / n
            } else {
                soil.ftsw(roots)?
            };
            let env = DayEnv {
                t_min_c: t_min,
                t_max_c: t_max,
                radiation_mj: global * entry.radiation_fraction,
                rain_mm: rain,
                ftsw,
                soil_water_kg: soil.total_water().0,
                soil_mineral_n_g: soil.mineral_nitrogen().0,
            };
            let plant = self.state.plants[p].id;
            self.state.history.push(DayRow {
                day,
                plant,
                report,
                env,
            });
        }
        // 7. Output: the chained daily hash.
        if let Some((scope, pending)) = defer {
            // The same tuple `WorldState::day_hash` hashes, on owned copies.
            let st = &self.state;
            let prev = st
                .hashes
                .last()
                .copied()
                .unwrap_or_else(|| hash_of(&(&st.model_version, &st.scenario, &st.catalog)));
            let tick = st.tick;
            let (soils, plants) = rayon::join(
                || st.soils.clone(),
                || st.plants.par_iter().cloned().collect::<Vec<_>>(),
            );
            let log = st.log[st.hashed_log..st.applied].to_vec();
            let rows = st.history[st.hashed_rows..].to_vec();
            let (tx, rx) = std::sync::mpsc::channel();
            scope.spawn(move |_| {
                let h = hash_of(&(prev, tick, &soils, &plants, &log[..], &rows[..]));
                // The receiver lives until this hash is collected.
                let _ = tx.send(h);
            });
            *pending = Some(rx);
        } else {
            let h = self.state.day_hash();
            self.state.hashes.push(h);
        }
        self.state.hashed_log = self.state.applied;
        self.state.hashed_rows = self.state.history.len();
        Ok(())
    }

    /// Waits for a deferred daily hash and appends it.
    fn finish_hash(&mut self, pending: &mut PendingHash) {
        if let Some(rx) = pending.take() {
            // Wait by running other pool work (possibly the hash task itself), so a
            // pool whose every worker is waiting here still makes progress.
            loop {
                match rx.try_recv() {
                    Ok(h) => {
                        self.state.hashes.push(h);
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        if rayon::yield_now() != Some(rayon::Yield::Executed) {
                            std::thread::yield_now();
                        }
                    }
                    // The task only drops its sender after sending, unless it
                    // panicked, and the scope then re-raises that panic.
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        unreachable!("deferred hash task panicked")
                    }
                }
            }
        }
    }

    /// Daily steps 1 to 6 for every plant, in parallel (each plant depends only on
    /// itself and its soil). A pot plant builds its own light grid; the plants of a
    /// plot share one grid, built from all their leaves once their geometry is updated
    /// (decision 0019). Returns the reports in plant order.
    /// Daily step 0 (decision 0020): each soil's joint nitrogen uptake from the same
    /// state, then each plant's nitrogen step, in plant order.
    fn daily_nitrogen(&mut self, now: Hours) -> Result<(), WorldError> {
        let WorldState { plants, soils, .. } = &mut self.state;
        for (si, entry) in soils.iter_mut().enumerate() {
            let members: Vec<usize> = (0..plants.len())
                .filter(|&p| plants[p].soil == si)
                .collect();
            if members.is_empty() {
                continue;
            }
            let requests: Vec<ps_soil::NitrogenRequest<'_>> = members
                .iter()
                .map(|&p| plants[p].plant.nitrogen_request())
                .collect();
            let got = entry.grid.take_up_nitrogen(&requests)?;
            for (&p, u) in members.iter().zip(got) {
                plants[p].plant.nitrogen_day(u.total(), &entry.grid, now)?;
            }
        }
        Ok(())
    }

    fn daily_plant_steps(
        &mut self,
        path: &[Option<[f64; 3]>; 24],
        now: Hours,
    ) -> Result<Vec<DayReport>, WorldError> {
        let WorldState { plants, soils, .. } = &mut self.state;
        let soils: &[SoilEntry] = soils;
        // Pots: the whole daily step. Plots: up to the light rebuild.
        let mut reports: Vec<Option<DayReport>> = plants
            .par_iter_mut()
            .map(|e| {
                let soil = &soils[e.soil].grid;
                if soils[e.soil].shared_light {
                    e.plant.day_before_light(soil, now)?;
                    Ok(None)
                } else {
                    e.plant.day(soil, path, now).map(Some)
                }
            })
            // Values keep plant order (indexed collect). If two plants fail in one step,
            // which error comes back depends on the schedule; either aborts the run.
            .collect::<Result<_, ArchError>>()?;
        let sky = ps_light::sky_sectors();
        for (si, entry) in soils.iter().enumerate() {
            if !entry.shared_light {
                continue;
            }
            let members: Vec<usize> = (0..plants.len())
                .filter(|&p| plants[p].soil == si)
                .collect();
            let Some(&first) = members.first() else {
                continue;
            };
            let mut leaves = Vec::new();
            let mut extra = Vec::new();
            let mut counts = Vec::with_capacity(members.len());
            for &p in &members {
                let (l, x) = plants[p].plant.light_inputs();
                counts.push(l.len());
                leaves.extend(l);
                extra.extend(x);
            }
            // One G per plot: the first plant's (one species per plot until WP23).
            let g = plants[first].plant.g_function();
            let (_, coeffs) = ps_light::rebuild_plot(&leaves, &extra, path, &sky, g);
            // Each plant's slice of the coefficients, then the rest of the daily step
            // for every member in parallel (each depends only on itself).
            let mut slices: Vec<Option<(usize, usize)>> = vec![None; plants.len()];
            let mut at = 0;
            for (&p, n) in members.iter().zip(counts) {
                slices[p] = Some((at, n));
                at += n;
            }
            let after: Vec<(usize, DayReport)> = plants
                .par_iter_mut()
                .enumerate()
                .filter_map(|(p, e)| {
                    slices[p].map(|(a, n)| {
                        let mine = coeffs.get(a..a + n).unwrap_or(&[]);
                        (p, e.plant.day_after_light(path, Some(mine)))
                    })
                })
                .collect();
            for (p, r) in after {
                reports[p] = Some(r);
            }
        }
        Ok(reports.into_iter().flatten().collect())
    }

    /// Total water taken up by a soil's plants since the start.
    #[must_use]
    pub fn uptake(&self, soil: u32) -> Option<KgWater> {
        self.soil_index(soil)
            .map(|i| self.state.soils[i].ledger.uptake)
    }
}
