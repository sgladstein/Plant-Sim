//! Scene sync: turns the read-only [`Garden`] into entities, every frame.
//!
//! - One entity per stem axis, keyed by its basal organ id. Its tube mesh is rebuilt
//!   only when [`AxisSignature::needs_rebuild`] says it changed by more than 2% (§4);
//!   lens and age colors are vertex colors (8-bit), rewritten without touching geometry.
//! - One entity per plant for all its leaves' petioles and one for all its roots (see
//!   `MergedGroup`). Roots are checked once per sim hour, not every interpolated frame,
//!   and a root under 10 cm is rebuilt only once it moved 2 mm (`ROOT_REBUILD_FLOOR`).
//! - Nothing is rebuilt on a frame where the garden, the lens and the camera focus are
//!   all unchanged (paused, or between sim frames at low speed).
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
    axis_mesh, build_template_shaded, instances, nodes_after_base,
};

use crate::camera::CameraRig;
use crate::diorama::{self, ColoredMesh};
use crate::garden::{Garden, ObstacleView, PotSize, PotView, SupportView, natural_color};
use crate::lens::Lens;

/// Rebuild threshold for stem and root meshes (§4).
pub const REBUILD_THRESHOLD: f32 = 0.02;
/// Roots grow at their tips several percent an hour while short, so the 2% rule alone
/// rebuilt nearly every root of a running plot every frame. Under the soil a step of
/// this much (m) does not show.
const ROOT_REBUILD_FLOOR: f32 = 0.002;

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
    /// Stems and roots while a lens colors them: unlit, so the color reads as the value.
    vertex_color_lens: Handle<StandardMaterial>,
    soil: Handle<StandardMaterial>,
    glass: Handle<StandardMaterial>,
    haze: Handle<StandardMaterial>,
    overlay: Handle<StandardMaterial>,
    palette: HashMap<([u8; 3], bool), Handle<StandardMaterial>>,
}

/// Quantizes a linear color with a square-root curve, so dark greens do not band.
fn quantize(rgb: [f32; 3]) -> [u8; 3] {
    rgb.map(|v| (v.clamp(0.0, 1.0).sqrt() * 255.0).round() as u8)
}

impl SceneAssets {
    /// A shared material for a templated organ. `glow` is set when the color is a lens
    /// value: most of it is then emitted, so shadows cannot move it along the scale.
    fn material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        rgb: [f32; 3],
        glow: bool,
    ) -> (Handle<StandardMaterial>, ([u8; 3], bool)) {
        let key = (quantize(rgb), glow);
        let handle = self
            .palette
            .entry(key)
            .or_insert_with(|| {
                let c = key.0.map(|v| (f32::from(v) / 255.0).powi(2));
                if glow {
                    let lit = 1.0 - crate::look::LENS_GLOW;
                    // Emissive ignores camera exposure (emissive_exposure_weight 0), so this is
                    // the displayed share directly.
                    let e = crate::look::LENS_GLOW;
                    materials.add(StandardMaterial {
                        base_color: Color::linear_rgb(c[0] * lit, c[1] * lit, c[2] * lit),
                        emissive: LinearRgba::rgb(c[0] * e, c[1] * e, c[2] * e),
                        perceptual_roughness: 0.9,
                        reflectance: 0.1,
                        ..default()
                    })
                } else {
                    materials.add(StandardMaterial {
                        base_color: Color::linear_rgb(c[0], c[1], c[2]),
                        // Leaves are matte with a little waxy sheen, and let some light
                        // through, so backlit leaves glow instead of going black.
                        perceptual_roughness: 0.8,
                        reflectance: 0.22,
                        diffuse_transmission: 0.35,
                        ..default()
                    })
                }
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
    color: ([u8; 3], bool),
    transform: Transform,
}

/// One axis inside its plant's merged mesh.
struct MergedPart {
    signature: AxisSignature,
    organs: Vec<OrganId>,
    mesh: MeshData,
    colors: Vec<[f32; 4]>,
}

/// Many small axes of one plant in one mesh and one entity: a bean at day 50 has about
/// 25 petioles and 65 root axes, and one entity each made a 30-plant plot draw about
/// 2,700 tubes. Each part keeps its own tube, rebuilt by the 2% rule; the merged mesh
/// is re-concatenated only when a part's geometry changed or a part came or went, and
/// only its colors are rewritten when only colors changed.
struct MergedGroup {
    entity: Entity,
    mesh: Handle<Mesh>,
    parts: BTreeMap<OrganId, MergedPart>,
    seen: Vec<OrganId>,
    geometry_changed: bool,
    colors_changed: bool,
}

impl MergedGroup {
    fn spawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        parent: Entity,
        material: Handle<StandardMaterial>,
    ) -> Self {
        let mesh = meshes.add(to_bevy_mesh(&MeshData::new(), Some(&[])));
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                ChildOf(parent),
            ))
            .id();
        Self {
            entity,
            mesh,
            parts: BTreeMap::new(),
            seen: Vec::new(),
            geometry_changed: false,
            colors_changed: false,
        }
    }

    /// Keeps part `key` current: rebuilds its tube with `build` when the signature says
    /// so, otherwise only notes a color change.
    fn update(
        &mut self,
        key: OrganId,
        signature: AxisSignature,
        organs: &[OrganId],
        colors: Vec<[f32; 4]>,
        floor: f32,
        build: impl FnOnce() -> MeshData,
    ) {
        self.seen.push(key);
        match self.parts.get_mut(&key) {
            Some(part)
                if !signature.needs_rebuild_beyond(&part.signature, REBUILD_THRESHOLD, floor)
                    && part.organs == organs =>
            {
                if part.colors != colors {
                    part.colors = colors;
                    self.colors_changed = true;
                }
            }
            _ => {
                self.parts.insert(
                    key,
                    MergedPart {
                        signature,
                        organs: organs.to_vec(),
                        mesh: build(),
                        colors,
                    },
                );
                self.geometry_changed = true;
            }
        }
    }

    /// Drops the parts not updated since the last call and sends what changed.
    fn finish(&mut self, meshes: &mut Assets<Mesh>) {
        let seen = std::mem::take(&mut self.seen);
        let before = self.parts.len();
        self.parts.retain(|k, _| seen.binary_search(k).is_ok());
        self.geometry_changed |= self.parts.len() != before;
        if self.geometry_changed {
            let (mesh, colors) =
                concat(self.parts.values().map(|p| (&p.mesh, p.colors.as_slice())));
            let _ = meshes.insert(&self.mesh, to_bevy_mesh(&mesh, Some(&colors)));
        } else if self.colors_changed
            && let Some(mut m) = meshes.get_mut(&self.mesh)
        {
            let colors: Vec<[f32; 4]> = self
                .parts
                .values()
                .flat_map(|p| p.colors.iter().copied())
                .collect();
            m.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        self.geometry_changed = false;
        self.colors_changed = false;
    }
}

/// Concatenates meshes already in one frame, with their vertex colors.
fn concat<'a>(
    parts: impl Iterator<Item = (&'a MeshData, &'a [[f32; 4]])>,
) -> (MeshData, Vec<[f32; 4]>) {
    let mut mesh = MeshData::new();
    let mut colors = Vec::new();
    for (part, c) in parts {
        let base = u32::try_from(mesh.positions.len()).unwrap_or(u32::MAX);
        mesh.positions.extend_from_slice(&part.positions);
        mesh.normals.extend_from_slice(&part.normals);
        mesh.uvs.extend_from_slice(&part.uvs);
        mesh.indices.extend(part.indices.iter().map(|i| i + base));
        colors.extend_from_slice(c);
    }
    (mesh, colors)
}

struct PotScene {
    entity: Entity,
    haze: Entity,
    size: PotSize,
    /// Whether this scene draws the container (see `PotView::container`).
    container: bool,
    /// Supports and obstacles drawn with the container; a change respawns it.
    supports: Vec<SupportView>,
    obstacles: Vec<ObstacleView>,
    axes: BTreeMap<OrganId, AxisEntity>,
    petioles: Option<MergedGroup>,
    roots: Option<MergedGroup>,
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
    /// Whether the scene has been synced once, and the camera focus it was synced for.
    synced: bool,
    synced_focus: Option<usize>,
    /// The sim hour roots were last checked at.
    roots_hour: Option<i64>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut index: ResMut<SceneIndex>,
) {
    let templates = TemplateKey::all()
        .into_iter()
        .map(|k| {
            // Leaf veins and pale undersides as a grey vertex color that multiplies the
            // organ's (or lens's) color.
            let (m, shade) = build_template_shaded(k);
            let grey: Vec<[f32; 4]> = shade.iter().map(|&g| [g, g, g, 1.0]).collect();
            (k, meshes.add(to_bevy_mesh(&m, Some(&grey))))
        })
        .collect();
    let vertex_color = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        reflectance: 0.2,
        ..default()
    });
    let vertex_color_lens = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });
    let soil = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        reflectance: 0.1,
        ..default()
    });
    let glass = materials.add(StandardMaterial {
        // Nearly invisible: a faint sky reflection says "glass" without greying the soil.
        base_color: Color::linear_rgba(0.9, 0.95, 1.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.15,
        reflectance: 0.06,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    let haze = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        reflectance: 0.1,
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
        vertex_color_lens,
        soil,
        glass,
        haze,
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
    // A low wooden deck under the pots, on gravel that fades into the horizon haze
    // (the fog), so there is no hard table edge against the sky.
    let floor = -PotSize::RHIZOBOX.soil_depth - 0.042;
    // Bench, lawn, pebbles and distant trees: vertex-colored, double-sided, rebuilt when
    // the pot layout changes (crate::dressing).
    let dressing_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        reflectance: 0.15,
        // No backface culling, and no normal flip on back faces: grass blades and
        // leaves of the lawn keep their up-facing normals from either side.
        double_sided: false,
        cull_mode: None,
        ..default()
    });
    let bench_mesh = meshes.add(colored(&crate::dressing::bench(Vec2::new(0.95, 0.42))));
    commands.spawn((
        Mesh3d(bench_mesh.clone()),
        MeshMaterial3d(dressing_material.clone()),
        Transform::from_xyz(0.0, floor, 0.0),
        Floor::Deck,
    ));
    let garden_mesh = meshes.add(colored(&ColoredMesh::default()));
    commands.spawn((
        Mesh3d(garden_mesh.clone()),
        MeshMaterial3d(dressing_material),
        Transform::from_xyz(0.0, floor - crate::dressing::BENCH_HEIGHT, 0.0),
        Floor::Dressing,
        // 20k grass blades in the shadow map cost time and speckle the lawn.
        bevy::light::NotShadowCaster,
    ));
    commands.insert_resource(DressingMeshes {
        bench: bench_mesh,
        garden: garden_mesh,
    });
    let g = crate::look::GROUND;
    let ground = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(g[0], g[1], g[2]),
        perceptual_roughness: 1.0,
        reflectance: 0.1,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(240.0, 240.0))),
        MeshMaterial3d(ground),
        Transform::from_xyz(0.0, floor - crate::dressing::BENCH_HEIGHT, 0.0),
        Floor::Ground,
    ));
    index.root = Some(root);
}

/// The deck and the ground, moved to sit under the pots whenever the pot layout changes.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Floor {
    Deck,
    Ground,
    Dressing,
}

/// The bench and garden meshes, replaced when the pot layout changes.
#[derive(Resource)]
pub(crate) struct DressingMeshes {
    bench: Handle<Mesh>,
    garden: Handle<Mesh>,
}

/// The lawn, pebbles and trees around a bench of `size` (x, z).
fn garden_dressing(size: Vec2) -> ColoredMesh {
    let half = size * 0.5;
    // Scale the lawn and tree ring with the layout: a 3 m trellis row is framed from
    // about 16 m, so a fixed 13 m ring would stand around the camera.
    let extent = half.max_element();
    let lawn_radius = (2.5 * extent).max(4.5);
    let ring = (6.0 * extent + 10.0).max(13.0);
    let mut m = crate::dressing::lawn(half, lawn_radius);
    for part in [
        crate::dressing::pebbles(half),
        crate::dressing::treeline(ring),
    ] {
        let base = m.mesh.vertex_count() as u32;
        m.mesh.positions.extend_from_slice(&part.mesh.positions);
        m.mesh.normals.extend_from_slice(&part.mesh.normals);
        m.mesh.uvs.extend_from_slice(&part.mesh.uvs);
        m.mesh
            .indices
            .extend(part.mesh.indices.iter().map(|i| i + base));
        m.colors.extend_from_slice(&part.colors);
    }
    m
}

/// Deck center x, z (Bevy frame), size x, z and top height y for a set of pots.
fn deck_layout(pots: &[PotView]) -> Option<(Vec3, Vec2)> {
    let (mut lo, mut hi) = (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY));
    let mut depth = 0.0_f32;
    for p in pots {
        let c = Vec2::from(p.center);
        let half = Vec2::new(p.size.width, p.size.depth) * 0.5;
        lo = lo.min(c - half);
        hi = hi.max(c + half);
        depth = depth.max(p.size.soil_depth);
    }
    if !lo.is_finite() || !hi.is_finite() {
        return None;
    }
    let margin = Vec2::new(0.12, 0.1);
    let size = hi - lo + 2.0 * margin;
    let c = 0.5 * (lo + hi);
    // Sim (x, y) maps to Bevy (x, -z); the pot frames' feet sit 0.042 m below the soil.
    Some((Vec3::new(c.x, -depth - 0.042, -c.y), size))
}

fn spawn_pot(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &SceneAssets,
    root: Entity,
    pot: &PotView,
    ground: f32,
) -> PotScene {
    let entity = commands
        .spawn((
            Transform::from_xyz(pot.center[0], pot.center[1], 0.0),
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    let size = pot.size;
    if !pot.container {
        // A plant sharing a plot's bed: the first plant's view draws the bed.
        let haze = commands.spawn((Transform::default(), ChildOf(entity))).id();
        let overlay = commands.spawn((Transform::default(), ChildOf(entity))).id();
        return PotScene {
            entity,
            haze,
            size,
            container: false,
            supports: Vec::new(),
            obstacles: Vec::new(),
            axes: BTreeMap::new(),
            petioles: None,
            roots: None,
            instances: BTreeMap::new(),
            overlay,
            // Never shown: only the container's view draws the soil overlay.
            overlay_mesh: Handle::default(),
            overlay_state: None,
        };
    }
    let overlay_mesh = meshes.add(colored(&ColoredMesh::default()));
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
    if !pot.supports.is_empty() {
        commands.spawn((
            Mesh3d(meshes.add(colored(&diorama::supports(&pot.supports)))),
            MeshMaterial3d(assets.vertex_color.clone()),
            Transform::default(),
            ChildOf(entity),
        ));
    }
    for (cloth, material) in [(false, &assets.vertex_color), (true, &assets.haze)] {
        if pot.obstacles.iter().any(|o| o.is_cloth() == cloth) {
            commands.spawn((
                Mesh3d(meshes.add(colored(&diorama::obstacles(&pot.obstacles, cloth, ground)))),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                ChildOf(entity),
            ));
        }
    }
    commands.spawn((
        Mesh3d(meshes.add(to_bevy_mesh(&diorama::glass(size), None))),
        MeshMaterial3d(assets.glass.clone()),
        Transform::default(),
        ChildOf(entity),
    ));
    let haze = commands
        .spawn((
            Mesh3d(meshes.add(colored(&diorama::soil_haze(size, 0.42)))),
            MeshMaterial3d(assets.haze.clone()),
            Transform::default(),
            ChildOf(entity),
        ))
        .id();
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
        haze,
        size,
        container: true,
        supports: pot.supports.clone(),
        obstacles: pot.obstacles.clone(),
        axes: BTreeMap::new(),
        petioles: None,
        roots: None,
        instances: BTreeMap::new(),
        overlay,
        overlay_mesh,
        overlay_state: None,
    }
}

/// The color of organ `i` of `pot` under `lens`, and whether it is a lens value.
fn organ_color(pot: &PotView, lens: Lens, i: usize, p: &Placement) -> ([f32; 3], bool) {
    let natural = crate::garden::varied_color(natural_color(p), p, i);
    let Some(info) = lens.info() else {
        return (natural, false);
    };
    if !lens.colors_organs() {
        return (natural, false);
    }
    if lens == Lens::RootsXray && p.kind != OrganKind::Root {
        return (natural, false);
    }
    // Organs the lens says nothing about fade to gray so the colored ones read.
    pot.organ_value(lens, i)
        .and_then(|v| info.color(v))
        .map_or_else(
            || {
                let g = 0.3 * natural[0] + 0.6 * natural[1] + 0.1 * natural[2];
                ([0.5 * g + 0.12; 3], false)
            },
            |c| (c, true),
        )
}

fn axis_colors(
    pot: &PotView,
    lens: Lens,
    organs: &[OrganId],
    style: AxisStyle,
    copies: u16,
) -> Vec<[f32; 4]> {
    let ring = style.sides as usize + 1;
    let mut out = Vec::with_capacity((organs.len() + 1) * ring * usize::from(copies));
    // One ring for the base node (the first organ's color), then each organ's rings
    // (several along a twining internode's coil), matching `axis_nodes`.
    for k in 0..=organs.len() {
        let id = organs[k.saturating_sub(1).min(organs.len() - 1)];
        let p = pot.skeleton.get(id);
        // To 8 bits, as the GPU shows them: thermal age moves every interpolated frame,
        // and unquantized colors re-sent every stem and root's colors every frame.
        let c = p.map_or([1.0; 3], |p| organ_color(pot, lens, id.index(), p).0);
        let c = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() / 255.0);
        let rings = if k == 0 {
            1
        } else {
            p.map_or(1, nodes_after_base)
        };
        out.extend(std::iter::repeat_n([c[0], c[1], c[2], 1.0], ring * rings));
    }
    // Copies of a root cohort share their organs' colors.
    let one = out.len();
    for _ in 1..copies {
        out.extend_from_within(..one);
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
    mut floors: Query<(&Floor, &mut Transform)>,
    dressing: Option<Res<DressingMeshes>>,
) {
    let Some(root) = index.root else {
        return;
    };
    // Nothing moved: keep every entity and the camera bounds as they are.
    let focus = rig.focus;
    // The lens resource is written by the UI every frame, so compare its value.
    if !garden.is_changed()
        && index.lens == Some(*lens)
        && index.synced
        && index.synced_focus == focus
    {
        return;
    }
    index.synced = true;
    index.synced_focus = focus;
    let lens = *lens;
    // Pots are static; respawn everything if their number or sizes change.
    let layout_changed = index.pots.len() != garden.pots.len()
        || index.pots.iter().zip(&garden.pots).any(|(s, p)| {
            s.size != p.size
                || s.container != p.container
                || (p.container && (s.supports != p.supports || s.obstacles != p.obstacles))
        });
    if layout_changed {
        for p in index.pots.drain(..) {
            commands.entity(p.entity).despawn();
        }
        let layout = deck_layout(&garden.pots);
        // The lawn below the soil surface, so walls and cloth posts stand on it.
        let ground = layout.map_or(-1.0, |(top, _)| top.y - crate::dressing::BENCH_HEIGHT);
        for pot in &garden.pots {
            let scene = spawn_pot(&mut commands, &mut meshes, &assets, root, pot, ground);
            index.pots.push(scene);
        }
        if let Some((top, size)) = layout {
            if let Some(d) = &dressing {
                let _ = meshes.insert(&d.bench, colored(&crate::dressing::bench(size)));
                let _ = meshes.insert(&d.garden, colored(&garden_dressing(size)));
            }
            for (floor, mut t) in &mut floors {
                let ground = top.y - crate::dressing::BENCH_HEIGHT;
                match floor {
                    Floor::Deck => t.translation = top,
                    Floor::Ground => t.translation.y = ground,
                    Floor::Dressing => t.translation = Vec3::new(top.x, ground, top.z),
                }
            }
        }
    }

    // Stems and roots go unlit while a lens colors them.
    let axis_material = if lens.info().is_some() && lens.colors_organs() {
        assets.vertex_color_lens.clone()
    } else {
        assets.vertex_color.clone()
    };
    let lens_changed = index.lens != Some(lens) || layout_changed;
    let hour = garden.hour.floor() as i64;
    let roots_hour_changed = index.roots_hour != Some(hour);
    index.roots_hour = Some(hour);
    // Soil opacity follows the lens.
    if index.lens != Some(lens) {
        for scene in &index.pots {
            commands
                .entity(scene.haze)
                .insert(if lens.soil_opacity() < 1.0 {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                });
            for e in scene
                .axes
                .values()
                .map(|a| a.entity)
                .chain(scene.petioles.iter().chain(&scene.roots).map(|g| g.entity))
            {
                commands
                    .entity(e)
                    .insert(MeshMaterial3d(axis_material.clone()));
            }
        }
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

    for (k, (scene, pot)) in index.pots.iter_mut().zip(&garden.pots).enumerate() {
        let c = Vec3::new(pot.center[0], pot.center[1], 0.0);
        let half = Vec3::new(pot.size.width / 2.0, pot.size.depth / 2.0, 0.0);
        match focus {
            // A close-up frames one plant's shoot and the top of its pot.
            Some(f) if f == k => grow(c - Vec3::Z * 0.03),
            Some(_) => {}
            None => {
                grow(c - half - Vec3::Z * pot.size.soil_depth);
                grow(c + half);
            }
        }
        if let Some((lo, hi)) = pot
            .skeleton
            .bounds()
            .filter(|_| focus.is_none_or(|f| f == k))
        {
            // Frame the shoot; roots are framed by the pot.
            grow(c + lo.max(Vec3::new(lo.x, lo.y, 0.0)));
            grow(c + hi);
        }
        if focus.is_none() {
            for s in &pot.supports {
                grow(c + Vec3::new(s.base[0], s.base[1], s.height + 0.05));
            }
        }
        for o in &pot.obstacles {
            grow(c + Vec3::from(o.min));
            grow(c + Vec3::from(o.max));
        }

        // Axes.
        let current = axes(&pot.skeleton, true);
        let mut seen = Vec::with_capacity(current.len());
        let mut petioles = scene.petioles.take().unwrap_or_else(|| {
            MergedGroup::spawn(
                &mut commands,
                &mut meshes,
                scene.entity,
                axis_material.clone(),
            )
        });
        // Roots move at most a millimeter or so an hour: check them when the sim hour or
        // the lens changes, not on every interpolated frame.
        let check_roots = scene.roots.is_none() || lens_changed || roots_hour_changed;
        let mut roots = scene.roots.take().unwrap_or_else(|| {
            MergedGroup::spawn(
                &mut commands,
                &mut meshes,
                scene.entity,
                axis_material.clone(),
            )
        });
        for axis in &current {
            let key = axis.key();
            match axis.kind {
                AxisKind::Root if check_roots => {
                    // A little thicker than ps-mesh's default so fine roots read against
                    // soil.
                    let style = AxisStyle {
                        min_radius: 0.000_5,
                        ..AxisStyle::ROOT
                    };
                    roots.update(
                        key,
                        AxisSignature::of(&pot.skeleton, axis, style),
                        &axis.organs,
                        axis_colors(pot, lens, &axis.organs, style, axis.copies),
                        ROOT_REBUILD_FLOOR,
                        || axis_mesh(&pot.skeleton, axis, style),
                    );
                    continue;
                }
                AxisKind::Root => continue,
                AxisKind::Petiole => {
                    let style = AxisStyle::for_kind(axis.kind);
                    petioles.update(
                        key,
                        AxisSignature::of(&pot.skeleton, axis, style),
                        &axis.organs,
                        axis_colors(pot, lens, &axis.organs, style, axis.copies),
                        0.0,
                        || axis_mesh(&pot.skeleton, axis, style),
                    );
                    continue;
                }
                AxisKind::Shoot => {}
            }
            let style = AxisStyle::for_kind(axis.kind);
            let signature = AxisSignature::of(&pot.skeleton, axis, style);
            let colors = axis_colors(pot, lens, &axis.organs, style, axis.copies);
            seen.push(key);
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
                            MeshMaterial3d(axis_material.clone()),
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
            let keep = seen.binary_search(k).is_ok();
            if !keep {
                commands.entity(a.entity).despawn();
            }
            keep
        });
        petioles.finish(&mut meshes);
        scene.petioles = Some(petioles);
        if check_roots {
            roots.finish(&mut meshes);
        }
        scene.roots = Some(roots);

        // Templated organs.
        let options = InstanceOptions {
            water_stress_factor: pot.water_stress,
            wilt: Some(pot.wilt),
            ..InstanceOptions::default()
        };
        let list = instances(&pot.skeleton, options);
        let mut alive = Vec::with_capacity(list.len());
        for inst in &list {
            alive.push(inst.organ);
            let Some(p) = pot.skeleton.get(inst.organ) else {
                continue;
            };
            let (rgb, glow) = organ_color(pot, lens, inst.organ.index(), p);
            let (material, color) = assets.material(&mut materials, rgb, glow);
            let transform = Transform {
                translation: inst.translation,
                rotation: inst.rotation,
                scale: inst.scale.max(Vec3::splat(1e-5)),
            };
            let mesh = assets.templates.get(&inst.key).cloned().unwrap_or_default();
            if let Some(e) = scene.instances.get_mut(&inst.organ) {
                let mut ec = commands.entity(e.entity);
                if e.transform != transform {
                    ec.insert(transform);
                    e.transform = transform;
                }
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
                        transform,
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
        let show = lens == Lens::SoilWater && scene.container;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tri(z: f32) -> MeshData {
        let mut m = MeshData::new();
        let a = m.push_vertex(Vec3::new(0.0, 0.0, z), Vec3::Z, Vec2::ZERO);
        let b = m.push_vertex(Vec3::new(1.0, 0.0, z), Vec3::Z, Vec2::X);
        let c = m.push_vertex(Vec3::new(0.0, 1.0, z), Vec3::Z, Vec2::Y);
        m.push_triangle(a, b, c);
        m
    }

    #[test]
    fn merged_parts_keep_each_part_and_its_colors() {
        let (a, b) = (tri(0.0), tri(1.0));
        let (ca, cb) = ([[0.3, 0.3, 0.3, 1.0]; 3], [[0.7, 0.7, 0.7, 1.0]; 3]);
        let (mesh, colors) = concat([(&a, &ca[..]), (&b, &cb[..])].into_iter());
        assert_eq!(mesh.positions.len(), 6);
        assert_eq!(colors.len(), 6);
        // The second part's indices are offset past the first's vertices.
        assert_eq!(mesh.indices, [0, 1, 2, 3, 4, 5]);
        assert!((mesh.positions[3][2] - 1.0).abs() < 1e-6);
        assert!((colors[0][0] - 0.3).abs() < 1e-6 && (colors[5][0] - 0.7).abs() < 1e-6);
    }
}
