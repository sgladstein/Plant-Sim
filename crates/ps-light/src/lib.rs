//! Light: turbid-medium voxel model (design plan §3.5, §3.2 hourly step 3 and daily
//! step 6; WP6).
//!
//! Once a day, [`rebuild_daily`] fits an adaptive voxel grid around the shoot, spreads
//! each leaf's area over its footprint (leaf area density, m² m⁻³), and marches rays
//! from seven points on every leaf toward each of the day's 24 hourly sun positions and
//! each of 16 sky sectors. The optical depth along a ray is `Σ k · LAD · path length`,
//! where `k` is the G-function value for small leaves (Beer's law) and rises toward
//! `−ln(1 − G)` for a leaf that fills its cell layer, because one leaf is one opaque
//! blade rather than a cloud of area (decision 0008). The results are each leaf's
//! [`LightCoeffs`]: 24 direct transmittances (the probability the leaf is sunlit that
//! hour), one weighted diffuse transmittance, and the red:far-red proxy.
//!
//! Every hour, [`absorbed_par`] turns those coefficients and the hour's direct and
//! diffuse radiation into absorbed PAR per unit leaf area for the sunlit and shaded parts
//! of each leaf.
//!
//! A leaf's footprint is a horizontal disc of the leaf's area at its height, and a
//! leaf's own footprint is excluded when its rays are marched, so an isolated leaf is
//! fully lit while other foliage in the same cells still shades it. The seven ray
//! origins lie in the plane of the blade (its normal), so a tilted leaf half under a
//! neighbor is half shaded.
//!
//! Model card: docs/models/light.md.

use core::f64::consts::PI;

pub use ps_graph::LightCoeffs;
use ps_units::{M2, Meters, UmolPerM2S, WPerM2};
use serde::{Deserialize, Serialize};

/// Most cells along any axis (§3.5).
pub const MAX_CELLS_PER_AXIS: usize = 32;
/// Smallest cell edge (§3.5).
pub const MIN_CELL_M: f64 = 0.005;
/// Margin added to the shoot's bounding box, as a fraction of its extent per side
/// (20% in total per axis, §3.5).
pub const MARGIN_FRACTION_PER_SIDE: f64 = 0.10;
/// PAR share of global radiation ("~", 0.45 to 0.5 in the literature).
pub const PAR_FRACTION: f64 = 0.5;
/// Photon flux per watt of PAR, µmol J⁻¹ (daylight average; source in the light model card).
pub const UMOL_PER_J_PAR: f64 = 4.57;
/// Smallest sine of solar elevation used when projecting the beam onto leaves.
pub const MIN_SIN_ELEVATION: f64 = 0.05;

/// A leaf as the light model sees it: its center and one-sided area.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeafPoint {
    /// Leaf center (east, north, up), m.
    pub center: [f64; 3],
    /// One-sided leaf area.
    pub area: M2,
    /// Unit normal of the leaf blade (pointing up); `[0, 0, 1]` for a flat leaf.
    pub normal: [f64; 3],
}

/// A leaf's footprint on a grid and its optical depth there (see
/// [`LightGrid::footprint`] and [`LightGrid::own_depth`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Footprint {
    /// `(flat cell index, leaf area density)`, sorted by index.
    pub cells: Vec<(usize, f64)>,
    /// `(flat cell index, optical depth density)`, matching `cells`.
    pub depth: Vec<(usize, f64)>,
}

/// Radius of a disc with the leaf's area.
#[must_use]
pub fn leaf_radius(area: M2) -> f64 {
    libm::sqrt(area.0.max(0.0) / PI)
}

/// The voxel grid of one day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightGrid {
    /// Lower corner of the grid.
    pub origin: [f64; 3],
    /// Cell edge.
    pub cell: Meters,
    /// Cells per axis.
    pub n: [usize; 3],
    /// Leaf area density per cell, m² m⁻³, index `(k * ny + j) * nx + i`.
    pub lad: Vec<f64>,
    /// The G-function value the grid was built for.
    pub g: f64,
    /// Optical depth per unit path length per cell, m⁻¹: `Σ k_i · LAD_i` over the
    /// leaves in the cell, with each leaf's `k_i` from [`LightGrid::extinction`].
    pub depth: Vec<f64>,
}

#[allow(clippy::cast_precision_loss)]
fn as_f64(n: usize) -> f64 {
    n as f64
}

impl LightGrid {
    /// An empty grid with explicit geometry (tests and obstacles), for G-function
    /// value `g`.
    #[must_use]
    pub fn with_geometry(origin: [f64; 3], cell: Meters, n: [usize; 3], g: f64) -> Self {
        Self {
            origin,
            cell,
            n,
            lad: vec![0.0; n[0] * n[1] * n[2]],
            g,
            depth: vec![0.0; n[0] * n[1] * n[2]],
        }
    }

    /// The adaptive grid of §3.5 around `leaves` (and any `extra` points, such as stem
    /// tips), with 20% margin, at most 48 cells per axis and cells no smaller than 5 mm.
    /// Returns `None` when there are no leaves. `g` is the G-function value.
    #[must_use]
    pub fn fit(leaves: &[LeafPoint], extra: &[[f64; 3]], g: f64) -> Option<Self> {
        let mut grid = Self::fit_geometry(leaves, extra, g)?;
        grid.add_leaves(leaves);
        Some(grid)
    }

    /// The empty grid [`LightGrid::fit`] places around `leaves` and `extra`.
    fn fit_geometry(leaves: &[LeafPoint], extra: &[[f64; 3]], g: f64) -> Option<Self> {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for l in leaves {
            let r = leaf_radius(l.area);
            for a in 0..3 {
                let pad = if a < 2 { r } else { 0.0 };
                lo[a] = lo[a].min(l.center[a] - pad);
                hi[a] = hi[a].max(l.center[a] + pad);
            }
        }
        for p in extra {
            for a in 0..3 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
        }
        if leaves.is_empty() {
            return None;
        }
        let extent = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
        let largest = extent[0].max(extent[1]).max(extent[2]);
        let padded = largest * (1.0 + 2.0 * MARGIN_FRACTION_PER_SIDE);
        let cell = (padded / as_f64(MAX_CELLS_PER_AXIS)).max(MIN_CELL_M);
        let mut origin = [0.0; 3];
        let mut n = [1usize; 3];
        for a in 0..3 {
            let span = extent[a] * (1.0 + 2.0 * MARGIN_FRACTION_PER_SIDE);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let cells = (libm::ceil(span / cell).max(1.0) as usize).min(MAX_CELLS_PER_AXIS);
            n[a] = cells;
            let mid = 0.5 * (lo[a] + hi[a]);
            origin[a] = mid - 0.5 * as_f64(cells) * cell;
        }
        Some(Self::with_geometry(origin, Meters(cell), n, g))
    }

    /// [`LightGrid::fit`] that also returns each leaf's footprint and its
    /// [`LightGrid::own_depth`], in input order, so [`rebuild_daily`] does not compute
    /// them twice. The grid is identical to `fit`'s.
    #[must_use]
    pub fn fit_with_footprints(
        leaves: &[LeafPoint],
        extra: &[[f64; 3]],
        g: f64,
    ) -> Option<(Self, Vec<Footprint>)> {
        let mut grid = Self::fit_geometry(leaves, extra, g)?;
        let feet = grid.add_leaves_keeping(leaves);
        Some((grid, feet))
    }

    /// [`LightGrid::add_leaves`] that returns each leaf's footprint and own depth, in
    /// input order. The grid ends up identical to `add_leaves`'s.
    pub fn add_leaves_keeping(&mut self, leaves: &[LeafPoint]) -> Vec<Footprint> {
        use rayon::prelude::*;
        let grid = self;
        // Footprints in parallel (collect keeps leaf order); the additions below are
        // sequential, in leaf order.
        let feet = leaves
            .par_iter()
            .map(|l| {
                let cells = grid.footprint(l);
                let depth = grid.own_depth(&cells);
                Footprint { cells, depth }
            })
            .collect::<Vec<_>>();
        // Same additions in the same order as `add_leaves`.
        for f in &feet {
            for (&(i, v), &(_, d)) in f.cells.iter().zip(&f.depth) {
                grid.lad[i] += v;
                grid.depth[i] += d;
            }
        }
        feet
    }

    /// Cell volume, m³.
    #[must_use]
    pub fn cell_volume(&self) -> f64 {
        self.cell.0 * self.cell.0 * self.cell.0
    }

    /// Cell coordinates of a point, if inside the grid.
    #[must_use]
    pub fn locate(&self, p: [f64; 3]) -> Option<[usize; 3]> {
        let mut c = [0usize; 3];
        for a in 0..3 {
            let u = (p[a] - self.origin[a]) / self.cell.0;
            if !(u >= 0.0 && u < as_f64(self.n[a])) {
                return None;
            }
            // `u` is finite and nonnegative here, so truncation is `floor`.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let i = u as usize;
            c[a] = i.min(self.n[a] - 1);
        }
        Some(c)
    }

    /// Flat index of cell coordinates.
    #[must_use]
    pub fn flat(&self, c: [usize; 3]) -> usize {
        (c[2] * self.n[1] + c[1]) * self.n[0] + c[0]
    }

    /// A leaf's footprint: its area spread over a horizontal disc of the same area at
    /// the leaf's height, sampled on a lattice of spacing at most half a cell, as
    /// `(flat cell index, leaf area density)` pairs sorted by index. Samples outside the
    /// grid are dropped. (decision 0008: refines §3.5's "area at its center" so leaves
    /// larger than a cell do not concentrate their area in one cell.)
    #[must_use]
    pub fn footprint(&self, l: &LeafPoint) -> Vec<(usize, f64)> {
        let r = leaf_radius(l.area);
        let h = 0.5 * self.cell.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let k = (libm::ceil(r / h) as usize).max(1);
        let mut pts = Vec::new();
        for j in 0..2 * k {
            for i in 0..2 * k {
                let u = (as_f64(i) + 0.5) / as_f64(k) - 1.0;
                let w = (as_f64(j) + 0.5) / as_f64(k) - 1.0;
                if u * u + w * w <= 1.0 {
                    pts.push([l.center[0] + u * r, l.center[1] + w * r, l.center[2]]);
                }
            }
        }
        if pts.is_empty() {
            pts.push(l.center);
        }
        let share = l.area.0 / as_f64(pts.len()) / self.cell_volume();
        let mut cells: Vec<(usize, f64)> = Vec::new();
        for p in pts {
            if let Some(c) = self.locate(p) {
                cells.push((self.flat(c), share));
            }
        }
        cells.sort_by_key(|c| c.0);
        let mut merged: Vec<(usize, f64)> = Vec::with_capacity(cells.len());
        for (i, v) in cells {
            match merged.last_mut() {
                Some(last) if last.0 == i => last.1 += v,
                _ => merged.push((i, v)),
            }
        }
        merged
    }

    /// Adds each leaf's footprint to the grid.
    pub fn add_leaves(&mut self, leaves: &[LeafPoint]) {
        for l in leaves {
            for (i, v) in self.footprint(l) {
                self.lad[i] += v;
                self.depth[i] += self.extinction(v) * v;
            }
        }
    }

    /// Extinction coefficient of one leaf's area in one cell, where it has leaf area
    /// density `lad` (decision 0008). Within its footprint a leaf is one opaque blade,
    /// not a Poisson cloud of area: crossing a cell layer where the leaf covers a
    /// fraction `ΔL = lad · cell` of the layer, a ray is blocked with probability
    /// `G · ΔL` (the positive binomial of Nilson 1971 with one leaf per layer), so
    /// `k = −ln(1 − G·ΔL)/ΔL`. Small leaves (`ΔL → 0`) recover Beer's `k = G`.
    #[must_use]
    pub fn extinction(&self, lad: f64) -> f64 {
        let dl = (lad * self.cell.0).clamp(0.0, 1.0);
        let gdl = (self.g * dl).min(0.999);
        if gdl < 1e-9 {
            self.g
        } else {
            -libm::log(1.0 - gdl) / dl
        }
    }

    /// The optical depth of a leaf's own footprint, as `(cell, depth density)` pairs
    /// matching [`LightGrid::footprint`], for excluding it from its own rays.
    #[must_use]
    pub fn own_depth(&self, footprint: &[(usize, f64)]) -> Vec<(usize, f64)> {
        footprint
            .iter()
            .map(|&(i, v)| (i, self.extinction(v) * v))
            .collect()
    }

    /// `Σ LAD · length` along a ray (see [`LightGrid::path_sum`]): the leaf area index
    /// crossed.
    #[must_use]
    pub fn path_lad(&self, from: [f64; 3], dir: [f64; 3], own: &[(usize, f64)]) -> f64 {
        self.path_sum(&self.lad, from, dir, own)
    }

    /// Optical depth along a ray; the transmittance is `exp(−depth)`. `own` is the
    /// leaf's [`LightGrid::own_depth`].
    #[must_use]
    pub fn path_depth(&self, from: [f64; 3], dir: [f64; 3], own: &[(usize, f64)]) -> f64 {
        self.path_sum(&self.depth, from, dir, own)
    }

    /// Where a ray from outside the grid first enters it (slab method), nudged a hair
    /// inside, or `None` if it misses the grid.
    fn entry_point(&self, from: [f64; 3], dir: [f64; 3]) -> Option<[f64; 3]> {
        let (mut t0, mut t1) = (0.0_f64, f64::INFINITY);
        for a in 0..3 {
            let lo = self.origin[a];
            let hi = lo + as_f64(self.n[a]) * self.cell.0;
            if dir[a].abs() < 1e-15 {
                if from[a] < lo || from[a] >= hi {
                    return None;
                }
            } else {
                let (ta, tb) = ((lo - from[a]) / dir[a], (hi - from[a]) / dir[a]);
                t0 = t0.max(ta.min(tb));
                t1 = t1.min(ta.max(tb));
            }
        }
        if t0 > t1 {
            return None;
        }
        let t = t0 + 1e-9 * self.cell.0;
        Some([
            from[0] + t * dir[0],
            from[1] + t * dir[1],
            from[2] + t * dir[2],
        ])
    }

    /// `Σ field · length` along the ray from `from` in unit direction `dir` until it
    /// leaves the grid (Amanatides and Woo 1987 traversal). `own` is the leaf's own
    /// contribution to `field` (sorted by cell), excluded wherever the ray crosses it.
    /// Inclusive cell-coordinate bounds of the cells with nonzero optical depth (the
    /// whole grid when it is empty). A ray can stop once it leaves this box.
    #[must_use]
    pub fn occupied_bounds(&self) -> ([usize; 3], [usize; 3]) {
        let mut lo = self.n;
        let mut hi = [0usize; 3];
        for (i, d) in self.depth.iter().enumerate() {
            if *d != 0.0 {
                let c = [
                    i % self.n[0],
                    (i / self.n[0]) % self.n[1],
                    i / (self.n[0] * self.n[1]),
                ];
                for a in 0..3 {
                    lo[a] = lo[a].min(c[a]);
                    hi[a] = hi[a].max(c[a]);
                }
            }
        }
        if lo[0] > hi[0] {
            return ([0; 3], [self.n[0] - 1, self.n[1] - 1, self.n[2] - 1]);
        }
        (lo, hi)
    }

    /// Optical depth along a ray that stops once it leaves `bounds` (from
    /// [`LightGrid::occupied_bounds`]); bit-identical to [`LightGrid::path_depth`].
    #[must_use]
    pub fn path_depth_bounded(
        &self,
        from: [f64; 3],
        dir: [f64; 3],
        own: &[(usize, f64)],
        bounds: &([usize; 3], [usize; 3]),
    ) -> f64 {
        self.path_sum_b(&self.depth, from, dir, own, Some(bounds))
    }

    fn path_sum(&self, field: &[f64], from: [f64; 3], dir: [f64; 3], own: &[(usize, f64)]) -> f64 {
        self.path_sum_b(field, from, dir, own, None)
    }

    /// The traversal behind [`LightGrid::path_sum`], optionally stopping at `bounds`.
    fn path_sum_b(
        &self,
        field: &[f64],
        from: [f64; 3],
        dir: [f64; 3],
        own: &[(usize, f64)],
        bounds: Option<&([usize; 3], [usize; 3])>,
    ) -> f64 {
        // A ray that starts outside the grid (a ring point of a tilted blade can lie
        // just above or below the box) is advanced to where it enters the box.
        let (from, c) = if let Some(c) = self.locate(from) {
            (from, c)
        } else {
            let Some(p) = self.entry_point(from, dir) else {
                return 0.0;
            };
            let Some(c) = self.locate(p) else {
                return 0.0;
            };
            (p, c)
        };
        let s = self.cell.0;
        let mut step = [0isize; 3];
        let mut t_max = [f64::INFINITY; 3];
        let mut t_delta = [f64::INFINITY; 3];
        for a in 0..3 {
            if dir[a] > 0.0 {
                step[a] = 1;
                let boundary = self.origin[a] + as_f64(c[a] + 1) * s;
                t_max[a] = (boundary - from[a]) / dir[a];
                t_delta[a] = s / dir[a];
            } else if dir[a] < 0.0 {
                step[a] = -1;
                let boundary = self.origin[a] + as_f64(c[a]) * s;
                t_max[a] = (boundary - from[a]) / dir[a];
                t_delta[a] = -s / dir[a];
            }
        }
        let (lo, hi) = match bounds {
            Some((lo, hi)) => (*lo, *hi),
            None => ([0; 3], [self.n[0] - 1, self.n[1] - 1, self.n[2] - 1]),
        };
        // The own footprint is one horizontal layer: skip the search outside its range.
        let (own_lo, own_hi) = match (own.first(), own.last()) {
            (Some(a), Some(b)) => (a.0, b.0),
            _ => (1, 0),
        };
        let stride = [1, self.n[0], self.n[0] * self.n[1]];
        // Per axis: cells left before the ray leaves the bounds, and the index step.
        // (A zero direction component takes the negative branch, as before; its
        // `t_max` is infinite, so it is only chosen when the ray ends.)
        let mut left = [0usize; 3];
        let mut dstep = [0isize; 3];
        for a in 0..3 {
            #[allow(clippy::cast_possible_wrap)]
            let st = stride[a] as isize;
            if step[a] > 0 {
                left[a] = hi[a].saturating_sub(c[a]);
                dstep[a] = st;
            } else {
                left[a] = c[a].saturating_sub(lo[a]);
                dstep[a] = -st;
            }
        }
        let [mut tx, mut ty, mut tz] = t_max;
        let [dx, dy, dz] = t_delta;
        let mut idx = self.flat(c);
        let mut t = 0.0;
        let mut sum = 0.0;
        // Amanatides–Woo with the axes unrolled: the same comparisons, sums and
        // `t_max` updates, in the same order, as the indexed form.
        macro_rules! cross {
            ($tm:ident, $d:ident, $a:literal) => {{
                let t_next = $tm;
                let mut v = field[idx];
                if (own_lo..=own_hi).contains(&idx)
                    && let Ok(k) = own.binary_search_by_key(&idx, |o| o.0)
                {
                    v = (v - own[k].1).max(0.0);
                }
                sum += v * (t_next - t);
                t = t_next;
                if left[$a] == 0 {
                    break;
                }
                left[$a] -= 1;
                idx = idx.wrapping_add_signed(dstep[$a]);
                $tm += $d;
                if !t_next.is_finite() {
                    break;
                }
            }};
        }
        loop {
            if tx <= ty && tx <= tz {
                cross!(tx, dx, 0);
            } else if ty <= tz {
                cross!(ty, dy, 1);
            } else {
                cross!(tz, dz, 2);
            }
        }
        sum
    }
}

/// A sky sector for diffuse light: its central direction and its two weights.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkySector {
    /// Unit vector toward the sector center (east, north, up).
    pub direction: [f64; 3],
    /// Share of diffuse irradiance on a horizontal surface from this sector (sum 1).
    pub weight: f64,
    /// Radiance times solid angle of this sector, `∫ L cos β dβ dφ`, over the
    /// hemisphere's horizontal irradiance `∫ L sin β cos β dβ dφ` (sum 12/7 for the
    /// overcast sky). Times G, the sector's contribution to the diffuse flux absorbed
    /// per unit area of a randomly oriented leaf (decision 0008).
    pub absorb_weight: f64,
}

/// The 16 sky sectors of §3.5: two elevation bands (0° to 45°, 45° to 90°) × eight
/// azimuths, weighted for a standard overcast sky (radiance ∝ (1 + 2 sin β)/3, Moon and
/// Spencer 1942) integrated over each band for irradiance on a horizontal surface.
#[must_use]
pub fn sky_sectors() -> [SkySector; 16] {
    // Radiance (1 + 2 sin β)/3 times solid angle (cos β) per unit β and azimuth; times
    // sin β (projection) it is the irradiance on a horizontal surface.
    let radiance = |b: f64| (1.0 + 2.0 * libm::sin(b)) * libm::cos(b);
    // Midpoint integration of each band: horizontal irradiance, radiance times solid
    // angle, and the irradiance-weighted mean elevation.
    let band = |b0: f64, b1: f64| {
        let steps = 512u32;
        let h = (b1 - b0) / f64::from(steps);
        let (mut w, mut a, mut wb) = (0.0, 0.0, 0.0);
        for i in 0..steps {
            let b = b0 + (f64::from(i) + 0.5) * h;
            let irr = radiance(b) * libm::sin(b);
            w += irr * h;
            a += radiance(b) * h;
            wb += irr * b * h;
        }
        (w, a, wb / w)
    };
    let bands = [band(0.0, PI / 4.0), band(PI / 4.0, PI / 2.0)];
    let total = bands[0].0 + bands[1].0;
    let mut out = [SkySector {
        direction: [0.0, 0.0, 1.0],
        weight: 0.0,
        absorb_weight: 0.0,
    }; 16];
    for (b, (w, a, elev)) in bands.iter().enumerate() {
        for k in 0..8u32 {
            let az = (f64::from(k) + 0.5) * PI / 4.0;
            let ce = libm::cos(*elev);
            out[b * 8 + k as usize] = SkySector {
                direction: [ce * libm::sin(az), ce * libm::cos(az), libm::sin(*elev)],
                weight: w / total / 8.0,
                absorb_weight: a / total / 8.0,
            };
        }
    }
    out
}

/// Daily rebuild (§3.2 daily step 6): fits the grid and computes every leaf's
/// coefficients for the next day.
///
/// `sun_path[h]` is the unit vector toward the sun at the midpoint of hour `h`, or
/// `None` when the sun is below the horizon. `g` is the G-function value. Returns the
/// grid (for lenses and debugging) and one [`LightCoeffs`] per leaf, in input order.
#[must_use]
pub fn rebuild_daily(
    leaves: &[LeafPoint],
    extra_bounds: &[[f64; 3]],
    sun_path: &[Option<[f64; 3]>; 24],
    sky: &[SkySector],
    g: f64,
) -> (Option<LightGrid>, Vec<LightCoeffs>) {
    let Some((grid, feet)) = LightGrid::fit_with_footprints(leaves, extra_bounds, g) else {
        return (None, Vec::new());
    };
    let coeffs = coefficients_with(&grid, leaves, &feet, sun_path, sky);
    (Some(grid), coeffs)
}

/// Per-leaf coefficients on a given grid (which must already hold the leaves' area),
/// using the grid's G-function value.
#[must_use]
pub fn coefficients(
    grid: &LightGrid,
    leaves: &[LeafPoint],
    sun_path: &[Option<[f64; 3]>; 24],
    sky: &[SkySector],
) -> Vec<LightCoeffs> {
    let feet: Vec<Footprint> = leaves
        .iter()
        .map(|l| {
            let cells = grid.footprint(l);
            let depth = grid.own_depth(&cells);
            Footprint { cells, depth }
        })
        .collect();
    coefficients_with(grid, leaves, &feet, sun_path, sky)
}

/// [`coefficients`] with each leaf's footprint already computed on `grid`.
pub(crate) fn coefficients_with(
    grid: &LightGrid,
    leaves: &[LeafPoint],
    feet: &[Footprint],
    sun_path: &[Option<[f64; 3]>; 24],
    sky: &[SkySector],
) -> Vec<LightCoeffs> {
    use rayon::prelude::*;
    let bounds = grid.occupied_bounds();
    // Each leaf is independent; collect keeps the input order, so the result does not
    // depend on the thread count.
    leaves
        .par_iter()
        .zip(feet)
        .map(|(l, f)| {
            let own = &f.cells;
            let own_depth = &f.depth;
            let pts = sample_points(l);
            // Mean transmittance over the leaf's sample points (decision 0008).
            let trans = |d: [f64; 3]| {
                pts.iter()
                    .map(|p| libm::exp(-grid.path_depth_bounded(*p, d, own_depth, &bounds)))
                    .fold(0.0, |a, b| a + b)
                    / as_f64(pts.len())
            };
            let mut direct = [0.0; 24];
            for (h, sun) in sun_path.iter().enumerate() {
                if let Some(d) = sun {
                    direct[h] = trans(*d);
                }
            }
            let sky_t: Vec<f64> = sky.iter().map(|s| trans(s.direction)).collect();
            let diffuse = sky
                .iter()
                .zip(&sky_t)
                .map(|(s, t)| s.weight * t)
                .fold(0.0, |a, b| a + b);
            let diffuse_absorbed = grid.g
                * sky
                    .iter()
                    .zip(&sky_t)
                    .map(|(s, t)| s.absorb_weight * t)
                    .fold(0.0, |a, b| a + b);
            let lai_above = grid.path_lad(l.center, [0.0, 0.0, 1.0], own);
            LightCoeffs {
                direct,
                diffuse,
                diffuse_absorbed,
                sunlit_fraction: 0.0,
                rfr: red_far_red(lai_above),
            }
        })
        .collect()
}

/// Number of points on a leaf's disc from which rays are cast: the center and a ring.
pub const SAMPLE_POINTS: usize = 7;

/// The leaf's sample points: its center and six points on a ring at `r/√2` (the ring
/// that splits the disc into equal-area halves), in the plane of the leaf blade.
#[must_use]
pub fn sample_points(l: &LeafPoint) -> [[f64; 3]; SAMPLE_POINTS] {
    let rr = leaf_radius(l.area) * core::f64::consts::FRAC_1_SQRT_2;
    let (u, v) = blade_basis(l.normal);
    let mut out = [l.center; SAMPLE_POINTS];
    for (k, p) in out.iter_mut().skip(1).enumerate() {
        let a = as_f64(k) * PI / 3.0;
        let (ca, sa) = (libm::cos(a), libm::sin(a));
        for i in 0..3 {
            p[i] += rr * (ca * u[i] + sa * v[i]);
        }
    }
    out
}

/// Two orthonormal vectors spanning the blade plane of a leaf with normal `n`. A
/// zero or non-finite normal is treated as horizontal.
fn blade_basis(n: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let len = libm::sqrt(n[0] * n[0] + n[1] * n[1] + n[2] * n[2]);
    if !(len > 1e-12 && len.is_finite()) {
        return ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    }
    let n = [n[0] / len, n[1] / len, n[2] / len];
    let a = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let cross = |x: [f64; 3], y: [f64; 3]| {
        [
            x[1] * y[2] - x[2] * y[1],
            x[2] * y[0] - x[0] * y[2],
            x[0] * y[1] - x[1] * y[0],
        ]
    };
    let u = cross(n, a);
    let ul = libm::sqrt(u[0] * u[0] + u[1] * u[1] + u[2] * u[2]);
    let u = [u[0] / ul, u[1] / ul, u[2] / ul];
    (u, cross(n, u))
}

/// Red:far-red proxy from the leaf area index above a leaf (§3.5):
/// `1.2 · exp(−0.5 · LAI_above)`, clipped to [0.2, 1.2].
#[must_use]
pub fn red_far_red(lai_above: f64) -> f64 {
    (1.2 * libm::exp(-0.5 * lai_above)).clamp(0.2, 1.2)
}

/// Absorbed PAR of one leaf in one hour, per unit leaf area.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbsorbedPar {
    /// Share of the leaf that is sunlit this hour.
    pub sunlit_fraction: f64,
    /// Absorbed PAR on the sunlit part (diffuse plus beam).
    pub sunlit: UmolPerM2S,
    /// Absorbed PAR on the shaded part (diffuse only).
    pub shaded: UmolPerM2S,
}

impl AbsorbedPar {
    /// Area-weighted mean absorbed PAR over the leaf.
    #[must_use]
    pub fn mean(&self) -> UmolPerM2S {
        UmolPerM2S(
            self.sunlit_fraction * self.sunlit.0 + (1.0 - self.sunlit_fraction) * self.shaded.0,
        )
    }
}

/// Hourly step 3 of §3.2 for one leaf. `direct` and `diffuse` are horizontal
/// irradiances of global radiation; `sin_elevation` is the sine of the sun's elevation;
/// `absorptance` is leaf PAR absorptance.
///
/// Beam on the sunlit part, per unit leaf area: `G · I_b,horizontal / sin β`
/// (the beam's flux normal to the sun times the mean projection G). Diffuse on both
/// parts: `diffuse_absorbed · I_d,horizontal`, the same random-orientation convention
/// (decision 0008).
#[must_use]
pub fn absorbed_par(
    c: &LightCoeffs,
    hour: usize,
    direct: WPerM2,
    diffuse: WPerM2,
    sin_elevation: f64,
    g: f64,
    absorptance: f64,
) -> AbsorbedPar {
    let to_par = PAR_FRACTION * UMOL_PER_J_PAR * absorptance;
    let diffuse_par = c.diffuse_absorbed * diffuse.0.max(0.0) * to_par;
    let (fraction, beam) = if sin_elevation > 0.0 && direct.0 > 0.0 {
        let f = c.direct[hour.min(23)];
        let b = g * direct.0 / sin_elevation.max(MIN_SIN_ELEVATION) * to_par;
        (f, b)
    } else {
        (0.0, 0.0)
    };
    AbsorbedPar {
        sunlit_fraction: fraction,
        sunlit: UmolPerM2S(diffuse_par + beam),
        shaded: UmolPerM2S(diffuse_par),
    }
}

pub mod plot;
pub use plot::rebuild_plot;

#[cfg(test)]
mod tests;
