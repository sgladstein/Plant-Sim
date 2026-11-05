# Leaf energy balance

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
A leaf in the sun is not at air temperature. Sunlight warms it, and it loses heat to the
air, to the sky as infrared, and by evaporating water through its pores. A well-watered
leaf in dry air can run a few degrees cooler than the air; a thirsty leaf with closed
pores, in still air, can run several degrees warmer; and on a clear, calm night an
exposed leaf radiates its heat to the cold sky and falls below air temperature, so it can
freeze while the air stays just above freezing. The leaf's own temperature sets how fast
it photosynthesises, respires and loses water.

## Equations
Per unit one-sided leaf area; both faces exchange heat and longwave (crate
`ps-physio::energy`, research `docs/research/wp27-leaf-energy-balance`, decision 0034).

- Absorbed shortwave: `S = Q / 4.57 · (1 + r)`, with `Q` the absorbed PAR (µmol m⁻² s⁻¹,
  from the light model) and `r = (1 − 0.5)/0.5 · a_nir / a_par` the absorbed near
  infrared per absorbed PAR energy (0.235 for bean and wheat).
- Longwave: `L = ε σ Ta⁴ ((2 − f) + f ε_sky)`, with `f` the leaf's sky view (its diffuse-sky
  transmittance, 1 in the open) and the sky's emissivity
  `ε_sky = (1 − 0.84 c) · 1.72 (ea / Ta)^(1/7) + 0.84 c` (Brutsaert 1975, clear sky; cloud
  correction of Unsworth and Monteith 1975), `c` the hour's cloud fraction, `ea` kPa, `Ta` K.
  The rest of the leaf's surroundings radiate at air temperature.
- Boundary layer, heat, per face (mol m⁻² s⁻¹):
  `g_bh = (1.4 · 0.00328 √(u / d) + 0.5 D_H (1.6e8 |ΔT| d³)^¼ / d) · P / (R Ta)`: forced
  convection at the leaf's wind `u` (floored at 0.1 m s⁻¹) plus free convection at the
  leaf-air difference `ΔT` (Leuning et al. 1995, as plantecophys), with `d = 0.72 ×` leaf
  width. Vapor: `g_bw = 1.075 g_bh`.
- Vapor conductance: each face's stomata (share `a` of `gs` on the lower face) in series
  with that face's boundary layer, the faces in parallel:
  `g_v = series(a gs, g_bw) + series((1 − a) gs, g_bw)`, `series(x, y) = xy/(x + y)`.
- Balance: `S + L − 2εσT⁴ − 2 cp g_bh (T − Ta) − λ g_v (es(T) − ea)/P = 0`, with
  `cp = 29.3 J mol⁻¹ K⁻¹`, `λ` the latent heat at air temperature (J mol⁻¹), `es` Tetens
  (the weather's form, so the leaf-to-air deficit equals the air's when `T = Ta`).
  Solved by the linearised (Penman–Monteith) form with radiative conductance
  `g_r = 4εσTa³/cp`, free convection at that estimate, then two Newton steps on the
  full balance. Transpiration `E = g_v (es(T) − ea)/P`.
- Coupling (each light-class node): stomata respond to leaf temperature (FvCB kinetics,
  Rd) and to the leaf-to-air deficit `es(T) − ea`, so `g(T) = T_eb(gs(T)) − T` is solved
  for its root: a first guess (air temperature, or the neighboring nodes' answer), one
  fixed-point step, clamped secant steps until bracketed, then Illinois (modified
  regula falsi), until `|g| < 0.01 K` or the bracket is that narrow, at most 8 gas-exchange
  solves; the answer is the evaluation nearest its own balance. In the dark `gs = g0`, so
  one balance does.
- The class table (§3.8, decision 0009) is solved this way at the hour's reference
  environment: a horizontal leaf under the open sky (`f = 1`) at 0.5 m, in the hour's air
  and wind. Plants of a soil with the same leaf and energy parameters and stress share it.
- Each leaf part (sunlit, shaded) then solves its own temperature and transpiration with
  the closed form and Newton steps above, from its own absorbed PAR, its own sky view,
  its own wind and the conductance it read from the table (scaled with assimilation by
  the sink feedback). Assimilation and day respiration stay the table's (the node's
  temperature). The leaf's temperature is the parts' area-weighted mean; its water use
  is the parts' transpiration, at least 0 (dew does not water the plant).
- Wind at a leaf: `u = u2 · ln(z/z0)/ln(2/z0) · √τ`, the 2 m wind on a neutral log
  profile over the surroundings' roughness `z0 = 0.02 m` at the leaf's height `z` (held
  ≥ 0.04 m), attenuated by the leaf area above it through its diffuse-sky transmittance
  `τ` (wind and diffuse light extinguished alike, as GECROS).
- Hourly wind (ps-weather): a daily mean (from the weather file, else 2 m s⁻¹, FAO-56)
  spread over the day by DSSAT's HWIND: night `0.3 W_max`, a daytime sine peaking at solar
  noon, `W_max = W_avg / (0.3 + 0.7 · daylength/48)`, so the daily mean is kept.

What reads leaf temperature: FvCB kinetics, day respiration, stomata (leaf-to-air
deficit), transpiration, and leaf frost (owner decision 2026-11-05: a leaf dies when its
own lowest temperature of the day is below `leaf_frost_kill`). What stays on air:
thermal time and phenology, leaf expansion, maintenance of stems, flower and pod heat
responses, and the frost that kills growing points, buds and flowers.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| leaf_width | bean 0.08, wheat 0.015 | m | "~" research notes/boundary_layer_wind.md (bean leaflet 0.05 to 0.11; wheat blade 0.012 to 0.022) |
| leaf_nir_absorptance | 0.20 | 1 | "~" PROSPECT-D runs; Goudriaan NIR scattering 0.8 (research notes/leaf_optics.md) |
| leaf_emissivity | 0.97 | 1 | Chen 2015 review (bean 0.957, Fuchs and Tanner 1966); tealeaves default |
| abaxial_conductance_share | bean 0.875, wheat 0.5 | 1 | Kanemasu and Tanner 1969 (bean about 7:1); wheat "~" |
| characteristic dimension / width | 0.72 | 1 | Campbell and Norman 1998 |
| forced convection coefficient | 0.00328 | m s⁻½ | laminar flat plate (0.135 mol m⁻² s⁻¹ at sea level, 20 °C) |
| field enhancement | 1.4 | 1 | Campbell and Norman 1998 (Chamberlain 1974: 1.25 for bean leaves in a canopy) |
| thermal diffusivity D_H | 21.5e-6 | m² s⁻¹ | Leuning et al. 1995, as plantecophys |
| vapor/heat boundary-layer ratio | 1.075 | 1 | Leuning et al. 1995 |
| wind floor | 0.1 | m s⁻¹ | DSSAT (Kimball 2014), CLM |
| default wind | 2 | m s⁻¹ at 2 m | FAO-56 |
| surroundings' roughness z0 | 0.02 | m | "~" short ground around a garden plot (0.01 to 0.03) |
| reference leaf height | 0.5 | m | "~" a leaf near the top of a small garden plant |
| coupled tolerance, evaluations | 0.01, 8 | K, 1 | research prototype: Illinois with 8 evaluations within 0.03 K of bisection |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | air temperature, VPD, pressure, cloud fraction, wind at 2 m | °C, kPa, kPa, 1, m s⁻¹ | ps-weather |
| in | absorbed PAR (sunlit, shaded), sky view (diffuse transmittance) | µmol m⁻² s⁻¹, 1 | ps-light |
| in | leaf height | m | the plant's daily layout |
| in | stomatal conductance | mol m⁻² s⁻¹ | gas exchange (class table) |
| out | leaf temperature, leaf's lowest today | °C | gas exchange, frost, the temperature lens |
| out | transpiration | mol m⁻² s⁻¹ | water demand (soil uptake) |

## Encoded or emergent
Encoded: the energy balance terms, the conductances, which processes read leaf
temperature. Emergent: transpirational cooling of well-watered leaves in dry air, hot
leaves under drought, warmer leaves in still air and inside the canopy, the difference
between sunlit tops and shaded inner leaves, radiation frost on the exposed leaves of a
clear night while covered leaves survive, and the water saved by the boundary layer.

## Known limits
- Assimilation and day respiration come from the class table, solved for a horizontal
  leaf in the open at 0.5 m; a leaf's own temperature corrects its transpiration but not
  its assimilation (error about (∂A/∂T)·(T_leaf − T_node), a few percent per leaf, less
  in the plant total).
- Between table nodes, interpolated assimilation is within 7% of a direct coupled solve
  (4.6% without coupling) and leaf temperature within 0.24 K
  (`coupled_table_matches_coupled_solves`).
- Near infrared is taken at its open-sky share for sunlit and shaded parts alike; shade is
  NIR-rich, so shaded leaves run slightly too cool.
- No boundary layer for CO2 (it would cut A by 1 to 3% at low wind and break the
  plantecophys fixture).
- When the soil cannot meet the demand, the leaf does not warm for the water it did not
  get (the stress factor already closes stomata; GECROS re-solves).
- Leaves are flat and fixed; bean leaflets that fold edge-on to the sun under stress
  (paraheliotropism) would run cooler.
- One leaf width per species; a young leaf is as wide as a grown one.
- The plot does not slow its own wind beyond each leaf's shading; there is no canopy air
  temperature or humidity, and the soil surface does not feel the leaves.

## Validation
WP27 "done when" (`ps_physio::energy::tests`, a sunlit horizontal bean leaflet under a
clear sky at PAR 1500 unless stated):
1. `hot_still_day_well_watered`: Ta 35 °C, RH 30%, 0.5 m s⁻¹: −1.17 K (pass −7 to +1).
2. `dry_air_and_wind_cool_a_watered_leaf`: VPD ≥ 3 kPa, 1 to 4 m s⁻¹: −0.31 to −1.33 K
   (pass < 0).
3. `closed_stomata_heat_the_leaf`: stress factor 0: +5.44 K, 6.6 K above case 1 (pass +1
   to +8 and ≥ 3 above case 1).
4. `clear_night_cools_an_exposed_leaf`: Ta 15 °C, 0.5 m s⁻¹, sky view 1: −1.51 K (pass −3
   to −0.5); overcast warmer.
5. `irrigated_wheat_runs_cool`: Ta 33 to 37 °C, low RH: −1.83 to −1.97 K (pass < 0).
6. `ps-arch/tests/leaf_energy.rs::transpiration_from_the_soil_equals_the_plant_total`:
   every hour of 30 days, uptake plus shortfall equals demand and the soil ledger books
   the plant's uptake, to 1e-9.

Solver: `newton_steps_close_the_balance` (within 0.02 K of the full balance over 5 to
42 °C, wind 0.05 to 3 m s⁻¹, gs 0 to 1), `coupled_solve_matches_bisection` (within
0.05 K of a 60-step bisection over 180 cases, 5 to 42 °C; measured 0.003 K),
`ps_physio::tests::coupled_table_matches_coupled_solves`.
Plant: `ps-arch/tests/leaf_energy.rs::a_clear_calm_night_frosts_exposed_leaves_but_not_the_growing_points`
(air −1.5 °C, frost kill −2.2 °C: 3 of 5 leaves freeze, the apices live).

## References
- Brutsaert, W. (1975). On a derivable formula for long-wave radiation from clear skies.
  Water Resources Research 11.
- Campbell, G. S. and Norman, J. M. (1998). An Introduction to Environmental Biophysics,
  2nd ed. Springer.
- Chamberlain, A. C. (1974). Mass transfer to bean leaves. Boundary-Layer Meteorology 6.
- Duursma, R. A. (2015). Plantecophys. PLoS ONE 10 (`PhotosynEB`).
- Kanemasu, E. T. and Tanner, C. B. (1969). Stomatal diffusion resistance of snap beans.
  Plant Physiology 44.
- Leuning, R. et al. (1995). Leaf nitrogen, photosynthesis, conductance and transpiration:
  scaling from leaves to canopies. Plant, Cell and Environment 18.
- Pickering, N. B. et al. DSSAT `HMET.for` (HWIND) and `ETPHR.for`.
- Unsworth, M. H. and Monteith, J. L. (1975). Long-wave radiation at the ground.
  Quarterly Journal of the Royal Meteorological Society 101.
- Yin, X. and van Laar, H. H. (2005). Crop Systems Dynamics (GECROS).
