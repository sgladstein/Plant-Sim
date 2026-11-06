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

#[test]
fn individuals_are_keyed_by_plant() {
    use crate::Individual;
    use ps_units::{Hours, Meters};
    let p = params();
    let d = Meters(0.03);
    let a = Individual::draw(&p, 7, 3, Hours(0), d);
    assert_eq!(a, Individual::draw(&p, 7, 3, Hours(0), d));
    let others: Vec<Individual> = (0..20)
        .map(|e| Individual::draw(&p, 7, e, Hours(0), d))
        .collect();
    let seeds: Vec<f64> = others.iter().map(|i| i.seed_factor).collect();
    assert!(
        seeds.iter().any(|&f| f > 1.02) && seeds.iter().any(|&f| f < 0.98),
        "{seeds:?}"
    );
    for i in &others {
        assert!((0.25..=1.75).contains(&i.seed_factor));
        assert!((0.0..core::f64::consts::TAU).contains(&i.azimuth.0));
        assert!(i.emergence.0 >= 0.25 * p.thresholds.emergence.0);
    }
}

#[test]
fn deeper_seeds_emerge_later_and_no_variation_means_the_species_value() {
    use crate::{Individual, VariationParams};
    use ps_units::{Hours, Meters};
    let mut p = params();
    let shallow = Individual::draw(&p, 7, 1, Hours(0), Meters(0.02));
    let deep = Individual::draw(&p, 7, 1, Hours(0), Meters(0.05));
    let per_cm = p.variation.emergence_per_depth * 0.01;
    assert!(deep.emergence.0 > shallow.emergence.0 + 2.0 * per_cm);
    p.variation = VariationParams {
        reference_depth: Meters(0.03),
        ..VariationParams::default()
    };
    let i = Individual::draw(&p, 7, 1, Hours(0), Meters(0.03));
    assert!((i.seed_factor - 1.0).abs() < f64::EPSILON);
    assert!((i.emergence.0 - p.thresholds.emergence.0).abs() < 1e-12);
}

#[test]
fn scatter_is_bounded() {
    assert!((crate::scatter(0.0, 0.2) - 1.0).abs() < f64::EPSILON);
    assert!((crate::scatter(10.0, 0.2) - 1.5).abs() < 1e-12);
    assert!((crate::scatter(-10.0, 0.5) - 0.25).abs() < 1e-12);
}

/// The stomatal supply factor (R2a, decision 0037): with spare root capacity it opens
/// fully; short of water it closes to the share delivered; at night it resets.
#[test]
fn supply_factor_follows_what_the_roots_could_deliver() {
    use ps_soil::Uptake;
    use ps_units::KgWater;
    let u = |actual: f64, shortfall: f64, capacity: f64| Uptake {
        actual: KgWater(actual),
        shortfall: KgWater(shortfall),
        per_cell: Vec::new(),
        capacity: KgWater(capacity),
    };
    // Met with room to spare: fully open, never above 1.
    assert!((crate::supply_factor(0.5, &u(1.0, 0.0, 3.0)) - 1.0).abs() < 1e-15);
    // Met just: the factor stays where it was.
    assert!((crate::supply_factor(0.5, &u(1.0, 0.0, 1.0)) - 0.5).abs() < 1e-15);
    // Short: closes to the share delivered, from the factor in force.
    assert!((crate::supply_factor(0.8, &u(0.6, 0.4, 0.6)) - 0.48).abs() < 1e-15);
    // Short because neighbours shared the cells: reads what was delivered.
    assert!((crate::supply_factor(1.0, &u(0.5, 0.5, 2.0)) - 0.5).abs() < 1e-15);
    // No demand: resets.
    assert!((crate::supply_factor(0.1, &u(0.0, 0.0, 0.0)) - 1.0).abs() < 1e-15);
}

/// R1c (decision 0041): an apex lit from +x turns toward +x by the weight's share of
/// the angle, never more than the daily turn per day of light seen.
#[test]
fn an_apex_turns_toward_the_light_within_the_daily_bound() {
    use crate::shoot::turn_toward_light;
    let p = params();
    let (w, max) = (p.phototropism_weight, p.phototropism_max_per_day.0);
    assert!(w > 0.0 && max > 0.0);
    let up = [0.0, 0.0, 1.0];
    let angle = |a: [f64; 3], b: [f64; 3]| {
        libm::acos((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0))
    };
    // Light from the side: the bound holds, one day or two.
    for days in [1.0, 2.0] {
        let d = turn_toward_light(up, [5.0, 0.0, 0.0], w, max * days);
        assert!(d[0] > 0.0 && d[1].abs() < 1e-12, "{d:?}");
        let turned = angle(up, d);
        let want = (w * core::f64::consts::FRAC_PI_2).min(max * days);
        assert!((turned - want).abs() < 1e-9, "{turned} against {want}");
        assert!(turned <= max * days + 1e-12);
    }
    // Light 20° off the stem: the weight's share of it.
    let off = [libm::sin(0.35), 0.0, libm::cos(0.35)];
    let d = turn_toward_light(up, off, w, max);
    assert!((angle(up, d) - (w * 0.35).min(max)).abs() < 1e-9);
    // No light, light straight along the stem, or a zero turn: unchanged.
    let bits = |v: [f64; 3]| v.map(f64::to_bits);
    assert_eq!(bits(turn_toward_light(up, [0.0; 3], w, max)), bits(up));
    assert_eq!(
        bits(turn_toward_light(up, [0.0, 0.0, 3.0], w, max)),
        bits(up)
    );
    assert_eq!(
        bits(turn_toward_light(up, [1.0, 0.0, 0.0], w, 0.0)),
        bits(up)
    );
}
