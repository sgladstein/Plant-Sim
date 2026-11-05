//! Coupled Farquhar–von Caemmerer–Berry photosynthesis and Medlyn stomatal conductance
//! for one leaf class (design plan §3.8, §11.4). Model card: docs/models/gas-exchange.md.

use ps_units::{Celsius, JPerMol, JPerMolK, KPa, MolPerM2S, Ppm, SqrtKPa, UmolPerM2S};
use serde::{Deserialize, Serialize};

/// Gas constant, J mol⁻¹ K⁻¹ (the value plantecophys uses).
pub const R_GAS: f64 = 8.314;
/// Ratio of the diffusivities of water vapor and CO2 in air (plantecophys hard-wires
/// 1.57; Medlyn et al. 2011 write 1.6; decision 0009).
pub const H2O_PER_CO2: f64 = 1.57;
/// Reference temperature of all `*25` parameters, K.
pub const T_REF_K: f64 = 298.15;
/// Kelvin offset.
pub const ZERO_C_K: f64 = 273.15;
/// Michaelis constant of Rubisco for CO2 at 25 °C, µmol mol⁻¹ (Bernacchi et al. 2001).
pub const KC25: f64 = 404.9;
/// Activation energy of Kc, J mol⁻¹ (Bernacchi et al. 2001).
pub const KC_EA: f64 = 79_430.0;
/// Michaelis constant of Rubisco for O2 at 25 °C, mmol mol⁻¹ (Bernacchi et al. 2001).
pub const KO25: f64 = 278.4;
/// Activation energy of Ko, J mol⁻¹ (Bernacchi et al. 2001).
pub const KO_EA: f64 = 36_380.0;
/// O2 mole fraction at 100 kPa, mmol mol⁻¹.
pub const O2: f64 = 210.0;
/// CO2 compensation point without day respiration at 25 °C, µmol mol⁻¹ (Bernacchi et al. 2001).
pub const GAMMA_STAR25: f64 = 42.75;
/// Activation energy of Γ*, J mol⁻¹ (Bernacchi et al. 2001).
pub const GAMMA_STAR_EA: f64 = 37_830.0;
/// Medlyn's VPD floor, kPa (plantecophys `vpdmin`): below it the slope stops rising.
pub const VPD_MIN: f64 = 0.5;
/// Ci tolerance of the solver, µmol mol⁻¹ (§11.4).
pub const CI_TOLERANCE: f64 = 1e-6;
/// Iteration cap of the solver (§11.4).
pub const MAX_ITERATIONS: u32 = 50;

/// Peaked Arrhenius temperature response (Medlyn et al. 2002, eq. 18).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeakedArrhenius {
    /// Activation energy.
    pub activation: JPerMol,
    /// Deactivation energy (zero switches the peak off).
    pub deactivation: JPerMol,
    /// Entropy term.
    pub entropy: JPerMolK,
}

impl PeakedArrhenius {
    /// Multiplier on the 25 °C value at leaf temperature `t`.
    #[must_use]
    pub fn factor(&self, t: Celsius) -> f64 {
        let tk = t.0 + ZERO_C_K;
        let ea = self.activation.0;
        let ed = self.deactivation.0;
        let ds = self.entropy.0;
        let arr = libm::exp((tk - T_REF_K) * ea / (R_GAS * tk * T_REF_K));
        if ed > 0.0 {
            let v25 = 1.0 + libm::exp((ds * T_REF_K - ed) / (R_GAS * T_REF_K));
            let vt = 1.0 + libm::exp((ds * tk - ed) / (R_GAS * tk));
            arr * v25 / vt
        } else {
            arr
        }
    }
}

/// Plain Arrhenius multiplier on a 25 °C value.
fn arrhenius(t: Celsius, ea: f64) -> f64 {
    let tk = t.0 + ZERO_C_K;
    libm::exp(ea * (tk - T_REF_K) / (T_REF_K * R_GAS * tk))
}

/// Leaf photosynthetic parameters (species phenotype plus constants).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeafParams {
    /// Maximum carboxylation rate at 25 °C.
    pub vcmax25: UmolPerM2S,
    /// Maximum electron transport rate at 25 °C.
    pub jmax25: UmolPerM2S,
    /// Day respiration at 25 °C.
    pub rd25: UmolPerM2S,
    /// Q10 of day respiration.
    pub rd_q10: f64,
    /// Medlyn slope.
    pub g1: SqrtKPa,
    /// Residual conductance to water vapor.
    pub g0: MolPerM2S,
    /// Electrons per absorbed photon.
    pub quantum_yield: f64,
    /// Curvature of the light response of J.
    pub curvature: f64,
    /// Vcmax temperature response.
    pub vcmax_t: PeakedArrhenius,
    /// Jmax temperature response.
    pub jmax_t: PeakedArrhenius,
}

/// The environment of one leaf class for one hour.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeafEnv {
    /// Absorbed PAR.
    pub par: UmolPerM2S,
    /// Leaf temperature (air temperature in M0).
    pub t_leaf: Celsius,
    /// Leaf-to-air vapor pressure deficit.
    pub vpd: KPa,
    /// Ambient CO2.
    pub ca: Ppm,
    /// Air pressure.
    pub patm: KPa,
}

/// Which process limits assimilation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Limitation {
    /// Rubisco carboxylation (Ac).
    Rubisco,
    /// Regeneration of `RuBP` by electron transport (Aj).
    Electron,
    /// Net assimilation is not positive even at Ci = Ca: stomata at g0.
    Dark,
}

/// Gas exchange per unit leaf area.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GasExchange {
    /// Net assimilation (gross minus day respiration).
    pub a_net: UmolPerM2S,
    /// Day respiration.
    pub rd: UmolPerM2S,
    /// Stomatal conductance to water vapor.
    pub gs: MolPerM2S,
    /// Transpiration.
    pub e: MolPerM2S,
    /// Intercellular CO2.
    pub ci: Ppm,
    /// Limiting process, for the inspector.
    pub limitation: Limitation,
    /// Solver iterations used (both limitations).
    pub iterations: u32,
}

/// Temperature-adjusted kinetics for one leaf and hour.
#[derive(Copy, Clone, Debug)]
struct Kinetics {
    vcmax: f64,
    vj: f64,
    km: f64,
    gamma_star: f64,
    rd: f64,
}

/// The temperature and pressure terms of [`kinetics`], which depend only on the
/// leaf's temperature and the air pressure, not on PAR or capacity. A class table
/// computes them once for its 68 solves.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TemperatureTerms {
    gamma_star: f64,
    km: f64,
    vcmax_factor: f64,
    jmax_factor: f64,
    rd_factor: f64,
}

impl TemperatureTerms {
    /// The terms for parameters `p` at the leaf temperature and air pressure of `env`.
    #[must_use]
    pub fn new(p: &LeafParams, env: &LeafEnv) -> Self {
        let t = env.t_leaf;
        let pscale = env.patm.0 / 100.0;
        let gamma_star = GAMMA_STAR25 * arrhenius(t, GAMMA_STAR_EA) * pscale;
        let ko = KO25 * arrhenius(t, KO_EA);
        let km = KC25 * arrhenius(t, KC_EA) * (1.0 + O2 * pscale / ko);
        Self {
            gamma_star,
            km,
            vcmax_factor: p.vcmax_t.factor(t),
            jmax_factor: p.jmax_t.factor(t),
            rd_factor: libm::pow(p.rd_q10, (t.0 - 25.0) / 10.0),
        }
    }
}

fn kinetics(p: &LeafParams, env: &LeafEnv, capacity: f64, tt: &TemperatureTerms) -> Kinetics {
    let (gamma_star, km) = (tt.gamma_star, tt.km);
    let vcmax = capacity * p.vcmax25.0 * tt.vcmax_factor;
    let jmax = capacity * p.jmax25.0 * tt.jmax_factor;
    let rd = capacity * p.rd25.0 * tt.rd_factor;
    // Non-rectangular hyperbola of absorbed PAR.
    let (a, th) = (p.quantum_yield, p.curvature);
    let q = env.par.0.max(0.0);
    let s = a * q + jmax;
    let j = (s - libm::sqrt((s * s - 4.0 * a * th * q * jmax).max(0.0))) / (2.0 * th);
    Kinetics {
        vcmax,
        vj: j / 4.0,
        km,
        gamma_star,
        rd,
    }
}

/// One limitation's demand curve: `A(Ci) = v (Ci − Γ*) / (Ci + k) − Rd`.
#[derive(Copy, Clone, Debug)]
struct Demand {
    v: f64,
    k: f64,
    gamma_star: f64,
    rd: f64,
}

impl Demand {
    fn a(&self, ci: f64) -> f64 {
        self.v * (ci - self.gamma_star) / (ci + self.k) - self.rd
    }
    fn da(&self, ci: f64) -> f64 {
        self.v * (self.k + self.gamma_star) / ((ci + self.k) * (ci + self.k))
    }
}

/// Safeguarded Newton on `f` over `[lo, hi]` with `f(lo) < 0 < f(hi)`: a Newton step
/// that leaves the bracket, or does not shrink the residual by half, is replaced by
/// bisection. Returns the root and the iteration count.
fn solve_bracketed(
    f: impl Fn(f64) -> (f64, f64),
    mut lo: f64,
    mut hi: f64,
    start: f64,
) -> (f64, u32) {
    let mut x = start.clamp(lo, hi);
    let (mut fx, mut dfx) = f(x);
    for it in 1..=MAX_ITERATIONS {
        if fx < 0.0 {
            lo = x;
        } else {
            hi = x;
        }
        let newton = if dfx > 0.0 { x - fx / dfx } else { f64::NAN };
        let mut next = if newton > lo && newton < hi {
            newton
        } else {
            0.5 * (lo + hi)
        };
        let (mut fn_, mut dfn) = f(next);
        if fn_.abs() > 0.5 * fx.abs() && (hi - lo) > CI_TOLERANCE {
            let mid = 0.5 * (lo + hi);
            if (mid - next).abs() > CI_TOLERANCE {
                next = mid;
                (fn_, dfn) = f(next);
            }
        }
        let step = (next - x).abs();
        x = next;
        fx = fn_;
        dfx = dfn;
        if step < CI_TOLERANCE || hi - lo < CI_TOLERANCE || fx == 0.0 {
            return (x, it);
        }
    }
    (x, MAX_ITERATIONS)
}

/// Solves the coupled system for one limitation. Returns `(A, Ci, iterations)`, or
/// `None` when net assimilation is not positive at `Ci = Ca`.
fn solve_limitation(d: &Demand, ca: f64, m: f64, g0c: f64) -> Option<(f64, f64, u32)> {
    if d.a(ca) <= 0.0 {
        return None;
    }
    // Supply with Medlyn conductance: A = (g0c + m A)(Ca − Ci). Residual
    // f(Ci) = A(Ci)(1 − m (Ca − Ci)) − g0c (Ca − Ci), increasing on the bracket from
    // Ci = Ca − 1/m (infinite conductance per unit A) where f < 0 to Ci = Ca where f > 0.
    let f = |ci: f64| {
        let x = ca - ci;
        let a = d.a(ci);
        let val = a * (1.0 - m * x) - g0c * x;
        let der = d.da(ci) * (1.0 - m * x) + a * m + g0c;
        (val, der)
    };
    let lo = (ca - 1.0 / m).max(0.0);
    let start = (0.7 * ca).max(0.5 * (lo + ca));
    let (ci, it) = solve_bracketed(f, lo, ca, start);
    Some((d.a(ci), ci, it))
}

/// Coupled `FvCB` and Medlyn gas exchange for one leaf class (§3.8, §11.4).
///
/// `capacity` (0 to 1) scales Vcmax, Jmax and Rd for leaf aging (§3.8 "leaf aging");
/// `stress` (0 to 1) is the water stress factor, which multiplies the Medlyn slope term
/// (§3.2 hourly step 5: stress acts on stomatal conductance).
///
/// The Rubisco- and electron-limited cases are each solved by safeguarded Newton on
/// Ci; net assimilation is the smaller of the two.
#[must_use]
pub fn solve(p: &LeafParams, env: &LeafEnv, capacity: f64, stress: f64) -> GasExchange {
    solve_with(p, env, capacity, stress, &TemperatureTerms::new(p, env))
}

/// [`solve`] with the temperature terms already computed for `env` (bit-identical).
#[must_use]
pub fn solve_with(
    p: &LeafParams,
    env: &LeafEnv,
    capacity: f64,
    stress: f64,
    tt: &TemperatureTerms,
) -> GasExchange {
    let k = kinetics(p, env, capacity.clamp(0.0, 1.0), tt);
    let ca = env.ca.0.max(1.0);
    let vpd = env.vpd.0.max(VPD_MIN);
    let g0c = p.g0.0 / H2O_PER_CO2;
    // gs/A for CO2 (Medlyn 2011 without the 1.6, which converts to water vapor).
    let m = stress.clamp(0.0, 1.0) * (1.0 + p.g1.0 / libm::sqrt(vpd)) / ca;
    let rub = Demand {
        v: k.vcmax,
        k: k.km,
        gamma_star: k.gamma_star,
        rd: k.rd,
    };
    let ele = Demand {
        v: k.vj,
        k: 2.0 * k.gamma_star,
        gamma_star: k.gamma_star,
        rd: k.rd,
    };
    // Below the light compensation point (no positive net assimilation even at
    // Ci = Ca), stomata sit at g0 and Ci is taken as Ca, as plantecophys and MAESTRA do
    // (decision 0009); the fluxes involved are a fraction of a µmol m⁻² s⁻¹.
    let at_ca = rub.a(ca).min(ele.a(ca));
    if at_ca <= 0.0 {
        let gs = p.g0.0;
        return GasExchange {
            a_net: UmolPerM2S(at_ca),
            rd: UmolPerM2S(k.rd),
            gs: MolPerM2S(gs),
            e: MolPerM2S(gs * env.vpd.0.max(0.0) / env.patm.0),
            ci: Ppm(ca),
            limitation: Limitation::Dark,
            iterations: 0,
        };
    }
    let solved = if m > 0.0 {
        match (
            solve_limitation(&rub, ca, m, g0c),
            solve_limitation(&ele, ca, m, g0c),
        ) {
            (Some(c), Some(j)) => Some(if c.0 <= j.0 {
                (c.0, c.1, Limitation::Rubisco, c.2 + j.2)
            } else {
                (j.0, j.1, Limitation::Electron, c.2 + j.2)
            }),
            _ => None,
        }
    } else {
        None
    };
    let (a, ci, limitation, iterations) = solved.unwrap_or_else(|| {
        // Stomata at g0 (fully stressed): A = g0c (Ca − Ci), solved on the limiting
        // demand curve.
        dark(&rub, &ele, ca, g0c)
    });
    let gsc = (g0c + m * a).max(g0c);
    let gs = gsc * H2O_PER_CO2;
    let e = gs * env.vpd.0.max(0.0) / env.patm.0;
    GasExchange {
        a_net: UmolPerM2S(a),
        rd: UmolPerM2S(k.rd),
        gs: MolPerM2S(gs),
        e: MolPerM2S(e),
        ci: Ppm(ci),
        limitation,
        iterations,
    }
}

/// Assimilation with conductance fixed at `g0c`: the smaller root of
/// `A(Ci) = g0c (Ca − Ci)` over both demand curves.
fn dark(rub: &Demand, ele: &Demand, ca: f64, g0c: f64) -> (f64, f64, Limitation, u32) {
    let one = |d: &Demand| {
        if g0c <= 0.0 {
            // No conductance: Ci falls to the compensation point and A = 0 is
            // approximated by the demand at Γ* (−Rd).
            return (d.a(d.gamma_star), d.gamma_star, 0);
        }
        let f = |ci: f64| (d.a(ci) - g0c * (ca - ci), d.da(ci) + g0c);
        let lo = 0.0;
        let hi = ca + (d.v + d.rd) / g0c + 1.0;
        let (ci, it) = solve_bracketed(f, lo, hi, ca);
        (d.a(ci), ci, it)
    };
    let c = one(rub);
    let j = one(ele);
    let lim = |positive: bool, rubisco: bool| {
        if !positive {
            Limitation::Dark
        } else if rubisco {
            Limitation::Rubisco
        } else {
            Limitation::Electron
        }
    };
    if c.0 <= j.0 {
        (c.0, c.1, lim(c.0 > 0.0, true), c.2 + j.2)
    } else {
        (j.0, j.1, lim(j.0 > 0.0, false), c.2 + j.2)
    }
}
