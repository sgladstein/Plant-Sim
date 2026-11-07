//! The plant's water network (realism plan R3b, decision 0044): organs are nodes, the
//! organ tree gives the axial edges, and each root organ links radially to every soil
//! cell it runs through. Leaves draw their hour's transpiration; the soil cells are
//! fixed potentials for the hour. One tree solve gives every organ's water potential
//! and every root's flow to or from each of its cells.
//!
//! The simplified level (R3c, decision 0045; Couvreur et al. 2012) solves the same
//! network once a day with the soil equally wet everywhere and keeps its summary: the
//! root system's conductance, each root cell's standard uptake fraction and each
//! organ's potential drop per unit of transpiration. Each hour it then costs one pass
//! over the root cells and one over the organs.
//!
//! Model card: docs/models/plant-hydraulics.md.

use ps_graph::{OrganId, OrganKind};
use ps_physio::hydraulics::{self, Edge, GRAVITY_MPA_PER_M, KG_PER_HOUR_PER_M3_PER_S, NodeTerms};
use ps_soil::SoilGrid;
use ps_species::RootOrigin;
use ps_units::{KgPerHourMPa, KgWater, MPa, Meters};
use serde::{Deserialize, Serialize};

use crate::Plant;

/// A root organ's length in one soil cell: one radial link of the network.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RootSpan {
    /// The root organ's id.
    pub organ: u32,
    /// The soil cell.
    pub cell: usize,
    /// Root length in that cell (all the roots a cohort stands for).
    pub length: Meters,
}

/// What a plant's network asks of each soil cell this hour, for the soil's joint
/// uptake.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WaterFlows {
    /// The leaves' transpiration this hour.
    pub demand: KgWater,
    /// Net flow out of each root cell, sorted by cell: positive is uptake, negative is
    /// water the roots lose to dry soil (hydraulic redistribution).
    pub flows: Vec<(usize, KgWater)>,
}

/// Which form of the water network a plant runs (R3c, decision 0045). Part of the
/// plant's state, so of the hash; changed only by a logged action or the scenario.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HydraulicsLevel {
    /// The full network solve every hour (R3b).
    #[default]
    Detailed,
    /// The daily summary with Couvreur's uptake every hour (R3c).
    Simplified,
}

/// The daily summary of a plant's water network for the simplified level, from one
/// solve with the soil equally wet everywhere and the transpiration shared by leaf area
/// (Couvreur et al. 2012). Rebuilt every day at both levels, with the root cells.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WaterSummary {
    /// The root system's conductance from the soil to the collar, `K_rs`, kg h⁻¹
    /// MPa⁻¹; 0 without a working root link.
    pub k_rs: f64,
    /// Each root cell's standard uptake fraction, `SUF`, sorted by cell (sums to 1).
    pub suf: Vec<(usize, f64)>,
    /// The standard uptake fractions summed by soil layer, top first.
    pub layers: Vec<f64>,
    /// Mean depth of uptake below the surface, m: the cells' center depths weighted by
    /// their standard uptake fractions.
    pub depth: f64,
    /// Each organ's drop in total potential below the root zone's per kg h⁻¹ of the
    /// plant's transpiration (megapascals per kg h⁻¹), by organ index.
    pub drop: Vec<f64>,
}

/// Conductance given to edges that carry no flow (buds, flowers, pods, seeds,
/// cotyledons, nodules, tendrils), and the floor of every edge, kg h⁻¹ MPa⁻¹: only
/// their potential matters, and it follows the parent's.
const PASSIVE_EDGE: f64 = 1.0;
const EDGE_FLOOR: f64 = 1e-9;
/// Shortest length an edge is computed over, m (a primordium of zero length).
const MIN_EDGE_LENGTH: f64 = 1e-3;
/// Steps of the plant's stomatal factor: plants whose factors round to the same step
/// share one class table in a plot (decision 0044).
pub const STOMATAL_STEPS: f64 = 50.0;

/// Memo of root segments' axial conductivity, m⁴ s⁻¹ MPa⁻¹ per m of root, by organ
/// index: the bits of its inputs (distance from the tip, mature value, tip value,
/// maturation length) and the conductivity, as last computed. A
/// segment's distance from its axis's tip changes only when the axis grows, so most
/// hours reuse the `pow` (bit-identical: the same inputs give the same value).
/// Scratch: it compares equal to any other memo and is not saved.
#[derive(Clone, Debug, Default)]
pub(crate) struct AxialMemo(Vec<Option<([u64; 4], f64)>>);

impl PartialEq for AxialMemo {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Conductance of the soil around a root, per metre of root, m² s⁻¹ MPa⁻¹ (R3c,
/// decision 0045; Gardner 1960): steady radial flow through a cylinder of soil from
/// the root's surface out to half the distance between roots,
/// `2π K(θ) / (ρ g ln(r_b / r))` with `r_b = 1 / √(π RLD)` and at least `2r`.
/// `conductivity` is the soil's `K(θ)` over `ρ g`, m² s⁻¹ MPa⁻¹; `root_density` the
/// root length per volume of soil, m m⁻³.
#[must_use]
pub fn rhizosphere_conductance(conductivity: f64, root_radius: f64, root_density: f64) -> f64 {
    if conductivity <= 0.0 || root_radius <= 0.0 || root_density <= 0.0 {
        return 0.0;
    }
    let half_spacing =
        (1.0 / libm::sqrt(core::f64::consts::PI * root_density)).max(2.0 * root_radius);
    2.0 * core::f64::consts::PI * conductivity / libm::log(half_spacing / root_radius)
}

/// Each cell's matric potential and total potential (with gravity at the cell's
/// center) in megapascals, and its soil conductivity `K(θ) / ρ g` (m² s⁻¹ MPa⁻¹; R3c),
/// in the order given.
fn cell_terms(soil: &SoilGrid, cells: impl Iterator<Item = usize>) -> Vec<(f64, f64, f64)> {
    let retention = soil.retention();
    let k_sat = soil.texture().hydraulics().saturated_conductivity / GRAVITY_MPA_PER_M;
    let edge = soil.cell_size().0;
    cells
        .map(|c| {
            let (psi, k_rel) = retention.psi_and_conductivity(soil.theta(c));
            let psi = psi.to_mpa().0;
            #[allow(clippy::cast_precision_loss)] // a layer index
            let z = -(soil.cell_of(c).z as f64 + 0.5) * edge;
            (psi, psi + GRAVITY_MPA_PER_M * z, k_sat * k_rel)
        })
        .collect()
}

/// [`cell_terms`] without the conductivity, for the simplified level's hour (two
/// powers a cell instead of three; the same potentials).
fn cell_potentials(soil: &SoilGrid, cells: impl Iterator<Item = usize>) -> Vec<(f64, f64)> {
    let retention = soil.retention();
    let edge = soil.cell_size().0;
    cells
        .map(|c| {
            let psi = retention.psi(soil.theta(c)).to_mpa().0;
            #[allow(clippy::cast_precision_loss)] // a layer index
            let z = -(soil.cell_of(c).z as f64 + 0.5) * edge;
            (psi, psi + GRAVITY_MPA_PER_M * z)
        })
        .collect()
}

impl Plant {
    /// Height of an organ's middle, m (0 at the soil surface, negative below).
    fn organ_height(&self, i: usize) -> f64 {
        self.positions
            .get(i)
            .map_or(self.collar_position[2], |p| 0.5 * (p.0[2] + p.1[2]))
    }

    /// Distance of each main-axis root segment's base from its axis's tip, m, and
    /// whether the axis is the tap root, by organ index; `None` for other organs.
    fn main_root_positions(&self) -> Vec<Option<(f64, bool)>> {
        let mut out = vec![None; self.arena.len()];
        for ax in &self.root_axes {
            let rp = &self.params.roots[ax.kind];
            if rp.order > 0 {
                continue;
            }
            let tap = rp.origin == RootOrigin::Radicle;
            for &(seg, start) in &ax.segments {
                if let Some(slot) = out.get_mut(seg.index()) {
                    *slot = Some(((ax.length - start).max(0.0), tap));
                }
            }
        }
        out
    }

    /// Axial conductance of the edge from organ `i`'s parent to it, kg h⁻¹ MPa⁻¹.
    fn edge_conductance(
        &self,
        i: usize,
        main: &[Option<(f64, bool)>],
        memo: &mut AxialMemo,
    ) -> f64 {
        let hp = &self.params.hydraulics;
        let a = &self.arena;
        let size = a.size[i];
        let k = match a.kind[i] {
            OrganKind::Root => {
                let m = self.meta[i].multiplicity.max(1.0);
                let each = (size.length.0 / m).max(MIN_EDGE_LENGTH);
                let kx = match main.get(i).copied().flatten() {
                    Some((d, tap)) => {
                        let base = if tap {
                            hp.root_axial_tap
                        } else {
                            hp.root_axial_basal
                        };
                        let key = [
                            d.to_bits(),
                            base.to_bits(),
                            hp.root_axial_tip.to_bits(),
                            hp.root_axial_maturation.to_bits(),
                        ];
                        match memo.0.get(i).copied().flatten() {
                            Some((k, kx)) if k == key => kx,
                            _ => {
                                let s = (d / hp.root_axial_maturation.max(1e-9)).clamp(0.0, 1.0);
                                let kx = hp.root_axial_tip * libm::pow(base / hp.root_axial_tip, s);
                                if memo.0.len() <= i {
                                    memo.0.resize(i + 1, None);
                                }
                                memo.0[i] = Some((key, kx));
                                kx
                            }
                        }
                    }
                    None => hp.root_axial_tip,
                };
                m * kx * self.xylem_gain(i) / each * KG_PER_HOUR_PER_M3_PER_S
            }
            OrganKind::Internode => {
                let r = size.radius.0;
                let l = size.length.0.max(MIN_EDGE_LENGTH);
                hp.stem_conductivity * hp.xylem_fraction * core::f64::consts::PI * r * r / l
                    * 3600.0
            }
            OrganKind::Leaf => hp.leaf_conductance_kg() * size.area.0,
            _ => PASSIVE_EDGE,
        };
        k.max(EDGE_FLOOR)
    }

    /// The network's axial edges, one per parent link of the organ tree.
    fn axial_edges(&mut self) -> Vec<Edge> {
        let main = self.main_root_positions();
        // Sized up front: the filtered edge iterator has no length hint, and growing
        // the list by doubling every hour showed in profiles.
        let mut memo = core::mem::take(&mut self.axial_memo);
        let mut edges: Vec<Edge> = Vec::with_capacity(self.arena.len());
        edges.extend(self.arena.edges().map(|(p, c)| Edge {
            parent: p.index(),
            child: c.index(),
            conductance: KgPerHourMPa(self.edge_conductance(c.index(), &main, &mut memo)),
        }));
        self.axial_memo = memo;
        edges
    }

    /// Factor on a root's axial conductance from its secondary growth: secondary xylem
    /// adds conducting area, so the conductance scales with the cross-section,
    /// `(r / r_primary)²`, which is the organ's structural over its primary mass (the
    /// allometry keeps the length on the primary mass).
    fn xylem_gain(&self, i: usize) -> f64 {
        let id = OrganId(u32::try_from(i).unwrap_or(u32::MAX));
        let primary = self.arena.primary(id).0;
        if primary > 0.0 {
            (self.arena.structural(id).0 / primary).max(1.0)
        } else {
            1.0
        }
    }

    /// Radial conductivity of root organ `i`, m s⁻¹ MPa⁻¹: the young value, falling
    /// toward the thickened value with the organ's secondary share of its structure.
    fn radial_conductivity(&self, i: usize) -> f64 {
        let hp = &self.params.hydraulics;
        let id = OrganId(u32::try_from(i).unwrap_or(u32::MAX));
        let total = self.arena.structural(id).0;
        let s = if total > 0.0 {
            (1.0 - self.arena.primary(id).0 / total).clamp(0.0, 1.0)
        } else {
            0.0
        };
        hp.root_radial_young + (hp.root_radial_thickened - hp.root_radial_young) * s
    }

    /// The network's radial links (R3b): one per root span with a working cell, as
    /// `(organ, cell slot, conductance)`, and each organ's summed terms. Each link is
    /// the root's radial conductance in series with the soil's around it (R3c), the
    /// soil's from the cell's conductivity and this plant's root length density there.
    fn radial_links(
        &self,
        soil: &SoilGrid,
        cells: &[(usize, Meters)],
        soil_terms: &[(f64, f64, f64)],
    ) -> (Vec<NodeTerms>, Vec<(usize, usize, f64)>) {
        let n = self.arena.len();
        let slot = |c: usize| cells.binary_search_by_key(&c, |p| p.0).ok();
        let edge = soil.cell_size().0;
        let volume = edge * edge * edge;
        let mut nodes = vec![NodeTerms::default(); n];
        let mut links: Vec<(usize, usize, f64)> = Vec::with_capacity(self.root_spans.len());
        let mut lp_of = vec![f64::NAN; n];
        for s in &self.root_spans {
            let o = s.organ as usize;
            let Some(j) = slot(s.cell) else { continue };
            if o >= n || !self.arena.alive[o] {
                continue;
            }
            if lp_of[o].is_nan() {
                lp_of[o] = self.radial_conductivity(o);
            }
            let r = self.arena.size[o].radius.0;
            let root = lp_of[o] * 2.0 * core::f64::consts::PI * r;
            let around = rhizosphere_conductance(soil_terms[j].2, r, cells[j].1.0 / volume);
            let k = if root + around > 0.0 {
                root * around / (root + around) * s.length.0 * KG_PER_HOUR_PER_M3_PER_S
            } else {
                0.0
            };
            if k <= 0.0 {
                continue;
            }
            nodes[o].radial.0 += k;
            nodes[o].source.0 += k * soil_terms[j].1;
            links.push((o, j, k));
        }
        (nodes, links)
    }

    /// Hourly step 7 (R3b, R3c): at the plant's level, moves this hour's leaf
    /// transpiration from [`Plant::water_demand_with`] through the water network against
    /// the soil as it stands, keeps every organ's water potential for the next hour's
    /// stomata and expansion, and returns what the roots ask of each cell. The detailed
    /// level solves the network; the simplified level uses the day's summary.
    ///
    /// Without a working root link (no roots in the soil yet, or every root cell
    /// air-dry), the plant takes nothing and every organ reads the root zone's soil
    /// potential.
    pub fn water_flows(&mut self, soil: &SoilGrid) -> WaterFlows {
        match self.hydraulics_level {
            HydraulicsLevel::Detailed => self.water_flows_detailed(soil),
            HydraulicsLevel::Simplified => self.water_flows_simplified(soil),
        }
    }

    fn water_flows_detailed(&mut self, soil: &SoilGrid) -> WaterFlows {
        let n = self.arena.len();
        self.leaf_water.resize(n, KgWater(0.0));
        let demand = KgWater(self.leaf_water.iter().map(|w| w.0).sum());
        let cells = self.roots.cells();
        let soil_terms = cell_terms(soil, cells.iter().map(|c| c.0));
        let (mut nodes, links) = self.radial_links(soil, cells, &soil_terms);
        for (i, w) in self.leaf_water.iter().enumerate() {
            nodes[i].sink = *w;
        }
        let edges = self.axial_edges();
        // The root zone's soil potential, weighted by radial conductance: what the
        // leaves read at night, and the fallback.
        let (mut kw, mut ksum) = (0.0, 0.0);
        for &(_, j, k) in &links {
            kw += k * soil_terms[j].0;
            ksum += k;
        }
        let zone = if ksum > 0.0 {
            kw / ksum
        } else if soil_terms.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)] // a cell count
            let len = soil_terms.len() as f64;
            soil_terms.iter().map(|t| t.0).sum::<f64>() / len
        };
        self.root_zone_potential = MPa(zone);
        let solved = if links.is_empty() {
            None
        } else {
            hydraulics::solve_tree(&nodes, &edges)
        };
        let Some(total) = solved else {
            self.water_potential = vec![MPa(zone); n];
            return WaterFlows {
                demand,
                flows: Vec::new(),
            };
        };
        let mut per_cell = vec![0.0_f64; cells.len()];
        for &(o, j, k) in &links {
            per_cell[j] += k * (soil_terms[j].1 - total[o].0);
        }
        // Every node balance holds to the round-off of its conductance-times-
        // potential terms, axial and radial, so the network closes relative to their
        // size (or the demand's, if larger), not to the net or gross flow, which at
        // night in even soil can be far smaller.
        debug_assert!(
            {
                let net: f64 = per_cell.iter().sum();
                let radial = links
                    .iter()
                    .map(|&(o, j, k)| k * (soil_terms[j].1.abs() + total[o].0.abs()));
                let axial = edges
                    .iter()
                    .map(|e| e.conductance.0 * (total[e.parent].0.abs() + total[e.child].0.abs()));
                let terms = radial.chain(axial).sum::<f64>();
                (net - demand.0).abs() <= 1e-9 * terms.max(demand.0.abs()).max(1e-300)
            },
            "network flows {} against demand {}",
            per_cell.iter().sum::<f64>(),
            demand.0
        );
        self.water_potential = (0..n)
            .map(|i| MPa(total[i].0 - GRAVITY_MPA_PER_M * self.organ_height(i)))
            .collect();
        WaterFlows {
            demand,
            flows: cells
                .iter()
                .zip(&per_cell)
                .map(|(&(c, _), &q)| (c, KgWater(q)))
                .collect(),
        }
    }

    /// The organ the root system hangs from: the parent of the tap root's first
    /// segment (else of the first axis's).
    fn collar_organ(&self) -> Option<usize> {
        let first = |ax: &crate::RootAxis| ax.segments.first().map(|s| s.0);
        let tap = self
            .root_axes
            .iter()
            .find(|ax| self.params.roots[ax.kind].origin == RootOrigin::Radicle)
            .and_then(first);
        let seg = tap.or_else(|| self.root_axes.iter().find_map(first))?;
        self.arena.parent[seg.index()].map(OrganId::index)
    }

    /// The day's summary of the water network for the simplified level (R3c): one
    /// solve with every root cell at the same total potential (0) and 1 kg h⁻¹ of
    /// transpiration shared by leaf area (or drawn at the collar before there are
    /// leaves). Called with the root cells' rebuild, at both levels.
    pub(crate) fn summarize_water(&mut self, soil: &SoilGrid) -> WaterSummary {
        let n = self.arena.len();
        let cells = self.roots.cells();
        let mut terms = cell_terms(soil, cells.iter().map(|c| c.0));
        for t in &mut terms {
            t.1 = 0.0;
        }
        let (mut nodes, links) = self.radial_links(soil, cells, &terms);
        let Some(collar) = self.collar_organ().filter(|&c| c < n) else {
            return WaterSummary::default();
        };
        if links.is_empty() {
            return WaterSummary::default();
        }
        let leaves: Vec<(usize, f64)> = self
            .arena
            .alive_of(OrganKind::Leaf)
            .map(|id| (id.index(), self.arena.size[id.index()].area.0))
            .filter(|&(_, a)| a > 0.0)
            .collect();
        let area: f64 = leaves.iter().map(|l| l.1).sum();
        if area > 0.0 {
            for &(i, a) in &leaves {
                nodes[i].sink = KgWater(a / area);
            }
        } else {
            nodes[collar].sink = KgWater(1.0);
        }
        let edges = self.axial_edges();
        let Some(total) = hydraulics::solve_tree(&nodes, &edges) else {
            return WaterSummary::default();
        };
        let at_collar = -total[collar].0;
        if at_collar <= 0.0 {
            return WaterSummary::default();
        }
        let mut per_cell = vec![0.0_f64; cells.len()];
        for &(o, j, k) in &links {
            per_cell[j] -= k * total[o].0;
        }
        let taken: f64 = per_cell.iter().sum();
        let (_, _, nz) = soil.dims();
        let edge = soil.cell_size().0;
        let mut layers = vec![0.0; nz];
        let mut suf = Vec::with_capacity(cells.len());
        let mut depth = 0.0;
        if taken > 0.0 {
            for (&(c, _), &q) in cells.iter().zip(&per_cell) {
                let f = q / taken;
                let z = soil.cell_of(c).z;
                suf.push((c, f));
                #[allow(clippy::cast_precision_loss)] // a layer index
                let centre = (z as f64 + 0.5) * edge;
                depth += f * centre;
                if let Some(l) = layers.get_mut(z) {
                    *l += f;
                }
            }
        }
        WaterSummary {
            k_rs: 1.0 / at_collar,
            suf,
            layers,
            depth,
            drop: total.iter().map(|t| -t.0).collect(),
        }
    }

    /// Hourly step 7 at the simplified level (R3c, Couvreur et al. 2012): the root
    /// zone's effective potential `ψ_seq = Σ SUF_c ψ_c` (total potentials), uptake per
    /// cell `E SUF_c + K_comp SUF_c (ψ_c − ψ_seq)`, each organ at `ψ_seq` less its drop
    /// per unit transpiration times `E`, and each leaf at its parent's potential less
    /// its own transpiration over its own conductance.
    fn water_flows_simplified(&mut self, soil: &SoilGrid) -> WaterFlows {
        let n = self.arena.len();
        self.leaf_water.resize(n, KgWater(0.0));
        let demand = KgWater(self.leaf_water.iter().map(|w| w.0).sum());
        let summary = &self.water_summary;
        let terms = cell_potentials(soil, summary.suf.iter().map(|c| c.0));
        let (mut seq, mut zone) = (0.0, 0.0);
        for (&(_, f), t) in summary.suf.iter().zip(&terms) {
            seq += f * t.1;
            zone += f * t.0;
        }
        if summary.k_rs <= 0.0 || summary.suf.is_empty() {
            let all = cell_potentials(soil, self.roots.cells().iter().map(|c| c.0));
            #[allow(clippy::cast_precision_loss)] // a cell count
            let mean = if all.is_empty() {
                0.0
            } else {
                all.iter().map(|t| t.0).sum::<f64>() / all.len() as f64
            };
            self.root_zone_potential = MPa(mean);
            self.water_potential = vec![MPa(mean); n];
            return WaterFlows {
                demand,
                flows: Vec::new(),
            };
        }
        self.root_zone_potential = MPa(zone);
        let e = demand.0;
        let k_comp = self.params.hydraulics.compensation * summary.k_rs;
        let flows: Vec<(usize, KgWater)> = summary
            .suf
            .iter()
            .zip(&terms)
            .map(|(&(c, f), t)| (c, KgWater(e * f + k_comp * f * (t.1 - seq))))
            .collect();
        debug_assert!(
            {
                let net: f64 = flows.iter().map(|q| q.1.0).sum();
                let scale = summary
                    .suf
                    .iter()
                    .zip(&terms)
                    .map(|(&(_, f), t)| f * (e + k_comp * (t.1.abs() + seq.abs())))
                    .sum::<f64>();
                (net - e).abs() <= 1e-9 * scale.max(e).max(1e-300)
            },
            "simplified flows against demand {e}"
        );
        // Each organ's drop per unit transpiration; an organ born since the summary
        // takes its parent's (a parent's index is below its child's).
        let mut drop = summary.drop.clone();
        drop.truncate(n);
        for i in drop.len()..n {
            let d =
                self.arena.parent[i].map_or(0.0, |p| drop.get(p.index()).copied().unwrap_or(0.0));
            drop.push(d);
        }
        let leaf_k = self.params.hydraulics.leaf_conductance_kg();
        let total: Vec<f64> = (0..n)
            .map(|i| {
                let parent = self.arena.parent[i].map(OrganId::index);
                match (self.arena.kind[i], parent) {
                    (OrganKind::Leaf, Some(p)) => {
                        let k = (leaf_k * self.arena.size[i].area.0).max(EDGE_FLOOR);
                        seq - drop[p] * e - self.leaf_water[i].0 / k
                    }
                    _ => seq - drop[i] * e,
                }
            })
            .collect();
        self.water_potential = (0..n)
            .map(|i| MPa(total[i] - GRAVITY_MPA_PER_M * self.organ_height(i)))
            .collect();
        WaterFlows { demand, flows }
    }

    /// The day's water summary (R3c), for the lens and the daily outputs.
    #[must_use]
    pub fn water_summary_today(&self) -> &WaterSummary {
        &self.water_summary
    }

    /// Leaf-area-weighted mean water potential and turgor of the living leaves after
    /// the last solve; `None` before the first solve or without leaves.
    #[must_use]
    pub fn leaf_water_status(&self) -> Option<(MPa, f64)> {
        let hp = &self.params.hydraulics;
        let (mut area, mut psi, mut turg) = (0.0, 0.0, 0.0);
        for id in self.arena.alive_of(OrganKind::Leaf) {
            let i = id.index();
            let a = self.arena.size[i].area.0;
            let Some(p) = self.water_potential.get(i) else {
                continue;
            };
            if a <= 0.0 {
                continue;
            }
            area += a;
            psi += a * p.0;
            turg += a * hydraulics::turgor(*p, hp);
        }
        (area > 0.0).then(|| (MPa(psi / area), turg / area))
    }

    /// Each organ's expansion factor this hour from its own water potential after the
    /// last solve (turgor; R3b), by organ index: 1 for organs that do not expand by
    /// turgor here and for organs not yet solved.
    #[must_use]
    pub fn expansion_factors(&self) -> Vec<f64> {
        let hp = &self.params.hydraulics;
        (0..self.arena.len())
            .map(|i| {
                if ps_physio::allocation::is_expanding_kind(self.arena.kind[i]) {
                    // A leaf grows at its base, fed by the xylem before the lamina's
                    // own resistance: it reads its parent node's potential.
                    let at = match self.arena.kind[i] {
                        OrganKind::Leaf => self.arena.parent[i].map_or(i, OrganId::index),
                        _ => i,
                    };
                    self.water_potential
                        .get(at)
                        .map_or(1.0, |p| hydraulics::turgor(*p, hp))
                } else {
                    1.0
                }
            })
            .collect()
    }

    /// Each organ's relative turgor after the last solve, by organ index (0 at the
    /// turgor loss point, 1 at full turgor; 1 before the first solve).
    #[must_use]
    pub fn organ_turgor(&self) -> Vec<f64> {
        let hp = &self.params.hydraulics;
        (0..self.arena.len())
            .map(|i| {
                self.water_potential
                    .get(i)
                    .map_or(1.0, |p| hydraulics::turgor(*p, hp))
            })
            .collect()
    }

    /// Each organ's water potential after the last hour's solve, by organ index
    /// (pressure, without the gravity term); empty before the first.
    #[must_use]
    pub fn water_potentials(&self) -> &[MPa] {
        &self.water_potential
    }

    /// The root zone's soil water potential at the last solve, weighted by each root
    /// cell's radial conductance.
    #[must_use]
    pub fn root_zone_potential(&self) -> MPa {
        self.root_zone_potential
    }
}

/// The plant's stomatal factor from its leaves' mean water potential (Tuzet), rounded
/// to [`STOMATAL_STEPS`].
#[must_use]
pub fn stomatal_step(psi_leaf: MPa, hp: &ps_physio::HydraulicParams) -> f64 {
    let f = hydraulics::stomatal_factor(psi_leaf, hp);
    libm::round(f * STOMATAL_STEPS) / STOMATAL_STEPS
}
