# 0018: M0 tuning and lock (WP12)

- **Date:** 2026-11-05
- **Status:** accepted

## Context
The WP11 report showed four M0 misses with the reference data (PRs #33 and #36): maturity
thermal time, drought pods, shade internodes and shade SLA. It also showed four model
causes behind them:

1. Phenology ran fast in thermal time. The species file's flowering sum of 320 °C·day sat
   below every published sum.
2. A uniform shade cloth leaves red:far-red unchanged, so shade avoidance never started.
   Low light alone only took carbon away, which shortened internodes.
3. From flowering on, the plant is short of sinks: the sink feedback discards about 60%
   of potential gross assimilation, even at half light. So pod number, not filling, is
   where shade, drought and heat can act. Pod set read the node reserve with a reference
   (0.1 g C/g DM) that a fed plant always exceeds, so pod set ignored the carbon supply.
4. One plant per response scenario made each response a sample of one. Preset rain kept
   watering the drought pot.

## Decision
Model version 0.1.0 → 0.2.0. Golden hashes are regenerated. The Linux files are copied
to Windows, as hashes have been identical across platforms since WP10. Changes:

- **Phenology:** emergence 70 → 80, flowering 320 → 440 and maturity 820 → 900 °C·day
  from germination. These match the published sums (thermal-time-and-phenology card).
- **Low-light shade response:**
  - Each leaf keeps a running mean of the PAR it absorbs per day (mol m⁻² day⁻¹,
    weight 1/3 on yesterday).
  - A new organ's shade index is the larger of the red:far-red index and a low-light
    index: 0 above 30, 1 at 8 mol m⁻² day⁻¹ or below.
  - The index scales internode length (max 2.0, from Lopes 1987), SLA (max 1.4, from
    Cossu 2021) and pod set (down to 0.2).
  - A shaded internode is also thinner: its specific length rises by the same factor,
    so its potential mass stays the same (the first draft squared the factor, which
    halved the mass; the review caught it). A carbon-short internode is then still
    longer per gram, as etiolated stems are.
- **Pod set:**
  - A separate reserve reference, `pod_set_reserve_reference` = 0.22 g C/g DM. Pod set
    is proportional to the node reserve below it.
  - `drought_abortion_ftsw` 0.3 → 0.5, the CROPGRO SWFAC level.
  - Temperature is read at the day's peak (Tmax), not the mean. It counts over the 7 days
    of pollen development before the flower opens, as well as while it is open. The
    flower is as fertile as the worse of the two windows: `H = min(mean before,
    mean open)`. A product of the two would count an ordinary warm day twice (review).
  - Heat lowers the share of ovules that set.
  - `pod_set_base` 0.45 → 0.42.
- **Frost:** a killing frost stops every apex and kills buds and flowers, so the plant
  does not regrow (§7 M0 exit). Tested by `a_frost_night_kills_the_leaves_for_good`
  with the new `response/frost_night` scenario.
- **Scenarios:**
  - The four response scenarios have 8 replicate pots 3 m apart, and the harness
    averages over them.
  - Drought and heat start on the control's new flowering day, day 43 (tick 1032).
  - The drought pots stand under a rain shelter, using the new per-soil scenario field
    `rain_fraction`, so the drought is a drought.
- **Validation data:** the gated `biomass_logistic` target is added (§7 exit, logistic
  R² ≥ 0.95). `validation/known_misses.ron` is empty.

The values were chosen against four weather years (world seeds 7, 11, 23 and 31), not the
committed seed alone. That follows the WP11 review's caution that one seed is not a
robust pass.

## Consequences
- **All 14 M0 checks pass in every one of the four weather years:**

  | Check | Gate | Seed 7 (committed) | Four seeds |
  | --- | --- | --- | --- |
  | Flowering | 28 to 57 days | day 44 | 43 to 46 |
  | Maturity | 54 to 93 days | day 83 | 80 to 83 |
  | Dry mass | 12 to 35 g | 22.9 g | 22.9 to 26.1 g |
  | Pods | 6 to 20 | 10 | 10 to 13 |
  | Height | 0.25 to 0.5 m | 0.39 m | 0.39 to 0.41 m |
  | Peak leaf area | 0.07 to 0.35 m² | 0.22 m² | 0.22 to 0.24 m² |
  | Biomass logistic R² | 0.95 or more | 0.994 | 0.994 to 0.998 |
  | Shade: internodes | +20% or more | +28% | +28 to +34% |
  | Shade: SLA | +10 to +40% | +20% | +19 to +20% |
  | Shade: total mass | −30 to −50% | −42% | −37 to −49% |
  | Drought: pods | −30% or more | −55% | −53 to −64% |
  | Drought: seed size | down less than pods | yes | |
  | Drought: root:shoot | up | +23% | +16 to +39% |
  | Heat: seeds | down | −8% | −8 to −31% |

- **Calibrated or predicted.** Phenology, `shade_internode_factor` and
  `shade_sla_factor` were set from the same papers that gate them, so those checks show
  consistency, not prediction. The other gates are emergent: shade total mass, drought
  pods and root:shoot, heat seeds, the logistic shape, and the §8.2 size ranges.
- Pods per plant fell from 16 (WP9, seed 7) to 10. The ps-arch 90-day bound was
  loosened from 10..=20 to 6..=20 to match §8.2.

- **Known limits at the M0 exit (Info, not gated):**
  - Root:shoot is 0.04 at seed fill, against 0.08 to 0.30. There is no secondary
    thickening, and laterals are short and unbranched. So drought's +23% root:shoot
    passes on a base 2 to 8 times too low.
  - The reproductive phase is sink-limited in every scenario, so shade mass falls through
    pod number. Leaves keep their capacity through seed fill, since there is no nitrogen
    remobilization yet (M2).
  - Heat cuts seed set but hardly seed size: seed mass −10% against −50 to −71%. The
    CROPGRO curve puts a 32 °C heat wave (F_T 0.6) close to an ordinary 28 to 30 °C
    day (0.8 to 0.93), so the heat response is weak and varies by weather year.
  - Drought does not shrink seeds (+5% against −18%).
  - Shade height is +33% against Lopes's +99%.
  - Light per leaf: see the re-check below.
  - The thermal-time papers disagree among themselves. The sim passes Cavalcante 2020
    and CROPTIME 2020 and sits between Dapaah 1999 and Pinzón-Sandoval 2024, which are
    reported as misses.
- **Light re-check on a sim plant (decision 0008).** The ray tracer fixture now also
  has a 60-day sim plant (74 leaflets, `plant60_*`), exported by the ps-arch test
  `export_leaves_for_ray_tracer`. The synthetic WP6 shoot now gives 56 of 57 leaves
  within 0.10 and a total of +1.7%. The sim plant does worse:

  | G | Plant total | Leaflets within 0.10 | Worst |
  | --- | --- | --- | --- |
  | 0.5 (species file) | +9.9% | 49 of 74 | 0.32 |
  | 0.65 | −1.9% | 60 of 74 | 0.23 |

  The sim draws every blade tilted about 19° from horizontal. But G = 0.5 assumes
  spherical leaf angles, so the light model shades the plant too little, by about 10%.
  The M0 bounds stay as the owner accepted them, on the synthetic shoot. The new test
  `sim_plant_against_ray_tracer` pins the sim-plant numbers so they cannot drift
  unnoticed. Fixing this is the owner's call, and it is a model change. The preferred
  fix (WP21, review) is G per direction from the drawn leaf normals (area-weighted
  |n·d|). The other is to draw a spread of leaf angles that matches G = 0.5. Either fix
  needs the M0 tuning re-run, since the calibration absorbed the 10%. The per-leaf
  error (a third of the leaflets off by more than 0.10 at G = 0.5, a fifth at 0.65) is
  partly the grid's limit within a cell, and is a known limit at the M0 exit.
- The validation run grows 33 plants for 90 days (about 20 s in release).
- The heat and drought scenarios start on the control's flowering day. A change to the
  phenology thresholds needs those two days moved with it.
