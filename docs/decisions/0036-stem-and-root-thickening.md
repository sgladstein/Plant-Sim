# 0036: Stem and root thickening (R1a)

- **Date:** 2026-11-05
- **Status:** accepted (owner approved the realism plan's R1 to R3 on 2026-11-05; R1a
  is item A1, "Needed (owner: must)")

## Context
Stems and roots only grew in length: an organ's radius came from its primary allometry
and never changed after expansion. So bean stems were light and untapered, the root
system was about 4% of dry mass (root:shoot 0.045 at seed fill against 0.08 to 0.30),
and surplus sugar had nowhere to go once leaves and pods were set, which the sink
feedback then discarded. The realism plan's R1 spec
(`docs/research/realism-plan/r1-carbon-sinks.md`) asked for secondary growth as a
carbon sink at the §3.9 priority 0.4, and the research thread's R1 numbers
(`r1-sinks-numbers.md`) corrected the root rule: the secondary growth factor is keyed on
distance from the root's base, and OpenSimRoot's table is a diameter increment.

## Decision
1. **A secondary mass column** on the organ arena (`ps_graph::OrganArena::secondary`,
   serde default, so older saves load with none). Structure `W` is primary plus
   secondary. Length stays the primary allometry's, `L = SL · (W − W_s)`; the radius is
   `r = √(W / (ρ π L))`, so wood widens an organ at fixed length
   (`Allometry::size_split`). The primary radius `r_p = r √((W − W_s) / W)` is what the
   organ had before it thickened.
2. **Stems: the pipe model** (Shinozaki et al. 1964). Daily, after initiation, each
   living internode's target cross-section is `X* = h_L · A_max + h_F · ΣF`, with
   `A_max` the most leaf area it has carried above it (pipes stay after their leaves
   die) and `ΣF` the fruit dry mass above it now (flowers, pods, seeds). Hourly, once an
   internode has finished expanding, its potential is
   `ΔW = min(X* − π r², g_max · 2π r · Δt) · L · ρ_stem · f_T · s`.
   Bean `h_L` 3.0e-4 m² m⁻², `h_F` 0, `g_max` 0.1 mm of radius a day.
3. **Roots: OpenSimRoot's bean rates** (Postma et al. 2017; Strock et al. 2018). A root
   segment's diameter grows at a rate by its own age (0 to 0.003 cm a day over 29 days;
   a cumulative 0.1 cm by day 43), times a factor by its distance from the root's base
   (tap root 4 to 20 cm then down to 2 by 40 cm, basal roots 0.7 to 0.4, hypocotyl-borne
   0.6 to 0.2, laterals 0.1), so old basal segments grow widest and roots taper:
   `ΔW = 2π r L · ½ ḋ(age) · k(distance) · Δt · ρ_root · f_T · s`.
4. **A sink at priority 0.4.** Both are potentials; allocation step 12 builds, after the
   organ's primary growth, `min(ΔW · c_f · cc, 0.4 · max(0, R − 0.05 W_p))`, paying the
   organ's construction cost. Unmet thickening is not booked: wood takes surplus only.
   `f_T` is the expansion temperature multiplier at the organ's temperature and `s`
   the expansion stress factor (water times nitrogen). Nothing thickens before
   germination or after maturity.
5. **Wood is not phloem and holds no nitrogen.** Sugar concentration and the reserve
   floor use primary mass, the phloem conductance uses the primary radius, and every
   nitrogen rule (deficits, growth gate, mining, status, dead-organ N) uses primary
   mass. So wood pays no maintenance (maintenance is per gram of N).
6. **Grasses have no cambium**: wheat's `secondary_growth` is 0 and nothing thickens.
7. **Species schema 12**: a `thickening` group (`secondary_growth`,
   `pipe_area_per_leaf_area`, `pipe_area_per_fruit_mass`, `stem_radial_rate_max`), a
   `root_secondary_rate` table and a `secondary_factor` table per root type, as the new
   `ParamTable` type (points, x unit, unit, source) with linear interpolation, level at
   the ends.
8. **Outputs**: the daily rows gain `hypocotyl_diameter_mm` and `tap_root_diameter_mm`
   (5 cm below the collar), the organ rows `secondary_g`. Validation gains
   `basal_stem_diameter` (4.5 to 10 mm at seed fill), `tap_root_diameter` (2 to 6 mm)
   and `stem_share_peak` (stem over stem plus leaf mass, 0.35 to 0.60), all Info, with
   the derived variable `stem_share`. Model 0.17.0 → 0.18.0, goldens regenerated.

## Consequences
- Done-when (`ps-arch/tests/thickening.rs`, 56 days in a 24 cm pot): the hypocotyl is
  6.0 mm; the main stem tapers from 7.3 mm at the base to 4.6 mm at the tip; every
  thickened internode keeps its primary radius and length to 1e-9; the pipe record
  never falls; stem mass still rises in the last week; the tap root is 3.9 mm at 5 cm
  and 1.6 mm at 15 cm. Wheat makes no secondary mass, and the wheat scenarios' daily and
  organ outputs are identical to 0.17.0.
- Validation on four-seed means: 90 checks, **69 pass** (0.17.0: 67 of 87), every M0
  gate passes. New: hypocotyl 5.8 mm at seed fill (5.5 to 6.1) and tap root 4.1 mm (3.9
  to 4.3) pass; `stem_share_peak` misses at 0.31 (band 0.35 to 0.60; R1b's stem storage
  adds to it). `harvest_index_tops` passes again (0.392 → 0.424): wood draws carbon
  during pod set, so the pot plant sets fewer pods (13.0 → 9.5) and fills each better
  (0.235 → 0.254 g a seed); total mass at maturity 24.7 → 23.4 g, peak leaf area 0.221
  → 0.190 m². `drought_root_shoot_cited` now misses (+23% → +9%): thickening adds root
  mass in the control as well as under drought, so the relative rise is smaller.
  `root_shoot_seed_fill` rises 0.045 → 0.068 and still misses (0.08 to 0.30).
  Garden-plot Ndfa rises 0.32 → 0.43.
- **Root length falls** (about −20% at day 60 in the pot, −33% over the plot season):
  root reserve that went into elongation now also builds wood. Real roots make that
  trade (Strock et al. 2018). The root system was already far too short early, which
  is why uptake meets only about half the transpiration demand (#116); R2a raises root
  length and fixes that next.
- Speed, this container, interleaved runs: single_pot 3.75 → 3.99 ms per sim-day (+6%),
  plot_30_beans 27.1 to 28.6 → 30.4 to 31.6 ms (+10%). Most of it is the hourly root
  potential (two table lookups and a temperature factor per root segment) and the
  extra size update per thickening organ.
- Known limits (organ-growth card): no secondary phloem, so thickening adds neither
  conductance nor sugar capacity; wood holds no nitrogen; one root rate table per
  species; a thin hypocotyl draws less carbon than the epicotyl and is narrower than it
  until late in the season; the pipe target ignores wind and self-weight; no bark or
  heartwood.
