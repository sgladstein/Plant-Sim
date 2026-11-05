use super::*;

const BEAN: &str = include_str!("../../../data/species/bush_bean.ron");

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
    assert_eq!(s.roots.len(), 5);
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
    let bad = BEAN.replacen("schema: 10,", "schema: 6,", 1);
    assert!(matches!(
        Species::from_ron(&bad),
        Err(SpeciesError::Schema {
            found: 6,
            expected: 10
        })
    ));
}

/// M0 identity mapping: every seed's phenotype equals the species defaults.
#[test]
fn identity_mapping() {
    let s = bean();
    for entity in [0u64, 1, 77, 1 << 40] {
        let g = s.sample_genotype(42, entity);
        assert!(g.alleles.iter().all(|&(a, b)| a == 0.0 && b == 0.0));
        let p = s.phenotype(&g);
        for (k, h) in &s.heritable {
            assert_eq!(p.get(k), Some(h.default), "{k}");
        }
    }
    assert_eq!(
        s.phenotype(&s.default_genotype()),
        s.phenotype(&s.sample_genotype(1, 2))
    );
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
