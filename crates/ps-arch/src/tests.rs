//! Unit tests of the herb parameter profiles and geometry helpers. The 90-day run and
//! the whole-plant tests are in `tests/ninety_days.rs`.

use ps_species::Species;

use crate::geometry::{deflect, from_azimuth, norm};
use crate::{HerbParams, ShadeSignal};
use ps_units::MolPerM2Day;

fn params() -> HerbParams {
    let s = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    let t = s.phenotype(&s.default_genotype()).traits().unwrap();
    HerbParams::new(&s, &t)
}

#[test]
fn leaf_profile_rises_then_falls() {
    let p = params();
    let n = 6;
    let f: Vec<f64> = (1..=n).map(|r| p.leaf_factor(r, n)).collect();
    assert!((f[0] - p.leaf_profile[0]).abs() < 1e-12, "{f:?}");
    assert!((f[2] - 1.0).abs() < 1e-12, "{f:?}");
    assert!((f[5] - p.leaf_profile[1]).abs() < 1e-12, "{f:?}");
    assert!(f.iter().all(|x| *x > 0.0 && *x <= 1.0));
}

#[test]
fn internode_profile_is_linear_after_the_epicotyl() {
    let p = params();
    let n = 7;
    assert!((p.internode_factor(1, n) - 1.0).abs() < 1e-12);
    assert!((p.internode_factor(2, n) - p.internode_profile[0]).abs() < 1e-12);
    assert!((p.internode_factor(n, n) - p.internode_profile[1]).abs() < 1e-12);
}

#[test]
fn pod_set_temperature_is_a_plateau_with_quadratic_ramps() {
    let p = params();
    let [lo, o1, o2, hi] = p.pod_set_t;
    assert!(p.pod_set_temperature(lo).abs() < 1e-12);
    assert!(p.pod_set_temperature(hi).abs() < 1e-12);
    assert!((p.pod_set_temperature(0.5 * (o1 + o2)) - 1.0).abs() < 1e-12);
    let mid = 0.5 * (o2 + hi);
    assert!((p.pod_set_temperature(mid) - 0.75).abs() < 1e-12);
}

#[test]
fn sink_feedback_falls_from_onset_to_full() {
    let p = params();
    assert!((p.sink_feedback(0.0) - 1.0).abs() < 1e-12);
    assert!((p.sink_feedback(p.feedback_onset) - 1.0).abs() < 1e-12);
    assert!(p.sink_feedback(p.feedback_full).abs() < 1e-12);
    let mid = 0.5 * (p.feedback_onset + p.feedback_full);
    assert!((p.sink_feedback(mid) - 0.5).abs() < 1e-12);
}

#[test]
fn shade_factor_is_one_in_the_open() {
    let p = params();
    assert!((p.shade_factor(ShadeSignal::OPEN, 1.5) - 1.0).abs() < 1e-12);
    let dark_rfr = ShadeSignal {
        rfr: 0.2,
        dli: None,
    };
    assert!((p.shade_factor(dark_rfr, 1.5) - 1.5).abs() < 1e-12);
}

#[test]
fn low_light_alone_gives_the_shade_response() {
    let p = params();
    let at = |d: f64| {
        p.shade_factor(
            ShadeSignal {
                rfr: 1.2,
                dli: Some(MolPerM2Day(d)),
            },
            1.5,
        )
    };
    assert!((at(p.shade_dli_onset.0 + 1.0) - 1.0).abs() < 1e-12);
    assert!((at(p.shade_dli_full.0) - 1.5).abs() < 1e-12);
    let mid = 0.5 * (p.shade_dli_onset.0 + p.shade_dli_full.0);
    assert!((at(mid) - 1.25).abs() < 1e-12);
}

#[test]
fn deflection_keeps_the_angle() {
    for axis in [[0.0, 0.0, -1.0], [1.0, 0.0, 0.0], from_azimuth(1.0, -0.3)] {
        for az in [0.0, 1.0, 4.0] {
            let d = deflect(axis, 0.6, az);
            assert!((norm(d) - 1.0).abs() < 1e-12);
            let cos = d[0] * axis[0] + d[1] * axis[1] + d[2] * axis[2];
            assert!(
                (cos / norm(axis) - libm::cos(0.6)).abs() < 1e-9,
                "{axis:?} {az}"
            );
        }
    }
}

#[test]
fn root_types_convert_to_model_units() {
    let p = params();
    let tap = &p.roots[0];
    // 0.9 mm at 94,000 g/m³: about 17 m/g.
    assert!(
        (tap.specific_length.0 - 16.7).abs() < 0.1,
        "{}",
        tap.specific_length.0
    );
    assert!(tap.elongation_per_cd > 0.0);
    assert_eq!(
        p.roots
            .iter()
            .filter(|r| r.order == 0)
            .map(|r| r.count)
            .sum::<u32>(),
        19
    );
}
