//! Stem and root axes: chains of internodes (or root segments) drawn as one tube each.
//!
//! An axis continues through an organ's first child of the same kind and axis order;
//! any other child of that kind starts a new axis (a branch or a lateral root). The
//! hypocotyl starts the main stem; the first root segment starts the tap root.

use glam::Vec3;
use ps_graph::{OrganId, OrganKind};

use crate::skeleton::is_axis_kind;
use crate::{MeshData, Placement, Skeleton, TubeNode, tube};

/// Shoot or root.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AxisKind {
    /// Internodes (and tendrils).
    Shoot,
    /// Root segments.
    Root,
}

/// One axis: its organs in base-to-tip order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Axis {
    /// Shoot or root.
    pub kind: AxisKind,
    /// Branching order of the axis.
    pub axis_order: u8,
    /// Organs from base to tip. The first one's id names the axis for its whole life.
    pub organs: Vec<OrganId>,
    /// Identical copies drawn (a root cohort), spread around the parent axis.
    pub copies: u16,
}

impl Axis {
    /// The stable key of this axis: its basal organ.
    #[must_use]
    pub fn key(&self) -> OrganId {
        self.organs[0]
    }
}

/// Groups the axis organs of `skeleton` into axes, in order of their basal organ's id.
///
/// With `alive_only`, a dead organ ends its axis (an axis dies with its basal internode,
/// and a dead tip segment is simply not drawn).
#[must_use]
pub fn axes(skeleton: &Skeleton, alive_only: bool) -> Vec<Axis> {
    let n = skeleton.len();
    let mut axis_of: Vec<Option<usize>> = vec![None; n];
    let mut continued = vec![false; n];
    let mut out: Vec<Axis> = Vec::new();
    for (i, p) in skeleton.organs.iter().enumerate() {
        if !is_axis_kind(p.kind) || (alive_only && !p.alive) {
            continue;
        }
        let kind = if p.kind == OrganKind::Root {
            AxisKind::Root
        } else {
            AxisKind::Shoot
        };
        let parent_axis = p.parent.and_then(|pid| {
            let j = pid.index();
            let parent = skeleton.organs.get(j)?;
            let same = is_axis_kind(parent.kind)
                && (parent.kind == OrganKind::Root) == (p.kind == OrganKind::Root)
                && parent.axis_order == p.axis_order
                && !continued[j];
            if same {
                axis_of[j].map(|a| (a, j))
            } else {
                None
            }
        });
        let id = OrganId(i as u32);
        if let Some((a, j)) = parent_axis {
            continued[j] = true;
            out[a].organs.push(id);
            axis_of[i] = Some(a);
        } else {
            axis_of[i] = Some(out.len());
            out.push(Axis {
                kind,
                axis_order: p.axis_order,
                organs: vec![id],
                copies: p.copies.max(1),
            });
        }
    }
    out
}

/// How an axis is drawn.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AxisStyle {
    /// Faces around the tube.
    pub sides: u32,
    /// Smallest drawn radius, m, so fine roots stay visible.
    pub min_radius: f32,
    /// Apex radius as a fraction of the last segment's radius.
    pub tip_taper: f32,
}

impl AxisStyle {
    /// Stems: eight sides, 0.5 mm minimum, a pointed apex.
    pub const STEM: Self = Self {
        sides: 8,
        min_radius: 0.000_5,
        tip_taper: 0.3,
    };
    /// Roots: five sides, 0.3 mm minimum.
    pub const ROOT: Self = Self {
        sides: 5,
        min_radius: 0.000_3,
        tip_taper: 0.5,
    };
}

/// Tube nodes per turn of a twining stem: enough to read as a smooth coil.
pub const COIL_POINTS_PER_TURN: u32 = 12;

/// How many centerline nodes organ `p` adds to its axis after the base: one (its tip),
/// or a point per twelfth of a turn along its coil.
#[must_use]
pub fn nodes_after_base(p: &Placement) -> usize {
    match p.coil.filter(|_| p.length > 0.0) {
        Some(coil) => coil.sample_count(p.length, COIL_POINTS_PER_TURN),
        None => 1,
    }
}

/// The centerline of an axis: its base, then each organ's tip (a twining internode adds
/// points along its coil). The radius at a joint is
/// the mean of the two segments meeting there, so the taper is continuous.
#[must_use]
pub fn axis_nodes(skeleton: &Skeleton, axis: &Axis, style: AxisStyle) -> Vec<TubeNode> {
    let parts: Vec<_> = axis
        .organs
        .iter()
        .filter_map(|id| skeleton.get(*id))
        .collect();
    let Some(first) = parts.first() else {
        return Vec::new();
    };
    let visible = |r: f32, grown: bool| if grown { r.max(style.min_radius) } else { 0.0 };
    let mut nodes = Vec::with_capacity(parts.len() + 1);
    nodes.push(TubeNode {
        position: first.base,
        radius: visible(first.radius, first.length > 0.0),
    });
    for (k, p) in parts.iter().enumerate() {
        let r = match parts.get(k + 1) {
            Some(next) => 0.5 * (p.radius + next.radius),
            None => p.radius * style.tip_taper,
        };
        let r = visible(r, p.length > 0.0);
        match p.coil.filter(|_| p.length > 0.0) {
            // A twining internode: follow its helix, tapering from the joint below.
            Some(coil) => {
                let r0 = nodes.last().map_or(r, |n: &TubeNode| n.radius);
                let points = coil.samples(p.base, p.tip(), p.length, COIL_POINTS_PER_TURN);
                let n = points.len() as f32;
                for (j, position) in points.into_iter().enumerate() {
                    let f = (j + 1) as f32 / n;
                    nodes.push(TubeNode {
                        position,
                        radius: r0 + (r - r0) * f,
                    });
                }
            }
            None => nodes.push(TubeNode {
                position: p.tip(),
                radius: r,
            }),
        }
    }
    nodes
}

/// The golden angle, rad: successive copies never line up.
pub const GOLDEN_ANGLE: f32 = 2.399_963;

/// Rotations that place the copies of a cohort axis: copy `k` turns `k` golden angles
/// about the parent organ's direction, through the axis base (identity for copy 0).
#[must_use]
pub fn copy_transforms(skeleton: &Skeleton, axis: &Axis) -> Vec<glam::Affine3A> {
    let first = skeleton.get(axis.key());
    let pivot = first.map_or(Vec3::ZERO, |p| p.base);
    let about = first
        .and_then(|p| p.parent)
        .and_then(|pid| skeleton.get(pid))
        .map_or(Vec3::Z, |p| p.direction)
        .normalize_or(Vec3::Z);
    (0..axis.copies.max(1))
        .map(|k| {
            let r = glam::Quat::from_axis_angle(about, f32::from(k) * GOLDEN_ANGLE);
            glam::Affine3A::from_translation(pivot)
                * glam::Affine3A::from_quat(r)
                * glam::Affine3A::from_translation(-pivot)
        })
        .collect()
}

/// Builds the tube mesh of one axis, repeated for each copy of a root cohort. Vertices
/// come copy by copy, each copy laid out like a single tube.
#[must_use]
pub fn axis_mesh(skeleton: &Skeleton, axis: &Axis, style: AxisStyle) -> MeshData {
    let nodes = axis_nodes(skeleton, axis, style);
    let fallback = match axis.kind {
        AxisKind::Shoot => Vec3::Z,
        AxisKind::Root => Vec3::NEG_Z,
    };
    let mut one = MeshData::new();
    tube(&nodes, style.sides, false, fallback, &mut one);
    if axis.copies <= 1 {
        return one;
    }
    let mut mesh = MeshData::new();
    for t in copy_transforms(skeleton, axis) {
        mesh.append_transformed(&one, t);
    }
    mesh
}

/// What an axis looked like when its mesh was last built, to decide when to rebuild.
///
/// The design budget (§4) rebuilds a stem mesh only when its size changed by more than
/// 2%; comparing node positions as well catches a branch whose base moved because the
/// stem below it grew.
#[derive(Clone, Debug, PartialEq)]
pub struct AxisSignature {
    nodes: Vec<TubeNode>,
    length: f32,
    copies: u16,
}

impl AxisSignature {
    /// The signature of an axis as currently placed.
    #[must_use]
    pub fn of(skeleton: &Skeleton, axis: &Axis, style: AxisStyle) -> Self {
        let nodes = axis_nodes(skeleton, axis, style);
        let length = nodes
            .windows(2)
            .map(|w| (w[1].position - w[0].position).length())
            .sum();
        Self {
            nodes,
            length,
            copies: axis.copies,
        }
    }

    /// True when `self` (the current state) differs from `built` (the state the mesh was
    /// built from) by more than `threshold`: a different node count, a total length or
    /// any radius changed by more than that fraction, or any node moved by more than
    /// that fraction of the axis length.
    #[must_use]
    pub fn needs_rebuild(&self, built: &Self, threshold: f32) -> bool {
        if self.nodes.len() != built.nodes.len() || self.copies != built.copies {
            return true;
        }
        let rel = |now: f32, then: f32| {
            let scale = now.abs().max(then.abs());
            scale > 0.0 && (now - then).abs() > threshold * scale
        };
        if rel(self.length, built.length) {
            return true;
        }
        let tolerance = threshold * self.length.max(built.length);
        self.nodes
            .iter()
            .zip(&built.nodes)
            .any(|(a, b)| rel(a.radius, b.radius) || (a.position - b.position).length() > tolerance)
    }
}
