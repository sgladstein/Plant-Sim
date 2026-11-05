//! Set dressing: the garden around the experiment (visual design review, 2026-11-05).
//!
//! A slatted potting bench under the pots, a lawn of grass blades around it, pebbles at
//! the bench's feet and a ring of hedges and trees in the distance, softened by the fog.
//! None of it is simulated; it gives scale and place. Everything is built in Bevy's frame
//! (y up) relative to the bench center, as vertex-colored meshes, deterministic from a
//! fixed hash so screenshots are stable. About 60k triangles in all, built once per pot
//! layout.

use bevy::math::{Vec2, Vec3};

use crate::diorama::ColoredMesh;

/// Bench top height above the ground, m.
pub const BENCH_HEIGHT: f32 = 0.55;

/// A deterministic value in [0, 1) from two integers.
fn hash01(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn rgba(c: [f32; 3]) -> [f32; 4] {
    [c[0], c[1], c[2], 1.0]
}

/// A slatted bench, its top at y = 0, `size` (x, z) across, legs down to the ground.
#[must_use]
pub fn bench(size: Vec2) -> ColoredMesh {
    let mut m = ColoredMesh::default();
    let wood = [0.19, 0.14, 0.095];
    let slat_w = 0.09;
    let gap = 0.012;
    let n = ((size.y + gap) / (slat_w + gap)).ceil().max(1.0) as usize;
    let pitch = size.y / n as f32;
    let t = 0.028;
    for k in 0..n {
        let z0 = -size.y / 2.0 + pitch * k as f32;
        let tone = 0.9 + 0.2 * hash01(k as u32, 7);
        let c = rgba(wood.map(|v| v * tone));
        m.cuboid(
            Vec3::new(-size.x / 2.0, -t, z0 + gap / 2.0),
            Vec3::new(size.x / 2.0, 0.0, z0 + pitch - gap / 2.0),
            c,
        );
    }
    // Two rails under the slats and four legs.
    let dark = rgba(wood.map(|v| v * 0.7));
    let leg = 0.05;
    let inset = 0.06;
    for z in [-size.y / 2.0 + inset, size.y / 2.0 - inset - leg] {
        m.cuboid(
            Vec3::new(-size.x / 2.0 + inset, -t - 0.07, z),
            Vec3::new(size.x / 2.0 - inset, -t, z + leg),
            dark,
        );
        for x in [-size.x / 2.0 + inset, size.x / 2.0 - inset - leg] {
            m.cuboid(
                Vec3::new(x, -BENCH_HEIGHT, z),
                Vec3::new(x + leg, -t, z + leg),
                dark,
            );
        }
    }
    // A low stretcher between the legs.
    m.cuboid(
        Vec3::new(-size.x / 2.0 + inset, -BENCH_HEIGHT + 0.12, -0.015),
        Vec3::new(size.x / 2.0 - inset, -BENCH_HEIGHT + 0.15, 0.015),
        dark,
    );
    m
}

/// One grass blade: a tapered, bent strip of three segments.
fn blade(
    m: &mut ColoredMesh,
    base: Vec3,
    height: f32,
    width: f32,
    yaw: f32,
    lean: f32,
    c: [f32; 3],
) {
    let dir = Vec3::new(yaw.cos(), 0.0, yaw.sin());
    let side = Vec3::new(-yaw.sin(), 0.0, yaw.cos()) * width;
    let segs = 3;
    let first = m.mesh.vertex_count() as u32;
    for s in 0..=segs {
        let f = s as f32 / segs as f32;
        let p = base + Vec3::Y * height * f + dir * lean * height * f * f;
        let half = 0.5 * (1.0 - f).max(0.08);
        let n = (Vec3::Y * 0.85 + dir * 0.15).normalize_or(Vec3::Y);
        // Darker at the base, where the lawn shades itself.
        let k = 0.75 + 0.35 * f;
        let col = [c[0] * k, c[1] * k, c[2] * k, 1.0];
        m.mesh.push_vertex(p - side * half, n, Vec2::new(0.0, f));
        m.mesh.push_vertex(p + side * half, n, Vec2::new(1.0, f));
        m.colors.push(col);
        m.colors.push(col);
    }
    for s in 0..segs as u32 {
        let a = first + 2 * s;
        // The dressing material is double-sided, so one winding serves both faces.
        m.mesh.push_triangle(a, a + 1, a + 3);
        m.mesh.push_triangle(a, a + 3, a + 2);
    }
}

/// A lawn of grass blades on the ground (y = 0) around a bench footprint `keep_out`
/// (half sizes x, z), out to `radius`, thinning with distance.
#[must_use]
pub fn lawn(keep_out: Vec2, radius: f32) -> ColoredMesh {
    let mut m = ColoredMesh::default();
    let fresh = [0.16, 0.30, 0.06];
    let deep = [0.08, 0.17, 0.035];
    let straw = [0.34, 0.32, 0.14];
    // About 340 blades per m² of disc, capped so very wide plots stay cheap.
    let count = (340.0 * core::f32::consts::PI * radius * radius).min(60_000.0) as u32;
    for i in 0..count {
        // Area-uniform in a disc, biased toward the bench so the near lawn is thick.
        let r = radius * hash01(i, 1).powf(0.8);
        let a = core::f32::consts::TAU * hash01(i, 2);
        let p = Vec2::new(r * a.cos(), r * a.sin());
        if p.x.abs() < keep_out.x * 0.7 && p.y.abs() < keep_out.y * 0.7 {
            continue;
        }
        let tone = hash01(i, 3);
        let mut c = mix3(deep, fresh, tone);
        if hash01(i, 4) < 0.08 {
            c = mix3(c, straw, 0.7);
        }
        let h = 0.03 + 0.05 * hash01(i, 5);
        blade(
            &mut m,
            Vec3::new(p.x, 0.0, p.y),
            h,
            0.006 + 0.004 * hash01(i, 6),
            core::f32::consts::TAU * hash01(i, 7),
            0.25 + 0.5 * hash01(i, 8),
            c,
        );
    }
    m
}

/// A squashed low-poly pebble.
fn pebble(m: &mut ColoredMesh, center: Vec3, size: Vec3, seed: u32, c: [f32; 3]) {
    let rings = 4;
    let segs = 7;
    let first = m.mesh.vertex_count() as u32;
    for i in 0..=rings {
        let phi = core::f32::consts::PI * i as f32 / rings as f32;
        for j in 0..=segs {
            let th = core::f32::consts::TAU * j as f32 / segs as f32;
            let bump = 1.0 + 0.15 * (hash01(seed, i * 31 + (j % segs)) - 0.5);
            let d = Vec3::new(phi.sin() * th.cos(), phi.cos(), phi.sin() * th.sin());
            let n = (d / size).normalize_or(Vec3::Y);
            m.mesh.push_vertex(center + d * size * bump, n, Vec2::ZERO);
            m.colors.push(rgba(c));
        }
    }
    let row = segs + 1;
    for i in 0..rings {
        for j in 0..segs {
            let a = first + i * row + j;
            let b = a + row;
            m.mesh.push_triangle(a, a + 1, b);
            m.mesh.push_triangle(b, a + 1, b + 1);
        }
    }
}

/// Pebbles scattered around the bench's feet.
#[must_use]
pub fn pebbles(around: Vec2) -> ColoredMesh {
    let mut m = ColoredMesh::default();
    for i in 0..60_u32 {
        let a = core::f32::consts::TAU * hash01(i, 11);
        let r = 1.0 + 0.6 * hash01(i, 12);
        let p = Vec2::new(a.cos() * around.x * r, a.sin() * around.y * r);
        let s = 0.012 + 0.03 * hash01(i, 13).powi(2);
        let g = 0.2 + 0.16 * hash01(i, 14);
        let c = [g, g * 0.97, g * 0.9];
        pebble(
            &mut m,
            Vec3::new(p.x, s * 0.25, p.y),
            Vec3::new(s * (1.0 + 0.5 * hash01(i, 15)), s * 0.55, s),
            i,
            c,
        );
    }
    m
}

/// A ring of hedges and round-crowned trees from `ring` to 1.7 × `ring` m, for depth. Fog turns them
/// into soft silhouettes.
#[must_use]
pub fn treeline(ring: f32) -> ColoredMesh {
    let mut m = ColoredMesh::default();
    let foliage = [[0.05, 0.13, 0.04], [0.07, 0.16, 0.05], [0.10, 0.17, 0.06]];
    let trunk = [0.12, 0.08, 0.05];
    // Skip the arc in front of the default camera (south, +z in Bevy) so it never blocks.
    for i in 0..46_u32 {
        let a = core::f32::consts::TAU * (i as f32 + 0.5 * hash01(i, 20)) / 46.0;
        if a.sin() > 0.55 {
            continue;
        }
        let r = ring * (1.0 + 0.7 * hash01(i, 21));
        let base = Vec3::new(r * a.cos(), 0.0, r * a.sin());
        let c = foliage[(hash01(i, 22) * 3.0) as usize % 3];
        if hash01(i, 23) < 0.45 {
            // A tree: trunk and two or three overlapping crown blobs.
            let h = 1.6 + 1.6 * hash01(i, 24);
            pebble(
                &mut m,
                // Sunk a little into the ground, so the trunk meets the lawn at full
                // width instead of tapering to a point above it.
                base + Vec3::Y * h * 0.3,
                Vec3::new(0.14, h * 0.42, 0.14),
                i + 1000,
                trunk,
            );
            for k in 0..3 {
                let off = Vec3::new(
                    (hash01(i, 30 + k) - 0.5) * 1.2,
                    h * (0.7 + 0.15 * k as f32),
                    (hash01(i, 40 + k) - 0.5) * 1.2,
                );
                let s = h * (0.32 + 0.12 * hash01(i, 50 + k));
                pebble(&mut m, base + off, Vec3::new(s, s * 0.85, s), i * 7 + k, c);
            }
        } else {
            // A hedge: a long low rounded mound.
            let len = 1.5 + 2.5 * hash01(i, 25);
            for k in 0..4 {
                let along = Vec3::new(-a.sin(), 0.0, a.cos()) * (k as f32 - 1.5) * len * 0.3;
                pebble(
                    &mut m,
                    base + along + Vec3::Y * 0.5,
                    Vec3::new(len * 0.3, 0.75, 0.8),
                    i * 13 + k,
                    c,
                );
            }
        }
    }
    m
}
