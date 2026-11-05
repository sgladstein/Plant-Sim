//! WP22 done-when (design plan §7, decision 0021): a staked pole bean twines up its
//! support; without one it falls over and yields less.

use std::path::{Path, PathBuf};

use ps_arch::Climb;
use ps_validate::{At, replicate_mean};
use ps_world::{Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn run(name: &str, days: Option<u32>) -> World {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    if let Some(d) = days {
        s.duration_days = d;
    }
    let c = Catalog::load(&data(), &s).unwrap();
    let mut w = World::new(s, c).unwrap();
    w.run_to_end().unwrap();
    w
}

fn mean(w: &World, variable: &str, at: &At) -> f64 {
    replicate_mean(w.state(), variable, at).unwrap().value
}

/// In a row of 8, every vine on a string catches it, coils its main stem around it,
/// and climbs past 1.5 m; in the same row without strings they fall over and stay
/// under 30 cm, and yield much less. The -30 to -70% band of the field trials (IITA,
/// Reckling; decision 0021) is checked on the four-seed mean by the validation report
/// (`pole_unstaked_seed_yield`, decision 0026); one world seed spans about -50 to -80%,
/// so this single-seed test holds the direction with a wider band (decision 0029).
#[test]
fn staked_pole_beans_climb_and_unstaked_ones_lodge() {
    let staked = run("m2/pole_row_trellis.ron", None);
    let unstaked = run("m2/pole_row_no_trellis.ron", None);
    for p in &staked.state().plants {
        let plant = &p.plant;
        let main = &plant.axes[0];
        assert!(
            matches!(main.climb, Climb::Twining(_) | Climb::Topped),
            "plant {} main stem {:?}",
            p.id,
            main.climb
        );
        let coiled = plant.meta.iter().filter(|m| m.twine.is_some()).count();
        assert!(
            coiled >= 10,
            "plant {} has {coiled} coiled internodes",
            p.id
        );
    }
    for p in &unstaked.state().plants {
        assert_eq!(p.plant.axes[0].climb, Climb::Lodged, "plant {}", p.id);
        assert!(p.plant.meta.iter().all(|m| m.twine.is_none()));
    }
    let tall = mean(&staked, "height_m", &At::Final);
    let flat = mean(&unstaked, "height_m", &At::Final);
    assert!(tall > 1.5, "staked height {tall:.2} m");
    assert!(flat < 0.3, "unstaked height {flat:.2} m");
    let a = mean(&staked, "seed_mass_g", &At::Final);
    let b = mean(&unstaked, "seed_mass_g", &At::Final);
    let change = b / a - 1.0;
    assert!(
        (-0.9..=-0.2).contains(&change),
        "unstaked seed {b:.2} g vs staked {a:.2} g ({:+.0}%)",
        change * 100.0
    );
}

/// Supports change nothing for a determinate bush bean.
#[test]
fn bush_beans_ignore_supports() {
    let text = std::fs::read_to_string(data().join("scenarios/single_pot.ron")).unwrap();
    let plain = Scenario::from_ron(&text).unwrap();
    let staked_text = text.replace(
        "  duration_days: 90,",
        "  supports: [ (soil: 0, offset_cm: (4.0, 0.0), height_cm: 100.0, diameter_cm: 1.0) ],\n  duration_days: 90,",
    );
    let staked = Scenario::from_ron(&staked_text).unwrap();
    let go = |mut s: Scenario| {
        s.duration_days = 50;
        let c = Catalog::load(&data(), &s).unwrap();
        let mut w = World::new(s, c).unwrap();
        w.run_to_end().unwrap();
        w.state().plants[0].plant.report()
    };
    assert_eq!(go(plain), go(staked));
}
