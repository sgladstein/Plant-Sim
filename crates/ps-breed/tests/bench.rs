//! WP30: the seed library and the breeding bench (decision 0038).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ps_breed::{
    BenchCross, BenchPlan, BreedError, Looks, Origin, SeedLibrary, grow, run_generation,
};
use ps_species::Species;
use ps_world::{Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn species(id: &str) -> Species {
    let text = std::fs::read_to_string(data().join(format!("species/{id}.ron"))).unwrap();
    Species::from_ron(&text).unwrap()
}

fn looks() -> Looks {
    let mut sp = BTreeMap::new();
    for id in ["bush_bean", "pole_bean"] {
        sp.insert(id.to_owned(), species(id));
    }
    Looks::load(&data(), &sp).unwrap()
}

fn plan(label: &str, seeds: Vec<u64>) -> BenchPlan {
    BenchPlan {
        label: label.into(),
        seeds,
        crosses: Vec::new(),
        world_seed: 11,
        max_days: 120,
        conditions: ps_breed::BenchConditions::default(),
    }
}

fn catalog(lib: &SeedLibrary, p: &BenchPlan) -> Catalog {
    Catalog::load(&data(), &p.scenario(lib).unwrap()).unwrap()
}

/// WP30 done-when (correctness half; the wall time is measured in release by
/// `ps-bench --example breed_generation`): ten library seeds grow on the bench in one
/// generation, every plant is measured, and every seed it gives joins the library with
/// its mother and generation.
#[test]
fn a_ten_seed_generation_fills_the_library() {
    let looks = looks();
    let mut lib = SeedLibrary::new();
    let bush = species("bush_bean");
    let mut ids = vec![lib.add_line("bush_bean", &bush, &looks).unwrap()];
    ids.extend(lib.add_founders("bush_bean", &bush, 9, 3, &looks).unwrap());
    assert_eq!(ids, (0..10).collect::<Vec<u64>>());
    assert!(
        lib.seeds
            .iter()
            .all(|s| s.generation == 0 && s.coat.is_some())
    );

    let p = plan("bench 1", ids.clone());
    let c = catalog(&lib, &p);
    let g = run_generation(&mut lib, &p, &c, &looks).unwrap();
    assert_eq!(g.rows.len(), 10);
    assert!(
        g.days < 120,
        "the bench stops once all are mature ({} days)",
        g.days
    );
    for (i, r) in g.rows.iter().enumerate() {
        assert_eq!((r.pot, r.seed), (i, ids[i]));
        assert!(
            r.emergence_day.is_some() && r.flowering_day.is_some(),
            "{r:?}"
        );
        assert!(r.maturity_day.is_some(), "{r:?}");
        assert!(r.height_cm > 10.0 && r.leaf_area_cm2 > 100.0 && r.dry_mass_g > 1.0);
        assert!(r.seeds > 0 && r.pods > 0 && r.mean_seed_mg > 50.0, "{r:?}");
        assert_eq!(r.saved.len(), r.seeds as usize);
        assert!(r.look.is_some());
        for &id in &r.saved {
            let s = lib.get(id).unwrap();
            assert_eq!((s.mother, s.father), (Some(ids[i]), Some(ids[i])));
            assert!(!s.crossed);
            assert_eq!(s.generation, 1);
            assert!(
                matches!(&s.origin, Origin::Harvest { run, plant, mature: true, .. }
                if run == "bench 1" && *plant as usize == i)
            );
            // A seed's coat is its mother's: one look per plant.
            assert_eq!(s.coat, r.look);
        }
    }
    assert_eq!(
        g.saved.len(),
        g.rows.iter().map(|r| r.saved.len()).sum::<usize>()
    );
    // Plants of different lines differ measurably.
    let yields: Vec<f64> = g.rows.iter().map(|r| r.mean_seed_mg).collect();
    let (lo, hi) = yields
        .iter()
        .fold((f64::MAX, 0.0_f64), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(hi > lo * 1.1, "mean seed mass {lo} to {hi} mg");
    // The library survives a round trip through its file.
    let back = SeedLibrary::from_ron(&lib.to_ron().unwrap()).unwrap();
    assert_eq!(back, lib);
}

/// M3 exit, first steps: a cross on the bench gives F1 seeds that name both parents;
/// the F1 selfed on the next bench gives F2 seeds that segregate for habit (fin).
#[test]
fn a_bench_cross_gives_an_f1_and_its_f2_segregates() {
    let looks = looks();
    let mut lib = SeedLibrary::new();
    let bush = lib
        .add_line("bush_bean", &species("bush_bean"), &looks)
        .unwrap();
    let pole = lib
        .add_line("pole_bean", &species("pole_bean"), &looks)
        .unwrap();
    let mut p = plan("cross", vec![bush, pole]);
    p.crosses = vec![BenchCross {
        mother: 0,
        father: 1,
    }];
    let c = catalog(&lib, &p);
    let g = run_generation(&mut lib, &p, &c, &looks).unwrap();
    assert_eq!(g.rows[0].crossed_with, Some(1));
    let f1: Vec<u64> = g.rows[0]
        .saved
        .iter()
        .copied()
        .filter(|&id| lib.get(id).unwrap().crossed)
        .collect();
    assert!(!f1.is_empty(), "{:?}", g.rows[0]);
    let fin = species("bush_bean")
        .genome
        .loci
        .iter()
        .position(|l| l.name.as_deref() == Some("fin"))
        .unwrap();
    for &id in &f1 {
        let s = lib.get(id).unwrap();
        assert_eq!((s.mother, s.father), (Some(bush), Some(pole)));
        assert_eq!(s.species, "bush_bean");
        assert_eq!(
            s.genotype.alleles[fin],
            (0.0, 1.0),
            "fin from the bush, Fin from the pole"
        );
    }
    // The pole father's own seeds are selfed.
    assert!(g.rows[1].saved.iter().all(|&id| {
        let s = lib.get(id).unwrap();
        !s.crossed && s.father == Some(pole)
    }));

    // F2: two F1 seeds, selfed.
    let p2 = plan("F2", f1.iter().copied().cycle().take(2).collect());
    let c2 = catalog(&lib, &p2);
    let g2 = run_generation(&mut lib, &p2, &c2, &looks).unwrap();
    // The F1 climbs (Fin is dominant): taller than the bush line.
    assert!(
        g2.rows[0].height_cm > g.rows[0].height_cm,
        "F1 {} cm, bush {} cm",
        g2.rows[0].height_cm,
        g.rows[0].height_cm
    );
    let f2: Vec<_> = g2.saved.iter().map(|&id| lib.get(id).unwrap()).collect();
    assert!(f2.len() >= 8, "{} F2 seeds", f2.len());
    assert!(f2.iter().all(|s| s.generation == 2 && !s.crossed));
    let classes: std::collections::BTreeSet<u64> = f2
        .iter()
        .map(|s| {
            let (a, b) = s.genotype.alleles[fin];
            (a + b).to_bits()
        })
        .collect();
    assert!(classes.len() >= 2, "fin segregates in the F2: {classes:?}");
}

/// The same plan from the same library gives the same generation; a library seed sown
/// in the garden keeps its id, and garden harvests join the library with it as mother.
#[test]
fn generations_repeat_and_garden_seeds_join_the_library() {
    let looks = looks();
    let mut lib = SeedLibrary::new();
    let bush = species("bush_bean");
    let ids = lib.add_founders("bush_bean", &bush, 2, 5, &looks).unwrap();
    let mut p = plan("repeat", ids.clone());
    p.max_days = 50;
    let c = catalog(&lib, &p);
    let (a, b) = (grow(&lib, &p, &c).unwrap(), grow(&lib, &p, &c).unwrap());
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.state().plants[1].seed_id, Some(ids[1]));

    // The bench world is an ordinary scenario: run it as a garden for 85 days, pick
    // one pod and import it.
    let mut s: Scenario = p.scenario(&lib).unwrap();
    s.duration_days = 85;
    let mut w = World::new(s, c).unwrap();
    w.run_to_end().unwrap();
    let pod = {
        let arena = &w.state().plants[0].plant.arena;
        arena
            .alive_of(ps_graph::OrganKind::Pod)
            .find(|&pod| {
                arena.children[pod.index()]
                    .iter()
                    .any(|c| arena.alive[c.index()])
            })
            .unwrap()
    };
    w.apply(ps_world::ActionKind::Harvest {
        plant: 0,
        organ: pod.0,
    })
    .unwrap();
    w.run_until(w.tick() + 1).unwrap();
    let before = lib.seeds.len();
    let new = lib.add_lots(w.state(), 0, "garden", &looks).unwrap();
    assert!(!new.is_empty());
    assert_eq!(lib.seeds.len(), before + new.len());
    assert!(
        new.iter()
            .all(|&id| lib.get(id).unwrap().mother == Some(ids[0]))
    );
    // Importing from the end adds nothing.
    let n = w.state().seed_library.len();
    assert!(
        lib.add_lots(w.state(), n, "garden", &looks)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn plans_and_libraries_are_checked() {
    let looks = looks();
    let mut lib = SeedLibrary::new();
    let id = lib
        .add_line("bush_bean", &species("bush_bean"), &looks)
        .unwrap();
    let err = |p: &BenchPlan| match p.check(&lib) {
        Err(BreedError::Plan(why)) => why,
        other => panic!("{other:?}"),
    };
    assert!(err(&plan("x", vec![])).contains("1 to 10"));
    assert!(err(&plan("x", vec![id; 11])).contains("1 to 10"));
    assert!(err(&plan("x", vec![7])).contains("not in the library"));
    let mut p = plan("x", vec![id, id]);
    p.crosses = vec![BenchCross {
        mother: 0,
        father: 0,
    }];
    assert!(err(&p).contains("two different pots"));
    p.crosses = vec![
        BenchCross {
            mother: 0,
            father: 1,
        },
        BenchCross {
            mother: 0,
            father: 1,
        },
    ];
    assert!(err(&p).contains("two crosses"));
    p.crosses.clear();
    p.max_days = 0;
    assert!(err(&p).contains("max_days"));
    // A pure line seed may grow twice on one bench (replicates).
    p.max_days = 10;
    assert!(p.check(&lib).is_ok());

    // Ids must increase.
    let mut bad = lib.clone();
    bad.seeds.push(bad.seeds[0].clone());
    bad.next_id = 5;
    assert!(SeedLibrary::from_ron(&bad.to_ron().unwrap()).is_err());
    // The bean colour table covers both bean files.
    assert_eq!(
        looks.0.keys().map(String::as_str).collect::<Vec<_>>(),
        ["bush_bean", "pole_bean"]
    );
    // A plan file reads with the default conditions.
    let p: BenchPlan = ron::from_str("(label: \"g1\", seeds: [0], world_seed: 1)").unwrap();
    assert_eq!(p.max_days, 120);
    assert!((p.conditions.pot_diameter_cm - 40.0).abs() < 1e-12);
}
