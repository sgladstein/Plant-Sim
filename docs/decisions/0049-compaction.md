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
   ponds. Rain on a pan now perches above it, and a pan that stays near saturation runs
   short of air (R5b's hypoxia), as pans do.
4. **Strength.** The penetration resistance takes the effective-stress form of Whalley
   et al. (2007), `log Q = a log(S |ψ|) + b ρ + c`, as
   `Q = 0.5 MPa (S |ψ| / σ_fc)^0.4 · 10^(2.0 (ρ − ρ0) / 1000)`: 0.5 MPa for the
   texture's own soil at field capacity, three times that at the wilting point (field
   soils read about 0.3 MPa wet and over 1 MPa dry, Ge et al. 2019), and four times
   harder for a loam pan at 1.6 Mg m⁻³, near the 2 MPa where roots are held to be
   badly limited (Bengough et al. 2011). Whalley's own coefficients were not reachable
   (the publisher's page refused the fetch), so `a`, `b` and the anchor are "~", to
   verify. Roots read `Q` against its reference `Q_ref`, the same soil water at
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
7. **A lost tip releases its laterals, and the collar must have flooded** (R5b
   corrections found here). An axis whose tip died (R5b) or a tip at a stop now forms
   laterals up to where its tip was (item 5's zone, 0 once the tip is lost), and its
   last cohort grows no fine roots; and a plant that lost a tip grows new
   hypocotyl-borne roots only if the soil at its collar was hypoxic since the last set.
   At a deep anoxic pan a bean lost its tip, regrew a full set of hypocotyl roots at a
   dry collar, sent them down to the pan and lost them again, every few days. These
   change the R5b pots: the waterlogged pot has 14.7 m of root on day 74 (9.6 at
   0.29.0) and `two_pot_sun_shade`, whose pots each lose an axis tip to their wet base
   by day 13, changes from then.
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
  - plough pan: 120.6 m of root above 15 cm against 112.3 without the pan, 3.0 m at 15
    cm and below against 22.0, none below 20 cm against 8.7; axes in the pan's layer
    0.67 mm in radius against 0.51;
  - raised bed: deepest root 38.5 cm against 22.6 in the compacted plot, 7.1 m of root
    below 20 cm against 0.03;
  - dug strip, day 45: the soil differs from the undug plot only in the strip (still
    looser after 25 days of rain and watering), which holds 7.4 m of root against 2.9;
    the rest of the root system is 26.7 m against 29.3 (−9%), the plant putting its
    root growth where the soil is loose;
  - a session that digs and tramples replays from its log and resumes from a snapshot
    taken after both bit for bit; the debug ledgers close through every run.
- **Over seeds 8, 9 and 11** (other weather): the pan always holds more root above 15
  cm (+9%, +68%, +17%) and less at 15 cm and below (−69%, −56%, −29%); in the wet season
  (seed 11) the pan is soft enough to let 21.5 m of root below 20 cm (28 m without the
  pan). The raised bed always roots deeper than the compacted plot (deepest 46 against
  27, 30 against 24, 27 against 12 cm); the dug strip holds 2.5 to 13 times the undug
  strip's root on day 45 and the rest of the root system −4 to −14%.
- **Everything else**: scenarios without zones or digging give the same daily rows as
  0.29.0 (single_pot, the simplified pot, fidelity_switch, the 30-bean plot, split_pot,
  nitrate_patch, wheat_stand, pole_trellis, drought_from_flowering checked), except
  item 7's: `two_pot_sun_shade` (seed 1.39 → 1.63 g and 2.79 → 1.93 g on day 89) and
  `m2/waterlogged_pot`.
- **Validation** (seed means): 99 checks, 76 pass, every M0 check passes; the new
  `pan_root_shift` (−89% at 15 cm and below; −65 to −95% over the four seeds) and
  `pan_root_shift_above` (+21%; +8 to +68%) pass; every other check gives 0.29.0's value.
- **Fidelity**: 111 of 111, seed-yield bias −2.57% ± 1.65%, as at 0.29.0.
- **Challenges**: every challenge log and the thirsty-summer window plans (121 plans,
  eight-season means) score as at 0.29.0, hungry soil's 14 plans also under #151's
  rain shelter; no medal moves.
- **Cost**: within this container's noise for soil without structure (best of 5,
  alternating builds: single_pot 8.0 and 8.9 against 8.3 and 8.4 ms per sim-day,
  plot_30_beans over 30 days 21.5 and 24.8 against 21.3 and 24.5 ms). A plot with
  structure costs more: `m2/plough_pan` 13.2 to 13.6 ms per sim-day against 11.2 to 12.7
  for its control.
- **Limits** (cards' known limits): no cracks or biopores, so a pan at 1.2 stops nearly
  every root in a moist season; the strength coefficients, the conductivity exponent,
  the thickening cap and the branching factor are guesses; lateral water spread still
  runs on water content, not potential, so a dense zone does not slow it; strength has
  no memory of how the soil was wetted; dug soil settles by water only; roots do not
  loosen soil and tips do not turn along a pan.
