//! Maintenance and growth respiration (§3.8). Model card: docs/models/respiration.md.

use ps_units::{Celsius, GramsC, GramsN};

/// Maintenance respiration of one non-leaf organ over one hour:
/// `rate20 · N · Q10^((T − 20)/10) / 24`, with `rate20` in g C per g N per day.
/// Leaves pay maintenance through day respiration in gas exchange instead
/// (decision 0009), so callers skip them.
#[must_use]
pub fn maintenance_hourly(rate20: f64, q10: f64, nitrogen: GramsN, t: Celsius) -> GramsC {
    GramsC(rate20 * nitrogen.0.max(0.0) * libm::pow(q10, (t.0 - 20.0) / 10.0) / 24.0)
}

/// Split of carbon spent on growth: `(structural carbon, growth respiration)` for
/// `spent` g C at construction cost `cost` (g C used per g C built; 1.25 means 20% is
/// respired).
#[must_use]
pub fn growth_split(spent: GramsC, cost: f64) -> (GramsC, GramsC) {
    let built = spent.0 / cost.max(1.0);
    (GramsC(built), GramsC(spent.0 - built))
}
