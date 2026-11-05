//! Sun, sky color and ambient light from the sim's weather (design plan §2.1, §4).

use bevy::prelude::*;

use crate::scene::sim_to_bevy;

/// What the sky looks like this frame, filled by the data source.
#[derive(Copy, Clone, Debug, PartialEq, Resource)]
pub struct SkyState {
    /// Unit vector toward the sun in the sim frame (x east, y north, z up).
    pub sun_direction: Vec3,
    /// Global horizontal irradiance, W m⁻².
    pub global: f32,
    /// Cloud fraction, 0 to 1.
    pub cloud_fraction: f32,
}

impl Default for SkyState {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::new(0.3, -0.6, 0.75).normalize(),
            global: 700.0,
            cloud_fraction: 0.2,
        }
    }
}

/// Illuminance per W m⁻² of global irradiance. Real daylight is about 110 lx per W m⁻²;
/// this is scaled down to suit the default camera exposure.
const LUX_PER_W: f32 = 14.0;

/// Marker for the sun.
#[derive(Component)]
pub struct Sun;

pub(crate) fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun,
    ));
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Sky color for a sun elevation sine and cloud fraction, linear RGB.
#[must_use]
pub fn sky_color(sin_elevation: f32, cloud: f32) -> [f32; 3] {
    let night = [0.01, 0.012, 0.03];
    let dusk = [0.35, 0.2, 0.15];
    let day = [0.35, 0.55, 0.85];
    let grey = [0.45, 0.48, 0.52];
    let lit = if sin_elevation < 0.0 {
        mix(dusk, night, -sin_elevation * 8.0)
    } else {
        mix(dusk, day, sin_elevation * 4.0)
    };
    let overcast = mix(
        lit,
        mix(night, grey, sin_elevation * 4.0 + 0.1),
        cloud * 0.8,
    );
    if sin_elevation < 0.0 { lit } else { overcast }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sky(
    sky: Res<SkyState>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut suns: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
) {
    let dir = sky.sun_direction.normalize_or(Vec3::Z);
    let sin_e = dir.z;
    let c = sky_color(sin_e, sky.cloud_fraction);
    clear.0 = Color::linear_rgb(c[0], c[1], c[2]);
    let daylight = (sky.global / 800.0).clamp(0.0, 1.2);
    ambient.color = Color::linear_rgb(0.55 + 0.4 * c[0], 0.6 + 0.3 * c[1], 0.7 + 0.3 * c[2]);
    ambient.brightness = 30.0 + 420.0 * daylight;
    for (mut light, mut t) in &mut suns {
        // Direct light only; clouds turn it into ambient.
        let direct = sky.global * (1.0 - 0.75 * sky.cloud_fraction);
        light.illuminance = if sin_e > 0.0 { direct * LUX_PER_W } else { 0.0 };
        light.color = if sin_e < 0.25 {
            Color::srgb(1.0, 0.8, 0.6)
        } else {
            Color::srgb(1.0, 0.97, 0.92)
        };
        let toward = sim_to_bevy(dir);
        let up = if toward.y.abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        *t = Transform::from_translation(toward * 5.0).looking_at(Vec3::ZERO, up);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_is_dark_and_noon_is_blue() {
        let n = sky_color(-0.5, 0.0);
        let d = sky_color(0.9, 0.0);
        assert!(n.iter().all(|&v| v < 0.05));
        assert!(d[2] > d[0] && d[2] > 0.5);
        let overcast = sky_color(0.9, 1.0);
        assert!((overcast[2] - overcast[0]) < (d[2] - d[0]));
    }
}
