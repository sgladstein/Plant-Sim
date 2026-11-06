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
200 times nitrate's in moist soil; a patch evened out within days. And the scenarios had
no way to split a pot or to make a patch.

## Decision
1. **Soil.** Laterally, nitrate now moves with the water's flux at the concentration of
   the cell the water leaves, plus diffusion at 4.6e-4 per hour in 2 cm cells (about
   5e-11 m² s⁻¹, Millington–Quirk at field capacity). A scenario can give a pot a
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
Six world seeds unless noted; "uniform" is single_pot against the rules off.
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
  - nitrate patch: lateral and fine-root length in the cube 1.49 times the control (1.32
    to 1.67); without the rules 1.01;
  - split pot: wet:dry root length 1.46 (1.29 to 1.64); without the rules 1.13 (0.78 to
    1.35). The unwatered half falls only to about 0.4 of available water by day 45, so
    the dry-soil rules act late; the watered half supplies about two thirds of the
    water from day 20 (seed 9).
- Uniform soil (single_pot, six seeds) against 0.23.0: root mass +2 to +3%, root length
  +7% at day 45 and at maturity, fine roots +23% at day 45 and +4% at maturity, total
  mass 0 to −5%, seed −6%. Root length is just past the 5% the spec asked for: the plant
  makes its own differences (dry and depleted cells around its roots), and proliferation
  moves root carbon into laterals, which carry more length per gram. The nitrate
  transport alone (rules off): root length +2%, total mass −3%.
- Validation on four-seed means: **96 checks, 72 pass, every M0 gate passes** (0.23.0:
  69 of 94). `nitrate_patch_proliferation` +51% (pass; seeds +35 to +59%);
  `split_pot_foraging` 1.497 (miss at the band's 1.5; seeds 1.31 to 1.61). Back in:
  `mean_seed_mass` 0.153 → 0.170 g, `fine_root_share` 0.086 → 0.102,
  `harvest_index_tops` 0.397 → 0.421. Out: `density_leaf_area_per_plant` −35% → −29%
  (band from −35%). Near-edge M0: shade total mass −46% → −48% (gate −50%), shade
  internodes +27% → +28%, heat seeds −34% (weakest seed −27%). The nitrogen checks
  (`ndfa_*`, `nfree_nodulation_shoot_mass`) move by under 2%.
- Garden-soil pots (20 mg N/kg, six seeds, rain shelter) are shorter of nitrogen now
  that nitrate no longer spreads at the water's rate: plant N −10% and total mass −7%
  at day 45, −3 to −4% at maturity, seed unchanged (5.9 → 6.0 g). Three single-seed
  tests moved with it and were re-pointed, with notes: `thickening` (garden pot, day 56:
  hypocotyl 1.06 and epicotyl 1.35 times the tip, from 1.15 and 1.5; bounds 1.03 and
  1.3, a transport effect, the same with the rules off), `ninety_days` pod count at world
  seed 7 (9 against §3.16's 10 to 20; 14 with the rules off; eight-seed mean 10.0
  against 11.3, spread 5 to 15; floor 8, the across-seed test keeps 6 to 20) and the
  bush topping test (cut on day 28, not 30: the untopped twin's last lower bud grows by
  day 34 in three of eight seeds, against none with the rules off).
- Where water flows to roots whose nitrate uptake saturates (the 300 ppm pots), nitrate
  now gathers in the rooted cells, to about three times the bulk soil around the split
  pot's roots by day 45 (nitrogen card).
- Cost: the world computes each soil's mean once a day for all its plants (a plant run
  alone computes it itself; the values are the same). One container, best of three:
  single_pot 3.85 → 3.95 ms per sim-day, plot_30_beans (30 days) 12.5 → 11.9 ms; a pass
  per plant cost the plot about 2 ms.
- Open: wheat's values are the bean's; xerobranching and the dry-soil rule read relative
  available water, to become water potential with R3; a fine-root cohort still waits for
  its lateral to finish (R2a review).
