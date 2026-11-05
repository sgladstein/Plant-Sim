//! Generated-by-macro parameter groups of a species file (schema 3).
//!
//! Struct groups use RON struct syntax as in design plan §6.1, and every field is a
//! [`Param`] whose unit is checked against the unit given here. Unknown fields are
//! rejected (typos are errors). Adding or changing a field is a schema change (§6.7).

use serde::{Deserialize, Serialize};

use crate::Param;

/// Current species schema version.
pub const SCHEMA: u32 = 9;

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
        /// Distance, in nodes, over which apical dominance decays by a factor e, past
        /// the held nodes.
        apical_dominance_nodes: "-",
        /// Nodes below a growing main-stem apex whose buds stay fully dormant (WP25).
        apical_hold_nodes: "-",
        /// Days from emergence until the cotyledons drop.
        cotyledon_lifespan: "day",
        /// Absorptance of leaves for PAR.
        leaf_absorptance: "-",
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
    /// Plant nitrogen (§3.6 "Nitrogen (M2)", WP24, decision 0020). "Initial" is the
    /// concentration of new tissue, which the plant tops organs up to while they grow;
    /// "final" is what is left when all mobile nitrogen is gone.
    Nitrogen {
        /// Nitrogen per dry mass of new leaves.
        leaf_n_initial: "g N / g DM",
        /// Nitrogen per dry mass left in a fully mined leaf.
        leaf_n_final: "g N / g DM",
        /// Nitrogen per dry mass of new stems (and buds).
        stem_n_initial: "g N / g DM",
        /// Nitrogen per dry mass left in a fully mined stem.
        stem_n_final: "g N / g DM",
        /// Nitrogen per dry mass of new roots.
        root_n_initial: "g N / g DM",
        /// Nitrogen per dry mass left in a fully mined root.
        root_n_final: "g N / g DM",
        /// Nitrogen per dry mass of new flowers and pod walls.
        shell_n_initial: "g N / g DM",
        /// Nitrogen per dry mass left in a fully mined pod wall.
        shell_n_final: "g N / g DM",
        /// Nitrogen per dry mass of filling seeds (and of the sown seed).
        seed_n_initial: "g N / g DM",
        /// Nitrogen per dry mass below which a seed stops filling.
        seed_n_final: "g N / g DM",
        /// Nitrogen per dry mass of nodules.
        nodule_n: "g N / g DM",
        /// Leaf nitrogen per area at which photosynthetic capacity is zero.
        leaf_n_area_base: "g N / m2",
        /// Uptake per length of root per day when soil mineral nitrogen is high.
        uptake_capacity: "g N / m / day",
        /// How far around each drawn root the plant takes up nitrogen: its unresolved
        /// fine roots and the nitrate the soil water carries to them.
        root_reach: "cm",
        /// Share of an organ's mobile nitrogen that can move out per day once seeds fill.
        mobilization_rate: "1/day",
        /// Mobilization rate before seed fill, relative to the rate above.
        vegetative_mobilization: "-",
        /// A leaf drops once its mobile nitrogen falls below this share of a new leaf's.
        leaf_drop_mobile: "-",
        /// Nitrogen fixed per nodule dry mass per day at best (0 for a non-legume).
        nodule_activity: "g N / g DM / day",
        /// Most the plant's nodule mass can grow per day, relative to itself.
        nodule_relative_growth: "1/day",
        /// Nodule mass a plant starts when it first nodulates.
        nodule_initial_mass: "g DM",
        /// Potential dry mass of one nodule cluster organ.
        nodule_mass: "g DM",
        /// Thermal time for a nodule cluster to reach full size.
        nodule_expansion: "Cd",
        /// Roots deeper than this carry no nodules.
        nodule_depth: "cm",
        /// Carbon respired per nitrogen fixed.
        fixation_cost: "g C / g N",
        /// Soil temperature below which nodules fix nothing.
        fixation_t_min: "C",
        /// Lower end of the soil temperature range of full fixation.
        fixation_t_opt_low: "C",
        /// Upper end of the soil temperature range of full fixation.
        fixation_t_opt_high: "C",
        /// Soil temperature above which nodules fix nothing.
        fixation_t_max: "C",
        /// Expansion water-stress factor below which fixation falls, linearly to 0.
        fixation_drought_threshold: "-",
        /// Root-zone nitrate above which new nodules are suppressed.
        nodule_nitrate_onset: "mg N / kg",
        /// Root-zone nitrate at which nodule initiation reaches its floor.
        nodule_nitrate_full: "mg N / kg",
        /// Nodule initiation at high nitrate, relative to none.
        nodule_nitrate_floor: "-",
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

param_group!(
    /// Non-heritable plant-to-plant variation (schema 7, decision 0027): what makes
    /// two plants of one pure line differ although they share a genotype. Every draw
    /// is keyed by (world seed, plant, tick, purpose), so a world seed always gives the
    /// same plants. All zeros make every plant of a species start alike.
    Variation {
        /// Coefficient of variation of the mass of the individual seeds sown.
        seed_mass_cv: "-",
        /// Exponent of the preformed (unifoliate or first) leaves' size on the relative
        /// seed mass.
        seed_leaf_exponent: "-",
        /// Coefficient of variation of each seed's thermal time from germination to
        /// emergence.
        emergence_cv: "-",
        /// Extra thermal time to emergence per cm sown deeper than the reference depth.
        emergence_per_depth: "Cd/cm",
        /// Sowing depth at which the species' emergence threshold applies.
        emergence_reference_depth: "cm",
        /// Coefficient of variation of each leaf's potential area about the node
        /// profile.
        leaf_size_cv: "-",
        /// Standard deviation of each node's divergence angle about the phyllotaxis.
        divergence_sd: "deg",
        /// Coefficient of variation of each internode's potential length about the
        /// node profile (schema 8, decision 0029).
        internode_length_cv: "-",
    }
);

param_group!(
    /// Climbing and indeterminate growth (schema 3, WP22, decision 0021). Read only
    /// when the `habit` trait is 0.5 or more; a determinate species carries the group
    /// with its values unused.
    Climbing {
        /// Most main-stem nodes an indeterminate stem makes.
        max_main_nodes: "-",
        /// Thermal time after first flower over which the main stem keeps making nodes
        /// (CROPGRO FL-VS).
        vegetative_after_flowering: "Cd",
        /// Main-stem nodes over which internodes rise from the first fraction to full
        /// length.
        internode_ramp_nodes: "-",
        /// Radius of the circle a young shoot tip sweeps while it searches for a
        /// support.
        circumnutation_radius: "cm",
        /// Sweep radius gained per length of stem grown upright without a support
        /// (schema 8, decision 0029): the longer the free tip, the wider it swings.
        circumnutation_growth: "-",
        /// Widest sweep radius a searching tip reaches (schema 8, decision 0029).
        circumnutation_radius_max: "cm",
        /// Supports up to this diameter are always caught.
        support_diameter_sure: "cm",
        /// Supports this thick or more are never caught.
        support_diameter_max: "cm",
        /// Rise of the twining helix per turn; with the support's radius it sets the
        /// helix angle.
        helix_pitch: "cm",
        /// Flattest helix angle above the horizontal (thick supports).
        helix_inclination_min: "deg",
        /// Steepest helix angle above the horizontal (thin strings).
        helix_inclination_max: "deg",
        /// Stem length an unsupported vine holds up before it lodges.
        self_support_length: "cm",
        /// Radius around the collar within which a lodged vine coils.
        heap_radius: "cm",
        /// Height below which a lodged stem runs level instead of sagging.
        lodge_height: "cm",
        /// Daily chance that a vine's pod hanging onto the soil rots.
        pod_rot_rate: "1/day",
    }
);

param_group!(
    /// The grass shoot (schema 4, WP23, decision 0022): tillers, sheaths and blades,
    /// stem elongation and the ear. Required when the growth form is `Grass`, absent
    /// otherwise. The final leaf number is the `nodes_before_terminal` trait.
    Grass {
        /// Length of a crown internode (the short internodes of the tillering phase).
        crown_internode_length: "cm",
        /// Main-stem internodes below the peduncle that elongate (from jointing on).
        elongating_internodes: "-",
        /// Sheath length of the first leaf.
        sheath_length_first: "cm",
        /// Sheath length of the flag leaf; lengths rise linearly with rank between.
        sheath_length_last: "cm",
        /// Sheath length per gram of sheath dry mass.
        sheath_specific_length: "cm/g",
        /// Blade length over blade width.
        blade_aspect: "-",
        /// Angle of a blade above the horizontal at its base.
        blade_elevation: "deg",
        /// Phyllochrons from a leaf's appearance to the tiller in its axil.
        tiller_delay: "-",
        /// Chance that a tiller site, when its time comes, grows a tiller.
        site_filling: "-",
        /// Red:far-red below which no new tillers start.
        tiller_stop_rfr: "-",
        /// Angle of a new tiller from the vertical.
        tiller_angle: "deg",
        /// Azimuth of a tiller's first leaf from its parent's leaf.
        tiller_azimuth: "deg",
        /// Highest tiller order (1 primary, 2 secondary).
        tiller_max_order: "-",
        /// At jointing, tillers with fewer visible leaves than this die.
        tiller_survive_leaves: "-",
        /// Potential length of the peduncle, the internode that carries the ear.
        peduncle_length: "cm",
        /// Area of the first blade as a fraction of the largest.
        blade_first_fraction: "-",
        /// Area of the flag blade as a fraction of the largest.
        blade_flag_fraction: "-",
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
/// Unit of the QTL mutation step SD (decision 0023).
pub const MUTATION_EFFECT_SD_UNIT: &str = "allele SD";
/// Unit of a chromosome's genetic length (decision 0023).
pub const CHROMOSOME_LENGTH_UNIT: &str = "cM";

/// Looks up a spec by key.
#[must_use]
pub fn find(table: &[Spec], key: &str) -> Option<Spec> {
    table.iter().copied().find(|s| s.key == key)
}
