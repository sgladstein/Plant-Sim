//! Fixed benchmark scenes for the sim's hot paths (docs/performance.md).
//!
//! Benches live in `benches/` and run with `cargo bench -p ps-bench`. The scenes here
//! are deterministic and built without randomness, so numbers compare across commits.

use ps_soil::{SoilGrid, SoilHour, Texture};
use ps_units::{Celsius, Meters, Millimeters, WPerM2};

/// Plants in the M2 plot budget (design plan §7, WP21: 30 plants under 20 ms per sim-day).
pub const PLOT_PLANTS: usize = 30;

/// A summer hour with light rain: exercises infiltration, drainage and evaporation.
#[must_use]
pub fn summer_hour() -> SoilHour {
    SoilHour {
        water_in: Millimeters(0.5),
        surface_radiation: WPerM2(300.0),
        t_air: Celsius(20.0),
    }
}

/// The default M1 pot: 20 cm diameter, 20 cm deep, 2 cm cells, loam (§3.6).
///
/// # Panics
/// Never: the geometry is fixed and valid.
#[must_use]
pub fn pot() -> SoilGrid {
    SoilGrid::new_pot(
        Meters(0.2),
        Meters(0.2),
        Meters(0.02),
        Texture::Loam,
        Celsius(15.0),
    )
    .expect("valid pot")
}

/// The largest M2 plot: 3 × 3 m, 1 m deep, 5 cm cells (60 × 60 × 20 = 72,000 cells).
///
/// # Panics
/// Never: the geometry is fixed and valid.
#[must_use]
pub fn plot() -> SoilGrid {
    SoilGrid::new_box(60, 60, 20, Meters(0.05), Texture::Loam, Celsius(15.0)).expect("valid plot")
}

/// Dense per-cell root length arrays (today's `ftsw` and `take_up` input) for `plants`
/// plants on a grid: plant `p` has 0.3 m of root in each cell of its own 30 cm × 30 cm
/// × 50 cm block, laid out 6 × 5 across a 60-cell-wide plot.
#[must_use]
pub fn dense_roots(grid: &SoilGrid, plants: usize) -> Vec<Vec<Meters>> {
    let (nx, ny, nz) = grid.dims();
    (0..plants)
        .map(|p| {
            let (bx, by) = (p % 6, p / 6);
            let mut r = vec![Meters(0.0); nx * ny * nz];
            for z in 0..nz.min(10) {
                for y in (by * 6)..((by + 1) * 6).min(ny) {
                    for x in (bx * 6)..((bx + 1) * 6).min(nx) {
                        r[(z * ny + y) * nx + x] = Meters(0.3);
                    }
                }
            }
            r
        })
        .collect()
}
