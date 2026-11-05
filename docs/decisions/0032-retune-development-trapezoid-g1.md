# 0032: Re-tune: development trapezoid, Medlyn g1, per-organ construction cost

- **Date:** 2026-11-05
- **Status:** accepted

## Context
The research thread's re-tune package (`docs/research/retune/items-1-4.md`, PR #88) checked
four "~" values of the bean against the literature:

1. Vcmax25 and Jmax25 (89.3, 159.8): keep. Round 1's "20% high" assumed the wrong kinetic
   constants; on Ci basis the values are about 87 and 160.
2. Development temperatures: the single peak 8 / 28 / 38 °C halves the development rate at
   33 °C, where CROPGRO, APSIM and Ferreira 1997 all hold it at its maximum. Proposed
   trapezoid 7 / 25 / 33 / 42 °C, with every stage sum restated so the default scenario
   keeps its calendar, and `vegetative_after_flowering` = 13 photothermal days × 18 = 234.
3. Construction cost by organ from CROPGRO's composition costs: leaf 1.39, stem 1.29,
   root 1.25, seed and pod 1.36 g C/g C (nodule as root), against 1.25 for all.
4. Medlyn g1 = 5.79 kPa^0.5 (CLM5 PFT table: soybean, spring wheat, cotton, rice), against
   4.0. About 34% more stomatal conductance at 1.5 kPa.

The package asked for 3 and 4 together, then a re-tune of seed means.

## Decision
1. **Item 1: no change.** New test `ps_physio::tests::bean_matches_pimentel_at_ppfd_700`:
   the sim's leaf at 25 °C, VPD 1.5 kPa, PPFD 700 and Ca 380 gives A within 15% of
   Pimentel 2011's 20 µmol m⁻² s⁻¹.
2. **Item 2: development trapezoid.** `Cardinal` gets `opt_high`; the rate is `T − Tb` up
   to `To`, `To − Tb` on the plateau to `To2`, then falls linearly to zero at `Tm`. New
   phenology key `t_opt_high` (wheat: equal to `t_opt`, its old shape). Bean and pole bean:
   7 / 25 / 33 / 42 °C.
   - Stage sums restated by running the default scenarios hour by hour under both
     functions (four seeds) and scaling each threshold by the ratio at the stage's hour:
     bush emergence 80 → 90, flowering 440 → 480, maturity 900 → 965; pole flowering
     575 → 613, maturity 1150 → 1232, emergence 80 → 87.
   - Organ clocks in °C·day restated by the mean ratio over their phase: vegetative ×1.08
     (phyllochron 55 → 59.5, leaf expansion 150 → 162, internode expansion 120 → 130,
     nodule expansion 100 → 108, root type emergences 50/75/120 → 54/81/130, emergence
     per depth 12 → 13 Cd/cm, leaf lifespan 600 → 650 with its range), reproductive ×1.06
     (pod expansion 145 → 154, seed fill delay 70 → 74, seed fill 330 → 350).
   - `vegetative_after_flowering` 260 → 234 (13 × (25 − 7)), as the package says.
   - Rates given per day at the optimum (root elongation) are normalised by the plateau
     rate `To − Tb` = 18 (§11.4), so at 20 °C they run 13/18 of their optimum instead of
     12/20: the bean's optimum is lower.
3. **Item 3: the code, not the values.** The species file's `construction_cost` becomes
   four keys, `construction_cost_leaf`, `_stem`, `_root`, `_fruit`, each organ kind built at
   its class's cost (`ps_physio::ConstructionCost`). All four stay 1.25 for now. With
   CROPGRO's values the shade-internode M0 gate fails: on eight world seeds the shaded
   pots' internodes are +17% longer than the control's against a +20% floor (+24% at 1.25).
   One-at-a-time runs put it on the leaf cost: a costlier leaf leaves the shaded plant less
   carbon, and its shade-lengthened internodes, which keep their potential mass, are
   carbon-limited. The heat responses weaken the same way (heat_pods -20% → -9%). That is
   the transport and priority question (WP7, decision 0009), so the per-organ values go in
   with that revisit; the source notes in the species files carry them.
4. **Item 4: g1 = 5.79** for bean and pole bean; the wheat source now names the CLM5
   table. The plantecophys fixture is regenerated at g1 5.79 (`photosyn_grid.py`, the R
   script), so CI's plantecophys job re-checks it.
5. **Re-tune:** the pole bean's `internode_last_fraction` 1.6 → 1.75 on seed means. The
   234 Cd after flowering ends its node production about 4 days sooner at 20 °C, and day-75
   height fell from 2.04 to 1.89 m.
6. Species schema 10 (after the genetics bump's 9), model 0.15.0 (after its 0.14.0),
   goldens regenerated. The bean genome's QTL `allele_sd` is CV_G × the trait default / √n
   (decision 0031), so it follows the new defaults: medlyn_g1 0.5091 → 0.7369 (18% over two
   loci), leaf_lifespan 60 → 65 Cd (10% over one locus).

## Consequences
- Validation on four-seed means: 87 checks, **68 pass** (was 67), 19 Info misses, every M0
  check passes. drought_root_shoot_cited now passes (+27%, band +21 to +84%). No check
  flips the other way.
- The calendar holds: single_pot flowers on day 43.75 and matures on day 79 (both
  unchanged), emerges on day 10.75 (unchanged).
- Pole bean (seed means): 1.61 m at day 60, 2.04 m at day 75, 2.05 m final (all pass).
- Bush bean: more leaves (24.5 from 21.5) and branches (5.5 from 4.5), peak leaf area
  0.217 m² (0.180), pods 11 (12), mean seed mass 0.26 g (0.22). The extra leaf area comes
  from the faster organ clocks at temperatures between the old and new optimum.
- Heat: development no longer slows between 28 and 33 °C, so the heat_at_flowering pots
  move through flowering faster; heat_seeds -32% (was -37%), heat_seeds_per_pod -14%
  (was -19%), still passing.
- Not done: the review of decision 0029 noted that seed number does not adjust to carbon
  after pod set, so seed size takes every shortfall (world seed 9: 42 seeds at 0.145 g).
  That is the same root as the drought and heat seed-size misses and needs a seed
  abortion rule, a later model change.
- Two tests read too narrow a sample for the new clocks. `ps-validate/tests/pole_bean.rs::staked_pole_beans_climb_and_unstaked_ones_lodge` gave -16% on world seed 7 alone (the four
  seeds span -16 to -69%; at 0.12.0 seed 7 was the other extreme, -80%), so it now runs the
  validation report's four seeds, checks every plant of each, and holds the -30 to -70% band
  on the mean, as decision 0021 first had it. `actions.rs::topping_a_pole_bean_releases_its_buds`
  cuts on day 18 instead of 22: the untopped vine now adds a node in the four days after day
  22, which lifts its apical hold above the buds under test.
- `ps-session/tests/save.rs::lens_data_never_changes_the_hash` (WP26): its crowd check
  compared two single plants on day 21, before the canopy closes, and flipped under the
  re-tune; #102 on main replaced it with a check that plants differ, which this change
  keeps.
- Every golden hash changes.
