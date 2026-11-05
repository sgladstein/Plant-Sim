//! Science lenses (design plan §5.2, §5.8): what each lens colors, its quantity, unit,
//! scale and legend text, and the colorblind-safe color scales.
//!
//! Lenses change color and opacity only, never geometry. This module is plain data and
//! math, tested without a GPU.

use bevy::prelude::Resource;

/// A lens, or none.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Resource)]
pub enum Lens {
    /// Natural colors.
    #[default]
    Off,
    /// Absorbed PAR per leaf.
    Light,
    /// Fraction of transpirable soil water on the cut face.
    SoilWater,
    /// Plant water stress factor.
    PlantWater,
    /// Soil fades to 15%; roots by age.
    RootsXray,
    /// Reserve concentration per organ.
    Carbon,
    /// Thermal age per organ.
    Phenology,
    /// Red:far-red proxy per leaf and internode.
    ShadeSignal,
    /// Each plant by the light its leaves get, as a share of the open sky (M2, WP26).
    Competition,
    /// Organ temperature (M2, WP26).
    Temperature,
    /// Nitrogen per structural dry mass of each organ (M2, WP26).
    Nitrogen,
    /// Carbon moved into each organ from its parent this hour (M2, WP26), with arrows.
    CarbonFlow,
}

/// How a lens maps values to colors.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Scale {
    /// Viridis, low to high.
    Sequential,
    /// White to deep blue (water).
    SequentialBlue,
    /// Blue, white, orange, with white at `center`.
    Diverging {
        /// The value drawn white.
        center: f32,
    },
}

/// Whether the colored quantity is put into the model directly or comes out of it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Set by an equation or parameter.
    Encoded,
    /// Arises from interacting processes.
    Emergent,
}

/// What a lens shows, for its legend (§5.2: every lens has a legend with units, a
/// two-sentence explanation, an encoded-or-emergent tag and a model card link).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LensInfo {
    /// Short name.
    pub name: &'static str,
    /// What is colored.
    pub colors: &'static str,
    /// Quantity name.
    pub quantity: &'static str,
    /// Unit.
    pub unit: &'static str,
    /// Value at the low end of the scale.
    pub min: f32,
    /// Value at the high end of the scale.
    pub max: f32,
    /// Color scale.
    pub scale: Scale,
    /// Two sentences for the legend.
    pub explanation: &'static str,
    /// Encoded or emergent.
    pub origin: Origin,
    /// Model card in docs/models/.
    pub model_card: &'static str,
}

impl Lens {
    /// Every lens in panel order (the M1 set, then M2's).
    pub const ALL: [Self; 12] = [
        Self::Off,
        Self::Light,
        Self::SoilWater,
        Self::PlantWater,
        Self::RootsXray,
        Self::Carbon,
        Self::Phenology,
        Self::ShadeSignal,
        Self::Competition,
        Self::Temperature,
        Self::Nitrogen,
        Self::CarbonFlow,
    ];

    /// Legend data, or `None` for [`Lens::Off`].
    #[must_use]
    #[allow(clippy::too_many_lines)] // One legend per lens.
    pub fn info(self) -> Option<LensInfo> {
        let info = match self {
            Self::Off => return None,
            Self::Light => LensInfo {
                name: "Light",
                colors: "Leaves",
                quantity: "Absorbed PAR",
                unit: "µmol/m²/s",
                min: 0.0,
                max: 2000.0,
                scale: Scale::Sequential,
                explanation: "How much photosynthetically active light each leaf absorbs this hour, averaged over its sunlit and shaded parts. Leaves low in the canopy or in the shaded pot sit at the dark end.",
                origin: Origin::Emergent,
                model_card: "docs/models/light.md",
            },
            Self::SoilWater => LensInfo {
                name: "Soil water",
                colors: "Soil cells on the cut face",
                quantity: "Fraction of transpirable soil water (FTSW)",
                unit: "0 to 1",
                min: 0.0,
                max: 1.0,
                scale: Scale::SequentialBlue,
                explanation: "1 is field capacity and 0 is the wilting point. Roots dry the soil around them, so the colors follow where the roots are.",
                origin: Origin::Emergent,
                model_card: "docs/models/soil-and-water.md",
            },
            Self::PlantWater => LensInfo {
                name: "Plant water",
                colors: "Whole plant tint and leaf droop",
                quantity: "Water stress factor",
                unit: "0 to 1",
                min: 0.0,
                max: 1.0,
                scale: Scale::Sequential,
                explanation: "1 means the roots supply all the water the leaves could use; lower values close stomata and slow growth. Leaves droop as it falls.",
                origin: Origin::Emergent,
                model_card: "docs/models/soil-and-water.md",
            },
            Self::RootsXray => LensInfo {
                name: "Roots X-ray",
                colors: "Roots, with soil faded to 15%",
                quantity: "Root age",
                unit: "days",
                min: 0.0,
                max: 60.0,
                scale: Scale::Sequential,
                explanation: "The soil fades so the whole root system shows from any side. Young root tips are bright; older roots are dark.",
                origin: Origin::Emergent,
                model_card: "docs/models/organ-growth.md",
            },
            Self::Carbon => LensInfo {
                name: "Carbon",
                colors: "Organs",
                quantity: "Reserve concentration",
                unit: "g C per g dry matter",
                min: 0.0,
                max: 0.1,
                scale: Scale::Diverging { center: 0.05 },
                explanation: "White is the 5% safety reserve. Orange organs hold spare sugar; blue organs are drawing their reserve down and will starve if it stays low.",
                origin: Origin::Emergent,
                model_card: "docs/models/organ-growth.md",
            },
            Self::Phenology => LensInfo {
                name: "Phenology",
                colors: "Organs",
                quantity: "Thermal age",
                unit: "°C·day",
                min: 0.0,
                max: 1000.0,
                scale: Scale::Sequential,
                explanation: "Organs age by accumulated warmth above the base temperature, not by calendar days. Warm weeks age the plant faster.",
                origin: Origin::Encoded,
                model_card: "docs/models/organ-growth.md",
            },
            Self::ShadeSignal => LensInfo {
                name: "Shade signal",
                colors: "Leaves and internodes",
                quantity: "Red:far-red proxy",
                unit: "ratio",
                min: 0.2,
                max: 1.2,
                scale: Scale::Diverging { center: 0.7 },
                explanation: "Leaves absorb red light and pass far-red, so the ratio falls under foliage. Below 0.7 (blue) new internodes stretch and new leaves thin out.",
                origin: Origin::Encoded,
                model_card: "docs/models/light.md",
            },
            Self::Competition => LensInfo {
                name: "Competition",
                colors: "Each whole plant",
                quantity: "Light reaching its leaves, as a share of the open sky",
                unit: "0 to 1",
                min: 0.0,
                max: 1.0,
                scale: Scale::Sequential,
                explanation: "1 means its leaves absorb as much light as an unshaded leaf would this hour. Dark plants are shaded by their neighbours and their own upper leaves; the inspector gives each plant's share of the light on its soil.",
                origin: Origin::Emergent,
                model_card: "docs/models/light.md",
            },
            Self::Temperature => LensInfo {
                name: "Temperature",
                colors: "Organs",
                quantity: "Temperature the organ grew at this hour",
                unit: "°C",
                min: 5.0,
                max: 40.0,
                scale: Scale::Diverging { center: 25.0 },
                explanation: "Shoots follow the air and roots the soil layer they sit in, so roots run cooler by day and warmer at night. White is 25 °C, near the bean's optimum; blue is cool and orange is hot.",
                origin: Origin::Encoded,
                model_card: "docs/models/thermal-time-and-phenology.md",
            },
            Self::Nitrogen => LensInfo {
                name: "Nitrogen",
                colors: "Organs",
                quantity: "Nitrogen per structural dry mass",
                unit: "g N per g",
                min: 0.0,
                max: 0.06,
                scale: Scale::Sequential,
                explanation: "Young leaves are rich in nitrogen and old or shaded ones give theirs up to growing organs and seeds. A plant short of nitrogen dims from the bottom up.",
                origin: Origin::Emergent,
                model_card: "docs/models/nitrogen.md",
            },
            Self::CarbonFlow => LensInfo {
                name: "Carbon flow",
                colors: "Organs, with arrows along the stems",
                quantity: "Carbon moved in from the parent organ this hour",
                unit: "mg C/h",
                min: -1.0,
                max: 1.0,
                scale: Scale::Diverging { center: 0.0 },
                explanation: "Orange organs import sugar through the phloem and blue ones export it; arrows point the way it moves, longer for more. Leaves in the sun export and growing tips, roots and pods import.",
                origin: Origin::Emergent,
                model_card: "docs/models/carbon-allocation.md",
            },
        };
        Some(info)
    }

    /// Soil opacity under this lens (§4: X-ray fades soil to 15%).
    #[must_use]
    pub fn soil_opacity(self) -> f32 {
        if self == Self::RootsXray { 0.15 } else { 1.0 }
    }

    /// True when the lens colors plant organs (rather than only the soil).
    #[must_use]
    pub fn colors_organs(self) -> bool {
        !matches!(self, Self::Off | Self::SoilWater)
    }
}

impl LensInfo {
    /// Position of `value` on the scale, clamped to \[0, 1\]; NaN maps to `None`.
    #[must_use]
    pub fn normalize(&self, value: f32) -> Option<f32> {
        if !value.is_finite() {
            return None;
        }
        let span = self.max - self.min;
        if span <= 0.0 {
            return Some(0.0);
        }
        Some(((value - self.min) / span).clamp(0.0, 1.0))
    }

    /// Linear RGB color for `value`, or `None` when there is no value.
    #[must_use]
    pub fn color(&self, value: f32) -> Option<[f32; 3]> {
        let t = self.normalize(value)?;
        Some(match self.scale {
            Scale::Sequential => viridis(t),
            Scale::SequentialBlue => blues(t),
            Scale::Diverging { center } => {
                let c = self.normalize(center).unwrap_or(0.5);
                // Map so the center is exactly white.
                let u = if t < c {
                    0.5 * t / c.max(1e-6)
                } else {
                    0.5 + 0.5 * (t - c) / (1.0 - c).max(1e-6)
                };
                blue_white_orange(u)
            }
        })
    }
}

fn ramp(stops: &[[f32; 3]], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (t.floor() as usize).min(stops.len() - 2);
    let f = t - i as f32;
    let (a, b) = (stops[i], stops[i + 1]);
    [
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
    ]
}

/// sRGB stops to linear.
fn srgb(c: [u8; 3]) -> [f32; 3] {
    c.map(|v| {
        let x = f32::from(v) / 255.0;
        if x <= 0.040_45 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    })
}

/// Viridis (Smith and van der Walt), nine stops, linear RGB.
#[must_use]
pub fn viridis(t: f32) -> [f32; 3] {
    const STOPS: [[u8; 3]; 9] = [
        [68, 1, 84],
        [71, 44, 122],
        [59, 81, 139],
        [44, 113, 142],
        [33, 144, 141],
        [39, 173, 129],
        [92, 200, 99],
        [170, 220, 50],
        [253, 231, 37],
    ];
    ramp(&STOPS.map(srgb), t)
}

/// White to deep blue, linear RGB.
#[must_use]
pub fn blues(t: f32) -> [f32; 3] {
    const STOPS: [[u8; 3]; 5] = [
        [247, 251, 255],
        [198, 219, 239],
        [107, 174, 214],
        [33, 113, 181],
        [8, 48, 107],
    ];
    ramp(&STOPS.map(srgb), t)
}

/// Blue, white, orange (colorblind-safe diverging), linear RGB; white at 0.5.
#[must_use]
pub fn blue_white_orange(t: f32) -> [f32; 3] {
    const STOPS: [[u8; 3]; 5] = [
        [33, 102, 172],
        [146, 197, 222],
        [247, 247, 247],
        [244, 165, 130],
        [230, 97, 1],
    ];
    ramp(&STOPS.map(srgb), t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_lens_but_off_has_a_full_legend() {
        for lens in Lens::ALL {
            match lens.info() {
                None => assert_eq!(lens, Lens::Off),
                Some(i) => {
                    assert!(!i.unit.is_empty() && !i.quantity.is_empty());
                    assert!(i.max > i.min);
                    assert!(i.model_card.starts_with("docs/models/"));
                    // Two sentences.
                    assert_eq!(i.explanation.matches(". ").count() + 1, 2, "{}", i.name);
                }
            }
        }
    }

    #[test]
    fn diverging_center_is_white() {
        let shade = Lens::ShadeSignal.info().unwrap();
        let c = shade.color(0.7).unwrap();
        for v in c {
            assert!(v > 0.9, "{c:?}");
        }
        let carbon = Lens::Carbon.info().unwrap();
        assert!(carbon.color(0.05).unwrap().iter().all(|&v| v > 0.9));
        // Low is blue, high is orange.
        let lo = shade.color(0.2).unwrap();
        let hi = shade.color(1.2).unwrap();
        assert!(lo[2] > lo[0] && hi[0] > hi[2]);
    }

    #[test]
    fn scales_clamp_and_reject_nan() {
        let light = Lens::Light.info().unwrap();
        assert_eq!(light.color(-5.0), light.color(0.0));
        assert_eq!(light.color(9e9), light.color(2000.0));
        assert_eq!(light.color(f32::NAN), None);
        assert!(viridis(1.0)[1] > viridis(0.0)[1]);
    }

    #[test]
    fn only_xray_fades_soil() {
        for lens in Lens::ALL {
            let expected = if lens == Lens::RootsXray { 0.15 } else { 1.0 };
            assert!((lens.soil_opacity() - expected).abs() < f32::EPSILON);
        }
    }
}
