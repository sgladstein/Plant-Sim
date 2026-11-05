//! Direct and diffuse split, cloud proxy and humidity helpers.

/// Diffuse fraction of global horizontal radiation from the hourly clearness index
/// (Erbs, Klein and Duffie 1982).
#[must_use]
pub fn erbs_diffuse_fraction(kt: f64) -> f64 {
    let kt = kt.clamp(0.0, 1.0);
    if kt <= 0.22 {
        1.0 - 0.09 * kt
    } else if kt <= 0.80 {
        let k2 = kt * kt;
        0.9511 - 0.1604 * kt + 4.388 * k2 - 16.638 * k2 * kt + 12.336 * k2 * k2
    } else {
        0.165
    }
}

/// Cloud fraction proxy from the daily clearness index: 0 at Kt ≥ 0.75, 1 at Kt ≤ 0.2,
/// linear between. "~". The renderer draws it and the leaf energy balance reads it for
/// the sky's longwave (WP27, decision 0034).
#[must_use]
pub fn cloud_fraction_from_kt(kt: f64) -> f64 {
    ((0.75 - kt) / 0.55).clamp(0.0, 1.0)
}

/// Wind speed at 2 m when a weather source gives none, m s⁻¹ (FAO-56's default for
/// missing wind data, WP27).
pub const DEFAULT_WIND_M_S: f64 = 2.0;

/// [`DEFAULT_WIND_M_S`] as a serde default.
#[must_use]
pub fn default_wind() -> ps_units::MetersPerSecond {
    ps_units::MetersPerSecond(DEFAULT_WIND_M_S)
}

/// Mean wind over clock hours `[h0, h1)` of a day whose mean wind is `avg`, with sunrise
/// at `rise` and day length `dayl` hours (DSSAT's HWIND, decision 0034): calm at
/// `0.3 W_max` through the night, a sine through the day from `0.3 W_max` at sunrise to
/// `W_max` at solar noon, with `W_max = avg / (0.3 + 0.7 dayl / 48)`, so the 24 hourly
/// means average to `avg` exactly. The sine is integrated over the hour.
#[must_use]
pub fn hourly_wind(avg: f64, rise: f64, dayl: f64, h0: f64, h1: f64) -> f64 {
    let dayl = dayl.clamp(0.0, 24.0);
    let w_max = avg / (0.3 + 0.7 * dayl / 48.0);
    let night = 0.3 * w_max;
    if h1 <= h0 {
        return night;
    }
    // The part of the hour between sunrise and sunset.
    let (x0, x1) = (h0.max(rise), h1.min(rise + dayl));
    let mut sum = night * (h1 - h0);
    if dayl > 0.0 && x1 > x0 {
        let w = 2.0 * core::f64::consts::PI / dayl;
        let phase = 1.5 * core::f64::consts::PI;
        let cos_at = |x: f64| libm::cos(w * (x - rise) + phase);
        // Above the night floor: 0.35 W_max (1 + sin(...)).
        sum += 0.35 * w_max * ((x1 - x0) - (cos_at(x1) - cos_at(x0)) / w);
    }
    sum / (h1 - h0)
}

/// Saturation vapor pressure over water, kPa (Tetens; FAO-56 eq. 11).
#[must_use]
pub fn saturation_vapor_pressure_kpa(t_c: f64) -> f64 {
    0.6108 * libm::exp(17.27 * t_c / (t_c + 237.3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erbs_is_continuous_at_breakpoints() {
        for &k in &[0.22, 0.80] {
            let lo = erbs_diffuse_fraction(k - 1e-9);
            let hi = erbs_diffuse_fraction(k + 1e-9);
            assert!((lo - hi).abs() < 2e-3, "{k}: {lo} vs {hi}");
        }
    }

    #[test]
    fn erbs_bounds() {
        for i in 0..=100 {
            let f = erbs_diffuse_fraction(f64::from(i) / 100.0);
            assert!((0.0..=1.0).contains(&f));
        }
    }

    #[test]
    fn hourly_wind_keeps_the_daily_mean() {
        for (rise, dayl) in [(5.2, 14.6), (7.9, 8.2), (0.0, 24.0), (6.0, 0.0)] {
            let hours: Vec<f64> = (0..24)
                .map(|h| hourly_wind(2.0, rise, dayl, f64::from(h), f64::from(h + 1)))
                .collect();
            let mean = hours.iter().sum::<f64>() / 24.0;
            assert!((mean - 2.0).abs() < 1e-12, "{rise} {dayl}: {mean}");
            assert!(hours.iter().all(|w| *w > 0.0));
        }
        // Calm night, windiest about solar noon.
        let (rise, dayl) = (5.0, 14.0);
        let w = |h: u32| hourly_wind(2.0, rise, dayl, f64::from(h), f64::from(h + 1));
        assert!(w(2) < w(8) && w(8) < w(11) && w(11) > w(16));
        assert!((w(2) - w(23)).abs() < 1e-12);
    }

    #[test]
    fn tetens_reference() {
        // FAO-56 Annex 2 Table 2.3: e°(20 °C) = 2.338 kPa, e°(30 °C) = 4.243 kPa.
        assert!((saturation_vapor_pressure_kpa(20.0) - 2.338).abs() < 1e-3);
        assert!((saturation_vapor_pressure_kpa(30.0) - 4.243).abs() < 1e-3);
    }
}
