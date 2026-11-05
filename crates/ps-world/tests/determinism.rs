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
    // The biggest leaf, so the cut removes some carbon (the newest may be a primordium).
    let leaf = p
        .arena
        .alive_of(ps_graph::OrganKind::Leaf)
        .max_by(|a, b| {
            let m = |id: ps_graph::OrganId| p.arena.size[id.index()].area.0;
            m(*a).total_cmp(&m(*b))
        })
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

/// WP21 (decision 0019): a plot grows its plants in one soil and one light grid, and
/// the result does not depend on how many threads step the plants.
#[test]
fn a_plot_is_independent_of_the_thread_count() {
    let (s, c) = short("plot_30_beans.ron", 20);
    let hash = |threads: usize| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        pool.install(|| run(&s, &c).state_hash())
    };
    assert_eq!(hash(1), hash(4));
}

#[test]
fn neighbors_in_a_plot_take_each_others_light() {
    // The same plant alone on the plot and among 29 neighbors: once the canopy is
    // up, its leaves get less light in the crowd.
    let (crowd, c) = short("plot_30_beans.ron", 45);
    let mut alone = crowd.clone();
    let center = 14;
    alone.sowings = vec![crowd.sowings[center].clone()];
    let light = |s: &Scenario, plant: usize| {
        let w = run(s, &c);
        let p = &w.state().plants[plant].plant;
        let leaves: Vec<_> = p.arena.alive_of(ps_graph::OrganKind::Leaf).collect();
        let area: f64 = leaves
            .iter()
            .map(|id| p.arena.size[id.index()].area.0)
            .sum();
        leaves
            .iter()
            .map(|id| p.arena.size[id.index()].area.0 * p.meta[id.index()].light_index)
            .sum::<f64>()
            / area
    };
    let (in_crowd, by_itself) = (light(&crowd, center), light(&alone, 0));
    assert!(
        in_crowd < 0.85 * by_itself,
        "light index {in_crowd:.3} in the plot vs {by_itself:.3} alone"
    );
}

#[test]
fn plots_outside_the_size_range_are_rejected() {
    let (ok, _) = load("plot_30_beans.ron");
    assert!(ok.validate().is_ok());
    for (w, l) in [(90.0, 120.0), (150.0, 320.0), (f64::NAN, 120.0)] {
        let mut s = ok.clone();
        if let ps_world::scenario::SoilSpec::Plot {
            width_cm,
            length_cm,
            ..
        } = &mut s.soils[0]
        {
            *width_cm = w;
            *length_cm = l;
        }
        assert!(s.validate().is_err(), "{w} x {l} cm accepted");
    }
}

/// Decision 0027: the plants of a plot differ, and a plant's own draws do not depend
/// on the other plants.
#[test]
fn plot_plants_differ_and_keep_their_draws() {
    let (s, c) = short("plot_30_beans.ron", 1);
    let w = World::new(s.clone(), c.clone()).unwrap();
    let plants = &w.state().plants;
    let firsts: Vec<_> = plants.iter().map(|p| p.plant.individual).collect();
    assert!(firsts.windows(2).all(|p| p[0] != p[1]));
    let mut fewer = s.clone();
    fewer.sowings.truncate(5);
    let w5 = World::new(fewer, c.clone()).unwrap();
    for (a, b) in w5.state().plants.iter().zip(plants) {
        assert_eq!(a.plant.individual, b.plant.individual);
        assert_eq!(
            a.plant.collar_position.map(f64::to_bits),
            b.plant.collar_position.map(f64::to_bits)
        );
    }
    // No field variation: every seed sits on its mark at its depth.
    let mut exact = s;
    exact.variation = ps_world::FieldVariation::NONE;
    let we = World::new(exact.clone(), c).unwrap();
    for (p, sow) in we.state().plants.iter().zip(&exact.sowings) {
        assert!((p.plant.collar_position[2] + sow.depth_cm * 0.01).abs() < 1e-12);
    }
    for (p, q) in we.state().plants.iter().zip(plants) {
        let d = (p.plant.collar_position[0] - q.plant.collar_position[0]).abs()
            + (p.plant.collar_position[1] - q.plant.collar_position[1]).abs();
        assert!(d < 0.06, "scatter stays within 2.5 sd: {d}");
    }
}

/// Decision 0027: a plot's soil nitrogen is patchy around the scenario's mean, and a pot
/// stays uniform.
#[test]
fn plot_soil_nitrogen_is_patchy() {
    let (s, c) = short("plot_30_beans.ron", 1);
    let w = World::new(s.clone(), c.clone()).unwrap();
    let g = &w.state().soils[0].grid;
    let (nx, ny, _) = g.dims();
    let col: Vec<f64> = (0..nx * ny).map(|i| g.no3(i).0).collect();
    #[allow(clippy::cast_precision_loss)]
    let n = col.len() as f64;
    let mean = col.iter().sum::<f64>() / n;
    let sd = libm::sqrt(col.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n);
    let mut flat = s;
    flat.variation = ps_world::FieldVariation::NONE;
    let uniform = World::new(flat, c).unwrap().state().soils[0].grid.no3(0).0;
    assert!(
        (mean / uniform - 1.0).abs() < 0.2,
        "mean {mean} vs {uniform}"
    );
    assert!((0.1..0.45).contains(&(sd / mean)), "cv {}", sd / mean);
    let (p, pc) = short("single_pot.ron", 1);
    let wp = World::new(p, pc).unwrap();
    let pg = &wp.state().soils[0].grid;
    let first = (0..pg.len())
        .map(|i| pg.no3(i).0)
        .find(|&v| v > 0.0)
        .unwrap();
    assert!((0..pg.len()).all(|i| {
        let v = pg.no3(i).0;
        v.abs() < 1e-300 || (v - first).abs() < 1e-15
    }));
}

/// Decision 0029: a pole bean sown beside its stake is never scattered away from it, over
/// many world seeds.
#[test]
fn seeds_sown_at_a_support_stay_at_it() {
    let (s, c) = short("m2/pole_trellis.ron", 1);
    for seed in 0..64 {
        let mut t = s.clone();
        t.world_seed = seed;
        let w = World::new(t.clone(), c.clone()).unwrap();
        for p in &w.state().plants {
            let k = &p.plant.supports[0];
            let at = p.plant.collar_position;
            let d = libm::hypot(at[0] - k.base[0], at[1] - k.base[1]);
            assert!(
                d <= 0.05 + 1e-12,
                "seed {seed}, plant {}: {d} m from the stake",
                p.id
            );
        }
    }
}

/// Decision 0029: soil patch lattice nodes are keyed by where they are, so enlarging a
/// bed keeps the patches of the part it already had.
#[test]
fn soil_patches_stay_put_when_a_bed_grows() {
    let (s, c) = short("plot_30_beans.ron", 1);
    let field = |width: f64| {
        let mut t = s.clone();
        if let ps_world::scenario::SoilSpec::Plot { width_cm, .. } = &mut t.soils[0] {
            *width_cm = width;
        }
        t.sowings.clear();
        let w = World::new(t, c.clone()).unwrap();
        let g = &w.state().soils[0].grid;
        let (nx, ny, _) = g.dims();
        (
            nx,
            ny,
            (0..nx * ny).map(|i| g.no3(i).0).collect::<Vec<f64>>(),
        )
    };
    let (nx, ny, small) = field(150.0);
    let (wx, wy, big) = field(240.0);
    assert_eq!(ny, wy);
    assert!(wx > nx);
    // Columns away from the small bed's east edge (where its last lattice cell is
    // clamped) see the same four nodes in both beds.
    for y in 0..ny {
        for x in 0..nx - 6 {
            assert_eq!(
                small[y * nx + x].to_bits(),
                big[y * wx + x].to_bits(),
                "column ({x}, {y})"
            );
        }
    }
}
