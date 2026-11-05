//! WP25 (design plan §7, decision 0024): prune, harvest, obstacles, supports added
//! mid-run, and a weather file driving a run.

use std::path::{Path, PathBuf};

use ps_arch::BudFate;
use ps_graph::OrganKind;
use ps_world::{ActionKind, Catalog, ObstacleSpec, Scenario, SupportSpec, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn world(name: &str, days: u32) -> World {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.duration_days = days;
    let c = Catalog::load(&data(), &s).unwrap();
    World::new(s, c).unwrap()
}

fn total_mass(w: &World) -> f64 {
    w.state().history.last().unwrap().report.total_dry_mass
}

/// Buds of the main stem at or below `rank` that have grown into branches, and all
/// such buds that did not become racemes.
fn released(w: &World, rank: u32) -> (usize, usize) {
    let p = &w.state().plants[0].plant;
    let buds: Vec<_> = p
        .buds
        .iter()
        .filter(|b| b.axis == 0 && b.rank <= rank && b.fate != BudFate::Raceme)
        .collect();
    let grown = buds.iter().filter(|b| b.fate == BudFate::Branch).count();
    (grown, buds.len())
}

/// Done-when: topping a bean releases its buds. Four days after the cut, every bud
/// left below it has grown a branch, while the untopped twin still holds some dormant
/// under its growing apex.
fn topping_releases_buds(name: &str, day: u32) {
    let mut cut = world(name, day + 10);
    let mut kept = world(name, day + 10);
    cut.run_days(day).unwrap();
    kept.run_days(day).unwrap();
    let main = cut.state().plants[0].plant.axes[0].clone();
    cut.apply(ActionKind::Prune {
        plant: 0,
        organ: main.apex.0,
    })
    .unwrap();
    cut.run_days(4).unwrap();
    kept.run_days(4).unwrap();
    let below = main.nodes - 1;
    let (a, n) = released(&cut, below);
    let (b, _) = released(&kept, below);
    assert!(n >= 2, "{n} buds below the cut");
    assert_eq!(a, n, "topped: {a} of {n} buds grown");
    assert!(b < n, "untopped: {b} of {n} buds grown");
}

#[test]
fn topping_a_bush_bean_releases_its_buds() {
    topping_releases_buds("single_pot.ron", 30);
}

#[test]
fn topping_a_pole_bean_releases_its_buds() {
    topping_releases_buds("m2/pole_trellis.ron", 22);
}

/// A harvest picks a pod with its seeds into the seed library; only pods and ears can
/// be harvested.
#[test]
fn harvesting_a_pod_fills_the_seed_library() {
    let mut w = world("single_pot.ron", 80);
    w.run_to_end().unwrap();
    let plant = &w.state().plants[0].plant;
    let a = &plant.arena;
    let pod = a
        .alive_of(OrganKind::Pod)
        .max_by_key(|p| {
            a.children[p.index()]
                .iter()
                .filter(|c| a.alive[c.index()] && a.kind[c.index()] == OrganKind::SeedInPod)
                .count()
        })
        .unwrap();
    let seeds_before = plant.report().seeds;
    let leaf = a.alive_of(OrganKind::Leaf).next();
    w.apply(ActionKind::Harvest {
        plant: 0,
        organ: pod.0,
    })
    .unwrap();
    w.run_days(1).unwrap();
    let lot = &w.state().seed_library[0];
    assert!(lot.seeds > 0 && lot.seed_mass_g > 0.0, "{lot:?}");
    assert_eq!(lot.species, "bush_bean");
    let after = w.state().plants[0].plant.report().seeds;
    assert_eq!(seeds_before - lot.seeds, after);
    if let Some(l) = leaf.filter(|l| w.state().plants[0].plant.arena.alive[l.index()]) {
        assert!(
            w.apply(ActionKind::Harvest {
                plant: 0,
                organ: l.0,
            })
            .is_err()
        );
    }
}

/// A shade cloth placed over a pot cuts the plant's growth.
#[test]
fn an_obstacle_shades_the_plant() {
    let mut open = world("single_pot.ron", 40);
    let mut shaded = world("single_pot.ron", 40);
    shaded
        .apply(ActionKind::PlaceObstacle(ObstacleSpec {
            soil: 0,
            min_cm: (-100.0, -100.0, 80.0),
            max_cm: (100.0, 100.0, 81.0),
            transmittance: 0.3,
        }))
        .unwrap();
    open.run_to_end().unwrap();
    shaded.run_to_end().unwrap();
    let (a, b) = (total_mass(&open), total_mass(&shaded));
    assert!(b < 0.8 * a, "shaded {b:.2} g against open {a:.2} g");
}

/// An obstacle placed over a plot shades the shared light grid too.
#[test]
fn an_obstacle_shades_a_plot() {
    let mut open = world("m2/pole_row_no_trellis.ron", 30);
    let mut shaded = world("m2/pole_row_no_trellis.ron", 30);
    shaded
        .apply(ActionKind::PlaceObstacle(ObstacleSpec {
            soil: 0,
            min_cm: (-80.0, -60.0, 100.0),
            max_cm: (80.0, 60.0, 101.0),
            transmittance: 0.2,
        }))
        .unwrap();
    open.run_to_end().unwrap();
    shaded.run_to_end().unwrap();
    let mass = |w: &World| -> f64 {
        w.state()
            .plants
            .iter()
            .map(|p| p.plant.report().total_dry_mass)
            .sum()
    };
    let (a, b) = (mass(&open), mass(&shaded));
    assert!(b < 0.8 * a, "shaded {b:.2} g against open {a:.2} g");
}

/// Stakes stood beside an unstaked pole bean row on day 12 give the vines something to
/// climb: the row ends taller than without.
#[test]
fn a_support_added_mid_run_is_climbed() {
    let mut bare = world("m2/pole_row_no_trellis.ron", 50);
    let mut staked = world("m2/pole_row_no_trellis.ron", 50);
    bare.run_days(12).unwrap();
    staked.run_days(12).unwrap();
    for i in 0..8_i32 {
        let x = -48.5 + 15.0 * f64::from(i);
        staked
            .apply(ActionKind::AddSupport(SupportSpec {
                soil: 0,
                offset_cm: (x, 0.0),
                height_cm: 200.0,
                diameter_cm: 0.5,
            }))
            .unwrap();
    }
    bare.run_to_end().unwrap();
    staked.run_to_end().unwrap();
    let height = |w: &World| -> f64 {
        let p = &w.state().plants;
        #[allow(clippy::cast_precision_loss)]
        let n = p.len() as f64;
        p.iter().map(|p| p.plant.report().height).sum::<f64>() / n
    };
    let (a, b) = (height(&staked), height(&bare));
    assert!(a > 1.5 * b, "staked {a:.2} m against bare {b:.2} m");
    assert_eq!(staked.state().added_supports.len(), 8);
}

/// M2 exit: a weather file drives a run, and its weather is the file's, not a preset's.
#[test]
fn a_weather_file_drives_a_run() {
    let mut w = world("m2/single_pot_weather_file.ron", 40);
    w.run_to_end().unwrap();
    let rows = &w.state().history;
    assert_eq!(rows.len(), 40);
    assert!(total_mass(&w) > 1.0, "{:.2} g", total_mass(&w));
    let mut p = world("single_pot.ron", 40);
    p.run_to_end().unwrap();
    let t = |w: &World| -> Vec<f64> { w.state().history.iter().map(|r| r.env.t_max_c).collect() };
    assert_ne!(t(&w), t(&p));
}
