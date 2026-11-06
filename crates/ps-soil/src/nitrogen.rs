//! Soil nitrogen (design plan §3.6 "Nitrogen (M2)", WP24, decision 0020).
//!
//! Each active cell holds three pools, all in g N: humus (organic) N, ammonium and
//! nitrate. Every hour, after the water steps:
//!
//! 1. mineralization: humus N → NH₄ at `k_min · f_T · f_W / 24`;
//! 2. nitrification: NH₄ → NO₃ at `k_nit · f_T · f_W / 24`.
//!
//! Nitrate moves with water (drainage and lateral flow, inside
//! [`SoilGrid::step_hour`]); ammonium and humus do not. Roots take up both mineral forms
//! once a day ([`SoilGrid::take_up_nitrogen`]), each root cell drawing on its own
//! nitrogen (R2a, decision 0037). Fertilizer adds to the top layer.
//! Every flux is booked in the grid's [`NitrogenLedger`]; debug builds assert closure
//! to 1e-9 relative.
//!
//! Model card: docs/models/nitrogen.md.

use ps_units::GramsN;
use serde::{Deserialize, Serialize};

use crate::{CLOSURE_TOLERANCE, RootCells, SoilError, SoilGrid};

/// Humus N mineralization rate at 30 °C and optimal water, per day ("~", APSIM `SoilN`
/// humic pool 1.5e-4 d⁻¹; about 2% a year in a temperate summer).
pub const MINERALIZATION_RATE: f64 = 1.5e-4;
/// Nitrification rate at 30 °C and optimal water, per day ("~"; 0.15 d⁻¹ at 20 °C,
/// inside 0.1 to 0.2).
pub const NITRIFICATION_RATE: f64 = 0.34;
/// Residual nitrate roots cannot take, mg N per kg soil (CROPGRO NUPTAK).
pub const NO3_FLOOR_PPM: f64 = 0.25;
/// Residual ammonium roots cannot take, mg N per kg soil (CROPGRO NUPTAK).
pub const NH4_FLOOR_PPM: f64 = 0.5;
/// Steepness of the uptake response to soil mineral N, per mg N per kg soil (CROPGRO
/// NUPTAK `1 − exp(−0.08 ppm)`).
pub const UPTAKE_PPM_RESPONSE: f64 = 0.08;

/// A soil's starting nitrogen, uniform over its cells.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoilNitrogen {
    /// Nitrate N, mg N per kg dry soil.
    pub no3_ppm: f64,
    /// Ammonium N, mg N per kg dry soil.
    pub nh4_ppm: f64,
    /// Humus (organic) N, g N per kg dry soil.
    pub organic_g_per_kg: f64,
    /// Whether the soil carries rhizobia that can nodulate a legume.
    pub rhizobia: bool,
}

impl SoilNitrogen {
    /// A moderately fertile garden loam ("~"): 20 mg nitrate N and 2 mg ammonium N per
    /// kg, 1 g humus N per kg (about 2% organic matter at C:N 11), with rhizobia.
    pub const GARDEN: Self = Self {
        no3_ppm: 20.0,
        nh4_ppm: 2.0,
        organic_g_per_kg: 1.0,
        rhizobia: true,
    };

    /// Washed sand with no nitrogen at all, and no rhizobia unless the scenario adds
    /// them (the WP24 done-when pots).
    pub const NONE: Self = Self {
        no3_ppm: 0.0,
        nh4_ppm: 0.0,
        organic_g_per_kg: 0.0,
        rhizobia: false,
    };
}

impl Default for SoilNitrogen {
    fn default() -> Self {
        Self::GARDEN
    }
}

/// Mineral fertilizer forms (§2 "Fertilize").
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NitrogenForm {
    /// Nitrate (for example calcium nitrate).
    Nitrate,
    /// Ammonium (ammonium sulfate; urea hydrolyses to it within days, ignored).
    Ammonium,
}

/// Nitrogen fluxes since the grid was filled, g N, all positive.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NitrogenLedger {
    /// Stock when the ledger started.
    pub start: GramsN,
    /// Fertilizer added.
    pub added: GramsN,
    /// Nitrate drained out of the bottom.
    pub leached: GramsN,
    /// Taken up by roots.
    pub uptake: GramsN,
    /// Humus N mineralized to ammonium (internal; not part of the balance).
    pub mineralized: GramsN,
    /// Ammonium nitrified (internal; not part of the balance).
    pub nitrified: GramsN,
}

impl NitrogenLedger {
    /// `(stock − expected) / largest term`.
    #[must_use]
    pub fn relative_residual(&self, stock: GramsN) -> f64 {
        let expected = self.start + self.added - self.leached - self.uptake;
        let scale = [
            stock.0,
            self.start.0,
            self.added.0,
            self.leached.0,
            self.uptake.0,
        ]
        .into_iter()
        .fold(1e-12, f64::max);
        (stock - expected).0.abs() / scale
    }
}

/// One plant's request in the day's joint nitrogen uptake.
#[derive(Copy, Clone, Debug)]
pub struct NitrogenRequest<'a> {
    /// Nitrogen the plant can use today.
    pub demand: GramsN,
    /// Its roots.
    pub roots: &'a RootCells,
    /// Uptake capacity per meter of root per day at high soil N, g N m⁻¹ d⁻¹, each
    /// form; the roots draw from the cells they occupy (R2a, decision 0037).
    pub capacity_per_length: f64,
}

/// What one plant got from the joint nitrogen uptake.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct NitrogenUptake {
    /// Nitrate taken.
    pub no3: GramsN,
    /// Ammonium taken.
    pub nh4: GramsN,
}

impl NitrogenUptake {
    /// Both forms together.
    #[must_use]
    pub fn total(&self) -> GramsN {
        self.no3 + self.nh4
    }
}

/// Temperature factor of mineralization and nitrification: `(T/30)²` for T above 0 °C,
/// at most 1.2 ("~", APSIM-like).
#[must_use]
pub fn temperature_factor(t: f64) -> f64 {
    let r = t.max(0.0) / 30.0;
    (r * r).min(1.2)
}

/// Water factor of mineralization and nitrification (APSIM `SoilN` shape): 0 at the
/// wilting point, 1 from halfway to field capacity up to field capacity, falling to 0.5
/// at saturation.
#[must_use]
pub fn water_factor(theta: f64, wp: f64, fc: f64, sat: f64) -> f64 {
    if theta <= fc {
        let rel = (theta - wp) / (fc - wp);
        (2.0 * rel).clamp(0.0, 1.0)
    } else {
        let over = ((theta - fc) / (sat - fc)).clamp(0.0, 1.0);
        1.0 - 0.5 * over
    }
}

impl SoilGrid {
    /// Mass of dry soil in one cell, kg.
    #[must_use]
    pub fn cell_soil_mass(&self) -> f64 {
        self.texture.hydraulics().bulk_density * self.cell_volume()
    }

    /// Sets every active cell's nitrogen from `n` and restarts the ledger.
    pub fn set_nitrogen(&mut self, n: &SoilNitrogen) {
        let kg = self.cell_soil_mass();
        for i in 0..self.water.len() {
            let on = if self.active[i] { 1.0 } else { 0.0 };
            // mg per kg × kg / 1000 = g.
            self.no3[i] = GramsN(on * n.no3_ppm.max(0.0) * kg * 1e-3);
            self.nh4[i] = GramsN(on * n.nh4_ppm.max(0.0) * kg * 1e-3);
            self.organic_n[i] = GramsN(on * n.organic_g_per_kg.max(0.0) * kg);
        }
        self.rhizobia = n.rhizobia;
        self.n_ledger = NitrogenLedger {
            start: self.total_nitrogen(),
            ..NitrogenLedger::default()
        };
    }

    /// Multiplies each soil column's nitrogen (nitrate, ammonium and humus, every
    /// depth) by its factor in `columns`, indexed `y * nx + x`, and restarts the ledger:
    /// small-scale patchiness of a field soil (decision 0027). Missing or non-finite
    /// factors count as 1, and negative ones as 0.
    pub fn scale_nitrogen_columns(&mut self, columns: &[f64]) {
        let plane = self.nx * self.ny;
        for i in 0..self.water.len() {
            let f = columns
                .get(i % plane)
                .copied()
                .filter(|f| f.is_finite())
                .unwrap_or(1.0)
                .max(0.0);
            self.no3[i] = GramsN(self.no3[i].0 * f);
            self.nh4[i] = GramsN(self.nh4[i].0 * f);
            self.organic_n[i] = GramsN(self.organic_n[i].0 * f);
        }
        self.n_ledger = NitrogenLedger {
            start: self.total_nitrogen(),
            ..NitrogenLedger::default()
        };
    }

    /// Whether the soil can nodulate a legume.
    #[must_use]
    pub fn has_rhizobia(&self) -> bool {
        self.rhizobia
    }

    /// The nitrogen ledger since [`SoilGrid::set_nitrogen`].
    #[must_use]
    pub fn nitrogen_ledger(&self) -> &NitrogenLedger {
        &self.n_ledger
    }

    /// Nitrate in cell `i`.
    #[must_use]
    pub fn no3(&self, i: usize) -> GramsN {
        self.no3[i]
    }

    /// Ammonium in cell `i`.
    #[must_use]
    pub fn nh4(&self, i: usize) -> GramsN {
        self.nh4[i]
    }

    /// Humus N in cell `i`.
    #[must_use]
    pub fn organic_n(&self, i: usize) -> GramsN {
        self.organic_n[i]
    }

    /// Nitrate concentration of cell `i`, mg N per kg soil.
    #[must_use]
    pub fn no3_ppm(&self, i: usize) -> f64 {
        self.no3[i].0 * 1e3 / self.cell_soil_mass()
    }

    /// All nitrogen in the grid.
    #[must_use]
    pub fn total_nitrogen(&self) -> GramsN {
        (0..self.water.len())
            .map(|i| self.no3[i] + self.nh4[i] + self.organic_n[i])
            .sum()
    }

    /// Mineral (nitrate plus ammonium) nitrogen in the grid.
    #[must_use]
    pub fn mineral_nitrogen(&self) -> GramsN {
        (0..self.water.len())
            .map(|i| self.no3[i] + self.nh4[i])
            .sum()
    }

    /// Root-length-weighted nitrate concentration around `roots`, mg N per kg soil
    /// (0 with no roots).
    ///
    /// # Errors
    /// [`SoilError::RootCell`] if a root cell is outside the grid.
    pub fn root_zone_no3_ppm(&self, roots: &RootCells) -> Result<f64, SoilError> {
        self.check_roots(roots)?;
        let (mut num, mut den) = (0.0, 0.0);
        for &(i, l) in roots.cells() {
            if self.active[i] {
                num += l.0 * self.no3_ppm(i);
                den += l.0;
            }
        }
        Ok(if den > 0.0 { num / den } else { 0.0 })
    }

    /// Spreads `g_per_m2` of fertilizer N evenly over the top layer.
    pub fn fertilize(&mut self, form: NitrogenForm, g_per_m2: f64) {
        let per_cell = GramsN(g_per_m2.max(0.0) * self.cell.0 * self.cell.0);
        for i in 0..self.nx * self.ny {
            if !self.active[i] {
                continue;
            }
            match form {
                NitrogenForm::Nitrate => self.no3[i] += per_cell,
                NitrogenForm::Ammonium => self.nh4[i] += per_cell,
            }
            self.n_ledger.added += per_cell;
        }
        self.assert_nitrogen_closes();
    }

    /// One hour of mineralization and nitrification at each layer's temperature.
    pub(crate) fn transform_nitrogen(&mut self) {
        let hy = self.texture.hydraulics();
        let v = self.cell_volume() * crate::WATER_DENSITY;
        let layer = self.nx * self.ny;
        for z in 0..self.nz {
            let ft = temperature_factor(self.layer_temperature[z].0);
            if ft <= 0.0 {
                continue;
            }
            for i in z * layer..(z + 1) * layer {
                if !self.active[i] {
                    continue;
                }
                let theta = self.water[i].0 / v;
                let f =
                    ft * water_factor(theta, hy.wilting_point, hy.field_capacity, hy.saturation);
                let m = self.organic_n[i] * (MINERALIZATION_RATE * f / 24.0);
                self.organic_n[i] -= m;
                self.nh4[i] += m;
                let n = self.nh4[i] * (NITRIFICATION_RATE * f / 24.0).min(1.0);
                self.nh4[i] -= n;
                self.no3[i] += n;
                self.n_ledger.mineralized += m;
                self.n_ledger.nitrified += n;
            }
        }
    }

    pub(crate) fn assert_nitrogen_closes(&self) {
        debug_assert!(
            self.n_ledger.relative_residual(self.total_nitrogen()) < CLOSURE_TOLERANCE,
            "soil nitrogen ledger does not close: {:?}, stock {:?}",
            self.n_ledger,
            self.total_nitrogen()
        );
    }

    /// The day's root nitrogen uptake for every plant in the grid (§3.6, decision
    /// 0020), CROPGRO NUPTAK style. All plants see the same soil, so their order does
    /// not matter.
    ///
    /// 1. A plant's potential in each root cell, per form, is
    ///    `L · capacity · (1 − e^(−0.08 ppm)) · √f`, with `f` the cell's FTSW, and is
    ///    never more than the form's nitrogen above its residual floor.
    /// 2. It asks each cell for `min(demand, Σ potential) / Σ potential` of it.
    /// 3. Where the plants together ask a cell for more than it has above the floor,
    ///    every request there is scaled by the same factor.
    ///
    /// # Errors
    /// [`SoilError::RootCell`] if a root cell is outside the grid.
    pub fn take_up_nitrogen(
        &mut self,
        plants: &[NitrogenRequest<'_>],
    ) -> Result<Vec<NitrogenUptake>, SoilError> {
        for p in plants {
            self.check_roots(p.roots)?;
        }
        let kg = self.cell_soil_mass();
        let floor_no3 = NO3_FLOOR_PPM * kg * 1e-3;
        let floor_nh4 = NH4_FLOOR_PPM * kg * 1e-3;
        let avail = |pool: &[GramsN], i: usize, floor: f64| (pool[i].0 - floor).max(0.0);
        let response = |g: f64| {
            let ppm = g * 1e3 / kg;
            1.0 - libm::exp(-UPTAKE_PPM_RESPONSE * ppm)
        };
        // Per plant, from the same state: its potential per root cell (no3, nh4), and
        // its request per root cell, (no3, nh4). Plants are independent and collect
        // keeps plant order.
        let plan = |p: &NitrogenRequest<'_>| {
            let pot: Vec<(f64, f64)> = p
                .roots
                .cells()
                .iter()
                .map(|&(i, l)| {
                    if !self.active[i] {
                        return (0.0, 0.0);
                    }
                    let w = libm::sqrt(self.cell_ftsw(i));
                    let cap = l.0 * p.capacity_per_length.max(0.0) * w;
                    let a3 = avail(&self.no3, i, floor_no3);
                    let a4 = avail(&self.nh4, i, floor_nh4);
                    (
                        (cap * response(self.no3[i].0)).min(a3),
                        (cap * response(self.nh4[i].0)).min(a4),
                    )
                })
                .collect();
            let total: f64 = pot.iter().map(|x| x.0 + x.1).sum();
            let want = p.demand.0.max(0.0).min(total);
            let share = if total > 0.0 { want / total } else { 0.0 };
            let req: Vec<(f64, f64)> = pot.iter().map(|x| (x.0 * share, x.1 * share)).collect();
            req
        };
        let asked: Vec<Vec<(f64, f64)>> = if plants.len() > 1 {
            use rayon::prelude::*;
            plants.par_iter().map(plan).collect()
        } else {
            plants.iter().map(plan).collect()
        };
        let mut cell_total: Vec<(usize, f64, f64)> = Vec::new();
        for (p, req) in plants.iter().zip(&asked) {
            for (&(i, _), r) in p.roots.cells().iter().zip(req) {
                if r.0 > 0.0 || r.1 > 0.0 {
                    cell_total.push((i, r.0, r.1));
                }
            }
        }
        cell_total.sort_by_key(|c| c.0);
        let mut scale: Vec<(usize, f64, f64)> = Vec::new();
        for (i, a, b) in cell_total {
            match scale.last_mut() {
                Some(last) if last.0 == i => {
                    last.1 += a;
                    last.2 += b;
                }
                _ => scale.push((i, a, b)),
            }
        }
        for s in &mut scale {
            let a3 = avail(&self.no3, s.0, floor_no3);
            let a4 = avail(&self.nh4, s.0, floor_nh4);
            s.1 = if s.1 > a3 { a3 / s.1 } else { 1.0 };
            s.2 = if s.2 > a4 { a4 / s.2 } else { 1.0 };
        }
        let factor = |i: usize| {
            scale
                .binary_search_by_key(&i, |s| s.0)
                .map_or((1.0, 1.0), |k| (scale[k].1, scale[k].2))
        };
        let mut out = Vec::with_capacity(plants.len());
        for (p, req) in plants.iter().zip(&asked) {
            let mut got = NitrogenUptake::default();
            for (&(i, _), r) in p.roots.cells().iter().zip(req) {
                let (f3, f4) = factor(i);
                let (u3, u4) = (GramsN(r.0 * f3), GramsN(r.1 * f4));
                self.no3[i] -= u3;
                self.nh4[i] -= u4;
                got.no3 += u3;
                got.nh4 += u4;
            }
            self.n_ledger.uptake += got.total();
            out.push(got);
        }
        self.assert_nitrogen_closes();
        Ok(out)
    }
}

#[cfg(test)]
mod tests;
