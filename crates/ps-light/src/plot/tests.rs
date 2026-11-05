use super::*;
use crate::sky_sectors;
use ps_units::M2;

fn leaf(x: f64, y: f64, z: f64, area: f64) -> LeafPoint {
    LeafPoint {
        center: [x, y, z],
        area: M2(area),
        normal: [0.0, 0.0, 1.0],
    }
}

fn noon(d: [f64; 3]) -> [Option<[f64; 3]>; 24] {
    let mut s = [None; 24];
    s[12] = Some(d);
    s
}

fn direction(elev: f64, az: f64) -> [f64; 3] {
    let ce = libm::cos(elev);
    [ce * libm::sin(az), ce * libm::cos(az), libm::sin(elev)]
}

#[test]
fn the_plot_grid_has_fixed_cells_and_starts_at_the_ground() {
    let l = [leaf(0.3, 0.4, 0.2, 0.003), leaf(1.2, 1.0, 0.35, 0.003)];
    let g = LightGrid::fit_plot(&l, &[], 0.5).unwrap();
    assert!((g.cell.0 - PLOT_CELL_M).abs() < 1e-15);
    assert!(g.origin[2].abs() < 1e-15);
    for p in &l {
        assert!(g.locate(p.center).is_some());
    }
    // A wide plot gets larger cells instead of more than 96 per axis.
    let wide = [leaf(0.0, 0.0, 0.1, 0.003), leaf(4.0, 0.0, 0.1, 0.003)];
    let g = LightGrid::fit_plot(&wide, &[], 0.5).unwrap();
    assert!(g.n[0] <= PLOT_MAX_CELLS + 1 && g.cell.0 > PLOT_CELL_M);
}

#[test]
fn neighbors_in_a_plot_shade_each_other() {
    // A short leaf east of a tall one is shaded by it at a low western sun, and only
    // when both are in one grid.
    let tall = leaf(0.5, 0.5, 0.40, 0.004);
    let short = leaf(0.8, 0.5, 0.25, 0.004);
    let west_sun = noon(direction(0.46, -core::f64::consts::FRAC_PI_2));
    let (_, both) = rebuild_plot(&[tall, short], &[], &west_sun, &sky_sectors(), 0.5);
    let (_, alone) = rebuild_plot(&[short], &[], &west_sun, &sky_sectors(), 0.5);
    assert!(both[1].direct[12] < 0.9 * alone[0].direct[12], "{both:?}");
    assert!((alone[0].direct[12] - 1.0).abs() < 1e-12);
}
