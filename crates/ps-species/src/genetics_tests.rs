//! Tests of meiosis, mutation, crosses and the seed-colour table (WP28, decision 0023).

// Allele values here are exact small integers, so exact comparison is the point.
#![allow(clippy::float_cmp)]

use super::genetics::{GameteSide, MeiosisKey, allele_state, haldane_r};
use super::seed_colour::{SeedColourTable, SeedLook};
use super::*;

const BEAN: &str = include_str!("../../../data/species/bush_bean.ron");

fn bean() -> Species {
    Species::from_ron(BEAN).expect("bush bean loads")
}

fn qtl(trait_name: &str, effect: f64, at: Option<(u8, f64)>) -> Locus {
    Locus {
        trait_name: trait_name.into(),
        effect,
        dominance: 0.0,
        allele_sd: 1.0,
        name: None,
        tier: None,
        chromosome: at.map(|a| a.0),
        position_cm: at.map(|a| a.1),
        line: None,
        frequencies: None,
        source: Some("test".into()),
    }
}

fn named(name: &str, states: u8, at: Option<(u8, f64)>) -> Locus {
    Locus {
        name: Some(name.into()),
        tier: Some(LocusTier::Named { states }),
        allele_sd: 0.0,
        ..qtl("habit", 0.0, at)
    }
}

/// The bush bean with its loci replaced and mutation off.
fn with_loci(loci: Vec<Locus>) -> Species {
    let mut s = bean();
    s.genome.loci = loci;
    s.genome.chromosome_lengths_cm = None;
    s.genome.mutation_rate.value = 0.0;
    s.genome.named_mutation_rate = Some(Param {
        value: 0.0,
        unit: schema::MUTATION_RATE_UNIT.into(),
        source: "test".into(),
    });
    s.validate().expect("test species is valid");
    s
}

fn wheat() -> Species {
    Species::from_ron(include_str!("../../../data/species/spring_wheat.ron")).expect("wheat loads")
}

fn key(entity: u64, side: GameteSide) -> MeiosisKey {
    MeiosisKey {
        world_seed: 7,
        entity,
        tick: 1000,
        side,
    }
}

/// Share of `n` gametes of a double heterozygote in coupling (1, 1 / 0, 0) at loci 0
/// and 1 that are recombinant.
fn recombinant_share(s: &Species, n: u64) -> f64 {
    let parent = Genotype {
        alleles: vec![(1.0, 0.0), (1.0, 0.0)],
    };
    let rec = (0..n)
        .filter(|&e| {
            let g = s.meiosis(&parent, key(e, GameteSide::Egg));
            g.alleles[0] != g.alleles[1]
        })
        .count();
    #[allow(clippy::cast_precision_loss)]
    let share = rec as f64 / n as f64;
    share
}

fn variance(xs: &[f64]) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let n = xs.len() as f64;
    let m = xs.iter().sum::<f64>() / n;
    xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0)
}

#[test]
fn haldane_map_function() {
    assert_eq!(haldane_r(0.0), 0.0);
    assert!((haldane_r(20.0) - 0.5 * (1.0 - libm::exp(-0.4))).abs() < 1e-15);
    assert!(haldane_r(500.0) > 0.4999);
    assert!(haldane_r(10.0) < 0.1 && haldane_r(10.0) > 0.09);
}

/// Linked loci recombine at Haldane's rate; unlinked and distant loci at one half.
#[test]
fn recombination_follows_the_map() {
    let n = 20_000;
    // 3 SD of a binomial share at n = 20,000 is at most 0.0107.
    let tol = 0.011;
    let cases = [
        (Some((1, 10.0)), Some((1, 30.0)), haldane_r(20.0)),
        (Some((2, 5.0)), Some((2, 15.0)), haldane_r(10.0)),
        (Some((3, 40.0)), Some((3, 40.0)), 0.0),
        (Some((1, 0.0)), Some((1, 100.0)), haldane_r(100.0)),
        (Some((1, 10.0)), Some((4, 10.0)), 0.5),
        (None, Some((1, 10.0)), 0.5),
        (None, None, 0.5),
    ];
    for (a, b, expected) in cases {
        let s = with_loci(vec![qtl("habit", 0.0, a), qtl("habit", 0.0, b)]);
        let got = recombinant_share(&s, n);
        assert!(
            (got - expected).abs() < tol,
            "{a:?} {b:?}: {got} vs {expected}"
        );
    }
}

#[test]
fn meiosis_is_keyed() {
    let loci = (0..12u8)
        .map(|i| {
            qtl(
                "habit",
                0.0,
                (i % 3 != 0).then_some((1 + i % 11, 7.0 * f64::from(i))),
            )
        })
        .collect();
    let s = with_loci(loci);
    let parent = Genotype {
        alleles: vec![(1.0, 0.0); 12],
    };
    let a = s.meiosis(&parent, key(5, GameteSide::Egg));
    assert_eq!(a, s.meiosis(&parent, key(5, GameteSide::Egg)));
    // Another seed, the other side of the same seed, another tick or world: different
    // draws (12 loci, so a chance match is 2^-12 or rarer).
    assert_ne!(a, s.meiosis(&parent, key(6, GameteSide::Egg)));
    assert_ne!(a, s.meiosis(&parent, key(5, GameteSide::Pollen)));
    assert_ne!(
        a,
        s.meiosis(
            &parent,
            MeiosisKey {
                tick: 1001,
                ..key(5, GameteSide::Egg)
            }
        )
    );
    // A homozygous parent passes on exactly its alleles.
    let line = Genotype::homozygous(&[3.0; 12]);
    assert!(
        s.meiosis(&line, key(9, GameteSide::Pollen))
            .alleles
            .iter()
            .all(|&v| v == 3.0)
    );
}

/// Adding a locus at the end leaves the draws of the others alone.
#[test]
fn adding_a_locus_keeps_other_draws() {
    let base = vec![
        qtl("habit", 0.0, None),
        qtl("habit", 0.0, Some((1, 10.0))),
        qtl("habit", 0.0, Some((2, 50.0))),
    ];
    let mut more = base.clone();
    more.push(qtl("habit", 0.0, Some((5, 20.0))));
    more.push(qtl("habit", 0.0, None));
    let (s, t) = (with_loci(base), with_loci(more));
    let p3 = Genotype {
        alleles: vec![(1.0, 0.0); 3],
    };
    let p5 = Genotype {
        alleles: vec![(1.0, 0.0); 5],
    };
    for e in 0..200 {
        let a = s.meiosis(&p3, key(e, GameteSide::Egg));
        let b = t.meiosis(&p5, key(e, GameteSide::Egg));
        assert_eq!(a.alleles[..], b.alleles[..3]);
    }
}

/// A species with zero allele variance (the M0 wheat) breeds true; a bean pure line
/// breeds true but for rare QTL mutations.
#[test]
fn pure_lines_breed_true() {
    let s = wheat();
    let g = s.default_genotype();
    for e in 0..100 {
        assert_eq!(s.seed_genotype(&g, &g, 3, e, 500), g);
    }
    let b = bean();
    let g = b.default_genotype();
    let changed: usize = (0..1000)
        .map(|e| {
            let child = b.seed_genotype(&g, &g, 3, e, 500);
            child
                .alleles
                .iter()
                .zip(&g.alleles)
                .filter(|(x, y)| x != y)
                .count()
        })
        .sum();
    // 1,000 seeds × 2 gametes × 23 QTL at 1e-3: about 46 changed alleles.
    assert!((20..=80).contains(&changed), "{changed}");
}

/// Bush × pole: Fin is dominant, so the F1 climbs and the F2 splits 3 : 1.
#[test]
fn habit_segregates_three_to_one() {
    let bush = bean();
    let pole = Species::from_ron(include_str!("../../../data/species/pole_bean.ron")).unwrap();
    let habit = |g: &Genotype| bush.phenotype(g).get("habit").unwrap();
    let f1 = bush.seed_genotype(&bush.default_genotype(), &pole.default_genotype(), 5, 1, 10);
    assert_eq!(habit(&f1), 1.0);
    let n = 4000u64;
    let climbing = (0..n)
        .filter(|&e| habit(&bush.seed_genotype(&f1, &f1, 5, 100 + e, 2000)) > 0.5)
        .count();
    #[allow(clippy::cast_precision_loss)]
    let share = climbing as f64 / n as f64;
    // 3 SD of a binomial share at n = 4,000 is 0.021.
    assert!((share - 0.75).abs() < 0.021, "{share}");
}

/// WP28 done-when 2: F2 variance exceeds F1 variance (with linkage).
#[test]
fn f2_variance_exceeds_f1() {
    let mut loci = Vec::new();
    for c in 1..=5u8 {
        loci.push(qtl("leaf_area_potential", 2.0, Some((c, 10.0))));
        loci.push(qtl("leaf_area_potential", 2.0, Some((c, 60.0))));
    }
    loci.push(qtl("leaf_area_potential", 2.0, None));
    loci.push(qtl("leaf_area_potential", 2.0, None));
    let mut s = with_loci(loci);
    // Real mutation, so F1 is not trivially uniform.
    s.genome.mutation_rate.value = 1e-3;
    let p1 = Genotype::homozygous(&[1.0; 12]);
    let p2 = Genotype::homozygous(&[-1.0; 12]);
    let trait_of = |g: &Genotype| s.phenotype(g).get("leaf_area_potential").unwrap();

    let f1: Vec<Genotype> = (0..300)
        .map(|e| s.seed_genotype(&p1, &p2, 11, e, 100))
        .collect();
    let f2: Vec<Genotype> = f1
        .iter()
        .enumerate()
        .map(|(e, g)| s.seed_genotype(g, g, 11, 10_000 + e as u64, 3000))
        .collect();
    let v1 = variance(&f1.iter().map(trait_of).collect::<Vec<_>>());
    let v2 = variance(&f2.iter().map(trait_of).collect::<Vec<_>>());
    // Unlinked expectation: 12 loci × effect² × 1/2 = 24 cm4; coupling linkage raises it.
    assert!(v2 > 15.0, "F2 variance {v2}");
    assert!(v2 > 20.0 * v1.max(1e-9), "F1 {v1}, F2 {v2}");
    // The F2 mean is the midparent (no dominance).
    let m2 = f2.iter().map(trait_of).sum::<f64>() / 300.0;
    assert!((m2 - 133.0).abs() < 1.5, "F2 mean {m2}");
}

/// Two-tier mutation: QTL at their rate with small steps, named loci to another state.
#[test]
fn two_tier_mutation() {
    let mut loci: Vec<Locus> = (0..10).map(|_| qtl("habit", 0.0, None)).collect();
    loci.push(named("Z", 3, Some((7, 20.0))));
    let mut s = with_loci(loci);
    s.genome.mutation_rate.value = 0.01;
    s.genome.named_mutation_rate = Some(Param {
        value: 0.05,
        unit: schema::MUTATION_RATE_UNIT.into(),
        source: "test".into(),
    });
    let parent = Genotype::homozygous(&[0.0; 11]);
    let n = 4000u64;
    let mut steps = Vec::new();
    let mut named_changes = 0u32;
    for e in 0..n {
        let g = s.meiosis(&parent, key(e, GameteSide::Pollen));
        steps.extend(g.alleles[..10].iter().copied().filter(|&v| v != 0.0));
        let z = g.alleles[10];
        if z != 0.0 {
            assert!(z == 1.0 || z == 2.0, "{z}");
            named_changes += 1;
        }
    }
    // 40,000 QTL draws at 0.01: 400 ± 60 (3 SD).
    assert!((340..=460).contains(&steps.len()), "{}", steps.len());
    let sd = variance(&steps).sqrt();
    assert!((sd - 0.15).abs() < 0.03, "step SD {sd}");
    // 4,000 named draws at 0.05: 200 ± 41.
    assert!((159..=241).contains(&named_changes), "{named_changes}");
}

#[test]
fn allele_states_round_and_clamp() {
    assert_eq!(allele_state(0.4, 2), 0);
    assert_eq!(allele_state(0.6, 2), 1);
    assert_eq!(allele_state(7.0, 3), 2);
    assert_eq!(allele_state(-1.0, 3), 0);
}

#[test]
fn linkage_map_is_validated() {
    let check = |l: Locus| {
        let mut s = with_loci(vec![]);
        s.genome.loci.push(l);
        s.validate()
    };
    assert!(check(qtl("habit", 1.0, Some((11, 100.0)))).is_ok());
    assert!(check(qtl("habit", 1.0, Some((12, 10.0)))).is_err());
    assert!(check(qtl("habit", 1.0, Some((0, 10.0)))).is_err());
    assert!(check(qtl("habit", 1.0, Some((1, 100.5)))).is_err());
    assert!(check(qtl("habit", 1.0, Some((1, f64::NAN)))).is_err());
    let mut half = qtl("habit", 1.0, None);
    half.chromosome = Some(2);
    assert!(check(half).is_err());
    assert!(check(named("fin", 1, None)).is_err());
    let mut nameless = named("fin", 2, None);
    nameless.name = None;
    assert!(check(nameless).is_err());

    let mut s = bean();
    s.genome.loci.push(named("fin", 2, Some((1, 30.0))));
    s.genome.loci.push(named("fin", 2, Some((1, 40.0))));
    assert!(s.validate().is_err());

    let lengths = |values: Vec<f64>, unit: &str| ChromosomeLengths {
        values,
        unit: unit.into(),
        source: "test".into(),
    };
    let mut s = bean();
    s.genome.chromosome_lengths_cm = Some(lengths(vec![100.0; 11], "M"));
    assert!(matches!(s.validate(), Err(SpeciesError::Unit { .. })));
    s.genome.chromosome_lengths_cm = Some(lengths(vec![100.0; 10], "cM"));
    assert!(s.validate().is_err());
    let mut v = vec![100.0; 11];
    v[0] = 147.0;
    v[8] = 46.5;
    s.genome.chromosome_lengths_cm = Some(lengths(v, "cM"));
    assert!(s.validate().is_ok());
    // Positions must lie within their own chromosome.
    s.genome.loci.push(qtl("habit", 1.0, Some((1, 140.0))));
    assert!(s.validate().is_ok());
    s.genome.loci.push(qtl("habit", 1.0, Some((9, 50.0))));
    assert!(s.validate().is_err());
}

/// Species files without the decision-0023 keys still load and serialize as before.
#[test]
fn optional_genome_keys_round_trip() {
    let s = wheat();
    let text = ron::ser::to_string(&s.genome).unwrap();
    assert!(
        !text.contains("chromosome") && !text.contains("tier"),
        "{text}"
    );
    let mut t = s.clone();
    t.genome.loci.push(named("P", 2, Some((7, 25.0))));
    t.genome.chromosome_lengths_cm = Some(ChromosomeLengths {
        values: vec![110.0; 11],
        unit: schema::CHROMOSOME_LENGTH_UNIT.into(),
        source: "~".into(),
    });
    let text = ron::ser::to_string_pretty(&t, ron::ser::PrettyConfig::default()).unwrap();
    assert_eq!(Species::from_ron(&text).unwrap(), t);
    assert!(
        t.provenance()
            .iter()
            .any(|(path, ..)| path == "genome.chromosome_lengths_cm[11]")
    );
}

/// An illustrative two-layer table (not the bean table, which waits on the research).
const TABLE: &str = r##"(
  source: "test fixture",
  default: (seed_coat: "#f2eee0", ground: "#f2eee0", pattern: "solid", coverage: 1.0, glossy: true, flower: "#ffffff"),
  layers: [
    (name: "P gate", rules: [
      (when: [Homozygous(locus: "P", state: 0)],
       set: (seed_coat: Some("#f6f3ea"), flower: Some("#ffffff")), stop: true, source: "test"),
    ]),
    (name: "base colour", rules: [
      (when: [Carries(locus: "V", state: 1), Carries(locus: "T", state: 1)],
       set: (seed_coat: Some("#1b1b22"), flower: Some("#8a4fb5")), source: "test"),
      (when: [Carries(locus: "V", state: 1)],
       set: (seed_coat: Some("#3a2a40"), pattern: Some("partly coloured"), flower: Some("#8a4fb5")), source: "test"),
      (when: [], set: (seed_coat: Some("#9b6a3c")), source: "test"),
    ]),
  ],
)"##;

fn colour_species() -> Species {
    with_loci(vec![
        named("P", 2, Some((7, 30.0))),
        named("T", 2, Some((9, 10.0))),
        named("V", 2, Some((6, 50.0))),
    ])
}

#[test]
fn seed_colour_table_evaluates_in_layers() {
    let s = colour_species();
    let t = SeedColourTable::from_ron(TABLE, &s).expect("table loads");
    let look = |p: (f64, f64), tt: (f64, f64), v: (f64, f64)| {
        t.evaluate(
            &s,
            &Genotype {
                alleles: vec![p, tt, v],
            },
        )
    };
    let white = SeedLook {
        seed_coat: "#f6f3ea".into(),
        ground: "#f2eee0".into(),
        pattern: "solid".into(),
        coverage: 1.0,
        glossy: true,
        flower: "#ffffff".into(),
    };
    // pp is white whatever lies below.
    assert_eq!(look((0.0, 0.0), (1.0, 1.0), (1.0, 1.0)), white);
    // Pp V_ T_: black seed, violet flower.
    let black = look((1.0, 0.0), (0.0, 1.0), (1.0, 0.0));
    assert_eq!(black.seed_coat, "#1b1b22");
    assert_eq!(black.flower, "#8a4fb5");
    // P_ V_ tt: the second rule.
    assert_eq!(
        look((1.0, 1.0), (0.0, 0.0), (0.0, 1.0)).pattern,
        "partly coloured"
    );
    // P_ vv: the catch-all rule; the flower keeps the default.
    let brown = look((1.0, 1.0), (1.0, 1.0), (0.0, 0.0));
    assert_eq!(brown.seed_coat, "#9b6a3c");
    assert_eq!(brown.flower, "#ffffff");
}

#[test]
fn seed_colour_table_is_validated() {
    let s = colour_species();
    let unknown = TABLE.replacen("locus: \"V\"", "locus: \"Q\"", 1);
    assert!(SeedColourTable::from_ron(&unknown, &s).is_err());
    let state = TABLE.replacen("locus: \"T\", state: 1", "locus: \"T\", state: 2", 1);
    assert!(SeedColourTable::from_ron(&state, &s).is_err());
    let colour = TABLE.replacen("#1b1b22", "black", 1);
    assert!(SeedColourTable::from_ron(&colour, &s).is_err());
    let source = TABLE.replacen("source: \"test fixture\"", "source: \"\"", 1);
    assert!(SeedColourTable::from_ron(&source, &s).is_err());
}

/// `Expressed` reads an allelic series by its most dominant allele (V > v^lae > v), and
/// a patch can set pattern, coverage and shine.
#[test]
fn expressed_reads_an_allelic_series() {
    let s = with_loci(vec![named("V", 3, Some((6, 10.0)))]);
    let table = r##"(
      source: "test",
      default: (seed_coat: "#000000", ground: "#ffffff", pattern: "self", coverage: 1.0, glossy: true, flower: "#ffffff"),
      layers: [(name: "flower", rules: [
        (when: [Expressed(locus: "V", state: 2)], set: (flower: Some("#8a4fb5")), source: "test"),
        (when: [Expressed(locus: "V", state: 1)], set: (flower: Some("#d99bb8"), pattern: Some("virgarcus"), coverage: Some(0.35), glossy: Some(false)), source: "test"),
      ])],
    )"##;
    let t = SeedColourTable::from_ron(table, &s).unwrap();
    let flower = |a: f64, b: f64| {
        t.evaluate(
            &s,
            &Genotype {
                alleles: vec![(a, b)],
            },
        )
    };
    assert_eq!(flower(2.0, 0.0).flower, "#8a4fb5");
    assert_eq!(flower(1.0, 2.0).flower, "#8a4fb5");
    let pink = flower(1.0, 0.0);
    assert_eq!(pink.flower, "#d99bb8");
    assert!((pink.coverage - 0.35).abs() < 1e-15 && !pink.glossy);
    assert_eq!(flower(0.0, 0.0).flower, "#ffffff");
    let bad = table.replacen("coverage: Some(0.35)", "coverage: Some(1.5)", 1);
    assert!(SeedColourTable::from_ron(&bad, &s).is_err());
}

/// The bean seed-colour table loads against the bean files and reproduces the
/// genotype → colour pairs the research read in primary papers.
#[test]
fn bean_seed_colour_table() {
    let s = bean();
    let t = SeedColourTable::from_ron(
        include_str!("../../../data/genetics/bean_seed_colour.ron"),
        &s,
    )
    .expect("bean table loads");
    let pole = Species::from_ron(include_str!("../../../data/species/pole_bean.ron")).unwrap();
    t.validate(&pole).expect("the table fits the pole bean too");
    let index = |sym: &str| {
        s.genome
            .loci
            .iter()
            .position(|l| l.name.as_deref() == Some(sym))
            .unwrap()
    };
    // A genotype: the bean line with the named loci given as homozygous states.
    let look = |states: &[(&str, u8)]| {
        let mut g = s.default_genotype();
        for &(sym, v) in states {
            g.alleles[index(sym)] = (f64::from(v), f64::from(v));
        }
        t.evaluate(&s, &g)
    };
    let coloured = [("P", 2), ("T", 1), ("C", 2), ("J", 1), ("Asp", 1)];
    let with = |extra: &[(&'static str, u8)]| {
        let mut v: Vec<(&str, u8)> = coloured.to_vec();
        v.extend_from_slice(extra);
        look(&v)
    };
    // The archetype line is pp: white seed, white flower (its appearance colour).
    let line = t.evaluate(&s, &s.default_genotype());
    assert_eq!(
        (line.seed_coat.as_str(), line.flower.as_str()),
        ("#F4F1E8", "#F4F1E8")
    );
    // H3: P C J G B V black, purple flower; asp asp matte.
    let black = with(&[("G", 1), ("B", 1), ("V", 2)]);
    assert_eq!(black.seed_coat, "#1A1416");
    assert_eq!(black.flower, "#8A4FB5");
    assert!(black.glossy && (black.coverage - 1.0).abs() < 1e-15);
    assert!(!with(&[("G", 1), ("B", 1), ("V", 2), ("Asp", 0)]).glossy);
    // H5, H6, H7.
    assert_eq!(with(&[("G", 1), ("B", 0), ("V", 2)]).seed_coat, "#3F2630");
    assert_eq!(with(&[("G", 1), ("B", 1), ("V", 0)]).seed_coat, "#4A3020");
    assert_eq!(with(&[("G", 1), ("B", 0), ("V", 0)]).seed_coat, "#A5773A");
    // H10 dark red kidney: J g B v rk.
    assert_eq!(
        with(&[("G", 0), ("B", 1), ("V", 0), ("Rk", 0)]).seed_coat,
        "#6B1A1F"
    );
    // H13 Prim: j G b v^lae, light yellow with a pink flower.
    let prim = with(&[("J", 0), ("G", 1), ("B", 0), ("V", 1)]);
    assert_eq!(
        (prim.seed_coat.as_str(), prim.flower.as_str()),
        ("#E8D58A", "#D99BB8")
    );
    // Grey-white p^gri stops the pipeline.
    assert_eq!(look(&[("P", 1)]).seed_coat, "#DAD6CC");
    // Partly coloured (tt): white flower even with V; the pattern follows Z and Bip.
    let tt = |z: u8, bip: u8| with(&[("T", 0), ("V", 2), ("Z", z), ("Bip", bip)]);
    let full = tt(2, 2);
    assert_eq!(
        (full.pattern.as_str(), full.flower.as_str()),
        ("self", "#F4F1E8")
    );
    assert_eq!(tt(0, 2).pattern, "virgarcus");
    assert_eq!(tt(0, 0).pattern, "bipunctata");
    assert_eq!(tt(2, 1).pattern, "anasazi");
    assert_eq!(tt(1, 2).pattern, "sellatus");
    assert!((tt(0, 0).coverage - 0.1).abs() < 1e-15);
}

#[test]
fn lines_and_frequencies_are_validated() {
    let check = |l: Locus| {
        let mut s = with_loci(vec![]);
        s.genome.loci.push(l);
        s.validate()
    };
    let mut l = named("P", 3, None);
    l.frequencies = Some(vec![0.2, 0.3, 0.5]);
    l.line = Some(2.0);
    assert!(check(l.clone()).is_ok());
    let mut bad_line = l.clone();
    bad_line.line = Some(3.0);
    assert!(check(bad_line).is_err());
    let mut half_state = l.clone();
    half_state.line = Some(0.5);
    assert!(check(half_state).is_err());
    let mut bad_sum = l.clone();
    bad_sum.frequencies = Some(vec![0.2, 0.3, 0.4]);
    assert!(check(bad_sum).is_err());
    let mut wrong_len = l;
    wrong_len.frequencies = Some(vec![0.5, 0.5]);
    assert!(check(wrong_len).is_err());
    let mut qtl_freq = qtl("habit", 1.0, None);
    qtl_freq.frequencies = Some(vec![1.0]);
    assert!(check(qtl_freq).is_err());
    let mut no_source = qtl("habit", 1.0, None);
    no_source.source = None;
    assert!(check(no_source).is_err());
    // A visual-only named locus needs no trait; a QTL does.
    assert!(check(named("V", 3, None)).is_ok());
    let mut no_trait = qtl("habit", 1.0, None);
    no_trait.trait_name = String::new();
    assert!(check(no_trait).is_err());
}

/// WP29: only genomes of one layout cross; a cross draws each gamete from its own
/// parent's species, and a self through `cross_genotype` is `seed_genotype`.
#[test]
fn crosses_need_one_genome_layout() {
    let bush = bean();
    let pole = Species::from_ron(include_str!("../../../data/species/pole_bean.ron")).unwrap();
    assert!(bush.can_cross(&pole) && pole.can_cross(&bush));
    assert!(bush.can_cross(&bush));
    assert!(!bush.can_cross(&wheat()) && !wheat().can_cross(&bush));
    // Moving one locus breaks the layout.
    let mut moved = bean();
    moved.genome.loci[0].position_cm = moved.genome.loci[0].position_cm.map(|p| p + 1.0);
    assert!(!bush.can_cross(&moved));

    let (m, f) = (bush.sample_genotype(7, 1), pole.sample_genotype(7, 2));
    let selfed = bush.cross_genotype(&m, &bush, &m, 7, 99, 1000);
    assert_eq!(selfed, bush.seed_genotype(&m, &m, 7, 99, 1000));
    // Pure lines: the F1 takes the egg's allele from the mother and the pollen's from
    // the father (bar mutation), and the pole father brings Fin.
    let f1 = bush.cross_genotype(&m, &pole, &f, 7, 99, 1000);
    let fin = bush
        .genome
        .loci
        .iter()
        .position(|l| l.name.as_deref() == Some("fin"))
        .unwrap();
    assert_eq!(f1.alleles[fin], (m.alleles[fin].0, f.alleles[fin].0));
    assert_eq!((m.alleles[fin].0, f.alleles[fin].0), (0.0, 1.0));
    let t = bush.phenotype(&f1).traits().unwrap();
    assert!(
        (t.habit - 1.0).abs() < 1e-12,
        "Fin is dominant: {}",
        t.habit
    );
}
