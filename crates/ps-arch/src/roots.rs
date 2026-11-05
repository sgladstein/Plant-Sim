//! CRootBox-style root system (§3.7; decision 0013): axes of each root type appear at
//! their emergence time, grow as chains of segments, bend by gravitropism and deflect
//! along the pot wall; laterals are lumped into cohorts, one per parent segment.

use ps_graph::{OrganId, OrganKind, SpecificSize};
use ps_rng::Purpose;
use ps_soil::SoilGrid;
use ps_species::RootOrigin;
use ps_units::{DegreeDays, GramsDM, Hours};

use crate::geometry::{V3, add, deflect, from_azimuth, soil_cell, unit};
use crate::{ArchError, Plant, Role, RootAxis, Spawn, curve};

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
                };
                self.root_axes.push(ax);
                let a = self.root_axes.len() - 1;
                self.root_segment(a, parent, attach, base, dir, now, soil)?;
            }
        }
        Ok(())
    }

    /// Adds the next segment to root axis `a` from `base` along `dir`, deflecting at the
    /// pot wall; marks the axis done when it is full length or boxed in.
    #[allow(clippy::too_many_arguments)]
    fn root_segment(
        &mut self,
        a: usize,
        parent: OrganId,
        attach: f64,
        base: V3,
        dir: V3,
        now: Hours,
        soil: &SoilGrid,
    ) -> Result<(), ArchError> {
        let rp = self.params.roots[self.root_axes[a].kind].clone();
        let left = rp.max_length.0 - self.root_axes[a].length;
        let len = self.params.root_segment.0.min(left);
        if len < 1e-4 {
            self.root_axes[a].done = true;
            return Ok(());
        }
        let Some(dir) = wall(soil, base, dir, len) else {
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
        let ax = &mut self.root_axes[a];
        ax.segments.push((id, ax.length));
        ax.direction = dir;
        Ok(())
    }

    /// Daily step 5 (§3.7): a root axis whose tip segment has finished expanding adds
    /// the next segment, bent by gravitropism and noise; axes past their apical zone
    /// emit one lateral cohort per segment.
    pub(crate) fn root_elongation(&mut self, now: Hours, soil: &SoilGrid) -> Result<(), ArchError> {
        let lateral_type = self
            .params
            .roots
            .iter()
            .position(|r| r.origin == RootOrigin::Lateral);
        for a in 0..self.root_axes.len() {
            let ax = self.root_axes[a].clone();
            let rp = self.params.roots[ax.kind].clone();
            let Some(&(tip, _)) = ax.segments.last() else {
                continue;
            };
            if !self.arena.alive[tip.index()] {
                self.root_axes[a].done = true;
                continue;
            }
            // Laterals behind the apical zone, one cohort per parent segment.
            if let Some(lt) = lateral_type.filter(|_| rp.branch_spacing.0 > 0.0) {
                self.laterals(a, lt, now)?;
            }
            if ax.done {
                continue;
            }
            let i = tip.index();
            if !self.arena.curve[i].is_complete(self.arena.expansion_age[i]) {
                continue;
            }
            let grown = self.arena.size[i].length.0 / ax.multiplicity.max(1.0);
            let base = self.positions.get(i).map_or(self.collar_position, |p| p.0);
            let end = add(base, ax.direction, grown);
            self.root_axes[a].length += grown;
            if self.root_axes[a].length >= rp.max_length.0 - 1e-6 {
                self.root_axes[a].done = true;
                continue;
            }
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
            self.root_segment(a, tip, 1.0, end, dir, now, soil)?;
        }
        Ok(())
    }

    fn laterals(&mut self, a: usize, lt: usize, now: Hours) -> Result<(), ArchError> {
        let seg = self.params.root_segment.0;
        let parent_type = self.params.roots[self.root_axes[a].kind].clone();
        let lp = self.params.roots[lt].clone();
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
            let mut dir = deflect(
                self.arena.orientation[h].direction,
                lp.insertion.0,
                GOLDEN_ANGLE * f64::from(k),
            );
            // A cohort aimed upward is mirrored down: it stands for laterals all
            // around the parent, and the upward ones turn down within a few
            // millimeters (decision 0013).
            dir[2] = -dir[2].abs();
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
            });
        }
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
