//! Whole-sim benches: the committed scenarios run end to end through `World`, plus the
//! daily state hash and an on-demand snapshot (docs/performance.md). These are the
//! numbers the M0 (2 ms per sim-day, one plant) and M1 (a sim-week per second, two pots)
//! budgets are about.

use std::path::{Path, PathBuf};

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use ps_world::{Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn load(name: &str) -> (Scenario, Catalog) {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).expect("scenario");
    let s = Scenario::from_ron(&text).expect("ron");
    let c = Catalog::load(&data(), &s).expect("catalog");
    (s, c)
}

fn world(c: &mut Criterion) {
    let mut g = c.benchmark_group("world");
    g.sample_size(10);
    for name in ["single_pot.ron", "two_pot_sun_shade.ron"] {
        let (s, cat) = load(name);
        let days = s.duration_days;
        g.bench_function(format!("{name}: {days} days"), |b| {
            b.iter_batched(
                || World::new(s.clone(), cat.clone()).expect("world"),
                |mut w| {
                    w.run_to_end().expect("run");
                    black_box(w.state_hash())
                },
                BatchSize::LargeInput,
            );
        });
    }
    let (s, cat) = load("single_pot.ron");
    let mut w = World::new(s, cat).expect("world");
    w.run_to_end().expect("run");
    g.bench_function("single_pot.ron: state hash at the end", |b| {
        b.iter(|| black_box(w.state_hash()));
    });
    g.bench_function("single_pot.ron: RON snapshot at the end", |b| {
        b.iter(|| black_box(w.snapshot_ron().expect("ron")));
    });
    g.finish();
}

criterion_group!(benches, world);
criterion_main!(benches);
