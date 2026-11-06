//! Engine-free plant geometry (design plan §4, §10.5; WP14).
//!
//! This crate turns an organ graph into triangle meshes and instance transforms that a
//! renderer can upload as they are. It knows nothing about Bevy or the GPU, so every
//! piece of geometry is tested headlessly.
//!
//! The pipeline, per plant and per frame:
//!
//! 1. [`Skeleton::from_arena`] places every organ: a base point, a unit direction, and the
//!    rendered length, radius and area from the sim's allometry. [`Skeleton::lerp`]
//!    blends two hourly snapshots so growth is smooth at fractional sim time.
//! 2. [`axes`] groups internodes (and root segments) into axes, and [`axis_mesh`] sweeps
//!    one tapered tube along each, with smooth joins at the nodes. [`AxisSignature`]
//!    decides when an axis has changed enough (2%, §4) to be worth rebuilding.
//! 3. [`instances`] gives one [`Instance`] per leaf, cotyledon, flower, pod, seed and bud:
//!    a template ([`Template`]) plus a translation, rotation and scale, so leaves can be
//!    drawn GPU-instanced from a handful of shared meshes built by [`build_template`].
//! 4. [`pose`] adds the render-only moments (WP20): the emerging hook, wilting and the
//!    lean toward the light.
//!
//! Coordinates are the sim's frame: x east, y north, z up, in meters. A renderer with a
//! different up axis applies one root transform.
//!
//! [`demo`] grows a stand-in bush bean from the organ graph API alone, for tests and for
//! the viewer until ps-world exists. It is not the model.

// Geometry is f32 by design: GPU vertex formats are f32, and the sim's f64 state is read,
// never written. Converting sim f64 and counts to f32 is the point of this crate.
#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

pub mod axes;
pub mod demo;
pub mod instances;
pub mod mesh;
pub mod moments;
pub mod skeleton;
pub mod templates;
pub mod tube;

pub use axes::{
    Axis, AxisKind, AxisSignature, AxisStyle, GOLDEN_ANGLE, axes, axis_mesh, axis_nodes,
    copy_transforms, nodes_after_base,
};
pub use instances::{
    BLADE_TILT, COTYLEDON_SCALE, FLOWER_LENGTH, Instance, InstanceOptions, LEAF_JITTER,
    droop_level, flower_length, instances, jitter, leaf_form, pulvinus, unfold_level,
    wilt_droop_level,
};
pub use mesh::MeshData;
pub use moments::{
    Pose, lean_from_light, own_lean, pose, residual_lean, turgor_step, wilt_of_stress,
};
pub use skeleton::{Coil, Container, Hook, MAX_COPIES, Placement, Skeleton};
pub use templates::{
    DROOP_LEVELS, Template, TemplateKey, UNFOLD_LEVELS, build_template, build_template_shaded,
};
pub use tube::{TubeNode, tube};

#[cfg(test)]
mod tests;
