//! Lens data (WP26): per-hour values the viewer's competition, temperature, nitrogen,
//! carbon-flow and water-potential (R3a, R3b) lenses read, and each leaf's turgor for
//! wilting (R3b). They are derived from the step (copied out of it or
//! read from the state after it), live on the [`World`](crate::World) beside its
//! state, and are never saved or hashed: turning them on or off cannot change a hash,
//! and a loaded save has none until its next hour. Model card:
//! docs/models/world-and-schedule.md, "Lens data".

use ps_units::{Celsius, GramsC, KPa, M2, MPa, UmolPerM2S};

/// The lens data of the hour just stepped.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LensData {
    /// The tick of the hour these values describe (the world's tick is one more).
    pub tick: u64,
    /// One entry per plant, in plant id order.
    pub plants: Vec<PlantLens>,
    /// One entry per soil, in scenario order.
    pub soils: Vec<SoilLens>,
}

/// One plant's lens values for the hour.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlantLens {
    /// Plant id.
    pub plant: u32,
    /// Living leaf area.
    pub leaf_area: M2,
    /// Leaf-area-weighted mean PAR absorbed by the plant's leaves (competition).
    pub absorbed_par: UmolPerM2S,
    /// PAR a horizontal unshaded leaf would absorb under the hour's sky (after the
    /// soil's shade cloth): the reference for [`PlantLens::relative_light`].
    pub open_sky_par: UmolPerM2S,
    /// `absorbed_par / open_sky_par` (0 at night): how much of the open sky's light
    /// the plant's leaves get, on average. Leaves facing a low sun can exceed 1.
    pub relative_light: f64,
    /// The plant's share of the PAR absorbed by every plant on its soil (area times
    /// mean), 0 to 1; 0 when the soil's plants absorb nothing.
    pub light_share: f64,
    /// Carbon moved from each organ's parent into it during the hour (negative: an
    /// export to the parent), indexed by organ id: the flow arrows.
    pub phloem_import: Vec<GramsC>,
    /// Each organ's temperature during the hour, indexed by organ id: a leaf's own
    /// from the energy balance (WP27; its gas exchange and frost read it, its
    /// expansion reads air), air for the rest of the shoot, the soil layer for roots
    /// and seed.
    pub organ_temperature: Vec<Celsius>,
    /// Nitrogen per structural dry mass after the hour, g N g⁻¹, indexed by organ id
    /// (0 for dead or massless organs).
    pub organ_n: Vec<f64>,
    /// Each organ's water potential from the hour's network solve, indexed by organ id
    /// (pressure, without gravity; R3b).
    pub organ_water_potential: Vec<MPa>,
    /// Each organ's relative turgor from that potential, 0 at the turgor loss point to
    /// 1 at full turgor, indexed by organ id: how wilted a leaf looks (R3b).
    pub organ_turgor: Vec<f64>,
    /// Leaf-area-weighted mean leaf turgor (1 without leaves).
    pub leaf_turgor: f64,
    /// The root zone's soil water potential, weighted by each root cell's radial
    /// conductance (R3b).
    pub root_zone_potential: MPa,
    /// The plant's water-network level this hour (R3c): at the simplified level the
    /// organ potentials are the day summary's estimates.
    pub hydraulics_level: ps_arch::HydraulicsLevel,
    /// Each soil layer's share of the plant's uptake in evenly wet soil, top first
    /// (the day's standard uptake fractions; R3c).
    pub uptake_by_layer: Vec<f64>,
}

/// One soil's lens values after the hour.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoilLens {
    /// Soil id.
    pub soil: u32,
    /// Temperature of each soil layer, top first.
    pub layer_temperature: Vec<Celsius>,
    /// Water potential of each soil layer, top first: the potential of the layer's mean
    /// water content (R3a; `ps_soil::SoilGrid::layer_water_potentials`). A cell's own is
    /// `SoilGrid::water_potential`.
    pub layer_water_potential: Vec<KPa>,
}
