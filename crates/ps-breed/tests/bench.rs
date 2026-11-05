//! WP30: the seed library and the breeding bench (decision 0038).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ps_breed::{
    BenchCross, BenchPlan, BreedError, Looks, Origin, SeedLibrary, grow, load_species,
    run_generation,
};
use ps_species::Species;
use ps_world::{Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// Every species file in data/.
fn kin() -> BTreeMap<String, Species> {
    load_species(&data()).unwrap()
}

fn species(id: &str) -> Species {
    kin().remove(id).unwrap()
}

fn looks() -> Looks {
    Looks::load(&data(), &kin()).unwrap()
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
    let g = run_generation(&mut lib, &p, &c, &kin(), &looks).unwrap();
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
            // Selfed bush lines stay bush beans.
            assert_eq!(s.species, "bush_bean");
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

/// M3 exit, first steps: crosses on the bench give F1 seeds that name both parents, and
/// the F1s selfed on the next bench give F2 seeds that segregate for habit (fin). Fin is
/// dominant, so an F1 climbs whichever way it was crossed: it grows by the pole bean
/// file, and a bush (fin fin) F2 seed by the bush bean file (decision 0038 item 7).
#[test]
fn a_bench_cross_gives_an_f1_and_its_f2_segregates() {
    let (kin, looks) = (kin(), looks());
    let mut lib = SeedLibrary::new();
    let bush = lib
        .add_line("bush_bean", &kin["bush_bean"], &looks)
        .unwrap();
    let pole = lib
        .add_line("pole_bean", &kin["pole_bean"], &looks)
        .unwrap();
    let mut p = plan("cross", vec![bush, pole]);
    p.crosses = vec![
        BenchCross {
            mother: 0,
            father: 1,
        },
        BenchCross {
            mother: 1,
            father: 0,
        },
    ];
    let c = catalog(&lib, &p);
    let g = run_generation(&mut lib, &p, &c, &kin, &looks).unwrap();
    let fin = kin["bush_bean"]
        .genome
        .loci
        .iter()
        .position(|l| l.name.as_deref() == Some("fin"))
        .unwrap();
    // Alleles are (egg, pollen): fin from the bush, Fin from the pole.
    let lines = [
        (bush, pole, (0.0, 1.0), "bush_bean"),
        (pole, bush, (1.0, 0.0), "pole_bean"),
    ];
    let mut f1 = Vec::new();
    for (pot, &(mother, father, alleles, file)) in lines.iter().enumerate() {
        let row = &g.rows[pot];
        assert_eq!(row.crossed_with, Some(1 - pot));
        let (crossed, selfed): (Vec<u64>, Vec<u64>) = row
            .saved
            .iter()
            .partition(|&&id| lib.get(id).unwrap().crossed);
        assert!(!crossed.is_empty(), "{row:?}");
        for &id in &crossed {
            let s = lib.get(id).unwrap();
            assert_eq!((s.mother, s.father), (Some(mother), Some(father)));
            assert_eq!(s.genotype.alleles[fin], alleles);
            assert_eq!(s.species, "pole_bean", "an F1 climbs");
        }
        // Flowers open while the father has none self, true to the mother's file.
        for &id in &selfed {
            let s = lib.get(id).unwrap();
            assert_eq!(
                (s.mother, s.father, s.species.as_str()),
                (Some(mother), Some(mother), file)
            );
        }
        f1.push(crossed[0]);
    }

    // F2: one F1 from each mother, side by side and selfed.
    let p2 = plan("F2", f1);
    let c2 = catalog(&lib, &p2);
    let g2 = run_generation(&mut lib, &p2, &c2, &kin, &looks).unwrap();
    // The reciprocal F1s grow alike, as tall as the pole line and far above the bush.
    let (a, b) = (g2.rows[0].height_cm, g2.rows[1].height_cm);
    let (bush_cm, pole_cm) = (g.rows[0].height_cm, g.rows[1].height_cm);
    assert!(
        (a - b).abs() < 0.15 * a.max(b) && a.min(b) > 0.8 * pole_cm && a.min(b) > 2.0 * bush_cm,
        "F1s {a} and {b} cm, pole {pole_cm} cm, bush {bush_cm} cm"
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
    // A bush (fin fin) F2 seed grows by the bush file, a climbing one by the pole file.
    for s in &f2 {
        let (a, b) = s.genotype.alleles[fin];
        let file = if a + b < 0.5 {
            "bush_bean"
        } else {
            "pole_bean"
        };
        assert_eq!(s.species, file, "fin alleles ({a}, {b})");
    }
    assert!(f2.iter().any(|s| s.species == "bush_bean"));
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
    let new = lib
        .add_lots(w.state(), 0, "garden", &kin(), &looks)
        .unwrap();
    assert!(!new.is_empty());
    assert_eq!(lib.seeds.len(), before + new.len());
    assert!(
        new.iter()
            .all(|&id| lib.get(id).unwrap().mother == Some(ids[0]))
    );
    // Importing from the end adds nothing.
    let n = w.state().seed_library.len();
    assert!(
        lib.add_lots(w.state(), n, "garden", &kin(), &looks)
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
    // A cross between files of different genome layouts fails before the bench grows.
    let wheat = lib
        .add_line("spring_wheat", &species("spring_wheat"), &looks)
        .unwrap();
    let mut p = plan("x", vec![id, wheat]);
    p.crosses = vec![BenchCross {
        mother: 0,
        father: 1,
    }];
    let c = catalog(&lib, &p);
    match grow(&lib, &p, &c) {
        Err(BreedError::Plan(why)) => assert!(why.contains("genome layout"), "{why}"),
        other => panic!("{:?}", other.map(|w| w.tick())),
    }

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
