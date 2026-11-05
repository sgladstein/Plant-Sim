//! What the renderer reads each frame: a read-only description of the pots and plants.
//!
//! The data source (the demo driver now, the sim thread's snapshot in WP16) fills
//! [`Garden`]; the scene systems only read it. Nothing here writes sim state (§11.10).

use std::collections::BTreeMap;

use bevy::prelude::Resource;
use ps_graph::OrganKind;
use ps_mesh::{Placement, Skeleton};

use crate::lens::Lens;

/// The pot's shape.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PotShape {
    /// A rhizobox: a rectangular slab with a glass front.
    #[default]
    Box,
    /// A round pot of diameter `width`: the back half terracotta, the front half glass.
    Round,
}

/// A pot's footprint and depth, m. The soil surface is at z = 0 in the sim frame.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PotSize {
    /// Box or round.
    pub shape: PotShape,
    /// East-west width (the diameter of a round pot).
    pub width: f32,
    /// North-south depth (the glass front faces south).
    pub depth: f32,
    /// Soil depth below the surface.
    pub soil_depth: f32,
}

impl PotSize {
    /// The M1 rhizobox: 32 × 18 cm, 45 cm deep.
    pub const RHIZOBOX: Self = Self {
        shape: PotShape::Box,
        width: 0.32,
        depth: 0.18,
        soil_depth: 0.45,
    };
}

/// A grid of values on the pot's glass face, row 0 at the surface, for the soil water
/// lens. Values are FTSW, 0 to 1.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoilFace {
    /// Cells across (west to east).
    pub columns: usize,
    /// Cells down.
    pub rows: usize,
    /// `rows × columns` values, row-major from the top-west cell.
    pub values: Vec<f32>,
}

impl SoilFace {
    /// The value of a cell, if present.
    #[must_use]
    pub fn get(&self, row: usize, column: usize) -> Option<f32> {
        if column >= self.columns {
            return None;
        }
        self.values.get(row * self.columns + column).copied()
    }
}

/// A vertical support a climbing plant twines on: a stake or a trellis string.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SupportView {
    /// Foot at the soil surface, relative to the pot center (sim x, y), m.
    pub base: [f32; 2],
    /// Height above the soil, m.
    pub height: f32,
    /// Diameter, m.
    pub diameter: f32,
}

impl SupportView {
    /// Thinner than this, a support is a string hung from a frame; thicker, a stake.
    pub const STRING_MAX_DIAMETER: f32 = 0.01;

    /// True for a trellis string.
    #[must_use]
    pub fn is_string(&self) -> bool {
        self.diameter < Self::STRING_MAX_DIAMETER
    }
}

/// A box that shades a container's plants: a board wall or a shade cloth (WP25).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ObstacleView {
    /// Lower corner relative to the pot center and soil surface (sim x, y, z), m.
    pub min: [f32; 3],
    /// Upper corner, m.
    pub max: [f32; 3],
    /// Share of light passing through, 0 to 1.
    pub transmittance: f32,
}

impl ObstacleView {
    /// At or above this transmittance an obstacle is a cloth or mesh, drawn see-through.
    pub const CLOTH_MIN_TRANSMITTANCE: f32 = 0.05;

    /// True for a see-through shade cloth.
    #[must_use]
    pub fn is_cloth(&self) -> bool {
        self.transmittance >= Self::CLOTH_MIN_TRANSMITTANCE
    }
}

/// One pot with its plant.
#[derive(Clone, Debug, PartialEq)]
pub struct PotView {
    /// Label shown under the pot ("Sun", "Shade").
    pub label: String,
    /// Center of the pot's soil surface in the sim frame (x east, y north), m.
    pub center: [f32; 2],
    /// Pot dimensions.
    pub size: PotSize,
    /// The plant, placed relative to the pot center.
    pub skeleton: Skeleton,
    /// Plant water stress factor, 1 = unstressed.
    pub water_stress: f32,
    /// Sim hour the plant's first flower appeared, once it has flowered (WP20's
    /// flowering cue).
    pub first_flower: Option<f64>,
    /// Per-lens values by organ id; NaN where a lens has no value for an organ. Shared:
    /// they change once per sim hour, the skeleton every frame.
    pub organ_values: std::sync::Arc<BTreeMap<Lens, Vec<f32>>>,
    /// Soil water on the cut face.
    pub soil_water: SoilFace,
    /// Whether this view draws the container (soil, frame, glass). A plot holds many
    /// plants in one soil; only its first plant's view draws the bed, the others share
    /// its center and size and draw just their plant.
    pub container: bool,
    /// Stakes and strings standing in the container (drawn with it).
    pub supports: Vec<SupportView>,
    /// Walls and cloths shading the container (drawn with it).
    pub obstacles: Vec<ObstacleView>,
}

impl PotView {
    /// The lens value of organ `index`, if the lens has one.
    #[must_use]
    pub fn organ_value(&self, lens: Lens, index: usize) -> Option<f32> {
        if lens == Lens::PlantWater {
            return Some(self.water_stress);
        }
        self.organ_values
            .get(&lens)
            .and_then(|v| v.get(index))
            .copied()
            .filter(|v| v.is_finite())
    }
}

/// The whole scene the renderer draws.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct Garden {
    /// Pots, left to right.
    pub pots: Vec<PotView>,
    /// Sim time on screen, hours since the start (fractional between ticks).
    pub hour: f64,
    /// Lenses whose values come from stand-in data rather than the sim (shown in the
    /// legend until ps-world supplies them).
    pub stand_in_lenses: Vec<Lens>,
}

/// `natural` with a small per-organ offset, so no two leaves are quite the same green:
/// about ±6% in value and a slight shift between yellower and bluer. Leaves only.
#[must_use]
pub fn varied_color(natural: [f32; 3], p: &Placement, index: usize) -> [f32; 3] {
    if p.kind != OrganKind::Leaf || !p.alive {
        return natural;
    }
    let id = u32::try_from(index).unwrap_or(u32::MAX);
    let value = 1.0 + 0.06 * ps_mesh::jitter(id, 7);
    let warm = 0.05 * ps_mesh::jitter(id, 8);
    [
        natural[0] * value * (1.0 + warm),
        natural[1] * value,
        natural[2] * value * (1.0 - warm),
    ]
}

/// The natural (lens-off) color of an organ, linear RGB.
#[must_use]
pub fn natural_color(p: &Placement) -> [f32; 3] {
    let age = p.thermal_age;
    let mix = |a: [f32; 3], b: [f32; 3], t: f32| {
        let t = t.clamp(0.0, 1.0);
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    };
    if !p.alive {
        return [0.25, 0.16, 0.08];
    }
    match p.kind {
        // Young leaves are a fresh yellow-green and darken as they mature.
        OrganKind::Leaf => mix([0.24, 0.46, 0.08], [0.08, 0.21, 0.045], age / 150.0),
        OrganKind::Cotyledon => mix([0.45, 0.5, 0.12], [0.55, 0.45, 0.1], age / 150.0),
        OrganKind::Internode | OrganKind::Tendril | OrganKind::Bud => [0.16, 0.27, 0.07],
        // Active nodules are pink inside (leghemoglobin) and tan outside.
        OrganKind::Nodule => mix([0.86, 0.6, 0.55], [0.6, 0.42, 0.32], age / 900.0),
        OrganKind::Root => mix([0.95, 0.92, 0.8], [0.55, 0.42, 0.26], age / 600.0),
        OrganKind::Flower => [0.85, 0.75, 0.9],
        // Pods ripen from green to straw.
        OrganKind::Pod => mix([0.15, 0.38, 0.05], [0.6, 0.48, 0.22], (age - 300.0) / 300.0),
        OrganKind::Seed | OrganKind::SeedInPod => [0.75, 0.62, 0.42],
    }
}
