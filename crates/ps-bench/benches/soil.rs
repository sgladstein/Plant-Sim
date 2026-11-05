//! Soil benches: one sim-day (24 hourly steps) at pot and plot scale (docs/performance.md).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ps_bench::{PLOT_PLANTS, plant_roots, plot, pot, summer_hour};
use ps_soil::PlantUptake;
use ps_units::{KgPerMH, KgWater, M2PerHour};

/// Species uptake coefficient used by the benches.
const RATE: M2PerHour = M2PerHour(1e-4);
/// Uptake ceiling per root length used by the benches.
const MAX: KgPerMH = KgPerMH(1.25e-4);

fn requests(roots: &[ps_soil::RootCells]) -> Vec<PlantUptake<'_>> {
    roots
        .iter()
        .map(|r| PlantUptake {
            demand: KgWater(1e-3),
            roots: r,
            rate: RATE,
            max_per_length: MAX,
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
    let req = requests(&roots);
    c.bench_function("soil/pot/uptake+ftsw day, 1 plant", |b| {
        b.iter(|| {
            for _ in 0..24 {
                black_box(g.ftsw(&roots[0]).expect("cells"));
                black_box(g.take_up(&req).expect("cells"));
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
    let req = requests(&roots);
    c.bench_function("soil/plot/uptake+ftsw day, 30 plants", |b| {
        b.iter(|| {
            for _ in 0..24 {
                for r in &roots {
                    black_box(g.ftsw(r).expect("cells"));
                }
                black_box(g.take_up(&req).expect("cells"));
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
