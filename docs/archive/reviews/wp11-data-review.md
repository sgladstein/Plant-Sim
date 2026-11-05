# WP11 reference data review (PR #33, merged at 32f1f47)

Reviewed 2026-11-05. Read in full: docs/validation-data.md, data/validation/{targets,responses,
parameters,sources}.ron, the curve file list. Then ran the four response scenarios for 90 days on the
WP10 branch and computed every target the way the schema says, to see where the sim stands today.

## Overall
A careful, honest data set: every value has a unit, a source key and a note saying how it was read;
unverified plan quotes are labelled; substitutions are explained; derived variables are defined
precisely enough to implement. The schema maps onto the WP10 CSV columns and the Stage and Role enum
names correctly (checked). Approve as the WP11 contract, with the changes below.

## Where the sim stands today (control run, WP10 branch, seed 7)

| Gated target | Range | Sim | |
| --- | --- | --- | --- |
| flowering_day | 28-57 | 34 | pass |
| maturity_day | 54-93 | 76 | pass |
| maturity_thermal_time_b10 | 700-950 | 680 | FAIL |
| total_dry_mass_at_maturity | 12-35 g | 29.2 | pass |
| pods_at_maturity | 6-20 | 16 | pass |
| max_height | 0.25-0.5 m | 0.388 | pass |
| peak_leaf_area | 0.07-0.35 m² | 0.17 | pass |

| Gated response | Rule | Sim | |
| --- | --- | --- | --- |
| drought_pods | -30% or more | -12.5% | FAIL |
| drought_pods_fall_more_than_seed_size | pods below seed size | -12.5% vs -4% | pass |
| drought_root_shoot | increase | 0.033 to 0.043 | pass |
| heat_pods, heat_seeds | decrease | -19%, -20% | pass |
| shade_total_mass | -30 to -50% | -35% | pass |
| shade_internode | +20% or more | -22% | FAIL (wrong sign) |
| shade_sla at day 40 | +10 to +40% | 0% | FAIL (no response) |

Info targets: seeds per pod 3.75, mean seed 0.27 g, harvest index 0.49, SLA 305, NAR 5.8, main nodes 7,
leaves 20, branches 4, emergence day 11 all inside their ranges. Seed mass per plant 16.5 g is above
5-15. Flowering thermal time is low against Cavalcante (271 vs 430 b10), CROPTIME (461 vs 547-763
b4.4) and Pinzón (328 vs 544-680 b8.3) and inside Dapaah (373 vs 338-440 b7). Heat seed mass -21%
against -50 to -71%. Shade: height -6% (cited +99%), pods +12.5% (cited -35%), root mass -83% (cited
-30%). Root to shoot ratio is 0.03 in every run.

## Findings on the data

1. **maturity_thermal_time_b10 should not gate M0.** It rests on one source (Cavalcante, figure
   labels, one site). Converted to base 10, the set's other two maturity sources sit well below its
   range: Pinzón about 805 b8.3 over 75 d is about 680 b10, Dapaah about 853 b7 over 110 to 130 d is
   about 500 to 530 b10. The sim passes Pinzón (809 vs 723-889 b8.3) and Dapaah (908 vs 760-950 b7)
   and fails only this gate. Gate maturity_day (already gated) and make this Info, or gate a range
   that spans the three sources (about 500-950).
2. **Flowering thermal time really is low**, as the data file's findings say: two of four sources fail
   and the calendar day passes only because the May preset is cool. This is the one phenology
   change WP12 should make (flowering towards 450-500 b8 from germination, maturity kept).
3. **drought_root_shoot at Final** divides by living shoot mass after the leaves have dropped, so the
   ratio is dominated by stems plus pods and differs between treatments for the wrong reason. Read it
   at Stage(SeedFill) or Day(60).
4. **heat_pods gated as Decrease while the note says the literature is split** (Vargas +54% pods
   with hot nights). The plan's exit rule is "pod set falls", which the seed checks capture better.
   Keep heat_seeds gated, make heat_pods Info.
5. **Add a root to shoot range target.** Pinzón 2024 (already in the set) gives root 2.4 g of 13.6 g
   total at R8, about 0.21 root to shoot; bean literature sits around 0.1-0.25. The sim is at 0.03 and
   no target catches it. An Info target now, gated in WP12, will drive the sink work.
6. **Base-8 conversions in parameters.ron** apply the sowing-to-germination offset to Pinzón (805
   b8.3 becomes "about 790") but not to Cavalcante (827 b10 over 75 d becomes "about 975", which is
   977 without the offset). State the day count used and apply the offset the same way to all three.
7. **Small things.** Source key `rezene2012` cites a 2013 paper. `seed_mass_per_plant` 5-15 g is
   narrow for a lone full-sun pot plant (the sim gives 16.5 with 16 pods, inside the pod and seed-size
   ranges); widen to 5-20 or note it. Kumar 2025 is used in five gated targets; each has a verified
   second source, so fine as documented.

## Findings on the model (for WP12, from these runs; not the data thread's to fix)

- **The drought scenario is too mild.** Watering stops at day 33 but preset rain continues into a
  24 cm pot, so pods fall only 12.5%. The scenario format needs a rain shelter: a `rain_fraction` per
  soil (or a `SetRainShelter` event) so the drought run keeps the radiation and loses the rain.
- **Shade cloth shortens internodes instead of lengthening them.** The shade-avoidance trigger is
  keyed to the leaf's light signal from neighbouring leaves (red:far-red proxy), which a uniform cloth
  does not change, so the only effect left is less carbon. Lopes 1987's +99% height was under shade
  cloth. The internode target needs an irradiance term (low absorbed PAR lengthens internodes) as
  well as the R:FR term. Hanyu 1996 confirms node number stays fixed, which the sim already does.
- **SLA has no response.** SLA is the constant 305 everywhere; plan §3.4 asks for up to 1.3× under
  shade and Cossu 2021 measured +15 to +32%. Needs a leaf-level rule (SLA from the light the leaf's
  parent node saw while the leaf expanded).
- **Root to shoot 0.03** and shade cutting roots sixfold: the sink limitation already on the WP12 list.
- **Heat response**: the pod-set temperature factor uses the daily mean; a 32/20 wave sits on its
  plateau (see the WP10 review).
- **Nothing varies the architecture**: leaves, branches and main nodes are 20, 4, 7 in all four runs.
  Expected for M0 (counts come from phenology) but it means shade_main_nodes passes trivially.

## Verdict
Approve the data set as the WP11 contract. Change items 1, 3 and 4 before the harness gates on them,
add item 5, and carry the model findings into the WP12 list. The set already shows its worth: three of
the seven M0 exit response rules fail today for identifiable model reasons.

## Follow-up (PR #36)
All six data items applied by the data thread: maturity_thermal_time_b10 and heat_pods are Info,
drought root:shoot read at Stage(SeedFill), a root_shoot_seed_fill Info target (0.08-0.30) added,
seed_mass_per_plant widened to 5-20 g, base-8 conversions with uniform germination offset and stated
day counts, Rezene dated 2013. Gated set is now 6 range targets and 7 responses.
