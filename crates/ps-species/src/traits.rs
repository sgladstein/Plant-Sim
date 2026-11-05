//! A typed view of the phenotype (WP4 review): named fields instead of string keys, so
//! physiology and architecture code cannot misspell a trait.

use crate::{Phenotype, SpeciesError};

macro_rules! traits {
    ($($(#[doc = $doc:literal])* $field:ident),* $(,)?) => {
        /// Every heritable trait of schema 1, in the schema's units (see `HERITABLE`).
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct Traits {
            $($(#[doc = $doc])* pub $field: f64,)*
        }

        impl Phenotype {
            /// The typed view of this phenotype.
            ///
            /// # Errors
            /// [`SpeciesError::HeritableKey`] naming the first trait the phenotype lacks
            /// (impossible for a phenotype from a validated species).
            pub fn traits(&self) -> Result<Traits, SpeciesError> {
                let get = |key: &str| {
                    self.get(key).ok_or_else(|| SpeciesError::HeritableKey {
                        key: key.to_string(),
                        why: "missing",
                    })
                };
                Ok(Traits { $($field: get(stringify!($field))?,)* })
            }
        }

        /// The trait keys in declaration order (checked against `HERITABLE` in tests).
        pub const TRAIT_KEYS: &[&str] = &[$(stringify!($field)),*];
    };
}

traits!(
    /// umol/m2/s
    vcmax25,
    /// umol/m2/s
    jmax25,
    /// umol/m2/s
    rd25,
    /// kPa^0.5
    medlyn_g1,
    /// -
    nodes_before_terminal,
    /// 0 determinate / 1 indeterminate
    habit,
    /// -
    bud_activation,
    /// cm2
    leaf_area_potential,
    /// cm2/g
    specific_leaf_area,
    /// cm
    internode_length_potential,
    /// cm/g
    specific_stem_length,
    /// Cd
    leaf_lifespan,
    /// -
    root_elongation_scale,
    /// g DM
    seed_mass,
    /// -
    seeds_per_pod,
    /// -
    flowers_per_raceme,
);
