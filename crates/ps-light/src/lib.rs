//! Light: turbid-medium voxel model (design plan §3.5, §3.2 hourly step 3 and daily
//! step 6; WP6).
//!
//! Once a day, [`rebuild_daily`] fits an adaptive voxel grid around the shoot, spreads
//! each leaf's area over its footprint (leaf area density, m² m⁻³), and marches rays
//! from seven points on every leaf toward each of the day's hourly sun positions and
//! each of 16 sky sectors. The optical depth along a ray in direction `d` is
//! `Σ k · LAD · path length`, where each leaf's `k` is its own projection `|n · d|`
//! (its blade normal `n`, decision 0025) for small leaves (Beer's law) and rises toward
//! `−ln(1 − |n · d|)` for a leaf that fills its cell layer, because one leaf is one
//! opaque blade rather than a cloud of area (decision 0008). The grid keeps one optical
//! depth per occupied cell for each ray direction of the day. The results are each
//! leaf's [`LightCoeffs`]: 24 direct transmittances (the probability the leaf is sunlit
//! that hour), the leaf's projection toward the sun each hour, one weighted diffuse
//! transmittance, the diffuse flux it absorbs, and the red:far-red proxy.
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
/// The slot of a cell that no leaf touches (see [`LightGrid::slot`]): slot 0 holds
/// zeros, so a ray reads an empty cell without a branch.
pub const EMPTY: u32 = 0;

/// A leaf as the light model sees it: its center and one-sided area.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeafPoint {
    /// Leaf center (east, north, up), m.
    pub center: [f64; 3],
    /// One-sided leaf area.
    pub area: M2,
    /// Unit normal of the leaf blade (pointing up); `[0, 0, 1]` for a flat leaf.
    pub normal: [f64; 3],
    /// A narrow strip (a third of a grass blade) instead of a broad leaf: the offset
    /// from its center to each of its two end sample points, along the strip. `None`
    /// for a broad leaf, sampled on a disc (decision 0030).
    #[serde(default)]
    pub strip: Option<[f64; 3]>,
}

/// A leaf's footprint on a grid and its optical depth there (see
/// [`LightGrid::footprint_of`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Footprint {
    /// `(flat cell index, leaf area density)`, sorted by index.
    pub cells: Vec<(usize, f64)>,
    /// For each cell, its row in `depth`: cells with the same leaf area density share
    /// one.
    pub row: Vec<u32>,
    /// Optical depth density of the leaf for each of the grid's directions, one row
    /// per distinct leaf area density: `depth[row[k] * directions + j]` for `cells[k]`
    /// and direction `j`.
    pub depth: Vec<f64>,
}

impl Footprint {
    /// A footprint with no cells: rays from a point that is not on a leaf.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            cells: Vec::new(),
            row: Vec::new(),
            depth: Vec::new(),
        }
    }

    /// The leaf's optical depth density in `cells[k]` toward direction `j` of a grid
    /// with `nd` directions.
    #[must_use]
    pub fn depth_at(&self, k: usize, j: usize, nd: usize) -> f64 {
        self.depth[self.row[k] as usize * nd + j]
    }
}

/// Radius of a disc with the leaf's area.
#[must_use]
pub fn leaf_radius(area: M2) -> f64 {
    libm::sqrt(area.0.max(0.0) / PI)
}

/// Projection of a unit area of a blade with normal `normal` onto the plane normal to
/// the unit direction `d`: `|n · d|`, the leaf's own G-function value toward `d`
/// (decision 0025). Both faces intercept light. A zero or non-finite normal counts as
/// horizontal, as in the blade's sample points.
#[must_use]
pub fn projection(normal: [f64; 3], d: [f64; 3]) -> f64 {
    let len = libm::sqrt(normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]);
    if !(len > 1e-12 && len.is_finite()) {
        return d[2].abs().min(1.0);
    }
    ((normal[0] * d[0] + normal[1] * d[1] + normal[2] * d[2]) / len)
        .abs()
        .min(1.0)
}

/// The voxel grid of one day.
///
/// Only cells that a leaf touches hold data: `slot` maps every cell to
/// its place in `lad` and `depth`, or to [`EMPTY`] (slot 0, all zeros). The optical depth depends on the
/// ray direction, because each leaf intercepts in proportion to its projection, so the
/// grid keeps one value per occupied cell and direction (decision 0025).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightGrid {
    /// Lower corner of the grid.
    pub origin: [f64; 3],
    /// Cell edge.
    pub cell: Meters,
    /// Cells per axis.
    pub n: [usize; 3],
    /// Unit ray directions (toward the light) the optical depth is kept for.
    pub directions: Vec<[f64; 3]>,
    /// Each cell's slot, index `(k * ny + j) * nx + i`, or [`EMPTY`].
    pub slot: Vec<u32>,
    /// Leaf area density per occupied cell, m² m⁻³, by slot.
    pub lad: Vec<f64>,
    /// Optical depth per unit path length, m⁻¹, at `j * lad.len() + slot` for
    /// direction `j`: `Σ k_i · LAD_i` over the leaves in the cell, with each leaf's
    /// `k_i` from [`LightGrid::extinction`] at its projection toward the direction.
    pub depth: Vec<f64>,
    /// Boxes that shade every ray that crosses them, inside the grid or not (WP25).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstacles: Vec<Obstacle>,
}

#[allow(clippy::cast_precision_loss)]
fn as_f64(n: usize) -> f64 {
    n as f64
}

impl LightGrid {
    /// An empty grid with explicit geometry (tests and obstacles) that keeps optical
    /// depth toward `directions`.
    #[must_use]
    pub fn with_geometry(
        origin: [f64; 3],
        cell: Meters,
        n: [usize; 3],
        directions: &[[f64; 3]],
    ) -> Self {
        Self {
            origin,
            cell,
            n,
            directions: directions.to_vec(),
            slot: vec![EMPTY; n[0] * n[1] * n[2]],
            lad: vec![0.0],
            depth: vec![0.0; directions.len()],
            obstacles: Vec::new(),
        }
    }

    /// The adaptive grid of §3.5 around `leaves` (and any `extra` points, such as stem
    /// tips), with 20% margin, at most 32 cells per axis and cells no smaller than 5 mm,
    /// keeping optical depth toward `directions`. Returns `None` when there are no
    /// leaves.
    #[must_use]
    pub fn fit(leaves: &[LeafPoint], extra: &[[f64; 3]], directions: &[[f64; 3]]) -> Option<Self> {
        let mut grid = Self::fit_geometry(leaves, extra, directions)?;
        grid.add_leaves(leaves);
        Some(grid)
    }

    /// The empty grid [`LightGrid::fit`] places around `leaves` and `extra`.
    fn fit_geometry(
        leaves: &[LeafPoint],
        extra: &[[f64; 3]],
        directions: &[[f64; 3]],
    ) -> Option<Self> {
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
        Some(Self::with_geometry(origin, Meters(cell), n, directions))
    }

    /// [`LightGrid::fit`] that also returns each leaf's footprint and its
    /// optical depth, in input order, so [`rebuild_daily`] does not compute
    /// them twice. The grid is identical to `fit`'s.
    #[must_use]
    pub fn fit_with_footprints(
        leaves: &[LeafPoint],
        extra: &[[f64; 3]],
        directions: &[[f64; 3]],
    ) -> Option<(Self, Vec<Footprint>)> {
        let mut grid = Self::fit_geometry(leaves, extra, directions)?;
        let feet = grid.add_leaves_keeping(leaves);
        Some((grid, feet))
    }

    /// The slot of cell `i`, made if the cell had none. A new slot's depth is added by
    /// [`LightGrid::grow_depth`].
    fn slot_of(&mut self, i: usize) -> usize {
        if self.slot[i] == EMPTY {
            let s = self.lad.len();
            // A grid has at most 96 × 96 × a few hundred cells, well below u32::MAX.
            self.slot[i] = u32::try_from(s).unwrap_or(u32::MAX);
            self.lad.push(0.0);
        }
        self.slot[i] as usize
    }

    /// Lays `depth` out again for the current number of slots, keeping its values:
    /// direction-major, so a ray's lookups stay within one direction's run.
    fn grow_depth(&mut self) {
        let (nd, ns) = (self.directions.len(), self.lad.len());
        let old = self.depth.len().checked_div(nd).unwrap_or(0);
        if old == ns {
            return;
        }
        let mut depth = vec![0.0; nd * ns];
        for j in 0..nd {
            depth[j * ns..j * ns + old].copy_from_slice(&self.depth[j * old..(j + 1) * old]);
        }
        self.depth = depth;
    }

    /// [`LightGrid::add_leaves`] that returns each leaf's footprint and own depth, in
    /// input order. The grid ends up identical to `add_leaves`'s.
    pub fn add_leaves_keeping(&mut self, leaves: &[LeafPoint]) -> Vec<Footprint> {
        use rayon::prelude::*;
        let grid = &*self;
        // Footprints in parallel (collect keeps leaf order); the additions below are
        // sequential, in leaf order.
        let feet = leaves
            .par_iter()
            .map(|l| grid.footprint_of(l))
            .collect::<Vec<_>>();
        for f in &feet {
            for &(i, _) in &f.cells {
                self.slot_of(i);
            }
        }
        self.grow_depth();
        let (nd, ns) = (self.directions.len(), self.lad.len());
        for f in &feet {
            for &(i, v) in &f.cells {
                self.lad[self.slot[i] as usize] += v;
            }
        }
        // Each direction's run in parallel; within a run the additions go in leaf order.
        let slot = &self.slot;
        self.depth
            .par_chunks_mut(ns)
            .enumerate()
            .for_each(|(j, run)| {
                for f in &feet {
                    for (k, &(i, _)) in f.cells.iter().enumerate() {
                        run[slot[i] as usize] += f.depth_at(k, j, nd);
                    }
                }
            });
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

    /// Leaf area density of cell `i`, m² m⁻³.
    #[must_use]
    pub fn lad_at(&self, i: usize) -> f64 {
        self.lad[self.slot[i] as usize]
    }

    /// Optical depth density of cell `i` toward direction `j`, m⁻¹.
    #[must_use]
    pub fn depth_at(&self, i: usize, j: usize) -> f64 {
        self.depth[j * self.lad.len() + self.slot[i] as usize]
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
        let _ = self.add_leaves_keeping(leaves);
    }

    /// Extinction coefficient of one leaf's area in one cell, where it has leaf area
    /// density `lad` and projection `g` toward the ray (decision 0008, 0025). Within its
    /// footprint a leaf is one opaque blade, not a Poisson cloud of area: crossing a
    /// cell layer where the leaf covers a fraction `ΔL = lad · cell` of the layer, a ray
    /// is blocked with probability `g · ΔL` (the positive binomial of Nilson 1971 with
    /// one leaf per layer), so `k = −ln(1 − g·ΔL)/ΔL`. Small leaves (`ΔL → 0`) recover
    /// Beer's `k = g`.
    #[must_use]
    pub fn extinction(&self, g: f64, lad: f64) -> f64 {
        let dl = (lad * self.cell.0).clamp(0.0, 1.0);
        let gdl = (g * dl).min(0.999);
        if gdl < 1e-9 {
            g
        } else {
            -libm::log(1.0 - gdl) / dl
        }
    }

    /// Leaf `l`'s [`LightGrid::footprint`] with its optical depth density there toward
    /// each of the grid's directions, for adding it to the grid and excluding it from
    /// its own rays.
    #[must_use]
    pub fn footprint_of(&self, l: &LeafPoint) -> Footprint {
        let cells = self.footprint(l);
        let g: Vec<f64> = self
            .directions
            .iter()
            .map(|d| projection(l.normal, *d))
            .collect();
        // Most cells of a footprint hold the same number of samples, so a handful of
        // distinct densities cover it.
        let mut seen: Vec<u64> = Vec::new();
        let mut depth = Vec::new();
        let row = cells
            .iter()
            .map(|&(_, v)| {
                let bits = v.to_bits();
                let r = seen.iter().position(|&b| b == bits).unwrap_or_else(|| {
                    seen.push(bits);
                    depth.extend(g.iter().map(|&gj| self.extinction(gj, v) * v));
                    seen.len() - 1
                });
                u32::try_from(r).unwrap_or(u32::MAX)
            })
            .collect();
        Footprint { cells, row, depth }
    }

    /// `Σ LAD · length` along a ray (see [`LightGrid::path_sum`]): the leaf area index
    /// crossed. `own` is the leaf's footprint, excluded where the ray crosses it.
    #[must_use]
    pub fn path_lad(&self, from: [f64; 3], dir: [f64; 3], own: &[(usize, f64)]) -> f64 {
        self.path_sum(|i| self.lad_at(i), from, dir, own, |k| own[k].1, None)
    }

    /// Optical depth along a ray toward the grid's direction `j`, through the leaves and
    /// any obstacles; the transmittance is `exp(−depth)`. `own` is the leaf's footprint
    /// ([`Footprint::none`] for a point that is not on a leaf).
    #[must_use]
    pub fn path_depth(&self, from: [f64; 3], j: usize, own: &Footprint) -> f64 {
        self.path_depth_b(from, j, own, None)
    }

    /// Optical depth along a ray that stops once it leaves `bounds` (from
    /// [`LightGrid::occupied_bounds`]); bit-identical to [`LightGrid::path_depth`].
    #[must_use]
    pub fn path_depth_bounded(
        &self,
        from: [f64; 3],
        j: usize,
        own: &Footprint,
        bounds: &([usize; 3], [usize; 3]),
    ) -> f64 {
        self.path_depth_b(from, j, own, Some(bounds))
    }

    fn path_depth_b(
        &self,
        from: [f64; 3],
        j: usize,
        own: &Footprint,
        bounds: Option<&([usize; 3], [usize; 3])>,
    ) -> f64 {
        let nd = self.directions.len();
        let dir = self.directions[j];
        let ns = self.lad.len();
        let run = &self.depth[j * ns..(j + 1) * ns];
        let leaves = self.path_sum(
            |i| run[self.slot[i] as usize],
            from,
            dir,
            &own.cells,
            |k| own.depth_at(k, j, nd),
            bounds,
        );
        self.obstacles
            .iter()
            .fold(leaves, |a, o| a + o.depth_along(from, dir))
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

    /// Inclusive cell-coordinate bounds of the occupied cells (the whole grid when it
    /// is empty). A ray can stop once it leaves this box.
    #[must_use]
    pub fn occupied_bounds(&self) -> ([usize; 3], [usize; 3]) {
        let mut lo = self.n;
        let mut hi = [0usize; 3];
        for (i, s) in self.slot.iter().enumerate() {
            if *s != EMPTY {
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

    /// `Σ field · length` along the ray from `from` in unit direction `dir` until it
    /// leaves the grid, or `bounds` if given (Amanatides and Woo 1987 traversal).
    /// `own` lists the leaf's own cells (sorted by index); wherever the ray crosses one,
    /// `own_value(k)` for `own[k]` is taken off the field.
    fn path_sum<F, O>(
        &self,
        field: F,
        from: [f64; 3],
        dir: [f64; 3],
        own: &[(usize, f64)],
        own_value: O,
        bounds: Option<&([usize; 3], [usize; 3])>,
    ) -> f64
    where
        F: Fn(usize) -> f64,
        O: Fn(usize) -> f64,
    {
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
        // (A zero direction component takes the negative branch; its `t_max` is
        // infinite, so it is only chosen when the ray ends.)
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
                let mut v = field(idx);
                if (own_lo..=own_hi).contains(&idx)
                    && let Ok(k) = own.binary_search_by_key(&idx, |o| o.0)
                {
                    v = (v - own_value(k)).max(0.0);
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
    /// overcast sky). Times a leaf's projection toward the sector, the sector's
    /// contribution to the diffuse flux the leaf absorbs per unit area (decisions 0008,
    /// 0025).
    pub absorb_weight: f64,
}

impl SkySector {
    /// The direction a leaf's projection toward the sector is taken at: the sector's
    /// azimuth, at the elevation whose sine is the radiance-weighted mean of `sin β` over
    /// the band (`weight / absorb_weight`), so a horizontal leaf in the open absorbs
    /// exactly the horizontal diffuse irradiance (decision 0025). The rays keep
    /// [`SkySector::direction`].
    #[must_use]
    pub fn absorb_direction(&self) -> [f64; 3] {
        let sin_b = if self.absorb_weight > 0.0 {
            (self.weight / self.absorb_weight).clamp(0.0, 1.0)
        } else {
            self.direction[2]
        };
        let cos_b = libm::sqrt(1.0 - sin_b * sin_b);
        let h = libm::sqrt(
            self.direction[0] * self.direction[0] + self.direction[1] * self.direction[1],
        );
        if h < 1e-12 {
            return [0.0, 0.0, 1.0];
        }
        [
            cos_b * self.direction[0] / h,
            cos_b * self.direction[1] / h,
            sin_b,
        ]
    }
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

/// The ray directions of one day: the sun's at each hour it is up, then the sky
/// sectors', with each one's place in the list.
#[derive(Clone, Debug, PartialEq)]
pub struct DayDirections {
    /// Every direction, sun first (in hour order), then sky.
    pub all: Vec<[f64; 3]>,
    /// The index in `all` of the sun at each hour, if it is up.
    pub sun: [Option<usize>; 24],
    /// The index in `all` of each sky sector.
    pub sky: Vec<usize>,
}

impl DayDirections {
    /// The directions of a day with this sun path and these sky sectors.
    #[must_use]
    pub fn new(sun_path: &[Option<[f64; 3]>; 24], sky: &[SkySector]) -> Self {
        let mut all = Vec::with_capacity(24 + sky.len());
        let mut sun = [None; 24];
        for (h, d) in sun_path.iter().enumerate() {
            if let Some(d) = d {
                sun[h] = Some(all.len());
                all.push(*d);
            }
        }
        let sky = sky
            .iter()
            .map(|s| {
                all.push(s.direction);
                all.len() - 1
            })
            .collect();
        Self { all, sun, sky }
    }
}

/// Daily rebuild (§3.2 daily step 6): fits the grid and computes every leaf's
/// coefficients for the next day.
///
/// `sun_path[h]` is the unit vector toward the sun at the midpoint of hour `h`, or
/// `None` when the sun is below the horizon. Returns the grid (for lenses and
/// debugging) and one [`LightCoeffs`] per leaf, in input order.
#[must_use]
pub fn rebuild_daily(
    leaves: &[LeafPoint],
    extra_bounds: &[[f64; 3]],
    sun_path: &[Option<[f64; 3]>; 24],
    sky: &[SkySector],
) -> (Option<LightGrid>, Vec<LightCoeffs>) {
    rebuild_daily_with(leaves, extra_bounds, &[], sun_path, sky)
}

/// [`rebuild_daily`] with obstacles, which shade every ray that crosses them (WP25;
/// they stay out of the grid, decision 0025).
#[must_use]
pub fn rebuild_daily_with(
    leaves: &[LeafPoint],
    extra_bounds: &[[f64; 3]],
    obstacles: &[Obstacle],
    sun_path: &[Option<[f64; 3]>; 24],
    sky: &[SkySector],
) -> (Option<LightGrid>, Vec<LightCoeffs>) {
    let dirs = DayDirections::new(sun_path, sky);
    let Some((mut grid, feet)) = LightGrid::fit_with_footprints(leaves, extra_bounds, &dirs.all)
    else {
        return (None, Vec::new());
    };
    grid.obstacles = obstacles.to_vec();
    let coeffs = coefficients_with(&grid, &dirs, leaves, &feet, sky);
    (Some(grid), coeffs)
}

/// Per-leaf coefficients on `grid`, which holds the leaves' area and was built for
/// `dirs`, with each leaf's footprint already computed on it.
pub(crate) fn coefficients_with(
    grid: &LightGrid,
    dirs: &DayDirections,
    leaves: &[LeafPoint],
    feet: &[Footprint],
    sky: &[SkySector],
) -> Vec<LightCoeffs> {
    use rayon::prelude::*;
    debug_assert_eq!(grid.directions, dirs.all, "grid built for these directions");
    let bounds = grid.occupied_bounds();
    let nd = dirs.all.len();
    let pts: Vec<Samples> = leaves.iter().map(sample_points).collect();
    // Leaves in chunks, in parallel; within a chunk, direction by direction, so a
    // thread's rays read one direction's depth at a time. Each leaf is independent and
    // collect keeps the input order, so the result does not depend on the thread count.
    let idx: Vec<usize> = (0..leaves.len()).collect();
    idx.par_chunks(COEFF_CHUNK)
        .flat_map_iter(|chunk| {
            let mut t = vec![0.0; chunk.len() * nd];
            for j in 0..nd {
                for (c, &i) in chunk.iter().enumerate() {
                    // Mean transmittance over the leaf's sample points (decision 0008).
                    let own = pts[i].points();
                    t[c * nd + j] = own
                        .iter()
                        .map(|p| libm::exp(-grid.path_depth_bounded(*p, j, &feet[i], &bounds)))
                        .fold(0.0, |a, b| a + b)
                        / as_f64(own.len());
                }
            }
            chunk
                .iter()
                .enumerate()
                .map(|(c, &i)| {
                    leaf_coeffs(
                        grid,
                        dirs,
                        sky,
                        &leaves[i],
                        &feet[i],
                        &t[c * nd..(c + 1) * nd],
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Leaves per parallel task in [`coefficients_with`].
const COEFF_CHUNK: usize = 16;

/// One leaf's coefficients from its transmittance toward each of the day's directions.
fn leaf_coeffs(
    grid: &LightGrid,
    dirs: &DayDirections,
    sky: &[SkySector],
    l: &LeafPoint,
    f: &Footprint,
    trans: &[f64],
) -> LightCoeffs {
    let mut direct = [0.0; 24];
    let mut beam = [0.0; 24];
    for (h, j) in dirs.sun.iter().enumerate() {
        if let Some(j) = *j {
            direct[h] = trans[j];
            beam[h] = projection(l.normal, dirs.all[j]);
        }
    }
    let diffuse = sky
        .iter()
        .zip(&dirs.sky)
        .map(|(s, &j)| s.weight * trans[j])
        .fold(0.0, |a, b| a + b);
    // The leaf's own projection toward each sector (decision 0025).
    let diffuse_absorbed = sky
        .iter()
        .zip(&dirs.sky)
        .map(|(s, &j)| s.absorb_weight * projection(l.normal, s.absorb_direction()) * trans[j])
        .fold(0.0, |a, b| a + b);
    let lai_above = grid.path_lad(l.center, [0.0, 0.0, 1.0], &f.cells);
    LightCoeffs {
        direct,
        beam,
        diffuse,
        diffuse_absorbed,
        sunlit_fraction: 0.0,
        rfr: red_far_red(lai_above),
    }
}

/// Number of points on a leaf's disc from which rays are cast: the center and a ring.
pub const SAMPLE_POINTS: usize = 7;
/// Number of points along a narrow strip (a third of a grass blade) from which rays are
/// cast: its center and one toward each end (decision 0030).
pub const STRIP_SAMPLE_POINTS: usize = 3;

/// A leaf's sample points: up to [`SAMPLE_POINTS`] of them.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Samples {
    at: [[f64; 3]; SAMPLE_POINTS],
    len: usize,
}

impl Samples {
    /// The points.
    #[must_use]
    pub fn points(&self) -> &[[f64; 3]] {
        &self.at[..self.len]
    }
}

/// The leaf's sample points. A broad leaf: its center and six points on a ring at
/// `r/√2` (the ring that splits the disc into equal-area halves), in the plane of the
/// leaf blade. A narrow strip: its center and the two points its `strip` offset puts
/// along it (decision 0030).
#[must_use]
pub fn sample_points(l: &LeafPoint) -> Samples {
    let mut at = [l.center; SAMPLE_POINTS];
    if let Some(o) = l.strip {
        for i in 0..3 {
            at[1][i] += o[i];
            at[2][i] -= o[i];
        }
        return Samples {
            at,
            len: STRIP_SAMPLE_POINTS,
        };
    }
    Samples {
        at: disc_points(l),
        len: SAMPLE_POINTS,
    }
}

fn disc_points(l: &LeafPoint) -> [[f64; 3]; SAMPLE_POINTS] {
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
/// Beam on the sunlit part, per unit leaf area: `|n · s| · I_b,horizontal / sin β`
/// (the beam's flux normal to the sun times the leaf's projection toward it,
/// `LightCoeffs::beam`, decision 0025). Diffuse on both parts:
/// `diffuse_absorbed · I_d,horizontal` (decision 0008).
#[must_use]
pub fn absorbed_par(
    c: &LightCoeffs,
    hour: usize,
    direct: WPerM2,
    diffuse: WPerM2,
    sin_elevation: f64,
    absorptance: f64,
) -> AbsorbedPar {
    let to_par = PAR_FRACTION * UMOL_PER_J_PAR * absorptance;
    let diffuse_par = c.diffuse_absorbed * diffuse.0.max(0.0) * to_par;
    let (fraction, beam) = if sin_elevation > 0.0 && direct.0 > 0.0 {
        let h = hour.min(23);
        let b = c.beam[h] * direct.0 / sin_elevation.max(MIN_SIN_ELEVATION) * to_par;
        (c.direct[h], b)
    } else {
        (0.0, 0.0)
    };
    AbsorbedPar {
        sunlit_fraction: fraction,
        sunlit: UmolPerM2S(diffuse_par + beam),
        shaded: UmolPerM2S(diffuse_par),
    }
}

pub mod obstacle;
pub mod plot;
pub use obstacle::Obstacle;
pub use plot::{rebuild_plot, rebuild_plot_with};

#[cfg(test)]
mod tests;
