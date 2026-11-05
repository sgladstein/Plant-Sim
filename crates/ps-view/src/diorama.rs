//! Pot geometry: a rhizobox with a glass front (design plan §2.1, §4).
//!
//! In the pot's sim frame (x east, y north, z up, soil surface at z = 0) the soil fills
//! `x ∈ [−W/2, W/2]`, `y ∈ [−D/2, D/2]`, `z ∈ [−H, 0]`. The glass is the south face
//! (`y = −D/2`), so from the default camera the roots against it and inside the slab are
//! visible; the other faces are soil seen from inside, banded by depth, inside wooden
//! walls. Geometry is built as [`MeshData`] with vertex colors, so it is testable without
//! a GPU.

use bevy::math::{Vec2, Vec3};
use ps_mesh::MeshData;

use crate::garden::{PotSize, SoilFace};

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

const BANDS: usize = 8;

/// The soil seen from inside: back, side and bottom faces facing in, plus the surface.
#[must_use]
pub fn soil(size: PotSize) -> ColoredMesh {
    let (w, d, h) = (size.width, size.depth, size.soil_depth);
    let e = 0.000_5; // keep the soil just inside the walls
    let mut m = ColoredMesh::default();
    let band = h / BANDS as f32;
    for k in 0..BANDS {
        let z0 = -h + band * k as f32;
        let frac = 1.0 - (k as f32 + 0.5) / BANDS as f32;
        let c = soil_color(frac);
        let up = Vec3::Z * band;
        // Back wall, facing south.
        m.quad(Vec3::new(-w / 2.0, d / 2.0 - e, z0), Vec3::X * w, up, c);
        // West wall, facing east.
        m.quad(Vec3::new(-w / 2.0 + e, -d / 2.0, z0), Vec3::Y * d, up, c);
        // East wall, facing west.
        m.quad(Vec3::new(w / 2.0 - e, -d / 2.0, z0), up, Vec3::Y * d, c);
    }
    m.quad(
        Vec3::new(-w / 2.0, -d / 2.0, -h + e),
        Vec3::X * w,
        Vec3::Y * d,
        soil_color(1.0),
    );
    // The surface, slightly lighter and drier-looking.
    m.quad(
        Vec3::new(-w / 2.0, -d / 2.0, 0.0),
        Vec3::X * w,
        Vec3::Y * d,
        [0.16, 0.1, 0.06, 1.0],
    );
    m
}

/// Wooden walls and the frame around the glass.
#[must_use]
pub fn frame(size: PotSize) -> ColoredMesh {
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
