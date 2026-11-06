# 0043: Roots respond to dry and nitrate-rich soil (R2b)

- **Date:** 2026-11-06
- **Status:** accepted (the realism plan's R2b, owner-approved queue)

## Context
Since R2a (decision 0037) roots grow by thermal time, gravity and noise alone: they
neither slow in dry soil nor favour wet or nitrate-rich soil. The realism plan's R2b
(`docs/research/realism-plan/r2-root-system.md` §4, with the research thread's numbers)
adds five rules and three done-whens: a split pot whose watered half holds at least 1.5
times the root length of its dry half by day 45, a nitrate patch with at least 1.5 times
the lateral and fine-root length of the same cube in plain soil, and a uniform soil
within 5% of R2a.

Two soil changes were needed first. Nitrate spread sideways at the water's exchange
fraction, 0.1 per hour in pot cells, an effective diffusivity of about 1e-8 m² s⁻¹, some
20 times nitrate's in moist soil; a patch evened out within days. And the scenarios had
no way to split a pot or to make a patch.

## Decision
1. **Soil.** Laterally, nitrate now moves with the water's flux at the concentration of
   the cell the water leaves, plus diffusion at 5.1e-3 per hour in 2 cm cells,
   `D_l θ^{7/3} / θ_s² / Δx²` (Millington–Quirk at field capacity, about 5.7e-10 m² s⁻¹,
   Barber's value for nitrate in moist soil). A scenario can give a pot a
   watertight divider across its center (`dividers`), water a strip of columns
   (`Irrigation::ToFieldCapacityStrip`, the strip's own deficit) and scale the starting
   nitrate of a cube of cells (`nitrate_patches`; a cell is in when its center lies
   strictly inside the cube).
2. **Rules** (species schema 16, seven constants, the same values for the three
   species; wheat's are the bean's, unchecked):
   - dry soil slows elongation: a growing root organ's expansion clock runs at
     `max(0.3, clamp(θ/0.3, 0, 1))`, `θ` the relative available water of its tip's cell,
     set for the day after the layout;
   - xerobranching: no lateral or fine-root cohort where the parent's cell is below `θ`
     0.1, and none there later;
   - proliferation: a cohort stands for `m · F` roots, `F` the mean along its path of
     `(1 + a_N r_N)(1 + a_W r_W)`, `r = clamp((v − v̄)/v̄, 0, 2)`, `a_N` 1, `a_W` 0.5;
   - hydropatterning: the first golden-angle candidate whose tip cell holds at least
     the candidates' median `θ`, where they differ;
   - hydrotropism: an axis tip's direction gains 0.05 times the unit gradient of `θ`
     over its cell's six neighbors.
3. **What counts as rich.** `v̄` is the larger of the soil's mean over its active cells
   and the root-length-weighted mean over the plant's root cells. The spec's root-zone
   mean alone (root-weighted) let the roots' own depletion make every fresh cell look
   rich; the soil's mean alone let the nitrate that water carries to the roots, and
   leaves there when their uptake saturates, look rich to the roots already in it.
4. **Path mean.** A cohort is one straight organ, so it proliferates by the mean of its
   path rather than at its base: a cohort born outside a patch and crossing it
   proliferates in part (outside the patch too).
5. **Scenarios** `m2/split_pot`, `m2/nitrate_patch`, `m2/nitrate_patch_control` and two
   Info checks, `split_pot_foraging` and `nitrate_patch_proliferation`. All three pots
   are under a rain shelter and sow the bean exactly at the center (no sowing scatter),
   as an experimenter would. The split pot's stem stands on the divider. The patch is
   an 8 cm cube 6 to 14 cm deep on the tap root's path, as the experiments run the
   primary root through the patch; the check counts laterals and fine roots only.
6. No change to the §3.2 step list: the clock is set with the root cells at daily step
   6 and read at hourly step 11; the rules sit in daily step 5.

### Tried and not kept
Six world seeds unless noted; "uniform" is single_pot against the rules off. These runs
used the first diffusion fraction, 4.6e-4, which counted `θ` twice (11 times too slow;
Code review found it).
- **The root-weighted mean alone** (the spec's root-zone mean): in world seed 7 the
  uniform pot grew 21% more root length by day 45, and most new cohorts anywhere in the
  garden-soil pot stood for 1.5 to 3.6 times their roots once the roots had depleted the
  soil around them.
- **The soil's mean alone:** uniform root length +11% at day 45 and +12% at maturity,
  fine roots +18 to +39%, from nitrate gathered in the rooted cells of the 300 ppm pot;
  patch 1.58.
- **A dead band** (no response below 1.5 or 2 times the mean): with the soil's mean,
  uniform root length still +12% at maturity; with the larger of the two, patch 1.48 and
  uniform +4 to +6%, for one more constant.
- **`a_N` 2** (up to 5 times): patch 1.34 to 1.41 before the rain shelter; **`a_W` 1**:
  split pot unchanged (1.46).
- **The spec's patch placed 6 cm east of the stem, in the rain:** the axes hardly
  reach it, so only laterals born outside cross it (1.05 to 1.23), and a wet spell
  flushed it (594 to 38 mg N/kg in nine days in one seed).

## Consequences
- Model 0.24.0, species schema 16; every golden hash moves.
- Six world seeds (7 to 12), day 45:
  - nitrate patch: lateral and fine-root length in the cube 1.48 times the control (1.33
    to 1.65); without the rules 1.01;
  - split pot: wet:dry root length 1.45 (1.28 to 1.62); without the rules 1.13 (0.78 to
    1.35). The unwatered half falls only to about 0.4 of available water by day 45, so
    the dry-soil rules act late; the watered half supplies about two thirds of the
    water from day 20 (seed 9). Code review: stopping the east half's water on day 10
    or 15 gives 1.51 and 1.48, so the rules' strength, not late drying, sets the ratio.
- Uniform soil (single_pot, six seeds) against 0.23.0: root length +7% at day 45 and
  +9% at maturity, root mass +2% and +7%, fine roots +18% and +5%, total mass 0 to −4%,
  seed unchanged (7.94 → 7.90 g). Root length is past the 5% the spec asked for: the
  plant makes its own differences (dry and depleted cells around its roots),
  proliferation only ever adds roots (`r` is clamped at 0), and it moves root carbon
  into laterals, which carry more length per gram. The nitrate transport alone (rules
  off) stays within 1% of 0.23.0 in root length and total mass.
- Code review found the first diffusion fraction (4.6e-4) 11 times too slow: it
  multiplied by `θ` where the grid's exchange of amounts divides by it. Part of the
  first build's patch pass (1.49 over six seeds, +51% in validation) and of its
  fine-root share came from that: with nitrate held in place, it gathered in the rooted
  cells and stayed in the patch longer.
- Validation on four-seed means: **96 checks, 70 pass, every M0 gate passes** (0.23.0:
  69 of 94). Both new checks are Info misses at their band's edge:
  `nitrate_patch_proliferation` +48.8% (seeds +40 to +64%; band from +50%) and
  `split_pot_foraging` 1.495 (seeds 1.32 to 1.59; band from 1.5). Back in:
  `mean_seed_mass` 0.153 → 0.189 g and `harvest_index_tops` 0.397 → 0.435. Still out:
  `fine_root_share` 0.086 → 0.097 (band from 0.1). Out: `density_leaf_area_per_plant`
  −35% → −30% (band from −35%). Nearest M0 edge: shade total mass −46% → −49.1% (gate
  −50%); pods at maturity 11 (6 to 20), heat seeds −33%. The nitrogen checks (`ndfa_*`,
  `nfree_nodulation_shoot_mass`) move by under 2%.
- Garden-soil pots (20 mg N/kg, six seeds, rain shelter) are a little shorter of
  nitrogen now that nitrate no longer spreads at the water's rate: plant N −4% and total
  mass −4% at day 45; at maturity plant N unchanged, mass −3%, seed +9% (5.9 → 6.4 g).
  Two single-seed tests moved with it and were re-pointed, with notes: `thickening`
  (garden pot, day 56: hypocotyl 1.09 and epicotyl 1.39 times the tip, from 1.15 and
  1.5; bounds 1.05 and 1.35, from 1.1 and 1.4) and the bush topping test (cut on day 28,
  not 30: the untopped twin's last lower bud grows a few days sooner and has grown by
  day 34). Before the diffusion fix the garden pot was shorter of nitrogen (plant N
  −10% at day 45) and `ninety_days` set 9 pods at world seed 7; it sets 13 now, inside
  main's bounds again.
- Where water flows to roots whose nitrate uptake saturates (the 300 ppm pots), nitrate
  now gathers in the rooted cells: by day 45 the split pot's roots sit in about 2.3
  times the nitrate of its rootless cells (root-length weighted, seed 9; nitrogen card).
- Cost: the world computes each soil's mean once a day for all its plants (a plant run
  alone computes it itself; the values are the same). One container, best of three:
  single_pot 3.85 → 3.95 ms per sim-day, plot_30_beans (30 days) 12.5 → 11.9 ms; a pass
  per plant cost the plot about 2 ms.
- Open: wheat's values are the bean's; xerobranching and the dry-soil rule read relative
  available water, to become water potential with R3; a fine-root cohort still waits for
  its lateral to finish (R2a review).
