//! Plant nitrogen (design plan §3.6 "Nitrogen (M2)", WP24, decision 0020).
//!
//! Every organ holds nitrogen (`arena.nitrogen`, g N), and the plant keeps a small pool
//! of free nitrogen (amino acids and nitrate in transit).
//!
//! Every hour, after allocation, each growing organ is topped up from the pool to its
//! kind's "initial" concentration, `max(0, c_initial · W − N)`, all by the same share
//! when the pool is short (`Plant::top_up_hour`). An organ diluted to its kind's
//! growth floor does not grow that hour. Nodules fix before allocation
//! (`Plant::fix_hour`).
//!
//! Once a day, as step 0 of the daily step:
//!
//! 1. **Demand.** The growing organs' deficits plus the nitrogen yesterday's growth
//!    used, as the forecast of today's.
//! 2. **Supply, in order.** The pool (without yesterday's fixation), then the soil (the
//!    world's joint uptake, asked for the demand the pool leaves), then mining: up to
//!    `rate · (N − c_final · W)` per day from organs that have stopped growing,
//!    low-light leaves first.
//! 3. **Fixation for tomorrow.** Nodules are set to fix the demand that uptake and mining
//!    left, up to their capacity. Each hour a nodule pays `fixation_cost` g C per g N from
//!    its own reserve and adds the nitrogen to the pool (demand-driven fixation, CROPGRO
//!    NFIX).
//! 4. **Nodulation.** If that demand needs more nodule mass than the plant has, new
//!    nodule clusters start on shallow roots, at most `relative_growth` of the existing
//!    nodule mass per day, fewer in nitrate-rich soil.
//! 5. **Leaf drop.** A mined leaf whose mobile nitrogen falls below `leaf_drop_mobile` of
//!    a new leaf's drops; low-light leaves are mined first, so leaves drop from the bottom
//!    up (emergent senescence).
//! 6. **Status.** The share of the growing leaves' and stems' mobile nitrogen they have,
//!    against a full top-up, scales expansion like water stress.
//!
//! Leaf nitrogen per area sets photosynthetic capacity: `(Na − Nb) / (Na_ref − Nb)`,
//! with `Na_ref` a new leaf's nitrogen per area at the species SLA.
//!
//! Model card: docs/models/nitrogen.md.

use ps_graph::{OrganId, OrganKind};
use ps_physio::Stage;
use ps_physio::allocation::{Death, ORGAN_KINDS, kind_index};
use ps_soil::{NitrogenRequest, SoilGrid, stress_factor};
use ps_species::{Species, Traits};
use ps_units::{DegreeDays, GramsC, GramsDM, GramsN, Hours, Meters};
use serde::{Deserialize, Serialize};

use crate::{ArchError, Plant, Role, Spawn, curve};

/// Relative tolerance of the nitrogen ledger (CLAUDE.md, as water and carbon).
pub const N_LEDGER_TOLERANCE: f64 = 1e-9;
/// Most nodule clusters a plant starts in one day.
const MAX_NEW_NODULES: usize = 24;

/// Species nitrogen values in model units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NitrogenParams {
    /// New-tissue concentration by organ kind, g N per g DM.
    pub initial: [f64; ORGAN_KINDS],
    /// Concentration left when all mobile nitrogen is gone, by kind.
    pub floor: [f64; ORGAN_KINDS],
    /// Concentration at which an organ stops growing, by kind: the final one, except
    /// roots and nodules, which keep growing on carbon as they dilute (an N-poor plant
    /// keeps exploring the soil; decision 0020).
    pub growth_floor: [f64; ORGAN_KINDS],
    /// Leaf nitrogen per area at zero capacity, g N m⁻².
    pub leaf_area_base: f64,
    /// A new leaf's nitrogen per area at the species SLA, g N m⁻².
    pub leaf_area_reference: f64,
    /// Uptake capacity, g N per m of root per day, per form.
    pub uptake_capacity: f64,
    /// Mobilization rate during seed fill, per day.
    pub mobilization_rate: f64,
    /// Mobilization before seed fill, relative.
    pub vegetative_mobilization: f64,
    /// Leaf drop threshold, share of a new leaf's mobile nitrogen.
    pub leaf_drop_mobile: f64,
    /// Fixation per nodule dry mass per day at best, g N g⁻¹ d⁻¹.
    pub nodule_activity: f64,
    /// Most relative nodule growth per day.
    pub nodule_relative_growth: f64,
    /// Nodule mass started at first nodulation.
    pub nodule_initial_mass: GramsDM,
    /// Potential mass of one nodule cluster.
    pub nodule_mass: GramsDM,
    /// Expansion duration of a nodule cluster.
    pub nodule_expansion: DegreeDays,
    /// Deepest roots that carry nodules (below the surface).
    pub nodule_depth: Meters,
    /// Carbon respired per nitrogen fixed, g C per g N.
    pub fixation_cost: f64,
    /// Fixation temperature trapezoid: min, optimum low, optimum high, max, °C.
    pub fixation_t: [f64; 4],
    /// Expansion stress factor below which fixation falls.
    pub fixation_drought: f64,
    /// Nitrate inhibition of nodulation: onset ppm, full ppm, floor.
    pub nodule_nitrate: [f64; 3],
}

impl NitrogenParams {
    /// From a species file and phenotype.
    #[must_use]
    pub fn new(s: &Species, t: &Traits) -> Self {
        let n = &s.nitrogen;
        let mut initial = [0.0; ORGAN_KINDS];
        let mut floor = [0.0; ORGAN_KINDS];
        let set = |table: &mut [f64; ORGAN_KINDS], kinds: &[OrganKind], v: f64| {
            for k in kinds {
                table[kind_index(*k)] = v;
            }
        };
        let stem = [OrganKind::Internode, OrganKind::Bud, OrganKind::Tendril];
        let shell = [OrganKind::Flower, OrganKind::Pod];
        let seed = [OrganKind::Seed, OrganKind::Cotyledon, OrganKind::SeedInPod];
        set(&mut initial, &[OrganKind::Leaf], n.leaf_n_initial.value);
        set(&mut floor, &[OrganKind::Leaf], n.leaf_n_final.value);
        set(&mut initial, &stem, n.stem_n_initial.value);
        set(&mut floor, &stem, n.stem_n_final.value);
        set(&mut initial, &[OrganKind::Root], n.root_n_initial.value);
        set(&mut floor, &[OrganKind::Root], n.root_n_final.value);
        set(&mut initial, &shell, n.shell_n_initial.value);
        set(&mut floor, &shell, n.shell_n_final.value);
        set(&mut initial, &seed, n.seed_n_initial.value);
        set(&mut floor, &seed, n.seed_n_final.value);
        set(&mut initial, &[OrganKind::Nodule], n.nodule_n.value);
        set(&mut floor, &[OrganKind::Nodule], n.nodule_n.value);
        let mut growth_floor = floor;
        growth_floor[kind_index(OrganKind::Root)] = 0.0;
        growth_floor[kind_index(OrganKind::Nodule)] = 0.0;
        // SLA cm² g⁻¹ → m² g⁻¹.
        let sla = (t.specific_leaf_area * 1e-4).max(1e-9);
        Self {
            initial,
            floor,
            growth_floor,
            leaf_area_base: n.leaf_n_area_base.value,
            leaf_area_reference: n.leaf_n_initial.value / sla,
            uptake_capacity: n.uptake_capacity.value,
            mobilization_rate: n.mobilization_rate.value,
            vegetative_mobilization: n.vegetative_mobilization.value,
            leaf_drop_mobile: n.leaf_drop_mobile.value,
            nodule_activity: n.nodule_activity.value,
            nodule_relative_growth: n.nodule_relative_growth.value,
            nodule_initial_mass: GramsDM(n.nodule_initial_mass.value),
            nodule_mass: GramsDM(n.nodule_mass.value),
            nodule_expansion: DegreeDays(n.nodule_expansion.value),
            nodule_depth: Meters(n.nodule_depth.value * 0.01),
            fixation_cost: n.fixation_cost.value,
            fixation_t: [
                n.fixation_t_min.value,
                n.fixation_t_opt_low.value,
                n.fixation_t_opt_high.value,
                n.fixation_t_max.value,
            ],
            fixation_drought: n.fixation_drought_threshold.value,
            nodule_nitrate: [
                n.nodule_nitrate_onset.value,
                n.nodule_nitrate_full.value,
                n.nodule_nitrate_floor.value,
            ],
        }
    }

    /// New-tissue concentration of `kind`.
    #[must_use]
    pub fn initial_of(&self, kind: OrganKind) -> f64 {
        self.initial[kind_index(kind)]
    }

    /// Final concentration of `kind`.
    #[must_use]
    pub fn floor_of(&self, kind: OrganKind) -> f64 {
        self.floor[kind_index(kind)]
    }

    /// Photosynthetic capacity from leaf nitrogen per area (0 to 1).
    #[must_use]
    pub fn leaf_capacity(&self, n_per_area: f64) -> f64 {
        let span = (self.leaf_area_reference - self.leaf_area_base).max(1e-9);
        ((n_per_area - self.leaf_area_base) / span).clamp(0.0, 1.0)
    }

    /// Fixation temperature factor (trapezoid).
    #[must_use]
    pub fn fixation_temperature(&self, t: f64) -> f64 {
        let [lo, o1, o2, hi] = self.fixation_t;
        if t <= lo || t >= hi {
            0.0
        } else if t < o1 {
            (t - lo) / (o1 - lo)
        } else if t > o2 {
            (hi - t) / (hi - o2)
        } else {
            1.0
        }
    }

    /// Nodulation factor from root-zone nitrate, mg N per kg soil.
    #[must_use]
    pub fn nitrate_factor(&self, ppm: f64) -> f64 {
        let [onset, full, floor] = self.nodule_nitrate;
        let x = ((ppm - onset) / (full - onset).max(1e-9)).clamp(0.0, 1.0);
        1.0 - (1.0 - floor) * x
    }
}

/// Nitrogen flows of one plant since sowing, g N. The balance closes when
/// `stock = start + seed + uptake + fixed − litter − removed`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NitrogenLedger {
    /// Stock when the ledger started.
    pub start: GramsN,
    /// Nitrogen in the sown seed.
    pub seed: GramsN,
    /// Taken up from the soil.
    pub uptake: GramsN,
    /// Fixed by nodules.
    pub fixed: GramsN,
    /// Shed with dead organs.
    pub litter: GramsN,
    /// Cut off and taken away.
    pub removed: GramsN,
}

impl NitrogenLedger {
    /// `(stock − expected) / largest term`.
    #[must_use]
    pub fn relative_residual(&self, stock: GramsN) -> f64 {
        let expected =
            self.start + self.seed + self.uptake + self.fixed - self.litter - self.removed;
        let scale = [
            stock.0,
            self.start.0,
            self.seed.0,
            self.uptake.0,
            self.fixed.0,
            self.litter.0,
            self.removed.0,
        ]
        .into_iter()
        .fold(1e-15, f64::max);
        (stock - expected).0 / scale
    }
}

/// One day's nitrogen, g N (fixation accumulates over the day's hours).
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NitrogenDay {
    /// Nitrogen asked for at the daily step: growing organs' deficits plus a day of
    /// hourly top-ups.
    pub demand: GramsN,
    /// Given to growing organs hourly from the pool.
    #[serde(default)]
    pub used: GramsN,
    /// Nitrogen the day's growth needed (new tissue at the initial concentration):
    /// tomorrow's forecast.
    #[serde(default)]
    pub growth: GramsN,
    /// Growing organs' deficits left after the last top-up.
    #[serde(default)]
    pub residual: GramsN,
    /// From the soil.
    pub uptake: GramsN,
    /// Mobilized from organs that stopped growing.
    pub mined: GramsN,
    /// Fixed by nodules.
    pub fixed: GramsN,
    /// Carbon nodules respired to fix it.
    pub fixation_carbon: GramsC,
    /// Growing organs' deficits at the end of the day, before the nightly top-up.
    pub unmet: GramsN,
    /// Demand left for fixation after uptake and mining.
    pub fixation_need: GramsN,
}

/// The plant's nitrogen state.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlantNitrogen {
    /// Free nitrogen not yet in an organ.
    pub pool: GramsN,
    /// Ledger since sowing.
    pub ledger: NitrogenLedger,
    /// Nitrogen factor on expansion, 0 to 1.
    pub status: f64,
    /// Today's flows.
    pub today: NitrogenDay,
    /// Yesterday's flows, for the report.
    pub last: NitrogenDay,
}

impl Default for PlantNitrogen {
    fn default() -> Self {
        Self {
            pool: GramsN(0.0),
            ledger: NitrogenLedger::default(),
            status: 1.0,
            today: NitrogenDay::default(),
            last: NitrogenDay::default(),
        }
    }
}

impl Plant {
    /// All nitrogen in the plant: the pool plus every living organ.
    #[must_use]
    pub fn nitrogen_stock(&self) -> GramsN {
        self.nitrogen.pool
            + self
                .arena
                .alive_ids()
                .map(|id| self.arena.nitrogen[id.index()])
                .sum()
    }

    pub(crate) fn assert_nitrogen(&self) {
        debug_assert!(
            self.nitrogen
                .ledger
                .relative_residual(self.nitrogen_stock())
                .abs()
                < N_LEDGER_TOLERANCE,
            "plant nitrogen ledger residual {}: {:?}, stock {:?}",
            self.nitrogen
                .ledger
                .relative_residual(self.nitrogen_stock()),
            self.nitrogen.ledger,
            self.nitrogen_stock()
        );
    }

    /// Splits the sown seed's nitrogen: the mobilizable share goes to the pool, the
    /// rest stays in the seed's organs.
    pub(crate) fn sow_nitrogen(&mut self) {
        let np = &self.params.nitrogen;
        let total = self.params.seed_mass.0 * np.initial_of(OrganKind::Seed);
        let mobile = total * self.params.seed_mobilizable;
        let ids: Vec<OrganId> = self.arena.alive_ids().collect();
        let fixed_mass: f64 = ids.iter().map(|id| self.arena.structural(*id).0).sum();
        for id in ids {
            let share = if fixed_mass > 0.0 {
                self.arena.structural(id).0 / fixed_mass
            } else {
                0.0
            };
            self.arena.nitrogen[id.index()] = GramsN((total - mobile) * share);
        }
        self.nitrogen.pool = GramsN(mobile);
        self.nitrogen.ledger.seed = GramsN(total);
        self.assert_nitrogen();
    }

    /// Gives a new organ its nitrogen at its kind's initial concentration, from the
    /// pool, or what the pool has.
    pub(crate) fn feed_new_organ(&mut self, id: OrganId) {
        let i = id.index();
        let want =
            self.params.nitrogen.initial_of(self.arena.kind[i]) * self.arena.structural(id).0;
        let n = GramsN(want.max(0.0).min(self.nitrogen.pool.0.max(0.0)));
        self.arena.nitrogen[i] = n;
        self.nitrogen.pool -= n;
    }

    fn growing(&self, id: OrganId) -> bool {
        let i = id.index();
        self.meta[i].expanded.is_none() && self.arena.curve[i].potential.0 > 0.0
    }

    /// Each growing organ's deficit to its initial concentration, by id.
    fn deficits(&self) -> Vec<(usize, f64)> {
        let np = &self.params.nitrogen;
        self.arena
            .alive_ids()
            .filter(|id| self.growing(*id))
            .filter_map(|id| {
                let i = id.index();
                let d = np.initial_of(self.arena.kind[i]) * self.arena.primary(id).0
                    - self.arena.nitrogen[i].0;
                (d > 0.0).then_some((i, d))
            })
            .collect()
    }

    /// Today's nitrogen demand: the growing organs' deficits plus what today's growth
    /// needed, so the pool can feed tomorrow's growth.
    #[must_use]
    pub fn nitrogen_demand(&self) -> GramsN {
        GramsN(self.deficits().iter().map(|x| x.1).sum::<f64>()) + self.nitrogen.today.growth
    }

    /// The pool without today's fixation (fixation is the last source, decision 0020).
    fn pool_without_fixation(&self) -> GramsN {
        (self.nitrogen.pool - self.nitrogen.today.fixed).max(GramsN(0.0))
    }

    /// The plant's request in the soil's joint nitrogen uptake: the demand the pool
    /// does not cover.
    #[must_use]
    pub fn nitrogen_request(&self) -> NitrogenRequest<'_> {
        NitrogenRequest {
            demand: (self.nitrogen_demand() - self.pool_without_fixation()).max(GramsN(0.0)),
            roots: &self.roots,
            capacity_per_length: self.params.nitrogen.uptake_capacity,
        }
    }

    /// Hourly: tops growing organs up to their initial concentration from the pool,
    /// all by the same share when the pool is short.
    pub(crate) fn top_up_hour(&mut self) {
        let deficits = self.deficits();
        let want: f64 = deficits.iter().map(|d| d.1).sum();
        let today = &mut self.nitrogen.today;
        today.growth.0 += (want - today.residual.0).max(0.0);
        let give = want.min(self.nitrogen.pool.0.max(0.0));
        if want > 0.0 && give > 0.0 {
            let f = give / want;
            for &(i, d) in &deficits {
                self.arena.nitrogen[i].0 += d * f;
            }
            self.nitrogen.pool.0 -= give;
        }
        self.nitrogen.today.used.0 += give;
        self.nitrogen.today.residual = GramsN((want - give).max(0.0));
    }

    /// The day's nitrogen for a plant alone in its soil: its request, the soil's
    /// uptake, then [`Plant::nitrogen_day`]. The world uses the joint uptake instead.
    ///
    /// # Errors
    /// [`ArchError`] from the soil or an internal inconsistency.
    pub fn nitrogen_alone(&mut self, soil: &mut SoilGrid, now: Hours) -> Result<(), ArchError> {
        let got = soil.take_up_nitrogen(&[self.nitrogen_request()])?;
        let u = got
            .first()
            .map_or(GramsN(0.0), ps_soil::NitrogenUptake::total);
        self.nitrogen_day(u, soil, now)
    }

    /// The daily nitrogen step (decision 0020): `uptake` is what the soil gave this
    /// plant; see the module docs for the order.
    ///
    /// # Errors
    /// [`ArchError`] from an internal inconsistency.
    pub fn nitrogen_day(
        &mut self,
        uptake: GramsN,
        soil: &SoilGrid,
        now: Hours,
    ) -> Result<(), ArchError> {
        let pool_nonfix = self.pool_without_fixation();
        let demand = self.nitrogen_demand().0;
        self.nitrogen.pool += uptake;
        self.nitrogen.ledger.uptake += uptake;
        self.nitrogen.today.uptake = uptake;
        self.nitrogen.today.demand = GramsN(demand);
        // Mining covers what the pool (without today's fixation) and the soil leave.
        let need = (demand - pool_nonfix.0 - uptake.0).max(0.0);
        let mined = if need > 0.0 { self.mine(need) } else { 0.0 };
        self.nitrogen.today.mined = GramsN(mined);
        let fixation_need = (need - mined).max(0.0);
        self.nitrogen.today.fixation_need = GramsN(fixation_need);
        // Top up the growing organs now; the rest stays in the pool for tomorrow.
        // (Not counted in the hourly record that forecasts tomorrow.)
        self.nitrogen.today.unmet = GramsN(self.deficits().iter().map(|x| x.1).sum());
        let (used, growth) = (self.nitrogen.today.used, self.nitrogen.today.growth);
        self.top_up_hour();
        self.nitrogen.today.used = used;
        self.nitrogen.today.growth = growth;
        self.set_fixation(fixation_need);
        self.nodulate(fixation_need, soil, now)?;
        self.drop_mined_leaves(now)?;
        self.nitrogen.status = self.nitrogen_status();
        self.assert_nitrogen();
        Ok(())
    }

    /// Mines up to `need` from organs that have stopped growing; returns what it took.
    fn mine(&mut self, need: f64) -> f64 {
        let np = &self.params.nitrogen;
        if self.phenology.stage < Stage::Emerged {
            return 0.0;
        }
        let filling = self
            .arena
            .alive_of(OrganKind::SeedInPod)
            .any(|id| self.growing(id));
        let rate = np.mobilization_rate
            * if filling {
                1.0
            } else {
                np.vegetative_mobilization
            };
        let top = self
            .arena
            .alive_of(OrganKind::Leaf)
            .map(|id| self.meta[id.index()].light_index)
            .fold(0.0_f64, f64::max);
        let caps: Vec<(usize, f64)> = self
            .arena
            .alive_ids()
            .filter(|id| !self.growing(*id))
            .filter_map(|id| {
                let i = id.index();
                let kind = self.arena.kind[i];
                if matches!(
                    kind,
                    OrganKind::Nodule | OrganKind::Seed | OrganKind::SeedInPod
                ) {
                    return None;
                }
                let mobile =
                    self.arena.nitrogen[i].0 - np.floor_of(kind) * self.arena.primary(id).0;
                if mobile <= 0.0 {
                    return None;
                }
                // Low-light leaves give first (decision 0020).
                let weight = if kind == OrganKind::Leaf && top > 0.0 {
                    (2.0 * (1.0 - self.meta[i].light_index / top)).clamp(0.1, 1.0)
                } else {
                    1.0
                };
                Some((i, rate * weight * mobile))
            })
            .collect();
        let total: f64 = caps.iter().map(|c| c.1).sum();
        if total <= 0.0 {
            return 0.0;
        }
        let take = need.min(total);
        let f = take / total;
        for (i, c) in caps {
            self.arena.nitrogen[i].0 -= c * f;
            if self.arena.kind[i] == OrganKind::Leaf {
                self.meta[i].mined = true;
            }
        }
        self.nitrogen.pool.0 += take;
        take
    }

    /// Fixation capacity per nodule dry mass today, g N g⁻¹ d⁻¹.
    fn fixation_per_mass(&self) -> f64 {
        let np = &self.params.nitrogen;
        let d = &self.today;
        let t_mean = if d.hours > 0 {
            d.t_sum / f64::from(d.hours)
        } else {
            20.0
        };
        let drought = (stress_factor(d.ftsw_min, self.params.ftsw_expansion)
            / np.fixation_drought.max(1e-9))
        .clamp(0.0, 1.0);
        np.nodule_activity * np.fixation_temperature(t_mean) * drought
    }

    /// Sets each nodule's fixation rate for tomorrow so the nodules together cover
    /// `need`, up to their capacity.
    fn set_fixation(&mut self, need: f64) {
        let per_mass = self.fixation_per_mass();
        let nodules: Vec<OrganId> = self.arena.alive_of(OrganKind::Nodule).collect();
        let cap: f64 = nodules
            .iter()
            .map(|id| self.arena.structural(*id).0 * per_mass)
            .sum();
        let u = if cap > 0.0 {
            (need / cap).min(1.0)
        } else {
            0.0
        };
        for id in nodules {
            self.meta[id.index()].fixation = self.arena.structural(id).0 * per_mass * u;
        }
    }

    /// Hourly fixation: each nodule pays for its rate from its own reserve (never into
    /// debt) and the fixed nitrogen joins the pool.
    pub(crate) fn fix_hour(&mut self) {
        let cost = self.params.nitrogen.fixation_cost;
        if cost <= 0.0 {
            return;
        }
        let nodules: Vec<OrganId> = self.arena.alive_of(OrganKind::Nodule).collect();
        for id in nodules {
            let i = id.index();
            let want = self.meta[i].fixation * cost / 24.0;
            let c = want.min(self.arena.reserve[i].0.max(0.0));
            if c <= 0.0 {
                continue;
            }
            self.arena.reserve[i].0 -= c;
            self.ledger.fixation += GramsC(c);
            let n = GramsN(c / cost);
            self.nitrogen.pool += n;
            self.nitrogen.ledger.fixed += n;
            self.nitrogen.today.fixed += n;
            self.nitrogen.today.fixation_carbon += GramsC(c);
        }
    }

    /// Starts nodule clusters on shallow roots when fixation needs more nodule mass.
    fn nodulate(&mut self, need: f64, soil: &SoilGrid, now: Hours) -> Result<(), ArchError> {
        let np = self.params.nitrogen.clone();
        if need <= 0.0
            || np.nodule_activity <= 0.0
            || !soil.has_rhizobia()
            || self.phenology.stage < Stage::Emerged
            || self.phenology.stage >= Stage::Mature
        {
            return Ok(());
        }
        let per_mass = self.fixation_per_mass();
        if per_mass <= 0.0 {
            return Ok(());
        }
        let nodules: Vec<OrganId> = self.arena.alive_of(OrganKind::Nodule).collect();
        let potential: f64 = nodules
            .iter()
            .map(|id| self.arena.curve[id.index()].potential.0)
            .sum();
        let target = need / per_mass;
        if target <= potential {
            return Ok(());
        }
        let step = if potential > 0.0 {
            np.nodule_relative_growth * potential
        } else {
            np.nodule_initial_mass.0
        };
        // The same cells the roots take up from (WP24 review).
        let ppm = soil.root_zone_no3_ppm(&self.roots)?;
        let add = (target - potential).min(step) * np.nitrate_factor(ppm);
        let per = np.nodule_mass.0.max(1e-9);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let count = (libm::ceil(add / per) as usize).min(MAX_NEW_NODULES);
        if count == 0 {
            return Ok(());
        }
        // Shallow root segments, those with the fewest nodules first, then by id.
        let mut load: Vec<(usize, OrganId)> = self
            .arena
            .alive_of(OrganKind::Root)
            .filter(|id| {
                self.positions
                    .get(id.index())
                    .is_some_and(|p| p.0[2] >= -np.nodule_depth.0)
            })
            .map(|id| {
                let n = nodules
                    .iter()
                    .filter(|x| self.arena.parent[x.index()] == Some(id))
                    .count();
                (n, id)
            })
            .collect();
        if load.is_empty() {
            return Ok(());
        }
        let c = curve(np.nodule_expansion, np.nodule_mass)?;
        for k in 0..count {
            load.sort_by_key(|x| (x.0, x.1));
            let parent = load[0].1;
            load[0].0 += 1;
            let mut s = Spawn::new(OrganKind::Nodule, Role::Nodule, parent, c);
            s.attach = 0.5;
            s.azimuth = f64::from(u32::try_from(k).unwrap_or(0)) * 2.4;
            self.spawn(&s, now)?;
        }
        Ok(())
    }

    /// Drops leaves mined below the drop threshold.
    fn drop_mined_leaves(&mut self, now: Hours) -> Result<(), ArchError> {
        let np = self.params.nitrogen.clone();
        let (init, floor) = (np.initial_of(OrganKind::Leaf), np.floor_of(OrganKind::Leaf));
        let leaves: Vec<OrganId> = self
            .arena
            .alive_of(OrganKind::Leaf)
            .filter(|id| {
                let i = id.index();
                let w = self.arena.structural(*id).0;
                self.meta[i].mined
                    && self.arena.nitrogen[i].0 - floor * w
                        < np.leaf_drop_mobile * (init - floor) * w
            })
            .collect();
        for id in leaves {
            self.shed_organ(id, now)?;
        }
        Ok(())
    }

    /// The share of their mobile nitrogen the growing leaves and stems have.
    fn nitrogen_status(&self) -> f64 {
        let np = &self.params.nitrogen;
        let (mut have, mut full) = (0.0, 0.0);
        for id in self.arena.alive_ids().filter(|id| self.growing(*id)) {
            let i = id.index();
            let kind = self.arena.kind[i];
            if !matches!(kind, OrganKind::Leaf | OrganKind::Internode) {
                continue;
            }
            let w = self.arena.primary(id).0;
            have += (self.arena.nitrogen[i].0 - np.floor_of(kind) * w).max(0.0);
            full += (np.initial_of(kind) - np.floor_of(kind)) * w;
        }
        if full > 0.0 {
            (have / full).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Books the nitrogen of organs that just died: their mobile part returns to the
    /// pool, the rest is litter.
    pub(crate) fn book_dead_nitrogen(&mut self, deaths: &[Death]) {
        let np = &self.params.nitrogen;
        for d in deaths {
            let i = d.id.index();
            let n = self.arena.nitrogen[i];
            let keep = np.floor_of(self.arena.kind[i]) * self.arena.primary(d.id).0;
            let back = (n.0 - keep).max(0.0);
            self.nitrogen.pool.0 += back;
            self.nitrogen.ledger.litter.0 += n.0 - back;
            self.arena.nitrogen[i] = GramsN(0.0);
        }
    }

    /// Ends the day's nitrogen record.
    pub(crate) fn close_nitrogen_day(&mut self) {
        self.nitrogen.last = self.nitrogen.today;
        self.nitrogen.today = NitrogenDay {
            residual: self.nitrogen.last.residual,
            ..NitrogenDay::default()
        };
    }
}
