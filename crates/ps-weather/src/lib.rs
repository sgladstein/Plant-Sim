//! Weather and sun for the plant simulation (design plan §3.2 steps 1 and 2, WP2).
//!
//! - [`solar`]: sun position, day length, extraterrestrial radiation.
//! - [`radiation`]: direct and diffuse split (Erbs), humidity helpers.
//! - [`presets`]: daily weather generators for the named presets.
//! - [`hourly`]: daily-to-hourly disaggregation and the [`WeatherSeries`] the world reads.
//! - [`csv`]: hourly and daily weather file readers.
//!
//! Model card: docs/models/weather-and-sun.md.

pub mod calendar;
pub mod csv;
pub mod hourly;
pub mod presets;
pub mod radiation;
pub mod solar;

pub use calendar::DateHour;
pub use hourly::{HourWeather, WeatherSeries};
pub use presets::{DailyWeather, Preset, PresetParams};
pub use solar::SunPosition;

use ps_units::Degrees;

/// Builds the hourly series for a preset covering `days` days from `start`'s date.
#[must_use]
pub fn preset_series(
    preset: Preset,
    latitude: Degrees,
    start: DateHour,
    days: u32,
    world_seed: u64,
) -> WeatherSeries {
    // One extra day so ticks from a non-midnight start reach the end of the last day.
    let daily = presets::generate_daily(&preset.params(), latitude, start, days + 1, world_seed);
    WeatherSeries::from_daily(latitude, start, daily)
}
