//! Carbon flow arrows (design plan §5.2, WP26): with the carbon flow lens on, an arrow
//! along every living stem and root segment shows which way the hour's phloem carbon
//! moved through it, longer and brighter for more.
//!
//! A segment's value is the carbon moved from its parent into it, so a positive value
//! flows from the segment's base (where its parent is) toward its tip. The arrows are
//! an overlay drawn with gizmos; the plant's geometry never changes.

use bevy::prelude::*;
use ps_graph::OrganKind;
use ps_mesh::Placement;

use crate::garden::Garden;
use crate::lens::Lens;
use crate::scene::sim_to_bevy;

/// Flows below this (mg C per hour) get no arrow.
const MIN_FLOW: f32 = 0.05;

/// The arrow for one segment with `flow` mg C per hour, in the plant's frame: start and
/// end points, or `None` for no arrow. `full` is the flow drawn at full length.
#[must_use]
pub fn arrow(p: &Placement, flow: f32, full: f32) -> Option<(Vec3, Vec3)> {
    if !p.alive
        || !matches!(p.kind, OrganKind::Internode | OrganKind::Root)
        || !flow.is_finite()
        || flow.abs() < MIN_FLOW
        || p.length <= 0.0
    {
        return None;
    }
    let scale = (flow.abs() / full.max(1e-6)).clamp(0.2, 1.0);
    let half = 0.4 * p.length * scale;
    let mid = p.base + p.direction * (0.5 * p.length);
    let way = p.direction * flow.signum();
    Some((mid - way * half, mid + way * half))
}

/// The arrows' gizmo group: drawn over the stems they run along, a little thicker than
/// the selection outline.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct FlowGizmos;

/// Sets the arrows' line width and draws them in front of the tubes they sit inside.
pub(crate) fn configure(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<FlowGizmos>();
    config.line.width = 3.0;
    config.depth_bias = -0.02;
}

/// Draws the arrows while the carbon flow lens is on.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn flow_arrows(lens: Res<Lens>, garden: Res<Garden>, mut gizmos: Gizmos<FlowGizmos>) {
    if *lens != Lens::CarbonFlow {
        return;
    }
    let Some(info) = lens.info() else {
        return;
    };
    for pot in &garden.pots {
        let shift = Vec3::new(pot.center[0], pot.center[1], 0.0);
        for (i, p) in pot.skeleton.organs.iter().enumerate() {
            let Some(flow) = pot.organ_value(Lens::CarbonFlow, i) else {
                continue;
            };
            let Some((a, b)) = arrow(p, flow, info.max) else {
                continue;
            };
            let c = info.color(flow).unwrap_or([1.0; 3]);
            let color = Color::linear_rgb(c[0], c[1], c[2]);
            gizmos
                .arrow(sim_to_bevy(a + shift), sim_to_bevy(b + shift), color)
                .with_tip_length(0.25 * a.distance(b));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(kind: OrganKind) -> Placement {
        Placement {
            kind,
            parent: None,
            axis_order: 0,
            alive: true,
            base: Vec3::ZERO,
            direction: Vec3::Z,
            azimuth: 0.0,
            length: 0.1,
            radius: 0.002,
            area: 0.0,
            thermal_age: 0.0,
            attach: 1.0,
            copies: 1,
            coil: None,
            petiole: 0.0,
            simple_leaf: None,
            hook: None,
        }
    }

    #[test]
    fn imports_point_away_from_the_parent_and_exports_toward_it() {
        let stem = segment(OrganKind::Internode);
        let (a, b) = arrow(&stem, 3.0, 5.0).unwrap_or_else(|| unreachable!());
        assert!(b.z > a.z, "import flows base to tip");
        let (a, b) = arrow(&stem, -5.0, 5.0).unwrap_or_else(|| unreachable!());
        assert!(b.z < a.z, "export flows tip to base");
        assert!(
            (a.distance(b) - 0.08).abs() < 1e-6,
            "full flow, 80% of the segment"
        );
        let (a, b) = arrow(&stem, 0.1, 5.0).unwrap_or_else(|| unreachable!());
        assert!(
            (a.distance(b) - 0.016).abs() < 1e-6,
            "small flows keep a visible stub"
        );
    }

    #[test]
    fn leaves_dead_organs_and_tiny_flows_get_no_arrow() {
        assert!(arrow(&segment(OrganKind::Leaf), 3.0, 5.0).is_none());
        let mut dead = segment(OrganKind::Root);
        dead.alive = false;
        assert!(arrow(&dead, 3.0, 5.0).is_none());
        assert!(arrow(&segment(OrganKind::Root), 0.001, 5.0).is_none());
        assert!(arrow(&segment(OrganKind::Root), f32::NAN, 5.0).is_none());
    }
}
