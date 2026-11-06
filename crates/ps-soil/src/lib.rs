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
//! 3. lateral redistribution: 10% of the water difference to each horizontal neighbor
//!    (none across a pot's divider); nitrate moves with that water at the concentration
//!    of the cell it leaves, plus slow diffusion (R2b, decision 0043);
//! 4. evaporation from the top layer only, reduced as it dries;
//! 5. soil temperature by layer from lagged, damped air temperature;
//! 6. nitrogen: mineralization and nitrification ([`nitrogen`]; nitrate also moves with
//!    the drainage and lateral water of steps 2 and 3).
//!
//! Irrigation can water a strip of columns only ([`SoilHour::strip`]), and a pot can
//! carry a watertight divider across its center ([`SoilGrid::set_divider`]): the
//! split-root pot of R2b.
//!
//! Uptake (step 7) is [`SoilGrid::take_up`], one call per hour for every plant in the
//! grid; FTSW and the stress factor (step 5) are [`SoilGrid::ftsw`] and
//! [`stress_factor`]. Roots enter as sparse [`RootCells`] lists. Every flux is booked in a [`WaterLedger`];
//! debug builds assert closure to 1e-9 relative.
//!
//! Water potential (R3a) is derived from the water content through each texture's
//! [`Retention`] curve; nothing in the step reads it yet.
//!
//! Model card: docs/models/soil-and-water.md.

use ps_units::{Celsius, GramsN, KPa, KgPerMH, KgWater, M2PerHour, Meters, Millimeters, WPerM2};

pub mod nitrogen;
pub mod retention;
pub use nitrogen::{NitrogenForm, NitrogenLedger, NitrogenRequest, NitrogenUptake, SoilNitrogen};
pub use retention::Retention;
use serde::{Deserialize, Serialize};

/// Latent heat of vaporization, J kg⁻¹ (at about 20 °C).
pub const LATENT_HEAT_J_PER_KG: f64 = 2.45e6;
/// Density of water, kg m⁻³.
pub const WATER_DENSITY: f64 = 1000.0;
/// Share of the gradient moved to each horizontal neighbor per hour (§3.6).
pub const LATERAL_FRACTION: f64 = 0.1;
/// Cell edge at which [`LATERAL_FRACTION`] applies, m. Other cells scale it as a
/// diffusivity, by `(reference / cell)²` (4e-5 m² per hour; WP21, decision 0019), so a
/// plot's 5 cm cells move 1.6% per hour.
pub const LATERAL_REFERENCE_CELL_M: f64 = 0.02;
/// Fraction of radiation reaching wet bare soil that evaporates water ("~", about
/// Priestley-Taylor at 20 °C with net radiation 0.6 of global).
pub const EVAPORATION_COEFFICIENT: f64 = 0.5;
/// Diurnal damping depth of soil temperature, m ("~", typical moist loam 0.1 m).
pub const DAMPING_DEPTH_M: f64 = 0.10;
/// Share of a nitrate difference that diffuses between two neighboring 2 cm cells in
/// an hour, scaled like [`LATERAL_FRACTION`] for other cells (R2b, decision 0043):
/// `D_l f / (θ Δx²) · 3600 s` with nitrate's diffusivity in water `D_l` 1.9e-9 m² s⁻¹,
/// `θ` 0.3 and the Millington–Quirk factor `f = θ^(10/3) / θ_s²` (0.089 at a
/// saturation of 0.45). `D_l f`, about 1.7e-10 m² s⁻¹, is the flux per area of soil
/// for a gradient in the soil solution; dividing by `θ` turns it into the rate at
/// which a cell's nitrate evens out, about 5.7e-10 m² s⁻¹: nitrate spreads about 5 cm
/// in a month by diffusion alone.
pub const NITRATE_DIFFUSION_FRACTION: f64 = 5.1e-3;
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
    /// Dry bulk density, kg m⁻³ (for nitrogen concentrations; WP24).
    pub bulk_density: f64,
}

impl Texture {
    /// Hydraulic constants. Field capacity and wilting point from design plan §3.6;
    /// saturation, air-dry and bulk density are "~" (typical textbook values, Saxton and
    /// Rawls 2006 ranges).
    #[must_use]
    pub const fn hydraulics(self) -> Hydraulics {
        match self {
            Texture::Loam => Hydraulics {
                saturation: 0.45,
                field_capacity: 0.27,
                wilting_point: 0.12,
                air_dry: 0.05,
                bulk_density: 1300.0,
            },
            Texture::Sand => Hydraulics {
                saturation: 0.40,
                field_capacity: 0.15,
                wilting_point: 0.05,
                air_dry: 0.02,
                bulk_density: 1550.0,
            },
            Texture::Clay => Hydraulics {
                saturation: 0.50,
                field_capacity: 0.40,
                wilting_point: 0.22,
                air_dry: 0.10,
                bulk_density: 1250.0,
            },
        }
    }

    /// The texture's water retention curve, anchored on its field capacity and
    /// wilting point (R3a; [`Retention::anchored`]). It solves the curve each call
    /// (about 54 bisection steps), so callers that need many cells take it once.
    #[must_use]
    pub fn retention(self) -> Retention {
        Retention::anchored(&self.hydraulics())
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
    /// Water given to a strip of columns only (a split pot's watered half), on top of
    /// `water_in`; `None` for none.
    pub strip: Option<Strip>,
}

/// Water over a strip of columns `x0 ≤ x < x1` (every y), mm over those columns.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Strip {
    /// Water, mm over the strip's surface.
    pub water: Millimeters,
    /// First column of the strip.
    pub x0: usize,
    /// One past its last column.
    pub x1: usize,
}

/// One plant's root length per soil cell: `(cell index, length)` pairs sorted by cell,
/// each cell once, lengths positive. Sums over it equal sums over a dense per-cell array.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RootCells(Vec<(usize, Meters)>);

impl RootCells {
    /// Builds the list from pairs in any order: sorts by cell (stable), adds the lengths
    /// of repeated cells in input order, and drops cells with no positive length.
    #[must_use]
    pub fn from_pairs(mut pairs: Vec<(usize, Meters)>) -> Self {
        pairs.sort_by_key(|p| p.0);
        let mut out: Vec<(usize, Meters)> = Vec::with_capacity(pairs.len());
        for (i, l) in pairs {
            match out.last_mut() {
                Some(last) if last.0 == i => last.1 += l,
                _ => out.push((i, l)),
            }
        }
        out.retain(|p| p.1.0 > 0.0);
        Self(out)
    }

    /// The pairs, sorted by cell.
    #[must_use]
    pub fn cells(&self) -> &[(usize, Meters)] {
        &self.0
    }

    /// Total root length.
    #[must_use]
    pub fn total(&self) -> Meters {
        self.0.iter().map(|p| p.1).sum()
    }
}

/// One plant's request in the hour's joint uptake.
#[derive(Copy, Clone, Debug)]
pub struct PlantUptake<'a> {
    /// Water the plant asks for this hour (its transpiration demand).
    pub demand: KgWater,
    /// Its roots.
    pub roots: &'a RootCells,
    /// Species uptake coefficient (root length density times it is the hourly fraction
    /// of a cell's available water the roots can take).
    pub rate: M2PerHour,
    /// Species ceiling on uptake per meter of root per hour.
    pub max_per_length: KgPerMH,
}

/// What one plant got from the joint uptake.
#[derive(Clone, Debug, PartialEq)]
pub struct Uptake {
    /// Water actually removed.
    pub actual: KgWater,
    /// Demand that could not be met (logged, not penalized again, §3.2 step 7).
    pub shortfall: KgWater,
    /// Water removed per root cell, in the order of the plant's [`RootCells`].
    pub per_cell: Vec<(usize, KgWater)>,
    /// What the roots could have taken this hour from the same state, before other
    /// plants' requests on shared cells: the sum of the per-cell potentials (R2a).
    pub capacity: KgWater,
}

/// Result of the hour's joint uptake.
#[derive(Clone, Debug, PartialEq)]
pub struct JointUptake {
    /// One entry per plant, in request order.
    pub plants: Vec<Uptake>,
    /// Ledger of the whole uptake.
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
    /// A root list names a cell outside the grid.
    #[error("root cell {cell} outside a grid of {cells} cells")]
    RootCell {
        /// The cell index.
        cell: usize,
        /// Cells in the grid.
        cells: usize,
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
    /// Nitrate N per cell (WP24).
    no3: Vec<GramsN>,
    /// Ammonium N per cell.
    nh4: Vec<GramsN>,
    /// Humus N per cell.
    organic_n: Vec<GramsN>,
    /// Rhizobia present.
    rhizobia: bool,
    /// Nitrogen fluxes since the nitrogen was set.
    n_ledger: NitrogenLedger,
    /// A watertight divider between columns `x − 1` and `x` (a split pot, R2b): no water
    /// or nitrate moves across it.
    #[serde(default)]
    divider: Option<usize>,
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
            no3: vec![GramsN(0.0); n],
            nh4: vec![GramsN(0.0); n],
            organic_n: vec![GramsN(0.0); n],
            rhizobia: false,
            n_ledger: NitrogenLedger::default(),
            divider: None,
        };
        g.fill_to(texture.hydraulics().field_capacity);
        Ok(g)
    }

    /// Puts a watertight divider across the grid's center, north to south: the west
    /// columns `x < nx / 2` and the east ones exchange no water or nitrate (a split
    /// pot, R2b). Nothing happens on a grid one column wide.
    pub fn set_divider(&mut self) {
        let d = self.nx / 2;
        self.divider = (d > 0).then_some(d);
    }

    /// The first column east of the divider, if there is one.
    #[must_use]
    pub fn divider(&self) -> Option<usize> {
        self.divider
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
        self.step_hour_with(h, true)
    }

    /// [`SoilGrid::step_hour`] whose ledger has the fluxes but leaves the two storage
    /// totals at zero, except in debug builds, where they are summed and the ledger's
    /// closure is asserted. Each total is a sequential sum over every cell, a long
    /// chain of dependent additions; a caller that keeps only the fluxes (the world,
    /// which takes the storage from [`SoilGrid::take_up_lean`]) skips both.
    pub fn step_hour_lean(&mut self, h: &SoilHour) -> WaterLedger {
        self.step_hour_with(h, cfg!(debug_assertions))
    }

    fn step_hour_with(&mut self, h: &SoilHour, totals: bool) -> WaterLedger {
        let mut ledger = WaterLedger {
            storage_before: if totals {
                self.total_water()
            } else {
                KgWater(0.0)
            },
            ..WaterLedger::default()
        };
        self.infiltrate(h.water_in, h.strip, &mut ledger);
        self.drain(&mut ledger);
        self.redistribute_laterally();
        self.evaporate(h.surface_radiation, &mut ledger);
        self.update_temperature(h.t_air);
        self.transform_nitrogen();
        self.assert_nitrogen_closes();
        if totals {
            ledger.storage_after = self.total_water();
            debug_assert!(
                ledger.relative_residual() < CLOSURE_TOLERANCE,
                "soil water ledger does not close: {ledger:?}"
            );
        }
        ledger
    }

    fn infiltrate(
        &mut self,
        water_in: Millimeters,
        strip: Option<Strip>,
        ledger: &mut WaterLedger,
    ) {
        let area = self.cell.0 * self.cell.0;
        let everywhere = KgWater(water_in.0.max(0.0) * area);
        let extra = |x: usize| match strip {
            Some(s) if x >= s.x0 && x < s.x1 => KgWater(s.water.0.max(0.0) * area),
            _ => KgWater(0.0),
        };
        let fc = self.capacity(self.texture.hydraulics().field_capacity);
        let sat = self.capacity(self.texture.hydraulics().saturation);
        for y in 0..self.ny {
            for x in 0..self.nx {
                if !self.active[self.index(Cell { x, y, z: 0 })] {
                    continue;
                }
                let per_column = everywhere + extra(x);
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
        let n = self.water.len();
        // Jacobi: every cell's excess, and the nitrate it carries, comes from the
        // start-of-step state. A layer's moves are computed before the layer above
        // pours into it, so one layer of scratch is enough.
        let mut cur = vec![(KgWater(0.0), GramsN(0.0)); layer];
        let mut next = cur.clone();
        self.layer_excess(0, fc, &mut cur);
        for z in 0..self.nz {
            if z + 1 < self.nz {
                self.layer_excess(z + 1, fc, &mut next);
            }
            let first = z * layer;
            for (k, &(e, m)) in cur.iter().enumerate() {
                let i = first + k;
                let below = i + layer;
                let into = below < n && self.active[below];
                if m.0 > 0.0 {
                    self.no3[i] -= m;
                    if into {
                        self.no3[below] += m;
                    } else {
                        self.n_ledger.leached += m;
                    }
                }
                if e.0 > 0.0 {
                    self.water[i] -= e;
                    if into {
                        self.water[below] += e;
                    } else {
                        ledger.drained += e;
                    }
                }
            }
            core::mem::swap(&mut cur, &mut next);
        }
    }

    /// Water above field capacity in each cell of layer `z`, and the nitrate it
    /// carries down, from the current state.
    fn layer_excess(&self, z: usize, fc: KgWater, out: &mut [(KgWater, GramsN)]) {
        let layer = self.nx * self.ny;
        let r = z * layer..(z + 1) * layer;
        let cells = self.water[r.clone()]
            .iter()
            .zip(&self.active[r.clone()])
            .zip(&self.no3[r]);
        for (o, ((w, a), no3)) in out.iter_mut().zip(cells) {
            let e = if *a {
                (*w - fc).max(KgWater(0.0))
            } else {
                KgWater(0.0)
            };
            let m = if e.0 > 0.0 && w.0 > 0.0 {
                *no3 * (e.0 / w.0).min(1.0)
            } else {
                GramsN(0.0)
            };
            *o = (e, m);
        }
    }

    /// The share of a water difference that moves between two neighboring cells in
    /// an hour: [`LATERAL_FRACTION`] at 2 cm, scaled as a diffusivity for other cell
    /// sizes and capped at 0.25 so the explicit step stays stable.
    #[must_use]
    pub fn lateral_fraction(&self) -> f64 {
        let r = LATERAL_REFERENCE_CELL_M / self.cell.0;
        (LATERAL_FRACTION * r * r).min(0.25)
    }

    /// The share of a nitrate difference that diffuses between two neighboring cells
    /// in an hour: [`NITRATE_DIFFUSION_FRACTION`] scaled like [`Self::lateral_fraction`].
    #[must_use]
    pub fn nitrate_diffusion_fraction(&self) -> f64 {
        let r = LATERAL_REFERENCE_CELL_M / self.cell.0;
        (NITRATE_DIFFUSION_FRACTION * r * r).min(0.25)
    }

    /// Step 3: each pair of active horizontal neighbors moves `fraction` of its water
    /// difference; nitrate moves with that water at the concentration of the cell the
    /// water leaves, and diffuses slowly on its own (decision 0043). Nothing crosses a
    /// divider. Fluxes come from the start-of-step state.
    fn redistribute_laterally(&mut self) {
        let fraction = self.lateral_fraction();
        let diffusion = self.nitrate_diffusion_fraction();
        let (nx, ny) = (self.nx, self.ny);
        let (wx, wy) = lateral_fluxes(&self.water, &self.active, nx, ny, self.divider, |a, b| {
            (a.0 - b.0) * fraction
        });
        let (water, no3) = (&self.water, &self.no3);
        // Nitrate per kg of water in cell i (zero in a dry cell, which sends no water).
        let conc = |i: usize| {
            if water[i].0 > 0.0 {
                no3[i].0 / water[i].0
            } else {
                0.0
            }
        };
        let carried = |f: f64, from: usize, to: usize| {
            if f > 0.0 {
                f * conc(from)
            } else {
                f * conc(to)
            }
        };
        let (nxf, nyf) = lateral_fluxes(&self.no3, &self.active, nx, ny, self.divider, |a, b| {
            (a.0 - b.0) * diffusion
        });
        // Flux k of the x list runs from cell k − 1 to k; of the y list from k − nx to k.
        let mx: Vec<f64> = (0..wx.len())
            .map(|k| {
                if k == 0 || k >= self.water.len() {
                    nxf[k]
                } else {
                    nxf[k] + carried(wx[k], k - 1, k)
                }
            })
            .collect();
        let my: Vec<f64> = (0..wy.len())
            .map(|k| {
                if k < nx || k >= self.water.len() {
                    nyf[k]
                } else {
                    nyf[k] + carried(wy[k], k - nx, k)
                }
            })
            .collect();
        apply_fluxes(&mut self.water, nx, &wx, &wy, |w| w.0, KgWater);
        apply_fluxes(&mut self.no3, nx, &mx, &my, |g| g.0, GramsN);
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
    /// there are no roots (a seed in moist soil is not stressed). Inactive cells are
    /// skipped.
    ///
    /// # Errors
    /// [`SoilError::RootCell`] if a root cell is outside the grid.
    pub fn ftsw(&self, roots: &RootCells) -> Result<f64, SoilError> {
        self.check_roots(roots)?;
        let hy = self.texture.hydraulics();
        let wp = self.capacity(hy.wilting_point);
        let span = self.capacity(hy.field_capacity) - wp;
        let (mut num, mut den) = (0.0, 0.0);
        for &(i, l) in roots.cells() {
            if !self.active[i] {
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

    /// The mean relative available water and nitrate (mg N per kg soil) over the active
    /// cells, in cell order: the background a root judges a cell's richness against
    /// (R2b, decision 0043).
    #[must_use]
    pub fn means(&self) -> (f64, f64) {
        let (mut k, mut w, mut n) = (0.0_f64, 0.0_f64, 0.0_f64);
        for i in (0..self.len()).filter(|&i| self.is_active(i)) {
            k += 1.0;
            w += self.cell_ftsw(i);
            n += self.no3_ppm(i);
        }
        let k = k.max(1.0);
        (w / k, n / k)
    }

    /// The grid's retention curve ([`Texture::retention`]).
    #[must_use]
    pub fn retention(&self) -> Retention {
        self.texture.retention()
    }

    /// Water potential of a cell (R3a), `None` outside the pot. It solves the
    /// texture's curve each call; for many cells use [`SoilGrid::retention`] once and
    /// [`Retention::psi`] on each [`SoilGrid::theta`].
    #[must_use]
    pub fn water_potential(&self, i: usize) -> Option<KPa> {
        self.active[i].then(|| self.retention().psi(self.theta(i)))
    }

    /// Each layer's water potential, top first: the potential of the layer's mean water
    /// content over its active cells (R3a; the lens value). That is wetter than the
    /// mean of the cells' potentials where roots have dried some cells, since the curve
    /// is steep when dry; [`SoilGrid::water_potential`] gives a cell's own.
    #[must_use]
    pub fn layer_water_potentials(&self) -> Vec<KPa> {
        let r = self.retention();
        let per_layer = self.nx * self.ny;
        (0..self.nz)
            .map(|z| {
                let (mut theta, mut k) = (0.0_f64, 0.0_f64);
                for i in (z * per_layer..(z + 1) * per_layer).filter(|&i| self.active[i]) {
                    theta += self.theta(i);
                    k += 1.0;
                }
                r.psi(theta / k.max(1.0))
            })
            .collect()
    }

    /// FTSW of a single cell (for germination).
    #[must_use]
    pub fn cell_ftsw(&self, i: usize) -> f64 {
        let hy = self.texture.hydraulics();
        let wp = self.capacity(hy.wilting_point);
        let span = self.capacity(hy.field_capacity) - wp;
        ((self.water[i] - wp) / span).clamp(0.0, 1.0)
    }

    fn check_roots(&self, roots: &RootCells) -> Result<(), SoilError> {
        match roots.cells().last() {
            Some(&(i, _)) if i >= self.water.len() => Err(SoilError::RootCell {
                cell: i,
                cells: self.water.len(),
            }),
            _ => Ok(()),
        }
    }

    /// Root water uptake for every plant in the grid, once per hour (§3.2 step 7, §3.6;
    /// decision 0012). All plants see the same soil state, so their order does not
    /// matter.
    ///
    /// 1. Each plant's potential in each of its cells is
    ///    `min(rate · L / V · (W − W_wp)⁺, max_per_length · L, (W − W_wp)⁺)`.
    /// 2. It asks each cell for `min(demand, Σ potential) / Σ potential` of its potential.
    /// 3. Where the plants together ask a cell for more than its available water, every
    ///    request in that cell is scaled down by the same factor.
    ///
    /// # Errors
    /// [`SoilError::RootCell`] if a root cell is outside the grid.
    pub fn take_up(&mut self, plants: &[PlantUptake<'_>]) -> Result<JointUptake, SoilError> {
        self.take_up_with(plants, true)
    }

    /// [`SoilGrid::take_up`] whose ledger leaves `storage_before` at zero, except in
    /// debug builds, where it is summed and the ledger's closure is asserted.
    /// `storage_after` is always summed.
    ///
    /// # Errors
    /// [`SoilError::RootCell`] if a root cell is outside the grid.
    pub fn take_up_lean(&mut self, plants: &[PlantUptake<'_>]) -> Result<JointUptake, SoilError> {
        self.take_up_with(plants, cfg!(debug_assertions))
    }

    fn take_up_with(
        &mut self,
        plants: &[PlantUptake<'_>],
        totals: bool,
    ) -> Result<JointUptake, SoilError> {
        for p in plants {
            self.check_roots(p.roots)?;
        }
        let before = if totals {
            self.total_water()
        } else {
            KgWater(0.0)
        };
        let wp = self.capacity(self.texture.hydraulics().wilting_point);
        let v = self.cell_volume();
        let avail = |w: KgWater| (w - wp).max(KgWater(0.0));
        // Requests per plant per root cell, from the same state.
        let mut asked: Vec<Vec<f64>> = Vec::with_capacity(plants.len());
        let mut capacity: Vec<f64> = Vec::with_capacity(plants.len());
        let mut cell_total: Vec<(usize, f64)> = Vec::new();
        for p in plants {
            let pot: Vec<f64> = p
                .roots
                .cells()
                .iter()
                .map(|&(i, l)| {
                    if !self.active[i] {
                        return 0.0;
                    }
                    let a = avail(self.water[i]).0;
                    (a * (p.rate.0 * l.0 / v))
                        .min(p.max_per_length.0 * l.0)
                        .min(a)
                })
                .collect();
            let total: f64 = pot.iter().sum();
            capacity.push(total);
            let want = p.demand.0.max(0.0).min(total);
            let share = if total > 0.0 { want / total } else { 0.0 };
            let req: Vec<f64> = pot.iter().map(|x| x * share).collect();
            for (&(i, _), r) in p.roots.cells().iter().zip(&req) {
                if *r > 0.0 {
                    cell_total.push((i, *r));
                }
            }
            asked.push(req);
        }
        // Total request per cell, summed in a fixed order (plant, then cell), on a
        // dense scratch grid. Every pushed request is positive, so a touched cell's
        // total is positive. Each total is then replaced by its cell's factor. An
        // untouched cell reads 0, but only requests of exactly zero read it, and
        // `0 · 0` and `0 · 1` are the same +0.
        let mut scale = vec![0.0_f64; self.water.len()];
        let mut touched: Vec<usize> = Vec::new();
        for (i, r) in cell_total {
            if scale[i] == 0.0 {
                touched.push(i);
                scale[i] = r;
            } else {
                scale[i] += r;
            }
        }
        for &i in &touched {
            let a = avail(self.water[i]).0;
            let total = scale[i];
            scale[i] = if total > a { a / total } else { 1.0 };
        }
        let factor = |i: usize| scale[i];
        let mut out = Vec::with_capacity(plants.len());
        let mut uptake = KgWater(0.0);
        for ((p, req), &cap) in plants.iter().zip(&asked).zip(&capacity) {
            let per_cell: Vec<(usize, KgWater)> = p
                .roots
                .cells()
                .iter()
                .zip(req)
                .map(|(&(i, _), r)| (i, KgWater(r * factor(i))))
                .collect();
            for &(i, u) in &per_cell {
                self.water[i] -= u;
            }
            let actual: KgWater = per_cell.iter().map(|c| c.1).sum();
            uptake += actual;
            out.push(Uptake {
                actual,
                shortfall: p.demand.max(KgWater(0.0)) - actual,
                per_cell,
                capacity: KgWater(cap),
            });
        }
        let ledger = WaterLedger {
            uptake,
            storage_before: before,
            storage_after: self.total_water(),
            ..WaterLedger::default()
        };
        debug_assert!(
            !totals || ledger.relative_residual() < CLOSURE_TOLERANCE,
            "{ledger:?}"
        );
        Ok(JointUptake {
            plants: out,
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
        for i in 0..self.water.len() {
            f(self.no3[i].to_bits());
            f(self.nh4[i].to_bits());
            f(self.organic_n[i].to_bits());
        }
        f(u64::from(self.rhizobia));
        if let Some(d) = self.divider {
            f(d as u64);
        }
    }
}

/// The fluxes of one explicit step of lateral exchange between active +x and +y
/// neighbors on each layer, `pair(v_from, v_to)` for every pair, from the state at the
/// start: `fx[i + 1]` runs from cell `i` to `i + 1` and `fy[i + nx]` from `i` to
/// `i + nx`. No pair across the end of a row or of a layer, or across a divider before
/// column `divider`. A missing pair's flux is +0.
fn lateral_fluxes<T: Copy>(
    v: &[T],
    active: &[bool],
    nx: usize,
    ny: usize,
    divider: Option<usize>,
    pair: impl Fn(T, T) -> f64,
) -> (Vec<f64>, Vec<f64>) {
    let n = v.len();
    let flux = |a: (&bool, &bool), w: (&T, &T)| {
        if *a.0 && *a.1 { pair(*w.0, *w.1) } else { 0.0 }
    };
    // Built by pushing rather than zeroed and then filled: a large zeroed allocation
    // costs a memset of fresh memory every call (the same values either way).
    let mut fx = Vec::with_capacity(n + 1);
    fx.push(0.0_f64);
    fx.extend(
        active
            .iter()
            .zip(&active[1..])
            .zip(v.iter().zip(&v[1..]))
            .map(|(a, w)| flux(a, w)),
    );
    fx.push(0.0);
    let mut fy = Vec::with_capacity(n + nx);
    fy.resize(nx, 0.0_f64);
    fy.extend(
        active
            .iter()
            .zip(&active[nx..])
            .zip(v.iter().zip(&v[nx..]))
            .map(|(a, w)| flux(a, w)),
    );
    fy.resize(n + nx, 0.0);
    debug_assert_eq!((fx.len(), fy.len()), (n + 1, n + nx));
    // No pairs across the end of a row or of a layer, or across the divider.
    for row in (0..n).step_by(nx) {
        fx[row] = 0.0;
        if let Some(d) = divider.filter(|d| *d < nx) {
            fx[row + d] = 0.0;
        }
        if (row / nx) % ny == ny - 1 {
            fy[row + nx..row + 2 * nx].fill(0.0);
        }
    }
    (fx, fy)
}

/// Applies the fluxes of [`lateral_fluxes`]: each cell adds its fluxes in the order a
/// scatter over cells in index order would, in from the −y and −x neighbors, out to +x
/// and +y. The running sum starts at +0 and is never −0, so a missing pair's +0 leaves
/// it unchanged. The loop has no data-dependent branches.
fn apply_fluxes<T: Copy>(
    v: &mut [T],
    nx: usize,
    fx: &[f64],
    fy: &[f64],
    get: impl Fn(T) -> f64,
    put: impl Fn(f64) -> T,
) {
    let cells = v
        .iter_mut()
        .zip(fx.windows(2))
        .zip(fy.iter().zip(&fy[nx..]));
    for ((out, x), (&y_in, &y_out)) in cells {
        let d = 0.0 + y_in + x[0] - x[1] - y_out;
        *out = put(get(*out) + d);
    }
}

fn cells_across(length: f64, cell: f64) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = libm::round(length / cell).max(1.0) as usize;
    n
}

#[cfg(test)]
mod tests;
