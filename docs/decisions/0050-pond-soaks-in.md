# 0050: Ponded water soaks in during the hour's flow

- **Date:** 2026-11-07
- **Status:** accepted (a correction to R5a, decision 0046, found while building the R5
  explorer after R5c; the owner was told it would be fixed next)

## Context
A 24 cm loam pot with holes soaked with 60 mm over three hours kept a 13 mm pond on its
surface for 56 hours, its soil a hair under saturation, and drained 0.23 mm an hour
until the pond was gone; then it drained to rest in five hours. Loam's saturated
conductivity is 10.4 mm an hour, so the pond should have gone in about two.

The pond only entered the soil through infiltration (equation 1), which fills each
column top-down up to saturation at the start of the hour. A nearly saturated column has
almost no room, so the pond could only refill what the last hour's flow had drained, and
that flow was itself small: the flow solve (equation 2) had no water coming in at the
top during the step, and with every cell at saturation it found a nearly hydrostatic
profile after releasing a few tenths of a mm. The seepage face added to it: it read
whether the base was under pressure at the start of a step, and a cell just filled to
saturation sits at the air-entry head (−2 cm), below the face's threshold (−1 cm for 2
cm cells), so the face stayed shut for a step the solve then pressurized.

The same throttle held for any column with water standing on it: a plot after a storm
kept its pond, and from R5b the pond evaporates first, so standing water that should
have soaked in was partly lost to the air.

## Decision
1. **The pond is a head at the surface during the flow solve.** Water still ponded
   after infiltration stands on the column at depth `d_p` and enters the top cell as
   `q = K_t ((d_p − h_1) / (Δz/2) + 1)`, `K_t` the mean of the top cell's `K` and `K_s`
   (as the saucer's base, equation 2), solved implicitly with the column. It takes at
   most the pond (a fixed flux of what is left, solved again, as the saucer does) and
   never sends water back up. Pond water carries no nitrate. Over an hour's sub-steps
   the head falls as the pond empties.
2. **A seepage face opens within the step.** A pot's face closed at the start of a step
   opens, and the step is solved again, if the solve leaves the base cell's head above
   `−Δz/2`.
3. Model 0.31.0; goldens regenerated.

## Consequences
- **Done-when** (`ps_soil::tests::a_soaked_pot_with_holes_sheds_its_pond_in_hours`): the
  soaked pot has no pond after hour 5 (was hour 59), drains 99% of its outflow within
  12 hours, and rests where it rested before (base saturated, top at the curve's content
  23 cm above the base, 0.413).
- **The waterlogged pot** (R5b, `m2/waterlogged_pot`, world seed 7): its holes open on day
  49 and it now drains that day. It still loses its fine roots (5.9 → 0.1 m by day 49)
  and wilts in the wet days (−1.48 against −0.80 MPa), but its laterals and axis tips
  outlive the soak: its least root after it is 31.0 m (6.1 at 0.30.0), it regrows one
  hypocotyl set on day 51, no longer wilts to −5.5 MPa after the drain, and has 135.0 m
  of root on day 74 (73.7 at 0.30.0) against the control's 211.5; seed 5.02 g (4.43).
  `ps-world/tests/waterlogging.rs` now asks for under three quarters of the control's
  root on day 74 (was under half).
- **Waterlogging and yield.** Over world seeds 7 to 14 the flooded pot's seed against
  its control is +1.4% (−10.9% at 0.30.0, −47% at 0.29.0), from −22% to +91% by season:
  a pot that drains keeps enough root to fill its pods, and pod set reads the root
  zone's FTSW, which is 1 in wet soil, so the flood never aborts a flower. A flood at
  flowering should cost seed; pod set that reads the plant's water status or the root
  zone's air, with a waterlogging seed-yield check, is logged as the next model item
  (model to-do).
- **Compaction** (R5c): the plough pan and the raised bed are unchanged. The compacted
  plot, whose rain ponds on its hard top, changes from day 0: deepest root 22.3 cm
  (22.6), fine roots 0.25 m on day 70 (2.09), seed 2.21 g (1.78); the raised bed still
  roots to 38.5 cm. The dug strip holds 8.05 m of root on day 45 against 2.78 undug
  (7.41 and 2.92 at 0.30.0).
- **Scenarios** (every scenario run against 0.30.0): those without standing water give
  0.30.0's daily rows and organs. Changed: `two_pot_sun_shade` from day 6 (pods sun 4 →
  3, shade 0 → 4; seed 2.01 and 2.05 g; both 20 cm loam pots still lose their fine roots
  and wilt to −5.6 and −8.2 MPa, as a pot that shallow sits within its capillary fringe
  at rest, logged in the model to-do), `m2/waterlogged_pot` from day 45,
  `m2/single_pot_weather_file` from day 38 (15 pods, 9.63 g), `viewer/wilt_and_recover`
  from day 55, and `m2/compacted_plot` and `m2/dug_strip` from day 0.
- **Tests changed**: `standing_water_evaporates_before_the_soil` (a saturated closed
  clay column now takes in the little its cells store under a 2 mm pond, under 0.1 mm),
  `water_perches_on_a_pan` (three days on, the layer above the pan is 1.3% wetter than
  without the pan, was over 1.5%: the flat column's pond no longer lingers and feeds its
  top) and the waterlogged pot's recovery bound (above).
- **Validation** (seed means): 99 checks, 76 pass, every M0 check passes, no verdict
  moves. The rained validation pot changes with its seed 9, which waterlogged after rain
  and now drains (17 → 14 pods): pods 12.25 → 11.0, total mass 29.3 → 28.1 g, seed
  8.33 → 7.85 g, mean seed mass +4%, `feedback_loss_fill` 0.48 → 0.51 (Info, still
  out). `waterlogging_root_loss` −100% → −99.6%; `pan_root_shift` −54.1% → −53.9% (seeds
  −17% to −75%: in the wettest, seed 9, both plots hold under 0.6 m below 15 cm);
  `pan_root_shift_above` +14.1% → +16.2%; `density_leaf_area_per_plant` −40.1% →
  −40.7%; the pot's height −1.3%; the rest move by under 1%.
- **Fidelity**: 111 of 111, seed-yield bias −2.32% ± 1.63% (−2.57% at 0.30.0).
- **Challenges** (8-season means): no threshold moves. Thirsty summer's fixture 10.0 →
  10.38 pods; over its 94 plans the mean moves −0.11 pods (−1.75 to +0.88; the plans sit
  at the pod-abortion threshold), and 10 change tier (gold 22 → 20), the best 11.0 →
  10.6. The heat wave's 13 sowing days move by up to 0.5 pods (fixture 11.0,
  unchanged; none changes tier); hungry soil, under its rain shelter, is unchanged.
- **Cost**: at the edge of this container's noise. Over seven alternating runs (best of
  5 each) single_pot takes 8.4 to 9.0 ms per sim-day against 8.1 to 8.9 at 0.30.0
  (+3% on the mean); over four, plot_30_beans over 30 days 21.9 to 24.1 against 20.3
  to 22.4 (+6%). Neither ponds, so the difference is the solve's extra branches and
  layout, not extra solves; Performance re-times on the owner's PC.
- **Limits**: the pond still enters only at its own head (no surface sealing or
  crusting); lateral flow still ignores the pond; infiltration's `K_s · 1 h` cap on
  the hour's new water is kept, so a storm's first hour is unchanged.
