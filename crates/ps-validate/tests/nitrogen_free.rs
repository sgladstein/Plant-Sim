//! WP24 done-when (design plan §7, decision 0020): nodulated beans grow in
//! nitrogen-free soil and unnodulated ones do not.

use std::path::{Path, PathBuf};

use ps_validate::{At, replicate_mean};
use ps_world::{Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn run(name: &str, days: u32) -> World {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.duration_days = days;
    let c = Catalog::load(&data(), &s).unwrap();
    let mut w = World::new(s, c).unwrap();
    w.run_to_end().unwrap();
    w
}

/// In nitrogen-free sand the nodulated shoot is more than three times the
/// unnodulated one 45 days after emergence (day 12, so sim day 57). The
/// unnodulated plant takes up and fixes nothing, so it holds no more than its seed
/// nitrogen; the nodulated one fixes what it grows on.
#[test]
fn nitrogen_free_sand_needs_nodules() {
    let days = 58;
    let with = run("m2/nfree_nodulated.ron", days);
    let without = run("m2/nfree_unnodulated.ron", days);
    let at = At::Day(57);
    let shoot = |w: &World| {
        replicate_mean(w.state(), "shoot_mass_g", &at)
            .unwrap()
            .value
    };
    let (a, b) = (shoot(&with), shoot(&without));
    assert!(
        a > 3.0 * b,
        "shoot {a:.3} g nodulated vs {b:.3} g unnodulated"
    );
    for p in &without.state().plants {
        let plant = &p.plant;
        let ledger = &plant.nitrogen.ledger;
        assert!(
            ledger.uptake.0.abs() < f64::MIN_POSITIVE && ledger.fixed.0.abs() < f64::MIN_POSITIVE
        );
        let n = plant.nitrogen_stock().0;
        assert!(
            n <= ledger.seed.0 * 1.01,
            "plant N {n} vs seed N {}",
            ledger.seed.0
        );
        assert!(
            plant
                .arena
                .alive_of(ps_graph::OrganKind::Nodule)
                .next()
                .is_none()
        );
    }
    for p in &with.state().plants {
        let ledger = &p.plant.nitrogen.ledger;
        assert!(
            ledger.uptake.0.abs() < f64::MIN_POSITIVE,
            "no soil N to take up"
        );
        assert!(
            ledger.fixed.0 > 2.0 * ledger.seed.0,
            "fixed {}",
            ledger.fixed.0
        );
    }
}
