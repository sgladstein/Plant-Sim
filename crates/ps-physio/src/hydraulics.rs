//! Plant hydraulics (realism plan R3b, decision 0044): the stomatal and turgor factors
//! of a water potential, and the tree solve of a plant's water network.
//!
//! The factors are plain functions of a water potential and the species parameters, so
//! a leaf, a leaf class or a whole plant can call them (fidelity-levels.md §2.1).
//!
//! Model card: docs/models/plant-hydraulics.md.

use ps_units::{KgPerHourMPa, KgWater, MPa};
use serde::{Deserialize, Serialize};

/// `ρ g`: water potential per meter of height, in megapascals per meter (`ψ_total = ψ_p + ρ g z`).
pub const GRAVITY_MPA_PER_M: f64 = 9.806_65e-3;
/// Kilograms of water per hour in a flow of one cubic meter per second.
pub const KG_PER_HOUR_PER_M3_PER_S: f64 = 1000.0 * 3600.0;
/// Kilograms of water per hour in a flow of one millimole per second.
pub const KG_PER_HOUR_PER_MMOL_PER_S: f64 = 18.015e-6 * 3600.0;

/// Species hydraulic parameters (species file constants; R3b spec §4.4).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HydraulicParams {
    /// Radial conductivity of young and fine roots, `Lp_r`, m s⁻¹ MPa⁻¹.
    pub root_radial_young: f64,
    /// Radial conductivity of fully thickened root, m s⁻¹ MPa⁻¹.
    pub root_radial_thickened: f64,
    /// Axial conductance of one root at its tip, and of each lateral and fine root,
    /// `K_x`, m⁴ s⁻¹ MPa⁻¹.
    pub root_axial_tip: f64,
    /// Axial conductance of the tap root's mature part, m⁴ s⁻¹ MPa⁻¹.
    pub root_axial_tap: f64,
    /// Axial conductance of the other main axes' mature part (basal and
    /// hypocotyl-borne roots), m⁴ s⁻¹ MPa⁻¹.
    pub root_axial_basal: f64,
    /// Distance from a main axis's tip over which its axial conductance rises from the
    /// tip's to the mature value, m.
    pub root_axial_maturation: f64,
    /// Stem xylem conductivity `K_s`, kg m⁻¹ s⁻¹ MPa⁻¹ (per conducting area).
    pub stem_conductivity: f64,
    /// Share of a stem's cross-section that conducts.
    pub xylem_fraction: f64,
    /// Leaf hydraulic conductance per leaf area, petiole and blade together, `K_leaf`,
    /// mmol m⁻² s⁻¹ MPa⁻¹.
    pub leaf_conductance: f64,
    /// Tuzet sensitivity `s_f`, MPa⁻¹.
    pub tuzet_sensitivity: f64,
    /// Tuzet reference potential `ψ_f`, where the factor is about one half.
    pub tuzet_reference: MPa,
    /// Turgor loss point `ψ_tlp`: expansion stops and a leaf wilts.
    pub turgor_loss: MPa,
    /// Water potential at full turgor for expansion, `ψ_0`.
    pub turgor_full: MPa,
    /// Couvreur's compensation conductance over the root system's, `K_comp / K_rs`
    /// (the simplified level, R3c).
    pub compensation: f64,
}

impl HydraulicParams {
    /// Placeholder values for tests and examples (the R3b spec's bean values); species
    /// files set their own.
    #[must_use]
    pub fn bean() -> Self {
        HydraulicParams {
            root_radial_young: 2e-7,
            root_radial_thickened: 5e-8,
            root_axial_tip: 2e-11,
            root_axial_tap: 2e-9,
            root_axial_basal: 1e-10,
            root_axial_maturation: 0.2,
            stem_conductivity: 9.8,
            xylem_fraction: 0.15,
            leaf_conductance: 8.7,
            tuzet_sensitivity: 4.0,
            tuzet_reference: MPa(-1.0),
            turgor_loss: MPa(-0.9),
            turgor_full: MPa(-0.2),
            compensation: 1.0,
        }
    }

    /// Leaf conductance per leaf area in kg h⁻¹ m⁻² MPa⁻¹.
    #[must_use]
    pub fn leaf_conductance_kg(&self) -> f64 {
        self.leaf_conductance * KG_PER_HOUR_PER_MMOL_PER_S
    }
}

/// Tuzet et al. (2003) stomatal factor on the Medlyn slope from a leaf's water
/// potential: `(1 + e^(s_f ψ_f)) / (1 + e^(s_f (ψ_f − ψ_leaf)))`, 1 at zero potential
/// and about one half at `ψ_f`; at most 1.
#[must_use]
pub fn stomatal_factor(psi_leaf: MPa, p: &HydraulicParams) -> f64 {
    let sf = p.tuzet_sensitivity;
    let f = p.tuzet_reference.0;
    let top = 1.0 + libm::exp(sf * f);
    let bottom = 1.0 + libm::exp(sf * (f - psi_leaf.0));
    (top / bottom).clamp(0.0, 1.0)
}

/// Relative turgor of a tissue at water potential `psi`:
/// `clamp((ψ − ψ_tlp) / (ψ_0 − ψ_tlp), 0, 1)`. It scales expansion (0 at the turgor
/// loss point, 1 at `ψ_0` and wetter) and is how wilted a leaf looks.
#[must_use]
pub fn turgor(psi: MPa, p: &HydraulicParams) -> f64 {
    let span = p.turgor_full.0 - p.turgor_loss.0;
    if span <= 0.0 {
        return if psi.0 >= p.turgor_loss.0 { 1.0 } else { 0.0 };
    }
    ((psi.0 - p.turgor_loss.0) / span).clamp(0.0, 1.0)
}

/// One edge of a hydraulic tree. A parent's index is below its child's (an organ
/// arena's ids grow from the seed outward).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Edge {
    /// Parent node.
    pub parent: usize,
    /// Child node.
    pub child: usize,
    /// Conductance between them.
    pub conductance: KgPerHourMPa,
}

/// A node's links to fixed potentials and its own outflow, for [`solve_tree`].
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct NodeTerms {
    /// Sum of its conductances to fixed potentials (root to soil), `G_i`.
    pub radial: KgPerHourMPa,
    /// `Σ_c k_ic ψ_c` over those links: the flow it would get at zero potential, per
    /// hour.
    pub source: KgWater,
    /// Water it loses per hour (a leaf's transpiration), `E_i`.
    pub sink: KgWater,
}

/// Solves every node's water potential in a forest of hydraulic trees:
/// `Σ_j k_ij (ψ_j − ψ_i) + G_i ψ_s − G_i ψ_i − E_i = 0` (Kirchhoff, no storage). One
/// elimination from the highest index down and one substitution back up, O(n), as the
/// phloem transport solve (`allocation::transport`).
///
/// Nodes that touch nothing (no edge, no radial link, no sink) read 0. Returns `None`
/// when a tree that carries flow has no link to a fixed potential (its potential is
/// undefined).
#[must_use]
pub fn solve_tree(nodes: &[NodeTerms], edges: &[Edge]) -> Option<Vec<MPa>> {
    let n = nodes.len();
    let mut diag: Vec<f64> = nodes.iter().map(|t| t.radial.0).collect();
    let mut rhs: Vec<f64> = nodes.iter().map(|t| t.source.0 - t.sink.0).collect();
    let mut is_child = vec![false; n];
    let mut order: Vec<(usize, usize, f64)> = Vec::with_capacity(edges.len());
    for e in edges {
        debug_assert!(
            e.parent < e.child,
            "parent {} after child {}",
            e.parent,
            e.child
        );
        diag[e.parent] += e.conductance.0;
        diag[e.child] += e.conductance.0;
        is_child[e.child] = true;
        order.push((e.child, e.parent, e.conductance.0));
    }
    // A child has one parent, so sorting by child is a total order.
    order.sort_unstable_by_key(|&(ch, _, _)| ch);
    let scale: Vec<f64> = diag.clone();
    for &(ch, pa, k) in order.iter().rev() {
        let f = k / diag[ch];
        diag[pa] -= k * f;
        rhs[pa] += f * rhs[ch];
    }
    // Pushed rather than zeroed and then filled (a zeroed allocation pays a memset of
    // fresh memory every hour); children are set below.
    let mut psi: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        let mut v = 0.0_f64;
        if !(is_child[i] || (scale[i] <= 0.0 && nodes[i].sink.0 == 0.0)) {
            // A tree with no fixed potential ends with a pivot of rounding size.
            if diag[i] <= 1e-12 * scale[i] {
                return None;
            }
            v = rhs[i] / diag[i];
        }
        psi.push(v);
    }
    for &(ch, pa, k) in &order {
        psi[ch] = (rhs[ch] + k * psi[pa]) / diag[ch];
    }
    Some(psi.into_iter().map(MPa).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(radial: f64, source: f64, sink: f64) -> NodeTerms {
        NodeTerms {
            radial: KgPerHourMPa(radial),
            source: KgWater(source),
            sink: KgWater(sink),
        }
    }

    fn edge(parent: usize, child: usize, k: f64) -> Edge {
        Edge {
            parent,
            child,
            conductance: KgPerHourMPa(k),
        }
    }

    #[test]
    fn tuzet_is_one_at_zero_and_half_near_the_reference() {
        let p = HydraulicParams::bean();
        assert!((stomatal_factor(MPa(0.0), &p) - 1.0).abs() < 1e-15);
        let half = stomatal_factor(p.tuzet_reference, &p);
        assert!((half - 0.509).abs() < 1e-3, "{half}");
        let mut last = 1.0;
        for k in 1..=40 {
            let f = stomatal_factor(MPa(-0.05 * f64::from(k)), &p);
            assert!(f < last && f > 0.0, "{f} at step {k}");
            last = f;
        }
        assert!(stomatal_factor(MPa(0.1), &p) <= 1.0);
    }

    #[test]
    fn turgor_runs_from_the_loss_point_to_full() {
        let p = HydraulicParams::bean();
        assert!((turgor(MPa(-0.1), &p) - 1.0).abs() < 1e-15);
        assert!((turgor(MPa(-0.2), &p) - 1.0).abs() < 1e-15);
        assert!((turgor(MPa(-0.9), &p) - 0.0).abs() < 1e-15);
        assert!((turgor(MPa(-1.5), &p) - 0.0).abs() < 1e-15);
        assert!((turgor(MPa(-0.55), &p) - 0.5).abs() < 1e-12);
    }

    /// A root node in soil at `ψ_s` feeding a leaf through one edge: the flow `E` sets
    /// `ψ_root = ψ_s − E / G` and `ψ_leaf = ψ_root − E / k`.
    #[test]
    fn two_nodes_match_the_series_resistances() {
        let (g, k, e, s) = (0.05, 0.08, 0.04, -0.3);
        let psi = solve_tree(&[term(g, g * s, 0.0), term(0.0, 0.0, e)], &[edge(0, 1, k)]).unwrap();
        let root = s - e / g;
        let leaf = root - e / k;
        assert!((psi[0].0 - root).abs() < 1e-12, "{psi:?}");
        assert!((psi[1].0 - leaf).abs() < 1e-12, "{psi:?}");
    }

    /// Two roots in soils of different potential under one stem: each root's radial
    /// flow is `G (ψ_s − ψ)`, the flows add up to the leaf's sink, and a root in dry
    /// enough soil leaks water into it.
    #[test]
    fn flows_balance_and_dry_soil_takes_water_back() {
        // 0 stem base, 1 wet root, 2 dry root, 3 leaf.
        let nodes = [
            term(0.0, 0.0, 0.0),
            term(0.04, 0.04 * -0.1, 0.0),
            term(0.04, 0.04 * -2.0, 0.0),
            term(0.0, 0.0, 0.01),
        ];
        let edges = [edge(0, 1, 5.0), edge(0, 2, 5.0), edge(0, 3, 0.1)];
        let psi = solve_tree(&nodes, &edges).unwrap();
        let wet = 0.04 * (-0.1 - psi[1].0);
        let dry = 0.04 * (-2.0 - psi[2].0);
        assert!(((wet + dry) - 0.01).abs() < 1e-12, "{wet} {dry}");
        assert!(dry < 0.0, "the dry root should leak: {dry}");
        assert!(wet > 0.01);
    }

    #[test]
    fn no_flow_means_soil_potential_everywhere() {
        let nodes = [
            term(0.0, 0.0, 0.0),
            term(0.02, 0.02 * -0.4, 0.0),
            term(0.0, 0.0, 0.0),
        ];
        let psi = solve_tree(&nodes, &[edge(0, 1, 1.0), edge(0, 2, 0.5)]).unwrap();
        for p in &psi {
            assert!((p.0 + 0.4).abs() < 1e-12, "{psi:?}");
        }
    }

    #[test]
    fn a_tree_without_roots_has_no_solution() {
        let nodes = [term(0.0, 0.0, 0.0), term(0.0, 0.0, 0.01)];
        assert!(solve_tree(&nodes, &[edge(0, 1, 0.1)]).is_none());
        // An untouched node alone is fine.
        let psi = solve_tree(&[term(0.0, 0.0, 0.0)], &[]).unwrap();
        assert_eq!(psi[0].0.to_bits(), 0.0_f64.to_bits());
    }
}
