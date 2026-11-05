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
    assert!(curve.is_complete(fed.thermal_age[fl.index()]));
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
