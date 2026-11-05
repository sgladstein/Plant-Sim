# Viewer look: "fuller plants" (option C) — handoff

**Status:** Chosen by Scotttt as the current look on 2026-11-05 and merged as PR #78 (2392f58).
- Roots poking out of the soil: fixed on main by #84 (viewer thread); #86, the same fix from this workstream, was closed as a duplicate.
- Option C keeps the soft natural shading and changes plant geometry only. Everything is render-only: no sim crate changed.

## What option C changed

All the geometry is in `crates/ps-mesh`:
- `axes.rs`, `instances.rs`, `templates.rs`, `skeleton.rs`, plus tests in `tests.rs`.

Small hooks elsewhere:
- `ps-view/src/scene.rs`: axis style match, close-up framing.
- `ps-view/src/garden.rs`: `varied_color`, nodule color.
- `ps-view/src/camera.rs`: `focus`, `zoom`.
- `ps-app/src/sim_source.rs`: fills the petiole length and leaf role from `ps_arch` Meta and params. ps-arch is now a normal dependency of ps-app.
- `ps-app/src/main.rs`: CLI flags.

The changes:
- **Petioles are tubes.** `AxisKind::Petiole` gives one axis per broadleaf leaf. Grasses get none, because they set `blade_aspect`.
  - Length is the sim's: `params.petiole × sqrt(area / potential)`, the same as `ps_arch::geometry::leaf_points`.
  - The tube is a Bezier arch from the node to `ps_mesh::pulvinus(p) = base + direction × length`, with swollen pulvini at both ends.
  - `Placement` gained `petiole` (m) and `simple_leaf` (`Option<bool>`, from Role).
- **Blades start at the pulvinus.** They face the petiole's compass direction and are tilted `BLADE_TILT` = 0.337 rad tip-down, which is the light model's leaf normal, atan 0.35.
  - Only a ±0.14 rad yaw jitter is render-only, hashed from the organ id with `ps_mesh::jitter(id, k)`.
  - Pitch and roll follow the sim, because light depends on each blade's angle since #75 (decision 0025).
  - The old viewer pitched the whole leaf up along the petiole, which drew blades about 38° up and gave the "candelabra" look.
- **Unfolding.** `TemplateKey` gained an `unfold` field with 4 levels (`UNFOLD_LEVELS`).
  - It is picked by thermal age: folded shut at 0, open at `UNFOLD_THERMAL_AGE` = 70 °C·d.
  - Age is used rather than area/potential, because carbon-limited leaves stay small but still open.
  - Variants are prebuilt; switching one only swaps a mesh handle.
- **Droop** now goes with the square of the stress level, so mild stress barely shows.
- **Stems** swell at each node: three centerline nodes per internode (shoulder, shoulder, swollen node). The apex still tapers. `nodes_after_base()` must stay in sync with `axis_nodes()`, because scene.rs colors rings by it.
- **Flowers and pods.**
  - The flower has a pedicel, calyx, reflexed banner with nectar guides (a per-vertex shade), wings and a coiled keel.
  - The pod has a calyx and a beak.
  - Flowers and pods on one raceme tip get small hashed turns so they fan out.
- **Root nodules** are drawn as instanced pink lobed clusters (`Template::Nodule`), at least 1.5 mm across. Before, they were not drawn at all. They have not been confirmed visible in a render.
- **Leaf color** varies per leaf by ±6% in value, with a slight warm/cool shift (`garden::varied_color`).
- **Screenshot flags** (documented in docs/bevy-notes.md):
  - `--focus POT` frames one pot's shoot.
  - `--orbit YAW,PITCH` sets the camera angles in degrees.
  - `--zoom F` scales the framing distance.
  - Example: `xvfb-run -a -s "-screen 0 1600x900x24" env WGPU_BACKEND=vulkan target/debug/plant-sim --clean --day 50.5 --focus 0 --zoom 0.55 --orbit -30,20 --screenshot near.png`. Each shot takes 1 to 3 min on lavapipe.

## Cost

`cargo bench -p ps-bench --bench mesh`, 90-day demo bean:

| Benchmark | Before | After |
| --- | --- | --- |
| Full rebuild of every axis | 58 µs | 104 µs |
| Instances | 0.86 µs | 1.7 µs |

- This is under the perf thread's 150 µs-per-plant line.
- The new GPU-side cost is one extra entity and mesh per living bean leaf (its petiole).
- If draw count matters on the GTX 1070, the fix is to merge each plant's petioles into one mesh per rebuild.
- The 1070 timing of the plot scene had not been done at handoff.

## Open and next steps
1. Done: root copies keep their depth and stay in the pot (#84); the sim-side wall rule for laterals is model 0.11.0 (#87).
2. **Leaf angle variety belongs in the sim.** The visuals research (`research/plant-visuals-review.md` items 1–2) wants two things:
   - age-dependent petiole elevation: about 1.15 rad young to −0.35 old, replacing `PETIOLE_ELEVATION = 0.5` in `ps-arch/src/shoot.rs`
   - larger per-leaf yaw, about ±0.55 rad, because 180° phyllotaxis puts every leaf in one plane

   Since #75, light depends on the blade angle, so this must be a sim change with a model version bump.

   Proposed handoff: a per-leaf blade normal or elevation in `ps_arch::Meta`, which the viewer reads into `blade_frame()` in `instances.rs`. A mockup is on branch `claude/project-thread-aqg9ej`, commit 886f61a, under `PS_MOCK=pose`.
3. **The pole row looks like a grid**, because its 8 plants grow in lockstep. Per-plant variation (sim seed or architecture) would help.
4. **Framing.** The plant is small in the default wide shot, so the change reads best in close-ups. Camera framing (target the canopy, narrower FOV) was the illustrated-look thread's work.
5. **Not done yet:**
   - heliotropic and night leaf folding
   - wheat blade twist and sheath
   - canopy darkening by `light_index`

## Gotchas
- `TemplateKey` literals need `unfold`. Use `TemplateKey::new(t)` for the open, turgid variant.
- `build_template_shaded` returns a per-vertex grey shade. The illustrated branch (`claude/project-thread-u7o4cj`) changed it to [brightness, ink]. Whichever merges second must reconcile `leaf_raw`, `flower()` and `pod()` in `templates.rs`.
- `target/debug` grows large. `cargo clean` frees space.
- Headless renders need `libudev-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-0 mesa-vulkan-drivers xvfb libasound2-dev`. Run `apt-get update` first.

## Renders
`docs/archive/viewer/look-options/fuller-plants/` holds the following.
- `pair-*.jpg`: before on the left, after on the right, with the same camera.
- `before-*` and `after-*`: the full-size shots.
