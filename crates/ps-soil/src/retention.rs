//! Soil water potential and hydraulic conductivity from water content: a van Genuchten
//! (1980) retention curve per texture with the air entry of Ippisch, Vogel and Bastian
//! (2006), anchored on the texture's own field capacity and wilting point (R3a; air
//! entry R5a, decision 0046).
//!
//! `θ(ψ) = θ_s` while the suction `|ψ|` is at most the air-entry suction `h_e` (2 cm of
//! water), and `θ_s [1 + (|ψ| / ψ_s)^n]^(−m) / S_c` beyond it, with `m = 1 − 1/n` and
//! `S_c = [1 + (h_e / ψ_s)^n]^(−m)` the plain curve's saturation at `h_e`, so the curve
//! is continuous there. The residual content `θ_r` is 0, and `ψ_s` (the usual `1/α`) and
//! `n` are solved so that `θ(−33 kPa)` is the field capacity and `θ(−1500 kPa)` the
//! wilting point. Published texture tables (Carsel and Parrish 1988) don't put their
//! −33 kPa points at the sim's field capacities, and keeping a published `n` while
//! solving `θ_r` gives a negative `θ_r` for clay, one equal to the wilting point for sand
//! and one above air-dry for loam (docs/research/realism-plan/numbers/README.md). With
//! `θ_r = 0` the potential is finite down to air-dry.
//!
//! Without the air entry, Mualem's conductivity on a curve with `n` near 1.2 falls to
//! a sixth of `K_s` within 3 cm of saturation, as if the soil had pores of any size; with
//! it, the largest pores empty at `h_e` (Ippisch et al. 2006).
//!
//! Model card: docs/models/soil-and-water.md, "Water potential" and "Water flow".

use ps_units::KPa;

use crate::Hydraulics;

/// Water potential at field capacity (the −33 kPa anchor).
pub const FIELD_CAPACITY_POTENTIAL: KPa = KPa(-33.0);
/// Water potential at the permanent wilting point (the −1500 kPa anchor).
pub const WILTING_POINT_POTENTIAL: KPa = KPa(-1500.0);
/// Pressure of 1 cm of water head, kPa (`ρ g` with g = 9.80665 m s⁻²), to compare the
/// suction scale with tables that give `α` in cm⁻¹.
pub const KPA_PER_CM_WATER: f64 = 0.098_066_5;
/// Air-entry suction `h_e`, cm of water ("~", Ippisch et al. 2006 suggest a few cm;
/// R5a spec §3.2): the suction at which the largest pores start to empty.
pub const AIR_ENTRY_CM: f64 = 2.0;
/// Mualem's pore connectivity `l` (Mualem 1976): the conductivity's `S^l` is a square
/// root.
pub const PORE_CONNECTIVITY: f64 = 0.5;
/// Lowest relative saturation the curve is inverted at. The grid never dries below
/// air-dry (at least 0.11 of saturation for the three textures), so this only keeps an
/// empty cell's potential finite.
const MIN_RELATIVE_SATURATION: f64 = 1e-12;
/// Bracket for the anchor solve's `n`. Each texture's root is near 1.2; the two-anchor
/// condition changes sign once inside it.
const N_BRACKET: (f64, f64) = (1.01, 5.0);
/// Bracket for the anchor solve's `ln ψ_s` (`ψ_s` in kPa): 1e-3 to 1e4 kPa.
const LN_SCALE_BRACKET: (f64, f64) = (-6.907_755_278_982_137, 9.210_340_371_976_184);

/// A texture's retention curve (van Genuchten with an air entry, `θ_r = 0`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Retention {
    /// Saturated water content `θ_s`, m³ m⁻³.
    pub saturation: f64,
    /// Shape exponent `n` (above 1); `m = 1 − 1/n`.
    pub n: f64,
    /// Suction scale `ψ_s = 1/α`, positive: the curve bends at about this suction.
    pub suction_scale: KPa,
    /// Air-entry suction `h_e`, positive.
    pub air_entry: KPa,
    /// `S_c`, the plain curve's relative saturation at the air entry.
    pub entry_saturation: f64,
    /// Mualem's shell term at the air entry, `1 − (1 − S_c^(1/m))^m`, that scales the
    /// conductivity to 1 at saturation.
    entry_shell: f64,
}

/// Bisection on `f` over `(lo, hi)`, halving until the midpoint no longer moves (about
/// 54 steps), so the result is the same on every platform. Returns the low end.
fn bisect(mut lo: f64, mut hi: f64, f: impl Fn(f64) -> f64) -> f64 {
    let lo_positive = f(lo) > 0.0;
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        if (f(mid) > 0.0) == lo_positive {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

impl Retention {
    /// The curve through `(−33 kPa, field capacity)` and `(−1500 kPa, wilting point)`
    /// with `θ_s` the saturation, `θ_r = 0` and the air entry [`AIR_ENTRY_CM`].
    ///
    /// For a given `n`, `ψ_s` comes from a bisection on `ln ψ_s` that puts the
    /// field-capacity anchor at −33 kPa (`S_c` depends on `ψ_s`); `n` comes from an
    /// outer bisection on the wilting-point anchor. Each halves its bracket until the
    /// midpoint no longer moves, so the result is the same on every platform. The pair
    /// takes about 3,000 curve evaluations, once per texture per process.
    #[must_use]
    pub fn anchored(h: &Hydraulics) -> Self {
        debug_assert!(
            0.0 < h.wilting_point
                && h.wilting_point < h.field_capacity
                && h.field_capacity < h.saturation,
            "the anchors need 0 < wilting point < field capacity < saturation"
        );
        let he = AIR_ENTRY_CM * KPA_PER_CM_WATER;
        let s_fc = h.field_capacity / h.saturation;
        let s_wp = h.wilting_point / h.saturation;
        // Relative saturation of the air-entry curve at a suction, for `n` and `ψ_s`.
        let rel = |suction: f64, n: f64, scale: f64| {
            let m = 1.0 - 1.0 / n;
            let plain = |x: f64| libm::pow(1.0 + libm::pow(x / scale, n), -m);
            plain(suction) / plain(he)
        };
        let scale_for = |n: f64| {
            let ln = bisect(LN_SCALE_BRACKET.0, LN_SCALE_BRACKET.1, |l| {
                rel(-FIELD_CAPACITY_POTENTIAL.0, n, libm::exp(l)) - s_fc
            });
            libm::exp(ln)
        };
        debug_assert!(
            {
                let g = |n: f64| rel(-WILTING_POINT_POTENTIAL.0, n, scale_for(n)) - s_wp;
                (g(N_BRACKET.0) > 0.0) != (g(N_BRACKET.1) > 0.0)
            },
            "the n bracket {N_BRACKET:?} holds no root for these anchors"
        );
        let n = bisect(N_BRACKET.0, N_BRACKET.1, |n| {
            rel(-WILTING_POINT_POTENTIAL.0, n, scale_for(n)) - s_wp
        });
        Self::with(h.saturation, n, KPa(scale_for(n)), KPa(he))
    }

    /// The curve with these parameters (air entry `air_entry`, positive).
    #[must_use]
    pub fn with(saturation: f64, n: f64, suction_scale: KPa, air_entry: KPa) -> Self {
        let m = 1.0 - 1.0 / n;
        let sc = libm::pow(1.0 + libm::pow(air_entry.0 / suction_scale.0, n), -m);
        let shell = 1.0 - libm::pow(1.0 - libm::pow(sc, 1.0 / m), m);
        Retention {
            saturation,
            n,
            suction_scale,
            air_entry,
            entry_saturation: sc,
            entry_shell: shell,
        }
    }

    /// `α`, cm⁻¹, as retention tables give it.
    #[must_use]
    pub fn alpha_per_cm(&self) -> f64 {
        KPA_PER_CM_WATER / self.suction_scale.0
    }

    /// Volumetric water content at a water potential (at saturation from the air-entry
    /// suction up).
    #[must_use]
    pub fn theta(&self, psi: KPa) -> f64 {
        if -psi.0 <= self.air_entry.0 {
            return self.saturation;
        }
        let m = 1.0 - 1.0 / self.n;
        let x = libm::pow(-psi.0 / self.suction_scale.0, self.n);
        self.saturation * libm::pow(1.0 + x, -m) / self.entry_saturation
    }

    /// Water potential at a volumetric water content: the closed-form inverse of
    /// [`Retention::theta`], minus the air-entry suction at or above saturation.
    /// Contents at or below 0 read as a relative saturation of 1e-12, so the result is
    /// always finite.
    #[must_use]
    pub fn psi(&self, theta: f64) -> KPa {
        let s = (theta / self.saturation).clamp(MIN_RELATIVE_SATURATION, 1.0);
        if s >= 1.0 {
            return KPa(-self.air_entry.0);
        }
        self.psi_of(self.y_of(s))
    }

    /// Water potential and Mualem's relative hydraulic conductivity at a volumetric
    /// water content (R3c; air entry R5a): with `S = θ / θ_s` and `y = (S S_c)^(1/m)`,
    /// `K / K_s = S^l [(1 − (1 − y)^m) / (1 − (1 − S_c^(1/m))^m)]²`, 1 at saturation.
    /// The potential is [`Retention::psi`]'s; the two share `y`, so the pair costs
    /// three powers.
    #[must_use]
    pub fn psi_and_conductivity(&self, theta: f64) -> (KPa, f64) {
        let (psi, _, k) = self.flow_terms(theta);
        (psi, k)
    }

    /// Water potential, the curve's slope `dθ/dψ` (per kPa, positive) and the relative
    /// conductivity at a volumetric water content: what the water flow step needs per
    /// cell (R5a). The slope is `θ_s m n S (1 − y) / |ψ|`, from the same `y`, so the
    /// three cost the same three powers as [`Retention::psi_and_conductivity`]. At
    /// saturation the slope is 0 and the conductivity 1.
    #[must_use]
    pub fn flow_terms(&self, theta: f64) -> (KPa, f64, f64) {
        let s = (theta / self.saturation).clamp(MIN_RELATIVE_SATURATION, 1.0);
        if s >= 1.0 {
            return (KPa(-self.air_entry.0), 0.0, 1.0);
        }
        let m = 1.0 - 1.0 / self.n;
        let y = self.y_of(s);
        let psi = self.psi_of(y);
        let shell = (1.0 - libm::pow(1.0 - y, m)) / self.entry_shell;
        let k = libm::sqrt(s) * shell * shell;
        let slope = self.saturation * m * self.n * s * (1.0 - y) / -psi.0;
        (psi, slope, k)
    }

    /// `y = (S S_c)^(1/m)` for a relative saturation below 1.
    fn y_of(&self, s: f64) -> f64 {
        libm::pow(s * self.entry_saturation, self.n / (self.n - 1.0))
    }

    /// The potential at `y = (S S_c)^(1/m)`.
    fn psi_of(&self, y: f64) -> KPa {
        let excess = 1.0 / y - 1.0;
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
        // `n` as docs/research/realism-plan/r3-plant-hydraulics.md §3 (numbers/vg_fit.py);
        // `α` a little larger than its 0.0305, 0.088 and 0.0094, since the air entry
        // (R5a) scales the curve by 1 / S_c.
        for (t, n, alpha) in [
            (Texture::Loam, 1.215, 0.0314),
            (Texture::Sand, 1.2885, 0.0964),
            (Texture::Clay, 1.165, 0.0095),
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
            assert_eq!(
                last.to_bits(),
                (-r.air_entry.0).to_bits(),
                "{t:?} saturation reads {last}"
            );
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
    fn conductivity_falls_steeply_and_shares_the_potential() {
        // Mualem with the anchored air-entry curves: about 3.6e-4 of saturated at field
        // capacity and 2.4e-8 at the wilting point in loam, four to five times the
        // values without the air entry (7.9e-5 and 5.3e-9).
        for (t, fc, wp) in [
            (Texture::Loam, 3.61e-4, 2.41e-8),
            (Texture::Sand, 2.50e-5, 7.82e-10),
            (Texture::Clay, 3.96e-3, 5.24e-7),
        ] {
            let h = t.hydraulics();
            let r = t.retention();
            assert_eq!(
                r.psi_and_conductivity(h.saturation),
                (KPa(-r.air_entry.0), 1.0)
            );
            let (_, k_fc) = r.psi_and_conductivity(h.field_capacity);
            let (_, k_wp) = r.psi_and_conductivity(h.wilting_point);
            assert!(rel(k_fc, fc) < 0.02, "{t:?} field capacity {k_fc}");
            assert!(rel(k_wp, wp) < 0.03, "{t:?} wilting point {k_wp}");
            let mut last = 1.0;
            for k in 1..=1000 {
                let theta = h.saturation * (1.0 - f64::from(k) / 1000.0);
                let (psi, kr) = r.psi_and_conductivity(theta);
                assert_eq!(psi, r.psi(theta), "{t:?} at {theta}");
                assert!(
                    (kr < last || kr == 0.0) && kr >= 0.0,
                    "{t:?} at {theta}: {kr}"
                );
                last = kr;
            }
        }
    }

    /// R5a: the curve is flat at saturation up to the air-entry suction and continuous
    /// there, and the flow step's slope is the curve's derivative.
    #[test]
    fn air_entry_and_slope() {
        for t in TEXTURES {
            let h = t.hydraulics();
            let r = t.retention();
            let he = r.air_entry.0;
            assert!((he - AIR_ENTRY_CM * KPA_PER_CM_WATER).abs() < 1e-15);
            assert_eq!(r.theta(KPa(0.0)).to_bits(), h.saturation.to_bits());
            assert_eq!(r.theta(KPa(-he)).to_bits(), h.saturation.to_bits());
            assert!(
                rel(r.theta(KPa(-he * (1.0 + 1e-9))), h.saturation) < 1e-8,
                "{t:?}"
            );
            for k in 1..50 {
                let theta = h.air_dry + (h.saturation - h.air_dry) * f64::from(k) / 50.0;
                let (psi, slope, _) = r.flow_terms(theta);
                let d = 1e-6 * psi.0.abs();
                let numeric = (r.theta(KPa(psi.0 + d)) - r.theta(KPa(psi.0 - d))) / (2.0 * d);
                assert!(
                    rel(slope, numeric) < 1e-5,
                    "{t:?} at {theta}: {slope} vs {numeric}"
                );
            }
            assert_eq!(r.flow_terms(h.saturation), (KPa(-he), 0.0, 1.0));
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
