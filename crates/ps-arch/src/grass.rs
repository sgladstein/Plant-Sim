//! The grass shoot (§3.4; WP23, decision 0022): a main stem and tillers, each node a
//! short crown internode with a leaf of sheath and blade, tiller sites that fill one
//! leaf interval after another, stem elongation from jointing, and a terminal ear on
//! a peduncle. The same organ graph, carbon and light as the herb.

use core::f64::consts::{FRAC_PI_2, TAU};

use ps_graph::{OrganId, OrganKind, SpecificSize};
use ps_physio::Stage;
use ps_rng::Purpose;
use ps_species::Grass;
use ps_units::{DegreeDays, GramsDM, Hours, M2, M2PerG, MPerG, Meters, Radians};
use serde::{Deserialize, Serialize};

use crate::geometry::{V3, add, cross, from_azimuth, unit};
use crate::{ApexSight, ArchError, Bud, BudFate, Climb, Plant, Pod, Role, ShootAxis, Spawn, curve};

const CM: f64 = 0.01;

/// How far a blade's tip droops below its base angle, rad (~): a wheat blade arches.
const BLADE_DROOP: f64 = 0.6;
/// Blade area over length times width (~ a tapering strap).
const BLADE_SHAPE: f64 = 0.75;

/// The grass shoot in model units (`grass` species group).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrassParams {
    /// Length of a crown internode.
    pub crown_internode: Meters,
    /// Main-stem internodes below the peduncle that elongate.
    pub elongating: u32,
    /// Sheath length of the first and of the flag leaf.
    pub sheath: (Meters, Meters),
    /// Sheath length per gram.
    pub sheath_specific_length: MPerG,
    /// Blade length over width.
    pub blade_aspect: f64,
    /// Blade angle above horizontal at its base.
    pub blade_elevation: Radians,
    /// Phyllochrons from a leaf to the tiller in its axil.
    pub tiller_delay: u32,
    /// Chance a tiller site grows a tiller when its time comes.
    pub site_filling: f64,
    /// Red:far-red below which no tiller starts.
    pub tiller_stop_rfr: f64,
    /// Angle of a new tiller from the vertical.
    pub tiller_angle: Radians,
    /// Azimuth of a tiller's first leaf from its parent's leaf.
    pub tiller_azimuth: Radians,
    /// Highest tiller order.
    pub max_order: u8,
    /// Tillers with fewer leaves die at jointing.
    pub survive_leaves: u32,
    /// Peduncle length.
    pub peduncle: Meters,
    /// Blade area of the first and of the flag leaf, as fractions of the largest.
    pub blade_fractions: (f64, f64),
}

fn round(x: f64) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = libm::round(x.max(0.0)) as u32;
    n
}

impl GrassParams {
    /// Converts the species group.
    #[must_use]
    pub fn new(g: &Grass) -> Self {
        Self {
            crown_internode: Meters(g.crown_internode_length.value * CM),
            elongating: round(g.elongating_internodes.value).max(1),
            sheath: (
                Meters(g.sheath_length_first.value * CM),
                Meters(g.sheath_length_last.value * CM),
            ),
            sheath_specific_length: MPerG(g.sheath_specific_length.value * CM),
            blade_aspect: g.blade_aspect.value,
            blade_elevation: Radians(g.blade_elevation.value.to_radians()),
            tiller_delay: round(g.tiller_delay.value).max(1),
            site_filling: g.site_filling.value,
            tiller_stop_rfr: g.tiller_stop_rfr.value,
            tiller_angle: Radians(g.tiller_angle.value.to_radians()),
            tiller_azimuth: Radians(g.tiller_azimuth.value.to_radians()),
            max_order: u8::try_from(round(g.tiller_max_order.value)).unwrap_or(2),
            survive_leaves: round(g.tiller_survive_leaves.value),
            peduncle: Meters(g.peduncle_length.value * CM),
            blade_fractions: (g.blade_first_fraction.value, g.blade_flag_fraction.value),
        }
    }

    /// Blade area factor of leaf `k` of `n` on an axis: rising linearly from the first
    /// fraction to 1 at leaf `n − 1`, the flag leaf at its own fraction.
    #[must_use]
    pub fn blade_factor(&self, k: u32, n: u32) -> f64 {
        let (first, flag) = self.blade_fractions;
        if k >= n && n > 1 {
            return flag;
        }
        let span = f64::from(n.saturating_sub(2).max(1));
        first + (1.0 - first) * (f64::from(k.saturating_sub(1)) / span).min(1.0)
    }

    /// Sheath length of leaf `k` of `n`: linear from the first to the flag leaf's.
    #[must_use]
    pub fn sheath_length(&self, k: u32, n: u32) -> f64 {
        let x = f64::from(k.saturating_sub(1)) / f64::from(n.saturating_sub(1).max(1));
        self.sheath.0.0 + (self.sheath.1.0 - self.sheath.0.0) * x.min(1.0)
    }

    /// Whether node `k` of an axis with `n` leaves carries an elongating internode.
    #[must_use]
    pub fn elongates(&self, k: u32, n: u32) -> bool {
        k >= 1 && k + self.elongating > n
    }
}

/// Light points of a grass blade (§3.5): three strips along an arching blade from
/// `base`, each a third of its area, facing up and away from the blade's tilt. Each
/// strip is sampled at three points along it (decision 0030).
#[must_use]
pub fn blade_points(area: f64, base: V3, direction: V3, aspect: f64) -> Vec<ps_light::LeafPoint> {
    if area <= 0.0 {
        return Vec::new();
    }
    let width = libm::sqrt(area / (BLADE_SHAPE * aspect.max(1e-9)));
    let length = aspect * width;
    let h = unit([direction[0], direction[1], 0.0]);
    let e0 = libm::asin(direction[2].clamp(-1.0, 1.0));
    let seg = length / 3.0;
    let mut p = base;
    let mut out = Vec::with_capacity(3);
    for j in 0..3 {
        let e = e0 - BLADE_DROOP * (f64::from(j) + 0.5) / 3.0;
        let d = [h[0] * libm::cos(e), h[1] * libm::cos(e), libm::sin(e)];
        let center = add(p, d, 0.5 * seg);
        // The upper face: perpendicular to the blade in its vertical plane.
        let side = unit(cross([0.0, 0.0, 1.0], h));
        let normal = unit(cross(d, side));
        out.push(ps_light::LeafPoint {
            center,
            area: M2(area / 3.0),
            normal: if normal[2] < 0.0 {
                [-normal[0], -normal[1], -normal[2]]
            } else {
                normal
            },
            // Sampled at the centers of the strip's thirds, along the blade
            // (decision 0030).
            strip: Some([d[0] * seg / 3.0, d[1] * seg / 3.0, d[2] * seg / 3.0]),
        });
        p = add(p, d, seg);
    }
    out
}

impl Plant {
    /// Whether the plant is a grass (the species has the grass group).
    #[must_use]
    pub fn is_grass(&self) -> bool {
        self.params.grass.is_some()
    }

    /// Emergence of a grass: the coleoptile has reached the surface and the first leaf
    /// unrolls from the crown, with a tiller site in its axil.
    pub(crate) fn grass_emerge(&mut self, now: Hours) -> Result<(), ArchError> {
        let Some(g) = self.params.grass.clone() else {
            return Ok(());
        };
        self.emerged_day = Some(self.day);
        let n = self.params.main_nodes;
        // Each plant's first blade faces its own way, so a row is not a fence of
        // parallel leaves (WALTer: plant azimuth random).
        let az0 = self.stream(self.hypocotyl, now, Purpose::Tropism).uniform() * TAU;
        let mut ax = ShootAxis {
            order: 0,
            base: self.hypocotyl,
            apex: self.hypocotyl,
            nodes: 0,
            max_nodes: n,
            since_node: DegreeDays(0.0),
            terminated: false,
            raceme: None,
            azimuth: az0,
            last_leaf: None,
            climb: Climb::Free,
            free_length: 0.0,
            sight: ApexSight::default(),
        };
        self.grass_node(&g, &mut ax, 0, [0.0, 0.0, 1.0], now)?;
        ax.base = ax.apex;
        self.axes.push(ax);
        Ok(())
    }

    /// A new node on axis `ax` (index `x`): its internode, a leaf of sheath and blade,
    /// and a tiller site if the node is a crown node and the axis may tiller.
    fn grass_node(
        &mut self,
        g: &GrassParams,
        ax: &mut ShootAxis,
        x: usize,
        direction: V3,
        now: Hours,
    ) -> Result<(), ArchError> {
        let hp = self.params.clone();
        let k = ax.nodes + 1;
        let n = ax.max_nodes;
        let (length, specific) = if g.elongates(k, n) {
            // Elongating internodes, from the lowest to the one under the peduncle.
            let j = k + g.elongating - n;
            let [first, last, _] = hp.internode_profile;
            let x = f64::from(j - 1) / f64::from(g.elongating.saturating_sub(1).max(1));
            let shade = hp.shade_factor(self.shade_signal_at(ax.apex), hp.shade_internode);
            // Each internode's own scatter, keyed by the node below it (decision 0029).
            let z = self
                .stream(ax.apex, now, Purpose::InternodeLength)
                .standard_normal();
            let own = crate::scatter(z, hp.variation.internode_length_cv);
            (
                hp.internode_length.0 * (first + (last - first) * x) * shade * own,
                MPerG(hp.stem_specific_length.0 * shade),
            )
        } else {
            (g.crown_internode.0, hp.stem_specific_length)
        };
        let mut s = Spawn::new(
            OrganKind::Internode,
            Role::Stem,
            ax.apex,
            curve(hp.internode_expansion, GramsDM(length / specific.0))?,
        );
        s.order = ax.order;
        s.specific = SpecificSize::Length(specific);
        s.direction = direction;
        s.azimuth = ax.azimuth;
        let node = self.spawn(&s, now)?;
        if g.elongates(k, n) {
            // An internode starts to elongate a phyllochron after its node appears, so
            // each lifts the next while the upper leaves come out (WALTer).
            self.arena.expansion_age[node.index()] = DegreeDays(-hp.phyllochron.0);
        }
        // The sheath rolls round the stem above the node; the blade opens at its top.
        let sheath_len = g.sheath_length(k, n);
        let mut sh = Spawn::new(
            OrganKind::Internode,
            Role::Sheath,
            node,
            curve(
                hp.leaf_expansion,
                GramsDM(sheath_len / g.sheath_specific_length.0),
            )?,
        );
        sh.order = ax.order;
        sh.specific = SpecificSize::Length(g.sheath_specific_length);
        sh.direction = direction;
        sh.azimuth = ax.azimuth;
        let sheath = self.spawn(&sh, now)?;
        let area = hp.leaf_area.0 * g.blade_factor(k, n) * self.leaf_scatter(node, 0, now);
        let shade = self.shade_signal_at(node);
        let sla = hp.sla.0 * hp.shade_factor(shade, hp.shade_sla);
        let mut b = Spawn::new(
            OrganKind::Leaf,
            Role::Blade,
            sheath,
            curve(hp.leaf_expansion, GramsDM(area / sla))?,
        );
        b.order = ax.order;
        b.specific = SpecificSize::Area(M2PerG(sla));
        b.direction = from_azimuth(ax.azimuth, g.blade_elevation.0);
        b.azimuth = ax.azimuth;
        b.potential_area = area;
        let blade = self.spawn(&b, now)?;
        if !g.elongates(k, n) && ax.order < g.max_order {
            let mut bs = Spawn::new(
                OrganKind::Bud,
                Role::Bud,
                node,
                ps_graph::ExpansionCurve::none(),
            );
            bs.direction = from_azimuth(ax.azimuth, 1.0);
            bs.azimuth = ax.azimuth;
            bs.order = ax.order;
            let id = self.spawn(&bs, now)?;
            self.buds.push(Bud {
                id,
                rank: k,
                leaf: blade,
                azimuth: ax.azimuth,
                fate: BudFate::Dormant,
                axis: x,
            });
        }
        ax.apex = node;
        ax.nodes = k;
        ax.last_leaf = Some(blade);
        ax.azimuth += self.divergence(node, now);
        Ok(())
    }

    /// The red:far-red and light at `node` for a new organ's shade response.
    fn shade_signal_at(&self, node: OrganId) -> crate::ShadeSignal {
        let a = &self.arena;
        // A grass node's leaf hangs off its sheath: look one level down too.
        let mut cur = Some(node);
        while let Some(id) = cur {
            for c in &a.children[id.index()] {
                let blades = a.children[c.index()].iter().chain(core::iter::once(c));
                for l in blades.rev() {
                    let i = l.index();
                    if a.alive[i] && a.kind[i] == OrganKind::Leaf && self.meta[i].dli.is_some() {
                        return crate::ShadeSignal {
                            rfr: a.light[i].rfr,
                            dli: self.meta[i].dli,
                        };
                    }
                }
            }
            cur = a.parent[id.index()];
        }
        crate::ShadeSignal::OPEN
    }

    /// Daily step 2 for a grass: tiller sites whose time has come fill or stay empty,
    /// and at jointing the youngest tillers die (WP23, decision 0022).
    pub(crate) fn tillering(&mut self, now: Hours) -> Result<(), ArchError> {
        if self.axes.is_empty() || self.phenology.stage >= Stage::Flowering {
            return Ok(());
        }
        let Some(g) = self.params.grass.clone() else {
            return Ok(());
        };
        let main = self.axes[0].clone();
        // Jointing: the first elongating internode starts to grow, a phyllochron after
        // its node.
        let jointed = main.nodes >= 1 && g.elongates(main.nodes - 1, main.max_nodes);
        if jointed {
            if !self.jointed {
                self.jointed = true;
                self.kill_small_tillers(g.survive_leaves, now)?;
            }
            for b in &mut self.buds {
                if b.fate == BudFate::Dormant {
                    b.fate = BudFate::Suppressed;
                }
            }
            return Ok(());
        }
        let mut grow = Vec::new();
        for (b, bud) in self.buds.iter().enumerate() {
            if bud.fate != BudFate::Dormant || !self.arena.alive[bud.id.index()] {
                continue;
            }
            let ax = &self.axes[bud.axis];
            if ax.nodes < bud.rank + g.tiller_delay {
                continue;
            }
            let node = self.arena.parent[bud.id.index()].unwrap_or(bud.id);
            let signal = self.shade_signal_at(node);
            let leaves = (ax.max_nodes + 1).saturating_sub(bud.rank + g.tiller_delay);
            let p = if signal.rfr < g.tiller_stop_rfr || leaves < 2 {
                0.0
            } else {
                g.site_filling * self.carbon_ratio(node, self.params.bud_reserve_reference)
            };
            let fill = self.stream(bud.id, now, Purpose::BudFate).bernoulli(p);
            grow.push((b, fill, leaves));
        }
        for (b, fill, leaves) in grow {
            if !fill {
                self.buds[b].fate = BudFate::Suppressed;
                continue;
            }
            self.buds[b].fate = BudFate::Branch;
            let bud = self.buds[b].clone();
            let parent = self.axes[bud.axis].clone();
            // A tiller's leaves turn away from its parent's (WALTer: 40 deg).
            let azimuth = bud.azimuth + g.tiller_azimuth.0;
            let dir = from_azimuth(azimuth, FRAC_PI_2 - g.tiller_angle.0);
            let mut ax = ShootAxis {
                order: parent.order + 1,
                base: bud.id,
                apex: bud.id,
                nodes: 0,
                max_nodes: leaves,
                since_node: DegreeDays(0.0),
                terminated: false,
                raceme: None,
                azimuth,
                last_leaf: None,
                climb: Climb::Free,
                free_length: 0.0,
                sight: ApexSight::default(),
            };
            let x = self.axes.len();
            self.grass_node(&g, &mut ax, x, dir, now)?;
            ax.base = self.arena.children[bud.id.index()]
                .first()
                .copied()
                .unwrap_or(ax.apex);
            self.axes.push(ax);
        }
        Ok(())
    }

    /// Reserve concentration of `id` over `reference`, capped at 1.
    fn carbon_ratio(&self, id: OrganId, reference: f64) -> f64 {
        let i = id.index();
        let w = self.arena.structural(id).0;
        if w <= 0.0 || !self.arena.alive[i] {
            return 0.0;
        }
        (self.arena.reserve[i].0 / w / reference.max(1e-12)).clamp(0.0, 1.0)
    }

    /// At jointing, tillers with fewer than `min` leaves die with what they carry.
    fn kill_small_tillers(&mut self, min: u32, now: Hours) -> Result<(), ArchError> {
        for x in 1..self.axes.len() {
            let ax = self.axes[x].clone();
            if ax.nodes < min && self.arena.alive[ax.base.index()] {
                self.axes[x].terminated = true;
                self.shed_organ(ax.base, now)?;
            }
        }
        Ok(())
    }

    /// Daily step 3, shoot part, for a grass: each axis makes a leaf every phyllochron
    /// and, after its last, a peduncle with the ear.
    pub(crate) fn grow_tillers(&mut self, now: Hours) -> Result<(), ArchError> {
        let Some(g) = self.params.grass.clone() else {
            return Ok(());
        };
        let thermal = DegreeDays(self.today.thermal);
        for x in 0..self.axes.len() {
            let mut ax = self.axes[x].clone();
            if ax.terminated {
                continue;
            }
            if !self.arena.alive[ax.apex.index()] {
                self.axes[x].terminated = true;
                continue;
            }
            ax.since_node += thermal;
            while ax.since_node.0 >= self.params.phyllochron.0 && !ax.terminated {
                ax.since_node.0 -= self.params.phyllochron.0;
                if ax.nodes < ax.max_nodes {
                    let prev = self.arena.orientation[ax.apex.index()].direction;
                    let dir = unit(add(prev, [0.0, 0.0, 1.0], self.params.shoot_gravitropism));
                    self.grass_node(&g, &mut ax, x, dir, now)?;
                } else {
                    let ear = self.ear(&g, &ax, now)?;
                    ax.raceme = Some(ear);
                    ax.terminated = true;
                }
            }
            self.axes[x] = ax;
        }
        Ok(())
    }

    /// Heading: the peduncle and the ear at the top of an axis.
    fn ear(&mut self, g: &GrassParams, ax: &ShootAxis, now: Hours) -> Result<OrganId, ArchError> {
        let hp = self.params.clone();
        let prev = self.arena.orientation[ax.apex.index()].direction;
        let dir = unit(add(prev, [0.0, 0.0, 1.0], 1.0));
        let mut s = Spawn::new(
            OrganKind::Internode,
            Role::Peduncle,
            ax.apex,
            curve(
                hp.internode_expansion,
                GramsDM(g.peduncle.0 / hp.stem_specific_length.0),
            )?,
        );
        s.order = ax.order;
        s.specific = SpecificSize::Length(hp.stem_specific_length);
        s.direction = dir;
        s.azimuth = ax.azimuth;
        let peduncle = self.spawn(&s, now)?;
        let mut e = Spawn::new(
            OrganKind::Pod,
            Role::Ear,
            peduncle,
            curve(hp.pod_expansion, hp.pod_wall)?,
        );
        e.order = ax.order;
        e.direction = dir;
        e.azimuth = ax.azimuth;
        let id = self.spawn(&e, now)?;
        self.pods.push(Pod {
            id,
            seeds: false,
            fertility: 1.0,
            pollen: None,
        });
        Ok(id)
    }

    /// Grain set in each ear past the lag after heading: of the species' florets,
    /// each sets with the floret set chance times the heat factor of the days around
    /// anthesis and the ear's carbon (decision 0022), at least one per ear.
    pub(crate) fn set_grains(&mut self, now: Hours) -> Result<(), ArchError> {
        let hp = self.params.clone();
        #[allow(clippy::cast_precision_loss)] // a week of entries
        let heat = self.recent_heat.iter().sum::<f64>() / self.recent_heat.len().max(1) as f64;
        for k in 0..self.pods.len() {
            let pod = self.pods[k].clone();
            let i = pod.id.index();
            if pod.seeds
                || !self.arena.alive[i]
                || self.arena.expansion_age[i].0 < hp.seed_fill_delay.0
            {
                continue;
            }
            self.pods[k].seeds = true;
            let carbon = self.carbon_ratio(pod.id, hp.pod_set_reserve_reference);
            let p = hp.ovule_set * heat * carbon;
            self.pods[k].fertility = heat;
            let mut rng = self.stream(pod.id, now, Purpose::SeedNumber);
            let n = (0..hp.seeds_per_pod)
                .filter(|_| rng.bernoulli(p))
                .count()
                .max(1);
            for _ in 0..n {
                let s = Spawn::new(
                    OrganKind::SeedInPod,
                    Role::Grain,
                    pod.id,
                    curve(hp.seed_fill, hp.seed_potential)?,
                );
                self.spawn(&s, now)?;
            }
        }
        Ok(())
    }

    /// Living tillers (axes above the main stem).
    #[must_use]
    pub fn living_tillers(&self) -> u32 {
        let n = self
            .axes
            .iter()
            .filter(|x| x.order > 0 && self.arena.alive[x.base.index()])
            .count();
        u32::try_from(n).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A grass group for the unit tests.
    fn default_grass() -> Grass {
        let p = |v: f64| ps_species::Param {
            value: v,
            unit: String::new(),
            source: String::new(),
        };
        Grass {
            crown_internode_length: p(0.3),
            elongating_internodes: p(4.0),
            sheath_length_first: p(3.0),
            sheath_length_last: p(15.0),
            sheath_specific_length: p(150.0),
            blade_aspect: p(18.0),
            blade_elevation: p(40.0),
            tiller_delay: p(3.0),
            site_filling: p(0.7),
            tiller_stop_rfr: p(0.35),
            tiller_angle: p(15.0),
            tiller_azimuth: p(40.0),
            tiller_max_order: p(2.0),
            tiller_survive_leaves: p(3.0),
            peduncle_length: p(30.0),
            blade_first_fraction: p(0.15),
            blade_flag_fraction: p(0.8),
        }
    }

    #[test]
    fn blades_grow_with_rank_and_the_flag_is_smaller() {
        let g = GrassParams::new(&default_grass());
        let f: Vec<f64> = (1..=8).map(|k| g.blade_factor(k, 8)).collect();
        assert!((f[0] - 0.15).abs() < 1e-12);
        assert!(f.windows(2).take(6).all(|w| w[1] > w[0]));
        assert!((f[6] - 1.0).abs() < 1e-12);
        assert!((f[7] - 0.8).abs() < 1e-12);
        assert!(g.elongates(5, 8) && !g.elongates(4, 8));
    }

    #[test]
    fn blade_points_hold_the_area_and_face_up() {
        let pts = blade_points(30e-4, [0.0; 3], from_azimuth(0.0, 0.7), 18.0);
        assert_eq!(pts.len(), 3);
        let a: f64 = pts.iter().map(|p| p.area.0).sum();
        assert!((a - 30e-4).abs() < 1e-15);
        assert!(pts.iter().all(|p| p.normal[2] > 0.0));
        // The blade arches out and back down: the tip strip sits farther out.
        assert!(pts[2].center[1] > pts[0].center[1]);
    }
}
