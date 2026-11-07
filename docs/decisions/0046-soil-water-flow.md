# 0046: Soil water moves by Darcy's law (R5a)

- **Date:** 2026-11-07
- **Status:** accepted (the realism plan's R5a, `docs/research/realism-plan/r5-soil-extras.md`
  §3; the owner-approved queue after R3: soil extras, then branches on branches, then
  nitrogen along the stem)

## Context
Until now soil water moved by bucket rules (soil-and-water card, step 2): every cell
above field capacity passed its excess one cell down each hour, and the bottom cell
drained at once. A soaked pot was back at field capacity within hours, overwatering was
harmless, and water never moved up: no capillary rise, no wet layer at a pot's bottom,
and nitrate below the roots could not come back. R3a gave every cell a water potential
and R3c a conductivity (decision 0045), but no flux read them. The spec asks for
Darcy flow between cells from the potential and the conductivity, slow drainage,
capillary rise, a perched water table in pots, and pot drainage options.

## Decision
1. **Vertical flow by the Richards equation, one linearized step an hour**
   (`ps_soil::flow`; soil-and-water card equation 2). Per column, the downward flux
   between cells is `K_f ((h_i − h_{i+1}) / Δz + 1)` with `K_f` the arithmetic mean of
   the two cells' conductivities. Backward Euler with `K` and the retention slope taken
   at the start of the step (one Picard step), a tridiagonal system solved by the
   Thomas algorithm; the water then moves by the solved fluxes, so every column
   conserves water exactly. Lateral flow keeps decision 0019's diffusion step.
2. **Air entry** (Ippisch, Vogel and Bastian 2006; the spec's choice). The retention
   curve is flat at saturation up to 2 cm of suction and scaled by `1/S_c` beyond, and
   Mualem's conductivity is scaled so it is `K_s` at saturation. The curve is anchored
   on field capacity and wilting point as before, by an outer bisection on `n` around
   an inner one on `ln ψ_s`; `n` barely moves and `α` rises by 1% in clay, 3% in loam
   and 9% in sand. Without it, Mualem's conductivity with `n` near 1.2 falls to a sixth
   of `K_s` within 3 cm of saturation; with it, `K / K_s` is four to five times higher
   at every drier content. The root network's soil term (R3c) uses the same
   conductivity, so the soil now matches a young root's conductance near −0.53 MPa in
   loam instead of −0.3, and keeps a fourteenth of it at the wilting point instead of a
   sixtieth.
3. **Saturated cells hold a little more under pressure**: a numerical specific
   storage of 0.01 m⁻¹ (real soils ~1e-5), so a perched table has a pressure the solve
   sees, at most 0.0024 m³ m⁻³ under a full 24 cm pot. A step whose linearization
   leaves a cell above the column's own height of pressure (an unsaturated cell's slope
   is far larger than the storage it has left) pushes the excess up, and out of the
   top into the pond. That replaced sub-stepping on such steps, which cost 64 steps an
   hour in every rain hour.
4. **Sub-steps** only where a single step would change a cell by more than
   0.1 m³ m⁻³: 2, 4, … up to 64 equal steps, so the cost per hour is bounded. With
   0.03 a pot's every watering hour took 64 steps; with 0.1 almost every hour takes
   one. Within the hour after a soaking a column can be 0.1 to 0.2 off a finely stepped
   solution (sand, whose wetting front crosses a pot in minutes), within 0.02 after six
   hours (tested against 512 steps an hour in loam, sand and clay, 2 and 5 cm cells).
   **Speed.** The first build doubled the pot's time per sim-day (7.0 to 14.4 ms here),
   45% of it in `libm::pow` for the curve's head, slope and conductivity. The solver
   now reads them from a table per texture (4,096 even steps of relative saturation;
   the head by cubic Hermite interpolation on its exact slope, within 1e-7 of the
   curve; slope and conductivity linear, within 1e-3), and a plot's columns (256 or
   more) are solved in parallel in fixed chunks of 32, put back in column order. A
   pot's 113 columns stay on one thread (in parallel they were slower).
5. **Infiltration** keeps top-down filling but takes at most `K_s · 1 h` an hour
   (loam 10.4 mm, sand 297, clay 2). The rest ponds: 5 mm on a plot (the spec's value)
   and 25 mm in a pot (the space below the rim; with 5 mm a pot's morning watering
   would mostly have run off), then runs off. Ponded water soaks in the next hour and
   counts toward watering to field capacity.
6. **The base.** A plot drains freely (unit gradient) as before. A scenario pot's
   `drainage` is `Holes` (default: a seepage face, water leaves only while the base is
   saturated), `NoHoles`, or `Saucer(depth_mm)`: the saucer catches what drains, feeds
   the base back at its water's depth while it holds water, each column taking at most
   what is left in column order, and spills past its depth. A saucer does not
   evaporate; its water returns without nitrate (what drains into it leaches).
7. **Nitrate moves with the water**: each face's net water over the hour carries the
   upwind cell's start-of-hour concentration, each cell sending at most its store.
   Water soaking in carries nitrate down as a mixing cell (Godwin and Jones 1991, the
   CERES rule): a cell sends `p / (W + w_in)` of its nitrate and what came in, `p` the
   water passing on. Without it the day's evaporation, which now draws water and
   nitrate up, and the morning's watering, which filled cells without moving any, pumped
   the nitrate into the top layer: on the first run the 30-bean plot's Ndfa fell from
   0.59 to 0.15 and the nitrate patch's root response from +37% to +6%. With it a bare
   loam column refilled each night for 40 days ends with 40% more nitrate in its top
   2 cm, as salts gather where water evaporates.
8. **The drainage done-when.** The spec asked that a soaked loam column drain to
   within 10% of field capacity in 1 to 3 days. That cannot hold with these curves, or
   with Carsel and Parrish's own loam: by Mualem's conductivity a soaked loam column is
   still near 0.36 m³ m⁻³ after two days (Carsel and Parrish's loam, in a 24 cm column, 0.31 against its
   −33 kPa content of 0.16), since the −33 kPa field capacity is a laboratory value
   drier than what drains in days, and the curve reaches it only after weeks. The test
   reads the field definition (Veihmeyer and Hendrickson 1931: rapid drainage ends
   after 2 to 3 days) instead: a soaked 40 cm loam column sheds more than 15 mm on the
   first day, under 5 mm a day by the third, and still drains a little after a week.
   `K_s` was not tuned.
9. Model 0.27.0. Goldens regenerated. The scenario schema is unchanged (`drainage` is
   optional).

## Consequences
- **Validation** (four seeds): 72 of 96 pass (70 at 0.26.0), every M0 gate passes.
  split_pot_foraging passes (1.548, was 1.254; band from 1.5), as do fine_root_share
  (0.105) and density_leaf_area_per_plant (−41%, was −34.97% against −35%).
  ndfa_garden_plot falls out (0.24, 0.18 to 0.35 over the seeds; 0.59 at 0.26.0, about
  0.18 before R3c): the 30 plants take 10.9 g of soil N over the season against 5.7 g
  while the plot's mineral N follows nearly the same path (20 g to 12.6 g, was 11.2),
  so less is lost below the roots, and water drawn up toward the roots and the surface
  brings nitrate with it (inferred from the totals; leaching is not in the daily
  rows). nitrate_patch_proliferation falls from +37% to +16% (+2 to +32%). In a bare
  pot under the same daily sun and watering, the patch keeps 22% of its nitrate after
  45 days, against 30% with no vertical movement at all: sideways diffusion (decision
  0043) takes most of it either way, so most of the drop is in how the roots respond
  to a more uneven pot, not in the patch washing out; not isolated further.
  drought_pods −75% (was −70%), drought_shoot_mass −67% (−62%), shade_total_mass −37%
  (−45%), heat_seeds −34%; pods 9.25 and seed 6.8 g in the watered pot (10.8 and
  8.1 at 0.26.0; 4.0 to 9.5 g over the four seeds, not yet checked on eight). The R2b test pot (now watered through its surface, so the nightly
  water carries nitrate back down) gives 9.37 against 7.16 m of laterals in the patch
  and a wet:dry ratio of 1.67 against 0.86 without the rules. The flow table (item 4)
  moved no verdict: the numbers here are with it.
- **Test pots.** The whole-plant tests in `ps-arch/tests` refilled their pots in
  place each night, which now leaves at the top the nitrate the day's evaporation
  draws up (the thickening test's day-56 hypocotyl fell to 4.3 mm, under its band).
  They now water through the surface as the world does (5.5 mm). `ninety_days.rs`
  keeps refilling in place: watered through the surface its eight seeds average 9.0
  pods, inside §8.2's 6 to 20, but seed 7 sets 8, under the single-plant 10 to 20 of
  §3.16 that WP8's done-when reads.
- **Fidelity** (`ps-cli fidelity`): 111 of 111 outcomes pass, seed-yield bias −0.65% ±
  2.57% (was +1.73%). The largest gaps are in the drought pot: lowest leaf potential
  +10.1% (−21.5 against −19.3 MPa), pods −6.7% and seed −5.9%, inside their seed
  ranges.
- **Challenges** (51 strategies, 8-season means, docs/game/challenges.md): the pot's
  bare soil now evaporates water drawn up from below, 2.4 of the 7.5 L the
  thirsty-summer fixture's pot gives up (1.0 L by day 30), against about 0.2 L before.
  Every thirsty-summer plan loses two to five pods (best 6.4, was 9.5; the fixture
  3.4, spreading the water 1.8), and its medals come down from 5 / 7 / 8.5 to
  3 / 4.5 / 5.5. Its plans sit near the pod-abortion threshold: the flow table's 1e-5
  change in the curves moved single plans by up to 1.2 pods on the 8-season mean.
  Hungry-soil plans gain 1 to 2 g (probably the fertilizer soaking down with the
  watering), medals 5 / 7 / 7.9 to 6 / 8 / 9. The heat wave keeps its medals (late
  sowing 8.1 to 11.8, early 3.8 to 4.5). The fixture tests hold.
- **Cost** (this container, best of 3, against main built here): single_pot 7.0 to 7.2
  → 8.8 to 9.2 ms per sim-day (+25 to 30%), single_pot_simplified 5.5 to 5.7 → 7.1 to
  7.3, plot_30_beans over 30 days 14.8 → 19.3 to 20.4 (+30 to 38%). The column solve
  is about 3,000 instructions per pot column-hour, spread over the table lookups, the
  system's assembly, the Thomas solve and the nitrate moves; no single hot spot is
  left. Performance has it next (the pot was already over its ~4 ms budget).
- A soaked loam pot with holes rests near saturation (about 0.43 m³ m⁻³): 24 cm of
  height holds only 24 cm of suction, as garden soil in a pot does. Pots watered to
  field capacity each morning are never soaked (the watering adds only the deficit),
  but rain on an outdoor pot now stays until the plant uses it.
- Bare soil now loses far more to evaporation, as water rises to the drying surface:
  30 cm of loam under 12 h of 500 W m⁻² a day gives up 24 mm in 20 days (8 by day 3),
  against 4 mm when only the top 2 cm dried (stage-two evaporation, Ritchie 1972,
  is of this size).
- Open: lateral flow still diffuses by decision 0019's rule; infiltration ignores how
  much faster dry soil drinks at first; ponded and saucer water do not evaporate;
  waterlogging (R5b) and compaction (R5c) come next.
