//! Leaf energy balance (WP27, decision 0034): leaf temperature from absorbed radiation,
//! longwave exchange with the sky, convection and transpiration.
//!
//! Per unit one-sided leaf area; both faces exchange heat and longwave, and the faces'
//! stomata each sit in series with that face's boundary layer (Helios
//! `moistureConductance`; research `docs/research/wp27-leaf-energy-balance`).
//!
//! - [`leaf_temperature`]: one leaf part at a given stomatal conductance, in closed form
//!   (linearised Penman–Monteith) plus Newton steps on the full balance.
//! - [`solve_coupled`]: gas exchange and the energy balance together (stomata respond to
//!   the leaf's own temperature and leaf-to-air vapor pressure deficit), by a bracketed
//!   Illinois iteration on `g(T) = T_eb(gs(T)) − T`.
//!
//! Model card: docs/models/leaf-energy-balance.md.

use ps_units::{Celsius, KPa, Meters, MolPerM2S, Ppm, UmolPerM2S};
use serde::{Deserialize, Serialize};

use crate::gas::{self, GasExchange, LeafEnv, LeafParams, ZERO_C_K};

/// Stefan–Boltzmann constant, W m⁻² K⁻⁴.
pub const SIGMA: f64 = 5.670_374_419e-8;
/// Molar heat capacity of air, J mol⁻¹ K⁻¹ (Campbell and Norman 1998).
pub const CP_AIR: f64 = 29.3;
/// Gas constant, J mol⁻¹ K⁻¹.
const R_GAS: f64 = 8.314;
/// PAR photons per joule of PAR, µmol J⁻¹ (the light model's conversion, ps-light).
pub const UMOL_PER_J_PAR: f64 = 4.57;
/// Characteristic dimension of a leaf as a share of its width (Campbell and Norman 1998).
pub const DIMENSION_PER_WIDTH: f64 = 0.72;
/// Forced convection, heat, per face, m s⁻¹ per √(m s⁻¹ / m): Pohlhausen's laminar
/// flat plate at 20 °C (`0.135 mol m⁻² s⁻¹` at sea level).
pub const FORCED: f64 = 0.003_28;
/// Field enhancement of the laminar boundary-layer conductance by turbulence (Campbell
/// and Norman 1998; Chamberlain 1974 measured 1.25 for bean leaves in a canopy).
pub const ENHANCEMENT: f64 = 1.4;
/// Thermal diffusivity of air, m² s⁻¹ (Leuning et al. 1995, as in plantecophys).
pub const D_HEAT: f64 = 21.5e-6;
/// Vapor to heat boundary-layer conductance ratio (Leuning et al. 1995).
pub const VAPOR_PER_HEAT: f64 = 1.075;
/// Lowest wind a leaf sees, m s⁻¹ (DSSAT, CLM); free convection covers stiller air.
pub const WIND_FLOOR: f64 = 0.1;
/// Bracket half-width for the coupled iteration around air temperature, K.
const T_SPAN_LOW: f64 = 20.0;
const T_SPAN_HIGH: f64 = 25.0;
/// Most gas-exchange evaluations per coupled solve.
pub const MAX_EVALUATIONS: u32 = 8;
/// Convergence of the coupled solve, K (0.01 until model 0.18; 0.05 halves the
/// evaluations per class-table node for a node error of about 0.03 K).
pub const T_TOLERANCE: f64 = 0.05;

/// A species' leaf energy parameters.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeafEnergyParams {
    /// Leaflet or blade width across the wind.
    pub width: Meters,
    /// Absorbed near-infrared energy per absorbed PAR energy in the open.
    pub nir_ratio: f64,
    /// Longwave emissivity (and absorptivity) of the leaf.
    pub emissivity: f64,
    /// Share of stomatal conductance on the lower (abaxial) face.
    pub abaxial_share: f64,
}

impl LeafEnergyParams {
    /// Bits of every field, for class-table sharing.
    #[must_use]
    pub fn bit_key(&self) -> [u64; 4] {
        [
            self.width.0.to_bits(),
            self.nir_ratio.to_bits(),
            self.emissivity.to_bits(),
            self.abaxial_share.to_bits(),
        ]
    }

    /// Characteristic dimension, m.
    fn dimension(&self) -> f64 {
        DIMENSION_PER_WIDTH * self.width.0
    }
}

/// The hour's air around a plant.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Air {
    /// Air temperature.
    pub t_air: Celsius,
    /// Air vapor pressure deficit.
    pub vpd: KPa,
    /// Air pressure.
    pub patm: KPa,
    /// Cloud fraction, 0 to 1 (sky longwave).
    pub cloud: f64,
}

impl Air {
    /// Air vapor pressure, kPa.
    #[must_use]
    pub fn ea(&self) -> f64 {
        (saturation_kpa(self.t_air.0) - self.vpd.0.max(0.0)).max(0.0)
    }

    /// Longwave emissivity of the sky: Brutsaert 1975 for a clear sky, with the cloud
    /// correction of Unsworth and Monteith 1975 (Campbell and Norman 1998, eq. 10.12).
    #[must_use]
    pub fn sky_emissivity(&self) -> f64 {
        let tk = self.t_air.0 + ZERO_C_K;
        let clear = 1.72 * libm::pow(self.ea() / tk, 1.0 / 7.0);
        let c = self.cloud.clamp(0.0, 1.0);
        ((1.0 - 0.84 * c) * clear + 0.84 * c).min(1.0)
    }
}

/// What one leaf part sees.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Site {
    /// Absorbed PAR.
    pub par: UmolPerM2S,
    /// Share of the sky hemisphere the leaf's upper face sees (1 in the open).
    pub sky_view: f64,
    /// Wind at the leaf, m s⁻¹ (floored at [`WIND_FLOOR`]).
    pub wind: f64,
}

/// One leaf part's heat balance.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LeafHeat {
    /// Leaf temperature.
    pub t_leaf: Celsius,
    /// Transpiration (negative is dew).
    pub e: MolPerM2S,
    /// Total leaf conductance to water vapor (stomata and boundary layer).
    pub g_v: MolPerM2S,
}

/// Height of the weather's wind, m (FAO-56's standard 2 m).
pub const WIND_HEIGHT: f64 = 2.0;
/// Roughness length of the ground around a garden plot, m: the plot is too small to
/// make its own boundary layer, so the surroundings set the profile ("~"; 0.01 to
/// 0.03 for short ground, research `notes/boundary_layer_wind.md`).
pub const SURROUNDINGS_ROUGHNESS: f64 = 0.02;

/// Wind at height `z` (m above the soil) in the open, from the wind `u2` at
/// [`WIND_HEIGHT`]: a neutral log profile over [`SURROUNDINGS_ROUGHNESS`], with `z`
/// held at least twice the roughness length.
#[must_use]
pub fn wind_at(u2: f64, z: f64) -> f64 {
    let z0 = SURROUNDINGS_ROUGHNESS;
    let z = z.max(2.0 * z0);
    u2.max(0.0) * libm::log(z / z0) / libm::log(WIND_HEIGHT / z0)
}

/// Wind at a leaf inside a plant: the open-air wind at its height attenuated by the
/// leaf area above it, read from its diffuse-sky transmittance `tau_d` as
/// `√tau_d` (wind and diffuse light extinguished alike, as GECROS; decision 0034).
#[must_use]
pub fn leaf_wind(u2: f64, z: f64, tau_d: f64) -> f64 {
    wind_at(u2, z) * libm::sqrt(tau_d.clamp(0.0, 1.0))
}

/// Saturation vapor pressure over water, kPa (Tetens; FAO-56 eq. 11, the same form as
/// ps-weather's, so the leaf-to-air deficit equals the air's at leaf = air temperature).
#[must_use]
pub fn saturation_kpa(t_c: f64) -> f64 {
    0.6108 * libm::exp(17.27 * t_c / (t_c + 237.3))
}

/// Slope of [`saturation_kpa`], kPa K⁻¹.
#[must_use]
pub fn saturation_slope(t_c: f64) -> f64 {
    let d = t_c + 237.3;
    saturation_kpa(t_c) * 17.27 * 237.3 / (d * d)
}

/// Latent heat of vaporization, J mol⁻¹ (as in plantecophys).
#[must_use]
pub fn latent_heat(t_c: f64) -> f64 {
    (2.501e6 - 2365.0 * t_c) * 0.018_015
}

/// Absorbed shortwave (PAR and near infrared), W m⁻², from absorbed PAR.
#[must_use]
pub fn shortwave(par: UmolPerM2S, nir_ratio: f64) -> f64 {
    par.0.max(0.0) / UMOL_PER_J_PAR * (1.0 + nir_ratio)
}

/// The hour's air terms the balance reads, computed once per hour from an [`Air`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AirTerms {
    /// The air.
    pub air: Air,
    /// Air vapor pressure, kPa.
    ea: f64,
    /// Air temperature, K.
    tk: f64,
    /// Latent heat at air temperature, J mol⁻¹.
    lambda: f64,
    /// Black-body emission at air temperature, W m⁻².
    sigma_t4: f64,
    /// Sky emissivity.
    eps_sky: f64,
    /// Saturation slope at air temperature, kPa K⁻¹.
    slope: f64,
    /// Molar density of air, mol m⁻³.
    c_air: f64,
}

impl Air {
    /// The terms the balance reads, once per hour.
    #[must_use]
    pub fn terms(&self) -> AirTerms {
        let tk = self.t_air.0 + ZERO_C_K;
        AirTerms {
            air: *self,
            ea: self.ea(),
            tk,
            lambda: latent_heat(self.t_air.0),
            sigma_t4: SIGMA * tk * tk * tk * tk,
            eps_sky: self.sky_emissivity(),
            slope: saturation_slope(self.t_air.0),
            c_air: self.patm.0 * 1000.0 / (R_GAS * tk),
        }
    }
}

/// Forced-convection conductance to heat of one face at wind `u`, mol m⁻² s⁻¹.
fn forced_heat(p: &LeafEnergyParams, at: &AirTerms, u: f64) -> f64 {
    ENHANCEMENT * FORCED * libm::sqrt(u.max(WIND_FLOOR) / p.dimension()) * at.c_air
}

/// Free-convection conductance to heat of one face at leaf-air difference `dt`,
/// mol m⁻² s⁻¹ (Leuning et al. 1995: `0.5 D Gr^¼ / d`, Gr = 1.6e8 |ΔT| d³).
fn free_heat(p: &LeafEnergyParams, at: &AirTerms, dt: f64) -> f64 {
    let d = p.dimension();
    let grashof = 1.6e8 * dt.abs() * d * d * d;
    0.5 * D_HEAT * libm::sqrt(libm::sqrt(grashof)) / d * at.c_air
}

/// Two faces in parallel, each its stomata in series with its boundary layer.
fn vapor_conductance(gs: f64, gbw_face: f64, abaxial: f64) -> f64 {
    let series = |a: f64, b: f64| {
        if a > 0.0 && b > 0.0 {
            a * b / (a + b)
        } else {
            0.0
        }
    };
    let ab = abaxial.clamp(0.0, 1.0);
    series(gs * ab, gbw_face) + series(gs * (1.0 - ab), gbw_face)
}

/// Newton steps on the full balance after the linearised estimate.
pub const NEWTON_STEPS: u32 = 2;

/// Leaf temperature and transpiration of one leaf part at stomatal conductance `gs`
/// (mol m⁻² s⁻¹ to water vapor): the linearised balance (Penman–Monteith form; free
/// convection at the forced-only estimate), then [`NEWTON_STEPS`] Newton steps on the
/// full balance with the boundary layer at each step's own leaf-air difference. Each
/// face's boundary layer conducts heat by forced convection at the leaf's wind plus
/// free convection (Leuning et al. 1995).
#[must_use]
pub fn leaf_temperature(p: &LeafEnergyParams, at: &AirTerms, site: &Site, gs: f64) -> LeafHeat {
    let air = &at.air;
    let ta = air.t_air.0;
    let tk = at.tk;
    let pk = air.patm.0;
    let eps = p.emissivity;
    let (ea, lambda) = (at.ea, at.lambda);
    let s_abs = shortwave(site.par, p.nir_ratio);
    let f_sky = site.sky_view.clamp(0.0, 1.0);
    // Absorbed longwave: the upper face sees the sky over its sky view and surroundings
    // at air temperature elsewhere; the lower face sees surroundings at air temperature.
    let l_abs = eps * at.sigma_t4 * ((2.0 - f_sky) + f_sky * at.eps_sky);
    let rni = s_abs + l_abs - 2.0 * eps * at.sigma_t4;
    let g_r = 4.0 * eps * SIGMA * tk * tk * tk / CP_AIR;
    let d_air = air.vpd.0.max(0.0);
    let forced = forced_heat(p, at, site.wind);
    let gv_of = |gbh: f64| vapor_conductance(gs, VAPOR_PER_HEAT * gbh, p.abaxial_share);
    let linear = |gbh: f64| {
        let gv = gv_of(gbh);
        (rni - lambda * gv * d_air / pk)
            / (2.0 * CP_AIR * (gbh + g_r) + lambda * gv * at.slope / pk)
    };
    let dt0 = linear(forced);
    let mut t = ta + linear(forced + free_heat(p, at, dt0));
    // Newton on F(T) = S + L − 2εσT⁴ − 2 cp gbh(T) (T − Ta) − λ gv(T) (es(T) − ea)/P.
    // Free convection grows as |T − Ta|^¼, so d[gbh (T − Ta)]/dT = forced + 5/4 free.
    for _ in 0..NEWTON_STEPS {
        let dt = t - ta;
        let free = free_heat(p, at, dt);
        let gbh = forced + free;
        let gv = gv_of(gbh);
        let t_k = t + ZERO_C_K;
        let es = saturation_kpa(t);
        let den = t + 237.3;
        let f = s_abs + l_abs
            - 2.0 * eps * SIGMA * t_k * t_k * t_k * t_k
            - 2.0 * CP_AIR * gbh * dt
            - lambda * gv * (es - ea) / pk;
        let df = -8.0 * eps * SIGMA * t_k * t_k * t_k
            - 2.0 * CP_AIR * (forced + 1.25 * free)
            - lambda * gv * es * 17.27 * 237.3 / (den * den) / pk;
        if df < 0.0 {
            t -= f / df;
        }
    }
    let gv = gv_of(forced + free_heat(p, at, t - ta));
    LeafHeat {
        t_leaf: Celsius(t),
        e: MolPerM2S(gv * (saturation_kpa(t) - ea) / pk),
        g_v: MolPerM2S(gv),
    }
}

/// Gas exchange at leaf temperature `t`, with stomata seeing the leaf-to-air deficit.
fn gas_at(
    lp: &LeafParams,
    at: &AirTerms,
    site: &Site,
    ca: Ppm,
    capacity: f64,
    stress: f64,
    t: f64,
) -> GasExchange {
    let env = LeafEnv {
        par: site.par,
        t_leaf: Celsius(t),
        vpd: KPa((saturation_kpa(t) - at.ea).max(0.0)),
        ca,
        patm: at.air.patm,
    };
    gas::solve(lp, &env, capacity, stress)
}

/// Gas exchange and leaf temperature solved together (decision 0034). Returns the gas
/// exchange at the leaf's temperature (its `e` replaced by the energy balance's, with
/// the boundary layer in series) and the heat balance. Stomata respond to leaf
/// temperature and the leaf-to-air deficit; the CO2 supply has no boundary-layer term
/// (as plantecophys `Photosyn`, so the solver keeps its fixture).
///
/// Starts from air temperature; [`solve_coupled_from`] takes a warm start.
#[must_use]
pub fn solve_coupled(
    lp: &LeafParams,
    ep: &LeafEnergyParams,
    air: &Air,
    site: &Site,
    ca: Ppm,
    capacity: f64,
    stress: f64,
) -> (GasExchange, LeafHeat) {
    let (g, h, _) = solve_coupled_from(lp, ep, &air.terms(), site, ca, capacity, stress, air.t_air);
    (g, h)
}

/// [`solve_coupled`] from a first guess of the leaf temperature `start` (a class
/// table passes its neighboring node's), also returning the gas-exchange solves used.
///
/// The iteration: two fixed-point rounds from `start`, then a clamped secant until
/// `g(T) = T_eb(gs(T)) − T` changes sign, then Illinois (modified regula falsi), until
/// `|g| <` [`T_TOLERANCE`] or the bracket is that narrow, at most [`MAX_EVALUATIONS`]
/// gas-exchange solves; the answer is the evaluation nearest its own balance. Plain
/// fixed-point iteration fails in hot, still air, where a warmer leaf loses
/// assimilation, closes its stomata and warms further (research prototype: up to 4 K).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn solve_coupled_from(
    lp: &LeafParams,
    ep: &LeafEnergyParams,
    at: &AirTerms,
    site: &Site,
    ca: Ppm,
    capacity: f64,
    stress: f64,
    start: Celsius,
) -> (GasExchange, LeafHeat, u32) {
    let ta = at.air.t_air.0;
    // In the dark stomata sit at g0 whatever the temperature: one balance does.
    if site.par.0 <= 0.0 {
        let g0 = gas_at(lp, at, site, ca, capacity, stress, ta);
        let h = leaf_temperature(ep, at, site, g0.gs.0);
        let g = gas_at(lp, at, site, ca, capacity, stress, h.t_leaf.0);
        return (with_e(g, &h), h, 2);
    }
    let mut evals = 0u32;
    // The evaluation nearest its own balance so far: |g|, gas exchange, heat.
    let mut best: Option<(f64, GasExchange, LeafHeat)> = None;
    let mut eval = |t: f64| {
        evals += 1;
        let g = gas_at(lp, at, site, ca, capacity, stress, t);
        let h = leaf_temperature(ep, at, site, g.gs.0);
        let r = h.t_leaf.0 - t;
        if best.as_ref().is_none_or(|b| r.abs() < b.0) {
            best = Some((r.abs(), g, h));
        }
        (r, evals)
    };
    let (lo_t, hi_t) = (ta - T_SPAN_LOW, ta + T_SPAN_HIGH);
    let mut a = start.0.clamp(lo_t, hi_t);
    let (mut ga, mut n) = eval(a);
    if ga.abs() >= T_TOLERANCE {
        let mut b = (a + ga).clamp(lo_t, hi_t);
        let mut gb;
        (gb, n) = eval(b);
        // Extend by clamped secant steps until bracketed.
        while ga * gb > 0.0
            && gb.abs() >= T_TOLERANCE
            && n < MAX_EVALUATIONS
            && (b - a).abs() > T_TOLERANCE
        {
            let s = if (gb - ga).abs() > 1e-12 {
                b - gb * (b - a) / (gb - ga)
            } else {
                b + gb
            };
            a = b;
            ga = gb;
            b = s.clamp(lo_t, hi_t);
            (gb, n) = eval(b);
        }
        // Illinois (modified regula falsi) on the bracket.
        while ga * gb < 0.0
            && gb.abs() >= T_TOLERANCE
            && n < MAX_EVALUATIONS
            && (b - a).abs() > T_TOLERANCE
        {
            let c = (a * gb - b * ga) / (gb - ga);
            let (gc, k) = eval(c);
            n = k;
            if gc * gb < 0.0 {
                a = b;
                ga = gb;
            } else {
                ga *= 0.5;
            }
            b = c;
            gb = gc;
        }
    }
    match best {
        Some((_, g, h)) => (with_e(g, &h), h, n),
        None => unreachable!("at least one evaluation"),
    }
}

/// The gas exchange with the energy balance's transpiration.
fn with_e(g: GasExchange, h: &LeafHeat) -> GasExchange {
    GasExchange { e: h.e, ..g }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::bean;

    /// Bean leaflet energy parameters (decision 0034).
    fn bean_energy() -> LeafEnergyParams {
        LeafEnergyParams {
            width: Meters(0.08),
            nir_ratio: 0.235,
            emissivity: 0.97,
            abaxial_share: 0.875,
        }
    }

    fn bean_leaf() -> LeafParams {
        let (s, t) = bean();
        LeafParams::from_species(&s, &t)
    }

    fn air(t: f64, rh_pct: f64) -> Air {
        Air {
            t_air: Celsius(t),
            vpd: KPa(saturation_kpa(t) * (1.0 - rh_pct / 100.0)),
            patm: KPa(101.3),
            cloud: 0.0,
        }
    }

    fn sunlit(wind: f64) -> Site {
        Site {
            par: UmolPerM2S(1500.0),
            sky_view: 1.0,
            wind,
        }
    }

    /// Leaf minus air temperature of the coupled solve.
    fn dt(lp: &LeafParams, ep: &LeafEnergyParams, a: &Air, s: &Site, stress: f64) -> f64 {
        let (_, h) = solve_coupled(lp, ep, a, s, Ppm(420.0), 1.0, stress);
        h.t_leaf.0 - a.t_air.0
    }

    /// WP27 done-when, case 1: a hot still day, well watered (Ta 35 °C, RH 30%, 0.5 m/s
    /// at the leaf): leaf minus air in −7 to +1 K.
    #[test]
    fn hot_still_day_well_watered() {
        let d = dt(
            &bean_leaf(),
            &bean_energy(),
            &air(35.0, 30.0),
            &sunlit(0.5),
            1.0,
        );
        println!("hot still day: {d:+.2} K");
        assert!((-7.0..=1.0).contains(&d), "{d:+.2} K");
    }

    /// Case 2: at VPD 3 kPa or more and 1 m/s or more, a well-watered sunlit leaf is
    /// cooler than the air.
    #[test]
    fn dry_air_and_wind_cool_a_watered_leaf() {
        let (lp, ep) = (bean_leaf(), bean_energy());
        for (t, rh) in [(30.0, 25.0), (35.0, 30.0), (38.0, 40.0)] {
            let a = air(t, rh);
            assert!(a.vpd.0 >= 3.0, "{t} {rh}: {}", a.vpd.0);
            for u in [1.0, 2.0, 4.0] {
                let d = dt(&lp, &ep, &a, &sunlit(u), 1.0);
                println!("Ta {t}, RH {rh}, u {u}: {d:+.2} K");
                assert!(d < 0.0, "Ta {t}, RH {rh}, u {u}: {d:+.2} K");
            }
        }
    }

    /// Case 3: stomata closed by drought (stress factor 0) heat the leaf by 1 to 8 K, at
    /// least 3 K more than a watered leaf in the same air.
    #[test]
    fn closed_stomata_heat_the_leaf() {
        let (lp, ep) = (bean_leaf(), bean_energy());
        let (a, s) = (air(35.0, 30.0), sunlit(0.5));
        let wet = dt(&lp, &ep, &a, &s, 1.0);
        let dry = dt(&lp, &ep, &a, &s, 0.0);
        println!("closed {dry:+.2} K, watered {wet:+.2} K");
        assert!((1.0..=8.0).contains(&dry), "{dry:+.2} K");
        assert!(dry - wet >= 3.0, "dry {dry:+.2} vs watered {wet:+.2}");
    }

    /// Case 4: on a clear calm night an exposed leaf radiates to the sky and runs 0.5
    /// to 3 K below the air.
    #[test]
    fn clear_night_cools_an_exposed_leaf() {
        let (lp, ep) = (bean_leaf(), bean_energy());
        let a = air(15.0, 80.0);
        let s = Site {
            par: UmolPerM2S(0.0),
            sky_view: 1.0,
            wind: 0.5,
        };
        let d = dt(&lp, &ep, &a, &s, 1.0);
        println!("clear night: {d:+.2} K");
        assert!((-3.0..=-0.5).contains(&d), "{d:+.2} K");
        // Under cover (no sky) it stays at the air's temperature, and clouds warm it.
        let covered = dt(&lp, &ep, &a, &Site { sky_view: 0.0, ..s }, 1.0);
        assert!(covered.abs() < 0.1, "{covered:+.3} K");
        let cloudy = dt(&lp, &ep, &Air { cloud: 1.0, ..a }, &s, 1.0);
        assert!(cloudy > d + 0.5, "cloudy {cloudy:+.2} vs clear {d:+.2}");
    }

    /// Case 5: irrigated wheat in hot, dry air runs below air temperature (field
    /// canopies: 4 to 8 K below).
    #[test]
    fn irrigated_wheat_runs_cool() {
        let s =
            ps_species::Species::from_ron(include_str!("../../../data/species/spring_wheat.ron"))
                .unwrap();
        let t = s.phenotype(&s.default_genotype()).traits().unwrap();
        let lp = LeafParams::from_species(&s, &t);
        let ep = LeafEnergyParams {
            width: Meters(0.015),
            nir_ratio: 0.235,
            emissivity: 0.97,
            abaxial_share: 0.5,
        };
        for ta in [33.0, 35.0, 37.0] {
            let d = dt(&lp, &ep, &air(ta, 20.0), &sunlit(2.0), 1.0);
            println!("wheat Ta {ta}: {d:+.2} K");
            assert!(d < 0.0, "Ta {ta}: {d:+.2} K");
        }
    }

    /// The closed form plus the Newton steps solve the full balance to 0.02 K across hot, cold, still, windy, open and closed leaves.
    #[test]
    fn newton_steps_close_the_balance() {
        let ep = bean_energy();
        for (t, rh) in [(5.0, 90.0), (20.0, 60.0), (35.0, 30.0), (42.0, 15.0)] {
            let a = air(t, rh);
            for u in [0.05, 0.5, 3.0] {
                for gs in [0.0, 0.05, 0.4, 1.0] {
                    for par in [0.0, 500.0, 2000.0] {
                        let s = Site {
                            par: UmolPerM2S(par),
                            sky_view: 0.8,
                            wind: u,
                        };
                        let h = leaf_temperature(&ep, &a.terms(), &s, gs);
                        let tl = h.t_leaf.0;
                        let tk = tl + ZERO_C_K;
                        let tak = t + ZERO_C_K;
                        let at = a.terms();
                        let gbh = forced_heat(&ep, &at, u) + free_heat(&ep, &at, tl - t);
                        let gv = vapor_conductance(gs, VAPOR_PER_HEAT * gbh, ep.abaxial_share);
                        let eps = ep.emissivity;
                        let l_abs = eps
                            * SIGMA
                            * (tak * tak * tak * tak)
                            * ((2.0 - 0.8) + 0.8 * a.sky_emissivity());
                        let f = shortwave(s.par, ep.nir_ratio) + l_abs
                            - 2.0 * eps * SIGMA * (tk * tk * tk * tk)
                            - 2.0 * CP_AIR * gbh * (tl - t)
                            - latent_heat(t) * gv * (saturation_kpa(tl) - a.ea()) / a.patm.0;
                        // As a temperature error: the residual over the balance's
                        // slope (radiation, convection, latent heat).
                        let slope = 8.0 * eps * SIGMA * (tk * tk * tk)
                            + 2.0 * CP_AIR * gbh
                            + latent_heat(t) * gv * saturation_slope(tl) / a.patm.0;
                        assert!(
                            (f / slope).abs() < 0.02,
                            "T {t} RH {rh} u {u} gs {gs} PAR {par}: {f:.4} W/m2 at {tl:.3}"
                        );
                    }
                }
            }
        }
    }

    /// The coupled solve agrees with a bisection on `T_eb(gs(T)) − T` to 0.05 K over hot
    /// and cold, still and windy, wet and dry cases, within its evaluation cap.
    #[test]
    fn coupled_solve_matches_bisection() {
        let (lp, ep) = (bean_leaf(), bean_energy());
        let mut worst: f64 = 0.0;
        for (t, rh) in [
            (5.0, 80.0),
            (20.0, 60.0),
            (30.0, 40.0),
            (35.0, 30.0),
            (42.0, 40.0),
        ] {
            let a = air(t, rh);
            for u in [0.05, 0.2, 1.0, 5.0] {
                for par in [50.0, 500.0, 2000.0] {
                    for stress in [0.0, 0.3, 1.0] {
                        let s = Site {
                            par: UmolPerM2S(par),
                            sky_view: 1.0,
                            wind: u,
                        };
                        let (_, h) = solve_coupled(&lp, &ep, &a, &s, Ppm(420.0), 1.0, stress);
                        let g = |x: f64| {
                            let gx = gas_at(&lp, &a.terms(), &s, Ppm(420.0), 1.0, stress, x);
                            leaf_temperature(&ep, &a.terms(), &s, gx.gs.0).t_leaf.0 - x
                        };
                        let (mut lo, mut hi) = (t - 20.0, t + 25.0);
                        assert!(g(lo) > 0.0 && g(hi) < 0.0);
                        for _ in 0..60 {
                            let mid = 0.5 * (lo + hi);
                            if g(mid) > 0.0 {
                                lo = mid;
                            } else {
                                hi = mid;
                            }
                        }
                        let err = (h.t_leaf.0 - 0.5 * (lo + hi)).abs();
                        worst = worst.max(err);
                        assert!(
                            err < 0.05,
                            "T {t} RH {rh} u {u} PAR {par} stress {stress}: {err:.4} K"
                        );
                    }
                }
            }
        }
        println!("worst {worst:.5} K");
    }
}
