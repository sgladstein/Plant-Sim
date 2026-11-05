//! A plain indexed triangle mesh that any renderer can upload.

use glam::{Affine3A, Vec2, Vec3};

/// Indexed triangle list with per-vertex normals and texture coordinates.
///
/// Triangles wind counter-clockwise seen from the side the normals point to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshData {
    /// Vertex positions, m.
    pub positions: Vec<[f32; 3]>,
    /// Unit vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Texture coordinates.
    pub uvs: Vec<[f32; 2]>,
    /// Three indices per triangle.
    pub indices: Vec<u32>,
}

impl MeshData {
    /// An empty mesh.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of vertices.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Number of triangles.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// True when the mesh has no triangles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Adds a vertex and returns its index.
    pub fn push_vertex(&mut self, position: Vec3, normal: Vec3, uv: Vec2) -> u32 {
        let i = self.positions.len() as u32;
        self.positions.push(position.to_array());
        self.normals.push(normal.to_array());
        self.uvs.push(uv.to_array());
        i
    }

    /// Adds a triangle by vertex indices.
    pub fn push_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend_from_slice(&[a, b, c]);
    }

    /// Appends another mesh unchanged.
    pub fn append(&mut self, other: &Self) {
        self.append_transformed(other, Affine3A::IDENTITY);
    }

    /// Appends another mesh moved by `transform`; normals use its inverse transpose.
    pub fn append_transformed(&mut self, other: &Self, transform: Affine3A) {
        let base = self.positions.len() as u32;
        let normal_matrix = transform.matrix3.inverse().transpose();
        for (p, n) in other.positions.iter().zip(&other.normals) {
            self.positions
                .push(transform.transform_point3(Vec3::from(*p)).to_array());
            let n = Vec3::from(normal_matrix * glam::Vec3A::from(Vec3::from(*n)));
            self.normals.push(n.normalize_or(Vec3::Z).to_array());
        }
        self.uvs.extend_from_slice(&other.uvs);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }

    /// True when every position, normal and uv is finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.positions.iter().flatten().all(|v| v.is_finite())
            && self.normals.iter().flatten().all(|v| v.is_finite())
            && self.uvs.iter().flatten().all(|v| v.is_finite())
    }

    /// True when the attribute arrays agree in length, indices are in range and the
    /// index count is a multiple of three.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        let n = self.positions.len();
        self.normals.len() == n
            && self.uvs.len() == n
            && self.indices.len().is_multiple_of(3)
            && self.indices.iter().all(|&i| (i as usize) < n)
    }

    /// Axis-aligned bounds `(min, max)`, or `None` for an empty mesh.
    #[must_use]
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut it = self.positions.iter().map(|p| Vec3::from(*p));
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }

    /// One-sided area of all triangles.
    #[must_use]
    pub fn area(&self) -> f32 {
        self.indices
            .chunks_exact(3)
            .map(|t| {
                let [a, b, c] = [t[0], t[1], t[2]].map(|i| Vec3::from(self.positions[i as usize]));
                0.5 * (b - a).cross(c - a).length()
            })
            .sum()
    }

    /// Scales every position by `s` about the origin.
    pub fn scale(&mut self, s: f32) {
        for p in &mut self.positions {
            *p = (Vec3::from(*p) * s).to_array();
        }
    }
}
