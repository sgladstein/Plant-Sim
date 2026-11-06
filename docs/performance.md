# Performance

Budgets come from the design plan (§3.5, §3.8, §7). This file tracks how the code
measures against them. Update the table when a work package changes a hot path.

How the performance work runs, what is open and what was tried: [perf/handoff.md](perf/handoff.md).

## Budgets

| Scope | Budget | Source |
| --- | --- | --- |
| Single plant, M0 | under 4 ms per sim-day (was 2 ms; provisional until timed on the owner's PC) | decision 0039 (owner, 2026-11-06), §7 M0 exit |
| Plot, M2 | 30 plants under 30 ms per sim-day (was 20 ms; provisional until timed on the owner's PC) | decision 0039, §7 WP21 and M2 exit |
| Frame, M1 | under 16 ms at one sim-week per second (7 sim-days per second, about 2.3 ms of sim per frame at 60 fps) | §7, M1 exit |
| Light, per plant | about 40 rays × 50 cells per leaf per day (a 300-leaf plant: 600,000 cell visits) | §3.5 |
| Gas exchange | one solve per light class: 30 plants × 32 classes × 24 h ≈ 23,000 solves per day (WP7 uses a 17 PAR × 4 capacity table, about 68 solves per plant per hour) | §3.8 |

The owner re-baselined the sim budgets on 2026-11-06 (decision 0039): the old 2 ms
and 20 ms were set before WP27, thickening and the realism plan. Big plots and
fast-forward use the simplified mode of the multi-fidelity plan
(docs/research/realism-plan/fidelity-levels.md). Result-changing speedups are no
longer pursued just to hit a budget; bit-identical ones continue. Older
measurements below compare against the old budgets.

## Running

```
cargo bench -p ps-bench            # criterion, wall time
cargo bench -p ps-bench --bench soil    # one file (soil, light, physio, mesh, world)
```

Scenes are fixed and deterministic (`crates/ps-bench/src/lib.rs`), so numbers compare
across commits on the same machine. The instruction-count budget on CI (iai-callgrind,
§11.6) lands with the single-plant scenario in WP9/WP10.

## Measurements

Release build, 4-core Xeon cloud container, 2026-11-05 (after #21, #24, #25). The
owner's PC (GTX 1070 era) should be within a factor of about 1.5 either way. Light runs
leaves in parallel (rayon); single-thread numbers are in brackets.

| Bench | Time per sim-day | Against budget |
| --- | --- | --- |
| soil, pot (1,000 cells): hydrology | 0.14 ms | fine |
| soil, pot: uptake + FTSW, 1 plant | 0.07 ms | fine |
| soil, 3 × 3 m plot (72,000 cells): hydrology | 11 ms | over half the 20 ms plot budget on its own |
| soil, plot: joint uptake + FTSW, 30 plants (sparse roots) | 13 ms (was 215) | with hydrology, soil alone is 24 ms; see hot spot 2 |
| light, 60-leaf plant, daily rebuild | 1.26 ms [3.8] (was 6.5) | fine on 4 cores; over 2 ms single-thread |
| light, 300-leaf plant | 5.8 ms [19] (was 32) | over |
| light, 30 plants × 100 leaves, one grid | 25 ms (was 110) | over the plot budget |
| physio, class table build (24 × 68 solves) | 0.32 ms | fine for one plant; 9.7 ms for 30 per-plant tables |
| physio, 600 class lookups × 24 h | 0.39 ms (was 1.26) | fine |
| physio, allocation with implicit transport, 191 organs × 24 h | 0.65 ms | fine for one plant; 19 ms for 30 plants |

Whole sim through `World` (`benches/world.rs`, committed scenarios, 90 days):

| Scenario | Per sim-day, 4 cores [1 thread] | Against budget |
| --- | --- | --- |
| single_pot | 1.67 ms [2.34] (2.17 [3.4] after WP12 tuning) | M0 §7 "under 2 ms per sim-day": met on 4 cores; single-thread still over |
| two_pot_sun_shade | 2.94 ms (3.9 after WP12; single-thread not re-measured) | M1 sim-week per second needs 24 ms of sim per wall second: fine |
| plot_30_beans | 20.1 ms [51] (median of 5 runs, 19.7 to 20.7; 28.8 [59] at WP21) | M2 30 plants under 20 ms: at the target on 4 cores, not reliably under |

The WP12 plant is bigger (more leaves, branches and root cohorts), which took
single_pot from 1.90 to 2.17 ms. The M0 speed fix (bit-identical, same golden hashes)
brought it to 1.67 ms: the ray traversal in `LightGrid::path_sum_b` unrolls its three
axes, a ray locates its start cell once, `rebuild_daily` reuses each leaf's footprint
from the grid fit, `ClassTable::build` computes the temperature terms once per table
(`gas::TemperatureTerms`), and the maintenance Q10 cache keeps every temperature of the
hour. Callgrind, single_pot on one thread: 2.88G to 2.32G instructions over 90 days.

Plot speed follow-up to WP21 (bit-identical, golden hashes unchanged). Callgrind on the
plot showed light rays 38% of instructions, the daily state hash 3.6 ms of serial time
(one sequential mixing chain over about 2 million words by day 90, so it is
latency-bound: a flat loop over pre-collected words is no faster), and about 65 µs per
parallel section spent waking sleeping rayon workers (measured: a 30-item `par_iter`
after any idle gap takes 60 to 75 µs). Changes:

- Plants of one soil with identical leaf parameters and stress bits share one hourly
  class table (`Plant::class_table_key`, `Plant::water_demand_with`).
- The plot grid keeps each leaf's footprint from the fit, computed in parallel
  (`LightGrid::add_leaves_keeping`).
- Joint uptake totals requests per cell on a dense scratch grid instead of sorting.
- `day_after_light` runs in parallel; FTSW and stress are updated serially (cheaper than
  a parallel section's wake-up); leaf area comes from the demand pass.
- `run_until` runs on a rayon worker, and each day's hash runs on a snapshot (a parallel
  clone, about 0.6 ms) while the next day steps; it is collected at the next daily
  step and before `run_until` returns, so `WorldState::hashes` is complete at every API
  boundary. The waiting worker runs pool work meanwhile, so one thread cannot deadlock.

Remaining plot cost on 4 cores, per sim-day: daily light and growth step about 8 ms,
hourly growth 5.5 to 6.8 ms, demand 2.2 ms, soil 0.8 ms, uptake 0.8 ms. Further cuts
need the owner's call: fewer rays per leaf (a model change), or a cheaper hash
definition (new golden hashes).

After WP24 (nitrogen, model 0.5.0) the per-cell nitrogen passes and the root reach
took single_pot to 2.6 ms, two pots to 4.4 and the plot to 25.1 ms per sim-day. Two
bit-identical follow-ups (golden hashes unchanged) brought them back down:

- #60: `reached_cells` sums root length on a dense scratch grid in the order the
  stable sort would add it, instead of sorting up to 125 pairs per root cell, and
  `take_up` uses one scratch grid for totals and factors.
- This PR: the hourly soil step (pot, 1,728 cells) went from 20.6 to 17.5 µs.
  Lateral exchange computes each pair's flux once in loops without data-dependent
  branches, then each cell adds its fluxes in the order the old scatter did (a
  missing pair adds +0, which changes nothing because the running sum is never
  −0). Drainage computes each layer's excess and carried nitrate just before the
  layer above pours into it, so it needs one layer of scratch and one pass, and the
  nitrate and water moves share it. Tried and rejected, all slower: a branch-free
  drain (it divides for every cell, and most cells have no excess), splitting the
  nitrogen transform into a rate pass and an update pass, and a branch-free
  `water_factor`. The transform is bound by its four divisions per cell, not by
  branches.

| Scenario, 90 days, 4 cores | main 94b55db | after #60 | this PR |
| --- | --- | --- | --- |
| single_pot | 2.6 ms | 2.17 ms | 2.06 ms (fastest 2.00) |
| two_pot_sun_shade | 4.4 ms | 3.4 ms | 3.28 ms |
| plot_30_beans | 25.1 ms | 23.6 ms | 23.3 ms |

Medians of 5 runs, `cargo run --release -p ps-bench --example sim_days 5`. The
single pot is at the M0 budget, not reliably under it, and the plot is about 3 ms
over the M2 budget. Where a single-pot hour goes now (wall time, instrumented):
soil step 20%, growth 20%, the class table 12%, uptake 6%, and the daily step
(light rebuild and growth) 36%.

After WP22 (pole bean, model 0.6.0), medians of 5 runs on the same machine: single_pot
2.12 ms, two pots 3.38, plot 23.8 (within noise of the numbers above; callgrind
instructions are slightly lower). The pole scenarios are heavier per plant:
m2/pole_trellis (4 pots, 110 days) 16.4 ms per sim-day, about 4 ms per plant, and
m2/pole_row_trellis (8 plants on one 1.5 × 1 m plot) 12.2 ms. A 30-plant pole plot
at that rate would be about 45 ms, so pole beans need their own look before a large
pole plot is a target.

After WP23 (spring wheat, model 0.7.0), medians of 3 runs: m2/wheat_spaced (4
plants, 140 days) 15.5 ms per sim-day, and m2/wheat_stand (about 200 plants on a
1 × 1 m bed) 111 ms per sim-day, so a 140-day stand takes about 15.5 s. Callgrind
on a 60-day stand, single thread: the shared plot light rebuild is 48% of
instructions (ray traversal 43%), hourly growth 33% (allocation 26%), and the daily
hash 5.7%. Bit-identical work has little left to give here; the levers are model
choices for the owner: fewer rays per leaf, or one light sample per tiller rather
than per blade for grasses.

After the decision 0029 merge (model 0.12.0), a bit-identical pass (perf thread,
2026-11-05) cut every scenario; all seven state hashes are unchanged. `sim_days 5`,
fastest of 5 runs, two passes each, same container:

| Scenario | main 5eb2328 | this pass | Change |
| --- | --- | --- | --- |
| single_pot | 2.66 to 2.70 | 2.06 to 2.19 | about -20% |
| two_pot_sun_shade | 3.86 to 4.22 | 3.33 to 3.50 | about -15% |
| plot_30_beans | 27.2 to 29.1 | 22.7 to 22.8 | about -17% |
| m2/pole_trellis | 19.7 to 19.8 | 17.4 to 17.8 | about -11% |
| m2/pole_row_trellis | 13.8 to 14.5 | 12.0 to 12.3 | about -14% |
| m2/wheat_spaced | 17.9 | 15.9 to 16.5 | about -10% |
| m2/wheat_stand | 120 to 123 | 104 to 105 | about -14% |

This container ran about 15% slower than the one behind the earlier numbers (main
measured 2.66 against 2.2 to 2.3 for the single pot). What changed:

- **Allocator.** With glibc malloc, each plant's hourly growth ran about 38% slower in
  the plot's parallel section than alone (allocation contention). ps-cli, ps-app and
  the bench examples now use mimalloc (`#[global_allocator]`); the sim crates do not
  depend on it, and the allocator cannot change results. Hourly growth on the plot
  went from about 850 to 650 ms per run.
- **Ray tails.** A ray stops past the highest occupied layer it can reach. Per rising
  direction, `LightGrid::ray_bounds` builds a `Shadow`: the grid sheared along the ray
  so a ray is one point, with each bin holding the top occupied layer whose sheared
  cell box covers it (padded for rounding). The rest of the ray crosses only empty
  cells, which add exactly `+0`. On the day-50 plot canopy this removes 28% of cell
  steps (finer bins remove up to 34% but cost more to build than they save). A
  per-layer bounding box instead removed only 7%.
- **Footprints.** `LightGrid::footprint` locates the lattice samples per axis and
  counts samples per cell, instead of locating, sorting and merging each sample. The
  merged value is the same running sum of equal shares.
- **Own cells.** Each leaf's rays find its own footprint cells in a dense index over
  its range instead of a binary search (about 5% of ray time).
- **Daily nitrogen.** Root reach uses a summed-volume table of active cells and one
  pass per root cell; each plant's reach, uptake plan and daily nitrogen step run in
  parallel (collect keeps plant order). Plot daily nitrogen 1.6 to 1.1 ms per sim-day.
- **Soil storage sums.** The world keeps only the soil ledger's fluxes and the
  storage after uptake, so `step_hour_lean` and `take_up_lean` skip three of the four
  full-grid storage sums per hour in release builds (each a chain of dependent
  additions). Debug builds still sum them and assert closure every hour.

Tried and dropped: branchless axis selection in the ray loop (35% slower than the
unrolled form); per-layer horizontal bounds checked at every z step (no faster: the
check cost as much as the 7% of steps it saved).

Where the plot's time goes now (fastest of 5, 90 days, 4 threads): daily light
rebuild 8.1 ms per sim-day (rays 6.9, grid fit 0.7, shadows 0.4), hourly growth 7.3,
soil step 1.9, demand 1.6, daily nitrogen 1.1, uptake 0.7. Hourly growth does about
4.6 ms of work per sim-day spread over 4 threads; the other 2.7 ms is wake-up and
scheduling around 2,160 short parallel sections.

CI prints the same table into the summary of the linux workflow's `bench-summary`
job. That job only reports: shared runners are too noisy to gate on wall time, so it
cannot fail the build.

Full state hash at day 90: 0.25 ms (the daily hash is chained, so it no longer grows
with run length). RON snapshot at day 90: 3.3 ms, 740 KB, on demand only.

Owner's machine (i5-6600K, 4 cores, 8 GB RAM, GTX 1070), viewer from PR #26 at
ee586b1 with stand-in plants, two plants, 2560 × 1440, vsync off, 1,200 frames, single
run: Vulkan mean 2.70 ms per frame (p99 2.94), GPU about 51% busy; DX12 mean 5.78 ms
(p99 6.05), GPU about 98%. Both are well inside the 16.7 ms M1 budget; Vulkan is the
better default.

Pacing re-run on the same machine (main 58807c6, new look, still stand-in plants, so
not comparable to the 2.7 ms above):

| Backend, present mode | Frame time | Notes |
| --- | --- | --- |
| Vulkan, Mailbox | mean 4.0 ms, p99 4.4 ms | no spikes; recommended |
| Vulkan, Fifo | 7.2% of frames near 31 ms, every ~14 frames | a ~64 Hz present cadence beating against 60 Hz on the virtual display; frame latency setting has no effect |
| DX12, Fifo | p99 32.7 ms, 4.3% spikes | |

With the real sim instead of stand-in plants (f4bec25, Vulkan uncapped,
2352 × 1325): mean 6.55 ms, p99 9.61 ms. The #72 lawn and #78 fuller plants are not
yet timed.

Cold start: Vulkan about 5 s, DX12 about 180 s (pipeline compilation). Default to
Vulkan with Mailbox where supported, falling back to Fifo, and DX12 only when Vulkan
is unavailable.

Viewer CPU work per frame, one 90-day demo bean (257 organs, `benches/mesh.rs`): clone
the hourly arena and skeleton 8 µs, skeleton blend 1.6 µs, leaf instances 0.9 µs, a
full rebuild of every axis mesh 60 µs. Even 30 plants rebuilding every axis every frame
is about 2 ms of a 16.7 ms frame; GPU time on the GTX 1070 has to be measured on the
owner's machine.

### R2a cost (model 0.20.0, 2026-11-06)

R2a (decision 0037, #129) adds about 45% more organs (root segments) and moves the
thickening potential to the daily step. Callgrind, one thread, 90 days:

| Scenario | 0.19.0 (824eeb0) | 0.20.0 (3a55343) | with the pre-sized phloem edge list |
| --- | --- | --- | --- |
| single_pot | 3.10 G instructions | 3.61 G (+16.5%) | 3.61 G |
| plot_30_beans | 55.8 G | 68.3 G (+22.3%) | 66.8 G (-2.1%) |

Wall time, 4 cores, interleaved best of 3, in a container that ran 0.19.0's plot at
33 ms where the previous container ran it at about 26: single_pot 3.3 to 3.9 ms per
sim-day; plot_30_beans 32.9 to 38.2 ms, and 37.2 to 38.5 ms with the edge-list fix
(main 39.4 to 41.2 in the same runs). Scaled to the earlier container the plot is
about 30 ms and the pot about 3.5 to 4 ms: both at the edge of the decision 0039
budgets. Timing on the owner's PC settles it.

The extra cost is spread over the hourly per-organ loop in `allocation::hourly_recording`
(about 700 instructions per organ per hour), the hourly wood build
(`OrganArena::grow_secondary`, 17.5 M calls, 3%) and the daily hash, which hashes
more organs but runs overlapped. The one cut with no change to results: the phloem
step collected its edge list through a filter, which has no length hint, so the Vec
grew by doubling every hour (about 3% of the plot's instructions in reallocation and
copies); it is now sized to the organ count. Still open: cloning `PlantEntry` for the
overlapped daily hash is about 2.6% of instructions (1.8 G); making it cheaper means
sharing the plant's big vectors (for example behind `Arc`), a structural change.

## Breeding bench (WP30)

One ten-seed generation (WP30 done-when: under 10 s of wall time), measured with
`cargo run --release -p ps-bench --example breed_generation`. The run is ten bush bean
lines (the species line and nine sampled founders) on the default bench, growing to
maturity in 81 days and giving 388 seeds. On the 4-core container it took 4.6 to 5.2 s,
and 6.2 s on one thread, with the same hash on 1, 2 and 4 threads. CI prints it in the
bench summary as a report only.

## Known hot spots

1. **Light ray count.** Each leaf marches 7 sample points × (daylight sun hours + 16 sky
   sectors), about 217 rays per leaf (3 points per grass blade from decision 0030). On
   the day-50 plot canopy rays take about 16 steps each after the shadow cut. What is
   left is a model choice (fewer points or directions, or a rebuild every other day).
2. **Soil uptake.** Dense root arrays were fixed by decision 0012 (sparse sorted
   `(cell, length)` lists, one joint `take_up` per hour). What remains is 36% building
   `per_cell` with a binary search per root cell and 27% sorting the per-cell request
   list; a dense per-cell scratch accumulator summed in plant order gives the same bits
   without either.
3. **Physiology at plot scale.** One class table per plant and per-hour `Vec`s in
   allocation. Dedup tables by (leaf parameters, stress) and reuse scratch buffers
   before WP21.
4. **Soil hydrology and nitrogen per hour.** Lateral exchange and drainage were
   reworked after WP24 (see above). The nitrogen transform is next: four divisions
   per active cell per hour, so on a plot it scales with cell count.
