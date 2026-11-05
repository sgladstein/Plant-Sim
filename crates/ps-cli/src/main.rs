//! `ps-cli`: the headless runner (design plan §10.1, WP10).
//!
//! ```text
//! ps-cli run    SCENARIO... [--data DIR] [--out DIR] [--days N] [--snapshot-day D]
//! ps-cli hash   SCENARIO... [--data DIR] [--days N] [--write-golden DIR] [--check-golden DIR [--allow-missing]]
//! ps-cli bench  SCENARIO... [--data DIR] [--days N] [--repeat N]
//! ps-cli replay SCENARIO LOG [--data DIR] [--until-tick T] [--out DIR]
//! ps-cli replay --snapshot FILE [--out DIR]
//! ps-cli validate [--data DIR] [--validation DIR] [--known FILE] [--out DIR]
//! ```
//!
//! `run` writes `daily.csv`, `organs.csv`, `hashes.csv` and `log.ron` for each scenario
//! into `OUT/<scenario file stem>/` (default `out/`), running the scenarios in parallel.
//! Golden hash files live at `DIR/<os>-<arch>/<stem>.csv`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant; // determinism: allow (bench timing only; never reaches the sim)

use anyhow::{Context, Result, bail};
use ps_world::{Action, Catalog, Scenario, World};

const USAGE: &str = "usage:
  ps-cli run    SCENARIO... [--data DIR] [--out DIR] [--days N] [--snapshot-day D]
  ps-cli hash   SCENARIO... [--data DIR] [--days N] [--write-golden DIR] [--check-golden DIR [--allow-missing]]
  ps-cli bench  SCENARIO... [--data DIR] [--days N] [--repeat N]
  ps-cli replay SCENARIO LOG [--data DIR] [--until-tick T] [--out DIR]
  ps-cli replay --snapshot FILE [--out DIR]
  ps-cli validate [--data DIR] [--validation DIR] [--known FILE] [--out DIR]";

/// Options that take no value.
const FLAGS: [&str; 1] = ["allow-missing"];

/// Parsed command line: positional arguments and `--key value` options.
#[derive(Debug, Default)]
struct Args {
    command: String,
    files: Vec<PathBuf>,
    options: Vec<(String, String)>,
}

impl Args {
    fn parse(raw: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut it = raw.into_iter();
        let mut a = Args {
            command: it.next().unwrap_or_default(),
            ..Args::default()
        };
        while let Some(x) = it.next() {
            if let Some(key) = x.strip_prefix("--") {
                if FLAGS.contains(&key) {
                    a.options.push((key.to_string(), "true".into()));
                    continue;
                }
                let v = it
                    .next()
                    .with_context(|| format!("--{key} needs a value\n{USAGE}"))?;
                a.options.push((key.to_string(), v));
            } else {
                a.files.push(PathBuf::from(x));
            }
        }
        Ok(a)
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.options
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    fn number<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>> {
        self.get(key)
            .map(|v| {
                v.parse::<T>()
                    .map_err(|_| anyhow::anyhow!("--{key}: {v:?} is not a number"))
            })
            .transpose()
    }

    fn check_keys(&self, allowed: &[&str]) -> Result<()> {
        for (k, _) in &self.options {
            if !allowed.contains(&k.as_str()) {
                bail!("unknown option --{k} for {}\n{USAGE}", self.command);
            }
        }
        Ok(())
    }

    fn data_dir(&self) -> PathBuf {
        PathBuf::from(self.get("data").unwrap_or("data"))
    }

    fn out_dir(&self) -> PathBuf {
        PathBuf::from(self.get("out").unwrap_or("out"))
    }
}

/// A scenario file loaded with its catalog.
struct Loaded {
    stem: String,
    scenario: Scenario,
    catalog: Catalog,
}

fn load(path: &Path, data: &Path, days: Option<u32>) -> Result<Loaded> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut scenario =
        Scenario::from_ron(&text).with_context(|| format!("parsing {}", path.display()))?;
    if let Some(d) = days {
        scenario.duration_days = d;
    }
    let catalog = Catalog::load(data, &scenario)
        .with_context(|| format!("loading the catalog for {}", path.display()))?;
    let stem = path
        .file_stem()
        .map_or_else(|| "scenario".into(), |s| s.to_string_lossy().into_owned());
    Ok(Loaded {
        stem,
        scenario,
        catalog,
    })
}

fn load_all(a: &Args) -> Result<Vec<Loaded>> {
    if a.files.is_empty() {
        bail!("no scenario given\n{USAGE}");
    }
    let days = a.number::<u32>("days")?;
    let data = a.data_dir();
    a.files.iter().map(|f| load(f, &data, days)).collect()
}

/// Runs each scenario on its own thread; results come back in input order.
fn run_parallel<T: Send>(
    items: &[Loaded],
    f: impl Fn(&Loaded) -> Result<T> + Sync,
) -> Vec<Result<T>> {
    std::thread::scope(|s| {
        let handles: Vec<_> = items.iter().map(|l| s.spawn(|| f(l))).collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("a run panicked")))
            })
            .collect()
    })
}

/// [`run_parallel`] over `(item, flag)` pairs, on at most as many threads as the machine
/// has, so a validation's few dozen runs do not all start at once.
fn run_parallel_pairs<T: Send>(
    items: &[(Loaded, bool)],
    f: impl Fn(&Loaded, bool) -> Result<T> + Sync,
) -> Vec<Result<T>> {
    let threads = std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut out: Vec<Option<Result<T>>> = (0..items.len()).map(|_| None).collect();
    let done = std::sync::Mutex::new(&mut out);
    std::thread::scope(|s| {
        for _ in 0..threads.min(items.len()) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some((l, flag)) = items.get(i) else {
                        break;
                    };
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(l, *flag)))
                        .unwrap_or_else(|_| Err(anyhow::anyhow!("a run panicked")));
                    if let Ok(mut d) = done.lock() {
                        d[i] = Some(r);
                    }
                }
            });
        }
    });
    out.into_iter()
        .map(|r| r.unwrap_or_else(|| Err(anyhow::anyhow!("a run did not finish"))))
        .collect()
}

fn write_outputs(dir: &Path, w: &World) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let st = w.state();
    let put = |name: &str, text: String| -> Result<()> {
        let p = dir.join(name);
        std::fs::write(&p, text).with_context(|| format!("writing {}", p.display()))
    };
    put("daily.csv", ps_export::daily_csv(st))?;
    put("organs.csv", ps_export::organs_csv(st))?;
    put("hashes.csv", ps_export::hashes_csv(st))?;
    let log = ron::ser::to_string_pretty(&st.log[..st.applied], ron::ser::PrettyConfig::default())
        .context("writing the action log")?;
    put("log.ron", log)?;
    Ok(())
}

fn cmd_run(a: &Args) -> Result<()> {
    a.check_keys(&["data", "out", "days", "snapshot-day"])?;
    let loaded = load_all(a)?;
    let out = a.out_dir();
    let snapshot_day = a.number::<u32>("snapshot-day")?;
    let results = run_parallel(&loaded, |l| {
        let mut w = World::new(l.scenario.clone(), l.catalog.clone())?;
        let dir = out.join(&l.stem);
        if let Some(d) = snapshot_day {
            w.run_days(d.min(l.scenario.duration_days))?;
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join(format!("snapshot_day{d}.ron")), w.snapshot_ron()?)?;
        }
        w.run_to_end()?;
        write_outputs(&dir, &w)?;
        Ok(w.state().hashes.last().copied().unwrap_or_default())
    });
    let mut failed = false;
    for (l, r) in loaded.iter().zip(results) {
        match r {
            Ok(h) => println!(
                "{}: {} days, last daily hash {h:016x} -> {}",
                l.stem,
                l.scenario.duration_days,
                out.join(&l.stem).display()
            ),
            Err(e) => {
                failed = true;
                eprintln!("{}: {e:#}", l.stem);
            }
        }
    }
    if failed {
        bail!("some scenarios failed");
    }
    Ok(())
}

fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn cmd_hash(a: &Args) -> Result<()> {
    a.check_keys(&[
        "data",
        "days",
        "write-golden",
        "check-golden",
        "allow-missing",
    ])?;
    let loaded = load_all(a)?;
    let results = run_parallel(&loaded, |l| {
        let mut w = World::new(l.scenario.clone(), l.catalog.clone())?;
        w.run_to_end()?;
        Ok(ps_export::hashes_csv(w.state()))
    });
    let mut bad = Vec::new();
    for (l, r) in loaded.iter().zip(results) {
        let csv = r.with_context(|| l.stem.clone())?;
        let last = csv.lines().last().unwrap_or_default();
        println!("{}: {last}", l.stem);
        if let Some(dir) = a.get("write-golden") {
            let d = Path::new(dir).join(platform());
            std::fs::create_dir_all(&d)?;
            std::fs::write(d.join(format!("{}.csv", l.stem)), &csv)?;
        }
        if let Some(dir) = a.get("check-golden") {
            let p = Path::new(dir)
                .join(platform())
                .join(format!("{}.csv", l.stem));
            // Compare lines, so a checkout that turned LF into CRLF still matches.
            let same = |g: &str| g.lines().map(str::trim_end).eq(csv.lines());
            match std::fs::read_to_string(&p) {
                Ok(golden) if same(&golden) => println!("{}: matches {}", l.stem, p.display()),
                Ok(golden) => {
                    let day = golden
                        .lines()
                        .zip(csv.lines())
                        .position(|(g, c)| g.trim_end() != c)
                        .map_or_else(|| "length".into(), |i| format!("line {}", i + 1));
                    eprintln!("{}: differs from {} at {day}", l.stem, p.display());
                    bad.push(l.stem.clone());
                }
                Err(_) => {
                    println!(
                        "{}: no golden file at {}; this platform's hashes follow\n{csv}",
                        l.stem,
                        p.display()
                    );
                    if a.get("allow-missing").is_none() {
                        bad.push(format!("{} (no golden file)", l.stem));
                    }
                }
            }
        }
    }
    if !bad.is_empty() {
        bail!("golden hash mismatch: {}", bad.join(", "));
    }
    Ok(())
}

fn cmd_bench(a: &Args) -> Result<()> {
    a.check_keys(&["data", "days", "repeat"])?;
    let loaded = load_all(a)?;
    let repeat = a.number::<u32>("repeat")?.unwrap_or(3).max(1);
    if cfg!(debug_assertions) {
        eprintln!("note: a debug build; run `cargo run --release -p ps-cli -- bench ...`");
    }
    // One at a time, so runs do not compete for cores.
    for l in &loaded {
        let mut best = f64::MAX;
        for _ in 0..repeat {
            let mut w = World::new(l.scenario.clone(), l.catalog.clone())?;
            let t = Instant::now(); // determinism: allow (bench timing only)
            w.run_to_end()?;
            best = best.min(t.elapsed().as_secs_f64());
        }
        let days = f64::from(l.scenario.duration_days.max(1));
        println!(
            "{}: {} days in {:.1} ms (best of {repeat}), {:.3} ms per sim-day",
            l.stem,
            l.scenario.duration_days,
            best * 1e3,
            best * 1e3 / days
        );
    }
    Ok(())
}

fn cmd_replay(a: &Args) -> Result<()> {
    a.check_keys(&["data", "until-tick", "out", "snapshot"])?;
    let w = if let Some(snap) = a.get("snapshot") {
        let text = std::fs::read_to_string(snap).with_context(|| format!("reading {snap}"))?;
        let mut w = World::from_snapshot_ron(&text).context("reading the snapshot")?;
        w.run_to_end()?;
        w
    } else {
        let [scenario, log] = a.files.as_slice() else {
            bail!("replay needs a scenario and a log file\n{USAGE}");
        };
        let l = load(scenario, &a.data_dir(), None)?;
        let text =
            std::fs::read_to_string(log).with_context(|| format!("reading {}", log.display()))?;
        let actions: Vec<Action> =
            ron::from_str(&text).with_context(|| format!("parsing {}", log.display()))?;
        let end = u64::from(l.scenario.duration_days) * 24;
        let until = a.number::<u64>("until-tick")?.unwrap_or(end);
        World::replay(l.scenario, l.catalog, &actions, until)?
    };
    println!("tick {}: full-state hash {:016x}", w.tick(), w.state_hash());
    if let Some(last) = w.state().hashes.last() {
        println!("day {}: daily hash {last:016x}", w.state().hashes.len() - 1);
    }
    if a.get("out").is_some() {
        write_outputs(&a.out_dir(), &w)?;
    }
    Ok(())
}

/// Scenarios every validation runs, for the report's plots, besides those the data names.
const VALIDATION_SCENARIOS: [&str; 5] = [
    "single_pot",
    "response/control",
    "response/drought_from_flowering",
    "response/heat_at_flowering",
    "response/shade_50",
];

/// Day of the organ snapshot the report draws in 3D.
const SNAPSHOT_DAY: u32 = 45;

fn scenario_path(data: &Path, name: &str) -> Result<PathBuf> {
    let p = data.join("scenarios").join(format!("{name}.ron"));
    if p.exists() {
        return Ok(p);
    }
    bail!(
        "no scenario file for {name:?} under {}",
        data.join("scenarios").display()
    )
}

/// Prints the build-failing misses and the fixed known misses; returns the miss count.
fn print_misses(outcomes: &[ps_validate::Outcome], known_path: &Path) -> usize {
    let failing: Vec<&ps_validate::Outcome> = outcomes.iter().filter(|o| o.fails_build()).collect();
    for o in &failing {
        eprintln!(
            "MISS {} ({}, {} {}): expected {}, sim {}",
            o.id,
            o.scenario,
            o.variable,
            o.at,
            o.expected,
            o.got.map_or_else(
                || "-".into(),
                |v| if o.kind == "response" {
                    format!("{:+.0}%", v * 100.0)
                } else {
                    ps_validate::fmt_num(v)
                }
            )
        );
    }
    for o in outcomes
        .iter()
        .filter(|o| o.verdict == ps_validate::Verdict::Fixed)
    {
        eprintln!("FIXED {}: remove it from {}", o.id, known_path.display());
    }
    failing.len()
}

/// Runs every scenario once per seed offset (review of WP25). The scenario as written
/// (offset 0, listed first) also writes its outputs under `runs_dir` for the plots.
fn validation_runs(
    loaded: &[Loaded],
    offsets: &[u64],
    runs_dir: &Path,
) -> Result<ps_validate::Runs> {
    let jobs: Vec<(Loaded, bool)> = loaded
        .iter()
        .flat_map(|l| {
            offsets.iter().enumerate().map(move |(k, &off)| {
                let mut l = Loaded {
                    stem: l.stem.clone(),
                    scenario: l.scenario.clone(),
                    catalog: l.catalog.clone(),
                };
                l.scenario.world_seed = l.scenario.world_seed.wrapping_add(off);
                (l, k == 0)
            })
        })
        .collect();
    let results = run_parallel_pairs(&jobs, |l, primary| {
        let mut w = World::new(l.scenario.clone(), l.catalog.clone())?;
        if !primary {
            w.run_to_end()?;
            return Ok(w.state().clone());
        }
        let dir = runs_dir.join(&l.stem);
        std::fs::create_dir_all(&dir)?;
        if SNAPSHOT_DAY < l.scenario.duration_days {
            w.run_days(SNAPSHOT_DAY)?;
            std::fs::write(
                dir.join(format!("organs_day{SNAPSHOT_DAY}.csv")),
                ps_export::organs_csv(w.state()),
            )?;
        }
        w.run_to_end()?;
        write_outputs(&dir, &w)?;
        Ok(w.state().clone())
    });
    let mut runs = ps_validate::Runs::new();
    for ((l, _), r) in jobs.iter().zip(results) {
        let seed = l.scenario.world_seed;
        let state = r.with_context(|| format!("{} (world seed {seed})", l.stem))?;
        runs.entry(l.stem.clone()).or_default().push(state);
    }
    Ok(runs)
}

fn cmd_validate(a: &Args) -> Result<()> {
    a.check_keys(&["data", "validation", "known", "out"])?;
    let data = a.data_dir();
    let vdir = a
        .get("validation")
        .map_or_else(|| data.join("validation"), PathBuf::from);
    let known_path = PathBuf::from(a.get("known").unwrap_or("validation/known_misses.ron"));
    let out = PathBuf::from(a.get("out").unwrap_or("out/validation"));
    let vdata = ps_validate::Data::load(&vdir)?;
    let known = ps_validate::known_misses(&known_path)?;
    let mut names = vdata.scenarios();
    names.extend(VALIDATION_SCENARIOS.iter().map(|s| (*s).to_string()));
    names.sort();
    names.dedup();
    let loaded: Vec<Loaded> = names
        .iter()
        .map(|n| {
            let mut l = load(&scenario_path(&data, n)?, &data, None)?;
            l.stem.clone_from(n);
            Ok(l)
        })
        .collect::<Result<_>>()?;
    let runs = validation_runs(&loaded, &vdata.seed_offsets(), &out.join("runs"))?;
    let mut outcomes = Vec::new();
    if let Some(t) = &vdata.targets {
        outcomes.extend(ps_validate::evaluate_targets(t, &runs, &known));
    }
    if let Some(r) = &vdata.responses {
        outcomes.extend(ps_validate::evaluate_responses(r, &runs, &known));
    }
    if let Some(pf) = &vdata.parameters {
        let path = data.join("species").join(format!("{}.ron", pf.species));
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        outcomes.extend(ps_validate::evaluate_parameters(pf, &text, &known));
    }
    let curves = ps_validate::compare_curves(&vdata.curves, &runs);
    std::fs::create_dir_all(out.join("curves"))?;
    for c in &curves {
        let mut csv = String::from("x,y,sd,sim\n");
        for (p, s) in c.curve.points.iter().zip(&c.sim) {
            let opt = |v: Option<f64>| v.map_or_else(String::new, |v| v.to_string());
            let _ = writeln!(csv, "{},{},{},{}", p.x, p.y, opt(p.sd), opt(*s));
        }
        std::fs::write(out.join("curves").join(format!("{}.csv", c.curve.id)), csv)?;
    }
    std::fs::write(
        out.join("results.csv"),
        ps_validate::outcomes_csv(&outcomes),
    )?;
    let report = ps_validate::report_markdown(&outcomes, &curves);
    std::fs::write(out.join("report.md"), &report)?;
    println!("{}", report.lines().nth(2).unwrap_or_default());
    let failing = print_misses(&outcomes, &known_path);
    let stale = ps_validate::stale_known_misses(&known, &outcomes);
    for s in &stale {
        eprintln!("STALE {}: {s}", known_path.display());
    }
    println!("report: {}", out.join("report.md").display());
    if failing > 0 {
        bail!("{failing} M0 validation checks missed or not evaluated");
    }
    if !stale.is_empty() {
        bail!("{} stale entries in {}", stale.len(), known_path.display());
    }
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();
    let a = Args::parse(std::env::args().skip(1))?;
    match a.command.as_str() {
        "run" => cmd_run(&a),
        "hash" => cmd_hash(&a),
        "bench" => cmd_bench(&a),
        "replay" => cmd_replay(&a),
        "validate" => cmd_validate(&a),
        "" | "help" | "-h" | "--help" => {
            println!("{USAGE}");
            Ok(())
        }
        other => bail!("unknown command {other:?}\n{USAGE}"),
    }
}
