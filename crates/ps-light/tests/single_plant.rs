//! WP6 done-when 2: a single plant against the Monte Carlo ray tracer fixture
//! (`validation/fixtures/light_raytrace.py`). Two plants: the synthetic WP6 shoot
//! (decision 0008) and a 60-day sim plant (decision 0018).

use std::fmt::Write as _;

use ps_light::{LeafPoint, rebuild_daily, sky_sectors};
use ps_units::M2;

const LEAVES: &str = include_str!("fixtures/plant_leaves.csv");
const TRACE: &str = include_str!("fixtures/plant_raytrace.csv");
const LEAVES_60: &str = include_str!("fixtures/plant60_leaves.csv");
const TRACE_60: &str = include_str!("fixtures/plant60_raytrace.csv");

fn rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| l.split(',').collect())
}

fn num(s: &str) -> f64 {
    s.parse().expect("fixture number")
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

struct Reference {
    leaves: Vec<LeafPoint>,
    /// `sun[d][leaf]`: ray-traced transmittance toward sun direction `d`.
    sun: Vec<Vec<f64>>,
    sun_dirs: Vec<[f64; 3]>,
    /// `sky[s][leaf]`, in ps-light sky-sector order.
    sky: Vec<Vec<f64>>,
    sky_dirs: Vec<[f64; 3]>,
}

fn reference(leaves_csv: &str, trace_csv: &str) -> Reference {
    let leaves: Vec<LeafPoint> = rows(leaves_csv)
        .map(|r| LeafPoint {
            center: [num(r[1]), num(r[2]), num(r[3])],
            area: M2(num(r[4])),
            normal: [num(r[5]), num(r[6]), num(r[7])],
        })
        .collect();
    let mut r = Reference {
        leaves,
        sun: Vec::new(),
        sun_dirs: Vec::new(),
        sky: Vec::new(),
        sky_dirs: Vec::new(),
    };
    // Rows are grouped by leaf, then by direction in a fixed order.
    let mut order: Vec<(String, f64, f64)> = Vec::new();
    for row in rows(trace_csv) {
        let key = (row[1].to_string(), num(row[2]), num(row[3]));
        let d = if let Some(d) = order.iter().position(|k| *k == key) {
            d
        } else {
            order.push(key);
            order.len() - 1
        };
        let (kind, elev, az) = &order[d];
        let (vals, dirs, idx) = if kind == "sun" {
            let i = order[..d].iter().filter(|k| k.0 == "sun").count();
            (&mut r.sun, &mut r.sun_dirs, i)
        } else {
            let i = order[..d].iter().filter(|k| k.0 == "sky").count();
            (&mut r.sky, &mut r.sky_dirs, i)
        };
        if vals.len() == idx {
            vals.push(Vec::new());
            dirs.push(unit(*elev, *az));
        }
        vals[idx].push(num(row[4]));
    }
    r
}

#[test]
fn fixture_sky_directions_match_sky_sectors() {
    let r = reference(LEAVES, TRACE);
    let sky = sky_sectors();
    assert_eq!(r.sky_dirs.len(), sky.len());
    for (a, b) in r.sky_dirs.iter().zip(&sky) {
        for k in 0..3 {
            assert!(
                (a[k] - b.direction[k]).abs() < 1e-5,
                "{a:?} vs {:?}",
                b.direction
            );
        }
    }
}

/// The comparison of one plant: area-weighted total, per-leaf errors and a report.
struct Comparison {
    total_rel: f64,
    within: usize,
    n: usize,
    worst: f64,
    report: String,
}

fn compare(r: &Reference, g: f64) -> Comparison {
    let n = r.leaves.len();
    let sky = sky_sectors();
    let mut sun_path = [None; 24];
    for (h, d) in r.sun_dirs.iter().enumerate() {
        sun_path[8 + h] = Some(*d);
    }
    let (_, c) = rebuild_daily(&r.leaves, &[[0.0, 0.0, 0.0]], &sun_path, &sky, g);
    assert_eq!(c.len(), n);

    // Per leaf: the daily light index, the mean of the direct coefficients over the
    // test sun positions and the diffuse coefficient, equally weighted.
    let mut errors = Vec::with_capacity(n);
    let (mut total_model, mut total_ref) = (0.0, 0.0);
    let mut report = String::new();
    let mut mixed = 0;
    for i in 0..n {
        let ns = f64::from(u32::try_from(r.sun.len()).unwrap());
        let ref_direct: f64 = r.sun.iter().map(|d| d[i]).sum::<f64>() / ns;
        let ref_diffuse: f64 = r.sky.iter().zip(&sky).map(|(d, s)| s.weight * d[i]).sum();
        let mod_direct: f64 = (0..r.sun.len()).map(|h| c[i].direct[8 + h]).sum::<f64>() / ns;
        let mod_diffuse = c[i].diffuse;
        let model = 0.5 * (mod_direct + mod_diffuse);
        let refv = 0.5 * (ref_direct + ref_diffuse);
        let a = r.leaves[i].area.0;
        total_model += a * model;
        total_ref += a * refv;
        let err = model - refv;
        errors.push(err.abs());
        if err.abs() <= 0.05_f64.max(0.10 * refv) {
            mixed += 1;
        }
        // Diagnostic: the worst single sun direction, so a change that fixes the mean
        // while breaking one direction shows up.
        let worst_dir = (0..r.sun.len())
            .map(|h| c[i].direct[8 + h] - r.sun[h][i])
            .fold(0.0_f64, |a, b| if b.abs() > a.abs() { b } else { a });
        writeln!(
            report,
            "leaf {i:2} z {:.3}: direct {mod_direct:.3}/{ref_direct:.3} \
             diffuse {mod_diffuse:.3}/{ref_diffuse:.3} err {err:+.3} worst sun {worst_dir:+.3}",
            r.leaves[i].center[2]
        )
        .unwrap();
    }
    let total_rel = total_model / total_ref - 1.0;
    let worst = errors.iter().fold(0.0_f64, |a, &b| a.max(b));
    let within = errors.iter().filter(|&&e| e <= 0.10).count();
    writeln!(
        report,
        "total {total_model:.5} vs {total_ref:.5} ({total_rel:+.4}); \
         {within}/{n} leaves within 0.10, worst {worst:.3}; \
         {mixed}/{n} within max(0.05, 10%) (reported, not gated)"
    )
    .unwrap();
    Comparison {
        total_rel,
        within,
        n,
        worst,
        report,
    }
}

#[test]
fn single_plant_matches_ray_tracer() {
    let c = compare(&reference(LEAVES, TRACE), 0.5);
    let Comparison {
        total_rel,
        within,
        n,
        worst,
        ref report,
    } = c;
    println!("{report}");
    // Design target: plant total within 3%.
    assert!(total_rel.abs() < 0.03, "plant total {total_rel:+.4}");
    // Design target: every leaf within 0.10. Measured: 56 of 57, worst 0.122 (55 and
    // 0.142 at WP6, before the diffuse fix in the decision 0008 addendum); the misses
    // are leaflets that overlap within a grid cell. The owner accepted 95% within 0.10
    // for M0 (decision 0008).
    assert!(
        within * 100 >= 95 * n,
        "{within}/{n} leaves within 0.10\n{report}"
    );
    assert!(worst < 0.15, "worst per-leaf error {worst:.3}\n{report}");
}

/// The WP12 re-check (decisions 0008 and 0018): the same comparison on a 60-day sim
/// plant (74 leaflets), exported by the ps-arch test `export_leaves_for_ray_tracer`.
/// The sim's leaflets all tilt about 19° from horizontal, while the species file's
/// G = 0.5 assumes spherical leaf angles, so with G = 0.5 the model is about 10% too
/// bright on the plant total. G = 0.65, about this canopy's mean over the test
/// directions, brings the total within 2%, but per-leaf errors stay above the target
/// (about 80% of leaflets within 0.10). This test pins both results; it does not claim
/// the target is met. The fix is the owner's call (decision 0018).
#[test]
fn sim_plant_against_ray_tracer() {
    let r = reference(LEAVES_60, TRACE_60);
    let species_g = compare(&r, 0.5);
    let leaf_angle_g = compare(&r, 0.65);
    println!(
        "G = 0.5\n{}G = 0.65\n{}",
        species_g.report, leaf_angle_g.report
    );
    assert!(
        (0.05..0.15).contains(&species_g.total_rel),
        "G = 0.5 total {:+.4}",
        species_g.total_rel
    );
    assert!(
        leaf_angle_g.total_rel.abs() < 0.03,
        "G = 0.65 total {:+.4}",
        leaf_angle_g.total_rel
    );
    assert!(leaf_angle_g.within * 100 >= 75 * leaf_angle_g.n);
    assert!(leaf_angle_g.worst < 0.25);
}
