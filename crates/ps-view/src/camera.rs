//! Orbit camera with auto-framing (design plan §2.1).
//!
//! Left drag orbits, right or middle drag pans, the wheel zooms. By default the camera
//! frames the bounding box of the pots and plants, easing as the plants grow (they grow
//! about a thousandfold); any manual move switches auto-framing off until reset (F key
//! or the UI's reset button).

use bevy::camera::SubCameraView;
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
    /// Auto-framing frames only this pot's plant (a close-up), when set.
    pub focus: Option<usize>,
    /// Auto-framing distance multiplier (below 1 moves in), for close-up shots.
    pub zoom: f32,
    /// A further distance factor from the flowering cue (`moments`), 1 when none.
    pub cue_zoom: f32,
    /// Screen space covered by UI panels, logical pixels: (left, top, right, bottom).
    /// The view is shifted off-center so the scene centers in the uncovered area.
    pub insets: [f32; 4],
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
            focus: None,
            zoom: 1.0,
            cue_zoom: 1.0,
            insets: [0.0; 4],
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

    /// Target and distance that fit `bounds` in a view whose visible half-angles have
    /// tangents `(tan_h, tan_v)` (see [`visible_tans`]), seen from `yaw` and `pitch`, with
    /// some margin: each corner of the box is kept inside both the vertical and the
    /// horizontal field of view. Tighter than [`CameraRig::framing`] for a wide, low
    /// garden on a wide screen, where a bounding sphere leaves most of the frame empty.
    #[must_use]
    pub fn framing_fit(
        bounds: (Vec3, Vec3),
        yaw: f32,
        pitch: f32,
        (tan_h, tan_v): (f32, f32),
    ) -> (Vec3, f32) {
        let (lo, hi) = bounds;
        let center = 0.5 * (lo + hi);
        let back = Vec3::new(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        );
        let right = Vec3::Y.cross(back).normalize_or(Vec3::X);
        let up = back.cross(right);
        let full = (0.5 * FOV_Y).tan();
        let sane = |t: f32| if t.is_finite() && t > 0.01 { t } else { full };
        let (tan_h, tan_v) = (sane(tan_h), sane(tan_v));
        let mut distance: f32 = 0.05;
        for k in 0..8_u8 {
            let corner = Vec3::new(
                if k & 1 == 0 { lo.x } else { hi.x },
                if k & 2 == 0 { lo.y } else { hi.y },
                if k & 4 == 0 { lo.z } else { hi.z },
            ) - center;
            let near = corner.dot(back);
            distance = distance
                .max(near + corner.dot(right).abs() / tan_h)
                .max(near + corner.dot(up).abs() / tan_v);
        }
        (center, FIT_MARGIN * distance)
    }
}

/// Room left around the framed box by [`CameraRig::framing_fit`] (a factor on distance).
pub const FIT_MARGIN: f32 = 1.15;

/// Marker for the orbit camera entity.
#[derive(Component)]
pub struct OrbitCamera;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_camera(
    mut commands: Commands,
    rig: Res<CameraRig>,
    sky: Res<crate::look::SkyImages>,
) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV_Y,
            near: 0.005,
            far: 100.0,
            ..default()
        }),
        Transform::from_translation(rig.eye()).looking_at(rig.target, Vec3::Y),
        Msaa::Sample4,
        crate::look::camera_bundle(&sky),
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
        rig.distance = (rig.distance * (1.0 - 0.1 * lines)).clamp(0.03, 40.0);
        rig.auto_frame = false;
    }
}

/// The orbit camera's parts that [`apply_rig`] writes or reads.
type RigCamera = (
    &'static mut Transform,
    Option<&'static mut bevy::pbr::DistanceFog>,
    Option<&'static Projection>,
);

/// Eases toward the auto-frame and writes the camera transform.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn apply_rig(
    mut rig: ResMut<CameraRig>,
    time: Res<Time>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<RigCamera, With<OrbitCamera>>,
) {
    let tans = windows.single().map_or_else(
        |_| visible_tans(Vec2::new(16.0, 9.0), [0.0; 4]),
        |w| visible_tans(Vec2::new(w.width(), w.height()), rig.insets),
    );
    if rig.auto_frame
        && let Some(bounds) = rig.frame_bounds
    {
        let (target, distance) = CameraRig::framing_fit(bounds, rig.yaw, rig.pitch, tans);
        let distance = distance * rig.zoom * rig.cue_zoom;
        let k = 1.0 - (-3.0 * time.delta_secs()).exp();
        rig.target = rig.target.lerp(target, k);
        rig.distance += (distance - rig.distance) * k;
    }
    for (mut t, fog, _) in &mut cameras {
        *t = Transform::from_translation(rig.eye()).looking_at(rig.target, Vec3::Y);
        // The haze starts behind the scene however far out the camera is (a wide
        // garden frames from beyond the default fog's end).
        if let Some(mut fog) = fog {
            fog.falloff = fog_falloff(rig.distance);
        }
    }
}

/// Linear haze from just behind the framed scene to well beyond it: about 2 to 60 m for
/// a camera within 1.5 m (so the tree line reads as soft shapes), pushed back with the
/// camera.
#[must_use]
pub fn fog_falloff(distance: f32) -> bevy::pbr::FogFalloff {
    let d = distance.max(1.5);
    bevy::pbr::FogFalloff::Linear {
        start: d + 0.5,
        end: d + 58.5,
    }
}

/// Tangents of the half-angles `(horizontal, vertical)` of the part of the view that
/// the UI panels leave uncovered, for a window of `size` with panel `insets` (left, top,
/// right, bottom) in the same units. [`off_center`] renders a larger image of which the
/// window is a crop, so the uncovered area sees less than the full field of view.
#[must_use]
pub fn visible_tans(size: Vec2, insets: [f32; 4]) -> (f32, f32) {
    let [l, t, r, b] = insets.map(|v| v.max(0.0));
    let (full, _) = off_center(size, [l, t, r, b]);
    let free = Vec2::new(size.x - l - r, size.y - t - b).max(Vec2::splat(1.0));
    // Pixels per unit tangent: the full image's height spans the vertical field of view.
    let per_tan = 0.5 * full.y.max(1.0) / (0.5 * FOV_Y).tan();
    (0.5 * free.x / per_tan, 0.5 * free.y / per_tan)
}

/// The off-center view that puts the center of the uncovered area at the image center:
/// `(full size, offset)` for a window of `size` with panel `insets` (left, top, right,
/// bottom), all in the same pixel units.
#[must_use]
pub fn off_center(size: Vec2, insets: [f32; 4]) -> (Vec2, Vec2) {
    let [l, t, r, b] = insets.map(|v| v.max(0.0));
    let c = Vec2::new((l + (size.x - r)) * 0.5, (t + (size.y - b)) * 0.5).clamp(Vec2::ZERO, size);
    let full = Vec2::new(2.0 * c.x.max(size.x - c.x), 2.0 * c.y.max(size.y - c.y));
    (full, full * 0.5 - c)
}

/// Shifts the projection so the scene centers between the UI panels.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn apply_insets(
    rig: Res<CameraRig>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<OrbitCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.scale_factor();
    let size = Vec2::new(window.width(), window.height()) * scale;
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }
    let (full, offset) = off_center(size, rig.insets.map(|v| v * scale));
    let view = (full.x > size.x + 0.5 || full.y > size.y + 0.5).then(|| SubCameraView {
        full_size: full.round().as_uvec2(),
        offset: offset.round(),
        size: size.round().as_uvec2(),
    });
    for mut camera in &mut cameras {
        if camera.sub_camera_view != view {
            camera.sub_camera_view = view;
        }
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
    fn a_fitted_frame_keeps_every_corner_in_view_and_is_tighter_for_wide_scenes() {
        let bounds = (Vec3::new(-0.45, -0.2, -0.1), Vec3::new(0.45, 0.15, 0.1));
        let (yaw, pitch, aspect) = (0.0, 0.28, 16.0 / 9.0);
        let tans = visible_tans(Vec2::new(1600.0, 900.0), [0.0; 4]);
        assert!((tans.0 / tans.1 - aspect).abs() < 1e-4);
        assert!((tans.1 - (0.5 * FOV_Y).tan()).abs() < 1e-5);
        let (c, d) = CameraRig::framing_fit(bounds, yaw, pitch, tans);
        let rig = CameraRig {
            target: c,
            distance: d,
            yaw,
            pitch,
            ..CameraRig::default()
        };
        let eye = rig.eye();
        let fwd = (c - eye).normalize();
        let right = fwd.cross(Vec3::Y).normalize();
        let up = right.cross(fwd);
        let (tv, th) = ((0.5 * FOV_Y).tan(), (0.5 * FOV_Y).tan() * aspect);
        for k in 0..8_u8 {
            let p = Vec3::new(
                if k & 1 == 0 { -0.45 } else { 0.45 },
                if k & 2 == 0 { -0.2 } else { 0.15 },
                if k & 4 == 0 { -0.1 } else { 0.1 },
            ) - eye;
            let z = p.dot(fwd);
            assert!(p.dot(right).abs() <= z * th + 1e-4 && p.dot(up).abs() <= z * tv + 1e-4);
        }
        assert!(d < CameraRig::framing(bounds).1 * 0.8);
        // Panels over the right and top of the window leave less to fit into, so the
        // camera backs off.
        let covered = visible_tans(Vec2::new(1280.0, 720.0), [0.0, 78.0, 300.0, 22.0]);
        assert!(covered.0 < tans.0 * 0.8 && covered.1 < tans.1);
        assert!(CameraRig::framing_fit(bounds, yaw, pitch, covered).1 > d * 1.2);
    }

    #[test]
    fn off_center_centers_the_free_area() {
        let size = Vec2::new(1600.0, 900.0);
        let (full, offset) = off_center(size, [0.0, 80.0, 300.0, 20.0]);
        // The free area's center (650, 480) lands at the center of the full view.
        let center_in_full = Vec2::new(650.0, 480.0) + offset;
        assert!((center_in_full - full * 0.5).length() < 1e-3);
        assert!(offset.x >= 0.0 && offset.y >= 0.0);
        assert!(offset.x + size.x <= full.x + 1e-3 && offset.y + size.y <= full.y + 1e-3);
        let (f0, o0) = off_center(size, [0.0; 4]);
        assert_eq!((f0, o0), (size, Vec2::ZERO));
    }

    #[test]
    fn eye_is_south_of_target_at_zero_yaw() {
        let rig = CameraRig::default();
        let e = rig.eye();
        assert!(e.z > rig.target.z && e.y > rig.target.y);
        assert!((e.distance(rig.target) - rig.distance).abs() < 1e-5);
    }
}
