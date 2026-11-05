# Option B painterly — section from the painterly thread, relayed verbatim

State: PR #79 is open, CI is green, and it is NOT merged. It's one of three looks the owner is choosing between: natural, illustrated, painterly. Branch claude/project-thread-fml7eu is stacked on the illustrated branch claude/project-thread-u7o4cj, which has the --look flag and ps_view::toon::Style. It's up to date with that branch at 8223c2f. Everything is pushed.

What it is: `plant-sim --look painterly` adds one full-screen watercolor pass after tonemapping (crates/ps-view/src/painterly.rs and painterly.wgsl) plus a painted sky cubemap. Lens colors stay true. Also added `--view YAW,PITCH` for screenshots from a set angle.

Docs and renders: docs/look-options/painterly/README.md and its compare-*.jpg files, all on the branch. Copies are also in project files at viewer/look-options/painterly/.

Open: the frame cost on the GTX 1070 hasn't been timed. It's about 69 texture reads per pixel at 1440p, and half-resolution Kuwahara is the fallback. At dusk the leaves fade into the background. If the owner picks painterly, the next steps are painted shapes (softer canopies, a brush-stroke lawn) and turning on the illustrated branch's canopy_shade for this style too. If they pick another look, close #79.
