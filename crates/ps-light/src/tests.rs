use super::*;

/// A horizontally uniform canopy on an explicit grid: `nz` layers of `n × n` cells,
/// 8 leaves per cell on a 2 × 2 × 2 sub-lattice, total leaf area index `lai`.
fn uniform_canopy(n: usize, nz: usize, cell: f64, lai: f64) -> (LightGrid, Vec<LeafPoint>) {
    let mut g = LightGrid::with_geometry([0.0, 0.0, 0.0], Meters(cell), [n, n, nz], 0.5);
    let layer_lai = lai / as_f64(nz);
    let leaf_area = layer_lai * cell * cell / 8.0;
    let mut leaves = Vec::new();
    for k in 0..nz {
        for j in 0..n {
            for i in 0..n {
                for s in 0..8u32 {
                    let off = |b: u32| if s & b == 0 { 0.25 } else { 0.75 };
                    leaves.push(LeafPoint {
                        center: [
                            (as_f64(i) + off(1)) * cell,
                            (as_f64(j) + off(2)) * cell,
                            (as_f64(k) + off(4)) * cell,
                        ],
                        area: M2(leaf_area),
                        normal: [0.0, 0.0, 1.0],
                    });
                }
            }
        }
    }
    g.add_leaves(&leaves);
    (g, leaves)
}

fn unit(elev_deg: f64, az_deg: f64) -> [f64; 3] {
    let e = elev_deg.to_radians();
    let a = az_deg.to_radians();
    [
        libm::cos(e) * libm::sin(a),
        libm::cos(e) * libm::cos(a),
        libm::sin(e),
    ]
}

/// Absorbed fraction of a beam at elevation `elev` by the central part of the canopy.
fn absorbed_fraction(n: usize, nz: usize, cell: f64, lai: f64, elev: f64) -> f64 {
    let (g, leaves) = uniform_canopy(n, nz, cell, lai);
    let d = unit(elev, 30.0);
    let half = 0.5 * as_f64(n) * cell;
    let r = 0.2 * as_f64(n) * cell;
    let mut absorbed = 0.0;
    for l in &leaves {
        if (l.center[0] - half).abs() < r && (l.center[1] - half).abs() < r {
            let t = libm::exp(-g.path_depth(l.center, d, &g.own_depth(&g.footprint(l))));
            absorbed += l.area.0 * 0.5 / libm::sin(elev.to_radians()) * t;
        }
    }
    absorbed / (2.0 * r * 2.0 * r)
}

/// WP6 done-when 1: a uniform canopy absorbs within 2% of Beer's law, LAI 0.5 to 6, G = 0.5.
#[test]
fn uniform_canopy_matches_beers_law() {
    for &lai in &[0.5, 1.0, 2.0, 4.0, 6.0] {
        for &elev in &[90.0, 60.0, 45.0] {
            let got = absorbed_fraction(40, 12, 0.04, lai, elev);
            let expect = 1.0 - libm::exp(-0.5 * lai / libm::sin(f64::to_radians(elev)));
            let rel = got / expect - 1.0;
            assert!(
                rel.abs() < 0.02,
                "LAI {lai}, elevation {elev}: {got:.4} vs {expect:.4} ({rel:+.4})"
            );
        }
    }
}

#[test]
fn isolated_leaf_is_fully_lit() {
    let leaves = [LeafPoint {
        center: [0.0, 0.0, 0.1],
        area: M2(0.0075),
        normal: [0.0, 0.0, 1.0],
    }];
    let mut sun = [None; 24];
    sun[12] = Some([0.0, 0.0, 1.0]);
    sun[8] = Some(unit(20.0, 100.0));
    let (grid, c) = rebuild_daily(&leaves, &[[0.0, 0.0, 0.0]], &sun, &sky_sectors(), 0.5);
    assert!(grid.is_some());
    assert!((c[0].direct[12] - 1.0).abs() < 1e-12);
    assert!((c[0].direct[8] - 1.0).abs() < 1e-12);
    assert!(c[0].direct[0].abs() < 1e-15, "night");
    assert!((c[0].diffuse - 1.0).abs() < 1e-12);
    // A random leaf in the open under an overcast sky: (12/7) G of horizontal diffuse.
    assert!((c[0].diffuse_absorbed - 12.0 / 7.0 * 0.5).abs() < 1e-4);
    assert!((c[0].rfr - 1.2).abs() < 1e-12);
}

#[test]
fn leaf_under_another_is_shaded() {
    let leaves = [
        LeafPoint {
            center: [0.0, 0.0, 0.15],
            area: M2(0.0075),
            normal: [0.0, 0.0, 1.0],
        },
        LeafPoint {
            center: [0.0, 0.0, 0.10],
            area: M2(0.0075),
            normal: [0.0, 0.0, 1.0],
        },
    ];
    let mut sun = [None; 24];
    sun[12] = Some([0.0, 0.0, 1.0]);
    let (_, c) = rebuild_daily(&leaves, &[], &sun, &sky_sectors(), 0.5);
    assert!((c[0].direct[12] - 1.0).abs() < 1e-12);
    // One opaque leaf above with G = 0.5 casts a shadow over half its footprint:
    // transmittance 1 − G = 0.5 (Beer's law would give exp(−0.5) = 0.61).
    assert!((c[1].direct[12] - 0.5).abs() < 0.03, "{}", c[1].direct[12]);
    assert!(
        c[1].diffuse < c[0].diffuse,
        "{} vs {}",
        c[1].diffuse,
        c[0].diffuse
    );
    assert!(c[1].rfr < c[0].rfr);
}

#[test]
fn grid_respects_limits() {
    // Seedling: millimeter cells clamp at 5 mm.
    let tiny = [
        LeafPoint {
            center: [0.0, 0.0, 0.02],
            area: M2(1e-4),
            normal: [0.0, 0.0, 1.0],
        },
        LeafPoint {
            center: [0.004, 0.0, 0.025],
            area: M2(1e-4),
            normal: [0.0, 0.0, 1.0],
        },
    ];
    let g = LightGrid::fit(&tiny, &[], 0.5).unwrap();
    assert!((g.cell.0 - MIN_CELL_M).abs() < 1e-15);
    // Mature plant 1 m tall: at most 32 cells per axis (decision 0010).
    let tall = [
        LeafPoint {
            center: [0.0, 0.0, 0.0],
            area: M2(0.01),
            normal: [0.0, 0.0, 1.0],
        },
        LeafPoint {
            center: [0.3, -0.2, 1.0],
            area: M2(0.01),
            normal: [0.0, 0.0, 1.0],
        },
    ];
    let g = LightGrid::fit(&tall, &[], 0.5).unwrap();
    assert!(g.n.iter().all(|&n| n <= MAX_CELLS_PER_AXIS));
    assert!(g.locate(tall[0].center).is_some() && g.locate(tall[1].center).is_some());
    let total: f64 = g.lad.iter().sum::<f64>() * g.cell_volume();
    assert!((total - 0.02).abs() < 1e-15);
    assert!(LightGrid::fit(&[], &[], 0.5).is_none());
}

#[test]
fn sky_weights_sum_to_one() {
    let s = sky_sectors();
    let sum: f64 = s.iter().map(|x| x.weight).sum();
    assert!((sum - 1.0).abs() < 1e-12);
    for x in &s {
        let norm = x.direction.iter().map(|v| v * v).sum::<f64>();
        assert!((norm - 1.0).abs() < 1e-12 && x.direction[2] > 0.0);
    }
    // Radiance times solid angle over horizontal irradiance: ∫(1+2 sin β) cos β dβ = 2
    // over ∫(1+2 sin β) sin β cos β dβ = 7/6, so 12/7 (midpoint rule to 1e-5).
    let abs_sum: f64 = s.iter().map(|x| x.absorb_weight).sum();
    assert!((abs_sum - 12.0 / 7.0).abs() < 1e-5, "{abs_sum}");
    // The upper band carries more of the horizontal irradiance under an overcast sky.
    assert!(s[8].weight > s[0].weight);
}

#[test]
fn ray_from_below_the_grid_enters_it() {
    // A leaf layer at mid-height; a ray from below the grid must still cross it.
    let (g, _) = uniform_canopy(4, 2, 0.04, 2.0);
    let inside = g.path_lad([0.08, 0.08, 0.0], [0.0, 0.0, 1.0], &[]);
    let below = g.path_lad([0.08, 0.08, -0.05], [0.0, 0.0, 1.0], &[]);
    assert!(
        (inside - below).abs() < 1e-6 && below > 1.9,
        "{inside} vs {below}"
    );
    // A ray that misses the box crosses nothing.
    assert!(g.path_lad([1.0, 1.0, -0.05], [0.0, 0.0, 1.0], &[]).abs() < 1e-15);
}

#[test]
fn rfr_proxy() {
    assert!((red_far_red(0.0) - 1.2).abs() < 1e-15);
    assert!((red_far_red(1.0) - 1.2 * libm::exp(-0.5)).abs() < 1e-15);
    assert!((red_far_red(10.0) - 0.2).abs() < 1e-15);
}

#[test]
fn absorbed_par_splits_sunlit_and_shaded() {
    let mut c = LightCoeffs {
        diffuse: 0.8,
        diffuse_absorbed: 0.7,
        ..LightCoeffs::default()
    };
    c.direct[12] = 0.6;
    let a = absorbed_par(&c, 12, WPerM2(500.0), WPerM2(100.0), 1.0, 0.5, 0.85);
    let k = PAR_FRACTION * UMOL_PER_J_PAR * 0.85;
    assert!((a.shaded.0 - 0.7 * 100.0 * k).abs() < 1e-9);
    assert!((a.sunlit.0 - (0.7 * 100.0 + 0.5 * 500.0) * k).abs() < 1e-9);
    assert!((a.sunlit_fraction - 0.6).abs() < 1e-15);
    let night = absorbed_par(&c, 0, WPerM2(0.0), WPerM2(0.0), -0.3, 0.5, 0.85);
    assert!(night.mean().0.abs() < 1e-15);
}

#[test]
fn bounded_traversal_is_bit_identical() {
    // A sparse cluster in one corner of a larger grid, so the bounds cut most rays short.
    let mut g = LightGrid::with_geometry([0.0, 0.0, 0.0], Meters(0.02), [12, 10, 8], 0.5);
    let leaves: Vec<LeafPoint> = (0..20usize)
        .map(|i| {
            let f = as_f64(i);
            LeafPoint {
                center: [0.03 + 0.004 * f, 0.05 + 0.003 * (f % 5.0), 0.04 + 0.002 * f],
                area: M2(4e-4),
                normal: [0.1, 0.0, 1.0],
            }
        })
        .collect();
    g.add_leaves(&leaves);
    let bounds = g.occupied_bounds();
    assert!(bounds.1[0] < 11 && bounds.1[2] < 7, "{bounds:?}");
    let mut dirs: Vec<[f64; 3]> = sky_sectors().iter().map(|s| s.direction).collect();
    dirs.extend([unit(15.0, 250.0), unit(80.0, 10.0), [0.0, 0.0, 1.0]]);
    for l in &leaves {
        let own = g.own_depth(&g.footprint(l));
        for p in sample_points(l) {
            for d in &dirs {
                let a = g.path_depth(p, *d, &own);
                let b = g.path_depth_bounded(p, *d, &own, &bounds);
                assert_eq!(a.to_bits(), b.to_bits(), "{p:?} {d:?}");
            }
        }
    }
    // An empty grid bounds to the whole grid.
    let e = LightGrid::with_geometry([0.0, 0.0, 0.0], Meters(0.02), [3, 4, 5], 0.5);
    assert_eq!(e.occupied_bounds(), ([0; 3], [2, 3, 4]));
}
