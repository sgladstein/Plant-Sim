# Performance review 1 (WP0 to WP6), 2026-11-05

Benches: `cargo bench -p ps-bench` (PR #19). Machine: 4-core Xeon container, release build.

## Headline
- **Light (PR #17, WP6) is the M0 problem.** A daily rebuild costs 6.2 ms for a 60-leaf plant and 31 ms for 300 leaves, against 2 ms per sim-day for the *whole* single-plant sim. Cause: ~217 rays per leaf (7 sample points × (15 sun hours + 16 sky sectors)) × ~140 cells per ray on a 48³ grid = 9.5M cell visits/day for 300 leaves, versus the plan's 600k. The DDA loop itself is already tight (~18 instructions per cell visit; callgrind).
- **Soil at plot scale (M2).** Hydrology on a 72,000-cell plot costs 11 ms/day. Uptake + FTSW for 30 plants with dense per-cell root arrays costs 215 ms/day (10× the plot budget). Pots are fine (0.2 ms/day).

## Light: measured options
| Change | 300 leaves, 1 thread | Accuracy (fixture test) | Bits |
| --- | --- | --- | --- |
| WP6 as is | 31 ms | total +2.9%, 55/57 within 0.10 | — |
| Exact: own-footprint range check before binary search, stop ray at the occupied box | 26 ms | same | identical |
| + rayon over leaves (ordered collect), 4 cores | 8 ms | same | identical |
| MAX_CELLS_PER_AXIS 48 → 32 | 19 ms | total +1.7%, 56/57, worst 0.122 (better) | changes |
| MAX_CELLS_PER_AXIS 48 → 24 | 15 ms | total +0.9%, 55/57, worst 0.131 | changes |
| Diffuse from the center point only | 16 ms | total +4.8%: fails the 3% target | rejected |
| Early exit at optical depth 30 | no gain | — | — |

Patch for the exact changes plus rayon: /mnt/project-files/perf/wp6-light-speedups.patch (applies to PR #17's branch; coefficient hash unchanged on 60- and 300-leaf scenes; all ps-light tests and clippy pass). Also exact and not in the patch: `coefficients` recomputes every leaf's footprint that `add_leaves` already computed (~8% of instructions); cache them.

Recommended: take the patch, and consider 32 cells per axis (a decision record, since §3.5 says 48). Together about 19 ms single-thread, ~5 to 6 ms on 4 cores for 300 leaves; a realistic 60-leaflet bush bean lands near 1 ms wall time on 4 cores. The 2 ms M0 budget is by instruction count (single thread), so light alone still overruns it at 300 leaves; either the budget counts per-core work or the light model needs fewer rays (e.g. 3 ring points for diffuse; untested).

## Soil: recommendations (before WP7/WP8 consume the API)
1. Root input as a sparse sorted `(cell, length)` list per plant instead of a dense per-cell `Vec<Meters>`; `Uptake::per_cell` sparse too. Sums over nonzero cells only are bit-identical to the dense sums.
2. One joint uptake call per hour for all plants: compute every plant's potential from the same state (Jacobi), scale per cell if oversubscribed, one ledger. Removes the two full-grid `total_water` sums per plant per hour and makes plant order irrelevant (fairer competition; supports the §11.5 "extra plant" check).
3. Plot hydrology: reuse scratch buffers instead of two grid-sized allocations per hour; walk layers contiguously. Before WP21.

## Data layout
- OrganArena is SoA: good. `light: Vec<LightCoeffs>` (216 B) is stored for every organ, including roots and internodes; a leaf-only side table would shrink snapshots and hashes. Low priority.
