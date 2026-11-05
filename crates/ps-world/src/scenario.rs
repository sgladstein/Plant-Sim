//! Scenario files (design plan §6.2) and the action log (§6.3).

use std::collections::BTreeMap;

use ps_soil::Texture;
use serde::{Deserialize, Serialize};

use crate::WorldError;

/// The scenario schema version this build reads.
pub const SCENARIO_SCHEMA: u32 = 1;

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
}

impl SoilSpec {
    /// The soil id.
    #[must_use]
    pub fn id(&self) -> u32 {
        match self {
            Self::Pot { id, .. } => *id,
        }
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
    /// Automatic watering per soil; none if absent.
    #[serde(default)]
    pub irrigation: BTreeMap<u32, Irrigation>,
    /// Ambient CO2, ppm.
    pub co2_ppm: f64,
    /// Seeds sown at tick 0.
    pub sowings: Vec<Sowing>,
    /// Length of the run, days.
    pub duration_days: u32,
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
        if self.start.hour > 23 {
            return bad(format!("start hour {} out of range", self.start.hour));
        }
        let ids: Vec<u32> = self.soils.iter().map(SoilSpec::id).collect();
        for (i, id) in ids.iter().enumerate() {
            if ids[..i].contains(id) {
                return bad(format!("soil id {id} appears twice"));
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
        for (id, f) in &self.radiation_fraction {
            if !known(id) || !(0.0..=1.0).contains(f) {
                return bad(format!("radiation fraction {f} for soil {id}"));
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
}
