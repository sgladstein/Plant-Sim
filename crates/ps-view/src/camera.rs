//! Orbit camera with auto-framing (design plan §2.1).
//!
//! Left drag orbits, right or middle drag pans, the wheel zooms. By default the camera
//! frames the bounding box of the pots and plants, easing as the plants grow (they grow
//! about a thousandfold); any manual move switches auto-framing off until reset (F key
//! or the UI's reset button).

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

/// The orbit camera's state, in Bevy world space (y up).
#[derive(Clone, Debug, PartialEq, Resource)]
pub struct CameraRig {
    /// Point looked at.
    pub target: Vec3,
    /// Rotation about world up, rad (0 looks north from the south).
    pub yaw: f32,
    /// Elevation of the camera above the target's horizontal plane, rad.
    pub pitch: f32,
    /// Distance from target, m.
    pub distance: f32,
    /// True while the camera follows the scene bounds.
    pub auto_frame: bool,
    /// The bounds to frame, set by the scene each frame (min, max).
    pub frame_bounds: Option<(Vec3, Vec3)>,
    /// Set by the UI while the pointer is over a panel, so drags there do not orbit.
    pub input_blocked: bool,
    /// Request to return to auto-framing.
    pub reset_requested: bool,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            target: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.28,
            distance: 1.2,
            auto_frame: true,
            frame_bounds: None,
            input_blocked: false,
            reset_requested: false,
        }
    }
}

/// Vertical field of view, rad.
pub const FOV_Y: f32 = 0.7;

impl CameraRig {
    /// Camera position for the current yaw, pitch and distance.
    #[must_use]
    pub fn eye(&self) -> Vec3 {
        let horizontal = self.distance * self.pitch.cos();
        self.target
            + Vec3::new(
                horizontal * self.yaw.sin(),
                self.distance * self.pitch.sin(),
                horizontal * self.yaw.cos(),
            )
    }

    /// Target and distance that frame `bounds` with some margin.
    #[must_use]
    pub fn framing(bounds: (Vec3, Vec3)) -> (Vec3, f32) {
        let (lo, hi) = bounds;
        let center = 0.5 * (lo + hi);
        let radius = 0.5 * (hi - lo).length().max(0.05);
        let distance = 1.15 * radius / (0.5 * FOV_Y).sin();
        (center, distance)
    }
}

/// Marker for the orbit camera entity.
#[derive(Component)]
pub struct OrbitCamera;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_camera(mut commands: Commands, rig: Res<CameraRig>) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV_Y,
            near: 0.005,
            far: 100.0,
            ..default()
        }),
        Transform::from_translation(rig.eye()).looking_at(rig.target, Vec3::Y),
        OrbitCamera,
    ));
}

/// Reads the mouse and keyboard into the rig.
#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are passed by value.
pub(crate) fn orbit_input(
    mut rig: ResMut<CameraRig>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
) {
    if keys.just_pressed(KeyCode::KeyF) {
        rig.reset_requested = true;
    }
    if rig.reset_requested {
        rig.reset_requested = false;
        rig.auto_frame = true;
        rig.yaw = 0.0;
        rig.pitch = 0.28;
    }
    if rig.input_blocked {
        return;
    }
    let delta = motion.delta;
    if buttons.pressed(MouseButton::Left) && delta != Vec2::ZERO {
        rig.yaw -= delta.x * 0.006;
        rig.pitch = (rig.pitch + delta.y * 0.006).clamp(-0.2, 1.45);
    }
    if (buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle))
        && delta != Vec2::ZERO
    {
        let yaw = Quat::from_rotation_y(rig.yaw);
        let right = yaw * Vec3::X;
        let up = Vec3::Y;
        let k = rig.distance * 0.0015;
        rig.target += (-right * delta.x + up * delta.y) * k;
        rig.auto_frame = false;
    }
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    if lines != 0.0 {
        rig.distance = (rig.distance * (1.0 - 0.1 * lines)).clamp(0.03, 20.0);
        rig.auto_frame = false;
    }
}

/// Eases toward the auto-frame and writes the camera transform.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn apply_rig(
    mut rig: ResMut<CameraRig>,
    time: Res<Time>,
    mut cameras: Query<&mut Transform, With<OrbitCamera>>,
) {
    if rig.auto_frame
        && let Some(bounds) = rig.frame_bounds
    {
        let (target, distance) = CameraRig::framing(bounds);
        let k = 1.0 - (-3.0 * time.delta_secs()).exp();
        rig.target = rig.target.lerp(target, k);
        rig.distance += (distance - rig.distance) * k;
    }
    for mut t in &mut cameras {
        *t = Transform::from_translation(rig.eye()).looking_at(rig.target, Vec3::Y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_contains_the_box() {
        let (c, d) = CameraRig::framing((Vec3::splat(-0.5), Vec3::splat(0.5)));
        assert!(c.length() < 1e-6);
        let rig = CameraRig {
            target: c,
            distance: d,
            ..CameraRig::default()
        };
        // The bounding sphere fits inside the vertical field of view.
        let radius = 0.5 * Vec3::ONE.length();
        assert!(radius / rig.distance < (0.5 * FOV_Y).sin());
    }

    #[test]
    fn eye_is_south_of_target_at_zero_yaw() {
        let rig = CameraRig::default();
        let e = rig.eye();
        assert!(e.z > rig.target.z && e.y > rig.target.y);
        assert!((e.distance(rig.target) - rig.distance).abs() < 1e-5);
    }
}
