//! Hourly weather: disaggregation of daily values, and the series the world reads.

use core::f64::consts::PI;

use ps_units::{Celsius, Degrees, Hours, JPerM2, KPa, MetersPerSecond, Millimeters, WPerM2};
use serde::{Deserialize, Serialize};

use crate::calendar::{DateHour, day_of_year};
use crate::presets::DailyWeather;
use crate::radiation::{
    cloud_fraction_from_kt, default_wind, erbs_diffuse_fraction, hourly_wind,
    saturation_vapor_pressure_kpa,
};
use crate::solar::{
    SunPosition, daily_extraterrestrial_j_m2, day_light, extraterrestrial_horizontal_j_m2,
    sun_position,
};

/// Clock hour of the daily maximum temperature.
pub const T_MAX_HOUR: f64 = 14.0;
/// Clock hours over which daily rain falls (an afternoon shower), start inclusive.
pub const RAIN_HOURS: [u32; 3] = [15, 16, 17];

/// Weather for one sim hour (design plan §3.2 hourly step 1, plus step 2's sun).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HourWeather {
    /// Air temperature at the hour's midpoint.
    pub t_air: Celsius,
    /// Relative humidity, percent.
    pub rh_pct: f64,
    /// Vapor pressure deficit.
    pub vpd: KPa,
    /// Mean global horizontal irradiance over the hour.
    pub global: WPerM2,
    /// Mean direct (beam) horizontal irradiance over the hour.
    pub direct: WPerM2,
    /// Mean diffuse horizontal irradiance over the hour.
    pub diffuse: WPerM2,
    /// Rain in the hour.
    pub rain: Millimeters,
    /// Cloud fraction, 0 to 1: the sky's longwave emissivity for leaf temperature
    /// (WP27), and sky rendering.
    pub cloud_fraction: f64,
    /// Mean wind speed over the hour at 2 m (WP27). Older files without it read the
    /// FAO-56 default, 2 m s⁻¹.
    #[serde(default = "default_wind")]
    pub wind: MetersPerSecond,
    /// Sun position at the hour's midpoint.
    pub sun: SunPosition,
}

/// An hourly weather series starting at 00:00 of the scenario's first day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherSeries {
    /// Latitude of the site.
    pub latitude: Degrees,
    /// Scenario start; tick 0 is this hour.
    pub start: DateHour,
    /// Hourly records from 00:00 of the start date.
    pub hours: Vec<HourWeather>,
    /// Daily records, when the series was built from daily values (empty for hourly files).
    pub days: Vec<DailyWeather>,
}

impl WeatherSeries {
    /// Weather for a sim tick, or `None` past the end of the series.
    #[must_use]
    pub fn at(&self, tick: Hours) -> Option<&HourWeather> {
        let i = usize::try_from(tick.0 + u64::from(self.start.hour)).ok()?;
        self.hours.get(i)
    }

    /// Number of ticks available from the start.
    #[must_use]
    pub fn len_ticks(&self) -> u64 {
        (self.hours.len() as u64).saturating_sub(u64::from(self.start.hour))
    }

    /// Builds a series from daily values (presets or daily files).
    #[must_use]
    pub fn from_daily(latitude: Degrees, start: DateHour, days: Vec<DailyWeather>) -> Self {
        let hours = disaggregate(latitude, &days);
        Self {
            latitude,
            start,
            hours,
            days,
        }
    }
}

/// Air temperature at fractional clock hour `t` of day `i`, from a piecewise cosine
/// through `T_min` at sunrise and `T_max` at 14:00 (after Goudriaan and van Laar 1994,
/// simplified). Neighbor days at the ends of the series repeat the end day.
fn temperature_at(days: &[DailyWeather], sunrise: &[f64], i: usize, t: f64) -> f64 {
    let d = &days[i];
    let sr = sunrise[i];
    if t < sr {
        let p = if i == 0 { 0 } else { i - 1 };
        let t_prev_max = days[p].t_max.0;
        // From 14:00 of the previous day (t = T_MAX_HOUR - 24) to sunrise today.
        let span = sr - (T_MAX_HOUR - 24.0);
        let x = (t - (T_MAX_HOUR - 24.0)) / span;
        d.t_min.0 + (t_prev_max - d.t_min.0) * 0.5 * (1.0 + libm::cos(PI * x))
    } else if t <= T_MAX_HOUR {
        let x = (t - sr) / (T_MAX_HOUR - sr);
        d.t_min.0 + (d.t_max.0 - d.t_min.0) * 0.5 * (1.0 - libm::cos(PI * x))
    } else {
        let n = (i + 1).min(days.len() - 1);
        let next_min = days[n].t_min.0;
        let span = sunrise[n] + 24.0 - T_MAX_HOUR;
        let x = (t - T_MAX_HOUR) / span;
        next_min + (d.t_max.0 - next_min) * 0.5 * (1.0 + libm::cos(PI * x))
    }
}

/// Disaggregates daily values into 24 hourly records per day.
///
/// - Radiation: each hour gets the daily total in proportion to its extraterrestrial
///   radiation, so the hourly values sum exactly to the daily total. The hourly
///   clearness index equals the daily one; Erbs splits direct and diffuse.
/// - Temperature: piecewise cosine, minimum at sunrise, maximum at 14:00.
/// - Humidity: the day's vapor pressure is `rh_mean × e_s(T_mean)`; hourly RH and VPD
///   follow from the hourly temperature, with RH capped at 100%.
/// - Rain: spread evenly over [`RAIN_HOURS`].
/// - Wind: the day's mean spread over the hours by [`hourly_wind`] (DSSAT HWIND), calm
///   at night and windiest at solar noon, keeping the daily mean.
#[must_use]
pub fn disaggregate(latitude: Degrees, days: &[DailyWeather]) -> Vec<HourWeather> {
    if days.is_empty() {
        return Vec::new();
    }
    let sunrise: Vec<f64> = days
        .iter()
        .map(|d| {
            let dl = day_light(latitude, d.year, day_of_year(d.year, d.month, d.day));
            // Polar day or night: anchor the minimum at 06:00.
            if dl.day_length_h <= 0.0 || dl.day_length_h >= 24.0 {
                6.0
            } else {
                dl.sunrise_h
            }
        })
        .collect();
    let mut out = Vec::with_capacity(days.len() * 24);
    for (i, d) in days.iter().enumerate() {
        let doy = day_of_year(d.year, d.month, d.day);
        let dl = day_light(latitude, d.year, doy);
        let (rise, dayl) = if dl.day_length_h >= 24.0 {
            (0.0, 24.0)
        } else if dl.day_length_h <= 0.0 {
            (0.0, 0.0)
        } else {
            (dl.sunrise_h, dl.day_length_h)
        };
        let h0_day = daily_extraterrestrial_j_m2(latitude, d.year, doy);
        let kt = if h0_day > 0.0 {
            d.global.0 / h0_day
        } else {
            0.0
        };
        let diffuse_fraction = erbs_diffuse_fraction(kt);
        let t_mean = 0.5 * (d.t_min.0 + d.t_max.0);
        let ea = (d.rh_mean_pct / 100.0) * saturation_vapor_pressure_kpa(t_mean);
        let rain_per_hour = d.rain.0 / 3.0;
        for h in 0..24u32 {
            let mid = f64::from(h) + 0.5;
            let t = temperature_at(days, &sunrise, i, mid);
            let es = saturation_vapor_pressure_kpa(t);
            let e = ea.min(es);
            let h0_h = extraterrestrial_horizontal_j_m2(
                latitude,
                d.year,
                doy,
                f64::from(h),
                f64::from(h + 1),
            );
            let global_j = if h0_day > 0.0 {
                d.global.0 * h0_h / h0_day
            } else {
                0.0
            };
            let g = global_j / 3600.0;
            out.push(HourWeather {
                t_air: Celsius(t),
                rh_pct: 100.0 * e / es,
                vpd: KPa(es - e),
                global: WPerM2(g),
                direct: WPerM2(g * (1.0 - diffuse_fraction)),
                diffuse: WPerM2(g * diffuse_fraction),
                rain: Millimeters(if RAIN_HOURS.contains(&h) {
                    rain_per_hour
                } else {
                    0.0
                }),
                cloud_fraction: cloud_fraction_from_kt(kt),
                wind: MetersPerSecond(hourly_wind(
                    d.wind.0,
                    rise,
                    dayl,
                    f64::from(h),
                    f64::from(h + 1),
                )),
                sun: sun_position(latitude, d.year, doy, mid),
            });
        }
    }
    out
}

/// Sum of hourly global radiation for day `day_index` of a series, J m⁻².
#[must_use]
pub fn daily_global_total(hours: &[HourWeather], day_index: usize) -> JPerM2 {
    JPerM2(
        hours[day_index * 24..day_index * 24 + 24]
            .iter()
            .map(|h| h.global.0 * 3600.0)
            .fold(0.0, |a, b| a + b),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::{Preset, generate_daily};

    fn series(p: Preset) -> WeatherSeries {
        let start = DateHour::new(2000, 5, 15, 6).unwrap();
        let days = generate_daily(&p.params(), Degrees(45.0), start, 90, 42);
        WeatherSeries::from_daily(Degrees(45.0), start, days)
    }

    /// Done-when 2: preset daily radiation totals match after disaggregation.
    #[test]
    fn preset_daily_radiation_totals_match() {
        for p in Preset::ALL {
            let s = series(p);
            for (i, d) in s.days.iter().enumerate() {
                let sum = daily_global_total(&s.hours, i);
                let rel = (sum.0 - d.global.0).abs() / d.global.0.max(1.0);
                assert!(
                    rel < 1e-12,
                    "{} day {i}: {} vs {}",
                    p.name(),
                    sum.0,
                    d.global.0
                );
            }
        }
    }

    /// Preset climates are plausible for 45° N, mid-May to mid-August.
    #[test]
    fn preset_radiation_climatology() {
        let mean_mj = |p| {
            let s = series(p);
            s.days.iter().map(|d| d.global.0).sum::<f64>()
                / f64::from(u32::try_from(s.days.len()).unwrap())
                / 1e6
        };
        // Reference: 45° N summer global radiation is about 18 to 24 MJ m-2 day-1
        // (FAO-56 Ra ~ 40 to 42 times typical Kt 0.45 to 0.6).
        let ts = mean_mj(Preset::TemperateSummer);
        assert!((17.0..24.0).contains(&ts), "temperate summer {ts}");
        let oc = mean_mj(Preset::Overcast);
        assert!((6.0..13.0).contains(&oc), "overcast {oc}");
        let dr = mean_mj(Preset::Drought);
        assert!((22.0..30.0).contains(&dr), "drought {dr}");
    }

    #[test]
    fn hourly_temperature_hits_daily_extremes() {
        let s = series(Preset::TemperateSummer);
        for (i, d) in s.days.iter().enumerate().skip(1).take(80) {
            let day = &s.hours[i * 24..i * 24 + 24];
            let hi = day.iter().map(|h| h.t_air.0).fold(f64::MIN, f64::max);
            let lo = day.iter().map(|h| h.t_air.0).fold(f64::MAX, f64::min);
            let neighbors_min = d
                .t_min
                .0
                .min(s.days[i - 1].t_min.0)
                .min(s.days[i + 1].t_min.0);
            let neighbors_max = d.t_max.0.max(s.days[i - 1].t_max.0);
            assert!(
                hi <= neighbors_max + 1e-9 && hi > d.t_max.0 - 0.5,
                "day {i}: hi {hi} {d:?}"
            );
            assert!(lo >= neighbors_min - 1e-9, "day {i}: lo {lo} {d:?}");
            assert!(lo < d.t_min.0 + 1.0, "day {i}: lo {lo} {d:?}");
        }
    }

    #[test]
    fn night_is_dark_and_sun_is_down() {
        let s = series(Preset::TemperateSummer);
        let midnight = &s.hours[24 * 10];
        assert!(midnight.global.0.abs() < 1e-12);
        assert!(!midnight.sun.is_up());
        let noon = &s.hours[24 * 10 + 12];
        assert!(noon.global.0 > 300.0 && noon.sun.is_up());
        assert!((noon.direct.0 + noon.diffuse.0 - noon.global.0).abs() < 1e-9);
    }

    #[test]
    fn humidity_bounds() {
        for p in Preset::ALL {
            for h in &series(p).hours {
                assert!(h.rh_pct > 0.0 && h.rh_pct <= 100.0 + 1e-9);
                assert!(h.vpd.0 >= 0.0);
            }
        }
    }

    #[test]
    fn rain_conserved() {
        let s = series(Preset::Overcast);
        let daily: f64 = s.days.iter().map(|d| d.rain.0).sum();
        let hourly: f64 = s.hours.iter().map(|h| h.rain.0).sum();
        assert!((daily - hourly).abs() < 1e-9 * daily.max(1.0));
        assert!(daily > 0.0);
    }

    #[test]
    fn tick_zero_is_start_hour() {
        let s = series(Preset::TemperateSummer);
        assert_eq!(s.at(Hours(0)), Some(&s.hours[6]));
        assert_eq!(s.len_ticks(), 90 * 24 - 6);
    }
}
