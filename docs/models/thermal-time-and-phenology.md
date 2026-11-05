# Thermal time and phenology

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Plants keep time by warmth, not by the calendar. Each warm hour adds "degree-days", and
the plant moves through its stages (emergence, first true leaf, flowering, pods, seed
fill, maturity) when enough have added up. A warm spring makes beans flower sooner.

## Equations
- Development rate (§3.11, §11.4), a trapezoid with base `Tb`, plateau `To` to `To2`
  and maximum `Tm` (decision 0032): `f(T) = 0` for `T ≤ Tb` or `T ≥ Tm`; `T − Tb` up to
  `To`; `To − Tb` from `To` to `To2`; `(To − Tb)(Tm − T)/(Tm − To2)` above. With
  `To2 = To` it is the old three-point response (wheat). Hourly thermal time is
  `f(T)/24` °C·day, so the rate on the plateau is `To − Tb` and sums compare with
  literature °C·day below the optimum.
- Organ thermal age accumulates the same way at the organ's temperature.
- Expansion temperature multiplier: 0 at or below 5 °C, linear to 1 at 25 °C, 1 to
  30 °C, linear to 0 at 40 °C. It sets the pace of each organ's expansion clock
  (thermal time × multiplier), so cold delays expansion more than development but
  does not destroy it (decision 0011).
- Stages (daily step 1), one transition per day at most, never backward:
  Sown → Germinated (the seed module reports germination, §3.10: 24 hours, not
  necessarily in a row, with the seed's cell at or above `germination_ftsw` and warmer
  than the base temperature; a dry or cold hour pauses the count, decision 0029) → Emerged (thermal time
  since germination ≥ the plant's own emergence threshold, which varies with seed and
  depth, decision 0027 and plant-variation.md) → FirstTrifoliate (architecture reports an unfolded
  trifoliate) → Flowering (thermal time since germination ≥ flowering) → PodSet (a pod
  has set) → SeedFill (a seed is filling) → Mature (thermal time since germination ≥
  maturity). All thresholds count from germination, so time a seed waits in dry or
  cold soil does not bring flowering forward (decision 0011).

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| t_base, t_opt, t_opt_high, t_max (bean) | 7, 25, 33, 42 | °C | "~" synthesis of CROPGRO-Drybean (5/27/37/45 vegetative), Ferreira 1997 (6.8/24.0/34.1), APSIM bean (7.5/30/40) (decision 0032; was 8, 28, 38) |
| t_base, t_opt = t_opt_high, t_max (wheat) | 0, 26, 34 | °C | APSIM-Wheat |
| emergence (bean) | 90 | °C·day | 80 on base 8 (design plan §3.10, top of 60 to 80), restated so the default scenario emerges on the same day (decision 0032) |
| flowering (bean) | 480 | °C·day from germination | 440 on base 8 from published sowing-to-flowering sums (decision 0018), restated (×1.09) to keep the flowering day (decision 0032) |
| maturity (bean) | 965 | °C·day from germination | 900 on base 8 (Cavalcante 2020; Pinzón-Sandoval 2024; decision 0018), restated (×1.07) (decision 0032) |
| flowering, maturity (pole bean) | 613, 1232 | °C·day | 575, 1150 restated (×1.066, ×1.071) (decision 0032) |
| organ clocks (bean) | ×1.08 vegetative, ×1.06 reproductive | - | phyllochron 59.5, leaf and internode expansion 162 and 130, nodule 108, root emergences 54/81/130, emergence per depth 13 Cd/cm, leaf lifespan 650; pod expansion 154, seed fill delay 74, seed fill 350 (decision 0032) |
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
- A sown seed never loses germination progress when its cell dries (decision 0029),
  and it does not die in dry soil, so a seed hovering near the moisture threshold
  eventually germinates (review of decision 0029).

## Validation
`ps_physio::tests::three_point_thermal_time`, `trapezoid_thermal_time` (decision 0032), `growth_temperature_trapezoid`,
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
