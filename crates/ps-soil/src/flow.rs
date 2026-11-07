//! Vertical water flow in each soil column (R5a, decision 0046): Darcy's law on total
//! head between neighboring cells, the Richards equation stepped once an hour.
//!
//! Each column is solved on its own. With heads `h` in metres (negative is suction),
//! the downward flux across the face between cells `i` and `i + 1` is
//! `q = K_f ((h_i − h_{i+1}) / Δz + 1)`, `K_f` the mean of the two cells' conductivities.
//! A step of `dt` is backward Euler with `K` and the retention slope `C = dθ/dh` taken at
//! the start (one Picard step): `C_i Δz (h'_i − h_i) / dt = q_in − q_out`, a tridiagonal
//! system solved by the Thomas algorithm. The water contents then move by the solved
//! fluxes themselves, so water is conserved exactly whatever the linearization's error.
//!
//! The retention curve's head, slope and conductivity are read from a table on even
//! steps of relative saturation ([`FlowTable`]; the head by cubic Hermite interpolation
//! on its exact slope, the rest linearly), since the curve's powers would otherwise be
//! most of the soil's cost.
//!
//! A saturated cell holds a little more water as its pressure rises (a specific
//! storage, [`SPECIFIC_STORAGE_PER_M`]), so a perched water table has a pressure the
//! solve can see. An hour whose single step would move any cell's water content by more
//! than [`MAX_THETA_STEP`] is split into equal sub-steps, at most [`MAX_SUBSTEPS`]. A
//! step that leaves a cell under more pressure than the column's own height of water
//! pushes the excess up and out of the top, back to the surface.
//!
//! Model card: docs/models/soil-and-water.md, "Water flow".

use std::sync::{Arc, OnceLock};

use crate::structure::SoilClass;
use crate::{Retention, Texture};

/// Kilopascals per metre of water head (`ρ g`, g = 9.80665 m s⁻²).
pub const KPA_PER_M_WATER: f64 = 9.806_65;
/// Water a saturated cell takes per metre of pressure head, m³ m⁻³ m⁻¹ ("~", a
/// numerical stand-in for the soil's compressibility, far above the real ~1e-5): at
/// the 0.24 m pressure under a full 24 cm pot it holds 0.0024 m³ m⁻³ above saturation.
pub const SPECIFIC_STORAGE_PER_M: f64 = 0.01;
/// Largest change of a cell's water content in one step, m³ m⁻³ ("~"): an hour that
/// would move more is split into sub-steps.
pub const MAX_THETA_STEP: f64 = 0.1;
/// Most sub-steps in an hour: the cost stays bounded, and a step at this count is kept
/// whatever its change.
pub const MAX_SUBSTEPS: u32 = 64;
/// Smallest retention slope used, per metre: keeps a row of the system regular in
/// air-dry soil, where the slope tends to 0.
const MIN_SLOPE_PER_M: f64 = 1e-12;
/// Steps of relative saturation in a [`FlowTable`], from half the air-dry content to
/// saturation.
const TABLE_STEPS: usize = 4096;

/// What happens at the bottom of a column during a step.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Bottom {
    /// Unit gradient: the soil continues below and water drains under gravity at the
    /// bottom cell's conductivity (a plot).
    Free,
    /// A seepage face (a pot with holes): water leaves only while the pressure at the
    /// base would be positive, `q = K_N (h_N / (Δz/2) + 1)` when `h_N > −Δz/2`.
    Seepage,
    /// No flow (a pot without holes).
    Closed,
    /// Free water at the base standing `head` metres deep (a saucer that holds water):
    /// `q = K_b ((h_N − head) / (Δz/2) + 1)` either way, `K_b` the mean of the bottom
    /// cell's and the saturated conductivity. Inflow over the step is at most
    /// `limit` metres of water.
    Head {
        /// Depth of the free water, m.
        head: f64,
        /// Most water the base can give, m over the column's area.
        limit: f64,
    },
}

/// A boundary of one step's system, as the flux across it depends on the head `h` of
/// the cell next to it.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Edge {
    /// No flow.
    Closed,
    /// A fixed downward flux, m s⁻¹.
    Flux(f64),
    /// Free water at the face (half a cell from the cell's center) under `head` metres
    /// of pressure, conductivity `k` (m s⁻¹).
    Head {
        /// Conductivity across the face, m s⁻¹.
        k: f64,
        /// Pressure head of the water at the face, m.
        head: f64,
    },
}

impl Edge {
    /// The downward flux across the edge, m s⁻¹, for the cell's head `h` and the
    /// face's height above the cell's center `rise` (`+Δz/2` at the top, `−Δz/2` at
    /// the base).
    fn down(self, h: f64, rise: f64) -> f64 {
        match self {
            Edge::Closed => 0.0,
            Edge::Flux(q) => q,
            Edge::Head { k, head } => k * ((head - h) / rise + 1.0),
        }
    }
}

/// A texture's head, retention slope and conductivity on even steps of relative
/// saturation `S` from `S_lo` (half the air-dry content) to 1. Between two steps the head
/// is the cubic Hermite interpolant on the two heads and their exact slopes `dh/dS =
/// θ_s / C`, so a hydrostatic profile set from the curve stays put; the slope `C` and the
/// conductivity are linear. Below `S_lo` the curve is evaluated directly.
#[derive(Clone, Debug)]
pub(crate) struct FlowTable {
    retention: Retention,
    ks: f64,
    s_lo: f64,
    ds: f64,
    inv_ds: f64,
    /// Per step: head (m), `dh/dS` (m), slope `C` (per m), conductivity (m s⁻¹).
    nodes: Vec<[f64; 4]>,
}

impl FlowTable {
    /// The table for a retention curve, a saturated conductivity `ks` (m s⁻¹) and an
    /// air-dry water content.
    pub fn new(retention: Retention, ks: f64, air_dry: f64) -> Self {
        let s_lo = (0.5 * air_dry / retention.saturation).clamp(1e-6, 0.5);
        #[allow(clippy::cast_precision_loss)] // a few thousand steps
        let ds = (1.0 - s_lo) / TABLE_STEPS as f64;
        let mut t = Self {
            retention,
            ks,
            s_lo,
            ds,
            inv_ds: 1.0 / ds,
            nodes: Vec::with_capacity(TABLE_STEPS + 1),
        };
        for i in 0..=TABLE_STEPS {
            #[allow(clippy::cast_precision_loss)]
            let s = (s_lo + ds * i as f64).min(1.0 - 1e-12);
            let (h, c, k) = t.exact(s * retention.saturation);
            t.nodes.push([h, retention.saturation / c, c, k]);
        }
        t
    }

    /// The table of one of the textures, built once.
    pub fn for_texture(texture: Texture) -> &'static Self {
        static TABLES: [OnceLock<FlowTable>; 3] = [const { OnceLock::new() }; 3];
        let i = match texture {
            Texture::Loam => 0,
            Texture::Sand => 1,
            Texture::Clay => 2,
        };
        TABLES[i].get_or_init(|| {
            let hy = texture.hydraulics();
            Self::new(texture.retention(), hy.saturated_conductivity, hy.air_dry)
        })
    }

    /// Head (m), retention slope (per m) and conductivity (m s⁻¹) from the curve itself,
    /// below saturation.
    fn exact(&self, theta: f64) -> (f64, f64, f64) {
        let (psi, slope, kr) = self.retention.flow_terms(theta);
        (
            psi.0 / KPA_PER_M_WATER,
            (slope * KPA_PER_M_WATER).max(MIN_SLOPE_PER_M),
            self.ks * kr,
        )
    }

    /// Head (m), retention slope (per m) and conductivity (m s⁻¹) at a water content.
    pub fn terms(&self, theta: f64) -> (f64, f64, f64) {
        let r = &self.retention;
        if theta >= r.saturation {
            let entry = -r.air_entry.0 / KPA_PER_M_WATER;
            let h = entry + (theta - r.saturation) / SPECIFIC_STORAGE_PER_M;
            return (h, SPECIFIC_STORAGE_PER_M, self.ks);
        }
        let x = (theta / r.saturation - self.s_lo) * self.inv_ds;
        if x.is_nan() || x < 0.0 {
            return self.exact(theta);
        }
        // x ≥ 0, so truncation is the floor (and a cast saturates past the table).
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let i = (x as usize).min(TABLE_STEPS - 1);
        #[allow(clippy::cast_precision_loss)]
        let t = x - i as f64;
        let [h0, d0, c0, k0] = self.nodes[i];
        let [h1, d1, c1, k1] = self.nodes[i + 1];
        let (t2, t3) = (t * t, t * t * t);
        let h = (2.0 * t3 - 3.0 * t2 + 1.0) * h0
            + (t3 - 2.0 * t2 + t) * self.ds * d0
            + (3.0 * t2 - 2.0 * t3) * h1
            + (t3 - t2) * self.ds * d1;
        (h, c0 + t * (c1 - c0), k0 + t * (k1 - k0))
    }
}

/// Each cell's table: one for the whole column, or one per cell from its class (R5c).
#[derive(Copy, Clone, Debug)]
pub(crate) enum Tables<'a> {
    /// Every cell has the texture's own table.
    Uniform(&'a FlowTable),
    /// Cell `z`'s table is that of `classes[of_cell[z]]`.
    PerCell {
        /// Each cell's class, top first.
        of_cell: &'a [u16],
        /// The classes.
        classes: &'a [Arc<SoilClass>],
    },
}

impl Tables<'_> {
    /// Cell `z`'s table.
    fn at(&self, z: usize) -> &FlowTable {
        match self {
            Tables::Uniform(t) => t,
            Tables::PerCell { of_cell, classes } => &classes[usize::from(of_cell[z])].table,
        }
    }
}

/// Per-column constants.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Column<'a> {
    /// Cell height, m.
    pub dz: f64,
    /// The cells' head, slope and conductivity.
    pub tables: Tables<'a>,
}

/// What a column's hour moved. The net downward water across each face over the hour
/// is in the scratch ([`Scratch::faces`]).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct ColumnHour {
    /// Net water out of the base over the hour, m (negative: taken in from a saucer).
    pub bottom: f64,
    /// Water pushed back out of the top over the hour, m: what a step left wetter than
    /// the column can hold ([`Column::hour`]); it goes back to the surface's pond.
    pub backed_up: f64,
    /// Water taken in through the top from the pond over the hour, m.
    pub from_pond: f64,
    /// Sub-steps the hour took.
    pub substeps: u32,
}

/// Scratch space for one column, reused across columns.
#[derive(Clone, Debug, Default)]
pub(crate) struct Scratch {
    start: Vec<f64>,
    h: Vec<f64>,
    c: Vec<f64>,
    k: Vec<f64>,
    sub: Vec<f64>,
    diag: Vec<f64>,
    sup: Vec<f64>,
    rhs: Vec<f64>,
    faces: Vec<f64>,
    wettest: Vec<f64>,
    /// Each interior face's conductivity, m s⁻¹.
    kf: Vec<f64>,
}

impl Scratch {
    /// Net downward water across the face between cells `i` and `i + 1` over the last
    /// hour, m of water (negative upward).
    pub fn faces(&self) -> &[f64] {
        &self.faces
    }
}

impl Column<'_> {
    /// Advances `theta` (top first) by one hour of `dt` seconds with `bottom` at the
    /// base and `pond` metres of water standing on the surface. Water contents change
    /// only by the fluxes, so the column's water changes by exactly the water taken from
    /// the pond, the bottom flow and what backed up out of the top.
    pub fn hour(
        &self,
        theta: &mut [f64],
        dt: f64,
        pond: f64,
        bottom: Bottom,
        s: &mut Scratch,
    ) -> ColumnHour {
        let nz = theta.len();
        s.start.clear();
        s.start.extend_from_slice(theta);
        for v in [
            &mut s.h,
            &mut s.c,
            &mut s.k,
            &mut s.sub,
            &mut s.diag,
            &mut s.sup,
            &mut s.rhs,
        ] {
            v.resize(nz, 0.0);
        }
        // A column holds at most the pressure of its own height of water. A step whose
        // linearization overshoots that (an unsaturated cell's slope is far larger than
        // the storage it has left) pushes the excess up, cell by cell, and out of the
        // top: the water backs up as it would against a full column.
        #[allow(clippy::cast_precision_loss)] // a column has a few dozen cells
        let height = (nz + 1) as f64 * self.dz;
        s.wettest.clear();
        s.wettest.extend(
            (0..nz)
                .map(|z| self.tables.at(z).retention.saturation + SPECIFIC_STORAGE_PER_M * height),
        );
        let mut k = 1_u32;
        loop {
            theta.copy_from_slice(&s.start);
            s.faces.clear();
            s.faces.resize(nz.saturating_sub(1), 0.0);
            let mut out = 0.0;
            let mut backed = 0.0;
            let mut standing = pond.max(0.0);
            let mut limit = match bottom {
                Bottom::Head { limit, .. } => limit,
                _ => 0.0,
            };
            let mut worst = 0.0_f64;
            let mut ok = true;
            for _ in 0..k {
                let base = match bottom {
                    Bottom::Head { head, .. } if limit > 0.0 => Bottom::Head { head, limit },
                    Bottom::Head { .. } => Bottom::Seepage,
                    b => b,
                };
                let (change, q_bottom, q_top) =
                    self.step(theta, dt / f64::from(k), standing, base, s);
                out += q_bottom;
                standing = (standing - q_top).max(0.0);
                if q_bottom < 0.0 {
                    limit = (limit + q_bottom).max(0.0);
                }
                worst = worst.max(change);
                backed += back_up(theta, &s.wettest, self.dz, &mut s.faces);
                if change > MAX_THETA_STEP && k < MAX_SUBSTEPS {
                    ok = false;
                    break;
                }
            }
            if ok {
                return ColumnHour {
                    bottom: out,
                    backed_up: backed,
                    from_pond: pond.max(0.0) - standing,
                    substeps: k,
                };
            }
            k = if k == 1 {
                // From the single step's change: the next power of two that would keep
                // each sub-step under the limit, at least two.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let need = libm::ceil(worst / MAX_THETA_STEP).min(f64::from(MAX_SUBSTEPS)) as u32;
                need.max(2).next_power_of_two().min(MAX_SUBSTEPS)
            } else {
                (k * 2).min(MAX_SUBSTEPS)
            };
        }
    }

    /// One backward-Euler step of `dt` seconds with `pond` metres of water standing on
    /// the surface. Adds each face's downward water (m) to the scratch faces and returns
    /// the largest change of a cell's water content, the water out of the base (m,
    /// negative in) and the water taken from the pond (m).
    ///
    /// The pond is a head at the surface, half a cell above the top cell's center, so
    /// water standing on a column soaks in during the step at the rate the column
    /// passes it, never more than the pond holds and never back out (decision 0050).
    /// Without it a ponded, nearly saturated column took in only the room the last
    /// step had made, and a soaked pot held its pond for two days. A seepage face
    /// closed at the start of a step opens if the solve leaves the base under pressure.
    fn step(
        &self,
        theta: &mut [f64],
        dt: f64,
        pond: f64,
        bottom: Bottom,
        s: &mut Scratch,
    ) -> (f64, f64, f64) {
        let cells = s.h.iter_mut().zip(s.c.iter_mut().zip(s.k.iter_mut()));
        match self.tables {
            Tables::Uniform(table) => {
                for (&t, (h, (c, k))) in theta.iter().zip(cells) {
                    (*h, *c, *k) = table.terms(t);
                }
            }
            tables @ Tables::PerCell { .. } => {
                for (z, (&t, (h, (c, k)))) in theta.iter().zip(cells).enumerate() {
                    (*h, *c, *k) = tables.at(z).terms(t);
                }
            }
        }
        self.face_conductivities(s);
        let n = theta.len() - 1;
        let half = 0.5 * self.dz;
        let seep = Edge::Head {
            k: s.k[n],
            head: 0.0,
        };
        let mut top = if pond > 0.0 {
            Edge::Head {
                k: 0.5 * (s.k[0] + self.tables.at(0).ks),
                head: pond,
            }
        } else {
            Edge::Closed
        };
        let mut base = match bottom {
            Bottom::Free => Edge::Flux(s.k[n]),
            Bottom::Seepage if s.h[n] > -half => seep,
            Bottom::Seepage | Bottom::Closed => Edge::Closed,
            Bottom::Head { head, .. } => Edge::Head {
                k: 0.5 * (s.k[n] + self.tables.at(n).ks),
                head,
            },
        };
        self.solve(dt, s, top, base);
        if bottom == Bottom::Seepage && base == Edge::Closed && s.rhs[n] > -half {
            base = seep;
            self.solve(dt, s, top, base);
        }
        // The pond gives no more than it holds and takes nothing back.
        let q_in = top.down(s.rhs[0], half);
        if q_in <= 0.0 && top != Edge::Closed {
            top = Edge::Closed;
            self.solve(dt, s, top, base);
        } else if q_in * dt > pond {
            top = Edge::Flux(pond / dt);
            self.solve(dt, s, top, base);
        }
        let q_top = top.down(s.rhs[0], half);
        let q_bottom = match bottom {
            // A face can only let water out: if the solve ends below the face's
            // threshold, none leaves (the water stays in the column's cells).
            Bottom::Seepage => base.down(s.rhs[n], -half).max(0.0),
            Bottom::Head { limit, .. } => {
                let q = base.down(s.rhs[n], -half);
                if -q * dt > limit {
                    // A saucer gives no more than it holds: the base takes in exactly
                    // what is left, a fixed flux, and the column is solved again.
                    base = Edge::Flux(-limit / dt);
                    self.solve(dt, s, top, base);
                    -limit / dt
                } else {
                    q
                }
            }
            Bottom::Free | Bottom::Closed => base.down(s.rhs[n], -half),
        };
        let (change, out) = self.apply(theta, dt, q_top, q_bottom, s);
        (change, out, q_top * dt)
    }

    /// Assembles the step's system with `top` at the surface and `base` at the bottom
    /// and solves it; the new heads are left in the scratch's `rhs`.
    fn solve(&self, dt: f64, s: &mut Scratch, top: Edge, base: Edge) {
        let n = s.h.len() - 1;
        let half = 0.5 * self.dz;
        self.assemble(dt, s);
        match top {
            Edge::Closed => {}
            Edge::Flux(q) => s.rhs[0] += q,
            Edge::Head { k, head } => {
                s.diag[0] += k / half;
                s.rhs[0] += k * head / half + k;
            }
        }
        match base {
            Edge::Closed => {}
            Edge::Flux(q) => s.rhs[n] -= q,
            Edge::Head { k, head } => {
                s.diag[n] += k / half;
                s.rhs[n] += k * head / half - k;
            }
        }
        thomas(&s.sub, &mut s.diag, &s.sup, &mut s.rhs);
    }

    /// Each interior face's conductivity from its two cells': their mean in one class;
    /// between classes (R5c), the harmonic mean of their saturated conductivities
    /// times the mean of their relative conductivities, so a dense layer limits what
    /// passes it as layers in series do, while a wetting front within a class moves as
    /// before.
    fn face_conductivities(&self, s: &mut Scratch) {
        let k = &s.k;
        s.kf.clear();
        match self.tables {
            Tables::Uniform(_) => s.kf.extend(k.windows(2).map(|w| 0.5 * (w[0] + w[1]))),
            tables @ Tables::PerCell { .. } => {
                s.kf.extend((0..k.len().saturating_sub(1)).map(|f| {
                    let (a, b) = (tables.at(f), tables.at(f + 1));
                    if std::ptr::eq(a, b) {
                        0.5 * (k[f] + k[f + 1])
                    } else {
                        let ks = 2.0 * a.ks * b.ks / (a.ks + b.ks);
                        ks * 0.5 * (k[f] / a.ks + k[f + 1] / b.ks)
                    }
                }));
            }
        }
    }

    /// The system for the interior faces from the scratch's heads, slopes and
    /// face conductivities, with no flow at the base.
    fn assemble(&self, dt: f64, s: &mut Scratch) {
        let nz = s.h.len();
        let dz = self.dz;
        // Slices of one length, so the face loop needs no bounds checks.
        let (kfs, sub, sup) = (&s.kf[..nz - 1], &mut s.sub[..nz], &mut s.sup[..nz]);
        let (diag, rhs) = (&mut s.diag[..nz], &mut s.rhs[..nz]);
        let cells = diag.iter_mut().zip(rhs.iter_mut());
        for ((d, r), (&c, &h)) in cells.zip(s.c.iter().zip(&s.h)) {
            let storage = c * dz / dt;
            *d = storage;
            *r = storage * h;
        }
        for (f, &kf) in kfs.iter().enumerate() {
            let g = kf / dz;
            diag[f] += g;
            sup[f] = -g;
            rhs[f] -= kf;
            diag[f + 1] += g;
            sub[f + 1] = -g;
            rhs[f + 1] += kf;
        }
    }

    /// Moves the water by the fluxes of the solved heads (in the scratch's `rhs`),
    /// `q_top` (m s⁻¹ into the top) and `q_bottom` (m s⁻¹ out of the base) over `dt`;
    /// returns the largest change of a cell's water content and the water out of the
    /// base, m.
    fn apply(
        &self,
        theta: &mut [f64],
        dt: f64,
        q_top: f64,
        q_bottom: f64,
        s: &mut Scratch,
    ) -> (f64, f64) {
        let n = theta.len() - 1;
        let dz = self.dz;
        let (kfs, h, faces) = (&s.kf[..n], &s.rhs[..=n], &mut s.faces[..n]);
        let mut change = 0.0_f64;
        let mut above = q_top;
        for (i, t) in theta.iter_mut().enumerate() {
            let below = if i < n {
                let kf = kfs[i];
                let q = kf * ((h[i] - h[i + 1]) / dz + 1.0);
                faces[i] += q * dt;
                q
            } else {
                q_bottom
            };
            let d = dt / dz * (above - below);
            *t += d;
            change = change.max(d.abs());
            above = below;
        }
        (change, q_bottom * dt)
    }
}

/// Pushes each cell's water above its `wettest` into the cell above, bottom to top, and
/// returns what leaves the top cell, m of water. Each push is booked on its face as
/// upward water.
fn back_up(theta: &mut [f64], wettest: &[f64], dz: f64, faces: &mut [f64]) -> f64 {
    for i in (1..theta.len()).rev() {
        let excess = theta[i] - wettest[i];
        if excess > 0.0 {
            theta[i] = wettest[i];
            theta[i - 1] += excess;
            faces[i - 1] -= excess * dz;
        }
    }
    let excess = theta[0] - wettest[0];
    if excess > 0.0 {
        theta[0] = wettest[0];
        excess * dz
    } else {
        0.0
    }
}

/// Solves a tridiagonal system in place by the Thomas algorithm: `sub[i] x[i−1] +
/// diag[i] x[i] + sup[i] x[i+1] = rhs[i]`, the solution left in `rhs`. The matrix is
/// diagonally dominant (storage terms are positive), so no pivoting is needed.
fn thomas(sub: &[f64], diag: &mut [f64], sup: &[f64], rhs: &mut [f64]) {
    let n = rhs.len();
    let (sub, diag, sup) = (&sub[..n], &mut diag[..n], &sup[..n]);
    for i in 1..n {
        let w = sub[i] / diag[i - 1];
        diag[i] -= w * sup[i - 1];
        rhs[i] -= w * rhs[i - 1];
    }
    rhs[n - 1] /= diag[n - 1];
    for i in (0..n - 1).rev() {
        rhs[i] = (rhs[i] - sup[i] * rhs[i + 1]) / diag[i];
    }
}
