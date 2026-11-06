use super::*;

const BEAN: &str = include_str!("../../../data/species/bush_bean.ron");
const POLE: &str = include_str!("../../../data/species/pole_bean.ron");

fn bean() -> Species {
    Species::from_ron(BEAN).expect("bush bean loads")
}

/// WP4 done-when 1: the bean file loads.
#[test]
fn bean_file_loads() {
    let s = bean();
    assert_eq!(s.id, "bush_bean");
    assert_eq!(s.growth_form, GrowthForm::Herb);
    assert_eq!(s.heritable.len(), schema::HERITABLE.len());
    // Tap, two basal whorls, hypocotyl-borne, lateral and fine lateral (R2a).
    assert_eq!(s.roots.len(), 6);
    assert!((s.constants.carbon_fraction.value - 0.45).abs() < 1e-15);
}

/// WP4 done-when 2: a wrong unit fails with a clear error.
#[test]
fn wrong_unit_is_a_clear_error() {
    let bad = BEAN.replacen(
        "carbon_fraction: (value: 0.45, unit: \"g C / g DM\"",
        "carbon_fraction: (value: 0.45, unit: \"g DM / g C\"",
        1,
    );
    let e = Species::from_ron(&bad).unwrap_err();
    assert_eq!(
        e.to_string(),
        "constants.carbon_fraction: unit \"g DM / g C\" is wrong, expected \"g C / g DM\""
    );
}

#[test]
fn wrong_heritable_unit_is_named() {
    let bad = BEAN.replacen(
        "\"vcmax25\": (default: 89.3, min: 40.0, max: 140.0, unit: \"umol/m2/s\"",
        "\"vcmax25\": (default: 89.3, min: 40.0, max: 140.0, unit: \"mol/m2/s\"",
        1,
    );
    let e = Species::from_ron(&bad).unwrap_err();
    assert!(
        e.to_string()
            .starts_with("heritable.vcmax25: unit \"mol/m2/s\""),
        "{e}"
    );
}

/// WP4 done-when 2: a missing source fails with a clear error.
#[test]
fn missing_source_is_a_clear_error() {
    let bad = BEAN.replacen("source: \"typical (design plan §6.1)\"", "source: \"\"", 1);
    let e = Species::from_ron(&bad).unwrap_err();
    assert_eq!(
        e.to_string(),
        "constants.carbon_fraction: missing source (use \"~\" for a guess to validate)"
    );
}

#[test]
fn absent_source_field_is_a_parse_error() {
    let bad = BEAN.replacen(", source: \"typical (design plan §6.1)\"", "", 1);
    let e = Species::from_ron(&bad).unwrap_err();
    assert!(
        matches!(e, SpeciesError::Parse(ref m) if m.contains("source")),
        "{e}"
    );
}

#[test]
fn unknown_constant_is_rejected() {
    let bad = BEAN.replacen("    maintenance_q10:", "    maintenance_q1O:", 1);
    let e = Species::from_ron(&bad).unwrap_err();
    assert!(
        matches!(e, SpeciesError::Parse(ref m) if m.contains("maintenance_q1O")),
        "{e}"
    );
}

#[test]
fn unknown_and_missing_heritable_keys() {
    let typo = BEAN.replacen("\"habit\":", "\"habbit\":", 1);
    let e = Species::from_ron(&typo).unwrap_err();
    assert_eq!(e.to_string(), "heritable.habbit: unknown parameter");
}

#[test]
fn default_outside_bounds_rejected() {
    let bad = BEAN.replacen("\"rd25\": (default: 0.88", "\"rd25\": (default: 5.0", 1);
    let e = Species::from_ron(&bad).unwrap_err();
    assert!(
        e.to_string()
            .starts_with("heritable.rd25: default 5 must lie within"),
        "{e}"
    );
}

#[test]
fn locus_must_name_a_heritable_trait() {
    let bad = BEAN.replacen("(trait: \"habit\"", "(trait: \"wings\"", 1);
    let e = Species::from_ron(&bad).unwrap_err();
    assert!(matches!(e, SpeciesError::LocusTrait { ref trait_name, .. } if trait_name == "wings"));
}

#[test]
fn schema_version_checked() {
    let bad = BEAN.replacen("schema: 13,", "schema: 6,", 1);
    assert!(matches!(
        Species::from_ron(&bad),
        Err(SpeciesError::Schema {
            found: 6,
            expected: 13
        })
    ));
}

/// A `Default` seed carries the file's pure line, whose phenotype is the species
/// defaults (the pole bean's line carries Fin Fin, so its habit is 1).
#[test]
fn default_seed_is_the_pure_line() {
    let s = bean();
    let p = s.phenotype(&s.default_genotype());
    for (k, h) in &s.heritable {
        assert_eq!(p.get(k), Some(h.default), "{k}");
    }
    let pole = Species::from_ron(POLE).unwrap();
    let t = pole.phenotype(&pole.default_genotype()).traits().unwrap();
    assert!((t.habit - 1.0).abs() < 1e-15);
    assert!((t.internode_length_potential - 12.0).abs() < 1e-15);
}

/// WP28 done-when 1: two sampled seeds of one species differ measurably.
#[test]
fn sampled_seeds_differ() {
    let s = bean();
    let a = s.phenotype(&s.sample_genotype(42, 1)).traits().unwrap();
    let b = s.phenotype(&s.sample_genotype(42, 2)).traits().unwrap();
    // Several traits differ by more than 1% of the default.
    let pairs = [
        (a.vcmax25, b.vcmax25, 89.3),
        (a.leaf_area_potential, b.leaf_area_potential, 133.0),
        (a.seed_mass, b.seed_mass, 0.3),
        (a.specific_leaf_area, b.specific_leaf_area, 305.0),
        (a.leaf_lifespan, b.leaf_lifespan, 600.0),
    ];
    let differ = pairs
        .iter()
        .filter(|(x, y, d)| (x - y).abs() > 0.01 * d)
        .count();
    assert!(differ >= 3, "{pairs:?}");
    // Habit is fixed in the bush bean population (fin fin).
    assert!(a.habit.abs() < 1e-15 && b.habit.abs() < 1e-15);
    assert_eq!(s.sample_genotype(42, 1), s.sample_genotype(42, 1));
}

#[test]
fn nonzero_variance_varies_and_stays_in_bounds() {
    let mut s = bean();
    for l in &mut s.genome.loci {
        l.allele_sd = 50.0;
    }
    let a = s.phenotype(&s.sample_genotype(42, 1));
    let b = s.phenotype(&s.sample_genotype(42, 2));
    assert_ne!(a, b);
    for (k, h) in &s.heritable {
        let v = a.get(k).unwrap();
        assert!(v >= h.min && v <= h.max, "{k} = {v}");
    }
    // Same plant, same seed: same genotype.
    assert_eq!(s.sample_genotype(42, 1), s.sample_genotype(42, 1));
}

#[test]
fn every_parameter_has_unit_and_source() {
    for (path, v, unit, source) in bean().provenance() {
        assert!(v.is_finite(), "{path}");
        assert!(!unit.is_empty() && !source.trim().is_empty(), "{path}");
    }
}

#[test]
fn round_trip_through_ron() {
    let s = bean();
    let text = ron::ser::to_string_pretty(&s, ron::ser::PrettyConfig::default()).unwrap();
    assert_eq!(Species::from_ron(&text).unwrap(), s);
}

#[test]
fn trait_view_covers_every_heritable_key() {
    let keys: Vec<&str> = crate::schema::HERITABLE.iter().map(|s| s.key).collect();
    assert_eq!(keys, crate::TRAIT_KEYS);
    let s = bean();
    let t = s.phenotype(&s.default_genotype()).traits().unwrap();
    assert!((t.vcmax25 - 89.3).abs() < 1e-12);
    assert!((t.flowers_per_raceme - 6.0).abs() < 1e-12);
}

#[test]
fn root_types_are_validated() {
    let two_radicles = BEAN.replacen(
        "RootType(name: \"basal_1\", origin: Basal,",
        "RootType(name: \"basal_1\", origin: Radicle,",
        1,
    );
    assert!(matches!(
        Species::from_ron(&two_radicles),
        Err(SpeciesError::Roots(_))
    ));
    let duplicate = BEAN.replacen("name: \"basal_2\"", "name: \"basal_1\"", 1);
    assert!(matches!(
        Species::from_ron(&duplicate),
        Err(SpeciesError::Roots(_))
    ));
    let lateral_at_zero = BEAN.replacen(
        "origin: Lateral, order: 1,",
        "origin: Lateral, order: 0,",
        1,
    );
    assert!(matches!(
        Species::from_ron(&lateral_at_zero),
        Err(SpeciesError::Roots(_))
    ));
}

/// WP27: the leaf energy keys must be physical.
#[test]
fn energy_keys_are_checked() {
    let cases = [
        (
            "leaf_width: (value: 0.08,",
            "leaf_width: (value: 0.0,",
            "leaf_width",
        ),
        (
            "leaf_nir_absorptance: (value: 0.20,",
            "leaf_nir_absorptance: (value: 1.2,",
            "leaf_nir_absorptance",
        ),
        (
            "leaf_emissivity: (value: 0.97,",
            "leaf_emissivity: (value: 0.0,",
            "leaf_emissivity",
        ),
        (
            "abaxial_conductance_share: (value: 0.875,",
            "abaxial_conductance_share: (value: -0.1,",
            "abaxial_conductance_share",
        ),
    ];
    for (from, to, key) in cases {
        assert!(BEAN.contains(from), "{from}");
        let e = Species::from_ron(&BEAN.replacen(from, to, 1)).unwrap_err();
        assert!(
            e.to_string().starts_with(&format!("constants.{key}:")),
            "{e}"
        );
    }
}

/// R1a: the thickening keys and tables must be well formed.
#[test]
fn thickening_keys_are_checked() {
    let cases = [
        (
            "secondary_growth: (value: 1.0,",
            "secondary_growth: (value: 0.5,",
            "thickening.secondary_growth:",
        ),
        (
            "stem_radial_rate_max: (value: 0.1,",
            "stem_radial_rate_max: (value: -0.1,",
            "thickening.stem_radial_rate_max:",
        ),
        (
            "(4.0, 0.0005), (5.0, 0.001)",
            "(5.0, 0.0005), (5.0, 0.001)",
            "root_secondary_rate:",
        ),
        (
            "points: [(0.0, 4.0), (20.0, 4.0), (40.0, 2.0)], x_unit: \"cm\"",
            "points: [(0.0, 4.0), (20.0, 4.0), (40.0, 2.0)], x_unit: \"m\"",
            "roots[tap].secondary_factor.x_unit:",
        ),
        (
            "points: [(0.0, 0.1)]",
            "points: []",
            "roots[lateral].secondary_factor:",
        ),
    ];
    for (from, to, at) in cases {
        assert!(BEAN.contains(from), "{from}");
        let e = Species::from_ron(&BEAN.replacen(from, to, 1)).unwrap_err();
        assert!(e.to_string().starts_with(at), "{e}");
    }
}

/// Tables interpolate linearly and hold level beyond their ends.
#[test]
fn tables_interpolate() {
    let s = Species::from_ron(BEAN).unwrap();
    let t = &s.root_secondary_rate;
    assert!(t.at(-1.0).abs() < 1e-15);
    assert!((t.at(4.5) - 0.00075).abs() < 1e-15);
    assert!((t.at(100.0) - 0.003).abs() < 1e-15);
    let tap = &s.roots[0].secondary_factor;
    assert!((tap.at(30.0) - 3.0).abs() < 1e-12);
    assert!((tap.at(80.0) - 2.0).abs() < 1e-15);
    // The rate is a diameter increment: 0.1 cm in 43 days (OpenSimRoot's note).
    let n = 43_000;
    let dx = 43.0 / f64::from(n);
    let total: f64 = (0..n).map(|k| t.at((f64::from(k) + 0.5) * dx) * dx).sum();
    assert!((total - 0.1).abs() < 1e-3, "{total}");
}
