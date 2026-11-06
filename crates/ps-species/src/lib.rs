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

pub mod genetics;
pub mod schema;
pub mod seed_colour;

use std::collections::BTreeMap;

use ps_rng::{Purpose, RngKey};
use serde::{Deserialize, Serialize};

pub use schema::{
    Climbing, Constants, Grass, Nitrogen, Phenology, SCHEMA, Spec, Storage, Thickening, Variation,
};
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

/// A piecewise-linear table with provenance (schema 12): points `(x, y)` with `x`
/// strictly increasing, read by linear interpolation and held level beyond the ends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParamTable {
    /// Points `(x, y)`, `x` strictly increasing.
    pub points: Vec<(f64, f64)>,
    /// Unit of `x`; must match the schema exactly.
    pub x_unit: String,
    /// Unit of `y`; must match the schema exactly.
    pub unit: String,
    /// Where the values come from; "~" means a guess to validate.
    pub source: String,
}

impl ParamTable {
    /// The table at `x`: linear between points, level beyond the ends.
    #[must_use]
    pub fn at(&self, x: f64) -> f64 {
        interpolate(&self.points, x)
    }
}

/// Linear interpolation in points `(x, y)` with `x` increasing, level beyond the ends;
/// 0 for no points.
#[must_use]
pub fn interpolate(points: &[(f64, f64)], x: f64) -> f64 {
    let Some(&(x0, y0)) = points.first() else {
        return 0.0;
    };
    if x <= x0 {
        return y0;
    }
    for w in points.windows(2) {
        let ((xa, ya), (xb, yb)) = (w[0], w[1]);
        if x <= xb {
            return ya + (yb - ya) * (x - xa) / (xb - xa);
        }
    }
    points[points.len() - 1].1
}

fn check_table(
    at: &str,
    x_unit: &'static str,
    unit: &'static str,
    t: &ParamTable,
) -> Result<(), SpeciesError> {
    let value = |why: String| {
        Err(SpeciesError::Value {
            at: at.to_owned(),
            why,
        })
    };
    if t.x_unit != x_unit {
        return Err(SpeciesError::Unit {
            at: format!("{at}.x_unit"),
            expected: x_unit,
            found: t.x_unit.clone(),
        });
    }
    if t.unit != unit {
        return Err(SpeciesError::Unit {
            at: at.to_owned(),
            expected: unit,
            found: t.unit.clone(),
        });
    }
    if t.source.trim().is_empty() {
        return Err(SpeciesError::MissingSource { at: at.to_owned() });
    }
    if t.points.is_empty() {
        return value("needs at least one point".into());
    }
    if t.points
        .iter()
        .any(|&(x, y)| !(x.is_finite() && y.is_finite() && y >= 0.0))
    {
        return value("points must be finite with y at least 0".into());
    }
    if t.points.windows(2).any(|w| w[1].0 <= w[0].0) {
        return value("x must increase strictly".into());
    }
    Ok(())
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
    /// The heritable parameter this locus contributes to. Empty (absent from the file)
    /// for a named locus with no physiological effect: a colour gene, a marker such as
    /// phaseolin, or a gene whose process the sim does not model yet.
    #[serde(rename = "trait", default, skip_serializing_if = "String::is_empty")]
    pub trait_name: String,
    /// Additive effect per unit allele value, in the trait's unit.
    pub effect: f64,
    /// Dominance deviation per unit allele difference, in the trait's unit.
    pub dominance: f64,
    /// Standard deviation of allele values in the species population.
    pub allele_sd: f64,
    /// Gene symbol of a named Mendelian locus (`fin`, `Ppd`, `P`, ...), which the
    /// seed-colour table refers to. Absent for anonymous QTL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Mutation tier (decision 0023). Absent means an anonymous QTL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<LocusTier>,
    /// Chromosome, 1 to [`genetics::CHROMOSOMES`] (decision 0023). Absent means the
    /// locus is unlinked to every other locus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chromosome: Option<u8>,
    /// Position on its chromosome, cM from the end of the linkage group. Required with
    /// `chromosome`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_cm: Option<f64>,
    /// The allele value of the species file's own pure line, which every
    /// `SeedChoice::Default` seed carries on both homologues; 0 when absent. For a named
    /// locus it is an allele state code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<f64>,
    /// Allele-state frequencies of a named locus in the species population, one per
    /// state, summing to 1. Absent: the population is fixed for `line`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequencies: Option<Vec<f64>>,
    /// Where the locus (its placement, effect and spread) comes from; "~" for a guess.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// How a locus mutates (decision 0023: two tiers).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocusTier {
    /// An anonymous quantitative locus: continuous allele values, mutating at the
    /// genome's `mutation_rate` by a small Gaussian step.
    Qtl,
    /// A named Mendelian locus with `states` discrete alleles coded 0 to `states - 1`
    /// (0 the recessive or null allele), mutating at `named_mutation_rate` to another
    /// state.
    Named {
        /// Number of allele states, at least 2.
        states: u8,
    },
}

impl Locus {
    /// The tier, an anonymous QTL when absent.
    #[must_use]
    pub fn tier(&self) -> LocusTier {
        self.tier.unwrap_or(LocusTier::Qtl)
    }

    /// The pure line's allele value (0 when absent).
    #[must_use]
    pub fn line(&self) -> f64 {
        self.line.unwrap_or(0.0)
    }

    /// Checks `line` and `frequencies` against the tier.
    fn check_alleles(&self) -> Result<(), &'static str> {
        let line = self.line();
        if !line.is_finite() {
            return Err("line must be finite");
        }
        match (self.tier(), &self.frequencies) {
            (LocusTier::Qtl, Some(_)) => Err("only a named locus has allele frequencies"),
            (LocusTier::Qtl, None) => Ok(()),
            (LocusTier::Named { states }, f) => {
                if line.fract() != 0.0 || line < 0.0 || line >= f64::from(states) {
                    return Err("line must be an allele state of the locus");
                }
                let Some(f) = f else { return Ok(()) };
                let sum: f64 = f.iter().sum();
                if f.len() != usize::from(states)
                    || f.iter().any(|p| !(p.is_finite() && *p >= 0.0))
                    || (sum - 1.0).abs() > 1e-9
                {
                    return Err("frequencies: one per state, >= 0, summing to 1");
                }
                Ok(())
            }
        }
    }
}

/// The genome definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Genome {
    /// Loci, in a fixed order (the genotype is a list in this order).
    pub loci: Vec<Locus>,
    /// Mutation rate of anonymous QTL, per locus per gamete.
    pub mutation_rate: Param,
    /// Mutation rate of named Mendelian loci, per locus per gamete (decision 0023).
    /// Absent: [`genetics::DEFAULT_NAMED_MUTATION_RATE`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub named_mutation_rate: Option<Param>,
    /// SD of a QTL mutation step, as a fraction of that locus's `allele_sd`
    /// (decision 0023). Absent: [`genetics::DEFAULT_MUTATION_EFFECT_SD`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_effect_sd: Option<Param>,
    /// Genetic length of each chromosome, Pv01 first (decision 0023). Absent: every
    /// chromosome is [`genetics::DEFAULT_CHROMOSOME_LENGTH_CM`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chromosome_lengths_cm: Option<ChromosomeLengths>,
}

/// Genetic lengths of the chromosomes, one per chromosome in order, with provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChromosomeLengths {
    /// Lengths, one per chromosome, chromosome 1 first.
    pub values: Vec<f64>,
    /// Unit string; must be [`schema::CHROMOSOME_LENGTH_UNIT`].
    pub unit: String,
    /// Where the lengths come from.
    pub source: String,
}

impl Genome {
    /// The optional decision-0023 parameters present in the file, as
    /// `(key, expected unit, param)`.
    #[must_use]
    pub fn optional_params(&self) -> Vec<(&'static str, &'static str, &Param)> {
        [
            (
                "named_mutation_rate",
                schema::MUTATION_RATE_UNIT,
                self.named_mutation_rate.as_ref(),
            ),
            (
                "mutation_effect_sd",
                schema::MUTATION_EFFECT_SD_UNIT,
                self.mutation_effect_sd.as_ref(),
            ),
        ]
        .into_iter()
        .filter_map(|(k, u, p)| p.map(|p| (k, u, p)))
        .collect()
    }

    /// Mutation rate of named loci, per locus per gamete.
    #[must_use]
    pub fn named_mutation_rate(&self) -> f64 {
        self.named_mutation_rate
            .as_ref()
            .map_or(genetics::DEFAULT_NAMED_MUTATION_RATE, |p| p.value)
    }

    /// SD of a QTL mutation step, as a fraction of the locus's allele SD.
    #[must_use]
    pub fn mutation_effect_sd(&self) -> f64 {
        self.mutation_effect_sd
            .as_ref()
            .map_or(genetics::DEFAULT_MUTATION_EFFECT_SD, |p| p.value)
    }

    /// Genetic length of chromosome `c` (1-based), cM.
    #[must_use]
    pub fn chromosome_length_cm(&self, c: u8) -> f64 {
        self.chromosome_lengths_cm
            .as_ref()
            .and_then(|l| l.values.get(usize::from(c).wrapping_sub(1)).copied())
            .unwrap_or(genetics::DEFAULT_CHROMOSOME_LENGTH_CM)
    }
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
    /// Secondary growth scaling by distance from the root's base (schema 12, R1a):
    /// multiplies the species' [`Species::root_secondary_rate`].
    pub secondary_factor: ParamTable,
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
    /// Secondary growth (schema 12, R1a, decision 0036).
    pub thickening: Thickening,
    /// Root diameter growth rate by root segment age at optimum temperature, before
    /// each root type's [`RootType::secondary_factor`] (schema 12, R1a).
    pub root_secondary_rate: ParamTable,
    /// The starch store and root exudation (schema 14, R1b, decision 0040).
    pub storage: Storage,
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
            let visual_only =
                l.trait_name.is_empty() && matches!(l.tier(), LocusTier::Named { .. });
            if !visual_only && !self.heritable.contains_key(&l.trait_name) {
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
        self.validate_genome()?;
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
        for (spec, p) in self.thickening.entries() {
            check_param(&format!("thickening.{}", spec.key), spec.unit, p)?;
        }
        self.validate_thickening()?;
        for (spec, p) in self.storage.entries() {
            check_param(&format!("storage.{}", spec.key), spec.unit, p)?;
        }
        self.validate_storage()?;
        self.validate_development()?;
        self.validate_energy()?;
        self.validate_realism()?;
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

    /// The realism plan's later constants: phototropism (R1c), root plasticity (R2b)
    /// and plant hydraulics (R3b).
    fn validate_realism(&self) -> Result<(), SpeciesError> {
        self.validate_phototropism()?;
        self.validate_root_plasticity()?;
        self.validate_hydraulics()
    }

    /// Thickening (R1a): the flag is 0 or 1, the rates and pipe areas are not
    /// negative, and the tables are well formed.
    fn validate_thickening(&self) -> Result<(), SpeciesError> {
        let t = &self.thickening;
        let flag = t.secondary_growth.value;
        let off = flag.abs() < f64::EPSILON;
        let on = (flag - 1.0).abs() < f64::EPSILON;
        if !(off || on) {
            return Err(SpeciesError::Value {
                at: "thickening.secondary_growth".into(),
                why: "must be 0 or 1".into(),
            });
        }
        for (key, v) in [
            ("pipe_area_per_leaf_area", t.pipe_area_per_leaf_area.value),
            ("pipe_area_per_fruit_mass", t.pipe_area_per_fruit_mass.value),
            ("stem_radial_rate_max", t.stem_radial_rate_max.value),
        ] {
            // NaN fails too.
            if v.is_nan() || v < 0.0 {
                return Err(SpeciesError::Value {
                    at: format!("thickening.{key}"),
                    why: "must be 0 or more".into(),
                });
            }
        }
        check_table(
            "root_secondary_rate",
            "day",
            "cm/day",
            &self.root_secondary_rate,
        )?;
        for r in &self.roots {
            check_table(
                &format!("roots[{}].secondary_factor", r.name),
                "cm",
                "-",
                &r.secondary_factor,
            )?;
        }
        Ok(())
    }

    /// Storage (R1b): every value is 0 or more, the hourly rates and the exudation
    /// share are at most 1, the store releases below where it refills, and the
    /// mining-first flag is 0 or 1.
    fn validate_storage(&self) -> Result<(), SpeciesError> {
        let s = &self.storage;
        for (key, v, max) in [
            ("storage_capacity", s.storage_capacity.value, f64::INFINITY),
            (
                "root_storage_capacity",
                s.root_storage_capacity.value,
                f64::INFINITY,
            ),
            ("storage_on", s.storage_on.value, f64::INFINITY),
            ("storage_off", s.storage_off.value, f64::INFINITY),
            ("storage_refill_rate", s.storage_refill_rate.value, 1.0),
            ("storage_release_rate", s.storage_release_rate.value, 1.0),
            (
                "storage_fill_release_rate",
                s.storage_fill_release_rate.value,
                24.0,
            ),
            ("exudation_fraction", s.exudation_fraction.value, 1.0),
        ] {
            // NaN fails too.
            if !(v >= 0.0 && v <= max) {
                return Err(SpeciesError::Value {
                    at: format!("storage.{key}"),
                    why: format!("must lie within [0, {max}]"),
                });
            }
        }
        if s.storage_off.value > s.storage_on.value {
            return Err(SpeciesError::Value {
                at: "storage.storage_off".into(),
                why: "must not exceed storage_on".into(),
            });
        }
        let flag = self.nitrogen.seed_fill_mining_first.value;
        if !(flag.abs() < f64::EPSILON || (flag - 1.0).abs() < f64::EPSILON) {
            return Err(SpeciesError::Value {
                at: "nitrogen.seed_fill_mining_first".into(),
                why: "must be 0 or 1".into(),
            });
        }
        Ok(())
    }

    /// The development trapezoid must rise, hold and fall (decision 0032), and every
    /// construction cost must be at least 1 (building cannot make carbon).
    fn validate_development(&self) -> Result<(), SpeciesError> {
        let p = &self.phenology;
        let (b, o, o2, m) = (
            p.t_base.value,
            p.t_opt.value,
            p.t_opt_high.value,
            p.t_max.value,
        );
        if !(b < o && o <= o2 && o2 < m) {
            return Err(SpeciesError::Value {
                at: "phenology.t_opt_high".into(),
                why: "need t_base < t_opt <= t_opt_high < t_max".into(),
            });
        }
        let c = &self.constants;
        for (key, v) in [
            ("construction_cost_leaf", c.construction_cost_leaf.value),
            ("construction_cost_stem", c.construction_cost_stem.value),
            ("construction_cost_root", c.construction_cost_root.value),
            ("construction_cost_fruit", c.construction_cost_fruit.value),
        ] {
            // NaN fails too.
            if v.is_nan() || v < 1.0 {
                return Err(SpeciesError::Value {
                    at: format!("constants.{key}"),
                    why: "must be 1 or more".into(),
                });
            }
        }
        Ok(())
    }

    /// Root plasticity (R2b, decision 0043): the water thresholds, floor and
    /// hydrotropism weight are shares, 0 to 1; the proliferation weights 0 to 5; the
    /// hydropatterning flag 0 or 1.
    fn validate_root_plasticity(&self) -> Result<(), SpeciesError> {
        let c = &self.constants;
        for (key, v, max) in [
            ("root_dry_full", c.root_dry_full.value, 1.0),
            ("root_dry_floor", c.root_dry_floor.value, 1.0),
            (
                "xerobranching_threshold",
                c.xerobranching_threshold.value,
                1.0,
            ),
            (
                "root_proliferation_nitrate",
                c.root_proliferation_nitrate.value,
                5.0,
            ),
            (
                "root_proliferation_water",
                c.root_proliferation_water.value,
                5.0,
            ),
            ("hydrotropism_weight", c.hydrotropism_weight.value, 1.0),
        ] {
            // NaN fails too.
            if !(v >= 0.0 && v <= max) {
                return Err(SpeciesError::Value {
                    at: format!("constants.{key}"),
                    why: format!("{v} is outside 0 to {max}"),
                });
            }
        }
        let flag = c.hydropatterning.value;
        if flag.abs() >= f64::EPSILON && (flag - 1.0).abs() >= f64::EPSILON {
            return Err(SpeciesError::Value {
                at: "constants.hydropatterning".into(),
                why: "must be 0 or 1".into(),
            });
        }
        Ok(())
    }

    /// Plant hydraulics (R3b, decision 0044): conductances and the Tuzet sensitivity
    /// are positive, the xylem share is above 0 and at most 1, and the potentials are
    /// ordered `ψ_tlp < ψ_0 ≤ 0` with `ψ_f < 0`.
    fn validate_hydraulics(&self) -> Result<(), SpeciesError> {
        let c = &self.constants;
        let fail = |key: &str, why: String| SpeciesError::Value {
            at: format!("constants.{key}"),
            why,
        };
        for (key, v) in [
            ("root_radial_conductivity", c.root_radial_conductivity.value),
            (
                "root_radial_conductivity_thickened",
                c.root_radial_conductivity_thickened.value,
            ),
            (
                "root_axial_conductance_tip",
                c.root_axial_conductance_tip.value,
            ),
            (
                "root_axial_conductance_tap",
                c.root_axial_conductance_tap.value,
            ),
            (
                "root_axial_conductance_basal",
                c.root_axial_conductance_basal.value,
            ),
            ("root_axial_maturation", c.root_axial_maturation.value),
            ("stem_xylem_conductivity", c.stem_xylem_conductivity.value),
            (
                "leaf_hydraulic_conductance",
                c.leaf_hydraulic_conductance.value,
            ),
            ("tuzet_sensitivity", c.tuzet_sensitivity.value),
        ] {
            // NaN fails too.
            if !(v > 0.0 && v.is_finite()) {
                return Err(fail(key, format!("{v} must be positive")));
            }
        }
        let x = c.xylem_area_fraction.value;
        if !(x > 0.0 && x <= 1.0) {
            return Err(fail(
                "xylem_area_fraction",
                format!("{x} is outside (0, 1]"),
            ));
        }
        let (loss, full) = (c.turgor_loss_point.value, c.turgor_full.value);
        if !(loss < full && full <= 0.0) {
            return Err(fail(
                "turgor_loss_point",
                format!("needs turgor_loss_point {loss} < turgor_full {full} <= 0"),
            ));
        }
        let f = c.tuzet_reference.value;
        if f.is_nan() || f >= 0.0 {
            return Err(fail("tuzet_reference", format!("{f} must be negative")));
        }
        Ok(())
    }

    /// Phototropism (R1c, decision 0041): the weight is a share, 0 to 1, and the daily
    /// turn 0 to 90 degrees.
    fn validate_phototropism(&self) -> Result<(), SpeciesError> {
        let c = &self.constants;
        for (key, v, max) in [
            ("phototropism_weight", c.phototropism_weight.value, 1.0),
            (
                "phototropism_max_per_day",
                c.phototropism_max_per_day.value,
                90.0,
            ),
        ] {
            // NaN fails too.
            if !(v >= 0.0 && v <= max) {
                return Err(SpeciesError::Value {
                    at: format!("constants.{key}"),
                    why: format!("must lie within [0, {max}]"),
                });
            }
        }
        Ok(())
    }

    /// The leaf energy balance keys (WP27, decision 0034): a positive leaf width,
    /// and absorptance, emissivity and the abaxial conductance share as fractions.
    fn validate_energy(&self) -> Result<(), SpeciesError> {
        let c = &self.constants;
        if c.leaf_width.value.is_nan() || c.leaf_width.value <= 0.0 {
            return Err(SpeciesError::Value {
                at: "constants.leaf_width".into(),
                why: "must be more than 0".into(),
            });
        }
        for (key, v, low_open) in [
            ("leaf_nir_absorptance", c.leaf_nir_absorptance.value, false),
            ("leaf_emissivity", c.leaf_emissivity.value, true),
            (
                "abaxial_conductance_share",
                c.abaxial_conductance_share.value,
                false,
            ),
        ] {
            let below = if low_open { v <= 0.0 } else { v < 0.0 };
            // NaN fails too.
            if v.is_nan() || below || v > 1.0 {
                return Err(SpeciesError::Value {
                    at: format!("constants.{key}"),
                    why: if low_open {
                        "must be in (0, 1]".into()
                    } else {
                        "must be in [0, 1]".into()
                    },
                });
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
            ("internode_length_cv", &v.internode_length_cv, 0.5),
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
        if v(&c.circumnutation_growth) < 0.0 {
            return bad("circumnutation_growth", "must be 0 or more");
        }
        if v(&c.circumnutation_radius_max) < v(&c.circumnutation_radius) {
            return bad(
                "circumnutation_radius_max",
                "must be at least circumnutation_radius",
            );
        }
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

    /// Checks the mutation rates, the linkage map and the named loci (decision 0023).
    fn validate_genome(&self) -> Result<(), SpeciesError> {
        let g = &self.genome;
        check_param(
            "genome.mutation_rate",
            schema::MUTATION_RATE_UNIT,
            &g.mutation_rate,
        )?;
        for (key, unit, p) in g.optional_params() {
            check_param(&format!("genome.{key}"), unit, p)?;
        }
        let bad = |at: String, why: &str| {
            Err(SpeciesError::Value {
                at,
                why: why.to_owned(),
            })
        };
        for (key, rate) in [
            ("mutation_rate", g.mutation_rate.value),
            ("named_mutation_rate", g.named_mutation_rate()),
        ] {
            if !(0.0..=1.0).contains(&rate) {
                return bad(format!("genome.{key}"), "must be in [0, 1]");
            }
        }
        if !(g.mutation_effect_sd().is_finite() && g.mutation_effect_sd() >= 0.0) {
            return bad(
                "genome.mutation_effect_sd".into(),
                "must be finite and >= 0",
            );
        }
        if let Some(l) = &g.chromosome_lengths_cm {
            let at = "genome.chromosome_lengths_cm";
            if l.unit != schema::CHROMOSOME_LENGTH_UNIT {
                return Err(SpeciesError::Unit {
                    at: at.into(),
                    found: l.unit.clone(),
                    expected: schema::CHROMOSOME_LENGTH_UNIT,
                });
            }
            if l.source.trim().is_empty() {
                return Err(SpeciesError::MissingSource { at: at.into() });
            }
            if l.values.len() != usize::from(genetics::CHROMOSOMES)
                || l.values.iter().any(|v| !(v.is_finite() && *v > 0.0))
            {
                return bad(at.into(), "one finite length > 0 per chromosome (11)");
            }
        }
        let mut names = std::collections::BTreeSet::new();
        for (index, l) in g.loci.iter().enumerate() {
            let at = format!("genome.loci[{index}]");
            match (l.chromosome, l.position_cm) {
                (None, None) => {}
                (Some(c), Some(x)) => {
                    if !(1..=genetics::CHROMOSOMES).contains(&c) {
                        return bad(at, "chromosome must be 1 to 11");
                    }
                    if !(x.is_finite() && (0.0..=g.chromosome_length_cm(c)).contains(&x)) {
                        return bad(at, "position_cm must lie within its chromosome's length");
                    }
                }
                _ => return bad(at, "chromosome and position_cm come together"),
            }
            if let LocusTier::Named { states } = l.tier() {
                if states < 2 {
                    return bad(at, "a named locus needs at least 2 states");
                }
                if l.name.is_none() {
                    return bad(at, "a named locus needs a name");
                }
            }
            if let Some(n) = &l.name
                && (n.is_empty() || !names.insert(n.as_str()))
            {
                return bad(at, "locus names must be non-empty and unique");
            }
            if l.source.as_deref().is_none_or(|src| src.trim().is_empty()) {
                return bad(at, "missing source (use \"~\" for a guess to validate)");
            }
            if let Err(why) = l.check_alleles() {
                return bad(at, why);
            }
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

    /// Samples a pure line from the species population (bean is an inbreeder, so a
    /// founder seed is homozygous at every locus), keyed by (world seed, plant entity,
    /// tick 0, `GenotypeSample`), each locus on its own four draws. A QTL takes
    /// `line + N(0, allele_sd)`; a named locus takes a state drawn from its `frequencies`
    /// (its `line` state when it has none). With zero variance and no frequencies this
    /// is the default genotype.
    #[must_use]
    pub fn sample_genotype(&self, world_seed: u64, plant_entity: u64) -> Genotype {
        let key = RngKey::new(world_seed, plant_entity, 0, Purpose::GenotypeSample);
        let alleles = self
            .genome
            .loci
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let slot = u32::try_from(i).unwrap_or(u32::MAX).saturating_mul(4);
                let mut s = key.stream_at(slot);
                let line = l.line();
                let v = match (l.tier(), &l.frequencies) {
                    (LocusTier::Named { .. }, Some(f)) => {
                        let u = s.uniform();
                        let mut acc = 0.0;
                        let mut pick = f.len().saturating_sub(1);
                        for (k, p) in f.iter().enumerate() {
                            acc += p;
                            if u < acc {
                                pick = k;
                                break;
                            }
                        }
                        #[allow(clippy::cast_precision_loss)]
                        let state = pick as f64;
                        state
                    }
                    (LocusTier::Named { .. }, None) => line,
                    (LocusTier::Qtl, _) => {
                        let z = s.standard_normal();
                        if l.allele_sd > 0.0 {
                            line + z * l.allele_sd
                        } else {
                            line
                        }
                    }
                };
                (v, v)
            })
            .collect();
        Genotype { alleles }
    }

    /// The species file's pure line: every locus homozygous for its `line` allele.
    #[must_use]
    pub fn default_genotype(&self) -> Genotype {
        Genotype {
            alleles: self
                .genome
                .loci
                .iter()
                .map(|l| (l.line(), l.line()))
                .collect(),
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
    // One flat listing of every group in file order reads best as one function.
    #[allow(clippy::too_many_lines)]
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
        if let Some(l) = &self.genome.chromosome_lengths_cm {
            for (i, v) in l.values.iter().enumerate() {
                out.push((
                    format!("genome.chromosome_lengths_cm[{}]", i + 1),
                    *v,
                    l.unit.clone(),
                    l.source.clone(),
                ));
            }
        }
        for (key, unit, p) in self.genome.optional_params() {
            out.push((
                format!("genome.{key}"),
                p.value,
                unit.to_owned(),
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
        for (spec, p) in self.thickening.entries() {
            out.push((
                format!("thickening.{}", spec.key),
                p.value,
                p.unit.clone(),
                p.source.clone(),
            ));
        }
        for (spec, p) in self.storage.entries() {
            out.push((
                format!("storage.{}", spec.key),
                p.value,
                p.unit.clone(),
                p.source.clone(),
            ));
        }
        let table = |out: &mut Vec<(String, f64, String, String)>, at: &str, t: &ParamTable| {
            for (x, y) in &t.points {
                out.push((
                    format!("{at}[{x} {}]", t.x_unit),
                    *y,
                    t.unit.clone(),
                    t.source.clone(),
                ));
            }
        };
        table(&mut out, "root_secondary_rate", &self.root_secondary_rate);
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
            table(
                &mut out,
                &format!("roots[{}].secondary_factor", r.name),
                &r.secondary_factor,
            );
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

#[cfg(test)]
mod genetics_tests;
