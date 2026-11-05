//! WP9 done-when and the §11.5 determinism checklist.

use std::path::{Path, PathBuf};

use ps_world::{
    Action, ActionKind, Catalog, Irrigation, Scenario, SeedChoice, SoilSpec, Sowing, World,
};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn load(name: &str) -> (Scenario, Catalog) {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let s = Scenario::from_ron(&text).unwrap();
    let c = Catalog::load(&data(), &s).unwrap();
    (s, c)
}

fn short(name: &str, days: u32) -> (Scenario, Catalog) {
    let (mut s, c) = load(name);
    s.duration_days = days;
    (s, c)
}

fn run(s: &Scenario, c: &Catalog) -> World {
    let mut w = World::new(s.clone(), c.clone()).unwrap();
    w.run_to_end().unwrap();
    w
}

/// Done-when 1 and checklist 1: three runs give one hash, every day.
#[test]
fn three_runs_give_one_hash() {
    let (s, c) = load("single_pot.ron");
    let a = run(&s, &c);
    let b = run(&s, &c);
    let d = run(&s, &c);
    assert_eq!(a.state().hashes.len(), 90);
    assert_eq!(a.state().hashes, b.state().hashes);
    assert_eq!(a.state().hashes, d.state().hashes);
    assert_eq!(a.state_hash(), d.state_hash());
    // The plant grew: the hash is not of an empty pot.
    let last = &a.state().history.last().unwrap().report;
    assert!(last.pods >= 5, "{last:?}");
}

/// Done-when 2 and checklist 4: a run resumed from a day-30 snapshot (through its RON
/// text) gives the uninterrupted run's day-90 hash.
#[test]
fn resume_from_day_30_snapshot() {
    let (s, c) = load("single_pot.ron");
    let full = run(&s, &c);
    let mut w = World::new(s, c).unwrap();
    w.run_days(30).unwrap();
    let text = w.snapshot_ron().unwrap();
    drop(w);
    let mut resumed = World::from_snapshot_ron(&text).unwrap();
    assert_eq!(resumed.tick(), 30 * 24);
    resumed.run_to_end().unwrap();
    assert_eq!(resumed.state().hashes, full.state().hashes);
    assert_eq!(resumed.state_hash(), full.state_hash());
}

/// Checklist 2: adding a plant in another pot does not change the first plant.
#[test]
fn extra_plant_does_not_change_the_first() {
    let (s, c) = short("single_pot.ron", 40);
    let one = run(&s, &c);
    let mut s2 = s.clone();
    s2.soils.push(SoilSpec::Pot {
        id: 1,
        diameter_cm: 20.0,
        depth_cm: 20.0,
        texture: ps_soil::Texture::Loam,
        position_m: (0.6, 0.0),
    });
    s2.irrigation
        .insert(1, Irrigation::ToFieldCapacity { every_days: 1 });
    s2.sowings.push(Sowing {
        soil: 1,
        species: "bush_bean".into(),
        seed: SeedChoice::Default,
        depth_cm: 3.0,
        offset_cm: None,
    });
    let two = run(&s2, &c);
    assert_eq!(one.plant_hash(0), two.plant_hash(0));
    assert_ne!(one.state_hash(), two.state_hash());
}

/// Checklist 3: rayon on one thread gives the multi-threaded hash.
#[test]
fn one_thread_matches_many() {
    let (s, c) = short("single_pot.ron", 40);
    let many = run(&s, &c);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let one = pool.install(|| run(&s, &c));
    assert_eq!(one.state().hashes, many.state().hashes);
}

/// Checklist 5: replaying a recorded action log gives the session's hash.
#[test]
fn replay_of_the_action_log_matches_the_session() {
    let (s, c) = short("two_pot_sun_shade.ron", 40);
    let mut session = World::new(s.clone(), c.clone()).unwrap();
    session.run_days(10).unwrap();
    session
        .apply(ActionKind::Water { soil: 1, mm: 5.0 })
        .unwrap();
    session.run_days(15).unwrap();
    session
        .apply(ActionKind::SetRadiationFraction {
            soil: 1,
            fraction: 1.0,
        })
        .unwrap();
    session.run_days(5).unwrap();
    // Prune plant 0's newest leaf.
    let leaf = session.state().plants[0]
        .plant
        .arena
        .alive_of(ps_graph::OrganKind::Leaf)
        .last()
        .map(|id| id.0)
        .unwrap();
    session
        .apply(ActionKind::Prune {
            plant: 0,
            organ: leaf,
        })
        .unwrap();
    session.run_to_end().unwrap();
    let log: Vec<Action> = session.state().log.clone();
    assert_eq!(log.len(), 3);
    let replayed = World::replay(s, c, &log, session.tick()).unwrap();
    assert_eq!(replayed.state().hashes, session.state().hashes);
    assert_eq!(replayed.state_hash(), session.state_hash());
}

#[test]
fn actions_in_the_past_are_rejected() {
    let (s, c) = short("single_pot.ron", 3);
    let mut w = World::new(s, c).unwrap();
    w.run_days(1).unwrap();
    let e = w.schedule(Action {
        tick: 3,
        kind: ActionKind::Water { soil: 0, mm: 1.0 },
    });
    assert!(e.is_err());
}

#[test]
fn shade_cloth_slows_the_shaded_plant() {
    let (s, c) = short("two_pot_sun_shade.ron", 50);
    let w = run(&s, &c);
    let last = |p: u32| {
        w.state()
            .history
            .iter()
            .rev()
            .find(|r| r.plant == p)
            .unwrap()
            .report
    };
    let (sun, shade) = (last(0), last(1));
    assert!(
        shade.dry_mass < sun.dry_mass,
        "sun {sun:?}\nshade {shade:?}"
    );
}
