# 0029: Pole beans find their stake; variation-review items

- **Date:** 2026-11-05
- **Status:** accepted

## Context
On four-seed means (decision 0026) at model 0.11.0 the staked pole bean was 1.38 m at
day 60 and 1.86 m at day 75, against the owner's targets of 1.5 m and 2 m (decision
0023, confirmed by the owner on 2026-11-05), with a wide seed spread. The review of the
variation PR (`docs/archive/reviews/variation-review.md`) and a pre-review diagnostic by
the review thread (pole_trellis, seeds 7 to 14) found three causes:

1. **Capture.** A searching tip caught a support only within a fixed 6 cm sweep. With
   sowing scatter (1.0 cm SD) a seed sown 5 cm from its 1.5 cm stake could land up to
   6.75 cm from the stake's surface, so about 1 vine in 16 never found it and lodged at
   about 0.13 m.
2. **Germination.** Germination needed 24 *consecutive* moist, warm hours (§3.10). A seed
   that sowing scatter left shallow sits in a top cell that dips below
   `germination_ftsw` each afternoon, and the count reset to 0 every time: one plant
   germinated on day 13 instead of day 2 and reached only 1.0 m.
3. **Helix.** Vines that did catch their stake averaged 1.42 m at day 60, while the
   string row (5 mm strings) averaged 1.54 m. The coil's inclination
   `atan(pitch / 2πR)` was about 60° on the stake against the 75° clamp on a string, so
   a staked vine gained about 10% less height per metre of stem. `helix_pitch` (10 cm)
   was a guess ("~").

The variation review also left smaller items for the next model bump: key the soil
patch lattice by (soil, ix, iy); key `leaf_scatter` by node and leaf index instead of the
next arena id; add a per-internode length CV (stand height CV was 5% against about 10%).

## Decision
1. **The sweep widens with the free tip.** A searching axis catches the nearest support
   within `r_sweep = min(r_max, r_0 + k · L_free)`, where `L_free` is the stem length the
   axis has grown upright without a support (the length that makes it lodge at
   `self_support_length`). New climbing keys `circumnutation_growth` k = 0.25 (~) and
   `circumnutation_radius_max` r_max = 15 cm (~); `r_0` stays 6 cm. The free stem is what
   swings, so a longer one reaches farther, up to a cap that keeps a tip off a stake
   across a row.
2. **A climber sown at a support stays at it.** If a climbing species' sowing mark
   (habit 0.5 or more) is within 5 cm of a support's surface, sowing scatter may not leave the seed farther from that support's
   axis than the mark is (or 1 cm outside its surface, if the mark is closer). The
   scattered point is pulled radially toward the support; its bearing and depth keep
   their draws.
3. **Germination hours pause, not reset.** A dry or cold hour stops the germination
   count without zeroing it (seed hydration memory: an imbibed seed keeps its progress
   through a short dry spell). The design plan's "24 consecutive hours" becomes 24 moist,
   warm hours.
4. **Helix pitch 10 → 14 cm**, tuned on seed means to the staked height target: about
   68° on the 1.5 cm stake, inside the 35 to 75° range (Rutjens 2025). Strings stay at
   the 75° clamp, so the string row does not overshoot.
5. **Pod rot 0.02 → 0.017 per day**, re-set on seed means: with every staked row plant
   now finding its string the staked row yields more, and the unstaked row's loss went to
   -72%, past the -30 to -70% band.
6. **Variation keys.**
   - Soil patch lattice nodes are keyed by (soil, ix, iy), with the node packed into the
     key's tick slot, so enlarging a bed keeps its existing patches.
   - Leaf size is keyed by the leaf's node and its index on that node (the `i`-th normal
     of the node's `LeafSize` stream).
   - New `variation.internode_length_cv` = 0.1 (~): each internode's potential length is
     scattered, keyed by the organ it grows from, purpose `InternodeLength` (code 19). It
     applies to herb and vine internodes and to a grass's elongating internodes, not to
     crown internodes.
7. Species schema 7 → 8 (three new keys). Model 0.11.0 → 0.12.0, goldens regenerated.

Not done: emergence CV stays 0.15. The 30-bean plot emerges between days 8 and 14, 80%
of plants within days 10 to 13, which is not clearly too wide.

## Consequences
- pole_trellis on four seeds: day 60 height 1.53 m (1.51 to 1.56; was 1.38, 1.15 to
  1.46), day 75 2.04 m (1.98 to 2.11; was 1.86), final 2.08 m (2.04 to 2.12, inside 1.5
  to 2.1 on the mean, near its top). Both height targets pass.
- pole_unstaked_seed_yield -63% (was -68%; -72% before the rot re-set).
- Validation at 0.12.0 on four-seed means: 87 checks, 67 pass (was 65), 20 Info misses,
  every M0 check passes. No check flipped other than the two pole heights. Bush bean
  single_pot means moved within their bands (mean seed mass 0.27 → 0.22 g, seed yield
  10.8 → 9.8 g), from the new internode and leaf draws.
- Tests: `ps_arch::vine::tests::sweep_widens_with_free_length`;
  `ps-world` `tests/determinism.rs::seeds_sown_at_a_support_stay_at_it` (64 world seeds)
  and `soil_patches_stay_put_when_a_bed_grows`.
- `ps-validate/tests/pole_bean.rs::staked_pole_beans_climb_and_unstaked_ones_lodge`
  reads one world seed, which now gives -80% (the four seeds span -49 to -80%). Its
  yield band widens to -90 to -20%, a direction check; the -30 to -70% band stays on the
  seed mean in the validation report, as decision 0026 did for the pod counts.
  `bush_beans_ignore_supports` still holds: a bush bean's sowing ignores supports.
- Every golden hash changes.
