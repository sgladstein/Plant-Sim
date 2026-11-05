# Re-tune package: research items 1 to 4

Written 2026-11-05 by the research thread for the implementation thread's re-tune of the "~"
parameters (the model bump after the pole bean stake-catching fix). Values checked against
`main` at 8b66765 (model 0.11.0). Tags: **[F]** source read this session, **[K, to verify]**
from knowledge, **[I]** inference or arithmetic. Item numbers follow
`docs/handoff/research.md` "Findings not yet acted on".

## Summary

| # | Parameter (bean RON) | Now | Recommended | Change? | Confidence |
| --- | --- | --- | --- | --- | --- |
| 1 | `vcmax25`, `jmax25` | 89.3, 159.8 µmol m⁻² s⁻¹ | **keep** (Ci-basis equivalent about 87 and 160) | No: round 1's "20% high" was wrong | Medium |
| 2 | `t_base`/`t_opt`/`t_max` | 8 / 28 / 38 °C, single peak | trapezoid **7 / 25 / 33 / 42 °C**; targets restated so the default scenario keeps its calendar days | Yes, needs a code change (a second optimum) | Medium |
| 3 | `construction_cost` | 1.25 g C/g C, all organs | **leaf 1.39, stem 1.29, root 1.25, seed and pod 1.36** (nodule: root value) | Yes, needs a per-organ field | Medium |
| 4 | `medlyn_g1` | 4.0 kPa^0.5 | **5.79** (wheat already has 5.8) | Yes, data only | Medium |

Items 3 and 4 both lower growth per unit of carbon fixed or raise water use, so tune the seed
means after applying all of them together, not one at a time.

## 1. Vcmax and Jmax basis: keep the values

Round 1 said Pimentel et al. 2011's values are chloroplast (Cc) based with gm 0.47 mol m⁻² s⁻¹
bar⁻¹, so with infinite gm Rubisco-limited A would run about 20% high. Redoing the arithmetic
shows that only holds if the fit used the Ci-basis kinetic constants. A Cc-basis fit (the
Sharkey et al. 2007 fitting utility, the usual way gm and Vcmax were fitted together in 2011)
uses the Bernacchi 2002 Cc constants (Kc 272.38 µmol mol⁻¹, Ko 165.82 mmol mol⁻¹, Γ* 37.43 at
25 °C) [K, to verify]. Their lower Kc and Γ* almost exactly cancel the CO2 drawdown across gm.

Least-squares fit of the sim's Ci-basis model (Bernacchi 2001 constants, as in `gas.rs`) to
A/Ci curves generated with gm 0.47 [I]:

| Pimentel fit assumed | Apparent Vcmax (Ci 80 to 300) | Apparent J (Ci 300 to 800) | Sim's Ac error at Ci 250 |
| --- | --- | --- | --- |
| Cc constants (Bernacchi 2002), likely | 87.3 (−2%) | 159.6 (0%) | +3% |
| Ci constants (Bernacchi 2001) on Cc | 77.2 (−14%) | 154.8 (−3%) | about +16% |

Recommendation: keep 89.3 and 159.8 and drop this item. Pimentel 2011 measured cv. Carioca at
25 °C in 8 L greenhouse pots (`docs/archive/research/notes/validation_whole_plant_traits.md`
§4, [F]). Its Table 1 (an image) was never read, so the constants it used are unconfirmed. A
cheap check: the paper reports A ≈ 20 µmol m⁻² s⁻¹ at Ca 380 and PPFD 700, A ≈ 25 at saturating
Ci, and gs max ≈ 0.4 mol m⁻² s⁻¹ [F]. If the sim's leaf at 25 °C, VPD 1.5 kPa, PAR 700 gives
A within 15% of 20, keep the values.

## 2. Development temperatures: trapezoid 7 / 25 / 33 / 42 °C

Sources [F, in `docs/archive/research/notes/crop_models_bean.md`]:
- CROPGRO-Drybean BNGRO048.SPE lines 97 to 99: vegetative 5/27/37/45, early reproductive
  5/24/34/44, late reproductive 6/25/37/60 °C, linear with a plateau. Re-read this session
  from `docs/archive/research/source-files/dssat-bngro048/BNGRO048.SPE` [F].
- Ferreira et al. 1997, fitted with hourly data: 6.8 / 24.0 / 34.1 °C.
- APSIM bean: 7.5 / 30 / 40. Dapaah et al. 1999: base 7.

7 / 25 / 33 / 42 is a synthesis [I], between CROPGRO's plateau and Ferreira's field fit.
Today's single peak halves the development rate at 33 °C. The trapezoid keeps it at maximum
from 25 to 33 °C, which is what all three crop models do. Heat damage stays a separate process
on pod set (already in the RON as `pod_set_t_*`).

How to restate targets [I]: decision 0018 tuned `emergence` 80, `flowering` 440, `maturity` 900
Cd and the pod and seed sums on base 8. Don't convert with a fixed factor. Run the default
scenario once with each function, record the hour each stage is reached under the old one, and
set each new target to the base-7 sum at that hour. Validation then passes on day of year
unchanged, and only hot spells behave differently. `vegetative_after_flowering` is "13
photothermal days × (t_opt − t_base)", so on the new function it becomes 13 × (25 − 7) = 234 Cd.
Normalise the plateau to To1 − Tb, as §11.4 says. Pole bean takes the same function. Wheat is
unchanged (base 0, APSIM).

Code: `thermal.rs` has one optimum. Add `t_opt_high` (bean 33) and keep the old shape when
`t_opt_high == t_opt`, so the wheat file needs only one new line.

## 3. Construction cost by organ

CROPGRO computes the glucose needed per gram of new tissue from its composition. Re-read from
the DSSAT source this session [F]: `INCOMP.for` sets
`AGRLF = PLIPLF·RLIP + PLIGLF·RLIG + POALF·ROA + PMINLF·RMIN + PCARLF·RCH2O`, and `DEMAND.for`
adds protein at `PROLFI·RPROAV`, where `RPROAV` is the N-source-weighted cost (RNO3C 2.556 for
nitrate, RFIXN 2.830 for fixed N; `PLANT.for` lines 1229 to 1234). Costs and compositions are
from BNGRO048.SPE lines 19 to 30 [F].

Converted to the sim's unit, g C used per g C built, with carbon fractions carbohydrate 0.444,
protein 0.53, lipid 0.773, lignin 0.669, organic acid 0.375 and mineral 0 [K, to verify], and
glucose 0.40 g C/g [I]:

| Organ | g CH2O / g DM | C fraction of DM | **g C / g C** | Round 1 said |
| --- | --- | --- | --- | --- |
| Leaf (protein 0.24) | 1.54 | 0.443 | **1.39** | 1.36 |
| Stem (0.15) | 1.47 | 0.455 | **1.29** | 1.31 |
| Root (0.10) | 1.39 | 0.446 | **1.25** | 1.24 |
| Seed (0.235) | 1.56 | 0.456 | **1.36** | 1.40 |

The two sets agree to within 3%. Use the new column, since this one shows its arithmetic. Pods
(shell) can take the seed value. Nodules can take the root value until fixation costs are
revisited. With fixed N instead of nitrate, protein costs about 10% more glucose, about +0.03 on
leaf and seed [I]: not worth modelling now.

Effect [I]: weighted by a vegetative bean's growth (about 50% leaf, 30% stem, 20% root), the mean
cost goes from 1.25 to about 1.33, so about 6% less structure per unit of carbon. Expect lower
seed-mean biomass, so retune together with item 4.

## 4. Medlyn g1 = 5.79

CLM5's PFT table gives g1 = 5.79 kPa^0.5 for spring wheat, temperate soybean, tropical soybean,
cotton and rice (from Lin et al. 2015). C3 grass has 5.25 and C4 crops 1.79 [F: CTSM
`doc/source/tech_note/Photosynthesis/CLM50_Tech_Note_Photosynthesis.rst`, release-clm5.0, read
this session]. No bean-specific g1 was found. Soybean is the closest legume.

Set bean and pole bean `medlyn_g1` to 5.79 and the wheat source to the CLM5 table. Effect [I]: at
D = 1.5 kPa and equal A, gs rises by (1 + 5.79/√1.5)/(1 + 4/√1.5) = 1.34. Ci rises too, so A rises
by a few percent and transpiration by about a third. Soil drawdown and the onset of stress come
earlier. WP27's energy balance then cools leaves more, so after WP27 lands re-check the
water-use checks in `data/validation/targets.ron`.
