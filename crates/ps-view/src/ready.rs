//! Cold-start readiness: a loading screen until the GPU pipelines are compiled.
//!
//! On a first run, shader pipelines compile in the background for several seconds (longer
//! on DX12), and until they are ready the scene renders as sky only. The render world
//! counts waiting pipelines into shared atomics each frame; the main world declares the
//! scene [`Ready`] once nothing has been waiting for [`SETTLE_FRAMES`] frames, hides the
//! loading overlay, and lets screenshots proceed.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::prelude::*;
use bevy::render::render_resource::PipelineCache;
use bevy::render::{Render, RenderApp, RenderSystems};

/// Frames with no pipeline waiting before the scene counts as ready.
pub const SETTLE_FRAMES: u32 = 20;

/// Pipeline counts shared between the render world (writer) and the main world (reader).
#[derive(Clone, Default, Resource)]
struct PipelineCounts {
    waiting: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
}

/// Whether the scene has finished compiling its pipelines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
pub struct Ready {
    /// True once pipelines have settled; stays true afterwards.
    pub ready: bool,
    /// Consecutive frames with nothing waiting.
    pub quiet_frames: u32,
    /// Pipelines still compiling in the last frame seen.
    pub waiting: usize,
}

#[derive(Component)]
struct LoadingOverlay;

/// Adds the readiness tracking and the loading overlay.
pub(crate) fn build(app: &mut App) {
    let counts = PipelineCounts::default();
    app.insert_resource(counts.clone())
        .init_resource::<Ready>()
        .add_systems(Startup, spawn_overlay)
        .add_systems(Update, (update_ready, hide_overlay).chain());
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render
            .insert_resource(counts)
            .add_systems(Render, count_pipelines.in_set(RenderSystems::Cleanup));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn count_pipelines(cache: Res<PipelineCache>, counts: Res<PipelineCounts>) {
    counts
        .waiting
        .store(cache.waiting_pipelines().count(), Ordering::Relaxed);
    counts
        .total
        .store(cache.pipelines().count(), Ordering::Relaxed);
}

/// Advances the readiness state for one frame.
#[must_use]
pub fn step(mut state: Ready, waiting: usize, total: usize) -> Ready {
    state.waiting = waiting;
    if waiting == 0 && total > 0 {
        state.quiet_frames = state.quiet_frames.saturating_add(1);
    } else {
        state.quiet_frames = 0;
    }
    if state.quiet_frames >= SETTLE_FRAMES {
        state.ready = true;
    }
    state
}

#[allow(clippy::needless_pass_by_value)]
fn update_ready(counts: Res<PipelineCounts>, mut ready: ResMut<Ready>) {
    let next = step(
        *ready,
        counts.waiting.load(Ordering::Relaxed),
        counts.total.load(Ordering::Relaxed),
    );
    if next != *ready {
        *ready = next;
    }
}

fn spawn_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.13, 0.12, 0.11)),
            GlobalZIndex(100),
            LoadingOverlay,
        ))
        .with_child((
            Text::new("Preparing shaders…"),
            TextColor(Color::srgb(0.85, 0.88, 0.8)),
        ));
}

#[allow(clippy::needless_pass_by_value)]
fn hide_overlay(
    ready: Res<Ready>,
    mut commands: Commands,
    overlays: Query<Entity, With<LoadingOverlay>>,
) {
    if ready.ready {
        for e in &overlays {
            commands.entity(e).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_after_quiet_frames_and_stays_ready() {
        let mut s = Ready::default();
        for _ in 0..SETTLE_FRAMES - 1 {
            s = step(s, 0, 10);
        }
        assert!(!s.ready);
        s = step(s, 3, 12);
        assert_eq!(s.quiet_frames, 0);
        for _ in 0..SETTLE_FRAMES {
            s = step(s, 0, 12);
        }
        assert!(s.ready);
        // A lens switch compiling new pipelines later does not bring the overlay back.
        s = step(s, 2, 14);
        assert!(s.ready);
    }

    #[test]
    fn nothing_compiled_is_not_ready() {
        let mut s = Ready::default();
        for _ in 0..100 {
            s = step(s, 0, 0);
        }
        assert!(!s.ready);
    }
}
