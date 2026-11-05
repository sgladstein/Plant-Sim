//! The diorama renderer (design plan §2.1, §4, §5.2; WP15, WP18 groundwork).
//!
//! [`PlantViewPlugin`] draws whatever the [`Garden`] resource describes: pots with a
//! glass-fronted soil slab, stems and roots as tubes, leaves and fruit as instanced
//! templates, colored by the active [`Lens`], under a sun and sky from [`SkyState`], seen
//! through an auto-framing orbit [`CameraRig`].
//!
//! The plugin only reads. A data source (the demo driver in ps-app now, the sim thread's
//! snapshot later) writes `Garden` and `SkyState`; the UI writes `Lens` and the camera
//! rig's flags. Sim state never lives in the ECS (§11.10).

// Geometry and colors are f32 for the GPU; converting counts and sim values is intended.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

pub mod camera;
pub mod diorama;
pub mod garden;
pub mod lens;
pub mod look;
pub mod scene;
pub mod sky;

use bevy::prelude::*;

pub use camera::CameraRig;
pub use garden::{Garden, PotSize, PotView, SoilFace};
pub use lens::{Lens, LensInfo, Origin, Scale};
pub use sky::SkyState;

/// System sets, in order, so a data source can run before the scene sync.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewSet {
    /// Fill `Garden` and `SkyState` (the data source's systems go here).
    Source,
    /// Turn the garden into entities.
    Sync,
    /// Camera and sky.
    Present,
}

/// The diorama renderer.
pub struct PlantViewPlugin;

impl Plugin for PlantViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Garden>()
            .init_resource::<SkyState>()
            .init_resource::<Lens>()
            .init_resource::<CameraRig>()
            .init_resource::<scene::SceneIndex>()
            .insert_resource(ClearColor(Color::linear_rgb(
                look::HORIZON[0],
                look::HORIZON[1],
                look::HORIZON[2],
            )))
            .configure_sets(
                Update,
                (ViewSet::Source, ViewSet::Sync, ViewSet::Present).chain(),
            )
            .add_systems(
                Startup,
                (
                    scene::setup_scene,
                    (look::setup_sky_images, camera::spawn_camera).chain(),
                    sky::spawn_sun,
                ),
            )
            .add_systems(Update, scene::sync_scene.in_set(ViewSet::Sync))
            .add_systems(
                Update,
                (
                    camera::orbit_input,
                    camera::apply_rig,
                    camera::apply_insets,
                    sky::update_sky,
                )
                    .chain()
                    .in_set(ViewSet::Present),
            );
    }
}
