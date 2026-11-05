//! Scenario files (design plan §6.2) and the action log (§6.3).

use std::collections::BTreeMap;

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
}

/// Which genotype a sown seed has.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedChoice {
    /// The species-default genotype.
    Default,
    /// A genotype drawn from the species population (world seed, plant id).
    Sample,
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
    /// Ambient CO2, ppm.
    pub co2_ppm: f64,
    /// Seeds sown at tick 0.
    pub sowings: Vec<Sowing>,
    /// Vertical supports for climbing plants; none if absent (WP22).
    #[serde(default)]
    pub supports: Vec<SupportSpec>,
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
            let fine = |v: f64, hi: f64| v.is_finite() && v > 0.0 && v <= hi;
            let (x, y) = p.offset_cm;
            let valid = ids.contains(&p.soil)
                && fine(p.height_cm, MAX_SUPPORT_HEIGHT_CM)
                && fine(p.diameter_cm, MAX_SUPPORT_DIAMETER_CM)
                && x.is_finite()
                && y.is_finite();
            if !valid {
                return Err(WorldError::Scenario(format!(
                    "support {p:?}: height up to {MAX_SUPPORT_HEIGHT_CM} cm, diameter up to {MAX_SUPPORT_DIAMETER_CM} cm, on a known soil"
                )));
            }
        }
        Ok(())
    }

    /// Checks references and ranges.
    ///
    /// # Errors
    /// [`WorldError::Scenario`] naming the first problem.
    pub fn validate(&self) -> Result<(), WorldError> {
        let bad = |m: String| Err(WorldError::Scenario(m));
        if self.schema != SCENARIO_SCHEMA {
            return bad(format!(
                "schema {} is not supported (this build reads {SCENARIO_SCHEMA})",
                self.schema
            ));
        }
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
            };
            if !known(id) || every == 0 {
                return bad(format!("irrigation for soil {id}"));
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
    /// Cut an organ and everything it carries off a plant.
    Prune {
        /// Plant id.
        plant: u32,
        /// Organ id within the plant.
        organ: u32,
    },
    /// Spread mineral nitrogen fertilizer over a soil's surface (§2 "Fertilize", WP24).
    Fertilize {
        /// Soil id.
        soil: u32,
        /// Form.
        form: NitrogenForm,
        /// Nitrogen, g N per m² of surface (10 g m⁻² is 100 kg N ha⁻¹).
        g_per_m2: f64,
    },
}
