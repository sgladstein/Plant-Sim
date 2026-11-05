# 0020: Nitrogen: soil pools, uptake, remobilization and nodules (WP24)

- **Date:** 2026-11-05
- **Status:** accepted

## Context
WP24 (design plan §7, M2) brings nitrogen in as the second authority after carbon
(§2, §3.6): soil mineral N, mineralization, leaching, uptake, leaf N driving Vcmax,
remobilization, nodules for legumes, and a fertilize action. It is done when nodulated
beans grow in N-free soil and unnodulated ones do not, and when bottom-up senescence is
emergent. Until now organ nitrogen was a fixed fraction of dry mass, used only for
maintenance respiration, and leaf aging stood in for remobilization.

The research notes for M2 (project files, research/m2-m3-bean-science.md) give the
CROPGRO-Drybean values (BNGRO048.SPE) and made the done-when numeric: nodulated over
unnodulated shoot dry mass above 3 at 45 days after emergence, and unnodulated plant N at
or below seed N.

## Decision
Model version 0.3.0 → 0.4.0. Species schema 1 → 2 (a `nitrogen` group replaces the three
fixed N fractions). Scenario schema stays 1: the new `nitrogen` map and the `Fertilize`
action have defaults. Golden hashes are regenerated.

### Soil (ps-soil)
- **Pools per cell**, all in g N: humus N, ammonium, nitrate. Loam, sand and clay get a
  bulk density (1300, 1550, 1250 kg m⁻³) to convert to mg N per kg soil (ppm).
- **Hourly, after the water steps:** mineralization humus → NH₄ at 1.5e-4 d⁻¹ and
  nitrification NH₄ → NO₃ at 0.34 d⁻¹, both at 30 °C and optimal water, × `f_T` × `f_W`
  / 24. `f_T = min(1.2, (T/30)²)`; `f_W` is the APSIM shape of water-filled pore space.
- **Nitrate moves with water**, fully mixed in the cell: drainage carries the drained
  share down and out of the bottom (leached), and lateral flow spreads it with the water
  fraction. Ammonium and humus do not move.
- **Fertilize** adds NO₃ or NH₄ (g N m⁻²) to the top layer.
- **Default soil** (`SoilNitrogen::GARDEN`): 20 ppm NO₃, 2 ppm NH₄, 1 g kg⁻¹ humus N,
  rhizobia present. A scenario sets other values per soil.
- **Ledger:** start + added + mineralized balance = stock + leached + uptake; debug
  builds assert it every hour at 1e-9 relative.

### Uptake (once a day, joint over a soil's plants)
CROPGRO NUPTAK per form, in each cell a plant reaches:
`U = L · cap · (1 − e^(−0.08 ppm)) · √FTSW_cell`, down to floors of 0.25 ppm NO₃ and
0.5 ppm NH₄, scaled to the plant's request. All plants compute from the same soil state;
where requests to a cell exceed what it holds, all are scaled alike (the water uptake
rule of decision 0012), so plant order does not matter.

Two calibrations stand for root length the sim does not draw:

- **Capacity** `cap` = 6e-3 g N m⁻¹ d⁻¹ per form, 10× CROPGRO's RTNO3. The sim draws
  about a tenth of a bean's root length (no second-order laterals or hairs; decision
  0013).
- **Root reach** = 4 cm: each root cell's length is spread evenly over the active cells
  within that distance on each axis (5 × 5 × 5 cells in a 2 cm pot, 3 × 3 × 3 in a 5 cm
  plot). Without it the drawn roots, which sit mostly in the top 8 cm, emptied their own
  cells while 23 ppm stayed untouched below. That starved a 1.5 g plant in a pot that
  held 280 mg of mineral N. Water uptake does not use the reach.

### Plant (ps-arch)
- **Organ N is state.** New tissue is built at its kind's "initial" concentration
  (leaf 0.0384, stem 0.024, root 0.016, shell 0.0352, seed 0.0376, nodule 0.048 g N per
  g DM; CROPGRO protein × 0.16). The "final" concentrations are what mining leaves.
- **Seed N** = seed mass × 0.0376 (11.3 mg in a 0.3 g seed). 70% starts in the plant's
  free N pool, and the rest is spread over the seed's organs.
- **Hourly top-up**, after allocation (§3.2 hourly step 12): growing organs are topped up
  from the pool toward their initial concentration, all by the same share when it is
  short. An organ diluted to its kind's final concentration does not grow that hour.
  Roots and nodules have a growth floor of 0, so an N-poor plant keeps exploring.
- **Daily step 0** (new, before phenology):
  1. Demand = the growing organs' deficits plus what yesterday's growth used, as the
     forecast of today's.
  2. Supply in order: the pool (without yesterday's fixation), soil uptake, then mining.
     Mining takes up to 0.12 d⁻¹ (CROPGRO NMOBMX) of the mobile N of organs that have
     stopped growing, × 0.36 (NVSMOB) before seed fill. Leaves are weighted
     `clamp(2 (1 − L/L_max), 0.1, 1)` by their light index, so shaded leaves give first.
  3. What is still short sets tomorrow's fixation, up to the nodules' capacity
     (CROPGRO's demand-driven NFIX).
  4. Nodulation: if the need takes more nodule mass than the plant has, new clusters
     (2 mg potential, 100 °C·d to full size) start on root segments in the top 30 cm,
     on the segments with the fewest nodules first. Growth per day is limited to 0.17 of
     the existing potential (NODRGM), or 14 mg at the first nodulation (DWNODI), times a
     nitrate factor: 1 at 5 ppm root-zone NO₃ or less, falling to 0.1 at 30 ppm. At most
     24 new clusters a day.
  5. Leaf drop: a mined leaf whose mobile N is below 0.1 of a new leaf's drops.
  6. Status: the mobile N share of the growing leaves and internodes. It multiplies
     expansion like the water stress factor.
- **Fixation, hourly** before allocation: each nodule pays 7.1 g C per g N from its own
  reserve, never into debt (CROPGRO RFIXN), and credits the N to the pool. Fixation per
  nodule mass is 0.05 g N g⁻¹ d⁻¹ (SNACTM) × a soil temperature trapezoid
  (4, 19, 35, 44 °C) × a drought factor (full above an expansion stress of 0.85).
  Rhizobia must be present in the soil. The carbon ledger books fixation as its own
  respiration term.
- **Leaf capacity** = aging × `clamp((Na − 0.3) / (Na_ref − 0.3), 0, 1)`, with `Na` the
  leaf's N per area, 0.3 g m⁻² the intercept (Kattge 2009, Walker 2014), and `Na_ref`
  a new leaf's N per area at the species SLA (1.26 g m⁻²). It scales Vcmax, Jmax and Rd
  together through the class table's capacity axis.
- **Capacity nodes** of the class table are now 1.0, 0.7, 0.4, 0.1 (they were 1.0, 0.8,
  0.6, 0.4, as aging alone never fell below 0.4). The worst interpolation error is 4.6%
  of A (3% before).
- **Maintenance** (0.218 g C g N⁻¹ d⁻¹, decision 0011) now reads the organs' real N. Leaves
  still skip it (their dark respiration is the gas exchange's Rd). The sown seed and the
  cotyledons now skip it too, as they respire little (decision 0013), and so does every
  organ once the plant has matured.
- **Death:** a dying organ's mobile N returns to the pool and the rest is litter. Pruned
  N leaves the ledger as removed. The plant N ledger (start + seed + uptake + fixed =
  stock + litter + removed) is asserted daily at 1e-9.

### Validation scenarios are fertilized
The M0 reference data come from fed plants, and the M0 checks were tuned on plants with
no nitrogen limit. In an unfertilized 24 cm pot of garden soil the plant now runs short.
The pot holds about 280 mg of mineral N, and a 25 g bean needs about 700 mg. Until
nodules are working it is N-limited, and seed fill takes what is left. So `single_pot`
and the five response scenarios get 300 ppm NO₃: a fertilized pot, enough that uptake
covers demand and nodules stay small.

With garden soil instead:

| Unfertilized pot, model 0.4.0 | Garden soil | Fertilized (300 ppm) |
| --- | --- | --- |
| Total dry mass at day 90 | 19.6 g | 28.8 g |
| Mean seed mass | 0.170 g | 0.253 g |
| N fixed / taken up | 295 / 134 mg | 0 / 866 mg |
| Ndfa | 69% | 0% |
| Peak nodule mass | 270 mg | 0 |

The density plots and the 30-bean plot keep garden soil. In the 30-bean plot, Ndfa is
33% and fixation costs 11% of the plants' carbon. At 35 plants per m² Ndfa is 53% and
fixation costs 17% of the plants' carbon. The research band is 30 to 60% in garden soil
(Palmero 2022), and the plan's band for active nodules is 10 to 15% of assimilate.

### Done-when
- `m2/nfree_nodulated.ron` and `m2/nfree_unnodulated.ron`: four pots of N-free sand,
  rhizobia present or absent, watered daily. 45 days after emergence (day 57) the
  nodulated shoot is about 8 times the unnodulated one. The unnodulated plant holds 9.8
  mg N against 11.3 in its seed; the cotyledons' structural N went to litter.
  `ps-validate/tests/nitrogen_free.rs` enforces both. The report carries the ratio as
  the Info check `nfree_nodulation_shoot_mass`.
- Bottom-up senescence: in the garden pot, the main-stem leaves dropped during seed fill
  sit at nodes 2 to 5, and the ones still green at maturity at nodes 6 to 8
  (`ps-arch/tests/ninety_days.rs::seed_fill_drops_leaves_from_the_bottom_up`).

### Validation report
71 checks (70 and the N-free one): 57 pass, 14 Info misses, and every M0 check passes.
Compared with model 0.3.0:

- **Now pass:** density (leaf area per plant −43% from 5 to 34 plants per m², cited
  −70%, ×½ band; it was −25%), heat pods (−17%) and heat seeds per pod (−29%).
- **Now miss:** drought shoot mass, −64% against −13 to −52%. Drought cuts fixation and
  uptake as well as carbon.
- **Shade internodes:** +23% (it was +28%; the gate is 20%). Shaded plants are short of
  N sooner, as their roots are smaller.

### Speed
The 30-bean plot takes 24.6 ms per sim-day on 4 cores over 60 days. Main measures
19.4 ms on the same machine. The hourly top-up and fixation walk every organ, and the
daily nitrogen step runs plant by plant. The perf thread is asked for a bit-identical
follow-up.

## Consequences
- An unfertilized pot is now N-limited before nodules form (days 29 to 45) and during
  seed fill: fewer, smaller seeds than a fed plant. Fertilizing is now a visible choice
  for the player.
- Leaf aging still multiplies capacity. The plan meant leaf N to replace it at M2. In a
  trial without aging, the fertilized pot lost about a fifth of its mass. That is the
  sink feedback of decision 0018 at work, not evidence for aging. Aging stays until that
  budget is revisited; a higher aging floor (0.4 to 0.7) is the first thing to try.
- Pod set does not read N, so an N-limited plant sets as many pods as a fed one and
  fills them less (garden pot: 12 pods of 0.17 g seeds). Real beans shed pods instead.
  A pod set that reads N is a follow-up.
- The root reach and the 10× capacity both stand for missing root length. A fuller
  root system (second-order laterals, roots filling the pot) would retire both.
- The soil water still has no upward flow, so nitrate below the root zone only reaches
  roots through the reach.
- `Role::Nodule` organs exist for the viewer to draw. The daily CSV has 9 new columns
  (soil mineral N, plant N, uptake, fixed, mined, unmet, status, nodule mass, fixation
  carbon), and the organ CSV has `nitrogen_g`.

## Amendment after review (model 0.5.0)
The WP24 review (project files, reviews/wp24-review.md) approved with two fixes. Model
0.4.0 → 0.5.0, golden hashes regenerated.

- **Filled seeds are no longer mined.** `mine` skipped the sown seed and nodules but not
  seeds in pods, so a filled seed gave its N back down to 0.02 g/g. It now skips them.
- **The cause of the smaller shade internode response.** Bisected at non-limiting
  uptake (capacity × 100), seed fix in:

  | Variant | Shade internodes |
  | --- | --- |
  | As built | +24.0% |
  | Capacity nodes back to 1.0, 0.8, 0.6, 0.4 | +23.8% |
  | Maintenance back to leaves-only skip | +20.6% |
  | No leaf N-per-area factor | +27.5% |
  | Model 0.3.0 | +27.6% |

  N supply explains about one point, and the class nodes nothing. The rest is the leaf
  N-per-area factor. A shade leaf has 1.2× the SLA at the same N per mass, so it holds
  less N per area and gets about 0.78 of the capacity. The shaded plant then has less
  carbon to fill its longer, thinner internodes. Lower Na and Vcmax per area in shade leaves is real acclimation. But
  real shade leaves also carry somewhat more N per mass, and the sim does not. A
  light-dependent leaf N per mass is the follow-up if the margin matters.
- **The non-limiting control is now non-limiting for shaded plants too.** At 6e-3 the
  shaded plants' small roots still ran short of N at 300 ppm, and the seed fix pushed
  shade total mass to −51%, outside the M0 gate. `uptake_capacity` is now 1.5e-2,
  25× CROPGRO's RTNO3. The sim draws about a twentieth to a thirtieth of a field bean's
  root length at 30 to 45 days (5.7 m at day 40). The reach and the capacity do not
  double count: the reach decides which cells a root draws on, and the capacity how
  fast it draws. The M0 scenarios' comments now call 300 ppm a non-limiting control
  (about 4.2 g N in a 14 kg pot), and decision 0018 notes that the M0 gates run at
  non-limiting N.
- **Nodulation reads nitrate over the root reach**, the same cells uptake uses.
- **Fertilize** is capped at 100 g N per m² per action.

With these, 71 checks: 57 pass, 14 Info misses, every M0 check passes.

- Shade internodes +24% and shade total mass −43% (it was −47%).
- Density −45%. Most of that is topsoil N running out at 35 per m², not the field's
  branching and leaf-size response. With light alone it is −25%, and with non-limiting N
  −31%. At day 40 the density_35 plants are at N status 0.53 with 12 of the plot's 16 g
  of mineral N still in the soil, below roots that reach 23 cm of 40. The response
  note says so.
- Garden-soil Ndfa is now 18% in the 30-bean plot (it was 33%), 45% at 35 per m² and 55%
  in the garden pot, against the 30 to 60% band. Fixation costs 6%, 15% and 19% of the
  plants' carbon. In N-free sand it costs 34% over the run (the review measured 40% at
  day 50). That is high against the plan's 10 to 15%, but nothing else feeds that
  plant.
- Garden pot: 19.0 g at day 90 and mean seed 0.144 g (seeds in pods no longer pay N
  back).
- The 30-bean plot with the perf speedup (#53) runs at 23.2 ms per sim-day over 90
  days on 4 cores.

