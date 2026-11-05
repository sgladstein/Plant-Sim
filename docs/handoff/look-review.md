# Viewer look review: handoff

Thread "Review how to make the viewer look better", 2026-11-05. The full review is `docs/design/viewer-look-review.md` with renders in `docs/design/viewer-look/`, cherry-picked onto main by the handoff PR.

## How the workstream ran
- An independent design reviewer picked a direction, built a working prototype in ps-view on its own branch, and showed before/after renders on a review page. The viewer thread then cherry-picked or merged the work.
- Renders are headless with lavapipe. Run from the repo root, or the default scenario isn't found and you get the demo scene:
  `xvfb-run -a -s "-screen 0 1600x900x24" env WGPU_BACKEND=vulkan target/release/plant-sim --day 45.5 --lens off --screenshot x.png`
  The standard set is days 45.5 (noon), 45.8 (dusk) and 20.5, plus `--lens light` and `--lens roots`. The "before" binary is built from a git worktree of main.
- Round 1 was light and color, and round 2 was geometry and setting. Both are on main (#72).
- Round 3 ran three options in parallel threads:
  - A: illustrated (this branch).
  - B: painterly, PR #79, stacked on A.
  - C: fuller plants, PR #78, which changes form only.
  The owner picked **C with the natural style** "for right now". A and B are parked.

## Option A (branch `claude/project-thread-u7o4cj`)
- `ps-view/src/toon.rs` and `shaders/toon.wesl`: a `Toon = ExtendedMaterial<StandardMaterial, ToonExt>` material.
  - It has a cel ramp times the real shadow, a warm rim and a cool fill.
  - It draws ink on silhouettes and on edges flagged in vertex alpha (leaflet margins, from `ps_mesh::templates::build_template_shaded`).
  - Background surfaces are flat and pale.
  - A `Style` resource switches it at runtime; on the command line, use `--look natural|illustrated`.
- `look::Backdrop`, `--backdrop plate`: a seamless paper "botanical plate" backdrop.
- `scene.rs canopy_shade`: darkens each leaf by its PAR relative to the plant's brightest leaf.
- `camera.rs`: FOV 0.5.
- To revive it, merge main into the branch, then fix conflicts in scene.rs and templates.rs; C restructured leaf_raw but kept the leaflet grid. Force-push is blocked by auto mode, so merge rather than rebase.

## Findings to keep
- The owner judges by form, framing and setting. Color grading alone reads as "the same graphics".
- These work with the natural style and are worth porting: the tighter camera, the plate backdrop as a screenshot mode, and canopy_shade.
- Lens colors must stay unlit or emissive so they match their legends.
- Leaf pose (age-dependent elevation) is the biggest form win. It waits on a per-leaf elevation in the sim's Meta.
- Still open: roots X-ray as a light box, a root depth cue, a moonlit night fill, a warm egui theme and wood grain.
- Upward roots: the suspect is ps-mesh `copy_transforms` spinning cohort copies about a horizontal parent root. The implementation thread has it.

Images: `docs/design/viewer-look/` (before and after round 1, round 2 `v2-after`, illustrated `v3-illustrated-*`, plate `v3-plate`).
