# Handoff: plant visuals research

Thread "Research plant game and art visuals", 2026-11-05. Everything is in the repo at `docs/research/plant-visuals/`, on branch `claude/project-thread-aqg9ej` (PR into main). That folder holds the report, before/after renders, the review-board page template and the mockup code as a patch.

## Key findings, ranked

1. **Plant form matters more than shading or filters.** Two things made the bean look like a fir tree. Every leaf had the same pose, and bean phyllotaxis is 180°, so all leaves sat in one plane.
2. **Leaf pose variation was the clear win in the mockups.** It needs elevation that changes with leaf age (young near 1.15 rad, old drooping below level) plus a per-leaf yaw of about ±30°.
3. **A paper "plate" view was the most visibly different look.** It replaces the garden with a seamless paper sweep, which makes a good base for ink lines and a screenshot or plate mode.
4. **A hero camera on one plant beats set dressing.** A muted lawn with no tree ring also helps the plant stand out.
5. **Canopy shading from the sim's shade signal is true to the data but subtle** on a small plant. It should pay off under cel bands and in dense plots.
6. **True botanical details are the charm:** pink nodules, unfolding leaves, pulvini, leaflets that track the sun, and leaves that fold at night.
7. **Line work for an ink look:** heavy silhouette lines, light interior lines, no lines on roots. Add an organ-id buffer, because depth and normal edges miss leaf-over-leaf edges. Lens fills stay flat.

## Adopted on main (as of 2392f58)

- PR #78 (viewer look option C) added:
  - sim-length petioles with pulvini
  - swollen nodes
  - unfolding young leaves
  - pink nodule clusters
  - per-leaf ±6% value and warm/cool offsets
  - a hero camera (`--focus POT --orbit YAW,PITCH --zoom F`)
  - a per-leaf yaw jitter of ±0.14 rad (about 8°)
- PR #78 also holds leaf blades at the light model's 19° tip-down tilt, per decision 0025. Light now depends on that tilt, so pose changes must go through the sim.
- PR #80 (decision 0027) gives each plant its own seed, emergence and microsite. That should break the lockstep "grid" in the pole-bean row. Re-render to check.

## Not adopted yet (candidates)

- **Leaf elevation by age (sim change in ps-arch).** This would feed both the light model and the viewer, and needs a decision record and model version bump. Source the angles first; the mockup values are illustrative.
- **A larger yaw or petiole twist** (about ±30° against today's ±8°). Check it against the light model, since blade orientation now matters there.
- **Plate view** as a screenshot or plate mode: paper sweep, ink lines, scale bar, roots below.
- **Canopy darkening from the shade signal, and dome normals for cel bands.**
- **Organ-id buffer for outlines,** if the ink look goes ahead.
- **Sun-tracking (paraheliotropic) leaflets and night leaf folding.** Sources: Yu & Berg 1994, Plant Physiology 106:1567; the circumnutation atlas for runner bean.
- **Wheat blades:** twist, tip droop, sheath, tiller fan, and ripening from the base up.

## How to run the mockups

```sh
git apply docs/research/plant-visuals/mockups-886f61a.patch   # written against a12d5aa; main has moved (PR #78), so expect conflicts in scene.rs
cargo build -p ps-app
PS_MOCK=frame,pose,value,plate xvfb-run -a -s "-screen 0 1600x900x24" env WGPU_BACKEND=vulkan \
  target/debug/plant-sim --scenario data/scenarios/two_pot_sun_shade.ron --day 45.5 --clean --screenshot out.png
```

Flags, comma-separated:

| Flag | What it does |
| --- | --- |
| `frame` | Narrower FOV (0.42), lower pitch (0.16), aim at the canopy, muted lawn and ground, no tree ring |
| `pose` | Leaf elevation by age, plus yaw ±0.55, pitch ±0.14 and roll ±0.22 rad |
| `value` | Leaf shading by shade signal, plus per-leaf hue |
| `plate` | Paper sweep in place of the garden |

Headless setup needs `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-0 mesa-vulkan-drivers xvfb` (run `apt-get update` first).

## Ready-to-run brief: the next pass

> Re-render main (after #78 and #80) with `--focus 0 --orbit 0,10 --zoom 1.2` at day 45 and 60 for the two-pot, plot_30_beans and m2/pole_row_trellis scenarios. Compare with docs/research/plant-visuals/renders/. Then prototype, behind a flag in ps-view: (1) plate mode as a screenshot option, (2) yaw jitter raised to ±0.5 rad, with a check of how much the light lens changes, and (3) shade-signal canopy darkening under the current cel or illustrated look. Before proposing leaf-elevation-by-age as a sim change, find a source for bean leaf elevation by node or leaf age, and write the decision record. Collect 10-20 reference images (bean, pole bean and wheat habit photos; Thomé or Köhler plates; Viridi or Botany Manor screenshots) from a machine that can reach Wikimedia Commons; the research container could not. Deliver before/after renders and a short verdict per change.
