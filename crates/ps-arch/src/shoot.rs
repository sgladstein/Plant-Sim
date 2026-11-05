//! The determinate herb's shoot (§3.4, §3.10, §3.11): germination, emergence, nodes,
//! bud fate, racemes, flowers, pods and seeds, and senescence. Daily steps 2 to 4.

use core::f64::consts::{FRAC_PI_2, PI};

use ps_graph::{OrganId, OrganKind, SpecificSize};
use ps_physio::Stage;
use ps_physio::allocation;
use ps_rng::Purpose;
use ps_units::{DegreeDays, GramsC, GramsDM, Hours, M2PerG, MPerG};

use crate::geometry::{add, from_azimuth, unit};
use crate::{
    ArchError, Bud, BudFate, Climb, Flower, Plant, Pod, Raceme, Role, ShadeSignal, ShootAxis,
    Spawn, curve,
};

/// Elevation of a new petiole above the horizontal (~; decision 0013).
const PETIOLE_ELEVATION: f64 = 0.5;
/// Potential mass of a raceme's peduncle, g DM (~).
const PEDUNCLE_MASS: f64 = 0.03;
/// Peduncle length per gram, m/g (~; 3 cm for 0.03 g).
const PEDUNCLE_SPECIFIC_LENGTH: f64 = 1.0;
/// Potential mass of a flower, g DM (~).
const FLOWER_MASS: f64 = 0.01;
/// Flower expansion duration, °C·day (~).
const FLOWER_EXPANSION: f64 = 30.0;
/// Days of pollen development before a flower opens over which heat counts (decision
/// 0018).
const PRE_ANTHESIS_DAYS: usize = 7;

impl Plant {
    /// Starts the hypocotyl's expansion (germination, §3.10).
    pub(crate) fn germinate(&mut self) -> Result<(), ArchError> {
        let hp = &self.params;
        let length = self.depth.0 + hp.hypocotyl_above.0;
        let potential = GramsDM(length / hp.hypocotyl_specific_length.0);
        let h = self.hypocotyl.index();
        self.arena.curve[h] = curve(hp.internode_expansion, potential)?;
        self.arena.expansion_age[h] = DegreeDays(0.0);
        Ok(())
    }

    /// Emergence (VE): the epicotyl with its two opposite unifoliate leaves and their buds.
    pub(crate) fn emerge(&mut self, now: Hours) -> Result<(), ArchError> {
        self.emerged_day = Some(self.day);
        let az0 = self.individual.azimuth.0 + FRAC_PI_2;
        let n = if self.is_vine() {
            self.params.climbing.max_main_nodes
        } else {
            self.params.main_nodes
        };
        let epicotyl = self.internode(self.hypocotyl, [0.0, 0.0, 1.0], az0, 1, n, 0, now)?;
        let mut first_leaf = None;
        for k in 0..2u32 {
            let az = az0 + f64::from(k) * PI;
            let leaf = self.leaf(epicotyl, k, az, Role::Unifoliate, 1.0, 0, now)?;
            first_leaf.get_or_insert(leaf);
            self.bud(epicotyl, leaf, az, 1, now)?;
        }
        self.axes.push(ShootAxis {
            order: 0,
            base: epicotyl,
            apex: epicotyl,
            nodes: 1,
            max_nodes: n,
            since_node: DegreeDays(0.0),
            terminated: false,
            raceme: None,
            azimuth: az0 + FRAC_PI_2,
            last_leaf: first_leaf,
            climb: Climb::Free,
            free_length: 0.0,
        });
        Ok(())
    }

    /// A new internode at node `k` of an axis with `n` nodes, on `parent`.
    #[allow(clippy::too_many_arguments)]
    fn internode(
        &mut self,
        parent: OrganId,
        direction: [f64; 3],
        azimuth: f64,
        k: u32,
        n: u32,
        order: u8,
        now: Hours,
    ) -> Result<OrganId, ArchError> {
        let hp = &self.params;
        let factor = if order == 0 {
            hp.main_internode_factor(k, n)
        } else {
            hp.internode_profile[2]
        };
        // Shade lengthens an internode by making it thinner: the specific length rises
        // with the length, so the potential mass stays and a carbon-starved internode
        // is still longer per gram (decision 0018).
        let elongation = hp.shade_factor(self.shade_signal(parent), hp.shade_internode);
        // Each internode's own scatter, keyed by the organ it grows from (decision 0029).
        let z = self
            .stream(parent, now, Purpose::InternodeLength)
            .standard_normal();
        let length = hp.internode_length.0
            * factor
            * elongation
            * crate::scatter(z, hp.variation.internode_length_cv);
        let specific = MPerG(hp.stem_specific_length.0 * elongation);
        let potential = GramsDM(length / specific.0);
        let mut s = Spawn::new(
            OrganKind::Internode,
            Role::Stem,
            parent,
            curve(hp.internode_expansion, potential)?,
        );
        s.order = order;
        s.specific = SpecificSize::Length(specific);
        s.direction = direction;
        s.azimuth = azimuth;
        self.spawn(&s, now)
    }

    /// A leaf at the tip of internode `node`, facing `azimuth`, with size `factor` of
    /// the species' potential.
    #[allow(clippy::too_many_arguments)]
    fn leaf(
        &mut self,
        node: OrganId,
        index: u32,
        azimuth: f64,
        role: Role,
        factor: f64,
        order: u8,
        now: Hours,
    ) -> Result<OrganId, ArchError> {
        let hp = &self.params;
        let profile = if role == Role::Unifoliate {
            hp.unifoliate_area.0
        } else {
            hp.leaf_area.0 * factor
        };
        let area = profile * self.leaf_scatter(node, index, now);
        let shade = self.shade_signal(node);
        let sla = hp.sla.0 * hp.shade_factor(shade, hp.shade_sla);
        let mut s = Spawn::new(
            OrganKind::Leaf,
            role,
            node,
            curve(hp.leaf_expansion, GramsDM(area / sla))?,
        );
        s.order = order;
        s.specific = SpecificSize::Area(M2PerG(sla));
        s.direction = from_azimuth(azimuth, PETIOLE_ELEVATION);
        s.azimuth = azimuth;
        s.potential_area = area;
        self.spawn(&s, now)
    }

    /// The divergence angle to the next leaf of the axis whose newest node is `node`:
    /// the species' phyllotaxis with this node's scatter (decision 0027).
    pub(crate) fn divergence(&self, node: OrganId, now: Hours) -> f64 {
        let sd = self.params.variation.divergence_sd.0;
        let z = self
            .stream(node, now, Purpose::Phyllotaxis)
            .standard_normal();
        self.params.phyllotaxis.0 + sd * z.clamp(-2.5, 2.5)
    }

    /// The relative potential size of leaf `index` at node `node` (decision 0027),
    /// keyed by the node and the leaf's index on it (decision 0029), so each leaf has
    /// its own draw whatever organs are made around it.
    pub(crate) fn leaf_scatter(&self, node: OrganId, index: u32, now: Hours) -> f64 {
        let mut rng = self.stream(node, now, Purpose::LeafSize);
        let mut z = rng.standard_normal();
        for _ in 0..index {
            z = rng.standard_normal();
        }
        crate::scatter(z, self.params.variation.leaf_size_cv)
    }

    fn bud(
        &mut self,
        node: OrganId,
        leaf: OrganId,
        azimuth: f64,
        rank: u32,
        now: Hours,
    ) -> Result<(), ArchError> {
        let mut s = Spawn::new(
            OrganKind::Bud,
            Role::Bud,
            node,
            ps_graph::ExpansionCurve::none(),
        );
        s.direction = from_azimuth(azimuth, 1.0);
        s.azimuth = azimuth;
        let id = self.spawn(&s, now)?;
        self.buds.push(Bud {
            id,
            rank,
            leaf,
            azimuth,
            fate: BudFate::Dormant,
            axis: 0,
        });
        Ok(())
    }

    /// The shade signal at a new node: the red:far-red and yesterday's absorbed PAR
    /// of the newest living leaf on the axis below it that has seen a whole day (open
    /// sky if there is none).
    fn shade_signal(&self, node: OrganId) -> ShadeSignal {
        let a = &self.arena;
        let mut cur = Some(node);
        while let Some(id) = cur {
            let leaf = a.children[id.index()].iter().rev().find(|c| {
                a.alive[c.index()]
                    && a.kind[c.index()] == OrganKind::Leaf
                    && self.meta[c.index()].dli.is_some()
            });
            if let Some(l) = leaf {
                return ShadeSignal {
                    rfr: a.light[l.index()].rfr,
                    dli: self.meta[l.index()].dli,
                };
            }
            cur = a.parent[id.index()];
        }
        ShadeSignal::OPEN
    }

    /// Reserve concentration of an organ relative to `reference`, capped at 1.
    fn carbon_factor(&self, id: OrganId, reference: f64) -> f64 {
        let i = id.index();
        let w = self.arena.structural(id).0;
        if w <= 0.0 || !self.arena.alive[i] {
            return 0.0;
        }
        (self.arena.reserve[i].0 / w / reference.max(1e-12)).clamp(0.0, 1.0)
    }

    /// Daily step 2: bud fate (§3.4). A dormant bud on the lower main stem activates
    /// with probability `activation × light × carbon × dominance` and grows a branch.
    pub(crate) fn bud_fate(&mut self, now: Hours) -> Result<(), ArchError> {
        // Branching stops at flowering, when the pods become the dominant sinks
        // (decision 0013); remaining upper buds become racemes.
        if self.phenology.stage >= Stage::Flowering {
            return Ok(());
        }
        let Some(main) = self.axes.first().cloned() else {
            return Ok(());
        };
        let hp = &self.params;
        // Buds above the first `main_nodes - 2` ranks are kept for racemes (for a vine,
        // `main_nodes` is the node of the first raceme).
        let eligible = hp.main_nodes.saturating_sub(2);
        let mut grow = Vec::new();
        for (b, bud) in self.buds.iter().enumerate() {
            if bud.fate != BudFate::Dormant
                || bud.rank > eligible
                || !self.arena.alive[bud.id.index()]
            {
                continue;
            }
            let light = if self.arena.alive[bud.leaf.index()] {
                self.meta[bud.leaf.index()].light_index.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let node = self.arena.parent[bud.id.index()].unwrap_or(bud.id);
            let carbon = self.carbon_factor(node, self.params.bud_reserve_reference);
            // A cut apex (topping, WP25) frees the buds at once.
            let dominance = if main.terminated || !self.arena.alive[main.apex.index()] {
                1.0
            } else {
                // Buds of the top `hold_nodes` nodes stay dormant under a growing
                // apex; below that, dominance decays over `dominance_nodes` (WP25).
                let d = f64::from(main.nodes.saturating_sub(bud.rank)) - hp.hold_nodes;
                1.0 - libm::exp(-d.max(0.0) / hp.dominance_nodes)
            };
            let p = hp.bud_activation * light * carbon * dominance;
            if self.stream(bud.id, now, Purpose::BudFate).bernoulli(p) {
                grow.push(b);
            }
        }
        for b in grow {
            self.buds[b].fate = BudFate::Branch;
            let bud = self.buds[b].clone();
            let node = self.arena.parent[bud.id.index()].unwrap_or(self.hypocotyl);
            let up = self.arena.orientation[node.index()].direction;
            let dir = add(
                [0.0; 3],
                from_azimuth(bud.azimuth, FRAC_PI_2 - self.params.branch_angle.0),
                1.0,
            );
            let mut dir = unit(add(dir, up, 0.0));
            // A branch of a fallen vine grows along the ground (decision 0021).
            let lodged = self.is_vine() && self.axes[0].climb == Climb::Lodged;
            if lodged {
                dir = from_azimuth(bud.azimuth, 0.0);
            }
            let n = self.params.branch_nodes;
            let first = self.internode(bud.id, dir, bud.azimuth, 1, n, 1, now)?;
            let leaf = self.leaf(
                first,
                0,
                bud.azimuth,
                Role::Trifoliate,
                self.params.leaf_profile[2],
                1,
                now,
            )?;
            self.axes.push(ShootAxis {
                order: 1,
                base: first,
                apex: first,
                nodes: 1,
                max_nodes: n,
                since_node: DegreeDays(0.0),
                terminated: false,
                raceme: None,
                azimuth: bud.azimuth + self.divergence(first, now),
                last_leaf: Some(leaf),
                climb: if lodged { Climb::Lodged } else { Climb::Free },
                free_length: 0.0,
            });
        }
        Ok(())
    }

    /// Daily step 3, shoot part: each growing axis makes a node every phyllochron of
    /// air thermal time and ends in a terminal raceme after its last node.
    pub(crate) fn grow_axes(&mut self, now: Hours) -> Result<(), ArchError> {
        let thermal = DegreeDays(self.today.thermal);
        if self.flowering_started {
            self.since_flowering += thermal;
        }
        self.find_supports(now);
        let vine = self.is_vine();
        // A vine's main stem has a vegetative apex: it makes nodes until a set thermal
        // time after first flower (CROPGRO FL-VS) or its node cap, with no terminal
        // raceme (decision 0021).
        let vegetative_done =
            self.since_flowering.0 >= self.params.climbing.vegetative_after_flowering.0;
        for x in 0..self.axes.len() {
            let ax = self.axes[x].clone();
            if ax.terminated {
                continue;
            }
            if !self.arena.alive[ax.apex.index()] {
                self.axes[x].terminated = true;
                continue;
            }
            let mut ax = ax;
            ax.since_node += thermal;
            while ax.since_node.0 >= self.params.phyllochron.0 && !ax.terminated {
                ax.since_node.0 -= self.params.phyllochron.0;
                if vine && ax.order == 0 {
                    if ax.nodes < ax.max_nodes && !(self.flowering_started && vegetative_done) {
                        self.add_node(&mut ax, now)?;
                    } else {
                        ax.terminated = true;
                    }
                } else if ax.nodes < ax.max_nodes {
                    self.add_node(&mut ax, now)?;
                } else {
                    let r = self.raceme(ax.apex, ax.azimuth, now)?;
                    ax.raceme = Some(r);
                    ax.terminated = true;
                }
            }
            self.axes[x] = ax;
        }
        Ok(())
    }

    fn add_node(&mut self, ax: &mut ShootAxis, now: Hours) -> Result<(), ArchError> {
        let hp = &self.params;
        let prev = self.arena.orientation[ax.apex.index()].direction;
        let mut dir = unit(add(prev, [0.0, 0.0, 1.0], hp.shoot_gravitropism));
        let k = ax.nodes + 1;
        let (factor, rank) = if ax.order == 0 {
            (hp.leaf_factor(k - 1, ax.max_nodes - 1), Some(k))
        } else {
            (hp.leaf_profile[2], None)
        };
        let mut twine = None;
        if self.is_vine() {
            let length = if ax.order == 0 {
                hp.internode_length.0 * hp.main_internode_factor(k, ax.max_nodes)
            } else {
                hp.internode_length.0 * hp.internode_profile[2]
            };
            (dir, twine) = self.vine_internode(ax, dir, length, now);
        }
        let node = self.internode(ax.apex, dir, ax.azimuth, k, ax.max_nodes, ax.order, now)?;
        self.meta[node.index()].twine = twine;
        let leaf = self.leaf(node, 0, ax.azimuth, Role::Trifoliate, factor, ax.order, now)?;
        if let Some(r) = rank {
            self.bud(node, leaf, ax.azimuth, r, now)?;
            // An indeterminate stem flowers from the axils of the nodes it makes after
            // first flower.
            if self.is_vine()
                && self.flowering_started
                && let Some(b) = self.buds.last_mut()
            {
                b.fate = BudFate::Raceme;
                let (id, az) = (b.id, b.azimuth);
                self.raceme(id, az, now)?;
            }
        }
        ax.apex = node;
        ax.nodes = k;
        ax.last_leaf = Some(leaf);
        ax.azimuth += self.divergence(node, now);
        Ok(())
    }

    /// A raceme (its peduncle) on `parent`.
    fn raceme(&mut self, parent: OrganId, azimuth: f64, now: Hours) -> Result<OrganId, ArchError> {
        let ie = self.params.internode_expansion;
        let mut s = Spawn::new(
            OrganKind::Internode,
            Role::Raceme,
            parent,
            curve(ie, GramsDM(PEDUNCLE_MASS))?,
        );
        s.specific = SpecificSize::Length(ps_units::MPerG(PEDUNCLE_SPECIFIC_LENGTH));
        s.direction = from_azimuth(azimuth, 1.1);
        s.azimuth = azimuth;
        s.order = self.arena.axis_order[parent.index()];
        let id = self.spawn(&s, now)?;
        self.racemes.push(Raceme {
            id,
            opened: 0,
            due: 0.0,
        });
        Ok(id)
    }

    /// At flowering (R1), dormant buds on the upper main-stem nodes become racemes.
    pub(crate) fn start_racemes(&mut self, now: Hours) -> Result<(), ArchError> {
        if self.flowering_started || self.phenology.stage < Stage::Flowering {
            return Ok(());
        }
        self.flowering_started = true;
        let eligible = self.params.main_nodes.saturating_sub(2);
        for b in 0..self.buds.len() {
            let bud = self.buds[b].clone();
            if bud.fate == BudFate::Dormant
                && bud.rank > eligible
                && self.arena.alive[bud.id.index()]
            {
                self.buds[b].fate = BudFate::Raceme;
                self.raceme(bud.id, bud.azimuth, now)?;
            }
        }
        Ok(())
    }

    /// Daily step 3, reproductive part (§3.11): racemes open flowers over their
    /// flowering period; each flower sets a pod or aborts when it closes; pods make
    /// their seeds after the seed-fill delay.
    pub(crate) fn flowering(&mut self, now: Hours) -> Result<(), ArchError> {
        // Pollen and ovule damage follow the day's peak, not its mean, and act both
        // while the pollen develops and while the flower is open (decision 0018).
        let peak_t = if self.today.hours > 0 {
            self.today.t_max
        } else {
            20.0
        };
        let heat_today = self.params.pod_set_temperature(peak_t);
        self.recent_heat.push(heat_today);
        if self.recent_heat.len() > PRE_ANTHESIS_DAYS {
            self.recent_heat.remove(0);
        }
        if self.phenology.stage < Stage::Flowering {
            return Ok(());
        }
        if self.is_grass() {
            return self.set_grains(now);
        }
        let hp = self.params.clone();
        #[allow(clippy::cast_precision_loss)] // at most PRE_ANTHESIS_DAYS entries
        let pre_anthesis =
            self.recent_heat.iter().sum::<f64>() / self.recent_heat.len().max(1) as f64;
        // Open flowers.
        for r in 0..self.racemes.len() {
            let rc = self.racemes[r].clone();
            if !self.arena.alive[rc.id.index()] || rc.opened >= hp.flowers_per_raceme {
                continue;
            }
            let mut rc = rc;
            rc.due += f64::from(hp.flowers_per_raceme) / hp.raceme_days;
            while rc.due >= 1.0 && rc.opened < hp.flowers_per_raceme {
                rc.due -= 1.0;
                rc.opened += 1;
                self.flowers_opened += 1;
                let az = f64::from(rc.opened) * 2.4;
                let mut s = Spawn::new(
                    OrganKind::Flower,
                    Role::Flower,
                    rc.id,
                    curve(DegreeDays(FLOWER_EXPANSION), GramsDM(FLOWER_MASS))?,
                );
                s.direction = from_azimuth(az, -0.3);
                s.azimuth = az;
                let id = self.spawn(&s, now)?;
                self.flowers.push(Flower {
                    id,
                    raceme: rc.id,
                    days: 0,
                    temperature: 0.0,
                    carbon: 0.0,
                    light: 0.0,
                    pre_anthesis,
                    min_ftsw: 1.0,
                });
            }
            self.racemes[r] = rc;
        }
        // Close flowers: set or abort.
        let mut open = Vec::new();
        for mut f in std::mem::take(&mut self.flowers) {
            if !self.arena.alive[f.id.index()] {
                continue;
            }
            f.days += 1;
            f.temperature += heat_today;
            let node = self.arena.parent[f.raceme.index()].unwrap_or(f.raceme);
            f.carbon += self.carbon_factor(node, hp.pod_set_reserve_reference);
            // Shade raises flower and young-pod abscission (decision 0018).
            f.light += hp.shade_factor(self.shade_signal(node), hp.shade_pod_set);
            f.min_ftsw = f.min_ftsw.min(self.today.ftsw_min);
            if f.days < hp.flower_days {
                open.push(f);
                continue;
            }
            let days = f64::from(f.days);
            // Heat damages pollen as it develops and the flower while it is open; the
            // flower is as fertile as its worse stage, so an ordinary warm day is not
            // counted twice (decision 0018).
            let heat = f.pre_anthesis.min(f.temperature / days);
            let drought = (f.min_ftsw / hp.drought_abortion_ftsw.max(1e-9)).clamp(0.0, 1.0);
            let p = hp.pod_set_base * (f.carbon / days) * heat * (f.light / days) * drought;
            let set = self.stream(f.id, now, Purpose::PodSet).bernoulli(p);
            self.shed_organ(f.id, now)?;
            if set && self.arena.alive[f.raceme.index()] {
                let az = self.arena.orientation[f.id.index()].azimuth.0;
                let mut s = Spawn::new(
                    OrganKind::Pod,
                    Role::Pod,
                    f.raceme,
                    curve(hp.pod_expansion, hp.pod_wall)?,
                );
                s.direction = from_azimuth(az, -1.2);
                s.azimuth = az;
                let id = self.spawn(&s, now)?;
                self.pods.push(Pod {
                    id,
                    seeds: false,
                    fertility: heat,
                });
            }
        }
        self.flowers = open;
        self.make_seeds(now)
    }

    /// Pods make their seeds once past the seed-fill delay: each ovule sets with the
    /// species probability times the pod's fertility, at least one per pod.
    fn make_seeds(&mut self, now: Hours) -> Result<(), ArchError> {
        let hp = self.params.clone();
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
            let mut rng = self.stream(pod.id, now, Purpose::SeedNumber);
            let n = (0..hp.seeds_per_pod)
                .filter(|_| rng.bernoulli(hp.ovule_set * pod.fertility))
                .count()
                .max(1);
            for _ in 0..n {
                let s = Spawn::new(
                    OrganKind::SeedInPod,
                    Role::SeedInPod,
                    pod.id,
                    curve(hp.seed_fill, hp.seed_potential)?,
                );
                self.spawn(&s, now)?;
            }
        }
        Ok(())
    }

    /// Sheds an organ and what it carries; mobile reserve moves to the parent first.
    pub(crate) fn shed_organ(&mut self, id: OrganId, now: Hours) -> Result<(), ArchError> {
        let i = id.index();
        if !self.arena.alive[i] {
            return Ok(());
        }
        if let Some(p) = self.arena.parent[i] {
            let r = self.arena.reserve[i].0;
            if r > 0.0 && self.arena.alive[p.index()] {
                self.arena.reserve[i] = GramsC(0.0);
                self.arena.reserve[p.index()] += GramsC(r);
            }
        }
        let deaths = allocation::shed(
            &mut self.arena,
            &self.params.carbon,
            id,
            now,
            &mut self.ledger,
        )?;
        self.book_dead_nitrogen(&deaths);
        Ok(())
    }

    /// Daily step 4 (§3.9 step 6, §3.10, §3.11): starvation, leaf age, shade, frost,
    /// cotyledon drop and the end of the annual's life at maturity.
    pub(crate) fn senescence(&mut self, now: Hours) -> Result<(), ArchError> {
        let hp = self.params.clone();
        let mature = self.phenology.stage == Stage::Mature;
        // At maturity the annual's life ends: tissues dry and stop respiring
        // (`update_nitrogen`), so the harvest no longer starves (decision 0013).
        if !mature {
            let deaths = allocation::starvation_deaths(
                &mut self.arena,
                &self.params.carbon,
                now,
                &mut self.ledger,
            )?;
            self.book_dead_nitrogen(&deaths);
        }
        let frost = self.today.hours > 0 && self.today.t_min < hp.frost.0;
        let leaves: Vec<OrganId> = self.arena.alive_of(OrganKind::Leaf).collect();
        for id in leaves {
            let i = id.index();
            let m = &mut self.meta[i];
            if m.expanded.is_some() && m.light_index < hp.shade_light {
                m.shade_days += 1;
            } else {
                m.shade_days = 0;
            }
            // Lifespan counts from full expansion (species schema; WP8 review).
            let old = m
                .expanded
                .is_some_and(|x| self.arena.thermal_age[i].0 >= x.0 + hp.leaf_lifespan.0);
            if old || frost || mature || m.shade_days >= hp.shade_days {
                self.shed_organ(id, now)?;
            }
        }
        if mature {
            self.leaves_dropped = true;
        }
        // A killing frost also kills the growing points: apices stop, and buds and
        // flowers die, so the plant does not regrow (§7 M0 exit; decision 0018).
        if frost {
            for ax in &mut self.axes {
                ax.terminated = true;
            }
            let points: Vec<OrganId> = self
                .arena
                .alive_ids()
                .filter(|id| {
                    matches!(
                        self.arena.kind[id.index()],
                        OrganKind::Bud | OrganKind::Flower
                    )
                })
                .collect();
            for id in points {
                self.shed_organ(id, now)?;
            }
        }
        // Cotyledons drop when spent or two weeks after emergence.
        if let Some(e) = self.emerged_day {
            let cots: Vec<OrganId> = self.arena.alive_of(OrganKind::Cotyledon).collect();
            let left: f64 = cots.iter().map(|c| self.arena.reserve[c.index()].0).sum();
            let spent = left < hp.cotyledon_drop_reserve * self.seed_reserve.0;
            if spent || self.day >= e + hp.cotyledon_days {
                for c in cots {
                    self.shed_organ(c, now)?;
                }
            }
        }
        Ok(())
    }
}
