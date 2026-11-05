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
}

impl Placement {
    /// True for organs that extend along their direction (internodes, roots, tendrils).
    #[must_use]
    pub fn is_axis_organ(&self) -> bool {
        is_axis_kind(self.kind)
    }

    /// The far end of an axis organ; the base for point organs.
    #[must_use]
    pub fn tip(&self) -> Vec3 {
        if self.is_axis_organ() {
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
}

impl Skeleton {
    /// Places every organ of `arena` with the seed at `origin` (sim frame, m).
    #[must_use]
    pub fn from_arena(arena: &OrganArena, origin: [f64; 3]) -> Self {
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
                Placement {
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
                    length: finite_or_zero(size.length.0),
                    radius: finite_or_zero(size.radius.0),
                    area: finite_or_zero(size.area.0),
                    thermal_age: finite_or_zero(arena.thermal_age[i].0),
                }
            })
            .collect();
        let mut s = Self { origin, organs };
        s.relayout();
        s
    }

    /// Recomputes every base point from the parent links and the current sizes.
    pub fn relayout(&mut self) {
        for i in 0..self.organs.len() {
            let base = match self.organs[i].parent {
                Some(p) if p.index() < i => self.organs[p.index()].tip(),
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
