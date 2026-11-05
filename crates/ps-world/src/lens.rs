//! Lens data (WP26): per-hour values the viewer's competition, temperature, nitrogen
//! and carbon-flow lenses read. They are derived from the step (copied out of it or
//! read from the state after it), live on the [`World`](crate::World) beside its
//! state, and are never saved or hashed: turning them on or off cannot change a hash,
//! and a loaded save has none until its next hour. Model card:
//! docs/models/world-and-schedule.md, "Lens data".

use ps_units::{Celsius, GramsC, M2, UmolPerM2S};

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
    /// The temperature each organ grew at during the hour, indexed by organ id (air
    /// for the shoot, the soil layer for roots and seed; leaves get their own
    /// temperature with WP27).
    pub organ_temperature: Vec<Celsius>,
    /// Nitrogen per structural dry mass after the hour, g N g⁻¹, indexed by organ id
    /// (0 for dead or massless organs).
    pub organ_n: Vec<f64>,
}

/// One soil's lens values after the hour.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoilLens {
    /// Soil id.
    pub soil: u32,
    /// Temperature of each soil layer, top first.
    pub layer_temperature: Vec<Celsius>,
}
