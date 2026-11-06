//! Species and phenotype values converted to model units once, at sowing.

use ps_graph::Allometry;
use ps_physio::allocation::{CarbonParams, Temperature};
use ps_physio::{Cardinal, GrowthTemperature, LeafEnergyParams, LeafParams, Thresholds};
use ps_species::{RootOrigin, Species, Traits};
use ps_units::{
    Celsius, DegreeDays, GPerM3, GramsDM, KgPerMH, M2, M2PerG, M2PerHour, MPerG, Meters,
    MolPerM2Day, Radians,
};
use serde::{Deserialize, Serialize};

use crate::grass::GrassParams;
use crate::nitrogen::NitrogenParams;
use crate::thicken::ThickeningParams;
use crate::vine::ClimbingParams;

/// Thermal time per day at the optimum temperature, °C·day, for converting rates given
/// per day at the optimum into rates per °C·day.
#[must_use]
pub fn optimum_rate(cardinal: &Cardinal) -> f64 {
    (cardinal.opt.0 - cardinal.base.0).max(1e-9)
}

/// One root type in model units.
/// The light a new organ's node sees, for the shade response.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShadeSignal {
    /// Red:far-red proxy (1.2 in the open).
    pub rfr: f64,
    /// Yesterday's absorbed PAR per leaf area, mol m⁻² day⁻¹, if known.
    pub dli: Option<MolPerM2Day>,
}

impl ShadeSignal {
    /// No shade.
    pub const OPEN: Self = Self {
        rfr: 1.2,
        dli: None,
    };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RootParams {
    /// Name from the species file.
    pub name: String,
    /// Where its axes start.
    pub origin: RootOrigin,
    /// Branching order.
    pub order: u8,
    /// Axes of this type (1 for laterals).
    pub count: u32,
    /// Thermal time from germination to appearance.
    pub emergence: DegreeDays,
    /// Potential elongation per °C·day of the expansion clock, m.
    pub elongation_per_cd: f64,
    /// Maximum length of one root.
    pub max_length: Meters,
    /// Distance between laterals (0 for none).
    pub branch_spacing: Meters,
    /// Unbranched zone behind the tip.
    pub apical_zone: Meters,
    /// Angle from the vertical (axes) or from the parent (laterals).
    pub insertion: Radians,
    /// Specific root length from diameter and tissue density.
    pub specific_length: MPerG,
    /// Gravitropism weight.
    pub gravitropism: f64,
    /// Secondary growth factor by distance from the root's base, `(m, factor)`
    /// (R1a, decision 0036).
    #[serde(default)]
    pub secondary_factor: Vec<(f64, f64)>,
}

/// Non-heritable plant-to-plant variation (decision 0027), in model units.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VariationParams {
    /// Coefficient of variation of the sown seed's mass.
    pub seed_mass_cv: f64,
    /// Exponent of the preformed leaves' size on the relative seed mass.
    pub seed_leaf_exponent: f64,
    /// Coefficient of variation of the thermal time to emergence.
    pub emergence_cv: f64,
    /// Extra thermal time to emergence per metre sown below the reference depth.
    pub emergence_per_depth: f64,
    /// Depth the species' emergence threshold applies at.
    pub reference_depth: Meters,
    /// Coefficient of variation of a leaf's potential area.
    pub leaf_size_cv: f64,
    /// Standard deviation of a node's divergence angle.
    pub divergence_sd: Radians,
    /// Coefficient of variation of an internode's potential length.
    pub internode_length_cv: f64,
}

impl VariationParams {
    /// Converts the species' `variation` group.
    #[must_use]
    pub fn new(s: &Species) -> Self {
        let v = &s.variation;
        Self {
            seed_mass_cv: v.seed_mass_cv.value,
            seed_leaf_exponent: v.seed_leaf_exponent.value,
            emergence_cv: v.emergence_cv.value,
            emergence_per_depth: v.emergence_per_depth.value / CM,
            reference_depth: Meters(v.emergence_reference_depth.value * CM),
            leaf_size_cv: v.leaf_size_cv.value,
            divergence_sd: Radians(v.divergence_sd.value.to_radians()),
            internode_length_cv: v.internode_length_cv.value,
        }
    }
}

/// A relative size `1 + cv·z` from a standard normal draw `z`, with `z` held to ±2.5
/// so a factor stays positive and bounded (at most cv 0.3 gives 0.25 to 1.75).
#[must_use]
pub fn scatter(z: f64, cv: f64) -> f64 {
    (1.0 + cv * z.clamp(-2.5, 2.5)).max(0.25)
}

/// Everything the herb module reads, in model units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct HerbParams {
    /// Carbon constants for allocation.
    pub carbon: CarbonParams,
    /// Temperature responses for growth.
    pub temperature: Temperature,
    /// Leaf gas-exchange parameters.
    pub leaf: LeafParams,
    /// Phenology thresholds.
    pub thresholds: Thresholds,
    /// Plant-to-plant variation (decision 0027).
    #[serde(default)]
    pub variation: VariationParams,
    /// Organ allometry.
    pub allometry: Allometry,
    /// PAR absorptance of leaves.
    pub absorptance: f64,
    /// Leaf energy balance parameters (WP27, decision 0034).
    pub energy: LeafEnergyParams,
    /// Photosynthetic capacity at the end of leaf life.
    pub capacity_floor: f64,
    /// FTSW below which conductance falls.
    pub ftsw_conductance: f64,
    /// FTSW below which expansion falls.
    pub ftsw_expansion: f64,
    /// Seed dry mass.
    pub seed_mass: GramsDM,
    /// Carbon per seed dry mass.
    pub seed_carbon_fraction: f64,
    /// Share of seed carbon the seedling can use.
    pub seed_mobilizable: f64,
    /// Cotyledons drop below this share of the starting seed reserve.
    pub cotyledon_drop_reserve: f64,
    /// Days from emergence to cotyledon drop.
    pub cotyledon_days: u32,
    /// Seed-cell FTSW needed for germination.
    pub germination_ftsw: f64,
    /// Consecutive suitable hours to germinate.
    pub germination_hours: u32,
    /// Main-stem nodes before the terminal raceme.
    pub main_nodes: u32,
    /// Trifoliate nodes per branch.
    pub branch_nodes: u32,
    /// Thermal time per node.
    pub phyllochron: DegreeDays,
    /// Leaf expansion duration.
    pub leaf_expansion: DegreeDays,
    /// Internode expansion duration.
    pub internode_expansion: DegreeDays,
    /// Potential trifoliate area at the best nodes.
    pub leaf_area: M2,
    /// Potential area of one unifoliate leaf.
    pub unifoliate_area: M2,
    /// Specific leaf area.
    pub sla: M2PerG,
    /// Leaf profile: first trifoliate, last main-stem node, branches.
    pub leaf_profile: [f64; 3],
    /// Potential internode length.
    pub internode_length: Meters,
    /// Internode profile: first node, last node, branches.
    pub internode_profile: [f64; 3],
    /// Specific stem length.
    pub stem_specific_length: MPerG,
    /// Hypocotyl specific length.
    pub hypocotyl_specific_length: MPerG,
    /// Petiole length at full size.
    pub petiole: Meters,
    /// Hypocotyl length above ground.
    pub hypocotyl_above: Meters,
    /// Angle of new branches from vertical.
    pub branch_angle: Radians,
    /// Divergence angle between successive leaves.
    pub phyllotaxis: Radians,
    /// Shoot gravitropism per node.
    pub shoot_gravitropism: f64,
    /// Share of the angle to the light a new internode turns (R1c, decision 0041).
    #[serde(default)]
    pub phototropism_weight: f64,
    /// Most a shoot turns toward the light per day of light seen (R1c).
    #[serde(default)]
    pub phototropism_max_per_day: Radians,
    /// Base daily bud activation probability.
    pub bud_activation: f64,
    /// Reserve concentration of a fully fed bud, g C per g DM.
    pub bud_reserve_reference: f64,
    /// Reserve concentration at which a flower sets with the full base probability.
    pub pod_set_reserve_reference: f64,
    /// Apical dominance decay length in nodes.
    pub dominance_nodes: f64,
    /// Nodes below a growing main-stem apex whose buds stay dormant (WP25).
    pub hold_nodes: f64,
    /// Shade senescence light threshold.
    pub shade_light: f64,
    /// Days below it before a leaf senesces.
    pub shade_days: u32,
    /// Red:far-red threshold of shade avoidance.
    pub shade_rfr: f64,
    /// Daily absorbed PAR (mol m⁻² day⁻¹) at which the low-light response starts.
    pub shade_dli_onset: MolPerM2Day,
    /// Daily absorbed PAR (mol m⁻² day⁻¹) at which the low-light response is full.
    pub shade_dli_full: MolPerM2Day,
    /// Internode multiplier at full shade avoidance.
    pub shade_internode: f64,
    /// SLA multiplier at full shade avoidance.
    pub shade_sla: f64,
    /// Pod-set multiplier at full shade.
    pub shade_pod_set: f64,
    /// Leaf lifespan in thermal age.
    pub leaf_lifespan: DegreeDays,
    /// Leaf temperature that kills a leaf (its own, WP27; air for the growing points).
    pub frost: Celsius,
    /// Flowers per raceme.
    pub flowers_per_raceme: u32,
    /// Days over which a raceme opens its flowers.
    pub raceme_days: f64,
    /// Days a flower is open.
    pub flower_days: u32,
    /// Unstressed pod set probability.
    pub pod_set_base: f64,
    /// Pod-set temperature response on the daily mean: min, opt low, opt high, max.
    pub pod_set_t: [f64; 4],
    /// FTSW below which flowers abort.
    pub drought_abortion_ftsw: f64,
    /// Ovules per pod.
    pub seeds_per_pod: u32,
    /// Probability each ovule becomes a seed.
    pub ovule_set: f64,
    /// Pod wall potential mass.
    pub pod_wall: GramsDM,
    /// Pod wall expansion duration.
    pub pod_expansion: DegreeDays,
    /// Delay from pod set to seed fill.
    pub seed_fill_delay: DegreeDays,
    /// Seed fill duration.
    pub seed_fill: DegreeDays,
    /// Potential mass of one seed.
    pub seed_potential: GramsDM,
    /// Nitrogen (WP24, decision 0020).
    pub nitrogen: NitrogenParams,
    /// Indeterminate vine (`habit` ≥ 0.5, WP22).
    #[serde(default)]
    pub indeterminate: bool,
    /// Climbing (vines only).
    #[serde(default)]
    pub climbing: ClimbingParams,
    /// The grass shoot (grasses only, WP23).
    #[serde(default)]
    pub grass: Option<GrassParams>,
    /// Root segment and lateral cohort length.
    pub root_segment: Meters,
    /// Root types, in file order.
    pub roots: Vec<RootParams>,
    /// Stem and root secondary growth (R1a, decision 0036).
    #[serde(default)]
    pub thickening: ThickeningParams,
    /// Leaf reserve concentration where sink feedback on capacity starts, g C per g DM.
    pub feedback_onset: f64,
    /// Leaf reserve concentration where capacity reaches zero, g C per g DM.
    pub feedback_full: f64,
    /// Root uptake coefficient.
    pub uptake_rate: M2PerHour,
    /// Root uptake ceiling.
    pub uptake_max: KgPerMH,
}

fn round(x: f64) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = libm::round(x.max(0.0)) as u32;
    n
}

const CM: f64 = 0.01;
const CM2: f64 = 1e-4;

impl HerbParams {
    /// Converts a species and a phenotype.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(s: &Species, t: &Traits) -> Self {
        let c = &s.constants;
        let cardinal = Cardinal::from_species(s);
        let rate = optimum_rate(&cardinal);
        let root_density = c.root_density.value;
        let roots = s
            .roots
            .iter()
            .map(|r| {
                let d = r.diameter.value * 1e-3;
                let area = core::f64::consts::PI * 0.25 * d * d;
                RootParams {
                    name: r.name.clone(),
                    origin: r.origin,
                    order: r.order,
                    count: round(r.count.value),
                    emergence: DegreeDays(r.emergence.value),
                    elongation_per_cd: r.elongation.value * CM * t.root_elongation_scale / rate,
                    max_length: Meters(r.max_length.value * CM),
                    branch_spacing: Meters(r.branch_spacing.value * CM),
                    apical_zone: Meters(r.apical_zone.value * CM),
                    insertion: Radians(r.insertion_angle.value.to_radians()),
                    specific_length: MPerG(1.0 / (root_density * area).max(1e-12)),
                    gravitropism: r.gravitropism,
                    secondary_factor: r
                        .secondary_factor
                        .points
                        .iter()
                        .map(|&(x, f)| (x * CM, f))
                        .collect(),
                }
            })
            .collect();
        Self {
            carbon: CarbonParams::from_species(s),
            temperature: Temperature {
                cardinal,
                growth: GrowthTemperature::from_species(s),
            },
            leaf: LeafParams::from_species(s, t),
            thresholds: Thresholds::from_species(s),
            variation: VariationParams::new(s),
            allometry: Allometry {
                stem_density: GPerM3(c.stem_density.value),
                root_density: GPerM3(root_density),
                bulk_density: GPerM3(c.bulk_density.value),
                pod_aspect: c.pod_aspect.value,
                seed_aspect: c.seed_aspect.value,
            },
            absorptance: c.leaf_absorptance.value,
            energy: LeafEnergyParams {
                width: Meters(c.leaf_width.value),
                // Absorbed near infrared per absorbed PAR energy: the solar NIR per PAR
                // energy times the ratio of the absorptances.
                nir_ratio: (1.0 - ps_light::PAR_FRACTION) / ps_light::PAR_FRACTION
                    * c.leaf_nir_absorptance.value
                    / c.leaf_absorptance.value,
                emissivity: c.leaf_emissivity.value,
                abaxial_share: c.abaxial_conductance_share.value,
            },
            capacity_floor: c.leaf_capacity_floor.value,
            ftsw_conductance: c.ftsw_threshold.value,
            ftsw_expansion: c.ftsw_expansion_threshold.value,
            seed_mass: GramsDM(t.seed_mass),
            seed_carbon_fraction: c.seed_carbon_fraction.value,
            seed_mobilizable: c.seed_mobilizable_fraction.value,
            cotyledon_drop_reserve: c.cotyledon_drop_reserve.value,
            cotyledon_days: round(c.cotyledon_lifespan.value),
            germination_ftsw: c.germination_ftsw.value,
            germination_hours: round(c.germination_hours.value),
            main_nodes: round(t.nodes_before_terminal).max(2),
            branch_nodes: round(c.branch_nodes.value).max(1),
            phyllochron: DegreeDays(c.phyllochron.value),
            leaf_expansion: DegreeDays(c.leaf_expansion.value),
            internode_expansion: DegreeDays(c.internode_expansion.value),
            leaf_area: M2(t.leaf_area_potential * CM2),
            unifoliate_area: M2(c.unifoliate_area.value * CM2),
            sla: M2PerG(t.specific_leaf_area * CM2),
            leaf_profile: [
                c.leaf_first_fraction.value,
                c.leaf_last_fraction.value,
                c.branch_leaf_fraction.value,
            ],
            internode_length: Meters(t.internode_length_potential * CM),
            internode_profile: [
                c.internode_first_fraction.value,
                c.internode_last_fraction.value,
                c.branch_internode_fraction.value,
            ],
            stem_specific_length: MPerG(t.specific_stem_length * CM),
            hypocotyl_specific_length: MPerG(c.hypocotyl_specific_length.value * CM),
            petiole: Meters(c.petiole_length.value * CM),
            hypocotyl_above: Meters(c.hypocotyl_above.value * CM),
            branch_angle: Radians(c.branch_angle.value.to_radians()),
            phyllotaxis: Radians(c.phyllotaxis_angle.value.to_radians()),
            shoot_gravitropism: c.shoot_gravitropism.value,
            phototropism_weight: c.phototropism_weight.value,
            phototropism_max_per_day: Radians(c.phototropism_max_per_day.value.to_radians()),
            bud_activation: t.bud_activation,
            bud_reserve_reference: c.bud_reserve_reference.value,
            pod_set_reserve_reference: c.pod_set_reserve_reference.value,
            dominance_nodes: c.apical_dominance_nodes.value.max(1e-9),
            hold_nodes: c.apical_hold_nodes.value.max(0.0),
            shade_light: c.shade_light_threshold.value,
            shade_days: round(c.shade_senescence_days.value),
            shade_rfr: c.shade_rfr_threshold.value,
            shade_dli_onset: MolPerM2Day(c.shade_dli_onset.value),
            shade_dli_full: MolPerM2Day(c.shade_dli_full.value),
            shade_internode: c.shade_internode_factor.value,
            shade_sla: c.shade_sla_factor.value,
            shade_pod_set: c.shade_pod_set_factor.value,
            leaf_lifespan: DegreeDays(t.leaf_lifespan),
            frost: Celsius(c.leaf_frost_kill.value),
            flowers_per_raceme: round(t.flowers_per_raceme).max(1),
            raceme_days: c.raceme_flowering_days.value.max(1.0),
            flower_days: round(c.flower_lifespan.value).max(1),
            pod_set_base: c.pod_set_base.value,
            pod_set_t: [
                c.pod_set_t_min.value,
                c.pod_set_t_opt_low.value,
                c.pod_set_t_opt_high.value,
                c.pod_set_t_max.value,
            ],
            drought_abortion_ftsw: c.drought_abortion_ftsw.value,
            seeds_per_pod: round(t.seeds_per_pod).max(1),
            ovule_set: c.ovule_set.value,
            pod_wall: GramsDM(c.pod_wall_mass.value),
            pod_expansion: DegreeDays(c.pod_expansion.value),
            seed_fill_delay: DegreeDays(c.seed_fill_delay.value),
            seed_fill: DegreeDays(c.seed_fill.value),
            seed_potential: GramsDM(t.seed_mass),
            nitrogen: NitrogenParams::new(s, t),
            indeterminate: t.habit >= 0.5,
            climbing: ClimbingParams::new(s),
            grass: s.grass.as_ref().map(GrassParams::new),
            root_segment: Meters(c.root_segment_length.value * CM),
            roots,
            thickening: ThickeningParams::new(s),
            feedback_onset: c.reserve_feedback_onset.value,
            feedback_full: c.reserve_feedback_full.value,
            uptake_rate: M2PerHour(c.root_uptake_rate.value),
            uptake_max: KgPerMH(c.root_uptake_max.value),
        }
    }

    /// Size factor of the trifoliate at main-stem rank `r` (1 = first trifoliate) of
    /// `n` trifoliates: rising from the first fraction to 1 by rank 3, falling to the
    /// last fraction over the last two ranks (research review placeholder profile).
    #[must_use]
    pub fn leaf_factor(&self, r: u32, n: u32) -> f64 {
        let [first, last, _] = self.leaf_profile;
        let rf = f64::from(r.saturating_sub(1));
        let rise = first + (1.0 - first) * (rf / 2.0).min(1.0);
        let fall = last + (1.0 - last) * (f64::from(n.saturating_sub(r)) / 2.0).min(1.0);
        rise.min(fall)
    }

    /// Length factor of the main-stem internode at node `k` (1 = epicotyl) of `n`:
    /// 1 for the epicotyl, then linear from the first to the last fraction.
    #[must_use]
    pub fn internode_factor(&self, k: u32, n: u32) -> f64 {
        let [first, last, _] = self.internode_profile;
        if k <= 1 || n <= 2 {
            return 1.0;
        }
        let x = f64::from(k - 2) / f64::from(n - 2);
        first + (last - first) * x
    }

    /// Length factor of main-stem internode `k` of `n`: [`HerbParams::internode_factor`]
    /// for a herb; for a vine, rising from the first to the last fraction over the
    /// climbing ramp and level after it (decision 0021).
    #[must_use]
    pub fn main_internode_factor(&self, k: u32, n: u32) -> f64 {
        if self.indeterminate {
            let r = self.climbing.ramp_nodes + 2;
            self.internode_factor(k.min(r), r)
        } else {
            self.internode_factor(k, n)
        }
    }

    /// Shade multiplier for `max`. The response is the stronger of two signals
    /// (§3.4, decision 0018): red:far-red `rfr` (1 at the threshold, linear to `max`
    /// at the proxy's floor of 0.2) and low light, `dli` the daily absorbed PAR per
    /// leaf area (1 at the onset, linear to `max` at the full value).
    #[must_use]
    pub fn shade_factor(&self, signal: ShadeSignal, max: f64) -> f64 {
        let span = (self.shade_rfr - 0.2).max(1e-9);
        let x_rfr = ((self.shade_rfr - signal.rfr) / span).clamp(0.0, 1.0);
        let x_light = signal.dli.map_or(0.0, |d| {
            let span = (self.shade_dli_onset.0 - self.shade_dli_full.0).max(1e-9);
            ((self.shade_dli_onset.0 - d.0) / span).clamp(0.0, 1.0)
        });
        1.0 + (max - 1.0) * x_rfr.max(x_light)
    }

    /// Sink feedback on leaf capacity (decision 0013): 1 below the onset reserve
    /// concentration, falling linearly to 0 at the full value.
    #[must_use]
    pub fn sink_feedback(&self, concentration: f64) -> f64 {
        let span = (self.feedback_full - self.feedback_onset).max(1e-9);
        (1.0 - (concentration - self.feedback_onset) / span).clamp(0.0, 1.0)
    }

    /// Pod-set temperature factor at daily mean `t`: quadratic ramps to a plateau
    /// (CROPGRO's response, research review item 19).
    #[must_use]
    pub fn pod_set_temperature(&self, t: f64) -> f64 {
        let [lo, o1, o2, hi] = self.pod_set_t;
        if t <= lo || t >= hi {
            0.0
        } else if t < o1 {
            let x = (o1 - t) / (o1 - lo);
            1.0 - x * x
        } else if t > o2 {
            let x = (t - o2) / (hi - o2);
            1.0 - x * x
        } else {
            1.0
        }
    }
}
