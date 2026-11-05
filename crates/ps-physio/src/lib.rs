//! Plant physiology (design plan §3.2 hourly steps 6 to 12, §3.8 to §3.11; WP7).
//!
//! - [`gas`]: coupled Farquhar and Medlyn gas exchange for one leaf class.
//! - [`classes`]: the per-plant light-class table and leaf lookup.
//! - [`energy`]: the leaf energy balance (leaf temperature, WP27).
//! - [`respiration`]: maintenance and growth respiration.
//! - [`thermal`]: thermal time and the expansion temperature multiplier.
//! - [`phenology`]: V and R stages.
//! - [`allocation`]: hourly carbon steps 8 to 12 and starvation deaths.
//!
//! Model cards: docs/models/gas-exchange.md, respiration.md,
//! thermal-time-and-phenology.md, carbon-allocation.md.

pub mod allocation;
pub mod classes;
pub mod energy;
pub mod gas;
pub mod phenology;
pub mod respiration;
pub mod thermal;

pub use allocation::{
    CarbonLedger, CarbonParams, ConstructionCost, HourInputs, HourReport, Temperature,
};
pub use classes::{ClassTable, Rates};
pub use energy::{Air, LeafEnergyParams, LeafHeat, Site};
pub use gas::{GasExchange, LeafEnv, LeafParams, Limitation, PeakedArrhenius};
pub use phenology::{Events, Phenology, Stage, Thresholds};
pub use thermal::{Cardinal, GrowthTemperature};

use ps_species::{Species, SpeciesError, Traits};
use ps_units::{JPerMol, JPerMolK, MolPerM2S, SqrtKPa, UmolPerM2S};

impl LeafParams {
    /// Leaf parameters of one plant: heritable traits plus species constants.
    #[must_use]
    pub fn from_species(species: &Species, traits: &Traits) -> Self {
        let c = &species.constants;
        Self {
            vcmax25: UmolPerM2S(traits.vcmax25),
            jmax25: UmolPerM2S(traits.jmax25),
            rd25: UmolPerM2S(traits.rd25),
            rd_q10: c.rd_q10.value,
            g1: SqrtKPa(traits.medlyn_g1),
            g0: MolPerM2S(c.medlyn_g0.value),
            quantum_yield: c.quantum_yield.value,
            curvature: c.electron_transport_curvature.value,
            vcmax_t: PeakedArrhenius {
                activation: JPerMol(c.vcmax_activation.value),
                deactivation: JPerMol(c.vcmax_deactivation.value),
                entropy: JPerMolK(c.vcmax_entropy.value),
            },
            jmax_t: PeakedArrhenius {
                activation: JPerMol(c.jmax_activation.value),
                deactivation: JPerMol(c.jmax_deactivation.value),
                entropy: JPerMolK(c.jmax_entropy.value),
            },
        }
    }
}

/// Photosynthetic capacity of a leaf at thermal age `age` (§3.8 "leaf aging"): 1 until
/// full expansion at `expanded`, then falling linearly to `floor` at `lifespan`.
#[must_use]
pub fn leaf_capacity(
    age: ps_units::DegreeDays,
    expanded: ps_units::DegreeDays,
    lifespan: ps_units::DegreeDays,
    floor: f64,
) -> f64 {
    if age.0 <= expanded.0 {
        return 1.0;
    }
    let span = (lifespan.0 - expanded.0).max(1e-9);
    (1.0 - (1.0 - floor) * (age.0 - expanded.0) / span).clamp(floor, 1.0)
}

impl Cardinal {
    /// The species' cardinal temperatures of development.
    #[must_use]
    pub fn from_species(s: &Species) -> Self {
        let p = &s.phenology;
        Self {
            base: ps_units::Celsius(p.t_base.value),
            opt: ps_units::Celsius(p.t_opt.value),
            opt_high: ps_units::Celsius(p.t_opt_high.value),
            max: ps_units::Celsius(p.t_max.value),
        }
    }
}

impl GrowthTemperature {
    /// The species' expansion temperature response.
    #[must_use]
    pub fn from_species(s: &Species) -> Self {
        let c = &s.constants;
        Self {
            min: ps_units::Celsius(c.growth_t_min.value),
            opt_low: ps_units::Celsius(c.growth_t_opt_low.value),
            opt_high: ps_units::Celsius(c.growth_t_opt_high.value),
            max: ps_units::Celsius(c.growth_t_max.value),
        }
    }
}

impl Thresholds {
    /// The species' stage thresholds.
    #[must_use]
    pub fn from_species(s: &Species) -> Self {
        let p = &s.phenology;
        Self {
            emergence: ps_units::DegreeDays(p.emergence.value),
            flowering: ps_units::DegreeDays(p.flowering.value),
            maturity: ps_units::DegreeDays(p.maturity.value),
        }
    }
}

/// Errors from physiology setup.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PhysioError {
    /// The species or phenotype is unusable.
    #[error(transparent)]
    Species(#[from] SpeciesError),
    /// The organ graph rejected an update.
    #[error(transparent)]
    Graph(#[from] ps_graph::GraphError),
    /// A per-organ input does not have one entry per organ.
    #[error("{what} has {got} entries for {want} organs")]
    InputLength {
        /// Which input.
        what: &'static str,
        /// Its length.
        got: usize,
        /// The arena's length.
        want: usize,
    },
}

#[cfg(test)]
mod alloc_tests;
#[cfg(test)]
mod tests;
