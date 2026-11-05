//! The moments (design plan §2.3, WP20): render-only posture that makes what the sim
//! computes visible as a plant would show it. Nothing here feeds back into the sim.
//!
//! - **Emergence.** An epigeal bean's hypocotyl comes up as a hook with the cotyledons
//!   hanging from it, the hook straightens, and the cotyledons lift and spread. The sim
//!   grows the hypocotyl straight; [`pose`] bends its top by how far the tip has risen
//!   above the soil and swings the cotyledons from hanging to the sim's angle.
//! - **Phototropic lean.** The sim's stems grow straight up (gravitropism only). The
//!   viewer leans each shoot toward its brighter side: [`lean_from_light`] turns the
//!   leaves' absorbed light into a lean, and [`pose`] bends the shoot by it, more toward
//!   the top, the way a growing stem curves.
//! - **Wilting.** Below a water stress factor of about 0.9 the petioles and the shoot
//!   tip hang ([`wilt_of_stress`]); the blades droop with the template's droop levels.
//!   Recovery follows the sim's stress back up within hours of watering.
//!
//! The flowering cue is the camera's (ps-view `moments`), not geometry.

use glam::{Quat, Vec2, Vec3};
use ps_graph::OrganKind;

use crate::{Hook, Skeleton};

/// Height of the hypocotyl's tip above the soil by which its hook has straightened, m.
pub const HOOK_OPEN_HEIGHT: f32 = 0.015;

/// Tip heights over which the cotyledons swing from closed to the sim's angle, m: they
/// part as the hook straightens and are spread by the time the hypocotyl is near full
/// height (about 3.5 cm above the soil for a bean sown 3 cm deep).
pub const SPREAD_HEIGHTS: (f32, f32) = (0.01, 0.026);

/// How far wilted petioles hang below their turgid angle, rad.
pub const WILT_PETIOLE: f32 = 0.6;

/// How far a wilted shoot tip nods toward horizontal, rad.
pub const WILT_NOD: f32 = 0.7;

/// The largest lean of a shoot's top, rad (about 17°).
pub const MAX_LEAN: f32 = 0.3;

/// Lean per unit of light asymmetry (see [`lean_from_light`]), rad.
pub const LEAN_GAIN: f32 = 2.5;

/// The render-only posture of one plant.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pose {
    /// Lean of the shoot's top, rad, as a horizontal vector toward the light (x east,
    /// y north). Zero for an upright plant.
    pub lean: Vec2,
    /// Wilting, 0 (turgid) to 1 (fully wilted); see [`wilt_of_stress`].
    pub wilt: f32,
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Wilting for the plant's water stress factor (1 = unstressed): none above 0.92, full
/// at 0.45 and below. A pot bean at a stress factor near 0.5 (FTSW about 0.2) hangs its
/// leaves through the day; one near 0.9 has barely started to lose turgor.
#[must_use]
pub fn wilt_of_stress(water_stress_factor: f32) -> f32 {
    if !water_stress_factor.is_finite() {
        return 0.0;
    }
    1.0 - smoothstep(0.45, 0.92, water_stress_factor)
}

/// Hours over which drawn wilting follows the sim's stress: leaves lose turgor over about
/// half a day, so the hour-to-hour and day-night swings of the stress factor do not make
/// them twitch.
pub const TURGOR_HOURS: f32 = 12.0;

/// Hours over which leaves regain turgor once water arrives: a watered plant perks up
/// within a few hours.
pub const RECOVERY_HOURS: f32 = 2.0;

/// A drop in target wilt larger than this is a recovery (watering), followed at
/// [`RECOVERY_HOURS`]; smaller ones (the night's partial recovery) at [`TURGOR_HOURS`].
pub const RECOVERY_STEP: f32 = 0.25;

/// The drawn wilt `hours` sim hours after it was `shown`, moving toward `target` (both
/// 0 to 1, from [`wilt_of_stress`]): a first-order lag, fast when the target drops by more
/// than [`RECOVERY_STEP`].
#[must_use]
pub fn turgor_step(shown: f32, target: f32, hours: f32) -> f32 {
    if !shown.is_finite() || !target.is_finite() {
        return if target.is_finite() { target } else { 0.0 };
    }
    let tau = if shown - target > RECOVERY_STEP {
        RECOVERY_HOURS
    } else {
        TURGOR_HOURS
    };
    let k = 1.0 - (-hours.max(0.0) / tau).exp();
    (shown + (target - shown) * k).clamp(0.0, 1.0)
}

/// The lean toward the light, rad, from one entry per leaf: its horizontal offset from
/// the stem's base, m, its area, m², and the light it absorbs (any unit, per area).
///
/// The asymmetry is the area-weighted light on each side relative to the mean: leaves
/// that are all lit alike give no lean however they are arranged, and a plant lit only
/// on one side leans that way by about [`LEAN_GAIN`] times the share of the excess,
/// capped at [`MAX_LEAN`].
#[must_use]
pub fn lean_from_light(leaves: &[(Vec2, f32, f32)]) -> Vec2 {
    let usable = |(offset, area, light): &&(Vec2, f32, f32)| {
        offset.is_finite()
            && offset.length() > 1e-3
            && area.is_finite()
            && *area > 0.0
            && light.is_finite()
            && *light >= 0.0
    };
    let (area, total) = leaves
        .iter()
        .filter(usable)
        .fold((0.0_f32, 0.0_f32), |(a, t), (_, area, light)| {
            (a + area, t + area * light)
        });
    if area <= 0.0 || total <= 0.0 {
        return Vec2::ZERO;
    }
    let mean = total / area;
    let g = leaves
        .iter()
        .filter(usable)
        .map(|(offset, area, light)| offset.normalize() * (area * (light - mean)))
        .sum::<Vec2>()
        / total;
    let lean = g * LEAN_GAIN;
    if lean.length() > MAX_LEAN {
        lean.normalize() * MAX_LEAN
    } else {
        lean
    }
}

/// Turns `direction` down by `angle` (rad) in its own vertical plane, no lower than
/// `floor` (elevation, rad).
fn pitch_down(direction: Vec3, angle: f32, floor: f32) -> Vec3 {
    let h = Vec3::new(direction.x, direction.y, 0.0);
    let Some(h) = h.try_normalize() else {
        // Straight up or down: no compass direction to droop toward.
        return direction;
    };
    let elevation = direction.z.clamp(-1.0, 1.0).asin();
    let e = (elevation - angle).max(floor.min(elevation));
    h * e.cos() + Vec3::Z * e.sin()
}

/// The emerging hypocotyl and its cotyledons, if this is an epigeal seedling.
fn hypocotyl(skeleton: &Skeleton) -> Option<(usize, Vec<usize>)> {
    let cotyledons: Vec<usize> = skeleton
        .organs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.kind == OrganKind::Cotyledon)
        .map(|(i, _)| i)
        .collect();
    let parent = skeleton.organs.get(*cotyledons.first()?)?.parent?.index();
    let h = skeleton.organs.get(parent)?;
    (h.kind == OrganKind::Internode).then(|| {
        let cots = cotyledons
            .into_iter()
            .filter(|&i| skeleton.organs[i].parent.map(ps_graph::OrganId::index) == Some(parent))
            .collect();
        (parent, cots)
    })
}

/// The hook and cotyledon posture of an emerging seedling.
fn emerge(skeleton: &mut Skeleton) {
    let Some((h, cots)) = hypocotyl(skeleton) else {
        return;
    };
    if skeleton.blade_aspect.is_some() {
        return;
    }
    let hyp = skeleton.organs[h];
    let height = hyp.tip().z;
    let amount = 1.0 - smoothstep(0.0, HOOK_OPEN_HEIGHT, height);
    let spread = smoothstep(SPREAD_HEIGHTS.0, SPREAD_HEIGHTS.1, height);
    // The cotyledons part across the hook, so it bends at right angles to them.
    let across = cots
        .first()
        .map(|&c| skeleton.organs[c].direction)
        .and_then(|d| Vec2::new(d.x, d.y).try_normalize())
        .unwrap_or(Vec2::X);
    let toward = across.y.atan2(across.x) + core::f32::consts::FRAC_PI_2;
    skeleton.organs[h].hook = (amount > 1e-3).then_some(Hook { amount, toward });
    // The direction the hook's tip points: down at full hook, up when straight.
    let phi = core::f32::consts::PI * amount;
    let u = Vec3::new(toward.cos(), toward.sin(), 0.0);
    let d = hyp.direction;
    let side = (u - d * u.dot(d)).normalize_or(Vec3::X);
    let tangent = d * phi.cos() + side * phi.sin();
    if spread >= 1.0 {
        return;
    }
    // The epicotyl and first leaves are held between the cotyledons, so they hang with
    // the hook and come up with it, and open out with the cotyledons.
    let n = skeleton.organs.len();
    let mut above = vec![false; n];
    for i in 0..n {
        let parent = skeleton.organs[i].parent.map(ps_graph::OrganId::index);
        above[i] = parent.is_some_and(|p| p < i && (p == h || above[p]))
            && skeleton.organs[i].kind != OrganKind::Cotyledon;
        if above[i] {
            let p = &mut skeleton.organs[i];
            let out = Vec3::new(p.direction.x, p.direction.y, 0.0).normalize_or_zero();
            let held = (tangent + out * 0.12).normalize_or(tangent);
            p.direction = held.lerp(p.direction, spread).normalize_or(p.direction);
        }
    }
    for c in cots {
        let p = &mut skeleton.organs[c];
        let out = Vec3::new(p.direction.x, p.direction.y, 0.0).normalize_or(Vec3::X);
        // Closed: pressed together along the hook's tip, each just to its own side.
        let closed = (tangent + out * 0.22).normalize_or(tangent);
        p.direction = closed.lerp(p.direction, spread).normalize_or(p.direction);
    }
}

/// Petioles hang and the shoot tips nod.
fn wilt(skeleton: &mut Skeleton, wilt: f32) {
    if wilt.is_nan() || wilt <= 0.0 || skeleton.blade_aspect.is_some() {
        return;
    }
    let w = wilt.min(1.0);
    let n = skeleton.organs.len();
    let mut has_child_internode = vec![false; n];
    for p in &skeleton.organs {
        if p.kind == OrganKind::Internode
            && let Some(parent) = p.parent.map(ps_graph::OrganId::index)
            && parent < n
            && skeleton.organs[parent].kind == OrganKind::Internode
            && skeleton.organs[parent].axis_order == p.axis_order
        {
            has_child_internode[parent] = true;
        }
    }
    for (i, p) in skeleton.organs.iter_mut().enumerate() {
        if !p.alive || p.coil.is_some() {
            continue;
        }
        match p.kind {
            OrganKind::Leaf => p.direction = pitch_down(p.direction, WILT_PETIOLE * w, -1.3),
            OrganKind::Internode if !has_child_internode[i] && p.base.z > 0.0 => {
                p.direction = pitch_down(p.direction, WILT_NOD * w, -0.1);
            }
            _ => {}
        }
    }
}

/// Bends the shoot toward `lean`: each organ above the soil turns by the lean times how
/// high its base stands in the shoot, so the stem curves rather than tilting at the
/// collar.
fn lean(skeleton: &mut Skeleton, lean: Vec2) {
    let angle = lean.length().min(MAX_LEAN);
    if angle.is_nan() || angle <= 1e-4 || skeleton.organs.iter().any(|p| p.coil.is_some()) {
        return;
    }
    let shoot = |k: OrganKind| {
        !matches!(
            k,
            OrganKind::Root | OrganKind::Nodule | OrganKind::Seed | OrganKind::SeedInPod
        )
    };
    let top = skeleton
        .organs
        .iter()
        .filter(|p| p.alive && shoot(p.kind))
        .map(|p| p.tip().z)
        .fold(0.0_f32, f32::max);
    if top < 0.02 {
        return;
    }
    let axis = Vec3::Z.cross(lean.extend(0.0)).normalize_or_zero();
    if axis == Vec3::ZERO {
        return;
    }
    for p in &mut skeleton.organs {
        if !shoot(p.kind) || p.base.z <= 0.0 {
            continue;
        }
        let w = smoothstep(0.0, 1.0, p.base.z / top);
        p.direction =
            (Quat::from_axis_angle(axis, angle * w) * p.direction).normalize_or(p.direction);
    }
}

/// Applies the moments to a placed skeleton: the emerging hook and cotyledons, wilting,
/// and the lean toward the light, then re-derives the organ positions.
pub fn pose(skeleton: &mut Skeleton, pose: Pose) {
    emerge(skeleton);
    wilt(skeleton, pose.wilt);
    lean(skeleton, pose.lean);
    skeleton.relayout();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Placement;
    use ps_graph::OrganId;

    fn organ(kind: OrganKind, parent: Option<u32>, direction: Vec3, length: f32) -> Placement {
        Placement {
            kind,
            parent: parent.map(OrganId),
            axis_order: 0,
            alive: true,
            base: Vec3::ZERO,
            direction: direction.normalize(),
            azimuth: 0.0,
            length,
            radius: 0.001,
            area: if kind == OrganKind::Leaf { 0.002 } else { 0.0 },
            thermal_age: 50.0,
            attach: 1.0,
            copies: 1,
            coil: None,
            petiole: 0.03,
            simple_leaf: Some(true),
            hook: None,
        }
    }

    /// Seed 3 cm deep, a hypocotyl of `hyp` m, two cotyledons, an epicotyl and a leaf,
    /// then `stem` more internodes of 5 cm, each with a leaf.
    fn seedling(hyp: f32, stem: usize) -> Skeleton {
        let out = |a: f32| Vec3::new(a.cos(), a.sin(), 0.3);
        let mut organs = vec![
            organ(OrganKind::Seed, None, Vec3::Z, 0.0),
            organ(OrganKind::Internode, Some(0), Vec3::Z, hyp),
            organ(OrganKind::Cotyledon, Some(1), out(0.0), 0.008),
            organ(
                OrganKind::Cotyledon,
                Some(1),
                out(core::f32::consts::PI),
                0.008,
            ),
            organ(OrganKind::Internode, Some(1), Vec3::Z, 0.01),
            organ(OrganKind::Leaf, Some(4), out(1.0), 0.0),
        ];
        let mut top = 4;
        for k in 0..stem {
            let next = organs.len() as u32;
            organs.push(organ(OrganKind::Internode, Some(top), Vec3::Z, 0.05));
            organs.push(organ(OrganKind::Leaf, Some(next), out(k as f32 * 2.4), 0.0));
            top = next;
        }
        let mut s = Skeleton {
            origin: Vec3::new(0.0, 0.0, -0.03),
            organs,
            blade_aspect: None,
            container: None,
        };
        s.relayout();
        s
    }

    #[test]
    fn wilting_starts_below_about_0_9_and_is_full_by_0_45() {
        assert!(wilt_of_stress(1.0).abs() < 1e-6);
        assert!(wilt_of_stress(0.93).abs() < 1e-6);
        assert!(wilt_of_stress(0.7) > 0.3 && wilt_of_stress(0.7) < 0.7);
        assert!((wilt_of_stress(0.4) - 1.0).abs() < 1e-6);
        assert!(wilt_of_stress(f32::NAN).abs() < 1e-6);
        assert!(crate::droop_level(1.0) == 0 && crate::droop_level(0.3) == crate::DROOP_LEVELS - 1);
    }

    #[test]
    fn drawn_wilting_lags_the_stress_and_recovers_fast_after_watering() {
        // Wilting comes on over about half a day.
        let after_hour = turgor_step(0.0, 1.0, 1.0);
        assert!(after_hour > 0.0 && after_hour < 0.15);
        assert!(turgor_step(0.0, 1.0, 36.0) > 0.9);
        // A night's small recovery barely moves it; watering brings it back in hours.
        assert!(turgor_step(0.9, 0.8, 1.0) > 0.89);
        assert!(turgor_step(0.9, 0.0, 4.0) < 0.15);
        assert!((turgor_step(0.5, 0.5, 3.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn evenly_lit_leaves_give_no_lean_and_one_sided_light_leans_that_way() {
        let even = [
            (Vec2::X * 0.05, 0.01, 300.0),
            (Vec2::NEG_X * 0.05, 0.01, 300.0),
            (Vec2::Y * 0.05, 0.03, 300.0),
        ];
        assert!(lean_from_light(&even).length() < 1e-6);
        let west_lit = [
            (Vec2::X * 0.05, 0.01, 50.0),
            (Vec2::NEG_X * 0.05, 0.01, 400.0),
        ];
        let l = lean_from_light(&west_lit);
        assert!(l.x < -0.1 && l.y.abs() < 1e-6, "{l:?}");
        assert!(l.length() <= MAX_LEAN + 1e-6);
        assert_eq!(lean_from_light(&[]), Vec2::ZERO);
    }

    #[test]
    fn an_emerging_hypocotyl_is_hooked_and_straightens_as_it_rises() {
        // Tip 0.5 cm above the soil: hooked, cotyledons hanging below the hook's top.
        let mut s = seedling(0.035, 0);
        pose(&mut s, Pose::default());
        let hook = s.organs[1].hook.expect("hooked");
        assert!(hook.amount > 0.5);
        let top = s.organs[1].tip();
        assert!(s.organs[2].direction.z < 0.0 && s.organs[3].direction.z < 0.0);
        assert!(top.z < 0.035 - 0.03);
        // The tip is joined: the cotyledons start where the hook ends.
        assert!((s.organs[2].base - top).length() < 1e-6);
        // Tip 4 cm up: straight, cotyledons back at the sim's angle.
        let mut s = seedling(0.07, 0);
        let before = s.organs[2].direction;
        pose(&mut s, Pose::default());
        assert!(s.organs[1].hook.is_none());
        assert!((s.organs[2].direction - before).length() < 1e-6);
        // The tube follows the hook: node count matches what the colors expect.
        let mut hooked = seedling(0.035, 0);
        pose(&mut hooked, Pose::default());
        let axes = crate::axes(&hooked, true);
        let main = axes
            .iter()
            .find(|a| a.key() == OrganId(1))
            .expect("main stem");
        let nodes = crate::axis_nodes(&hooked, main, crate::AxisStyle::STEM);
        let expected = 1 + main
            .organs
            .iter()
            .map(|id| crate::nodes_after_base(&hooked.organs[id.index()]))
            .sum::<usize>();
        assert_eq!(nodes.len(), expected);
    }

    #[test]
    fn wilting_lowers_petioles_and_the_lean_bends_the_top_toward_the_light() {
        let grown = seedling(0.07, 6);
        let mut wilted = grown.clone();
        pose(
            &mut wilted,
            Pose {
                lean: Vec2::ZERO,
                wilt: 1.0,
            },
        );
        let leaf = grown.organs.len() - 1;
        assert!(wilted.organs[leaf].direction.z < grown.organs[leaf].direction.z - 0.3);
        let mut leaning = grown.clone();
        pose(
            &mut leaning,
            Pose {
                lean: Vec2::new(0.0, -0.2),
                wilt: 0.0,
            },
        );
        let apex = grown.organs.len() - 2;
        // The top moves south; the collar stays put.
        assert!(leaning.organs[apex].tip().y < grown.organs[apex].tip().y - 0.02);
        assert!((leaning.organs[1].base - grown.organs[1].base).length() < 1e-6);
        // Lengths are kept.
        assert!((leaning.organs[apex].length - grown.organs[apex].length).abs() < 1e-6);
    }
}
