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

use std::sync::OnceLock;

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

/// Per-column constants.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Column<'a> {
    /// Cell height, m.
    pub dz: f64,
    /// The texture's head, slope and conductivity.
    pub table: &'a FlowTable,
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
    /// base. Water contents change only by the fluxes, so the column's water changes by
    /// exactly the bottom flow and what backed up out of the top.
    pub fn hour(&self, theta: &mut [f64], dt: f64, bottom: Bottom, s: &mut Scratch) -> ColumnHour {
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
        let wettest = self.table.retention.saturation + SPECIFIC_STORAGE_PER_M * height;
        let mut k = 1_u32;
        loop {
            theta.copy_from_slice(&s.start);
            s.faces.clear();
            s.faces.resize(nz.saturating_sub(1), 0.0);
            let mut out = 0.0;
            let mut backed = 0.0;
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
                let (change, q_bottom) = self.step(theta, dt / f64::from(k), base, s);
                out += q_bottom;
                if q_bottom < 0.0 {
                    limit = (limit + q_bottom).max(0.0);
                }
                worst = worst.max(change);
                backed += back_up(theta, wettest, self.dz, &mut s.faces);
                if change > MAX_THETA_STEP && k < MAX_SUBSTEPS {
                    ok = false;
                    break;
                }
            }
            if ok {
                return ColumnHour {
                    bottom: out,
                    backed_up: backed,
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

    /// One backward-Euler step of `dt` seconds. Adds each face's downward water (m) to
    /// the scratch faces and returns the largest change of a cell's water content and
    /// the water out of the base (m, negative in).
    fn step(&self, theta: &mut [f64], dt: f64, bottom: Bottom, s: &mut Scratch) -> (f64, f64) {
        let cells = s.h.iter_mut().zip(s.c.iter_mut().zip(s.k.iter_mut()));
        for (&t, (h, (c, k))) in theta.iter().zip(cells) {
            (*h, *c, *k) = self.table.terms(t);
        }
        let n = theta.len() - 1;
        let half = 0.5 * self.dz;
        let seeping = s.h[n] > -half;
        self.assemble(dt, s);
        match bottom {
            Bottom::Free => s.rhs[n] -= s.k[n],
            Bottom::Seepage if seeping => {
                s.diag[n] += s.k[n] / half;
                s.rhs[n] -= s.k[n];
            }
            Bottom::Seepage | Bottom::Closed => {}
            Bottom::Head { head, .. } => {
                let kb = 0.5 * (s.k[n] + self.table.ks);
                s.diag[n] += kb / half;
                s.rhs[n] += kb * head / half - kb;
            }
        }
        thomas(&s.sub, &mut s.diag, &s.sup, &mut s.rhs);
        let h_n = s.rhs[n];
        let q_bottom = match bottom {
            Bottom::Free => s.k[n],
            // A face can only let water out: if the solve ends below the face's
            // threshold, none leaves (the water stays in the column's cells).
            Bottom::Seepage if seeping => (s.k[n] * (h_n / half + 1.0)).max(0.0),
            Bottom::Seepage | Bottom::Closed => 0.0,
            Bottom::Head { head, limit } => {
                let kb = 0.5 * (s.k[n] + self.table.ks);
                let q = kb * ((h_n - head) / half + 1.0);
                if -q * dt > limit {
                    // A saucer gives no more than it holds: the base takes in exactly
                    // what is left, a fixed flux, and the column is solved again.
                    let q = -limit / dt;
                    self.assemble(dt, s);
                    s.rhs[n] -= q;
                    thomas(&s.sub, &mut s.diag, &s.sup, &mut s.rhs);
                    q
                } else {
                    q
                }
            }
        };
        self.apply(theta, dt, q_bottom, s)
    }

    /// The system for the interior faces from the scratch's heads, slopes and
    /// conductivities, with no flow at the base.
    fn assemble(&self, dt: f64, s: &mut Scratch) {
        let nz = s.h.len();
        let dz = self.dz;
        let cells = s.diag.iter_mut().zip(s.rhs.iter_mut());
        for ((d, r), (&c, &h)) in cells.zip(s.c.iter().zip(&s.h)) {
            let storage = c * dz / dt;
            *d = storage;
            *r = storage * h;
        }
        for f in 0..nz - 1 {
            let kf = 0.5 * (s.k[f] + s.k[f + 1]);
            let g = kf / dz;
            s.diag[f] += g;
            s.sup[f] = -g;
            s.rhs[f] -= kf;
            s.diag[f + 1] += g;
            s.sub[f + 1] = -g;
            s.rhs[f + 1] += kf;
        }
    }

    /// Moves the water by the fluxes of the solved heads (in the scratch's `rhs`) and
    /// `q_bottom` (m s⁻¹ out of the base) over `dt`; returns the largest change of a
    /// cell's water content and the water out of the base, m.
    fn apply(&self, theta: &mut [f64], dt: f64, q_bottom: f64, s: &mut Scratch) -> (f64, f64) {
        let n = theta.len() - 1;
        let dz = self.dz;
        let mut change = 0.0_f64;
        let mut above = 0.0;
        for (i, t) in theta.iter_mut().enumerate() {
            let below = if i < n {
                let kf = 0.5 * (s.k[i] + s.k[i + 1]);
                let q = kf * ((s.rhs[i] - s.rhs[i + 1]) / dz + 1.0);
                s.faces[i] += q * dt;
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

/// Pushes each cell's water above `wettest` into the cell above, bottom to top, and
/// returns what leaves the top cell, m of water. Each push is booked on its face as
/// upward water.
fn back_up(theta: &mut [f64], wettest: f64, dz: f64, faces: &mut [f64]) -> f64 {
    for i in (1..theta.len()).rev() {
        let excess = theta[i] - wettest;
        if excess > 0.0 {
            theta[i] = wettest;
            theta[i - 1] += excess;
            faces[i - 1] -= excess * dz;
        }
    }
    let excess = theta[0] - wettest;
    if excess > 0.0 {
        theta[0] = wettest;
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
