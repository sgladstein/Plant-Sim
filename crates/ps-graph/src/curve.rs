//! Beta-function expansion curve (Yin et al. 2003), design plan §3.13 and §11.4.
//!
//! `y(t) = (1 + (te − t)/(te − tm)) · (t/te)^(te/(te − tm))` for `0 ≤ t ≤ te`, where `t` is
//! thermal age, `te` the duration and `tm` the time of the maximum rate. `y(0) = 0`,
//! `y(te) = 1`, and `y` increases monotonically between. An organ's potential growth
//! over a step is `potential × (y(t + dt) − y(t))`.

use ps_units::{DegreeDays, GramsDM};
use serde::{Deserialize, Serialize};

/// Errors building a curve.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CurveError {
    /// Duration must be positive and finite.
    #[error("expansion duration {0} Cd must be positive and finite")]
    Duration(f64),
    /// The time of the maximum rate must lie strictly inside (0, duration).
    #[error("time of maximum rate {t_max} Cd must lie strictly inside (0, {duration})")]
    TimeOfMax {
        /// Given time of maximum rate.
        t_max: f64,
        /// Given duration.
        duration: f64,
    },
    /// Potential size must be non-negative and finite.
    #[error("potential size {0} g DM must be non-negative and finite")]
    Potential(f64),
}

/// An organ's expansion curve over thermal time.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpansionCurve {
    /// Thermal time from birth to full size, `te`.
    pub duration: DegreeDays,
    /// Thermal age of the maximum expansion rate, `tm`, with `0 < tm < te`.
    pub t_max: DegreeDays,
    /// Potential final structural mass, before any reduction by unmet demand.
    pub potential: GramsDM,
}

impl ExpansionCurve {
    /// Builds a validated curve.
    ///
    /// # Errors
    /// [`CurveError`] when the parameters are out of range.
    pub fn new(
        duration: DegreeDays,
        t_max: DegreeDays,
        potential: GramsDM,
    ) -> Result<Self, CurveError> {
        let te = duration.0;
        let tm = t_max.0;
        if !(te.is_finite() && te > 0.0) {
            return Err(CurveError::Duration(te));
        }
        if !(tm.is_finite() && tm > 0.0 && tm < te) {
            return Err(CurveError::TimeOfMax {
                t_max: tm,
                duration: te,
            });
        }
        if !(potential.0.is_finite() && potential.0 >= 0.0) {
            return Err(CurveError::Potential(potential.0));
        }
        Ok(Self {
            duration,
            t_max,
            potential,
        })
    }

    /// A curve for organs that do not expand (seeds, buds): zero potential.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            duration: DegreeDays(1.0),
            t_max: DegreeDays(0.5),
            potential: GramsDM(0.0),
        }
    }

    /// Fraction of final size reached at thermal age `t`, in `[0, 1]`.
    #[must_use]
    pub fn fraction(&self, t: DegreeDays) -> f64 {
        let te = self.duration.0;
        let tm = self.t_max.0;
        let t = t.0;
        if t <= 0.0 {
            return 0.0;
        }
        if t >= te {
            return 1.0;
        }
        let shape = te / (te - tm);
        ((1.0 + (te - t) / (te - tm)) * libm::pow(t / te, shape)).min(1.0)
    }

    /// Relative rate `dy/dt` at thermal age `t`, per °C·day.
    #[must_use]
    pub fn rate(&self, t: DegreeDays) -> f64 {
        let te = self.duration.0;
        let tm = self.t_max.0;
        let t = t.0;
        if t <= 0.0 || t >= te {
            return 0.0;
        }
        // dy/dt = (2te − tm)(te − t) / (te (te − tm)²) · (t/te)^(tm/(te − tm))
        let base = t / te;
        let k = tm / (te - tm);
        (2.0 * te - tm) / (te * (te - tm)) * ((te - t) / (te - tm)) * libm::pow(base, k)
    }

    /// Potential structural growth of `potential_size` between thermal ages `t` and `t + dt`.
    #[must_use]
    pub fn increment(&self, potential_size: GramsDM, t: DegreeDays, dt: DegreeDays) -> GramsDM {
        let dy = self.fraction(t + dt) - self.fraction(t);
        potential_size * dy.max(0.0)
    }

    /// True once the organ has reached full size.
    #[must_use]
    pub fn is_complete(&self, t: DegreeDays) -> bool {
        t.0 >= self.duration.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn curve(te: f64, tm: f64, p: f64) -> ExpansionCurve {
        ExpansionCurve::new(DegreeDays(te), DegreeDays(tm), GramsDM(p)).unwrap()
    }

    #[test]
    fn endpoints() {
        let c = curve(100.0, 60.0, 2.0);
        assert!(c.fraction(DegreeDays(0.0)).abs() < 1e-15);
        assert!((c.fraction(DegreeDays(100.0)) - 1.0).abs() < 1e-15);
        // §11.4 check: y(te) = 1 by the formula itself, just inside the end.
        assert!((c.fraction(DegreeDays(100.0 - 1e-9)) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_bad_parameters() {
        assert!(ExpansionCurve::new(DegreeDays(0.0), DegreeDays(0.0), GramsDM(1.0)).is_err());
        assert!(ExpansionCurve::new(DegreeDays(10.0), DegreeDays(10.0), GramsDM(1.0)).is_err());
        assert!(ExpansionCurve::new(DegreeDays(10.0), DegreeDays(5.0), GramsDM(-1.0)).is_err());
        assert!(ExpansionCurve::new(DegreeDays(f64::NAN), DegreeDays(5.0), GramsDM(1.0)).is_err());
    }

    #[test]
    fn rate_peaks_at_t_max() {
        let c = curve(100.0, 60.0, 1.0);
        let at = |t: f64| c.rate(DegreeDays(t));
        assert!(at(60.0) > at(59.0) && at(60.0) > at(61.0));
    }

    #[test]
    fn rate_matches_numeric_derivative() {
        let c = curve(80.0, 30.0, 1.0);
        for i in 1..80 {
            let t = f64::from(i);
            let h = 1e-6;
            let num = (c.fraction(DegreeDays(t + h)) - c.fraction(DegreeDays(t - h))) / (2.0 * h);
            assert!(
                (num - c.rate(DegreeDays(t))).abs() < 1e-6,
                "t {t}: {num} vs {}",
                c.rate(DegreeDays(t))
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]
        /// WP3 done-when: the expansion curve integrates to the potential size (§8.1: 20 sets, 1e-6).
        #[test]
        fn increments_integrate_to_potential(
            te in 5.0f64..800.0,
            frac in 0.05f64..0.95,
            p in 1e-4f64..50.0,
            steps in 10u32..5000,
        ) {
            let c = curve(te, te * frac, p);
            let dt = DegreeDays(te * 1.2 / f64::from(steps));
            let mut t = DegreeDays(0.0);
            let mut total = GramsDM(0.0);
            for _ in 0..steps {
                let inc = c.increment(c.potential, t, dt);
                prop_assert!(inc.0 >= 0.0);
                total += inc;
                t += dt;
            }
            prop_assert!((total.0 - p).abs() <= 1e-6 * p.max(1.0), "{} vs {}", total.0, p);
        }

        #[test]
        fn fraction_is_monotone(te in 5.0f64..800.0, frac in 0.05f64..0.95) {
            let c = curve(te, te * frac, 1.0);
            let mut prev = 0.0;
            for i in 0..=200 {
                let y = c.fraction(DegreeDays(te * f64::from(i) / 200.0));
                prop_assert!(y >= prev - 1e-15);
                prop_assert!((0.0..=1.0).contains(&y));
                prev = y;
            }
        }
    }
}
