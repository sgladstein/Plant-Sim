# 0042: Seed growth ends at physiological maturity (R1d)

- **Date:** 2026-11-06
- **Status:** accepted (a correction inside the owner-approved realism plan; Code
  review's first R1b follow-up)

## Context
Physiological maturity (965 Cd from germination in the bush bean, 1232 in the pole
bean) ended phenology but not growth. A pod's seeds start 74 Cd after the pod and fill
for `seed_fill` (350 Cd), so a pod set late kept filling after maturity, drawing on the
reserves the plant had left (no organ respires after maturity, decision 0020). In the
bush pot (single_pot, world seed 7, model 0.22.0) pods set from day 50 to 66 after
sowing, the first seed grows from day 57 and maturity comes at day 82; 36% of the
final seed mass (5.86 g at maturity, 9.22 g at the end) was added after maturity. The
pole bean added 7 to 11%, wheat none.

In CROPGRO-Drybean seed growth stops at R7 (physiological maturity), and a pod cohort
fills for SFDUR photothermal days, shorter than R5 to R7 (SD-PM), so the last cohorts are
cut short (BNGRO048.CUL; `docs/archive/research/source-files/dssat-bngro048`). The
determinate cultivars have SFDUR 14 to 19 (generic Andean Habit 1: 15; Seafarer 17;
Redkloud 18) against SD-PM 17 to 22. Our 350 Cd is 18 photothermal days at 19.25 Cd
each (decision 0032), chosen from the 15 to 22 of every habit.

## Decision
1. **Nothing grows from maturity.** Hourly step 12 skips every organ once the plant is
   Mature (`HourInputs::growing`, set by the architecture from the phenology stage):
   no primary or secondary growth, no store release or refill. Late pods keep the seed
   mass they have; the reserves stay where they are. Starvation and maintenance were
   already off after maturity (decision 0020).
2. **Bush seed fill 289 Cd.** `seed_fill` for the bush bean becomes CROPGRO's SFDUR 15
   photothermal days of the generic determinate bush (Andean Habit 1) at 19.25 Cd per
   photothermal day, so early pods finish well before maturity, as in CROPGRO. The pole
   bean keeps 350 Cd (18 photothermal days, below the climbing generic's 22): its
   maturity comes later and stopping at maturity was enough.
3. No change to the §3.2 step list.

### Tried and not kept
- **Stop at maturity, keep 350 Cd.** Every seed is cut short: mean seed mass 0.140 g
  (band from 0.16), seed per plant 6.65 g (four validation seeds).
- **327 Cd** (17 photothermal days, Seafarer): harvest index of the tops 0.388, mean
  seed 0.148 g, seed per plant 6.96 g. Neither brings mean seed mass back into its band,
  and 289 Cd is the generic determinate bush's own value.

## Consequences
- Model 0.23.0 (species schema unchanged at 15); every golden hash moves.
- Seed mass at the end now equals seed mass at maturity (`ninety_days` asserts it on
  every seed). Eight world seeds in the pot: seed per plant at maturity 6.89 → 7.84 g
  (the same seed fills in fewer days, so more of it is in by maturity), at the end
  9.60 → 7.84 g; pods 11.9 → 11.8. Six seeds of the 30-bean plot: 3.66 → 3.89 g per
  plant at maturity, 4.40 → 3.89 g at the end.
- Validation on four-seed means: **94 checks, 69 pass, every M0 gate passes** (0.22.0:
  70). `mean_seed_mass` slips out (0.202 → 0.153 g, band 0.16 to 0.6; the four
  validation seeds range 0.10 to 0.18 g); `seed_mass_per_plant` 9.52 → 7.26 g (pass);
  `harvest_index_tops` 0.374 → 0.397 (band from 0.40); `feedback_loss_fill` 0.430 →
  0.407 (band to 0.35). Heat: seeds −32% → −34% (M0 pass; weakest seed −28%), seed
  mass −25% → −28% (band −50 to −71%), pods −21% → −22%. Shade M0 gates: total mass
  −46%, internodes +27%, pods −46%.
- Open, not caused here: under drought and heat the mean seed grows larger rather than
  smaller (`drought_seed_size` +19%, `heat_seed_size` +11%), likely because fewer pods
  share the same reserves; the bush's mean seed sits at its band's edge; pods set over
  16 days (day 50 to 66 in the pot), which sets how many finish before maturity.
