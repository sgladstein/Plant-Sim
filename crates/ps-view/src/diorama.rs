//! Pot geometry: a rhizobox or a round pot, each with a glass front (design plan §2.1, §4).
//!
//! In the pot's sim frame (x east, y north, z up, soil surface at z = 0) the soil fills
//! `x ∈ [−W/2, W/2]`, `y ∈ [−D/2, D/2]`, `z ∈ [−H, 0]`. The glass is the south face
//! (`y = −D/2`), so from the default camera the roots against it and inside the slab are
//! visible; the other faces are soil seen from inside, banded by depth, inside wooden
//! walls. A round pot of radius `R` fills `x² + y² ≤ R²`: its back half (`y ≥ 0`) is
//! terracotta lined with soil, and its front half is a curved glass window. Geometry is
//! built as [`MeshData`] with vertex colors, so it is testable without
//! a GPU.

use bevy::math::{Vec2, Vec3};
use ps_mesh::MeshData;

use crate::garden::{PotShape, PotSize, SoilFace};

/// Wall thickness, m.
pub const WALL: f32 = 0.012;

/// A mesh plus one RGBA color per vertex.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColoredMesh {
    /// Geometry.
    pub mesh: MeshData,
    /// Linear RGBA per vertex.
    pub colors: Vec<[f32; 4]>,
}

impl ColoredMesh {
    /// A quad from `corner` spanning `u` and `v`; its normal is `u × v`.
    pub fn quad(&mut self, corner: Vec3, u: Vec3, v: Vec3, color: [f32; 4]) {
        let n = u.cross(v).normalize_or(Vec3::Z);
        let m = &mut self.mesh;
        let a = m.push_vertex(corner, n, Vec2::ZERO);
        let b = m.push_vertex(corner + u, n, Vec2::X);
        let c = m.push_vertex(corner + u + v, n, Vec2::ONE);
        let d = m.push_vertex(corner + v, n, Vec2::Y);
        m.push_triangle(a, b, c);
        m.push_triangle(a, c, d);
        self.colors.extend([color; 4]);
    }

    /// A quad through `a b c d` (counterclockwise seen from the side `n` points to).
    pub fn quad4(&mut self, [a, b, c, d]: [Vec3; 4], n: Vec3, color: [f32; 4]) {
        let m = &mut self.mesh;
        let i = [
            m.push_vertex(a, n, Vec2::ZERO),
            m.push_vertex(b, n, Vec2::X),
            m.push_vertex(c, n, Vec2::ONE),
            m.push_vertex(d, n, Vec2::Y),
        ];
        m.push_triangle(i[0], i[1], i[2]);
        m.push_triangle(i[0], i[2], i[3]);
        self.colors.extend([color; 4]);
    }

    /// A parametric surface sampled on an `nu × nv` grid of shared vertices. `pos` maps
    /// (u, v) in \[0, 1\]² to a point, `normal` gives the facing at a point, and `color`
    /// the color at grid node (i, k) and point. Each triangle is wound to face its normal;
    /// degenerate triangles (at a disk's center) are left out.
    pub fn grid(
        &mut self,
        (nu, nv): (usize, usize),
        pos: impl Fn(f32, f32) -> Vec3,
        normal: impl Fn(Vec3) -> Vec3,
        color: impl Fn(usize, usize, Vec3) -> [f32; 4],
    ) {
        let (nu, nv) = (nu.max(1), nv.max(1));
        let base = self.mesh.vertex_count() as u32;
        let mut points = Vec::with_capacity((nu + 1) * (nv + 1));
        for k in 0..=nv {
            for i in 0..=nu {
                let uv = Vec2::new(i as f32 / nu as f32, k as f32 / nv as f32);
                let p = pos(uv.x, uv.y);
                self.mesh
                    .push_vertex(p, normal(p).normalize_or(Vec3::Z), uv);
                self.colors.push(color(i, k, p));
                points.push(p);
            }
        }
        let row = nu + 1;
        let tri = |m: &mut MeshData, [a, b, c]: [usize; 3]| {
            let g = (points[b] - points[a]).cross(points[c] - points[a]);
            if g.length_squared() < 1e-14 {
                return;
            }
            let n = Vec3::from(m.normals[base as usize + a]);
            let [a, b, c] = [a, b, c].map(|j| base + j as u32);
            if g.dot(n) >= 0.0 {
                m.push_triangle(a, b, c);
            } else {
                m.push_triangle(a, c, b);
            }
        };
        for k in 0..nv {
            for i in 0..nu {
                let a = k * row + i;
                tri(&mut self.mesh, [a, a + 1, a + row + 1]);
                tri(&mut self.mesh, [a, a + row + 1, a + row]);
            }
        }
    }

    /// An axis-aligned box with outward faces.
    pub fn cuboid(&mut self, min: Vec3, max: Vec3, color: [f32; 4]) {
        let s = max - min;
        let (x, y, z) = (Vec3::X * s.x, Vec3::Y * s.y, Vec3::Z * s.z);
        self.quad(min, y, x, color); // bottom, −z
        self.quad(min + z, x, y, color); // top, +z
        self.quad(min, x, z, color); // south, −y
        self.quad(min + y, z, x, color); // north, +y
        self.quad(min, z, y, color); // west, −x
        self.quad(min + x, y, z, color); // east, +x
    }
}

/// Soil color at a depth fraction (0 = surface, 1 = bottom): dark topsoil to paler subsoil.
#[must_use]
pub fn soil_color(depth_fraction: f32) -> [f32; 4] {
    let t = depth_fraction.clamp(0.0, 1.0);
    let top = [0.1, 0.055, 0.025];
    let sub = [0.32, 0.2, 0.1];
    [
        top[0] + (sub[0] - top[0]) * t,
        top[1] + (sub[1] - top[1]) * t,
        top[2] + (sub[2] - top[2]) * t,
        1.0,
    ]
}

/// Soil crumb cell size, m: the soil faces are tiled with cells of slightly varied
/// brightness so the cut face reads as soil, not as a flat brown card.
const CRUMB: f32 = 0.012;

/// A deterministic value in [−1, 1] for a cell.
fn jitter(a: usize, b: usize, c: usize) -> f32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B9)
        ^ (b as u32).wrapping_mul(0x85EB_CA6B)
        ^ (c as u32).wrapping_mul(0xC2B2_AE35);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 32767.5 - 1.0
}

fn speckle(c: [f32; 4], j: f32) -> [f32; 4] {
    let k = 1.0 + 0.16 * j;
    [c[0] * k, c[1] * k, c[2] * k, c[3]]
}

/// A face from `corner` spanning `u` (horizontal) and `v` (up), as a grid of shared
/// vertices whose colors follow depth with a little smooth mottling (vertex colors
/// interpolate, so it reads as soil texture, not as pixels). `flip` reverses the facing.
#[allow(clippy::too_many_arguments)]
fn crumb_face(
    m: &mut ColoredMesh,
    face: usize,
    corner: Vec3,
    u: Vec3,
    v: Vec3,
    h: f32,
    flip: bool,
    tone: f32,
) {
    let nu = ((u.length() / CRUMB).ceil() as usize).max(1);
    let nv = ((v.length() / CRUMB).ceil() as usize).max(1);
    let (du, dv) = (u / nu as f32, v / nv as f32);
    let n = if flip { dv.cross(du) } else { du.cross(dv) }.normalize_or(Vec3::Z);
    let base = m.mesh.vertex_count() as u32;
    for k in 0..=nv {
        for i in 0..=nu {
            let p = corner + du * i as f32 + dv * k as f32;
            let depth = (-p.z / h).clamp(0.0, 1.0);
            let c = speckle(soil_color(depth), jitter(face, i, k));
            m.mesh
                .push_vertex(p, n, Vec2::new(i as f32 / nu as f32, k as f32 / nv as f32));
            m.colors.push([c[0] * tone, c[1] * tone, c[2] * tone, 1.0]);
        }
    }
    let row = nu as u32 + 1;
    for k in 0..nv as u32 {
        for i in 0..nu as u32 {
            let a = base + k * row + i;
            let (b, c, d) = (a + 1, a + row + 1, a + row);
            if flip {
                m.mesh.push_triangle(a, c, b);
                m.mesh.push_triangle(a, d, c);
            } else {
                m.mesh.push_triangle(a, b, c);
                m.mesh.push_triangle(a, c, d);
            }
        }
    }
}

/// The soil seen from inside: back, side and bottom faces facing in, plus the surface.
#[must_use]
pub fn soil(size: PotSize) -> ColoredMesh {
    if size.shape == PotShape::Round {
        return round::soil(size);
    }
    let (w, d, h) = (size.width, size.depth, size.soil_depth);
    let e = 0.000_5; // keep the soil just inside the walls
    let mut m = ColoredMesh::default();
    let up = Vec3::Z * h;
    // Back wall, facing south.
    crumb_face(
        &mut m,
        0,
        Vec3::new(-w / 2.0, d / 2.0 - e, -h),
        Vec3::X * w,
        up,
        h,
        false,
        1.0,
    );
    // West wall, facing east.
    crumb_face(
        &mut m,
        1,
        Vec3::new(-w / 2.0 + e, -d / 2.0, -h),
        Vec3::Y * d,
        up,
        h,
        false,
        1.0,
    );
    // East wall, facing west.
    crumb_face(
        &mut m,
        2,
        Vec3::new(w / 2.0 - e, -d / 2.0, -h),
        Vec3::Y * d,
        up,
        h,
        true,
        1.0,
    );
    // The floor, seen from above through the glass: deep subsoil, in shade.
    crumb_face(
        &mut m,
        4,
        Vec3::new(-w / 2.0, -d / 2.0, -h + e),
        Vec3::X * w,
        Vec3::Y * d,
        h,
        false,
        0.55,
    );
    // The surface: drier and lighter than the topsoil below, with crumbs.
    let top = Vec3::new(-w / 2.0, -d / 2.0, 0.0);
    let start = m.mesh.vertex_count();
    crumb_face(&mut m, 3, top, Vec3::X * w, Vec3::Y * d, 1.0, false, 1.0);
    for c in &mut m.colors[start..] {
        let j = c[0] / soil_color(0.0)[0];
        *c = [0.16 * j, 0.1 * j, 0.06 * j, 1.0];
    }
    m
}

/// Soil haze just behind the glass: a translucent cut face, so the slab reads as full
/// of soil and roots deeper in it look buried rather than floating in an empty crate.
/// The lens system hides it in X-ray.
#[must_use]
pub fn soil_haze(size: PotSize, alpha: f32) -> ColoredMesh {
    let (w, d, h) = (size.width, size.depth, size.soil_depth);
    let mut m = if size.shape == PotShape::Round {
        round::haze(size)
    } else {
        ColoredMesh::default()
    };
    if size.shape == PotShape::Box {
        crumb_face(
            &mut m,
            5,
            Vec3::new(-w / 2.0, -d / 2.0 + 0.004, -h),
            Vec3::X * w,
            Vec3::Z * h,
            h,
            false,
            0.6,
        );
    }
    for c in &mut m.colors {
        c[3] = alpha;
    }
    m
}

/// Wooden walls and the frame around the glass.
#[must_use]
pub fn frame(size: PotSize) -> ColoredMesh {
    if size.shape == PotShape::Round {
        return round::frame(size);
    }
    let (w, d, h) = (size.width, size.depth, size.soil_depth);
    let t = WALL;
    let wood = [0.42, 0.27, 0.15, 1.0];
    let dark = [0.3, 0.19, 0.1, 1.0];
    let lip = 0.01; // walls stand a little above the soil
    let mut m = ColoredMesh::default();
    m.cuboid(
        Vec3::new(-w / 2.0 - t, d / 2.0, -h - t),
        Vec3::new(w / 2.0 + t, d / 2.0 + t, lip),
        wood,
    );
    for x in [-w / 2.0 - t, w / 2.0] {
        m.cuboid(
            Vec3::new(x, -d / 2.0 - t, -h - t),
            Vec3::new(x + t, d / 2.0, lip),
            wood,
        );
    }
    m.cuboid(
        Vec3::new(-w / 2.0 - t, -d / 2.0 - t, -h - t),
        Vec3::new(w / 2.0 + t, d / 2.0, -h),
        dark,
    );
    // Feet.
    for x in [-w / 2.0, w / 2.0 - 0.03] {
        m.cuboid(
            Vec3::new(x, -d / 2.0, -h - t - 0.03),
            Vec3::new(x + 0.03, d / 2.0, -h - t),
            dark,
        );
    }
    m
}

/// The glass pane on the south face.
#[must_use]
pub fn glass(size: PotSize) -> MeshData {
    if size.shape == PotShape::Round {
        return round::glass(size).mesh;
    }
    let (w, d, h) = (size.width, size.depth, size.soil_depth);
    let mut m = ColoredMesh::default();
    m.quad(
        Vec3::new(-w / 2.0, -d / 2.0 - 0.001, -h),
        Vec3::X * w,
        Vec3::Z * (h + 0.005),
        [1.0; 4],
    );
    m.mesh
}

/// The soil water overlay on the glass: one quad per cell of `face`, colored by `color`.
/// On a round pot the cells are wrapped onto the curved glass.
#[must_use]
pub fn soil_face_overlay(
    size: PotSize,
    face: &SoilFace,
    color: impl Fn(f32) -> Option<[f32; 3]>,
) -> ColoredMesh {
    let (w, d, h) = (size.width, size.depth, size.soil_depth);
    let mut m = ColoredMesh::default();
    if face.columns == 0 || face.rows == 0 {
        return m;
    }
    if size.shape == PotShape::Round {
        return round::overlay(size, face, color);
    }
    let cw = w / face.columns as f32;
    let ch = h / face.rows as f32;
    for r in 0..face.rows {
        for c in 0..face.columns {
            let Some(rgb) = face.get(r, c).and_then(&color) else {
                continue;
            };
            let corner = Vec3::new(
                -w / 2.0 + cw * c as f32,
                -d / 2.0 - 0.002,
                -ch * (r + 1) as f32,
            );
            m.quad(
                corner,
                Vec3::X * cw,
                Vec3::Z * ch,
                [rgb[0], rgb[1], rgb[2], 0.6],
            );
        }
    }
    m
}

/// Round pots: a terracotta back half lined with soil and a curved glass front half.
mod round {
    use std::f32::consts::{PI, TAU};

    use bevy::math::Vec3;

    use super::{CRUMB, ColoredMesh, PotSize, SoilFace, WALL, jitter, soil_color, speckle};

    const TERRACOTTA: [f32; 4] = [0.34, 0.13, 0.065, 1.0];
    const TERRACOTTA_DARK: [f32; 4] = [0.22, 0.085, 0.045, 1.0];
    const SAUCER: [f32; 4] = [0.26, 0.1, 0.05, 1.0];
    /// The rim stands a little above the soil, as on the box.
    const LIP: f32 = 0.01;

    fn radius(size: PotSize) -> f32 {
        size.width.min(size.depth) / 2.0
    }

    /// Segments for an arc of `angle` at radius `r`.
    fn segments(r: f32, angle: f32) -> usize {
        ((r * angle / CRUMB).ceil() as usize).clamp(8, 512)
    }

    fn radial(p: Vec3) -> Vec3 {
        Vec3::new(p.x, p.y, 0.0)
    }

    fn crumb(face: usize, h: f32, tone: f32) -> impl Fn(usize, usize, Vec3) -> [f32; 4] {
        move |i, k, p| {
            let c = speckle(soil_color((-p.z / h).clamp(0.0, 1.0)), jitter(face, i, k));
            [c[0] * tone, c[1] * tone, c[2] * tone, 1.0]
        }
    }

    /// A vertical band of the cylinder of radius `r` between angles `a0..a1`, facing
    /// outward or inward.
    #[allow(clippy::too_many_arguments)]
    fn band(
        m: &mut ColoredMesh,
        r: f32,
        (a0, a1): (f32, f32),
        (z0, z1): (f32, f32),
        outward: bool,
        rows: usize,
        color: impl Fn(usize, usize, Vec3) -> [f32; 4],
    ) {
        let s = if outward { 1.0 } else { -1.0 };
        m.grid(
            (segments(r, a1 - a0), rows),
            |u, v| {
                let a = a0 + (a1 - a0) * u;
                Vec3::new(r * a.cos(), r * a.sin(), z0 + (z1 - z0) * v)
            },
            |p| radial(p) * s,
            color,
        );
    }

    /// A flat ring (or a disk when `r0` is 0) at height `z` between angles `a0..a1`.
    fn ring(
        m: &mut ColoredMesh,
        (r0, r1): (f32, f32),
        (a0, a1): (f32, f32),
        z: f32,
        up: bool,
        color: impl Fn(usize, usize, Vec3) -> [f32; 4],
    ) {
        let rings = (((r1 - r0) / CRUMB).ceil() as usize).max(1);
        let n = if up { Vec3::Z } else { -Vec3::Z };
        m.grid(
            (segments(r1, a1 - a0), rings),
            |u, v| {
                let a = a0 + (a1 - a0) * u;
                let r = r0 + (r1 - r0) * v;
                Vec3::new(r * a.cos(), r * a.sin(), z)
            },
            |_| n,
            color,
        );
    }

    fn flat(c: [f32; 4]) -> impl Fn(usize, usize, Vec3) -> [f32; 4] {
        move |_, _, _| c
    }

    /// The soil lining the back half, the floor and the surface.
    pub(super) fn soil(size: PotSize) -> ColoredMesh {
        let (r, h) = (radius(size), size.soil_depth);
        let e = 0.000_5;
        let rows = ((h / CRUMB).ceil() as usize).max(1);
        let mut m = ColoredMesh::default();
        band(
            &mut m,
            r - e,
            (0.0, PI),
            (-h, 0.0),
            false,
            rows,
            crumb(0, h, 1.0),
        );
        ring(
            &mut m,
            (0.0, r),
            (0.0, TAU),
            -h + e,
            true,
            crumb(4, h, 0.55),
        );
        let start = m.mesh.vertex_count();
        ring(&mut m, (0.0, r), (0.0, TAU), 0.0, true, crumb(3, 1.0, 1.0));
        for c in &mut m.colors[start..] {
            let j = c[0] / soil_color(0.0)[0];
            *c = [0.16 * j, 0.1 * j, 0.06 * j, 1.0];
        }
        m
    }

    /// Soil haze just inside the curved glass.
    pub(super) fn haze(size: PotSize) -> ColoredMesh {
        let (r, h) = (radius(size), size.soil_depth);
        let rows = ((h / CRUMB).ceil() as usize).max(1);
        let mut m = ColoredMesh::default();
        band(
            &mut m,
            r - 0.004,
            (PI, TAU),
            (-h, 0.0),
            true,
            rows,
            crumb(5, h, 0.6),
        );
        m
    }

    /// The terracotta back half with its rim, the base and a saucer.
    pub(super) fn frame(size: PotSize) -> ColoredMesh {
        let (r, h, t) = (radius(size), size.soil_depth, WALL);
        let mut m = ColoredMesh::default();
        let back = (0.0, PI);
        let all = (0.0, TAU);
        // Back wall: outside, the lip above the soil inside, and the rim on top.
        band(
            &mut m,
            r + t,
            back,
            (-h - t, LIP),
            true,
            4,
            flat(TERRACOTTA),
        );
        band(&mut m, r, back, (0.0, LIP), false, 1, flat(TERRACOTTA));
        ring(&mut m, (r, r + t), back, LIP, true, flat(TERRACOTTA));
        // The cut ends of the wall, facing south.
        for (a, east) in [(0.0_f32, true), (PI, false)] {
            let (c, s) = (a.cos(), a.sin());
            let (inner, outer) = (
                Vec3::new(r * c, r * s, 0.0),
                Vec3::new((r + t) * c, (r + t) * s, 0.0),
            );
            let (lo, hi) = (Vec3::Z * (-h - t), Vec3::Z * LIP);
            let (p, q) = if east { (inner, outer) } else { (outer, inner) };
            m.quad4([p + lo, q + lo, q + hi, p + hi], -Vec3::Y, TERRACOTTA_DARK);
        }
        // Base under the soil and glass: a full disk and its edge.
        ring(
            &mut m,
            (0.0, r + t),
            all,
            -h - t,
            false,
            flat(TERRACOTTA_DARK),
        );
        band(
            &mut m,
            r + t,
            (PI, TAU),
            (-h - t, -h),
            true,
            1,
            flat(TERRACOTTA_DARK),
        );
        // Saucer.
        let (sr, s0, s1) = (r + t + 0.014, -h - t - 0.03, -h - t - 0.012);
        band(&mut m, sr, all, (s0, s1), true, 1, flat(SAUCER));
        ring(&mut m, (0.0, sr), all, s1, true, flat(SAUCER));
        ring(&mut m, (0.0, sr), all, s0, false, flat(TERRACOTTA_DARK));
        m
    }

    /// The curved glass on the front half.
    pub(super) fn glass(size: PotSize) -> ColoredMesh {
        let (r, h) = (radius(size), size.soil_depth);
        let mut m = ColoredMesh::default();
        band(
            &mut m,
            r + 0.001,
            (PI, TAU),
            (-h, 0.005),
            true,
            1,
            flat([1.0; 4]),
        );
        m
    }

    /// The soil water slice wrapped onto the glass: each column of cells sits where its x
    /// meets the front of the pot.
    pub(super) fn overlay(
        size: PotSize,
        face: &SoilFace,
        color: impl Fn(f32) -> Option<[f32; 3]>,
    ) -> ColoredMesh {
        let (r, h) = (radius(size), size.soil_depth);
        let rg = r + 0.002;
        let mut m = ColoredMesh::default();
        let cw = 2.0 * r / face.columns as f32;
        let ch = h / face.rows as f32;
        let front = |x: f32| -> Vec3 {
            let x = x.clamp(-rg, rg);
            Vec3::new(x, -(rg * rg - x * x).max(0.0).sqrt(), 0.0)
        };
        for c in 0..face.columns {
            let (x0, x1) = (-r + cw * c as f32, -r + cw * (c + 1) as f32);
            let (p0, p1) = (front(x0), front(x1));
            let n = radial(front(f32::midpoint(x0, x1))).normalize_or(-Vec3::Y);
            for row in 0..face.rows {
                let Some(rgb) = face.get(row, c).and_then(&color) else {
                    continue;
                };
                let (z0, z1) = (-ch * (row + 1) as f32, -ch * row as f32);
                let (lo, hi) = (Vec3::Z * z0, Vec3::Z * z1);
                m.quad4(
                    [p0 + lo, p1 + lo, p1 + hi, p0 + hi],
                    n,
                    [rgb[0], rgb[1], rgb[2], 0.6],
                );
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normals_face(m: &ColoredMesh) -> bool {
        m.mesh.indices.chunks_exact(3).all(|t| {
            let p = [t[0], t[1], t[2]].map(|i| Vec3::from(m.mesh.positions[i as usize]));
            let n = Vec3::from(m.mesh.normals[t[0] as usize]);
            (p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.0
        })
    }

    #[test]
    fn soil_faces_inward_and_glass_faces_south() {
        let size = PotSize::RHIZOBOX;
        let s = soil(size);
        assert!(normals_face(&s));
        assert_eq!(s.colors.len(), s.mesh.vertex_count());
        // No soil face points north: there is no soil wall in front, only glass.
        assert!(s.mesh.normals.iter().all(|n| n[1] < 0.5));
        let g = glass(size);
        assert!(g.normals.iter().all(|n| (n[1] + 1.0).abs() < 1e-6));
        let f = frame(size);
        assert!(normals_face(&f) && f.mesh.is_well_formed());
    }

    const ROUND: PotSize = PotSize {
        shape: PotShape::Round,
        width: 0.2,
        depth: 0.2,
        soil_depth: 0.18,
    };

    #[test]
    fn round_pot_faces_the_right_way_and_stays_in_its_circle() {
        let s = soil(ROUND);
        assert!(normals_face(&s) && s.mesh.is_well_formed());
        assert_eq!(s.colors.len(), s.mesh.vertex_count());
        assert!(s.mesh.normals.iter().all(|n| n[1] < 0.5));
        let r = ROUND.width / 2.0;
        assert!(
            s.mesh
                .positions
                .iter()
                .all(|p| p[0].hypot(p[1]) <= r + 1e-5 && p[2] >= -ROUND.soil_depth - 1e-5)
        );
        let g = glass(ROUND);
        assert!(g.is_well_formed());
        assert!(g.normals.iter().all(|n| n[1] <= 1e-6));
        assert!(g.positions.iter().all(|p| p[1] <= 1e-6));
        let f = frame(ROUND);
        assert!(normals_face(&f) && f.mesh.is_well_formed());
        // The terracotta wall stays behind the glass, apart from the base and saucer.
        let h = soil_haze(ROUND, 0.4);
        assert!(normals_face(&h) && h.colors.iter().all(|c| (c[3] - 0.4).abs() < 1e-6));
    }

    #[test]
    fn round_overlay_wraps_onto_the_glass() {
        let face = SoilFace {
            columns: 5,
            rows: 2,
            values: vec![0.5; 10],
        };
        let o = soil_face_overlay(ROUND, &face, |v| Some([v; 3]));
        assert_eq!(o.mesh.triangle_count(), 10 * 2);
        assert!(normals_face(&o));
        let r = ROUND.width / 2.0 + 0.002;
        assert!(
            o.mesh
                .positions
                .iter()
                .all(|p| (p[0].hypot(p[1]) - r).abs() < 1e-4 && p[1] <= 1e-6)
        );
    }

    #[test]
    fn overlay_has_one_quad_per_valued_cell() {
        let face = SoilFace {
            columns: 4,
            rows: 3,
            values: vec![
                0.5,
                0.5,
                f32::NAN,
                0.5,
                0.5,
                0.5,
                0.5,
                0.5,
                0.5,
                0.5,
                0.5,
                0.5,
            ],
        };
        let o = soil_face_overlay(PotSize::RHIZOBOX, &face, |v| {
            v.is_finite().then_some([v; 3])
        });
        assert_eq!(o.mesh.triangle_count(), 11 * 2);
        let (lo, hi) = o.mesh.bounds().unwrap();
        assert!(lo.z >= -PotSize::RHIZOBOX.soil_depth - 1e-6 && hi.z <= 1e-6);
    }
}
