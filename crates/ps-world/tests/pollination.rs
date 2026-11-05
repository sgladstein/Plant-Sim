//! WP29 done-when: a cross records both parents in each seed. Flowers self by default;
//! a hand-pollination puts another plant's pollen on an open flower, and each seed's
//! genotype is drawn at seed set from its mother's egg and its father's pollen
//! (decision 0035).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ps_graph::{OrganId, OrganKind};
use ps_species::Genotype;
use ps_world::{ActionKind, Catalog, Scenario, SeedChoice, SeedGenotype, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// Two bush beans of different sampled lines in full-sun pots: they flower together.
fn two_pots(days: u32) -> (Scenario, Catalog) {
    let text = std::fs::read_to_string(data().join("scenarios/two_pot_sun_shade.ron")).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.radiation_fraction.insert(1, 1.0);
    for sow in &mut s.sowings {
        sow.seed = SeedChoice::Sample;
    }
    s.duration_days = days;
    let c = Catalog::load(&data(), &s).unwrap();
    (s, c)
}

/// The living flowers of a plant that have not been hand-pollinated.
fn open_flowers(w: &World, plant: usize) -> Vec<u32> {
    let p = &w.state().plants[plant].plant;
    p.flowers
        .iter()
        .filter(|f| p.arena.alive[f.id.index()] && f.pollen.is_none())
        .map(|f| f.id.0)
        .collect()
}

/// Living seeds of a pod or ear.
fn seeds_of(w: &World, plant: usize, pod: OrganId) -> Vec<u32> {
    let a = &w.state().plants[plant].plant.arena;
    a.children[pod.index()]
        .iter()
        .filter(|c| a.alive[c.index()] && a.kind[c.index()] == OrganKind::SeedInPod)
        .map(|c| c.0)
        .collect()
}

/// Harvests every pod with seeds on `plant`; checks that each lot holds the seeds'
/// records as they were read on the plant, and returns them.
fn harvest_all(w: &mut World, plant: usize) -> Vec<SeedGenotype> {
    let pods: Vec<OrganId> = w.state().plants[plant]
        .plant
        .arena
        .alive_of(OrganKind::Pod)
        .collect();
    let id = u32::try_from(plant).unwrap();
    let mut expected = Vec::new();
    for pod in pods {
        let seeds = seeds_of(w, plant, pod);
        if seeds.is_empty() {
            continue;
        }
        let records: Vec<SeedGenotype> = seeds
            .iter()
            .map(|&s| w.seed_genotype(id, s).expect("a living seed has a record"))
            .collect();
        w.apply(ActionKind::Harvest {
            plant: id,
            organ: pod.0,
        })
        .unwrap();
        expected.push((pod.0, records));
    }
    w.run_until(w.tick() + 1).unwrap();
    let mut out = Vec::new();
    for (pod, records) in expected {
        let lot = w
            .state()
            .seed_library
            .iter()
            .find(|l| l.plant == id && l.organ == pod)
            .expect("the harvest made a lot");
        assert_eq!(lot.seed_genotypes.len(), lot.seeds as usize);
        // The genotype was fixed when the seed was made, not when it was picked.
        assert_eq!(lot.seed_genotypes, records);
        out.extend(records);
    }
    out
}

/// Loci where `seed` pairs `mother`'s first allele with `father`'s, out of all loci.
fn parental_loci(seed: &Genotype, mother: &Genotype, father: &Genotype) -> (usize, usize) {
    let same = |x: f64, y: f64| x.to_bits() == y.to_bits();
    let hits = seed
        .alleles
        .iter()
        .zip(mother.alleles.iter().zip(&father.alleles))
        .filter(|((a, b), (m, f))| same(*a, m.0) && same(*b, f.0))
        .count();
    (hits, seed.alleles.len())
}

/// WP29 done-when: every seed of a crossed flower names its mother and its father, and
/// carries one allele from each; seeds of flowers left alone name the mother twice.
/// The run replays from its action log to the same hash.
#[test]
fn a_cross_records_both_parents_in_each_seed() {
    let (s, c) = two_pots(75);
    let mut w = World::new(s.clone(), c.clone()).unwrap();
    let mut crossed = BTreeSet::new();
    while w.tick() < 75 * 24 {
        w.run_days(1).unwrap();
        if !w.state().plants[1].plant.has_open_flower() {
            continue;
        }
        for f in open_flowers(&w, 0) {
            if crossed.insert(f) {
                w.apply(ActionKind::Pollinate {
                    plant: 0,
                    organ: f,
                    pollen: 1,
                })
                .unwrap();
            }
        }
    }
    assert!(crossed.len() >= 10, "{} flowers crossed", crossed.len());
    // Pods of crossed flowers carry the pollen parent.
    let pods = &w.state().plants[0].plant.pods;
    assert!(pods.iter().any(|p| p.pollen == Some(1)), "{pods:?}");

    let replayed = World::replay(s, c, &w.state().log, w.tick()).unwrap();
    assert_eq!(replayed.state().hashes, w.state().hashes);

    let mothers: Vec<Genotype> = w
        .state()
        .plants
        .iter()
        .map(|e| e.genotype.clone())
        .collect();
    let f1 = harvest_all(&mut w, 0);
    let selfed = harvest_all(&mut w, 1);
    assert!(!f1.is_empty() && !selfed.is_empty());
    let crossed_seeds: Vec<&SeedGenotype> = f1.iter().filter(|g| g.father == 1).collect();
    assert!(!crossed_seeds.is_empty(), "{f1:?}");
    let (mut hits, mut loci) = (0, 0);
    for g in &crossed_seeds {
        assert_eq!(g.mother, 0);
        // Two pure lines: the F1 pairs the mother's allele with the father's at every
        // locus but a rare mutation (1 in 1,000 per QTL per gamete).
        let (h, n) = parental_loci(&g.genotype, &mothers[0], &mothers[1]);
        hits += h;
        loci += n;
    }
    assert!(hits * 100 >= loci * 97, "{hits} of {loci} loci");
    for g in &selfed {
        assert_eq!((g.mother, g.father), (1, 1));
        let (h, n) = parental_loci(&g.genotype, &mothers[1], &mothers[1]);
        assert!(h + 2 >= n, "{h} of {n}");
    }
}

/// Recombination at seed set: a selfed heterozygote's seeds segregate seed by seed,
/// siblings in one pod included, because each seed's meiosis is keyed by its own organ
/// (decision 0031 item 7).
#[test]
fn a_selfed_heterozygote_segregates_seed_by_seed() {
    let (s, c) = two_pots(75);
    let w = World::new(s.clone(), c).unwrap();
    // Give plant 0 the F1 genotype of the two lines (its phenotype stays its own line,
    // which this test does not read).
    let mut st = w.state().clone();
    let lines: Vec<Genotype> = st.plants.iter().map(|e| e.genotype.clone()).collect();
    let het: Vec<(f64, f64)> = lines[0]
        .alleles
        .iter()
        .zip(&lines[1].alleles)
        .map(|(a, b)| (a.0, b.0))
        .collect();
    st.plants[0].genotype = Genotype { alleles: het };
    let mother = st.plants[0].genotype.clone();
    let mut w = World::from_state(st).unwrap();
    w.run_until(75 * 24).unwrap();

    let seeds = harvest_all(&mut w, 0);
    assert!(seeds.len() >= 4, "{} seeds", seeds.len());
    assert!(seeds.iter().all(|g| (g.mother, g.father) == (0, 0)));
    // Siblings differ.
    let distinct: BTreeSet<String> = seeds.iter().map(|g| format!("{:?}", g.genotype)).collect();
    assert!(
        distinct.len() * 4 >= seeds.len() * 3,
        "{} of {}",
        distinct.len(),
        seeds.len()
    );
    // Each heterozygous locus throws homozygotes of both kinds and heterozygotes
    // across the seeds; count the loci where all three appear.
    let differs = |x: f64, y: f64| x.to_bits() != y.to_bits();
    let het_loci: Vec<usize> = (0..mother.alleles.len())
        .filter(|&i| differs(mother.alleles[i].0, mother.alleles[i].1))
        .collect();
    assert!(het_loci.len() >= 10, "{} heterozygous loci", het_loci.len());
    let segregating = het_loci
        .iter()
        .filter(|&&i| {
            let (m, f) = mother.alleles[i];
            let class = |g: &SeedGenotype| {
                let (a, b) = g.genotype.alleles[i];
                let is = |x: f64, y: f64| !differs(x, y);
                (is(a, m) && is(b, m), is(a, f) && is(b, f))
            };
            let mm = seeds.iter().any(|g| class(g) == (true, false));
            let ff = seeds.iter().any(|g| class(g) == (false, true));
            let mf = seeds.iter().any(|g| class(g) == (false, false));
            mm && ff && mf
        })
        .count();
    assert!(
        segregating * 2 >= het_loci.len(),
        "{segregating} of {} loci segregate",
        het_loci.len()
    );
}

/// The action is checked: only an open flower, once; the pollen plant needs an open
/// flower; a bush bean crosses with a pole bean.
#[test]
fn pollination_is_checked() {
    let (mut s, _) = two_pots(56);
    s.sowings[1].species = "pole_bean".into();
    let c = Catalog::load(&data(), &s).unwrap();
    let mut w = World::new(s, c).unwrap();
    let pollinate = |w: &mut World, organ: u32, pollen: u32| {
        w.apply(ActionKind::Pollinate {
            plant: 0,
            organ,
            pollen,
        })
    };
    // Bush beans flower first: the pole bean has no pollen yet.
    while open_flowers(&w, 0).is_empty() {
        w.run_days(1).unwrap();
    }
    assert!(!w.state().plants[1].plant.has_open_flower());
    let f = open_flowers(&w, 0)[0];
    let e = pollinate(&mut w, f, 1).unwrap_err().to_string();
    assert!(e.contains("no open flower"), "{e}");
    // Not a flower; not a plant.
    let leaf = w.state().plants[0]
        .plant
        .arena
        .alive_of(OrganKind::Leaf)
        .next()
        .unwrap();
    let e = pollinate(&mut w, leaf.0, 0).unwrap_err().to_string();
    assert!(e.contains("not an open flower"), "{e}");
    assert!(pollinate(&mut w, f, 7).is_err());
    // Once both flower, the cross takes, and only once.
    while !w.state().plants[1].plant.has_open_flower() {
        w.run_days(1).unwrap();
    }
    let f = open_flowers(&w, 0)[0];
    pollinate(&mut w, f, 1).unwrap();
    w.run_until(w.tick() + 1).unwrap();
    let flower = w.state().plants[0]
        .plant
        .flowers
        .iter()
        .find(|x| x.id.0 == f)
        .unwrap();
    assert_eq!(flower.pollen, Some(1));
    let e = pollinate(&mut w, f, 1).unwrap_err().to_string();
    assert!(e.contains("already hand-pollinated"), "{e}");
}
