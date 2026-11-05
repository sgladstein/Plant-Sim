//! Organ placement: where each organ starts, which way it points, and how big it is.
//!
//! The organ graph stores topology, orientation and size but no positions, so placement
//! walks parent links in id order (a parent always has a smaller id than its children).
//!
//! Attachment convention (shared with whatever later computes leaf centers for the light
//! model; change both together):
//!
//! - Axis organs (internodes, roots, tendrils) start at the attachment point of their
//!   parent and extend `length` along their direction. Their attachment point is their tip.
//! - All other organs (seed, cotyledons, leaves, buds, flowers, pods, seeds in pods)
//!   attach at a point: their base is the parent's attachment point, and they hand that
//!   same point on to their children.
//! - An organ whose `attach` is below 1 on an axis parent starts that fraction of the way
//!   along it instead (ps-arch's lateral root cohorts; matches `ps_arch::geometry::layout`).
//! - The seed sits at the plant origin.
//!
//! A zero or non-finite direction falls back to straight up for shoots and straight down
//! for roots, so placement never produces NaN.

use glam::Vec3;
use ps_graph::{OrganArena, OrganId, OrganKind};

/// One organ, placed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Placement {
    /// Organ type.
    pub kind: OrganKind,
    /// Parent organ.
    pub parent: Option<OrganId>,
    /// Branching order.
    pub axis_order: u8,
    /// False once the organ has died.
    pub alive: bool,
    /// Base point, m.
    pub base: Vec3,
    /// Unit direction.
    pub direction: Vec3,
    /// Phyllotactic azimuth, rad.
    pub azimuth: f32,
    /// Rendered length, m (stems, roots, fruit long axis).
    pub length: f32,
    /// Rendered radius, m.
    pub radius: f32,
    /// One-sided area, m² (leaves, cotyledons).
    pub area: f32,
    /// Thermal age, °C·day.
    pub thermal_age: f32,
    /// Where along an axis parent the organ attaches, 0 (its base) to 1 (its tip).
    /// Lateral root cohorts attach partway along their parent.
    pub attach: f32,
    /// How many identical roots this organ is drawn as (a lateral root cohort), at most
    /// [`MAX_COPIES`]; 1 for everything else.
    pub copies: u16,
    /// The helix a twining internode coils along, if any. The placement itself is the
    /// chord from base to tip; the coil is drawn by sampling [`Coil::point`].
    pub coil: Option<Coil>,
}

/// A stem coiled around a vertical support (the sim's `Twine`), in the skeleton frame.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coil {
    /// The support's axis, x and y, m.
    pub center: glam::Vec2,
    /// Radius of the stem's centerline around it, m.
    pub radius: f32,
    /// Helix angle above the horizontal, rad.
    pub inclination: f32,
}

impl Coil {
    /// How many points [`Coil::samples`] returns for a coil of arc length `length`.
    #[must_use]
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    pub fn sample_count(&self, length: f32, per_turn: u32) -> usize {
        let r = self.radius.max(1e-6);
        let turns = length.max(0.0) * self.inclination.cos().abs() / (std::f32::consts::TAU * r);
        let n = (turns * per_turn as f32).ceil();
        if n.is_finite() {
            (n as usize).clamp(1, 512)
        } else {
            1
        }
    }

    /// The point `s` metres along the coil from `base`, anticlockwise seen from above.
    #[must_use]
    pub fn point(&self, base: Vec3, s: f32) -> Vec3 {
        let phase0 = (base.y - self.center.y).atan2(base.x - self.center.x);
        let r = self.radius.max(1e-6);
        let phase = phase0 + s * self.inclination.cos() / r;
        Vec3::new(
            self.center.x + r * phase.cos(),
            self.center.y + r * phase.sin(),
            base.z + s * self.inclination.sin(),
        )
    }

    /// Points along an organ coiled from `base` to `tip` over arc length `length`,
    /// excluding `base` and ending exactly at `tip`: about `per_turn` points a turn. Any
    /// gap between the helix's end and the chord's tip (from blending two hours) is
    /// spread along the coil so the stem stays joined.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn samples(&self, base: Vec3, tip: Vec3, length: f32, per_turn: u32) -> Vec<Vec3> {
        let n = self.sample_count(length, per_turn);
        let gap = tip - self.point(base, length);
        (1..=n)
            .map(|k| {
                let f = k as f32 / n as f32;
                self.point(base, length * f) + gap * f
            })
            .collect()
    }
}

/// Most copies drawn for one root cohort: enough to read as a cluster in the cutaway
/// without multiplying the root mesh budget.
pub const MAX_COPIES: u16 = 8;

impl Placement {
    /// True for organs that extend along their direction (internodes, roots, tendrils).
    #[must_use]
    pub fn is_axis_organ(&self) -> bool {
        is_axis_kind(self.kind)
    }

    /// The far end of an axis organ (on its coil for a twining internode); the base for
    /// point organs.
    #[must_use]
    pub fn tip(&self) -> Vec3 {
        if let Some(coil) = self.coil.filter(|_| self.kind == OrganKind::Internode) {
            coil.point(self.base, self.length)
        } else if self.is_axis_organ() {
            self.base + self.direction * self.length
        } else {
            self.base
        }
    }
}

/// True for kinds that extend along their direction.
#[must_use]
pub fn is_axis_kind(kind: OrganKind) -> bool {
    matches!(
        kind,
        OrganKind::Internode | OrganKind::Root | OrganKind::Tendril
    )
}

fn fallback_direction(kind: OrganKind) -> Vec3 {
    if kind == OrganKind::Root {
        Vec3::NEG_Z
    } else {
        Vec3::Z
    }
}

fn unit_or(v: Vec3, kind: OrganKind) -> Vec3 {
    let n = v.normalize_or_zero();
    if n == Vec3::ZERO {
        fallback_direction(kind)
    } else {
        n
    }
}

fn finite_or_zero(v: f64) -> f32 {
    if v.is_finite() {
        v.max(0.0) as f32
    } else {
        0.0
    }
}

/// Every organ of one plant, placed, indexed by organ id.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Skeleton {
    /// Where the seed sits, m.
    pub origin: Vec3,
    /// Placements by organ id (alive and dead).
    pub organs: Vec<Placement>,
    /// For a grass: blade length over width, so leaves draw as blades and pods as ears.
    /// `None` for a broadleaf.
    pub blade_aspect: Option<f32>,
}

impl Skeleton {
    /// Places every organ of `arena` with the seed at `origin` (sim frame, m), every
    /// organ attached at its parent's tip.
    #[must_use]
    pub fn from_arena(arena: &OrganArena, origin: [f64; 3]) -> Self {
        Self::from_arena_with(arena, origin, &[], &[])
    }

    /// Places every organ of `arena` with per-organ attachment fractions and root
    /// multiplicities (ps-arch's `Meta::attach` and `Meta::multiplicity`, indexed by organ
    /// id; missing entries mean 1). An axis organ standing for `m` identical roots is drawn
    /// as one root of `length / m`, as ps-arch's layout places it.
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // copies are clamped to 1..=MAX_COPIES first
    pub fn from_arena_with(
        arena: &OrganArena,
        origin: [f64; 3],
        attach: &[f64],
        multiplicity: &[f64],
    ) -> Self {
        let origin = Vec3::new(origin[0] as f32, origin[1] as f32, origin[2] as f32);
        let organs = arena
            .ids()
            .map(|id| {
                let i = id.index();
                let kind = arena.kind[i];
                let o = arena.orientation[i];
                let d = Vec3::new(
                    o.direction[0] as f32,
                    o.direction[1] as f32,
                    o.direction[2] as f32,
                );
                let size = arena.size[i];
                let m = multiplicity
                    .get(i)
                    .copied()
                    .filter(|m| m.is_finite())
                    .unwrap_or(1.0)
                    .max(1.0);
                let a = attach
                    .get(i)
                    .copied()
                    .filter(|a| a.is_finite())
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                let length = if is_axis_kind(kind) {
                    size.length.0 / m
                } else {
                    size.length.0
                };
                Placement {
                    coil: None,
                    kind,
                    parent: arena.parent[i],
                    axis_order: arena.axis_order[i],
                    alive: arena.alive[i],
                    base: Vec3::ZERO,
                    direction: unit_or(d, kind),
                    azimuth: if o.azimuth.0.is_finite() {
                        o.azimuth.0 as f32
                    } else {
                        0.0
                    },
                    length: finite_or_zero(length),
                    radius: finite_or_zero(size.radius.0),
                    area: finite_or_zero(size.area.0),
                    thermal_age: finite_or_zero(arena.thermal_age[i].0),
                    attach: a as f32,
                    copies: if kind == OrganKind::Root {
                        m.round().clamp(1.0, f64::from(MAX_COPIES)) as u16
                    } else {
                        1
                    },
                }
            })
            .collect();
        let mut s = Self {
            origin,
            organs,
            blade_aspect: None,
        };
        s.relayout();
        s
    }

    /// Recomputes every base point from the parent links and the current sizes.
    pub fn relayout(&mut self) {
        for i in 0..self.organs.len() {
            let attach = self.organs[i].attach;
            let base = match self.organs[i].parent {
                Some(p) if p.index() < i => {
                    let parent = &self.organs[p.index()];
                    if attach < 1.0 && parent.is_axis_organ() {
                        parent.base.lerp(parent.tip(), attach.max(0.0))
                    } else {
                        parent.tip()
                    }
                }
                _ => self.origin,
            };
            self.organs[i].base = base;
        }
    }

    /// Number of organs placed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.organs.len()
    }

    /// True when the plant has no organs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.organs.is_empty()
    }

    /// The placement of one organ.
    #[must_use]
    pub fn get(&self, id: OrganId) -> Option<&Placement> {
        self.organs.get(id.index())
    }

    /// Blends two snapshots of the same plant, `a` earlier and `b` later, at `t` in
    /// \[0, 1\] (fractional sim time between two ticks).
    ///
    /// Organs only in `b` were born during the interval and grow from zero size, so they
    /// appear without a jump. Sizes and directions blend linearly; positions are then
    /// re-derived from the blended sizes, so joints stay attached.
    #[must_use]
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = if t.is_finite() {
            t.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let organs = b
            .organs
            .iter()
            .enumerate()
            .map(|(i, pb)| {
                let mut p = *pb;
                if let Some(pa) = a.organs.get(i) {
                    p.length = lerp(pa.length, pb.length, t);
                    p.radius = lerp(pa.radius, pb.radius, t);
                    p.area = lerp(pa.area, pb.area, t);
                    p.thermal_age = lerp(pa.thermal_age, pb.thermal_age, t);
                    p.direction = unit_or(pa.direction.lerp(pb.direction, t), pb.kind);
                    p.azimuth = lerp(pa.azimuth, pb.azimuth, t);
                    // An organ that died during the interval stays visible until the end.
                    p.alive = pa.alive && (pb.alive || t < 1.0);
                } else {
                    p.length *= t;
                    p.radius *= t;
                    p.area *= t;
                    p.thermal_age *= t;
                }
                p
            })
            .collect();
        let mut s = Self {
            origin: a.origin.lerp(b.origin, t),
            organs,
            blade_aspect: b.blade_aspect,
        };
        s.relayout();
        s
    }

    /// Bounds `(min, max)` of every living organ's base and tip, or `None` if no organ is
    /// alive. Used for auto-framing.
    #[must_use]
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut it = self
            .organs
            .iter()
            .filter(|p| p.alive)
            .flat_map(|p| [p.base, p.tip()]);
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
