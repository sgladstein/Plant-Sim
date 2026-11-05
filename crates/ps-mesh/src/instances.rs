//! Instance transforms for templated organs.

use glam::{Mat3, Quat, Vec3};
use ps_graph::{OrganId, OrganKind};

use crate::templates::BLADE_SHAPE;
use crate::{DROOP_LEVELS, Placement, Skeleton, Template, TemplateKey};

/// Downward tilt of a turgid blade toward its tip, rad: the light model's leaf normal
/// leans back by atan(0.35) (`ps_arch::geometry::leaf_points`).
pub const BLADE_TILT: f32 = 0.337;

/// Largest render-only pose jitter of a leaf blade (yaw, pitch, roll), rad. Only the yaw
/// varies: since decision 0025 the light model intercepts by each blade's angle, so the
/// drawn tilt stays the sim's (`BLADE_TILT`) until the sim varies it itself.
pub const LEAF_JITTER: [f32; 3] = [0.14, 0.0, 0.0];

/// A deterministic number in −1..1 for organ `id`, channel `k`.
#[must_use]
pub fn jitter(id: u32, k: u32) -> f32 {
    let mut x = u64::from(id) << 8 | u64::from(k);
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    // The top 24 bits, as a fraction.
    #[allow(clippy::cast_precision_loss)]
    let f = (x >> 40) as f32 / (1_u32 << 24) as f32;
    2.0 * f - 1.0
}

/// Thermal age by which a bean leaf has unfolded, °C·day: trifoliates emerge folded
/// along their midribs and open over about the first week (at 20 °C mean, base 10 °C).
pub const UNFOLD_THERMAL_AGE: f32 = 70.0;

/// Unfolding level for a leaf of thermal age `age` (°C·day): folded shut when it emerges,
/// open at [`UNFOLD_THERMAL_AGE`]. Age rather than area, because a carbon-limited leaf
/// stays small but still opens.
#[must_use]
pub fn unfold_level(age: f32) -> u8 {
    let a = if age.is_finite() {
        age
    } else {
        UNFOLD_THERMAL_AGE
    };
    let open = (a / UNFOLD_THERMAL_AGE).clamp(0.0, 1.0);
    // In 0..=OPEN by the clamp, so the cast is exact.
    #[allow(clippy::cast_sign_loss)]
    let level = (open * f32::from(TemplateKey::OPEN)).round() as u8;
    level
}

/// Where a leaf's petiole ends and its blade begins (the pulvinus), m: the petiole runs
/// straight from the node along the leaf's direction, as the light model places it.
#[must_use]
pub fn pulvinus(p: &Placement) -> Vec3 {
    p.base + p.direction * p.petiole_length()
}

/// Rotation of a leaf blade: facing the petiole's compass direction, tilted down toward
/// its tip as the light model holds it, plus the per-leaf jitter.
fn blade_frame(p: &Placement, id: u32) -> Quat {
    let h = Vec3::new(p.direction.x, p.direction.y, 0.0);
    let yaw = if h.length_squared() > 1e-10 {
        h.y.atan2(h.x)
    } else {
        p.azimuth
    };
    let [jy, jp, jr] = LEAF_JITTER;
    Quat::from_rotation_z(yaw + jy * jitter(id, 0))
        * Quat::from_rotation_y(BLADE_TILT + jp * jitter(id, 1))
        * Quat::from_rotation_x(jr * jitter(id, 2))
}

/// One templated organ: which shared mesh, and where.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Instance {
    /// The organ.
    pub organ: OrganId,
    /// The shared mesh.
    pub key: TemplateKey,
    /// Attachment point, m.
    pub translation: Vec3,
    /// Template frame to world.
    pub rotation: Quat,
    /// Per-axis scale (see the table in [`crate::templates`]).
    pub scale: Vec3,
}

/// What [`instances`] includes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InstanceOptions {
    /// The plant's water stress factor, 1 = unstressed, 0 = fully stressed (§5.2 plant
    /// water lens). Drives leaf droop.
    pub water_stress_factor: f32,
    /// Include dead organs (for a lens or the inspector); normally off.
    pub include_dead: bool,
    /// Include seeds inside pods (hidden inside the pod mesh); normally off.
    pub include_seeds_in_pods: bool,
}

impl Default for InstanceOptions {
    fn default() -> Self {
        Self {
            water_stress_factor: 1.0,
            include_dead: false,
            include_seeds_in_pods: false,
        }
    }
}

/// Droop level for a water stress factor (1 = turgid → level 0).
#[must_use]
pub fn droop_level(water_stress_factor: f32) -> u8 {
    let s = if water_stress_factor.is_finite() {
        water_stress_factor.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let levels = f32::from(DROOP_LEVELS - 1);
    // In 0..=levels by the clamp above, so the cast is exact.
    #[allow(clippy::cast_sign_loss)]
    let level = ((1.0 - s) * levels).round() as u8;
    level
}

/// The leaf template for a leaf organ.
///
/// The sim's role when the placement carries it; otherwise on a bean the first two leaves
/// of the main stem are unifoliate and every later leaf is trifoliate.
/// `main_stem_leaves_before` counts the main-stem leaves with smaller ids.
#[must_use]
pub fn leaf_form(placement: &Placement, main_stem_leaves_before: usize) -> Template {
    if let Some(simple) = placement.simple_leaf {
        return if simple {
            Template::Unifoliate
        } else {
            Template::Trifoliate
        };
    }
    if placement.axis_order == 0 && main_stem_leaves_before < 2 {
        Template::Unifoliate
    } else {
        Template::Trifoliate
    }
}

/// A rotation taking the template frame (x forward, z up) to `forward`, keeping the
/// template's up as close to world up as possible. A vertical `forward` uses the
/// phyllotactic azimuth to choose the side.
fn frame(forward: Vec3, azimuth: f32) -> Quat {
    let f = forward.normalize_or(Vec3::X);
    let mut side = Vec3::Z.cross(f).normalize_or_zero();
    if side == Vec3::ZERO {
        side = Vec3::new(-azimuth.sin(), azimuth.cos(), 0.0);
    }
    let up = f.cross(side);
    Quat::from_mat3(&Mat3::from_cols(f, side, up)).normalize()
}

/// One instance per templated organ of `skeleton`, in organ id order.
#[must_use]
pub fn instances(skeleton: &Skeleton, options: InstanceOptions) -> Vec<Instance> {
    let droop = droop_level(options.water_stress_factor);
    let mut main_leaves = 0_usize;
    let mut out = Vec::new();
    let grass = skeleton.blade_aspect.filter(|a| a.is_finite() && *a > 0.0);
    for (i, p) in skeleton.organs.iter().enumerate() {
        let template = match (p.kind, grass) {
            (OrganKind::Leaf, Some(_)) => Template::GrassBlade,
            (OrganKind::Pod, Some(_)) => Template::WheatEar,
            (OrganKind::Leaf, None) => {
                let t = leaf_form(p, main_leaves);
                if p.axis_order == 0 {
                    main_leaves += 1;
                }
                t
            }
            (OrganKind::Cotyledon, None) => Template::Cotyledon,
            (OrganKind::Flower, None) => Template::Flower,
            (OrganKind::Pod, None) => Template::Pod,
            (OrganKind::Seed, _) => Template::Seed,
            (OrganKind::SeedInPod, _) if options.include_seeds_in_pods => Template::Seed,
            (OrganKind::Bud, _) => Template::Bud,
            (OrganKind::Nodule, None) => Template::Nodule,
            // Roots and stems are axes; a grass's endosperm stays in the soil and its
            // florets are too small to draw.
            _ => continue,
        };
        if !p.alive && !options.include_dead {
            continue;
        }
        let scale = match template {
            Template::Cotyledon | Template::Unifoliate | Template::Trifoliate => {
                Vec3::splat(p.area.sqrt())
            }
            Template::GrassBlade => {
                let aspect = grass.unwrap_or(1.0);
                let width = (p.area.max(0.0) / (BLADE_SHAPE * aspect)).sqrt();
                Vec3::new(aspect * width, width, aspect * width)
            }
            Template::Pod | Template::Seed | Template::WheatEar => {
                Vec3::new(p.length, p.radius, p.radius)
            }
            Template::Flower => Vec3::splat(p.length),
            Template::Bud => Vec3::splat(p.radius),
            Template::Nodule => Vec3::splat(if p.radius > 0.0 {
                p.radius.clamp(0.000_7, 0.004)
            } else {
                0.001_5
            }),
        };
        let id = i as u32;
        let key = TemplateKey {
            template,
            droop: if template.droops() { droop } else { 0 },
            unfold: if template.unfolds() {
                unfold_level(p.thermal_age)
            } else {
                TemplateKey::OPEN
            },
        };
        let (translation, rotation) = match template {
            // The blade hangs from the end of its petiole (drawn as a tube).
            Template::Unifoliate | Template::Trifoliate => (pulvinus(p), blade_frame(p, id)),
            // Flowers and pods on one raceme share its tip: turn each a little so they
            // fan out instead of stacking.
            Template::Flower | Template::Pod => {
                let turn = Quat::from_rotation_z(0.25 * jitter(id, 0))
                    * Quat::from_rotation_y(0.2 * jitter(id, 1));
                (p.base, frame(p.direction, p.azimuth) * turn)
            }
            Template::Nodule => (
                p.base,
                Quat::from_rotation_z(3.0 * jitter(id, 0))
                    * Quat::from_rotation_x(3.0 * jitter(id, 1)),
            ),
            _ => (p.base, frame(p.direction, p.azimuth)),
        };
        out.push(Instance {
            organ: OrganId(id),
            key,
            translation,
            rotation,
            scale,
        });
    }
    out
}
