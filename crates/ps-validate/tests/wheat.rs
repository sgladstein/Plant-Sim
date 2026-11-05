//! WP23 done-when (design plan §7, decision 0022): spring wheat tillers, heads and
//! fills grain; a dense stand tillers less than a spaced plant.

use std::path::{Path, PathBuf};

use ps_arch::Role;
use ps_graph::OrganKind;
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

/// Spaced plants tiller (3 to 10 living tillers), every surviving shoot ends in an
/// ear on a peduncle, the plant reaches 0.7 to 1.0 m and fills grain; the leaves are
/// sheaths and blades, and no nodules form.
#[test]
fn spaced_wheat_tillers_heads_and_fills_grain() {
    let w = run("m2/wheat_spaced.ron", None);
    let tillers = mean(&w, "branches", &At::Final);
    assert!((3.0..=10.0).contains(&tillers), "tillers {tillers:.2}");
    let height = mean(&w, "height_m", &At::Final);
    assert!((0.7..=1.0).contains(&height), "height {height:.2} m");
    assert!(mean(&w, "seed_mass_g", &At::Final) > 1.0);
    for p in &w.state().plants {
        let plant = &p.plant;
        let a = &plant.arena;
        let ears = a
            .alive_of(OrganKind::Pod)
            .filter(|id| plant.meta[id.index()].role == Role::Ear)
            .count();
        assert_eq!(ears, 1 + plant.living_tillers() as usize, "plant {}", p.id);
        let blades = plant.meta.iter().filter(|m| m.role == Role::Blade).count();
        let sheaths = plant.meta.iter().filter(|m| m.role == Role::Sheath).count();
        assert_eq!(blades, sheaths);
        assert!(plant.meta.iter().all(|m| m.role != Role::Trifoliate));
        assert_eq!(a.alive_of(OrganKind::Nodule).count(), 0);
    }
}

/// At 200 plants per m², the stand's shade stops tillering early: by the end of
/// tillering each plant has fewer tillers than a spaced plant.
#[test]
fn a_dense_stand_tillers_less() {
    let spaced = run("m2/wheat_spaced.ron", Some(50));
    let stand = run("m2/wheat_stand.ron", Some(50));
    let a = mean(&spaced, "branches", &At::Final);
    let b = mean(&stand, "branches", &At::Final);
    assert!(b < 0.7 * a, "stand {b:.2} vs spaced {a:.2} tillers");
    assert!(b >= 1.0, "stand {b:.2} tillers");
}
