//! Per-class gas exchange (§3.8 "per light class, not per leaf").
//!
//! Each hour each plant solves gas exchange on a small table: 17 absorbed-PAR nodes
//! (zero, then 16 log-spaced from 1 to 2000 µmol m⁻² s⁻¹) × 4 capacity nodes (1.0, 0.7,
//! 0.4, 0.1, for leaf aging and leaf nitrogen; decision 0020). A leaf reads its rates by
//! interpolation, linear in PAR between neighboring nodes and linear in capacity
//! (decision 0009). Capacity below 0.1 reads the 0.1 node.
//! Sunlit and shaded parts read the same table at their own PAR.
//!
//! With the leaf energy balance (WP27, decision 0034) each node solves gas exchange and
//! leaf temperature together at the hour's reference environment
//! ([`ClassTable::build_coupled`]): a horizontal leaf at [`REFERENCE_HEIGHT`] under the
//! open sky. Each leaf then corrects its own temperature and transpiration for its own
//! light, sky view and wind (ps-arch).

use ps_units::{Celsius, MolPerM2S, Ppm, UmolPerM2S};

use crate::energy::{self, Air, LeafEnergyParams, Site};
use crate::gas::{self, GasExchange, LeafEnv, LeafParams};

/// Height of a coupled table's reference leaf, m (a horizontal leaf near the top of a
/// small garden plant; "~").
pub const REFERENCE_HEIGHT: f64 = 0.5;

/// Number of PAR classes above zero (§3.8).
pub const PAR_CLASSES: usize = 16;
/// Lowest nonzero class, µmol m⁻² s⁻¹.
pub const PAR_MIN: f64 = 1.0;
/// Highest class, µmol m⁻² s⁻¹.
pub const PAR_MAX: f64 = 2000.0;
/// Log-spaced steps between the lowest and highest nonzero class.
const STEPS: f64 = 15.0;
/// Capacity nodes, highest first.
pub const CAPACITY_NODES: [f64; 4] = [1.0, 0.7, 0.4, 0.1];

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
    /// Leaf temperature the node was solved at (the air's in an uncoupled table).
    pub t_leaf: Celsius,
}

impl Rates {
    fn of(g: &GasExchange, t_leaf: Celsius) -> Self {
        Self {
            a_net: g.a_net,
            gs: g.gs,
            e: g.e,
            rd: g.rd,
            t_leaf,
        }
    }

    fn lerp(a: Self, b: Self, w: f64) -> Self {
        let l = |x: f64, y: f64| x + (y - x) * w;
        Self {
            a_net: UmolPerM2S(l(a.a_net.0, b.a_net.0)),
            gs: MolPerM2S(l(a.gs.0, b.gs.0)),
            e: MolPerM2S(l(a.e.0, b.e.0)),
            rd: UmolPerM2S(l(a.rd.0, b.rd.0)),
            t_leaf: Celsius(l(a.t_leaf.0, b.t_leaf.0)),
        }
    }
}

/// The hour's reference environment of a coupled table (decision 0034).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Reference {
    /// The hour's air.
    pub air: Air,
    /// Ambient CO2.
    pub ca: Ppm,
    /// Wind at the reference leaf, m s⁻¹.
    pub wind: f64,
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
        let mut nodes = [0.0; PAR_CLASSES + 1];
        for (i, n) in nodes.iter_mut().enumerate() {
            *n = par_node(i);
        }
        // Temperature and pressure are the same for every node.
        let tt = gas::TemperatureTerms::new(p, env);
        let mut rates = [[Rates::default(); PAR_CLASSES + 1]; 4];
        let mut full = [gas::solve_with(p, env, 1.0, stress, &tt); PAR_CLASSES + 1];
        for (c, cap) in CAPACITY_NODES.iter().enumerate() {
            for (k, slot) in rates[c].iter_mut().enumerate() {
                let e = LeafEnv {
                    par: UmolPerM2S(nodes[k]),
                    ..*env
                };
                let g = gas::solve_with(p, &e, *cap, stress, &tt);
                if c == 0 {
                    full[k] = g;
                }
                *slot = Rates::of(&g, env.t_leaf);
            }
        }
        Self { rates, full, nodes }
    }

    /// Solves the table with gas exchange and the leaf energy balance coupled at every
    /// node ([`energy::solve_coupled_from`]), each node starting from its lower-PAR
    /// neighbor's leaf temperature. In a `dark` hour every leaf reads the PAR-0 node, so
    /// only it is solved and the other nodes copy it (a lookup at PAR 0 gives the same
    /// bits either way). Returns the table and the gas-exchange solves used.
    #[must_use]
    pub fn build_coupled(
        p: &LeafParams,
        ep: &LeafEnergyParams,
        r: &Reference,
        stress: f64,
        dark: bool,
    ) -> (Self, u32) {
        let mut nodes = [0.0; PAR_CLASSES + 1];
        for (i, n) in nodes.iter_mut().enumerate() {
            *n = par_node(i);
        }
        let mut solves = 0;
        let at = r.air.terms();
        let mut node = |par: f64, cap: f64, start: Celsius| {
            let site = Site {
                par: UmolPerM2S(par),
                sky_view: 1.0,
                wind: r.wind,
            };
            let (g, h, n) = energy::solve_coupled_from(p, ep, &at, &site, r.ca, cap, stress, start);
            solves += n;
            (g, h.t_leaf)
        };
        let (g0, t0) = node(0.0, 1.0, r.air.t_air);
        let mut rates = [[Rates::of(&g0, t0); PAR_CLASSES + 1]; 4];
        let mut full = [g0; PAR_CLASSES + 1];
        for (c, cap) in CAPACITY_NODES.iter().enumerate() {
            for k in 0..=PAR_CLASSES {
                if dark && k > 0 {
                    rates[c][k] = rates[c][0];
                    if c == 0 {
                        full[k] = full[0];
                    }
                    continue;
                }
                // First guess from the nodes already solved: the step from the previous
                // PAR node, carried over from the previous capacity row (or the row's own
                // last step on the first row).
                let t = |c: usize, k: usize| rates[c][k].t_leaf.0;
                let start = match (c, k) {
                    (_, 0) => r.air.t_air.0,
                    (0, 1) => t(0, 0),
                    (0, _) => 2.0 * t(0, k - 1) - t(0, k - 2),
                    _ => t(c - 1, k) + t(c, k - 1) - t(c - 1, k - 1),
                };
                let (g, tl) = if c == 0 && k == 0 {
                    (g0, t0)
                } else {
                    node(nodes[k], *cap, Celsius(start))
                };
                if c == 0 {
                    full[k] = g;
                }
                rates[c][k] = Rates::of(&g, tl);
            }
        }
        (Self { rates, full, nodes }, solves)
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
