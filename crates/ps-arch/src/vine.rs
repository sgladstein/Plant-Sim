//! The vine module (§3.4, WP22, decision 0021): an indeterminate main stem that twines
//! up vertical supports and lodges without one.
//!
//! Common bean climbs by twining, not tendrils. The shoot tip circumnutates (sweeps a
//! circle every 80 to 130 minutes) and wraps any support it touches, anticlockwise seen
//! from above, a right-handed helix. At a daily step the tip makes a dozen or more
//! sweeps, so a support within the sweep radius is caught at once, with a probability
//! that falls with the support's thickness. Internodes made while the axis twines coil
//! around the support's axis at a fixed inclination; the coil is fixed once made (the
//! sim does not move grown organs). A vine with no support holds up a short stem, then
//! lodges: new internodes run along the ground in a wandering heap around the collar,
//! where its leaves shade each other.
//!
//! Model card: docs/models/climbing.md.

use core::f64::consts::PI;

use ps_rng::Purpose;
use ps_species::Species;
use ps_units::{DegreeDays, Hours, Meters, Radians};
use serde::{Deserialize, Serialize};

use crate::geometry::{V3, from_azimuth};
use crate::{Plant, ShootAxis};

const CM: f64 = 0.01;

/// Most a lodged tip turns in a day, either way, radians (~).
const LODGED_WANDER: f64 = 0.8;
/// Elevation of an internode hanging down from the top of its support, radians below
/// the horizontal (~): it drapes beside the support instead of reaching sideways.
const TOPPED_HANG: f64 = 1.2;

/// A vertical support (stake, pole or string), in the plant's soil frame.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Support {
    /// Center of its foot at the soil surface, m.
    pub base: V3,
    /// Height above its foot, m.
    pub height: f64,
    /// Diameter, m.
    pub diameter: f64,
}

/// Climbing parameters in model units (species `climbing` group).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClimbingParams {
    /// Most main-stem nodes.
    pub max_main_nodes: u32,
    /// Thermal time after first flower over which the main stem makes nodes.
    pub vegetative_after_flowering: DegreeDays,
    /// Main-stem nodes over which internodes reach full length.
    pub ramp_nodes: u32,
    /// Tip sweep radius.
    pub sweep_radius: Meters,
    /// Supports up to this diameter are always caught.
    pub sure_diameter: Meters,
    /// Supports this thick or more are never caught.
    pub max_diameter: Meters,
    /// Rise of the helix per turn.
    pub pitch: Meters,
    /// Steepest and flattest helix angles above the horizontal.
    pub inclination: (Radians, Radians),
    /// Stem length held up without a support.
    pub self_support: Meters,
    /// Radius of the heap a lodged vine makes around its collar.
    pub heap_radius: Meters,
    /// Height below which lodged stems run level.
    pub lodge_height: Meters,
    /// Daily chance that a pod resting on the soil rots.
    pub pod_rot_rate: f64,
}

impl Default for ClimbingParams {
    /// The pole bean's values (`data/species/pole_bean.ron`).
    fn default() -> Self {
        Self {
            max_main_nodes: 30,
            vegetative_after_flowering: DegreeDays(260.0),
            ramp_nodes: 8,
            sweep_radius: Meters(0.06),
            sure_diameter: Meters(0.05),
            max_diameter: Meters(0.2),
            pitch: Meters(0.10),
            inclination: (Radians(35f64.to_radians()), Radians(75f64.to_radians())),
            self_support: Meters(0.4),
            heap_radius: Meters(0.15),
            lodge_height: Meters(0.02),
            pod_rot_rate: 0.02,
        }
    }
}

impl ClimbingParams {
    /// Converts the species' `climbing` group.
    #[must_use]
    pub fn new(s: &Species) -> Self {
        let c = &s.climbing;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let round = |x: f64| libm::round(x.max(0.0)) as u32;
        Self {
            max_main_nodes: round(c.max_main_nodes.value).max(2),
            vegetative_after_flowering: DegreeDays(c.vegetative_after_flowering.value),
            ramp_nodes: round(c.internode_ramp_nodes.value).max(1),
            sweep_radius: Meters(c.circumnutation_radius.value * CM),
            sure_diameter: Meters(c.support_diameter_sure.value * CM),
            max_diameter: Meters(c.support_diameter_max.value * CM),
            pitch: Meters(c.helix_pitch.value * CM),
            inclination: (
                Radians(c.helix_inclination_min.value.to_radians()),
                Radians(c.helix_inclination_max.value.to_radians()),
            ),
            self_support: Meters(c.self_support_length.value * CM),
            heap_radius: Meters(c.heap_radius.value * CM),
            lodge_height: Meters(c.lodge_height.value * CM),
            pod_rot_rate: c.pod_rot_rate.value,
        }
    }

    /// Helix angle around a support whose stem centerline runs at radius `r` (m): the
    /// angle that gives the pitch, `atan(pitch / 2πr)`, kept within the species' range.
    /// Thin strings make steep coils, thick stakes flat ones.
    #[must_use]
    pub fn inclination_for(&self, r: f64) -> f64 {
        let eta = libm::atan(self.pitch.0 / (2.0 * PI * r.max(1e-6)));
        eta.clamp(self.inclination.0.0, self.inclination.1.0)
    }

    /// Probability that a tip touching a support of diameter `d` (m) wraps it: 1 up to
    /// the sure diameter, falling linearly to 0 at the largest.
    #[must_use]
    pub fn capture_probability(&self, d: f64) -> f64 {
        let (lo, hi) = (self.sure_diameter.0, self.max_diameter.0);
        if d <= lo {
            1.0
        } else if d >= hi {
            0.0
        } else {
            (hi - d) / (hi - lo).max(1e-12)
        }
    }
}

/// How an axis climbs.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Climb {
    /// Upright, searching (or a determinate herb's axis).
    #[default]
    Free,
    /// Coiling around support `0`.
    Twining(u32),
    /// Grew past the top of its support and hangs over.
    Topped,
    /// Fell over: grows along the ground.
    Lodged,
}

/// The coil an internode makes around a support (in [`crate::Meta`]).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Twine {
    /// Support axis, x and y, m.
    pub center: [f64; 2],
    /// Radius of the stem's centerline around it, m.
    pub radius: f64,
    /// Helix angle above the horizontal, radians.
    pub inclination: f64,
}

impl Twine {
    /// The point `s` metres along the coil from `base`, anticlockwise seen from above.
    #[must_use]
    pub fn point(&self, base: V3, s: f64) -> V3 {
        let phase0 = libm::atan2(base[1] - self.center[1], base[0] - self.center[0]);
        let r = self.radius.max(1e-6);
        let phase = phase0 + s * libm::cos(self.inclination) / r;
        [
            self.center[0] + r * libm::cos(phase),
            self.center[1] + r * libm::sin(phase),
            base[2] + s * libm::sin(self.inclination),
        ]
    }
}

/// Elevation (negative: downward) at which a segment of length `len` starting at
/// height `z` ends at height `lodge`, or level if it starts there or below.
fn settle(z: f64, len: f64, lodge: f64) -> f64 {
    if len <= 0.0 {
        return 0.0;
    }
    -libm::asin(((z - lodge) / len).clamp(0.0, 1.0))
}

/// Compass azimuth of a direction (0 = +y, increasing toward +x), as
/// [`from_azimuth`] takes it.
fn azimuth_of(d: V3) -> f64 {
    libm::atan2(d[0], d[1])
}

impl Plant {
    /// Whether this plant grows as an indeterminate vine (`habit` ≥ 0.5).
    #[must_use]
    pub fn is_vine(&self) -> bool {
        self.params.indeterminate
    }

    /// Daily step 3, before new nodes: each searching vine axis catches the nearest
    /// support within its tip's sweep, and a twining axis that has passed its support's
    /// top lets go.
    pub(crate) fn find_supports(&mut self, now: Hours) {
        if !self.is_vine() || self.supports.is_empty() {
            return;
        }
        let cp = self.params.climbing.clone();
        for x in 0..self.axes.len() {
            let ax = &self.axes[x];
            let apex = ax.apex.index();
            if ax.terminated || apex >= self.positions.len() {
                continue;
            }
            let tip = self.positions[apex].1;
            match ax.climb {
                Climb::Twining(k) => {
                    let s = self.supports[k as usize];
                    if tip[2] >= s.base[2] + s.height {
                        self.axes[x].climb = Climb::Topped;
                    }
                }
                Climb::Free | Climb::Lodged => {
                    let mut best: Option<(f64, usize)> = None;
                    for (k, s) in self.supports.iter().enumerate() {
                        if tip[2] < s.base[2] || tip[2] > s.base[2] + s.height {
                            continue;
                        }
                        let (dx, dy) = (tip[0] - s.base[0], tip[1] - s.base[1]);
                        let gap = libm::sqrt(dx * dx + dy * dy) - 0.5 * s.diameter;
                        if gap <= cp.sweep_radius.0 && best.is_none_or(|(g, _)| gap < g) {
                            best = Some((gap, k));
                        }
                    }
                    if let Some((_, k)) = best {
                        let p = cp.capture_probability(self.supports[k].diameter);
                        if self.stream(ax.apex, now, Purpose::Twining).bernoulli(p) {
                            #[allow(clippy::cast_possible_truncation)]
                            let k = k as u32;
                            self.axes[x].climb = Climb::Twining(k);
                        }
                    }
                }
                Climb::Topped => {}
            }
        }
    }

    /// The direction and coil of a new internode on a vine axis, given the axis and
    /// the upright direction the herb rule would give it.
    pub(crate) fn vine_internode(
        &mut self,
        ax: &mut ShootAxis,
        upright: V3,
        length: f64,
        now: Hours,
    ) -> (V3, Option<Twine>) {
        let cp = &self.params.climbing;
        let apex = ax.apex.index();
        let tip = self
            .positions
            .get(apex)
            .map_or(self.collar_position, |p| p.1);
        match ax.climb {
            Climb::Twining(k) => {
                let s = self.supports[k as usize];
                let radius = 0.5 * s.diameter + self.stem_radius();
                let twine = Twine {
                    center: [s.base[0], s.base[1]],
                    radius,
                    inclination: cp.inclination_for(radius),
                };
                ([0.0, 0.0, 1.0], Some(twine))
            }
            Climb::Topped => {
                let az = azimuth_of(self.arena.orientation[apex].direction);
                (from_azimuth(az, -TOPPED_HANG), None)
            }
            Climb::Free => {
                ax.free_length += length;
                if ax.free_length > cp.self_support.0 {
                    // Too long to hold itself up: the stem falls over (decision 0021).
                    ax.climb = Climb::Lodged;
                    let az = self.fall(ax, now);
                    return (from_azimuth(az, 0.0), None);
                }
                (upright, None)
            }
            Climb::Lodged => {
                let heap = cp.heap_radius.0;
                let lodge = cp.lodge_height.0;
                let c = self.collar_position;
                let (dx, dy) = (tip[0] - c[0], tip[1] - c[1]);
                let mut rng = self.stream(ax.apex, now, Purpose::Tropism);
                let turn = (rng.uniform() * 2.0 - 1.0) * LODGED_WANDER;
                let prev = self.arena.orientation[apex].direction;
                let az = if dx * dx + dy * dy > heap * heap {
                    // Turned back toward the collar, a little to one side.
                    libm::atan2(-dx, -dy) + 0.5 * turn
                } else if prev[0].abs() + prev[1].abs() < 1e-9 {
                    ax.azimuth + turn
                } else {
                    azimuth_of(prev) + turn
                };
                (from_azimuth(az, settle(tip[2], length, lodge)), None)
            }
        }
    }

    /// Lays the axis' stem down (and, for the main stem, every branch with it): each
    /// internode from the axis' base to its apex turns toward one random compass
    /// direction and dips until it lies at the lodge height, and the organs on them
    /// follow at the next layout. Returns the main direction. This is the one place
    /// the sim moves grown organs: lodging is a mechanical failure, not growth.
    fn fall(&mut self, ax: &ShootAxis, now: Hours) -> f64 {
        let az = self.stream(ax.base, now, Purpose::Tropism).uniform() * 2.0 * PI;
        self.lay_down(ax.base, ax.apex, az);
        if ax.order == 0 {
            for y in 0..self.axes.len() {
                let b = self.axes[y].clone();
                if b.order == 0 || b.terminated {
                    continue;
                }
                self.lay_down(b.base, b.apex, b.azimuth);
                self.axes[y].climb = Climb::Lodged;
            }
        }
        az
    }

    /// Turns the stem internodes from `base` to `apex` toward `azimuth`, each dipping
    /// as far as it needs to reach the lodge height and no further.
    fn lay_down(&mut self, base: ps_graph::OrganId, apex: ps_graph::OrganId, azimuth: f64) {
        let mut chain = Vec::new();
        let mut cur = Some(apex);
        while let Some(id) = cur {
            let i = id.index();
            if self.arena.kind[i] != ps_graph::OrganKind::Internode
                || self.meta[i].role != crate::Role::Stem
            {
                break;
            }
            chain.push(id);
            if id == base {
                break;
            }
            cur = self.arena.parent[i];
        }
        let Some(&first) = chain.last() else {
            return;
        };
        let mut z = self.positions.get(first.index()).map_or(0.0, |p| p.0[2]);
        let lodge = self.params.climbing.lodge_height.0;
        for &id in chain.iter().rev() {
            let i = id.index();
            let len = self.arena.size[i].length.0;
            let elevation = settle(z, len, lodge);
            z += len * libm::sin(elevation);
            self.arena.orientation[i].direction = from_azimuth(azimuth, elevation);
        }
    }

    /// Daily step 4, vines: a pod that hangs down to the soil (its foot no higher than
    /// its length) rots with the daily chance `pod_rot_rate` and is shed with its
    /// seeds. Upright and staked vines hold their pods clear (decision 0021).
    pub(crate) fn rot_pods(&mut self, now: Hours) -> Result<(), crate::ArchError> {
        let rate = self.params.climbing.pod_rot_rate;
        if !self.is_vine() || rate <= 0.0 {
            return Ok(());
        }
        let mut rotten = Vec::new();
        for pod in &self.pods {
            let i = pod.id.index();
            if !self.arena.alive[i] || i >= self.positions.len() {
                continue;
            }
            let foot = self.positions[i].0[2];
            if foot <= self.arena.size[i].length.0
                && self.stream(pod.id, now, Purpose::Rot).bernoulli(rate)
            {
                rotten.push(pod.id);
            }
        }
        for id in rotten {
            self.shed_organ(id, now)?;
        }
        Ok(())
    }

    /// Radius of a stem of the species' specific length and density, m.
    fn stem_radius(&self) -> f64 {
        let ssl = self.params.stem_specific_length.0;
        let rho = self.params.allometry.stem_density.0;
        libm::sqrt(1.0 / (PI * ssl * rho).max(1e-12))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Darwin's stakes: wraps 0.5, 1.5 and 5 cm, fails on 25 cm, and a 10 cm post is a
    /// coin toss or worse.
    #[test]
    fn capture_falls_with_support_thickness() {
        let cp = ClimbingParams::default();
        for d in [0.005, 0.015, 0.05] {
            assert!((cp.capture_probability(d) - 1.0).abs() < 1e-12, "{d}");
        }
        assert!(cp.capture_probability(0.25).abs() < 1e-12);
        let mid = cp.capture_probability(0.10);
        assert!(mid > 0.0 && mid < 1.0, "{mid}");
    }

    /// The coil turns anticlockwise seen from above (a right-handed helix), rises at
    /// its inclination and keeps its radius.
    #[test]
    fn coil_is_anticlockwise_from_above() {
        let t = Twine {
            center: [0.0, 0.0],
            radius: 0.01,
            inclination: 60f64.to_radians(),
        };
        let base = [0.01, 0.0, 0.2];
        let p = t.point(base, 0.001);
        // From +x a small anticlockwise turn moves toward +y.
        assert!(p[1] > 0.0, "{p:?}");
        let q = t.point(base, 0.1);
        let r = libm::sqrt(q[0] * q[0] + q[1] * q[1]);
        assert!((r - 0.01).abs() < 1e-12);
        assert!((q[2] - (0.2 + 0.1 * libm::sin(60f64.to_radians()))).abs() < 1e-12);
    }

    #[test]
    fn settle_reaches_the_lodge_height() {
        let e = settle(0.10, 0.12, 0.02);
        assert!((0.10 + 0.12 * libm::sin(e) - 0.02).abs() < 1e-12);
        assert!(settle(0.01, 0.12, 0.02).abs() < 1e-12);
        // A short segment dips straight down as far as it can.
        assert!((settle(0.5, 0.1, 0.02) + core::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }
}
