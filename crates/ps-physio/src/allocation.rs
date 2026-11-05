//! Carbon: maintenance, income, transport, growth and starvation (§3.2 hourly steps 8
//! to 12 and daily step 4; §3.9). Model card: docs/models/carbon-allocation.md.

use ps_graph::{Allometry, GraphError, OrganArena, OrganId, OrganKind};
use ps_units::{Celsius, DegreeDays, GramsC, GramsDM, Hours};
use serde::{Deserialize, Serialize};

use crate::respiration::{growth_split, maintenance_hourly};
use crate::thermal::{Cardinal, GrowthTemperature};

/// Smallest structural mass used for a reserve concentration, g DM.
pub const MIN_MASS: f64 = 1e-6;
/// Smallest segment radius used for a transport conductance, m.
pub const MIN_RADIUS: f64 = 5e-4;
/// Smallest segment length used for a transport conductance, m.
pub const MIN_LENGTH: f64 = 5e-3;
/// Stability cap on transport (decision 0009): an edge's conductance is at most this
/// share of `W / degree` of either end, so each organ's hourly concentration change is
/// a convex combination of its neighbors' and the explicit step cannot oscillate.
pub const TRANSPORT_STABILITY: f64 = 0.5;
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
/// A [`GraphError`] if growth is rejected (a bug: increments are non-negative).
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
) -> Result<HourReport, GraphError> {
    let mut rep = HourReport::default();
    let ids: Vec<OrganId> = arena.alive_ids().collect();
    let t_of = |id: OrganId| {
        inp.temperature
            .get(id.index())
            .copied()
            .unwrap_or(Celsius(20.0))
    };

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
        if let Some(&c) = inp.income.get(i) {
            arena.reserve[i] += c;
            rep.income += c;
        }
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
    // 10. Transport: one Jacobi pass along parent-child edges, then the clamp.
    rep.transported = transport(arena, p);
    // 11. Thermal time.
    let mut dt = vec![DegreeDays(0.0); arena.len()];
    for &id in &ids {
        let i = id.index();
        dt[i] = temp.cardinal.hourly(t_of(id));
        arena.thermal_age[i] += dt[i];
    }
    // 12. Growth.
    let cf = p.carbon_fraction;
    for &id in &ids {
        let i = id.index();
        let age0 = DegreeDays(arena.thermal_age[i].0 - dt[i].0);
        let curve = arena.curve[i];
        if curve.potential.0 <= 0.0 || curve.is_complete(age0) {
            continue;
        }
        // Demand follows the species potential, not the reduced one: a shortfall is
        // lost (no catch-up) without compounding (decision 0009). At the end of
        // expansion the organ's size is `(1 − unmet / potential demand) × potential`.
        let mut pot = curve.increment(curve.potential, age0, dt[i]).0;
        pot *= temp.growth.factor(t_of(id));
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

/// One transport pass (§3.9 step 3, §11.4): fluxes from one set of concentrations,
/// then each organ's total outflow scaled so transport alone never takes its reserve
/// below zero. Returns the total carbon moved.
fn transport(arena: &mut OrganArena, p: &CarbonParams) -> GramsC {
    let conc =
        |a: &OrganArena, id: OrganId| a.reserve[id.index()].0 / a.structural(id).0.max(MIN_MASS);
    let edges: Vec<(OrganId, OrganId)> = arena.edges().collect();
    let mut degree = vec![0u32; arena.len()];
    for (pa, ch) in &edges {
        degree[pa.index()] += 1;
        degree[ch.index()] += 1;
    }
    let cap = |a: &OrganArena, id: OrganId| {
        a.structural(id).0.max(MIN_MASS) / f64::from(degree[id.index()].max(1))
    };
    // (from, to, flux >= 0), in edge order.
    let flows: Vec<(usize, usize, f64)> = edges
        .iter()
        .map(|&(pa, ch)| {
            let k = conductance(arena, pa, ch, p.phloem_conductance)
                .min(TRANSPORT_STABILITY * cap(arena, pa).min(cap(arena, ch)));
            let f = k * (conc(arena, pa) - conc(arena, ch));
            if f >= 0.0 {
                (pa.index(), ch.index(), f)
            } else {
                (ch.index(), pa.index(), -f)
            }
        })
        .collect();
    let mut out = vec![0.0; arena.len()];
    for &(from, _, f) in &flows {
        out[from] += f;
    }
    let scale: Vec<f64> = out
        .iter()
        .zip(&arena.reserve)
        .map(|(&o, r)| {
            if o > r.0.max(0.0) {
                r.0.max(0.0) / o
            } else {
                1.0
            }
        })
        .collect();
    let mut moved = 0.0;
    for (from, to, f) in flows {
        let f = f * scale[from];
        arena.reserve[from].0 -= f;
        arena.reserve[to].0 += f;
        moved += f;
    }
    GramsC(moved)
}

/// One organ that died of starvation and where its carbon went.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Death {
    /// The organ.
    pub id: OrganId,
    /// Carbon resorbed to the nearest living ancestor.
    pub resorbed: GramsC,
    /// Carbon shed as litter (structure not resorbed, plus the reserve).
    pub litter: GramsC,
}

/// Daily step 4, starvation part (§3.9 step 6): organs whose reserve has been negative
/// for the starvation window die, with their whole subtree if they are a stem segment
/// (an axis dies with its basal internode). `resorption_fraction` of each dead organ's
/// structural carbon moves to the nearest living ancestor; the rest of its structure
/// and its reserve (possibly negative) become litter.
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
            let litter = structural - resorbed + reserve;
            if let Some(a) = ancestor {
                arena.reserve[a.index()] += resorbed;
            }
            arena.reserve[id.index()] = GramsC(0.0);
            ledger.litter += litter;
            deaths.push(Death {
                id,
                resorbed,
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
