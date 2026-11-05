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
    assert_eq!(
        ps_world::hash_of(&one.state().soils[0]),
        ps_world::hash_of(&two.state().soils[0])
    );
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

#[test]
fn pruning_removes_carbon_and_refuses_the_hypocotyl() {
    let (s, c) = short("single_pot.ron", 30);
    let mut w = World::new(s, c).unwrap();
    w.run_days(28).unwrap();
    let p = &w.state().plants[0].plant;
    let hyp = p.hypocotyl.0;
    let litter = p.ledger.litter;
    let leaf = p
        .arena
        .alive_of(ps_graph::OrganKind::Leaf)
        .last()
        .map(|id| id.0)
        .unwrap();
    assert!(
        w.apply(ActionKind::Prune {
            plant: 0,
            organ: hyp
        })
        .is_err()
    );
    w.apply(ActionKind::Prune {
        plant: 0,
        organ: leaf,
    })
    .unwrap();
    // Actions apply at the start of the next hourly step.
    w.run_days(1).unwrap();
    let p = &w.state().plants[0].plant;
    assert!(!p.arena.alive[leaf as usize]);
    assert!(p.ledger.removed.0 > 0.0, "{:?}", p.ledger);
    // The cut leaf went with the gardener, not to litter (other organs may still shed).
    assert!(p.ledger.litter.0 - litter.0 < p.ledger.removed.0);
}

#[test]
fn scenario_events_and_heat_waves_apply() {
    let (mut s, c) = short("single_pot.ron", 3);
    s.events = vec![Action {
        tick: 30,
        kind: ActionKind::SetRadiationFraction {
            soil: 0,
            fraction: 0.25,
        },
    }];
    s.heat_wave = Some(ps_weather::presets::HeatWave {
        start_day: 1,
        days: 1,
        t_max_c: 32.0,
        t_min_c: 20.0,
    });
    let w = run(&s, &c);
    assert!((w.state().soils[0].radiation_fraction - 0.25).abs() < 1e-15);
    // Events are part of the scenario, not the player's log.
    assert!(w.state().log.is_empty());
    let env = |d: u32| w.state().history.iter().find(|r| r.day == d).unwrap().env;
    assert!(
        env(1).t_max_c > 31.0 && env(0).t_max_c < 31.0,
        "{:?}",
        env(1)
    );
    assert!(env(2).radiation_mj < env(0).radiation_mj);
}

#[test]
fn invalid_start_dates_are_rejected() {
    let (mut s, _) = load("single_pot.ron");
    s.start.month = 2;
    s.start.day = 30;
    assert!(s.validate().is_err());
}
