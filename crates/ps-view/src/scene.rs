//! Scene sync: turns the read-only [`Garden`] into entities, every frame.
//!
//! - One entity per stem or root axis, keyed by its basal organ id. Its tube mesh is
//!   rebuilt only when [`AxisSignature::needs_rebuild`] says it changed by more than 2%
//!   (§4); lens and age colors are vertex colors, rewritten without touching geometry.
//! - One entity per templated organ (leaf, flower, pod...), keyed by organ id, drawing a
//!   shared template mesh with a shared material per quantized color, so Bevy batches
//!   identical mesh-material pairs (§11.9).
//! - Per pot: soil, walls, glass and the soil water overlay (§4).
//!
//! Everything hangs under one root entity that turns the sim frame (z up) into Bevy's
//! (y up), so all plant geometry stays in sim coordinates.

use std::collections::{BTreeMap, HashMap};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use ps_graph::{OrganId, OrganKind};
use ps_mesh::{
    AxisKind, AxisSignature, AxisStyle, InstanceOptions, MeshData, Placement, TemplateKey, axes,
    axis_mesh, build_template, instances,
};

use crate::camera::CameraRig;
use crate::diorama::{self, ColoredMesh};
use crate::garden::{Garden, PotSize, PotView, natural_color};
use crate::lens::Lens;

/// Rebuild threshold for stem and root meshes (§4).
pub const REBUILD_THRESHOLD: f32 = 0.02;

/// Rotation from the sim frame (x east, y north, z up) to Bevy's (x east, y up, z south).
#[must_use]
pub fn sim_to_bevy_rotation() -> Quat {
    Quat::from_rotation_x(-core::f32::consts::FRAC_PI_2)
}

/// A sim-frame point in Bevy coordinates.
#[must_use]
pub fn sim_to_bevy(p: Vec3) -> Vec3 {
    Vec3::new(p.x, p.z, -p.y)
}

/// Converts engine-free geometry into a Bevy mesh.
#[must_use]
pub fn to_bevy_mesh(m: &MeshData, colors: Option<&[[f32; 4]]>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, m.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, m.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, m.uvs.clone());
    if let Some(c) = colors {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, c.to_vec());
    }
    mesh.insert_indices(Indices::U32(m.indices.clone()));
    mesh
}

fn colored(m: &ColoredMesh) -> Mesh {
    to_bevy_mesh(&m.mesh, Some(&m.colors))
}

/// Shared meshes and materials.
#[derive(Resource)]
pub struct SceneAssets {
    templates: BTreeMap<TemplateKey, Handle<Mesh>>,
    vertex_color: Handle<StandardMaterial>,
    soil: Handle<StandardMaterial>,
    glass: Handle<StandardMaterial>,
    overlay: Handle<StandardMaterial>,
    palette: HashMap<[u8; 3], Handle<StandardMaterial>>,
}

impl SceneAssets {
    fn material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        rgb: [f32; 3],
    ) -> (Handle<StandardMaterial>, [u8; 3]) {
        let key = rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
        let handle = self
            .palette
            .entry(key)
            .or_insert_with(|| {
                let c = key.map(|v| f32::from(v) / 255.0);
                materials.add(StandardMaterial {
                    base_color: Color::linear_rgb(c[0], c[1], c[2]),
                    perceptual_roughness: 0.65,
                    reflectance: 0.3,
                    ..default()
                })
            })
            .clone();
        (handle, key)
    }
}

/// The root entity that maps sim coordinates to Bevy's.
#[derive(Component)]
pub struct GardenRoot;

struct AxisEntity {
    entity: Entity,
    mesh: Handle<Mesh>,
    signature: AxisSignature,
    organs: Vec<OrganId>,
    style: AxisStyle,
    colors: Vec<[f32; 4]>,
}

struct InstanceEntity {
    entity: Entity,
    key: TemplateKey,
    color: [u8; 3],
}

struct PotScene {
    entity: Entity,
    size: PotSize,
    axes: BTreeMap<OrganId, AxisEntity>,
    instances: BTreeMap<OrganId, InstanceEntity>,
    overlay: Entity,
    overlay_mesh: Handle<Mesh>,
    overlay_state: Option<Vec<f32>>,
}

/// Entities created for the garden.
#[derive(Resource, Default)]
pub struct SceneIndex {
    root: Option<Entity>,
    pots: Vec<PotScene>,
    lens: Option<Lens>,
}

pub(crate) fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut index: ResMut<SceneIndex>,
) {
    let templates = TemplateKey::all()
        .into_iter()
        .map(|k| (k, meshes.add(to_bevy_mesh(&build_template(k), None))))
        .collect();
    let vertex_color = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.75,
        reflectance: 0.25,
        ..default()
    });
    let soil = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        reflectance: 0.1,
        ..default()
    });
    let glass = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.85, 0.95, 1.0, 0.05),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        reflectance: 0.15,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    let overlay = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(SceneAssets {
        templates,
        vertex_color,
        soil,
        glass,
        overlay,
        palette: HashMap::new(),
    });
    let root = commands
        .spawn((
            GardenRoot,
            Transform::from_rotation(sim_to_bevy_rotation()),
            Visibility::default(),
        ))
        .id();
    // A table under the pots.
    let table = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.5, 0.42),
        perceptual_roughness: 0.9,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(3.0, 2.0))),
        MeshMaterial3d(table),
        Transform::from_xyz(0.0, -PotSize::RHIZOBOX.soil_depth - 0.042, 0.0),
    ));
    index.root = Some(root);
}

fn spawn_pot(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &SceneAssets,
    root: Entity,
    pot: &PotView,
) -> PotScene {
    let entity = commands
        .spawn((
            Transform::from_xyz(pot.center[0], pot.center[1], 0.0),
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    let size = pot.size;
    commands.spawn((
        Mesh3d(meshes.add(colored(&diorama::soil(size)))),
        MeshMaterial3d(assets.soil.clone()),
        Transform::default(),
        ChildOf(entity),
    ));
    commands.spawn((
        Mesh3d(meshes.add(colored(&diorama::frame(size)))),
        MeshMaterial3d(assets.vertex_color.clone()),
        Transform::default(),
        ChildOf(entity),
    ));
    commands.spawn((
        Mesh3d(meshes.add(to_bevy_mesh(&diorama::glass(size), None))),
        MeshMaterial3d(assets.glass.clone()),
        Transform::default(),
        ChildOf(entity),
    ));
    let overlay_mesh = meshes.add(colored(&ColoredMesh::default()));
    let overlay = commands
        .spawn((
            Mesh3d(overlay_mesh.clone()),
            MeshMaterial3d(assets.overlay.clone()),
            Transform::default(),
            Visibility::Hidden,
            ChildOf(entity),
        ))
        .id();
    PotScene {
        entity,
        size,
        axes: BTreeMap::new(),
        instances: BTreeMap::new(),
        overlay,
        overlay_mesh,
        overlay_state: None,
    }
}

/// The color of organ `i` of `pot` under `lens`.
fn organ_color(pot: &PotView, lens: Lens, i: usize, p: &Placement) -> [f32; 3] {
    let natural = natural_color(p);
    let Some(info) = lens.info() else {
        return natural;
    };
    if !lens.colors_organs() {
        return natural;
    }
    if lens == Lens::RootsXray && p.kind != OrganKind::Root {
        return natural;
    }
    // Organs the lens says nothing about fade to gray so the colored ones read.
    pot.organ_value(lens, i)
        .and_then(|v| info.color(v))
        .unwrap_or_else(|| {
            let g = 0.3 * natural[0] + 0.6 * natural[1] + 0.1 * natural[2];
            [0.5 * g + 0.12; 3]
        })
}

fn axis_colors(pot: &PotView, lens: Lens, organs: &[OrganId], style: AxisStyle) -> Vec<[f32; 4]> {
    let ring = style.sides as usize + 1;
    let mut out = Vec::with_capacity((organs.len() + 1) * ring);
    for k in 0..=organs.len() {
        let id = organs[k.saturating_sub(1).min(organs.len() - 1)];
        let c = pot
            .skeleton
            .get(id)
            .map_or([1.0; 3], |p| organ_color(pot, lens, id.index(), p));
        out.extend(std::iter::repeat_n([c[0], c[1], c[2], 1.0], ring));
    }
    out
}

/// Spawns, updates and despawns entities to match the garden.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
pub(crate) fn sync_scene(
    mut commands: Commands,
    garden: Res<Garden>,
    lens: Res<Lens>,
    mut index: ResMut<SceneIndex>,
    mut assets: ResMut<SceneAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rig: ResMut<CameraRig>,
) {
    let Some(root) = index.root else {
        return;
    };
    let lens = *lens;
    // Pots are static; respawn everything if their number or sizes change.
    let layout_changed = index.pots.len() != garden.pots.len()
        || index
            .pots
            .iter()
            .zip(&garden.pots)
            .any(|(s, p)| s.size != p.size);
    if layout_changed {
        for p in index.pots.drain(..) {
            commands.entity(p.entity).despawn();
        }
        for pot in &garden.pots {
            let scene = spawn_pot(&mut commands, &mut meshes, &assets, root, pot);
            index.pots.push(scene);
        }
    }

    // Soil opacity follows the lens.
    if index.lens != Some(lens) {
        if let Some(mut m) = materials.get_mut(&assets.soil) {
            let a = lens.soil_opacity();
            m.base_color = Color::linear_rgba(1.0, 1.0, 1.0, a);
            m.alpha_mode = if a < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
        }
        index.lens = Some(lens);
    }

    let mut bounds: Option<(Vec3, Vec3)> = None;
    let mut grow = |p: Vec3| {
        let b = sim_to_bevy(p);
        bounds = Some(match bounds {
            None => (b, b),
            Some((lo, hi)) => (lo.min(b), hi.max(b)),
        });
    };

    for (scene, pot) in index.pots.iter_mut().zip(&garden.pots) {
        let c = Vec3::new(pot.center[0], pot.center[1], 0.0);
        let half = Vec3::new(pot.size.width / 2.0, pot.size.depth / 2.0, 0.0);
        grow(c - half - Vec3::Z * pot.size.soil_depth);
        grow(c + half);
        if let Some((lo, hi)) = pot.skeleton.bounds() {
            // Frame the shoot; roots are framed by the pot.
            grow(c + lo.max(Vec3::new(lo.x, lo.y, 0.0)));
            grow(c + hi);
        }

        // Axes.
        let current = axes(&pot.skeleton, true);
        let mut seen = Vec::with_capacity(current.len());
        for axis in &current {
            let style = match axis.kind {
                AxisKind::Shoot => AxisStyle::STEM,
                AxisKind::Root => AxisStyle::ROOT,
            };
            let key = axis.key();
            seen.push(key);
            let signature = AxisSignature::of(&pot.skeleton, axis, style);
            let colors = axis_colors(pot, lens, &axis.organs, style);
            match scene.axes.get_mut(&key) {
                Some(existing)
                    if !signature.needs_rebuild(&existing.signature, REBUILD_THRESHOLD)
                        && existing.organs == axis.organs =>
                {
                    if existing.colors != colors {
                        if let Some(mut m) = meshes.get_mut(&existing.mesh) {
                            m.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
                        }
                        existing.colors = colors;
                    }
                }
                Some(existing) => {
                    let mesh = axis_mesh(&pot.skeleton, axis, style);
                    let _ = meshes.insert(&existing.mesh, to_bevy_mesh(&mesh, Some(&colors)));
                    existing.signature = signature;
                    existing.organs.clone_from(&axis.organs);
                    existing.style = style;
                    existing.colors = colors;
                }
                None => {
                    let mesh = axis_mesh(&pot.skeleton, axis, style);
                    let handle = meshes.add(to_bevy_mesh(&mesh, Some(&colors)));
                    let entity = commands
                        .spawn((
                            Mesh3d(handle.clone()),
                            MeshMaterial3d(assets.vertex_color.clone()),
                            Transform::default(),
                            ChildOf(scene.entity),
                        ))
                        .id();
                    scene.axes.insert(
                        key,
                        AxisEntity {
                            entity,
                            mesh: handle,
                            signature,
                            organs: axis.organs.clone(),
                            style,
                            colors,
                        },
                    );
                }
            }
        }
        scene.axes.retain(|k, a| {
            let keep = seen.contains(k);
            if !keep {
                commands.entity(a.entity).despawn();
            }
            keep
        });

        // Templated organs.
        let options = InstanceOptions {
            water_stress_factor: pot.water_stress,
            ..InstanceOptions::default()
        };
        let list = instances(&pot.skeleton, options);
        let mut alive = Vec::with_capacity(list.len());
        for inst in &list {
            alive.push(inst.organ);
            let Some(p) = pot.skeleton.get(inst.organ) else {
                continue;
            };
            let rgb = organ_color(pot, lens, inst.organ.index(), p);
            let (material, color) = assets.material(&mut materials, rgb);
            let transform = Transform {
                translation: inst.translation,
                rotation: inst.rotation,
                scale: inst.scale.max(Vec3::splat(1e-5)),
            };
            let mesh = assets.templates.get(&inst.key).cloned().unwrap_or_default();
            if let Some(e) = scene.instances.get_mut(&inst.organ) {
                let mut ec = commands.entity(e.entity);
                ec.insert(transform);
                if e.key != inst.key {
                    ec.insert(Mesh3d(mesh));
                    e.key = inst.key;
                }
                if e.color != color {
                    ec.insert(MeshMaterial3d(material));
                    e.color = color;
                }
            } else {
                let entity = commands
                    .spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        transform,
                        ChildOf(scene.entity),
                    ))
                    .id();
                scene.instances.insert(
                    inst.organ,
                    InstanceEntity {
                        entity,
                        key: inst.key,
                        color,
                    },
                );
            }
        }
        scene.instances.retain(|k, e| {
            let keep = alive.binary_search(k).is_ok();
            if !keep {
                commands.entity(e.entity).despawn();
            }
            keep
        });

        // Soil water overlay on the glass.
        let show = lens == Lens::SoilWater;
        commands.entity(scene.overlay).insert(if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if show && scene.overlay_state.as_ref() != Some(&pot.soil_water.values) {
            if let Some(info) = Lens::SoilWater.info() {
                let overlay =
                    diorama::soil_face_overlay(pot.size, &pot.soil_water, |v| info.color(v));
                let _ = meshes.insert(&scene.overlay_mesh, colored(&overlay));
            }
            scene.overlay_state = Some(pot.soil_water.values.clone());
        }
    }
    rig.frame_bounds = bounds;
}
