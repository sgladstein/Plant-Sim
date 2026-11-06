//! Organ positions from the graph (daily, for light, roots in soil and the renderer).
//!
//! Positions are in the soil grid's frame: x and y across the grid from its corner,
//! z up with 0 at the soil surface. Each organ has a base and a tip. An internode or root
//! starts where it attaches to its parent (the parent's tip, or a point along a root for
//! laterals) and runs along its direction for its length; leaves, buds and fruit sit at
//! their node.

use ps_graph::{OrganArena, OrganId, OrganKind};
use ps_light::LeafPoint;
use ps_soil::{Cell, RootCells, SoilGrid};
use ps_units::{M2, Meters};

use crate::{Meta, Role};

/// A 3-vector.
pub type V3 = [f64; 3];

/// `a + s·b`.
#[must_use]
pub fn add(a: V3, b: V3, s: f64) -> V3 {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

/// Euclidean norm.
#[must_use]
pub fn norm(a: V3) -> f64 {
    libm::sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2])
}

/// `a / |a|`, or straight up for a zero vector.
#[must_use]
pub fn unit(a: V3) -> V3 {
    let n = norm(a);
    if n > 1e-12 {
        [a[0] / n, a[1] / n, a[2] / n]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Cross product.
#[must_use]
pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Direction at angle `from_up` from `axis`, rotated by `azimuth` around it.
#[must_use]
pub fn deflect(axis: V3, from_up: f64, azimuth: f64) -> V3 {
    let w = unit(axis);
    // A reference not parallel to the axis.
    let r = if w[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = unit(cross(r, w));
    let v = cross(w, u);
    let (s, c) = (libm::sin(from_up), libm::cos(from_up));
    let (sa, ca) = (libm::sin(azimuth), libm::cos(azimuth));
    let side = add(add([0.0; 3], u, ca), v, sa);
    unit(add(add([0.0; 3], w, c), side, s))
}

/// Direction at `elevation` above the horizontal toward compass `azimuth`.
#[must_use]
pub fn from_azimuth(azimuth: f64, elevation: f64) -> V3 {
    let ce = libm::cos(elevation);
    [
        ce * libm::sin(azimuth),
        ce * libm::cos(azimuth),
        libm::sin(elevation),
    ]
}

/// Base and tip of every organ (dead ones included, at their last size).
#[must_use]
pub fn layout(arena: &OrganArena, meta: &[Meta], collar: V3) -> Vec<(V3, V3)> {
    let mut out: Vec<(V3, V3)> = Vec::with_capacity(arena.len());
    for id in arena.ids() {
        let i = id.index();
        let m = &meta[i];
        let base = match arena.parent[i] {
            None => collar,
            Some(p) => {
                let (pb, pt) = out[p.index()];
                if m.attach >= 1.0 {
                    pt
                } else {
                    add(pb, [pt[0] - pb[0], pt[1] - pb[1], pt[2] - pb[2]], m.attach)
                }
            }
        };
        let dir = arena.orientation[i].direction;
        let size = arena.size[i];
        let tip = match (arena.kind[i], m.twine) {
            // A coiled internode ends on its helix (WP22, decision 0021).
            (OrganKind::Internode, Some(t)) => t.point(base, size.length.0),
            (OrganKind::Internode | OrganKind::Root | OrganKind::Tendril, _) => {
                add(base, dir, size.length.0 / m.multiplicity.max(1.0))
            }
            // A grass ear stands on its peduncle (WP23).
            (OrganKind::Pod, _) if m.role == crate::Role::Ear => add(base, dir, size.length.0),
            _ => base,
        };
        out.push((base, tip));
    }
    out
}

/// Leaflet points of a leaf at node `node` for the light model: one for a unifoliate,
/// three for a trifoliate (terminal and two laterals).
#[must_use]
pub fn leaf_points(
    arena: &OrganArena,
    meta: &[Meta],
    id: OrganId,
    node: V3,
    petiole: Meters,
    potential: M2,
) -> Vec<LeafPoint> {
    let i = id.index();
    let area = arena.size[i].area.0;
    if area <= 0.0 {
        return Vec::new();
    }
    let dir = arena.orientation[i].direction;
    let h = unit([dir[0], dir[1], 0.0]);
    // Petiole grows with the blade: full length at full area.
    let scale = libm::sqrt((area / potential.0.max(1e-12)).min(1.0));
    let end = add(node, dir, petiole.0 * scale);
    // Blades tilt slightly toward their tip (normal leans back, about 20 degrees).
    let normal = unit([-0.35 * h[0], -0.35 * h[1], 1.0]);
    if meta[i].role == Role::Unifoliate {
        let r = libm::sqrt(area / core::f64::consts::PI);
        return vec![LeafPoint {
            center: add(end, h, r),
            area: M2(area),
            normal,
            strip: None,
        }];
    }
    let a3 = area / 3.0;
    let r = libm::sqrt(a3 / core::f64::consts::PI);
    let side = unit(cross([0.0, 0.0, 1.0], h));
    let terminal = add(end, h, 2.0 * r);
    vec![
        LeafPoint {
            center: terminal,
            area: M2(a3),
            normal,
            strip: None,
        },
        LeafPoint {
            center: add(add(end, side, 1.1 * r), h, 0.3 * r),
            area: M2(a3),
            normal,
            strip: None,
        },
        LeafPoint {
            center: add(add(end, side, -1.1 * r), h, 0.3 * r),
            area: M2(a3),
            normal,
            strip: None,
        },
    ]
}

/// The soil cell holding point `p`, if it is inside the grid and active.
#[must_use]
pub fn soil_cell(soil: &SoilGrid, p: V3) -> Option<usize> {
    let c = soil.cell_size().0;
    let (nx, ny, nz) = soil.dims();
    let fx = libm::floor(p[0] / c);
    let fy = libm::floor(p[1] / c);
    let fz = libm::floor(-p[2] / c);
    if fx < 0.0 || fy < 0.0 || fz < 0.0 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (x, y, z) = (fx as usize, fy as usize, fz as usize);
    if x >= nx || y >= ny || z >= nz {
        return None;
    }
    let i = soil.index(Cell { x, y, z });
    soil.is_active(i).then_some(i)
}

/// Root length per soil cell: each living root organ's length spread evenly along its
/// geometric extent, sampled every half cell (decision 0013). Length outside the soil is
/// dropped.
#[must_use]
pub fn root_cells(arena: &OrganArena, pos: &[(V3, V3)], soil: &SoilGrid) -> RootCells {
    root_cells_from(arena, pos, soil, 0)
}

/// [`root_cells`] for the root organs of axis order `min_order` and above (1: laterals
/// and fine roots, for the R2b nitrate-patch check).
#[must_use]
pub fn root_cells_from(
    arena: &OrganArena,
    pos: &[(V3, V3)],
    soil: &SoilGrid,
    min_order: u8,
) -> RootCells {
    let half = 0.5 * soil.cell_size().0;
    let mut pairs = Vec::new();
    for id in arena
        .alive_of(OrganKind::Root)
        .filter(|id| arena.axis_order[id.index()] >= min_order)
    {
        let i = id.index();
        let total = arena.size[i].length.0;
        if total <= 0.0 {
            continue;
        }
        let (b, t) = pos[i];
        let d = [t[0] - b[0], t[1] - b[1], t[2] - b[2]];
        let extent = norm(d);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = (libm::ceil(extent / half) as usize).max(1);
        let nf = f64::from(u32::try_from(n).unwrap_or(u32::MAX));
        for k in 0..n {
            let kf = f64::from(u32::try_from(k).unwrap_or(0));
            let p = add(b, d, (kf + 0.5) / nf);
            if let Some(c) = soil_cell(soil, p) {
                pairs.push((c, Meters(total / nf)));
            }
        }
    }
    RootCells::from_pairs(pairs)
}
