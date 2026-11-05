//! Clicking an organ or a pot to inspect it (design plan §5.3, WP17).
//!
//! A click (a press and release without dragging) casts a ray from the camera. Organs
//! are tested as simple shapes in each pot's sim frame: axis organs (stems, roots) as
//! capsules around their segment, leaves and fruit as spheres around their blade or
//! body. The nearest hit along the ray wins; a ray that misses every organ but passes
//! through a pot's soil selects that pot's plant. A click on empty space clears the
//! selection. The selected organ is outlined with a gizmo.

use bevy::prelude::*;
use ps_graph::OrganKind;
use ps_mesh::Placement;

use crate::camera::{CameraRig, OrbitCamera};
use crate::garden::{Garden, PotView};
use crate::scene::sim_to_bevy;

/// What the user clicked.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Picked {
    /// Index of the pot in [`Garden::pots`] (one per plant; plants in a plot share a bed).
    pub pot: usize,
    /// Organ index in the pot's skeleton, or `None` for the plant as a whole.
    pub organ: Option<usize>,
}

/// The current selection, written by clicks and cleared by the UI.
#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct Selection {
    /// The selected plant or organ.
    pub picked: Option<Picked>,
}

/// Bevy world to the sim frame (inverse of [`sim_to_bevy`]).
#[must_use]
pub fn bevy_to_sim(p: Vec3) -> Vec3 {
    Vec3::new(p.x, -p.z, p.y)
}

/// Smallest pick radius, m, so thin roots and young stems can be clicked.
const MIN_RADIUS: f32 = 0.004;

/// The shape an organ is picked by: a segment and a radius (a sphere when the ends meet).
#[must_use]
pub fn pick_shape(p: &Placement) -> (Vec3, Vec3, f32) {
    let dir = p.direction.normalize_or(Vec3::Z);
    match p.kind {
        k if p.is_axis_organ() && k != OrganKind::Pod => {
            (p.base, p.tip(), (p.radius * 2.0).max(MIN_RADIUS))
        }
        OrganKind::Leaf | OrganKind::Cotyledon => {
            let s = p.area.max(0.0).sqrt();
            let c = p.base + dir * 0.7 * s;
            (c, c, (0.6 * s).max(MIN_RADIUS))
        }
        OrganKind::Pod | OrganKind::Flower => {
            let c = p.base + dir * 0.5 * p.length;
            (c, c, (0.5 * p.length).max(p.radius).max(MIN_RADIUS))
        }
        _ => (p.base, p.base, p.radius.max(0.5 * p.length).max(MIN_RADIUS)),
    }
}

/// Distance along the ray (`origin`, unit `dir`) to the closest approach to segment
/// `a`–`b`, and the distance between them there.
#[must_use]
pub fn ray_segment(origin: Vec3, dir: Vec3, a: Vec3, b: Vec3) -> (f32, f32) {
    let ab = b - a;
    let w = origin - a;
    let (aa, ad) = (ab.dot(ab), ab.dot(dir));
    let (aw, dw) = (ab.dot(w), dir.dot(w));
    let denom = aa - ad * ad;
    // Parameter along the segment, 0..1 (any point when the segment is a point or
    // parallel to the ray).
    let s = if aa < 1e-12 {
        0.0
    } else if denom.abs() < 1e-12 {
        (aw / aa).clamp(0.0, 1.0)
    } else {
        ((aw - ad * dw) / denom).clamp(0.0, 1.0)
    };
    let q = a + ab * s;
    let t = (q - origin).dot(dir).max(0.0);
    (t, (origin + dir * t - q).length())
}

/// The organ or plant a ray in the garden's sim frame hits, if any.
#[must_use]
pub fn pick(pots: &[PotView], origin: Vec3, dir: Vec3) -> Option<Picked> {
    let dir = dir.normalize_or(Vec3::NEG_Z);
    let mut best: Option<(f32, Picked)> = None;
    for (pi, pot) in pots.iter().enumerate() {
        let shift = Vec3::new(pot.center[0], pot.center[1], 0.0);
        let o = origin - shift;
        for (i, p) in pot.skeleton.organs.iter().enumerate() {
            if !p.alive {
                continue;
            }
            let (a, b, r) = pick_shape(p);
            let (t, d) = ray_segment(o, dir, a, b);
            if d <= r && best.is_none_or(|(bt, _)| t < bt) {
                best = Some((
                    t,
                    Picked {
                        pot: pi,
                        organ: Some(i),
                    },
                ));
            }
        }
    }
    if best.is_some() {
        return best.map(|(_, p)| p);
    }
    // No organ: the pot whose soil the ray passes through first.
    pots.iter()
        .enumerate()
        .filter(|(_, pot)| pot.container)
        .filter_map(|(pi, pot)| {
            let shift = Vec3::new(pot.center[0], pot.center[1], 0.0);
            let half = Vec3::new(pot.size.width / 2.0, pot.size.depth / 2.0, 0.0);
            let lo = shift - half - Vec3::Z * pot.size.soil_depth;
            let hi = shift + half + Vec3::Z * 0.02;
            ray_box(origin, dir, lo, hi).map(|t| (t, pi))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, pot)| Picked { pot, organ: None })
}

/// Entry distance of a ray into an axis-aligned box.
fn ray_box(origin: Vec3, dir: Vec3, lo: Vec3, hi: Vec3) -> Option<f32> {
    let inv = dir.recip();
    let a = (lo - origin) * inv;
    let b = (hi - origin) * inv;
    let near = a.min(b).max_element();
    let far = a.max(b).min_element();
    (far >= near.max(0.0)).then_some(near.max(0.0))
}

/// Where the left button went down, to tell a click from an orbit drag.
#[derive(Default)]
pub(crate) struct Press(Option<Vec2>);

/// Turns clicks into a selection.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn click_to_select(
    buttons: Res<ButtonInput<MouseButton>>,
    rig: Res<CameraRig>,
    garden: Res<Garden>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mut press: Local<Press>,
    mut selection: ResMut<Selection>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let cursor = window.cursor_position();
    if buttons.just_pressed(MouseButton::Left) {
        press.0 = if rig.input_blocked { None } else { cursor };
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let (Some(start), Some(end)) = (press.0.take(), cursor) else {
        return;
    };
    if rig.input_blocked || start.distance(end) > 4.0 {
        return;
    }
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, end) else {
        return;
    };
    let picked = pick(
        &garden.pots,
        bevy_to_sim(ray.origin),
        bevy_to_sim(*ray.direction),
    );
    if selection.picked != picked {
        selection.picked = picked;
    }
}

/// Outlines the selected organ (or the selected plant's pot).
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn outline_selection(
    selection: Res<Selection>,
    garden: Res<Garden>,
    mut gizmos: Gizmos,
) {
    let Some(picked) = selection.picked else {
        return;
    };
    let Some(pot) = garden.pots.get(picked.pot) else {
        return;
    };
    let shift = Vec3::new(pot.center[0], pot.center[1], 0.0);
    let color = Color::srgb(0.95, 0.97, 0.9);
    if let Some(p) = picked.organ.and_then(|i| pot.skeleton.organs.get(i)) {
        let (a, b, r) = pick_shape(p);
        let (a, b) = (sim_to_bevy(a + shift), sim_to_bevy(b + shift));
        if a.distance(b) < 1e-6 {
            gizmos.sphere(Isometry3d::from_translation(a), r, color);
        } else {
            gizmos.line(a, b, color);
            gizmos.sphere(Isometry3d::from_translation(a), r, color);
            gizmos.sphere(Isometry3d::from_translation(b), r, color);
        }
    } else {
        let h = pot.size.soil_depth;
        let c = sim_to_bevy(shift - Vec3::Z * (h / 2.0));
        gizmos.cube(
            Transform::from_translation(c).with_scale(Vec3::new(pot.size.width, h, pot.size.depth)),
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::PotSize;
    use ps_mesh::Skeleton;

    fn placement(kind: OrganKind, base: Vec3, direction: Vec3, length: f32) -> Placement {
        Placement {
            hook: None,
            kind,
            parent: None,
            axis_order: 0,
            alive: true,
            base,
            direction,
            azimuth: 0.0,
            length,
            radius: 0.002,
            area: 0.0,
            thermal_age: 0.0,
            attach: 1.0,
            copies: 1,
            coil: None,
            petiole: 0.0,
            simple_leaf: None,
        }
    }

    fn pot(organs: Vec<Placement>, x: f32) -> PotView {
        PotView {
            label: String::new(),
            center: [x, 0.0],
            size: PotSize::RHIZOBOX,
            skeleton: Skeleton {
                origin: Vec3::ZERO,
                organs,
                blade_aspect: None,
                container: None,
            },
            water_stress: 1.0,
            first_flower: None,
            organ_values: std::collections::BTreeMap::new(),
            soil_water: crate::garden::SoilFace::default(),
            container: true,
            supports: Vec::new(),
            obstacles: Vec::new(),
        }
    }

    #[test]
    fn frames_round_trip() {
        let p = Vec3::new(0.1, 0.2, 0.3);
        assert!((bevy_to_sim(sim_to_bevy(p)) - p).length() < 1e-6);
    }

    #[test]
    fn ray_segment_distance() {
        // A ray along +y passing 1 cm east of a vertical segment.
        let (t, d) = ray_segment(
            Vec3::new(0.01, -1.0, 0.05),
            Vec3::Y,
            Vec3::ZERO,
            Vec3::Z * 0.1,
        );
        assert!((t - 1.0).abs() < 1e-5 && (d - 0.01).abs() < 1e-5);
    }

    #[test]
    fn picks_the_nearest_organ_then_the_pot() {
        let stem = placement(OrganKind::Internode, Vec3::ZERO, Vec3::Z, 0.2);
        let mut leaf = placement(OrganKind::Leaf, Vec3::new(0.0, 0.0, 0.1), Vec3::X, 0.0);
        leaf.area = 0.0025; // 5 cm square: center 3.5 cm east of its base
        let pots = vec![pot(vec![stem, leaf], 0.0), pot(Vec::new(), 1.0)];
        // Looking north from the south at the stem.
        let hit = pick(&pots, Vec3::new(0.0, -2.0, 0.05), Vec3::Y);
        assert_eq!(
            hit,
            Some(Picked {
                pot: 0,
                organ: Some(0)
            })
        );
        // At the leaf.
        let hit = pick(&pots, Vec3::new(0.035, -2.0, 0.1), Vec3::Y);
        assert_eq!(
            hit,
            Some(Picked {
                pot: 0,
                organ: Some(1)
            })
        );
        // Through the second pot's soil.
        let hit = pick(&pots, Vec3::new(1.0, -2.0, -0.1), Vec3::Y);
        assert_eq!(
            hit,
            Some(Picked {
                pot: 1,
                organ: None
            })
        );
        // Into the sky.
        assert_eq!(pick(&pots, Vec3::new(0.5, -2.0, 1.0), Vec3::Y), None);
    }
}
