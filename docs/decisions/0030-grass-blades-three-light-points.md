# 0030: Grass blades cast rays from three points per strip

- **Date:** 2026-11-05
- **Status:** accepted (the owner chose option (a) on 2026-11-05, HANDOFF §2 item 4)

## Context
The dense wheat stand (`m2/wheat_stand`, 200 plants per m²) cost about 110 to 140 ms per
sim-day, too slow to play live. Light is the largest share: every leaf point casts rays
from 7 sample points (the center and a ring of six at `r/√2`, decision 0008) toward every
sun-up hour and 16 sky sectors, and a wheat blade is three strips (decision 0022), so 21
origins per blade. The ring is a disc's sampling; a strip of a blade a few mm wide is
narrower than that ring and much narrower than a light cell. The performance handoff gave
two options, both of which change results: (a) fewer sample points per grass blade, or
(b) one light sample per tiller. The owner chose (a), 7 to 3 points.

## Decision
1. `ps_light::LeafPoint` gets an optional `strip` field: for a narrow strip, the offset
   from its center to its two end sample points, along the strip. A strip casts rays from
   3 points (`STRIP_SAMPLE_POINTS`): its center and the centers of its outer thirds,
   `±ℓ/3` along the blade. Broad leaves (`strip: None`) keep the 7-point disc. The field
   is `#[serde(default)]`, so older snapshots read as broad leaves.
2. `ps_arch::grass::blade_points` sets `strip` on each of a blade's three strips. Beans
   are unchanged.
3. No species key changes (schema stays 8). Model 0.12.0 → 0.13.0. The golden scenarios
   are bean-only, so their trajectories are bit-identical; the hashes change only through
   the model version, which the hash includes.

## Consequences
- Speed (one run each, this container, 4 cores, full 140 days): wheat_stand 141 → 124 ms
  per sim-day (-12%). The perf handoff's estimate was about -25%; ray casting is a smaller
  share of the stand's time than estimated, so the stand is still not live-playable.
  Option (b) or a cheaper footprint/grid build would be the next lever (perf thread).
- Validation on four-seed means at 0.13.0: 87 checks, 67 pass, the same verdicts as
  0.12.0. Every bean check is bit-identical. Wheat stand checks move by under 0.5%
  (tillers 1.575 → 1.583, height 0.452 → 0.451 m, shoot mass 2.912 → 2.915 g, grain
  0.940 → 0.942 g per plant); spaced wheat is unchanged.
