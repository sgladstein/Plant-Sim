//! Obstacles: boxes that shade the light grid (design plan §3.5, "obstacles and
//! trellises are opaque or partly transmitting cells"; WP25, decision 0024).
//!
//! An obstacle is an axis-aligned box with an optical depth per unit path length. A
//! leaf's ray adds `κ` times the length of the ray inside each box (slab method), so an
//! obstacle is exact at any distance and does not touch the voxel grid, which stays
//! fitted to the leaves (review of WP25, decision 0025). Obstacles add to the optical
//! depth only: they are neutral in color, so they do not change the red:far-red proxy,
//! which counts leaf area.

use serde::{Deserialize, Serialize};

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
}

impl Obstacle {
    /// Optical depth along the ray from `from` in unit direction `dir` inside the box:
    /// `κ` times the length of the ray within it (slab method). Exact for any cell size
    /// and any distance from the grid.
    #[must_use]
    pub fn depth_along(&self, from: [f64; 3], dir: [f64; 3]) -> f64 {
        let (mut t0, mut t1) = (0.0_f64, f64::INFINITY);
        for a in 0..3 {
            if dir[a].abs() < 1e-15 {
                if from[a] < self.min[a] || from[a] > self.max[a] {
                    return 0.0;
                }
            } else {
                let ta = (self.min[a] - from[a]) / dir[a];
                let tb = (self.max[a] - from[a]) / dir[a];
                t0 = t0.max(ta.min(tb));
                t1 = t1.min(ta.max(tb));
            }
        }
        if t1 > t0 { self.kappa * (t1 - t0) } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Footprint, LightGrid};
    use ps_units::Meters;

    #[test]
    fn a_wall_keeps_its_depth_across_the_grid() {
        let dirs = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut g = LightGrid::with_geometry([0.0; 3], Meters(0.1), [10, 10, 10], &dirs);
        // A 1 cm wall at x = 0.42 to 0.43 with 10% transmittance.
        g.obstacles = vec![Obstacle::with_transmittance(
            [0.42, 0.0, 0.0],
            [0.43, 1.0, 1.0],
            0.1,
        )];
        let none = Footprint::none();
        let d = g.path_depth([0.05, 0.55, 0.55], 0, &none);
        assert!((d - (-libm::log(0.1))).abs() < 1e-9, "depth {d}");
        // A ray that misses the wall is clear.
        let d = g.path_depth([0.05, 0.55, 0.55], 1, &none);
        assert!(d.abs() < 1e-12);
    }

    #[test]
    fn an_obstacle_outside_the_grid_still_shades() {
        let up = [0.0, 0.0, 1.0];
        let mut g = LightGrid::with_geometry([0.0; 3], Meters(0.01), [10, 10, 10], &[up]);
        // A shade cloth 2 m up, far above the 10 cm grid, passing half the light.
        g.obstacles = vec![Obstacle::with_transmittance(
            [-1.0, -1.0, 2.0],
            [1.0, 1.0, 2.001],
            0.5,
        )];
        let d = g.path_depth([0.05, 0.05, 0.05], 0, &Footprint::none());
        assert!((d - core::f64::consts::LN_2).abs() < 1e-9, "{d}");
        // A slanted ray through a box gets its slant length.
        let o = Obstacle::with_transmittance([-5.0, -5.0, 1.0], [5.0, 5.0, 1.1], 0.5);
        let s = libm::sqrt(0.5);
        let d = o.depth_along([0.5, 0.5, 0.0], [s, 0.0, s]);
        assert!((d - core::f64::consts::LN_2 * core::f64::consts::SQRT_2).abs() < 1e-9);
        assert!(
            o.depth_along([0.5, 0.5, 2.0], up).abs() < 1e-15,
            "behind the ray"
        );
        assert!(o.kappa.is_finite() && o.kappa > 0.0);
    }
}
