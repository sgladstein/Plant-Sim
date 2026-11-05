//! Seed-coat and flower colour from a rule table (decision 0023 item 6, WP28).
//!
//! Colour is a visual-trait layer outside the additive physiological map. A table is a
//! list of layers evaluated in order (for bean: the *P* gate, then *T*, then the
//! *C*/*J*/*V* base colour, then *B*/*G*/*Rk* intensity, then pattern loci). In each
//! layer the first rule whose conditions all hold sets some of the outputs; a rule may
//! stop the evaluation (pp: white seed and white flower, nothing below matters). The
//! result drives only appearance, never physiology.
//!
//! Conditions name named Mendelian loci by gene symbol and test allele state codes
//! (see [`crate::LocusTier::Named`]): `Carries` holds when at least one allele has the
//! state, `Homozygous` when both do (recessive action), and `Expressed` when the higher
//! of the two codes is the state. State codes rise with dominance (0 the most
//! recessive), so `Expressed` reads an allelic series such as V > v^lae > v.
//!
//! The bean table itself waits on the research (Bassett 2007, Parker 2024); this module
//! is the evaluator and its checks. Nothing in the sim calls it yet.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::genetics::allele_state;
use crate::{Genotype, LocusTier, Species, SpeciesError};

/// A seed-colour rule table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedColourTable {
    /// Species ids the table applies to (WP30); the loader matches tables to species by
    /// this list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub species: Vec<String>,
    /// Where the table comes from; "~" marks a guess to validate.
    pub source: String,
    /// The look before any rule applies.
    pub default: SeedLook,
    /// Layers, evaluated in order.
    pub layers: Vec<Layer>,
}

/// One layer of the table: the first matching rule applies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    /// Layer name, for the model card and errors ("P gate", "base colour", ...).
    pub name: String,
    /// Rules, tried in order.
    pub rules: Vec<Rule>,
}

/// A rule: when every condition holds, set the given outputs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Conditions, all of which must hold (an empty list always holds).
    pub when: Vec<Condition>,
    /// Outputs this rule sets; absent fields keep their value.
    #[serde(default)]
    pub set: LookPatch,
    /// Stop evaluating after this rule.
    #[serde(default)]
    pub stop: bool,
    /// Where the rule comes from.
    pub source: String,
}

/// A condition on one named locus.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// At least one allele has this state.
    Carries {
        /// Gene symbol.
        locus: String,
        /// Allele state code.
        state: u8,
    },
    /// Both alleles have this state.
    Homozygous {
        /// Gene symbol.
        locus: String,
        /// Allele state code.
        state: u8,
    },
    /// The more dominant allele (the higher code) has this state.
    Expressed {
        /// Gene symbol.
        locus: String,
        /// Allele state code.
        state: u8,
    },
}

impl Condition {
    fn locus(&self) -> &str {
        match self {
            Condition::Carries { locus, .. }
            | Condition::Homozygous { locus, .. }
            | Condition::Expressed { locus, .. } => locus,
        }
    }

    fn state(&self) -> u8 {
        match self {
            Condition::Carries { state, .. }
            | Condition::Homozygous { state, .. }
            | Condition::Expressed { state, .. } => *state,
        }
    }
}

/// What a seed and its flower look like.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedLook {
    /// Seed-coat pigment colour, `#rrggbb`: the whole coat of a self-coloured seed, the
    /// pattern colour of a patterned one.
    pub seed_coat: String,
    /// Background the pattern sits on, `#rrggbb` (buff under a mottle, white under a
    /// partly coloured pattern). Unseen when `coverage` is 1.
    pub ground: String,
    /// Pattern name ("self", "mottled", "virgarcus", "bipunctata", ...).
    pub pattern: String,
    /// Share of the coat in the pigment colour, 0 to 1.
    pub coverage: f64,
    /// Glossy (true) or matte coat.
    pub glossy: bool,
    /// Flower colour, `#rrggbb`.
    pub flower: String,
}

/// Outputs a rule sets; absent fields keep their value.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LookPatch {
    /// Seed-coat pigment colour, `#rrggbb`.
    #[serde(default)]
    pub seed_coat: Option<String>,
    /// Pattern background, `#rrggbb`.
    #[serde(default)]
    pub ground: Option<String>,
    /// Pattern name.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Share of the coat in the pigment colour, 0 to 1.
    #[serde(default)]
    pub coverage: Option<f64>,
    /// Glossy or matte.
    #[serde(default)]
    pub glossy: Option<bool>,
    /// Flower colour, `#rrggbb`.
    #[serde(default)]
    pub flower: Option<String>,
}

impl LookPatch {
    fn apply(&self, look: &mut SeedLook) {
        if let Some(c) = &self.seed_coat {
            look.seed_coat.clone_from(c);
        }
        if let Some(c) = &self.ground {
            look.ground.clone_from(c);
        }
        if let Some(p) = &self.pattern {
            look.pattern.clone_from(p);
        }
        if let Some(f) = self.coverage {
            look.coverage = f;
        }
        if let Some(g) = self.glossy {
            look.glossy = g;
        }
        if let Some(f) = &self.flower {
            look.flower.clone_from(f);
        }
    }
}

fn is_hex_colour(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

impl SeedColourTable {
    /// Parses a table and checks it against the species' named loci.
    ///
    /// # Errors
    /// [`SpeciesError::Parse`] for bad RON, [`SpeciesError::Value`] naming the first
    /// bad rule.
    pub fn from_ron(text: &str, species: &Species) -> Result<Self, SpeciesError> {
        let t: Self = ron::from_str(text).map_err(|e| SpeciesError::Parse(e.to_string()))?;
        t.validate(species)?;
        Ok(t)
    }

    /// Checks sources, colours, and that every condition names a named locus of the
    /// species with an existing state.
    ///
    /// # Errors
    /// [`SpeciesError::Value`] naming the first problem.
    pub fn validate(&self, species: &Species) -> Result<(), SpeciesError> {
        let bad = |at: String, why: String| Err(SpeciesError::Value { at, why });
        if self.source.is_empty() {
            return bad("seed_colour.source".into(), "missing source".into());
        }
        let states = named_states(species);
        let colours = |at: &str, look: &LookPatch| {
            for c in [&look.seed_coat, &look.ground, &look.flower]
                .into_iter()
                .flatten()
            {
                if !is_hex_colour(c) {
                    return bad(at.to_owned(), format!("\"{c}\" is not #rrggbb"));
                }
            }
            if look.coverage.is_some_and(|f| !(0.0..=1.0).contains(&f)) {
                return bad(at.to_owned(), "coverage must lie in [0, 1]".into());
            }
            Ok(())
        };
        let d = &self.default;
        colours(
            "seed_colour.default",
            &LookPatch {
                seed_coat: Some(d.seed_coat.clone()),
                ground: Some(d.ground.clone()),
                pattern: None,
                coverage: Some(d.coverage),
                glossy: None,
                flower: Some(d.flower.clone()),
            },
        )?;
        for layer in &self.layers {
            for (r, rule) in layer.rules.iter().enumerate() {
                let at = format!("seed_colour.{}.rules[{r}]", layer.name);
                if rule.source.is_empty() {
                    return bad(at, "missing source".into());
                }
                colours(&at, &rule.set)?;
                for c in &rule.when {
                    match states.get(c.locus()) {
                        None => {
                            return bad(at, format!("\"{}\" is not a named locus", c.locus()));
                        }
                        Some(&(_, n)) if c.state() >= n => {
                            return bad(
                                at,
                                format!("{} has states 0 to {}", c.locus(), n.saturating_sub(1)),
                            );
                        }
                        Some(_) => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// The look of a genotype. Conditions on loci the species lacks never hold
    /// (impossible for a validated table).
    #[must_use]
    pub fn evaluate(&self, species: &Species, g: &Genotype) -> SeedLook {
        let states = named_states(species);
        let holds = |c: &Condition| {
            let Some(&(i, n)) = states.get(c.locus()) else {
                return false;
            };
            let Some(&(a, b)) = g.alleles.get(i) else {
                return false;
            };
            let (a, b) = (allele_state(a, n), allele_state(b, n));
            match c {
                Condition::Carries { state, .. } => a == *state || b == *state,
                Condition::Homozygous { state, .. } => a == *state && b == *state,
                Condition::Expressed { state, .. } => a.max(b) == *state,
            }
        };
        let mut look = self.default.clone();
        for layer in &self.layers {
            if let Some(rule) = layer.rules.iter().find(|r| r.when.iter().all(holds)) {
                rule.set.apply(&mut look);
                if rule.stop {
                    break;
                }
            }
        }
        look
    }
}

/// Named loci by symbol: `(index in the genome, allele states)`.
fn named_states(species: &Species) -> BTreeMap<&str, (usize, u8)> {
    species
        .genome
        .loci
        .iter()
        .enumerate()
        .filter_map(|(i, l)| match (l.name.as_deref(), l.tier()) {
            (Some(name), LocusTier::Named { states }) => Some((name, (i, states))),
            _ => None,
        })
        .collect()
}
