# 0047: Daily rows carry the soil's water and nitrogen totals; WP8's counts at 300 ppm

- **Date:** 2026-11-07
- **Status:** accepted (R5a code review, `reviews/r5a-soil-water-flow-0046-review.md` in
  the project files, findings 2, 3, 5 and 7)

## Context
R5a (decision 0046) moved where water and nitrate go, but the daily rows held only the
soil's water and mineral nitrogen, so the review had to infer from the totals whether
the 30-bean plot kept more nitrogen by leaching less or by mineralizing more. R5b
(waterlogging) and R5c (compaction) will move these again.

The review also asked that `ninety_days.rs`, WP8's done-when test, water its pot through
the surface as the other whole-plant tests and the world do. Watered that way in the
driver's garden nitrogen its seed 7 sets 8 pods, under the 10 to 20 of design plan
§3.16, and the eight seeds the test runs average 9.0. WP8 came before the nitrogen
model; garden nitrogen arrived with WP24.

## Decision
1. **Five soil totals in each daily row** (`DayEnv`), for the plant's soil, summed since
   the start of the run: water drained out of the base (with a saucer's spill), water
   run off, water evaporated from the surface, nitrate drained out of the base, and
   humus nitrogen mineralized. They come from the soil's water ledger and nitrogen
   ledger, which already kept them. The CSV adds `soil_drained_total_kg`,
   `soil_runoff_total_kg`, `soil_evaporated_total_kg`, `soil_leached_n_total_g` and
   `soil_mineralized_n_total_g` at the end of its columns; `_total` columns are sums
   since the start. With the plants' daily uptake the mineral nitrogen budget closes:
   start + mineralized + fertilizer − leached − uptake = mineral N now. The rows close
   from one row to the next; day 0's row already includes day 0's flows, so the stock
   at the start is the first row's mineral N less its mineralized total, plus its
   leached total and its uptake. Fertilizer is not in the rows; it is in the action log.
2. **Model 0.28.0.** No result changes: every existing column of the single pot, the
   split pot and the 30-bean plot is identical to 0.27.0 for 60 days. The rows are part
   of the state hash, so the goldens move, and a 0.27.0 save would no longer replay to
   its hashes.
3. **`ninety_days.rs` waters through the surface**, and its two count tests grow the
   plant at the world pot's non-limiting 300 ppm nitrate (decision 0020), the soil
   `seed_fill_mines_first_in_a_fertilized_pot` already used, so they check what WP8
   promised in the conditions it was written for (code review of this change). §3.16's
   10 to 20 pods holds on seed 7 (11) and on the eight-seed mean (12.4; seeds 1 to 8
   set 13, 16, 12, 12, 12, 10, 11 and 13); each plant keeps 4 to 30. The other tests
   stay in garden soil.

## Consequences
- **The plot's season at 0.28.0** (plot_30_beans, 90 days): mineral nitrogen 20.02 g
  before day 0 → 12.58 g, with 5.05 g mineralized, 1.63 g leached and 10.86 g taken up
  (2.8 g fixed), which closes exactly; the soil drains 14.7 kg and evaporates 393 kg. At 0.26.0 the plot went 20.1 → 11.2 g on
  5.7 g of uptake, so mineralized minus leached was −3.2 g; if it mineralized about as
  much, it leached near 8 g. Most of the 6.5 g that R5a keeps in the plot is then less
  leaching, as 0046 inferred (not measured at 0.26.0, which had no such columns).
- The single pot, watered to field capacity each morning, drains nothing and
  evaporates 6.2 kg in 60 days.
- WP8's done-when is read at 300 ppm: in garden nitrogen a lone pot plant now sets
  fewer pods than §3.16's band (mean 9.0 over eight seeds).
