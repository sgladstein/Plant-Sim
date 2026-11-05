# Review: each leaf intercepts by its own angle; obstacles leave the grid (PR #75, decision 0025)

Reviewed at commit ccad33c on `claude/project-thread-ur2udi` against `origin/main` (model 0.8.0).
Reviewer: Fable review thread. Date: 2026-11-05.

## Verdict

**Approve.** This closes the open item from decision 0018 (one G of 0.5 for a canopy drawn at
19° tilt) the right way: the per-leaf projection `|n · d|` is the exact G-function of a flat
blade, both faces count, the one-blade extinction keeps its form with `g` in place of `G`, the
sunlit beam is `|n · s| · I_h / sin β`, and the diffuse term reduces to the horizontal diffuse
irradiance for a horizontal leaf in the open. The grid holds one depth per occupied cell and
direction with a shared zero slot, and rays read one direction's run at a time; the thread-count
test still holds. Obstacles are now exact at any distance by the slab test and the grid is
fitted to the leaves only, as asked in the WP25 review. Species lose `g_function` (schema 6),
so a plot with wheat and beans no longer borrows the first plant's G. Nothing is re-tuned, which
is right after the seed-noise finding.

## What I checked

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| clippy, sim and CLI crates, `-D warnings` | clean (CI covers the Bevy crates; the three `ps-app` call sites are one-line edits) |
| `cargo test` on the sim and CLI crates | all pass, including the ray tracer comparisons, `a_tilted_leaf_shades_by_its_projection`, `projection_is_the_cosine_to_the_ray`, `an_obstacle_outside_the_grid_still_shades` |
| `scripts/check_deps.py`, `scripts/check_determinism.py` | pass |
| golden hashes (pots 90 d, plot 30 d) | match |
| `ps-cli validate` | 87 checks: 63 pass, 24 Info misses, 0 failing; every M0 gate passes |
| `ps-cli bench plot_30_beans --days 60` | 19.7 ms per sim-day on 4 cores (20.9 at WP21) |

## Findings, none blocking

1. **The ray tracer tests are pinned at the measured numbers.** `within ≥ 82%`, `worst < 0.3`,
   total in 0 to 6% on the sim plant; `|total| < 3%`, `within ≥ 80%` on the plot grid. They
   are regression pins, not gates, and should say so in a comment, because the M0 gate
   (decision 0008, 95% of leaves within 10%) still holds only on the synthetic shoot. The sim
   plant sits at 85% within 0.10 and the card names the cause (who is on top within a cell).
   That follow-up is now the whole remaining light error and is worth doing before the stand
   scenarios are read for yield.
2. **The diffuse term takes each sector's projection at one mean direction.** `|n · d'_s|` at
   the radiance-weighted mean `sin β` of the band is exact for a horizontal leaf and a good
   approximation for the beans' 19°; for an erect wheat blade and a sector whose band crosses
   the blade's plane, `|mean|` under-reads the mean of `|·|`. With 16 sectors the error is a
   few percent of the diffuse flux on erect blades only. Note it in the card; fix by
   sub-sampling the band if wheat's diffuse ever matters.
3. **Beam per organ is sunlit-weighted over leaflets.** `beam = Σ direct_i · beam_i / Σ direct_i`
   gives the exact mean absorbed beam for the organ, as the comment says. The gas-exchange
   classes still see one beam value for all of an organ's sunlit leaflets, so the within-organ
   spread of angles is averaged before the nonlinear light response. Small for three leaflets
   at the same tilt; worth one line in the card.
4. **Soil surface radiation still ignores obstacles** (card says so under known limits). Fine
   for this PR; it belongs with the obstacle tool.
5. **Memory and cost.** Depth is occupied cells × directions; for `wheat_stand` on the 96-cell
   plot grid that is a few million f64, and the decision's 100 → 120 ms per day is the cost.
   The pot bench is unchanged. Acceptable; the perf thread may want the direction loop inside
   the chunk the other way round (leaf-major) to compare.
6. **Heliotropism and leaf-angle spread** are correctly listed as the model questions this
   opens: the light is now only as right as the drawn normals, and every bean leaflet is drawn
   at the same tilt and never tracks the sun. Real beans are diaheliotropic in the morning and
   paraheliotropic under water stress; a drawn-angle distribution (mean 20°, SD 15°) would be
   the cheap first step and would also break the leaf fences the WP23 review found.

## Numbers

- Validation on seed 7: 66 → 63 passes, all three new misses at a band edge
  (`pole_height_day60` 1.498 m, `ndfa_low_n_bed` 0.494, `drought_shoot_mass` −56%);
  `pole_height_day75` 1.93 m. Wheat stand height 0.564 → 0.535 m and grain 1.16 → 1.12 g,
  the expected direction for erect blades under a high sun.
- Eight-seed bean mean 28.0 → 26.5 g per the decision record; within the seed spread.
  Re-tuning waits for the multi-seed harness (next PR), agreed.
- Model 0.9.0, species schema 6, goldens regenerated.
