//! Tapered tubes swept along a polyline, with smooth joins.
//!
//! Each node gets one ring of vertices. The ring's tangent is the average of the two
//! adjacent segment directions, and ring frames are carried from node to node by
//! parallel transport, so the tube neither pinches nor twists at a bend. Normals are the
//! ring's radial directions tilted by the taper, which stay unit length even where the
//! radius is zero (the apex of a tapered stem, or an organ that has not grown yet).

use core::f32::consts::TAU;

use glam::{Vec2, Vec3};

use crate::MeshData;

/// Steepest taper used for normals; beyond it the surface is treated as a cap.
const MAX_SLOPE: f32 = 8.0;

/// One node of a tube's centerline.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TubeNode {
    /// Centerline point, m.
    pub position: Vec3,
    /// Radius at that point, m.
    pub radius: f32,
}

/// A unit vector perpendicular to `t`.
pub(crate) fn any_perpendicular(t: Vec3) -> Vec3 {
    let helper = if t.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    t.cross(helper).normalize_or(Vec3::Y)
}

/// Tangents at each node: central differences inside, one-sided at the ends, carrying
/// the previous tangent across zero-length segments (and `fallback` before any).
pub(crate) fn tangents(points: &[Vec3], fallback: Vec3) -> Vec<Vec3> {
    let n = points.len();
    let seg: Vec<Vec3> = points
        .windows(2)
        .map(|w| (w[1] - w[0]).normalize_or_zero())
        .collect();
    let mut out = Vec::with_capacity(n);
    let mut last = fallback;
    for i in 0..n {
        let before = if i > 0 { seg[i - 1] } else { Vec3::ZERO };
        let after = if i < seg.len() { seg[i] } else { Vec3::ZERO };
        let t = (before + after).normalize_or_zero();
        let t = if t == Vec3::ZERO {
            // A hairpin or a degenerate run: use whichever side has a direction.
            let one = if after == Vec3::ZERO { before } else { after };
            if one == Vec3::ZERO { last } else { one }
        } else {
            t
        };
        last = t;
        out.push(t);
    }
    // Leading degenerate nodes took the fallback; give them the first real tangent.
    if let Some(first_real) = out.iter().position(|t| *t != fallback) {
        let t0 = out[first_real];
        for t in out.iter_mut().take(first_real) {
            *t = t0;
        }
    }
    out
}

/// Sweeps a tube with `sides` faces around the centerline `nodes` into `mesh`.
///
/// Each ring has `sides + 1` vertices (the seam is duplicated for texture coordinates),
/// so a tube of `n` nodes adds `n · (sides + 1)` vertices and `(n − 1) · sides · 2`
/// triangles. With `cap_end`, a last ring of nonzero radius is closed with a fan (`sides + 2`
/// more vertices and `sides` triangles). `fallback` is the tangent used when every node
/// coincides. Fewer than two nodes, or fewer than three sides, add nothing.
pub fn tube(nodes: &[TubeNode], sides: u32, cap_end: bool, fallback: Vec3, mesh: &mut MeshData) {
    if nodes.len() < 2 || sides < 3 {
        return;
    }
    let points: Vec<Vec3> = nodes.iter().map(|n| n.position).collect();
    let fallback = fallback.normalize_or(Vec3::Z);
    let tan = tangents(&points, fallback);

    // Radius slope dr/ds at each node, so normals tilt with the taper (a surface of
    // revolution's normal is radial − tangent · dr/ds, normalized).
    let arc: Vec<f32> = core::iter::once(0.0)
        .chain(points.windows(2).scan(0.0, |acc, w| {
            *acc += (w[1] - w[0]).length();
            Some(*acc)
        }))
        .collect();
    let radius = |k: usize| {
        let r = nodes[k].radius;
        if r.is_finite() { r.max(0.0) } else { 0.0 }
    };
    let last = nodes.len() - 1;
    let slope: Vec<f32> = (0..nodes.len())
        .map(|k| {
            let (lo, hi) = (k.saturating_sub(1), (k + 1).min(last));
            let ds = arc[hi] - arc[lo];
            if ds > 1e-9 {
                ((radius(hi) - radius(lo)) / ds).clamp(-MAX_SLOPE, MAX_SLOPE)
            } else {
                0.0
            }
        })
        .collect();

    // Parallel-transported frames.
    let mut normal = any_perpendicular(tan[0]);
    let mut v_coord = 0.0_f32;
    let first_vertex = mesh.positions.len() as u32;
    for (i, node) in nodes.iter().enumerate() {
        let t = tan[i];
        if i > 0 {
            v_coord += (points[i] - points[i - 1]).length();
            let projected = normal - t * normal.dot(t);
            normal = projected.normalize_or_zero();
            if normal == Vec3::ZERO {
                normal = any_perpendicular(t);
            }
        }
        let binormal = t.cross(normal);
        let r = radius(i);
        for j in 0..=sides {
            let angle = TAU * j as f32 / sides as f32;
            let radial = normal * angle.cos() + binormal * angle.sin();
            mesh.push_vertex(
                node.position + radial * r,
                (radial - t * slope[i]).normalize_or(radial),
                Vec2::new(j as f32 / sides as f32, v_coord),
            );
        }
    }
    let ring = sides + 1;
    for i in 0..(nodes.len() as u32 - 1) {
        for j in 0..sides {
            let a = first_vertex + i * ring + j;
            let b = a + 1;
            let c = a + ring;
            let d = c + 1;
            mesh.push_triangle(a, b, c);
            mesh.push_triangle(b, d, c);
        }
    }
    if cap_end {
        let last = nodes[nodes.len() - 1];
        if last.radius > 0.0 && last.radius.is_finite() {
            let t = tan[tan.len() - 1];
            let start = first_vertex + (nodes.len() as u32 - 1) * ring;
            // The cap needs its own vertices so its normal is the tangent, not radial.
            let cap_first = mesh.positions.len() as u32;
            for j in 0..=sides {
                let p = Vec3::from(mesh.positions[(start + j) as usize]);
                mesh.push_vertex(p, t, Vec2::new(j as f32 / sides as f32, v_coord));
            }
            let center = mesh.push_vertex(last.position, t, Vec2::new(0.5, v_coord));
            for j in 0..sides {
                mesh.push_triangle(cap_first + j, cap_first + j + 1, center);
            }
        }
    }
}
