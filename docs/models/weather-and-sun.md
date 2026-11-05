# Weather and sun

## Purpose
Plants respond to light, warmth, humidity and rain hour by hour. This process works out
where the sun is, how much sunlight reaches the garden directly and how much comes from
the whole sky, and turns a day's weather (lows, highs, sunshine, rain) into a believable
hour-by-hour course: cool at dawn, warmest mid-afternoon, showers in the afternoon.

## Equations
**Sun.** Low-precision solar coordinates (Meeus 1998, ch. 25; the NOAA calculator):
from Julian centuries `T` since J2000, the sun's mean longitude `L0`, mean anomaly `M`,
equation of center `C`, apparent longitude `λ` and obliquity `ε` give declination
`δ = asin(sin ε sin λ)`, the equation of time `EoT`, and Earth–sun distance `R`.
Hour angle `ω = (60·h + EoT)/4 − 180` degrees on the local mean solar clock
(decision 0005). Then

- `sin α = sin φ sin δ + cos φ cos δ cos ω` (elevation α, latitude φ, no refraction),
- azimuth from north `= atan2(sin ω, cos ω sin φ − tan δ cos φ) + 180°`.

**Day length.** Sunrise hour angle `ω_s` from `cos ω_s = (cos 90.833° − sin φ sin δ)/(cos φ cos δ)`;
day length `= 2 ω_s / 15°` hours.

**Extraterrestrial radiation** on a horizontal surface over clock hours `[h₀, h₁]`
(Duffie and Beckman eq. 1.10.4): `H₀ = (12·3600/π) G_sc (1/R²) [(ω₂ − ω₁) sin φ sin δ + cos φ cos δ (sin ω₂ − sin ω₁)]`,
with `ω` clipped to `[−ω_s, ω_s]` at the geometric horizon. Daily `H₀` is the sum of the 24 hours.

**Clearness and split.** Daily global radiation `H = K_t · H₀,day`. Each hour gets
`H · H₀,h / H₀,day`, so hours sum exactly to the day. Diffuse fraction from the hourly
clearness index by Erbs et al. (1982); direct = global − diffuse.

**Temperature.** Piecewise cosine: `T_min` at sunrise, `T_max` at 14:00, falling after
14:00 to the next day's `T_min` at the next sunrise.

**Humidity.** Vapor pressure for the day `e_a = RH_mean · e_s(T_mean)` (Tetens, FAO-56
eq. 11); hourly `RH = min(e_a, e_s(T))/e_s(T)`, `VPD = e_s(T) − e`.

**Hourly files.** Global radiation is read as measured; the split uses the hourly
clearness index `G / G₀`, except that hours with `G₀` below 10 W m⁻² (twilight) are
treated as all diffuse. Files must start at 00:00 of the start date and have no gaps;
daily files must have consecutive dates from the start date.

**Rain.** Daily total spread evenly over 15:00 to 18:00.

**Cloud fraction.** `c = clamp((0.75 − K_t)/0.55, 0, 1)` from the day's clearness, the
same for every hour of a daily or preset day. Hourly files take the `cloud` column when
there is one, else the hour's clearness; night and twilight hours keep the last daylight
hour's value (1 before the first). Read by the renderer and, since WP27, by the leaf
energy balance for the sky's longwave.

**Wind** (WP27, decision 0034). Wind at 2 m, m s⁻¹. Hourly files take an optional
`wind_m_s` column (0 to 75). Daily files take an optional daily mean `wind_m_s`; presets
use 2 m s⁻¹ (FAO-56's default when wind is missing). A daily mean `W` is spread over the
day by DSSAT's HWIND: `W_max = W/(0.3 + 0.7 · DL/48)` for day length `DL`; `0.3 W_max`
from sunset to sunrise; by day `W_max (0.3 + 0.35 (1 + sin(2π (t − t_rise)/DL + 1.5π)))`,
peaking at solar noon, integrated over each hour, so the 24 hours average to `W`
exactly. Polar day and night are handled as day length 24 and 0.

**Presets.** Daily mean temperature `= T_annual + A cos(2π(doy − doy_warm)/365.25) + a_d`,
with an AR(1) anomaly `a_d = ρ a_{d−1} + σ z`. Rain occurs with probability `p`, amount
exponential with mean `μ`. Clearness is uniform in `[K_t,min, K_t,max]` on dry days.
Draws use ps-rng keyed by (world seed, weather entity, day × 24, Weather).

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Solar constant G_sc | 1361 | W/m2 | Kopp and Lean 2011 |
| Sunrise zenith | 90.833 | deg | NOAA (refraction + solar radius) |
| Hour of T_max | 14 | h | ~ (common crop-model choice) |
| Rain hours | 15 to 17 | h | ~ |
| Temperate summer: T_annual, A, doy_warm | 11, 10, 200 | C, C, - | ~ (about 45° N) |
| Diurnal range (dry, rainy ×0.6) | 11 | C | ~ |
| Anomaly σ, ρ | 1.5, 0.6 | C, - | ~ |
| K_t dry range, rainy | 0.40 to 0.68, 0.30 | - | ~ |
| Rain probability, mean | 0.15, 6 | -, mm | ~ |
| RH mean dry, rainy extra | 65, +15 | % | ~ |
| Overcast: range, K_t, rain p, RH | 6, 0.18 to 0.30, 0.35, 82 | C, -, -, % | ~ |
| Heat wave: days 25 to 31, T_max, T_min | 35, 21 | C | ~ (Gross 1994 uses 32 °C days) |
| Drought: +2 C, K_t 0.55 to 0.72, no rain, RH 45 | | | ~ |
| Frost night: day 20, T_min | −3 | C | ~ (below the −1 °C leaf kill) |
| Default wind (presets, files without wind) | 2 | m/s at 2 m | FAO-56 |
| HWIND night share of W_max | 0.3 | - | DSSAT `HMET.for` |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | latitude | deg | scenario |
| in | start date and hour | - | scenario |
| in | preset or weather file | - | scenario |
| out | air temperature | C | soil, physiology, phenology |
| out | RH, VPD | %, kPa | gas exchange |
| out | global, direct, diffuse irradiance | W/m2 | light |
| out | rain | mm | soil |
| out | sun elevation and azimuth | deg | light, renderer |
| out | cloud fraction | - | renderer, leaf energy balance (sky longwave) |
| out | wind at 2 m | m/s | leaf energy balance |

## Encoded or emergent
All encoded. Nothing here responds to plants.

## Known limits
- No refraction in elevation (it matters only within a degree of the horizon).
- No longitude or time zone (decision 0005).
- Hourly clearness equals the day's, so a day is uniformly cloudy; no passing clouds.
- Vapor pressure is constant within a day and jumps at midnight.
- Wind has no direction, gusts or day-to-day weather in presets (a fixed 2 m s⁻¹ daily
  mean); its daily course is DSSAT's fixed shape.

## Validation
- `ps-weather/tests/solar_reference.rs::sun_position_matches_spa_within_half_a_degree`: 720 hourly
  positions, 10 dates × 3 latitudes (0°, 45°, 60°) vs pvlib NREL SPA; tolerance 0.5°, worst 0.014°.
- `...::sunrise_and_sunset_match_spa_within_three_minutes`: 30 dates and latitudes.
- `ps_weather::hourly::tests::preset_daily_radiation_totals_match`: hourly sums equal daily totals to 1e-12.
- `ps_weather::hourly::tests::preset_radiation_climatology`: preset means in plausible 45° N ranges.
- `ps_weather::solar::tests::daily_extraterrestrial_reference_values`: FAO-56 eq. 21 within 2%.
- `ps_weather::radiation::tests::tetens_reference`: FAO-56 Table 2.3.
- `ps_weather::radiation::tests::hourly_wind_keeps_the_daily_mean`: 24 hourly means
  average to the daily mean at several day lengths, polar cases included.
- Fixture generator: `validation/fixtures/solar_pvlib.py` (pvlib 0.16.1).

## References
- Meeus, J. (1998). Astronomical Algorithms, 2nd ed., ch. 25.
- NOAA Global Monitoring Laboratory, Solar Calculation Details.
- Duffie, J. A., Beckman, W. A. (2013). Solar Engineering of Thermal Processes, 4th ed., §1.10.
- Erbs, D. G., Klein, S. A., Duffie, J. A. (1982). Solar Energy 28, 293–302.
- Allen, R. G., et al. (1998). FAO Irrigation and Drainage Paper 56.
- Kopp, G., Lean, J. L. (2011). Geophysical Research Letters 38, L01706.
- Reda, I., Andreas, A. (2004). Solar position algorithm (SPA), NREL (reference via pvlib).
