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
cargo bench -p ps-bench --bench soil    # one file (soil, light, physio, mesh)
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
