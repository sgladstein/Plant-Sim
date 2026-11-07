//! Unit newtypes for the plant simulation.
//!
//! Every physical quantity that crosses a function boundary in the sim is one of these
//! types. Raw `f64` is allowed only inside a function body (CLAUDE.md). Conversions
//! between units are explicit functions named after what they do, so a reader can see
//! every place where, say, dry matter becomes carbon.
//!
//! Constructors do not panic. [`Quantity::try_new`] rejects non-finite values;
//! the tuple constructor is public for literals and hot loops where the value is known
//! to be finite.

use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

/// Molar mass of carbon, g per mol.
pub const CARBON_MOLAR_MASS_G_PER_MOL: f64 = 12.0;
/// Seconds in one sim hour.
pub const SECONDS_PER_HOUR: f64 = 3600.0;
/// Offset between Celsius and Kelvin.
pub const KELVIN_OFFSET: f64 = 273.15;

/// Error returned by checked constructors and conversions.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UnitError {
    /// The value was NaN or infinite.
    #[error("{unit}: value {value} is not finite")]
    NotFinite {
        /// Unit symbol of the quantity being built.
        unit: &'static str,
        /// The offending value.
        value: f64,
    },
    /// The value was outside the physically meaningful range for the unit.
    #[error("{unit}: value {value} is outside [{min}, {max}]")]
    OutOfRange {
        /// Unit symbol of the quantity being built.
        unit: &'static str,
        /// The offending value.
        value: f64,
        /// Lower bound, inclusive.
        min: f64,
        /// Upper bound, inclusive.
        max: f64,
    },
    /// A unit string read from a data file did not match the expected unit.
    #[error("expected unit \"{expected}\", found \"{found}\"")]
    Mismatch {
        /// The unit the field requires.
        expected: &'static str,
        /// The unit the file gave.
        found: String,
    },
}

/// Behaviour shared by every scalar unit newtype.
pub trait Quantity: Copy + Sized {
    /// Unit symbol as written in data files, for example `"g C"`.
    const UNIT: &'static str;

    /// Wraps a raw value without checks.
    fn from_raw(v: f64) -> Self;

    /// The raw value in this unit.
    fn value(self) -> f64;

    /// Wraps a raw value, rejecting NaN and infinity.
    ///
    /// # Errors
    /// [`UnitError::NotFinite`] when `v` is NaN or infinite.
    fn try_new(v: f64) -> Result<Self, UnitError> {
        if v.is_finite() {
            Ok(Self::from_raw(v))
        } else {
            Err(UnitError::NotFinite {
                unit: Self::UNIT,
                value: v,
            })
        }
    }

    /// Wraps a raw value that must also lie in `[min, max]`.
    ///
    /// # Errors
    /// [`UnitError::NotFinite`] or [`UnitError::OutOfRange`].
    fn try_in_range(v: f64, min: f64, max: f64) -> Result<Self, UnitError> {
        let q = Self::try_new(v)?;
        if v < min || v > max {
            return Err(UnitError::OutOfRange {
                unit: Self::UNIT,
                value: v,
                min,
                max,
            });
        }
        Ok(q)
    }

    /// Checks that a unit string from a data file matches this type's unit.
    ///
    /// # Errors
    /// [`UnitError::Mismatch`] when the strings differ.
    fn check_unit(found: &str) -> Result<(), UnitError> {
        if found == Self::UNIT {
            Ok(())
        } else {
            Err(UnitError::Mismatch {
                expected: Self::UNIT,
                found: found.to_owned(),
            })
        }
    }
}

/// Defines an f64 newtype with same-unit arithmetic and scaling by a dimensionless f64.
macro_rules! quantity {
    ($(#[$doc:meta])* $name:ident, $unit:literal) => {
        $(#[$doc])*
        #[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub f64);

        impl Quantity for $name {
            const UNIT: &'static str = $unit;
            #[inline]
            fn from_raw(v: f64) -> Self {
                Self(v)
            }
            #[inline]
            fn value(self) -> f64 {
                self.0
            }
        }

        impl $name {
            /// Zero in this unit.
            pub const ZERO: Self = Self(0.0);

            /// The larger of two values.
            #[must_use]
            pub fn max(self, other: Self) -> Self {
                if other.0 > self.0 { other } else { self }
            }

            /// The smaller of two values.
            #[must_use]
            pub fn min(self, other: Self) -> Self {
                if other.0 < self.0 { other } else { self }
            }

            /// Bit pattern of the value, for state hashing (never hash formatted floats).
            #[must_use]
            pub fn to_bits(self) -> u64 {
                self.0.to_bits()
            }
        }

        impl Add for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }
        impl Sub for $name {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }
        impl AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
        impl Neg for $name {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }
        impl Mul<f64> for $name {
            type Output = Self;
            #[inline]
            fn mul(self, rhs: f64) -> Self {
                Self(self.0 * rhs)
            }
        }
        impl Mul<$name> for f64 {
            type Output = $name;
            #[inline]
            fn mul(self, rhs: $name) -> $name {
                $name(self * rhs.0)
            }
        }
        impl Div<f64> for $name {
            type Output = Self;
            #[inline]
            fn div(self, rhs: f64) -> Self {
                Self(self.0 / rhs)
            }
        }
        /// Ratio of two quantities in the same unit is dimensionless.
        impl Div for $name {
            type Output = f64;
            #[inline]
            fn div(self, rhs: Self) -> f64 {
                self.0 / rhs.0
            }
        }
        impl core::iter::Sum for $name {
            /// Sums in iteration order (callers own the order; the sim uses fixed order).
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |a, b| a + b)
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", self.0, $unit)
            }
        }
    };
}

quantity!(
    /// Grams of carbon.
    GramsC,
    "g C"
);
quantity!(
    /// Grams of dry matter.
    GramsDM,
    "g DM"
);
quantity!(
    /// Grams of nitrogen.
    GramsN,
    "g N"
);
quantity!(
    /// Temperature in degrees Celsius.
    Celsius,
    "C"
);
quantity!(
    /// Temperature in kelvin.
    Kelvin,
    "K"
);
quantity!(
    /// Length in meters.
    Meters,
    "m"
);
quantity!(
    /// Length in centimeters (data files and root parameters).
    Centimeters,
    "cm"
);
quantity!(
    /// Area in square meters.
    M2,
    "m2"
);
quantity!(
    /// Area in square centimeters (data files).
    Cm2,
    "cm2"
);
quantity!(
    /// Photon or CO2 flux density, µmol m⁻² s⁻¹.
    UmolPerM2S,
    "umol/m2/s"
);
quantity!(
    /// Molar flux density, mol m⁻² s⁻¹ (stomatal conductance, transpiration).
    MolPerM2S,
    "mol/m2/s"
);
quantity!(
    /// Daily photon integral per area, mol m⁻² day⁻¹ (daily light integral).
    MolPerM2Day,
    "mol/m2/day"
);
quantity!(
    /// Thermal time, °C·day.
    DegreeDays,
    "Cd"
);
quantity!(
    /// Pressure or vapor pressure deficit in kilopascals.
    KPa,
    "kPa"
);
quantity!(
    /// CO2 mole fraction, µmol mol⁻¹ (ppm).
    Ppm,
    "ppm"
);
quantity!(
    /// Irradiance, W m⁻².
    WPerM2,
    "W/m2"
);
quantity!(
    /// Radiant energy per area, J m⁻² (daily or hourly radiation totals).
    JPerM2,
    "J/m2"
);
quantity!(
    /// Specific length, m per g dry matter (internodes, roots).
    MPerG,
    "m/g"
);
quantity!(
    /// Specific area, m² per g dry matter (specific leaf area).
    M2PerG,
    "m2/g"
);
quantity!(
    /// Density, g dry matter per m³ of tissue.
    GPerM3,
    "g/m3"
);
quantity!(
    /// Water depth in millimeters (rain, irrigation); 1 mm over 1 m² is 1 kg.
    Millimeters,
    "mm"
);
quantity!(
    /// Mass of water in kilograms.
    KgWater,
    "kg"
);
quantity!(
    /// Speed, m s⁻¹ (wind).
    MetersPerSecond,
    "m/s"
);
quantity!(
    /// Square root of pressure, kPa^0.5 (the Medlyn slope g1).
    SqrtKPa,
    "kPa^0.5"
);
quantity!(
    /// Molar energy, J mol⁻¹ (activation and deactivation energies).
    JPerMol,
    "J/mol"
);
quantity!(
    /// Molar entropy, J mol⁻¹ K⁻¹.
    JPerMolK,
    "J/mol/K"
);
quantity!(
    /// Area per hour, m² h⁻¹ (the root uptake coefficient: root length density times it
    /// is the hourly fraction of a cell's available water the roots can take).
    M2PerHour,
    "m2/h"
);
quantity!(
    /// Mass flux per unit length, kg m⁻¹ h⁻¹ (most water a meter of root can take up in
    /// an hour).
    KgPerMH,
    "kg/m/h"
);
quantity!(
    /// Pressure in megapascals: a water potential (negative under suction; R3b) or a
    /// soil's penetration resistance (R5c).
    MPa,
    "MPa"
);
quantity!(
    /// Dry bulk density of soil, kg per m³ (R5c).
    KgPerM3,
    "kg/m3"
);
quantity!(
    /// Hydraulic conductance, kg of water per hour per megapascal of water potential
    /// difference (R3b).
    KgPerHourMPa,
    "kg/h/MPa"
);
quantity!(
    /// Angle in degrees.
    Degrees,
    "deg"
);
quantity!(
    /// Angle in radians.
    Radians,
    "rad"
);

/// Sim time in whole hours since scenario start. This is the tick.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Hours(pub u64);

impl Hours {
    /// Hours per sim-day.
    pub const PER_DAY: u64 = 24;

    /// Sim-day index (0-based) containing this tick.
    #[must_use]
    pub fn day(self) -> u64 {
        self.0 / Self::PER_DAY
    }

    /// Hour of the day, 0 to 23.
    #[must_use]
    pub fn hour_of_day(self) -> u64 {
        self.0 % Self::PER_DAY
    }

    /// The next tick.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for Hours {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} h", self.0)
    }
}

// ---------------------------------------------------------------------------
// Explicit conversions. Each one is the single place that conversion happens.
// ---------------------------------------------------------------------------

impl GramsDM {
    /// Carbon in this dry matter at the given carbon fraction (g C per g DM).
    #[must_use]
    pub fn to_carbon(self, carbon_fraction: f64) -> GramsC {
        GramsC(self.0 * carbon_fraction)
    }
}

impl GramsC {
    /// Dry matter holding this carbon at the given carbon fraction (g C per g DM).
    ///
    /// # Errors
    /// [`UnitError::OutOfRange`] when the fraction is not in (0, 1].
    pub fn to_dry_matter(self, carbon_fraction: f64) -> Result<GramsDM, UnitError> {
        if !(carbon_fraction > 0.0 && carbon_fraction <= 1.0) {
            return Err(UnitError::OutOfRange {
                unit: "g C / g DM",
                value: carbon_fraction,
                min: f64::MIN_POSITIVE,
                max: 1.0,
            });
        }
        GramsDM::try_new(self.0 / carbon_fraction)
    }
}

impl UmolPerM2S {
    /// Carbon fixed over `seconds` on `area` at this CO2 flux: 1 µmol CO2 = 12 µg C.
    #[must_use]
    pub fn carbon_over(self, area: M2, seconds: f64) -> GramsC {
        GramsC(self.0 * area.0 * seconds * CARBON_MOLAR_MASS_G_PER_MOL * 1e-6)
    }

    /// Carbon fixed in one sim hour on `area` (design plan §11.4).
    #[must_use]
    pub fn carbon_per_hour(self, area: M2) -> GramsC {
        self.carbon_over(area, SECONDS_PER_HOUR)
    }
}

impl Celsius {
    /// The same temperature in kelvin.
    #[must_use]
    pub fn to_kelvin(self) -> Kelvin {
        Kelvin(self.0 + KELVIN_OFFSET)
    }
}

impl Kelvin {
    /// The same temperature in Celsius.
    #[must_use]
    pub fn to_celsius(self) -> Celsius {
        Celsius(self.0 - KELVIN_OFFSET)
    }
}

impl Centimeters {
    /// The same length in meters.
    #[must_use]
    pub fn to_meters(self) -> Meters {
        Meters(self.0 / 100.0)
    }
}

impl Meters {
    /// The same length in centimeters.
    #[must_use]
    pub fn to_centimeters(self) -> Centimeters {
        Centimeters(self.0 * 100.0)
    }
}

impl KPa {
    /// The same pressure in megapascals.
    #[must_use]
    pub fn to_mpa(self) -> MPa {
        MPa(self.0 * 1e-3)
    }
}

impl MPa {
    /// The same pressure in kilopascals.
    #[must_use]
    pub fn to_kpa(self) -> KPa {
        KPa(self.0 * 1e3)
    }
}

impl Cm2 {
    /// The same area in square meters.
    #[must_use]
    pub fn to_m2(self) -> M2 {
        M2(self.0 * 1e-4)
    }
}

impl M2 {
    /// The same area in square centimeters.
    #[must_use]
    pub fn to_cm2(self) -> Cm2 {
        Cm2(self.0 * 1e4)
    }
}

impl Millimeters {
    /// Mass of water this depth represents over `area` (1 mm on 1 m² = 1 kg).
    #[must_use]
    pub fn to_kg_over(self, area: M2) -> KgWater {
        KgWater(self.0 * area.0)
    }
}

impl Degrees {
    /// The same angle in radians.
    #[must_use]
    pub fn to_radians(self) -> Radians {
        Radians(self.0 * core::f64::consts::PI / 180.0)
    }
}

impl Radians {
    /// The same angle in degrees.
    #[must_use]
    pub fn to_degrees(self) -> Degrees {
        Degrees(self.0 * 180.0 / core::f64::consts::PI)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn carbon_per_hour_matches_design_example() {
        // §11.4: A_net × area × 3600 × 12e-6.
        let c = UmolPerM2S(10.0).carbon_per_hour(M2(0.01));
        assert!((c.0 - 10.0 * 0.01 * 3600.0 * 12e-6).abs() < 1e-15);
    }

    #[test]
    fn try_new_rejects_non_finite() {
        assert!(GramsC::try_new(f64::NAN).is_err());
        assert!(GramsC::try_new(f64::INFINITY).is_err());
        assert_eq!(GramsC::try_new(1.5), Ok(GramsC(1.5)));
    }

    #[test]
    fn range_check() {
        assert!(Celsius::try_in_range(-300.0, -273.15, 100.0).is_err());
        assert!(Celsius::try_in_range(20.0, -273.15, 100.0).is_ok());
    }

    #[test]
    fn unit_strings_checked() {
        assert!(GramsC::check_unit("g C").is_ok());
        let err = GramsC::check_unit("g DM").unwrap_err();
        assert_eq!(err.to_string(), "expected unit \"g C\", found \"g DM\"");
    }

    #[test]
    fn carbon_fraction_validated() {
        assert!(GramsC(1.0).to_dry_matter(0.0).is_err());
        assert!(GramsC(1.0).to_dry_matter(1.5).is_err());
        assert!(GramsC(1.0).to_dry_matter(f64::NAN).is_err());
    }

    #[test]
    fn hours_day_and_hour() {
        assert_eq!(Hours(49).day(), 2);
        assert_eq!(Hours(49).hour_of_day(), 1);
    }

    #[test]
    fn serde_is_transparent() {
        assert_eq!(ron::to_string(&GramsC(2.5)).unwrap(), "2.5");
        let back: GramsC = ron::from_str("2.5").unwrap();
        assert_eq!(back, GramsC(2.5));
    }

    proptest! {
        #[test]
        fn dm_carbon_round_trip(dm in 0.0f64..1e6, cf in 0.01f64..1.0) {
            let back = GramsDM(dm).to_carbon(cf).to_dry_matter(cf).unwrap();
            prop_assert!((back.0 - dm).abs() <= 1e-12 * dm.max(1.0));
        }

        #[test]
        fn celsius_kelvin_round_trip(t in -100.0f64..100.0) {
            let back = Celsius(t).to_kelvin().to_celsius();
            prop_assert!((back.0 - t).abs() < 1e-12);
        }

        #[test]
        fn length_and_area_round_trip(x in 0.0f64..1e4) {
            prop_assert!((Centimeters(x).to_meters().to_centimeters().0 - x).abs() <= 1e-12 * x.max(1.0));
            prop_assert!((Cm2(x).to_m2().to_cm2().0 - x).abs() <= 1e-12 * x.max(1.0));
        }

        #[test]
        fn angle_round_trip(a in -720.0f64..720.0) {
            prop_assert!((Degrees(a).to_radians().to_degrees().0 - a).abs() < 1e-10);
        }

        #[test]
        fn add_sub_inverse(a in -1e6f64..1e6, b in -1e6f64..1e6) {
            let r = (GramsC(a) + GramsC(b)) - GramsC(b);
            prop_assert!((r.0 - a).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0));
        }
    }
}
