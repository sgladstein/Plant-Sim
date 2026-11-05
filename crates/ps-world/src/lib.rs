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
use serde::{Deserialize, Serialize};

pub use hash::hash_of;
pub use scenario::{
    Action, ActionKind, Irrigation, Scenario, SeedChoice, SoilSpec, Sowing, Start, WeatherSource,
};

/// Model version: bumped on any change to an equation, a default parameter or the
/// step order (CLAUDE.md). Saves record it; replays from another version warn.
pub const MODEL_VERSION: &str = "0.1.0";

/// Air pressure used for gas exchange, kPa (sea level; no site elevation in M0).
const PATM_KPA: f64 = 101.325;
/// Clock hour of scheduled irrigation.
const IRRIGATION_HOUR: u64 = 6;
/// Initial soil temperature, °C, before the soil's air history fills.
const SOIL_T_INITIAL: f64 = 15.0;
/// Soil grid cell size in pots, m (§3.6).
const POT_CELL: f64 = 0.02;
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoilEntry {
    /// Scenario id.
    pub id: u32,
    /// The grid.
    pub grid: SoilGrid,
    /// Share of radiation reaching this soil and its plants.
    pub radiation_fraction: f64,
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
    /// Plant id.
    pub plant: u32,
    /// The plant's report.
    pub report: DayReport,
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
        let prev = self.hashes.last().copied().unwrap_or_else(|| {
            hash_of(&(&self.model_version, &self.scenario, &self.catalog))
        });
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
            Ok(ps_weather::preset_series(p, lat, start, days, s.world_seed))
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

fn pot(spec: &SoilSpec) -> Result<SoilGrid, WorldError> {
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
        let weather = weather_for(&scenario, &catalog)?;
        let mut soils = Vec::new();
        for spec in &scenario.soils {
            let id = spec.id();
            soils.push(SoilEntry {
                id,
                grid: pot(spec)?,
                radiation_fraction: scenario.radiation_fraction.get(&id).copied().unwrap_or(1.0),
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
        Ok(w)
    }

    /// Resumes from a saved state.
    ///
    /// # Errors
    /// [`WorldError::Weather`] if the weather cannot be rebuilt.
    pub fn from_state(state: WorldState) -> Result<Self, WorldError> {
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
        self.schedule(Action { tick, kind })
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
        while self.state.tick < tick {
            self.step_hour()?;
        }
        Ok(())
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
        self.run_until(u64::from(self.state.scenario.duration_days) * 24)
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
        let plant = Plant::sow(
            species,
            &traits,
            self.state.scenario.world_seed,
            u64::from(id),
            at,
            Hours(tick),
        )?;
        self.state.plants.push(PlantEntry {
            id,
            soil,
            species: s.species.clone(),
            plant,
        });
        Ok(())
    }

    fn apply_due(&mut self) -> Result<(), WorldError> {
        while let Some(a) = self.state.log.get(self.state.applied).cloned() {
            if a.tick > self.state.tick {
                break;
            }
            self.state.applied += 1;
            let err = |why: String| WorldError::Action { tick: a.tick, why };
            match a.kind {
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
                ActionKind::Prune { plant, organ } => {
                    let p = self
                        .state
                        .plants
                        .get_mut(plant as usize)
                        .ok_or_else(|| err(format!("unknown plant {plant}")))?;
                    p.plant
                        .prune(ps_graph::OrganId(organ), Hours(a.tick))
                        .map_err(|e| err(e.to_string()))?;
                }
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
            let members: Vec<usize> = (0..self.state.plants.len())
                .filter(|p| self.state.plants[*p].soil == si)
                .collect();
            // 3, 5, 6. Light, water status, gas exchange: each plant's demand, from
            // the soil before this hour's water moves.
            let mut demand = Vec::with_capacity(members.len());
            for &p in &members {
                let soil = &self.state.soils[si].grid;
                demand.push(self.state.plants[p].plant.water_demand(&ctx, soil)?);
            }
            // 4. Soil water: rain, irrigation and action water; radiation under the canopy.
            let leaf_area: f64 = members
                .iter()
                .map(|p| self.state.plants[*p].plant.leaf_area())
                .sum();
            let irr = Self::irrigation_mm(&self.state.soils[si], hour, day);
            let entry = &mut self.state.soils[si];
            let lai = leaf_area / entry.grid.surface_area().max(1e-12);
            let water_in = wx.rain.0 + irr + entry.pending_mm;
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
                .map(|(p, d)| self.state.plants[*p].plant.uptake_request(*d))
                .collect();
            let joint = self.state.soils[si].grid.take_up(&requests)?;
            self.state.soils[si].ledger.accumulate(&joint.ledger);
            // 8 to 12. Carbon.
            for (k, &p) in members.iter().enumerate() {
                let soil = &self.state.soils[si].grid;
                self.state.plants[p]
                    .plant
                    .grow_hour(&ctx, soil, joint.plants.get(k))?;
            }
        }
        self.state.tick += 1;
        if hour == 23 {
            self.step_day()?;
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
    fn step_day(&mut self) -> Result<(), WorldError> {
        let now = self.state.tick;
        let path = self.sun_path(now);
        for p in 0..self.state.plants.len() {
            let si = self.state.plants[p].soil;
            let soil = &self.state.soils[si].grid;
            let report = self.state.plants[p].plant.day(soil, &path, Hours(now))?;
            let plant = self.state.plants[p].id;
            self.state.history.push(DayRow { plant, report });
        }
        // 7. Output: the chained daily hash.
        let h = self.state.day_hash();
        self.state.hashes.push(h);
        self.state.hashed_log = self.state.applied;
        self.state.hashed_rows = self.state.history.len();
        Ok(())
    }

    /// Total water taken up by a soil's plants since the start.
    #[must_use]
    pub fn uptake(&self, soil: u32) -> Option<KgWater> {
        self.soil_index(soil)
            .map(|i| self.state.soils[i].ledger.uptake)
    }
}
