# WP25 review: harvest, obstacles, supports mid-run, weather files, apical hold (PR #73, decision 0024)

Reviewed at commit a590e70 on `claude/project-thread-ur2udi` against `origin/main` (model 0.7.0).
Reviewer: Fable review thread. Date: 2026-11-05.

## Verdict

**Approve, with one correction to the decision record before merge and one design change to
schedule.** The actions are small, well guarded and tested; the done-when (topping releases
every bud below the cut within four days) holds for both beans; a weather file drives a run.
All checks are green.

**The correction: the bush bean did not grow because of the apical hold.** Decision 0024 says
the bean "grows bigger: 32.8 g instead of 25.0, 18 pods instead of 12" and leaves the cause
untraced. I ran `single_pot` under eight other world seeds on `main` and on this branch:

| world seed | main: mass g, pods, seed g | branch: mass g, pods, seed g |
| --- | --- | --- |
| 11 | 28.9, 11, 10.7 | 28.1, 12, 10.3 |
| 12 | 34.5, 17, 14.3 | 32.8, 15, 13.7 |
| 13 | 26.5, 10, 8.7 | 30.4, 12, 12.5 |
| 14 | 31.5, 13, 13.2 | 29.0, 12, 12.1 |
| 15 | 21.4, 11, 7.2 | 22.6, 15, 6.6 |
| 16 | 27.9, 11, 10.7 | 24.0, 9, 8.9 |
| 17 | 38.8, 17, 16.9 | 23.8, 8, 8.3 |
| 18 | 33.8, 15, 13.6 | 33.6, 16, 12.9 |
| **mean** | **30.4, 13.1, 11.9** | **28.0, 12.4, 10.7** |

The hold changes nothing systematic (if anything the branch is 8% smaller). The 25.0 → 32.8 g on
seed 7 is one draw: the plant has the same 9 racemes and 54 flowers on both sides, and sets 18
pods instead of 12 because the per-flower pod-set draws come out differently once the bud
stream's history changes. Then, because the budget is sink-limited (`feedback_loss` 0.3 to 0.8),
total mass follows pod count. Write that into 0024 in place of "the cause is not traced".

**The finding behind it is bigger than WP25.** One plant's season varies from 21 to 39 g and 8
to 17 pods with the world seed alone, under identical weather and parameters. The M0 gates, the
response checks (shade, heat, drought, density) and the WP22 sweep that chose `h = 1.5, d = 0.7`
were all evaluated on seed 7. A check that moves from pass to miss when a pod-set draw flips
(`heat_seeds_per_pod` −7% now, `drought_shoot_mass` −45% now) is reading noise. Two fixes, both
for the validation harness and neither a model change:

- Evaluate every target on the mean of `n` seeds (4 is enough for ±10% on mass at this
  variance; the runs are 2 s each), with the seed list in `targets.ron` and the spread in the
  report. Response checks compare treatment and control means.
- Do not tune "~" parameters against one seed's result. The 0024 sweep's constraint "the
  unstaked pole row stays past −70%" is a seed-level number too (−34% at WP22 review, −67% now).

Why the plant is so noisy is a model question for M2 exit: real bush beans set 12 to 18 pods
with a coefficient of variation near 20%, the sim's is about 25%, so it is not far off; but the
sim puts all of the variance in a few dozen Bernoulli draws at flowering while the real plant
spreads it across the season. Hold for the transport/priority revisit.

## What I checked

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| clippy, sim and CLI crates, `-D warnings` | clean (CI covers the Bevy crates) |
| `cargo test` on the sim and CLI crates | all pass, including the six WP25 tests in `actions.rs` |
| `scripts/check_deps.py`, `scripts/check_determinism.py` | pass |
| golden hashes (pots 90 d, plot 30 d) | match |
| `ps-cli validate` | 87 checks: 66 pass, 21 Info misses, 0 failing, 0 known; every M0 gate passes |
| seed sweep | `single_pot` × 8 seeds, main against branch (table above) |

## Obstacles: the design change to schedule

The smearing is correct where the card says (a thin wall keeps `κ t` across a cell at normal
incidence; the unit test checks it) and grey-only is the right call for R:FR. The problem is the
grid fit. A pot's adaptive grid has at most 32 cells per axis; it is fitted to the leaves plus
the obstacles' corners. The test's 2 m shade cloth turns a 50 cm bean's 2 cm cells into 6 cm
cells, and a garden wall 1 m from a pot does the same for the whole season: the plant's own
self-shading, the thing the M0 light gate was tuned on, is now computed on a grid three times
coarser, with no obstacle nearer than a metre. Every pot in a scene with one fence pays this.

The fix is to not put obstacles in the grid at all: a handful of axis-aligned boxes is cheaper
to intersect analytically than to voxelize. `path_depth` already walks the ray out of the grid;
add `Σ κ_b · (ray length inside box b)` by the slab test for each box the ray exits toward.
Exact (no smearing, no soft edge), independent of cell size, the grid stays fitted to the
leaves, and rays from the sky sectors see a wall at any distance. The same code gives supports
their shade for free (the WP22 review's 1% stake). Do it in the obstacle follow-up before the
viewer's obstacle tool ships, since the tool will put obstacles near pots.

Also:

- **Soil surface radiation ignores obstacles** (`global · exp(−k LAI)` per soil). Under a shade
  cloth the pot's soil still evaporates in full sun. Multiply by the obstacle transmittance
  along the zenith ray from the soil centre, or list it in the card.
- Scenario obstacles are copied into every plant (`Plant::obstacles`), and the plot grid reads
  the first plant's copy. Fine today; a plant-level copy will drift from the soil's list the day
  obstacles can be removed. Keep the list on the soil and pass a reference.

## Other findings

1. **`Harvested::mature` is the plant's stage, not the pod's.** On an indeterminate pole bean
   the first pods are ripe at day 70 while the plant is still flowering, and on wheat the ear
   is one organ. For M3 seed viability the right flag is per pod: every seed in it has finished
   filling (no growing `SeedInPod`). Cheap to compute in `harvest` from the arena.
2. **Harvested seed mass is structural only** (`a.structural(c)`); the daily report's
   `seed_mass` is the same, so they agree, but a seed's reserve is part of what a breeder
   weighs. Say so in the `SeedLot` doc comment or add the reserve.
3. **Bud eligibility after topping** uses the species constant `main_nodes − 2`, so a bean
   topped at 4 nodes releases buds 1 to 4 but a bean topped at 6 of a 7-node stem keeps the
   bud under the cut for a raceme it will never make. Minor; after a cut, every bud below it
   should be eligible.
4. **Topping after flowering releases nothing**: `bud_fate` returns at `Stage::Flowering`.
   Right for the determinate bush bean, but a pole bean topped at 1.8 m in flower is the
   classic grower's move and in the sim it just stops. Follow-up with the branch-as-leader item.
5. **The hash.** The three lists join the hash only when non-empty, so old runs keep their
   hashes; good. `added_supports` and `added_obstacles` are already implied by the hashed action
   log, so hashing them is belt and braces, harmless.
6. **Weather file**: the generator labels mid-hour radiation with the hour's start, which is
   what `ps_weather::csv` expects (decision 0005); say so in the README so a real station file
   (usually end-of-hour) is shifted on import. The test only proves the file is read; fine for
   the exit criterion.
7. **Validation**: `pole_height_day75` misses at 1.989 m against 2.0 (the Info target the
   0023 paperwork added, as expected until the string-helix gain lands). `nar_early` now misses
   at 6.9 against 2 to 6 for the same seed-7 reason as above.

## For the record

- Model 0.8.0; species schema 5 (`apical_hold_nodes`); scenario `obstacles` default empty;
  new `ActionKind::Harvest`, `PlaceObstacle`, `AddSupport`. Hold 0 and fade 2 reproduce 0.7.0
  byte for byte per the decision record (not re-verified here; the seed sweep covers the claim
  that matters).
- Validation at WP25: 87 checks, 66 pass, 21 Info misses. Changed: `nar_early` miss,
  `heat_seeds_per_pod` miss, `drought_shoot_mass` pass, `pole_unstaked_seed_yield` −67%.
- Seed sweep: main 30.4 ± 5.3 g, branch 28.0 ± 4.2 g over seeds 11 to 18; seed 7 is 25.0 and
  32.8.
