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
  30 °C, linear to 0 at 40 °C. It sets the pace of each organ's expansion clock
  (thermal time × multiplier), so cold delays expansion more than development but
  does not destroy it (decision 0011).
- Stages (daily step 1), one transition per day at most, never backward:
  Sown → Germinated (the seed module reports germination, §3.10) → Emerged (thermal time
  since germination ≥ the plant's own emergence threshold, which varies with seed and
  depth, decision 0027 and plant-variation.md) → FirstTrifoliate (architecture reports an unfolded
  trifoliate) → Flowering (thermal time since germination ≥ flowering) → PodSet (a pod
  has set) → SeedFill (a seed is filling) → Mature (thermal time since germination ≥
  maturity). All thresholds count from germination, so time a seed waits in dry or
  cold soil does not bring flowering forward (decision 0011).

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| t_base, t_opt, t_max | 8, 28, 38 | °C | "~" design plan §3.11 |
| emergence | 80 | °C·day | "~" design plan §3.10 (60 to 80), top of the range (WP12) |
| flowering | 440 | °C·day from germination | published sowing-to-flowering sums on base 8: about 350 (Dapaah 1999), 470 (CROPTIME 2020), 515 (Cavalcante 2020), 600 (Pinzón-Sandoval 2024); 440 is about 470 from sowing (decision 0018) |
| maturity | 900 | °C·day from germination | Cavalcante 2020 (827 above 10 °C) and Pinzón-Sandoval 2024 (about 800 above 8.3 °C) (decision 0018) |
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
`stages_advance_in_order`. In the WP11 report (default scenario, WP12): flowering on day
44 and maturity on day 83 (gated ranges 28 to 57 and 54 to 93); 370 °C·day above 10 °C
to flowering and 747 to maturity (Cavalcante 2020, 430 ± 15% and 700 to 950), 617 above
4.4 °C to flowering (CROPTIME 2020, 547 to 763). The papers disagree among themselves:
the sim's sums sit above Dapaah 1999 (base 7) and below Pinzón-Sandoval 2024 (base
8.3), which are reported as Info misses.

## References
- Fernández, F., Gepts, P. and López, M. (1986). Etapas de desarrollo de la planta de
  frijol. CIAT.
- Yin, X. et al. (1995). A nonlinear model for crop development as a function of
  temperature. Agricultural and Forest Meteorology 77.
