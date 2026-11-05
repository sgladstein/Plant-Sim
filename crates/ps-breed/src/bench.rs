//! The breeding bench: one headless world of up to ten pots, a generation per run
//! (WP30).

use std::collections::BTreeSet;

use ps_graph::{OrganId, OrganKind};
use ps_physio::Stage;
use ps_soil::Texture;
use ps_species::seed_colour::SeedLook;
use ps_world::{
    ActionKind, Catalog, Irrigation, MODEL_VERSION, SCENARIO_SCHEMA, Scenario, SeedChoice,
    SoilSpec, Sowing, Start, SupportSpec, WeatherSource, World,
};
use serde::{Deserialize, Serialize};

use crate::BreedError;
use crate::library::{Looks, Origin, SeedLibrary};

/// Pots on the bench.
pub const MAX_POTS: usize = 10;

/// Longest bench run, days.
const MAX_DAYS: u32 = 365;

fn default_max_days() -> u32 {
    120
}

/// A hand cross on the bench: every flower the mother opens while the father has an
/// open flower takes his pollen (WP29); her other flowers self.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchCross {
    /// Mother's pot (index into [`BenchPlan::seeds`]).
    pub mother: usize,
    /// Father's pot.
    pub father: usize,
}

/// How the bench grows its plants. The default is the WP22 trellis pot: a 40 cm loam
/// pot watered to field capacity every morning, garden nitrogen, a 2 m stake 5 cm from
/// the seed (bush beans ignore it), full sun at 45° N from 15 May.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BenchConditions {
    /// Site latitude, degrees north.
    pub latitude_deg: f64,
    /// Sowing date and hour.
    pub start: Start,
    /// Weather preset name.
    pub weather: String,
    /// Pot diameter, cm.
    pub pot_diameter_cm: f64,
    /// Pot depth, cm.
    pub pot_depth_cm: f64,
    /// Soil texture.
    pub texture: Texture,
    /// Watering, the same for every pot.
    pub irrigation: Irrigation,
    /// Share of radiation reaching the bench, 0 to 1.
    pub radiation_fraction: f64,
    /// Ambient CO2, ppm.
    pub co2_ppm: f64,
    /// Stake height, cm; 0 for none.
    pub stake_cm: f64,
}

impl Default for BenchConditions {
    fn default() -> Self {
        Self {
            latitude_deg: 45.0,
            start: Start {
                year: 2026,
                month: 5,
                day: 15,
                hour: 6,
            },
            weather: "temperate_summer".into(),
            pot_diameter_cm: 40.0,
            pot_depth_cm: 40.0,
            texture: Texture::Loam,
            irrigation: Irrigation::ToFieldCapacity { every_days: 1 },
            radiation_fraction: 1.0,
            co2_ppm: 420.0,
            stake_cm: 200.0,
        }
    }
}

/// One bench generation: which library seeds to grow, which pots to cross, how long.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchPlan {
    /// The run's name; harvested seeds name it as their origin.
    pub label: String,
    /// Library ids, one per pot (1 to [`MAX_POTS`]).
    pub seeds: Vec<u64>,
    /// Hand crosses; a pot is a mother in at most one.
    #[serde(default)]
    pub crosses: Vec<BenchCross>,
    /// World seed.
    pub world_seed: u64,
    /// Last day; the bench stops sooner once every plant is mature.
    #[serde(default = "default_max_days")]
    pub max_days: u32,
    /// Growing conditions.
    #[serde(default)]
    pub conditions: BenchConditions,
}

impl BenchPlan {
    /// Checks the plan against the library.
    ///
    /// # Errors
    /// [`BreedError::Plan`] naming the first problem.
    pub fn check(&self, library: &SeedLibrary) -> Result<(), BreedError> {
        let bad = |why: String| Err(BreedError::Plan(why));
        let n = self.seeds.len();
        if n == 0 || n > MAX_POTS {
            return bad(format!("{n} seeds; the bench holds 1 to {MAX_POTS}"));
        }
        if let Some(id) = self.seeds.iter().find(|&&id| library.get(id).is_none()) {
            return bad(format!("seed {id} is not in the library"));
        }
        if !(1..=MAX_DAYS).contains(&self.max_days) {
            return bad(format!(
                "max_days {} outside 1 to {MAX_DAYS}",
                self.max_days
            ));
        }
        let mut mothers = BTreeSet::new();
        for c in &self.crosses {
            if c.mother >= n || c.father >= n || c.mother == c.father {
                return bad(format!("cross {c:?} needs two different pots below {n}"));
            }
            if !mothers.insert(c.mother) {
                return bad(format!("pot {} is a mother in two crosses", c.mother));
            }
        }
        let k = &self.conditions;
        if !(0.0..=1.0).contains(&k.radiation_fraction) {
            return bad(format!("radiation fraction {}", k.radiation_fraction));
        }
        if !(k.stake_cm.is_finite() && k.stake_cm >= 0.0) {
            return bad(format!("stake {} cm", k.stake_cm));
        }
        Ok(())
    }

    /// The bench as a scenario: pot `i` (soil `i`, 1 m apart in a row) holds seed `i`,
    /// sown as `SeedChoice::Saved` so the plant keeps its library id.
    ///
    /// # Errors
    /// [`BreedError::Plan`] from [`BenchPlan::check`].
    pub fn scenario(&self, library: &SeedLibrary) -> Result<Scenario, BreedError> {
        self.check(library)?;
        let k = &self.conditions;
        let mut s = Scenario {
            schema: SCENARIO_SCHEMA,
            model_version: MODEL_VERSION.to_owned(),
            world_seed: self.world_seed,
            latitude_deg: k.latitude_deg,
            start: k.start,
            weather: WeatherSource::Preset(k.weather.clone()),
            soils: Vec::new(),
            radiation_fraction: std::collections::BTreeMap::new(),
            rain_fraction: std::collections::BTreeMap::new(),
            nitrogen: std::collections::BTreeMap::new(),
            irrigation: std::collections::BTreeMap::new(),
            co2_ppm: k.co2_ppm,
            sowings: Vec::new(),
            supports: Vec::new(),
            obstacles: Vec::new(),
            duration_days: self.max_days,
            heat_wave: None,
            events: Vec::new(),
            variation: ps_world::FieldVariation::default(),
        };
        for (i, &id) in self.seeds.iter().enumerate() {
            let seed = library
                .get(id)
                .ok_or_else(|| BreedError::Plan(format!("seed {id} is not in the library")))?;
            let soil = u32::try_from(i).unwrap_or(u32::MAX);
            #[allow(clippy::cast_precision_loss)] // at most ten pots
            let x = i as f64;
            s.soils.push(SoilSpec::Pot {
                id: soil,
                diameter_cm: k.pot_diameter_cm,
                depth_cm: k.pot_depth_cm,
                texture: k.texture,
                position_m: (x, 0.0),
            });
            s.irrigation.insert(soil, k.irrigation.clone());
            if (k.radiation_fraction - 1.0).abs() > 0.0 {
                s.radiation_fraction.insert(soil, k.radiation_fraction);
            }
            s.sowings.push(Sowing {
                soil,
                species: seed.species.clone(),
                seed: SeedChoice::Saved {
                    id,
                    genotype: seed.genotype.clone(),
                },
                depth_cm: 3.0,
                offset_cm: None,
            });
            if k.stake_cm > 0.0 {
                s.supports.push(SupportSpec {
                    soil,
                    offset_cm: (5.0, 0.0),
                    height_cm: k.stake_cm,
                    diameter_cm: 1.5,
                });
            }
        }
        Ok(s)
    }
}

/// What the bench measured on one plant: what a breeder writes down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BenchRow {
    /// Pot, 0 to 9.
    pub pot: usize,
    /// Library id of the seed sown.
    pub seed: u64,
    /// Species id.
    pub species: String,
    /// Father's pot when the plan crossed this plant.
    pub crossed_with: Option<usize>,
    /// Day of emergence (VE), days after sowing.
    pub emergence_day: Option<u32>,
    /// Day of the first open flower (R1).
    pub flowering_day: Option<u32>,
    /// Day of physiological maturity (R9).
    pub maturity_day: Option<u32>,
    /// Tallest shoot over the run, cm.
    pub height_cm: f64,
    /// Largest living leaf area over the run, cm².
    pub leaf_area_cm2: f64,
    /// Largest living structural dry mass over the run, g.
    pub dry_mass_g: f64,
    /// Pods picked with seeds.
    pub pods: u32,
    /// Seeds picked.
    pub seeds: u32,
    /// Their dry mass, g.
    pub seed_yield_g: f64,
    /// Mean seed dry mass, mg; 0 without seeds.
    pub mean_seed_mg: f64,
    /// Its flower and the coat of the seeds it bears, when the species has a colour table.
    pub look: Option<SeedLook>,
    /// Library ids of the seeds it gave (filled by [`run_generation`]).
    pub saved: Vec<u64>,
}

/// A grown generation.
#[derive(Clone, Debug, PartialEq)]
pub struct Generation {
    /// One row per pot.
    pub rows: Vec<BenchRow>,
    /// Library ids of every seed picked, in pot, pod and seed order.
    pub saved: Vec<u64>,
    /// Days the bench ran.
    pub days: u32,
    /// The bench world's full-state hash at the end.
    pub hash: u64,
}

/// The living seeds of a pod.
fn seeds_of(arena: &ps_graph::OrganArena, pod: OrganId) -> usize {
    arena.children[pod.index()]
        .iter()
        .filter(|c| arena.alive[c.index()] && arena.kind[c.index()] == OrganKind::SeedInPod)
        .count()
}

/// Grows the plan: steps day by day, crossing as the plan says while both plants flower,
/// stops once every plant is mature or at `max_days`, then picks every pod with seeds.
///
/// # Errors
/// [`BreedError::Plan`] for a bad plan, [`BreedError::World`] from the world.
pub fn grow(
    library: &SeedLibrary,
    plan: &BenchPlan,
    catalog: &Catalog,
) -> Result<World, BreedError> {
    let mut w = World::new(plan.scenario(library)?, catalog.clone())?;
    let end = u64::from(plan.max_days) * 24;
    let mut crossed = BTreeSet::new();
    while w.tick() < end {
        w.run_days(1)?;
        let mut due = Vec::new();
        let plants = &w.state().plants;
        for c in &plan.crosses {
            if !plants[c.father].plant.has_open_flower() {
                continue;
            }
            let m = &plants[c.mother].plant;
            for f in &m.flowers {
                if m.arena.alive[f.id.index()] && crossed.insert((c.mother, f.id.0)) {
                    due.push((c.mother, f.id.0, c.father));
                }
            }
        }
        for (mother, organ, father) in due {
            w.apply(ActionKind::Pollinate {
                plant: u32::try_from(mother).unwrap_or(u32::MAX),
                organ,
                pollen: u32::try_from(father).unwrap_or(u32::MAX),
            })?;
        }
        let done = w
            .state()
            .plants
            .iter()
            .all(|e| e.plant.phenology.stage == Stage::Mature);
        if done {
            break;
        }
    }
    let mut picks = Vec::new();
    for e in &w.state().plants {
        let a = &e.plant.arena;
        for pod in a.alive_of(OrganKind::Pod) {
            if seeds_of(a, pod) > 0 {
                picks.push((e.id, pod.0));
            }
        }
    }
    for (plant, organ) in picks {
        w.apply(ActionKind::Harvest { plant, organ })?;
    }
    // The picks apply at the start of the next hour.
    w.run_until(w.tick() + 1)?;
    Ok(w)
}

/// Measures every plant of a grown bench.
///
/// # Errors
/// [`BreedError::Species`] if a plant's species is missing from the world.
pub fn measure(
    world: &World,
    plan: &BenchPlan,
    looks: &Looks,
) -> Result<Vec<BenchRow>, BreedError> {
    let st = world.state();
    let mut rows = Vec::with_capacity(st.plants.len());
    for (pot, e) in st.plants.iter().enumerate() {
        let species = st
            .catalog
            .species
            .get(&e.species)
            .ok_or_else(|| BreedError::Species(format!("{} not in the world", e.species)))?;
        let history = || st.history.iter().filter(|r| r.plant == e.id);
        let first = |stage: Stage| {
            history()
                .find(|r| r.report.stage.is_some_and(|s| s >= stage))
                .map(|r| r.day)
        };
        let most =
            |f: fn(&ps_arch::DayReport) -> f64| history().map(|r| f(&r.report)).fold(0.0, f64::max);
        let lots = || st.seed_library.iter().filter(|l| l.plant == e.id);
        let seeds: u32 = lots().map(|l| l.seeds).sum();
        let seed_yield_g: f64 = lots().map(|l| l.seed_mass_g).sum();
        rows.push(BenchRow {
            pot,
            seed: e.seed_id.unwrap_or(plan.seeds[pot]),
            species: e.species.clone(),
            crossed_with: plan
                .crosses
                .iter()
                .find(|c| c.mother == pot)
                .map(|c| c.father),
            emergence_day: first(Stage::Emerged),
            flowering_day: first(Stage::Flowering),
            maturity_day: first(Stage::Mature),
            height_cm: most(|r| r.height) * 100.0,
            leaf_area_cm2: most(|r| r.leaf_area) * 1e4,
            dry_mass_g: most(|r| r.dry_mass),
            pods: u32::try_from(lots().count()).unwrap_or(u32::MAX),
            seeds,
            seed_yield_g,
            mean_seed_mg: if seeds > 0 {
                seed_yield_g / f64::from(seeds) * 1000.0
            } else {
                0.0
            },
            look: looks.of(&e.species, species, &e.genotype),
            saved: Vec::new(),
        });
    }
    Ok(rows)
}

/// Grows, measures and saves one generation: the picked seeds join the library with
/// their parents, and each row lists the ids of the seeds its plant gave.
///
/// # Errors
/// As [`grow`], [`measure`] and [`SeedLibrary::add_lots`].
pub fn run_generation(
    library: &mut SeedLibrary,
    plan: &BenchPlan,
    catalog: &Catalog,
    looks: &Looks,
) -> Result<Generation, BreedError> {
    let w = grow(library, plan, catalog)?;
    let mut rows = measure(&w, plan, looks)?;
    let saved = library.add_lots(w.state(), 0, &plan.label, looks)?;
    for &id in &saved {
        if let Some(Origin::Harvest { plant, .. }) = library.get(id).map(|s| &s.origin)
            && let Some(row) = rows.get_mut(*plant as usize)
        {
            row.saved.push(id);
        }
    }
    let days = u32::try_from(w.tick() / 24).unwrap_or(u32::MAX);
    Ok(Generation {
        rows,
        saved,
        days,
        hash: w.state_hash(),
    })
}
