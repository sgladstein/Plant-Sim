//! The plant's water network (realism plan R3b, decision 0044): organs are nodes, the
//! organ tree gives the axial edges, and each root organ links radially to every soil
//! cell it runs through. Leaves draw their hour's transpiration; the soil cells are
//! fixed potentials for the hour. One tree solve gives every organ's water potential
//! and every root's flow to or from each of its cells.
//!
//! Model card: docs/models/plant-hydraulics.md.

use ps_graph::{OrganId, OrganKind};
use ps_physio::hydraulics::{self, Edge, GRAVITY_MPA_PER_M, KG_PER_HOUR_PER_M3_PER_S, NodeTerms};
use ps_soil::{Hydraulics, SoilGrid};
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

/// Rhizosphere drying factor on a root's radial conductance: 1 at and above the
/// wilting point, falling linearly to 0 at air-dry, where the soil next to the root
/// no longer conducts (R3b spec §4.1).
#[must_use]
pub fn rhizosphere_factor(theta: f64, h: &Hydraulics) -> f64 {
    let span = h.wilting_point - h.air_dry;
    if span <= 0.0 {
        return 1.0;
    }
    ((theta - h.air_dry) / span).clamp(0.0, 1.0)
}

/// Each root cell's matric potential and total potential (with gravity at the cell's
/// center) in megapascals, and its rhizosphere factor, in the order of `cells`.
fn cell_terms(soil: &SoilGrid, cells: &[(usize, Meters)]) -> Vec<(f64, f64, f64)> {
    let retention = soil.retention();
    let hy = soil.texture().hydraulics();
    let edge = soil.cell_size().0;
    cells
        .iter()
        .map(|&(c, _)| {
            let theta = soil.theta(c);
            let psi = retention.psi(theta).to_mpa().0;
            #[allow(clippy::cast_precision_loss)] // a layer index
            let z = -(soil.cell_of(c).z as f64 + 0.5) * edge;
            (
                psi,
                psi + GRAVITY_MPA_PER_M * z,
                rhizosphere_factor(theta, &hy),
            )
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
    fn edge_conductance(&self, i: usize, main: &[Option<(f64, bool)>]) -> f64 {
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
                        let s = (d / hp.root_axial_maturation.max(1e-9)).clamp(0.0, 1.0);
                        hp.root_axial_tip * libm::pow(base / hp.root_axial_tip, s)
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
    /// `(organ, cell slot, conductance)`, and each organ's summed terms.
    fn radial_links(
        &self,
        cells: &[(usize, Meters)],
        soil_terms: &[(f64, f64, f64)],
    ) -> (Vec<NodeTerms>, Vec<(usize, usize, f64)>) {
        let n = self.arena.len();
        let slot = |c: usize| cells.binary_search_by_key(&c, |p| p.0).ok();
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
            let k = lp_of[o]
                * 2.0
                * core::f64::consts::PI
                * r
                * s.length.0
                * soil_terms[j].2
                * KG_PER_HOUR_PER_M3_PER_S;
            if k <= 0.0 {
                continue;
            }
            nodes[o].radial.0 += k;
            nodes[o].source.0 += k * soil_terms[j].1;
            links.push((o, j, k));
        }
        (nodes, links)
    }

    /// Hourly step 7 at the detailed level (R3b): solves the water network against the
    /// soil as it stands, with this hour's leaf transpiration from
    /// [`Plant::water_demand_with`], keeps every organ's water potential for the next
    /// hour's stomata and expansion, and returns what the roots ask of each cell.
    ///
    /// Without a working root link (no roots in the soil yet, or every root cell
    /// air-dry), the plant takes nothing and every organ reads the root zone's soil
    /// potential.
    pub fn water_flows(&mut self, soil: &SoilGrid) -> WaterFlows {
        let n = self.arena.len();
        self.leaf_water.resize(n, KgWater(0.0));
        let demand = KgWater(self.leaf_water.iter().map(|w| w.0).sum());
        let cells = self.roots.cells();
        let soil_terms = cell_terms(soil, cells);
        let (mut nodes, links) = self.radial_links(cells, &soil_terms);
        for (i, w) in self.leaf_water.iter().enumerate() {
            nodes[i].sink = *w;
        }
        let main = self.main_root_positions();
        let edges: Vec<Edge> = self
            .arena
            .edges()
            .map(|(p, c)| Edge {
                parent: p.index(),
                child: c.index(),
                conductance: KgPerHourMPa(self.edge_conductance(c.index(), &main)),
            })
            .collect();
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
