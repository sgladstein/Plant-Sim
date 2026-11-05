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

`--present` and `--frame-latency` set the swap chain's present mode and maximum queued
frames. **The default is mailbox** (Bevy falls back to immediate, then FIFO) with a frame
cap at the monitor's refresh rate (60 if unknown); `--fps-cap N` changes the cap, 0 turns
it off, and FIFO/auto vsync are never capped. `--window WxH` sets the logical window size
and keeps it; without it the 1600x900 default shrinks to fit 92% of the primary monitor. `--bench-frames N [--bench-out FILE.csv]`
records N frame times (real time between frames, so vsync waits count) after the pipelines
are ready and a 120-frame warm-up, writes one row per frame plus a `# summary` line
(adapter, backend, size, present mode, frame latency, frame cap, mean, p50, p95, p99, max), prints the summary, and
exits. Example: `plant-sim --day 45 --present fifo --frame-latency 2 --bench-frames 2000`.

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
- Both are far under the 16 ms budget. Re-run on 58807c6 (2000 frames): Vulkan FIFO mean
  16.66, p95 30.5, p99 31.4 ms, with 7.2% of frames ≥ 28 ms (one every ~14 frames, a
  ~64 Hz present cadence beating against 60 Hz, likely the virtual display); frame
  latency 2 vs 3 and window size made no difference. Vulkan mailbox: mean 4.0, p99 4.4,
  no spikes, uncapped (~250 fps). DX12 FIFO: p95 17.5, p99 32.7, 4.3% irregular spikes.
  Hence the mailbox + frame cap default. Snapshot reads clone an `Arc` under a lock held
  only for the clone, so the render thread never waits on sim work.
- Real sim on 74132fc (Vulkan, capped mailbox at 60, day 45.5): mean 16.67, p95 17.2,
  p99 29.8 ms; spikes ≥ 28 ms down to 2.3–2.8%, in bursts with quiet stretches of 500+
  frames, each followed by a ~8 ms catch-up frame. The limiter now spins the last 2 ms
  instead of 1 ms; if bursts remain, check whether they line up with sim ticks (mesh
  rebuilds) before blaming pacing.
- DX12's slow cold start is FXC, the old shader compiler wgpu uses when DXC is missing.
  ps-app now links DXC statically on Windows (Bevy feature `statically-linked-dxc`), so
  DX12 should compile shaders far faster; to be measured on the 1070.
- DX12 took about 180 s to first render (Vulkan about 5 s), so DX12 is only a fallback.
- DX12 cold start was slow enough that the old fixed-frame `--screenshot` saved nothing
  or a black frame; it now waits for readiness.
- Not done yet: wgpu's on-disk pipeline cache for Vulkan, to shorten cold starts.
