//! Carbon: maintenance, income, transport, growth and starvation (§3.2 hourly steps 8
//! to 12 and daily step 4; §3.9). Model card: docs/models/carbon-allocation.md.

use ps_graph::{Allometry, GraphError, OrganArena, OrganId, OrganKind};
use ps_units::{Celsius, DegreeDays, GramsC, GramsDM, Hours};
use serde::{Deserialize, Serialize};

use crate::PhysioError;
use crate::respiration::{growth_split, maintenance_hourly};
use crate::thermal::{Cardinal, GrowthTemperature};

/// Smallest structural mass used for a reserve concentration, g DM.
pub const MIN_MASS: f64 = 1e-6;
/// Smallest segment radius used for a transport conductance, m.
pub const MIN_RADIUS: f64 = 5e-4;
/// Smallest segment length used for a transport conductance, m.
pub const MIN_LENGTH: f64 = 5e-3;
/// Relative tolerance of the carbon ledger (CLAUDE.md).
pub const LEDGER_TOLERANCE: f64 = 1e-9;

/// Species carbon constants.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarbonParams {
    /// g C per g DM.
    pub carbon_fraction: f64,
    /// g C used per g C built.
    pub construction_cost: f64,
    /// Maintenance at 20 °C, g C per g N per day.
    pub maintenance_rate_20c: f64,
    /// Q10 of maintenance.
    pub maintenance_q10: f64,
    /// Reserve concentration kept back from growth, g C per g DM.
    pub reserve_floor: f64,
    /// Share of structural carbon resorbed to the parent at death.
    pub resorption_fraction: f64,
    /// Hours of negative reserve before an organ dies.
    pub starvation_hours: u32,
    /// Phloem conductance per unit segment cross-section over length, g DM m⁻¹ h⁻¹.
    pub phloem_conductance: f64,
}

impl CarbonParams {
    /// From a species file.
    #[must_use]
    pub fn from_species(s: &ps_species::Species) -> Self {
        let c = &s.constants;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let starvation_hours = (c.starvation_window.value * 24.0).round().max(1.0) as u32;
        Self {
            carbon_fraction: c.carbon_fraction.value,
            construction_cost: c.construction_cost.value,
            maintenance_rate_20c: c.maintenance_rate_20c.value,
            maintenance_q10: c.maintenance_q10.value,
            reserve_floor: c.reserve_floor.value,
            resorption_fraction: c.resorption_fraction.value,
            starvation_hours,
            phloem_conductance: c.phloem_conductance.value,
        }
    }
}

/// Sink priority by organ kind (§3.9 step 2). The root term is the one encoded piece
/// of functional balance.
#[must_use]
pub fn priority(kind: OrganKind, ftsw: f64) -> f64 {
    match kind {
        OrganKind::SeedInPod => 1.0,
        OrganKind::Pod => 0.9,
        OrganKind::Root | OrganKind::Nodule => 0.6 + 0.4 * (1.0 - ftsw.clamp(0.0, 1.0)),
        _ => 0.7,
    }
}

/// True for organs whose growth is expansion and so is limited by water stress.
#[must_use]
pub fn is_expanding_kind(kind: OrganKind) -> bool {
    matches!(
        kind,
        OrganKind::Leaf | OrganKind::Internode | OrganKind::Tendril | OrganKind::Cotyledon
    )
}

/// Carbon flows of one plant since the ledger started. The balance closes when
/// `stock now = start + added + income − maintenance − growth respiration − litter`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CarbonLedger {
    /// Stock (reserves plus structural carbon of living organs) when the ledger started.
    pub start: GramsC,
    /// Carbon added from outside the plant (the sown seed, tests).
    pub added: GramsC,
    /// Net leaf assimilation (negative at night).
    pub income: GramsC,
    /// Maintenance respiration of non-leaf organs.
    pub maintenance: GramsC,
    /// Growth respiration.
    pub growth_respiration: GramsC,
    /// Carbon shed with dead organs.
    pub litter: GramsC,
}

impl CarbonLedger {
    /// A ledger starting from the plant's current stock.
    #[must_use]
    pub fn start(arena: &OrganArena, carbon_fraction: f64) -> Self {
        Self {
            start: stock(arena, carbon_fraction),
            ..Self::default()
        }
    }

    /// Stock expected from the flows.
    #[must_use]
    pub fn expected(&self) -> GramsC {
        GramsC(
            self.start.0 + self.added.0 + self.income.0
                - self.maintenance.0
                - self.growth_respiration.0
                - self.litter.0,
        )
    }

    /// `(stock − expected)` relative to the largest term.
    #[must_use]
    pub fn relative_residual(&self, arena: &OrganArena, carbon_fraction: f64) -> f64 {
        let s = stock(arena, carbon_fraction).0;
        let scale = [
            s.abs(),
            self.start.0.abs(),
            self.added.0.abs(),
            self.income.0.abs(),
            self.maintenance.0,
            self.growth_respiration.0,
            self.litter.0.abs(),
            1e-12,
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        (s - self.expected().0) / scale
    }
}

/// Reserves plus structural carbon of the living organs, in id order.
#[must_use]
pub fn stock(arena: &OrganArena, carbon_fraction: f64) -> GramsC {
    arena
        .alive_ids()
        .map(|id| {
            let i = id.index();
            arena.reserve[i] + arena.structural(id).to_carbon(carbon_fraction)
        })
        .sum()
}

/// What one hour of carbon did, for the inspector and the CSV.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct HourReport {
    /// Maintenance paid.
    pub maintenance: GramsC,
    /// Net income.
    pub income: GramsC,
    /// Carbon moved along edges (sum of absolute fluxes).
    pub transported: GramsC,
    /// Carbon spent on growth (structure plus growth respiration).
    pub growth: GramsC,
    /// Demand not met this hour.
    pub unmet: GramsC,
}

/// Per-hour inputs to [`hourly`], all indexed by organ id.
#[derive(Copy, Clone, Debug)]
pub struct HourInputs<'a> {
    /// Organ temperature (air for the shoot, soil layer for roots).
    pub temperature: &'a [Celsius],
    /// Net leaf assimilation this hour (zero for organs that are not leaves).
    pub income: &'a [GramsC],
    /// Plant FTSW, for the root priority.
    pub ftsw: f64,
    /// Plant water stress factor, for expansion.
    pub stress: f64,
}

/// Temperature responses used by growth.
#[derive(Copy, Clone, Debug)]
pub struct Temperature {
    /// Thermal time.
    pub cardinal: Cardinal,
    /// Expansion multiplier.
    pub growth: GrowthTemperature,
}

fn segment(arena: &OrganArena, id: OrganId) -> Option<(f64, f64)> {
    let i = id.index();
    matches!(
        arena.kind[i],
        OrganKind::Internode | OrganKind::Root | OrganKind::Tendril
    )
    .then(|| {
        (
            arena.size[i].radius.0.max(MIN_RADIUS),
            arena.size[i].length.0.max(MIN_LENGTH),
        )
    })
}

/// Edge conductance, g DM h⁻¹: `K · π r² / l` of the segment the edge runs through
/// (the child if it is a stem or root segment, else the parent, else a minimal one).
fn conductance(arena: &OrganArena, p: OrganId, c: OrganId, k: f64) -> f64 {
    let (r, l) = segment(arena, c)
        .or_else(|| segment(arena, p))
        .unwrap_or((MIN_RADIUS, MIN_LENGTH));
    k * core::f64::consts::PI * r * r / l
}

/// Hourly steps 8 to 12 of §3.2 for one plant.
///
/// # Errors
/// [`PhysioError::InputLength`] if `temperature` or `income` is not one entry per
/// organ; [`PhysioError::Graph`] if growth is rejected (a bug: increments are
/// non-negative).
///
/// # Panics
/// In debug builds, if the carbon ledger does not close to 1e-9 relative.
pub fn hourly(
    arena: &mut OrganArena,
    allometry: &Allometry,
    p: &CarbonParams,
    temp: &Temperature,
    inp: &HourInputs<'_>,
    ledger: &mut CarbonLedger,
) -> Result<HourReport, PhysioError> {
    for (what, got) in [
        ("temperature", inp.temperature.len()),
        ("income", inp.income.len()),
    ] {
        if got != arena.len() {
            return Err(PhysioError::InputLength {
                what,
                got,
                want: arena.len(),
            });
        }
    }
    let mut rep = HourReport::default();
    let ids: Vec<OrganId> = arena.alive_ids().collect();
    let t_of = |id: OrganId| inp.temperature[id.index()];

    // 8. Maintenance from each organ's own reserve (leaves pay it as day respiration).
    for &id in &ids {
        let i = id.index();
        if arena.kind[i] == OrganKind::Leaf {
            continue;
        }
        let m = maintenance_hourly(
            p.maintenance_rate_20c,
            p.maintenance_q10,
            arena.nitrogen[i],
            t_of(id),
        );
        arena.reserve[i] -= m;
        rep.maintenance += m;
    }
    // 9. Income into each leaf's own reserve.
    for &id in &ids {
        let i = id.index();
        let c = inp.income[i];
        arena.reserve[i] += c;
        rep.income += c;
    }
    // Starvation signal: reserve negative after maintenance and respiration.
    for &id in &ids {
        let i = id.index();
        arena.starving_hours[i] = if arena.reserve[i].0 < 0.0 {
            arena.starving_hours[i].saturating_add(1)
        } else {
            0
        };
    }
    // 10. Transport: one implicit diffusion step on the organ tree.
    rep.transported = transport(arena, p);
    // 11. Thermal time, and the expansion clock: thermal time scaled by the expansion
    // temperature factor, so cold delays expansion instead of destroying it
    // (decision 0011).
    let mut dt_exp = vec![DegreeDays(0.0); arena.len()];
    for &id in &ids {
        let i = id.index();
        let dt = temp.cardinal.hourly(t_of(id));
        arena.thermal_age[i] += dt;
        dt_exp[i] = DegreeDays(dt.0 * temp.growth.factor(t_of(id)));
        arena.expansion_age[i] += dt_exp[i];
    }
    // 12. Growth.
    let cf = p.carbon_fraction;
    for &id in &ids {
        let i = id.index();
        let age0 = DegreeDays(arena.expansion_age[i].0 - dt_exp[i].0);
        let curve = arena.curve[i];
        if curve.potential.0 <= 0.0 || curve.is_complete(age0) {
            continue;
        }
        // Demand follows the species potential, not the reduced one: a shortfall is
        // lost (no catch-up) without compounding (decision 0009). At the end of
        // expansion the organ's size is `(1 − unmet / potential demand) × potential`.
        let mut pot = curve.increment(curve.potential, age0, dt_exp[i]).0;
        if is_expanding_kind(arena.kind[i]) {
            pot *= inp.stress.clamp(0.0, 1.0);
        }
        let demand = pot * cf * p.construction_cost;
        if demand <= 0.0 {
            continue;
        }
        let available = (arena.reserve[i].0 - p.reserve_floor * arena.structural(id).0).max(0.0);
        let spent = demand.min(priority(arena.kind[i], inp.ftsw) * available);
        let unmet = demand - spent;
        arena.record_demand(id, GramsC(demand), GramsC(unmet));
        rep.unmet += GramsC(unmet);
        if spent > 0.0 {
            arena.reserve[i] -= GramsC(spent);
            let (built, resp) = growth_split(GramsC(spent), p.construction_cost);
            arena.grow(id, GramsDM(built.0 / cf), allometry)?;
            rep.growth += GramsC(spent);
            ledger.growth_respiration += resp;
        }
    }
    ledger.maintenance += rep.maintenance;
    ledger.income += rep.income;
    debug_assert!(
        ledger.relative_residual(arena, cf).abs() < LEDGER_TOLERANCE,
        "carbon ledger residual {}",
        ledger.relative_residual(arena, cf)
    );
    Ok(rep)
}

/// One hour of phloem transport (§3.9 step 3; decision 0011): a backward-Euler
/// diffusion step of reserve concentration on the organ tree, solved exactly.
///
/// With `C = R / W` and edge conductance `k`, the step solves
/// `W_i C_i' + Σ_j k_ij (C_i' − C_j') = R_i⁺` for every living organ, where `R⁺` is the
/// reserve's positive part (a debt neither pulls carbon by itself nor is spread to
/// neighbors; it stays with its organ). The matrix is a tree Laplacian plus a positive
/// diagonal, so one leaf-to-root elimination and one root-to-leaf substitution solve it
/// in O(n) (the Thomas algorithm on a tree). It is unconditionally stable and monotone:
/// no concentration leaves the range of its neighbors', so no reserve is driven
/// negative and no step size limit applies. Carbon moves as edge fluxes
/// `k (C_p' − C_c')`, so it is conserved to rounding. Returns the carbon moved.
pub(crate) fn transport(arena: &mut OrganArena, p: &CarbonParams) -> GramsC {
    // Edges by child id; a parent is always created before its children, so its id is
    // smaller and descending child order eliminates every subtree before its root.
    let edges: Vec<(usize, usize, f64)> = arena
        .edges()
        .map(|(pa, ch)| {
            (
                pa.index(),
                ch.index(),
                conductance(arena, pa, ch, p.phloem_conductance),
            )
        })
        .collect();
    let n = arena.len();
    let mut diag = vec![0.0; n];
    let mut rhs = vec![0.0; n];
    for id in arena.alive_ids() {
        let i = id.index();
        diag[i] = arena.structural(id).0.max(MIN_MASS);
        rhs[i] = arena.reserve[i].0.max(0.0);
    }
    for &(pa, ch, k) in &edges {
        debug_assert!(pa < ch, "parent {pa} after child {ch}");
        diag[pa] += k;
        diag[ch] += k;
    }
    // Leaf-to-root elimination.
    for &(pa, ch, k) in edges.iter().rev() {
        let f = k / diag[ch];
        diag[pa] -= k * f;
        rhs[pa] += f * rhs[ch];
    }
    // Root-to-leaf substitution. Roots (no living parent) solve directly.
    let mut conc = vec![0.0; n];
    for id in arena.alive_ids() {
        let i = id.index();
        conc[i] = rhs[i] / diag[i];
    }
    for &(pa, ch, k) in &edges {
        conc[ch] = (rhs[ch] + k * conc[pa]) / diag[ch];
    }
    let mut moved = 0.0;
    for &(pa, ch, k) in &edges {
        let f = k * (conc[pa] - conc[ch]);
        arena.reserve[pa].0 -= f;
        arena.reserve[ch].0 += f;
        moved += f.abs();
    }
    GramsC(moved)
}

/// One organ that died of starvation and where its carbon went.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Death {
    /// The organ.
    pub id: OrganId,
    /// Net carbon moved to the nearest living ancestor: resorbed structure minus any
    /// reserve debt the ancestor paid.
    pub resorbed: GramsC,
    /// Carbon shed as litter (structure not resorbed, plus the reserve).
    pub litter: GramsC,
}

/// Daily step 4, starvation part (§3.9 step 6): organs whose reserve has been negative
/// for the starvation window die, with their whole subtree if they are a stem segment
/// (an axis dies with its basal internode). `resorption_fraction` of each dead organ's
/// structural carbon moves to the nearest living ancestor, which also pays any
/// negative reserve (carbon the organ already respired, so litter is never negative;
/// decision 0011); the rest of its structure and any positive reserve become litter.
/// With no living ancestor everything, including a debt, goes to litter.
///
/// # Errors
/// A [`GraphError`] only on an inconsistent arena.
pub fn starvation_deaths(
    arena: &mut OrganArena,
    p: &CarbonParams,
    now: Hours,
    ledger: &mut CarbonLedger,
) -> Result<Vec<Death>, GraphError> {
    let starved: Vec<OrganId> = arena
        .alive_ids()
        .filter(|id| arena.starving_hours[id.index()] >= p.starvation_hours)
        .collect();
    let mut deaths = Vec::new();
    for root in starved {
        if !arena.alive[root.index()] {
            continue; // already died with an ancestor's axis
        }
        let killed = if arena.kind[root.index()] == OrganKind::Internode {
            arena.kill_subtree(root, now)?
        } else {
            vec![(root, arena.kill(root, now)?)]
        };
        let ancestor = {
            let mut a = arena.parent[root.index()];
            while let Some(x) = a {
                if arena.alive[x.index()] {
                    break;
                }
                a = arena.parent[x.index()];
            }
            a
        };
        for (id, mass) in killed {
            let structural = mass.to_carbon(p.carbon_fraction);
            let resorbed = if ancestor.is_some() {
                structural * p.resorption_fraction
            } else {
                GramsC(0.0)
            };
            let reserve = arena.reserve[id.index()];
            let (litter, to_ancestor) = match ancestor {
                Some(a) => {
                    let debt = GramsC(reserve.0.min(0.0));
                    arena.reserve[a.index()] += resorbed + debt;
                    (
                        structural - resorbed + GramsC(reserve.0.max(0.0)),
                        resorbed + debt,
                    )
                }
                None => (structural + reserve, GramsC(0.0)),
            };
            arena.reserve[id.index()] = GramsC(0.0);
            ledger.litter += litter;
            deaths.push(Death {
                id,
                resorbed: to_ancestor,
                litter,
            });
        }
    }
    debug_assert!(
        ledger.relative_residual(arena, p.carbon_fraction).abs() < LEDGER_TOLERANCE,
        "carbon ledger residual after deaths {}",
        ledger.relative_residual(arena, p.carbon_fraction)
    );
    Ok(deaths)
}
