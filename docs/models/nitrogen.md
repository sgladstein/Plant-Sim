# Nitrogen

## Purpose
Plants build proteins from nitrogen, and leaves need it to photosynthesize. The soil
holds nitrogen in humus, which microbes slowly release as ammonium and then nitrate.
Rain and watering wash nitrate down and out of the pot. Roots take nitrate and ammonium
from the soil around them. A bean can also grow root nodules: bacteria in them fix
nitrogen from the air, paid for with the plant's sugar. When the plant runs short, it
moves nitrogen out of its oldest, most shaded leaves, which yellow and drop from the
bottom up. Fertilizer adds nitrogen to the topsoil.

## Equations
Code: `ps_soil::nitrogen` (soil), `ps_arch::nitrogen` (plant). Decision 0020.

**Soil, per cell, hourly after the water steps** (`SoilGrid::step_hour`):

1. Mineralization `ΔNH₄ = k_min · f_T · f_W / 24 · N_humus`.
2. Nitrification `ΔNO₃ = k_nit · f_T · f_W / 24 · NH₄`.
3. `f_T = min(1.2, (T/30 °C)²)` (T in °C, 0 below 0); `f_W` rises linearly from 0 at
   the wilting point to 1 halfway to field capacity, stays 1 up to field capacity, and
   falls linearly to 0.5 at saturation (APSIM SoilN shape).
4. Drainage moves `NO₃ · drained / W` with the drained water (complete mixing); what
   leaves the bottom is leached. Lateral flow moves `NO₃` with the same fraction as the
   water. NH₄ and humus N do not move.
5. Concentration `ppm = N · 10³ / (ρ_b V)` (mg N per kg soil).

**Uptake, daily step 0, joint over a soil's plants** (`SoilGrid::take_up_nitrogen`):

6. Each root cell's length `L` is spread over the active cells within `r = round(reach /
   Δx)` cells on each axis: `L_j = Σ L_i / n_i` (`SoilGrid::reached_cells`).
7. Per reached cell and form: `P = L_j · cap · (1 − e^(−0.08 ppm)) · √FTSW_j`, at most
   the cell's N above the floor (0.25 ppm NO₃, 0.5 ppm NH₄).
8. Each plant asks `P · min(request, ΣP) / ΣP`. Where the asks to a cell exceed what it
   holds, all are scaled by the same factor.

**Plant, hourly:**

9. Fixation, before allocation: nodule `k` pays `c = min(R_k, f_k · cost / 24)` from
   its reserve and adds `c / cost` to the pool.
10. Growth gate: an organ with `N ≤ c_floor,kind · W` does not grow this hour (roots and
    nodules have floor 0).
11. Expansion stress = water stress × N status (step 18).
12. Top-up, after allocation: deficits `d_i = max(0, c_initial,i W_i − N_i)` of growing
    organs; each gets `d_i · min(1, pool / Σd)`. `growth += max(0, Σd − residual)`, with
    residual the deficit left the hour before.

**Plant, daily step 0** (`Plant::nitrogen_day`):

13. Demand `D = Σ d_i + growth(yesterday)`.
14. The soil request is `max(0, D − pool + fixed(yesterday))`.
15. Mining, if still short: each non-growing organ (not seeds, seeds in pods or nodules) gives up to
    `rate · w · (N − c_final W)`, with `rate = 0.12 d⁻¹` during seed fill and
    `0.12 × 0.36` before, and leaf weight `w = clamp(2 (1 − L / L_max), 0.1, 1)` by light
    index (1 for other organs). All give the same share of their cap.
16. Fixation for tomorrow: `need = max(0, D − pool)`; nodule rates
    `f_k = W_k · a · f_T,fix · f_drought · min(1, need / capacity)`, with
    `a = 0.05 g N g⁻¹ d⁻¹`.
17. Nodulation, with rhizobia in the soil, after emergence and before maturity: target
    mass `need / (a · f_T,fix · f_drought)`. Root-zone NO₃ is read over the reached cells (step 6). If above the clusters' potential mass,
    add `min(target − potential, step) · f_NO₃` of new clusters, with step `0.17 ·
    potential`, or 14 mg at first. `f_NO₃` is 1 at 5 ppm root-zone NO₃ or less, falling
    linearly to 0.1 at 30 ppm. Clusters are 2 mg each, at most 24 a day, on root
    segments above 30 cm depth with the fewest nodules.
18. Status `S = Σ (N − c_floor W)⁺ / Σ (c_initial − c_floor) W` over growing leaves and
    internodes.
19. Leaf drop: a mined leaf with `N − c_final W < 0.1 (c_initial − c_final) W` is shed.
20. Leaf capacity `= aging × clamp((Na − 0.3) / (Na_ref − 0.3), 0, 1)`, with
    `Na = N / area` and `Na_ref = c_initial,leaf / SLA`.
21. Death: `(N − c_final W)⁺` returns to the pool, the rest is litter.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Mineralization rate at 30 °C | 1.5e-4 | 1/day | ~ APSIM SoilN humic pool |
| Nitrification rate at 30 °C | 0.34 | 1/day | ~ (0.15 at 20 °C) |
| Uptake floors NO₃, NH₄ | 0.25, 0.5 | mg N/kg | CROPGRO NUPTAK |
| Uptake response | 0.08 | per mg N/kg | CROPGRO NUPTAK |
| Bulk density loam, sand, clay | 1300, 1550, 1250 | kg/m3 | ~ typical mineral soils |
| Garden soil NO₃, NH₄, humus N | 20, 2, 1000 | mg N/kg | ~ (decision 0020) |
| leaf, stem, root, shell, seed N initial | 0.0384, 0.024, 0.016, 0.0352, 0.0376 | g N/g DM | CROPGRO-Drybean BNGRO048.SPE (protein × 0.16) |
| leaf, stem, root, shell N final | 0.0112, 0.008, 0.0096, 0.0128 | g N/g DM | CROPGRO-Drybean BNGRO048.SPE |
| seed N final | 0.02 | g N/g DM | ~ |
| nodule N | 0.048 | g N/g DM | CROPGRO-Drybean PRONOD |
| leaf_n_area_base | 0.3 | g N/m2 | ~ Kattge 2009, Walker 2014 |
| uptake_capacity | 1.5e-2 | g N/m/day | ~ 25 × CROPGRO RTNO3 (root length the sim does not draw; decision 0020 amendment) |
| root_reach | 4 | cm | ~ (decision 0020) |
| mobilization_rate, vegetative_mobilization | 0.12, 0.36 | 1/day, - | CROPGRO NMOBMX, NVSMOB |
| leaf_drop_mobile | 0.1 | - | ~ |
| nodule_activity | 0.05 | g N/g DM/day | CROPGRO SNACTM |
| nodule_relative_growth | 0.17 | 1/day | CROPGRO NODRGM |
| nodule_initial_mass, nodule_mass | 0.014, 0.002 | g DM | CROPGRO DWNODI; ~ |
| nodule_expansion, nodule_depth | 100, 30 | Cd, cm | ~ |
| fixation_cost | 7.1 | g C/g N | CROPGRO RFIXN |
| fixation temperatures | 4, 19, 35, 44 | C | CROPGRO FNFXT |
| fixation_drought_threshold | 0.85 | - | CROPGRO FNFXD |
| nodule_nitrate onset, full, floor | 5, 30, 0.1 | mg N/kg, mg N/kg, - | ~ research notes |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | water drained and moved laterally, cell water | kg | soil water |
| in | layer temperature | C | soil |
| in | root length per cell | m | roots |
| in | fertilizer | g N/m2 | actions |
| in | nodule reserve | g C | carbon allocation |
| out | leaf capacity | - | gas exchange (class table) |
| out | expansion stress, growth gate | - | organ growth |
| out | fixation carbon | g C | carbon ledger |
| out | organ N | g N | maintenance respiration |
| out | soil and plant N ledgers | g N | world ledger, daily CSV |

## Encoded or emergent
Encoded: the soil rates, the uptake law, the order of supply (pool, soil, mining,
fixation), the mining weight by light, the nitrate factor on nodulation, the drop
threshold.

Emergent:

- Leaves drop from the bottom up during seed fill.
- Nodules form only when the soil runs short, so a fertilized plant fixes almost
  nothing.
- A nodulated bean grows in N-free sand, and an unnodulated one stalls on its seed N.
- Nitrate is washed below the roots of an over-watered pot.
- Shaded leaves hold less N per area (thinner leaves, same N per mass), so they have
  less capacity.

## Known limits
- The root reach and the 25× uptake capacity stand for root length the sim does not
  draw. A fuller root system would replace them.
- The soil water has no upward flow, so nitrate below the root zone reaches roots only
  through the reach.
- Leaf aging still multiplies capacity next to leaf N (decision 0020).
- Pod set does not read N. An N-short plant sets as many pods as a fed one and fills
  smaller seeds; real beans shed pods instead.
- No denitrification, volatilization, immobilization or residue decay; litter N leaves
  the system.
- An unfertilized 24 cm pot holds far less N per plant than a field, so it is N-limited
  (seed mass 0.17 g against 0.25 fed).
- Seed N during seed fill can dilute to 0.02 g/g, which is a guess.

## Validation
- `ps_soil::nitrogen::tests::*`: rates to 1e-9 relative, factor shapes, leaching closes
  the ledger, lateral spread, fertilizer in the top layer, uptake limited by root length,
  soil N and demand and fair between plants, root reach conserves length.
- Debug builds assert the soil N ledger every hour and each plant's N ledger every day
  at 1e-9 relative.
- `ps-validate/tests/nitrogen_free.rs::nitrogen_free_sand_needs_nodules` (WP24
  done-when): nodulated over unnodulated shoot mass above 3 at day 57; the
  unnodulated plant took up and fixed nothing and holds at most its seed N.
- `ps-arch/tests/ninety_days.rs::seed_fill_drops_leaves_from_the_bottom_up` (WP24
  done-when): leaves dropped during seed fill sit at nodes 2 to 5, survivors at 6 to 8.
- Validation report: `nfree_nodulation_shoot_mass` (Info). Season Ndfa in garden soil
  is 18% in the 30-bean plot, 45% at 35 plants per m² and 55% in a garden pot (research
  band 30 to 60%, Palmero 2022).

## References
- DSSAT CSM CROPGRO-Drybean, BNGRO048.SPE, and CROPGRO NUPTAK, NFIX, MOBIL routines
  (github.com/DSSAT/dssat-csm-os).
- APSIM SoilN documentation (mineralization and nitrification factors).
- Palmero F. et al. (2022) Ndfa across legumes (meta-analysis); research notes
  m2-m3-bean-science.md.
- Kattge J. et al. (2009); Walker A.P. et al. (2014): Vcmax and leaf N per area.
