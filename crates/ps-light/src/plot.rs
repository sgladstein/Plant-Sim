//! One light grid for a plot (design plan §3.5 "Multiple plants", WP21, decision 0019).
//!
//! A plot's plants share one grid, so neighbors shade each other. The grid has fixed
//! 3 cm cells (larger only if a horizontal axis would need more than 96) instead of
//! the per-plant grid's 32 cells across the shoot, and its bottom sits at the ground.
//! The coefficients are the per-leaf rays of the pot model ([`crate::coefficients`]),
//! unchanged.
//!
//! The plan proposed per-voxel transmittance for plots. Measured on bean plants it was
//! no cheaper (about one leaflet per voxel, so a voxel per leaf) and less accurate, so
//! plots keep per-leaf rays (decision 0019).

use ps_graph::LightCoeffs;
use ps_units::Meters;

use crate::{LeafPoint, LightGrid, SkySector, as_f64, coefficients_with, leaf_radius};

/// Cell edge of a plot light grid, m (decision 0019).
pub const PLOT_CELL_M: f64 = 0.03;
/// Most cells along a horizontal axis of a plot light grid; a larger plot gets larger
/// cells.
pub const PLOT_MAX_CELLS: usize = 96;
/// Margin around the plants of a plot, m, so a low sun enters through empty cells.
pub const PLOT_MARGIN_M: f64 = 0.10;

impl LightGrid {
    /// The light grid of a plot (decision 0019): a box around `leaves` and `extra`
    /// points with a 10 cm horizontal margin and 3 cm cells, or larger cells if a
    /// horizontal axis would need more than 96. The grid's bottom is at the ground
    /// (`z = 0`) or below the lowest leaf. Returns `None` when there are no leaves.
    #[must_use]
    pub fn fit_plot(leaves: &[LeafPoint], extra: &[[f64; 3]], g: f64) -> Option<Self> {
        let mut grid = Self::fit_plot_geometry(leaves, extra, g)?;
        grid.add_leaves(leaves);
        Some(grid)
    }

    /// The empty grid [`LightGrid::fit_plot`] places around `leaves` and `extra`.
    fn fit_plot_geometry(leaves: &[LeafPoint], extra: &[[f64; 3]], g: f64) -> Option<Self> {
        if leaves.is_empty() {
            return None;
        }
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
        lo[2] = lo[2].min(0.0);
        let widest = (hi[0] - lo[0]).max(hi[1] - lo[1]) + 2.0 * PLOT_MARGIN_M;
        let cell = PLOT_CELL_M.max(widest / as_f64(PLOT_MAX_CELLS));
        let mut origin = [0.0; 3];
        let mut n = [1usize; 3];
        for a in 0..3 {
            let margin = if a < 2 { PLOT_MARGIN_M } else { 0.0 };
            let span = hi[a] - lo[a] + 2.0 * margin;
            // One cell more than the span needs, so a leaf on the top face is inside.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let cells = libm::floor(span / cell) as usize + 1;
            n[a] = cells;
            origin[a] = if a < 2 {
                0.5 * (lo[a] + hi[a]) - 0.5 * as_f64(cells) * cell
            } else {
                lo[a]
            };
        }
        Some(Self::with_geometry(origin, Meters(cell), n, g))
    }
}

/// Daily light rebuild for a plot (decision 0019): one [`LightGrid::fit_plot`] grid
/// around every plant's leaves, and one [`LightCoeffs`] per leaf in input order, with
/// the same meaning as [`crate::rebuild_daily`].
#[must_use]
pub fn rebuild_plot(
    leaves: &[LeafPoint],
    extra_bounds: &[[f64; 3]],
    sun_path: &[Option<[f64; 3]>; 24],
    sky: &[SkySector],
    g: f64,
) -> (Option<LightGrid>, Vec<LightCoeffs>) {
    let Some(mut grid) = LightGrid::fit_plot_geometry(leaves, extra_bounds, g) else {
        return (None, Vec::new());
    };
    // Footprints are computed once, for the grid and for the rays.
    let feet = grid.add_leaves_keeping(leaves);
    let coeffs = coefficients_with(&grid, leaves, &feet, sun_path, sky);
    (Some(grid), coeffs)
}

#[cfg(test)]
mod tests;
