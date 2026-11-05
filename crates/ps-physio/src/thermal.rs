//! Thermal time and the temperature multiplier on expansion (§3.11, §11.4).
//! Model card: docs/models/thermal-time-and-phenology.md.

use ps_units::{Celsius, DegreeDays};
use serde::{Deserialize, Serialize};

/// Cardinal temperatures of development: a trapezoid (decision 0032), or the old
/// three-point response when `opt_high` equals `opt`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cardinal {
    /// Base: no development at or below.
    pub base: Celsius,
    /// Start of the optimum plateau.
    pub opt: Celsius,
    /// End of the optimum plateau.
    pub opt_high: Celsius,
    /// Maximum: no development at or above.
    pub max: Celsius,
}

impl Cardinal {
    /// Development rate in °C (degree-days per day) at temperature `t`: linear from
    /// the base to the optimum, flat to `opt_high`, linear to zero at the maximum,
    /// normalized so the rate on the plateau is `opt − base` (§11.4, decision 0032).
    #[must_use]
    pub fn rate(&self, t: Celsius) -> f64 {
        let (b, o, o2, m) = (self.base.0, self.opt.0, self.opt_high.0, self.max.0);
        let t = t.0;
        if t <= b || t >= m {
            0.0
        } else if t <= o {
            t - b
        } else if t <= o2 {
            o - b
        } else {
            (o - b) * (m - t) / (m - o2)
        }
    }

    /// Thermal time accumulated over one hour at temperature `t`.
    #[must_use]
    pub fn hourly(&self, t: Celsius) -> DegreeDays {
        DegreeDays(self.rate(t).max(0.0) / 24.0)
    }
}

/// Temperature multiplier on expansion growth (§3.11): zero at or below `min`, rising
/// linearly to one at `opt_low`, one to `opt_high`, falling linearly to zero at `max`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrowthTemperature {
    /// Zero at or below.
    pub min: Celsius,
    /// Start of the plateau.
    pub opt_low: Celsius,
    /// End of the plateau.
    pub opt_high: Celsius,
    /// Zero at or above.
    pub max: Celsius,
}

impl GrowthTemperature {
    /// The multiplier, in `[0, 1]`.
    #[must_use]
    pub fn factor(&self, t: Celsius) -> f64 {
        let t = t.0;
        let f = if t <= self.min.0 || t >= self.max.0 {
            0.0
        } else if t < self.opt_low.0 {
            (t - self.min.0) / (self.opt_low.0 - self.min.0)
        } else if t <= self.opt_high.0 {
            1.0
        } else {
            (self.max.0 - t) / (self.max.0 - self.opt_high.0)
        };
        f.clamp(0.0, 1.0)
    }
}
