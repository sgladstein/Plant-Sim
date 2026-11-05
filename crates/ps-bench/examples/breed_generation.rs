//! WP30 done-when: wall time of one ten-seed bench generation (release build). Ten bush
//! bean lines (the species line and nine sampled founders) grow to maturity on the
//! bench; prints the wall time of each run, the days grown and the end hash as
//! Markdown for the CI job summary. Report only: shared runners are noisy, so this
//! never fails on a slow run.
//!
//! `cargo run --release -p ps-bench --example breed_generation [RUNS]` (default 3).

// mimalloc: the sim's parallel sections allocate per plant, and glibc malloc serializes
// them (docs/performance.md). The allocator never changes results.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::error::Error;
use std::path::Path;
use std::time::Instant;

use ps_breed::{BenchConditions, BenchPlan, Looks, SeedLibrary, load_species, run_generation};
use ps_world::Catalog;

fn main() -> Result<(), Box<dyn Error>> {
    let runs: usize = match std::env::args().nth(1) {
        Some(a) => a.parse()?,
        None => 3,
    };
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let kin = load_species(&data)?;
    let bush = kin
        .get("bush_bean")
        .ok_or("no bush_bean species file")?
        .clone();
    let looks = Looks::load(&data, &kin)?;
    let mut founders = SeedLibrary::new();
    let mut seeds = vec![founders.add_line("bush_bean", &bush, &looks)?];
    seeds.extend(founders.add_founders("bush_bean", &bush, 9, 3, &looks)?);
    let plan = BenchPlan {
        label: "bench".into(),
        seeds,
        crosses: Vec::new(),
        world_seed: 11,
        max_days: 120,
        conditions: BenchConditions::default(),
    };
    let catalog = Catalog::load(&data, &plan.scenario(&founders)?)?;
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    println!("Release build, {threads} threads. Budget: under 10 s per generation (WP30).\n");
    println!("| Run | Days grown | Seeds saved | Wall time, s | State hash |");
    println!("| --- | --- | --- | --- | --- |");
    for run in 1..=runs.max(1) {
        let mut lib = founders.clone();
        let t = Instant::now();
        let g = run_generation(&mut lib, &plan, &catalog, &kin, &looks)?;
        let s = t.elapsed().as_secs_f64();
        println!(
            "| {run} | {} | {} | {s:.2} | `{:016x}` |",
            g.days,
            g.saved.len(),
            g.hash
        );
    }
    Ok(())
}
