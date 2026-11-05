# 0025: Each leaf intercepts light by its own angle; obstacles leave the grid

- **Date:** 2026-11-05
- **Status:** accepted. The owner chose to fix the leaf-angle mismatch in M2 (decision
  0018's open item); the obstacle change follows the review of WP25.

## Context
The light model used one G-function value, the species file's 0.5 for spherical leaf
angles, for every leaf and every direction. The sim draws every bean leaflet tilted
about 19° from horizontal, a planophile canopy whose true G is near 0.9 for a high sun.
Against the Monte Carlo ray tracer on a 60-day sim plant the model was 9.9% too bright,
with 49 of 74 leaflets within 0.10 (decision 0018). A plot used the first plant's G for
every plant in it, so wheat and beans in one plot could not differ.

The review of WP25 found that obstacles, voxelized into the light grid, stretch a pot's
32-cell grid: a 2 m shade cloth turns a bean's 2 cm cells into 6 cm, and a wall a metre
away coarsens the plant's self-shading all season.

## Decision
1. **Own projection.** Leaf `i` intercepts a ray in direction `d` in proportion to
   `G_i(d) = |n_i · d|`, its blade normal against the ray. The one-blade extinction of
   decision 0008 uses it: `k = −ln(1 − G_i(d) ΔL)/ΔL`.
2. **Per-direction depth.** The grid keeps the optical depth of each occupied cell for
   each of the day's directions (the hours the sun is up, then the 16 sky sectors), in
   one run per direction. Empty cells point at a slot of zeros. Memory is occupied cells
   × directions, about 5000 × 32 for a 300-leaf plant.
3. **Absorbed light.** The beam on a sunlit leaf is `|n · s| · I_dir / sin β`, from a
   new per-hour field `LightCoeffs::beam` (sunlit-weighted over a leaf's leaflets). The
   diffuse flux absorbed is `Σ_s a_s |n · d'_s| T_s`, where `d'_s` sits at the
   radiance-weighted mean `sin β` of the sector's band, so a horizontal leaf in the open
   absorbs exactly the horizontal diffuse irradiance.
4. **Species parameter removed.** `g_function` is gone from the species files (schema 5
   → 6); `rebuild_daily`, `rebuild_plot` and `absorbed_par` lose their G argument.
5. **Obstacles out of the grid.** A ray adds `κ` times its length inside each obstacle
   box (slab method), wherever the box is. The grid is fitted to the leaves only.
6. **Version.** Model 0.8.0 → 0.9.0, goldens regenerated.

## Consequences
- Ray tracer, 60-day sim plant (pot grid): +3.8%, 63 of 74 leaflets within 0.10, worst
  0.26 (was +9.9%, 49, 0.32). Synthetic WP6 shoot with spherical normals: +2.8%, all 57
  within 0.10, worst 0.08 (was +1.7%, 56, 0.12). Plot grid: synthetic −0.7%, 54 of 57;
  sim plant −0.8%, 61 of 74 (was +5.6%, 51). The remaining pot error is the
  within-cell limit (who is on top inside one cell).
- The plant gets about 6% less light. Over world seeds 11 to 18 the bush bean averages
  26.5 g and 10.9 pods (28.0 g and 12.4 pods on 0.8.0); over seeds 7 and 11 to 15 the
  pole bean is 1.39 m tall at day 60 (1.43 m on 0.8.0). Both shifts are within about one
  standard error of the seed spread.
- Validation on seed 7 goes from 66 to 63 passes of 87, all Info, each at a band edge:
  `pole_height_day60` 1.498 m (band from 1.5), `ndfa_low_n_bed` 0.494 (from 0.5) and
  `drought_shoot_mass` −56% (to −52%). Every M0 check passes. No parameter is re-tuned
  here: the WP25 review showed that one seed cannot carry a tuning decision, and the
  pole bean already averaged under 1.5 m at day 60 on 0.8.0. Re-tuning waits for the
  multi-seed validation harness, the next PR.
- Cost: a 300-leaf rebuild takes 3.3 ms instead of 2.8 ms; plot_30_beans takes 23.5 ms
  per sim-day instead of 21.3; wheat_stand about 120 ms instead of 100.
- The light is now only as right as the drawn leaf angles. Every bean leaflet tilts
  about 19° and none tracks the sun, so heliotropism and a spread of leaf angles are
  open model questions.
- The viewer's ps-app call sites drop the G argument (three one-line edits).
