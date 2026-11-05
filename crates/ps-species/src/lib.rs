//! Species and genome schema (design plan §3.12, §6.1, WP4).
//!
//! A species file is RON. Every numeric parameter carries a value, a unit and a source;
//! on load the unit must match the schema exactly and the source must be non-empty
//! ("~" is a valid source meaning a guess to validate). Heritable parameters also carry
//! bounds, and the genome lists the loci that map genotype to phenotype.
//!
//! M0 ships the degenerate genome: one locus per heritable parameter with zero allele
//! variance, so every seed's phenotype equals the species defaults (identity mapping).
//!
//! Model card: docs/models/species-and-genome.md.

pub mod schema;

use std::collections::BTreeMap;

use ps_rng::{Purpose, RngKey};
use serde::{Deserialize, Serialize};

pub use schema::{Climbing, Constants, Grass, Nitrogen, Phenology, SCHEMA, Spec, Variation};
pub use traits::{TRAIT_KEYS, Traits};

/// A numeric parameter with provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    /// The value, in `unit`.
    pub value: f64,
    /// Unit string; must match the schema exactly.
    pub unit: String,
    /// Where the value comes from; "~" means a guess to validate.
    pub source: String,
}

/// A heritable parameter: species default, bounds, unit and source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritableParam {
    /// Species default (the phenotype of a seed with all-zero alleles).
    pub default: f64,
    /// Lower bound of the phenotype.
    pub min: f64,
    /// Upper bound of the phenotype.
    pub max: f64,
    /// Unit string; must match the schema exactly.
    pub unit: String,
    /// Where the default comes from.
    pub source: String,
}

/// One locus: which trait it affects and how (§3.12).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Locus {
    /// The heritable parameter this locus contributes to.
    #[serde(rename = "trait")]
    pub trait_name: String,
    /// Additive effect per unit allele value, in the trait's unit.
    pub effect: f64,
    /// Dominance deviation per unit allele difference, in the trait's unit.
    pub dominance: f64,
    /// Standard deviation of allele values in the species population.
    pub allele_sd: f64,
}

/// The genome definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Genome {
    /// Loci, in a fixed order (the genotype is a list in this order).
    pub loci: Vec<Locus>,
    /// Mutation rate.
    pub mutation_rate: Param,
}

/// Growth form, which selects the architecture module (§3.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthForm {
    /// Herb (bush bean, M0).
    Herb,
    /// Vine (pole bean, M2).
    Vine,
    /// Grass (M2).
    Grass,
    /// Tree (M4).
    Tree,
}

/// Where the axes of a root type start (§3.7, decision 0013).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RootOrigin {
    /// The tap root, from the seed at germination.
    Radicle,
    /// A whorl of basal roots at the base of the hypocotyl.
    Basal,
    /// Roots along the buried part of the hypocotyl.
    Hypocotyl,
    /// Laterals on roots of the previous order.
    Lateral,
}

/// One root type (§3.7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootType {
    /// Name, unique within the species.
    pub name: String,
    /// Where its axes start.
    pub origin: RootOrigin,
    /// Branching order: 0 for axes from the seed, 1 for their laterals.
    pub order: u8,
    /// Number of axes (1 for laterals).
    pub count: Param,
    /// Thermal time from germination to the appearance of these axes.
    pub emergence: Param,
    /// Potential tip elongation at optimum temperature.
    pub elongation: Param,
    /// Maximum root length.
    pub max_length: Param,
    /// Distance between laterals on this root (0 for none).
    pub branch_spacing: Param,
    /// Unbranched zone behind the tip.
    pub apical_zone: Param,
    /// Angle from the vertical (axes) or from the parent root (laterals).
    pub insertion_angle: Param,
    /// Root diameter.
    pub diameter: Param,
    /// Gravitropism weight, dimensionless.
    pub gravitropism: f64,
    /// Hydrotropism weight, dimensionless.
    pub hydrotropism: f64,
}

impl RootType {
    /// `(spec, param)` pairs in [`schema::ROOT_TYPE`] order.
    #[must_use]
    pub fn entries(&self) -> Vec<(Spec, &Param)> {
        let params = [
            &self.count,
            &self.emergence,
            &self.elongation,
            &self.max_length,
            &self.branch_spacing,
            &self.apical_zone,
            &self.insertion_angle,
            &self.diameter,
        ];
        schema::ROOT_TYPE.iter().copied().zip(params).collect()
    }
}

/// Look of the species in the renderer (M1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    /// Leaf template name.
    pub leaf_template: String,
    /// Leaf color, `#rrggbb`.
    pub leaf_color: String,
    /// Flower color, `#rrggbb`.
    pub flower_color: String,
    /// Pod template name.
    pub pod_template: String,
}

/// A species file as written (§6.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Species {
    /// Schema version; must equal [`SCHEMA`].
    pub schema: u32,
    /// Identifier, snake case.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Growth form.
    pub growth_form: GrowthForm,
    /// Non-heritable constants.
    pub constants: Constants,
    /// Heritable parameters by key.
    pub heritable: BTreeMap<String, HeritableParam>,
    /// Loci.
    pub genome: Genome,
    /// Phenology.
    pub phenology: Phenology,
    /// Plant nitrogen (schema 2, WP24).
    pub nitrogen: Nitrogen,
    /// Climbing and indeterminate growth (schema 3, WP22).
    pub climbing: Climbing,
    /// Non-heritable plant-to-plant variation (schema 7, decision 0027).
    pub variation: Variation,
    /// The grass shoot (schema 4, WP23); present exactly when the growth form is
    /// `Grass`.
    #[serde(default)]
    pub grass: Option<Grass>,
    /// Root types, by order.
    pub roots: Vec<RootType>,
    /// Appearance.
    pub appearance: Appearance,
}

/// Errors loading or validating a species file. Each names the file part at fault.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SpeciesError {
    /// RON syntax or structure error (includes unknown fields and wrong types).
    #[error("species file does not parse: {0}")]
    Parse(String),
    /// Unsupported schema version.
    #[error("species schema {found} is not supported (expected {expected})")]
    Schema {
        /// Version in the file.
        found: u32,
        /// Supported version.
        expected: u32,
    },
    /// A parameter's unit string does not match the schema.
    #[error("{at}: unit \"{found}\" is wrong, expected \"{expected}\"")]
    Unit {
        /// Where, for example `constants.carbon_fraction`.
        at: String,
        /// Expected unit.
        expected: &'static str,
        /// Unit in the file.
        found: String,
    },
    /// A parameter has an empty source.
    #[error("{at}: missing source (use \"~\" for a guess to validate)")]
    MissingSource {
        /// Where.
        at: String,
    },
    /// A value is not finite, or a heritable default is outside its bounds.
    #[error("{at}: {why}")]
    Value {
        /// Where.
        at: String,
        /// What is wrong.
        why: String,
    },
    /// A heritable key is unknown, or a required one is missing.
    #[error("heritable.{key}: {why}")]
    HeritableKey {
        /// The key.
        key: String,
        /// "unknown parameter" or "missing".
        why: &'static str,
    },
    /// A locus names a trait that is not a heritable parameter.
    #[error("genome.loci[{index}]: trait \"{trait_name}\" is not a heritable parameter")]
    LocusTrait {
        /// Locus index.
        index: usize,
        /// The trait name.
        trait_name: String,
    },
    /// The root types are inconsistent.
    #[error("roots: {0}")]
    Roots(String),
}

fn check_param(at: &str, unit: &'static str, p: &Param) -> Result<(), SpeciesError> {
    if p.unit != unit {
        return Err(SpeciesError::Unit {
            at: at.to_owned(),
            expected: unit,
            found: p.unit.clone(),
        });
    }
    if p.source.trim().is_empty() {
        return Err(SpeciesError::MissingSource { at: at.to_owned() });
    }
    if !p.value.is_finite() {
        return Err(SpeciesError::Value {
            at: at.to_owned(),
            why: format!("value {} is not finite", p.value),
        });
    }
    Ok(())
}

impl Species {
    /// Parses and validates a species file.
    ///
    /// # Errors
    /// Any [`SpeciesError`]; the message names the offending field.
    pub fn from_ron(text: &str) -> Result<Self, SpeciesError> {
        let s: Self = ron::from_str(text).map_err(|e| SpeciesError::Parse(e.to_string()))?;
        s.validate()?;
        Ok(s)
    }

    /// Checks schema version, units, sources, bounds, keys, loci and root orders.
    ///
    /// # Errors
    /// The first problem found, in file order.
    pub fn validate(&self) -> Result<(), SpeciesError> {
        if self.schema != SCHEMA {
            return Err(SpeciesError::Schema {
                found: self.schema,
                expected: SCHEMA,
            });
        }
        for (spec, p) in self.constants.entries() {
            check_param(&format!("constants.{}", spec.key), spec.unit, p)?;
        }
        for (key, h) in &self.heritable {
            let Some(spec) = schema::find(schema::HERITABLE, key) else {
                return Err(SpeciesError::HeritableKey {
                    key: key.clone(),
                    why: "unknown parameter",
                });
            };
            let at = format!("heritable.{key}");
            check_param(
                &at,
                spec.unit,
                &Param {
                    value: h.default,
                    unit: h.unit.clone(),
                    source: h.source.clone(),
                },
            )?;
            if !(h.min.is_finite() && h.max.is_finite() && h.min <= h.default && h.default <= h.max)
            {
                return Err(SpeciesError::Value {
                    at,
                    why: format!(
                        "default {} must lie within [min {}, max {}]",
                        h.default, h.min, h.max
                    ),
                });
            }
        }
        for spec in schema::HERITABLE {
            if !self.heritable.contains_key(spec.key) {
                return Err(SpeciesError::HeritableKey {
                    key: spec.key.to_owned(),
                    why: "missing",
                });
            }
        }
        for (index, l) in self.genome.loci.iter().enumerate() {
            if !self.heritable.contains_key(&l.trait_name) {
                return Err(SpeciesError::LocusTrait {
                    index,
                    trait_name: l.trait_name.clone(),
                });
            }
            if !(l.effect.is_finite()
                && l.dominance.is_finite()
                && l.allele_sd.is_finite()
                && l.allele_sd >= 0.0)
            {
                return Err(SpeciesError::Value {
                    at: format!("genome.loci[{index}]"),
                    why: "effect, dominance and allele_sd must be finite, allele_sd >= 0".into(),
                });
            }
        }
        check_param(
            "genome.mutation_rate",
            schema::MUTATION_RATE_UNIT,
            &self.genome.mutation_rate,
        )?;
        for (spec, p) in self.phenology.entries() {
            check_param(&format!("phenology.{}", spec.key), spec.unit, p)?;
        }
        for (spec, p) in self.nitrogen.entries() {
            check_param(&format!("nitrogen.{}", spec.key), spec.unit, p)?;
        }
        for (spec, p) in self.climbing.entries() {
            check_param(&format!("climbing.{}", spec.key), spec.unit, p)?;
        }
        for (spec, p) in self.variation.entries() {
            check_param(&format!("variation.{}", spec.key), spec.unit, p)?;
        }
        self.validate_climbing()?;
        self.validate_variation()?;
        self.validate_grass()?;
        self.validate_roots()?;
        for r in &self.roots {
            for (spec, p) in r.entries() {
                check_param(&format!("roots[{}].{}", r.name, spec.key), spec.unit, p)?;
            }
        }
        Ok(())
    }

    fn validate_variation(&self) -> Result<(), SpeciesError> {
        let v = &self.variation;
        for (key, p, hi) in [
            ("seed_mass_cv", &v.seed_mass_cv, 0.5),
            ("seed_leaf_exponent", &v.seed_leaf_exponent, 2.0),
            ("emergence_cv", &v.emergence_cv, 0.5),
            ("emergence_per_depth", &v.emergence_per_depth, 100.0),
            (
                "emergence_reference_depth",
                &v.emergence_reference_depth,
                20.0,
            ),
            ("leaf_size_cv", &v.leaf_size_cv, 0.5),
            ("divergence_sd", &v.divergence_sd, 90.0),
        ] {
            if !(0.0..=hi).contains(&p.value) {
                return Err(SpeciesError::Value {
                    at: format!("variation.{key}"),
                    why: format!("must lie within 0 to {hi}"),
                });
            }
        }
        Ok(())
    }

    fn validate_climbing(&self) -> Result<(), SpeciesError> {
        let c = &self.climbing;
        let bad = |key: &str, why: &str| {
            Err(SpeciesError::Value {
                at: format!("climbing.{key}"),
                why: why.into(),
            })
        };
        let v = |p: &Param| p.value;
        if !(0.0..=1.0).contains(&v(&c.pod_rot_rate)) {
            return bad("pod_rot_rate", "a daily chance, 0 to 1");
        }
        if !(v(&c.support_diameter_sure) >= 0.0
            && v(&c.support_diameter_max) > v(&c.support_diameter_sure))
        {
            return bad(
                "support_diameter_max",
                "must exceed support_diameter_sure, which is 0 or more",
            );
        }
        if !(v(&c.helix_inclination_min) > 0.0
            && v(&c.helix_inclination_max) >= v(&c.helix_inclination_min)
            && v(&c.helix_inclination_max) < 90.0)
        {
            return bad(
                "helix_inclination_max",
                "0 < helix_inclination_min <= helix_inclination_max < 90",
            );
        }
        for (key, p) in [
            ("max_main_nodes", &c.max_main_nodes),
            ("internode_ramp_nodes", &c.internode_ramp_nodes),
            ("circumnutation_radius", &c.circumnutation_radius),
            ("helix_pitch", &c.helix_pitch),
            ("self_support_length", &c.self_support_length),
            ("heap_radius", &c.heap_radius),
        ] {
            if v(p) <= 0.0 {
                return bad(key, "must be positive");
            }
        }
        if !(v(&c.vegetative_after_flowering) >= 0.0 && v(&c.lodge_height) >= 0.0) {
            return bad(
                "lodge_height",
                "lodge_height and vegetative_after_flowering must be 0 or more",
            );
        }
        Ok(())
    }

    fn validate_grass(&self) -> Result<(), SpeciesError> {
        let bad = |key: &str, why: &str| {
            Err(SpeciesError::Value {
                at: format!("grass.{key}"),
                why: why.into(),
            })
        };
        let Some(g) = &self.grass else {
            return if self.growth_form == GrowthForm::Grass {
                bad("", "a Grass growth form needs the grass group")
            } else {
                Ok(())
            };
        };
        if self.growth_form != GrowthForm::Grass {
            return bad("", "only a Grass growth form has the grass group");
        }
        for (spec, p) in g.entries() {
            check_param(&format!("grass.{}", spec.key), spec.unit, p)?;
        }
        for (key, p) in [
            ("site_filling", &g.site_filling),
            ("blade_first_fraction", &g.blade_first_fraction),
            ("blade_flag_fraction", &g.blade_flag_fraction),
        ] {
            if !(0.0..=1.0).contains(&p.value) {
                return bad(key, "a fraction, 0 to 1");
            }
        }
        for (key, p) in [
            ("crown_internode_length", &g.crown_internode_length),
            ("sheath_length_first", &g.sheath_length_first),
            ("sheath_length_last", &g.sheath_length_last),
            ("sheath_specific_length", &g.sheath_specific_length),
            ("blade_aspect", &g.blade_aspect),
            ("tiller_delay", &g.tiller_delay),
            ("peduncle_length", &g.peduncle_length),
        ] {
            if p.value <= 0.0 {
                return bad(key, "must be positive");
            }
        }
        if !(0.0..90.0).contains(&g.blade_elevation.value)
            || !(0.0..90.0).contains(&g.tiller_angle.value)
        {
            return bad("blade_elevation", "angles are 0 to 90 deg");
        }
        if !(0.0..=3.0).contains(&g.tiller_max_order.value)
            || g.elongating_internodes.value < 1.0
            || g.tiller_survive_leaves.value < 0.0
        {
            return bad(
                "tiller_max_order",
                "tiller_max_order 0 to 3, elongating_internodes 1 or more, tiller_survive_leaves 0 or more",
            );
        }
        Ok(())
    }

    fn validate_roots(&self) -> Result<(), SpeciesError> {
        let err = |m: String| Err(SpeciesError::Roots(m));
        if self
            .roots
            .iter()
            .filter(|r| r.origin == RootOrigin::Radicle)
            .count()
            != 1
        {
            return err("exactly one root type must have origin Radicle".into());
        }
        for (i, r) in self.roots.iter().enumerate() {
            if r.name.is_empty() || self.roots[..i].iter().any(|o| o.name == r.name) {
                return err(format!(
                    "root type {i}: name {:?} is empty or repeated",
                    r.name
                ));
            }
            let lateral = r.origin == RootOrigin::Lateral;
            if lateral != (r.order > 0) {
                return err(format!(
                    "{}: laterals have order 1 or more, other origins order 0",
                    r.name
                ));
            }
            if lateral && !self.roots.iter().any(|p| p.order + 1 == r.order) {
                return err(format!("{}: no root type of order {}", r.name, r.order - 1));
            }
            if !(r.gravitropism.is_finite() && r.hydrotropism.is_finite()) {
                return err(format!("{}: tropism weights must be finite", r.name));
            }
        }
        Ok(())
    }

    /// Samples a genotype from the species population: for each locus, two allele
    /// values drawn from N(0, `allele_sd`), keyed by (world seed, plant entity, tick 0,
    /// `GenotypeSample`). With zero variance every allele is exactly 0.
    #[must_use]
    pub fn sample_genotype(&self, world_seed: u64, plant_entity: u64) -> Genotype {
        let mut s = RngKey::new(world_seed, plant_entity, 0, Purpose::GenotypeSample).stream();
        let alleles = self
            .genome
            .loci
            .iter()
            .map(|l| {
                let a = s.standard_normal();
                let b = s.standard_normal();
                if l.allele_sd > 0.0 {
                    (a * l.allele_sd, b * l.allele_sd)
                } else {
                    (0.0, 0.0)
                }
            })
            .collect();
        Genotype { alleles }
    }

    /// The species-default genotype (all alleles zero).
    #[must_use]
    pub fn default_genotype(&self) -> Genotype {
        Genotype {
            alleles: vec![(0.0, 0.0); self.genome.loci.len()],
        }
    }

    /// Maps a genotype to a phenotype (§3.12): for each heritable parameter,
    /// `value = clamp(default + Σ_loci [effect · (a + b)/2 + dominance · |a − b|], min, max)`.
    ///
    /// The dominance form is provisional until M3 breeding (decision 0006).
    #[must_use]
    pub fn phenotype(&self, g: &Genotype) -> Phenotype {
        let mut values: BTreeMap<String, f64> = self
            .heritable
            .iter()
            .map(|(k, h)| (k.clone(), h.default))
            .collect();
        for (l, (a, b)) in self.genome.loci.iter().zip(&g.alleles) {
            if let Some(v) = values.get_mut(&l.trait_name) {
                *v += l.effect * 0.5 * (a + b) + l.dominance * (a - b).abs();
            }
        }
        for (k, v) in &mut values {
            let h = &self.heritable[k];
            *v = v.clamp(h.min, h.max);
        }
        Phenotype { values }
    }

    /// Every numeric parameter as `(path, value, unit, source)`, in file order, for
    /// model cards, the inspector and the provenance check in CI.
    #[must_use]
    pub fn provenance(&self) -> Vec<(String, f64, String, String)> {
        let mut out = Vec::new();
        for (spec, p) in self.constants.entries() {
            out.push((
                format!("constants.{}", spec.key),
                p.value,
                p.unit.clone(),
                p.source.clone(),
            ));
        }
        for (k, h) in &self.heritable {
            out.push((
                format!("heritable.{k}"),
                h.default,
                h.unit.clone(),
                h.source.clone(),
            ));
        }
        for (spec, p) in self.phenology.entries() {
            out.push((
                format!("phenology.{}", spec.key),
                p.value,
                p.unit.clone(),
                p.source.clone(),
            ));
        }
        for (spec, p) in self.nitrogen.entries() {
            out.push((
                format!("nitrogen.{}", spec.key),
                p.value,
                p.unit.clone(),
                p.source.clone(),
            ));
        }
        for (spec, p) in self.climbing.entries() {
            out.push((
                format!("climbing.{}", spec.key),
                p.value,
                p.unit.clone(),
                p.source.clone(),
            ));
        }
        for (spec, p) in self.variation.entries() {
            out.push((
                format!("variation.{}", spec.key),
                p.value,
                p.unit.clone(),
                p.source.clone(),
            ));
        }
        if let Some(g) = &self.grass {
            for (spec, p) in g.entries() {
                out.push((
                    format!("grass.{}", spec.key),
                    p.value,
                    p.unit.clone(),
                    p.source.clone(),
                ));
            }
        }
        for r in &self.roots {
            for (spec, p) in r.entries() {
                out.push((
                    format!("roots[{}].{}", r.name, spec.key),
                    p.value,
                    p.unit.clone(),
                    p.source.clone(),
                ));
            }
        }
        out
    }
}

/// Allele values per locus, in genome order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Genotype {
    /// `(maternal, paternal)` allele values per locus.
    pub alleles: Vec<(f64, f64)>,
}

/// Phenotype values by heritable key, in the schema's units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Phenotype {
    /// Values by key.
    pub values: BTreeMap<String, f64>,
}

impl Phenotype {
    /// The value of a heritable parameter, or `None` for an unknown key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<f64> {
        self.values.get(key).copied()
    }
}

mod traits;

#[cfg(test)]
mod tests;
