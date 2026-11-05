//! Light benches: the daily rebuild (grid fit plus every leaf's coefficients) for one
//! plant and for a 30-plant plot canopy (docs/performance.md).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ps_bench::{bush, plot_canopy, summer_sun_path};
use ps_light::{rebuild_daily, sky_sectors};

/// Spherical leaf angle distribution.
const G: f64 = 0.5;

fn light(c: &mut Criterion) {
    let sky = sky_sectors();
    let sun = summer_sun_path();
    for n in [60, 300] {
        let leaves = bush(n, 0.0, 0.0);
        c.bench_function(&format!("light/one plant {n} leaves/daily rebuild"), |b| {
            b.iter(|| black_box(rebuild_daily(&leaves, &[], &sun, &sky, G)));
        });
    }
    let plot = plot_canopy(100);
    c.bench_function("light/plot 30 x 100 leaves/daily rebuild", |b| {
        b.iter(|| black_box(rebuild_daily(&plot, &[], &sun, &sky, G)));
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = light
}
criterion_main!(benches);
