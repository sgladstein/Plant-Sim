# Plant visuals: what games and plant art do, and what to change in our viewer

Research and review, 2026-11-05. Asked for by Scotttt after the first two look rounds "mostly look like changing colors or filters". The viewer is now moving to an illustrated look (cel shading, ink lines, warmer palette), so this report weighs **shape, form and fullness first** and shading second.

Reviewed: renders in `/mnt/project-files/viewer/` (round 2, real-sim day 45, 30-bean plot, pole row, wheat) and `/mnt/project-files/design/viewer-look/`; the look review; `ps-mesh` templates and instances, `ps-view` garden colors, `ps-arch` shoot geometry on `main` (d5ab169); a fresh `ps-cli` run of `two_pot_sun_shade` to day 50.

## The short version

Our plants look like a stack of identical flat leaves on a pole. That is a **form** problem, and no shader (cel, ink, painterly) fixes it. Three causes, all findable in code:

1. **Every leaf sits at the same angle.** `ps-arch/src/shoot.rs:19` sets `PETIOLE_ELEVATION = 0.5` rad for every leaf, and the mesh template holds every blade at the same pitch (`templates.rs`, `pitch = 0.17`). Same angle, same size ratio, same pose: the plant reads as a fir tree or a candelabra, not a bean mound.
2. **The plant is tiny in the frame and the setting outshouts it.** In round 2 the bean fills about 15% of the frame height; the grass is brighter, busier and the same hue. In the four-pot shot the plants are a few pixels wide.
3. **No variation between leaves.** One green per age, one shape, no paler undersides, no darker interior. Real canopies read as full because of value layering (light outer leaves, dark interior), not because of leaf count.

The sim numbers are honest and small (day 45 in a 20 cm pot: 25 leaves, 0.052 m² leaf area, 23 cm tall, 2.9 g dry mass), so **do not inflate sizes to look fuller**. Fullness has to come from pose, overlap, value and framing.

## What the references teach

### Games

- **Sable (Shedworks).** Flat 2D-style shading means depth must be rebuilt with lines, fog and cast shadows. Outlines are used where forms overlap and fade with distance so things don't pop in; fog was "really, really key" for readability. Lesson: with a cel look, **lines and value layering do the depth work** that soft lighting did before. [Game Developer](https://www.gamedeveloper.com/marketing/how-shedworks-refined-the-art-of-sable-in-pursuit-of-readability)
- **Moebius-style post process (Heckel).** Practical recipe: Sobel on depth and normal buffers, depth weighted about 25× normals, a line-thickness sample offset, low-frequency UV wobble for a hand-drawn line, luma thresholds for bands. [blog](https://blog.maximeheckel.com/posts/moebius-style-post-processing)
- **Stylized tree breakdowns (Girard and the "spherical normals" practice).** Foliage looks full when (a) the normals of a leaf clump are bent toward a sphere or dome around the clump, so the whole canopy shades as one soft volume, (b) the interior is darkened by a "shell gradient" (fake occlusion), (c) each leaf gets a random color offset, (d) a top-to-bottom gradient and a rim light are added, and (e) extra leaves are pushed to the silhouette. "Always mess with the normals." [ArtStation](https://cremuss.artstation.com/projects/Vyz12R); see also [Unreal forum thread](https://forums.unrealengine.com/t/stylized-fluffy-tree-style/625736)
- **Tiny Glade (Pounce Light).** Cozy, Ghibli-like "fuzzy warmth"; started in Bevy and moved to its own GPU pipeline for prettier procedural output. Their stated principle: design from the target look and feel, then build the pipeline. [80.lv interview](https://80.lv/articles/exclusive-tiny-glade-developers-discuss-bevy-proceduralism-publishers-cozy-games)
- **Botany Manor (Balloon Studios).** Every invented plant is drawn from a real reference plant (fishbone cactus, ghost plant, pine lily). Lesson for us, the inverse: our plants are real, so **real botanical tells are the art** (pulvini, cordate unifoliates, paler undersides, nodules). [Journal of Geek Studies interview](https://jgeekstudies.org/2025/02/09/botany-manor-a-cozy-game-unveiling-the-inequalities-in-academia/)
- From play knowledge, not re-verified this session: Viridi and Garden Life frame one plant large, close and low; cozy games keep the background desaturated and simple so the subject is the only saturated thing on screen.

### Botanical illustration and scientific figures

- **Structure before surface.** "Map habit, axes, nodes and attachment before outlines, veins or surface pattern." Gaps between branches "often reveal errors sooner than leaf detail". Don't "regularise intervals". Foreshortening changes the whole leaf. Use fewer veins, not equal dark veins pasted everywhere. [thehistoryofart.org](https://www.thehistoryofart.org/drawings/botanical/)
- **The plate.** A classic botanical plate shows habit, then dissected parts (flower, pod, seed, root) with scale bars, on plain paper. Ink outline for structure, watercolor wash for value, highlights left as paper. [Fiveable summary](https://fiveable.me/history-of-scientific-illustration/unit-6/techniques-conventions-botanical-illustration/study-guide/T78xKyhin9vuIg9A)
- **Rhizobox and root-scan imagery.** Root scientists show roots pale against dark soil or as dark lines on white. Our earlier look review already proposed the dark "light box" X-ray; that still stands.
- **Procedural plant science (Prusinkiewicz, Palubicki).** Already summarized in `research/notes/procedural_visuals_games.md`: generalized-cylinder stems, leaf templates with developmental parameters, Crysis-style vertex wind. Nothing there changes; this report adds the art layer.

## Prioritized suggestions

Ordered by visible impact per hour. "Sim" means it touches the science crates and needs the implementation thread and a decision record; "view" is render-only.

### A. Form and pose (do these first; they are what Scotttt will notice)

1. **Vary leaf elevation with age (sim, high impact).** Replace the fixed 0.5 rad with an elevation that starts near vertical for an unfolding leaf (about 1.2 rad), settles to about 0.5 when mature, and falls below horizontal for old lower leaves (about −0.2). That single change turns the candelabra into a dome: young leaves cup upward at the top, old leaves spread out and down at the bottom. It's real (leaf epinasty with age) and it changes light interception, so it belongs in ps-arch with a model-version bump, and the light lens stays true.
2. **Per-leaflet pose jitter (view, cheap).** Deterministic from organ id: ±12° roll and ±10° pitch per leaflet, ±15° yaw for the laterals. Small enough not to falsify the light lens; enough to break the "pasted" look. Do it as instance rotation, not new meshes.
3. **Petiole length from the sim, not 0.8 × leaflet.** Bean petioles are long and are what push the leaves out into a spread canopy. ps-arch already has a petiole length in `leaf_points`; pass it into the instance (scale the petiole part separately from the blade, or build petiole as its own tube like stems).
4. **Heliotropic leaflet tilt (sim later, view stand-in now).** Bean leaflets turn on their pulvini to face the sun in the morning and edge-on at hot noon (paraheliotropism), and fold down at night (nyctinasty). A noon frame where top leaflets tilt edge-on and a night frame where every leaflet hangs is a gorgeous, true, day-cycle cue that no other garden game has. Render-only first (driven by sun direction and the water-stress factor), sim later if the light model wants it.
5. **Visible nodes and a zigzag stem.** Bean main stems kink slightly at each node and each petiole has a swollen pulvinus at its base. Add a 5–8° direction alternation per internode and a small bulge at petiole base. This is what makes the structure read under ink lines.
6. **Frame the subject.** Default camera: target the plant canopy, not the pot center; plant fills 50–60% of frame height; lower the camera to about pot-rim height with a 15–20° look-down; narrower FOV (0.45–0.5 rad). The pot and soil cutaway stay in frame below. For multi-pot layouts, frame to the canopy bounds, not the bench.
7. **Pole bean row: break the grid.** Today eight strings carry identical leaves at identical heights, which reads as wallpaper. Leaves should hang off the vine on long petioles, alternate sides, and droop outward (item 1 does most of this). Draw the twining stem thicker and darker than the string so the coil reads. If node heights are in sync because seeds are identical, a little seed variation per plant (already in the sim) should show.
8. **Wheat blades: twist, droop and sheath.** Give each blade a half-turn twist along its length and a tip droop that grows with length; wrap the lower part as a sheath around the stem; fan tillers out from the base at 10–25°. Ripening should yellow from the lowest leaves upward, with ears last. Today the blades are rigid flat ribbons.

### B. Value and color (makes the form read)

9. **Canopy value layering from the sim (view).** Darken each leaf by its own light index from ps-light (it is already in the organ data: `light_index`). Inner, shaded leaves go darker and cooler; outer, sunlit leaves lighter and warmer. This is the "shell gradient" trick, but true. Off under lenses. This was item 6 in the earlier look review; it matters more now because cel bands flatten everything else.
10. **Dome normals for the canopy (view, for the cel look).** Blend each leaf's shading normal 30–50% toward the direction from the plant's canopy center to the leaf. Cel bands then fall across the whole bush as one form (lit side, shadow side) instead of flickering leaf by leaf. Keep true normals for the light lens.
11. **Leaf color variety.** Pale, slightly grey-green undersides (bean undersides are visibly paler); a ±4% hue and value offset per leaf from organ id; young leaves yellow-green (already there); old leaves bluer and darker; senescing leaves yellow with brown margins.
12. **Figure and ground.** The subject should be the most saturated, highest-contrast thing on screen. Desaturate and lighten the grass and background by about 30–40% (or cut the lawn to a short ring and use a warm paper-colored backdrop), and drop the faceted low-poly trees to flat value silhouettes; their facets read as "asset store" and compete with the bean.

### C. Lines (for the illustrated look)

13. **Line hierarchy, as in botanical ink drawing.** Heavy line on the plant's outer silhouette and the pot; medium on overlapping leaf edges; thin or none on veins and folds; no lines on roots (they are already thin lines) or draw the roots themselves as ink strokes. Fade line weight with distance.
14. **Use an organ-id buffer for edges, not just depth and normals.** Two overlapping leaves at similar depth make no depth edge, and flat leaves make weak normal edges, so a depth-plus-normal Sobel will miss exactly the leaf-over-leaf edges that define a canopy. Bevy's prepass gives depth and normals; add a small id target (organ id hash) and edge on id change. The picking code already has organ ids.
15. **Lenses keep their colors.** Lines stay a neutral warm dark in every mode; cel bands apply only in natural mode; lens fills stay flat (the current emissive mix) so the legend matches.

### D. Science details that are also charm

16. **Root nodules.** The sim has about 260 nodule organs per bean, and the viewer colors them like roots. Draw them as small pink-tan spheres on the roots; pink is the true color of active nodules (leghemoglobin). An instant "oh, that's real" moment, and it ties to the nitrogen lens.
17. **Unfolding leaves.** Emerging trifoliates are folded along the midrib and hang; they open as they expand. Drive the template fold from expansion fraction (area over potential area).
18. **Botanical plate mode.** A screenshot mode that renders the selected plant on a paper-colored background in the ink style, with a scale bar, the root system below, and its pods and a seed drawn enlarged beside it, plus the species, day and genotype as a caption. Cheap (same renderer, new backdrop and camera), very shareable, and exactly what the "botanical illustration meets cozy game" brief asked for.

## Suggested order for the look thread

1. Items 6, 12 (framing, figure and ground): an afternoon, and the biggest single "this looks different" change.
2. Items 2, 3, 5, 11 (render-only form and color): a day.
3. Item 1 (age-dependent elevation): ask the implementation thread; it's a sim change with a decision record.
4. Items 9, 10, 13–15 together with the cel and ink pass.
5. Items 4, 7, 8, 16, 17, then 18.

Performance: all of the above is per-instance data or a vertex-shader tweak, except the id buffer (one extra small render target in the prepass, well under 1 ms at 1440p on a GTX 1070) and the outline pass itself.

## Caveats

- The Viridi and Garden Life notes and the bean pulvinus, nyctinasty and paraheliotropism claims come from prior knowledge, not sources read this session. The bean behaviors are textbook plant physiology, but the research thread should cite a source before item 4 goes into the sim.
- The JoVE rhizobox protocol page would not load, so the root-imaging note relies on the earlier look review.
- Elevation values in item 1 are illustrative starting points, not measured bean data.

## Round 2 (deeper pass, 2026-11-05 afternoon)

Board with before/after renders: https://claude.ai/artifact/UR7MuxjDU8sHLHBD4eJ9zh . Renders: `/mnt/project-files/research/visuals-mockups/`. Mockup code: branch `claude/project-thread-aqg9ej`, commit 886f61a, ps-view only, behind `PS_MOCK=frame,pose,value,plate` (not for merge as is).

What the mockups showed, rendered in our own viewer:

- **Leaf pose is the clear win.** Age-dependent elevation (young ~1.15 rad, old ~-0.35) plus per-leaf yaw up to ±31°, pitch ±8° and roll ±13° turns the fir-shaped stack into a rounded bush. The yaw matters most: bean phyllotaxis is distichous (180°, `bush_bean.ron`), so without a petiole twist every leaf sits in one plane. Put the elevation in ps-arch (sim change, light lens stays true); the twist can stay render-only if small.
- **Paper "plate" view is the most visibly different look.** The garden swapped for a seamless paper sweep. Strong candidate for a screenshot or plate mode and a base for the ink-line look.
- **Framing and a muted background help**, but in the two-pot layout the pots still take much of the frame; a hero camera on one plant's canopy matters more than set dressing.
- **Canopy shading from the shade signal is subtle** on a 25-leaf plant; it should matter more under cel bands and in dense plots.
- **Pole row is still a grid**: the eight plants grow in lockstep. That needs per-plant variation from the sim.

Added sources: Dorfromantik (value-first palettes, silhouette test), Alba (normals from smooth proxies, gradient colours), Meadows (sphere-normal canopies, sharpened shadows for a cel look), Viridi (single potted plant as the stage), BotW (stylized but information-dense), Yu & Berg 1994 Plant Physiology 106:1567 (light and heat raise bean leaflets; drier soil makes it stronger), circumnutation atlas (runner bean: ~90 min shoot circling, leaves drop 19-20h and rise 03-04h). Image hosts (Wikimedia Commons, PubMed Central) were blocked from the research machine, so there are no reference photos or historical plates.
