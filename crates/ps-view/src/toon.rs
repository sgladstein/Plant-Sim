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
    /// x: style (0 = natural, 1 = illustrated); y: background (1 for set dressing such
    /// as lawn and trees, drawn flatter and paler so the plant stands out); z, w unused.
    #[uniform(100)]
    pub params: Vec4,
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
        extension: ToonExt {
            params: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    }
}

/// A [`Toon`] for background set dressing (flatter and paler in the illustrated style).
#[must_use]
pub fn toon_background(base: StandardMaterial) -> Toon {
    ExtendedMaterial {
        base,
        extension: ToonExt {
            params: Vec4::new(1.0, 1.0, 0.0, 0.0),
        },
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
    /// Painterly soft: PBR lighting under a full-screen watercolor pass
    /// (`crate::painterly`), with a painted sky.
    Painterly,
}

impl Style {
    /// Parses `natural`, `illustrated` or `painterly`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "natural" => Some(Self::Natural),
            "illustrated" => Some(Self::Illustrated),
            "painterly" => Some(Self::Painterly),
            _ => None,
        }
    }

    /// The shader's style value.
    #[must_use]
    pub fn strength(self) -> f32 {
        match self {
            // Painterly keeps PBR lighting; its look comes from the full-screen pass.
            Self::Natural | Self::Painterly => 0.0,
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
        .any(|(_, m)| (m.extension.params.x - s).abs() > f32::EPSILON);
    if stale {
        for (_, m) in toons.iter_mut() {
            m.extension.params.x = s;
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
