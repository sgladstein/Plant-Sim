//! Growth-form modules (design plan §3.4, §3.7, §3.10, §3.11; WP8).
//!
//! M0 has one module, the determinate herb (bush bean), and the root system shared by
//! all forms. A [`Plant`] owns its organ graph and the module state around it (axes,
//! buds, racemes, flowers, pods, root axes) and runs the plant's part of the §3.2
//! schedule:
//!
//! - hourly, [`Plant::water_demand`] (steps 3, 5 and 6: light, water status, gas
//!   exchange) and, after the world's joint uptake (step 7), [`Plant::grow_hour`]
//!   (steps 8 to 12 through `ps_physio::allocation`, plus germination and thermal time);
//! - daily, [`Plant::day`] (steps 1 to 6: phenology, bud fate, initiation, senescence,
//!   root elongation, light rebuild).
//!
//! The world (WP9) owns weather, soil, the action log and the hash.
//! Model card: docs/models/herb-architecture.md; decisions 0013.

pub mod geometry;
pub mod grass;
pub mod nitrogen;
pub mod params;
mod roots;
mod shoot;
pub mod vine;

use ps_graph::{
    ExpansionCurve, GraphError, LightCoeffs, NewOrgan, OrganArena, OrganId, OrganKind, Orientation,
    SpecificSize,
};
use ps_light::SkySector;
use ps_physio::allocation::{self, CarbonLedger, HourInputs, HourReport};
use ps_physio::classes::{REFERENCE_HEIGHT, Reference};
use ps_physio::energy::{self, Air, Site};
use ps_physio::{ClassTable, Events, Phenology, PhysioError, Stage, leaf_capacity};
use ps_rng::{Purpose, RngKey};
use ps_soil::{RootCells, SoilError, SoilGrid, Uptake, stress_factor};
use ps_species::{Species, Traits};
use ps_units::{
    Celsius, DegreeDays, GramsC, GramsDM, GramsN, Hours, KPa, KgWater, Meters, MolPerM2Day, Ppm,
    UmolPerM2S,
};
use ps_weather::HourWeather;
use serde::{Deserialize, Serialize};

pub use geometry::V3;
pub use grass::GrassParams;
pub use nitrogen::{NitrogenDay, NitrogenParams, PlantNitrogen};
pub use params::{HerbParams, RootParams, ShadeSignal, VariationParams, scatter};
pub use vine::{Climb, ClimbingParams, Support, Twine};

/// kg of water per mol.
const KG_PER_MOL_WATER: f64 = 0.018_015;

/// Errors from the growth-form module.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ArchError {
    /// The organ graph rejected an operation.
    #[error(transparent)]
    Graph(#[from] GraphError),
    /// Physiology rejected its inputs.
    #[error(transparent)]
    Physio(#[from] PhysioError),
    /// The soil rejected a root list.
    #[error(transparent)]
    Soil(#[from] SoilError),
    /// An expansion curve could not be built.
    #[error("expansion curve: {0}")]
    Curve(String),
    /// The organ cannot be cut (the collar or hypocotyl: cutting it kills the plant).
    #[error("organ {0:?} cannot be pruned")]
    NotPrunable(OrganId),
    /// Only a pod or an ear can be harvested.
    #[error("organ {0:?} is not a pod or an ear")]
    NotHarvestable(OrganId),
}

/// What a harvest took off a plant (WP25): the seeds of one pod or ear.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Harvested {
    /// Seeds (grains) the pod or ear carried.
    pub seeds: u32,
    /// Their structural dry mass.
    pub seed_mass: GramsDM,
    /// Whether the plant had reached maturity: seeds from an earlier harvest are green.
    pub mature: bool,
}

/// What an organ is in the architecture (finer than [`OrganKind`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// The seed, then the collar at the base of the hypocotyl.
    Collar,
    /// The hypocotyl.
    Hypocotyl,
    /// A stem internode.
    Stem,
    /// A cotyledon.
    Cotyledon,
    /// A unifoliate leaf.
    Unifoliate,
    /// A trifoliate leaf.
    Trifoliate,
    /// An axillary bud.
    Bud,
    /// A raceme (an inflorescence axis).
    Raceme,
    /// A flower.
    Flower,
    /// A pod wall.
    Pod,
    /// A seed in a pod.
    SeedInPod,
    /// A root segment or a cohort of laterals.
    Root,
    /// A cluster of root nodules (WP24).
    Nodule,
    /// A grass leaf sheath, rolled round the stem above its node (WP23).
    Sheath,
    /// A grass leaf blade (WP23).
    Blade,
    /// The internode that carries a grass ear (WP23).
    Peduncle,
    /// A grass ear (WP23).
    Ear,
    /// A grain in an ear (WP23).
    Grain,
    /// A grass seed's endosperm, holding the seed reserve (WP23).
    Endosperm,
}

/// Days over which the leaf light record averages (decision 0018).
const DLI_MEMORY_DAYS: f64 = 3.0;

/// Architecture data per organ, indexed like the arena.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    /// Role.
    pub role: Role,
    /// Where along the parent it attaches (1 = the parent's tip).
    pub attach: f64,
    /// Number of identical roots the organ stands for (a cohort of laterals).
    pub multiplicity: f64,
    /// Thermal age when expansion completed.
    pub expanded: Option<DegreeDays>,
    /// Yesterday's light index (leaves): mean of the direct and diffuse coefficients.
    pub light_index: f64,
    /// Consecutive days below the shade threshold (leaves).
    pub shade_days: u32,
    /// Potential area (leaves), for the petiole length.
    pub potential_area: f64,
    /// PAR absorbed so far today per unit leaf area (leaves), mol m⁻².
    #[serde(default)]
    pub par_today: MolPerM2Day,
    /// Hours counted in `par_today`.
    #[serde(default)]
    pub par_hours: u32,
    /// Running mean of daily absorbed PAR per unit leaf area (leaves), mol m⁻² day⁻¹;
    /// `None` until the leaf has seen a whole day (decision 0018).
    #[serde(default)]
    pub dli: Option<MolPerM2Day>,
    /// Nitrogen this nodule fixes per day at the current setting, g N (decision 0020).
    #[serde(default)]
    pub fixation: f64,
    /// Whether nitrogen has been mined from this leaf.
    #[serde(default)]
    pub mined: bool,
    /// The coil of an internode wound around a support (WP22).
    #[serde(default)]
    pub twine: Option<Twine>,
    /// The leaf's temperature in the last hour, area-weighted over its sunlit and
    /// shaded parts (leaves; WP27).
    #[serde(default)]
    pub t_leaf: Option<Celsius>,
    /// The leaf's lowest temperature so far today, for frost (leaves; WP27).
    #[serde(default)]
    pub t_leaf_min: Option<Celsius>,
}

impl Meta {
    fn new(role: Role) -> Self {
        Self {
            role,
            attach: 1.0,
            multiplicity: 1.0,
            expanded: None,
            light_index: 1.0,
            shade_days: 0,
            potential_area: 0.0,
            par_today: MolPerM2Day(0.0),
            par_hours: 0,
            dli: None,
            fixation: 0.0,
            mined: false,
            twine: None,
            t_leaf: None,
            t_leaf_min: None,
        }
    }
}

/// A shoot axis: the main stem or a branch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShootAxis {
    /// Branching order.
    pub order: u8,
    /// The axis' first internode (the axis dies with it).
    pub base: OrganId,
    /// The organ new internodes attach to (the last internode, or the bud).
    pub apex: OrganId,
    /// Nodes made.
    pub nodes: u32,
    /// Nodes before the terminal raceme.
    pub max_nodes: u32,
    /// Thermal time since the last node.
    pub since_node: DegreeDays,
    /// No more nodes.
    pub terminated: bool,
    /// The terminal raceme, once made.
    pub raceme: Option<OrganId>,
    /// Azimuth of the axis' first leaf.
    pub azimuth: f64,
    /// The newest leaf, for the shade signal.
    pub last_leaf: Option<OrganId>,
    /// How the axis climbs (vines, WP22).
    #[serde(default)]
    pub climb: Climb,
    /// Stem length made while upright and unsupported, m (vines).
    #[serde(default)]
    pub free_length: f64,
}

/// Fate of an axillary bud.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudFate {
    /// Waiting.
    Dormant,
    /// Grew into a branch.
    Branch,
    /// Became a raceme.
    Raceme,
    /// A grass tiller site that stayed empty (WP23).
    Suppressed,
}

/// An axillary bud on the main stem.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bud {
    /// The bud organ.
    pub id: OrganId,
    /// Main-stem node.
    pub rank: u32,
    /// The subtending leaf.
    pub leaf: OrganId,
    /// Azimuth of the subtending leaf.
    pub azimuth: f64,
    /// Fate.
    pub fate: BudFate,
    /// The axis it sits on (index into [`Plant::axes`]; grasses tiller from tillers).
    #[serde(default)]
    pub axis: usize,
}

/// A raceme opening its flowers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Raceme {
    /// The raceme organ.
    pub id: OrganId,
    /// Flowers opened.
    pub opened: u32,
    /// Fractional flowers due.
    pub due: f64,
}

/// An open flower.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flower {
    /// The flower organ.
    pub id: OrganId,
    /// Its raceme.
    pub raceme: OrganId,
    /// Days open.
    pub days: u32,
    /// Sum of daily pod-set temperature factors.
    pub temperature: f64,
    /// Sum of daily carbon factors.
    pub carbon: f64,
    /// Sum of daily shade factors on pod set (decision 0018).
    #[serde(default)]
    pub light: f64,
    /// Mean pod-set temperature factor over the days before the flower opened, while
    /// its pollen developed (decision 0018).
    #[serde(default = "one")]
    pub pre_anthesis: f64,
    /// Lowest daily FTSW while open.
    pub min_ftsw: f64,
}

/// A pod.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pod {
    /// The pod organ.
    pub id: OrganId,
    /// Seeds created.
    pub seeds: bool,
    /// Mean pod-set temperature factor while its flower was open: the share of ovules
    /// that can still set after heat (decision 0018).
    #[serde(default = "one")]
    pub fertility: f64,
}

/// One root axis (a sequence of segments) or one lateral cohort.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RootAxis {
    /// Index into [`HerbParams::roots`].
    pub kind: usize,
    /// Segments from base to tip, with the axis length at each segment's start.
    pub segments: Vec<(OrganId, f64)>,
    /// Realized length of completed segments.
    pub length: f64,
    /// Direction of the newest segment.
    pub direction: V3,
    /// Roots the axis stands for.
    pub multiplicity: f64,
    /// No more growth.
    pub done: bool,
    /// Lateral cohorts made.
    pub laterals: u32,
}

/// Weather and soil seen during the current day, for daily rules.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DayStats {
    /// Hours recorded.
    pub hours: u32,
    /// Sum of air temperatures.
    pub t_sum: f64,
    /// Lowest air temperature.
    pub t_min: f64,
    /// Highest air temperature.
    #[serde(default = "lowest")]
    pub t_max: f64,
    /// Lowest FTSW.
    pub ftsw_min: f64,
    /// Plant thermal time (air) gained, for axis phyllochrons.
    pub thermal: f64,
    /// Gross assimilation the leaves could have made, µmol s⁻¹ summed over hours.
    pub gross_potential: f64,
    /// Gross assimilation lost to the sink feedback, same units.
    pub gross_lost: f64,
    /// Net leaf assimilation today (after leaf day respiration and the sink feedback),
    /// g C.
    #[serde(default)]
    pub assimilation: f64,
    /// Water taken from the soil today, which the plant transpires, kg.
    #[serde(default)]
    pub transpiration: f64,
}

fn lowest() -> f64 {
    f64::MIN
}

fn one() -> f64 {
    1.0
}

impl Default for DayStats {
    fn default() -> Self {
        Self {
            hours: 0,
            t_sum: 0.0,
            t_min: f64::MAX,
            t_max: f64::MIN,
            ftsw_min: 1.0,
            thermal: 0.0,
            gross_potential: 0.0,
            gross_lost: 0.0,
            assimilation: 0.0,
            transpiration: 0.0,
        }
    }
}

/// Counts and totals after a day, for tests, the CSV and the validation report.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DayReport {
    /// Days since sowing.
    pub day: u32,
    /// Phenological stage.
    pub stage: Option<Stage>,
    /// Main-stem nodes (the unifoliate node is node 1).
    pub main_nodes: u32,
    /// Branches.
    pub branches: u32,
    /// Living leaves.
    pub leaves: u32,
    /// Living leaf area, m².
    pub leaf_area: f64,
    /// Racemes.
    pub racemes: u32,
    /// Flowers opened so far.
    pub flowers: u32,
    /// Living pods.
    pub pods: u32,
    /// Living seeds in pods.
    pub seeds: u32,
    /// Root axes from the seed (order 0).
    pub root_axes: u32,
    /// Lateral cohorts.
    pub lateral_cohorts: u32,
    /// Total living root length, m.
    pub root_length: f64,
    /// Root dry mass, g.
    pub root_mass: f64,
    /// Total living structural dry mass, g.
    pub dry_mass: f64,
    /// Structural dry mass ever built, living or shed, g (the plant's total
    /// production, for comparison with harvested totals).
    pub total_dry_mass: f64,
    /// Share of today's potential gross assimilation lost to the sink feedback
    /// (decision 0013 diagnostic: above about 0.2 the plant lacks sinks).
    pub feedback_loss: f64,
    /// Pod and seed dry mass, g.
    pub fruit_mass: f64,
    /// Living leaf blade and petiole dry mass (unifoliates and trifoliates), g.
    pub leaf_mass: f64,
    /// Living stem dry mass (internodes, the hypocotyl included), g.
    pub stem_mass: f64,
    /// Seed dry mass in pods, g.
    pub seed_mass: f64,
    /// Height of the highest shoot point above the soil, m.
    pub height: f64,
    /// Water uptake shortfall today, kg.
    pub shortfall: f64,
    /// Nitrogen in the plant (organs and pool), g N (WP24).
    #[serde(default)]
    pub nitrogen: f64,
    /// Nitrogen taken up from the soil today, g N.
    #[serde(default)]
    pub n_uptake: f64,
    /// Nitrogen fixed today, g N.
    #[serde(default)]
    pub n_fixed: f64,
    /// Nitrogen mobilized from old organs today, g N.
    #[serde(default)]
    pub n_mined: f64,
    /// Nitrogen demand not met today, g N.
    #[serde(default)]
    pub n_unmet: f64,
    /// Nitrogen factor on expansion (1 = no deficit).
    #[serde(default)]
    pub n_status: f64,
    /// Living nodule dry mass, g.
    #[serde(default)]
    pub nodule_mass: f64,
    /// Carbon respired to fix nitrogen today, g C.
    #[serde(default)]
    pub fixation_carbon: f64,
    /// Net leaf assimilation today (after leaf day respiration and the sink feedback),
    /// g C.
    #[serde(default)]
    pub assimilation: f64,
    /// Water taken from the soil today, which the plant transpires, kg (the demand less
    /// the shortfall).
    #[serde(default)]
    pub transpiration: f64,
}

/// What sets one plant apart from its siblings at sowing (decision 0027): the seed it
/// grew from and how fast that seed comes up. Drawn once in [`Plant::sow`] from the
/// plant's own keyed streams, so adding another plant never changes it.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Individual {
    /// Mass of the seed sown relative to the species' seed mass.
    pub seed_factor: f64,
    /// This seed's thermal time from germination to emergence, depth included.
    pub emergence: DegreeDays,
    /// Compass direction of the seedling's first pair of organs (a dicot's
    /// cotyledons and unifoliates turn with it; a grass draws its first blade at
    /// emergence instead).
    pub azimuth: ps_units::Radians,
}

impl Default for Individual {
    fn default() -> Self {
        Self {
            seed_factor: 1.0,
            emergence: DegreeDays(0.0),
            azimuth: ps_units::Radians(0.0),
        }
    }
}

impl Individual {
    /// Draws a plant's individual traits from its keyed streams at sowing.
    #[must_use]
    pub fn draw(
        params: &HerbParams,
        world_seed: u64,
        entity: u64,
        tick: Hours,
        depth: Meters,
    ) -> Self {
        let v = &params.variation;
        let key = |purpose| RngKey::new(world_seed, plant_key(entity), tick.0, purpose).stream();
        let seed_factor = scatter(key(Purpose::SeedSize).standard_normal(), v.seed_mass_cv);
        let base = params.thresholds.emergence.0;
        let deeper = v.emergence_per_depth * (depth.0 - v.reference_depth.0);
        let spread = scatter(key(Purpose::Germination).standard_normal(), v.emergence_cv);
        let emergence = DegreeDays(((base + deeper) * spread).max(0.25 * base));
        let azimuth = key(Purpose::Phyllotaxis).uniform() * core::f64::consts::TAU;
        Self {
            seed_factor,
            emergence,
            azimuth: ps_units::Radians(azimuth),
        }
    }
}

/// The random-key entity of a plant's own (not an organ's) draws: the plant entity in
/// the high word and an organ index no arena reaches in the low word.
const fn plant_key(entity: u64) -> u64 {
    (entity << 32) | 0xFFFF_FFFF
}

/// One plant: its organ graph and the growth-form state around it. Serializes
/// completely for snapshots (the hourly scratch and the constant sky are rebuilt).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plant {
    /// Parameters.
    pub params: HerbParams,
    /// World seed, for the plant's random streams.
    pub world_seed: u64,
    /// Plant entity id, for random keys.
    pub entity: u64,
    /// Organ graph.
    pub arena: OrganArena,
    /// Architecture data per organ.
    pub meta: Vec<Meta>,
    /// Carbon ledger.
    pub ledger: CarbonLedger,
    /// Nitrogen pool, ledger and status (WP24).
    #[serde(default)]
    pub nitrogen: PlantNitrogen,
    /// Phenology.
    pub phenology: Phenology,
    /// The seed this plant grew from and its emergence (decision 0027).
    #[serde(default)]
    pub individual: Individual,
    /// Collar position in the soil frame (x, y, z = −sowing depth).
    pub collar_position: V3,
    /// Sowing depth.
    pub depth: Meters,
    /// The seed / collar organ.
    pub collar: OrganId,
    /// The hypocotyl.
    pub hypocotyl: OrganId,
    /// Shoot axes; index 0 is the main stem once emerged.
    pub axes: Vec<ShootAxis>,
    /// Main-stem axillary buds.
    pub buds: Vec<Bud>,
    /// Racemes.
    pub racemes: Vec<Raceme>,
    /// Open flowers.
    pub flowers: Vec<Flower>,
    /// Daily pod-set temperature factors of the last days, newest last, for heat
    /// during pollen development (decision 0018).
    #[serde(default)]
    pub recent_heat: Vec<f64>,
    /// Pods.
    pub pods: Vec<Pod>,
    /// Root axes.
    pub root_axes: Vec<RootAxis>,
    /// Root types whose axes have appeared.
    pub root_types_started: Vec<bool>,
    /// Root length per soil cell, rebuilt daily.
    pub roots: RootCells,
    /// Organ base and tip positions, rebuilt daily.
    pub positions: Vec<(V3, V3)>,
    /// Hours the seed has spent moist and warm enough to germinate.
    pub germination_hours: u32,
    /// Days since sowing.
    pub day: u32,
    /// Day of emergence.
    pub emerged_day: Option<u32>,
    /// Seed reserve at sowing.
    pub seed_reserve: GramsC,
    /// Flowers opened so far.
    pub flowers_opened: u32,
    /// Upper buds turned to racemes at flowering.
    pub flowering_started: bool,
    /// Leaves shed at maturity.
    pub leaves_dropped: bool,
    /// Plant thermal time since first flower (vines: the main stem stops after
    /// `vegetative_after_flowering`).
    #[serde(default)]
    pub since_flowering: DegreeDays,
    /// Supports in the plant's soil frame (WP22).
    #[serde(default)]
    pub supports: Vec<Support>,
    /// Obstacles that shade the plant, in its soil frame (WP25).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstacles: Vec<ps_light::Obstacle>,
    /// A grass has started stem elongation (WP23).
    #[serde(default)]
    pub jointed: bool,
    /// Today's weather and soil.
    pub today: DayStats,
    /// Today's uptake shortfall.
    pub shortfall: KgWater,
    /// Current FTSW.
    pub ftsw: f64,
    /// Stress factors on conductance and expansion.
    pub stress: (f64, f64),
    /// This hour's leaf income, indexed like the arena (scratch between the two
    /// hourly calls; empty at the daily boundary).
    #[serde(skip)]
    income: Vec<GramsC>,
    #[serde(skip)]
    sky: Vec<SkySector>,
}

/// What a plant's hourly class table depends on besides the hour's weather: leaf
/// parameter bits, leaf energy parameter bits and the conductance stress factor's bits
/// ([`Plant::class_table_key`]).
pub type TableKey = ([u64; 14], [u64; 4], u64);

/// The hour's air for the leaf energy balance.
fn hour_air(ctx: &HourCtx<'_>) -> Air {
    let wx = ctx.weather;
    Air {
        t_air: wx.t_air,
        vpd: wx.vpd,
        patm: ctx.patm,
        cloud: wx.cloud_fraction,
    }
}

/// Inputs to the plant's hourly steps.
#[derive(Copy, Clone, Debug)]
pub struct HourCtx<'a> {
    /// The hour's weather.
    pub weather: &'a HourWeather,
    /// Clock hour 0 to 23 (index into the light coefficients).
    pub hour: usize,
    /// Sim tick.
    pub tick: Hours,
    /// Ambient CO2.
    pub co2: Ppm,
    /// Air pressure.
    pub patm: KPa,
}

/// One plant's hourly lens data (WP26), filled by [`Plant::grow_hour_recording`]:
/// values the step computes anyway, kept for the viewer. Never part of the state, so
/// never saved or hashed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HourLens {
    /// Carbon moved from each organ's parent into it during the hour (negative: an
    /// export to the parent), indexed by organ id.
    pub phloem_import: Vec<GramsC>,
    /// Each organ's temperature: a leaf's own (WP27, its gas exchange and frost; its
    /// expansion reads air), air for the rest of the shoot, the soil layer for roots
    /// and the seed.
    pub temperature: Vec<Celsius>,
}

/// Share of potential mass a new organ starts with (decision 0013; ~).
pub const PRIMORDIUM: f64 = 0.02;

/// What [`Plant::spawn`] needs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Spawn {
    pub kind: OrganKind,
    pub role: Role,
    pub parent: OrganId,
    pub order: u8,
    pub curve: ExpansionCurve,
    pub specific: SpecificSize,
    pub direction: V3,
    pub azimuth: f64,
    pub attach: f64,
    pub multiplicity: f64,
    pub potential_area: f64,
}

impl Spawn {
    pub(crate) fn new(kind: OrganKind, role: Role, parent: OrganId, curve: ExpansionCurve) -> Self {
        Self {
            kind,
            role,
            parent,
            order: 0,
            curve,
            specific: SpecificSize::Bulk,
            direction: [0.0, 0.0, 1.0],
            azimuth: 0.0,
            attach: 1.0,
            multiplicity: 1.0,
            potential_area: 0.0,
        }
    }
}

pub(crate) fn curve(duration: DegreeDays, potential: GramsDM) -> Result<ExpansionCurve, ArchError> {
    if potential.0 <= 0.0 {
        return Ok(ExpansionCurve::none());
    }
    ExpansionCurve::new(duration, DegreeDays(0.5 * duration.0), potential)
        .map_err(|e| ArchError::Curve(e.to_string()))
}

impl Plant {
    /// Sows a seed at `collar_position` (soil frame; z is minus the sowing depth).
    ///
    /// The seed is four organs from the start (§3.10; decision 0013): the collar (the
    /// embryo's root pole, where roots attach), the hypocotyl, not yet expanding, and the
    /// two cotyledons on top of it holding the mobilizable reserve. Germination starts
    /// the hypocotyl's expansion curve, which lifts the cotyledons to the surface.
    ///
    /// # Errors
    /// [`ArchError::Graph`] only on an internal inconsistency.
    #[allow(clippy::too_many_lines)]
    pub fn sow(
        species: &Species,
        traits: &Traits,
        world_seed: u64,
        entity: u64,
        collar_position: V3,
        tick: Hours,
    ) -> Result<Self, ArchError> {
        let mut params = HerbParams::new(species, traits);
        let individual = Individual::draw(
            &params,
            world_seed,
            entity,
            tick,
            Meters(-collar_position[2]),
        );
        // A bigger seed carries more reserve and bigger preformed leaves (decision 0027).
        params.seed_mass = GramsDM(params.seed_mass.0 * individual.seed_factor);
        params.unifoliate_area = ps_units::M2(
            params.unifoliate_area.0
                * libm::pow(individual.seed_factor, params.variation.seed_leaf_exponent),
        );
        params.thresholds.emergence = individual.emergence;
        let az_seed = individual.azimuth.0;
        let mut arena = OrganArena::new();
        let fixed = params.seed_mass.0 * (1.0 - params.seed_mobilizable);
        let reserve =
            GramsC(params.seed_mass.0 * params.seed_carbon_fraction * params.seed_mobilizable);
        let organ = |kind, parent, structural: f64, reserve: GramsC, specific| NewOrgan {
            kind,
            parent,
            axis_order: 0,
            birth: tick,
            curve: ExpansionCurve::none(),
            specific,
            orientation: Orientation::UP,
            structural: GramsDM(structural),
            reserve,
            nitrogen: GramsN(0.0),
        };
        let collar = arena.add(
            &organ(
                OrganKind::Seed,
                None,
                0.2 * fixed,
                GramsC(0.0),
                SpecificSize::Bulk,
            ),
            &params.allometry,
        )?;
        let hypocotyl = arena.add(
            &organ(
                OrganKind::Internode,
                Some(collar),
                0.0,
                GramsC(0.0),
                SpecificSize::Length(params.hypocotyl_specific_length),
            ),
            &params.allometry,
        )?;
        let mut meta = vec![Meta::new(Role::Collar), Meta::new(Role::Hypocotyl)];
        if params.grass.is_some() {
            // A grass seed keeps its reserve in the endosperm, which stays in the soil
            // while the coleoptile carries the shoot up (decision 0022).
            let o = organ(
                OrganKind::Cotyledon,
                Some(collar),
                0.8 * fixed,
                reserve,
                SpecificSize::Bulk,
            );
            arena.add(&o, &params.allometry)?;
            meta.push(Meta::new(Role::Endosperm));
        }
        for k in 0..if params.grass.is_some() { 0 } else { 2u32 } {
            let az = az_seed + f64::from(k) * core::f64::consts::PI;
            let mut o = organ(
                OrganKind::Cotyledon,
                Some(hypocotyl),
                0.4 * fixed,
                reserve * 0.5,
                SpecificSize::Bulk,
            );
            o.orientation = Orientation {
                direction: geometry::from_azimuth(az + core::f64::consts::FRAC_PI_2, 0.3),
                azimuth: ps_units::Radians(az),
            };
            arena.add(&o, &params.allometry)?;
            meta.push(Meta::new(Role::Cotyledon));
        }
        let ledger = CarbonLedger::start(&arena, params.carbon.carbon_fraction);
        let n_roots = params.roots.len();
        let mut p = Self {
            params,
            world_seed,
            entity,
            arena,
            meta,
            ledger,
            nitrogen: PlantNitrogen::default(),
            phenology: Phenology::default(),
            individual,
            collar_position,
            depth: Meters(-collar_position[2]),
            collar,
            hypocotyl,
            axes: Vec::new(),
            buds: Vec::new(),
            racemes: Vec::new(),
            flowers: Vec::new(),
            recent_heat: Vec::new(),
            pods: Vec::new(),
            root_axes: Vec::new(),
            root_types_started: vec![false; n_roots],
            roots: RootCells::default(),
            positions: Vec::new(),
            germination_hours: 0,
            day: 0,
            emerged_day: None,
            seed_reserve: reserve,
            flowers_opened: 0,
            flowering_started: false,
            leaves_dropped: false,
            since_flowering: DegreeDays(0.0),
            supports: Vec::new(),
            obstacles: Vec::new(),
            jointed: false,
            today: DayStats::default(),
            shortfall: KgWater(0.0),
            ftsw: 1.0,
            stress: (1.0, 1.0),
            income: Vec::new(),
            sky: ps_light::sky_sectors().to_vec(),
        };
        p.positions = geometry::layout(&p.arena, &p.meta, p.collar_position);
        p.sow_nitrogen();
        Ok(p)
    }

    /// Creates an organ with a primordium: a small share of its potential mass built at
    /// once from the parent's reserve at the usual construction cost (decision 0013), so
    /// that a new sink can hold reserve and start growing.
    pub(crate) fn spawn(&mut self, s: &Spawn, now: Hours) -> Result<OrganId, ArchError> {
        let cf = self.params.carbon.carbon_fraction;
        let pi = s.parent.index();
        let available = self.arena.reserve[pi].0.max(0.0);
        // Built like any growth: construction cost includes growth respiration.
        let cost = self.params.carbon.construction_cost.of(s.kind);
        let mass = (PRIMORDIUM * s.curve.potential.0).min(0.5 * available / (cf * cost));
        self.arena.reserve[pi] -= GramsC(mass * cf * cost);
        self.ledger.growth_respiration += GramsC(mass * cf * (cost - 1.0));
        let id = self.arena.add(
            &NewOrgan {
                kind: s.kind,
                parent: Some(s.parent),
                axis_order: s.order,
                birth: now,
                curve: s.curve,
                specific: s.specific,
                orientation: Orientation {
                    direction: geometry::unit(s.direction),
                    azimuth: ps_units::Radians(s.azimuth),
                },
                structural: GramsDM(mass),
                reserve: GramsC(0.0),
                nitrogen: GramsN(0.0),
            },
            &self.params.allometry,
        )?;
        let mut m = Meta::new(s.role);
        m.attach = s.attach;
        m.multiplicity = s.multiplicity;
        m.potential_area = s.potential_area;
        self.meta.push(m);
        self.feed_new_organ(id);
        Ok(id)
    }

    /// A random stream for organ `id` at `tick`.
    pub(crate) fn stream(&self, id: OrganId, tick: Hours, purpose: Purpose) -> ps_rng::Stream {
        let entity = (self.entity << 32) | u64::from(id.0);
        RngKey::new(self.world_seed, entity, tick.0, purpose).stream()
    }

    /// Whether the seedling has emerged.
    #[must_use]
    pub fn emerged(&self) -> bool {
        self.phenology.stage >= Stage::Emerged
    }

    fn soil_temperature(soil: &SoilGrid, z: f64) -> Celsius {
        let c = soil.cell_size().0;
        let (_, _, nz) = soil.dims();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let layer = (libm::floor((-z / c).max(0.0)) as usize).min(nz.saturating_sub(1));
        soil.layer_temperature(layer)
    }

    /// Hourly steps 3, 5 and 6: water status, light and gas exchange. Stores each
    /// leaf's carbon income for [`Plant::grow_hour`] and returns the plant's
    /// transpiration demand for the world's joint uptake (step 7).
    ///
    /// # Errors
    /// [`ArchError::Soil`] if the root list does not fit the soil.
    pub fn water_demand(
        &mut self,
        ctx: &HourCtx<'_>,
        soil: &SoilGrid,
    ) -> Result<KgWater, ArchError> {
        self.update_water_status(soil)?;
        let table = self.class_table(ctx);
        Ok(self.water_demand_with(ctx, &table))
    }

    /// The first part of [`Plant::water_demand`]: this hour's FTSW over the roots and
    /// the stress factors.
    ///
    /// # Errors
    /// [`ArchError::Soil`] if the root list does not fit the soil.
    pub fn update_water_status(&mut self, soil: &SoilGrid) -> Result<(), ArchError> {
        self.ftsw = if self.roots.cells().is_empty() {
            1.0
        } else {
            soil.ftsw(&self.roots)?
        };
        let hp = &self.params;
        self.stress = (
            stress_factor(self.ftsw, hp.ftsw_conductance),
            stress_factor(self.ftsw, hp.ftsw_expansion),
        );
        Ok(())
    }

    /// What this hour's class table depends on besides the hour's weather: the leaf
    /// parameters, the leaf energy parameters and the conductance stress factor, as
    /// bits. Plants of one soil with equal keys have identical tables (call after
    /// [`Plant::update_water_status`]).
    #[must_use]
    pub fn class_table_key(&self) -> TableKey {
        (
            self.params.leaf.bit_key(),
            self.params.energy.bit_key(),
            self.stress.0.to_bits(),
        )
    }

    /// This hour's class table (call after [`Plant::update_water_status`]): gas
    /// exchange and the leaf energy balance solved together at every node, for a
    /// horizontal leaf at [`REFERENCE_HEIGHT`] under the open sky in the hour's air and
    /// wind (WP27, decision 0034).
    #[must_use]
    pub fn class_table(&self, ctx: &HourCtx<'_>) -> ClassTable {
        let wx = ctx.weather;
        let reference = Reference {
            air: hour_air(ctx),
            ca: ctx.co2,
            wind: energy::wind_at(wx.wind.0, REFERENCE_HEIGHT),
        };
        // Without sun or sky light every leaf reads the PAR-0 node.
        let dark = wx.direct.0 <= 0.0 && wx.diffuse.0 <= 0.0;
        ClassTable::build_coupled(
            &self.params.leaf,
            &self.params.energy,
            &reference,
            self.stress.0,
            dark,
        )
        .0
    }

    /// The rest of [`Plant::water_demand`] with this hour's class table, which may be
    /// shared with other plants of equal [`Plant::class_table_key`].
    pub fn water_demand_with(&mut self, ctx: &HourCtx<'_>, table: &ClassTable) -> KgWater {
        let wx = ctx.weather;
        let hp = &self.params;
        self.income.clear();
        self.income.resize(self.arena.len(), GramsC(0.0));
        let leaves: Vec<OrganId> = self
            .arena
            .alive_of(OrganKind::Leaf)
            .filter(|id| self.arena.size[id.index()].area.0 > 0.0)
            .collect();
        let mut water = 0.0;
        if !leaves.is_empty() {
            let sin_e = if wx.sun.is_up() {
                wx.sun.direction()[2]
            } else {
                0.0
            };
            let air = hour_air(ctx).terms();
            for id in leaves {
                let i = id.index();
                let a = ps_light::absorbed_par(
                    &self.arena.light[i],
                    ctx.hour,
                    wx.direct,
                    wx.diffuse,
                    sin_e,
                    hp.absorptance,
                );
                let age = self.arena.thermal_age[i];
                let w = self.arena.structural(id).0;
                let conc = if w > 0.0 {
                    self.arena.reserve[i].0 / w
                } else {
                    0.0
                };
                // Aging scales Vcmax, Jmax and Rd together (the table's capacity axis);
                // leaf lifespan counts from full expansion.
                let expanded = self.meta[i].expanded.unwrap_or(age);
                let area = self.arena.size[i].area.0;
                // Leaf nitrogen per area scales capacity too (decision 0020).
                let cap = leaf_capacity(
                    age,
                    expanded,
                    expanded + hp.leaf_lifespan,
                    hp.capacity_floor,
                ) * hp.nitrogen.leaf_capacity(self.arena.nitrogen[i].0 / area);
                let sun = table.lookup(a.sunlit, cap);
                let shade = table.lookup(a.shaded, cap);
                let f = a.sunlit_fraction;
                let a_net = f * sun.a_net.0 + (1.0 - f) * shade.a_net.0;
                let rd = f * sun.rd.0 + (1.0 - f) * shade.rd.0;
                // Sink feedback (decision 0013, amended in WP9 review) acts on gross
                // assimilation only, so a sugar-filled leaf still pays its respiration;
                // conductance and transpiration follow assimilation.
                let fb = hp.sink_feedback(conc);
                let gross = (a_net + rd).max(0.0);
                let a_net = fb * gross - rd;
                // Each part's own temperature and transpiration (decision 0034): its
                // light, its sky view, the wind at its height inside the plant, and
                // the conductance it read, which follows assimilation.
                let ratio = if gross > 0.0 {
                    (a_net + rd) / gross
                } else {
                    1.0
                };
                let light = &self.arena.light[i];
                let z = self
                    .positions
                    .get(i)
                    .map_or(0.0, |p| 0.5 * (p.0[2] + p.1[2]));
                let part = |par: UmolPerM2S, gs: f64| {
                    let site = Site {
                        par,
                        sky_view: light.diffuse,
                        wind: energy::leaf_wind(wx.wind.0, z, light.diffuse),
                    };
                    energy::leaf_temperature(&hp.energy, &air, &site, gs * ratio)
                };
                let shaded = part(a.shaded, shade.gs.0);
                let (t_leaf, e) = if f > 0.0 {
                    let sunlit = part(a.sunlit, sun.gs.0);
                    (
                        f * sunlit.t_leaf.0 + (1.0 - f) * shaded.t_leaf.0,
                        f * sunlit.e.0 + (1.0 - f) * shaded.e.0,
                    )
                } else {
                    (shaded.t_leaf.0, shaded.e.0)
                };
                // Dew on a leaf does not water the plant.
                let e = e.max(0.0);
                let m = &mut self.meta[i];
                m.t_leaf = Some(Celsius(t_leaf));
                m.t_leaf_min = Some(Celsius(m.t_leaf_min.map_or(t_leaf, |t| t.0.min(t_leaf))));
                m.par_today.0 += a.mean().0 * 3600.0 * 1e-6;
                m.par_hours += 1;
                self.today.gross_potential += gross * area;
                self.today.gross_lost += (1.0 - fb) * gross * area;
                self.income[i] = UmolPerM2S(a_net).carbon_per_hour(ps_units::M2(area));
                self.today.assimilation += self.income[i].0;
                water += e * area * 3600.0 * KG_PER_MOL_WATER;
            }
        }
        KgWater(water)
    }

    /// Light on the plant this hour, for the competition lens (WP26): the leaf area
    /// and its area-weighted mean absorbed PAR, read from the leaves' light
    /// coefficients exactly as [`Plant::water_demand_with`] reads them. Pure; call after
    /// the day's light rebuild.
    #[must_use]
    pub fn absorbed_par(&self, ctx: &HourCtx<'_>) -> (ps_units::M2, UmolPerM2S) {
        let wx = ctx.weather;
        let sin_e = if wx.sun.is_up() {
            wx.sun.direction()[2]
        } else {
            0.0
        };
        let (mut area, mut sum) = (0.0, 0.0);
        for id in self.arena.alive_of(OrganKind::Leaf) {
            let i = id.index();
            let a_leaf = self.arena.size[i].area.0;
            if a_leaf <= 0.0 {
                continue;
            }
            let a = ps_light::absorbed_par(
                &self.arena.light[i],
                ctx.hour,
                wx.direct,
                wx.diffuse,
                sin_e,
                self.params.absorptance,
            );
            area += a_leaf;
            sum += a.mean().0 * a_leaf;
        }
        let mean = if area > 0.0 { sum / area } else { 0.0 };
        (ps_units::M2(area), UmolPerM2S(mean))
    }

    /// PAR a horizontal, unshaded leaf of this plant would absorb under this hour's
    /// open sky (the competition lens's reference, WP26).
    #[must_use]
    pub fn open_sky_par(&self, ctx: &HourCtx<'_>) -> UmolPerM2S {
        let wx = ctx.weather;
        let to_par = ps_light::PAR_FRACTION * ps_light::UMOL_PER_J_PAR * self.params.absorptance;
        UmolPerM2S((wx.direct.0.max(0.0) + wx.diffuse.0.max(0.0)) * to_par)
    }

    /// Each organ's nitrogen per unit dry mass, g N g⁻¹, indexed by organ id (zero for
    /// dead or massless organs), for the nitrogen lens (WP26).
    #[must_use]
    pub fn organ_n_concentration(&self) -> Vec<f64> {
        (0..self.arena.len())
            .map(|i| {
                let dm = self.arena.structural_all()[i].0;
                if self.arena.alive[i] && dm > 0.0 {
                    self.arena.nitrogen[i].0 / dm
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// The plant's roots, for the world's joint uptake.
    #[must_use]
    pub fn uptake_request(&self, demand: KgWater) -> ps_soil::PlantUptake<'_> {
        ps_soil::PlantUptake {
            demand,
            roots: &self.roots,
            rate: self.params.uptake_rate,
            max_per_length: self.params.uptake_max,
        }
    }

    /// Hourly steps 8 to 12 (allocation), plus germination and thermal time. `uptake`
    /// is this plant's share of the joint uptake, for the shortfall log.
    ///
    /// # Errors
    /// [`ArchError::Physio`] or [`ArchError::Graph`] on an internal inconsistency.
    pub fn grow_hour(
        &mut self,
        ctx: &HourCtx<'_>,
        soil: &SoilGrid,
        uptake: Option<&Uptake>,
    ) -> Result<HourReport, ArchError> {
        self.grow_hour_recording(ctx, soil, uptake, None)
    }

    /// [`Plant::grow_hour`], also filling `lens` with the hour's organ temperatures and
    /// phloem fluxes (WP26). Recording only copies values the step computes, so the
    /// plant ends bit for bit as after [`Plant::grow_hour`].
    ///
    /// # Errors
    /// As [`Plant::grow_hour`].
    pub fn grow_hour_recording(
        &mut self,
        ctx: &HourCtx<'_>,
        soil: &SoilGrid,
        uptake: Option<&Uptake>,
        lens: Option<&mut HourLens>,
    ) -> Result<HourReport, ArchError> {
        let wx = ctx.weather;
        if let Some(u) = uptake {
            self.shortfall += u.shortfall;
            self.today.transpiration += u.actual.0;
        }
        let seed_t = Self::soil_temperature(soil, self.collar_position[2]);
        // Germination (§3.10): hours with the seed cell above the FTSW threshold and
        // warmer than the base temperature. A dry or cold hour pauses the count and
        // does not reset it (decision 0029): an imbibed seed keeps its progress through
        // a dry afternoon (seed hydration memory), so a shallow seed in a top cell that
        // dries each afternoon still comes up.
        if self.phenology.stage == Stage::Sown {
            let ok = geometry::soil_cell(soil, self.collar_position).is_some_and(|c| {
                soil.cell_ftsw(c) >= self.params.germination_ftsw
                    && seed_t.0 > self.params.temperature.cardinal.base.0
            });
            if ok {
                self.germination_hours += 1;
            }
        }
        let n = self.arena.len();
        self.income.resize(n, GramsC(0.0));
        let temps: Vec<Celsius> = (0..n)
            .map(|i| match self.arena.kind[i] {
                OrganKind::Root | OrganKind::Seed => {
                    let z = self
                        .positions
                        .get(i)
                        .map_or(self.collar_position[2], |p| p.0[2]);
                    Self::soil_temperature(soil, z)
                }
                _ => wx.t_air,
            })
            .collect();
        // Nodules pay for the hour's fixation first (decision 0020).
        self.fix_hour();
        let respiring = self.phenology.stage != Stage::Mature;
        let (flows, lens_temps) = match lens {
            Some(l) => (Some(&mut l.phloem_import), Some(&mut l.temperature)),
            None => (None, None),
        };
        let rep = allocation::hourly_recording(
            &mut self.arena,
            &self.params.allometry,
            &self.params.carbon,
            &self.params.temperature,
            &HourInputs {
                temperature: &temps,
                income: &self.income,
                ftsw: self.ftsw,
                stress: self.stress.1 * self.nitrogen.status,
                respiring,
                n_floor: Some(&self.params.nitrogen.growth_floor),
            },
            &mut self.ledger,
            flows,
        )?;
        if let Some(t) = lens_temps {
            t.clone_from(&temps);
            // Leaves show their own temperature (WP27); their expansion reads air.
            for id in self.arena.alive_of(OrganKind::Leaf) {
                if let Some(tl) = self.meta[id.index()].t_leaf {
                    t[id.index()] = tl;
                }
            }
        }
        // Growing organs draw nitrogen for the hour's growth from the pool.
        self.top_up_hour();
        // Plant thermal time: soil at seed depth until emergence, then air.
        let t_plant = if self.emerged() { wx.t_air } else { seed_t };
        let dt = self.params.temperature.cardinal.hourly(t_plant);
        self.phenology.accumulate(dt);
        let emerged = self.emerged();
        let d = &mut self.today;
        d.hours += 1;
        d.t_sum += wx.t_air.0;
        d.t_min = d.t_min.min(wx.t_air.0);
        d.t_max = d.t_max.max(wx.t_air.0);
        d.ftsw_min = d.ftsw_min.min(self.ftsw);
        if emerged {
            d.thermal += self.params.temperature.cardinal.hourly(wx.t_air).0;
        }
        self.income.iter_mut().for_each(|c| *c = GramsC(0.0));
        Ok(rep)
    }

    /// Daily steps 1 to 6 of §3.2 for this plant. `sun_path` is the next day's sun
    /// direction for each clock hour (`None` below the horizon), for the light rebuild,
    /// which uses this plant's own grid (a pot).
    ///
    /// # Errors
    /// Any [`ArchError`] from an internal inconsistency.
    pub fn day(
        &mut self,
        soil: &SoilGrid,
        sun_path: &[Option<V3>; 24],
        now: Hours,
    ) -> Result<DayReport, ArchError> {
        self.day_before_light(soil, now)?;
        Ok(self.day_after_light(sun_path, None))
    }

    /// The daily step up to the light rebuild: phenology, buds, organ initiation,
    /// senescence, root growth, then the layout and root cells. A plot calls this for
    /// every plant, builds one light grid from their [`Plant::light_inputs`], and hands
    /// each plant its leaves' coefficients with [`Plant::day_after_light`] (decision
    /// 0019).
    ///
    /// # Errors
    /// Any [`ArchError`] from an internal inconsistency.
    pub fn day_before_light(&mut self, soil: &SoilGrid, now: Hours) -> Result<(), ArchError> {
        self.day += 1;
        // 1. Phenology.
        let events = self.events();
        let before = self.phenology.stage;
        self.phenology.advance(&self.params.thresholds, events);
        let stage = self.phenology.stage;
        if before == Stage::Sown && stage == Stage::Germinated {
            self.germinate()?;
        }
        if before == Stage::Germinated && stage == Stage::Emerged {
            if self.is_grass() {
                self.grass_emerge(now)?;
            } else {
                self.emerge(now)?;
            }
        }
        if stage < Stage::Mature {
            if self.is_grass() {
                // 2. Tiller sites; 3. leaves, ears and grains (WP23).
                self.tillering(now)?;
                self.grow_tillers(now)?;
                self.flowering(now)?;
            } else {
                // 2. Bud fate.
                self.bud_fate(now)?;
                // 3. Organ initiation.
                self.grow_axes(now)?;
                self.start_racemes(now)?;
                self.flowering(now)?;
            }
            self.root_initiation(now, soil)?;
        }
        // 4. Senescence and shedding.
        self.senescence(now)?;
        self.rot_pods(now)?;
        // 5. Root elongation and branching.
        if stage < Stage::Mature {
            self.root_elongation(now, soil)?;
        }
        // 6. Light geometry, root cells, nitrogen, expansion records.
        self.positions = geometry::layout(&self.arena, &self.meta, self.collar_position);
        self.roots = geometry::root_cells(&self.arena, &self.positions, soil);
        Ok(())
    }

    /// The light model's input for this plant: its leaflet discs (see
    /// [`Plant::light_points`]) and the internode tips that bound the shoot, in the
    /// soil frame.
    #[must_use]
    pub fn light_inputs(&self) -> (Vec<ps_light::LeafPoint>, Vec<V3>) {
        (self.light_points(), self.internode_tips())
    }

    fn internode_tips(&self) -> Vec<V3> {
        self.arena
            .alive_of(OrganKind::Internode)
            .map(|id| self.positions[id.index()].1)
            .collect()
    }

    /// The rest of the daily step after [`Plant::day_before_light`]: the light rebuild,
    /// nitrogen and the day's report. `coeffs` holds one entry per leaflet of
    /// [`Plant::light_inputs`] from a shared grid, or `None` to build this plant's own.
    pub fn day_after_light(
        &mut self,
        sun_path: &[Option<V3>; 24],
        coeffs: Option<&[LightCoeffs]>,
    ) -> DayReport {
        self.rebuild_light(sun_path, coeffs);
        self.mark_expanded();
        let rep = self.report();
        self.close_nitrogen_day();
        self.today = DayStats::default();
        self.shortfall = KgWater(0.0);
        self.income.clear();
        rep
    }

    fn events(&self) -> Events {
        let a = &self.arena;
        Events {
            germinated: self.germination_hours >= self.params.germination_hours,
            // A grass's second leaf stands in for the first trifoliate.
            trifoliate: a
                .alive_ids()
                .filter(|id| {
                    let i = id.index();
                    matches!(self.meta[i].role, Role::Trifoliate | Role::Blade)
                        && a.expansion_age[i].0 >= 0.5 * a.curve[i].duration.0
                })
                .nth(usize::from(self.is_grass()))
                .is_some(),
            pod: a.alive_of(OrganKind::Pod).next().is_some(),
            seed_filling: a.alive_of(OrganKind::SeedInPod).next().is_some(),
        }
    }

    /// The leaflet discs the light model sees: center, area and normal of every leaflet
    /// of every living leaf, in organ order (§3.5). Used to export ray tracer fixtures.
    #[must_use]
    pub fn light_points(&self) -> Vec<ps_light::LeafPoint> {
        self.leaflets().0
    }

    /// Leaflet points and, for each, the index of the leaf it belongs to.
    fn leaflets(&self) -> (Vec<ps_light::LeafPoint>, Vec<usize>) {
        let mut points = Vec::new();
        let mut owner = Vec::new();
        let aspect = self.params.grass.as_ref().map(|g| g.blade_aspect);
        for id in self.arena.alive_of(OrganKind::Leaf) {
            let i = id.index();
            if let Some(aspect) = aspect {
                let pts = grass::blade_points(
                    self.arena.size[i].area.0,
                    self.positions[i].0,
                    self.arena.orientation[i].direction,
                    aspect,
                );
                owner.extend(std::iter::repeat_n(i, pts.len()));
                points.extend(pts);
                continue;
            }
            let pts = geometry::leaf_points(
                &self.arena,
                &self.meta,
                id,
                self.positions[i].0,
                self.params.petiole,
                ps_units::M2(self.meta[i].potential_area),
            );
            owner.extend(std::iter::repeat_n(i, pts.len()));
            points.extend(pts);
        }
        (points, owner)
    }

    fn rebuild_light(&mut self, sun_path: &[Option<V3>; 24], shared: Option<&[LightCoeffs]>) {
        // Absorbed PAR, for low-light acclimation of new organs: a running mean that
        // gives yesterday a weight of 1/3, so one dull day does not set a leaf's
        // thickness (decision 0018).
        for id in self.arena.alive_of(OrganKind::Leaf).collect::<Vec<_>>() {
            let m = &mut self.meta[id.index()];
            if m.par_hours >= 24 {
                let y = m.par_today.0;
                m.dli = Some(MolPerM2Day(
                    m.dli.map_or(y, |old| old.0 + (y - old.0) / DLI_MEMORY_DAYS),
                ));
            }
            m.par_today = MolPerM2Day(0.0);
            m.par_hours = 0;
            m.t_leaf_min = None;
        }
        if self.sky.is_empty() {
            self.sky = ps_light::sky_sectors().to_vec();
        }
        let leaves: Vec<OrganId> = self.arena.alive_of(OrganKind::Leaf).collect();
        let (points, owner) = self.leaflets();
        let coeffs = if let Some(c) = shared {
            debug_assert_eq!(c.len(), points.len(), "one coefficient per leaflet");
            c.to_vec()
        } else {
            let extra = self.internode_tips();
            ps_light::rebuild_daily_with(&points, &extra, &self.obstacles, sun_path, &self.sky).1
        };
        let mut sums: Vec<(LightCoeffs, f64)> = vec![(zero_coeffs(), 0.0); self.arena.len()];
        for (c, &i) in coeffs.iter().zip(&owner) {
            let s = &mut sums[i];
            for h in 0..24 {
                s.0.direct[h] += c.direct[h];
                // Sunlit-weighted, so the leaf's mean absorbed beam is exact.
                s.0.beam[h] += c.direct[h] * c.beam[h];
            }
            s.0.diffuse += c.diffuse;
            s.0.diffuse_absorbed += c.diffuse_absorbed;
            s.0.rfr += c.rfr;
            s.1 += 1.0;
        }
        let up: Vec<usize> = (0..24).filter(|h| sun_path[*h].is_some()).collect();
        for &id in &leaves {
            let i = id.index();
            let (mut c, n) = sums[i];
            if n == 0.0 {
                continue;
            }
            for h in 0..24 {
                c.beam[h] = if c.direct[h] > 0.0 {
                    c.beam[h] / c.direct[h]
                } else {
                    0.0
                };
                c.direct[h] /= n;
            }
            c.diffuse /= n;
            c.diffuse_absorbed /= n;
            c.rfr /= n;
            let direct_mean = if up.is_empty() {
                0.0
            } else {
                up.iter().map(|h| c.direct[*h]).sum::<f64>()
                    / f64::from(u32::try_from(up.len()).unwrap_or(1))
            };
            self.meta[i].light_index = 0.5 * (direct_mean + c.diffuse);
            self.arena.light[i] = c;
        }
    }

    /// Records the thermal age at which each organ finished expanding.
    fn mark_expanded(&mut self) {
        let ids: Vec<OrganId> = self.arena.alive_ids().collect();
        for id in ids {
            let i = id.index();
            let c = &self.arena.curve[i];
            if self.meta[i].expanded.is_none()
                && c.potential.0 > 0.0
                && c.is_complete(self.arena.expansion_age[i])
            {
                self.meta[i].expanded = Some(self.arena.thermal_age[i]);
            }
        }
    }

    /// Living leaf area, m².
    #[must_use]
    pub fn leaf_area(&self) -> f64 {
        self.arena
            .alive_of(OrganKind::Leaf)
            .map(|id| self.arena.size[id.index()].area.0)
            .sum()
    }

    /// Whether [`Plant::prune`] would accept `id` now.
    ///
    /// # Errors
    /// As [`Plant::prune`].
    pub fn check_prune(&self, id: OrganId) -> Result<(), ArchError> {
        if id.index() >= self.arena.len() {
            return Err(GraphError::NoSuchOrgan(id).into());
        }
        if !self.arena.alive[id.index()] {
            return Err(GraphError::Dead(id).into());
        }
        if id == self.collar || id == self.hypocotyl {
            return Err(ArchError::NotPrunable(id));
        }
        Ok(())
    }

    /// Cuts an organ and everything it carries off the plant; the material leaves the
    /// plant (`allocation::cut`, ledger `removed`).
    ///
    /// # Errors
    /// [`ArchError::Graph`] for a missing or dead organ, [`ArchError::NotPrunable`] for
    /// the collar or hypocotyl.
    pub fn prune(&mut self, id: OrganId, now: Hours) -> Result<(), ArchError> {
        self.check_prune(id)?;
        let alive: Vec<bool> = self.arena.alive.clone();
        allocation::cut(
            &mut self.arena,
            &self.params.carbon,
            id,
            now,
            &mut self.ledger,
        )?;
        // Everything cut leaves with its nitrogen.
        for (i, was) in alive.into_iter().enumerate() {
            if was && !self.arena.alive[i] {
                self.nitrogen.ledger.removed += self.arena.nitrogen[i];
                self.arena.nitrogen[i] = GramsN(0.0);
            }
        }
        self.assert_nitrogen();
        Ok(())
    }

    /// Checks that `id` is a living pod or ear (see [`Plant::harvest`]).
    ///
    /// # Errors
    /// As [`Plant::harvest`].
    pub fn check_harvest(&self, id: OrganId) -> Result<(), ArchError> {
        self.check_prune(id)?;
        if self.arena.kind[id.index()] != OrganKind::Pod {
            return Err(ArchError::NotHarvestable(id));
        }
        Ok(())
    }

    /// Picks a pod or ear: it leaves the plant with its seeds, as a prune does, and the
    /// seeds are counted for the seed library (WP25).
    ///
    /// # Errors
    /// [`ArchError::Graph`] for a missing or dead organ, [`ArchError::NotHarvestable`]
    /// for anything but a pod or an ear.
    pub fn harvest(&mut self, id: OrganId, now: Hours) -> Result<Harvested, ArchError> {
        self.check_harvest(id)?;
        let a = &self.arena;
        let seeds: Vec<OrganId> = a.children[id.index()]
            .iter()
            .copied()
            .filter(|c| a.alive[c.index()] && a.kind[c.index()] == OrganKind::SeedInPod)
            .collect();
        let seed_mass = seeds.iter().map(|&c| a.structural(c).0).sum();
        let out = Harvested {
            seeds: u32::try_from(seeds.len()).unwrap_or(u32::MAX),
            seed_mass: GramsDM(seed_mass),
            mature: self.phenology.stage >= Stage::Mature,
        };
        self.prune(id, now)?;
        Ok(out)
    }

    /// Counts and totals now.
    #[must_use]
    pub fn report(&self) -> DayReport {
        let a = &self.arena;
        let alive = |k: OrganKind| a.alive_of(k);
        let mass = |k: OrganKind| a.total_structural(k).0;
        let height = a
            .alive_ids()
            .filter(|id| a.kind[id.index()] != OrganKind::Root)
            .filter_map(|id| self.positions.get(id.index()))
            .map(|p| p.1[2].max(p.0[2]))
            .fold(0.0, f64::max);
        let root_length: f64 = alive(OrganKind::Root)
            .map(|id| a.size[id.index()].length.0)
            .sum();
        DayReport {
            day: self.day,
            stage: Some(self.phenology.stage),
            main_nodes: self.axes.first().map_or(0, |x| x.nodes),
            branches: if self.is_grass() {
                self.living_tillers()
            } else {
                u32::try_from(self.axes.iter().filter(|x| x.order > 0).count()).unwrap_or(0)
            },
            leaves: u32::try_from(alive(OrganKind::Leaf).count()).unwrap_or(0),
            leaf_area: alive(OrganKind::Leaf)
                .map(|id| a.size[id.index()].area.0)
                .sum(),
            racemes: u32::try_from(self.racemes.len()).unwrap_or(0),
            flowers: self.flowers_opened,
            pods: u32::try_from(alive(OrganKind::Pod).count()).unwrap_or(0),
            seeds: u32::try_from(alive(OrganKind::SeedInPod).count()).unwrap_or(0),
            root_axes: u32::try_from(
                self.root_axes
                    .iter()
                    .filter(|x| self.params.roots[x.kind].order == 0)
                    .count(),
            )
            .unwrap_or(0),
            lateral_cohorts: u32::try_from(
                self.root_axes
                    .iter()
                    .filter(|x| self.params.roots[x.kind].order > 0)
                    .count(),
            )
            .unwrap_or(0),
            root_length,
            root_mass: mass(OrganKind::Root),
            dry_mass: a.alive_ids().map(|id| a.structural(id).0).sum(),
            total_dry_mass: a.ids().map(|id| a.structural(id).0).sum(),
            feedback_loss: if self.today.gross_potential > 0.0 {
                self.today.gross_lost / self.today.gross_potential
            } else {
                0.0
            },
            fruit_mass: mass(OrganKind::Pod) + mass(OrganKind::SeedInPod),
            leaf_mass: mass(OrganKind::Leaf),
            stem_mass: mass(OrganKind::Internode),
            seed_mass: mass(OrganKind::SeedInPod),
            height,
            shortfall: self.shortfall.0,
            nitrogen: self.nitrogen_stock().0,
            n_uptake: self.nitrogen.today.uptake.0,
            n_fixed: self.nitrogen.today.fixed.0,
            n_mined: self.nitrogen.today.mined.0,
            n_unmet: self.nitrogen.today.unmet.0,
            n_status: self.nitrogen.status,
            nodule_mass: mass(OrganKind::Nodule),
            fixation_carbon: self.nitrogen.today.fixation_carbon.0,
            assimilation: self.today.assimilation,
            transpiration: self.today.transpiration,
        }
    }
}

fn zero_coeffs() -> LightCoeffs {
    LightCoeffs {
        rfr: 0.0,
        ..LightCoeffs::default()
    }
}

#[cfg(test)]
mod tests;
