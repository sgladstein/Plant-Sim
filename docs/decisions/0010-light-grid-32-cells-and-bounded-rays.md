# 0010: Light grid at most 32 cells per axis; rays stop at the occupied box

- **Date:** 2026-11-05
- **Status:** accepted

## Context

The performance review (/mnt/project-files/perf/perf-review-1.md) measured the daily
light rebuild at 31 ms single-threaded for a 300-leaf plant, against a 2 ms whole-plant
budget per sim-day. Three exact changes (same output bits) and one parameter change were
measured:

| Change | 300 leaves, 1 core | Fixture result |
|---|---|---|
| Baseline (48 cells per axis) | 31 ms | total +2.9%, 55/57 within 0.10, worst 0.142 |
| Own-footprint range check before the binary search; ray stops on leaving the box of cells with nonzero depth | 26 ms | identical bits |
| + rayon over leaves, ordered collect (4 cores) | 8 ms wall | identical bits |
| `MAX_CELLS_PER_AXIS` 48 → 32 | 19 ms (1 core) | total +1.7%, 56/57, worst 0.122 |
| 48 → 24 | 15 ms | total +0.9%, 55/57, worst 0.131 |

Design plan §3.5 says 48 cells per axis.

## Decision

- Take the three exact changes. The bounded traversal is `LightGrid::path_depth_bounded`
  with `LightGrid::occupied_bounds`; a cell outside the box has zero depth, so the sum
  is unchanged to the bit (tested by `bounded_traversal_is_bit_identical`). The parallel
  map over leaves collects in input order, so results do not depend on the thread count.
- Set `MAX_CELLS_PER_AXIS` to 32. It is faster and, on the single-plant fixture, more
  accurate on every metric: at 48 cells a 2 cm cell is smaller than a bean leaflet
  (about 6 cm across), so the per-cell positive-binomial correction acts on fragments of
  a blade; 3 cm cells are closer to the leaflet scale. 24 is faster still but worse per
  leaf, and the gain is small.
- §3.5 of the design plan is amended to 32.

## Consequences

- The ps-light grid test now checks the 32 limit. The light model card states 32.
- Light coefficients change (no golden hashes exist yet, so nothing to regenerate).
- At 300 leaves light is still about 19 ms of single-core work per day. The 2 ms M0
  budget holds only if it counts wall time on 4 cores (about 5 ms) or the plant has
  about 60 leaflets (a bush bean has about 60: about 1 ms wall). Revisit in WP12 with
  the real plant; further options are fewer ring points for diffuse rays (untested) and
  caching the footprints `fit` already computed (about 8% of instructions).
