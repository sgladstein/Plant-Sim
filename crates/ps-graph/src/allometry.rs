//! Allometry: rendered dimensions from structural mass (§3.3, §3.4 secondary growth).
//!
//! - Leaves (and cotyledons): area = specific leaf area × mass.
//! - Internodes and roots: length = specific length × primary mass; radius from tissue
//!   density and the whole mass, `r = sqrt(mass / (ρ π L))` (R1a, decision 0036). With
//!   no secondary mass the radius is constant for a fixed specific length; secondary
//!   growth ([`Allometry::size_split`]) widens the organ at fixed length.
//! - Seeds, flowers, pods and buds: a prolate shape, `L = aspect^(2/3) · (6 m / (π ρ))^(1/3)`,
//!   radius `L / (2 · aspect)`.
//!
//! Model card: docs/models/organ-growth.md.

use ps_units::{GPerM3, GramsDM, M2, M2PerG, MPerG, Meters};
use serde::{Deserialize, Serialize};

use crate::OrganKind;

/// An organ's specific size: what one gram of structure buys.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpecificSize {
    /// Specific area (leaves, cotyledons).
    Area(M2PerG),
    /// Specific length (internodes, roots, tendrils).
    Length(MPerG),
    /// Shape from mass alone (seeds, flowers, pods, buds).
    Bulk,
}

impl SpecificSize {
    pub(crate) fn visit_bits(self, f: &mut impl FnMut(u64)) {
        match self {
            Self::Area(a) => {
                f(1);
                f(a.to_bits());
            }
            Self::Length(l) => {
                f(2);
                f(l.to_bits());
            }
            Self::Bulk => f(3),
        }
    }
}

/// Rendered dimensions.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganSize {
    /// Length (stems, roots, fruit long axis); zero for leaves.
    pub length: Meters,
    /// Radius (stems, roots, fruit); zero for leaves.
    pub radius: Meters,
    /// One-sided area (leaves); zero otherwise.
    pub area: M2,
}

/// Species allometry constants.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Allometry {
    /// Stem tissue density.
    pub stem_density: GPerM3,
    /// Root tissue density.
    pub root_density: GPerM3,
    /// Fruit, seed and flower density.
    pub bulk_density: GPerM3,
    /// Length over diameter of bulk organs (pods are long, seeds are near round).
    pub pod_aspect: f64,
    /// Length over diameter of seeds, flowers and buds.
    pub seed_aspect: f64,
}

impl Allometry {
    /// Placeholder values for tests and examples ("~"); species files override them.
    #[must_use]
    pub const fn generic_herb() -> Self {
        Self {
            stem_density: GPerM3(1.5e5),
            root_density: GPerM3(1.0e5),
            bulk_density: GPerM3(4.0e5),
            pod_aspect: 8.0,
            seed_aspect: 1.5,
        }
    }

    fn density(&self, kind: OrganKind) -> f64 {
        match kind {
            OrganKind::Root | OrganKind::Nodule => self.root_density.0,
            OrganKind::Internode | OrganKind::Tendril => self.stem_density.0,
            _ => self.bulk_density.0,
        }
    }

    /// Dimensions of a stem or root segment with structural mass `mass`, of which
    /// `secondary` is secondary growth: `L = SL · (W − W_s)`, `r = sqrt(W / (ρ π L))`
    /// (R1a). Other shapes ignore `secondary`; with `secondary` zero this is
    /// [`Allometry::size`].
    #[must_use]
    pub fn size_split(
        &self,
        kind: OrganKind,
        specific: SpecificSize,
        mass: GramsDM,
        secondary: GramsDM,
    ) -> OrganSize {
        let SpecificSize::Length(sl) = specific else {
            return self.size(kind, specific, mass);
        };
        let m = mass.0.max(0.0);
        let length = sl.0 * (m - secondary.0.max(0.0)).max(0.0);
        let radius = if length > 0.0 {
            libm::sqrt(m / (self.density(kind) * core::f64::consts::PI * length))
        } else {
            0.0
        };
        OrganSize {
            length: Meters(length),
            radius: Meters(radius),
            area: M2(0.0),
        }
    }

    /// Dimensions of an organ of `kind` and specific size with structural mass `mass`.
    #[must_use]
    pub fn size(&self, kind: OrganKind, specific: SpecificSize, mass: GramsDM) -> OrganSize {
        let m = mass.0.max(0.0);
        match specific {
            SpecificSize::Area(sla) => OrganSize {
                area: M2(sla.0 * m),
                ..OrganSize::default()
            },
            SpecificSize::Length(sl) => {
                let length = sl.0 * m;
                let rho = self.density(kind);
                let radius = if length > 0.0 {
                    libm::sqrt(m / (rho * core::f64::consts::PI * length))
                } else {
                    0.0
                };
                OrganSize {
                    length: Meters(length),
                    radius: Meters(radius),
                    area: M2(0.0),
                }
            }
            SpecificSize::Bulk => {
                let aspect = if kind == OrganKind::Pod {
                    self.pod_aspect
                } else {
                    self.seed_aspect
                };
                let rho = self.density(kind);
                // Prolate spheroid volume V = π L d² / 6 with L = aspect·d, so d = (6V/(π aspect))^(1/3).
                let v = m / rho;
                let d = libm::cbrt(6.0 * v / (core::f64::consts::PI * aspect));
                OrganSize {
                    length: Meters(aspect * d),
                    radius: Meters(0.5 * d),
                    area: M2(0.0),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_area_is_sla_times_mass() {
        let a = Allometry::generic_herb();
        // 250 cm² per g = 0.025 m² per g; 0.3 g → 75 cm².
        let s = a.size(
            OrganKind::Leaf,
            SpecificSize::Area(M2PerG(0.025)),
            GramsDM(0.3),
        );
        assert!((s.area.0 - 0.0075).abs() < 1e-15);
    }

    #[test]
    fn root_length_from_specific_root_length() {
        let a = Allometry::generic_herb();
        // 70 m per g (§3.16).
        let s = a.size(
            OrganKind::Root,
            SpecificSize::Length(MPerG(70.0)),
            GramsDM(0.01),
        );
        assert!((s.length.0 - 0.7).abs() < 1e-12);
        // Volume check: π r² L ρ = m.
        let m = core::f64::consts::PI * s.radius.0 * s.radius.0 * s.length.0 * 1.0e5;
        assert!((m - 0.01).abs() < 1e-12);
    }

    #[test]
    fn bulk_volume_matches_mass() {
        let a = Allometry::generic_herb();
        let s = a.size(OrganKind::Pod, SpecificSize::Bulk, GramsDM(1.2));
        let d = 2.0 * s.radius.0;
        let v = core::f64::consts::PI * s.length.0 * d * d / 6.0;
        assert!((v * 4.0e5 - 1.2).abs() < 1e-9);
        assert!((s.length.0 / d - 8.0).abs() < 1e-9);
    }

    #[test]
    fn secondary_mass_widens_at_fixed_length() {
        let a = Allometry::generic_herb();
        let sp = SpecificSize::Length(MPerG(0.4));
        let p = a.size(OrganKind::Internode, sp, GramsDM(0.1));
        // No secondary mass: the same bits as the one-mass rule.
        assert_eq!(
            a.size_split(OrganKind::Internode, sp, GramsDM(0.1), GramsDM(0.0)),
            p
        );
        // Tripling the mass with secondary growth keeps the length, widens by √3.
        let s = a.size_split(OrganKind::Internode, sp, GramsDM(0.3), GramsDM(0.2));
        assert!((s.length.0 - p.length.0).abs() < 1e-15);
        assert!((s.radius.0 / p.radius.0 - libm::sqrt(3.0)).abs() < 1e-12);
        // Leaves and bulk organs ignore it.
        let leaf = SpecificSize::Area(M2PerG(0.02));
        assert_eq!(
            a.size_split(OrganKind::Leaf, leaf, GramsDM(0.1), GramsDM(0.05)),
            a.size(OrganKind::Leaf, leaf, GramsDM(0.1))
        );
    }

    #[test]
    fn zero_mass_is_zero_size() {
        let a = Allometry::generic_herb();
        for sp in [
            SpecificSize::Area(M2PerG(0.02)),
            SpecificSize::Length(MPerG(1.0)),
            SpecificSize::Bulk,
        ] {
            let s = a.size(OrganKind::Internode, sp, GramsDM(0.0));
            assert_eq!(s, OrganSize::default());
        }
    }
}
