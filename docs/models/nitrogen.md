# Nitrogen

## Purpose
Plants build proteins from nitrogen, and leaves need it to photosynthesize. The soil
holds nitrogen in humus, which microbes slowly release as ammonium and then nitrate.
Rain and watering wash nitrate down and out of the pot. Roots take nitrate and ammonium
from the soil around them. A bean can also grow root nodules: bacteria in them fix
nitrogen from the air, paid for with the plant's sugar. When the plant runs short, it
moves nitrogen out of its oldest, most shaded leaves, which yellow and drop from the
bottom up. Once the seeds start filling, they take nitrogen from the leaves first,
however rich the soil. Fertilizer adds nitrogen to the topsoil.

## Equations
Code: `ps_soil::nitrogen` (soil), `ps_arch::nitrogen` (plant). Decision 0020.

**Soil, per cell, hourly after the water steps** (`SoilGrid::step_hour`):

1. Mineralization `ΔNH₄ = k_min · f_T · f_W / 24 · N_humus`.
2. Nitrification `ΔNO₃ = k_nit · f_T · f_W / 24 · NH₄`.
3. `f_T = min(1.2, (T/30 °C)²)` (T in °C, 0 below 0); `f_W` rises linearly from 0 at
   the wilting point to 1 halfway to field capacity, stays 1 up to field capacity, and
   falls linearly to 0.5 at saturation (APSIM SoilN shape).
4. Drainage moves `NO₃ · drained / W` with the drained water (complete mixing); what
   leaves the bottom is leached. Laterally (R2b, decision 0043) nitrate moves with the
   water's flux `f` at the concentration of the cell the water leaves,
   `f · NO₃_from / W_from`, plus diffusion `d (NO₃_i − NO₃_j)` with
   `d = D_l θ^{7/3} / θ_s² · 3600 s / Δx²`, 5.1e-3 `(0.02 m / Δx)²` per hour (a cell's
   nitrate evens out at about 5.7e-10 m² s⁻¹, Millington–Quirk at field capacity: the
   factor `θ^{10/3}/θ_s²` already holds the water's share of the section, and the grid
   exchanges amounts, which divides by `θ`); both stop at a pot divider. Before R2b
   nitrate spread at the water's fraction, 0.1 per hour in pot cells (a diffusivity
   about 20 times too high), so a patch evened out in days. NH₄ and humus N do not
   move.
5. Concentration `ppm = N · 10³ / (ρ_b V)` (mg N per kg soil).

**Uptake, daily step 0, joint over a soil's plants** (`SoilGrid::take_up_nitrogen`):

6. Each root cell takes up from itself only, by its own root length `L` (R2a, decision
   0037: the 4 cm reach and the 25× capacity that stood in for undrawn roots are gone).
7. Per root cell and form: `P = L · cap · (1 − e^(−0.08 ppm)) · √FTSW`, at most
   the cell's N above the floor (0.25 ppm NO₃, 0.5 ppm NH₄).
8. Each plant asks `P · min(request, ΣP) / ΣP`. Where the asks to a cell exceed what it
   holds, all are scaled by the same factor.

**Plant, hourly:**

9. Fixation, before allocation: nodule `k` pays `c = min(R_k, f_k · cost / 24)` from
   its reserve and adds `c / cost` to the pool.
10. Growth gate: an organ with `N ≤ c_floor,kind · W` does not grow this hour (roots and
    nodules have floor 0). Here and in every rule of this card `W` is the organ's
    primary structure: secondary wood from thickening (decision 0036) needs no
    nitrogen, so a thickening stem does not dilute its N or raise its demand.
11. Expansion stress = water stress × N status (step 18).
12. Top-up, after allocation: deficits `d_i = max(0, c_initial,i W_i − N_i)` of growing
    organs; each gets `d_i · min(1, pool / Σd)`. `growth += max(0, Σd − residual)`, with
    residual the deficit left the hour before; the seeds' share is tracked the same way
    as `seed_growth` (R1b).

**Plant, daily step 0** (`Plant::nitrogen_day`):

13. Demand `D = Σ d_i + growth(yesterday)`. From seed fill (stage R5) on, with
    `seed_fill_mining_first`, the seeds come first: `M₁ = min(D_seeds, capacity)`, with
    `D_seeds` the filling seeds' deficits plus `seed_growth(yesterday)` and capacity the
    day's mining caps of step 15 (R1b, decision 0040; CROPGRO's order after R5, the
    "self-destruction" of Sinclair and de Wit 1975). Before seed fill `M₁ = 0`. The
    mined nitrogen goes into the pool, which every growing organ shares pro rata, so
    the rule sets how much is mined, not who gets it; in seed fill the seeds hold
    nearly all the demand.
14. The soil request is `max(0, D − pool + fixed(yesterday) − M₁)`.
15. Mining: `M₁` plus whatever the pool and the soil still leave short. Each
    non-growing organ (not seeds, seeds in pods or nodules) gives up to
    `rate · w · (N − c_final W)`, with `rate = 0.12 d⁻¹` during seed fill and
    `0.12 × 0.36` before, and leaf weight `w = clamp(2 (1 − L / L_max), 0.1, 1)` by light
    index (1 for other organs). All give the same share of their cap. This reorders the
    supply inside daily step 0 only, not the §3.2 step list.
16. Fixation for tomorrow: `need = max(0, D − pool)`; nodule rates
    `f_k = W_k · a · f_T,fix · f_drought · min(1, need / capacity)`, with
    `a = 0.05 g N g⁻¹ d⁻¹`.
17. Nodulation, with rhizobia in the soil, after emergence and before maturity: target
    mass `need / (a · f_T,fix · f_drought)`. Root-zone NO₃ is the root-length-weighted mean over the root cells. If above the clusters' potential mass,
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
| Nitrate diffusion fraction at 2 cm | 5.1e-3 | per hour | Millington–Quirk at field capacity, `D₀ θ^{7/3}/φ²` with D₀ 1.9e-9 m² s⁻¹, θ 0.3, φ 0.45 (R2b, decision 0043) |
| Bulk density loam, sand, clay | 1300, 1550, 1250 | kg/m3 | ~ typical mineral soils |
| Garden soil NO₃, NH₄, humus N | 20, 2, 1000 | mg N/kg | ~ (decision 0020) |
| leaf, stem, root, shell, seed N initial | 0.0384, 0.024, 0.016, 0.0352, 0.0376 | g N/g DM | CROPGRO-Drybean BNGRO048.SPE (protein × 0.16) |
| leaf, stem, root, shell N final | 0.0112, 0.008, 0.0096, 0.0128 | g N/g DM | CROPGRO-Drybean BNGRO048.SPE |
| seed N final | 0.02 | g N/g DM | ~ |
| nodule N | 0.048 | g N/g DM | CROPGRO-Drybean PRONOD |
| leaf_n_area_base | 0.3 | g N/m2 | ~ Kattge 2009, Walker 2014 |
| uptake_capacity | 6.0e-4 | g N/m/day | CROPGRO RTNO3 = RTNH4, each form (decision 0037; was 25× with a 4 cm reach, decision 0020 amendment) |
| mobilization_rate, vegetative_mobilization | 0.12, 0.36 | 1/day, - | CROPGRO NMOBMX, NVSMOB |
| leaf_drop_mobile | 0.1 | - | ~ (0.3 dropped lower leaves in a fertilized pot but cost 12% of seed mass; decision 0040) |
| seed_fill_mining_first | 1 | - | CROPGRO order after R5 (archive m2-m3 science); wheat ~ (Kichey et al. 2007, to verify) |
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
fixation; seeds mine first in seed fill), the mining weight by light, the nitrate
factor on nodulation, the drop threshold.

Emergent:

- Leaves drop from the bottom up during seed fill.
- Nodules form only when the soil runs short, so a fertilized plant fixes almost
  nothing.
- A nodulated bean grows in N-free sand, and an unnodulated one stalls on its seed N.
- Nitrate is washed below the roots of an over-watered pot.
- Shaded leaves hold less N per area (thinner leaves, same N per mass), so they have
  less capacity: a leaf at 1.2× the SLA gets about 0.78 of it, since Vcmax per area
  tracks N per area (Hirose and Werger). This is now the main way shade costs a plant
  mass (decision 0020 amendment).

## Known limits
- Uptake follows CROPGRO's per-length rate on the drawn root length (R2a). That length
  is near the low end of pot data, so in the 300 ppm M0 pots the daily capacity is close
  to demand around day 40: the end-of-day deficits (`n_unmet_g`) reach about 20% of
  demand before the nightly top-up, though growth is rarely gated. CROPGRO's response
  saturates by about 40 ppm, with no low-affinity uptake at high nitrate (decision 0037).
- Mineralization is low for a garden: about 0.03 g N per m² a day on the plot against
  0.05 to 0.15.
- The soil water has no upward flow, and nitrate diffuses only slowly (R2b), so a
  cell's roots see the nitrate that is in it or that water brings. Where water flows to
  roots that take up less nitrate than it carries (the 300 ppm M0 pots, whose uptake
  saturates), nitrate gathers in the rooted cells: by day 45 the split pot's roots sit
  in about 2.3 times the nitrate of its rootless cells (root-length weighted, seed 9;
  decision 0043). Real roots in rich soil
  do this too, but more of it would diffuse back at the scale of a root.
- Leaf aging still multiplies capacity next to leaf N (decision 0020).
- Pod set does not read N. An N-short plant sets as many pods as a fed one and fills
  smaller seeds; real beans shed pods instead.
- No denitrification, volatilization, immobilization or residue decay; litter N leaves
  the system.
- An unfertilized 24 cm pot holds far less N per plant than a field, so it is N-limited
  (seed mass 0.17 g against 0.25 fed).
- Seed N during seed fill can dilute to 0.02 g/g, which is a guess.
- A fertilized pot mines its leaves through seed fill (R1b) but keeps them to maturity:
  its lowest leaves end with 16 to 21% of their mobile N, above the 10% drop threshold.
  CROPGRO instead sheds leaf mass in proportion to the protein mined (`SENRTE`, to
  verify); a candidate follow-up (decision 0040).

## Validation
- `ps_soil::nitrogen::tests::*`: rates to 1e-9 relative, factor shapes, leaching closes
  the ledger, nitrate moving with the water and diffusing slowly
  (`nitrate_moves_with_the_water_and_diffuses_slowly`, `a_divider_stops_water_and_nitrate`,
  R2b), fertilizer in the top layer, uptake limited by root length, soil N and demand
  and fair between plants.
- Debug builds assert the soil N ledger every hour and each plant's N ledger every day
  at 1e-9 relative.
- `ps-validate/tests/nitrogen_free.rs::nitrogen_free_sand_needs_nodules` (WP24
  done-when): nodulated over unnodulated shoot mass above 3 at day 57; the
  unnodulated plant took up and fixed nothing and holds at most its seed N.
- `ps-arch/tests/ninety_days.rs::seed_fill_drops_leaves_from_the_bottom_up` (WP24
  done-when): leaves dropped during seed fill sit at nodes 2 to 6, survivors at 7 to 8.
- `ps-arch/tests/ninety_days.rs::seed_fill_mines_first_in_a_fertilized_pot` (R1b): at
  300 ppm nitrate the plant mines 5 mg before seed fill and 0.27 g during it, against
  0.37 g taken up.
- Validation report: `nfree_nodulation_shoot_mass` (Info). `ndfa_garden_plot` (Info):
  season Ndfa in the 30-bean plot is 18% against the 30 to 60% band (Palmero 2022), a
  miss. `ndfa_low_n_bed` (Info): the same plot at 5 ppm nitrate gives 56% against 50 to
  80%. Ndfa is 45% at 35 plants per m² and 55% in a garden pot.

## References
- DSSAT CSM CROPGRO-Drybean, BNGRO048.SPE, and CROPGRO NUPTAK, NFIX, MOBIL routines
  (github.com/DSSAT/dssat-csm-os).
- APSIM SoilN documentation (mineralization and nitrification factors).
- Palmero F. et al. (2022) Ndfa across legumes (meta-analysis); research notes
  m2-m3-bean-science.md.
- Kattge J. et al. (2009); Walker A.P. et al. (2014): Vcmax and leaf N per area.
- Hirose T., Werger M.J.A. (1987): leaf N per area and photosynthetic capacity in a canopy.
- Sinclair T.R., de Wit C.T. (1975). Photosynthate and nitrogen requirements for seed
  production by various crops. Science 189: seed fill strips leaf nitrogen
  ("self-destruction").
