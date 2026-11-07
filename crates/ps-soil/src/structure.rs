//! Soil structure (R5c, decision 0049): a bulk density per cell, what it does to the
//! soil's water, and how hard the soil is for a root.
//!
//! A grid without structure is uniform at its texture's bulk density, as before R5c, and
//! runs exactly as it did. Once a cell's density is set (a scenario's plough pan or
//! raised bed, [`SoilGrid::set_density`]; digging, [`SoilGrid::dig`]; a trampled path,
//! [`SoilGrid::compact`]) every cell carries:
//! - its bulk density `ρ` and the density it settles to: the texture's, or a layer's a
//!   scenario set, or a trampled path's;
//! - a hydraulic class ([`SoilClass`]): the saturation loses or gains the solid volume,
//!   `θ_s = θ_s0 − (ρ − ρ0) / ρ_p` with the particle density `ρ_p` 2650 kg m⁻³, at
//!   least [`MIN_STRUCTURAL_AIR`] above field capacity; field capacity and wilting point
//!   stay, and the retention curve is anchored through them again
//!   ([`Retention::anchored`]); the saturated conductivity follows the effective
//!   porosity `φ_e = θ_s − θ_fc` as `K_s = K_s0 (φ_e / φ_e0)^4` (Ahuja et al. 1984).
//!   Classes come in steps of [`DENSITY_STEP`] of the texture's density, each built once
//!   per process and shared;
//! - settling: a loosened cell closes the gap to its settled density by `R / (R + R_0)`
//!   of it, `R` the water that has reached the surface above it since it was loosened
//!   and `R_0` 10 mm (WEPP's rainfall consolidation, Alberts et al. 1995, eq. 7.7.9).
//!
//! Strength ([`SoilGrid::strength`]): the penetration resistance of the effective-stress
//! form of Whalley et al. (2007), `log Q = a log(S |ψ|) + b ρ + c` (`S` the relative
//! saturation, `ψ` the matric potential), with `a`, `b` and `c` set here ("~") so the
//! texture's own soil at field capacity reads [`STRENGTH_AT_FIELD_CAPACITY`]. Roots
//! read the resistance against the reference: the same soil water at the texture's own
//! density. The reference soil is what the root elongation rates were measured in, so
//! only the difference changes a root.
//!
//! Model card: docs/models/soil-and-water.md, "Structure".

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use ps_units::{KgPerM3, MPa};
use serde::{Deserialize, Serialize};

use crate::flow::FlowTable;
use crate::retention::FIELD_CAPACITY_POTENTIAL;
use crate::{Hydraulics, Retention, SoilGrid, Texture};

/// Density of the mineral particles, kg m⁻³ (quartz and clay minerals, the usual
/// 2650).
pub const PARTICLE_DENSITY: f64 = 2650.0;
/// Step of the hydraulic classes, a share of the texture's density: a cell's
/// saturation, retention curve and conductivity are those of its density rounded to
/// the nearest step ("~": 1% of a loam's density moves its saturation by 0.005).
pub const DENSITY_STEP: f64 = 0.01;
/// Least and most bulk density as a share of the texture's: a freshly dug bed to a
/// plough pan (the class steps run from −40 to +40).
pub const DENSITY_FACTOR_RANGE: (f64, f64) = (0.6, 1.4);
/// Least air a cell's saturation keeps above field capacity, m³ m⁻³: a loam compacted
/// past it (about 1.6 times its density) or a clay past about 1.12 keeps this air, and
/// its anchors stay solvable ("~").
pub const MIN_STRUCTURAL_AIR: f64 = 0.02;
/// Exponent of the saturated conductivity on the effective porosity, `K_s ∝ φ_e^n`
/// (Ahuja, Naney, Green and Nielsen 1984, a Kozeny–Carman form with n about 4
/// [K, to verify]).
pub const CONDUCTIVITY_EXPONENT: f64 = 4.0;
/// Bulk density of freshly dug soil, a share of the texture's settled density: WEPP's
/// fully disturbed soil, `ρ_t = 0.667 ρ_c` (Alberts et al. 1995, eq. 7.7.1).
pub const DUG_FACTOR: f64 = 0.667;
/// Bulk density of a trampled path, a share of the texture's ("~", R5c spec §5: a few
/// passes of feet raise a loam from about 1.3 to 1.5 Mg m⁻³).
pub const TRAMPLED_FACTOR: f64 = 1.15;
/// Water through the surface that settles a loosened soil halfway back, mm: WEPP's
/// rainfall consolidation `Δρ = Δρ_max R / (0.01 m + R)` (Alberts et al. 1995, eq.
/// 7.7.9), with the whole gap as `Δρ_max` (WEPP's eq. 7.7.10 gives about 90% of it for
/// a loam).
pub const SETTLING_WATER_MM: f64 = 10.0;
/// Penetration resistance of a texture's own soil at field capacity ("~": field soils
/// read about 0.3 `MPa` wet and over 1 `MPa` dry, Ge et al. 2019).
pub const STRENGTH_AT_FIELD_CAPACITY: MPa = MPa(0.5);
/// Exponent `a` of the penetration resistance on the effective stress `S |ψ|`
/// ("~": from field capacity to the wilting point the stress rises about 20-fold and
/// the resistance about 3-fold, Ge et al. 2019's 0.3 to over 1 `MPa`).
pub const STRENGTH_STRESS_EXPONENT: f64 = 0.4;
/// Slope `b` of `log10 Q` on bulk density, per 1000 kg m⁻³ ("~": a loam pan at 1.6
/// against 1.3 Mg m⁻³ reads four times as hard at field capacity, 2 `MPa`, where root
/// growth is held to be badly limited; Bengough et al. 2011).
pub const STRENGTH_DENSITY_SLOPE: f64 = 2.0;

/// One hydraulic class: a texture at one density step ([`DENSITY_STEP`]).
#[derive(Clone, Debug)]
pub struct SoilClass {
    /// The density step, 0 for the texture's own density.
    pub level: i32,
    /// Saturation, bulk density and saturated conductivity of the class; the texture's
    /// field capacity, wilting point and air-dry content.
    pub hydraulics: Hydraulics,
    /// The retention curve anchored on the class's saturation.
    pub retention: Retention,
    /// The flow step's table of the class.
    pub(crate) table: FlowTable,
}

impl SoilClass {
    fn build(texture: Texture, level: i32) -> Self {
        let base = texture.hydraulics();
        if level == 0 {
            return Self {
                level,
                hydraulics: base,
                retention: texture.retention(),
                table: FlowTable::for_texture(texture).clone(),
            };
        }
        let density = base.bulk_density * (1.0 + DENSITY_STEP * f64::from(level));
        let saturation = (base.saturation - (density - base.bulk_density) / PARTICLE_DENSITY)
            .max(base.field_capacity + MIN_STRUCTURAL_AIR);
        let porosity = (saturation - base.field_capacity) / (base.saturation - base.field_capacity);
        let ks = base.saturated_conductivity * libm::pow(porosity, CONDUCTIVITY_EXPONENT);
        let hydraulics = Hydraulics {
            saturation,
            bulk_density: density,
            saturated_conductivity: ks,
            ..base
        };
        let retention = Retention::anchored(&hydraulics);
        Self {
            level,
            hydraulics,
            retention,
            table: FlowTable::new(retention, ks, base.air_dry),
        }
    }
}

fn texture_slot(texture: Texture) -> usize {
    match texture {
        Texture::Loam => 0,
        Texture::Sand => 1,
        Texture::Clay => 2,
    }
}

/// The class of a texture at a density step, built once per process. A class depends
/// on its texture and step alone, so which thread builds it first does not matter.
#[must_use]
pub fn soil_class(texture: Texture, level: i32) -> Arc<SoilClass> {
    static CLASSES: Mutex<BTreeMap<(usize, i32), Arc<SoilClass>>> = Mutex::new(BTreeMap::new());
    let mut map = CLASSES.lock().unwrap_or_else(PoisonError::into_inner);
    map.entry((texture_slot(texture), level))
        .or_insert_with(|| Arc::new(SoilClass::build(texture, level)))
        .clone()
}

/// The texture's own class, for a grid without structure.
pub(crate) fn default_class(texture: Texture) -> &'static SoilClass {
    static DEFAULTS: [OnceLock<Arc<SoilClass>>; 3] = [const { OnceLock::new() }; 3];
    DEFAULTS[texture_slot(texture)].get_or_init(|| soil_class(texture, 0))
}

/// The density step of a bulk density in a texture.
#[must_use]
pub fn level_of(texture: Texture, density: f64) -> i32 {
    let base = texture.hydraulics().bulk_density;
    let (lo, hi) = DENSITY_FACTOR_RANGE;
    let factor = (density / base).clamp(lo, hi);
    // Within ±40 steps.
    #[allow(clippy::cast_possible_truncation)]
    let level = libm::round((factor - 1.0) / DENSITY_STEP) as i32;
    level
}

/// A cell's penetration resistance, and the reference it is read against (R5c).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Strength {
    /// Penetration resistance of the cell.
    pub resistance: MPa,
    /// The resistance of the same soil water at the texture's own density.
    pub reference: MPa,
}

/// Each cell's class index and the classes, built from the densities when first read.
#[derive(Clone, Debug)]
struct Classes {
    of_cell: Vec<u16>,
    classes: Vec<Arc<SoilClass>>,
}

/// Scratch: compares equal to any other and is not saved; rebuilt from the densities.
#[derive(Clone, Debug, Default)]
struct Palette(OnceLock<Classes>);

impl PartialEq for Palette {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// A grid's structure: per cell, its bulk density now, the density it settles to, the
/// density it was loosened to and the water that has reached the surface above it
/// since, all kg m⁻³ but the water, mm.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Structure {
    density: Vec<f64>,
    settled: Vec<f64>,
    loosened: Vec<f64>,
    water_since: Vec<f64>,
    #[serde(skip)]
    palette: Palette,
}

impl Structure {
    fn uniform(cells: usize, density: f64) -> Self {
        Self {
            density: vec![density; cells],
            settled: vec![density; cells],
            loosened: vec![density; cells],
            water_since: vec![0.0; cells],
            palette: Palette::default(),
        }
    }

    fn classes(&self, texture: Texture) -> &Classes {
        self.palette.0.get_or_init(|| {
            let mut index: BTreeMap<i32, u16> = BTreeMap::new();
            let mut classes = Vec::new();
            let of_cell = self
                .density
                .iter()
                .map(|&d| {
                    let level = level_of(texture, d);
                    *index.entry(level).or_insert_with(|| {
                        classes.push(soil_class(texture, level));
                        // At most 81 steps.
                        u16::try_from(classes.len() - 1).unwrap_or(u16::MAX)
                    })
                })
                .collect();
            Classes { of_cell, classes }
        })
    }

    /// The class of each cell and the classes (for the flow step).
    pub(crate) fn cell_classes(&self, texture: Texture) -> (&[u16], &[Arc<SoilClass>]) {
        let c = self.classes(texture);
        (&c.of_cell, &c.classes)
    }

    fn class(&self, texture: Texture, i: usize) -> &SoilClass {
        let c = self.classes(texture);
        &c.classes[usize::from(c.of_cell[i])]
    }

    /// Forgets the classes after a density changed its step.
    fn changed(&mut self) {
        self.palette = Palette::default();
    }

    /// Settles each loosened cell of a column by `mm` of water reaching its surface,
    /// and forgets the classes if a cell moved to another step.
    fn settle_column(&mut self, texture: Texture, cells: impl Iterator<Item = usize>, mm: f64) {
        let mut moved = false;
        for i in cells {
            let gap = self.settled[i] - self.loosened[i];
            if gap <= 0.0 || self.density[i] >= self.settled[i] {
                continue;
            }
            self.water_since[i] += mm;
            let r = self.water_since[i];
            let d = self.loosened[i] + gap * (r / (r + SETTLING_WATER_MM));
            moved |= level_of(texture, d) != level_of(texture, self.density[i]);
            self.density[i] = d;
        }
        if moved {
            self.changed();
        }
    }

    fn visit_bits(&self, f: &mut impl FnMut(u64)) {
        for v in [
            &self.density,
            &self.settled,
            &self.loosened,
            &self.water_since,
        ] {
            for x in v {
                f(x.to_bits());
            }
        }
    }
}

impl SoilGrid {
    /// Whether any cell's density was ever set (R5c): a grid without structure is
    /// uniform at its texture's density.
    #[must_use]
    pub fn has_structure(&self) -> bool {
        self.structure.is_some()
    }

    fn structure_mut(&mut self) -> &mut Structure {
        let (n, d) = (self.water.len(), self.texture.hydraulics().bulk_density);
        self.structure
            .get_or_insert_with(|| Structure::uniform(n, d))
    }

    /// The hydraulic class of cell `i` (R5c).
    #[must_use]
    pub fn class(&self, i: usize) -> &SoilClass {
        match &self.structure {
            Some(s) => s.class(self.texture, i),
            None => default_class(self.texture),
        }
    }

    /// Saturation, bulk density and saturated conductivity of cell `i`'s class, with
    /// the texture's field capacity, wilting point and air-dry content (R5c).
    #[must_use]
    pub fn hydraulics_at(&self, i: usize) -> Hydraulics {
        self.class(i).hydraulics
    }

    /// The retention curve of cell `i` (R5c): the texture's, anchored on the cell's
    /// saturation.
    #[must_use]
    pub fn retention_at(&self, i: usize) -> Retention {
        self.class(i).retention
    }

    /// Bulk density of cell `i` now.
    #[must_use]
    pub fn bulk_density(&self, i: usize) -> KgPerM3 {
        KgPerM3(match &self.structure {
            Some(s) => s.density[i],
            None => self.texture.hydraulics().bulk_density,
        })
    }

    /// Bulk density cell `i` settles back to after digging.
    #[must_use]
    pub fn settled_density(&self, i: usize) -> KgPerM3 {
        KgPerM3(match &self.structure {
            Some(s) => s.settled[i],
            None => self.texture.hydraulics().bulk_density,
        })
    }

    /// Sets each of `cells` to `factor` times the texture's density, where it stays (a
    /// scenario's plough pan or raised bed; R5c). The factor is held to
    /// [`DENSITY_FACTOR_RANGE`].
    pub fn set_density(&mut self, cells: &[usize], factor: f64) {
        let (lo, hi) = DENSITY_FACTOR_RANGE;
        let d = self.texture.hydraulics().bulk_density * factor.clamp(lo, hi);
        let s = self.structure_mut();
        for &i in cells {
            if i < s.density.len() {
                s.density[i] = d;
                s.settled[i] = d;
                s.loosened[i] = d;
                s.water_since[i] = 0.0;
            }
        }
        s.changed();
    }

    /// Digs each of `cells` (R5c): it is loosened to [`DUG_FACTOR`] of the texture's
    /// density (or stays looser), and settles back as water reaches it toward the
    /// texture's density, or its own if that is looser: digging breaks a pan up.
    pub fn dig(&mut self, cells: &[usize]) {
        let base = self.texture.hydraulics().bulk_density;
        let target = base * DUG_FACTOR;
        let s = self.structure_mut();
        for &i in cells {
            if i < s.density.len() {
                s.settled[i] = s.settled[i].min(base);
                s.density[i] = s.density[i].min(target);
                s.loosened[i] = s.density[i];
                s.water_since[i] = 0.0;
            }
        }
        s.changed();
    }

    /// Compacts each of `cells` (a trampled path; R5c): to at least
    /// [`TRAMPLED_FACTOR`] of the texture's density, where it stays.
    pub fn compact(&mut self, cells: &[usize]) {
        let target = self.texture.hydraulics().bulk_density * TRAMPLED_FACTOR;
        let s = self.structure_mut();
        for &i in cells {
            if i < s.density.len() {
                s.density[i] = s.density[i].max(target);
                s.settled[i] = s.settled[i].max(s.density[i]);
                s.loosened[i] = s.density[i];
                s.water_since[i] = 0.0;
            }
        }
        s.changed();
    }

    /// Settles the loosened cells under each column by the water that reached its
    /// surface this hour, `mm` per column in column order.
    pub(crate) fn settle(&mut self, mm: &[f64]) {
        let (layer, nz, texture) = (self.nx * self.ny, self.nz, self.texture);
        let Some(s) = self.structure.as_mut() else {
            return;
        };
        for (col, &w) in mm.iter().enumerate().filter(|(_, w)| **w > 0.0) {
            s.settle_column(texture, (0..nz).map(|z| col + z * layer), w);
        }
    }

    /// Penetration resistance of cell `i` and its reference (R5c); `None` in a grid
    /// without structure or outside the pot. With `S = θ / θ_s` and `ψ` the cell's
    /// potential, `Q = Q_fc (S |ψ| / σ_fc)^a 10^(b (ρ − ρ0))`, `σ_fc` the effective
    /// stress of the texture's own soil at field capacity; the reference is the same
    /// with `ρ = ρ0`.
    #[must_use]
    pub fn strength(&self, i: usize) -> Option<Strength> {
        let s = self.structure.as_ref()?;
        if !self.active[i] {
            return None;
        }
        let base = self.texture.hydraulics();
        let class = s.class(self.texture, i);
        let theta = self.theta(i);
        let rel = (theta / class.hydraulics.saturation).clamp(0.0, 1.0);
        let stress = rel * -class.retention.psi(theta).0;
        let stress_fc = base.field_capacity / base.saturation * -FIELD_CAPACITY_POTENTIAL.0;
        let reference = STRENGTH_AT_FIELD_CAPACITY.0
            * libm::pow(stress.max(0.0) / stress_fc, STRENGTH_STRESS_EXPONENT);
        let excess = (s.density[i] - base.bulk_density) / 1000.0;
        Some(Strength {
            resistance: MPa(reference * libm::exp10(STRENGTH_DENSITY_SLOPE * excess)),
            reference: MPa(reference),
        })
    }

    pub(crate) fn visit_structure_bits(&self, f: &mut impl FnMut(u64)) {
        if let Some(s) = &self.structure {
            f(1);
            s.visit_bits(f);
        }
    }
}
