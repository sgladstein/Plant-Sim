//! Stem and root axes: chains of internodes (or root segments) drawn as one tube each.
//!
//! An axis continues through an organ's first child of the same kind and axis order;
//! any other child of that kind starts a new axis (a branch or a lateral root). The
//! hypocotyl starts the main stem; the first root segment starts the tap root.
//!
//! Each broadleaf leaf adds a one-organ petiole axis: a curved tube from its node to the
//! pulvinus where the blade template starts.

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
    /// A leaf's petiole, keyed by the leaf.
    Petiole,
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
    let broadleaf = skeleton.blade_aspect.is_none();
    for (i, p) in skeleton.organs.iter().enumerate() {
        if alive_only && !p.alive {
            continue;
        }
        if p.kind == OrganKind::Leaf {
            if broadleaf && p.petiole_length() > 0.0 {
                out.push(Axis {
                    kind: AxisKind::Petiole,
                    axis_order: p.axis_order,
                    organs: vec![OrganId(i as u32)],
                    copies: 1,
                });
            }
            continue;
        }
        if !is_axis_kind(p.kind) {
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
    /// Petioles: six sides, 0.35 mm minimum, ending in the pulvinus.
    pub const PETIOLE: Self = Self {
        sides: 6,
        min_radius: 0.000_35,
        tip_taper: 1.0,
    };

    /// The usual style for an axis of `kind`.
    #[must_use]
    pub fn for_kind(kind: AxisKind) -> Self {
        match kind {
            AxisKind::Shoot => Self::STEM,
            AxisKind::Root => Self::ROOT,
            AxisKind::Petiole => Self::PETIOLE,
        }
    }
}

/// Centerline points of a petiole after its base. A turgid petiole rises from its node
/// more steeply than its chord and arches over to the pulvinus.
pub const PETIOLE_POINTS: usize = 7;

/// Petiole radius over the square root of blade area: about 0.6 mm for a 20 cm² leaf.
pub const PETIOLE_RADIUS: f32 = 0.013;

/// Where a stem's node swellings sit: the ring before and after each node, as fractions
/// of the internode, and how much wider the node is than the stem beside it.
const NODE_SHOULDER: f32 = 0.12;
const NODE_SWELL: f32 = 1.3;

/// Tube nodes per turn of a twining stem: enough to read as a smooth coil.
pub const COIL_POINTS_PER_TURN: u32 = 12;

/// How many centerline nodes organ `p` adds to its axis after the base: one (its tip),
/// or a point per twelfth of a turn along its coil.
#[must_use]
pub fn nodes_after_base(p: &Placement) -> usize {
    if p.kind == OrganKind::Leaf {
        return PETIOLE_POINTS;
    }
    if p.kind == OrganKind::Internode && p.hook.is_some() && p.coil.is_none() {
        // A shoulder above the base, the hook's start, then its arc.
        return 2 + crate::Hook::POINTS;
    }
    match p.coil.filter(|_| p.length > 0.0) {
        Some(coil) => coil.sample_count(p.length, COIL_POINTS_PER_TURN),
        // A straight internode swells at the node it ends in: a ring before the node,
        // the node, and one after it is added by the next internode.
        None if p.kind == OrganKind::Internode => 3,
        None => 1,
    }
}

/// The curved centerline of a leaf's petiole: from the node to [`crate::pulvinus`],
/// leaving the stem a little steeper than the straight line and arriving a little
/// flatter, thickest at its two pulvini.
fn petiole_nodes(p: &Placement, style: AxisStyle) -> Vec<TubeNode> {
    let length = p.petiole_length();
    let start = p.base;
    let end = crate::pulvinus(p);
    let d = p.direction.normalize_or(Vec3::X);
    let h = Vec3::new(d.x, d.y, 0.0).normalize_or(Vec3::X);
    let elevation = d.z.clamp(-1.0, 1.0).asin();
    // The control point lies along a direction 0.35 rad steeper (at most near vertical).
    let up = (elevation + 0.35).min(1.45);
    let control = start + (h * up.cos() + Vec3::Z * up.sin()) * (0.5 * length);
    let r = (PETIOLE_RADIUS * p.area.max(0.0).sqrt()).max(style.min_radius);
    let grown = length > 0.0 && p.area > 0.0;
    (0..=PETIOLE_POINTS)
        .map(|k| {
            let t = k as f32 / PETIOLE_POINTS as f32;
            let s = 1.0 - t;
            let position = start * (s * s) + control * (2.0 * s * t) + end * (t * t);
            // Swollen pulvinus at the base, a smaller one under the blade.
            let swell = 1.0 + 0.6 * (1.0 - t * 5.0).max(0.0) + 0.35 * (t * 6.0 - 5.0).max(0.0);
            TubeNode {
                position,
                radius: if grown {
                    r * swell * (1.0 - 0.25 * t)
                } else {
                    0.0
                },
            }
        })
        .collect()
}

/// The centerline of an axis: its base, then each organ's tip (a twining internode adds
/// points along its coil). The radius at a joint is
/// the mean of the two segments meeting there, so the taper is continuous.
#[must_use]
pub fn axis_nodes(skeleton: &Skeleton, axis: &Axis, style: AxisStyle) -> Vec<TubeNode> {
    if axis.kind == AxisKind::Petiole {
        return axis
            .organs
            .first()
            .and_then(|id| skeleton.get(*id))
            .map_or_else(Vec::new, |p| petiole_nodes(p, style));
    }
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
            None if p.kind == OrganKind::Internode && p.hook.is_some() && p.coil.is_none() => {
                // The emerging hook: straight up to where it bends, then the arc,
                // tapering to the tip.
                let hook = p.hook.unwrap_or(crate::Hook {
                    amount: 0.0,
                    toward: 0.0,
                });
                let own = visible(p.radius, p.length > 0.0);
                let r0 = nodes.last().map_or(own, |n: &TubeNode| n.radius);
                let (start, arc) = hook.points(p);
                nodes.push(TubeNode {
                    position: p.base.lerp(start, NODE_SHOULDER),
                    radius: r0.max(own),
                });
                nodes.push(TubeNode {
                    position: start,
                    radius: own,
                });
                let n = arc.len() as f32;
                for (j, position) in arc.into_iter().enumerate() {
                    let f = (j + 1) as f32 / n;
                    nodes.push(TubeNode {
                        position,
                        radius: own + (r - own) * f,
                    });
                }
            }
            None if p.kind == OrganKind::Internode => {
                // Shoulders either side of the node, and the node itself swollen (not at
                // the apex, which tapers to its point).
                let own = visible(p.radius, p.length > 0.0);
                let r0 = nodes.last().map_or(own, |n: &TubeNode| n.radius);
                let below = if k == 0 { r0 } else { own };
                nodes.push(TubeNode {
                    position: p.base.lerp(p.tip(), NODE_SHOULDER),
                    radius: below.max(own),
                });
                nodes.push(TubeNode {
                    position: p.base.lerp(p.tip(), 1.0 - NODE_SHOULDER),
                    radius: own.max(r),
                });
                let swell = if k + 1 < parts.len() { NODE_SWELL } else { 1.0 };
                nodes.push(TubeNode {
                    position: p.tip(),
                    radius: r * swell,
                });
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

/// Transforms that place the copies of a cohort axis. Copy `k` turns `k` golden angles
/// about the vertical through the axis base (identity for copy 0), so every copy keeps
/// the representative's dip: the sim mirrors a cohort's representative downward
/// (decision 0013), and turning about a near-horizontal parent would tip half the copies
/// up out of the soil. When the skeleton knows its container, a copy whose organs would
/// leave the soil is turned half a turn more, and if that still leaves it, shrunk about
/// its base until it fits.
#[must_use]
pub fn copy_transforms(skeleton: &Skeleton, axis: &Axis) -> Vec<glam::Affine3A> {
    let first = skeleton.get(axis.key());
    let pivot = first.map_or(Vec3::ZERO, |p| p.base);
    let tips: Vec<Vec3> = axis
        .organs
        .iter()
        .filter_map(|&id| skeleton.get(id))
        .map(Placement::tip)
        .collect();
    let about = |angle: f32| {
        glam::Affine3A::from_translation(pivot)
            * glam::Affine3A::from_quat(glam::Quat::from_rotation_z(angle))
            * glam::Affine3A::from_translation(-pivot)
    };
    let fits = |t: &glam::Affine3A| {
        skeleton
            .container
            .is_none_or(|c| tips.iter().all(|&p| c.contains(t.transform_point3(p))))
    };
    (0..axis.copies.max(1))
        .map(|k| {
            let angle = f32::from(k) * GOLDEN_ANGLE;
            let turned = about(angle);
            if k == 0 || fits(&turned) {
                return turned;
            }
            let flipped = about(angle + std::f32::consts::PI);
            if fits(&flipped) {
                return flipped;
            }
            let mut scale = 1.0_f32;
            let mut t = turned;
            while scale > 0.1 {
                scale *= 0.8;
                t = glam::Affine3A::from_translation(pivot)
                    * glam::Affine3A::from_scale(Vec3::splat(scale))
                    * glam::Affine3A::from_translation(-pivot)
                    * turned;
                if fits(&t) {
                    break;
                }
            }
            t
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
        AxisKind::Petiole => Vec3::X,
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
