# 0047: Daily rows carry the soil's water and nitrogen totals; WP8's pod band on one plant

- **Date:** 2026-11-07
- **Status:** accepted (R5a code review, `reviews/r5a-soil-water-flow-0046-review.md` in
  the project files, findings 2, 3, 5 and 7)

## Context
R5a (decision 0046) moved where water and nitrate go, but the daily rows held only the
soil's water and mineral nitrogen, so the review had to infer from the totals whether
the 30-bean plot kept more nitrogen by leaching less or by mineralizing more. R5b
(waterlogging) and R5c (compaction) will move these again.

The review also asked that `ninety_days.rs`, WP8's done-when test, water its pot through
the surface as the other whole-plant tests and the world do. Watered that way its seed 7
sets 8 pods, under the single-plant 10 to 20 of design plan §3.16, while the eight seeds
the test already runs average 9.0, inside the 6 to 20 of §8.2 (validation's
`pods_at_maturity`). Its driver grows the plant in garden nitrogen, below the world
pot's non-limiting 300 ppm, which over world seeds 7 to 14 sets 11.1 pods.

## Decision
1. **Five soil totals in each daily row** (`DayEnv`), for the plant's soil, summed since
   the start of the run: water drained out of the base (with a saucer's spill), water
   run off, water evaporated from the surface, nitrate drained out of the base, and
   humus nitrogen mineralized. They come from the soil's water ledger and nitrogen
   ledger, which already kept them. The CSV adds `soil_drained_total_kg`,
   `soil_runoff_total_kg`, `soil_evaporated_total_kg`, `soil_leached_n_total_g` and
   `soil_mineralized_n_total_g` at the end of its columns; `_total` columns are sums
   since the start. With the plants' daily uptake the mineral nitrogen budget closes:
   start + mineralized + fertilizer − leached − uptake = mineral N now.
2. **Model 0.28.0.** No result changes: every existing column of the single pot, the
   split pot and the 30-bean plot is identical to 0.27.0 for 60 days. The rows are part
   of the state hash, so the goldens move, and a 0.27.0 save would no longer replay to
   its hashes.
3. **`ninety_days.rs` waters through the surface**, and its one-plant check reads
   §8.2's 6 to 20 pods instead of §3.16's 10 to 20, as single plants spread wider than
   the band (decision 0027) and validation reads the band on seed means (decision
   0026). The eight-seed test keeps 4 to 30 on each plant and 6 to 20 on the mean.

## Consequences
- **The plot's season at 0.28.0** (plot_30_beans, 90 days): mineral nitrogen 20.0 →
  12.6 g, with 5.0 g mineralized, 1.6 g leached and 10.9 g taken up (2.8 g fixed); the
  soil drains 14.7 kg and evaporates 393 kg. At 0.26.0 the plot went 20.1 → 11.2 g on
  5.7 g of uptake, so mineralized minus leached was −3.2 g; if it mineralized about as
  much, it leached near 8 g. Most of the 6.5 g that R5a keeps in the plot is then less
  leaching, as 0046 inferred (not measured at 0.26.0, which had no such columns).
- The single pot, watered to field capacity each morning, drains nothing and
  evaporates 6.2 kg in 60 days.
- WP8's done-when is now met on seed means, not by one plant inside §3.16's band.
