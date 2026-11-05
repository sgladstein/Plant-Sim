//! Physiology benches: one sim-day of class-table builds, leaf lookups and carbon
//! allocation for one plant (docs/performance.md).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ps_graph::{
    Allometry, ExpansionCurve, NewOrgan, OrganArena, OrganId, OrganKind, Orientation, SpecificSize,
};
use ps_physio::allocation::{self, CarbonLedger, CarbonParams, HourInputs, Temperature};
use ps_physio::{Cardinal, ClassTable, GrowthTemperature, LeafEnv, LeafParams};
use ps_species::Species;
use ps_units::{
    Celsius, DegreeDays, GramsC, GramsDM, GramsN, Hours, KPa, M2PerG, MPerG, Ppm, UmolPerM2S,
};

fn organ(kind: OrganKind, parent: Option<OrganId>, potential: f64) -> NewOrgan {
    NewOrgan {
        kind,
        parent,
        axis_order: 0,
        birth: Hours(0),
        curve: ExpansionCurve::new(DegreeDays(100.0), DegreeDays(60.0), GramsDM(potential))
            .expect("curve"),
        specific: match kind {
            OrganKind::Leaf => SpecificSize::Area(M2PerG(0.025)),
            OrganKind::Internode | OrganKind::Root => SpecificSize::Length(MPerG(2.0)),
            _ => SpecificSize::Bulk,
        },
        orientation: Orientation::UP,
        structural: GramsDM(0.05),
        reserve: GramsC(0.01),
        nitrogen: GramsN(0.001),
    }
}

/// Seed, a 30-internode stem with two leaves per node, and a 100-segment root chain
/// with laterals: 191 organs.
fn plant() -> OrganArena {
    let a = Allometry::generic_herb();
    let mut g = OrganArena::new();
    let seed = g.add(&organ(OrganKind::Seed, None, 0.0), &a).expect("add");
    let mut node = seed;
    for _ in 0..30 {
        node = g
            .add(&organ(OrganKind::Internode, Some(node), 0.1), &a)
            .expect("add");
        for _ in 0..2 {
            g.add(&organ(OrganKind::Leaf, Some(node), 0.2), &a)
                .expect("add");
        }
    }
    let mut root = seed;
    for k in 0..100 {
        let parent = if k % 4 == 0 { seed } else { root };
        root = g
            .add(&organ(OrganKind::Root, Some(parent), 0.02), &a)
            .expect("add");
    }
    g
}

fn physio(c: &mut Criterion) {
    let s = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).expect("ron");
    let t = s.phenotype(&s.default_genotype()).traits().expect("traits");
    let p = LeafParams::from_species(&s, &t);
    let env = LeafEnv {
        par: UmolPerM2S(0.0),
        t_leaf: Celsius(25.0),
        vpd: KPa(1.5),
        ca: Ppm(420.0),
        patm: KPa(100.0),
    };
    c.bench_function("physio/class table build, 24 h", |b| {
        b.iter(|| {
            for _ in 0..24 {
                black_box(ClassTable::build(&p, &env, 1.0));
            }
        });
    });
    let table = ClassTable::build(&p, &env, 1.0);
    c.bench_function("physio/lookups, 300 leaves x 2 parts x 24 h", |b| {
        b.iter(|| {
            for h in 0..24u32 {
                for l in 0..600u32 {
                    let par = UmolPerM2S(f64::from(l * 3 + h));
                    black_box(table.lookup(par, 0.4 + f64::from(l % 7) * 0.1));
                }
            }
        });
    });
    let cp = CarbonParams::from_species(&s);
    let temp = Temperature {
        cardinal: Cardinal::from_species(&s),
        growth: GrowthTemperature::from_species(&s),
    };
    let allometry = Allometry::generic_herb();
    let base = plant();
    let temps = vec![Celsius(22.0); base.len()];
    let income: Vec<GramsC> = (0..base.len())
        .map(|i| {
            if i % 3 == 0 {
                GramsC(1e-4)
            } else {
                GramsC(0.0)
            }
        })
        .collect();
    c.bench_function("physio/allocation, 191 organs x 24 h", |b| {
        b.iter_batched(
            || (base.clone(), CarbonLedger::start(&base, cp.carbon_fraction)),
            |(mut g, mut ledger)| {
                for _ in 0..24 {
                    let inp = HourInputs {
                        temperature: &temps,
                        income: &income,
                        ftsw: 1.0,
                        stress: 1.0,
                        respiring: true,
                        n_floor: None,
                    };
                    black_box(
                        allocation::hourly(&mut g, &allometry, &cp, &temp, &inp, &mut ledger)
                            .expect("hour"),
                    );
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = physio
}
criterion_main!(benches);
