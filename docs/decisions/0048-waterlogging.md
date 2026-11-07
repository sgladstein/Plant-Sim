# 0048: Waterlogged soil runs short of oxygen (R5b)

- **Date:** 2026-11-07
- **Status:** accepted (the realism plan's R5b, `docs/research/realism-plan/r5-soil-extras.md`
  §4; the owner-approved queue: soil extras, then branches on branches, then nitrogen
  along the stem)

## Context
R5a (decision 0046) made soil water drain slowly: a soaked loam pot with holes rests
near saturation at its base, and rain on an outdoor pot stays until the plant uses it.
Nothing in the plant read how wet the soil was beyond its water potential, so
overwatering cost nothing. The spec asks for air-filled porosity per cell, hypoxic
cells where roots stop, lose conductance and nitrogen uptake and die after hours,
recovery once the soil drains, and denitrification, with the done-when that a pot
without holes soaked and kept wet for 4 days loses most of its fine roots, wilts at
midday while its soil is wet and recovers slowly, while the same plant in a pot with
holes does not.

## Decision
1. **Oxygen deficit, graded** (`ps_soil`; soil-and-water card equation 12). A cell's
   air-filled porosity is `a = θ_s − θ` and its oxygen deficit
   `d = clamp((0.10 − a) / (0.10 − 0.02), 0, 1)`: none at the classic 10% for root
   function (Grable and Siemer 1968, the spec's `a_crit`), full at 2%, CROPGRO's
   minimum pore space for roots (`PORMIN` 0.02 in its dry bean and soybean species
   files). Its hypoxic hours `H` add `d` each hour and return to 0 in an hour with
   `d = 0`; once `H` reaches 6 the cell is hypoxic, with hypoxia `h = d`. The spec's
   on/off threshold at 10% was the first build: a rained-on pot of garden loam sat at
   4 to 10% air for 11 days (seed 9 of the validation pot, after 36 mm of rain), and
   every root in it stopped for the whole time, so `root_length_flowering` and
   `fine_root_share` fell out of their bands (55 m, 0.082). Soil at 6 to 10% air is
   short of oxygen, not without it.
2. **Roots** (herb-architecture card): a tip's clock runs at `1 − h`; no lateral or
   fine-root cohort forms while its parent's cell has `h ≥ 0.5`; radial conductance
   falls to `1 − h (1 − 0.3)` (anoxia closes root aquaporins, Tournaire-Roux et al.
   2003; the plant wilts in wet soil, as bean leaves do within hours of waterlogging,
   Haverroth et al. 2024, Schildwacht 1989) and nitrogen uptake to `1 − h (1 − 0.2)`.
   A root organ dies when more than half its length lies in cells whose `H` has reached
   its death time: fine-root cohorts at 48 h, lateral cohorts and an axis's tip
   segment at 120 h (wheat 72 and 180 h: it tolerates waterlogging better). Only the
   tip of an axis dies: when every segment could die, a soaked sealed pot lost its
   whole root system on the fifth day and never recovered. Species schema 19 adds
   `hypoxia_radial_factor`, `hypoxia_n_uptake_factor`, `hypoxia_fine_root_death` and
   `hypoxia_root_death`; a species without them takes no notice of air.
3. **Recovery**: a lateral whose fine roots died grows new ones once its cell has
   `h < 0.5`, and a plant that lost an axis tip grows a new set of its hypocotyl-borne
   roots (wheat: crown roots) once the soil at its collar has `h < 0.5`. These are the
   adventitious roots beans make under flooding (Soltani et al. 2018 score them).
   Dead laterals are not replaced.
4. **Denitrification** (nitrogen card): in a hypoxic cell
   `ΔNO₃ = −0.02 h⁻¹ · f_T · h · NO₃`, a new ledger line (`denitrified`, nitrate lost to
   the air) and a daily column `soil_denitrified_n_total_g`; nitrification is scaled by
   `1 − h`. The ledger closes with the new line.
5. **Standing water evaporates first**, at the soil's full potential rate (R5a's open
   item). Without it a sealed pot under its canopy kept 25 mm of standing water for
   weeks.
6. **`SetDrainage { soil, drainage }`**, a new action: a pot's holes can be sealed,
   opened or stood in a saucer from any hour, as a waterlogging experiment drains its
   pots. Pots only; the game counts it as its own action type, which no challenge
   allows.
7. **Scenarios and checks.** `m2/waterlogged_pot` (holes sealed, rain shelter, watered
   to field capacity until day 45, soaked with 60 mm that morning and topped up 10 mm a
   day to day 48, holes opened on day 49) and `m2/waterlogged_pot_control` (holes,
   watered to field capacity throughout). Validation adds `waterlogging_root_loss`
   (fine-root length on day 49 against the control, at least −50%). The done-when test
   is `ps-world/tests/waterlogging.rs`.
8. **The simplified water network** (decision 0045) makes a new summary within the
   day once any root cell's hypoxia has moved by more than 0.05 from the summary's
   (plant-hydraulics card, equation 12). Soil that drains, or runs short of air, within
   hours otherwise reads as it stood at midnight: in the fidelity report the rained
   validation pot (seed 9) fell to −1.37 MPa at the simplified level against −0.98
   detailed on the day its top drained, and the four-seed lowest leaf potential missed
   its band (−1.006 against −0.910, allowed 0.065). Pots that never waterlog make no
   extra summary.
9. **The response pots stand under a rain shelter** (`rain_fraction: 0.0`), as
   the drought pot already did: control, shade_50, heat_at_flowering and frost_night.
   Rain on garden loam in a pot now waterlogs it for days, so one wet season among the
   four seeds moved the shade and heat responses by more than their treatments (below).
   A pot experiment controls its watering; the field checks keep their rain.
10. Model 0.29.0, species schema 19. Goldens regenerated.

## Consequences
- **The done-when** (world seed 7): fine roots 5.9 → 0.0 m from day 44 to 49 (control
  5.9 → 12.8 m); lowest midday leaf ψ −1.22 MPa against the control's −0.80 over days
  45 to 49, with FTSW 1, below the bean's −0.9 MPa turgor loss point. Laterals and
  axis tips die on day 50, after the holes are opened on day 49, and the leaves fall to
  −5.7 MPa on day 52; new fine roots from day 50 and new hypocotyl roots from day 51;
  9.6 m of root on day 74 against the control's 211 m. Read literally, the done-when's
  "the same plant in a pot with holes" fails in garden loam: soaked the same way, a pot
  with holes also loses its fine roots by day 47, as its base stays saturated (R5a's
  seepage face), and its top regains 10% air only on day 54. It keeps more root (26 m
  on day 50) and grows fine roots again from day 49. The control here is a pot with
  holes watered as a gardener does.
- **Rain on a pot.** The validation pot stands in the rain. On world seed 9, 36 mm on
  day 19 and 37 mm on day 35 (149 mm over the season) keep its loam at 4 to 10% air
  for days at a time: it denitrifies 2.9 of its 4.2 g of mineral nitrogen by day 45
  (about 0.05 d⁻¹ over days 19 to 45, at the spec's `k_dn` of 0.02 h⁻¹ under full
  hypoxia, a guess no source has checked yet), and its roots reach 187 m against
  222 m. It flowers as at 0.28.0 (54 flowers), but the pod-set draws are keyed by organ
  id and hypoxia changed its root organs, so the draws changed: 5 pods at 0.28.0
  became 17, and 4.0 g of seed became 10.2.
  Seeds 7, 8 and 10 never waterlog and are identical to 0.28.0. The four-seed means
  move with seed 9: pods 9.25 → 12.25, seed 6.8 → 8.3 g, total mass 26.1 → 29.3 g
  (0047 found the same spread of luck over eight seeds).
- **Plots** drain freely: after 36 mm of rain the 30-bean plot has hypoxic cells for a
  few hours only.
- **Validation** (four seeds): 97 checks, 74 pass, every M0 gate passes; the new
  `waterlogging_root_loss` passes (−100%). Under the rain shelter (item 9) every pot
  response (drought, shade, heat, frost) is the same at 0.28.0 and 0.29.0, since no
  response pot waterlogs; the pole rows, in the field, move pole_unstaked_seed_yield
  −64 → −59%.
  The shelter alone, at 0.28.0, moves shade_total_mass −37 → −40%, shade_pods −37 →
  −45%, heat_seeds −34 → −26% and heat_seed_size −2 → −8% (now passes: 73 of 96 at
  0.28.0). Without the shelter the graded model passed every M0 gate (72 of 96), with
  shade_total_mass −48% and shade_internode +23% near their gates' edges (−50%,
  +20%); the first graded build, before pond evaporation, failed both (−54%, +10%).
  The on/off first build (item 1) failed root_length_flowering (55 m) and
  fine_root_share (0.082).
- **Fidelity** (`ps-cli fidelity`): 111 of 111 outcomes pass, seed-yield bias −2.57% ±
  1.65% (0.28.0: −0.65% ± 2.57%). Without item 8, 109: the validation pot's lowest leaf
  potential at the simplified level was −1.006 against −0.910 MPa detailed; with it,
  −0.909. The largest gaps left are the validation pot's seed (−8.3%, pod-set draws on
  seed 9) and the drought pot's lowest leaf potential (+10.1%, as at 0.27.0).
- **Challenges** (51 strategies and the 70-plan thirsty-summer window sweep, 8-season
  means, docs/game/challenges.md): the heat-wave and hungry-soil pots stand in the
  rain, and in one season of the eight (world seed 9) heavy rain leaves their loam
  short of air for days. The hungry-soil fixture's seed in that season falls from 8.7
  to 5.5 g while the other seven are unchanged; every hungry-soil plan loses 0.1 to
  0.6 g on the mean (best 8.70, was 9.04), so its gold comes down from 8.75 to 8.4
  (the top five plans 8.4 to 8.7). Heat-wave plans move by up to 0.75 pods either way
  (day 24 still best at 11.75), medals unchanged. Thirsty summer, under its rain
  shelter, holds its window ranges (the fixture 10.0, best 11.0); single plans move by
  up to 0.6 pods, which its plans already did on far smaller changes (0046).
- **Cost** (this container, against main built here, best of 6 to 15 runs): single_pot
  7.14 → 7.10 ms per sim-day, single_pot_simplified 6.43 → 6.58, plot_30_beans over
  30 days 18.9 → 19.7, all inside the container's run-to-run spread (7.1 to 8.9 ms for
  the same pot binary). The aeration pass is one loop over the cells an hour.
