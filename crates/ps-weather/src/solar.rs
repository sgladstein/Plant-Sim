//! Solar geometry: declination, equation of time, sun position, day length and
//! extraterrestrial radiation (Meeus 1998 low-precision solar coordinates, as used by
//! the NOAA solar calculator; hourly integrals after Duffie and Beckman, eq. 1.10.4).
//!
//! Clock convention: local mean solar time (docs/decisions/0005). Elevation is the true
//! (geometric) elevation without refraction; azimuth is degrees clockwise from north.

use core::f64::consts::PI;

use ps_units::{Degrees, Radians};
use serde::{Deserialize, Serialize};

use crate::calendar::days_from_civil;

/// Solar constant, W m⁻² (Kopp and Lean 2011).
pub const SOLAR_CONSTANT_W_M2: f64 = 1361.0;

/// Zenith angle at which sunrise and sunset are defined, degrees (center of the disc
/// with standard refraction and the solar radius, as in NOAA).
pub const SUNRISE_ZENITH_DEG: f64 = 90.833;

/// The sun's direction.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SunPosition {
    /// Elevation above the horizon (negative below).
    pub elevation: Degrees,
    /// Azimuth clockwise from north.
    pub azimuth: Degrees,
}

impl SunPosition {
    /// Unit vector toward the sun in (east, north, up) coordinates.
    #[must_use]
    pub fn direction(self) -> [f64; 3] {
        let e = self.elevation.to_radians().0;
        let a = self.azimuth.to_radians().0;
        let ce = libm::cos(e);
        [ce * libm::sin(a), ce * libm::cos(a), libm::sin(e)]
    }

    /// True when the sun's center is above the geometric horizon.
    #[must_use]
    pub fn is_up(self) -> bool {
        self.elevation.0 > 0.0
    }
}

/// Declination, equation of time and Earth-sun distance factor for an instant.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SolarCoords {
    /// Solar declination.
    pub declination: Radians,
    /// Equation of time, minutes (apparent minus mean solar time).
    pub eot_min: f64,
    /// (mean distance / distance)², multiplies the solar constant.
    pub eccentricity_factor: f64,
}

/// Julian day for a clock hour of a day of year (clock = UT at longitude 0).
#[must_use]
pub fn julian_day(year: i32, doy: u32, clock_hour: f64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let days = (days_from_civil(year, 1, 1) + i64::from(doy) - 1) as f64;
    days + 2_440_587.5 + clock_hour / 24.0
}

fn deg(x: f64) -> f64 {
    x * PI / 180.0
}

/// Low-precision solar coordinates (Meeus 1998, ch. 25, as in the NOAA solar
/// calculator spreadsheet); declination good to about 0.01° for 1950 to 2050.
#[must_use]
pub fn solar_coords(year: i32, doy: u32, clock_hour: f64) -> SolarCoords {
    let t = (julian_day(year, doy, clock_hour) - 2_451_545.0) / 36_525.0;
    let l0 = (280.466_46 + t * (36_000.769_83 + t * 0.000_303_2)).rem_euclid(360.0);
    let m = 357.529_11 + t * (35_999.050_29 - 0.000_153_7 * t);
    let e = 0.016_708_634 - t * (0.000_042_037 + 0.000_000_126_7 * t);
    let c = libm::sin(deg(m)) * (1.914_602 - t * (0.004_817 + 0.000_014 * t))
        + libm::sin(deg(2.0 * m)) * (0.019_993 - 0.000_101 * t)
        + libm::sin(deg(3.0 * m)) * 0.000_289;
    let true_long = l0 + c;
    let nu = m + c;
    let omega = 125.04 - 1934.136 * t;
    let lambda = true_long - 0.005_69 - 0.004_78 * libm::sin(deg(omega));
    let eps0 =
        23.0 + (26.0 + (21.448 - t * (46.815 + t * (0.000_59 - t * 0.001_813))) / 60.0) / 60.0;
    let eps = eps0 + 0.002_56 * libm::cos(deg(omega));
    let decl = libm::asin(libm::sin(deg(eps)) * libm::sin(deg(lambda)));
    let y = libm::tan(deg(eps) / 2.0);
    let y = y * y;
    let l0r = deg(l0);
    let mr = deg(m);
    let eot_rad = y * libm::sin(2.0 * l0r) - 2.0 * e * libm::sin(mr)
        + 4.0 * e * y * libm::sin(mr) * libm::cos(2.0 * l0r)
        - 0.5 * y * y * libm::sin(4.0 * l0r)
        - 1.25 * e * e * libm::sin(2.0 * mr);
    let r_au = 1.000_001_018 * (1.0 - e * e) / (1.0 + e * libm::cos(deg(nu)));
    SolarCoords {
        declination: Radians(decl),
        eot_min: 4.0 * eot_rad * 180.0 / PI,
        eccentricity_factor: 1.0 / (r_au * r_au),
    }
}

/// Hour angle in radians for a clock hour (local mean solar time), given the
/// equation of time in minutes. Zero at solar noon, positive in the afternoon.
#[must_use]
pub fn hour_angle(clock_hour: f64, eot_min: f64) -> f64 {
    let true_solar_min = clock_hour * 60.0 + eot_min;
    (true_solar_min / 4.0 - 180.0) * PI / 180.0
}

/// Sun position at a fractional clock hour of a date.
#[must_use]
pub fn sun_position(latitude: Degrees, year: i32, doy: u32, clock_hour: f64) -> SunPosition {
    let sc = solar_coords(year, doy, clock_hour);
    let decl = sc.declination.0;
    let ha = hour_angle(clock_hour, sc.eot_min);
    let phi = latitude.to_radians().0;

    let cos_zen = (libm::sin(phi) * libm::sin(decl)
        + libm::cos(phi) * libm::cos(decl) * libm::cos(ha))
    .clamp(-1.0, 1.0);
    let elevation = PI / 2.0 - libm::acos(cos_zen);
    // Azimuth from south, westward positive, then rotated to clockwise from north.
    let az_s = libm::atan2(
        libm::sin(ha),
        libm::cos(ha) * libm::sin(phi) - libm::tan(decl) * libm::cos(phi),
    );
    let mut az = az_s + PI;
    if az >= 2.0 * PI {
        az -= 2.0 * PI;
    }
    SunPosition {
        elevation: Radians(elevation).to_degrees(),
        azimuth: Radians(az).to_degrees(),
    }
}

/// Sunrise hour angle in radians for latitude and declination, using `zenith_deg`
/// as the horizon. 0 means polar night, π means polar day.
#[must_use]
pub fn sunrise_hour_angle(phi: f64, decl: f64, zenith_deg: f64) -> f64 {
    let z = zenith_deg * PI / 180.0;
    let c = (libm::cos(z) - libm::sin(phi) * libm::sin(decl)) / (libm::cos(phi) * libm::cos(decl));
    libm::acos(c.clamp(-1.0, 1.0))
}

/// Sunrise, sunset and day length for a date (clock hours, local mean solar time).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DayLight {
    /// Sunrise, clock hours.
    pub sunrise_h: f64,
    /// Sunset, clock hours.
    pub sunset_h: f64,
    /// Day length, hours.
    pub day_length_h: f64,
}

/// Sunrise, sunset and day length (NOAA definition, 90.833° zenith), evaluated with
/// the declination and equation of time at clock noon. Within a few minutes of an
/// iterated solution (checked against pvlib SPA).
#[must_use]
pub fn day_light(latitude: Degrees, year: i32, doy: u32) -> DayLight {
    let sc = solar_coords(year, doy, 12.0);
    let ws = sunrise_hour_angle(
        latitude.to_radians().0,
        sc.declination.0,
        SUNRISE_ZENITH_DEG,
    );
    let half_h = ws * 180.0 / PI / 15.0;
    let noon = 12.0 - sc.eot_min / 60.0;
    DayLight {
        sunrise_h: noon - half_h,
        sunset_h: noon + half_h,
        day_length_h: 2.0 * half_h,
    }
}

/// Extraterrestrial radiation on a horizontal surface integrated over clock hours
/// `[h0, h1]` of a date, J m⁻². Uses the declination and eccentricity at the interval
/// midpoint and integrates the hour angle exactly between geometric sunrise and sunset.
#[must_use]
pub fn extraterrestrial_horizontal_j_m2(
    latitude: Degrees,
    year: i32,
    doy: u32,
    h0: f64,
    h1: f64,
) -> f64 {
    let mid = 0.5 * (h0 + h1);
    let sc = solar_coords(year, doy, mid);
    let eot = sc.eot_min;
    let decl = sc.declination.0;
    let e0 = sc.eccentricity_factor;
    let phi = latitude.to_radians().0;
    let ws = sunrise_hour_angle(phi, decl, 90.0);
    let w1 = hour_angle(h0, eot).clamp(-ws, ws);
    let w2 = hour_angle(h1, eot).clamp(-ws, ws);
    if w2 <= w1 {
        return 0.0;
    }
    // ∫ G_sc E0 (sinφ sinδ + cosφ cosδ cos ω) dt with dt = dω × 12·3600/π.
    let secs_per_rad = 12.0 * 3600.0 / PI;
    let v = SOLAR_CONSTANT_W_M2
        * e0
        * secs_per_rad
        * ((w2 - w1) * libm::sin(phi) * libm::sin(decl)
            + libm::cos(phi) * libm::cos(decl) * (libm::sin(w2) - libm::sin(w1)));
    v.max(0.0)
}

/// Daily extraterrestrial radiation on a horizontal surface, J m⁻², as the sum of the
/// 24 hourly integrals (so hourly disaggregation sums back exactly).
#[must_use]
pub fn daily_extraterrestrial_j_m2(latitude: Degrees, year: i32, doy: u32) -> f64 {
    (0..24u32)
        .map(|h| {
            extraterrestrial_horizontal_j_m2(latitude, year, doy, f64::from(h), f64::from(h + 1))
        })
        .fold(0.0, |a, b| a + b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equinox_noon_at_equator_is_near_zenith() {
        let p = sun_position(Degrees(0.0), 2000, 80, 12.0);
        assert!(p.elevation.0 > 88.0, "{p:?}");
    }

    #[test]
    fn june_day_length_at_45n_is_about_15_and_a_half_hours() {
        let d = day_light(Degrees(45.0), 2000, 172);
        assert!((d.day_length_h - 15.5).abs() < 0.2, "{d:?}");
    }

    #[test]
    fn polar_day_and_night() {
        assert!((day_light(Degrees(80.0), 2000, 172).day_length_h - 24.0).abs() < 1e-9);
        assert!(day_light(Degrees(80.0), 2000, 355).day_length_h.abs() < 1e-9);
    }

    #[test]
    fn daily_extraterrestrial_reference_values() {
        // FAO-56 eq. 21 (Allen et al. 1998), 45 N: Ra = 41.86 MJ m-2 day-1 on day 166 and
        // 10.48 on day 350. FAO uses G_sc = 1367 W m-2 and simpler series, so allow 2%.
        let jun = daily_extraterrestrial_j_m2(Degrees(45.0), 2000, 166) / 1e6;
        let dec = daily_extraterrestrial_j_m2(Degrees(45.0), 2000, 350) / 1e6;
        assert!((jun / 41.86 - 1.0).abs() < 0.02, "{jun}");
        assert!((dec / 10.48 - 1.0).abs() < 0.02, "{dec}");
    }

    #[test]
    fn night_hours_have_no_extraterrestrial() {
        let v = extraterrestrial_horizontal_j_m2(Degrees(45.0), 2000, 172, 0.0, 1.0);
        assert!(v.abs() < 1e-9);
    }
}
