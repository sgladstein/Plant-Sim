//! Secondary growth: stem and root thickening (R1a, decision 0036).
//!
//! - **Stems** follow the pipe model (Shinozaki et al. 1964): daily, each living
//!   internode's target cross-section is `X* = h_L · A_max + h_F · ΣF`, with `A_max` the
//!   most leaf area it has carried above it (pipes stay after their leaves die) and
//!   `ΣF` the fruit dry mass above it now. Hourly, an internode whose expansion is
//!   complete may widen toward the target at most `g_max` in radius per day:
//!   `ΔW = min(X* − π r², g_max · 2π r · Δt) · L · ρ_stem · f_T · s`.
//! - **Roots** follow the `OpenSimRoot` bean rates: a diameter growth rate by the segment's
//!   own age, scaled by a factor by the segment's distance from its root's base, so
//!   old basal segments grow widest and roots taper:
//!   `ΔW = 2π r L · ½ (dd/dt)(age) · factor(distance) · Δt · ρ_root · f_T · s`.
//!
//! Both are potentials: `ps_physio::allocation` builds what the organ's reserve allows
//! at the secondary-growth priority. `f_T` is the expansion temperature multiplier at
//! the organ's temperature and `s` the expansion stress factor (water times nitrogen).
//! Secondary mass widens an organ at fixed length (`ps_graph::Allometry::size_split`).
//! Model card: docs/models/organ-growth.md.

use ps_graph::OrganKind;
use ps_physio::Stage;
use ps_units::{Celsius, GramsDM, Hours};
use serde::{Deserialize, Serialize};

use crate::Plant;

const CM: f64 = 0.01;

/// Secondary growth parameters in model units.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThickeningParams {
    /// The species has a cambium (false for grasses: no thickening at all).
    pub enabled: bool,
    /// Pipe-model stem cross-section per leaf area carried above, m² m⁻².
    pub pipe_per_area: f64,
    /// Extra stem cross-section per fruit dry mass above, m² g⁻¹.
    pub pipe_per_fruit: f64,
    /// Most a stem's radius grows per hour, m.
    pub stem_rate: f64,
    /// Root radius growth per hour at optimum temperature by segment age in days,
    /// `(age, m h⁻¹)`: half the species' diameter rate.
    pub root_rate: Vec<(f64, f64)>,
}

impl ThickeningParams {
    /// Converts the species' `thickening` group and root rate table.
    #[must_use]
    pub fn new(s: &ps_species::Species) -> Self {
        let t = &s.thickening;
        Self {
            enabled: t.secondary_growth.value >= 0.5,
            pipe_per_area: t.pipe_area_per_leaf_area.value,
            pipe_per_fruit: t.pipe_area_per_fruit_mass.value,
            // mm per day → m per hour.
            stem_rate: t.stem_radial_rate_max.value * 1e-3 / 24.0,
            // A diameter rate in cm per day → a radius rate in m per hour.
            root_rate: s
                .root_secondary_rate
                .points
                .iter()
                .map(|&(age, rate)| (age, 0.5 * rate * CM / 24.0))
                .collect(),
        }
    }
}

impl Plant {
    /// Daily (after organ initiation): each living internode's pipe-model target
    /// cross-section from the leaf area it has carried and the fruit it carries now.
    pub(crate) fn pipe_targets(&mut self) {
        let tp = &self.params.thickening;
        if !tp.enabled {
            return;
        }
        let a = &self.arena;
        let n = a.len();
        // One tip-to-base pass: children are created after their parents, so
        // descending ids fold every subtree into its root.
        let mut area = vec![0.0; n];
        let mut fruit = vec![0.0; n];
        for i in (0..n).rev() {
            if !a.alive[i] {
                continue;
            }
            match a.kind[i] {
                OrganKind::Leaf => area[i] += a.size[i].area.0,
                OrganKind::Pod | OrganKind::SeedInPod | OrganKind::Flower => {
                    fruit[i] += a.structural_all()[i].0;
                }
                _ => {}
            }
            if let Some(p) = a.parent[i] {
                area[p.index()] += area[i];
                fruit[p.index()] += fruit[i];
            }
        }
        let (h_l, h_f) = (tp.pipe_per_area, tp.pipe_per_fruit);
        for i in 0..n {
            if !a.alive[i] || a.kind[i] != OrganKind::Internode {
                continue;
            }
            let m = &mut self.meta[i];
            m.supported_area = m.supported_area.max(area[i]);
            m.pipe_target = h_l * m.supported_area + h_f * fruit[i];
        }
    }

    /// This hour's potential secondary growth by organ id (g DM of structure), from
    /// the organs' temperatures `temps` and the expansion stress factor `stress`;
    /// empty when the plant does not thicken (a grass, or a mature plant).
    pub(crate) fn secondary_potential(
        &self,
        temps: &[Celsius],
        now: Hours,
        stress: f64,
    ) -> Vec<GramsDM> {
        let tp = &self.params.thickening;
        if !tp.enabled
            || self.phenology.stage < Stage::Germinated
            || self.phenology.stage >= Stage::Mature
        {
            return Vec::new();
        }
        let a = &self.arena;
        let al = &self.params.allometry;
        let growth = &self.params.temperature.growth;
        let s = stress.clamp(0.0, 1.0);
        let mut pot = vec![GramsDM(0.0); a.len()];
        let two_pi = 2.0 * core::f64::consts::PI;
        // Stems: toward the pipe target, once expanded.
        for id in a.alive_of(OrganKind::Internode) {
            let i = id.index();
            let target = self.meta[i].pipe_target;
            if target <= 0.0 || !a.curve[i].is_complete(a.expansion_age[i]) {
                continue;
            }
            let (r, l) = (a.size[i].radius.0, a.size[i].length.0);
            let gap = target - core::f64::consts::PI * r * r;
            if gap <= 0.0 || l <= 0.0 {
                continue;
            }
            let da = gap.min(tp.stem_rate * two_pi * r);
            pot[i] = GramsDM(da * l * al.stem_density.0 * growth.factor(temps[i]) * s);
        }
        // Roots: by segment age and distance from the root's base.
        for ax in &self.root_axes {
            let factor = &self.params.roots[ax.kind].secondary_factor;
            for &(id, start) in &ax.segments {
                let i = id.index();
                if !a.alive[i] {
                    continue;
                }
                #[allow(clippy::cast_precision_loss)]
                let age = now.0.saturating_sub(a.birth[i].0) as f64 / 24.0;
                let rate = ps_species::interpolate(&tp.root_rate, age);
                if rate <= 0.0 {
                    continue;
                }
                let (r, l) = (a.size[i].radius.0, a.size[i].length.0);
                let along = start + 0.5 * l / self.meta[i].multiplicity.max(1.0);
                let dr = rate * ps_species::interpolate(factor, along);
                pot[i] =
                    GramsDM(two_pi * r * l * dr * al.root_density.0 * growth.factor(temps[i]) * s);
            }
        }
        pot
    }

    /// Diameter of the main root at `depth` below its base (the shovelomics tap root
    /// diameter), m; 0 when the tap root is shorter.
    #[must_use]
    pub fn tap_root_diameter(&self, depth: f64) -> f64 {
        let Some(tap) = self
            .root_axes
            .iter()
            .find(|x| self.params.roots[x.kind].origin == ps_species::RootOrigin::Radicle)
        else {
            return 0.0;
        };
        tap.segments
            .iter()
            .rev()
            .find(|&&(id, start)| {
                start <= depth && start + self.arena.size[id.index()].length.0 > depth
            })
            .map_or(0.0, |&(id, _)| 2.0 * self.arena.size[id.index()].radius.0)
    }
}
