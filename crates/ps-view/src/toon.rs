//! Illustrated shading (visual design review, 2026-11-05; the owner chose an
//! "illustrated" look with cel shading and ink lines).
//!
//! [`Toon`] is a `StandardMaterial` with a custom fragment shader (`shaders/toon.wesl`):
//! a soft two-tone sun ramp with a cool shade tone, light through backlit leaves, a warm
//! rim, and ink darkening at silhouettes. Base color, vertex colors, emissive (lens
//! colors), alpha, fog and tonemapping work as in `StandardMaterial`, so the scene code
//! only swaps the material type.

use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// The illustrated material.
pub type Toon = ExtendedMaterial<StandardMaterial, ToonExt>;

/// Settings for the shader: `strength` 0 draws standard PBR, 1 the illustrated look.
#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
pub struct ToonExt {
    /// 0 = natural, 1 = illustrated.
    #[uniform(100)]
    pub strength: f32,
}

impl MaterialExtension for ToonExt {
    fn fragment_shader() -> ShaderRef {
        "embedded://ps_view/shaders/toon.wesl".into()
    }
}

/// A [`Toon`] from a standard material, drawn in the current [`Style`].
#[must_use]
pub fn toon(base: StandardMaterial) -> Toon {
    ExtendedMaterial {
        base,
        extension: ToonExt { strength: 1.0 },
    }
}

/// The art style. A resource; the app sets it from `--look`.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Style {
    /// Soft natural light (PBR).
    Natural,
    /// Cel shading with ink lines.
    #[default]
    Illustrated,
}

impl Style {
    /// Parses `natural` or `illustrated`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "natural" => Some(Self::Natural),
            "illustrated" => Some(Self::Illustrated),
            _ => None,
        }
    }

    fn strength(self) -> f32 {
        match self {
            Self::Natural => 0.0,
            Self::Illustrated => 1.0,
        }
    }
}

/// Pushes the style into every illustrated material when it changes (also catches
/// materials created since, such as new lens colors).
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn apply_style(style: Res<Style>, mut toons: ResMut<Assets<Toon>>) {
    let s = style.strength();
    let stale = toons
        .iter()
        .any(|(_, m)| (m.extension.strength - s).abs() > f32::EPSILON);
    if stale {
        for (_, m) in toons.iter_mut() {
            m.extension.strength = s;
        }
    }
}

/// Registers the shader and the material.
pub(crate) fn register(app: &mut App) {
    bevy::asset::embedded_asset!(app, "shaders/toon.wesl");
    app.add_plugins(MaterialPlugin::<Toon>::default())
        .init_resource::<Style>()
        .add_systems(PostUpdate, apply_style);
}
