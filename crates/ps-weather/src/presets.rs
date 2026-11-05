//! Daily weather presets.
//!
//! A preset is a small stochastic weather generator with fixed parameters. Draws go
//! through ps-rng keyed by (world seed, weather entity, day, `Purpose::Weather`), so a
//! preset gives the same weather for the same seed, on any platform.
//!
//! All parameters are "~" (plausible temperate values to be checked); see the model
//! card docs/models/weather-and-sun.md.

use ps_rng::{Purpose, RngKey};
use ps_units::{Celsius, Degrees, JPerM2, Millimeters};
use serde::{Deserialize, Serialize};

use crate::calendar::{DateHour, civil_from_days, day_of_year, days_from_civil};
use crate::solar::daily_extraterrestrial_j_m2;

/// Entity id used for weather draws. Distinct from every plant and organ id.
pub const WEATHER_ENTITY: u64 = 0xFFFF_0000_0000_0001;

/// One day of weather, the unit presets produce and daily files contain.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DailyWeather {
    /// Calendar year.
    pub year: i32,
    /// Month 1 to 12.
    pub month: u32,
    /// Day of month.
    pub day: u32,
    /// Minimum air temperature, reached near sunrise.
    pub t_min: Celsius,
    /// Maximum air temperature, reached at 14:00.
    pub t_max: Celsius,
    /// Daily mean relative humidity, percent. Sets the day's vapor pressure.
    pub rh_mean_pct: f64,
    /// Global horizontal radiation, daily total.
    pub global: JPerM2,
    /// Rain, daily total.
    pub rain: Millimeters,
}

/// Named presets (design plan §2.4, WP2).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    /// A temperate summer with a seasonal temperature cycle and occasional rain.
    TemperateSummer,
    /// Dull, cool, humid and showery.
    Overcast,
    /// Temperate summer with a hot, dry spell (default days 25 to 31 after the start).
    HeatWave,
    /// Warm, bright, dry, no rain.
    Drought,
    /// Temperate summer with one frost night (default 20 days after the start).
    FrostNight,
}

impl Preset {
    /// Parses the snake-case name used in scenario files.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "temperate_summer" => Self::TemperateSummer,
            "overcast" => Self::Overcast,
            "heat_wave" => Self::HeatWave,
            "drought" => Self::Drought,
            "frost_night" => Self::FrostNight,
            _ => return None,
        })
    }

    /// The snake-case name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TemperateSummer => "temperate_summer",
            Self::Overcast => "overcast",
            Self::HeatWave => "heat_wave",
            Self::Drought => "drought",
            Self::FrostNight => "frost_night",
        }
    }

    /// All presets, in a fixed order.
    pub const ALL: [Preset; 5] = [
        Self::TemperateSummer,
        Self::Overcast,
        Self::HeatWave,
        Self::Drought,
        Self::FrostNight,
    ];

    /// The generator parameters for this preset.
    #[must_use]
    pub fn params(self) -> PresetParams {
        let base = PresetParams::temperate_summer();
        match self {
            Self::TemperateSummer => base,
            Self::Overcast => PresetParams {
                diurnal_range_c: 6.0,
                kt_min: 0.18,
                kt_max: 0.30,
                rain_probability: 0.35,
                rh_mean_pct: 82.0,
                ..base
            },
            Self::HeatWave => PresetParams {
                heat_wave: Some(HeatWave {
                    start_day: 25,
                    days: 7,
                    t_max_c: 35.0,
                    t_min_c: 21.0,
                }),
                ..base
            },
            Self::Drought => PresetParams {
                annual_mean_c: base.annual_mean_c + 2.0,
                kt_min: 0.55,
                kt_max: 0.72,
                rain_probability: 0.0,
                rh_mean_pct: 45.0,
                ..base
            },
            Self::FrostNight => PresetParams {
                frost_night: Some(FrostNight {
                    day: 20,
                    t_min_c: -3.0,
                }),
                ..base
            },
        }
    }
}

/// A hot spell inside a preset, in days after the start date.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeatWave {
    /// First day of the spell (0 = the start date).
    pub start_day: u32,
    /// Length in days.
    pub days: u32,
    /// Daily maximum during the spell.
    pub t_max_c: f64,
    /// Daily minimum during the spell.
    pub t_min_c: f64,
}

/// A single frost night inside a preset.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrostNight {
    /// Day whose minimum (near sunrise) is the frost (0 = the start date).
    pub day: u32,
    /// The frost minimum.
    pub t_min_c: f64,
}

/// Parameters of the daily generator. Units are in the field names.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresetParams {
    /// Annual mean of the daily mean temperature.
    pub annual_mean_c: f64,
    /// Seasonal amplitude of the daily mean temperature.
    pub seasonal_amplitude_c: f64,
    /// Day of year of the warmest mean (northern hemisphere; shifted by half a year south).
    pub warmest_doy: f64,
    /// Daily maximum minus minimum on a dry day.
    pub diurnal_range_c: f64,
    /// Standard deviation of the day-to-day temperature anomaly innovation.
    pub anomaly_sd_c: f64,
    /// Lag-one autocorrelation of the temperature anomaly.
    pub anomaly_ar1: f64,
    /// Lower bound of the daily clearness index on dry days.
    pub kt_min: f64,
    /// Upper bound of the daily clearness index on dry days.
    pub kt_max: f64,
    /// Clearness index on rainy days.
    pub kt_rain: f64,
    /// Probability of rain on a day.
    pub rain_probability: f64,
    /// Mean rain on a rainy day (exponential distribution), mm.
    pub rain_mean_mm: f64,
    /// Daily mean relative humidity on dry days, percent.
    pub rh_mean_pct: f64,
    /// Added relative humidity on rainy days, percent (capped at 95).
    pub rh_rain_extra_pct: f64,
    /// Reduction of the diurnal range on rainy days, as a fraction.
    pub rain_range_factor: f64,
    /// Optional hot spell.
    pub heat_wave: Option<HeatWave>,
    /// Optional frost night.
    pub frost_night: Option<FrostNight>,
}

impl PresetParams {
    /// The temperate summer baseline (about 45° N, oceanic to continental margin).
    #[must_use]
    pub const fn temperate_summer() -> Self {
        Self {
            annual_mean_c: 11.0,
            seasonal_amplitude_c: 10.0,
            warmest_doy: 200.0,
            diurnal_range_c: 11.0,
            anomaly_sd_c: 1.5,
            anomaly_ar1: 0.6,
            kt_min: 0.40,
            kt_max: 0.68,
            kt_rain: 0.30,
            rain_probability: 0.15,
            rain_mean_mm: 6.0,
            rh_mean_pct: 65.0,
            rh_rain_extra_pct: 15.0,
            rain_range_factor: 0.6,
            heat_wave: None,
            frost_night: None,
        }
    }
}

/// Generates `days` days of weather starting on `start`'s date.
#[must_use]
pub fn generate_daily(
    params: &PresetParams,
    latitude: Degrees,
    start: DateHour,
    days: u32,
    world_seed: u64,
) -> Vec<DailyWeather> {
    let first = days_from_civil(start.year, start.month, start.day);
    let mut out = Vec::with_capacity(days as usize);
    let mut anomaly = 0.0;
    let phase_shift = if latitude.0 < 0.0 { 182.5 } else { 0.0 };
    for d in 0..days {
        let (year, month, day) = civil_from_days(first + i64::from(d));
        let doy = day_of_year(year, month, day);
        let mut s = RngKey::new(
            world_seed,
            WEATHER_ENTITY,
            u64::from(d) * 24,
            Purpose::Weather,
        )
        .stream();
        // Fixed draw order: anomaly, rain flag, rain amount, clearness.
        let z = s.standard_normal();
        let rain_u = s.uniform();
        let amount_u = s.uniform();
        let kt_u = s.uniform();

        anomaly = params.anomaly_ar1 * anomaly + params.anomaly_sd_c * z;
        let season = libm::cos(
            2.0 * core::f64::consts::PI * (f64::from(doy) - params.warmest_doy - phase_shift)
                / 365.25,
        );
        let mean = params.annual_mean_c + params.seasonal_amplitude_c * season + anomaly;

        let raining = rain_u < params.rain_probability;
        let rain_mm = if raining {
            -params.rain_mean_mm * libm::log(1.0 - amount_u)
        } else {
            0.0
        };
        let range = if raining {
            params.diurnal_range_c * params.rain_range_factor
        } else {
            params.diurnal_range_c
        };
        let mut t_min = mean - 0.5 * range;
        let mut t_max = mean + 0.5 * range;
        let mut kt = if raining {
            params.kt_rain
        } else {
            params.kt_min + (params.kt_max - params.kt_min) * kt_u
        };
        let mut rh = if raining {
            (params.rh_mean_pct + params.rh_rain_extra_pct).min(95.0)
        } else {
            params.rh_mean_pct
        };
        let mut rain = rain_mm;

        if let Some(hw) = params.heat_wave
            && d >= hw.start_day
            && d < hw.start_day + hw.days
        {
            t_max = hw.t_max_c;
            t_min = hw.t_min_c;
            kt = params.kt_max;
            rh = rh.min(45.0);
            rain = 0.0;
        }
        if let Some(fr) = params.frost_night
            && d == fr.day
        {
            t_min = fr.t_min_c;
            kt = params.kt_max; // clear nights are frost nights
            rain = 0.0;
            rh = params.rh_mean_pct;
        }

        let h0 = daily_extraterrestrial_j_m2(latitude, year, doy);
        out.push(DailyWeather {
            year,
            month,
            day,
            t_min: Celsius(t_min),
            t_max: Celsius(t_max.max(t_min)),
            rh_mean_pct: rh,
            global: JPerM2(kt * h0),
            rain: Millimeters(rain),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start() -> DateHour {
        DateHour::new(2000, 5, 15, 6).unwrap()
    }

    #[test]
    fn names_round_trip() {
        for p in Preset::ALL {
            assert_eq!(Preset::from_name(p.name()), Some(p));
        }
        assert_eq!(Preset::from_name("monsoon"), None);
    }

    #[test]
    fn deterministic_for_seed() {
        let p = Preset::TemperateSummer.params();
        let a = generate_daily(&p, Degrees(45.0), start(), 90, 42);
        let b = generate_daily(&p, Degrees(45.0), start(), 90, 42);
        let c = generate_daily(&p, Degrees(45.0), start(), 90, 43);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn drought_has_no_rain() {
        let w = generate_daily(&Preset::Drought.params(), Degrees(45.0), start(), 90, 1);
        assert!(w.iter().all(|d| d.rain.0 == 0.0));
    }

    #[test]
    fn heat_wave_window() {
        let w = generate_daily(&Preset::HeatWave.params(), Degrees(45.0), start(), 40, 1);
        for (i, d) in w.iter().enumerate() {
            if (25..32).contains(&i) {
                assert!((d.t_max.0 - 35.0).abs() < 1e-12);
            } else {
                assert!(d.t_max.0 < 34.0, "day {i}: {}", d.t_max.0);
            }
        }
    }

    #[test]
    fn frost_night_is_below_leaf_kill() {
        let w = generate_daily(&Preset::FrostNight.params(), Degrees(45.0), start(), 30, 1);
        assert!(w[20].t_min.0 < -1.0);
        assert!(
            w.iter()
                .enumerate()
                .all(|(i, d)| i == 20 || d.t_min.0 > -1.0)
        );
    }
}
