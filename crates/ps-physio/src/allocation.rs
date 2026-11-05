//! Carbon: maintenance, income, transport, growth and starvation (§3.2 hourly steps 8
//! to 12 and daily step 4; §3.9). Model card: docs/models/carbon-allocation.md.

use ps_graph::{Allometry, GraphError, OrganArena, OrganId, OrganKind};
use ps_units::{Celsius, DegreeDays, GramsC, GramsDM, Hours};
use serde::{Deserialize, Serialize};

use crate::PhysioError;
use crate::respiration::{growth_split, maintenance_with_factor, q10_factor};
use crate::thermal::{Cardinal, GrowthTemperature};

/// Smallest structural mass used for a reserve concentration, g DM.
pub const MIN_MASS: f64 = 1e-6;
/// Smallest segment radius used for a transport conductance, m.
pub const MIN_RADIUS: f64 = 5e-4;
/// Smallest segment length used for a transport conductance, m.
pub const MIN_LENGTH: f64 = 5e-3;
/// Relative tolerance of the carbon ledger (CLAUDE.md).
pub const LEDGER_TOLERANCE: f64 = 1e-9;

/// Carbon used per carbon built, by organ class (decision 0032): tissue richer in
/// protein and lipid costs more glucose per gram (CROPGRO's composition costs).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionCost {
    /// Leaves.
    pub leaf: f64,
    /// Stems (internodes and sheaths), buds and tendrils.
    pub stem: f64,
    /// Roots and nodules.
    pub root: f64,
    /// Flowers, pods, seeds and the sown seed's organs.
    pub fruit: f64,
}

impl ConstructionCost {
    /// The same cost for every organ.
    #[must_use]
    pub const fn uniform(c: f64) -> Self {
        Self {
            leaf: c,
            stem: c,
            root: c,
            fruit: c,
        }
    }

    /// The cost of building an organ of kind `kind`.
    #[must_use]
    pub const fn of(&self, kind: OrganKind) -> f64 {
        match kind {
            OrganKind::Leaf => self.leaf,
            OrganKind::Internode | OrganKind::Bud | OrganKind::Tendril => self.stem,
            OrganKind::Root | OrganKind::Nodule => self.root,
            OrganKind::Seed
            | OrganKind::Cotyledon
            | OrganKind::Flower
            | OrganKind::Pod
            | OrganKind::SeedInPod => self.fruit,
        }
    }
}

/// Species carbon constants.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarbonParams {
    /// g C per g DM.
    pub carbon_fraction: f64,
    /// g C used per g C built, by organ class (decision 0032).
    pub construction_cost: ConstructionCost,
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
            construction_cost: ConstructionCost {
                leaf: c.construction_cost_leaf.value,
                stem: c.construction_cost_stem.value,
                root: c.construction_cost_root.value,
                fruit: c.construction_cost_fruit.value,
            },
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

/// Sink priority of secondary growth, stem and root thickening (§3.9 step 2, R1a).
pub const SECONDARY_PRIORITY: f64 = 0.4;

/// True for organs whose growth is expansion and so is limited by water stress.
#[must_use]
pub fn is_expanding_kind(kind: OrganKind) -> bool {
    matches!(
        kind,
        OrganKind::Leaf | OrganKind::Internode | OrganKind::Tendril | OrganKind::Cotyledon
    )
}

/// Carbon flows of one plant since the ledger started. The balance closes when
/// `stock now = start + added + income − maintenance − growth respiration − litter −
/// removed`.
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
    /// Carbon cut off and taken away (pruning, harvest).
    #[serde(default)]
    pub removed: GramsC,
    /// Carbon respired by nodules to fix nitrogen (WP24, decision 0020).
    #[serde(default)]
    pub fixation: GramsC,
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
                - self.litter.0
                - self.removed.0
                - self.fixation.0,
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
            self.removed.0.abs(),
            self.fixation.0,
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
    /// Demand not met this hour (primary growth).
    pub unmet: GramsC,
    /// Carbon spent on secondary growth, structure plus growth respiration (part of
    /// `growth`).
    pub secondary: GramsC,
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
    /// Plant water stress factor, for expansion (the caller multiplies in the
    /// nitrogen factor; decision 0020).
    pub stress: f64,
    /// Whether organs pay maintenance (false once the annual has matured and its
    /// tissue is dry; decision 0013).
    pub respiring: bool,
    /// Nitrogen per dry mass at or below which an organ cannot grow, by
    /// [`kind_index`] (decision 0020); `None` for no nitrogen limit.
    pub n_floor: Option<&'a [f64; ORGAN_KINDS]>,
    /// Potential secondary growth (thickening) this hour by organ id, g DM of
    /// structure, from the caller's rules (R1a); empty for none.
    pub secondary: &'a [GramsDM],
}

/// Number of organ kinds, for per-kind tables.
pub const ORGAN_KINDS: usize = 11;

/// Index of an organ kind into per-kind tables.
#[must_use]
pub fn kind_index(kind: OrganKind) -> usize {
    match kind {
        OrganKind::Seed => 0,
        OrganKind::Cotyledon => 1,
        OrganKind::Internode => 2,
        OrganKind::Bud => 3,
        OrganKind::Leaf => 4,
        OrganKind::Root => 5,
        OrganKind::Flower => 6,
        OrganKind::Pod => 7,
        OrganKind::SeedInPod => 8,
        OrganKind::Nodule => 9,
        OrganKind::Tendril => 10,
    }
}

/// Temperature responses used by growth.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
            arena.primary_radius(id).0.max(MIN_RADIUS),
            arena.size[i].length.0.max(MIN_LENGTH),
        )
    })
}

/// Edge conductance, g DM h⁻¹: `K · π r² / l` of the segment the edge runs through
/// (the child if it is a stem or root segment, else the parent, else a minimal one).
/// `r` is the primary radius: secondary growth widens a stem without easing transport
/// (R1a, decision 0036).
fn conductance(arena: &OrganArena, p: OrganId, c: OrganId, k: f64) -> f64 {
    let (r, l) = segment(arena, c)
        .or_else(|| segment(arena, p))
        .unwrap_or((MIN_RADIUS, MIN_LENGTH));
    k * core::f64::consts::PI * r * r / l
}

/// Whether an organ of `kind` pays maintenance this hour: leaves pay it as day
/// respiration, the sown seed and cotyledons respire little (decision 0013), and nothing
/// does once the annual has matured (decision 0020).
fn pays_maintenance(kind: OrganKind, respiring: bool) -> bool {
    respiring
        && !matches!(
            kind,
            OrganKind::Leaf | OrganKind::Seed | OrganKind::Cotyledon
        )
}

/// Whether an organ is diluted to its kind's nitrogen growth floor (decision 0020).
/// Secondary tissue carries no nitrogen requirement, so the floor applies to the
/// primary mass (R1a).
fn nitrogen_gated(arena: &OrganArena, id: OrganId, floor: &[f64; ORGAN_KINDS]) -> bool {
    let w = arena.primary(id).0;
    w > 0.0 && arena.nitrogen[id.index()].0 <= floor[kind_index(arena.kind[id.index()])] * w
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
    hourly_recording(arena, allometry, p, temp, inp, ledger, None)
}

/// What step 12 reads besides the organ: parameters, the hour's inputs, allometry.
struct GrowCtx<'a, 'b> {
    p: &'a CarbonParams,
    inp: &'a HourInputs<'b>,
    allometry: &'a Allometry,
}

/// Primary growth of one organ this hour (step 12) along its expansion curve, whose
/// clock advanced by `dt` this hour, at the organ's priority.
fn build_primary(
    arena: &mut OrganArena,
    id: OrganId,
    dt: DegreeDays,
    cost: f64,
    g: &GrowCtx<'_, '_>,
    rep: &mut HourReport,
    ledger: &mut CarbonLedger,
) -> Result<(), PhysioError> {
    let (p, inp) = (g.p, g.inp);
    let i = id.index();
    let cf = p.carbon_fraction;
    let age0 = DegreeDays(arena.expansion_age[i].0 - dt.0);
    let curve = arena.curve[i];
    if curve.potential.0 <= 0.0 || curve.is_complete(age0) {
        return Ok(());
    }
    // An organ diluted to its final nitrogen concentration cannot build more
    // tissue until the plant tops it up (decision 0020).
    if inp.n_floor.is_some_and(|f| nitrogen_gated(arena, id, f)) {
        return Ok(());
    }
    // Demand follows the species potential, not the reduced one: a shortfall
    // is lost (no catch-up) without compounding (decision 0009). At the end of
    // expansion the organ's size is `(1 − unmet / potential demand) × potential`.
    let mut pot = curve.increment(curve.potential, age0, dt).0;
    if is_expanding_kind(arena.kind[i]) {
        pot *= inp.stress.clamp(0.0, 1.0);
    }
    let demand = pot * cf * cost;
    if demand <= 0.0 {
        return Ok(());
    }
    let available = (arena.reserve[i].0 - p.reserve_floor * arena.primary(id).0).max(0.0);
    let spent = demand.min(priority(arena.kind[i], inp.ftsw) * available);
    let unmet = demand - spent;
    arena.record_demand(id, GramsC(demand), GramsC(unmet));
    rep.unmet += GramsC(unmet);
    if spent > 0.0 {
        arena.reserve[i] -= GramsC(spent);
        let (built, resp) = growth_split(GramsC(spent), cost);
        arena.grow(id, GramsDM(built.0 / cf), g.allometry)?;
        rep.growth += GramsC(spent);
        ledger.growth_respiration += resp;
    }
    Ok(())
}

/// Secondary growth of one organ this hour (R1a, decision 0036): the caller's
/// potential increment `pot`, met at [`SECONDARY_PRIORITY`] of the reserve above the
/// floor; what is not met is simply not built (no deficit is kept).
fn build_secondary(
    arena: &mut OrganArena,
    id: OrganId,
    pot: GramsDM,
    cost: f64,
    g: &GrowCtx<'_, '_>,
    rep: &mut HourReport,
    ledger: &mut CarbonLedger,
) -> Result<(), PhysioError> {
    let p = g.p;
    let i = id.index();
    let cf = p.carbon_fraction;
    let demand = pot.0 * cf * cost;
    let available = (arena.reserve[i].0 - p.reserve_floor * arena.primary(id).0).max(0.0);
    let spent = demand.min(SECONDARY_PRIORITY * available);
    if spent > 0.0 {
        arena.reserve[i] -= GramsC(spent);
        let (built, resp) = growth_split(GramsC(spent), cost);
        arena.grow_secondary(id, GramsDM(built.0 / cf), g.allometry)?;
        rep.growth += GramsC(spent);
        rep.secondary += GramsC(spent);
        ledger.growth_respiration += resp;
    }
    Ok(())
}

/// [`hourly`], also recording each organ's phloem import for the hour in `flows`
/// (WP26, the carbon flow lens): the carbon moved from its parent into it, negative
/// for an export to the parent, indexed by organ id (zero for organs without a living
/// parent). Recording only reads the fluxes, so the state and ledger are bit for bit
/// those of [`hourly`].
///
/// # Errors
/// As [`hourly`].
///
/// # Panics
/// As [`hourly`].
pub fn hourly_recording(
    arena: &mut OrganArena,
    allometry: &Allometry,
    p: &CarbonParams,
    temp: &Temperature,
    inp: &HourInputs<'_>,
    ledger: &mut CarbonLedger,
    flows: Option<&mut Vec<GramsC>>,
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

    // (temperature bits, factor) for each temperature seen this hour, in first-seen order.
    let mut q10_cache: Vec<(u64, f64)> = Vec::new();
    // 8. Maintenance from each organ's own reserve (leaves pay it as day respiration;
    // seed tissues respire little, decision 0013).
    for &id in &ids {
        let i = id.index();
        if !pays_maintenance(arena.kind[i], inp.respiring) {
            continue;
        }
        // Organs at one temperature (the shoot in air, roots in one soil layer) share
        // one Q10 factor: cache each one instead of calling pow per organ.
        let t = t_of(id);
        let bits = t.0.to_bits();
        let factor = if let Some(&(_, f)) = q10_cache.iter().find(|c| c.0 == bits) {
            f
        } else {
            let f = q10_factor(p.maintenance_q10, t);
            q10_cache.push((bits, f));
            f
        };
        let m = maintenance_with_factor(p.maintenance_rate_20c, arena.nitrogen[i], factor);
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
    rep.transported = transport_recording(arena, p, flows);
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
    // 12. Growth: primary growth along each organ's expansion curve, then secondary
    // growth (stem and root thickening, R1a) from what is left, at its own priority.
    let cf = p.carbon_fraction;
    let g = GrowCtx { p, inp, allometry };
    for &id in &ids {
        let i = id.index();
        let cost = p.construction_cost.of(arena.kind[i]);
        build_primary(arena, id, dt_exp[i], cost, &g, &mut rep, ledger)?;
        let pot = inp.secondary.get(i).copied().unwrap_or(GramsDM(0.0));
        if pot.0 > 0.0 {
            build_secondary(arena, id, pot, cost, &g, &mut rep, ledger)?;
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
#[cfg(test)]
pub(crate) fn transport(arena: &mut OrganArena, p: &CarbonParams) -> GramsC {
    transport_recording(arena, p, None)
}

/// [`transport`], writing each edge's flux into `flows` at the child's index.
fn transport_recording(
    arena: &mut OrganArena,
    p: &CarbonParams,
    flows: Option<&mut Vec<GramsC>>,
) -> GramsC {
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
        // Concentration is reserve over primary mass: secondary tissue is a sink, not
        // part of the phloem's volume (R1a).
        diag[i] = arena.primary(id).0.max(MIN_MASS);
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
    let mut flows = flows;
    if let Some(v) = flows.as_deref_mut() {
        v.clear();
        v.resize(n, GramsC(0.0));
    }
    for &(pa, ch, k) in &edges {
        let f = k * (conc[pa] - conc[ch]);
        arena.reserve[pa].0 -= f;
        arena.reserve[ch].0 += f;
        moved += f.abs();
        if let Some(v) = flows.as_deref_mut() {
            v[ch] = GramsC(f);
        }
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

/// Kills `root` and everything it carries (an axis dies with its basal internode, a
/// pod with its seeds) and books the carbon (§3.9 step 6, daily step 4).
/// `resorption_fraction` of each dead organ's structural carbon moves to the nearest
/// living ancestor of `root`, which also pays any negative reserve (carbon the organ
/// already respired, so litter is never negative; decision 0011); the rest of its
/// structure and any positive reserve become litter. With no living ancestor
/// everything, including a debt, goes to litter.
///
/// # Errors
/// A [`GraphError`] if `root` is dead or missing.
pub fn shed(
    arena: &mut OrganArena,
    p: &CarbonParams,
    root: OrganId,
    now: Hours,
    ledger: &mut CarbonLedger,
) -> Result<Vec<Death>, GraphError> {
    let killed = arena.kill_subtree(root, now)?;
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
    let mut deaths = Vec::with_capacity(killed.len());
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
    Ok(deaths)
}

/// Cuts `root` and everything it carries off the plant (pruning, harvest). Unlike
/// [`shed`], nothing is resorbed: each organ's structure and reserve leave with the
/// gardener (ledger `removed`), and a negative reserve leaves with its organ instead of
/// being charged to the ancestor. Returns the carbon removed.
///
/// # Errors
/// A [`GraphError`] if `root` is dead or missing.
pub fn cut(
    arena: &mut OrganArena,
    p: &CarbonParams,
    root: OrganId,
    now: Hours,
    ledger: &mut CarbonLedger,
) -> Result<GramsC, GraphError> {
    let killed = arena.kill_subtree(root, now)?;
    let mut total = GramsC(0.0);
    for (id, mass) in killed {
        total += mass.to_carbon(p.carbon_fraction) + arena.reserve[id.index()];
        arena.reserve[id.index()] = GramsC(0.0);
    }
    ledger.removed += total;
    Ok(total)
}

/// Daily step 4, starvation part (§3.9 step 6): organs whose reserve has been negative
/// for the starvation window die through [`shed`], with everything they carry.
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
            continue; // already died with an ancestor
        }
        deaths.extend(shed(arena, p, root, now, ledger)?);
    }
    debug_assert!(
        ledger.relative_residual(arena, p.carbon_fraction).abs() < LEDGER_TOLERANCE,
        "carbon ledger residual after deaths {}",
        ledger.relative_residual(arena, p.carbon_fraction)
    );
    Ok(deaths)
}
