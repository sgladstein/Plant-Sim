//! CRootBox-style root system (§3.7; decision 0013): axes of each root type appear at
//! their emergence time, grow as chains of segments, bend by gravitropism and deflect
//! along the pot wall; laterals are lumped into cohorts, one per parent segment.

use ps_graph::{OrganId, OrganKind, SpecificSize};
use ps_rng::Purpose;
use ps_soil::SoilGrid;
use ps_species::RootOrigin;
use ps_units::{DegreeDays, GramsDM, Hours};

use crate::geometry::{V3, add, deflect, from_azimuth, soil_cell, unit};
use crate::{ArchError, PRIMORDIUM, Plant, Role, RootAxis, Spawn, curve};

/// Golden angle, rad: the azimuth step between successive lateral cohorts.
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;
/// Standard deviation of the random component added to a root direction per segment.
const TROPISM_NOISE: f64 = 0.15;

impl Plant {
    /// Daily step 3, root part: axes of each seed-borne root type appear once thermal
    /// time since germination passes the type's emergence time.
    pub(crate) fn root_initiation(&mut self, now: Hours, soil: &SoilGrid) -> Result<(), ArchError> {
        if self.phenology.stage < ps_physio::Stage::Germinated {
            return Ok(());
        }
        let since = self.phenology.since_germination.0;
        let mut whorl = 0u32;
        for t in 0..self.params.roots.len() {
            let rp = self.params.roots[t].clone();
            if rp.origin == RootOrigin::Basal {
                whorl += 1;
            }
            if self.root_types_started[t] || rp.order > 0 || since < rp.emergence.0 {
                continue;
            }
            self.root_types_started[t] = true;
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
        let potential = GramsDM(len * mult / rp.specific_length.0);
        let duration = DegreeDays(len / rp.elongation_per_cd.max(1e-12));
        let mut s = Spawn::new(
            OrganKind::Root,
            Role::Root,
            parent,
            curve(duration, potential)?,
        );
        s.order = rp.order;
        s.specific = SpecificSize::Length(rp.specific_length);
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
    /// past their apical zone emit one lateral cohort per segment.
    pub(crate) fn root_elongation(&mut self, now: Hours, soil: &SoilGrid) -> Result<(), ArchError> {
        let child = |order: u8| {
            self.params
                .roots
                .iter()
                .position(|r| r.origin == RootOrigin::Lateral && r.order == order + 1)
        };
        let lateral_type = child(0);
        let fine_type = child(1);
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
            if !self.arena.alive[tip.index()] {
                self.root_axes[a].done = true;
                continue;
            }
            let rp = self.params.roots[self.root_axes[a].kind].clone();
            if rp.order > 0 {
                // A lateral cohort: its fine roots once it has finished.
                if let Some(ft) = fine_type.filter(|_| rp.order == 1) {
                    self.fine_roots(a, ft, now, soil)?;
                }
                continue;
            }
            self.settle_axis(a);
            // Laterals behind the apical zone, one cohort per parent segment.
            if let Some(lt) = lateral_type.filter(|_| rp.branch_spacing.0 > 0.0) {
                self.laterals(a, lt, now, soil)?;
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
            let dir = unit(add(
                add(ax.direction, [0.0, 0.0, -1.0], rp.gravitropism * 0.2),
                noise,
                1.0,
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
    ) -> Result<(), ArchError> {
        let seg = self.params.root_segment.0;
        let parent_type = self.params.roots[self.root_axes[a].kind].clone();
        loop {
            let ax = &self.root_axes[a];
            let k = ax.laterals;
            // Completed length behind the apical zone.
            let reach = ax.length - parent_type.apical_zone.0;
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
            let mult = seg / parent_type.branch_spacing.0;
            self.cohort(host, attach, lt, k, mult, now, soil)?;
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
        self.cohort(host, 0.5 * branched / len1, ft, k, mult, now, soil)
    }

    /// One lateral cohort of type `lt` on root organ `host` at `attach` (a fraction of
    /// its length): a single straight organ of the lateral's full length standing for
    /// `mult` roots, aimed by the golden angle `k`.
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
    ) -> Result<(), ArchError> {
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
        let len = lp.max_length.0;
        // The cohort reaches its potential plus the primordium it starts as.
        let reach = len * (1.0 + PRIMORDIUM);
        let inside = |d: V3| soil_cell(soil, add(base, d, reach)).is_some();
        let inward = [
            self.collar_position[0] - base[0],
            self.collar_position[1] - base[1],
            0.0,
        ];
        let found = wall(soil, base, dir, reach)
            .or_else(|| {
                (1..8)
                    .map(|j| {
                        let mut d =
                            deflect(host_dir, lp.insertion.0, GOLDEN_ANGLE * f64::from(k + j));
                        d[2] = -d[2].abs();
                        d
                    })
                    .find(|d| inside(*d))
            })
            .or_else(|| {
                (libm::hypot(inward[0], inward[1]) > 1e-6)
                    .then(|| unit(inward))
                    .filter(|d| inside(*d))
            });
        let Some(d) = found else {
            return Ok(());
        };
        dir = d;
        let potential = GramsDM(lp.max_length.0 * mult / lp.specific_length.0);
        let duration = DegreeDays(lp.max_length.0 / lp.elongation_per_cd.max(1e-12));
        let mut s = Spawn::new(
            OrganKind::Root,
            Role::Root,
            host,
            curve(duration, potential)?,
        );
        s.order = lp.order;
        s.specific = SpecificSize::Length(lp.specific_length);
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
        Ok(())
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
