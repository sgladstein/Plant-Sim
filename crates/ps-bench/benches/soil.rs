//! Soil benches: one sim-day (24 hourly steps) at pot and plot scale (docs/performance.md).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ps_bench::{PLOT_PLANTS, dense_roots, plot, pot, summer_hour};
use ps_units::KgWater;

/// Species uptake coefficient used by the benches, m² h⁻¹.
const RATE: f64 = 1e-4;

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
    let roots = dense_roots(&g, 1);
    c.bench_function("soil/pot/uptake+ftsw day, 1 plant", |b| {
        b.iter(|| {
            for _ in 0..24 {
                black_box(g.ftsw(&roots[0]).expect("len"));
                black_box(g.take_up(KgWater(1e-3), &roots[0], RATE).expect("len"));
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
    let roots = dense_roots(&g, PLOT_PLANTS);
    c.bench_function("soil/plot/uptake+ftsw day, 30 plants", |b| {
        b.iter(|| {
            for _ in 0..24 {
                for r in &roots {
                    black_box(g.ftsw(r).expect("len"));
                    black_box(g.take_up(KgWater(1e-3), r, RATE).expect("len"));
                }
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
