# Bevy notes (WP13)

The renderer is pinned. Use only the APIs of these versions; if the compiler disagrees with
memory, the compiler is right (CLAUDE.md).

## Pinned versions (chosen 2026-11-05)

| Crate | Version | Why |
| --- | --- | --- |
| bevy | =0.20.1 | Latest stable on the day M1 work started. Needs Rust 1.97.1. |
| bevy_egui | =0.43.1 | The release for Bevy 0.20 (egui 0.36). |
| glam | 0.33 | The version Bevy 0.20 uses, so ps-mesh's `Vec3` is Bevy's `Vec3`. |

Bevy features: `default-features = false`, `["3d", "ui"]` (no audio, no 2D).
bevy_egui features: `render`, `default_fonts`.

Only ps-view, ps-ui and ps-app may depend on Bevy (scripts/check_deps.py). The sim CI
jobs exclude them; `.github/workflows/render.yml` lints and tests them on Linux and uploads
a Windows release build (`plant-sim-windows` artifact).

## Running

```text
cargo run --release -p ps-app -- [--day N] [--speed H] [--paused] [--lens NAME] [--screenshot FILE.png]
                                 [--present auto|fifo|mailbox|immediate|novsync] [--frame-latency N]
```

`--present` and `--frame-latency` set the swap chain's present mode (default auto vsync)
and maximum queued frames, for frame-pacing tests.

Lens names: off, light, soil-water, plant-water, roots, carbon, phenology, shade.
`--screenshot` waits until the GPU pipelines have compiled (`ps_view::Ready`) plus 60
frames, saves the window and exits after the file is written (or fails after 180 s); with `xvfb-run` and Mesa's
lavapipe it works in a Linux container (slow, but enough for a visual check):

```text
sudo apt-get install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-0 mesa-vulkan-drivers xvfb
xvfb-run -a -s "-screen 0 1600x900x24" env WGPU_BACKEND=vulkan target/debug/plant-sim --day 45.5 --screenshot shot.png
```

## Idioms used

- Sim frame (x east, y north, z up) is kept for all plant geometry; one root entity
  (`GardenRoot`) rotates it into Bevy's y-up frame. `ps_view::scene::sim_to_bevy` converts
  points for camera framing.
- Hierarchy: `ChildOf(parent)` on spawn; `despawn()` is recursive.
- Meshes: `Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())`,
  `insert_attribute(Mesh::ATTRIBUTE_*)`, `insert_indices(Indices::U32)`. Replace a mesh in
  place with `Assets::insert(&handle, mesh)` (returns a `Result`).
- Vertex colors (`Mesh::ATTRIBUTE_COLOR`) multiply `StandardMaterial::base_color`; stems,
  roots, soil and walls use them, so lens recoloring rewrites one attribute, not geometry.
- Templated organs share one mesh per template and one material per quantized color, so
  Bevy batches identical mesh-material pairs (design plan §11.9).
- Input: `Res<AccumulatedMouseMotion>`, `Res<AccumulatedMouseScroll>`,
  `Res<ButtonInput<MouseButton>>`.
- Ambient light is the `GlobalAmbientLight` resource (`brightness` in cd/m²).
- Messages (formerly events): `MessageWriter<AppExit>` and `exit.write(AppExit::Success)`.
- Screenshots: `commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path))`
  from `bevy::render::view::screenshot`.
- egui 0.36 panels take a `&mut Ui`, not a context: build a root `egui::Ui` over
  `ctx.viewport_rect()` on the background layer, then `egui::Panel::top(id).show(&mut root, ..)`.
  UI systems run in the `EguiPrimaryContextPass` schedule and return `Result`
  (`contexts.ctx_mut()?`).
- Camera input is blocked while the pointer is over egui via
  `bevy_egui::input::EguiWantsInput`.

## Gotchas hit

- `anyhow::Context` and Bevy's prelude `ContextExt` both add `with_context` to `Option`;
  call `anyhow::Context::with_context(..)` explicitly in files that glob-import the prelude.
- On Linux, winit needs `libxkbcommon-x11.so` at run time (not only the -dev headers).
- A mesh with zero vertices logs a warning from the mesh allocator; harmless (empty
  overlays before a lens is turned on).
- clippy pedantic: Bevy systems take `Res<T>` by value, so they carry
  `#[allow(clippy::needless_pass_by_value)]`; big sync systems also allow
  `too_many_arguments` and `too_many_lines`.

- Cold start: pipelines compile in the background, so the first seconds render sky only.
  `ps_view::ready` copies `PipelineCache::waiting_pipelines().count()` from the render
  world into shared atomics each frame; the scene is `Ready` after 20 quiet frames, and
  a "Preparing shaders…" overlay (Bevy UI) covers the view until then.

## GTX 1070 results (WP13, 2026-11-05, stand-in plants, 2560×1440)

| Backend | vsync off mean | p99 | vsync on |
| --- | --- | --- | --- |
| Vulkan | 2.70 ms | 2.94 ms | p95 30.4 ms (uneven pacing) |
| DX12 | 5.78 ms | 6.05 ms | p99 32.6 ms |

- With no `WGPU_BACKEND` set, wgpu picked **Vulkan** on the owner's PC (not DX12). Keep
  that: Vulkan is twice as fast here (it runs at ~51% GPU, so it is CPU/submit bound;
  DX12 sits at 98% GPU). `WGPU_BACKEND=dx12` remains the fallback.
- Both are far under the 16 ms budget. The vsync-on spikes (missed vblanks with 3 ms of
  work) are open: compare `--present fifo|mailbox|auto` and `--frame-latency 2|3` with a
  frame-time histogram per backend. When ps-session drives the view, snapshot reads must
  not block the render thread on a lock.
- DX12 cold start was slow enough that the old fixed-frame `--screenshot` saved nothing
  or a black frame; it now waits for readiness.
- Not done yet: wgpu's on-disk pipeline cache for Vulkan, to shorten cold starts.
