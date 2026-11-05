# Viewer look review

> **Status at handoff (2026-11-05).** Three rounds. Round 1 (light and color) and round 2 (bench, lawn, tree ring, finer leaflets) are merged on main (#72). Round 3 explored stylized looks in parallel: **illustrated** (cel shading and ink, this review's branch `claude/project-thread-u7o4cj`), **painterly** (PR #79, stacked on that branch) and **fuller plants** (PR #78, form only). The owner chose **fuller plants with the natural style** "for right now". The illustrated and painterly branches are parked, not merged. Live review page (old account): https://claude.ai/artifact/JrNdkF6T17RumErir7hPzW; this file is its full text.
>
> Parts of the illustrated branch could be reused with the natural style: the tighter camera (FOV 0.5 in `camera.rs`), the paper "plate" backdrop (`--backdrop plate`, `look::Backdrop`), and `canopy_shade` (darkens each leaf by its PAR relative to the plant's brightest leaf). Also reusable: the research thread's leaf-pose mockup (age-dependent elevation), which is waiting on a per-leaf elevation in the sim's Meta. Open items: roots X-ray as a light box, root depth cue, night fill, UI theme, wood grain.

## Round 1: soft naturalism

Visual design review of the M1 viewer (PR #26, merged as 888e803), 2026-11-05.
Before/after renders: `docs/design/viewer-look/`.
Prototype: branch `claude/project-thread-u7o4cj` (one commit, 2aba35d, on top of main). Not a PR; the viewer thread cherry-picks it.

## The decision

**Direction: soft naturalism.** The plants should look like a real bean specimen on a garden bench on a hazy summer day, rendered cleanly: true colors, soft light, matte surfaces, no textures, no outlines, no painterly filter. Think of a good nature-documentary macro shot, simplified.

The owner asked for "less photorealism and not stylistic" and left the call to me. I read that as: drop the hard, glossy, default-engine look, but don't go cartoon. This direction sits between the two and also serves the plan's "botanical illustration meets cozy indie game" brief, because it keeps the plant's true shape and color (the illustration half) and adds soft light and a warm setting (the cozy half).

Rejected:

- **Photorealism.** Procedural leaves with no textures look uncanny next to real-looking light; the shader and texture cost grows with every plant (30 in M2) on a GTX 1070; and realistic lighting fights the lenses, because a lens color in shadow no longer matches its legend.
- **Toon or cel shading with outlines.** The owner said "not stylistic". Cel bands also turn a smooth color scale into steps, which misleads on lenses.
- **Painterly or watercolor post-processing.** Costly full-screen passes, and the paper texture smears thin roots and lens colors.

## What was wrong (before)

Renders of the merged viewer at day 45 noon, day 45 dusk, day 20 and day 56:

1. **Leaves went near-black.** The only fill light was a flat ambient term, so any leaf facing away from the sun or in self-shadow dropped to almost black. The canopy read as a dark blob.
2. **Lens colors lied.** Light lens leaves at the low end of viridis are dark purple; put them in shadow and they become black, which the legend does not contain. Shading moved values along the scale.
3. **The soil cutaway, the science hero, read as a grey empty box.** The soil was a hollow shell of dark faces seen through a glass pane that added a whitish film; with the sun overhead the faces sat in shadow. Roots were faint grey lines.
4. **Default engine stage.** An endless beige plane met a flat blue sky in a hard diagonal line, and it continued under the right-hand panel.
5. **Blocky shadows.** Bevy's default four shadow cascades span 150 m; this diorama is 1 m across, so shadow texels were large.
6. **Dusk was a black silhouette** on a pink sky.

## What the prototype changes (after)

All in ps-view. The renders in the folder above are the prototype.

| Area | Change | Where |
| --- | --- | --- |
| Fill light | A procedural sky cubemap (zenith blue to warm horizon haze to ground) feeds `GeneratedEnvironmentMapLight`, a soft hemispheric fill. Flat ambient drops to a small floor. | `look.rs`, `sky.rs` |
| Background | The same gradient drawn as a `Skybox`, retinted with the time of day (warm at dawn and dusk, greyer under cloud, blue-black at night). | `look.rs`, `sky.rs` |
| Ground | The hard-edged table plane is replaced by a small wooden deck on an 80 m ground plane, and `DistanceFog` (linear, 3 to 14 m, colored like the horizon) melts the ground into the sky. | `scene.rs`, `look.rs` |
| Shadows | One cascade, 0.01 to 5 m. Sharper and cheaper than four over 150 m. | `sky.rs` |
| Tonemapping | `KhronosPbrNeutral`: base colors and lens colors come out close to their values, so the legend matches the leaves. | `look.rs` |
| Leaves | Roughness 0.8, reflectance 0.22, `diffuse_transmission` 0.35, so backlit leaves glow green instead of going black. Leaf greens lightened slightly; stems less acid. | `scene.rs`, `garden.rs` |
| Lens readability | When a lens colors an organ, 70% of the color is emissive and 30% is lit, so shading keeps the form but cannot push a value along the scale. Stems and roots go unlit under a lens. | `scene.rs`, `look.rs` |
| Soil | Soil faces are a grid with depth color plus smooth mottling (vertex colors, no textures). A translucent soil haze sits just behind the glass, so the box reads as full of soil and deeper roots look buried. The haze hides in the Roots X-ray lens. The glass itself is nearly invisible (a faint sky reflection only). | `diorama.rs`, `scene.rs` |
| Roots | Paler cream, and a 0.5 mm minimum drawn radius (was 0.3 mm), so they read against soil. | `garden.rs`, `scene.rs` |
| Color keys | Material palette keys are quantized on a square-root curve, so dark greens do not band. | `scene.rs` |
| Post | A light vignette (0.22). | `look.rs` |

Checks run: `cargo clippy -p ps-view -p ps-app --all-targets -- -D warnings` is clean and `cargo test -p ps-view` passes (10 tests). Renders were made with lavapipe under xvfb; they have not been seen on the GTX 1070 yet.

## Next steps, in order

These are for the viewer thread. 1 to 3 are the prototype itself; the rest are new work.

1. **Cherry-pick the prototype** (2aba35d) onto a viewer branch. Tune the numbers by eye on the 1070: env intensity (`40 + 1100 × daylight`), skybox brightness (`120 + 900 × daylight`), fog range, `LENS_GLOW`, haze alpha (0.42).
2. **Fix the dusk horizon seam.** At dusk the fogged ground comes out a cooler, darker color than the sky texel at the horizon. Compute the fog color from the same `sky_gradient` at y = 0 times the same scale, or swap fog for `FogFalloff::from_visibility_colors` with the sky's horizon color.
3. **Add a unit test** that `sky_tint` at noon is near white and that the cubemap builder makes 6 × 32 × 32 texels (the GPU parts can't be tested).
4. **Roots X-ray as a light box.** In X-ray, darken the soil interior to a near-black warm brown (and dim the deck) so colored roots glow, the way root scientists photograph rhizoboxes.
5. **Depth cue on roots.** Fade each root vertex's color toward the soil color by its distance from the glass (0 at the glass, 70% at the back wall). This shows the 3D root system in the natural view without X-ray. Vertex colors only, no extra pass.
6. **Canopy shading from the sim.** Darken each leaf's diffuse color slightly by its own shaded fraction from ps-light (inner leaves get less sky). That is ambient occlusion computed by the science model, so it is both cheap and true. Off under lenses.
7. **Leaf shape.** Give blades a slight cup (raise the edges 5 to 10% of the half-width), a midrib drawn as a lighter vertex-color line, and more outline vertices. This is what makes a bean leaf read as a bean leaf from the default camera.
8. **Night.** Keep a moonlit blue fill (ambient floor plus a weak cool directional) instead of near-black, so a paused night frame is still readable.
9. **UI theme to match the scene.** Warm charcoal panels instead of cold slate, an opaque right panel (today the ground shows through it), and a sage accent for the selected speed instead of the stock green. The plan's §4 already calls for a custom egui theme.
10. **Camera.** Try a narrower field of view (0.5 rad instead of 0.7) at a longer distance: less perspective distortion on the boxes, more of a product-shot look.
11. **Wood.** Faint grain stripes by vertex color on the frame and deck, and slightly cooler wood so it doesn't compete with the soil.
12. **Later.** Wind sway (already in the plan), a photo mode with depth of field for screenshots and time-lapses, and impostors in M4.

Kept out on purpose: SSAO. It needs MSAA off in Bevy, and the thin roots and stems alias badly without MSAA. Items 5 and 6 give most of the same depth cues for free.

## Performance on the GTX 1070

Not measured on the card yet; these are estimates from what each feature does.

- Skybox: one cheap full-screen pass. Under 0.1 ms at 1440p.
- `GeneratedEnvironmentMapLight` from a 32 px cubemap: the GPU filters it when the image changes, and only the backdrop image changes with the time of day (the fill image is fixed). Per pixel it adds two cube samples.
- Fog and diffuse transmission: a few ALU ops in the existing PBR shader.
- Vignette: one post pass, about 0.1 ms.
- One shadow cascade instead of four: cheaper than before.
- Soil grid: about 4,000 quads per pot, built once.
- Lens materials: same quantized palette as before, so batching is unchanged.

Expect the whole look to cost under 1 ms per frame against the 16 ms budget. The performance thread should confirm with the 1070 build (the `plant-sim-windows` CI artifact) at 1440p.

## Round 2: shapes and setting (merged, #72)

The owner said round 1 "mostly looks like changing colors or filters", so round 2 changed geometry and setting rather than grading: a slatted potting bench (`ps-view/src/dressing.rs`), a lawn of about 20k grass blades, pebbles, a fogged ring of hedges and trees, and rebuilt bean leaflets (folded along the midrib, wavy margins, acuminate tips, asymmetric side leaflets on petiolules, vertex-color veins via `ps_mesh::templates::build_template_shaded`). Renders: `viewer-look/v2-*.jpg`.

Lesson for future look work: this owner judges by form, framing and setting, not color grading. Show visibly different geometry.

## Round 3: illustrated (explored, parked)

Commits 0c9ed89 and ae2aba3 on `claude/project-thread-u7o4cj`. Renders: `viewer-look/v3-*.jpg`.

- `ps-view/src/toon.rs` and `shaders/toon.wesl`: `Toon = ExtendedMaterial<StandardMaterial, ToonExt>`. Two-step cel ramp from the sun times the real shadow, warm rim, cool fill, diffuse transmission for backlit leaves, ink on silhouettes (n·v) and on edges flagged in vertex alpha. A `Style` resource (Natural or Illustrated) flips every Toon at runtime; `--look natural|illustrated` in ps-app.
- `ps-mesh/src/templates.rs`: `build_template_shaded` returns brightness and ink per vertex; leaflet margins are inked.
- `scene.rs`: lawn and ground use `toon_background` (flat, paler, desaturated) so the plant is the figure. `canopy_shade` darkens leaves by their own PAR from ps-light (illustrated, lens off). Lens materials stay unlit.
- `camera.rs`: FOV 0.5 rad, tighter framing.
- No extra render passes. Same cost class as round 2.

Still open: dome normals on leaves for softer ramps, an organ-id outline pass if mesh ink is not enough, the roots X-ray light box, and a botanical-plate mode.

### Botanical plate backdrop (commit 524bbb6)

`--backdrop plate` (`ps_view::look::Backdrop`, set at startup) replaces the garden with a seamless paper sweep. The bench and lawn are hidden, the sky cubemap, ground, fog and clear color are all `PAPER`, and the ground sits at the pot base. From the research thread's mockups. Renders: `viewer-look/v3-plate-*.jpg`. At dusk the paper dims with the light, which is fine for a screenshot mode.
