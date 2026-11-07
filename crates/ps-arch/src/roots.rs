//! CRootBox-style root system (§3.7; decision 0013): axes of each root type appear at
//! their emergence time, grow as chains of segments, bend by gravitropism and deflect
//! along the pot wall; laterals are lumped into cohorts, one per parent segment.
//!
//! Plasticity (R2b, decision 0043): a tip in dry soil slows, a dry stretch of parent
//! root forms no laterals, laterals proliferate where nitrate or water is richer than
//! both the soil's mean and the mean the roots sit in, a cohort turns toward the wetter
//! side of its parent, and a
//! tip leans weakly up the water gradient. Every rule reads the cell a tip or segment
//! is in at the daily root step; none draws random numbers.

use ps_graph::{OrganId, OrganKind, SpecificSize};
use ps_rng::Purpose;
use ps_soil::SoilGrid;
use ps_species::RootOrigin;
use ps_units::{DeficitHours, DegreeDays, GramsDM, Hours};

use crate::geometry::{V3, add, deflect, from_azimuth, norm, soil_cell, unit};
use crate::{ArchError, PRIMORDIUM, Plant, Role, RootAxis, RootSpan, Spawn, curve};

/// Golden angle, rad: the azimuth step between successive lateral cohorts.
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;
/// Standard deviation of the random component added to a root direction per segment.
const TROPISM_NOISE: f64 = 0.15;
/// Hypoxia of a parent's cell at which its laterals wait for air (R5b, decision 0048;
/// "~": half the oxygen deficit, 6% air-filled porosity).
const LATERAL_HYPOXIA: f64 = 0.5;

impl Plant {
    /// Daily step 3, root part: axes of each seed-borne root type appear once thermal
    /// time since germination passes the type's emergence time. A plant that lost an
    /// axis tip to waterlogged soil grows a new set of hypocotyl-borne roots once the soil
    /// at its collar is no longer half hypoxic (R5b, decision 0048: adventitious roots,
    /// the bean's and wheat's answer to waterlogging), if that soil was hypoxic in the
    /// meantime: a flooded stem base roots, a tip lost in a wet layer deep down does not
    /// (R5c, decision 0049).
    pub(crate) fn root_initiation(&mut self, now: Hours, soil: &SoilGrid) -> Result<(), ArchError> {
        if self.phenology.stage < ps_physio::Stage::Germinated {
            return Ok(());
        }
        let since = self.phenology.since_germination.0;
        let collar = soil_cell(soil, self.collar_position);
        let regrow = self.drowned
            && self.collar_flooded
            && collar.is_none_or(|c| soil.hypoxia(c) < LATERAL_HYPOXIA);
        if regrow {
            // One flood, one set: a tip lost later licenses another only if the collar
            // is hypoxic again.
            self.drowned = false;
            self.collar_flooded = false;
            self.collar_hypoxic_at = None;
        }
        let mut whorl = 0u32;
        for t in 0..self.params.roots.len() {
            let rp = self.params.roots[t].clone();
            if rp.origin == RootOrigin::Basal {
                whorl += 1;
            }
            if rp.order > 0 {
                continue;
            }
            let again = regrow && rp.origin == RootOrigin::Hypocotyl && self.root_types_started[t];
            if !again && (self.root_types_started[t] || since < rp.emergence.0) {
                continue;
            }
            self.root_types_started[t] = true;
            self.start_axes(t, whorl, now, soil)?;
        }
        Ok(())
    }

    /// The axes of seed-borne root type `t`, `whorl` the basal whorls counted so far.
    fn start_axes(
        &mut self,
        t: usize,
        whorl: u32,
        now: Hours,
        soil: &SoilGrid,
    ) -> Result<(), ArchError> {
        let rp = self.params.roots[t].clone();
        let count = rp.count.max(1);
        for k in 0..count {
            let kf = f64::from(k);
            let cf = f64::from(count);
            let (parent, attach, dir, base) = match rp.origin {
                RootOrigin::Radicle | RootOrigin::Lateral => {
                    (self.collar, 1.0, [0.0, 0.0, -1.0], self.collar_position)
                }
                RootOrigin::Basal => {
                    let offset = if whorl.is_multiple_of(2) {
                        core::f64::consts::PI / cf
                    } else {
                        0.0
                    };
                    let az = 2.0 * core::f64::consts::PI * kf / cf + offset;
                    let elev = -(core::f64::consts::FRAC_PI_2 - rp.insertion.0);
                    (
                        self.collar,
                        1.0,
                        from_azimuth(az, elev),
                        self.collar_position,
                    )
                }
                RootOrigin::Hypocotyl => {
                    let az = GOLDEN_ANGLE * kf;
                    let elev = -(core::f64::consts::FRAC_PI_2 - rp.insertion.0);
                    let total = self.depth.0 + self.params.hypocotyl_above.0;
                    let buried = (self.depth.0 / total.max(1e-9)).clamp(0.0, 1.0);
                    let attach = buried * (0.2 + 0.7 * (kf + 0.5) / cf);
                    let base = add(self.collar_position, [0.0, 0.0, 1.0], attach * total);
                    (self.hypocotyl, attach, from_azimuth(az, elev), base)
                }
            };
            let ax = RootAxis {
                kind: t,
                segments: Vec::new(),
                length: 0.0,
                direction: dir,
                multiplicity: 1.0,
                done: false,
                laterals: 0,
                end: base,
                base,
            };
            self.root_axes.push(ax);
            let a = self.root_axes.len() - 1;
            self.root_segment(a, parent, attach, dir, DegreeDays(0.0), now, soil)?;
        }
        Ok(())
    }

    /// Adds the next segment to root axis `a` at its planned end, along `dir`,
    /// deflecting at the pot wall; marks the axis done when it is full length or boxed
    /// in. The segment's expansion clock starts at `-wait`, so it begins to grow when
    /// the segment before it finishes (R2a): the tip moves at the root's elongation
    /// rate, not one segment per daily step.
    #[allow(clippy::too_many_arguments)]
    fn root_segment(
        &mut self,
        a: usize,
        parent: OrganId,
        attach: f64,
        dir: V3,
        wait: DegreeDays,
        now: Hours,
        soil: &SoilGrid,
    ) -> Result<(), ArchError> {
        let rp = self.params.roots[self.root_axes[a].kind].clone();
        let left = rp.max_length.0 - self.planned_length(a);
        let len = self.params.root_segment.0.min(left);
        if len < 1e-4 {
            self.root_axes[a].done = true;
            return Ok(());
        }
        let base = self.root_axes[a].end;
        // The segment reaches its potential plus the primordium it starts as.
        let reach = len * (1.0 + PRIMORDIUM);
        let Some(dir) = wall(soil, base, dir, reach) else {
            self.root_axes[a].done = true;
            return Ok(());
        };
        let mult = self.root_axes[a].multiplicity;
        // A segment formed in hard soil is wider (R5c): its length costs the square of
        // the widening more.
        let wide = self.impeded_width(soil, base);
        let specific = rp.specific_length.0 / (wide * wide);
        let potential = GramsDM(len * mult / specific);
        let duration = DegreeDays(len / rp.elongation_per_cd.max(1e-12));
        let mut s = Spawn::new(
            OrganKind::Root,
            Role::Root,
            parent,
            curve(duration, potential)?,
        );
        s.order = rp.order;
        s.specific = SpecificSize::Length(ps_units::MPerG(specific));
        s.direction = dir;
        s.attach = attach;
        s.multiplicity = mult;
        let id = self.spawn(&s, now)?;
        self.arena.expansion_age[id.index()] = DegreeDays(-wait.0.max(0.0));
        let start = self.planned_length(a);
        let ax = &mut self.root_axes[a];
        ax.segments.push((id, start));
        ax.direction = dir;
        ax.end = add(base, dir, reach);
        Ok(())
    }

    /// Length of axis `a` once its growing segments finish: realized lengths of the
    /// finished ones, the most the rest can reach.
    fn planned_length(&self, a: usize) -> f64 {
        let ax = &self.root_axes[a];
        ax.segments.last().map_or(0.0, |&(id, start)| {
            start + self.segment_length(id, ax.multiplicity)
        })
    }

    /// One root's length along segment `id`: realized once it has finished expanding,
    /// before that the most it can reach, its length now plus the rest of its potential.
    fn segment_length(&self, id: OrganId, mult: f64) -> f64 {
        let a = &self.arena;
        let i = id.index();
        let now = a.size[i].length.0;
        let c = &a.curve[i];
        let rest = if c.is_complete(a.expansion_age[i]) {
            0.0
        } else {
            match a.specific[i] {
                SpecificSize::Length(sl) => {
                    c.potential.0 * (1.0 - c.fraction(a.expansion_age[i])) * sl.0
                }
                _ => 0.0,
            }
        };
        (now + rest) / mult.max(1.0)
    }

    /// Restates axis `a`'s segment starts from the lengths its segments reached, its
    /// finished length (segments finished from the base on), and its planned end: a
    /// segment that fell short of its potential moves every later one back, as the
    /// layout draws them, so the pot-wall rule sees where the tip really is.
    fn settle_axis(&mut self, a: usize) {
        let mult = self.root_axes[a].multiplicity;
        let mut s = 0.0;
        let mut finished = 0.0;
        let mut contiguous = true;
        let mut end = self.root_axes[a].base;
        let n = self.root_axes[a].segments.len();
        for k in 0..n {
            let id = self.root_axes[a].segments[k].0;
            self.root_axes[a].segments[k].1 = s;
            let i = id.index();
            let l = self.segment_length(id, mult);
            s += l;
            end = add(end, self.arena.orientation[i].direction, l);
            if contiguous && self.arena.curve[i].is_complete(self.arena.expansion_age[i]) {
                finished = s;
            } else {
                contiguous = false;
            }
        }
        self.root_axes[a].length = finished;
        if n > 0 {
            self.root_axes[a].end = end;
        }
    }

    /// Daily step 5 (§3.7): each growing root axis queues its next segment behind
    /// the tip, bent by gravitropism and noise, to start when the tip finishes; axes
    /// past their apical zone emit one lateral cohort per segment. `soil_means` is the
    /// soil's [`SoilGrid::means`] if the world computed it already.
    pub(crate) fn root_elongation(
        &mut self,
        now: Hours,
        soil: &SoilGrid,
        soil_means: Option<(f64, f64)>,
    ) -> Result<(), ArchError> {
        let child = |order: u8| {
            self.params
                .roots
                .iter()
                .position(|r| r.origin == RootOrigin::Lateral && r.order == order + 1)
        };
        let lateral_type = child(0);
        let fine_type = child(1);
        let means = self.root_zone_means(soil, soil_means);
        let hydrotropism = self.params.root_plasticity.hydrotropism;
        for a in 0..self.root_axes.len() {
            // A finished axis that does not branch (a lateral cohort) has nothing to do.
            if self.root_axes[a].done
                && self.params.roots[self.root_axes[a].kind].branch_spacing.0 <= 0.0
            {
                continue;
            }
            let Some(&(tip, _)) = self.root_axes[a].segments.last() else {
                continue;
            };
            let lost = !self.arena.alive[tip.index()];
            if lost {
                self.root_axes[a].done = true;
            }
            let rp = self.params.roots[self.root_axes[a].kind].clone();
            if rp.order > 0 {
                // A lateral cohort: its fine roots once it has finished.
                if let Some(ft) = fine_type.filter(|_| rp.order == 1 && !lost) {
                    self.fine_roots(a, ft, now, soil, means)?;
                }
                continue;
            }
            self.settle_axis(a);
            // Laterals behind the apical zone, one cohort per parent segment; up to the
            // tip once the tip is lost (R5c).
            if let Some(lt) = lateral_type.filter(|_| rp.branch_spacing.0 > 0.0) {
                self.laterals(a, lt, now, soil, means)?;
            }
            let ax = self.root_axes[a].clone();
            if ax.done {
                continue;
            }
            let i = tip.index();
            let age = self.arena.expansion_age[i].0;
            if age < 0.0 {
                // The next segment is already queued.
                continue;
            }
            if self.planned_length(a) >= rp.max_length.0 - 1e-6 {
                if self.arena.curve[i].is_complete(self.arena.expansion_age[i]) {
                    self.root_axes[a].done = true;
                }
                continue;
            }
            let wait = DegreeDays((self.arena.curve[i].duration.0 - age).max(0.0));
            let mut rng = self.stream(tip, now, Purpose::Tropism);
            let noise = [
                TROPISM_NOISE * rng.standard_normal(),
                TROPISM_NOISE * rng.standard_normal(),
                TROPISM_NOISE * rng.standard_normal(),
            ];
            // Hydrotropism (R2b): a weak pull up the water gradient at the tip.
            let wet = water_gradient(soil, ax.end);
            let dir = unit(add(
                add(
                    add(ax.direction, [0.0, 0.0, -1.0], rp.gravitropism * 0.2),
                    noise,
                    1.0,
                ),
                wet,
                hydrotropism,
            ));
            self.root_segment(a, tip, 1.0, dir, wait, now, soil)?;
        }
        Ok(())
    }

    fn laterals(
        &mut self,
        a: usize,
        lt: usize,
        now: Hours,
        soil: &SoilGrid,
        means: Option<(f64, f64)>,
    ) -> Result<(), ArchError> {
        let seg = self.params.root_segment.0;
        let parent_type = self.params.roots[self.root_axes[a].kind].clone();
        // Laterals form more densely behind a tip that hard soil slows (R5c).
        let denser = self.impeded_branching(soil, self.root_axes[a].end);
        // The unbranched zone behind a tip is what it grows while its laterals emerge,
        // so it shortens with the tip's speed in hard soil, and is gone with the tip
        // (R5c, decision 0049).
        let zone = parent_type.apical_zone.0 * self.apical_factor(a, soil);
        loop {
            let ax = &self.root_axes[a];
            let k = ax.laterals;
            // Completed length behind the apical zone.
            let reach = ax.length - zone;
            let start = f64::from(k) * seg;
            if start + seg > reach + 1e-9 {
                return Ok(());
            }
            let mid = start + 0.5 * seg;
            let Some(&(host, host_start)) = ax.segments.iter().rev().find(|(_, s0)| *s0 <= mid)
            else {
                return Ok(());
            };
            self.root_axes[a].laterals += 1;
            if !self.arena.alive[host.index()] {
                continue;
            }
            let h = host.index();
            let host_len =
                (self.arena.size[h].length.0 / self.meta[h].multiplicity.max(1.0)).max(1e-9);
            let attach = ((mid - host_start) / host_len).clamp(0.0, 1.0);
            let mult = seg / parent_type.branch_spacing.0 * denser;
            if !self.cohort(host, attach, lt, k, mult, now, soil, means)? {
                // Hypoxic soil: this stretch branches once it has air again (R5b).
                self.root_axes[a].laterals -= 1;
                return Ok(());
            }
        }
    }

    /// Second-order laterals on lateral cohort `a` (R2a, decision 0037): once the
    /// cohort organ has finished, one fine-root cohort standing for every fine root
    /// along its length behind the apical zone, `m₂ = m₁ (L₁ − apical) / spacing`,
    /// attached in the middle of that stretch.
    fn fine_roots(
        &mut self,
        a: usize,
        ft: usize,
        now: Hours,
        soil: &SoilGrid,
        means: Option<(f64, f64)>,
    ) -> Result<(), ArchError> {
        let ax = &self.root_axes[a];
        let Some(&(host, _)) = ax.segments.first() else {
            return Ok(());
        };
        let h = host.index();
        if ax.laterals > 0
            || !self.arena.alive[h]
            || !self.arena.curve[h].is_complete(self.arena.expansion_age[h])
        {
            return Ok(());
        }
        let k = u32::try_from(a).unwrap_or(0);
        let m1 = ax.multiplicity.max(1.0);
        let pt = &self.params.roots[ax.kind];
        let (apical, spacing) = (pt.apical_zone.0, pt.branch_spacing.0);
        self.root_axes[a].laterals = 1;
        let len1 = (self.arena.size[h].length.0 / m1).max(1e-9);
        let branched = len1 - apical;
        if branched <= 0.0 {
            return Ok(());
        }
        let mult = m1 * branched / spacing;
        if !self.cohort(host, 0.5 * branched / len1, ft, k, mult, now, soil, means)? {
            // Hypoxic soil: the fine roots form once it has air again (R5b).
            self.root_axes[a].laterals = 0;
        }
        Ok(())
    }

    /// One lateral cohort of type `lt` on root organ `host` at `attach` (a fraction of
    /// its length): a single straight organ of the lateral's full length standing for
    /// `mult` roots, aimed by the golden angle `k`. Where the parent's cell is drier
    /// than the xerobranching threshold none forms; where it is richer in nitrate or
    /// water than the root zone (`means`) the cohort stands for more roots (R2b).
    /// Where the parent's cell is at least half hypoxic none forms yet, and `false`
    /// asks the caller to try again another day (R5b).
    #[allow(clippy::too_many_arguments)]
    fn cohort(
        &mut self,
        host: OrganId,
        attach: f64,
        lt: usize,
        k: u32,
        mult: f64,
        now: Hours,
        soil: &SoilGrid,
        means: Option<(f64, f64)>,
    ) -> Result<bool, ArchError> {
        let lp = self.params.roots[lt].clone();
        let h = host.index();
        let host_len = (self.arena.size[h].length.0 / self.meta[h].multiplicity.max(1.0)).max(1e-9);
        let host_dir = self.arena.orientation[h].direction;
        let mut dir = deflect(host_dir, lp.insertion.0, GOLDEN_ANGLE * f64::from(k));
        // A cohort aimed upward is mirrored down: it stands for laterals all around
        // the parent, and the upward ones turn down within a few millimeters
        // (decision 0013).
        dir[2] = -dir[2].abs();
        // A cohort is one straight organ of the lateral's full length: keep its tip in
        // active soil, as an axis segment is kept (decision 0028). Try the drawn
        // direction, the pot-wall rule, the next golden-angle turns, then level toward
        // the plant's collar; where none fits (a pot's bottom edge) there is no cohort
        // (decision 0037).
        let host_base = self.positions.get(h).map_or(self.collar_position, |p| p.0);
        let base = add(host_base, host_dir, attach * host_len);
        let plastic = &self.params.root_plasticity;
        let cell = soil_cell(soil, base);
        // No laterals from a parent in soil at least half hypoxic until it has air
        // again (R5b).
        if self.params.waterlogging.fine_root_hours < u32::MAX
            && cell.is_some_and(|c| soil.hypoxia(c) >= LATERAL_HYPOXIA)
        {
            return Ok(false);
        }
        // Xerobranching: no laterals from a parent in dry soil, and none later.
        if cell.is_some_and(|c| soil.cell_ftsw(c) < plastic.xerobranching) {
            return Ok(true);
        }
        let len = lp.max_length.0;
        // The cohort reaches its potential plus the primordium it starts as.
        let reach = len * (1.0 + PRIMORDIUM);
        let inside = |d: V3| soil_cell(soil, add(base, d, reach)).is_some();
        let turn = |j: u32| {
            let mut d = deflect(host_dir, lp.insertion.0, GOLDEN_ANGLE * f64::from(k + j));
            d[2] = -d[2].abs();
            d
        };
        // Hydropatterning: the first of the golden-angle candidates whose tip cell
        // holds at least their median available water, where they differ.
        let wetter = if plastic.hydropatterning {
            let cands: Vec<(V3, f64)> = (0..8)
                .map(turn)
                .filter_map(|d| {
                    soil_cell(soil, add(base, d, reach)).map(|c| (d, soil.cell_ftsw(c)))
                })
                .collect();
            let mut ws: Vec<f64> = cands.iter().map(|c| c.1).collect();
            ws.sort_by(f64::total_cmp);
            match (ws.first(), ws.last()) {
                (Some(lo), Some(hi)) if hi - lo > 1e-9 => {
                    let median = ws[(ws.len() - 1) / 2];
                    cands.iter().find(|c| c.1 >= median).map(|c| c.0)
                }
                _ => None,
            }
        } else {
            None
        };
        let inward = [
            self.collar_position[0] - base[0],
            self.collar_position[1] - base[1],
            0.0,
        ];
        let found = wetter
            .or_else(|| wall(soil, base, dir, reach))
            .or_else(|| (1..8).map(turn).find(|d| inside(*d)))
            .or_else(|| {
                (libm::hypot(inward[0], inward[1]) > 1e-6)
                    .then(|| unit(inward))
                    .filter(|d| inside(*d))
            });
        let Some(d) = found else {
            return Ok(true);
        };
        dir = d;
        // Proliferation: the cohort is one straight organ, so it stands for more roots by
        // the mean factor of the cells along its path (R2b, decision 0043).
        let mult = mult * path_proliferation(plastic, soil, base, dir, reach, means);
        let wide = self.impeded_width(soil, base);
        let specific = lp.specific_length.0 / (wide * wide);
        let potential = GramsDM(lp.max_length.0 * mult / specific);
        let duration = DegreeDays(lp.max_length.0 / lp.elongation_per_cd.max(1e-12));
        let mut s = Spawn::new(
            OrganKind::Root,
            Role::Root,
            host,
            curve(duration, potential)?,
        );
        s.order = lp.order;
        s.specific = SpecificSize::Length(ps_units::MPerG(specific));
        s.direction = dir;
        s.attach = attach;
        s.multiplicity = mult;
        let id = self.spawn(&s, now)?;
        self.root_axes.push(RootAxis {
            kind: lt,
            segments: vec![(id, 0.0)],
            length: lp.max_length.0,
            direction: dir,
            multiplicity: mult,
            done: true,
            laterals: 0,
            end: add(base, dir, reach),
            base,
        });
        Ok(true)
    }
}

impl Plant {
    /// The diameter factor of a root organ formed at `p` (R5c, decision 0049): 1 in a
    /// grid without structure.
    fn impeded_width(&self, soil: &SoilGrid, p: V3) -> f64 {
        let rp = &self.params.root_plasticity;
        if rp.impedance_half <= 0.0 || !soil.has_structure() {
            return 1.0;
        }
        soil_cell(soil, p)
            .and_then(|c| soil.strength(c))
            .map_or(1.0, |q| rp.thickening(q))
    }

    /// The factor on axis `a`'s unbranched apical zone (R5c): 0 once its tip is lost;
    /// the tip's impedance factor where densities differ; else 1.
    fn apical_factor(&self, a: usize, soil: &SoilGrid) -> f64 {
        let ax = &self.root_axes[a];
        if ax
            .segments
            .last()
            .is_some_and(|&(id, _)| !self.arena.alive[id.index()])
        {
            return 0.0;
        }
        let rp = &self.params.root_plasticity;
        if rp.impedance_half <= 0.0 || !soil.has_structure() {
            return 1.0;
        }
        soil_cell(soil, ax.end)
            .and_then(|c| soil.strength(c))
            .map_or(1.0, |q| rp.impedance(q))
    }

    /// The factor on the laterals an axis forms while its tip at `tip` is slowed by
    /// hard soil (R5c): `1 + k (1 − f)` with `f` the tip's impedance factor, when it is
    /// below 1.
    fn impeded_branching(&self, soil: &SoilGrid, tip: V3) -> f64 {
        let rp = &self.params.root_plasticity;
        if rp.impedance_half <= 0.0 || rp.impeded_branching <= 0.0 || !soil.has_structure() {
            return 1.0;
        }
        soil_cell(soil, tip)
            .and_then(|c| soil.strength(c))
            .map_or(1.0, |q| {
                1.0 + rp.impeded_branching * (1.0 - rp.impedance(q)).max(0.0)
            })
    }

    /// The levels a cell's richness is judged against (R2b, decision 0043): relative
    /// available water and nitrate (mg N per kg soil), each the larger of the soil's
    /// mean over its active cells and the mean the roots sit in (weighted by root
    /// length); `None` without roots. The soil's mean keeps the roots' own depletion
    /// from making every fresh cell look rich; the roots' mean keeps nitrate the water
    /// carried to them, and left there, from looking rich to roots already in it. `None`
    /// also when proliferation is off. `soil_means` is the soil's mean if already known.
    fn root_zone_means(
        &self,
        soil: &SoilGrid,
        soil_means: Option<(f64, f64)>,
    ) -> Option<(f64, f64)> {
        let rp = &self.params.root_plasticity;
        if rp.proliferation_nitrate <= 0.0 && rp.proliferation_water <= 0.0 {
            return None;
        }
        let (mut l, mut w, mut n) = (0.0_f64, 0.0_f64, 0.0_f64);
        for &(i, len) in self.roots.cells() {
            if i < soil.len() {
                l += len.0;
                w += len.0 * soil.cell_ftsw(i);
                n += len.0 * soil.no3_ppm(i);
            }
        }
        if l <= 0.0 {
            return None;
        }
        let (sw, sn) = soil_means.unwrap_or_else(|| soil.means());
        Some((sw.max(w / l), sn.max(n / l)))
    }

    /// Daily step 4 (R5b, decision 0048): a root organ dies when more than half its
    /// length lies in cells whose hypoxic hours have reached the species' death time,
    /// fine-root cohorts (order 2) sooner than lateral cohorts and the tip segments of
    /// axes. It is shed as any dead organ, with what it carries; an axis whose tip died
    /// grows no further, and its older segments live on. A lateral that loses its fine
    /// roots grows new ones once its cell has air again, and a plant that loses an axis
    /// tip roots again from its hypocotyl ([`Plant::root_initiation`]).
    pub(crate) fn drown_roots(&mut self, now: Hours, soil: &SoilGrid) -> Result<(), ArchError> {
        let w = self.params.waterlogging;
        if w.fine_root_hours == u32::MAX && w.root_hours == u32::MAX {
            return Ok(());
        }
        // Each root's clock: the cell's count gained since the last look, or the whole
        // count if the cell had air in between. A root new to a cell started from the
        // cell's count then (`carry_exposure`), so it is not charged the hours the cell
        // was short of air before it arrived.
        for sp in &mut self.root_spans {
            if sp.cell < soil.len() {
                advance_clock(sp, soil.hypoxic_hours(sp.cell));
            }
        }
        let n = self.arena.len();
        // Of an axis only the tip segment dies: the older, thicker parts live on.
        let mut tip = vec![false; n];
        for ax in &self.root_axes {
            if let Some(&(id, _)) = ax.segments.last()
                && let Some(t) = tip.get_mut(id.index())
            {
                *t = true;
            }
        }
        let (mut total, mut drowned) = (vec![0.0_f64; n], vec![0.0_f64; n]);
        for sp in &self.root_spans {
            let o = sp.organ as usize;
            if o >= n || sp.cell >= soil.len() || (self.arena.axis_order[o] == 0 && !tip[o]) {
                continue;
            }
            let limit = if self.arena.axis_order[o] >= 2 {
                w.fine_root_hours
            } else {
                w.root_hours
            };
            total[o] += sp.length.0;
            if sp.exposure.0 >= f64::from(limit) {
                drowned[o] += sp.length.0;
            }
        }
        for o in 0..n {
            if self.arena.alive[o] && total[o] > 0.0 && drowned[o] > 0.5 * total[o] {
                let id = OrganId(u32::try_from(o).unwrap_or(u32::MAX));
                let order = self.arena.axis_order[o];
                let parent = self.arena.parent[o];
                self.shed_organ(id, now)?;
                if order == 0 {
                    // An axis tip lost: the hypocotyl roots again once it has air, if
                    // the flood reached it within the tip's death clock.
                    self.drowned = true;
                    let clock = Hours(u64::from(w.root_hours));
                    self.collar_flooded |= self
                        .collar_hypoxic_at
                        .is_some_and(|t| now.0.saturating_sub(t.0) <= clock.0);
                } else if order >= 2 {
                    // A fine-root cohort lost: its lateral, if alive, makes another
                    // once its cell has air.
                    if let Some(ax) = self
                        .root_axes
                        .iter_mut()
                        .find(|ax| ax.segments.first().map(|s| s.0) == parent && ax.laterals > 0)
                    {
                        ax.laterals = 0;
                    }
                }
            }
        }
        // A stem base flooded since a tip was lost roots again once it has air too.
        if soil_cell(soil, self.collar_position).is_some_and(|c| soil.hypoxia(c) > 0.0) {
            self.collar_hypoxic_at = Some(now);
            self.collar_flooded |= self.drowned;
        }
        Ok(())
    }

    /// Carries each root's waterlogging clock from the spans it had, `before`, to the
    /// spans just rebuilt (R5b's death rule, counted per root, decision 0049): a root
    /// still in a cell keeps its count there; a root that has reached a new cell starts
    /// at 0 there, from the cell's count now. Both lists are sorted by organ, then cell.
    pub(crate) fn carry_exposure(&mut self, before: &[RootSpan], soil: &SoilGrid) {
        let w = self.params.waterlogging;
        if w.fine_root_hours == u32::MAX && w.root_hours == u32::MAX {
            return;
        }
        carry_clocks(before, &mut self.root_spans, |c| soil.hypoxic_hours(c));
    }

    /// Each growing root organ's expansion clock factor for the coming day (R2b): the
    /// relative available water `θ` of its tip's cell over `dry_full`, at most 1 and at
    /// least `dry_floor`, times `1 − h` with `h` that cell's hypoxia for a species that
    /// answers waterlogging (R5b), times the impedance factor of the cell's strength
    /// where densities differ ([`crate::params::RootPlasticity::impedance`], R5c).
    /// Cleared when no factor differs from 1.
    pub(crate) fn set_root_clock(&mut self, soil: &SoilGrid) {
        let rp = &self.params.root_plasticity;
        let (full, floor) = (rp.dry_full, rp.dry_floor);
        let dry = floor < 1.0 && full > 0.0;
        let wet = self.params.waterlogging.fine_root_hours < u32::MAX;
        let hard = rp.impedance_half > 0.0 && soil.has_structure();
        self.root_clock.clear();
        if !dry && !wet && !hard {
            return;
        }
        let mut clock = vec![1.0; self.arena.len()];
        let mut slowed = false;
        for id in self.arena.alive_of(OrganKind::Root) {
            let i = id.index();
            if self.arena.curve[i].is_complete(self.arena.expansion_age[i]) {
                continue;
            }
            let Some(c) = self.positions.get(i).and_then(|p| soil_cell(soil, p.1)) else {
                continue;
            };
            let f_dry = if dry {
                (soil.cell_ftsw(c) / full).clamp(0.0, 1.0).max(floor)
            } else {
                1.0
            };
            let f = if wet {
                f_dry * (1.0 - soil.hypoxia(c))
            } else {
                f_dry
            };
            // Hard soil slows a tip, loose soil lets it run a little faster (R5c).
            let f = match soil.strength(c).filter(|_| hard) {
                Some(q) => f * rp.impedance(q),
                None => f,
            };
            clock[i] = f;
            slowed |= (f - 1.0).abs() > 0.0;
        }
        if slowed {
            self.root_clock = clock;
        }
    }
}

/// Brings one root's waterlogging clock in its cell up to date once the cell's own count
/// has moved from the span's `seen` to `h` (R5c): the gain, or all of `h` if the cell
/// had air in between and started again. A root is never further on than its cell.
fn advance_clock(sp: &mut RootSpan, h: f64) {
    let e = if h < sp.seen.0 {
        h
    } else {
        (sp.exposure.0 + h - sp.seen.0).min(h)
    };
    sp.exposure = DeficitHours(e);
    sp.seen = DeficitHours(h);
}

/// Copies each root's clock from `before` to `now` where the root is still in the same
/// cell; a root new to a cell starts at 0 there from the cell's count, `count(cell)`.
/// Both lists are sorted by organ, then cell.
fn carry_clocks(before: &[RootSpan], now: &mut [RootSpan], count: impl Fn(usize) -> f64) {
    let mut j = 0;
    for sp in now {
        let key = (sp.organ, sp.cell);
        while before.get(j).is_some_and(|b| (b.organ, b.cell) < key) {
            j += 1;
        }
        if let Some(b) = before.get(j).filter(|b| (b.organ, b.cell) == key) {
            sp.exposure = b.exposure;
            sp.seen = b.seen;
        } else {
            sp.exposure = DeficitHours(0.0);
            sp.seen = DeficitHours(count(sp.cell));
        }
    }
}

/// Proliferation (R2b): the factor on the roots a cohort formed in cell `c` stands for,
/// `(1 + a_N r_N)(1 + a_W r_W)` with `r` the cell's nitrate and available water above
/// the levels of [`Plant::root_zone_means`], relative to them, clamped to 0 to 2.
fn proliferation(
    p: &crate::params::RootPlasticity,
    soil: &SoilGrid,
    c: usize,
    means: Option<(f64, f64)>,
) -> f64 {
    let Some((mw, mn)) = means else {
        return 1.0;
    };
    let rel = |v: f64, m: f64| {
        if m > 1e-12 {
            ((v - m) / m).clamp(0.0, 2.0)
        } else {
            0.0
        }
    };
    (1.0 + p.proliferation_nitrate * rel(soil.no3_ppm(c), mn))
        * (1.0 + p.proliferation_water * rel(soil.cell_ftsw(c), mw))
}

/// The mean proliferation factor along a cohort's path from `base` along `dir` for
/// `len`, sampled every half cell as root length is spread over cells; 1 without a
/// root zone or outside the soil.
fn path_proliferation(
    p: &crate::params::RootPlasticity,
    soil: &SoilGrid,
    base: V3,
    dir: V3,
    len: f64,
    means: Option<(f64, f64)>,
) -> f64 {
    if means.is_none() || (p.proliferation_nitrate <= 0.0 && p.proliferation_water <= 0.0) {
        return 1.0;
    }
    let half = 0.5 * soil.cell_size().0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = (libm::ceil(len / half) as u32).max(1);
    let (mut sum, mut count) = (0.0, 0.0);
    for k in 0..n {
        let at = add(base, dir, len * (f64::from(k) + 0.5) / f64::from(n));
        if let Some(c) = soil_cell(soil, at) {
            sum += proliferation(p, soil, c, means);
            count += 1.0;
        }
    }
    if count > 0.0 { sum / count } else { 1.0 }
}

/// The unit gradient of relative available water at `p` (R2b hydrotropism), from the
/// six neighbors of its cell (a missing or inactive one counts as the cell itself),
/// in garden axes (z up); zero where they are equal or outside the soil.
fn water_gradient(soil: &SoilGrid, p: V3) -> V3 {
    let Some(i) = soil_cell(soil, p) else {
        return [0.0; 3];
    };
    let c = soil.cell_of(i);
    let (nx, ny, nz) = soil.dims();
    let f0 = soil.cell_ftsw(i);
    let at = |dx: i64, dy: i64, dz: i64| {
        let step = |v: usize, d: i64, n: usize| {
            let v = i64::try_from(v).ok()? + d;
            usize::try_from(v).ok().filter(|v| *v < n)
        };
        match (step(c.x, dx, nx), step(c.y, dy, ny), step(c.z, dz, nz)) {
            (Some(x), Some(y), Some(z)) => {
                let j = soil.index(ps_soil::Cell { x, y, z });
                if soil.is_active(j) {
                    soil.cell_ftsw(j)
                } else {
                    f0
                }
            }
            _ => f0,
        }
    };
    // Grid z counts down from the surface, so up is z − 1.
    let g = [
        at(1, 0, 0) - at(-1, 0, 0),
        at(0, 1, 0) - at(0, -1, 0),
        at(0, 0, -1) - at(0, 0, 1),
    ];
    let n = norm(g);
    if n > 1e-9 {
        [g[0] / n, g[1] / n, g[2] / n]
    } else {
        [0.0; 3]
    }
}

/// A direction from `base` that keeps a segment of length `len` in active soil: `dir`
/// itself, else (at the pot wall) straight down, else along the wall either way, else
/// none (decision 0013). Roots never grow above the surface.
fn wall(soil: &SoilGrid, base: V3, dir: V3, len: f64) -> Option<V3> {
    let mut d = dir;
    if d[2] > 0.0 {
        d[2] = -d[2];
    }
    let inside = |d: V3| soil_cell(soil, add(base, d, len)).is_some();
    if inside(d) {
        return Some(d);
    }
    let down = [0.0, 0.0, -1.0];
    if inside(down) {
        return Some(down);
    }
    let side = unit([-d[1], d[0], 0.0]);
    let other = [-side[0], -side[1], 0.0];
    [side, other].into_iter().find(|s| inside(*s))
}

#[cfg(test)]
mod tests {
    use super::{advance_clock, carry_clocks, path_proliferation, proliferation, water_gradient};
    use crate::RootSpan;
    use crate::params::RootPlasticity;
    use ps_soil::{Cell, SoilGrid, SoilHour, SoilNitrogen, Strip, Texture};
    use ps_units::{Celsius, DeficitHours, Meters, Millimeters, WPerM2};

    fn span(organ: u32, cell: usize) -> RootSpan {
        RootSpan {
            organ,
            cell,
            length: Meters(0.01),
            exposure: DeficitHours(0.0),
            seen: DeficitHours(0.0),
        }
    }

    /// R5c (Code review of #153): a root's death clock runs from when it reached its
    /// cell, not from when the cell went short of air.
    #[test]
    fn a_root_counts_its_own_hours_short_of_air() {
        // Cell 5 has been short of air for 400 full-deficit hours, cell 6 never.
        let count = |c: usize| if c == 5 { 400.0 } else { 0.0 };
        let mut old = span(1, 5);
        old.exposure = DeficitHours(100.0);
        old.seen = DeficitHours(400.0);
        let before = [old, span(2, 7)];
        // Organ 1 is still in cell 5 and has grown into 6; organ 3 has just reached 5.
        let mut now = [span(1, 5), span(1, 6), span(3, 5)];
        carry_clocks(&before, &mut now, count);
        let clock = |s: &RootSpan| (s.exposure.0.to_bits(), s.seen.0.to_bits());
        let bits = |e: f64, h: f64| (e.to_bits(), h.to_bits());
        assert_eq!(clock(&now[0]), bits(100.0, 400.0));
        assert_eq!(clock(&now[1]), bits(0.0, 0.0));
        assert_eq!(clock(&now[2]), bits(0.0, 400.0));
        // A day at a deficit of 0.25: every root in cell 5 gains 6 hours, the newcomer
        // too, and none is charged the cell's 400.
        advance_clock(&mut now[0], 406.0);
        advance_clock(&mut now[2], 406.0);
        assert!((now[0].exposure.0 - 106.0).abs() < 1e-12);
        assert!((now[2].exposure.0 - 6.0).abs() < 1e-12);
        // The cell had air and started again: the clock is the cell's new count.
        advance_clock(&mut now[0], 3.0);
        assert!((now[0].exposure.0 - 3.0).abs() < 1e-12);
        // A flood: roots and cell share the clock.
        let mut flooded = span(4, 9);
        for h in [0.0, 24.0, 48.0, 72.0] {
            advance_clock(&mut flooded, h);
        }
        assert!((flooded.exposure.0 - 72.0).abs() < 1e-12);
    }

    fn pot() -> SoilGrid {
        let mut soil = SoilGrid::new_pot(
            Meters(0.24),
            Meters(0.24),
            Meters(0.02),
            Texture::Loam,
            Celsius(15.0),
        )
        .unwrap();
        soil.fill_to(Texture::Loam.hydraulics().field_capacity);
        soil.set_nitrogen(&SoilNitrogen::GARDEN);
        soil
    }

    fn rules() -> RootPlasticity {
        RootPlasticity {
            proliferation_nitrate: 1.0,
            proliferation_water: 0.5,
            ..RootPlasticity::default()
        }
    }

    /// The center of grid cell (x, y, z) in garden coordinates (z up from the surface).
    fn at(x: usize, y: usize, z: usize) -> [f64; 3] {
        #[allow(clippy::cast_precision_loss)]
        let c = |v: usize| (v as f64 + 0.5) * 0.02;
        [c(x), c(y), -c(z)]
    }

    #[test]
    fn proliferation_follows_nitrate_above_the_mean() {
        let mut soil = pot();
        let rich = soil.index(Cell { x: 6, y: 6, z: 5 });
        let plain = soil.index(Cell { x: 3, y: 6, z: 5 });
        let base = soil.no3_ppm(plain);
        soil.scale_nitrate_cells(&[rich], 3.0);
        let means = Some((1.0, base));
        let p = rules();
        // Uniform: nothing; twice the mean above it (three times the mean): 1 + 2.
        assert!((proliferation(&p, &soil, plain, means) - 1.0).abs() < 1e-12);
        assert!((proliferation(&p, &soil, rich, means) - 3.0).abs() < 1e-9);
        // Leaner than the mean: nothing either; off: nothing.
        assert!((proliferation(&p, &soil, plain, Some((1.0, 2.0 * base))) - 1.0).abs() < 1e-12);
        let off = RootPlasticity::default();
        assert!((proliferation(&off, &soil, rich, means) - 1.0).abs() < 1e-12);
        // Without a root zone there is nothing to compare with.
        assert!((proliferation(&p, &soil, rich, None) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_cohort_proliferates_by_the_share_of_its_path_in_rich_soil() {
        let mut soil = pot();
        let rich = soil.index(Cell { x: 6, y: 6, z: 5 });
        let base = soil.no3_ppm(soil.index(Cell { x: 3, y: 6, z: 5 }));
        soil.scale_nitrate_cells(&[rich], 3.0);
        let means = Some((1.0, base));
        let p = rules();
        let start = at(6, 6, 5);
        let east = [1.0, 0.0, 0.0];
        // From the rich cell's center 4 cm east: a quarter of the path is in it.
        let f = path_proliferation(&p, &soil, start, east, 0.04, means);
        assert!((f - 1.5).abs() < 1e-9, "{f}");
        // Entirely inside it.
        let f = path_proliferation(
            &p,
            &soil,
            [start[0] - 0.009, start[1], start[2]],
            east,
            0.018,
            means,
        );
        assert!((f - 3.0).abs() < 1e-9, "{f}");
    }

    #[test]
    fn the_water_gradient_points_to_the_wet_side() {
        let mut soil = pot();
        let (nx, _, _) = soil.dims();
        // Uniform soil: no pull.
        assert!(
            water_gradient(&soil, at(6, 6, 5))
                .iter()
                .all(|g| g.abs() < 1e-12)
        );
        // Dry the pot, then water its west half behind a divider.
        let h = Texture::Loam.hydraulics();
        soil.fill_to(h.wilting_point + 0.2 * (h.field_capacity - h.wilting_point));
        soil.set_divider();
        soil.step_hour(&SoilHour {
            water_in: Millimeters(0.0),
            surface_radiation: WPerM2(0.0),
            t_air: Celsius(15.0),
            strip: Some(Strip {
                water: Millimeters(20.0),
                x0: 0,
                x1: nx / 2,
            }),
        });
        // Just east of the divider the wet side is west.
        let g = water_gradient(&soil, at(nx / 2, 6, 0));
        assert!(g[0] < -0.5, "{g:?}");
        // Outside the soil: nothing.
        assert!(
            water_gradient(&soil, [0.1, 0.1, 0.05])
                .iter()
                .all(|g| g.abs() < 1e-12)
        );
    }
}
