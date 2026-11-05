# Viewer workstream handoff (M1, crates ps-mesh / ps-view / ps-ui / ps-app)

Written 2026-11-05 by the viewer thread. Main at the time of writing: 2392f58 (#78), plus PR #84 (root copies fix) pending CI.

## 1. What the viewer is

- **ps-mesh** (engine-free, glam only): `Skeleton` built from an organ arena (`Skeleton::from_arena_with(arena, origin, attach, multiplicity)`), tube axes (`axes`, `axis_mesh`, `copy_transforms` for root cohorts), and instanced organ templates (`TemplateKey`, `build_template_shaded`). The skeleton frame has its origin at the soil surface center, with z up.
- **ps-view** (Bevy):
  - It reads only two resources, `Garden` (a list of `PotView`) and `SkyState`.
  - `scene.rs` spawns and syncs the meshes; `PotView` changes in size, container, supports or obstacles respawn the pot.
  - Other modules: `diorama.rs` (pots, beds, glass, soil, supports, obstacles), `dressing.rs` (bench, lawn, tree ring), `camera.rs` (auto-framing orbit rig, camera-relative fog), `pick.rs` (click picking), lenses and legends.
- **ps-ui** (bevy_egui): the top bar (time, speeds, steps, replay scrubber, Charts, Screenshot, Export CSV, New run…) and the right panel. The right panel holds the lenses, the inspector with Harvest and Cut buttons, Garden tools, and the Seed library. The charts sit in the bottom panel. The UI never edits sim state: it queues `TimeCommand`s and `PlayerAction`s.
- **ps-app** (binary `plant-sim`):
  - `sim_source.rs` runs a ps-world scenario on the ps-session sim thread and fills `Garden` from each `Snapshot`.
  - `tools.rs` turns `PlayerAction` into `ActionKind` and runs the new-run dialog.
  - `experiment.rs` loads experiment cards, builds the experiment scenario and writes the lab notebook.
  - `inspect.rs` fills the inspector.
  - `capture.rs` handles screenshots, CSV export and offline time-lapse.
  - `demo_source.rs` has the stand-in plants (`--demo`).
- New model cards must be added to the `cards!` list in `ps-ui/src/model_cards.rs` (a test fails otherwise).
- **Rule:** the sim crates (ps-world and below) never depend on Bevy. `scripts/check_deps.py` enforces this.

## 2. How to work on it

- **Pins:** Bevy =0.20.1 and bevy_egui 0.43.1 (egui 0.36), glam 0.33, Rust 1.97. Version idioms are in `docs/bevy-notes.md`. If the compiler disagrees with memory, the compiler is right.
- **Checks before every push:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (clippy pedantic), `cargo test --workspace`, `python3 scripts/check_deps.py`, `python3 scripts/check_determinism.py`, `cargo deny check`, `cargo metadata --locked`.
- **Headless screenshots** (Linux, xvfb plus the lavapipe software Vulkan driver):
  ```
  xvfb-run -a -s "-screen 0 1600x900x24" env WGPU_BACKEND=vulkan timeout 400 \
    ./target/debug/plant-sim --scenario data/scenarios/X.ron --day 45.5 --lens light --screenshot out.png
  ```
  - `--screenshot` waits until the pipelines have compiled and the sim has reached `--day`, then saves and exits. It fails after 180 s.
  - `--data DIR` lets a scenario live outside the repo.
  - Other useful flags: `--clean` (caption only), `--inspect POT[:ORGAN]`, `--charts`, `--paused`, `--card NAME` (open a model card).
  - The container needs `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-0 mesa-vulkan-drivers` to build and screenshot.
  - A debug build is too slow for wheat_stand (200 plants) past about day 80, so use a single-pot or small scenario.
  - Always look at the PNG; this is how every visual change was verified.
- **Disk:** target/debug grows to 25+ GB. Build with `CARGO_INCREMENTAL=0`, and run `cargo clean` when free space drops under about 10 GB.
- **CI:**
  - `linux.yml` and `windows.yml` exclude the render crates.
  - `render.yml` runs clippy and tests for ps-view, ps-ui and ps-app. It also builds the Windows release with `--features static-dxc` and uploads the `plant-sim-windows` artifact (exe plus data/).
- **Static DXC:**
  - Every Windows build that is shipped must use `--features static-dxc`.
  - Without it, DX12 falls back to FXC and takes about 176 s to its first frame on the owner's GTX 1070. With it, the first frame comes in 8.7 s, then 5.5 s.
  - Building it locally needs the Visual Studio "C++ ATL for latest build tools" component. Without it the build fails with LNK1104 atls.lib.
- **Cutting a release:**
  - GitHub's MCP tools cannot create releases, so `.github/workflows/prerelease.yml` does it.
  - Dispatch it on main with `run_id` (a green render.yml run on main) and `tag`.
  - It downloads that run's `plant-sim-windows` artifact, zips it as `plant-sim-windows-<tag>.zip`, and publishes a GitHub pre-release with the zip and exe SHA-256 in the notes.
  - v0.1.0-pre.1 is main d5ab169: https://github.com/sgladstein/Plant-Sim/releases/tag/v0.1.0-pre.1. Zip SHA-256: fe288957d02b5491c560895227e5db478183d7ac2d415ff9b53a7a1696c03316.
- **Branch workflow used:** one branch per thread. After each squash merge, reset the branch onto main, do the new work, then `git merge -s ours origin/<branch>` so a plain push works (force push was denied). PRs are merged by the thread once CI is green; the owner pre-approved all merges.

## 3. Owner's PC facts (GTX 1070, Windows, 2560x1440 at 175%)

- wgpu picks Vulkan by default. It is about 2x faster than DX12 there (2.7 vs 5.8 ms with stand-in plants), so keep Vulkan as the default and `WGPU_BACKEND=dx12` as the fallback.
- **Present mode:**
  - The default is mailbox, capped at 2x the monitor refresh rate. `--fps-cap N` overrides the cap, and 0 turns it off.
  - FIFO spiked to about 30 ms every 14 frames, and a 60 fps cap spiked too.
  - With the real sim on Vulkan, frames take about 4 to 8 ms.
- With static DXC, DX12 at 2353x1324 ran a mean of 8.33 ms, p99 10.95 ms, with no spikes.
- The window is fitted to 92% of the monitor. `--window WxH` is in physical pixels.
- `--bench-frames N --bench-out f.csv` records frame times with a percentile summary.
- Full numbers are in `docs/bevy-notes.md`.

## 4. State of the M1 work packages

| WP | State |
| --- | --- |
| 13 Lock the engine | Done. The frame-time gate passed on the 1070 (Vulkan and DX12); the static-DXC cold start is fixed. |
| 14 Mesh generation | Done (#23). Later additions: coils for twining stems (WP22), grass blades and wheat ears (WP23), finer leaflets, option C fuller beans (#78: petioles, level blades, nodes, flowers, pods, nodules). |
| 15 Scene sync and look | Working. Round pots and glass-fronted plot beds with a soil cutaway, stakes and trellis strings, obstacles (walls reach the lawn, cloths on posts), a garden setting (bench, lawn, tree ring), camera-relative fog, a 2% rebuild rule, and high-speed daily-mean light. |
| 16 Sim thread | Done (sim side #32, viewer side in ps-app). |
| 17 UI | Working: top bar, lenses with legends, charts, inspector with a why line and units (and each plant's own seed), model-card side panel, garden tools, seed library, new-run dialog. Experiment mode (dialog, cards in data/experiments, lab notebook, `--experiment CARD`). Not done: assimilation and transpiration charts (they are not in the daily rows). |
| 18 Lenses | All eleven lenses read sim values (the seven M1 lenses plus the four WP26 lenses). |
| 19 Capture | Done: screenshot (P), CSV export, offline time-lapse (`--timelapse DIR --day --to-day --timelapse-hours 6-19`), and `--clean`. |
| 20 The moments | Built (`ps_mesh::moments::pose`, `ps_view::moments`): emerging hook and cotyledons, lean toward the light, wilting and recovery, flowering cue (camera move-in and banner on sim time). Done-when: the owner signs off on a 60-second sun-versus-shade clip. |
| 25 UI side | Done (#76). |
| 26 UI side | Done: save and load (ps-app `saves.rs`, ps-ui `saves.rs`; the load note reports whether the state hash matches), `--load DIR`, replay-bar markers for scenario events and player actions, competition, temperature, nitrogen and carbon-flow lenses read from `Snapshot::lens` (`SessionOptions.lens = true`), phloem flow arrows (`ps_view::arrows`, a gizmo group with a small depth bias so they show through the tubes). |

## 5. The look decision

- **Chosen for now:** option C, fuller plants on the natural look (#78, merged). This came after rounds 1 and 2 (#72), which the owner said read as "colors or filters" rather than visibly different graphics.
- **Parked (not merged):**
  - A, illustrated or cel-shaded: branch `claude/project-thread-u7o4cj`. Commits 0c9ed89 and ae2aba3 add ps-view `toon.rs`, `shaders/toon.wesl` and `--look natural|illustrated`.
  - B, painterly: branch `claude/project-thread-fml7eu`.
  - Both were based on main before #76, #78 and #84, so they need a rebase. Both touch `scene.rs` materials.
- Review and renders: [docs/design/viewer-look-review.md](design/viewer-look-review.md) and `docs/design/viewer-look/`; option renders in `docs/archive/viewer/look-options/`.

## 6. Open owner items

- Sign off on WP20's 60-second sun-versus-shade clip once WP20 exists.
- Decide whether the public pre-release v0.1.0-pre.1 stays public after the account move.
- Pick a final look, or keep option C plus natural.

## 7. Next steps, in order

1. Merge PR #84 if it is still open: the root-copy fix, which keeps drawn lateral copies below the surface and inside the pot. The sim-side lateral wall rule is in model 0.11.0, from the implementation thread.
2. Done: `Plant::individual` in the whole-plant inspector.
3. WP20, the moments, then render the sun-versus-shade clip with `--timelapse` and `--clean` for sign-off.
4. The rest of WP17: assimilation and transpiration charts once the sim adds them to the daily rows. Experiment mode varies only what one world can (per-pot light, water, pot size); latitude, sowing date or weather sweeps would need one world per pot.
5. Performance with many plants. Done: a frame where the snapshot, the interpolation fraction, the high-speed flag, the lens and the camera focus are all unchanged rebuilds nothing (`SimSource.shown`, `SceneIndex.synced`); on the 30-bean plot paused at day 50 the CPU cost of drive plus scene sync fell from about 10 ms to about 0.25 ms a frame (release, lavapipe container). Each plant's petioles are one merged mesh and entity (`PetioleGroup`, 25 fewer entities per bean at day 50), instance transforms are written only when they change, and lens values are computed once per snapshot (`SimSource.lens_values`), not per frame. While running, the 30-bean plot at day 50 spent about 95 ms a frame in scene sync (release, this container): growing lateral roots a few centimeters long passed the 2% rule every frame (about 1,800 tube rebuilds a frame) and interpolated thermal age re-sent about 2,100 axes' vertex colors a frame. Now each plant's roots are one merged mesh and entity like its petioles (`MergedGroup`, about 2,000 fewer entities on the plot), roots are checked once per sim hour, a root rebuild also needs a 2 mm move (`ROOT_REBUILD_FLOOR`, `AxisSignature::needs_rebuild_beyond`), and axis colors are 8-bit, so sync is about 7 ms a frame. Open: every frame still re-lerps every skeleton (`show`, about 2 ms) and recomputes stem and petiole signatures and every instance.
6. If a look option is chosen, rebase that branch onto main and gate it behind `--look`.

## 8. Gotchas

- The moments are render-only posture applied after `Skeleton::lerp` in `sim_source::drive` (`ps_mesh::pose`). Picking and the inspector see the posed skeleton. A hooked hypocotyl adds `2 + Hook::POINTS` tube nodes (`nodes_after_base` counts them).
- The flowering cue moves the auto-frame only while `CameraRig::focus` is unset; `CameraRig::cue_zoom` multiplies the framing distance.

- `Skeleton::from_arena_with` lays out before coils are set. Call `skeleton.relayout()` after setting `Placement.coil`, or bases drift.
- Axis vertex colors must count one ring per node (`nodes_after_base`), not one per organ. Otherwise coiled axes draw giant garbage triangles.
- A plot is one soil with many plants. Only the first plant's `PotView` has `container: true` and draws the bed, label, supports and obstacles.
- The fog is camera-relative (`camera::fog_falloff`). A fixed fog end hides wide layouts.
- Dead organs are not drawn, so ripe wheat loses its dead leaves while the culms stay green (a cosmetic limit).
- Plants of one variety used to grow identically under `SeedChoice::Default`. Since #80 each plant has its own seed, emergence and microsite.
- A stale ps-cli or plant-sim binary will not read newer scenario schemas. Rebuild after pulling.
- Obstacles and supports are measured in cm from the soil's center and surface (east, north, up). The viewer converts them to m in the pot frame.
