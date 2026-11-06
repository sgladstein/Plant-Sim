//! Soil benches: one sim-day (24 hourly steps) at pot and plot scale (docs/performance.md).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ps_bench::{PLOT_PLANTS, plant_roots, plot, pot, summer_hour};
use ps_soil::PlantFlows;
use ps_units::KgWater;

/// Each plant's hourly flows: 1 g of water split over its root cells by root length, as
/// a network in evenly wet soil splits it (R3b, decision 0044).
fn flows(roots: &[ps_soil::RootCells]) -> Vec<Vec<(usize, KgWater)>> {
    roots
        .iter()
        .map(|r| {
            let total: f64 = r.cells().iter().map(|c| c.1.0).sum();
            r.cells()
                .iter()
                .map(|&(i, l)| (i, KgWater(1e-3 * l.0 / total)))
                .collect()
        })
        .collect()
}

fn requests(flows: &[Vec<(usize, KgWater)>]) -> Vec<PlantFlows<'_>> {
    flows
        .iter()
        .map(|f| PlantFlows {
            demand: KgWater(1e-3),
            flows: f,
        })
        .collect()
}

fn soil(c: &mut Criterion) {
    let hour = summer_hour();

    let mut g = pot();
    c.bench_function("soil/pot/hydrology day", |b| {
        b.iter(|| {
            for _ in 0..24 {
                black_box(g.step_hour(&hour));
            }
        });
    });
    let roots = plant_roots(&g, 1);
    let per_plant = flows(&roots);
    let req = requests(&per_plant);
    c.bench_function("soil/pot/uptake+ftsw day, 1 plant", |b| {
        b.iter(|| {
            for _ in 0..24 {
                black_box(g.ftsw(&roots[0]).expect("cells"));
                black_box(g.take_flows(&req).expect("cells"));
            }
        });
    });

    let mut g = plot();
    c.bench_function("soil/plot/hydrology day", |b| {
        b.iter(|| {
            for _ in 0..24 {
                black_box(g.step_hour(&hour));
            }
        });
    });
    let roots = plant_roots(&g, PLOT_PLANTS);
    let per_plant = flows(&roots);
    let req = requests(&per_plant);
    c.bench_function("soil/plot/uptake+ftsw day, 30 plants", |b| {
        b.iter(|| {
            for _ in 0..24 {
                for r in &roots {
                    black_box(g.ftsw(r).expect("cells"));
                }
                black_box(g.take_flows(&req).expect("cells"));
            }
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = soil
}
criterion_main!(benches);
