//! R3c (decision 0045): the water network's two levels and switching between them.
//! The level is part of each plant's state; `Auto` runs pots detailed and plots of
//! more than eight plants simplified; `SetFidelity` actions switch plants at the next
//! hour and replay exactly; a locked scenario refuses them. The fidelity report
//! (`ps-cli fidelity`) compares the levels' outcomes.

use std::path::{Path, PathBuf};

use ps_world::{
    ActionKind, Catalog, Fidelity, HydraulicsLevel, MAX_DETAILED, Scenario, World, WorldError,
};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn load(name: &str, days: u32) -> (Scenario, Catalog) {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.duration_days = days;
    let c = Catalog::load(&data(), &s).unwrap();
    (s, c)
}

fn levels(w: &World) -> Vec<HydraulicsLevel> {
    w.state()
        .plants
        .iter()
        .map(|p| p.plant.hydraulics_level)
        .collect()
}

#[test]
fn auto_runs_pots_detailed_and_big_plots_simplified() {
    let (s, c) = load("single_pot.ron", 1);
    let w = World::new(s, c).unwrap();
    assert_eq!(levels(&w), vec![HydraulicsLevel::Detailed]);
    let (s, c) = load("plot_30_beans.ron", 1);
    assert!(s.sowings.len() > MAX_DETAILED);
    let w = World::new(s, c).unwrap();
    assert!(levels(&w).iter().all(|&l| l == HydraulicsLevel::Simplified));
    let (mut s, c) = load("plot_30_beans.ron", 1);
    s.fidelity = Fidelity::Detailed;
    let w = World::new(s, c).unwrap();
    assert!(levels(&w).iter().all(|&l| l == HydraulicsLevel::Detailed));
}

/// The simplified pot grows like the detailed one over a month (the report checks the
/// season on four seeds) and its water ledger closes every hour (debug builds).
#[test]
fn a_simplified_pot_tracks_the_detailed_one() {
    let (s, c) = load("single_pot.ron", 30);
    let mut simple = s.clone();
    simple.fidelity = Fidelity::Simplified;
    let mut d = World::new(s, c.clone()).unwrap();
    d.run_to_end().unwrap();
    let mut q = World::new(simple, c).unwrap();
    q.run_to_end().unwrap();
    assert_eq!(levels(&q), vec![HydraulicsLevel::Simplified]);
    let last = |w: &World| w.state().history.last().unwrap().report;
    let (a, b) = (last(&d), last(&q));
    let gap = (b.total_dry_mass - a.total_dry_mass) / a.total_dry_mass;
    println!(
        "day 30: detailed {:.3} g, simplified {:.3} g ({:+.1}%); K_rs {:.3} kg/h/MPa, \
         uptake depth {:.3} m",
        a.total_dry_mass,
        b.total_dry_mass,
        gap * 100.0,
        b.root_conductance,
        b.uptake_depth
    );
    assert!(
        gap.abs() < 0.1,
        "simplified mass off by {:+.1}%",
        gap * 100.0
    );
    assert!(b.root_conductance > 0.0 && b.uptake_depth > 0.0);
    assert!(
        (a.root_conductance - b.root_conductance).abs() < 0.2 * a.root_conductance,
        "the summary is built at both levels"
    );
}

#[test]
fn switching_is_logged_and_replays_to_the_same_hash() {
    let (s, c) = load("single_pot.ron", 12);
    let mut w = World::new(s.clone(), c.clone()).unwrap();
    w.run_until(5 * 24 + 7).unwrap();
    w.apply(ActionKind::SetFidelity {
        plants: vec![0],
        level: HydraulicsLevel::Simplified,
    })
    .unwrap();
    w.run_until(8 * 24).unwrap();
    assert_eq!(levels(&w), vec![HydraulicsLevel::Simplified]);
    w.apply(ActionKind::SetFidelity {
        plants: Vec::new(),
        level: HydraulicsLevel::Detailed,
    })
    .unwrap();
    w.run_to_end().unwrap();
    assert_eq!(levels(&w), vec![HydraulicsLevel::Detailed]);
    let log = w.state().log.clone();
    assert_eq!(log.len(), 2);
    let replayed = World::replay(s.clone(), c.clone(), &log, w.tick()).unwrap();
    assert_eq!(replayed.state().hashes, w.state().hashes);
    assert_eq!(replayed.state_hash(), w.state_hash());
    // The switch changed the run: the unswitched pot hashes differently from day 6.
    let mut plain = World::new(s, c).unwrap();
    plain.run_to_end().unwrap();
    assert_eq!(plain.state().hashes[..5], w.state().hashes[..5]);
    assert_ne!(plain.state().hashes[5], w.state().hashes[5]);
}

#[test]
fn a_locked_scenario_and_unknown_plants_refuse_set_fidelity() {
    let (mut s, c) = load("single_pot.ron", 2);
    let mut w = World::new(s.clone(), c.clone()).unwrap();
    let unknown = w.apply(ActionKind::SetFidelity {
        plants: vec![3],
        level: HydraulicsLevel::Simplified,
    });
    assert!(
        matches!(unknown, Err(WorldError::Action { .. })),
        "{unknown:?}"
    );
    s.fidelity = Fidelity::Locked(HydraulicsLevel::Simplified);
    let mut w = World::new(s, c).unwrap();
    assert_eq!(levels(&w), vec![HydraulicsLevel::Simplified]);
    let locked = w.apply(ActionKind::SetFidelity {
        plants: Vec::new(),
        level: HydraulicsLevel::Detailed,
    });
    assert!(
        matches!(locked, Err(WorldError::Action { .. })),
        "{locked:?}"
    );
    assert!(w.state().log.is_empty());
}
