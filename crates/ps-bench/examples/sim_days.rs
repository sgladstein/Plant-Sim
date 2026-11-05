//! Prints wall time per sim-day for the committed scenarios as a Markdown table, for
//! the CI job summary (docs/performance.md). Shared runners are noisy, so this only
//! reports: it never compares against a budget and never fails on a slow run.
//!
//! `cargo run --release -p ps-bench --example sim_days [RUNS]` (default 3 runs; the
//! table shows the fastest and the median).

use std::error::Error;
use std::path::Path;
use std::time::Instant;

use ps_world::{Catalog, Scenario, World};

const SCENARIOS: [&str; 7] = [
    "single_pot",
    "two_pot_sun_shade",
    "plot_30_beans",
    "m2/pole_trellis",
    "m2/pole_row_trellis",
    "m2/wheat_spaced",
    "m2/wheat_stand",
];

fn main() -> Result<(), Box<dyn Error>> {
    let runs: usize = match std::env::args().nth(1) {
        Some(a) => a.parse()?,
        None => 3,
    };
    let runs = runs.max(1);
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    println!("Release build, {threads} threads, {runs} runs per scenario.\n");
    println!("| Scenario | Days | Fastest, ms per sim-day | Median, ms per sim-day | State hash |");
    println!("| --- | --- | --- | --- | --- |");
    for name in SCENARIOS {
        let text = std::fs::read_to_string(data.join(format!("scenarios/{name}.ron")))?;
        let scenario = Scenario::from_ron(&text)?;
        let catalog = Catalog::load(&data, &scenario)?;
        let days = scenario.duration_days;
        let mut ms = Vec::with_capacity(runs);
        let mut hash = 0;
        for _ in 0..runs {
            let mut world = World::new(scenario.clone(), catalog.clone())?;
            let t = Instant::now();
            world.run_to_end()?;
            ms.push(t.elapsed().as_secs_f64() * 1e3 / f64::from(days));
            hash = world.state_hash();
        }
        ms.sort_by(f64::total_cmp);
        println!(
            "| {name} | {days} | {:.2} | {:.2} | `{hash:016x}` |",
            ms[0],
            ms[ms.len() / 2]
        );
    }
    Ok(())
}
