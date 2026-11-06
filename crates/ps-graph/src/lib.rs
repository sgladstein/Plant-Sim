//! The organ graph (design plan §3.3, WP3).
//!
//! One [`OrganArena`] per plant: structs of arrays indexed by [`OrganId`], with parent
//! pointers and children lists in creation order, so every traversal is deterministic.
//! Organs are never removed; death sets `alive = false` and records the death time,
//! so ids stay stable for the action log, the inspector and replays.
//!
//! Invariant (property-tested): an organ's structural mass never shrinks. At death it is
//! frozen; resorption and litter are booked by the caller from the returned mass.
//!
//! Structural mass is primary plus secondary (R1a, decision 0036): secondary growth
//! (stem and root thickening) widens a stem or root segment at fixed length, so the
//! length follows the primary mass and the radius the total.

pub mod allometry;
pub mod curve;

pub use allometry::{Allometry, OrganSize, SpecificSize};
pub use curve::{CurveError, ExpansionCurve};

use ps_units::{DegreeDays, GramsC, GramsDM, GramsN, Hours, Meters, Radians};
use serde::{Deserialize, Serialize};

/// Index of an organ within its plant's arena.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrganId(pub u32);

impl OrganId {
    /// The index into the arena's arrays.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Organ types (§3.3). `Nodule` and `Tendril` arrive in M2.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OrganKind {
    /// The seed before and during germination.
    Seed,
    /// Seed leaf holding the seed reserve.
    Cotyledon,
    /// Stem segment between nodes (the hypocotyl is the first).
    Internode,
    /// Axillary or apical bud.
    Bud,
    /// Leaf blade plus petiole (unifoliate or trifoliate).
    Leaf,
    /// Root segment.
    Root,
    /// Flower.
    Flower,
    /// Pod.
    Pod,
    /// Seed developing inside a pod.
    SeedInPod,
    /// Root nodule (M2).
    Nodule,
    /// Tendril (M2).
    Tendril,
}

/// Base direction of an organ and its phyllotactic azimuth.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Orientation {
    /// Unit vector (east, north, up).
    pub direction: [f64; 3],
    /// Phyllotactic azimuth around the parent axis.
    pub azimuth: Radians,
}

impl Orientation {
    /// Straight up, azimuth zero.
    pub const UP: Self = Self {
        direction: [0.0, 0.0, 1.0],
        azimuth: Radians(0.0),
    };
    /// Straight down, azimuth zero.
    pub const DOWN: Self = Self {
        direction: [0.0, 0.0, -1.0],
        azimuth: Radians(0.0),
    };
}

/// Light coefficients for a leaf, rebuilt daily by ps-light (§3.5).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightCoeffs {
    /// Probability the leaf is sunlit at each of the 24 hourly sun positions.
    pub direct: [f64; 24],
    /// The leaf's projection toward the sun at each hour, `|n · s|` for blade normal
    /// `n`, averaged over its sunlit leaflets: the beam it absorbs per unit leaf area
    /// as a share of the beam's flux normal to the sun (decision 0025).
    #[serde(default)]
    pub beam: [f64; 24],
    /// Diffuse-sky transmittance to the leaf: the share of horizontal diffuse
    /// irradiance that would reach a horizontal surface at the leaf (1 in the open).
    pub diffuse: f64,
    /// Diffuse irradiance absorbed per unit leaf area, from the leaf's own projection
    /// toward each sky sector, as a share of horizontal diffuse irradiance before
    /// absorptance (about 1 for a horizontal leaf in the open, decision 0025).
    pub diffuse_absorbed: f64,
    /// Sunlit fraction for the current hour.
    pub sunlit_fraction: f64,
    /// Red:far-red proxy.
    pub rfr: f64,
}

impl Default for LightCoeffs {
    fn default() -> Self {
        Self {
            direct: [0.0; 24],
            beam: [0.0; 24],
            diffuse: 0.0,
            diffuse_absorbed: 0.0,
            sunlit_fraction: 0.0,
            rfr: 1.2,
        }
    }
}

/// Errors from graph operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GraphError {
    /// No organ has this id.
    #[error("no organ {0:?}")]
    NoSuchOrgan(OrganId),
    /// The organ is dead.
    #[error("organ {0:?} is dead")]
    Dead(OrganId),
    /// Structural mass may only grow.
    #[error("organ {id:?}: structural increment {delta} g DM must be non-negative and finite")]
    NegativeGrowth {
        /// The organ.
        id: OrganId,
        /// The rejected increment.
        delta: f64,
    },
    /// The arena is full (more than `u32::MAX` organs).
    #[error("organ arena is full")]
    Full,
    /// A starch store may not go negative (R1b).
    #[error(
        "organ {id:?}: store change {delta} g C must be finite and leave the store non-negative"
    )]
    NegativeStore {
        /// The organ.
        id: OrganId,
        /// The rejected change.
        delta: f64,
    },
}

/// Everything needed to create an organ.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NewOrgan {
    /// Organ type.
    pub kind: OrganKind,
    /// Parent organ (`None` only for the seed).
    pub parent: Option<OrganId>,
    /// Branching order: 0 for the main stem and tap root.
    pub axis_order: u8,
    /// Sim tick of birth, the start of the expansion curve.
    pub birth: Hours,
    /// Expansion curve.
    pub curve: ExpansionCurve,
    /// Per-organ specific length or area, set at initiation (shade avoidance changes it).
    pub specific: SpecificSize,
    /// Initial direction.
    pub orientation: Orientation,
    /// Initial structural mass.
    pub structural: GramsDM,
    /// Initial carbon reserve.
    pub reserve: GramsC,
    /// Initial nitrogen.
    pub nitrogen: GramsN,
}

/// The organ graph of one plant, as structs of arrays.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganArena {
    /// Organ type.
    pub kind: Vec<OrganKind>,
    /// Parent organ.
    pub parent: Vec<Option<OrganId>>,
    /// Children in creation order.
    pub children: Vec<Vec<OrganId>>,
    /// Branching order.
    pub axis_order: Vec<u8>,
    /// Birth tick.
    pub birth: Vec<Hours>,
    /// Death tick, if dead.
    pub death: Vec<Option<Hours>>,
    /// Accumulated thermal age.
    pub thermal_age: Vec<DegreeDays>,
    /// Expansion clock: thermal time scaled by the expansion temperature factor, the
    /// time axis of the organ's expansion curve (decision 0011).
    pub expansion_age: Vec<DegreeDays>,
    /// Expansion curve.
    pub curve: Vec<ExpansionCurve>,
    /// Specific length or area.
    pub specific: Vec<SpecificSize>,
    /// Rendered dimensions derived from structural mass.
    pub size: Vec<OrganSize>,
    structural: Vec<GramsDM>,
    /// The secondary part of the structural mass (R1a). Missing entries (a save from
    /// before model 0.18.0) read as zero.
    #[serde(default)]
    secondary: Vec<GramsDM>,
    /// Starch store (R1b, decision 0040): carbon held apart from the reserve, which
    /// neither moves in the phloem nor counts in the reserve concentration. Missing
    /// entries (a save from before model 0.21.0) read as zero.
    #[serde(default)]
    storage: Vec<GramsC>,
    /// Local carbon reserve; may go negative briefly (starvation signal).
    pub reserve: Vec<GramsC>,
    /// Nitrogen content.
    pub nitrogen: Vec<GramsN>,
    /// Cumulative unmet carbon demand.
    pub unmet: Vec<GramsC>,
    /// Cumulative potential carbon demand (denominator of the deficit, §3.9 step 5).
    pub potential_demand: Vec<GramsC>,
    /// Hours the reserve has been negative after maintenance (starvation clock).
    pub starving_hours: Vec<u32>,
    /// Light coefficients.
    pub light: Vec<LightCoeffs>,
    /// Orientation.
    pub orientation: Vec<Orientation>,
    /// Alive flag.
    pub alive: Vec<bool>,
}

impl OrganArena {
    /// An empty arena.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of organs ever created (alive or dead).
    #[must_use]
    pub fn len(&self) -> usize {
        self.kind.len()
    }

    /// True when no organ exists.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kind.is_empty()
    }

    /// All ids in creation order.
    pub fn ids(&self) -> impl Iterator<Item = OrganId> + '_ {
        (0..self.len()).map(|i| OrganId(u32::try_from(i).unwrap_or(u32::MAX)))
    }

    /// Alive ids in creation order.
    pub fn alive_ids(&self) -> impl Iterator<Item = OrganId> + '_ {
        self.ids().filter(|id| self.alive[id.index()])
    }

    /// Alive ids of one kind, in creation order.
    pub fn alive_of(&self, kind: OrganKind) -> impl Iterator<Item = OrganId> + '_ {
        self.alive_ids()
            .filter(move |id| self.kind[id.index()] == kind)
    }

    fn check(&self, id: OrganId) -> Result<usize, GraphError> {
        let i = id.index();
        if i >= self.len() {
            return Err(GraphError::NoSuchOrgan(id));
        }
        Ok(i)
    }

    fn check_alive(&self, id: OrganId) -> Result<usize, GraphError> {
        let i = self.check(id)?;
        if !self.alive[i] {
            return Err(GraphError::Dead(id));
        }
        Ok(i)
    }

    /// Creates an organ and returns its id. The parent must exist and be alive.
    ///
    /// # Errors
    /// [`GraphError::NoSuchOrgan`], [`GraphError::Dead`], [`GraphError::NegativeGrowth`]
    /// for a negative initial mass, or [`GraphError::Full`].
    pub fn add(&mut self, o: &NewOrgan, allometry: &Allometry) -> Result<OrganId, GraphError> {
        let id = OrganId(u32::try_from(self.len()).map_err(|_| GraphError::Full)?);
        if let Some(p) = o.parent {
            self.check_alive(p)?;
        }
        if !(o.structural.0.is_finite() && o.structural.0 >= 0.0) {
            return Err(GraphError::NegativeGrowth {
                id,
                delta: o.structural.0,
            });
        }
        self.kind.push(o.kind);
        self.parent.push(o.parent);
        self.children.push(Vec::new());
        self.axis_order.push(o.axis_order);
        self.birth.push(o.birth);
        self.death.push(None);
        self.thermal_age.push(DegreeDays(0.0));
        self.expansion_age.push(DegreeDays(0.0));
        self.curve.push(o.curve);
        self.specific.push(o.specific);
        self.size
            .push(allometry.size(o.kind, o.specific, o.structural));
        self.structural.push(o.structural);
        self.secondary.resize(id.index(), GramsDM(0.0));
        self.secondary.push(GramsDM(0.0));
        self.storage.resize(id.index(), GramsC(0.0));
        self.storage.push(GramsC(0.0));
        self.reserve.push(o.reserve);
        self.nitrogen.push(o.nitrogen);
        self.unmet.push(GramsC(0.0));
        self.potential_demand.push(GramsC(0.0));
        self.starving_hours.push(0);
        self.light.push(LightCoeffs::default());
        self.orientation.push(o.orientation);
        self.alive.push(true);
        if let Some(p) = o.parent {
            self.children[p.index()].push(id);
        }
        Ok(id)
    }

    /// Structural mass of an organ (frozen at death).
    ///
    /// # Panics
    /// Never for ids produced by this arena; out-of-range ids index-panic like a slice.
    #[must_use]
    pub fn structural(&self, id: OrganId) -> GramsDM {
        self.structural[id.index()]
    }

    /// All structural masses, indexed by id.
    #[must_use]
    pub fn structural_all(&self) -> &[GramsDM] {
        &self.structural
    }

    /// The secondary part of an organ's structural mass (R1a; zero for organs that
    /// never thickened).
    #[must_use]
    pub fn secondary(&self, id: OrganId) -> GramsDM {
        self.secondary
            .get(id.index())
            .copied()
            .unwrap_or(GramsDM(0.0))
    }

    /// An organ's starch store (R1b; zero for organs that store nothing).
    #[must_use]
    pub fn storage(&self, id: OrganId) -> GramsC {
        self.storage.get(id.index()).copied().unwrap_or(GramsC(0.0))
    }

    /// Moves `amount` of carbon from a living organ's reserve into its store (a
    /// negative amount releases from the store into the reserve). The reserve may go
    /// negative like any reserve; the store may not.
    ///
    /// # Errors
    /// [`GraphError::NegativeStore`] for a non-finite amount or one that would leave
    /// the store negative, or if the organ is missing or dead.
    pub fn store(&mut self, id: OrganId, amount: GramsC) -> Result<(), GraphError> {
        let i = self.check_alive(id)?;
        let s = self.storage(id).0 + amount.0;
        if !(amount.0.is_finite() && s >= 0.0) {
            return Err(GraphError::NegativeStore {
                id,
                delta: amount.0,
            });
        }
        if self.storage.len() < self.len() {
            self.storage.resize(self.len(), GramsC(0.0));
        }
        self.storage[i] = GramsC(s);
        self.reserve[i] -= amount;
        Ok(())
    }

    /// The primary part of an organ's structural mass: what its expansion built, and
    /// what sets its length and its nitrogen requirement.
    ///
    /// # Panics
    /// As [`OrganArena::structural`].
    #[must_use]
    pub fn primary(&self, id: OrganId) -> GramsDM {
        self.structural(id) - self.secondary(id)
    }

    /// An organ's radius without its secondary growth, `r · sqrt(W_p / W)`: the radius
    /// of the primary tissue, which carries the phloem (R1a, decision 0036).
    ///
    /// # Panics
    /// As [`OrganArena::structural`].
    #[must_use]
    pub fn primary_radius(&self, id: OrganId) -> Meters {
        let r = self.size[id.index()].radius;
        let s = self.secondary(id).0;
        if s <= 0.0 {
            return r;
        }
        let w = self.structural(id).0;
        Meters(r.0 * libm::sqrt(((w - s) / w).max(0.0)))
    }

    /// Adds structural mass to a living organ and updates its size.
    ///
    /// # Errors
    /// [`GraphError::NegativeGrowth`] for a negative or non-finite increment, or if the
    /// organ is missing or dead.
    pub fn grow(
        &mut self,
        id: OrganId,
        delta: GramsDM,
        allometry: &Allometry,
    ) -> Result<(), GraphError> {
        let i = self.check_alive(id)?;
        if !(delta.0.is_finite() && delta.0 >= 0.0) {
            return Err(GraphError::NegativeGrowth { id, delta: delta.0 });
        }
        self.structural[i] += delta;
        let secondary = self.secondary(id);
        self.size[i] = if secondary.0 > 0.0 {
            allometry.size_split(
                self.kind[i],
                self.specific[i],
                self.structural[i],
                secondary,
            )
        } else {
            allometry.size(self.kind[i], self.specific[i], self.structural[i])
        };
        Ok(())
    }

    /// Adds secondary structural mass to a living organ (R1a thickening): the length
    /// stays, the radius widens.
    ///
    /// # Errors
    /// As [`OrganArena::grow`].
    pub fn grow_secondary(
        &mut self,
        id: OrganId,
        delta: GramsDM,
        allometry: &Allometry,
    ) -> Result<(), GraphError> {
        let i = self.check_alive(id)?;
        if !(delta.0.is_finite() && delta.0 >= 0.0) {
            return Err(GraphError::NegativeGrowth { id, delta: delta.0 });
        }
        if self.secondary.len() < self.len() {
            self.secondary.resize(self.len(), GramsDM(0.0));
        }
        self.structural[i] += delta;
        self.secondary[i] += delta;
        self.size[i] = allometry.size_split(
            self.kind[i],
            self.specific[i],
            self.structural[i],
            self.secondary[i],
        );
        Ok(())
    }

    /// Kills an organ at tick `at` and returns its structural mass at death, from which
    /// the caller books resorption and litter (§3.9 step 6). Children are not killed.
    /// The organ's starch store joins its reserve, so the caller books it as reserve
    /// (R1b, decision 0040).
    ///
    /// # Errors
    /// If the organ is missing or already dead.
    pub fn kill(&mut self, id: OrganId, at: Hours) -> Result<GramsDM, GraphError> {
        let i = self.check_alive(id)?;
        if let Some(s) = self.storage.get_mut(i) {
            self.reserve[i] += *s;
            *s = GramsC(0.0);
        }
        self.alive[i] = false;
        self.death[i] = Some(at);
        Ok(self.structural[i])
    }

    /// Kills an organ and all its living descendants (an axis dies with its basal
    /// internode). Returns `(id, mass)` for each organ killed, in depth-first creation order.
    ///
    /// # Errors
    /// If the root organ is missing or already dead.
    pub fn kill_subtree(
        &mut self,
        id: OrganId,
        at: Hours,
    ) -> Result<Vec<(OrganId, GramsDM)>, GraphError> {
        self.check_alive(id)?;
        let order = self.subtree(id);
        let mut out = Vec::new();
        for o in order {
            if self.alive[o.index()] {
                out.push((o, self.kill(o, at)?));
            }
        }
        Ok(out)
    }

    /// The organ and its descendants, depth-first, children in creation order.
    #[must_use]
    pub fn subtree(&self, id: OrganId) -> Vec<OrganId> {
        let mut out = Vec::new();
        let mut stack = vec![id];
        while let Some(o) = stack.pop() {
            out.push(o);
            for c in self.children[o.index()].iter().rev() {
                stack.push(*c);
            }
        }
        out
    }

    /// Living parent-child edges, ordered by child id. These are the transport edges of §3.9.
    pub fn edges(&self) -> impl Iterator<Item = (OrganId, OrganId)> + '_ {
        self.alive_ids().filter_map(|c| {
            let p = self.parent[c.index()]?;
            self.alive[p.index()].then_some((p, c))
        })
    }

    /// Potential final size reduced by unmet demand: `(1 − unmet / potential demand) × potential` (§3.9 step 5).
    #[must_use]
    pub fn effective_potential(&self, id: OrganId) -> GramsDM {
        let i = id.index();
        let p = self.curve[i].potential;
        let d = self.potential_demand[i].0;
        if d <= 0.0 {
            return p;
        }
        p * (1.0 - self.unmet[i].0 / d).clamp(0.0, 1.0)
    }

    /// Books one step of demand for an organ: the potential demand and the part not met.
    pub fn record_demand(&mut self, id: OrganId, potential: GramsC, unmet: GramsC) {
        let i = id.index();
        self.potential_demand[i] += potential.max(GramsC(0.0));
        self.unmet[i] += unmet.max(GramsC(0.0));
    }

    /// Sum of structural mass over living organs of a kind, in id order.
    #[must_use]
    pub fn total_structural(&self, kind: OrganKind) -> GramsDM {
        self.alive_of(kind).map(|id| self.structural(id)).sum()
    }

    /// Visits every number of the arena's state as bits, in a fixed order, for the
    /// full-state hash (§11.4: hash bit patterns, never formatted floats).
    pub fn visit_bits(&self, mut f: impl FnMut(u64)) {
        f(self.len() as u64);
        for i in 0..self.len() {
            f(self.kind[i] as u64);
            f(self.parent[i].map_or(u64::MAX, |p| u64::from(p.0)));
            f(u64::from(self.axis_order[i]));
            f(self.birth[i].0);
            f(self.death[i].map_or(u64::MAX, |h| h.0));
            f(self.thermal_age[i].to_bits());
            f(self.expansion_age[i].to_bits());
            f(self.curve[i].duration.to_bits());
            f(self.curve[i].t_max.to_bits());
            f(self.curve[i].potential.to_bits());
            self.specific[i].visit_bits(&mut f);
            f(self.size[i].length.to_bits());
            f(self.size[i].radius.to_bits());
            f(self.size[i].area.to_bits());
            f(self.structural[i].to_bits());
            let id = OrganId(u32::try_from(i).unwrap_or(u32::MAX));
            f(self.secondary(id).to_bits());
            f(self.storage(id).to_bits());
            f(self.reserve[i].to_bits());
            f(self.nitrogen[i].to_bits());
            f(self.unmet[i].to_bits());
            f(self.potential_demand[i].to_bits());
            f(u64::from(self.starving_hours[i]));
            for d in self.light[i].direct {
                f(d.to_bits());
            }
            f(self.light[i].diffuse.to_bits());
            f(self.light[i].diffuse_absorbed.to_bits());
            f(self.light[i].sunlit_fraction.to_bits());
            f(self.light[i].rfr.to_bits());
            for d in self.orientation[i].direction {
                f(d.to_bits());
            }
            f(self.orientation[i].azimuth.to_bits());
            f(u64::from(self.alive[i]));
        }
    }
}

#[cfg(test)]
mod tests;
