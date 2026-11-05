# Thermal time and phenology

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Plants keep time by warmth, not by the calendar. Each warm hour adds "degree-days", and
the plant moves through its stages (emergence, first true leaf, flowering, pods, seed
fill, maturity) when enough have added up. A warm spring makes beans flower sooner.

## Equations
- Development rate (§3.11, §11.4), with base `Tb`, optimum `To`, maximum `Tm`:
  `f(T) = 0` for `T ≤ Tb` or `T ≥ Tm`; `T − Tb` up to `To`; `(To − Tb)(Tm − T)/(Tm − To)`
  above. Hourly thermal time is `f(T)/24` °C·day, so the rate at the optimum is
  `To − Tb` and sums compare with literature °C·day below the optimum.
- Organ thermal age accumulates the same way at the organ's temperature.
- Expansion temperature multiplier: 0 at or below 5 °C, linear to 1 at 25 °C, 1 to
  30 °C, linear to 0 at 40 °C.
- Stages (daily step 1), one transition per day at most, never backward:
  Sown → Germinated (the seed module reports germination, §3.10) → Emerged (thermal time
  since germination ≥ emergence) → FirstTrifoliate (architecture reports an unfolded
  trifoliate) → Flowering (thermal time since sowing ≥ flowering) → PodSet (a pod has
  set) → SeedFill (a seed is filling) → Mature (thermal time since sowing ≥ maturity).

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| t_base, t_opt, t_max | 8, 28, 38 | °C | "~" design plan §3.11 |
| emergence | 70 | °C·day | "~" design plan §3.10 (60 to 80) |
| flowering | 350 | °C·day | "~" 40% of the cycle, design plan §6.1 |
| maturity | 850 | °C·day | Pinzón-Sandoval 2024 (design plan §8.2) |
| growth_t_min, opt_low, opt_high, max | 5, 25, 30, 40 | °C | "~" design plan §3.11 |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | hourly air and soil temperature | °C | ps-weather, ps-soil |
| in | daily organ events | - | architecture (WP8) |
| out | stage, thermal times, organ thermal ages | -, °C·day | architecture, reproduction, lenses |

## Encoded or emergent
Encoded: cardinal temperatures and thresholds. Emergent: the calendar date of each stage
under each weather preset; slower development in cold springs; heat stalling growth.

## Known limits
- Water stress does not change the pace of development yet (the plan's "and stress"
  is open, decision 0009).
- No photoperiod response (common bean is mostly day-neutral in temperate cultivars).

## Validation
`ps_physio::tests::three_point_thermal_time`, `growth_temperature_trapezoid`,
`stages_advance_in_order`. Calendar targets come with the WP11 report.

## References
- Fernández, F., Gepts, P. and López, M. (1986). Etapas de desarrollo de la planta de
  frijol. CIAT.
- Yin, X. et al. (1995). A nonlinear model for crop development as a function of
  temperature. Agricultural and Forest Meteorology 77.
