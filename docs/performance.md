# Performance

Budgets come from the design plan (§3.5, §3.8, §7). This file tracks how the code
measures against them. Update the table when a work package changes a hot path.

## Budgets

| Scope | Budget | Source |
| --- | --- | --- |
| Single plant, M0 | under 2 ms per sim-day (instruction count on CI) | §7, M0 exit |
| Plot, M2 | 30 plants under 20 ms per sim-day | §7, WP21 and M2 exit |
| Frame, M1 | under 16 ms at one sim-week per second (7 sim-days per second, about 2.3 ms of sim per frame at 60 fps) | §7, M1 exit |
| Light, per plant | about 40 rays × 50 cells per leaf per day (a 300-leaf plant: 600,000 cell visits) | §3.5 |
| Gas exchange | one solve per light class: 30 plants × 32 classes × 24 h ≈ 23,000 solves per day (WP7 uses a 17 PAR × 4 capacity table, about 68 solves per plant per hour) | §3.8 |

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

Full state hash at day 90: 0.25 ms (the daily hash is chained, so it no longer grows
with run length). RON snapshot at day 90: 3.3 ms, 740 KB, on demand only.

Owner's machine (i5-6600K, 4 cores, 8 GB RAM, GTX 1070), viewer from PR #26 at
ee586b1 with stand-in plants, two plants, 2560 × 1440, vsync off, 1,200 frames, single
run: Vulkan mean 2.70 ms per frame (p99 2.94), GPU about 51% busy; DX12 mean 5.78 ms
(p99 6.05), GPU about 98%. Both are well inside the 16.7 ms M1 budget; Vulkan is the
better default.

Pacing re-run on the same machine (main 58807c6, new look, so not comparable to the
2.7 ms above):

| Backend, present mode | Frame time | Notes |
| --- | --- | --- |
| Vulkan, Mailbox | mean 4.0 ms, p99 4.4 ms | no spikes; recommended |
| Vulkan, Fifo | 7.2% of frames near 31 ms, every ~14 frames | a ~64 Hz present cadence beating against 60 Hz on the virtual display; frame latency setting has no effect |
| DX12, Fifo | p99 32.7 ms, 4.3% spikes | |

Cold start: Vulkan about 5 s, DX12 about 180 s (pipeline compilation). Default to
Vulkan with Mailbox where supported, falling back to Fifo, and DX12 only when Vulkan
is unavailable.

Viewer CPU work per frame, one 90-day demo bean (257 organs, `benches/mesh.rs`): clone
the hourly arena and skeleton 8 µs, skeleton blend 1.6 µs, leaf instances 0.9 µs, a
full rebuild of every axis mesh 60 µs. Even 30 plants rebuilding every axis every frame
is about 2 ms of a 16.7 ms frame; GPU time on the GTX 1070 has to be measured on the
owner's machine.

## Known hot spots

1. **Light ray count.** Each leaf marches 7 sample points × (daylight sun hours + 16 sky
   sectors), about 217 rays per leaf, at about 140 cells per ray on a 48-cell grid: 16×
   the §3.5 estimate. Exact speedups (same bits) and model options are in the perf
   review.
2. **Soil uptake.** Dense root arrays were fixed by decision 0012 (sparse sorted
   `(cell, length)` lists, one joint `take_up` per hour). What remains is 36% building
   `per_cell` with a binary search per root cell and 27% sorting the per-cell request
   list; a dense per-cell scratch accumulator summed in plant order gives the same bits
   without either.
3. **Physiology at plot scale.** One class table per plant and per-hour `Vec`s in
   allocation. Dedup tables by (leaf parameters, stress) and reuse scratch buffers
   before WP21.
4. **Soil hydrology at plot scale** allocates two grid-sized vectors per hour and walks
   columns with a layer stride. Fine for pots; worth a pass before WP21.
