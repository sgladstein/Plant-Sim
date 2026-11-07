//! Scenario files (design plan §6.2) and the action log (§6.3).

use std::collections::BTreeMap;

use ps_arch::HydraulicsLevel;
use ps_soil::{NitrogenForm, SoilNitrogen, Texture};
use ps_weather::presets::HeatWave;
use serde::{Deserialize, Serialize};

use crate::WorldError;

/// The scenario schema version this build reads.
pub const SCENARIO_SCHEMA: u32 = 1;
/// Smallest side of a plot, cm (§7 WP21: 1 × 1 to 3 × 3 m).
pub const PLOT_MIN_CM: f64 = 100.0;
/// Largest side of a plot, cm.
pub const PLOT_MAX_CM: f64 = 300.0;

/// Start date and hour on the sim clock (local mean solar time, decision 0005).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Start {
    /// Year.
    pub year: i32,
    /// Month, 1 to 12.
    pub month: u32,
    /// Day of month.
    pub day: u32,
    /// Hour, 0 to 23.
    pub hour: u32,
}

/// Where the weather comes from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WeatherSource {
    /// A built-in preset by name, generated from the world seed.
    Preset(String),
    /// An hourly CSV file, by path relative to the data directory.
    File(String),
    /// A daily CSV file, disaggregated to hours.
    DailyFile(String),
}

/// How a pot drains (R5a, decision 0046).
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum PotDrainage {
    /// Holes in the bottom: water leaves only while the soil at the base is saturated,
    /// so a soaked pot keeps a wet layer at its bottom.
    #[default]
    Holes,
    /// No holes: no water leaves.
    NoHoles,
    /// Holes, and the pot stands in a saucer that catches what drains and feeds it
    /// back while it holds water.
    Saucer {
        /// The saucer's depth, mm (0 to 100).
        depth_mm: f64,
    },
}

impl PotDrainage {
    /// The soil grid's drainage.
    #[must_use]
    pub fn grid(self) -> ps_soil::Drainage {
        match self {
            Self::Holes => ps_soil::Drainage::Holes,
            Self::NoHoles => ps_soil::Drainage::NoHoles,
            Self::Saucer { depth_mm } => ps_soil::Drainage::Saucer {
                depth: ps_units::Millimeters(depth_mm),
            },
        }
    }
}

/// A soil container.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SoilSpec {
    /// A cylindrical pot.
    Pot {
        /// Soil id, unique in the scenario.
        id: u32,
        /// Diameter, cm.
        diameter_cm: f64,
        /// Depth, cm.
        depth_cm: f64,
        /// Texture class.
        texture: Texture,
        /// Pot center in the garden, m (east, north); used from M1.
        position_m: (f64, f64),
        /// How the pot drains (R5a, decision 0046): holes by default.
        #[serde(default)]
        drainage: PotDrainage,
    },
    /// A rectangular plot bed (M2, WP21): 5 cm cells, bottom draining to a deep layer
    /// that never returns water, and one light grid for all its plants (decision 0019).
    Plot {
        /// Soil id, unique in the scenario.
        id: u32,
        /// East-west size, cm (100 to 300).
        width_cm: f64,
        /// North-south size, cm (100 to 300).
        length_cm: f64,
        /// Depth, cm.
        depth_cm: f64,
        /// Texture class.
        texture: Texture,
        /// Plot center in the garden, m (east, north).
        position_m: (f64, f64),
    },
}

impl SoilSpec {
    /// The soil id.
    #[must_use]
    pub fn id(&self) -> u32 {
        match self {
            Self::Pot { id, .. } | Self::Plot { id, .. } => *id,
        }
    }

    /// The container's center in the garden, m (east, north).
    #[must_use]
    pub fn position_m(&self) -> (f64, f64) {
        match self {
            Self::Pot { position_m, .. } | Self::Plot { position_m, .. } => *position_m,
        }
    }

    /// Whether the container's plants share one light grid (a plot).
    #[must_use]
    pub fn shared_light(&self) -> bool {
        matches!(self, Self::Plot { .. })
    }
}

/// How a soil is watered automatically.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Irrigation {
    /// No automatic watering.
    None,
    /// `mm` of water at 06:00 every `every_days` days, counted from day 0.
    Schedule {
        /// Interval in days (at least 1).
        every_days: u32,
        /// Water per event, mm over the soil surface.
        mm: f64,
    },
    /// At 06:00 every `every_days` days, enough water to bring every cell to field
    /// capacity (a gardener watering until the pot drains).
    ToFieldCapacity {
        /// Interval in days (at least 1).
        every_days: u32,
    },
    /// As [`Irrigation::ToFieldCapacity`], for a strip of columns only: those whose
    /// centers lie from `east_cm.0` to `east_cm.1` east of the soil's center (a split
    /// pot's watered half, R2b). Water lands on the strip alone.
    ToFieldCapacityStrip {
        /// Interval in days (at least 1).
        every_days: u32,
        /// West and east edge of the strip, cm east of the soil's center.
        east_cm: (f64, f64),
    },
}

/// A cube of soil whose starting nitrate is multiplied by `no3_factor` (a nitrate
/// patch, R2b, decision 0043): every cell whose center lies inside it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NitratePatch {
    /// Soil id.
    pub soil: u32,
    /// The cube's center: cm east and north of the soil's center, cm below the surface.
    pub center_cm: (f64, f64, f64),
    /// The cube's edge, cm.
    pub size_cm: f64,
    /// Factor on the cells' starting nitrate (1 leaves it, for a control that only
    /// marks the region).
    pub no3_factor: f64,
}

/// Which genotype a sown seed has.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SeedChoice {
    /// The species-default genotype.
    Default,
    /// A genotype drawn from the species population (world seed, plant id).
    Sample,
    /// A seed saved in the seed library (WP30, decision 0038): its library id, kept on
    /// the plant so its seeds can name it as a parent, and its genotype.
    Saved {
        /// Library id.
        id: u64,
        /// Its genotype, in the species' locus order.
        genotype: ps_species::Genotype,
    },
}

/// A vertical support for climbing plants: a stake, pole or trellis string (WP22).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupportSpec {
    /// Soil id it stands in.
    pub soil: u32,
    /// Position from the soil's center, cm (east, north).
    pub offset_cm: (f64, f64),
    /// Height above the soil surface, cm.
    pub height_cm: f64,
    /// Diameter, cm.
    pub diameter_cm: f64,
}

/// A box that shades the soil's plants: a wall, fence, shade cloth or bucket (WP25).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObstacleSpec {
    /// Soil id whose light it shades.
    pub soil: u32,
    /// Lower corner from the soil's center and surface, cm (east, north, up).
    pub min_cm: (f64, f64, f64),
    /// Upper corner, cm.
    pub max_cm: (f64, f64, f64),
    /// Share of light passing straight through its thinnest side, 0 (opaque) to 1.
    pub transmittance: f64,
}

/// Largest obstacle extent along any axis, cm.
pub const MAX_OBSTACLE_CM: f64 = 1000.0;

impl ObstacleSpec {
    /// Checks the box is finite, not inverted, at most [`MAX_OBSTACLE_CM`] on a side,
    /// and its transmittance is 0 to 1.
    ///
    /// # Errors
    /// The reason, for the scenario or action error.
    pub fn check(&self) -> Result<(), String> {
        let lo = [self.min_cm.0, self.min_cm.1, self.min_cm.2];
        let hi = [self.max_cm.0, self.max_cm.1, self.max_cm.2];
        let fine = lo.iter().chain(&hi).all(|v| v.is_finite())
            && lo
                .iter()
                .zip(&hi)
                .all(|(a, b)| b > a && b - a <= MAX_OBSTACLE_CM)
            && (0.0..=1.0).contains(&self.transmittance);
        if fine {
            Ok(())
        } else {
            Err(format!(
                "obstacle {self:?}: corners finite, max above min, sides up to {MAX_OBSTACLE_CM} cm, transmittance 0 to 1"
            ))
        }
    }
}

impl SupportSpec {
    /// Checks height and diameter are in range and the position is finite.
    ///
    /// # Errors
    /// The reason, for the scenario or action error.
    pub fn check(&self) -> Result<(), String> {
        let fine = |v: f64, hi: f64| v.is_finite() && v > 0.0 && v <= hi;
        let (x, y) = self.offset_cm;
        if fine(self.height_cm, MAX_SUPPORT_HEIGHT_CM)
            && fine(self.diameter_cm, MAX_SUPPORT_DIAMETER_CM)
            && x.is_finite()
            && y.is_finite()
        {
            Ok(())
        } else {
            Err(format!(
                "support {self:?}: height up to {MAX_SUPPORT_HEIGHT_CM} cm, diameter up to {MAX_SUPPORT_DIAMETER_CM} cm"
            ))
        }
    }
}

/// Tallest support a scenario may stand, cm.
pub const MAX_SUPPORT_HEIGHT_CM: f64 = 500.0;
/// Thickest support a scenario may stand, cm.
pub const MAX_SUPPORT_DIAMETER_CM: f64 = 100.0;

/// A seed sown at the start.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sowing {
    /// Soil id.
    pub soil: u32,
    /// Species id (file stem under data/species).
    pub species: String,
    /// Genotype.
    pub seed: SeedChoice,
    /// Sowing depth, cm.
    pub depth_cm: f64,
    /// Position in the pot from its center, cm (east, north); the center if absent.
    #[serde(default)]
    pub offset_cm: Option<(f64, f64)>,
}

/// Variation the gardener and the ground bring to a scenario (decision 0027): hand
/// sowing scatters each seed about its mark, and a field soil is patchy. Every draw is
/// keyed by (world seed, plant or soil, tick, purpose). Absent from a scenario file,
/// the defaults below apply; [`FieldVariation::NONE`] turns it all off.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FieldVariation {
    /// Standard deviation of a seed's position about its mark along each axis, cm.
    pub sowing_position_sd_cm: f64,
    /// Standard deviation of a seed's sowing depth, cm.
    pub sowing_depth_sd_cm: f64,
    /// Coefficient of variation of a plot's starting soil nitrogen (nitrate, ammonium
    /// and humus) between soil columns. Pots hold mixed soil and stay uniform.
    pub soil_n_cv: f64,
    /// Spacing of the lattice the soil field is drawn on and interpolated from, cm:
    /// roughly the size of a soil patch.
    pub soil_patch_cm: f64,
}

impl FieldVariation {
    /// No sowing scatter and a uniform soil: every seed exactly on its mark.
    pub const NONE: Self = Self {
        sowing_position_sd_cm: 0.0,
        sowing_depth_sd_cm: 0.0,
        soil_n_cv: 0.0,
        soil_patch_cm: 30.0,
    };

    /// Hand sowing in a garden bed ("~", decision 0027): seeds dibbed at a mark land
    /// within about 2 cm of it and 1 cm of the intended depth (two standard
    /// deviations), and
    /// mineral and humus nitrogen vary about 25% between patches some 30 cm across.
    /// Plant-scale soil nitrate is commonly that patchy or more, with coefficients of
    /// variation of 30 to 60% in field surveys; 25% keeps the mean garden soil.
    pub const GARDEN: Self = Self {
        sowing_position_sd_cm: 1.0,
        sowing_depth_sd_cm: 0.5,
        soil_n_cv: 0.25,
        soil_patch_cm: 30.0,
    };

    fn check(&self) -> Result<(), String> {
        let ok = |v: f64, hi: f64| v.is_finite() && (0.0..=hi).contains(&v);
        if !ok(self.sowing_position_sd_cm, 10.0) {
            return Err("variation.sowing_position_sd_cm must lie within 0 to 10".into());
        }
        if !ok(self.sowing_depth_sd_cm, 5.0) {
            return Err("variation.sowing_depth_sd_cm must lie within 0 to 5".into());
        }
        if !ok(self.soil_n_cv, 1.0) {
            return Err("variation.soil_n_cv must lie within 0 to 1".into());
        }
        if !(self.soil_patch_cm.is_finite() && (5.0..=500.0).contains(&self.soil_patch_cm)) {
            return Err("variation.soil_patch_cm must lie within 5 to 500".into());
        }
        Ok(())
    }
}

impl Default for FieldVariation {
    fn default() -> Self {
        Self::GARDEN
    }
}

/// A scenario: everything needed to start a world.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    /// Schema version.
    pub schema: u32,
    /// Model version the scenario was written for.
    pub model_version: String,
    /// World seed.
    pub world_seed: u64,
    /// Site latitude, degrees north.
    pub latitude_deg: f64,
    /// Start date and hour.
    pub start: Start,
    /// Weather source.
    pub weather: WeatherSource,
    /// Soils.
    pub soils: Vec<SoilSpec>,
    /// Share of radiation reaching each soil and its plants (shade cloth); 1 if absent.
    #[serde(default)]
    pub radiation_fraction: BTreeMap<u32, f64>,
    /// Share of rain reaching each soil (a rain shelter sets 0); 1 if absent.
    #[serde(default)]
    pub rain_fraction: BTreeMap<u32, f64>,
    /// Starting nitrogen of each soil; a moderately fertile garden loam with rhizobia
    /// ([`SoilNitrogen::GARDEN`]) if absent (WP24, decision 0020).
    #[serde(default)]
    pub nitrogen: BTreeMap<u32, SoilNitrogen>,
    /// Automatic watering per soil; none if absent.
    #[serde(default)]
    pub irrigation: BTreeMap<u32, Irrigation>,
    /// Soils with a watertight divider across their center, north to south: the west
    /// and east halves exchange no water or nitrate (a split pot, R2b); none if absent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dividers: Vec<u32>,
    /// Nitrate patches set at the start (R2b); none if absent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nitrate_patches: Vec<NitratePatch>,
    /// Ambient CO2, ppm.
    pub co2_ppm: f64,
    /// Seeds sown at tick 0.
    pub sowings: Vec<Sowing>,
    /// Vertical supports for climbing plants; none if absent (WP22).
    #[serde(default)]
    pub supports: Vec<SupportSpec>,
    /// Boxes that shade the light grid; none if absent (WP25).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstacles: Vec<ObstacleSpec>,
    /// Length of the run, days.
    pub duration_days: u32,
    /// A hot spell replacing the preset's own (preset weather only).
    #[serde(default)]
    pub heat_wave: Option<HeatWave>,
    /// Scripted treatments, part of the scenario rather than the player's log (a
    /// drought from a set day, a shade cloth taken off). Sorted by tick; applied before
    /// log actions of the same tick.
    #[serde(default)]
    pub events: Vec<Action>,
    /// Sowing scatter and soil patchiness; [`FieldVariation::GARDEN`] if absent
    /// (decision 0027).
    #[serde(default)]
    pub variation: FieldVariation,
    /// Which form of the water network the plants run (R3c, decision 0045);
    /// [`Fidelity::Auto`] if absent.
    #[serde(default, skip_serializing_if = "Fidelity::is_auto")]
    pub fidelity: Fidelity,
}

/// Most plants a scenario at [`Fidelity::Auto`] runs at the detailed level (the
/// fidelity plan's `max_detailed`, a guess for Performance to time).
pub const MAX_DETAILED: usize = 8;

/// The level of the plants' water network a scenario asks for (R3c, decision 0045).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fidelity {
    /// Detailed when the scenario sows at most [`MAX_DETAILED`] seeds at the start,
    /// simplified above.
    #[default]
    Auto,
    /// Every plant detailed.
    Detailed,
    /// Every plant simplified.
    Simplified,
    /// Every plant at this level, and `SetFidelity` actions are refused (a challenge's
    /// lock).
    Locked(HydraulicsLevel),
}

impl Fidelity {
    /// Whether this is [`Fidelity::Auto`] (the default, left out of files).
    #[must_use]
    pub fn is_auto(&self) -> bool {
        *self == Fidelity::Auto
    }

    /// The level a plant is sown at, for a scenario that sows `sowings` seeds at the
    /// start.
    #[must_use]
    pub fn level(self, sowings: usize) -> HydraulicsLevel {
        match self {
            Fidelity::Auto if sowings > MAX_DETAILED => HydraulicsLevel::Simplified,
            Fidelity::Auto | Fidelity::Detailed => HydraulicsLevel::Detailed,
            Fidelity::Simplified => HydraulicsLevel::Simplified,
            Fidelity::Locked(level) => level,
        }
    }
}

impl Scenario {
    /// Parses and checks a scenario file.
    ///
    /// # Errors
    /// [`WorldError::Scenario`] for a parse error, a wrong schema version, or an
    /// inconsistent scenario.
    pub fn from_ron(text: &str) -> Result<Self, WorldError> {
        let s: Self = ron::from_str(text).map_err(|e| WorldError::Scenario(e.to_string()))?;
        s.validate()?;
        Ok(s)
    }

    /// Checks the supports: a known soil, a finite position, height and diameter in range.
    fn validate_supports(&self, ids: &[u32]) -> Result<(), WorldError> {
        for p in &self.supports {
            if !ids.contains(&p.soil) {
                return Err(WorldError::Scenario(format!(
                    "support {p:?} on unknown soil"
                )));
            }
            p.check().map_err(WorldError::Scenario)?;
        }
        for o in &self.obstacles {
            if !ids.contains(&o.soil) {
                return Err(WorldError::Scenario(format!(
                    "obstacle {o:?} on unknown soil"
                )));
            }
            o.check().map_err(WorldError::Scenario)?;
        }
        Ok(())
    }

    /// Checks references and ranges.
    ///
    /// # Errors
    /// [`WorldError::Scenario`] naming the first problem.
    #[allow(clippy::too_many_lines)]
    pub fn validate(&self) -> Result<(), WorldError> {
        let bad = |m: String| Err(WorldError::Scenario(m));
        if self.schema != SCENARIO_SCHEMA {
            return bad(format!(
                "schema {} is not supported (this build reads {SCENARIO_SCHEMA})",
                self.schema
            ));
        }
        self.variation.check().map_err(WorldError::Scenario)?;
        if !(-90.0..=90.0).contains(&self.latitude_deg) {
            return bad(format!("latitude {} out of range", self.latitude_deg));
        }
        let st = self.start;
        if ps_weather::DateHour::new(st.year, st.month, st.day, st.hour).is_none() {
            return bad(format!(
                "start {}-{:02}-{:02} hour {} is not a valid date and hour",
                st.year, st.month, st.day, st.hour
            ));
        }
        let ids: Vec<u32> = self.soils.iter().map(SoilSpec::id).collect();
        for (i, id) in ids.iter().enumerate() {
            if ids[..i].contains(id) {
                return bad(format!("soil id {id} appears twice"));
            }
        }
        for spec in &self.soils {
            if let SoilSpec::Pot {
                id,
                drainage: PotDrainage::Saucer { depth_mm },
                ..
            } = spec
                && !(depth_mm.is_finite() && (0.0..=100.0).contains(depth_mm))
            {
                return bad(format!("pot {id} saucer depth {depth_mm} mm; 0 to 100"));
            }
            if let SoilSpec::Plot {
                id,
                width_cm,
                length_cm,
                depth_cm,
                ..
            } = spec
            {
                let side = |v: &f64| v.is_finite() && (PLOT_MIN_CM..=PLOT_MAX_CM).contains(v);
                if !side(width_cm) || !side(length_cm) {
                    return bad(format!(
                        "plot {id} is {width_cm} x {length_cm} cm; sides must be {PLOT_MIN_CM} to {PLOT_MAX_CM} cm"
                    ));
                }
                if !(depth_cm.is_finite() && *depth_cm >= 10.0) {
                    return bad(format!("plot {id} depth {depth_cm} cm"));
                }
            }
        }
        let known = |id: &u32| ids.contains(id);
        for s in &self.sowings {
            if !known(&s.soil) {
                return bad(format!("sowing names unknown soil {}", s.soil));
            }
            if !(s.depth_cm.is_finite() && s.depth_cm > 0.0) {
                return bad(format!("sowing depth {} cm must be positive", s.depth_cm));
            }
        }
        self.validate_supports(&ids)?;
        for (id, f) in &self.radiation_fraction {
            if !known(id) || !(0.0..=1.0).contains(f) {
                return bad(format!("radiation fraction {f} for soil {id}"));
            }
        }
        for (id, n) in &self.nitrogen {
            let ok = |v: f64| v.is_finite() && v >= 0.0;
            let valid = ok(n.no3_ppm) && ok(n.nh4_ppm) && ok(n.organic_g_per_kg);
            if !known(id) || !valid {
                return bad(format!("nitrogen {n:?} for soil {id}"));
            }
        }
        for (id, f) in &self.rain_fraction {
            if !known(id) || !(0.0..=1.0).contains(f) {
                return bad(format!("rain fraction {f} for soil {id}"));
            }
        }
        for (id, irr) in &self.irrigation {
            let every = match irr {
                Irrigation::None => 1,
                Irrigation::Schedule { every_days, mm } => {
                    if !(mm.is_finite() && *mm >= 0.0) {
                        return bad(format!("irrigation of {mm} mm for soil {id}"));
                    }
                    *every_days
                }
                Irrigation::ToFieldCapacity { every_days } => *every_days,
                Irrigation::ToFieldCapacityStrip {
                    every_days,
                    east_cm: (w, e),
                } => {
                    if !(w.is_finite() && e.is_finite() && w < e) {
                        return bad(format!("irrigation strip {w} to {e} cm for soil {id}"));
                    }
                    *every_days
                }
            };
            if !known(id) || every == 0 {
                return bad(format!("irrigation for soil {id}"));
            }
        }
        for id in &self.dividers {
            if !known(id) {
                return bad(format!("divider on unknown soil {id}"));
            }
        }
        for p in &self.nitrate_patches {
            let (x, y, z) = p.center_cm;
            if !known(&p.soil)
                || ![x, y, z, p.size_cm, p.no3_factor]
                    .iter()
                    .all(|v| v.is_finite())
                || p.size_cm <= 0.0
                || p.no3_factor < 0.0
                || z < 0.0
            {
                return bad(format!("nitrate patch {p:?}"));
            }
        }
        if !(self.co2_ppm.is_finite() && self.co2_ppm > 0.0) {
            return bad(format!("CO2 {} ppm", self.co2_ppm));
        }
        if let Some(hw) = &self.heat_wave {
            if !matches!(self.weather, WeatherSource::Preset(_)) {
                return bad("a heat wave needs preset weather".into());
            }
            if !(hw.t_min_c.is_finite() && hw.t_max_c.is_finite() && hw.t_min_c <= hw.t_max_c) {
                return bad(format!("heat wave {} to {} °C", hw.t_min_c, hw.t_max_c));
            }
        }
        if self.events.windows(2).any(|w| w[1].tick < w[0].tick) {
            return bad("events must be sorted by tick".into());
        }
        Ok(())
    }
}

/// A player or script action (§6.3). Ticks are sim hours since scenario start.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    /// When it applies: at the start of this hour, before the hourly step.
    pub tick: u64,
    /// What it does.
    pub kind: ActionKind,
}

/// What an action does. M0 implements these; harvest of single organs, fertilizer,
/// pollination and obstacles arrive with the milestones that need them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionKind {
    /// Sow a seed.
    Sow(Sowing),
    /// Add water to a soil now, mm over its surface.
    Water {
        /// Soil id.
        soil: u32,
        /// Water, mm.
        mm: f64,
    },
    /// Change a soil's radiation fraction.
    SetRadiationFraction {
        /// Soil id.
        soil: u32,
        /// New fraction, 0 to 1.
        fraction: f64,
    },
    /// Change a soil's automatic watering.
    SetIrrigation {
        /// Soil id.
        soil: u32,
        /// New schedule.
        irrigation: Irrigation,
    },
    /// Change how a pot drains (R5b, decision 0048): seal its holes, open them, or
    /// stand it in a saucer, as a waterlogging experiment drains its pots. Pots only.
    SetDrainage {
        /// Soil id of a pot.
        soil: u32,
        /// The new drainage.
        drainage: PotDrainage,
    },
    /// Cut an organ and everything it carries off a plant.
    Prune {
        /// Plant id.
        plant: u32,
        /// Organ id within the plant.
        organ: u32,
    },
    /// Pick a pod or ear; its seeds go to the seed library (§2 "Harvest pods", WP25).
    Harvest {
        /// Plant id.
        plant: u32,
        /// Organ id of the pod or ear within the plant.
        organ: u32,
    },
    /// Place an obstacle that shades the soil's plants from now on (WP25).
    PlaceObstacle(ObstacleSpec),
    /// Stand a stake, pole or trellis string; vines in the soil can climb it (WP25).
    AddSupport(SupportSpec),
    /// Spread mineral nitrogen fertilizer over a soil's surface (§2 "Fertilize", WP24).
    Fertilize {
        /// Soil id.
        soil: u32,
        /// Form.
        form: NitrogenForm,
        /// Nitrogen, g N per m² of surface (10 g m⁻² is 100 kg N ha⁻¹).
        g_per_m2: f64,
    },
    /// Hand-pollinate an open flower with another plant's pollen (§2, WP29): every
    /// seed of the flower's pod has `pollen` as its father. Flowers left alone self. A
    /// hand-self (`pollen == plant`) is accepted and gives the same seeds as selfing.
    Pollinate {
        /// Seed parent: the plant that carries the flower.
        plant: u32,
        /// Organ id of the open flower within that plant.
        organ: u32,
        /// Pollen parent: a plant with an open flower whose genome crosses with the
        /// seed parent's.
        pollen: u32,
    },
    /// Run these plants' water network at a level from the next hour (R3c, decision
    /// 0045): the viewer's zoom and speed rule writes it, so a replay is exact. An
    /// empty list sets every plant. Refused under [`Fidelity::Locked`].
    SetFidelity {
        /// Plant ids; empty for all.
        plants: Vec<u32>,
        /// The level.
        level: HydraulicsLevel,
    },
}
