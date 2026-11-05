# Plant visuals research (2026-11-05)

Research into how games, botanical illustration and stylized-foliage practice draw plants, plus a review of our viewer and four mockups rendered in it. Asked for by the owner after the first look rounds read as "mostly colors or filters".

- `report.md`: the full report. Section "Prioritized suggestions" lists 18 items, and "Round 2" has what the mockups showed.
- `renders/`: before and after renders (day 45 two-pot scenario, the 30-bean plot at day 45 and the pole-bean trellis row at day 60) from the software renderer (lavapipe).
- `mockups-886f61a.patch`: the mockup code (ps-view only, on top of main at a12d5aa). Each experiment is behind `PS_MOCK=frame,pose,value,plate`. It is a sketch for ideas, not code to merge as is.
- `board-template.html`: the review board page. `{{name}}` placeholders stand for the images in `renders/`. The published copy lived at https://claude.ai/artifact/UR7MuxjDU8sHLHBD4eJ9zh in the original account.

## Findings in brief

1. **The problem is form, not shading.** Every leaf has the same elevation (`ps-arch/src/shoot.rs`, `PETIOLE_ELEVATION = 0.5` rad), and bean phyllotaxis is 180°, so every leaf sits in one plane. As a result the bean reads as a fir tree. Varying leaf elevation by age and adding a per-leaf yaw of about ±30° was the clear win in the mockups. If the elevation moves into ps-arch, the light lens stays true. Note: viewer look option C (PR #78) has since reworked petioles and blades, so re-check this against the current main.
2. **A paper "plate" view** (a seamless paper sweep in place of the garden) was the most visibly different look. It is a good base for an ink-line style and a screenshot mode.
3. **A hero camera** aimed at one plant's canopy matters more than set dressing. Muting the lawn and removing the tree ring helps the plant stand out.
4. **Canopy shading from the shade signal** is true to the sim but subtle on a 25-leaf plant.
5. **The pole-bean row is a perfect grid** because its plants grow in lockstep. Fixing this needs per-plant variation in the sim.

## Limits

Wikimedia Commons and PubMed Central were blocked from the research container, so there are no reference photos or historical plates. Elevation numbers are illustrative, not measured. A source for bean leaf angle by node would firm them up.

## How to re-render

On Linux with xvfb and Mesa lavapipe (see `docs/bevy-notes.md`):

```sh
git apply docs/research/plant-visuals/mockups-886f61a.patch   # may need rebasing onto current main
cargo build -p ps-app
PS_MOCK=frame,pose,value xvfb-run -a -s "-screen 0 1600x900x24" env WGPU_BACKEND=vulkan \
  target/debug/plant-sim --scenario data/scenarios/two_pot_sun_shade.ron --day 45.5 --clean --screenshot out.png
```
