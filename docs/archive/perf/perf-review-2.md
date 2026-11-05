# Performance review 2 (WP7, WP8, viewer CPU), 2026-11-05

Benches: `cargo bench -p ps-bench` (soil, light, physio, mesh). Numbers in docs/performance.md.

## Whole plant (WP8, PR #27, 90 days, 24 cm pot)
- 1.94 ms/day wall on 4 cores; 3.19 ms/day single-thread; 27.8M instructions/day.
- Shares (callgrind, inclusive): light rebuild ~2/3, allocation 23%, soil hydrology 12%, class table 9%.
- Exact patch: cache the Q10 factor per temperature in allocation (/mnt/project-files/perf/wp8-q10-cache.patch): 3.19 → 2.72 ms/day single-thread, same report hash.
- Under 2 ms single-thread needs a light model choice (fewer sky sample points, or less frequent rebuilds). Owner decision.

## Landed since review 1
- #21 light: 60 leaves 6.5 → 1.26 ms/day (4 cores), identical bits; 32 cells per axis (decision 0010).
- #21 lookups: 1.26 → 0.39 ms/day.
- #24 implicit transport: no cost change.
- #25 sparse roots + joint uptake: 30-plant plot 215 → 13 ms/day. Remaining cost: binary search and sort in take_up; dense scratch accumulator suggested (same bits).

## Viewer CPU (benches/mesh.rs)
Per plant per frame: snapshot clone 8 µs, skeleton blend 1.6 µs, instances 0.9 µs, full axis rebuild 60 µs. Not a concern; GPU time must be measured on the owner's GTX 1070.

## Plot-scale backlog (before WP21)
Class-table dedup by (params, stress); allocation scratch buffers; soil hydrology buffers; take_up scratch accumulator.
