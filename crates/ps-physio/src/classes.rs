//! Per-class gas exchange (§3.8 "per light class, not per leaf").
//!
//! Each hour each plant solves gas exchange on a small table: 17 absorbed-PAR nodes
//! (zero, then 16 log-spaced from 1 to 2000 µmol m⁻² s⁻¹) × 4 capacity nodes (1.0, 0.8,
//! 0.6, 0.4, for leaf aging). A leaf reads its rates by interpolation, linear in PAR
//! between neighboring nodes and linear in capacity (decision 0009).
//! Sunlit and shaded parts read the same table at their own PAR.

use ps_units::{MolPerM2S, UmolPerM2S};

use crate::gas::{self, GasExchange, LeafEnv, LeafParams};

/// Number of PAR classes above zero (§3.8).
pub const PAR_CLASSES: usize = 16;
/// Lowest nonzero class, µmol m⁻² s⁻¹.
pub const PAR_MIN: f64 = 1.0;
/// Highest class, µmol m⁻² s⁻¹.
pub const PAR_MAX: f64 = 2000.0;
/// Log-spaced steps between the lowest and highest nonzero class.
const STEPS: f64 = 15.0;
/// Capacity nodes, highest first.
pub const CAPACITY_NODES: [f64; 4] = [1.0, 0.8, 0.6, 0.4];

/// Rates per unit leaf area read from the table.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rates {
    /// Net assimilation.
    pub a_net: UmolPerM2S,
    /// Stomatal conductance to water vapor.
    pub gs: MolPerM2S,
    /// Transpiration.
    pub e: MolPerM2S,
    /// Day respiration (included in `a_net`).
    pub rd: UmolPerM2S,
}

impl Rates {
    fn of(g: &GasExchange) -> Self {
        Self {
            a_net: g.a_net,
            gs: g.gs,
            e: g.e,
            rd: g.rd,
        }
    }

    fn lerp(a: Self, b: Self, w: f64) -> Self {
        let l = |x: f64, y: f64| x + (y - x) * w;
        Self {
            a_net: UmolPerM2S(l(a.a_net.0, b.a_net.0)),
            gs: MolPerM2S(l(a.gs.0, b.gs.0)),
            e: MolPerM2S(l(a.e.0, b.e.0)),
            rd: UmolPerM2S(l(a.rd.0, b.rd.0)),
        }
    }
}

/// PAR at node `i` (0 is darkness).
#[must_use]
pub fn par_node(i: usize) -> f64 {
    if i == 0 {
        return 0.0;
    }
    let f = f64::from(u32::try_from(i - 1).unwrap_or(0)) / STEPS;
    PAR_MIN * libm::pow(PAR_MAX / PAR_MIN, f)
}

/// The class table of one plant for one hour.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassTable {
    /// `rates[c][p]`: capacity node `c`, PAR node `p`.
    pub rates: [[Rates; PAR_CLASSES + 1]; 4],
    /// Full solver output at capacity 1, for the inspector.
    pub full: [GasExchange; PAR_CLASSES + 1],
    /// `par_node(i)` for every node, cached so lookups do not call `pow`.
    pub nodes: [f64; PAR_CLASSES + 1],
}

impl ClassTable {
    /// Solves the table for the hour's leaf environment (its `par` is ignored) and the
    /// plant's water stress factor. 68 solves.
    #[must_use]
    pub fn build(p: &LeafParams, env: &LeafEnv, stress: f64) -> Self {
        let mut rates = [[Rates::default(); PAR_CLASSES + 1]; 4];
        let mut full = [gas::solve(p, env, 1.0, stress); PAR_CLASSES + 1];
        for (c, cap) in CAPACITY_NODES.iter().enumerate() {
            for (k, slot) in rates[c].iter_mut().enumerate() {
                let e = LeafEnv {
                    par: UmolPerM2S(par_node(k)),
                    ..*env
                };
                let g = gas::solve(p, &e, *cap, stress);
                if c == 0 {
                    full[k] = g;
                }
                *slot = Rates::of(&g);
            }
        }
        let mut nodes = [0.0; PAR_CLASSES + 1];
        for (i, n) in nodes.iter_mut().enumerate() {
            *n = par_node(i);
        }
        Self { rates, full, nodes }
    }

    /// Rates at absorbed PAR `par` and capacity `capacity` (clamped to the table).
    #[must_use]
    pub fn lookup(&self, par: UmolPerM2S, capacity: f64) -> Rates {
        let q = par.0.clamp(0.0, PAR_MAX);
        // PAR position: node index k and weight toward k + 1.
        let (k, w) = if q < PAR_MIN {
            (0, q / PAR_MIN)
        } else {
            let span = libm::log(PAR_MAX / PAR_MIN);
            let x = libm::log(q / PAR_MIN) / span * STEPS;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let i = (libm::floor(x) as usize).min(PAR_CLASSES - 2);
            // Linear in PAR between the two log-spaced nodes: A is nearly linear in
            // PAR at low light, where log interpolation overestimates it by 3%.
            let (lo, hi) = (self.nodes[i + 1], self.nodes[i + 2]);
            (i + 1, ((q - lo) / (hi - lo)).clamp(0.0, 1.0))
        };
        let at =
            |c: usize| Rates::lerp(self.rates[c][k], self.rates[c][(k + 1).min(PAR_CLASSES)], w);
        // Capacity position.
        let cap = capacity.clamp(CAPACITY_NODES[3], CAPACITY_NODES[0]);
        let pos = (CAPACITY_NODES[0] - cap) / (CAPACITY_NODES[0] - CAPACITY_NODES[1]);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let c = (libm::floor(pos) as usize).min(2);
        let wc = (pos - f64::from(u32::try_from(c).unwrap_or(0))).clamp(0.0, 1.0);
        Rates::lerp(at(c), at(c + 1), wc)
    }
}
