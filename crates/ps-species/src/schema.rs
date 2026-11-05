//! Generated-by-macro parameter groups of a species file (schema 1).
//!
//! Struct groups use RON struct syntax as in design plan §6.1, and every field is a
//! [`Param`] whose unit is checked against the unit given here. Unknown fields are
//! rejected (typos are errors). Adding or changing a field is a schema change (§6.7).

use serde::{Deserialize, Serialize};

use crate::Param;

/// Current species schema version.
pub const SCHEMA: u32 = 1;

/// A parameter the schema knows: key, exact unit string, meaning.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// Key in the file.
    pub key: &'static str,
    /// Exact unit string.
    pub unit: &'static str,
    /// What it means.
    pub doc: &'static str,
}

macro_rules! param_group {
    ($(#[$m:meta])* $name:ident { $($(#[doc = $doc:literal])* $field:ident : $unit:literal),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $($(#[doc = $doc])* pub $field: Param,)*
        }

        impl $name {
            /// Field specs in declaration order.
            pub const SPECS: &'static [Spec] = &[
                $(Spec { key: stringify!($field), unit: $unit, doc: concat!($($doc),*) },)*
            ];

            /// `(spec, param)` pairs in declaration order.
            #[must_use]
            pub fn entries(&self) -> Vec<(Spec, &Param)> {
                let params: Vec<&Param> = vec![$(&self.$field),*];
                Self::SPECS.iter().copied().zip(params).collect()
            }
        }
    };
}

param_group!(
    /// Fixed physiological constants: not heritable (§3.12).
    Constants {
        /// Carbon per unit dry matter.
        carbon_fraction: "g C / g DM",
        /// Carbon used per carbon built into structure (1.25 = 20% growth respiration).
        construction_cost: "g C / g C",
        /// Leaf angle G-function (0.5 = spherical).
        g_function: "-",
        /// Maintenance respiration per nitrogen at 20 C.
        maintenance_rate_20c: "g C / g N / day",
        /// Q10 of maintenance respiration.
        maintenance_q10: "-",
        /// Residual stomatal conductance.
        medlyn_g0: "mol/m2/s",
        /// Curvature of the J light response.
        electron_transport_curvature: "-",
        /// Electrons per absorbed photon.
        quantum_yield: "mol/mol",
        /// Photosynthetic capacity at end of leaf life, relative to full.
        leaf_capacity_floor: "-",
        /// Reserve concentration below which an organ cannot pay for growth.
        reserve_floor: "g C / g DM",
        /// Structural carbon resorbed to the parent at death.
        resorption_fraction: "-",
        /// Carbon per unit seed dry matter.
        seed_carbon_fraction: "g C / g DM",
        /// Share of seed carbon available to the seedling.
        seed_mobilizable_fraction: "-",
        /// Cotyledons senesce below this share of their starting reserve.
        cotyledon_drop_reserve: "-",
        /// Seed-cell FTSW needed for germination.
        germination_ftsw: "-",
        /// Consecutive hours above thresholds to germinate.
        germination_hours: "h",
        /// FTSW below which water stress begins.
        ftsw_threshold: "-",
        /// Days of negative reserve before an organ dies.
        starvation_window: "day",
        /// Leaves below this share of full-sun PAR for 5 days senesce.
        shade_light_threshold: "-",
        /// Red:far-red below which shade avoidance acts.
        shade_rfr_threshold: "-",
        /// Maximum internode length multiplier under shade.
        shade_internode_factor: "-",
        /// Maximum specific leaf area multiplier under shade.
        shade_sla_factor: "-",
        /// Air temperature that kills leaves.
        leaf_frost_kill: "C",
        /// Temperature where the expansion multiplier reaches zero (low).
        growth_t_min: "C",
        /// Lower end of the optimal expansion range.
        growth_t_opt_low: "C",
        /// Upper end of the optimal expansion range.
        growth_t_opt_high: "C",
        /// Temperature where the expansion multiplier reaches zero (high).
        growth_t_max: "C",
        /// Daytime temperature above which flowers abort.
        heat_abortion_t: "C",
        /// FTSW below which flowers abort.
        drought_abortion_ftsw: "-",
        /// Days over which one raceme opens its flowers.
        raceme_flowering_days: "day",
        /// Thermal time for a pod to reach full length.
        pod_expansion: "Cd",
        /// Thermal time for seeds to fill.
        seed_fill: "Cd",
        /// Root length per root dry mass.
        specific_root_length: "m/g",
        /// Stem tissue density.
        stem_density: "g/m3",
        /// Root tissue density.
        root_density: "g/m3",
        /// Fruit, seed and flower tissue density.
        bulk_density: "g/m3",
        /// Pod length over diameter.
        pod_aspect: "-",
        /// Seed length over diameter.
        seed_aspect: "-",
        /// Divergence angle between successive leaves.
        phyllotaxis_angle: "deg",
    }
);

param_group!(
    /// Cardinal temperatures and stage thresholds (§3.11, §6.1).
    Phenology {
        /// Base temperature of thermal time.
        t_base: "C",
        /// Optimum temperature of thermal time.
        t_opt: "C",
        /// Maximum temperature of thermal time.
        t_max: "C",
        /// Thermal time from germination to emergence.
        emergence: "Cd",
        /// Thermal time from sowing to first flower.
        flowering: "Cd",
        /// Thermal time from sowing to physiological maturity.
        maturity: "Cd",
    }
);

/// Numeric fields of one root type (§3.7): key, unit, meaning.
pub const ROOT_TYPE: &[Spec] = &[
    Spec {
        key: "elongation",
        unit: "cm/day",
        doc: "Potential tip elongation at optimum temperature.",
    },
    Spec {
        key: "max_length",
        unit: "cm",
        doc: "Maximum root length.",
    },
    Spec {
        key: "branch_spacing",
        unit: "cm",
        doc: "Distance between lateral branches.",
    },
    Spec {
        key: "apical_zone",
        unit: "cm",
        doc: "Unbranched zone behind the tip.",
    },
    Spec {
        key: "insertion_angle",
        unit: "deg",
        doc: "Angle of laterals from the parent root.",
    },
];

/// Heritable phenotype parameters with bounds (§3.12): key, unit, meaning. In the file
/// they form the `heritable` map, each entry with default, min, max, unit and source.
pub const HERITABLE: &[Spec] = &[
    Spec {
        key: "vcmax25",
        unit: "umol/m2/s",
        doc: "maximum carboxylation rate at 25 C",
    },
    Spec {
        key: "jmax25",
        unit: "umol/m2/s",
        doc: "maximum electron transport rate at 25 C",
    },
    Spec {
        key: "rd25",
        unit: "umol/m2/s",
        doc: "leaf dark respiration at 25 C",
    },
    Spec {
        key: "medlyn_g1",
        unit: "kPa^0.5",
        doc: "Medlyn stomatal slope",
    },
    Spec {
        key: "nodes_before_terminal",
        unit: "-",
        doc: "main-stem nodes before the terminal raceme",
    },
    Spec {
        key: "habit",
        unit: "0 determinate / 1 indeterminate",
        doc: "growth habit locus",
    },
    Spec {
        key: "bud_activation",
        unit: "-",
        doc: "base probability that a lit, fed axillary bud activates",
    },
    Spec {
        key: "leaf_area_potential",
        unit: "cm2",
        doc: "potential area of a mature trifoliate leaf",
    },
    Spec {
        key: "specific_leaf_area",
        unit: "cm2/g",
        doc: "leaf area per leaf dry mass",
    },
    Spec {
        key: "internode_length_potential",
        unit: "cm",
        doc: "potential length of a main-stem internode",
    },
    Spec {
        key: "specific_stem_length",
        unit: "cm/g",
        doc: "internode length per internode dry mass",
    },
    Spec {
        key: "leaf_lifespan",
        unit: "Cd",
        doc: "leaf lifespan after full expansion",
    },
    Spec {
        key: "root_elongation_scale",
        unit: "-",
        doc: "multiplier on all root elongation rates (root depth tendency)",
    },
    Spec {
        key: "seed_mass",
        unit: "g DM",
        doc: "potential final mass of one seed",
    },
    Spec {
        key: "seeds_per_pod",
        unit: "-",
        doc: "maximum seeds per pod",
    },
    Spec {
        key: "flowers_per_raceme",
        unit: "-",
        doc: "flowers per raceme",
    },
];

/// Unit required for `genome.mutation_rate`.
pub const MUTATION_RATE_UNIT: &str = "per locus per gamete";

/// Looks up a spec by key.
#[must_use]
pub fn find(table: &[Spec], key: &str) -> Option<Spec> {
    table.iter().copied().find(|s| s.key == key)
}
