//! Obstacles: boxes that shade the light grid (design plan §3.5, "obstacles and
//! trellises are opaque or partly transmitting cells"; WP25, decision 0024).
//!
//! An obstacle is an axis-aligned box with an optical depth per unit path length. Its
//! optical depth is spread over the grid cells it overlaps, in proportion to the
//! overlap volume. A thin wall in a coarse cell then keeps its depth along a ray that
//! crosses it at right angles, at the cost of smearing it over the cell. Obstacles add
//! to the optical depth only: they are neutral in color, so they do not change the
//! red:far-red proxy, which counts leaf area.

use serde::{Deserialize, Serialize};

use crate::{LightGrid, as_f64};

/// Optical depth per metre of a box that lets `transmittance` through its thickness
/// `thickness` (m) at normal incidence. Transmittance is clamped to 1e-6 so an opaque box
/// stays finite.
#[must_use]
pub fn kappa_for(transmittance: f64, thickness: f64) -> f64 {
    let t = transmittance.clamp(1e-6, 1.0);
    -libm::log(t) / thickness.max(1e-6)
}

/// An axis-aligned box that shades: a wall, a fence, a shade cloth or a bucket.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Obstacle {
    /// Lower corner (east, north, up), m, in the soil frame.
    pub min: [f64; 3],
    /// Upper corner, m.
    pub max: [f64; 3],
    /// Optical depth per unit path length, m⁻¹ (see [`kappa_for`]).
    pub kappa: f64,
}

impl Obstacle {
    /// A box with `transmittance` through its thinnest side at normal incidence.
    #[must_use]
    pub fn with_transmittance(min: [f64; 3], max: [f64; 3], transmittance: f64) -> Self {
        let thin = (0..3)
            .map(|a| max[a] - min[a])
            .fold(f64::INFINITY, f64::min);
        Self {
            min,
            max,
            kappa: kappa_for(transmittance, thin),
        }
    }

    /// The eight corners, for fitting a grid that contains the box.
    #[must_use]
    pub fn corners(&self) -> [[f64; 3]; 8] {
        let (a, b) = (self.min, self.max);
        let mut out = [[0.0; 3]; 8];
        for (i, c) in out.iter_mut().enumerate() {
            *c = [
                if i & 1 == 0 { a[0] } else { b[0] },
                if i & 2 == 0 { a[1] } else { b[1] },
                if i & 4 == 0 { a[2] } else { b[2] },
            ];
        }
        out
    }
}

impl LightGrid {
    /// Adds each obstacle's optical depth to the cells it overlaps, weighted by the
    /// share of the cell's volume inside the box. Obstacles outside the grid shade
    /// nothing: fit the grid with their [`Obstacle::corners`] as extra bounds.
    pub fn add_obstacles(&mut self, obstacles: &[Obstacle]) {
        let c = self.cell.0;
        let vol = c * c * c;
        for o in obstacles {
            let mut range = [(0usize, 0usize); 3];
            let mut empty = false;
            for (a, r) in range.iter_mut().enumerate() {
                let lo = ((o.min[a] - self.origin[a]) / c).floor();
                let hi = ((o.max[a] - self.origin[a]) / c).ceil();
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let (lo, hi) = (lo.max(0.0) as usize, (hi.max(0.0) as usize).min(self.n[a]));
                if lo >= hi {
                    empty = true;
                }
                *r = (lo, hi);
            }
            if empty {
                continue;
            }
            for k in range[2].0..range[2].1 {
                for j in range[1].0..range[1].1 {
                    for i in range[0].0..range[0].1 {
                        let idx = [i, j, k];
                        let mut overlap = 1.0;
                        for (a, &ix) in idx.iter().enumerate() {
                            let lo = self.origin[a] + as_f64(ix) * c;
                            let w = (o.max[a].min(lo + c) - o.min[a].max(lo)).max(0.0);
                            overlap *= w;
                        }
                        let cell = (k * self.n[1] + j) * self.n[0] + i;
                        self.depth[cell] += o.kappa * overlap / vol;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ps_units::Meters;

    #[test]
    fn a_wall_keeps_its_depth_across_the_cells() {
        let mut g = LightGrid::with_geometry([0.0; 3], Meters(0.1), [10, 10, 10], 0.5);
        // A 1 cm wall at x = 0.42 to 0.43 with 10% transmittance.
        let o = Obstacle::with_transmittance([0.42, 0.0, 0.0], [0.43, 1.0, 1.0], 0.1);
        g.add_obstacles(&[o]);
        let d = g.path_depth([0.05, 0.55, 0.55], [1.0, 0.0, 0.0], &[]);
        assert!((d - (-libm::log(0.1))).abs() < 1e-9, "depth {d}");
        // A ray that misses the wall is clear.
        let d = g.path_depth([0.05, 0.55, 0.55], [0.0, 1.0, 0.0], &[]);
        assert!(d.abs() < 1e-12);
    }

    #[test]
    fn corners_span_the_box() {
        let o = Obstacle::with_transmittance([0.0, 1.0, 2.0], [3.0, 4.0, 5.0], 0.0);
        let c = o.corners();
        assert!(c.contains(&[0.0, 1.0, 2.0]) && c.contains(&[3.0, 4.0, 5.0]));
        assert!(o.kappa.is_finite() && o.kappa > 0.0);
    }
}
