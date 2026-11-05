//! Allocation tests (WP7 done-when 2 and 3) and invariants.

use proptest::prelude::*;
use ps_graph::{
    Allometry, ExpansionCurve, NewOrgan, OrganArena, OrganId, OrganKind, Orientation, SpecificSize,
};
use ps_units::{Celsius, DegreeDays, GramsC, GramsDM, GramsN, Hours, M2PerG, MPerG};

use crate::allocation::{self, CarbonLedger, CarbonParams, HourInputs, Temperature};
use crate::thermal::{Cardinal, GrowthTemperature};

fn params() -> CarbonParams {
    let s = crate::tests::bean().0;
    CarbonParams::from_species(&s)
}

fn temp() -> Temperature {
    let s = crate::tests::bean().0;
    Temperature {
        cardinal: Cardinal::from_species(&s),
        growth: GrowthTemperature::from_species(&s),
    }
}

fn organ(kind: OrganKind, parent: Option<OrganId>, potential: f64, reserve: f64) -> NewOrgan {
    NewOrgan {
        kind,
        parent,
        axis_order: 0,
        birth: Hours(0),
        curve: if potential > 0.0 {
            ExpansionCurve::new(DegreeDays(100.0), DegreeDays(60.0), GramsDM(potential)).unwrap()
        } else {
            ExpansionCurve::none()
        },
        specific: match kind {
            OrganKind::Leaf | OrganKind::Cotyledon => SpecificSize::Area(M2PerG(0.025)),
            OrganKind::Internode | OrganKind::Root => SpecificSize::Length(MPerG(2.0)),
            _ => SpecificSize::Bulk,
        },
        orientation: Orientation::UP,
        structural: GramsDM(0.01),
        reserve: GramsC(reserve),
        nitrogen: GramsN(0.001),
    }
}

/// Seed (source) → internode → two leaves.
fn plant(seed_reserve: f64) -> (OrganArena, [OrganId; 4]) {
    let a = Allometry::generic_herb();
    let mut g = OrganArena::new();
    let s = g
        .add(&organ(OrganKind::Seed, None, 0.0, seed_reserve), &a)
        .unwrap();
    let i = g
        .add(&organ(OrganKind::Internode, Some(s), 0.1, 0.0), &a)
        .unwrap();
    let l1 = g
        .add(&organ(OrganKind::Leaf, Some(i), 0.2, 0.0), &a)
        .unwrap();
    let l2 = g
        .add(&organ(OrganKind::Leaf, Some(i), 0.2, 0.0), &a)
        .unwrap();
    (g, [s, i, l1, l2])
}

fn run_hour(
    g: &mut OrganArena,
    income: &[GramsC],
    ledger: &mut CarbonLedger,
    t: f64,
) -> allocation::HourReport {
    let temps = vec![Celsius(t); g.len()];
    allocation::hourly(
        g,
        &Allometry::generic_herb(),
        &params(),
        &temp(),
        &HourInputs {
            temperature: &temps,
            income,
            ftsw: 1.0,
            stress: 1.0,
            respiring: true,
            n_floor: None,
            secondary: &[],
        },
        ledger,
    )
    .unwrap()
}

#[test]
fn seed_reserve_feeds_growth_and_ledger_closes() {
    let p = params();
    let (mut g, [s, i, l1, _]) = plant(0.5);
    let mut ledger = CarbonLedger::start(&g, p.carbon_fraction);
    let income = vec![GramsC(0.0); g.len()];
    for _ in 0..24 * 20 {
        run_hour(&mut g, &income, &mut ledger, 25.0);
    }
    assert!(
        g.structural(l1).0 > 0.15,
        "leaf grew {}",
        g.structural(l1).0
    );
    assert!(g.structural(i).0 > 0.08);
    assert!(g.reserve[s.index()].0 < 0.5);
    assert!(ledger.relative_residual(&g, p.carbon_fraction).abs() < 1e-9);
    assert!(ledger.growth_respiration.0 > 0.0 && ledger.maintenance.0 > 0.0);
}

/// WP7 done-when: a starving organ dies on schedule.
#[test]
fn starving_leaf_dies_after_the_window() {
    let p = params();
    assert_eq!(p.starvation_hours, 72);
    let (mut g, [_, _, l1, l2]) = plant(0.0);
    // No maintenance anywhere, so only leaf 1 runs a deficit.
    for n in &mut g.nitrogen {
        *n = GramsN(0.0);
    }
    let mut ledger = CarbonLedger::start(&g, p.carbon_fraction);
    // Leaf 1 loses carbon every hour (respiration in the dark); nothing can feed it.
    let mut income = vec![GramsC(0.0); g.len()];
    income[l1.index()] = GramsC(-1e-4);
    let mut died_at = None;
    for h in 1..=24 * 6u64 {
        run_hour(&mut g, &income, &mut ledger, 20.0);
        if h % 24 == 0 {
            let deaths = allocation::starvation_deaths(&mut g, &p, Hours(h), &mut ledger).unwrap();
            if deaths.iter().any(|d| d.id == l1) {
                died_at = Some(h);
                break;
            }
            assert!(g.alive[l1.index()]);
        }
    }
    // Its reserve (0 at start) goes negative in hour 1, so the window of 72 hours
    // ends at hour 72, the third daily step.
    assert_eq!(died_at, Some(72));
    assert!(!g.alive[l1.index()] && g.alive[l2.index()]);
    assert!(ledger.litter.0 != 0.0);
    assert!(ledger.relative_residual(&g, p.carbon_fraction).abs() < 1e-9);
}

/// WP7 done-when: unmet demand shrinks final size, with no catch-up.
#[test]
fn unmet_demand_shrinks_final_size() {
    let p = params();
    let a = Allometry::generic_herb();
    let mut fed = OrganArena::new();
    let fl = fed
        .add(&organ(OrganKind::Leaf, None, 0.2, 10.0), &a)
        .unwrap();
    let mut hungry = OrganArena::new();
    let hl = hungry
        .add(&organ(OrganKind::Leaf, None, 0.2, 0.0), &a)
        .unwrap();
    let mut lf = CarbonLedger::start(&fed, p.carbon_fraction);
    let mut lh = CarbonLedger::start(&hungry, p.carbon_fraction);
    let none = vec![GramsC(0.0)];
    for h in 0..24 * 12 {
        run_hour(&mut fed, &none, &mut lf, 25.0);
        // The hungry leaf gets nothing for its first 3 days, then plenty.
        let feed = if h >= 72 {
            vec![GramsC(0.05)]
        } else {
            none.clone()
        };
        run_hour(&mut hungry, &feed, &mut lh, 25.0);
    }
    let curve = fed.curve[fl.index()];
    assert!(curve.is_complete(fed.expansion_age[fl.index()]));
    let full = fed.structural(fl).0 - 0.01;
    let short = hungry.structural(hl).0 - 0.01;
    assert!(
        (full - 0.2).abs() < 1e-3,
        "fed leaf reaches its potential: {full}"
    );
    assert!(short < 0.9 * full, "no catch-up: {short} vs {full}");
    let eff = hungry.effective_potential(hl).0;
    assert!(
        eff < 0.2 && (short - eff).abs() < 0.05 * eff,
        "{short} vs effective {eff}"
    );
}

proptest! {
    /// Transport conserves carbon and never drives a reserve negative by itself.
    #[test]
    fn transport_conserves_and_clamps(r in proptest::collection::vec(-0.01f64..0.5, 4)) {
        let p = params();
        let (mut g, ids) = plant(0.0);
        for (id, v) in ids.iter().zip(&r) {
            g.reserve[id.index()] = GramsC(*v);
        }
        // No maintenance, no growth: zero nitrogen and complete curves.
        for id in ids {
            g.nitrogen[id.index()] = GramsN(0.0);
            g.curve[id.index()] = ExpansionCurve::none();
        }
        let before: f64 = g.reserve.iter().map(|x| x.0).sum();
        let mut ledger = CarbonLedger::start(&g, p.carbon_fraction);
        run_hour(&mut g, &[GramsC(0.0); 4], &mut ledger, 20.0);
        let after: f64 = g.reserve.iter().map(|x| x.0).sum();
        prop_assert!((after - before).abs() < 1e-12);
        for (id, v) in ids.iter().zip(&r) {
            let now = g.reserve[id.index()].0;
            // An organ that gave carbon did not go below zero by giving.
            if now < *v {
                prop_assert!(now >= -1e-15, "organ {id:?}: {v} -> {now}");
            }
        }
    }
}

/// Decision 0011: cold slows the expansion clock, so a fed leaf still reaches its
/// potential, later; it is not cut short.
#[test]
fn cold_delays_expansion_without_loss() {
    let p = params();
    let a = Allometry::generic_herb();
    let grow = |t: f64| {
        let mut g = OrganArena::new();
        let l = g.add(&organ(OrganKind::Leaf, None, 0.2, 10.0), &a).unwrap();
        let mut ledger = CarbonLedger::start(&g, p.carbon_fraction);
        let none = vec![GramsC(0.0)];
        let mut hours = 0;
        while !g.curve[l.index()].is_complete(g.expansion_age[l.index()]) && hours < 24 * 200 {
            run_hour(&mut g, &none, &mut ledger, t);
            hours += 1;
        }
        (g.structural(l).0 - 0.01, hours)
    };
    let (warm, warm_h) = grow(25.0);
    let (cold, cold_h) = grow(15.0);
    assert!(
        (warm - 0.2).abs() < 1e-3 && (cold - 0.2).abs() < 1e-3,
        "{warm} {cold}"
    );
    assert!(cold_h > 2 * warm_h, "{cold_h} vs {warm_h} hours");
}

/// Decision 0011: a starved organ's negative reserve is charged to its living ancestor,
/// so litter is never negative.
#[test]
fn starved_debt_is_charged_to_the_ancestor() {
    let p = params();
    let (mut g, [_, i, l1, _]) = plant(0.0);
    g.reserve[l1.index()] = GramsC(-0.002);
    let mut ledger = CarbonLedger::start(&g, p.carbon_fraction);
    g.starving_hours[l1.index()] = p.starvation_hours;
    let before = g.reserve[i.index()].0;
    let deaths = allocation::starvation_deaths(&mut g, &p, Hours(24), &mut ledger).unwrap();
    assert_eq!(deaths.len(), 1);
    let d = deaths[0];
    let structural = 0.01 * p.carbon_fraction;
    assert!((d.litter.0 - structural * (1.0 - p.resorption_fraction)).abs() < 1e-15);
    assert!((d.resorbed.0 - (structural * p.resorption_fraction - 0.002)).abs() < 1e-15);
    assert!((g.reserve[i.index()].0 - before - d.resorbed.0).abs() < 1e-15);
    assert!(ledger.relative_residual(&g, p.carbon_fraction).abs() < 1e-9);
}

#[test]
fn inputs_must_cover_every_organ() {
    let (mut g, _) = plant(0.1);
    let mut ledger = CarbonLedger::start(&g, params().carbon_fraction);
    let temps = vec![Celsius(20.0); g.len() - 1];
    let income = vec![GramsC(0.0); g.len()];
    let r = allocation::hourly(
        &mut g,
        &Allometry::generic_herb(),
        &params(),
        &temp(),
        &HourInputs {
            temperature: &temps,
            income: &income,
            ftsw: 1.0,
            stress: 1.0,
            respiring: true,
            n_floor: None,
            secondary: &[],
        },
        &mut ledger,
    );
    assert!(matches!(
        r,
        Err(crate::PhysioError::InputLength {
            what: "temperature",
            got: 3,
            want: 4
        })
    ));
}

/// A random tree of stem segments and leaves with random masses and reserves.
fn random_tree(parents: &[usize], masses: &[f64], reserves: &[f64]) -> OrganArena {
    let a = Allometry::generic_herb();
    let mut g = OrganArena::new();
    let mut ids: Vec<OrganId> = Vec::new();
    for (i, (&m, &r)) in masses.iter().zip(reserves).enumerate() {
        let parent = (i > 0).then(|| ids[parents[i - 1] % i]);
        let kind = if i % 3 == 2 {
            OrganKind::Leaf
        } else {
            OrganKind::Internode
        };
        let id = g.add(&organ(kind, parent, 0.0, r), &a).unwrap();
        g.grow(id, GramsDM(m), &a).unwrap();
        ids.push(id);
    }
    g
}

proptest! {
    /// Decision 0011: the implicit step conserves carbon, keeps every concentration
    /// within the range of the old ones (monotone).
    #[test]
    fn implicit_transport_is_conservative_and_monotone(
        parents in proptest::collection::vec(0usize..1000, 15),
        masses in proptest::collection::vec(0.0f64..0.5, 16),
        reserves in proptest::collection::vec(0.0f64..0.05, 16),
    ) {
        let p = params();
        let mut g = random_tree(&parents, &masses, &reserves);
        let w: Vec<f64> = g.alive_ids().map(|id| g.structural(id).0.max(allocation::MIN_MASS)).collect();
        let c0: Vec<f64> = g.reserve.iter().zip(&w).map(|(r, w)| r.0 / w).collect();
        let before: f64 = g.reserve.iter().map(|x| x.0).sum();
        allocation::transport(&mut g, &p);
        let after: f64 = g.reserve.iter().map(|x| x.0).sum();
        prop_assert!((after - before).abs() <= 1e-12 * before.max(1e-12));
        let lo = c0.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = c0.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        for (r, w) in g.reserve.iter().zip(&w) {
            let c = r.0 / w;
            prop_assert!(c >= lo - 1e-9 * hi.max(1e-9) && c <= hi * (1.0 + 1e-9) + 1e-12, "{c} not in [{lo}, {hi}]");
        }
    }
}

/// The implicit step moves carbon beyond the first edge in one hour (an explicit pass
/// moves it one edge), with a profile that falls monotonically along the chain. How
/// far it reaches is set by the phloem conductance (a WP12 tuning parameter).
#[test]
fn implicit_transport_crosses_a_chain_in_one_hour() {
    let p = params();
    let n = 20;
    let parents: Vec<usize> = (0..n - 1).collect();
    let masses = vec![0.05; n];
    let mut reserves = vec![0.0; n];
    reserves[0] = 0.1;
    let mut g = OrganArena::new();
    let a = Allometry::generic_herb();
    let mut ids = Vec::new();
    for i in 0..n {
        let parent = (i > 0).then(|| ids[parents[i - 1]]);
        let id = g
            .add(&organ(OrganKind::Internode, parent, 0.0, reserves[i]), &a)
            .unwrap();
        g.grow(id, GramsDM(masses[i]), &a).unwrap();
        ids.push(id);
    }
    allocation::transport(&mut g, &p);
    let r: Vec<f64> = ids.iter().map(|id| g.reserve[id.index()].0).collect();
    assert!(r[4] > 1e-7, "four edges away: {}", r[4]);
    assert!(r.windows(2).all(|w| w[0] >= w[1] && w[1] >= 0.0), "{r:?}");
}

/// R1a: secondary growth takes at most `SECONDARY_PRIORITY` of the reserve above the
/// floor, widens a finished internode at fixed length, leaves the primary deficit
/// record alone and keeps the ledger closed.
#[test]
fn secondary_growth_is_a_low_priority_sink() {
    let p = params();
    let a = Allometry::generic_herb();
    let mut g = OrganArena::new();
    let s = g.add(&organ(OrganKind::Seed, None, 0.0, 0.0), &a).unwrap();
    // A finished internode (no primary potential) holding a reserve.
    let i = g
        .add(&organ(OrganKind::Internode, Some(s), 0.0, 0.004), &a)
        .unwrap();
    let mut ledger = CarbonLedger::start(&g, p.carbon_fraction);
    let temps = vec![Celsius(20.0); g.len()];
    let income = vec![GramsC(0.0); g.len()];
    let mut pot = vec![GramsDM(0.0); g.len()];
    pot[i.index()] = GramsDM(1.0);
    let len0 = g.size[i.index()].length;
    let r0 = g.size[i.index()].radius;
    let before = g.reserve[i.index()].0 + g.reserve[s.index()].0;
    let inp = HourInputs {
        temperature: &temps,
        income: &income,
        ftsw: 1.0,
        stress: 1.0,
        respiring: false,
        n_floor: None,
        secondary: &pot,
    };
    let rep = allocation::hourly(&mut g, &a, &p, &temp(), &inp, &mut ledger).unwrap();
    // A large potential is limited by the priority share of the reserve above the
    // floor after transport.
    assert!(rep.secondary.0 > 0.0);
    assert!(rep.secondary.0 <= allocation::SECONDARY_PRIORITY * before + 1e-15);
    assert_eq!(rep.unmet, GramsC(0.0));
    assert!(g.secondary(i).0 > 0.0);
    assert!((g.size[i.index()].length.0 - len0.0).abs() < 1e-15);
    assert!(g.size[i.index()].radius.0 > r0.0);
    // The primary radius is recomputed as r √(W_p / W): equal up to rounding.
    assert!((g.primary_radius(i).0 - r0.0).abs() <= 1e-12 * r0.0);
    assert_eq!(g.potential_demand[i.index()], GramsC(0.0));
    assert!(ledger.relative_residual(&g, p.carbon_fraction).abs() < 1e-12);
    // No reserve above the floor: nothing is built.
    let floor = p.reserve_floor * g.structural(i).0;
    g.reserve[i.index()] = GramsC(floor);
    g.reserve[s.index()] = GramsC(0.0);
    let w = g.structural(i);
    let mut ledger = CarbonLedger::start(&g, p.carbon_fraction);
    let rep = allocation::hourly(&mut g, &a, &p, &temp(), &inp, &mut ledger).unwrap();
    assert!(rep.secondary.0 < 1e-12, "{}", rep.secondary.0);
    assert!((g.structural(i).0 - w.0).abs() < 1e-12);
}
