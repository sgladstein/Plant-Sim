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
/// linear between. "~": a display quantity only, never used in physiology.
#[must_use]
pub fn cloud_fraction_from_kt(kt: f64) -> f64 {
    ((0.75 - kt) / 0.55).clamp(0.0, 1.0)
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
    fn tetens_reference() {
        // FAO-56 Annex 2 Table 2.3: e°(20 °C) = 2.338 kPa, e°(30 °C) = 4.243 kPa.
        assert!((saturation_vapor_pressure_kpa(20.0) - 2.338).abs() < 1e-3);
        assert!((saturation_vapor_pressure_kpa(30.0) - 4.243).abs() < 1e-3);
    }
}
