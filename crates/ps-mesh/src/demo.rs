//! A stand-in bush bean grown from the organ graph API alone.
//!
//! **Not the model.** Organ timing and sizes are scripted from rough bush bean numbers
//! (design plan §3.16) so the mesh tests and the viewer have a plausible, growing organ
//! graph before ps-world exists. Nothing here feeds back into the sim.
//!
//! [`demo_bean`] rebuilds the plant at a given hour from a fixed plan sorted by birth
//! time, so organ ids are stable across hours (organs born by hour `h` are always a
//! prefix of the plan), which is what [`crate::Skeleton::lerp`] needs.

use core::f64::consts::{PI, TAU};

use ps_graph::{
    Allometry, ExpansionCurve, NewOrgan, OrganArena, OrganId, OrganKind, Orientation, SpecificSize,
};
use ps_units::{DegreeDays, GramsC, GramsDM, GramsN, Hours, M2PerG, MPerG, Radians};

/// Knobs that make two demo plants differ the way the two-pot screen expects.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DemoParams {
    /// Scales leaf, pod and root mass (1 in full sun).
    pub vigor: f64,
    /// Scales internode length (shade avoidance stretches internodes).
    pub stretch: f64,
    /// Scales specific leaf area (shade leaves are thinner).
    pub sla_factor: f64,
    /// Number of axillary branches (0 to 3).
    pub branches: usize,
}

impl DemoParams {
    /// Full sun.
    pub const SUN: Self = Self {
        vigor: 1.0,
        stretch: 1.0,
        sla_factor: 1.0,
        branches: 3,
    };
    /// 50% shade: weaker, taller, thinner leaves, fewer branches.
    pub const SHADE: Self = Self {
        vigor: 0.6,
        stretch: 1.35,
        sla_factor: 1.25,
        branches: 1,
    };
}

/// Hours in a day.
const DAY: f64 = 24.0;
/// Planting depth of the seed, m (the plant origin is the seed; the soil surface is at z = 0).
pub const SEED_DEPTH: f64 = 0.03;

#[derive(Clone, Copy)]
struct Planned {
    kind: OrganKind,
    parent: Option<usize>,
    axis_order: u8,
    birth: f64,
    duration: f64,
    mass: f64,
    specific: SpecificSize,
    direction: [f64; 3],
    azimuth: f64,
    death: Option<f64>,
}

fn dir(azimuth: f64, elevation: f64) -> [f64; 3] {
    [
        elevation.cos() * azimuth.cos(),
        elevation.cos() * azimuth.sin(),
        elevation.sin(),
    ]
}

/// The sim tick nearest a non-negative hour.
#[allow(clippy::cast_sign_loss)] // clamped to non-negative first
fn tick(hour: f64) -> Hours {
    Hours(hour.round().max(0.0) as u64)
}

/// A deterministic wobble in \[−1, 1\] from an integer (no RNG needed for a demo).
fn wobble(k: usize) -> f64 {
    let x = (k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 11;
    (x as f64 / (1_u64 << 53) as f64) * 2.0 - 1.0
}

struct Planner {
    organs: Vec<Planned>,
    p: DemoParams,
}

impl Planner {
    fn push(&mut self, o: Planned) -> usize {
        self.organs.push(o);
        self.organs.len() - 1
    }

    fn leaf(&mut self, parent: usize, birth: f64, area_cm2: f64, azimuth: f64, order: u8) {
        let sla = 0.025 * self.p.sla_factor; // m² g⁻¹ (250 cm² g⁻¹)
        let area = area_cm2 * 1e-4 * self.p.vigor;
        self.push(Planned {
            kind: OrganKind::Leaf,
            parent: Some(parent),
            axis_order: order,
            birth,
            duration: 9.0 * DAY,
            mass: area / sla,
            specific: SpecificSize::Area(M2PerG(sla)),
            direction: dir(azimuth, 0.75),
            azimuth,
            death: None,
        });
    }

    fn internode(
        &mut self,
        parent: usize,
        birth: f64,
        length: f64,
        direction: [f64; 3],
        order: u8,
    ) -> usize {
        let sl = if order == 0 { 0.2 } else { 0.3 }; // m g⁻¹
        let length = length * self.p.stretch;
        self.push(Planned {
            kind: OrganKind::Internode,
            parent: Some(parent),
            axis_order: order,
            birth,
            duration: 5.0 * DAY,
            mass: length / sl,
            specific: SpecificSize::Length(MPerG(sl)),
            direction,
            azimuth: 0.0,
            death: None,
        })
    }

    fn root(
        &mut self,
        parent: usize,
        birth: f64,
        length: f64,
        direction: [f64; 3],
        order: u8,
    ) -> usize {
        let sl = if order == 0 { 5.0 } else { 20.0 };
        self.push(Planned {
            kind: OrganKind::Root,
            parent: Some(parent),
            axis_order: order,
            birth,
            duration: 2.0 * DAY,
            mass: length * self.p.vigor.sqrt() / sl,
            specific: SpecificSize::Length(MPerG(sl)),
            direction,
            azimuth: 0.0,
            death: None,
        })
    }

    fn fruit(&mut self, parent: usize, birth: f64, azimuth: f64, order: u8, k: usize) {
        let flower = self.push(Planned {
            kind: OrganKind::Flower,
            parent: Some(parent),
            axis_order: order,
            birth,
            duration: 3.0 * DAY,
            mass: 0.06,
            specific: SpecificSize::Bulk,
            direction: dir(azimuth, 0.3),
            azimuth,
            death: Some(birth + 6.0 * DAY),
        });
        // Fewer pods set on a weak plant.
        if wobble(k * 7 + 3).abs() > self.p.vigor {
            return;
        }
        self.push(Planned {
            kind: OrganKind::Pod,
            parent: Some(flower),
            axis_order: order,
            birth: birth + 5.0 * DAY,
            duration: 18.0 * DAY,
            mass: 1.5 * self.p.vigor,
            specific: SpecificSize::Bulk,
            direction: dir(azimuth, -1.1 + 0.2 * wobble(k)),
            azimuth,
            death: None,
        });
    }
}

fn plan(p: DemoParams) -> Vec<Planned> {
    let mut pl = Planner {
        organs: Vec::new(),
        p,
    };
    let seed = pl.push(Planned {
        kind: OrganKind::Seed,
        parent: None,
        axis_order: 0,
        birth: 0.0,
        duration: DAY,
        mass: 0.4,
        specific: SpecificSize::Bulk,
        direction: [1.0, 0.0, 0.0],
        azimuth: 0.0,
        death: None,
    });

    // Tap root and laterals.
    let mut parent = seed;
    let mut tap = Vec::new();
    for k in 0..14 {
        let d = [0.08 * wobble(k), 0.08 * wobble(k + 50), -1.0];
        parent = pl.root(parent, DAY * (1.0 + 1.2 * k as f64), 0.022, d, 0);
        tap.push(parent);
    }
    for (k, &t) in tap.iter().enumerate().skip(1).take(10) {
        for s in 0..3 {
            let az = TAU * (k as f64 * 0.37 + f64::from(s) / 3.0);
            let mut prev = t;
            let born = DAY * (4.0 + 1.5 * k as f64);
            for seg in 0..6 {
                let el = -0.5 - 0.08 * f64::from(seg);
                prev = pl.root(prev, born + DAY * f64::from(seg), 0.018, dir(az, el), 1);
            }
        }
    }

    // Hypocotyl, cotyledons, epicotyl, unifoliates.
    let hypocotyl = pl.internode(seed, 2.0 * DAY, 0.065, [0.0, 0.0, 1.0], 0);
    for az in [0.0, PI] {
        let sla = 0.01;
        pl.push(Planned {
            kind: OrganKind::Cotyledon,
            parent: Some(hypocotyl),
            axis_order: 0,
            birth: 5.0 * DAY,
            duration: 2.0 * DAY,
            mass: 8e-4 / sla,
            specific: SpecificSize::Area(M2PerG(sla)),
            direction: dir(az, 0.15),
            azimuth: az,
            death: Some(18.0 * DAY),
        });
    }
    let mut top = pl.internode(hypocotyl, 6.0 * DAY, 0.04, [0.02, 0.0, 1.0], 0);
    for az in [0.5 * PI, 1.5 * PI] {
        pl.leaf(top, 6.5 * DAY, 45.0, az, 0);
    }

    // Main stem: six trifoliate nodes, alternate at 137.5°, then a terminal raceme.
    let golden = 137.5_f64.to_radians();
    let mut branch_nodes = Vec::new();
    for k in 0..6 {
        let born = DAY * (10.0 + 4.0 * k as f64);
        let d = [0.06 * wobble(k + 10), 0.06 * wobble(k + 20), 1.0];
        top = pl.internode(top, born, 0.05 - 0.004 * k as f64, d, 0);
        let az = golden * k as f64;
        pl.leaf(top, born + 1.0 * DAY, 70.0, az, 0);
        if k < 3 {
            branch_nodes.push((top, born, az));
        }
    }
    let mut fruit_k = 0;
    for f in 0..4 {
        let az = golden * f64::from(f) + 0.4;
        pl.fruit(top, DAY * (38.0 + 2.0 * f64::from(f)), az, 0, fruit_k);
        fruit_k += 1;
    }

    // Axillary branches with their own leaves and fruit.
    for &(node, born, az) in branch_nodes.iter().take(p.branches) {
        let mut b = node;
        for s in 0..3 {
            let el = 0.9 + 0.15 * f64::from(s);
            let start = born + DAY * (12.0 + 4.0 * f64::from(s));
            b = pl.internode(b, start, 0.035, dir(az, el), 1);
            pl.leaf(b, start + DAY, 55.0, az + golden * (f64::from(s) + 1.0), 1);
        }
        for f in 0..3 {
            pl.fruit(
                b,
                born + DAY * (30.0 + 2.0 * f64::from(f)),
                az + f64::from(f),
                1,
                fruit_k,
            );
            fruit_k += 1;
        }
    }

    pl.organs
}

/// Grows the demo bean to `hour` (sim ticks since sowing).
#[must_use]
pub fn demo_bean(params: DemoParams, hour: u64) -> OrganArena {
    let allometry = Allometry::generic_herb();
    let planned = plan(params);
    // Stable birth order: organs born by any hour form a prefix, so ids never shift.
    let mut order: Vec<usize> = (0..planned.len()).collect();
    order.sort_by(|&a, &b| {
        planned[a]
            .birth
            .total_cmp(&planned[b].birth)
            .then(a.cmp(&b))
    });
    let mut id_of: Vec<Option<OrganId>> = vec![None; planned.len()];
    let h = hour as f64;
    let mut arena = OrganArena::new();
    for &i in &order {
        let o = &planned[i];
        if o.birth > h {
            break;
        }
        let x = ((h - o.birth) / o.duration).clamp(0.0, 1.0);
        let fraction = x * x * (3.0 - 2.0 * x);
        let parent = o.parent.and_then(|p| id_of[p]);
        let new = NewOrgan {
            kind: o.kind,
            parent,
            axis_order: o.axis_order,
            birth: tick(o.birth),
            curve: ExpansionCurve::new(
                DegreeDays(o.duration / DAY * 15.0),
                DegreeDays(o.duration / DAY * 7.5),
                GramsDM(o.mass),
            )
            .unwrap_or(ExpansionCurve::none()),
            specific: o.specific,
            orientation: Orientation {
                direction: o.direction,
                azimuth: Radians(o.azimuth),
            },
            structural: GramsDM(o.mass * fraction.max(1e-3)),
            reserve: GramsC(0.0),
            nitrogen: GramsN(0.0),
        };
        if let Ok(id) = arena.add(&new, &allometry) {
            arena.thermal_age[id.index()] = DegreeDays((h - o.birth) / DAY * 15.0);
            id_of[i] = Some(id);
        }
    }
    for (i, o) in planned.iter().enumerate() {
        if let (Some(id), Some(death)) = (id_of[i], o.death)
            && death <= h
        {
            let _ = arena.kill(id, tick(death));
        }
    }
    arena
}

/// The seed position for a demo plant whose pot is centered at `(x, y)` with the soil
/// surface at z = 0.
#[must_use]
pub fn demo_origin(x: f64, y: f64) -> [f64; 3] {
    [x, y, -SEED_DEPTH]
}
