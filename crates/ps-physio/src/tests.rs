use super::*;
use ps_units::{Celsius, KPa, Ppm};

pub(crate) fn bean() -> (Species, Traits) {
    let s = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    let t = s.phenotype(&s.default_genotype()).traits().unwrap();
    (s, t)
}

fn env(par: f64, t: f64, vpd: f64, ca: f64) -> LeafEnv {
    LeafEnv {
        par: UmolPerM2S(par),
        t_leaf: Celsius(t),
        vpd: KPa(vpd),
        ca: Ppm(ca),
        patm: KPa(100.0),
    }
}

#[test]
fn bean_at_reference_conditions() {
    let (s, t) = bean();
    let p = LeafParams::from_species(&s, &t);
    let g = gas::solve(&p, &env(1500.0, 25.0, 1.5, 420.0), 1.0, 1.0);
    assert!(g.a_net.0 > 15.0 && g.a_net.0 < 30.0, "{g:?}");
    assert!(g.ci.0 > 250.0 && g.ci.0 < 380.0, "{g:?}");
    // Supply equals demand: A = gs/1.57 (Ca − Ci).
    let supply = g.gs.0 / gas::H2O_PER_CO2 * (420.0 - g.ci.0);
    assert!(
        (supply - g.a_net.0).abs() < 1e-6,
        "{supply} vs {}",
        g.a_net.0
    );
    assert!(g.iterations <= 2 * gas::MAX_ITERATIONS);
}

#[test]
fn darkness_gives_respiration_and_g0() {
    let (s, t) = bean();
    let p = LeafParams::from_species(&s, &t);
    let g = gas::solve(&p, &env(0.0, 25.0, 1.5, 420.0), 1.0, 1.0);
    assert_eq!(g.limitation, Limitation::Dark);
    // Ci = Ca below light compensation; with J = 0, A = −Rd exactly.
    assert!((g.a_net.0 + g.rd.0).abs() < 1e-12, "{g:?}");
    assert!((g.gs.0 - p.g0.0).abs() < 1e-12);
    assert!((g.ci.0 - 420.0).abs() < 1e-12);
}

#[test]
fn full_stress_closes_stomata_to_g0() {
    let (s, t) = bean();
    let p = LeafParams::from_species(&s, &t);
    let open = gas::solve(&p, &env(1500.0, 25.0, 1.5, 420.0), 1.0, 1.0);
    let half = gas::solve(&p, &env(1500.0, 25.0, 1.5, 420.0), 1.0, 0.5);
    let shut = gas::solve(&p, &env(1500.0, 25.0, 1.5, 420.0), 1.0, 0.0);
    assert!(open.gs.0 > half.gs.0 && half.gs.0 > shut.gs.0);
    assert!((shut.gs.0 - p.g0.0).abs() < 1e-12);
    assert!(open.a_net.0 > half.a_net.0 && half.a_net.0 > shut.a_net.0);
}

#[test]
fn aging_scales_capacity() {
    let (s, t) = bean();
    let p = LeafParams::from_species(&s, &t);
    let young = gas::solve(&p, &env(1500.0, 25.0, 1.5, 420.0), 1.0, 1.0);
    let old = gas::solve(&p, &env(1500.0, 25.0, 1.5, 420.0), 0.4, 1.0);
    assert!(old.a_net.0 < 0.6 * young.a_net.0 && old.a_net.0 > 0.2 * young.a_net.0);
    assert!((old.rd.0 - 0.4 * young.rd.0).abs() < 1e-12);
}

// ---------------------------------------------------------------- light classes

#[test]
fn class_table_matches_direct_solves() {
    let (s, t) = bean();
    let p = LeafParams::from_species(&s, &t);
    let e = env(0.0, 22.0, 1.2, 420.0);
    let table = ClassTable::build(&p, &e, 1.0);
    // At nodes, exact.
    for k in 0..=classes::PAR_CLASSES {
        let par = classes::par_node(k);
        let r = table.lookup(UmolPerM2S(par), 1.0);
        let g = gas::solve(
            &p,
            &LeafEnv {
                par: UmolPerM2S(par),
                ..e
            },
            1.0,
            1.0,
        );
        assert!((r.a_net.0 - g.a_net.0).abs() < 1e-12, "node {k}");
    }
    assert!((classes::par_node(16) - 2000.0).abs() < 1e-9);
    // Between nodes and capacities: interpolation error on A for PAR 50 to 2000 and
    // capacity 0.45 to 1. Measured 4.6% (at 333 µmol m⁻² s⁻¹ and capacity 0.55, where
    // the light response bends most; decisions 0009, 0020). Below capacity 0.4, a leaf
    // mined of its nitrogen, the table underestimates A by up to 15% at low light.
    let mut worst = 0.0_f64;
    for &par in &[50.0, 77.0, 130.0, 333.0, 640.0, 1111.0, 1750.0] {
        for &cap in &[1.0, 0.9, 0.85, 0.7, 0.55, 0.45] {
            let r = table.lookup(UmolPerM2S(par), cap);
            let g = gas::solve(
                &p,
                &LeafEnv {
                    par: UmolPerM2S(par),
                    ..e
                },
                cap,
                1.0,
            );
            worst = worst.max((r.a_net.0 / g.a_net.0 - 1.0).abs());
            assert!((r.e.0 / g.e.0 - 1.0).abs() < 0.045);
        }
    }
    assert!(worst < 0.05, "worst interpolation error {worst}");
}

// ---------------------------------------------------------------- thermal time

#[test]
fn three_point_thermal_time() {
    let c = Cardinal {
        base: Celsius(8.0),
        opt: Celsius(28.0),
        max: Celsius(38.0),
    };
    assert!(c.rate(Celsius(5.0)).abs() < 1e-15);
    assert!((c.rate(Celsius(18.0)) - 10.0).abs() < 1e-12);
    assert!((c.rate(Celsius(28.0)) - 20.0).abs() < 1e-12);
    assert!((c.rate(Celsius(33.0)) - 10.0).abs() < 1e-12);
    assert!(c.rate(Celsius(40.0)).abs() < 1e-15);
    // 24 hours at 18 °C is 10 °C·day.
    let day: f64 = (0..24).map(|_| c.hourly(Celsius(18.0)).0).sum();
    assert!((day - 10.0).abs() < 1e-12);
}

#[test]
fn growth_temperature_trapezoid() {
    let (s, _) = bean();
    let g = GrowthTemperature::from_species(&s);
    assert!(g.factor(Celsius(5.0)).abs() < 1e-15);
    assert!((g.factor(Celsius(15.0)) - 0.5).abs() < 1e-12);
    assert!((g.factor(Celsius(27.0)) - 1.0).abs() < 1e-12);
    assert!((g.factor(Celsius(35.0)) - 0.5).abs() < 1e-12);
    assert!(g.factor(Celsius(40.0)).abs() < 1e-15);
}

// ---------------------------------------------------------------- respiration

#[test]
fn maintenance_and_growth_respiration() {
    use ps_units::{GramsC, GramsN};
    let m = respiration::maintenance_hourly(0.015, 2.0, GramsN(0.1), Celsius(20.0));
    assert!((m.0 - 0.015 * 0.1 / 24.0).abs() < 1e-15);
    let warm = respiration::maintenance_hourly(0.015, 2.0, GramsN(0.1), Celsius(30.0));
    assert!((warm.0 / m.0 - 2.0).abs() < 1e-12);
    let (built, resp) = respiration::growth_split(GramsC(1.25), 1.25);
    assert!((built.0 - 1.0).abs() < 1e-15 && (resp.0 - 0.25).abs() < 1e-15);
}

#[test]
fn leaf_capacity_declines_after_expansion() {
    use ps_units::DegreeDays as D;
    assert!((leaf_capacity(D(50.0), D(100.0), D(600.0), 0.4) - 1.0).abs() < 1e-15);
    assert!((leaf_capacity(D(350.0), D(100.0), D(600.0), 0.4) - 0.7).abs() < 1e-12);
    assert!((leaf_capacity(D(900.0), D(100.0), D(600.0), 0.4) - 0.4).abs() < 1e-15);
}

// ---------------------------------------------------------------- phenology

#[test]
fn stages_advance_in_order() {
    use ps_units::DegreeDays as D;
    let (s, _) = bean();
    let th = Thresholds::from_species(&s);
    let mut ph = Phenology::default();
    let mut ev = Events::default();
    ph.accumulate(D(30.0));
    assert_eq!(ph.advance(&th, ev), None, "not germinated");
    ev.germinated = true;
    assert_eq!(ph.advance(&th, ev), Some(Stage::Germinated));
    ph.accumulate(D(79.0));
    assert_eq!(ph.advance(&th, ev), None, "79 of 80 Cd since germination");
    ph.accumulate(D(1.0));
    assert_eq!(ph.advance(&th, ev), Some(Stage::Emerged));
    // Flowering needs both the trifoliate and the thermal time.
    ph.accumulate(D(400.0));
    assert_eq!(ph.advance(&th, ev), None);
    ev.trifoliate = true;
    assert_eq!(ph.advance(&th, ev), Some(Stage::FirstTrifoliate));
    assert_eq!(ph.advance(&th, ev), Some(Stage::Flowering));
    ev.pod = true;
    ev.seed_filling = true;
    assert_eq!(ph.advance(&th, ev), Some(Stage::PodSet));
    assert_eq!(ph.advance(&th, ev), Some(Stage::SeedFill));
    assert_eq!(
        ph.advance(&th, ev),
        None,
        "maturity at 900 Cd since germination"
    );
    // The 30 Cd the seed waited before germinating do not count (decision 0011).
    assert!((ph.since_sowing.0 - ph.since_germination.0 - 30.0).abs() < 1e-12);
    ph.accumulate(D(420.0));
    assert_eq!(ph.advance(&th, ev), Some(Stage::Mature));
    assert_eq!(ph.advance(&th, ev), None);
}
