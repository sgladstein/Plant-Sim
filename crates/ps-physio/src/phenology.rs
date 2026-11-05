//! Phenological stages on the bean V and R scale (§3.11, daily step 1).
//! Model card: docs/models/thermal-time-and-phenology.md.

use ps_units::DegreeDays;
use serde::{Deserialize, Serialize};

/// Development stages, in order. Transitions never go backward.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage {
    /// Sown, not yet germinated.
    Sown,
    /// Germinated; the hypocotyl is growing toward the surface.
    Germinated,
    /// VE: cotyledons at the surface; the unifoliate leaves open.
    Emerged,
    /// V1: the first trifoliate leaf has unfolded.
    FirstTrifoliate,
    /// R1: first open flower.
    Flowering,
    /// R3: first pod set.
    PodSet,
    /// R5: seeds filling in the first pod.
    SeedFill,
    /// R9: physiological maturity; the annual's life ends.
    Mature,
}

/// Thresholds of thermal time since sowing (species phenology block).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Thresholds {
    /// Sowing to emergence.
    pub emergence: DegreeDays,
    /// Sowing to first flower.
    pub flowering: DegreeDays,
    /// Sowing to maturity.
    pub maturity: DegreeDays,
}

/// Organ events the architecture module reports each day.
#[allow(clippy::struct_excessive_bools)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Events {
    /// The seed has germinated (§3.10 soil conditions met).
    pub germinated: bool,
    /// At least one trifoliate leaf has unfolded.
    pub trifoliate: bool,
    /// At least one pod has set.
    pub pod: bool,
    /// At least one seed is filling.
    pub seed_filling: bool,
}

/// A plant's phenological state.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Phenology {
    /// Current stage.
    pub stage: Stage,
    /// Thermal time since sowing (counted only after germination for emergence).
    pub since_sowing: DegreeDays,
    /// Thermal time since germination.
    pub since_germination: DegreeDays,
}

impl Default for Phenology {
    fn default() -> Self {
        Self {
            stage: Stage::Sown,
            since_sowing: DegreeDays(0.0),
            since_germination: DegreeDays(0.0),
        }
    }
}

impl Phenology {
    /// Adds an hour's (or a day's) thermal time.
    pub fn accumulate(&mut self, dt: DegreeDays) {
        self.since_sowing += dt;
        if self.stage >= Stage::Germinated {
            self.since_germination += dt;
        }
    }

    /// Daily step 1: advances the stage at most as far as thresholds and events allow,
    /// one stage per call, and returns the new stage if it changed.
    ///
    /// Emergence counts thermal time from germination (§3.10: "emergence follows at
    /// 60 to 80 °C·day"); flowering and maturity count from sowing (§3.11). Flowering
    /// needs the first trifoliate, pod set needs a pod, seed fill needs a filling seed.
    pub fn advance(&mut self, th: &Thresholds, ev: Events) -> Option<Stage> {
        let next = match self.stage {
            Stage::Sown if ev.germinated => Some(Stage::Germinated),
            Stage::Germinated if self.since_germination.0 >= th.emergence.0 => Some(Stage::Emerged),
            Stage::Emerged if ev.trifoliate => Some(Stage::FirstTrifoliate),
            Stage::FirstTrifoliate if self.since_sowing.0 >= th.flowering.0 => {
                Some(Stage::Flowering)
            }
            Stage::Flowering if ev.pod => Some(Stage::PodSet),
            Stage::PodSet if ev.seed_filling => Some(Stage::SeedFill),
            Stage::SeedFill if self.since_sowing.0 >= th.maturity.0 => Some(Stage::Mature),
            _ => None,
        };
        if let Some(s) = next {
            self.stage = s;
        }
        next
    }
}
