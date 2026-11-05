use super::*;

/// A horizontally uniform canopy on an explicit grid: `nz` layers of `n × n` cells,
/// 8 leaves per cell on a 2 × 2 × 2 sub-lattice, total leaf area index `lai`.
fn uniform_canopy(
    n: usize,
    nz: usize,
    cell: f64,
    lai: f64,
    normal: impl Fn(usize) -> [f64; 3],
    dirs: &[[f64; 3]],
) -> (LightGrid, Vec<LeafPoint>) {
    let mut g = LightGrid::with_geometry([0.0, 0.0, 0.0], Meters(cell), [n, n, nz], dirs);
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
                        normal: normal(leaves.len()),
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

/// A pseudo-random unit normal on the upper hemisphere with uniform density over the
/// sphere (a spherical leaf angle distribution), from a fixed sequence.
fn spherical_normal(i: usize) -> [f64; 3] {
    // Golden-ratio sequences in cos θ and azimuth.
    let f = as_f64(i);
    let u = (f * 0.618_033_988_749_895 + 0.5) % 1.0;
    let v = (f * 0.754_877_666_246_693 + 0.25) % 1.0;
    let ct = u.max(1e-6);
    let st = libm::sqrt(1.0 - ct * ct);
    let a = 2.0 * PI * v;
    [st * libm::cos(a), st * libm::sin(a), ct]
}

/// Absorbed fraction of a beam at elevation `elev` by the central part of the canopy.
fn absorbed_fraction(n: usize, nz: usize, cell: f64, lai: f64, elev: f64, sphere: bool) -> f64 {
    let d = unit(elev, 30.0);
    let normal = |i| {
        if sphere {
            spherical_normal(i)
        } else {
            [0.0, 0.0, 1.0]
        }
    };
    let (g, leaves) = uniform_canopy(n, nz, cell, lai, normal, &[d]);
    let half = 0.5 * as_f64(n) * cell;
    let r = 0.2 * as_f64(n) * cell;
    let mut absorbed = 0.0;
    for l in &leaves {
        if (l.center[0] - half).abs() < r && (l.center[1] - half).abs() < r {
            let own = g.footprint_of(l);
            let t = libm::exp(-g.path_depth(l.center, 0, &own));
            let p = projection(l.normal, d);
            absorbed += l.area.0 * p / libm::sin(elev.to_radians()) * t;
        }
    }
    absorbed / (2.0 * r * 2.0 * r)
}

/// WP6 done-when 1: a uniform canopy absorbs within 2% of Beer's law, LAI 0.5 to 6:
/// with spherical leaf angles at `G = 0.5`, and with horizontal leaves at `G = sin β`
/// (decision 0025).
#[test]
fn uniform_canopy_matches_beers_law() {
    for &lai in &[0.5, 1.0, 2.0, 4.0, 6.0] {
        for &elev in &[90.0, 60.0, 45.0] {
            let sin_b = libm::sin(f64::to_radians(elev));
            for (sphere, g) in [(true, 0.5), (false, sin_b)] {
                let got = absorbed_fraction(40, 12, 0.04, lai, elev, sphere);
                let expect = 1.0 - libm::exp(-g * lai / sin_b);
                let rel = got / expect - 1.0;
                assert!(
                    rel.abs() < 0.02,
                    "LAI {lai}, elevation {elev}, spherical {sphere}: {got:.4} vs \
                     {expect:.4} ({rel:+.4})"
                );
            }
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
    let (grid, c) = rebuild_daily(&leaves, &[[0.0, 0.0, 0.0]], &sun, &sky_sectors());
    assert!(grid.is_some());
    assert!((c[0].direct[12] - 1.0).abs() < 1e-12);
    assert!((c[0].direct[8] - 1.0).abs() < 1e-12);
    assert!(c[0].direct[0].abs() < 1e-15, "night");
    assert!((c[0].diffuse - 1.0).abs() < 1e-12);
    // A horizontal leaf faces the noon sun and sees a 20° sun at sin 20°.
    assert!((c[0].beam[12] - 1.0).abs() < 1e-12);
    assert!((c[0].beam[8] - libm::sin(20f64.to_radians())).abs() < 1e-12);
    // A horizontal leaf in the open under an overcast sky absorbs the horizontal
    // diffuse irradiance.
    assert!(
        (c[0].diffuse_absorbed - 1.0).abs() < 1e-12,
        "{}",
        c[0].diffuse_absorbed
    );
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
    let (_, c) = rebuild_daily(&leaves, &[], &sun, &sky_sectors());
    assert!((c[0].direct[12] - 1.0).abs() < 1e-12);
    // One opaque horizontal leaf right above casts a full shadow on the noon sun (but
    // for the partly covered cells at its rim).
    assert!(c[1].direct[12] < 0.08, "{}", c[1].direct[12]);
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
    let g = LightGrid::fit(&tiny, &[], &[]).unwrap();
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
    let g = LightGrid::fit(&tall, &[], &[]).unwrap();
    assert!(g.n.iter().all(|&n| n <= MAX_CELLS_PER_AXIS));
    assert!(g.locate(tall[0].center).is_some() && g.locate(tall[1].center).is_some());
    let total: f64 = g.lad.iter().sum::<f64>() * g.cell_volume();
    assert!((total - 0.02).abs() < 1e-15);
    assert!(LightGrid::fit(&[], &[], &[]).is_none());
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
    let (g, _) = uniform_canopy(4, 2, 0.04, 2.0, |_| [0.0, 0.0, 1.0], &[]);
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
    c.beam[12] = 0.5;
    let a = absorbed_par(&c, 12, WPerM2(500.0), WPerM2(100.0), 1.0, 0.85);
    let k = PAR_FRACTION * UMOL_PER_J_PAR * 0.85;
    assert!((a.shaded.0 - 0.7 * 100.0 * k).abs() < 1e-9);
    assert!((a.sunlit.0 - (0.7 * 100.0 + 0.5 * 500.0) * k).abs() < 1e-9);
    assert!((a.sunlit_fraction - 0.6).abs() < 1e-15);
    let night = absorbed_par(&c, 0, WPerM2(0.0), WPerM2(0.0), -0.3, 0.85);
    assert!(night.mean().0.abs() < 1e-15);
}

#[test]
fn bounded_traversal_is_bit_identical() {
    // A sparse cluster in one corner of a larger grid, so the bounds cut most rays short.
    let mut dirs: Vec<[f64; 3]> = sky_sectors().iter().map(|s| s.direction).collect();
    dirs.extend([unit(15.0, 250.0), unit(80.0, 10.0), [0.0, 0.0, 1.0]]);
    let mut g = LightGrid::with_geometry([0.0, 0.0, 0.0], Meters(0.02), [12, 10, 8], &dirs);
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
    let bounds = g.ray_bounds();
    assert!(
        bounds.cells.1[0] < 11 && bounds.cells.1[2] < 7,
        "{:?}",
        bounds.cells
    );
    for l in &leaves {
        let own = g.footprint_of(l);
        for p in sample_points(l) {
            for j in 0..dirs.len() {
                let a = g.path_depth(p, j, &own);
                let b = g.path_depth_bounded(p, j, &own, &bounds);
                assert_eq!(a.to_bits(), b.to_bits(), "{p:?} {j}");
            }
        }
    }
    // Two clumps of different heights, so a ray's reach depends on where it starts
    // (the shadows), with rays from every cell center too.
    let mut g = LightGrid::with_geometry([0.0, 0.0, 0.0], Meters(0.02), [16, 14, 12], &dirs);
    let clumps: Vec<LeafPoint> = (0..60usize)
        .map(|i| {
            let f = as_f64(i % 30);
            let (x, y, z) = if i < 30 {
                (0.04, 0.05, 0.03)
            } else {
                (0.2, 0.17, 0.12)
            };
            LeafPoint {
                center: [x + 0.003 * f, y + 0.002 * (f % 7.0), z + 0.0025 * f],
                area: M2(3e-4),
                normal: [0.2, -0.1, 1.0],
            }
        })
        .collect();
    g.add_leaves(&clumps);
    let bounds = g.ray_bounds();
    for l in &clumps {
        let own = g.footprint_of(l);
        for p in sample_points(l) {
            for j in 0..dirs.len() {
                let a = g.path_depth(p, j, &own);
                let b = g.path_depth_bounded(p, j, &own, &bounds);
                assert_eq!(a.to_bits(), b.to_bits(), "{p:?} {j}");
            }
        }
    }
    for i in 0..g.slot.len() {
        let c = [i % 16, (i / 16) % 14, i / (16 * 14)];
        let p = [
            (as_f64(c[0]) + 0.5) * 0.02,
            (as_f64(c[1]) + 0.5) * 0.02,
            (as_f64(c[2]) + 0.5) * 0.02,
        ];
        for j in 0..dirs.len() {
            let a = g.path_depth(p, j, &Footprint::none());
            let b = g.path_depth_bounded(p, j, &Footprint::none(), &bounds);
            assert_eq!(a.to_bits(), b.to_bits(), "{p:?} {j}");
        }
    }
    // An empty grid bounds to the whole grid.
    let e = LightGrid::with_geometry([0.0, 0.0, 0.0], Meters(0.02), [3, 4, 5], &dirs);
    assert_eq!(e.occupied_bounds(), ([0; 3], [2, 3, 4]));
}

/// Decision 0025: a leaf's interception follows its own angle to each ray. A blade
/// tilted 60° shades a noon sun half as much as a flat one, and an edge-on blade not
/// at all.
#[test]
fn a_tilted_leaf_shades_by_its_projection() {
    let mut sun = [None; 24];
    sun[12] = Some([0.0, 0.0, 1.0]);
    let below = |normal: [f64; 3]| {
        let leaves = [
            LeafPoint {
                center: [0.0, 0.0, 0.15],
                area: M2(0.0075),
                normal,
            },
            LeafPoint {
                center: [0.0, 0.0, 0.10],
                area: M2(0.0075),
                normal: [0.0, 0.0, 1.0],
            },
        ];
        let (_, c) = rebuild_daily(&leaves, &[], &sun, &sky_sectors());
        (c[0].beam[12], c[1].direct[12])
    };
    let s60 = libm::sin(60f64.to_radians());
    let (beam, t) = below([s60, 0.0, 0.5]);
    assert!((beam - 0.5).abs() < 1e-12);
    assert!((t - 0.5).abs() < 0.03, "{t}");
    let (beam, t) = below([1.0, 0.0, 0.0]);
    assert!(beam.abs() < 1e-12 && (t - 1.0).abs() < 1e-12, "{beam} {t}");
}

#[test]
fn projection_is_the_cosine_to_the_ray() {
    assert!((projection([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]) - 1.0).abs() < 1e-15);
    assert!((projection([0.0, 0.0, 2.0], unit(30.0, 70.0)) - 0.5).abs() < 1e-12);
    // Both faces count; a degenerate normal is horizontal.
    assert!((projection([0.0, 0.0, -1.0], unit(30.0, 0.0)) - 0.5).abs() < 1e-12);
    assert!((projection([0.0; 3], unit(30.0, 0.0)) - 0.5).abs() < 1e-12);
}

/// The original form of [`LightGrid::footprint`]: every lattice sample located on its
/// own, then sorted and merged. The per-axis form must give the same bits.
fn footprint_reference(g: &LightGrid, l: &LeafPoint) -> Vec<(usize, f64)> {
    let r = leaf_radius(l.area);
    let h = 0.5 * g.cell.0;
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
    let share = l.area.0 / as_f64(pts.len()) / g.cell_volume();
    let mut cells: Vec<(usize, f64)> = pts
        .into_iter()
        .filter_map(|p| g.locate(p).map(|c| (g.flat(c), share)))
        .collect();
    cells.sort_by_key(|c| c.0);
    let mut merged: Vec<(usize, f64)> = Vec::new();
    for (i, v) in cells {
        match merged.last_mut() {
            Some(last) if last.0 == i => last.1 += v,
            _ => merged.push((i, v)),
        }
    }
    merged
}

/// Footprints of leaves of many sizes, some hanging over the grid's sides or outside
/// it, match the original sample-by-sample form bit for bit.
#[test]
fn footprint_matches_the_reference_bit_for_bit() {
    let g = LightGrid::with_geometry(
        [0.0, 0.0, 0.0],
        Meters(0.013),
        [11, 9, 6],
        &[[0.0, 0.0, 1.0]],
    );
    for i in 0..300usize {
        let f = as_f64(i);
        let l = LeafPoint {
            center: [-0.02 + 0.0007 * f, 0.13 - 0.00041 * f, -0.005 + 0.00029 * f],
            area: M2(1e-6 + 2.3e-5 * as_f64(i % 37)),
            normal: [0.0, 0.0, 1.0],
        };
        let a = g.footprint(&l);
        let b = footprint_reference(&g, &l);
        assert_eq!(a.len(), b.len(), "leaf {i}");
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x.0, y.0, "leaf {i}");
            assert_eq!(x.1.to_bits(), y.1.to_bits(), "leaf {i}");
        }
    }
}
