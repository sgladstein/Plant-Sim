//! Shared organ templates, drawn instanced (design plan §4).
//!
//! Every template is built in a local frame with x forward (petiole, midrib, pod axis),
//! y to the left and z up, with the attachment point at the origin, and normalized so
//! that an instance's scale comes straight from the sim's sizes:
//!
//! | Template | Normalized so that | Instance scale |
//! | --- | --- | --- |
//! | Unifoliate, Trifoliate | flat blade area (all leaflets, one side) is 1 m² | `sqrt(area)` |
//! | Cotyledon | projected area is 1 m² | `sqrt(area)` |
//! | Pod | length 1 along x, peak radius 1 | `(length, radius, radius)` |
//! | Seed | semi-axes 0.5, 1, 1, centered | `(length, radius, radius)` |
//! | Flower | length about 1 | `length` |
//! | Bud | unit sphere | `radius` |
//! | `GrassBlade` | arc length 1 along x, base width 1 | `(length, width, length)` |
//! | `WheatEar` | length 1 along x, peak radius 1 | `(length, radius, radius)` |
//!
//! Leaf blades are two-sided: the back is a copy of the front with flipped normals and
//! winding, so back-face culling shows the right side with the right lighting. Wilting
//! is a template parameter: [`DROOP_LEVELS`] prebuilt variants with the blade pitched
//! down and the midrib bent more, chosen per instance from the plant's water stress.

use core::f32::consts::{FRAC_PI_2, PI, TAU};

use glam::{Affine3A, Quat, Vec2, Vec3};

use crate::{MeshData, TubeNode, tube};

/// Number of prebuilt droop variants per leaf template (0 = turgid, last = fully wilted).
pub const DROOP_LEVELS: u8 = 5;

/// A shared organ shape.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Template {
    /// Seed leaf: thick and fleshy, no petiole.
    Cotyledon,
    /// Simple leaf (the first pair on a bean).
    Unifoliate,
    /// Three leaflets on a petiole and short rachis.
    Trifoliate,
    /// Papilionaceous flower, stylized.
    Flower,
    /// Pod with seed bumps and a beak.
    Pod,
    /// Seed (the sown seed and seeds in pods).
    Seed,
    /// Bud.
    Bud,
    /// A grass blade: a long strap arching over toward its tip.
    GrassBlade,
    /// A wheat ear: two rows of plump spikelets with short awns.
    WheatEar,
}

impl Template {
    /// All templates, in a fixed order.
    pub const ALL: [Self; 9] = [
        Self::Cotyledon,
        Self::Unifoliate,
        Self::Trifoliate,
        Self::Flower,
        Self::Pod,
        Self::Seed,
        Self::Bud,
        Self::GrassBlade,
        Self::WheatEar,
    ];

    /// True for templates that have droop variants.
    #[must_use]
    pub fn droops(self) -> bool {
        matches!(self, Self::Unifoliate | Self::Trifoliate | Self::GrassBlade)
    }
}

/// A template and its droop variant: the key of one shared mesh.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TemplateKey {
    /// The shape.
    pub template: Template,
    /// Droop level, `0..DROOP_LEVELS`; always 0 for templates that do not droop.
    pub droop: u8,
}

impl TemplateKey {
    /// Every key a renderer needs to prebuild, in a fixed order.
    #[must_use]
    pub fn all() -> Vec<Self> {
        Template::ALL
            .iter()
            .flat_map(|&template| {
                let levels = if template.droops() { DROOP_LEVELS } else { 1 };
                (0..levels).map(move |droop| Self { template, droop })
            })
            .collect()
    }
}

/// Builds the mesh of one template variant.
#[must_use]
pub fn build_template(key: TemplateKey) -> MeshData {
    let droop = if key.template.droops() {
        f32::from(key.droop.min(DROOP_LEVELS - 1)) / f32::from(DROOP_LEVELS - 1)
    } else {
        0.0
    };
    match key.template {
        Template::Unifoliate => leaf(LeafForm::Unifoliate, droop),
        Template::Trifoliate => leaf(LeafForm::Trifoliate, droop),
        Template::Cotyledon => cotyledon(),
        Template::Flower => flower(),
        Template::Pod => pod(),
        Template::Seed => {
            let mut m = MeshData::new();
            ellipsoid(&mut m, Vec3::ZERO, Vec3::new(0.5, 1.0, 1.0), 10, 12);
            m
        }
        Template::Bud => {
            let mut m = MeshData::new();
            ellipsoid(&mut m, Vec3::ZERO, Vec3::ONE, 6, 8);
            m
        }
        Template::GrassBlade => grass_blade(droop),
        Template::WheatEar => wheat_ear(),
    }
}

/// How far a turgid grass blade's centerline turns down from base to tip, rad (the sim's
/// `grass::BLADE_DROOP`).
pub const BLADE_ARCH: f32 = 0.6;

/// Blade area over length × base width for [`grass_blade`]'s `1 − u³` outline (the sim's
/// `grass::BLADE_SHAPE`).
pub const BLADE_SHAPE: f32 = 0.75;

/// A two-sided grass blade of arc length 1 and base width 1 along x, its centerline turning
/// down by [`BLADE_ARCH`] (more as it wilts), folded slightly along the midrib. Scale it
/// by `(length, width, length)`.
fn grass_blade(droop: f32) -> MeshData {
    const NU: u32 = 16;
    const NV: u32 = 4;
    let arch = BLADE_ARCH + 1.2 * droop;
    // Centerline points and directions by arc length.
    let mut center = Vec::with_capacity(NU as usize + 1);
    let mut p = Vec3::ZERO;
    for i in 0..=NU {
        let u = i as f32 / NU as f32;
        let angle = -arch * u;
        center.push((p, Vec3::new(angle.cos(), 0.0, angle.sin())));
        let mid = -arch * (u + 0.5 / NU as f32);
        p += Vec3::new(mid.cos(), 0.0, mid.sin()) / NU as f32;
    }
    let mut mesh = MeshData::new();
    let points: Vec<Vec3> = (0..=NU)
        .flat_map(|i| {
            let (c, d) = center[i as usize];
            let u = i as f32 / NU as f32;
            let half = 0.5 * (1.0 - u * u * u);
            let up = Vec3::Y.cross(d) * -1.0;
            (0..=NV).map(move |j| {
                let v = 2.0 * j as f32 / NV as f32 - 1.0;
                // The margins stand a little above the midrib.
                c + Vec3::Y * (v * half) + up * (0.12 * half * v.abs())
            })
        })
        .collect();
    let at = |i: u32, j: u32| points[(i * (NV + 1) + j) as usize];
    let front = mesh.positions.len() as u32;
    for i in 0..=NU {
        for j in 0..=NV {
            let du = at((i + 1).min(NU), j) - at(i.saturating_sub(1), j);
            let dv = at(i, (j + 1).min(NV)) - at(i, j.saturating_sub(1));
            let n = du.cross(dv).normalize_or(Vec3::Z);
            let uv = Vec2::new(j as f32 / NV as f32, i as f32 / NU as f32);
            mesh.push_vertex(at(i, j), n, uv);
        }
    }
    let back = mesh.positions.len() as u32;
    for k in front as usize..back as usize {
        let (q, n, uv) = (mesh.positions[k], mesh.normals[k], mesh.uvs[k]);
        mesh.push_vertex(Vec3::from(q), -Vec3::from(n), Vec2::from(uv));
    }
    for i in 0..NU {
        for j in 0..NV {
            let idx = |ii: u32, jj: u32| ii * (NV + 1) + jj;
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i, j + 1), idx(i + 1, j + 1));
            mesh.push_triangle(front + a, front + b, front + c);
            mesh.push_triangle(front + b, front + d, front + c);
            mesh.push_triangle(back + a, back + c, back + b);
            mesh.push_triangle(back + b, back + c, back + d);
        }
    }
    mesh
}

/// A wheat ear of length 1 along x and peak radius 1: spikelets alternating on two sides
/// of the rachis, tapering at both ends, with short awns from the upper spikelets.
fn wheat_ear() -> MeshData {
    const SPIKELETS: u32 = 16;
    let mut m = MeshData::new();
    // The rachis.
    let rachis = [
        TubeNode {
            position: Vec3::ZERO,
            radius: 0.25,
        },
        TubeNode {
            position: Vec3::X,
            radius: 0.12,
        },
    ];
    tube(&rachis, 6, true, Vec3::X, &mut m);
    for k in 0..SPIKELETS {
        let u = (k as f32 + 0.5) / SPIKELETS as f32;
        // Fullest a third of the way up, smaller at the base and tip.
        let size = (PI * u.powf(0.8)).sin().max(0.25);
        let side = if k % 2 == 0 { 1.0 } else { -1.0 };
        let c = Vec3::new(u * 0.95, side * 0.45 * size, 0.0);
        ellipsoid(&mut m, c, Vec3::new(0.09, 0.55, 0.75) * size, 5, 7);
        if u > 0.3 {
            let tip = c + Vec3::new(0.22, side * 0.35, 0.0) * (0.6 + u);
            let awn = [
                TubeNode {
                    position: c + Vec3::X * 0.05,
                    radius: 0.05,
                },
                TubeNode {
                    position: tip,
                    radius: 0.01,
                },
            ];
            tube(&awn, 3, false, Vec3::X, &mut m);
        }
    }
    m
}

#[derive(Copy, Clone, PartialEq)]
enum LeafForm {
    Unifoliate,
    Trifoliate,
}

/// Shape of one leaflet, before placement.
#[derive(Copy, Clone)]
struct Leaflet {
    length: f32,
    width: f32,
    /// Width profile `u^a (1 − u)^b`: `a` sets the base, `b` the tip.
    a: f32,
    b: f32,
    /// Midrib curvature (z drop at the tip as a fraction of length).
    bend: f32,
    /// Cupping across the blade (z rise at the margin as a fraction of half-width).
    cup: f32,
    /// Fold along the midrib: the halves rise in a shallow V (fraction of half-width).
    fold: f32,
    /// Margin waviness (z amplitude at the margin as a fraction of half-width).
    wave: f32,
    /// Asymmetry: the +y half is wider by this fraction, the −y half narrower
    /// (lateral bean leaflets are lopsided, the outer half wider).
    asym: f32,
    /// Narrowing of the last 30% into a drawn-out tip (acuminate), 0 to 1.
    acumen: f32,
}

/// Where a leaflet sits: its base point, yaw about z, pitch (positive = up).
#[derive(Copy, Clone)]
struct Pose {
    base: Vec3,
    yaw: f32,
    pitch: f32,
}

const LEAFLET_U: u32 = 18;
const LEAFLET_V: u32 = 6;

/// Brightness multiplier for a blade vertex at (u along, v across in −1..1): a pale
/// midrib, faint pinnate veins, slightly darker margins, a lighter base. Front side; the
/// underside is paler overall (abaxial surfaces of bean leaves are lighter).
fn blade_shade(u: f32, v: f32, underside: bool) -> f32 {
    let midrib = (1.0 - (v.abs() / 0.12)).max(0.0);
    // Secondary veins run from the midrib toward the tip at about 45°.
    let phase = (u * 6.0 - v.abs() * 1.1).rem_euclid(1.0);
    let vein =
        (1.0 - (phase - 0.5).abs() * 6.0).max(0.0) * (1.0 - v.abs()) * f32::from(u8::from(u < 0.9));
    let margin = v.abs().powi(3);
    let base = (1.0 - u * 4.0).max(0.0);
    let s = 1.0 + 0.32 * midrib + 0.08 * vein - 0.16 * margin + 0.08 * base;
    if underside { 1.18 * s } else { s }
}

/// Appends a two-sided leaflet and returns its one-sided area. `shade` gets one
/// brightness multiplier per vertex (padded with 1 for vertices added before).
fn leaflet(mesh: &mut MeshData, shade: &mut Vec<f32>, shape: Leaflet, pose: Pose) -> f32 {
    let peak_u = shape.a / (shape.a + shape.b);
    let peak = peak_u.powf(shape.a) * (1.0 - peak_u).powf(shape.b);
    let half_width = |u: f32, v: f32| {
        let tip = ((u - 0.7) / 0.3).max(0.0);
        let side = 1.0 + shape.asym * v.signum();
        0.5 * shape.width * u.powf(shape.a) * (1.0 - u).powf(shape.b) / peak
            * (1.0 - shape.acumen * 0.55 * tip * tip)
            * side
    };
    let rotation = Quat::from_rotation_z(pose.yaw) * Quat::from_rotation_y(-pose.pitch);
    let nu = LEAFLET_U;
    let nv = 2 * LEAFLET_V;
    let local = |i: u32, j: u32| {
        let u = i as f32 / nu as f32;
        let v = 2.0 * j as f32 / nv as f32 - 1.0;
        let w = half_width(u, v);
        let x = u * shape.length;
        let envelope = 4.0 * u * (1.0 - u);
        let z = -shape.bend * shape.length * u * u
            + shape.cup * w * v * v
            + shape.fold * w * v.abs()
            + shape.wave * w * v * v * envelope * (u * 5.0 * PI + 0.7 * v.signum()).sin();
        Vec3::new(x, v * w, z)
    };
    let mut points = Vec::with_capacity(((nu + 1) * (nv + 1)) as usize);
    for i in 0..=nu {
        for j in 0..=nv {
            points.push(local(i, j));
        }
    }
    let at = |i: u32, j: u32| points[(i * (nv + 1) + j) as usize];
    let mut normals = Vec::with_capacity(points.len());
    for i in 0..=nu {
        for j in 0..=nv {
            let du = at((i + 1).min(nu), j) - at(i.saturating_sub(1), j);
            let dv = at(i, (j + 1).min(nv)) - at(i, j.saturating_sub(1));
            // The blade's width collapses at the base and tip; use local up there.
            let n = du.cross(dv).normalize_or(Vec3::Z);
            normals.push(rotation * n);
        }
    }
    shade.resize(mesh.positions.len(), 1.0);
    let front = mesh.positions.len() as u32;
    for (k, p) in points.iter().enumerate() {
        let i = k as u32 / (nv + 1);
        let j = k as u32 % (nv + 1);
        let uv = Vec2::new(j as f32 / nv as f32, i as f32 / nu as f32);
        let n = normals[k];
        mesh.push_vertex(pose.base + rotation * *p, n, uv);
        shade.push(blade_shade(uv.y, 2.0 * uv.x - 1.0, false));
    }
    let back = mesh.positions.len() as u32;
    for k in 0..points.len() {
        let p = Vec3::from(mesh.positions[(front as usize) + k]);
        let n = -Vec3::from(mesh.normals[(front as usize) + k]);
        let uv = Vec2::from(mesh.uvs[(front as usize) + k]);
        mesh.push_vertex(p, n, uv);
        shade.push(blade_shade(uv.y, 2.0 * uv.x - 1.0, true));
    }
    let mut area = 0.0;
    for i in 0..nu {
        for j in 0..nv {
            let idx = |ii: u32, jj: u32| ii * (nv + 1) + jj;
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i, j + 1), idx(i + 1, j + 1));
            mesh.push_triangle(front + a, front + b, front + c);
            mesh.push_triangle(front + b, front + d, front + c);
            mesh.push_triangle(back + a, back + c, back + b);
            mesh.push_triangle(back + b, back + c, back + d);
            let tri = |p: u32, q: u32, r: u32| {
                let (p, q, r) = (points[p as usize], points[q as usize], points[r as usize]);
                0.5 * (q - p).cross(r - p).length()
            };
            area += tri(a, b, c) + tri(b, d, c);
        }
    }
    area
}

/// Builds a leaf at droop 0..=1 and normalizes its flat blade area to 1 m².
fn leaf(form: LeafForm, droop: f32) -> MeshData {
    leaf_shaded(form, droop).0
}

fn leaf_shaded(form: LeafForm, droop: f32) -> (MeshData, Vec<f32>) {
    let (mut mesh, mut shade, _) = leaf_raw(form, droop, false);
    let (_, _, flat_area) = leaf_raw(form, 0.0, true);
    if flat_area > 0.0 {
        mesh.scale(1.0 / flat_area.sqrt());
    }
    shade.resize(mesh.positions.len(), 1.0);
    (mesh, shade)
}

/// Like [`build_template`], plus one brightness multiplier per vertex (leaf veins,
/// margins and pale undersides; 1 everywhere for other templates). A renderer uploads
/// it as a grey vertex color, which multiplies the organ's color.
#[must_use]
pub fn build_template_shaded(key: TemplateKey) -> (MeshData, Vec<f32>) {
    let droop = if key.template.droops() {
        f32::from(key.droop.min(DROOP_LEVELS - 1)) / f32::from(DROOP_LEVELS - 1)
    } else {
        0.0
    };
    match key.template {
        Template::Unifoliate => leaf_shaded(LeafForm::Unifoliate, droop),
        Template::Trifoliate => leaf_shaded(LeafForm::Trifoliate, droop),
        _ => {
            let m = build_template(key);
            let n = m.vertex_count();
            (m, vec![1.0; n])
        }
    }
}

#[allow(clippy::too_many_lines)]
fn leaf_raw(form: LeafForm, droop: f32, flat: bool) -> (MeshData, Vec<f32>, f32) {
    let mut mesh = MeshData::new();
    let mut shade = Vec::new();
    let leaflet_length = 1.0;
    let petiole = match form {
        LeafForm::Unifoliate => 0.35,
        LeafForm::Trifoliate => 0.8,
    } * leaflet_length;
    // A turgid blade is held a little above the petiole; a wilted one hangs.
    let pitch = if flat { 0.0 } else { 0.17 - 1.1 * droop };
    let bend = if flat { 0.0 } else { 0.08 + 0.25 * droop };
    let cup = if flat { 0.0 } else { 0.06 };
    let fold = if flat { 0.0 } else { 0.14 };
    let wave = if flat { 0.0 } else { 0.07 };
    let petiole_rise = if flat {
        0.0
    } else {
        0.12 * petiole * (1.0 - droop)
    };
    let petiole_end = Vec3::new(petiole, 0.0, petiole_rise);
    let radius = 0.025 * leaflet_length;
    let petiole_nodes: Vec<TubeNode> = (0..=4)
        .map(|k| {
            let s = k as f32 / 4.0;
            TubeNode {
                position: Vec3::new(s * petiole, 0.0, petiole_rise * (PI * 0.5 * s).sin()),
                radius: radius * (1.0 - 0.4 * s),
            }
        })
        .collect();
    if !flat {
        tube(&petiole_nodes, 4, false, Vec3::X, &mut mesh);
    }
    let shape = |length: f32, ratio: f32, a: f32, asym: f32| Leaflet {
        length,
        width: length / ratio,
        a,
        b: 1.1,
        bend,
        cup,
        fold,
        wave,
        asym,
        acumen: 1.0,
    };
    let mut area = 0.0;
    match form {
        LeafForm::Unifoliate => {
            area += leaflet(
                &mut mesh,
                &mut shade,
                shape(leaflet_length, 1.35, 0.45, 0.0),
                Pose {
                    base: petiole_end,
                    yaw: 0.0,
                    pitch,
                },
            );
        }
        LeafForm::Trifoliate => {
            let rachis = 0.28 * leaflet_length;
            let rachis_end =
                petiole_end + Quat::from_rotation_y(-pitch) * Vec3::new(rachis, 0.0, 0.0);
            if !flat {
                tube(
                    &[
                        TubeNode {
                            position: petiole_end,
                            radius: radius * 0.6,
                        },
                        TubeNode {
                            position: rachis_end,
                            radius: radius * 0.5,
                        },
                    ],
                    4,
                    false,
                    Vec3::X,
                    &mut mesh,
                );
            }
            area += leaflet(
                &mut mesh,
                &mut shade,
                shape(leaflet_length, 1.4, 0.6, 0.0),
                Pose {
                    base: rachis_end,
                    yaw: 0.0,
                    pitch,
                },
            );
            for side in [1.0_f32, -1.0] {
                // Lateral leaflets sit on short petiolules; their outer half is wider.
                let yaw = side * 1.15;
                let stalk = 0.07 * leaflet_length;
                let base = petiole_end + Quat::from_rotation_z(yaw) * Vec3::new(stalk, 0.0, 0.0);
                if !flat {
                    tube(
                        &[
                            TubeNode {
                                position: petiole_end,
                                radius: radius * 0.5,
                            },
                            TubeNode {
                                position: base,
                                radius: radius * 0.6,
                            },
                        ],
                        4,
                        false,
                        Vec3::X,
                        &mut mesh,
                    );
                }
                area += leaflet(
                    &mut mesh,
                    &mut shade,
                    shape(0.85 * leaflet_length, 1.45, 0.6, -side * 0.18),
                    Pose {
                        base,
                        yaw,
                        pitch: pitch - 0.1,
                    },
                );
            }
        }
    }
    (mesh, shade, area)
}

/// Appends an ellipsoid with semi-axes `radii` (x is the pole axis).
pub(crate) fn ellipsoid(mesh: &mut MeshData, center: Vec3, radii: Vec3, rings: u32, segments: u32) {
    let first = mesh.positions.len() as u32;
    for i in 0..=rings {
        let phi = PI * i as f32 / rings as f32;
        for j in 0..=segments {
            let theta = TAU * j as f32 / segments as f32;
            // Exact poles, so the pole rings collapse to one point instead of slivers.
            let (sin_phi, cos_phi) = match i {
                0 => (0.0, 1.0),
                _ if i == rings => (0.0, -1.0),
                _ => phi.sin_cos(),
            };
            let d = Vec3::new(cos_phi, sin_phi * theta.cos(), sin_phi * theta.sin());
            let n = (d / radii).normalize_or(d);
            mesh.push_vertex(
                center + d * radii,
                n,
                Vec2::new(j as f32 / segments as f32, i as f32 / rings as f32),
            );
        }
    }
    let row = segments + 1;
    for i in 0..rings {
        for j in 0..segments {
            let a = first + i * row + j;
            let b = a + row;
            let c = a + 1;
            let d = b + 1;
            mesh.push_triangle(a, b, c);
            mesh.push_triangle(b, d, c);
        }
    }
}

/// A thick seed leaf: a flattened ellipsoid from the node outward, projected area 1 m².
fn cotyledon() -> MeshData {
    let (a, b, c) = (0.5_f32, 0.32_f32, 0.11_f32);
    let s = 1.0 / (PI * a * b).sqrt();
    let mut m = MeshData::new();
    ellipsoid(
        &mut m,
        Vec3::new(a * s, 0.0, 0.0),
        Vec3::new(a, b, c) * s,
        10,
        14,
    );
    m
}

/// A pod: length 1 along x, peak radius 1, gently curved down, with seed bumps and a beak.
///
/// Built slender (peak radius 0.08) so the swept tube cannot fold over itself at the
/// bend, then stretched across to unit radius.
fn pod() -> MeshData {
    const SLENDER: f32 = 0.08;
    let n = 24;
    let nodes: Vec<TubeNode> = (0..=n)
        .map(|k| {
            let u = k as f32 / n as f32;
            let envelope = (PI * u).sin().max(0.0).sqrt();
            let bumps = 1.0 + 0.08 * (TAU * 4.5 * u).cos();
            TubeNode {
                position: Vec3::new(u, 0.0, -0.6 * SLENDER * (PI * u).sin()),
                radius: SLENDER * envelope * bumps,
            }
        })
        .collect();
    let mut slender = MeshData::new();
    tube(&nodes, 10, false, Vec3::X, &mut slender);
    let mut m = MeshData::new();
    m.append_transformed(
        &slender,
        Affine3A::from_scale(Vec3::new(1.0, 1.0 / SLENDER, 1.0 / SLENDER)),
    );
    m
}

/// A stylized bean flower about 1 long: calyx, an upright banner, two wings and a keel.
fn flower() -> MeshData {
    let mut m = MeshData::new();
    tube(
        &[
            TubeNode {
                position: Vec3::ZERO,
                radius: 0.04,
            },
            TubeNode {
                position: Vec3::new(0.3, 0.0, 0.0),
                radius: 0.12,
            },
        ],
        6,
        true,
        Vec3::X,
        &mut m,
    );
    let petal = |length: f32| Leaflet {
        length,
        width: length * 0.9,
        a: 0.5,
        b: 0.6,
        bend: -0.1,
        cup: 0.25,
        fold: 0.0,
        wave: 0.0,
        asym: 0.0,
        acumen: 0.0,
    };
    let mut shade = Vec::new();
    leaflet(
        &mut m,
        &mut shade,
        petal(0.65),
        Pose {
            base: Vec3::new(0.3, 0.0, 0.05),
            yaw: 0.0,
            pitch: FRAC_PI_2 * 0.8,
        },
    );
    for side in [1.0_f32, -1.0] {
        leaflet(
            &mut m,
            &mut shade,
            petal(0.45),
            Pose {
                base: Vec3::new(0.3, 0.0, 0.0),
                yaw: side * 0.6,
                pitch: 0.15,
            },
        );
    }
    ellipsoid(
        &mut m,
        Vec3::new(0.5, 0.0, -0.02),
        Vec3::new(0.2, 0.09, 0.09),
        6,
        8,
    );
    m
}
