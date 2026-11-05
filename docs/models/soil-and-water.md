# Soil and water

## Purpose
Rain and watering soak in from the top. Water beyond what soil can hold (its field
capacity) drains down and out of the pot's bottom. Water spreads sideways a little, the
sun dries the surface, and roots draw water from where they are. A plant feels thirsty
when the soil around its roots holds less than about 40% of the water it could use; that
stress narrows its stomata and slows leaf expansion.

## Equations
Per cell of volume `V` (cubic, edge `Δx`), water mass `W` (kg); capacities
`W_x = θ_x V ρ_w` for saturation `s`, field capacity `fc`, wilting point `wp`, air-dry `ad`.

1. **Infiltration**: water in (mm) × `Δx²` per active column; fill cells top-down to `W_fc`,
   then to `W_s`; leftover is runoff.
2. **Drainage**: `e_i = max(0, W_i − W_fc)` from the start-of-step state; `W_i −= e_i`,
   added to the cell below, or drained if there is none.
3. **Lateral**: for each horizontal neighbor pair, `f = 0.1 (W_i − W_j)`; `W_i −= f`, `W_j += f`.
4. **Evaporation** (top layer): `E = 0.5 · R_soil · 3600 / λ · Δx² · clamp((W − W_ad)/(W_fc − W_ad), 0, 1)`,
   at most `W − W_ad`; `λ = 2.45 MJ/kg`.
5. **Soil temperature** of layer at depth `z`: `T = T̄₂₄ + e^{−z/D} (T_air(t − 24z/(2πD)) − T̄₂₄)`.
6. **FTSW** over root-occupied cells: `Σ L_i (W_i − W_wp) / Σ L_i (W_fc − W_wp)` in [0, 1];
   1 with no roots.
7. **Stress factor**: `min(1, FTSW / 0.4)` (threshold is a species parameter).
8. **Uptake**: potential `P_i = min(1, k L_i / V) · (W_i − W_wp)⁺`; actual `U = min(demand, Σ P)`,
   from each cell in proportion to `P_i`; the shortfall is logged.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Field capacity loam, sand, clay | 0.27, 0.15, 0.40 | m3/m3 | design plan §3.6 |
| Wilting point loam, sand, clay | 0.12, 0.05, 0.22 | m3/m3 | design plan §3.6 |
| Saturation loam, sand, clay | 0.45, 0.40, 0.50 | m3/m3 | ~ (decision 0007) |
| Air-dry loam, sand, clay | 0.05, 0.02, 0.10 | m3/m3 | ~ (decision 0007) |
| Lateral fraction | 0.1 | per hour | design plan §3.6 |
| Evaporation coefficient | 0.5 | - | ~ (decision 0007) |
| Damping depth | 0.10 | m | ~ (typical moist loam) |
| Cell size, pots | 0.02 | m | design plan §3.6 |
| FTSW threshold | 0.4 | - | species file |
| Root uptake rate | 1e-5 | m2/h | species file, ~ |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | rain + irrigation | mm | weather, actions |
| in | radiation at the soil surface | W/m2 | light |
| in | air temperature | C | weather |
| in | root length per cell, water demand | m, kg | roots, gas exchange |
| out | FTSW, stress factor | - | gas exchange, growth |
| out | uptake, shortfall | kg | ledger |
| out | layer temperature | C | germination, roots |
| out | water ledger | kg | world ledger |

## Encoded or emergent
Encoded: all of the above. Emergent at plant level: drought stress timing, which
depends on how fast roots explore the pot and how much leaf area transpires.

## Known limits
No matric potential, no hysteresis, no capillary rise (§3.6). Drainage is a bucket
cascade, faster than real soils. Evaporation ignores humidity and wind.

## Validation
- `ps_soil::tests::wet_column_drains_to_field_capacity`: loam, sand, clay; closure < 1e-9 every hour;
  analytic outflow exact; drainage time within 5% (WP5 done-when).
- `ps_soil::tests::random_hours_conserve`: property test, random rain, radiation, temperature and uptake.
- `infiltration_fills_from_the_top`, `excess_runs_off_when_saturated`,
  `lateral_redistribution_smooths_and_conserves`, `evaporation_takes_from_top_only_and_stops_dry`,
  `ftsw_and_stress`, `uptake_is_proportional_and_limited`, `deeper_soil_is_damped_and_lagged`,
  `pot_mask_area_is_close_to_circle`.

## References
- Sinclair, T. R., Ludlow, M. M. (1986). Influence of soil water supply on the plant water balance of four tropical grain legumes. Aust. J. Plant Physiol. 13, 329–341 (FTSW).
- Ritchie, J. T. (1998). Soil water balance and plant water stress. In Understanding Options for Agricultural Production (bucket models).
- Allen, R. G., et al. (1998). FAO-56 (latent heat; Priestley–Taylor scale for the evaporation coefficient).
