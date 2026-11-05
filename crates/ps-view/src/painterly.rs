//! Option B of the look exploration: "painterly soft" (`--look painterly`).
//!
//! No outlines. Shapes keep their geometry; what changes is how they are painted:
//!
//! - A full-screen pass after tonemapping ([`PainterlyPost`], `painterly.wgsl`) that
//!   flattens detail into brush patches (a soft Kuwahara filter), pulls values toward a
//!   few soft bands, mutes the palette toward watercolor (cool shadows, warm lights,
//!   lifted darks) and lays the image on paper (grain, blotches, pigment pooling).
//! - A painted sky: the gradient in soft horizontal bands with loose cloud washes and
//!   brush streaks, in a muted cerulean over a warm, paper-colored horizon.
//!
//! With a lens on, the pass drops the value steps, palette shift and pigment pooling, so
//! lens colors stay readable against their legends; the brush patches and paper remain.
//! The pass costs one full-screen draw (about 100 texture reads a pixel at 1440p).

use bevy::asset::embedded_asset;
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::Core3dSystems;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;
use bevy::render::RenderApp;
use bevy::shader::ShaderRef;

use crate::camera::OrbitCamera;
use crate::lens::Lens;
use crate::toon::Style;

/// Settings of the painterly pass, one per camera (see `painterly.wgsl`).
#[derive(Component, ExtractComponent, Clone, Copy, Debug, PartialEq, ShaderType)]
#[extract_app(RenderApp)]
pub struct PainterlyPost {
    /// Brush patch radius at a 1440 px tall view, px.
    pub radius: f32,
    /// How far values are pulled to their band, 0 to 1.
    pub steps: f32,
    /// Desaturation and warm/cool split, 0 to 1.
    pub mute: f32,
    /// Paper grain and blotches, 0 to 1.
    pub paper: f32,
    /// Edge wobble at 1440 px, px.
    pub wobble: f32,
    /// Pigment pooling at edges, 0 to 1.
    pub edge: f32,
    /// Number of value bands.
    pub bands: f32,
    /// Share of the unfiltered pixel kept, 0 to 1, so thin roots and stems survive.
    pub detail: f32,
}

impl Default for PainterlyPost {
    fn default() -> Self {
        Self::for_lens(Lens::Off)
    }
}

impl PainterlyPost {
    /// Settings for a lens: full strength with no lens, values and hues left alone with
    /// one (the lens colors are the data).
    #[must_use]
    pub fn for_lens(lens: Lens) -> Self {
        let off = lens == Lens::Off;
        Self {
            // Smaller patches under a lens: root and stem values are thin lines there.
            radius: if off { 3.0 } else { 2.0 },
            steps: if off { 0.8 } else { 0.0 },
            mute: if off { 1.0 } else { 0.0 },
            paper: if off { 1.0 } else { 0.45 },
            wobble: 1.6,
            edge: if off { 1.0 } else { 0.0 },
            bands: 5.0,
            detail: if off { 0.45 } else { 0.6 },
        }
    }
}

impl FullscreenMaterial for PainterlyPost {
    fn fragment_shader() -> ShaderRef {
        "embedded://ps_view/painterly.wgsl".into()
    }

    fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
        // After tonemapping: the pass works on display values, so its bands and palette
        // do not depend on exposure.
        system.in_set(Core3dSystems::PostProcess).after(tonemapping)
    }
}

pub(crate) fn build(app: &mut App) {
    embedded_asset!(app, "painterly.wgsl");
    app.add_plugins(FullscreenMaterialPlugin::<PainterlyPost>::default());
}

/// Puts the pass on the camera (or takes it off) to follow [`Style`], and retunes it for
/// the lens.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn apply_style(
    mut commands: Commands,
    style: Res<Style>,
    lens: Res<Lens>,
    cameras: Query<(Entity, Option<&PainterlyPost>), With<OrbitCamera>>,
) {
    for (entity, post) in &cameras {
        if *style == Style::Painterly {
            let want = PainterlyPost::for_lens(*lens);
            if post != Some(&want) {
                commands.entity(entity).insert(want);
            }
        } else if post.is_some() {
            commands.entity(entity).remove::<PainterlyPost>();
        }
    }
}

// --- Painted sky ---

/// Side of each painted sky cubemap face, px (the washes need more texels than the
/// smooth gradient).
pub const SKY_FACE: u32 = 128;

/// Horizon: warm paper, linear RGB (also the fog color).
pub const HORIZON: [f32; 3] = [0.74, 0.68, 0.55];
/// Zenith: muted cerulean, linear RGB.
pub const ZENITH: [f32; 3] = [0.25, 0.40, 0.62];
/// Ground seen far away: a sage wash, linear RGB.
pub const GROUND: [f32; 3] = [0.12, 0.16, 0.075];
/// Cloud wash color, linear RGB (warm white, lighter than the sky behind it).
const CLOUD: [f32; 3] = [0.86, 0.82, 0.74];

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash3(p: [i32; 3]) -> f32 {
    let mut h = (p[0].cast_unsigned())
        .wrapping_mul(0x8da6_b343)
        .wrapping_add(p[1].cast_unsigned().wrapping_mul(0xd816_3841))
        .wrapping_add(p[2].cast_unsigned().wrapping_mul(0xcb1a_b31f));
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// Smooth 3D value noise in 0..1.
fn noise3(p: Vec3) -> f32 {
    let i = p.floor();
    let f = p - i;
    let u = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let (x, y, z) = (i.x as i32, i.y as i32, i.z as i32);
    let c = |dx: i32, dy: i32, dz: i32| hash3([x + dx, y + dy, z + dz]);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(c(0, 0, 0), c(1, 0, 0), u.x);
    let x10 = lerp(c(0, 1, 0), c(1, 1, 0), u.x);
    let x01 = lerp(c(0, 0, 1), c(1, 0, 1), u.x);
    let x11 = lerp(c(0, 1, 1), c(1, 1, 1), u.x);
    lerp(lerp(x00, x10, u.y), lerp(x01, x11, u.y), u.z)
}

fn fbm3(p: Vec3) -> f32 {
    let mut s = 0.0;
    let mut amp = 0.5;
    let mut q = p;
    for _ in 0..4 {
        s += amp * noise3(q);
        q = q * 2.07 + Vec3::new(5.3, 1.7, 9.1);
        amp *= 0.5;
    }
    s / 0.9375
}

/// Painted sky radiance (relative, linear RGB) toward a Bevy-frame direction (y up),
/// before the time-of-day tint.
#[must_use]
pub fn sky_gradient(dir: Vec3) -> [f32; 3] {
    let d = dir.normalize_or(Vec3::Y);
    let y = d.y;
    if y < 0.0 {
        return mix3(HORIZON, GROUND, (-y * 6.0).powf(0.7));
    }
    // The gradient in five soft bands, as if laid in with a wide flat brush, with the
    // band edges wavering a little along the horizon.
    let waver = 0.04 * (fbm3(Vec3::new(d.x * 2.0, 0.0, d.z * 2.0)) - 0.5);
    let t = (y + waver).clamp(0.0, 1.0).powf(0.55);
    let bands = 5.0;
    let x = t * bands;
    let band = (x.floor() + smoothstep(0.3, 0.7, x.fract())) / bands;
    let mut c = mix3(HORIZON, ZENITH, 0.6 * band + 0.4 * t);
    // Brush streaks: noise stretched along the horizon.
    let streak = noise3(Vec3::new(d.x * 3.0, y * 40.0, d.z * 3.0)) - 0.5;
    c = c.map(|v| v * (1.0 + 0.06 * streak));
    // Loose cloud washes in the low and middle sky, with soft but definite edges.
    let n = fbm3(d * 2.6 + Vec3::new(0.0, 3.0, 0.0));
    let cover = smoothstep(0.52, 0.62, n) * smoothstep(0.02, 0.12, y) * (1.0 - smoothstep(0.55, 0.9, y));
    let rim = smoothstep(0.52, 0.56, n) - smoothstep(0.56, 0.64, n);
    let cloud = mix3(CLOUD, [0.66, 0.66, 0.72], 0.6 * (n - 0.6).max(0.0) * 4.0);
    c = mix3(c, cloud, 0.85 * cover);
    // A faint darker edge where a wash dried.
    c.map(|v| v * (1.0 - 0.08 * rim * smoothstep(0.02, 0.12, y)))
}

/// The painted sky's horizon color for a tint (matches the fog).
#[must_use]
pub fn horizon(tint: [f32; 3]) -> [f32; 3] {
    [HORIZON[0] * tint[0], HORIZON[1] * tint[1], HORIZON[2] * tint[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lens_turns_value_and_hue_shifts_off() {
        let p = PainterlyPost::for_lens(Lens::Light);
        assert!(p.steps == 0.0 && p.mute == 0.0 && p.edge == 0.0);
        let off = PainterlyPost::for_lens(Lens::Off);
        assert!(off.steps > 0.0 && off.mute > 0.0);
    }

    #[test]
    fn painted_sky_is_blue_above_and_warm_at_the_horizon() {
        let z = sky_gradient(Vec3::Y);
        assert!(z[2] > z[0], "{z:?}");
        let h = sky_gradient(Vec3::new(1.0, 0.0, 0.0));
        assert!(h[0] > h[2], "{h:?}");
        let g = sky_gradient(Vec3::new(0.0, -1.0, 0.0));
        assert!(g.iter().zip(GROUND).all(|(a, b)| (a - b).abs() < 1e-6));
    }

    #[test]
    fn noise_is_in_range() {
        for i in 0..200 {
            let p = Vec3::new(i as f32 * 0.37, i as f32 * -0.11, i as f32 * 0.07);
            let n = fbm3(p);
            assert!((0.0..=1.0).contains(&n), "{n}");
        }
    }
}
