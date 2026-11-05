# Look option B: painterly soft (PR #79)

Status (2026-11-05): an option to compare. **Not merged; the owner picks one of the looks
(natural, illustrated, painterly) before anything merges.** The branch
`claude/project-thread-fml7eu` is stacked on the illustrated option
(`claude/project-thread-u7o4cj`, which adds `--look` and `ps_view::toon::Style`).

## Run it

```text
plant-sim --look painterly [--day 45.5] [--lens light] [--view 25,-4] [--screenshot x.png]
```

`--view YAW,PITCH` (degrees) sets the camera angle for screenshots; a low pitch shows the sky.
Headless renders use xvfb + lavapipe (docs/bevy-notes.md).

## How it works

- `crates/ps-view/src/painterly.rs` + `painterly.wgsl`: one full-screen pass after tonemapping
  (Bevy `FullscreenMaterial`): edge wobble, soft-weighted 4-sector Kuwahara (brush patches; part
  of the source pixel kept so roots survive), soft value steps in sqrt-luminance, muted
  watercolor palette (cool shadows, warm lights, lifted darks), paper grain/blotches, faint
  pigment pooling. No outlines. Toon strength stays 0 (PBR) for this style.
- Painted sky: `painterly::sky_gradient` baked into a 128 px cubemap once (cached in
  `look::styled_sky_cubemap`), retinted with time of day; fog matches `painterly::horizon`.
- With a lens on: no value steps, palette shift or pooling, smaller patches, so lens colors
  match their legends.

## Cost

About 69 texture reads per pixel at 1440p with lenses off (radius 3: 64 Kuwahara + 1 + 4), about
41 with a lens on. Not yet timed on the GTX 1070. If too slow, run the Kuwahara at half
resolution.

## Known weak spots / next steps

- Dusk (day 45.8): leaves sink into the background more than in natural.
- It is a paint treatment over the same shapes. The owner earlier asked for visible shape
  changes, not just filters; if this direction is chosen, pair it with painted shapes (softer
  tree canopies, brush-stroke lawn) and the illustrated branch's sim-driven `canopy_shade`.

Renders: `compare-*.jpg` here (left natural, right painterly; days 20.5, 45.5 off/light/roots/low
view, 45.8, 56.5 shade).
