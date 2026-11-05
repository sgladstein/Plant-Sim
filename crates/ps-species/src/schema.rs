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
        /// Q10 of leaf day respiration.
        rd_q10: "-",
        /// Activation energy of Vcmax (peaked Arrhenius).
        vcmax_activation: "J/mol",
        /// Deactivation energy of Vcmax.
        vcmax_deactivation: "J/mol",
        /// Entropy term of Vcmax.
        vcmax_entropy: "J/mol/K",
        /// Activation energy of Jmax (peaked Arrhenius).
        jmax_activation: "J/mol",
        /// Deactivation energy of Jmax.
        jmax_deactivation: "J/mol",
        /// Entropy term of Jmax.
        jmax_entropy: "J/mol/K",
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
        /// FTSW below which stomatal conductance is reduced.
        ftsw_threshold: "-",
        /// FTSW below which expansion growth is reduced.
        ftsw_expansion_threshold: "-",
        /// Days of negative reserve before an organ dies.
        starvation_window: "day",
        /// Phloem conductance per unit segment cross-section over length (§3.9 step 3).
        phloem_conductance: "g/m/h",
        /// Leaves below this share of full-sun PAR for 5 days senesce.
        shade_light_threshold: "-",
        /// Red:far-red below which shade avoidance acts.
        shade_rfr_threshold: "-",
        /// Daily absorbed PAR per leaf area above which low light causes no shade
        /// response (decision 0018).
        shade_dli_onset: "mol/m2/day",
        /// Daily absorbed PAR per leaf area at or below which the shade response is
        /// full.
        shade_dli_full: "mol/m2/day",
        /// Maximum internode length multiplier under shade.
        shade_internode_factor: "-",
        /// Maximum specific leaf area multiplier under shade.
        shade_sla_factor: "-",
        /// Pod-set probability multiplier under full shade (decision 0018).
        shade_pod_set_factor: "-",
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
        /// Daily mean temperature below which no pods set.
        pod_set_t_min: "C",
        /// Lower end of the daily mean temperature range of full pod set.
        pod_set_t_opt_low: "C",
        /// Upper end of the daily mean temperature range of full pod set.
        pod_set_t_opt_high: "C",
        /// Daily mean temperature above which no pods set.
        pod_set_t_max: "C",
        /// FTSW below which flowers abort.
        drought_abortion_ftsw: "-",
        /// Days over which one raceme opens its flowers.
        raceme_flowering_days: "day",
        /// Days a flower stays open before it sets a pod or drops.
        flower_lifespan: "day",
        /// Probability that an open flower sets a pod with no stress.
        pod_set_base: "-",
        /// Probability that each ovule becomes a seed.
        ovule_set: "-",
        /// Potential dry mass of a pod wall.
        pod_wall_mass: "g DM",
        /// Thermal time for a pod wall to reach full size.
        pod_expansion: "Cd",
        /// Thermal time from pod set to the start of seed fill.
        seed_fill_delay: "Cd",
        /// Thermal time for seeds to fill.
        seed_fill: "Cd",
        /// Thermal time between successive nodes on an axis.
        phyllochron: "Cd",
        /// Thermal time for a leaf to reach full size.
        leaf_expansion: "Cd",
        /// Thermal time for an internode to reach full length.
        internode_expansion: "Cd",
        /// Potential area of one unifoliate leaf.
        unifoliate_area: "cm2",
        /// Leaf size at the first trifoliate, relative to the potential.
        leaf_first_fraction: "-",
        /// Leaf size at the last main-stem node, relative to the potential.
        leaf_last_fraction: "-",
        /// Leaf size on branches, relative to the main stem.
        branch_leaf_fraction: "-",
        /// Internode length at the first main-stem node, relative to the potential.
        internode_first_fraction: "-",
        /// Internode length at the last main-stem node, relative to the potential.
        internode_last_fraction: "-",
        /// Internode length on branches, relative to the main stem.
        branch_internode_fraction: "-",
        /// Length of the leaf stalk at full size.
        petiole_length: "cm",
        /// Hypocotyl length above the soil surface.
        hypocotyl_above: "cm",
        /// Trifoliate nodes a branch makes before its terminal raceme.
        branch_nodes: "-",
        /// Angle of a new branch from the vertical.
        branch_angle: "deg",
        /// Reserve concentration at which a bud or raceme is fully fed.
        bud_reserve_reference: "g C / g DM",
        /// Reserve concentration at the raceme's node at which a closing flower sets a
        /// pod with the full base probability (decision 0018).
        pod_set_reserve_reference: "g C / g DM",
        /// Distance, in nodes, over which apical dominance decays by a factor e.
        apical_dominance_nodes: "-",
        /// Days from emergence until the cotyledons drop.
        cotyledon_lifespan: "day",
        /// Absorptance of leaves for PAR.
        leaf_absorptance: "-",
        /// Nitrogen per dry mass of stems.
        stem_n_fraction: "g N / g DM",
        /// Nitrogen per dry mass of roots.
        root_n_fraction: "g N / g DM",
        /// Nitrogen per dry mass of flowers, pods and seeds.
        fruit_n_fraction: "g N / g DM",
        /// Length of one root segment, and of parent root covered by one cohort of laterals.
        root_segment_length: "cm",
        /// Share of the way to vertical a new internode turns per node (shoot gravitropism).
        shoot_gravitropism: "-",
        /// Days below the shade light threshold before a leaf senesces.
        shade_senescence_days: "day",
        /// Hypocotyl length per dry mass.
        hypocotyl_specific_length: "cm/g",
        reserve_feedback_onset: "g C / g DM",
        reserve_feedback_full: "g C / g DM",
        /// Root water uptake coefficient: root length density times this is the hourly share of available water a cell can give.
        root_uptake_rate: "m2/h",
        /// Most water a meter of root can take up in an hour.
        root_uptake_max: "kg/m/h",
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
        /// Thermal time from germination to first flower (decision 0011).
        flowering: "Cd",
        /// Thermal time from germination to physiological maturity (decision 0011).
        maturity: "Cd",
    }
);

/// Numeric fields of one root type (§3.7): key, unit, meaning.
pub const ROOT_TYPE: &[Spec] = &[
    Spec {
        key: "count",
        unit: "-",
        doc: "Number of axes of this type (1 for laterals, whose number follows from the parent's branch spacing).",
    },
    Spec {
        key: "emergence",
        unit: "Cd",
        doc: "Thermal time from germination to the appearance of these axes (unused for laterals).",
    },
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
        doc: "Distance between laterals on this root (0 for none).",
    },
    Spec {
        key: "apical_zone",
        unit: "cm",
        doc: "Unbranched zone behind the tip.",
    },
    Spec {
        key: "insertion_angle",
        unit: "deg",
        doc: "Angle from the vertical (axes from the seed) or from the parent root (laterals).",
    },
    Spec {
        key: "diameter",
        unit: "mm",
        doc: "Root diameter; with the root tissue density it sets the specific root length.",
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
