//! Fixed benchmark scenes for the sim's hot paths (docs/performance.md).
//!
//! Benches live in `benches/` and run with `cargo bench -p ps-bench`. The scenes here
//! are deterministic and built without randomness, so numbers compare across commits.

use ps_light::LeafPoint;
use ps_soil::{RootCells, SoilGrid, SoilHour, Texture};
use ps_units::{Celsius, M2, Meters, Millimeters, WPerM2};

/// Plants in the M2 plot budget (design plan §7, WP21: 30 plants under 30 ms per sim-day, decision 0039).
pub const PLOT_PLANTS: usize = 30;

/// A summer hour with light rain: exercises infiltration, drainage and evaporation.
#[must_use]
pub fn summer_hour() -> SoilHour {
    SoilHour {
        water_in: Millimeters(0.5),
        surface_radiation: WPerM2(300.0),
        t_air: Celsius(20.0),
        strip: None,
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

/// Sparse root lists (the `ftsw` and `take_up` input) for `plants` plants on a grid:
/// plant `p` has 0.3 m of root in each cell of its own 30 cm × 30 cm × 50 cm block,
/// laid out 6 × 5 across a 60-cell-wide plot.
#[must_use]
pub fn plant_roots(grid: &SoilGrid, plants: usize) -> Vec<RootCells> {
    let (nx, ny, nz) = grid.dims();
    (0..plants)
        .map(|p| {
            let (bx, by) = (p % 6, p / 6);
            let mut r = Vec::new();
            for z in 0..nz.min(10) {
                for y in (by * 6)..((by + 1) * 6).min(ny) {
                    for x in (bx * 6)..((bx + 1) * 6).min(nx) {
                        r.push(((z * ny + y) * nx + x, Meters(0.3)));
                    }
                }
            }
            RootCells::from_pairs(r)
        })
        .collect()
}

/// A bush of `n` leaves of 40 cm² around `(cx, cy)`: a golden-angle spiral up to
/// 0.45 m tall with tilted blades. Deterministic.
#[must_use]
pub fn bush(n: usize, cx: f64, cy: f64) -> Vec<LeafPoint> {
    let ga = core::f64::consts::PI * (3.0 - libm::sqrt(5.0));
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let (fi, fn_) = (i as f64, n as f64);
            let f = (fi + 0.5) / fn_;
            let a = ga * fi;
            let r = 0.18 * libm::sqrt(f);
            let z = 0.05 + 0.40 * (1.0 - f * 0.6) * (0.5 + 0.5 * libm::sin(a * 0.37));
            let tilt = 0.4 * libm::cos(a * 1.3);
            LeafPoint {
                center: [cx + r * libm::cos(a), cy + r * libm::sin(a), z],
                area: M2(0.004),
                normal: [tilt * libm::cos(a), tilt * libm::sin(a), 1.0],
                strip: None,
            }
        })
        .collect()
}

/// Thirty bushes of `leaves_each` leaves on a 6 × 5 grid at 40 cm spacing (one merged
/// light grid, as in a plot).
#[must_use]
pub fn plot_canopy(leaves_each: usize) -> Vec<LeafPoint> {
    let mut out = Vec::with_capacity(PLOT_PLANTS * leaves_each);
    for i in 0..6u8 {
        for j in 0..5u8 {
            out.extend(bush(leaves_each, 0.4 * f64::from(i), 0.4 * f64::from(j)));
        }
    }
    out
}

/// Unit vectors toward the sun at each hour's midpoint, midsummer at 45° N (15
/// daylight hours); `None` below the horizon.
#[must_use]
pub fn summer_sun_path() -> [Option<[f64; 3]>; 24] {
    let deg = core::f64::consts::PI / 180.0;
    let (lat, dec) = (45.0 * deg, 23.0 * deg);
    let mut p = [None; 24];
    for (h, s) in p.iter_mut().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let ha = (h as f64 + 0.5 - 12.0) * core::f64::consts::PI / 12.0;
        let sin_e =
            libm::sin(lat) * libm::sin(dec) + libm::cos(lat) * libm::cos(dec) * libm::cos(ha);
        if sin_e > 0.0 {
            let ce = libm::sqrt(1.0 - sin_e * sin_e);
            let az = libm::atan2(
                libm::sin(ha),
                libm::cos(ha) * libm::sin(lat) - libm::tan(dec) * libm::cos(lat),
            ) + core::f64::consts::PI;
            *s = Some([ce * libm::sin(az), ce * libm::cos(az), sin_e]);
        }
    }
    p
}
