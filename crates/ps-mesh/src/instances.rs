//! Instance transforms for templated organs.

use glam::{Mat3, Quat, Vec3};
use ps_graph::{OrganId, OrganKind};

use crate::templates::BLADE_SHAPE;
use crate::{DROOP_LEVELS, Placement, Skeleton, Template, TemplateKey};

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
/// Provisional until ps-arch records leaf form: on a bean the first two leaves of the
/// main stem are unifoliate and every later leaf is trifoliate. `main_stem_leaves_before`
/// counts the main-stem leaves with smaller ids.
#[must_use]
pub fn leaf_form(placement: &Placement, main_stem_leaves_before: usize) -> Template {
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
        };
        let key = TemplateKey {
            template,
            droop: if template.droops() { droop } else { 0 },
        };
        out.push(Instance {
            organ: OrganId(i as u32),
            key,
            translation: p.base,
            rotation: frame(p.direction, p.azimuth),
            scale,
        });
    }
    out
}
