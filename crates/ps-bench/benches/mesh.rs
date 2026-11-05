//! Per-frame CPU work of the viewer for one 90-day demo bean (docs/performance.md):
//! cloning the hourly snapshot, blending two skeletons, building leaf instances, and a
//! worst-case rebuild of every axis mesh. At 60 fps the frame budget is 16.7 ms for
//! everything, two plants in M1 and 30 in M2.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ps_mesh::demo::{DemoParams, demo_bean, demo_origin};
use ps_mesh::{AxisKind, AxisStyle, InstanceOptions, Skeleton, axes, axis_mesh, instances};

const DAY_90: u64 = 90 * 24;

fn mesh(c: &mut Criterion) {
    let arena = demo_bean(DemoParams::SUN, DAY_90);
    let next = demo_bean(DemoParams::SUN, DAY_90 + 1);
    let a = Skeleton::from_arena(&arena, demo_origin(0.0, 0.0));
    let b = Skeleton::from_arena(&next, demo_origin(0.0, 0.0));
    println!("demo bean day 90: {} organs", arena.len());

    c.bench_function("mesh/demo_bean (one sim hour of the demo source)", |bn| {
        bn.iter(|| black_box(demo_bean(DemoParams::SUN, DAY_90)));
    });
    c.bench_function("mesh/clone arena + skeleton", |bn| {
        bn.iter(|| black_box((arena.clone(), a.clone())));
    });
    c.bench_function("mesh/skeleton from arena", |bn| {
        bn.iter(|| black_box(Skeleton::from_arena(&arena, demo_origin(0.0, 0.0))));
    });
    c.bench_function("mesh/skeleton lerp", |bn| {
        bn.iter(|| black_box(Skeleton::lerp(&a, &b, 0.5)));
    });
    c.bench_function("mesh/instances", |bn| {
        bn.iter(|| black_box(instances(&a, InstanceOptions::default())));
    });
    c.bench_function("mesh/all axis meshes (full rebuild)", |bn| {
        bn.iter(|| {
            for axis in axes(&a, true) {
                let style = if axis.kind == AxisKind::Root {
                    AxisStyle::ROOT
                } else {
                    AxisStyle::STEM
                };
                black_box(axis_mesh(&a, &axis, style));
            }
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = mesh
}
criterion_main!(benches);
