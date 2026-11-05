# 0033: Broad leaves cast rays from five points

- **Date:** 2026-11-05
- **Status:** accepted (the performance thread's option 1, agreed by the implementation
  thread under the owner's rule that a perf-driven model change needs implementation or
  review agreement)

## Context
The 30-bean plot (`plot_30_beans`) has a 20 ms per sim-day budget (M2). After the
performance thread's bit-identical pass (#103) it ran at 22.7 ms per sim-day in a slow
container (main 27.2 to 29.1 ms there), about 19.5 to 21 ms on earlier ones: at the
budget, not reliably under it. The daily light rebuild is the largest share (8.1 ms, of
which rays 6.9 ms). Every broad leaf casts rays from 7 points, its center and six on a
ring at `r/√2` (decision 0008), toward each sun-up hour and the 16 sky sectors. The
performance thread's options (`/mnt/project-files/perf/plot-speed-model-options.md`):
(1) 5 points per broad leaf, (2) 12 sky sectors instead of 16, (3) rebuild plot light
every other day. It recommended (1), the same kind of change decision 0030 made for
grass blades.

## Decision
1. A broad leaf casts rays from 5 points (`ps_light::SAMPLE_POINTS`): its center and four
   on the same `r/√2` ring, 90° apart in the blade plane. Grass strips keep their 3 points
   (decision 0030).
2. Option 2 stays the fallback if the 5-point sampling costs accuracy later. Option 3 is
   not taken: growth would read a day-old canopy, and a leaf born on an odd day would have
   no light for a day.
3. No species key changes. Model 0.15.0 → 0.16.0, goldens regenerated.

## Consequences
- Ray-tracer fixtures (`ps-light/tests/single_plant.rs`), 7 → 5 points:
  - Synthetic shoot, 57 leaflets: plant total +2.82% → +2.99% (the test's gate is
    under 3%), worst leaf 0.076 → 0.093, all 57 within 0.10.
  - 60-day sim plant, 74 leaflets: +3.8% → +4.4%, 63 → 61 within 0.10. The test's gate
    is 82%, which is 61 leaflets, so it now passes with no margin; a later light change
    that loses one more leaflet will need to look at this first.
  - Through the 3 cm plot grid: synthetic −0.7% → −0.5%; sim plant −0.8% → −0.35%, 61 →
    60 within 0.10. Its gate is 80%, also 60 leaflets, so it too passes with no margin.
- All three fixture gates now pass at their edge. The fixtures are fixed leaf sets, so
  the margins move only when the light code changes; the next light change (the
  within-cell "who is on top" fix, decision 0025) must re-read them first, and option 2
  or a better 5-point layout is the way back if one fails.
- Validation on four-seed means at 0.16.0: 87 checks, 68 pass, the same verdicts as
  0.15.0, every M0 check passes. Seed-level results move by a few percent as pod-set
  draws shift (pods at maturity 11 → 10.5, seed per plant 9.3 → 8.8 g,
  pole_unstaked_seed_yield −54% → −56%); nothing else moves more than 3%.
- Speed, plot_30_beans over its 90 days, best of 3, this container (slower than the
  performance thread's): 0.12.0 33.9 to 34.1 ms per sim-day, 0.15.0 32.1 to 33.6 ms,
  0.16.0 30.2 to 30.7 ms, about −8% against main. The performance thread re-times it
  on top of its bit-identical pass (#103).
