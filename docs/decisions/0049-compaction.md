# 0049: Compacted and loosened soil (R5c)

- **Date:** 2026-11-07
- **Status:** accepted (the realism plan's R5c, `docs/research/realism-plan/r5-soil-extras.md`
  §5; the owner-approved queue: soil extras, then branches on branches, then nitrogen
  along the stem)

## Context
After R5a (water flow) and R5b (waterlogging) every soil cell of a texture was the
same soil: the same air, the same conductivity, and no strength at all, so roots grew
as fast through a plough pan as through a dug bed. The spec asks for a bulk density
per cell that sets the cell's saturation and conductivity, a penetration resistance
from density and water, roots that slow, thicken and branch in hard soil, scenario
layers (a plough pan, a raised bed), `Dig` and `Compact` actions, and dug soil that
settles back with rain. Done when: a plot with a compacted layer at 15 cm shows fewer
and thicker roots below it and more above it; a raised bed (loose 30 cm) grows deeper
roots than flat compacted soil; digging a strip mid-run changes root growth only
there; determinism and ledgers hold.

## Decision
1. **Bulk density per cell, only where it differs** (`ps_soil::structure`;
   soil-and-water card equation 13). A grid has no structure until a scenario's density
   zone, a `Dig` or a `Compact` sets one; without it every cell is at its texture's
   density and every code path is the old one, so every existing scenario's daily
   results are unchanged, except item 7's. With structure each cell keeps its density,
   the density it settles to, the density it was loosened to, and the water that has
   reached its column since (all in the snapshot and the hash).
2. **Hydraulic classes.** A cell's density, rounded to 1% steps of the texture's,
   picks a class: saturation loses the solid volume added,
   `θ_s = θ_s0 − (ρ − ρ0) / 2650`, at least field capacity + 0.02; field capacity and
   the wilting point stay, and the retention curve is anchored through them again (the
   spec's re-solve of `n` and `α`); `K_s = K_s0 (φ_e / φ_e0)^4` on the effective
   porosity `φ_e = θ_s − θ_fc` (Ahuja et al. 1984) instead of the spec's
   `(BD_ref / BD)^m`, which has no published `m` and does not know that a dense soil
   loses its large pores first. A loam pan at 1.2 times (1560 kg m⁻³) keeps 8% air at
   field capacity and conducts a 23rd as fast. Each class and its flow table are built
   once per process, shared and looked up through a per-grid palette; the flow,
   infiltration, potential, aeration, nitrogen and root-network code reads each cell's
   class. The palette is rebuilt from the densities (not stored), so a snapshot stays
   small.
3. **Water through layers.** Between cells of different classes a face conducts
   `K_s,h (K_i/K_s,i + K_j/K_s,j) / 2`, `K_s,h` the harmonic mean of the saturated
   conductivities; within a class it keeps the arithmetic mean of `K`. With the
   arithmetic mean alone a pan passed water at half the loam's rate and nothing
   perched. Infiltration's top-down fill (R5a) let rain jump past a pan the same way,
   so in a grid with structure water filling a cell passes every cell above it, no
   more than a class's `K_s · 1 h` entering any cell below the top in an hour; the rest
   ponds. Rain on a pan now perches above it. A loam pan at 1.2 is short of air even at
   field capacity (8.2% air, under R5b's 10%), so in a moist season it stays mildly
   hypoxic (`h` about 0.25), as pans do.
4. **Strength.** The penetration resistance takes the effective-stress form of Whalley
   et al. (2007), `log Q = a log(S |ψ|) + b ρ + c`, as
   `Q = 0.5 MPa (S |ψ| / σ_fc)^0.4 · 10^(2.0 (ρ − ρ0) / 1000)`: 0.5 MPa for the
   texture's own soil at field capacity, three times that at the wilting point (field
   soils read about 0.3 MPa wet and over 1 MPa dry, Ge et al. 2019), and four times
   harder for a loam pan at 1.6 Mg m⁻³, near the 2 MPa where roots are held to be
   badly limited (Bengough et al. 2011). Whalley's own coefficients were not reachable
   (the publisher's page refused the fetch), so `a`, `b` and the anchor are "~", to
   verify. The density slope is steeper than the two fits of Whalley's form the review
   found quoted (Zhang, Gao, Ren and Whalley: 0.93 per Mg m⁻³ for Canadian soils, 1.45
   for a loamy sand, with stress exponents 0.35 and 0.58 that bracket ours), on
   purpose: at 0.93 a loam pan reaches a pan's strength only near 1.4 times the
   texture's density, where item 2 holds saturation at field capacity + 0.02, so the
   pan would be anoxic at field capacity and nearly shut to water, and its air rather
   than its strength would stop roots. At 2.0 the 1.2 pan has 8% air and 1.8 MPa at
   field capacity. Roots read `Q` against its reference `Q_ref`, the same soil water at
   the texture's density: the species' elongation rates were measured in ordinary soil
   and the dry-soil factor (R2b) already slows roots in dry soil, so only the
   density's part may change a root.
5. **Roots** (herb-architecture card; species schema 20 adds `root_impedance_half`,
   `root_impedance_thickening`, `root_impedance_max_thickening` and
   `root_impedance_branching`; a species without them takes no notice):
   - a tip's clock factor gains `g(Q_ref) / g(Q)`, `g(Q) = 1 + (Q / Q_half)²`, at most
     `g(0.5 MPa)`: halved at `Q_half` against soil with no strength, `Q_half` 1 MPa for
     the beans and 2 for wheat (0.8 to 2 MPa, Bengough et al. 2011; wheat lost 88 to 97%
     over 4 MPa, Materechera et al. 1991). The spec's `1 / (1 + Q / Q_half)` keeps 20%
     of the rate at 4 MPa with `Q_half` 1, against the 3 to 12% Materechera et al.
     measured; the square keeps 6%. A bean tip in a loam pan at 1.2 at field capacity
     (1.8 MPa against 0.55) grows at 30% of its rate; in a loose bed at most 1.25 times
     its rate;
   - a segment or lateral cohort formed where `Q > Q_ref` is `1 + 0.33 (Q − Q_ref)`
     times wider, at most twice (Ge et al. 2019: wheat seminal roots 17 to 20% wider at
     0.75 than at 0.19 MPa), and its specific length falls by the square;
   - laterals behind a slowed tip stand for `1 + 1.0 (1 − f_Q)` times their roots
     (Goss 1977, barley; "~");
   - the unbranched zone behind an axis tip shrinks with the tip's speed, `a f_Q`: it is
     what the tip grows while its laterals emerge. Without this a tip stopped at a pan
     kept its last 8 to 10 cm (the bean axes' apical zones) unbranched for good, and the
     pan plot had less root above the pan than the plot without one.
6. **Settling** (WEPP, Alberts et al. 1995): dug soil starts at 0.667 of the texture's
   density (eq. 7.7.1) and closes the gap to the density it settles to by
   `R / (R + 10 mm)`, `R` the water that has reached its column's surface since the
   dig (eq. 7.7.9, with the whole gap as WEPP's `Δρ_max`). Digging resets what the
   soil settles to to at most the texture's density, so a pan dug through is gone; a
   trampled path (`Compact`, at least 1.15 times) stays. The spec's "a fraction per
   wetting" is WEPP's form on the water.
7. **R5b corrections found here.**
   - *Each root keeps its own death clock.* R5b shed a root once more than half its
     length lay in cells whose own count of hypoxic hours had reached the species'
     time, and a cell counts from when it went short of air, so a fresh tip entering a
     cell that had been mildly short of air for three weeks died at once. In the pan,
     short of air at field capacity (item 3), the old rule killed every tip that
     reached it from about day 20 (14 tips on seed 7, no root below 17.5 cm), whatever
     its strength. Each root now keeps, for each cell it is in, the cell's oxygen
     deficit summed over the hours it has been there since the cell last had air
     (`RootSpan::exposure`, brought up to date from the cell's count at the daily
     check and started at 0 when the root reaches the cell), and dies once more than
     half its length has its own count at the time. In a flood the roots and the cells
     share the clock, as R5b built it for. (Code review of #153.)
   - *A lost tip releases its laterals.* An axis whose tip died or a tip at a stop now
     forms laterals up to where its tip was (item 5's zone, 0 once the tip is lost),
     and its last cohort grows no fine roots.
   - *The collar must have flooded.* A plant that lost a tip grows new hypocotyl-borne
     roots only if the soil at its collar was hypoxic at most one axis death time
     (120 h for the beans) before the loss, or since it, and one flood gives one set.
     At a deep anoxic pan a bean lost its tip, regrew a full set of hypocotyl roots at
     a dry collar, sent them down to the pan and lost them again, every few days.
   These change the R5b pots: the waterlogged pot, whose base stays short of air after
   it drains, has 73.7 m of root on day 74 (9.6 at 0.29.0); `two_pot_sun_shade`, whose
   20 cm pots drown at the base, changes from day 10; and `m2/single_pot_weather_file`
   from day 42.
8. **Scenarios and actions.** `density_zones: [(zone, factor)]` on a scenario (factor
   0.6 to 1.4 of the texture's density, applied after nitrate patches); a
   `SoilZone { soil, east_cm, north_cm, depth_cm }` spans east and north offsets from
   the soil's center (either left out for the whole width) and a depth range. New
   actions `Dig(zone)` and `Compact(zone)`, each its own action type in the game (no
   challenge allows them yet). New scenarios, a bush bean in the middle of a 1 m loam
   plot 60 cm deep, watered 6 mm every other morning on top of the temperate-summer
   rain: `m2/plough_pan` (15 to 20 cm at 1.2), `m2/plough_pan_control`,
   `m2/raised_bed` (top 30 cm at 0.85), `m2/compacted_plot` (all at 1.15) and
   `m2/dug_strip` (the compacted plot dug on day 20 from 5 to 25 cm east, 30 cm deep).
   Rain alone dried the plot into drought, and daily watering to field capacity
   waterlogged the soil above the pan, either of which hid the pan.
9. **Checks.** Validation adds the derived variables `root_length_above_<D>cm_m` and
   `root_length_below_<D>cm_m` and the Info responses `pan_root_shift` (root length at
   15 cm and below, pan against no pan, −100 to −20%) and `pan_root_shift_above` (above
   15 cm, 0 to +100%). The done-when test is `ps-world/tests/compaction.rs`; the curves
   are checked in `ps-arch/tests/root_plasticity.rs`, the soil in `ps_soil::tests`.
10. Model 0.30.0, species schema 20. Goldens regenerated.

## Consequences
- **The done-when** (world seed 7, day 70 unless said):
  - plough pan: 114.6 m of root above 15 cm against 112.3 without the pan, 5.6 m at 15
    cm and below against 22.0, 2.3 m below 20 cm against 8.7, the deepest root at 39.5
    cm against 42.5; no root died to the pan's air; axes in the pan's layer 0.65 mm in
    radius against 0.48 in the same pan without the thickening response (0.51 without
    the pan);
  - raised bed: deepest root 38.5 cm against 22.6 in the compacted plot, 7.1 m of root
    below 20 cm against 0.03;
  - dug strip, day 45: the soil differs from the undug plot only in the strip (still
    looser after 25 days of rain and watering), which holds 7.4 m of root against 2.9;
    the rest of the root system is 26.7 m against 29.3 (−9%), the plant putting its
    root growth where the soil is loose;
  - a session that digs and tramples replays from its log and resumes from a snapshot
    taken after both bit for bit; the debug ledgers close through every run.
- **Over seeds 8 to 11** (other weather): the pan always holds more root above 15 cm
  (+7%, +38%, +17%, +17%) and less at 15 cm and below (−47%, −78%, −48%, −29%). Seed 11
  is the driest of the five seasons at the pan's depth (θ 0.248, 10.4% air): the pan is
  at its hardest there (2.1 MPa), never short of air, and lets 21.5 m of root below 20
  cm (27.9 m without the pan). Seed 9 is the wettest (θ 0.30, 5% air): its pan is soft
  but short of air, and there the pan's own death clock stops the roots (4 tips lost;
  without the pan only 0.5 m of root gets below 15 cm). The raised bed always roots
  deeper than the compacted plot (deepest 30 against 24 cm, 27 against 18, 46 against
  27 on seeds 8, 9 and 11); the dug strip holds 2.5 to 14 times the undug strip's root
  on day 45 and the rest of the root system −4 to −9%.
- **Everything else**: scenarios without zones or digging give the same daily rows as
  0.29.0 (single_pot, the simplified pot, fidelity_switch, the 30-bean plot, split_pot,
  nitrate_patch, wheat_stand, pole_trellis, drought_from_flowering checked; every
  scenario in `data/scenarios` compared in daily rows and organs), except item 7's
  three:
  - `two_pot_sun_shade` from day 10: on day 89 the sunny pot has 4 pods and 3.33 g of
    seed (2.79 at 0.29.0), the shaded pot none (2 pods, 1.39 g); their 20 cm loam pots
    drown at the base (the seepage stall, the next fix), so the scenario's contrast is
    a drainage story until that fix;
  - `m2/waterlogged_pot` from day 50: 73.7 m of root on day 74 (9.6), 12 pods (13),
    4.43 g of seed (2.34);
  - `m2/single_pot_weather_file` from day 42: 19 axes (49, one hypocotyl set where
    there were three), 13 pods (11), 9.63 g of seed (9.39).
- **Validation** (seed means): 99 checks, 76 pass, every M0 check passes; the new
  `pan_root_shift` (−54% at 15 cm and below; −43 to −75% over the four seeds) and
  `pan_root_shift_above` (+14%; +2 to +37%) pass; every other check gives 0.29.0's value
  (the waterlogged pot's fine roots are gone by day 49 as before, and the rained
  validation pot's wet season, seed 9, is unchanged). With R5b's clock read per cell
  the pan's shift was −89%: the pan's air, not its strength, did most of it. Over eight
  seeds `pan_root_shift` is −44% (−30 to −75%) and `pan_root_shift_above` +13% (−0.1 to
  +37%: in one season of the eight the pan plot has as much root above 15 cm as its
  control, not more).
- **Fidelity**: 111 of 111, seed-yield bias −2.57% ± 1.65%, as at 0.29.0.
- **Challenges**: every challenge log and the thirsty-summer window plans (121 plans,
  eight-season means) score as at 0.29.0, hungry soil's 14 plans also under #151's
  rain shelter; no medal moves.
- **Cost**: within this container's noise for soil without structure (best of 5,
  alternating builds: single_pot 8.0 and 8.9 against 8.3 and 8.4 ms per sim-day,
  plot_30_beans over 30 days 21.5 and 24.8 against 21.3 and 24.5 ms). A plot with
  structure costs more: `m2/plough_pan` 13.2 to 13.6 ms per sim-day against 11.2 to 12.7
  for its control.
- **Limits** (cards' known limits): no cracks or biopores, so roots cross a pan at 1.2
  only by crawling through it; the strength coefficients, the conductivity exponent,
  the thickening cap and the branching factor are guesses; lateral water spread still
  runs on water content, not potential, so a dense zone does not slow it; strength has
  no memory of how the soil was wetted; dug soil settles by water only; roots do not
  loosen soil and tips do not turn along a pan.
