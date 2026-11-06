//! Soil water potential from water content (R3a): a van Genuchten (1980) retention
//! curve per texture, anchored on the texture's own field capacity and wilting point.
//!
//! `θ(ψ) = θ_s / [1 + (|ψ| / ψ_s)^n]^(1 − 1/n)`, with the residual content `θ_r` set
//! to 0 and the suction scale `ψ_s` (the usual `1/α`) and `n` solved so that
//! `θ(−33 kPa)` is the field capacity and `θ(−1500 kPa)` the wilting point. Published
//! texture tables (Carsel and Parrish 1988) don't put their −33 kPa points at the sim's
//! field capacities, and keeping a published `n` while solving `θ_r` gives a negative
//! `θ_r` for clay, one equal to the wilting point for sand and one above air-dry for
//! loam (docs/research/realism-plan/numbers/README.md). With `θ_r = 0` the potential is
//! finite down to air-dry.
//!
//! Nothing the sim computes reads it yet: R3a only adds the derived value. Model card:
//! docs/models/soil-and-water.md, "Water potential".

use ps_units::KPa;

use crate::Hydraulics;

/// Water potential at field capacity (the −33 kPa anchor).
pub const FIELD_CAPACITY_POTENTIAL: KPa = KPa(-33.0);
/// Water potential at the permanent wilting point (the −1500 kPa anchor).
pub const WILTING_POINT_POTENTIAL: KPa = KPa(-1500.0);
/// Pressure of 1 cm of water head, kPa (`ρ g` with g = 9.80665 m s⁻²), to compare the
/// suction scale with tables that give `α` in cm⁻¹.
pub const KPA_PER_CM_WATER: f64 = 0.098_066_5;
/// Lowest relative saturation the curve is inverted at. The grid never dries below
/// air-dry (at least 0.11 of saturation for the three textures), so this only keeps an
/// empty cell's potential finite.
const MIN_RELATIVE_SATURATION: f64 = 1e-12;
/// Bracket for the anchor solve's `n`. Each texture's root is near 1.2; the two-anchor
/// condition changes sign once inside it.
const N_BRACKET: (f64, f64) = (1.01, 5.0);

/// A texture's retention curve (van Genuchten, `θ_r = 0`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Retention {
    /// Saturated water content `θ_s`, m³ m⁻³.
    pub saturation: f64,
    /// Shape exponent `n` (above 1); `m = 1 − 1/n`.
    pub n: f64,
    /// Suction scale `ψ_s = 1/α`, positive: the curve bends at about this suction.
    pub suction_scale: KPa,
}

impl Retention {
    /// The curve through `(−33 kPa, field capacity)` and `(−1500 kPa, wilting point)`
    /// with `θ_s` the saturation and `θ_r = 0`.
    ///
    /// `n` comes from a bisection on `ln B(n) − ln A(n) − n ln(1500 / 33)`, with
    /// `A = S_fc^(−1/m) − 1` and `B = S_wp^(−1/m) − 1` (S the relative saturation): the
    /// two anchors' `(|ψ| / ψ_s)^n`. It halves its bracket until the midpoint no longer
    /// moves (about 54 steps), so the result is the same on every platform. `ψ_s` then
    /// puts the field-capacity anchor exactly at −33 kPa.
    #[must_use]
    pub fn anchored(h: &Hydraulics) -> Self {
        debug_assert!(
            0.0 < h.wilting_point
                && h.wilting_point < h.field_capacity
                && h.field_capacity < h.saturation,
            "the anchors need 0 < wilting point < field capacity < saturation"
        );
        let s_fc = h.field_capacity / h.saturation;
        let s_wp = h.wilting_point / h.saturation;
        let ratio = libm::log(WILTING_POINT_POTENTIAL.0 / FIELD_CAPACITY_POTENTIAL.0);
        let excess = |s: f64, n: f64| libm::pow(s, -n / (n - 1.0)) - 1.0;
        let g = |n: f64| libm::log(excess(s_wp, n)) - libm::log(excess(s_fc, n)) - n * ratio;
        let (mut lo, mut hi) = N_BRACKET;
        let lo_positive = g(lo) > 0.0;
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if (g(mid) > 0.0) == lo_positive {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let n = lo;
        let scale = -FIELD_CAPACITY_POTENTIAL.0 / libm::pow(excess(s_fc, n), 1.0 / n);
        Retention {
            saturation: h.saturation,
            n,
            suction_scale: KPa(scale),
        }
    }

    /// `α`, cm⁻¹, as retention tables give it.
    #[must_use]
    pub fn alpha_per_cm(&self) -> f64 {
        KPA_PER_CM_WATER / self.suction_scale.0
    }

    /// Volumetric water content at a water potential (at saturation for `ψ ≥ 0`).
    #[must_use]
    pub fn theta(&self, psi: KPa) -> f64 {
        if psi.0 >= 0.0 {
            return self.saturation;
        }
        let m = 1.0 - 1.0 / self.n;
        let x = libm::pow(-psi.0 / self.suction_scale.0, self.n);
        self.saturation * libm::pow(1.0 + x, -m)
    }

    /// Water potential at a volumetric water content: the closed-form inverse of
    /// [`Retention::theta`], 0 at or above saturation. Contents at or below 0 read as a
    /// relative saturation of 1e-12, so the result is always finite.
    #[must_use]
    pub fn psi(&self, theta: f64) -> KPa {
        let s = (theta / self.saturation).clamp(MIN_RELATIVE_SATURATION, 1.0);
        if s >= 1.0 {
            return KPa(0.0);
        }
        let excess = libm::pow(s, -self.n / (self.n - 1.0)) - 1.0;
        KPa(-self.suction_scale.0 * libm::pow(excess, 1.0 / self.n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Texture;

    const TEXTURES: [Texture; 3] = [Texture::Loam, Texture::Sand, Texture::Clay];

    fn rel(a: f64, b: f64) -> f64 {
        ((a - b) / b).abs()
    }

    #[test]
    fn anchors_hold_for_every_texture() {
        for t in TEXTURES {
            let h = t.hydraulics();
            let r = t.retention();
            let fc = r.psi(h.field_capacity);
            let wp = r.psi(h.wilting_point);
            assert!(rel(fc.0, -33.0) < 1e-9, "{t:?} field capacity {fc:?}");
            assert!(rel(wp.0, -1500.0) < 1e-9, "{t:?} wilting point {wp:?}");
            assert!(rel(r.theta(KPa(-33.0)), h.field_capacity) < 1e-9, "{t:?}");
            assert!(rel(r.theta(KPa(-1500.0)), h.wilting_point) < 1e-9, "{t:?}");
        }
    }

    #[test]
    fn parameters_match_the_realism_plan() {
        // docs/research/realism-plan/r3-plant-hydraulics.md §3, from numbers/vg_fit.py.
        for (t, n, alpha) in [
            (Texture::Loam, 1.215, 0.0305),
            (Texture::Sand, 1.289, 0.088),
            (Texture::Clay, 1.165, 0.0094),
        ] {
            let r = t.retention();
            assert!((r.n - n).abs() < 5e-4, "{t:?} n {}", r.n);
            assert!(
                rel(r.alpha_per_cm(), alpha) < 0.01,
                "{t:?} alpha {}",
                r.alpha_per_cm()
            );
        }
    }

    #[test]
    fn potential_falls_as_the_soil_dries() {
        for t in TEXTURES {
            let h = t.hydraulics();
            let r = t.retention();
            let mut last = r.psi(h.saturation).0;
            assert!(last.abs() < 1e-12, "{t:?} saturation reads {last}");
            for k in 1..=1000 {
                let theta = h.saturation * (1.0 - f64::from(k) / 1000.0);
                let psi = r.psi(theta).0;
                assert!(psi < last, "{t:?} at {theta}: {psi} not below {last}");
                assert!(psi.is_finite(), "{t:?} at {theta}");
                last = psi;
            }
        }
    }

    #[test]
    fn inverse_is_exact() {
        for t in TEXTURES {
            let h = t.hydraulics();
            let r = t.retention();
            for k in 1..100 {
                let theta = h.air_dry + (h.saturation - h.air_dry) * f64::from(k) / 100.0;
                let back = r.theta(r.psi(theta));
                assert!(rel(back, theta) < 1e-12, "{t:?} {theta} came back {back}");
            }
            for psi in [-0.5, -10.0, -33.0, -100.0, -1500.0, -1e4, -5e4] {
                let back = r.psi(r.theta(KPa(psi))).0;
                assert!(rel(back, psi) < 1e-9, "{t:?} {psi} came back {back}");
            }
        }
    }

    #[test]
    fn finite_down_to_air_dry() {
        // The realism plan's figures: loam about −88 MPa, sand −36, clay −178.
        for (t, mpa) in [
            (Texture::Loam, -88.0),
            (Texture::Sand, -36.0),
            (Texture::Clay, -178.0),
        ] {
            let h = t.hydraulics();
            let r = t.retention();
            // θ_r = 0 sits at or below air-dry, so air-dry has a finite potential.
            let psi = r.psi(h.air_dry).0;
            assert!(psi.is_finite(), "{t:?}");
            assert!(rel(psi / 1000.0, mpa) < 0.01, "{t:?} air-dry {psi} kPa");
            assert!(r.psi(0.0).0.is_finite(), "{t:?} empty");
        }
    }
}
