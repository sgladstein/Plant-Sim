//! `ps-cli`: the headless runner (design plan §10.1, WP10).
//!
//! ```text
//! ps-cli run    SCENARIO... [--data DIR] [--out DIR] [--days N] [--snapshot-day D]
//! ps-cli hash   SCENARIO... [--data DIR] [--days N] [--write-golden DIR] [--check-golden DIR [--allow-missing]]
//! ps-cli bench  SCENARIO... [--data DIR] [--days N] [--repeat N]
//! ps-cli replay SCENARIO LOG [--data DIR] [--until-tick T] [--out DIR]
//! ps-cli replay --snapshot FILE [--out DIR]
//! ```
//!
//! `run` writes `daily.csv`, `organs.csv`, `hashes.csv` and `log.ron` for each scenario
//! into `OUT/<scenario file stem>/` (default `out/`), running the scenarios in parallel.
//! Golden hash files live at `DIR/<os>-<arch>/<stem>.csv`.

use std::path::{Path, PathBuf};
use std::time::Instant; // determinism: allow (bench timing only; never reaches the sim)

use anyhow::{Context, Result, bail};
use ps_world::{Action, Catalog, Scenario, World};

const USAGE: &str = "usage:
  ps-cli run    SCENARIO... [--data DIR] [--out DIR] [--days N] [--snapshot-day D]
  ps-cli hash   SCENARIO... [--data DIR] [--days N] [--write-golden DIR] [--check-golden DIR [--allow-missing]]
  ps-cli bench  SCENARIO... [--data DIR] [--days N] [--repeat N]
  ps-cli replay SCENARIO LOG [--data DIR] [--until-tick T] [--out DIR]
  ps-cli replay --snapshot FILE [--out DIR]";

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
        "" | "help" | "-h" | "--help" => {
            println!("{USAGE}");
            Ok(())
        }
        other => bail!("unknown command {other:?}\n{USAGE}"),
    }
}
