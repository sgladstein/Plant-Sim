//! The look: "soft naturalism" (visual design review, 2026-11-05).
//!
//! A specimen on a potting bench under an open sky: soft hemispheric light from a
//! procedural sky cubemap, one sun with tight shadows, matte materials, a neutral
//! tonemapper so lens colors match their legends, a ground that fades into the horizon
//! instead of ending at a hard edge, and a light vignette. No outlines, no cel bands, no
//! textures: everything here is procedural and cheap on a GTX 1070.

use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::Image;
use bevy::light::{GeneratedEnvironmentMapLight, Skybox};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::post_process::effect_stack::Vignette;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

/// Side of each sky cubemap face, px (a power of two for the GPU filter).
pub const SKY_FACE: u32 = 32;

/// Camera exposure multiplier at Bevy's default EV100 of 9.7: 1 / (1.2 · 2^9.7).
pub const EXPOSURE: f32 = 1.0 / 1003.0;

/// Fraction of a lens color that is emitted rather than lit, so shading cannot push a
/// value toward the dark end of its scale.
pub const LENS_GLOW: f32 = 0.7;

/// Horizon haze, linear RGB (also the fog color, so the ground melts into the sky).
pub const HORIZON: [f32; 3] = [0.66, 0.63, 0.55];
/// Zenith, linear RGB.
pub const ZENITH: [f32; 3] = [0.22, 0.40, 0.72];
/// Ground seen far away, linear RGB.
pub const GROUND: [f32; 3] = [0.085, 0.15, 0.045];

/// Paper color of the botanical-plate backdrop, linear RGB.
pub const PAPER: [f32; 3] = [0.80, 0.74, 0.60];

/// What surrounds the specimen: the garden, or a seamless paper sweep like a botanical
/// plate (no bench, lawn or trees; sky, ground and fog all one paper color). Set at
/// startup.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Backdrop {
    /// Bench, lawn, hedges and sky.
    #[default]
    Garden,
    /// A paper sweep.
    Plate,
}

impl Backdrop {
    /// Parses `garden` or `plate`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "garden" => Some(Self::Garden),
            "plate" | "paper" => Some(Self::Plate),
            _ => None,
        }
    }

    /// How far the ground sits below the soil surface, m: the bench height in the
    /// garden, a hair in the plate (the pots stand on the paper).
    #[must_use]
    pub fn ground_drop(self) -> f32 {
        match self {
            Self::Garden => crate::dressing::BENCH_HEIGHT,
            Self::Plate => 0.002,
        }
    }
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Sky radiance (relative, linear RGB) toward a Bevy-frame direction (y up).
#[must_use]
pub fn sky_gradient(dir: Vec3, tint: [f32; 3]) -> [f32; 3] {
    let y = dir.normalize_or(Vec3::Y).y;
    let c = if y >= 0.0 {
        mix(HORIZON, ZENITH, y.powf(0.6))
    } else {
        mix(HORIZON, GROUND, (-y * 6.0).powf(0.7))
    };
    [c[0] * tint[0], c[1] * tint[1], c[2] * tint[2]]
}

/// Hue of the sky for a sun elevation sine and cloud fraction (1 at a clear noon):
/// warm at dawn and dusk, blue-black at night, greyer under cloud.
#[must_use]
pub fn sky_tint(sin_elevation: f32, cloud: f32) -> [f32; 3] {
    let dusk = [1.35, 0.85, 0.6];
    let night = [0.25, 0.3, 0.55];
    let hue = if sin_elevation < 0.0 {
        mix(dusk, night, -sin_elevation * 8.0)
    } else {
        mix(dusk, [1.0; 3], sin_elevation * 4.0)
    };
    let grey = (hue[0] + hue[1] + hue[2]) / 3.0;
    let g = 0.7 * cloud.clamp(0.0, 1.0);
    [
        hue[0] + (grey - hue[0]) * g,
        hue[1] + (grey - hue[1]) * g,
        hue[2] + (grey - hue[2]) * g,
    ]
}

/// Direction through texel (u, v) of cube face `face` (+x, −x, +y, −y, +z, −z).
fn cube_dir(face: u32, u: f32, v: f32) -> Vec3 {
    let (a, b) = (2.0 * u - 1.0, 2.0 * v - 1.0);
    match face {
        0 => Vec3::new(1.0, -b, -a),
        1 => Vec3::new(-1.0, -b, a),
        2 => Vec3::new(a, 1.0, b),
        3 => Vec3::new(a, -1.0, -b),
        4 => Vec3::new(a, -b, 1.0),
        _ => Vec3::new(-a, -b, -1.0),
    }
}

fn f16_bits(v: f32) -> u16 {
    // Positive, finite, small values only (sky radiance): a plain truncating encoder.
    let v = v.clamp(0.0, 60_000.0);
    if v < 6.2e-5 {
        return 0;
    }
    let bits = v.to_bits();
    let exp = ((bits >> 23) & 0xff).cast_signed() - 127 + 15;
    let mant = (bits >> 13) & 0x3ff;
    ((exp.clamp(1, 30) as u32) << 10 | mant) as u16
}

/// Fills cube texel data (`Rgba16Float`) with the sky gradient.
fn sky_texels(tint: [f32; 3]) -> Vec<u8> {
    sky_texels_with(|d| sky_gradient(d, tint))
}

/// Fills cube texel data (`Rgba16Float`) from a direction → color function.
fn sky_texels_with(color: impl Fn(Vec3) -> [f32; 3]) -> Vec<u8> {
    let n = SKY_FACE as usize;
    let mut data = Vec::with_capacity(n * n * 6 * 8);
    for face in 0..6 {
        for y in 0..n {
            for x in 0..n {
                let d = cube_dir(
                    face,
                    (x as f32 + 0.5) / n as f32,
                    (y as f32 + 0.5) / n as f32,
                );
                let c = color(d);
                for ch in [c[0], c[1], c[2], 1.0] {
                    data.extend_from_slice(&f16_bits(ch).to_le_bytes());
                }
            }
        }
    }
    data
}

/// A sky cubemap image with the gradient.
#[must_use]
pub fn sky_cubemap(tint: [f32; 3]) -> Image {
    cube_image(sky_texels(tint))
}

/// A cubemap of one flat color (the plate's paper).
#[must_use]
pub fn flat_cubemap(c: [f32; 3]) -> Image {
    cube_image(sky_texels_with(|_| c))
}

/// The sky cubemap for a style: the smooth gradient, or the painted sky
/// (`crate::painterly`) when `painted`. The painted washes are computed once and cached.
#[must_use]
pub fn styled_sky_cubemap(painted: bool, tint: [f32; 3]) -> Image {
    static PAINTED: std::sync::OnceLock<Vec<[f32; 3]>> = std::sync::OnceLock::new();
    if !painted {
        return sky_cubemap(tint);
    }
    let base = PAINTED.get_or_init(|| {
        let n = crate::painterly::SKY_FACE as usize;
        let mut v = Vec::with_capacity(n * n * 6);
        for face in 0..6 {
            for y in 0..n {
                for x in 0..n {
                    v.push(crate::painterly::sky_gradient(cube_dir(
                        face,
                        (x as f32 + 0.5) / n as f32,
                        (y as f32 + 0.5) / n as f32,
                    )));
                }
            }
        }
        v
    });
    let mut data = Vec::with_capacity(base.len() * 8);
    for c in base {
        for ch in [c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], 1.0] {
            data.extend_from_slice(&f16_bits(ch).to_le_bytes());
        }
    }
    cube_image_sized(crate::painterly::SKY_FACE, data)
}

fn cube_image(texels: Vec<u8>) -> Image {
    cube_image_sized(SKY_FACE, texels)
}

fn cube_image_sized(face: u32, texels: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: face,
            height: face,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        texels,
        TextureFormat::Rgba16Float,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image
}

/// Handles to the sky images.
#[derive(Resource)]
pub struct SkyImages {
    /// What the camera sees behind the scene; retinted with the time of day.
    pub backdrop: Handle<Image>,
    /// Source of the image-based fill light; fixed, scaled by daylight.
    pub fill: Handle<Image>,
    /// Tint the backdrop was last built with.
    pub tint: [f32; 3],
    /// Whether the backdrop is the painted sky (`Style::Painterly`).
    pub painted: bool,
}

/// Creates the sky images (Startup, before the camera spawns).
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn setup_sky_images(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    backdrop: Res<Backdrop>,
    style: Res<crate::toon::Style>,
) {
    let tint = [1.0; 3];
    let painted = *style == crate::toon::Style::Painterly;
    let sky = match *backdrop {
        Backdrop::Garden => styled_sky_cubemap(painted, tint),
        Backdrop::Plate => flat_cubemap(PAPER),
    };
    commands.insert_resource(SkyImages {
        backdrop: images.add(sky),
        fill: images.add(sky_cubemap(tint)),
        tint,
        painted,
    });
}

/// Camera components for the look.
#[must_use]
pub fn camera_bundle(sky: &SkyImages) -> impl Bundle {
    (
        // Neutral: base colors (and lens colors) come out close to what they are.
        Tonemapping::KhronosPbrNeutral,
        Skybox {
            image: Some(sky.backdrop.clone()),
            brightness: 1000.0,
            ..default()
        },
        GeneratedEnvironmentMapLight {
            environment_map: sky.fill.clone(),
            intensity: 900.0,
            ..default()
        },
        DistanceFog {
            color: Color::linear_rgb(HORIZON[0], HORIZON[1], HORIZON[2]),
            // Trees at 13 to 22 m stay visible as soft shapes; the ground's edge is gone.
            falloff: FogFalloff::Linear {
                start: 2.0,
                end: 60.0,
            },
            ..default()
        },
        Vignette {
            intensity: 0.22,
            radius: 0.9,
            smoothness: 2.5,
            ..default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noon_tint_is_neutral() {
        let t = sky_tint(1.0, 0.0);
        assert!(t.iter().all(|&v| (v - 1.0).abs() < 1e-6), "{t:?}");
        let dusk = sky_tint(0.0, 0.0);
        assert!(dusk[0] > dusk[2]);
        let night = sky_tint(-0.5, 0.0);
        assert!(night[2] > night[0]);
    }

    #[test]
    fn cubemap_has_six_square_faces() {
        let n = SKY_FACE as usize;
        assert!(SKY_FACE.is_power_of_two());
        assert_eq!(sky_texels([1.0; 3]).len(), n * n * 6 * 8);
        let img = sky_cubemap([1.0; 3]);
        assert_eq!(img.texture_descriptor.size.depth_or_array_layers, 6);
    }

    #[test]
    fn horizon_matches_fog_color_and_zenith_is_blue() {
        let h = sky_gradient(Vec3::X, [1.0; 3]);
        assert!(h.iter().zip(HORIZON).all(|(a, b)| (a - b).abs() < 1e-6));
        let z = sky_gradient(Vec3::Y, [1.0; 3]);
        assert!(z[2] > z[0]);
    }

    #[test]
    fn f16_encoder_round_trips_simple_values() {
        assert_eq!(f16_bits(1.0), 0x3c00);
        assert_eq!(f16_bits(0.5), 0x3800);
        assert_eq!(f16_bits(0.0), 0);
    }
}
