//! Soil and water (design plan §3.6, §3.2 hourly steps 4, 5 and 7; WP5).
//!
//! A 3D bucket model on a regular grid of cubic cells. Each active cell holds water as a
//! mass (kg). Texture gives field capacity, wilting point and saturation by volume.
//! Pots are masks on the grid; pot bottoms drain freely.
//!
//! Hourly order inside [`SoilGrid::step_hour`] (step 4 of §3.2):
//! 1. infiltration of rain and irrigation, filling each column from the top to field
//!    capacity, then above it up to saturation; what does not fit runs off;
//! 2. drainage: every cell's water above field capacity moves one cell down; the bottom
//!    layer's excess leaves the grid;
//! 3. lateral redistribution: 10% of the water difference to each horizontal neighbor;
//! 4. evaporation from the top layer only, reduced as it dries;
//! 5. soil temperature by layer from lagged, damped air temperature.
//!
//! Uptake (step 7) is [`SoilGrid::take_up`]; FTSW and the stress factor (step 5) are
//! [`SoilGrid::ftsw`] and [`stress_factor`]. Every flux is booked in a [`WaterLedger`];
//! debug builds assert closure to 1e-9 relative.
//!
//! Model card: docs/models/soil-and-water.md.

use ps_units::{Celsius, KgWater, Meters, Millimeters, WPerM2};
use serde::{Deserialize, Serialize};

/// Latent heat of vaporization, J kg⁻¹ (at about 20 °C).
pub const LATENT_HEAT_J_PER_KG: f64 = 2.45e6;
/// Density of water, kg m⁻³.
pub const WATER_DENSITY: f64 = 1000.0;
/// Share of the gradient moved to each horizontal neighbor per hour (§3.6).
pub const LATERAL_FRACTION: f64 = 0.1;
/// Fraction of radiation reaching wet bare soil that evaporates water ("~", about
/// Priestley-Taylor at 20 °C with net radiation 0.6 of global).
pub const EVAPORATION_COEFFICIENT: f64 = 0.5;
/// Diurnal damping depth of soil temperature, m ("~", typical moist loam 0.1 m).
pub const DAMPING_DEPTH_M: f64 = 0.10;
/// Relative conservation tolerance (CLAUDE.md).
pub const CLOSURE_TOLERANCE: f64 = 1e-9;

/// Soil texture class.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Texture {
    /// Loam.
    Loam,
    /// Sand.
    Sand,
    /// Clay.
    Clay,
}

/// Volumetric water contents of a texture (m³ m⁻³).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hydraulics {
    /// Saturation (total porosity).
    pub saturation: f64,
    /// Field capacity.
    pub field_capacity: f64,
    /// Permanent wilting point.
    pub wilting_point: f64,
    /// Air-dry content: evaporation stops here.
    pub air_dry: f64,
}

impl Texture {
    /// Hydraulic constants. Field capacity and wilting point from design plan §3.6;
    /// saturation and air-dry are "~" (typical textbook values, Saxton and Rawls 2006 ranges).
    #[must_use]
    pub const fn hydraulics(self) -> Hydraulics {
        match self {
            Texture::Loam => Hydraulics {
                saturation: 0.45,
                field_capacity: 0.27,
                wilting_point: 0.12,
                air_dry: 0.05,
            },
            Texture::Sand => Hydraulics {
                saturation: 0.40,
                field_capacity: 0.15,
                wilting_point: 0.05,
                air_dry: 0.02,
            },
            Texture::Clay => Hydraulics {
                saturation: 0.50,
                field_capacity: 0.40,
                wilting_point: 0.22,
                air_dry: 0.10,
            },
        }
    }
}

/// Grid position of a cell: x, y horizontal, z down from the surface (0 = top layer).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell {
    /// Column index along x.
    pub x: usize,
    /// Column index along y.
    pub y: usize,
    /// Layer index, 0 at the surface.
    pub z: usize,
}

/// Water fluxes over one step, all positive, kg.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WaterLedger {
    /// Rain and irrigation applied at the surface.
    pub applied: KgWater,
    /// Applied water that did not fit and ran off.
    pub runoff: KgWater,
    /// Water drained out of the bottom.
    pub drained: KgWater,
    /// Water evaporated from the surface.
    pub evaporated: KgWater,
    /// Water taken up by roots.
    pub uptake: KgWater,
    /// Storage at the start of the step.
    pub storage_before: KgWater,
    /// Storage at the end of the step.
    pub storage_after: KgWater,
}

impl WaterLedger {
    /// Mass balance residual: Δstorage − (applied − runoff − drained − evaporated − uptake).
    #[must_use]
    pub fn residual(&self) -> KgWater {
        (self.storage_after - self.storage_before)
            - (self.applied - self.runoff - self.drained - self.evaporated - self.uptake)
    }

    /// Residual relative to the largest term (so a closed ledger is about 1e-15).
    #[must_use]
    pub fn relative_residual(&self) -> f64 {
        let scale = [
            self.storage_before.0,
            self.storage_after.0,
            self.applied.0,
            self.drained.0,
            self.evaporated.0,
            self.uptake.0,
            self.runoff.0,
        ]
        .into_iter()
        .fold(1e-12, f64::max);
        self.residual().0.abs() / scale
    }

    /// Adds another step's fluxes (`storage_before` is kept, `storage_after` replaced).
    pub fn accumulate(&mut self, o: &WaterLedger) {
        self.applied += o.applied;
        self.runoff += o.runoff;
        self.drained += o.drained;
        self.evaporated += o.evaporated;
        self.uptake += o.uptake;
        self.storage_after = o.storage_after;
    }
}

/// Inputs to the soil for one hour.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SoilHour {
    /// Rain plus irrigation over the soil surface.
    pub water_in: Millimeters,
    /// Radiation reaching the soil surface (after the canopy).
    pub surface_radiation: WPerM2,
    /// Air temperature.
    pub t_air: Celsius,
}

/// Result of a root uptake request.
#[derive(Clone, Debug, PartialEq)]
pub struct Uptake {
    /// Water actually removed.
    pub actual: KgWater,
    /// Demand that could not be met (logged, not penalized again, §3.2 step 7).
    pub shortfall: KgWater,
    /// Water removed per cell, indexed like the grid.
    pub per_cell: Vec<KgWater>,
    /// Ledger of this uptake.
    pub ledger: WaterLedger,
}

/// Errors building a grid.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SoilError {
    /// Grid dimensions or cell size invalid.
    #[error("invalid soil grid: {0}")]
    Geometry(String),
    /// A per-cell array has the wrong length.
    #[error("expected {expected} cells, got {found}")]
    Length {
        /// Cells in the grid.
        expected: usize,
        /// Length given.
        found: usize,
    },
}

/// The soil of one pot or plot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoilGrid {
    nx: usize,
    ny: usize,
    nz: usize,
    cell: Meters,
    texture: Texture,
    active: Vec<bool>,
    water: Vec<KgWater>,
    layer_temperature: Vec<Celsius>,
    /// Last 24 hourly air temperatures, oldest first; drives soil temperature.
    air_history: Vec<Celsius>,
}

/// Water stress factor from FTSW: 1 above `threshold`, linear to 0 at FTSW 0 (§3.6).
#[must_use]
pub fn stress_factor(ftsw: f64, threshold: f64) -> f64 {
    if threshold <= 0.0 {
        return 1.0;
    }
    (ftsw / threshold).clamp(0.0, 1.0)
}

impl SoilGrid {
    /// A box of `nx × ny × nz` active cells at field capacity.
    ///
    /// # Errors
    /// [`SoilError::Geometry`] for empty dimensions or a non-positive cell size.
    pub fn new_box(
        nx: usize,
        ny: usize,
        nz: usize,
        cell: Meters,
        texture: Texture,
        t_initial: Celsius,
    ) -> Result<Self, SoilError> {
        Self::with_mask(nx, ny, nz, cell, texture, t_initial, |_, _| true)
    }

    /// A cylindrical pot: a square grid around a circle of `diameter`, `depth` deep,
    /// with columns whose centers fall inside the circle active. Starts at field capacity.
    ///
    /// # Errors
    /// [`SoilError::Geometry`] if the pot is smaller than one cell.
    pub fn new_pot(
        diameter: Meters,
        depth: Meters,
        cell: Meters,
        texture: Texture,
        t_initial: Celsius,
    ) -> Result<Self, SoilError> {
        if !(cell.0 > 0.0 && diameter.0 >= cell.0 && depth.0 >= cell.0) {
            return Err(SoilError::Geometry(format!(
                "pot {} x {} m with cell {} m",
                diameter.0, depth.0, cell.0
            )));
        }
        let n = cells_across(diameter.0, cell.0);
        let nz = cells_across(depth.0, cell.0);
        let r = 0.5 * diameter.0;
        #[allow(clippy::cast_precision_loss)]
        let center = 0.5 * n as f64 * cell.0;
        Self::with_mask(n, n, nz, cell, texture, t_initial, |x, y| {
            #[allow(clippy::cast_precision_loss)]
            let cx = (x as f64 + 0.5) * cell.0 - center;
            #[allow(clippy::cast_precision_loss)]
            let cy = (y as f64 + 0.5) * cell.0 - center;
            cx * cx + cy * cy <= r * r
        })
    }

    fn with_mask(
        nx: usize,
        ny: usize,
        nz: usize,
        cell: Meters,
        texture: Texture,
        t_initial: Celsius,
        inside: impl Fn(usize, usize) -> bool,
    ) -> Result<Self, SoilError> {
        if nx == 0 || ny == 0 || nz == 0 || !(cell.0 > 0.0 && cell.0.is_finite()) {
            return Err(SoilError::Geometry(format!(
                "{nx} x {ny} x {nz} cells of {} m",
                cell.0
            )));
        }
        let n = nx * ny * nz;
        let mut active = vec![false; n];
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    active[(z * ny + y) * nx + x] = inside(x, y);
                }
            }
        }
        let mut g = Self {
            nx,
            ny,
            nz,
            cell,
            texture,
            active,
            water: vec![KgWater(0.0); n],
            layer_temperature: vec![t_initial; nz],
            air_history: vec![t_initial; 24],
        };
        g.fill_to(texture.hydraulics().field_capacity);
        Ok(g)
    }

    /// Sets every active cell to volumetric water content `theta`.
    pub fn fill_to(&mut self, theta: f64) {
        let w = KgWater(theta * self.cell_volume() * WATER_DENSITY);
        for i in 0..self.water.len() {
            self.water[i] = if self.active[i] { w } else { KgWater(0.0) };
        }
    }

    /// Grid dimensions `(nx, ny, nz)`.
    #[must_use]
    pub fn dims(&self) -> (usize, usize, usize) {
        (self.nx, self.ny, self.nz)
    }

    /// Number of cells, active or not.
    #[must_use]
    pub fn len(&self) -> usize {
        self.water.len()
    }

    /// True for an empty grid (never, for a built grid).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.water.is_empty()
    }

    /// Cell edge length.
    #[must_use]
    pub fn cell_size(&self) -> Meters {
        self.cell
    }

    /// Texture.
    #[must_use]
    pub fn texture(&self) -> Texture {
        self.texture
    }

    /// Volume of one cell, m³.
    #[must_use]
    pub fn cell_volume(&self) -> f64 {
        self.cell.0 * self.cell.0 * self.cell.0
    }

    /// Flat index of a cell.
    #[must_use]
    pub fn index(&self, c: Cell) -> usize {
        (c.z * self.ny + c.y) * self.nx + c.x
    }

    /// Cell of a flat index.
    #[must_use]
    pub fn cell_of(&self, i: usize) -> Cell {
        let x = i % self.nx;
        let y = (i / self.nx) % self.ny;
        let z = i / (self.nx * self.ny);
        Cell { x, y, z }
    }

    /// Whether a cell is inside the pot.
    #[must_use]
    pub fn is_active(&self, i: usize) -> bool {
        self.active[i]
    }

    /// Water in a cell.
    #[must_use]
    pub fn water(&self, i: usize) -> KgWater {
        self.water[i]
    }

    /// Volumetric water content of a cell.
    #[must_use]
    pub fn theta(&self, i: usize) -> f64 {
        self.water[i].0 / (self.cell_volume() * WATER_DENSITY)
    }

    /// Soil temperature of a layer.
    #[must_use]
    pub fn layer_temperature(&self, z: usize) -> Celsius {
        self.layer_temperature[z]
    }

    /// Total water in the grid, summed in index order.
    #[must_use]
    pub fn total_water(&self) -> KgWater {
        self.water.iter().copied().sum()
    }

    /// Area of the soil surface (active top cells), m².
    #[must_use]
    pub fn surface_area(&self) -> f64 {
        let top = self.nx * self.ny;
        #[allow(clippy::cast_precision_loss)]
        let n = self.active[..top].iter().filter(|a| **a).count() as f64;
        n * self.cell.0 * self.cell.0
    }

    fn capacity(&self, theta: f64) -> KgWater {
        KgWater(theta * self.cell_volume() * WATER_DENSITY)
    }

    /// Runs hydrological step 4 of §3.2 for one hour and returns its ledger.
    pub fn step_hour(&mut self, h: &SoilHour) -> WaterLedger {
        let mut ledger = WaterLedger {
            storage_before: self.total_water(),
            ..WaterLedger::default()
        };
        self.infiltrate(h.water_in, &mut ledger);
        self.drain(&mut ledger);
        self.redistribute_laterally();
        self.evaporate(h.surface_radiation, &mut ledger);
        self.update_temperature(h.t_air);
        ledger.storage_after = self.total_water();
        debug_assert!(
            ledger.relative_residual() < CLOSURE_TOLERANCE,
            "soil water ledger does not close: {ledger:?}"
        );
        ledger
    }

    fn infiltrate(&mut self, water_in: Millimeters, ledger: &mut WaterLedger) {
        let per_column = KgWater(water_in.0.max(0.0) * self.cell.0 * self.cell.0);
        let fc = self.capacity(self.texture.hydraulics().field_capacity);
        let sat = self.capacity(self.texture.hydraulics().saturation);
        for y in 0..self.ny {
            for x in 0..self.nx {
                if !self.active[self.index(Cell { x, y, z: 0 })] {
                    continue;
                }
                ledger.applied += per_column;
                let mut left = per_column;
                // Fill to field capacity top-down, then to saturation top-down.
                for limit in [fc, sat] {
                    for z in 0..self.nz {
                        if left.0 <= 0.0 {
                            break;
                        }
                        let i = self.index(Cell { x, y, z });
                        let room = (limit - self.water[i]).max(KgWater(0.0));
                        let add = room.min(left);
                        self.water[i] += add;
                        left -= add;
                    }
                }
                ledger.runoff += left.max(KgWater(0.0));
            }
        }
    }

    fn drain(&mut self, ledger: &mut WaterLedger) {
        let fc = self.capacity(self.texture.hydraulics().field_capacity);
        let layer = self.nx * self.ny;
        // Jacobi: compute all excesses from the start-of-step state, then move them.
        let excess: Vec<KgWater> = self
            .water
            .iter()
            .zip(&self.active)
            .map(|(w, a)| {
                if *a {
                    (*w - fc).max(KgWater(0.0))
                } else {
                    KgWater(0.0)
                }
            })
            .collect();
        for (i, e) in excess.iter().enumerate() {
            if e.0 <= 0.0 {
                continue;
            }
            self.water[i] -= *e;
            let below = i + layer;
            if below < self.water.len() && self.active[below] {
                self.water[below] += *e;
            } else {
                ledger.drained += *e;
            }
        }
    }

    fn redistribute_laterally(&mut self) {
        let mut delta = vec![KgWater(0.0); self.water.len()];
        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let i = self.index(Cell { x, y, z });
                    if !self.active[i] {
                        continue;
                    }
                    // Each pair once: +x and +y neighbors.
                    let mut pair = |j: usize| {
                        if self.active[j] {
                            let f = (self.water[i] - self.water[j]) * LATERAL_FRACTION;
                            delta[i] -= f;
                            delta[j] += f;
                        }
                    };
                    if x + 1 < self.nx {
                        pair(self.index(Cell { x: x + 1, y, z }));
                    }
                    if y + 1 < self.ny {
                        pair(self.index(Cell { x, y: y + 1, z }));
                    }
                }
            }
        }
        for (w, d) in self.water.iter_mut().zip(delta) {
            *w += d;
        }
    }

    fn evaporate(&mut self, radiation: WPerM2, ledger: &mut WaterLedger) {
        let hy = self.texture.hydraulics();
        let fc = self.capacity(hy.field_capacity);
        let dry = self.capacity(hy.air_dry);
        let potential_per_m2 =
            EVAPORATION_COEFFICIENT * radiation.0.max(0.0) * 3600.0 / LATENT_HEAT_J_PER_KG;
        let potential = KgWater(potential_per_m2 * self.cell.0 * self.cell.0);
        for i in 0..self.nx * self.ny {
            if !self.active[i] {
                continue;
            }
            let reduction = ((self.water[i] - dry) / (fc - dry)).clamp(0.0, 1.0);
            let e = (potential * reduction).min((self.water[i] - dry).max(KgWater(0.0)));
            self.water[i] -= e;
            ledger.evaporated += e;
        }
    }

    fn update_temperature(&mut self, t_air: Celsius) {
        self.air_history.remove(0);
        self.air_history.push(t_air);
        let mean = self.air_history.iter().map(|t| t.0).sum::<f64>() / 24.0;
        for z in 0..self.nz {
            #[allow(clippy::cast_precision_loss)]
            let depth = (z as f64 + 0.5) * self.cell.0;
            let damp = libm::exp(-depth / DAMPING_DEPTH_M);
            // Phase lag of a diurnal wave: depth / D radians = 24 depth / (2π D) hours.
            let lag_h = 24.0 * depth / (2.0 * core::f64::consts::PI * DAMPING_DEPTH_M);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let lag = (libm::round(lag_h) as usize).min(23);
            let lagged = self.air_history[23 - lag].0;
            self.layer_temperature[z] = Celsius(mean + damp * (lagged - mean));
        }
    }

    /// Fraction of transpirable soil water over root-occupied cells, weighted by root
    /// length: `Σ L (W − W_wp) / Σ L (W_fc − W_wp)`, clipped to [0, 1]. Returns 1 when
    /// there are no roots (a seed in moist soil is not stressed).
    ///
    /// # Errors
    /// [`SoilError::Length`] if `root_length` does not have one entry per cell.
    pub fn ftsw(&self, root_length: &[Meters]) -> Result<f64, SoilError> {
        self.check_len(root_length.len())?;
        let hy = self.texture.hydraulics();
        let wp = self.capacity(hy.wilting_point);
        let span = self.capacity(hy.field_capacity) - wp;
        let (mut num, mut den) = (0.0, 0.0);
        for (i, l) in root_length.iter().enumerate() {
            if !self.active[i] || l.0 <= 0.0 {
                continue;
            }
            num += l.0 * (self.water[i] - wp).0.max(0.0);
            den += l.0 * span.0;
        }
        Ok(if den > 0.0 {
            (num / den).clamp(0.0, 1.0)
        } else {
            1.0
        })
    }

    /// FTSW of a single cell (for germination).
    #[must_use]
    pub fn cell_ftsw(&self, i: usize) -> f64 {
        let hy = self.texture.hydraulics();
        let wp = self.capacity(hy.wilting_point);
        let span = self.capacity(hy.field_capacity) - wp;
        ((self.water[i] - wp) / span).clamp(0.0, 1.0)
    }

    fn check_len(&self, n: usize) -> Result<(), SoilError> {
        if n == self.water.len() {
            Ok(())
        } else {
            Err(SoilError::Length {
                expected: self.water.len(),
                found: n,
            })
        }
    }

    /// Root water uptake (§3.2 step 7, §3.6). Potential uptake per cell is
    /// `rate × L / V × (W − W_wp)`, with root length `L` in the cell of volume `V`; the
    /// actual uptake is `min(demand, Σ potential)`, taken from cells in proportion to
    /// their potential.
    ///
    /// `rate` is the species uptake coefficient in m² h⁻¹ (root length density times
    /// rate is the hourly fraction of available water a cell can give).
    ///
    /// # Errors
    /// [`SoilError::Length`] if `root_length` does not have one entry per cell.
    pub fn take_up(
        &mut self,
        demand: KgWater,
        root_length: &[Meters],
        rate_m2_per_h: f64,
    ) -> Result<Uptake, SoilError> {
        self.check_len(root_length.len())?;
        let before = self.total_water();
        let wp = self.capacity(self.texture.hydraulics().wilting_point);
        let v = self.cell_volume();
        let potential: Vec<KgWater> = self
            .water
            .iter()
            .zip(root_length)
            .zip(&self.active)
            .map(|((w, l), a)| {
                if *a && l.0 > 0.0 {
                    let avail = (*w - wp).max(KgWater(0.0));
                    (avail * (rate_m2_per_h * l.0 / v).min(1.0)).min(avail)
                } else {
                    KgWater(0.0)
                }
            })
            .collect();
        let total: KgWater = potential.iter().copied().sum();
        let want = demand.max(KgWater(0.0));
        let actual = want.min(total);
        let share = if total.0 > 0.0 { actual / total } else { 0.0 };
        let per_cell: Vec<KgWater> = potential.iter().map(|p| *p * share).collect();
        for (w, u) in self.water.iter_mut().zip(&per_cell) {
            *w -= *u;
        }
        let ledger = WaterLedger {
            uptake: per_cell.iter().copied().sum(),
            storage_before: before,
            storage_after: self.total_water(),
            ..WaterLedger::default()
        };
        debug_assert!(ledger.relative_residual() < CLOSURE_TOLERANCE, "{ledger:?}");
        Ok(Uptake {
            actual: ledger.uptake,
            shortfall: want - ledger.uptake,
            per_cell,
            ledger,
        })
    }

    /// Visits all state as bits in a fixed order, for the full-state hash.
    pub fn visit_bits(&self, mut f: impl FnMut(u64)) {
        f(self.nx as u64);
        f(self.ny as u64);
        f(self.nz as u64);
        f(self.cell.to_bits());
        f(self.texture as u64);
        for (a, w) in self.active.iter().zip(&self.water) {
            f(u64::from(*a));
            f(w.to_bits());
        }
        for t in &self.layer_temperature {
            f(t.to_bits());
        }
        for t in &self.air_history {
            f(t.to_bits());
        }
    }
}

fn cells_across(length: f64, cell: f64) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = libm::round(length / cell).max(1.0) as usize;
    n
}

#[cfg(test)]
mod tests;
