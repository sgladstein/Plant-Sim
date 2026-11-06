# 0037: Root length and uptake (R2a, issue #116)

- **Date:** 2026-11-05
- **Status:** accepted (owner approved the realism plan's R1 to R3 on 2026-11-05; the
  coordinator moved R2a with the water fix ahead of R1b at 21:20Z)

## Context
Issue #116: in `single_pot` the roots met about 53% of the leaves' water demand over the
season, and only 16 to 34% on days 25 to 50, while stomata ignored the shortfall. Two
causes:

- **The hourly ceiling.** `u_max` was CERES's daily RWUMX (0.03 cm³ of water per cm of
  root per day) spread over 24 hours. Nearly all of a day's transpiration falls in about
  10 hours, so at midday the ceiling bound at the 24 h mean and the plant could never
  take more than about half its noon demand, whatever its root length.
- **Short roots.** Root length was 10 to 100 times below pot data early. An axis
  queued its next segment only at a daily step after the tip finished, so a root moved
  one segment per day at most, and lateral cohorts had no laterals of their own. The 25×
  nitrogen uptake capacity and the 4 cm reach (decision 0020) stood in for the missing
  length.

The realism plan's R2 spec (`docs/research/realism-plan/r2-root-system.md` §3) asked
for early vigor, second-order laterals as cohorts and uptake from real root length; the
research thread's R2 numbers (`docs/research/realism-plan/numbers/r2_roots.md`) set the
bands: root length 60 to 180 m at flowering in pots, fine-root share 0.1 to 0.4 by order.

## Decision
1. **Root segments queue behind the tip.** At each daily step an axis whose tip has
   started expanding spawns its next segment at the tip's planned end with its
   expansion clock at `−(D_tip − age_tip)`: it waits, then starts the moment the tip
   finishes, so the tip moves at the root type's elongation rate. The axis length counts
   finished segments at the length they reached, and segment starts and the planned
   end are restated from those lengths each day (`settle_axis`), a growing segment at
   the most it can still reach (its length plus the rest of its potential). The
   pot-wall rule tests a segment's reach with its 2% primordium, and a lateral cohort
   with no room for its length in the soil (at a pot's bottom edge) turns level
   toward the collar, or is not made. At most one segment waits per axis. (A first version kept the planned end from the potentials: roots
   that fell short were drawn centimeters above where the wall rule thought they were,
   so deep roots turned sideways in mid-pot.)
2. **Tip feeding.** A growing root organ draws on its own reserve and on its parent
   root's (or the seed's) reserve above the floor, at its priority, own first. A queued
   primordium holds almost no reserve, so without this the tip grew a few percent of its
   potential an hour; phloem unloads at the tip from the root behind it.
3. **Fine roots as cohorts** (R2 spec §3.2). Once a first-order lateral cohort finishes,
   it gets one second-order cohort of the new `fine_lateral` type,
   `m₂ = m₁ (ℓ₁ − a₁) / s₁`: realized length past the lateral's apical zone over the
   fine-root spacing. Bean fine laterals follow OpenSimRoot's carioca set: every 0.5 cm,
   none in the last 4 cm, 0.1 mm across (about 1,350 m/g), 0.12 cm/day to 0.5 cm, no
   thickening.
4. **`u_max` over the transpiring hours.** 3.0e-4 kg m⁻¹ h⁻¹: RWUMX over 10 hours, for
   all three species. This alone was the cause of #116.
5. **Stomata follow supply.** The soil returns each plant's hourly capacity
   (`Uptake::capacity`). The plant's supply factor is
   `g_{t+1} = clamp(g_t · met / demand, 0, 1)` (`met` the uptake when demand went
   unmet, else the capacity; 1 with no demand) and multiplies the FTSW stress factor on
   conductance: a one-hour lag, as CROPGRO's SWFAC is a one-day lag. Coupling stomata
   alone, before the roots grew, collapsed growth, so it came after 1 to 4.
6. **Root priority** reads `min(FTSW, g)`, so a plant whose roots cannot keep up puts
   more into roots.
7. **Nitrogen from each cell's own roots** at CROPGRO's RTNO3 = RTNH4 (6e-4 g N per m of
   root per day for each form). `root_reach` is removed (species schema 13) and the 25×
   factor with it.
8. **Wood is not resorbed** when an organ is shed (R1a review): only primary structure
   moves 30% to the parent.
9. **Wood yields first when carbon is short.** The thickening potential is scaled by
   the day's share of the expanding shoot's carbon demand that was met (leaves and
   internodes, `Plant::carbon_met`, read at daily step 4 from the organs' lifetime
   demand records). With realistic roots the seedling's root system
   built about 5 to 7 mg of root wood a day in 50% shade, as much as its primary root
   growth, while leaves got a third of their demand. Shade-grown plants have thinner
   stems and roots; cambial growth is the lowest-priority sink.
10. **Thickening potential once a day** (the Performance thread's lever; the owner
    approved computing thickening once a day on 2026-11-05: thickness does not change
    visibly within a day). A new daily step 5b, after root elongation, reads each
    living root segment's rate from the age table and the distance factor
    (`Plant::set_wood_rates`) and sets every stem's and root's potential per hour for
    the coming day (`Plant::set_wood_potential`), from the day's mean air temperature
    and its mean stress (water times nitrogen) times its `carbon_met`. Allocation
    still builds it each hour after primary growth, at priority 0.4, as in R1a.
    Building it once a day as well was tried and starved wood: one draw from the
    midnight reserve, even of all of it above the floor, gave a tap root of 1.9 to
    2.1 mm against 3.6 mm (band 2 to 6), since through the day transport refills a
    root's reserve that primary growth then spends. The hourly build costs about 5% on
    plot_30_beans. This is a step order change (design plan §3.2).
11. **Shade response recalibrated.** Realistic roots took the shade gates below their
    bands (four-seed means: shade internodes +6% against at least +20%, total mass −51%
    against −30 to −50%):
    - `shade_internode_factor` 2.0 → 2.6 (both beans). 2.0 was Lopes 1987's 62% light
      result read as the maximum, but the factor is the maximum at the full low-light
      index. Lopes's 30% light treatment (+159% height, leaf number unchanged) gives
      2.6; the 50% shade pots reach an index of about 0.4, so about 1.7×.
    - `shade_pod_set_factor` 0.2 → 0.5 (both beans). With lower node reserves in shade,
      pods fell 56% against Cossu 2021's 35%; 0.5 gives about −45%.
12. **Outputs and checks.** Daily rows gain `fine_root_length_m`; the derived
    `fine_root_share` is its share of root length. New Info targets
    `root_length_flowering` (single_pot at flowering, 60 to 180 m; Trindade 2014,
    Araújo 2004) and `fine_root_share` (0.1 to 0.4; Miller 2003, Jin 2013).

### Tried and not kept
- **Saturation in root length density** (`L/V ÷ (L/V + 0.5 cm cm⁻³)`, R2 spec §3.3): cut
  supply to about 72% of demand in the pot, since dense root cells near the crown dry
  out by the afternoon. Left for R3's root hydraulics.
- **Shoot-first root growth** (root potential times `carbon_met`, OpenSimRoot's
  allocation order and Poorter et al. 2012's lower root share in shade): it cut root
  length 20 to 40% in shade, and the lost nitrogen and water uptake cost the shoot more
  than the carbon helped it; with a squared factor wheat height also failed.
- **A lower root priority in shade** and **less tip feeding**: worse on both gates,
  for the same reason.
- **A stronger low-light index** (`shade_dli_full` 8 → 12 or 16): longer internodes,
  but the same index also raises SLA and cuts pod set, so shade mass fell to −57%.
- **Longer fine roots** (OpenSimRoot's fast fine laterals, 1.5 cm): +40% root length
  for 0.07 g, but no help to either shade gate.
- **Three times the nitrogen uptake capacity**: helped shade mass by a few points, with
  no source for the number.

## Consequences
- Model 0.20.0, species schema 13; every golden hash moves.
- Validation on seed means: 92 checks, 68 pass, every M0 gate passes (0.19.0: 69 of
  90). root_shoot_seed_fill now passes (0.068 → 0.152), drought_root_shoot_cited
  passes again (+9% → +39%), root_length_flowering passes at 68.9 m (56 to 93 over
  the seeds). Shade gates: internodes +26%, total mass −48%, pods −48%. Tap root
  3.6 mm, basal stem 5.3 mm; 10.8 pods and 9.0 g of seed a plant (0.19.0: 9.5 pods
  and 8.8 g).
- New Info misses: drought_shoot_mass (−42% → −65%, against −26%): stomata now close
  with the supply, and drought seed yield falls to −71% (passes). fine_root_share is
  0.082 at flowering (0.1 to 0.4) and about 0.2 at maturity: a lateral gets its fine
  roots only once it finishes. Three verdicts slip at their band edges:
  harvest_index_tops 0.395 (0.4), density_leaf_area_per_plant −34% (−35%) as each
  plant's own roots now take its nitrogen, and pod_set_thermal_time_b10 411 Cd (414).
- Water and roots (single_pot, the default seed, against 0.19.0): root length at
  flowering 14 → 56 m, at day 30 2.6 → 14 m; the plant transpires 8.3 kg over the
  season instead of 3.9 kg, and 2.5 kg instead of 0.5 kg on days 25 to 50. Roots now
  reach the bottom of the 24 cm pot by flowering and spread along it.
- Wheat moves through the soil changes only (it does not thicken and has no fine
  laterals): stand shoot mass 2.95 → 2.64 g, tillers 1.6 → 1.4; its verdicts hold.
- Speed, one container against main at 0.19.0 (best runs): single_pot about +4%
  (3.12 → 3.24 ms per sim-day); plot_30_beans about +18% (30.6 → 36.8 ms), since the
  plot carries 45% more organs (38,000 living at the end, against 26,500); the
  hourly wood build is about 5% of it.
- The viewer's root drawing gains fine-root cohorts (order 2); they have no geometry
  of their own and sit in the cells next to their lateral.
