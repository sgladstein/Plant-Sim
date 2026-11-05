//! The validation harness (design plan §8, WP11): reads the reference targets under
//! `data/validation/`, evaluates them on world runs, and reports each as a pass, a
//! miss, a known miss or not evaluated.
//!
//! Files (schema in docs/validation-data.md):
//! - `targets.ron`: whole-plant ranges and values (§8.2).
//! - `responses.ron`: treatment responses against a baseline scenario (§8.3).
//! - `curves/*.ron`: digitized time courses, compared and plotted (information only).
//! - `known_misses.ron` (under `validation/`): target ids expected to miss until a named
//!   work package, so the build fails only on new misses.
//!
//! Variables are the `daily.csv` columns (ps-export) or the derived names in
//! [`derived`]. Each scenario runs once per seed offset in `targets.ron`. A value is the
//! mean over the runs of each run's mean over its plants, its replicates; the report
//! gives the range over the runs too. A response compares the treatment's mean with the
//! baseline's. The logistic shape check reads the first plant of each run; the curves
//! read the mean of the runs' first plants.

pub mod derived;
pub mod files;
pub mod logistic;

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ps_world::{DayRow, WorldState};

pub use files::{
    At, Axis, Curve, Data, Expect, Gate, KnownMiss, KnownMisses, Response, ResponseExpect,
    ResponseFile, Tag, Target, TargetFile, known_misses,
};

/// Errors from the harness.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ValidateError {
    /// A data file could not be read or parsed.
    #[error("{file}: {why}")]
    File {
        /// The file.
        file: String,
        /// What went wrong.
        why: String,
    },
}

/// What happened to one check.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    /// Inside the target.
    Pass,
    /// Outside the target.
    Miss,
    /// Outside the target and listed in the known misses.
    KnownMiss,
    /// Listed as a known miss but now inside the target (remove it from the list).
    Fixed,
    /// The variable or scenario was not available.
    NotEvaluated,
}

impl Verdict {
    /// Short name for reports.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Miss => "MISS",
            Self::KnownMiss => "known miss",
            Self::Fixed => "fixed",
            Self::NotEvaluated => "not evaluated",
        }
    }
}

/// One evaluated check.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    /// Target or response id.
    pub id: String,
    /// "target", "response" or "parameter".
    pub kind: &'static str,
    /// Scenario (for a response, `scenario vs baseline`).
    pub scenario: String,
    /// Variable.
    pub variable: String,
    /// When it is read.
    pub at: String,
    /// The expectation, as text.
    pub expected: String,
    /// The sim's value (for a response, the relative change), if available.
    pub got: Option<f64>,
    /// Verdict.
    pub verdict: Verdict,
    /// Gate.
    pub gate: Gate,
    /// Source key.
    pub source: String,
    /// Note, or why it was not evaluated.
    pub note: String,
    /// Over the seed runs, when more than one has a value.
    pub spread: Option<Spread>,
}

impl Outcome {
    /// Whether this outcome fails the build: a miss on an M0 check that is not a known
    /// miss, or an M0 check that could not be evaluated.
    #[must_use]
    pub fn fails_build(&self) -> bool {
        matches!(self.verdict, Verdict::Miss | Verdict::NotEvaluated) && self.gate == Gate::M0
    }
}

/// The runs a validation needs: scenario name to the final world state of each seed run,
/// the scenario as written first.
pub type Runs = BTreeMap<String, Vec<WorldState>>;

/// How the seed runs of one check differ.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spread {
    /// Lowest run.
    pub min: f64,
    /// Highest run.
    pub max: f64,
    /// Sample standard deviation over the runs.
    pub sd: f64,
    /// Runs with a value.
    pub n: usize,
}

/// A mean over seed runs.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedMean {
    /// Mean over the runs that have the value.
    pub value: f64,
    /// Each run's value, `None` where it has none.
    pub per_seed: Vec<Option<f64>>,
    /// A note from the runs (replicate counts), and how many runs have the value when
    /// not all do.
    pub note: String,
}

impl SeedMean {
    /// The runs' range and standard deviation, when more than one run has the value.
    #[must_use]
    pub fn spread(&self) -> Option<Spread> {
        let v: Vec<f64> = self.per_seed.iter().flatten().copied().collect();
        if v.len() < 2 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)] // a handful of runs
        let n = v.len() as f64;
        let mean = v.iter().sum::<f64>() / n;
        let var = v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
        let (min, max) = v
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &x| {
                (lo.min(x), hi.max(x))
            });
        Some(Spread {
            min,
            max,
            sd: libm::sqrt(var),
            n: v.len(),
        })
    }
}

/// The mean of `f` over the seed runs `states`, of which at least half must have a
/// value. `f` gives a run's value and a note.
///
/// # Errors
/// The first run's reason when too few runs have a value.
pub fn seed_mean<S>(
    states: &[S],
    f: impl Fn(&S) -> Result<(f64, String), String>,
) -> Result<SeedMean, String> {
    if states.is_empty() {
        return Err("no runs".into());
    }
    let mut per_seed = Vec::with_capacity(states.len());
    let mut note = String::new();
    let mut first_error = None;
    for s in states {
        match f(s) {
            Ok((v, n)) => {
                if note.is_empty() {
                    note = n;
                }
                per_seed.push(Some(v));
            }
            Err(e) => {
                first_error.get_or_insert(e);
                per_seed.push(None);
            }
        }
    }
    let have: Vec<f64> = per_seed.iter().flatten().copied().collect();
    if have.is_empty() || 2 * have.len() < states.len() {
        return Err(format!(
            "{} ({} of {} seed runs have a value)",
            first_error.unwrap_or_default(),
            have.len(),
            states.len()
        ));
    }
    if have.len() < states.len() {
        note = with_note(
            &note,
            &format!("mean of {} of {} seed runs", have.len(), states.len()),
        );
    }
    #[allow(clippy::cast_precision_loss)] // a handful of runs
    let value = have.iter().sum::<f64>() / have.len() as f64;
    Ok(SeedMean {
        value,
        per_seed,
        note,
    })
}

/// The rows of one plant, in day order.
#[must_use]
pub fn plant_rows(state: &WorldState, plant: u32) -> Vec<&DayRow> {
    state.history.iter().filter(|r| r.plant == plant).collect()
}

/// A replicate mean: the value and how many of the run's plants it averages.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mean {
    /// Mean over the plants that have the value.
    pub value: f64,
    /// Plants that have it.
    pub used: usize,
    /// Plants in the run.
    pub total: usize,
}

impl Mean {
    /// "k of n plants" when some replicates lack the value, else empty.
    #[must_use]
    pub fn note(&self) -> String {
        if self.used < self.total {
            format!("mean of {} of {} plants", self.used, self.total)
        } else {
            String::new()
        }
    }
}

/// The value of `variable` at `at` in a run, or why there is none: the mean over the
/// run's plants that have it, which a scenario uses as replicates (decision 0017). A
/// replicate can lack a value (a pot that never matures, a plant with no pods for
/// seeds per pod); at least half the plants must have it.
///
/// # Errors
/// A sentence saying why the value is not available.
pub fn value(state: &WorldState, variable: &str, at: &At) -> Result<f64, String> {
    replicate_mean(state, variable, at).map(|m| m.value)
}

/// [`value`] with the count of plants it averages.
///
/// # Errors
/// As [`value`].
pub fn replicate_mean(state: &WorldState, variable: &str, at: &At) -> Result<Mean, String> {
    let ids: Vec<u32> = state.plants.iter().map(|p| p.id).collect();
    if ids.is_empty() {
        return Err("the run has no plants".into());
    }
    let mut values = Vec::with_capacity(ids.len());
    let mut first_error = None;
    for &id in &ids {
        match plant_value(state, id, variable, at) {
            Ok(v) => values.push(v),
            // An unknown name is the same for every plant: report it at once.
            Err(e) if e.starts_with("unknown") => return Err(e),
            Err(e) => {
                first_error.get_or_insert(e);
            }
        }
    }
    if values.is_empty() || 2 * values.len() < ids.len() {
        let why = first_error.unwrap_or_default();
        return Err(format!(
            "{why} ({} of {} plants have a value; at least half are needed)",
            values.len(),
            ids.len()
        ));
    }
    #[allow(clippy::cast_precision_loss)] // a handful of plants
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    Ok(Mean {
        value: mean,
        used: values.len(),
        total: ids.len(),
    })
}

fn with_note(note: &str, extra: &str) -> String {
    match (note.is_empty(), extra.is_empty()) {
        (_, true) => note.to_string(),
        (true, false) => extra.to_string(),
        (false, false) => format!("{extra}. {note}"),
    }
}

/// The value of `variable` at `at` for one plant.
///
/// # Errors
/// A sentence saying why the value is not available.
pub fn plant_value(state: &WorldState, plant: u32, variable: &str, at: &At) -> Result<f64, String> {
    let rows = plant_rows(state, plant);
    if rows.is_empty() {
        return Err("the run has no daily rows".into());
    }
    if let Some(v) = derived::stage_day(&rows, variable)? {
        return Ok(v);
    }
    if variable == "mean_internode_cm" {
        if !matches!(at, At::Final) {
            return Err("mean_internode_cm is only available at Final".into());
        }
        return derived::mean_internode_cm(state, plant);
    }
    let one = |i: usize| derived::row_value(state, &rows, i, variable);
    match at {
        At::Final => one(rows.len() - 1),
        At::Day(d) => {
            let i = rows
                .iter()
                .position(|r| r.day == *d)
                .ok_or_else(|| format!("the run has no day {d}"))?;
            one(i)
        }
        At::Stage(s) => {
            let i = rows
                .iter()
                .position(|r| r.report.stage.is_some_and(|x| x >= *s))
                .ok_or_else(|| format!("the run never reaches {s:?}"))?;
            one(i)
        }
        At::Max => {
            let mut best: Option<f64> = None;
            for i in 0..rows.len() {
                if let Ok(v) = one(i)
                    && v.is_finite()
                {
                    best = Some(best.map_or(v, |b: f64| b.max(v)));
                }
            }
            best.ok_or_else(|| format!("{variable} has no finite value"))
        }
    }
}

/// Known-miss entries that match no M0 check: a typo, a removed check or one that is no
/// longer gated. Each is a sentence; the build fails on any.
#[must_use]
pub fn stale_known_misses(known: &KnownMisses, outcomes: &[Outcome]) -> Vec<String> {
    known
        .misses
        .iter()
        .filter_map(|k| match outcomes.iter().find(|o| o.id == k.id) {
            None => Some(format!("{} matches no check", k.id)),
            Some(o) if o.gate != Gate::M0 => {
                Some(format!("{} is gated {:?}, not M0; remove it", k.id, o.gate))
            }
            Some(_) => None,
        })
        .collect()
}

fn known(known: &KnownMisses, id: &str) -> Option<String> {
    known
        .misses
        .iter()
        .find(|k| k.id == id)
        .map(|k| format!("known miss until {}: {}", k.until, k.why))
}

fn settle(pass: bool, id: &str, known_misses: &KnownMisses, note: &mut String) -> Verdict {
    match (pass, known(known_misses, id)) {
        (true, None) => Verdict::Pass,
        (true, Some(_)) => {
            *note = format!("listed as a known miss but now passes; remove it. {note}");
            Verdict::Fixed
        }
        (false, None) => Verdict::Miss,
        (false, Some(k)) => {
            *note = if note.is_empty() {
                k
            } else {
                format!("{k}. {note}")
            };
            Verdict::KnownMiss
        }
    }
}

/// Evaluates the whole-plant targets.
#[must_use]
pub fn evaluate_targets(file: &TargetFile, runs: &Runs, known: &KnownMisses) -> Vec<Outcome> {
    file.targets
        .iter()
        .map(|t| {
            let mut note = t.note.clone();
            let mut spread = None;
            let expected = t.expect.describe(&t.unit);
            let base = |got, verdict, note, spread| Outcome {
                id: t.id.clone(),
                kind: "target",
                scenario: t.scenario.clone(),
                variable: t.variable.clone(),
                at: t.at.describe(),
                expected: expected.clone(),
                got,
                verdict,
                gate: t.gate,
                source: t.sources.join(" "),
                note,
                spread,
            };
            if t.gate == Gate::Later {
                return base(
                    None,
                    Verdict::NotEvaluated,
                    "later: scenario not written yet".into(),
                    None,
                );
            }
            let Some(states) = runs.get(&t.scenario) else {
                return base(
                    None,
                    Verdict::NotEvaluated,
                    format!("no run for scenario {}", t.scenario),
                    None,
                );
            };
            let got = seed_mean(states, |state| {
                if let Expect::LogisticR2 { .. } = t.expect {
                    logistic::series(state, "day", &t.variable)
                        .map(|(x, y)| (logistic::fit(&x, &y).r2, String::new()))
                } else {
                    replicate_mean(state, &t.variable, &t.at).map(|m| (m.value, m.note()))
                }
            })
            .map(|m| {
                note = with_note(&note, &m.note);
                spread = m.spread();
                m.value
            });
            match got {
                Ok(v) => {
                    let verdict = settle(t.expect.holds(v), &t.id, known, &mut note);
                    base(Some(v), verdict, note, spread)
                }
                Err(why) => base(None, Verdict::NotEvaluated, why, None),
            }
        })
        .collect()
}

/// Evaluates the treatment responses. `got` is the relative change
/// `(treatment − baseline) / |baseline|`.
#[must_use]
pub fn evaluate_responses(file: &ResponseFile, runs: &Runs, known: &KnownMisses) -> Vec<Outcome> {
    file.responses
        .iter()
        .map(|r| {
            let mut note = r.note.clone();
            let base = |got, verdict, note, spread| Outcome {
                id: r.id.clone(),
                kind: "response",
                scenario: format!("{} vs {}", r.scenario, r.baseline),
                variable: r.variable.clone(),
                at: r.at.describe(),
                expected: r.expect.describe(),
                got,
                verdict,
                gate: r.gate,
                source: r.sources.join(" "),
                note,
                spread,
            };
            if r.gate == Gate::Later {
                return base(
                    None,
                    Verdict::NotEvaluated,
                    "later: scenario not written yet".into(),
                    None,
                );
            }
            let (Some(t), Some(b)) = (runs.get(&r.scenario), runs.get(&r.baseline)) else {
                return base(
                    None,
                    Verdict::NotEvaluated,
                    "a scenario has no run".into(),
                    None,
                );
            };
            let mut counts = String::new();
            // The change of the treatment's seed mean against the baseline's, and the
            // range of the per-seed changes (seed k of the treatment against seed k of
            // the baseline).
            let mut change = |var: &str| -> Result<(f64, Option<Spread>), String> {
                let per =
                    |s: &WorldState| replicate_mean(s, var, &r.at).map(|m| (m.value, m.note()));
                let mt = seed_mean(t, per)?;
                let mb = seed_mean(b, per)?;
                for (m, which) in [(&mt, "treatment"), (&mb, "baseline")] {
                    if !m.note.is_empty() {
                        counts = with_note(&counts, &format!("{which} {var}: {}", m.note));
                    }
                }
                if mb.value == 0.0 {
                    return Err(format!("{var} is zero in the baseline"));
                }
                let paired = SeedMean {
                    value: 0.0,
                    per_seed: mt
                        .per_seed
                        .iter()
                        .zip(&mb.per_seed)
                        .map(|(a, b)| match (a, b) {
                            (Some(a), Some(b)) if *b != 0.0 => Some((a - b) / b.abs()),
                            _ => None,
                        })
                        .collect(),
                    note: String::new(),
                };
                Ok(((mt.value - mb.value) / mb.value.abs(), paired.spread()))
            };
            let (got, spread) = match change(&r.variable) {
                Ok(c) => c,
                Err(why) => return base(None, Verdict::NotEvaluated, why, None),
            };
            let pass = if let ResponseExpect::FallsMoreThan(other) = &r.expect {
                match change(other) {
                    Ok((o, _)) => got < 0.0 && got < o,
                    Err(why) => return base(Some(got), Verdict::NotEvaluated, why, spread),
                }
            } else {
                r.expect.holds(got)
            };
            note = with_note(&note, &counts);
            let verdict = settle(pass, &r.id, known, &mut note);
            base(Some(got), verdict, note, spread)
        })
        .collect()
}

/// Evaluates the parameter checks against the species file's defaults (`species_text`
/// is the species RON). A parameter is found by name anywhere in the file (a heritable
/// key, a constants field) or by `section.field`; its value is the entry's `default`
/// (heritable) or `value` (constants and the other sections).
#[must_use]
pub fn evaluate_parameters(
    file: &files::ParameterFile,
    species_text: &str,
    known: &KnownMisses,
) -> Vec<Outcome> {
    let species: Result<ron::Value, String> =
        ron::from_str(species_text).map_err(|e| e.to_string());
    file.checks
        .iter()
        .map(|c| {
            let mut note = c.note.clone();
            let base = |got, verdict, note| Outcome {
                id: format!("param_{}", c.parameter),
                kind: "parameter",
                scenario: file.species.clone(),
                variable: c.parameter.clone(),
                at: "default".into(),
                expected: c.expect.describe(&c.unit),
                got,
                verdict,
                gate: c.gate,
                source: c.sources.join(" "),
                note,
                spread: None,
            };
            if c.gate == Gate::Later {
                return base(None, Verdict::NotEvaluated, "later".into());
            }
            let v = species
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|sp| derived::species_parameter(sp, &c.parameter));
            match v {
                Ok(v) => {
                    let id = format!("param_{}", c.parameter);
                    let verdict = settle(c.expect.holds(v), &id, known, &mut note);
                    base(Some(v), verdict, note)
                }
                Err(why) => base(None, Verdict::NotEvaluated, why),
            }
        })
        .collect()
}

/// A digitized curve compared with the sim: the sim's value at each point's x (days
/// after sowing, linear interpolation), and the normalized RMSE.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveComparison {
    /// The curve.
    pub curve: Curve,
    /// The sim's value at each point, if available.
    pub sim: Vec<Option<f64>>,
    /// RMSE over the points with a sim value, divided by the mean observed value.
    pub nrmse: Option<f64>,
    /// The same with each series first divided by its own maximum: the shape alone.
    /// Field curves are at stand density, so a lone pot plant is larger and only the
    /// shape and timing compare (decision 0017).
    pub shape_nrmse: Option<f64>,
    /// Why there is no comparison, if there is none.
    pub note: String,
}

/// Compares each curve with its scenario's run (information only, §8: curves are
/// plotted; the logistic-shape check is a target).
#[must_use]
pub fn compare_curves(curves: &[Curve], runs: &Runs) -> Vec<CurveComparison> {
    curves
        .iter()
        .filter(|c| c.gate != Gate::Later)
        .map(|c| {
            let scenario = c.scenario.clone().unwrap_or_else(|| "single_pot".into());
            let Some(states) = runs.get(&scenario).filter(|s| !s.is_empty()) else {
                return CurveComparison {
                    curve: c.clone(),
                    sim: vec![None; c.points.len()],
                    nrmse: None,
                    shape_nrmse: None,
                    note: format!("no run for scenario {scenario}"),
                };
            };
            let x_var =
                if c.x.variable == "days_since_sowing" || c.x.variable == "days_after_sowing" {
                    "day"
                } else {
                    c.x.variable.as_str()
                };
            // Each point is the mean of the runs that reach it.
            let series: Vec<_> = states
                .iter()
                .map(|s| logistic::series(s, x_var, &c.y.variable))
                .collect();
            let (sim, note) = if let Some(Err(e)) = series.iter().find(|s| s.is_err()) {
                (vec![None; c.points.len()], e.clone())
            } else {
                let sim = c
                    .points
                    .iter()
                    .map(|p| {
                        let v: Vec<f64> = series
                            .iter()
                            .flatten()
                            .filter_map(|(x, y)| interpolate(x, y, p.x))
                            .collect();
                        #[allow(clippy::cast_precision_loss)] // a handful of runs
                        let n = v.len() as f64;
                        (!v.is_empty()).then(|| v.iter().sum::<f64>() / n)
                    })
                    .collect();
                (sim, String::new())
            };
            let pairs: Vec<(f64, f64)> = c
                .points
                .iter()
                .zip(&sim)
                .filter_map(|(p, s)| s.map(|s| (p.y, s)))
                .collect();
            let peak = |f: fn(&(f64, f64)) -> f64| {
                pairs.iter().map(f).fold(0.0_f64, |a, v| a.max(v.abs()))
            };
            let (obs_max, sim_max) = (peak(|p| p.0), peak(|p| p.1));
            let shape: Vec<(f64, f64)> = if obs_max > 0.0 && sim_max > 0.0 {
                pairs
                    .iter()
                    .map(|p| (p.0 / obs_max, p.1 / sim_max))
                    .collect()
            } else {
                Vec::new()
            };
            CurveComparison {
                curve: c.clone(),
                sim,
                nrmse: nrmse_of(&pairs),
                shape_nrmse: nrmse_of(&shape),
                note,
            }
        })
        .collect()
}

/// RMSE of `(observed, sim)` pairs over the mean observed value.
fn nrmse_of(pairs: &[(f64, f64)]) -> Option<f64> {
    if pairs.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)] // point counts are small
    let n = pairs.len() as f64;
    let mean = pairs.iter().map(|p| p.0).sum::<f64>() / n;
    let mse = pairs.iter().map(|p| (p.1 - p.0) * (p.1 - p.0)).sum::<f64>() / n;
    (mean != 0.0).then(|| libm::sqrt(mse) / mean.abs())
}

fn interpolate(x: &[f64], y: &[f64], at: f64) -> Option<f64> {
    debug_assert!(x.windows(2).all(|w| w[0] <= w[1]), "x must ascend");
    let i = x.iter().position(|&v| v >= at)?;
    if i == 0 {
        return ((x[0] - at).abs() < 1e-9).then_some(y[0]);
    }
    let (x0, x1, y0, y1) = (x[i - 1], x[i], y[i - 1], y[i]);
    Some(y0 + (y1 - y0) * (at - x0) / (x1 - x0))
}

/// The results table as CSV, for the plot script.
#[must_use]
pub fn outcomes_csv(outcomes: &[Outcome]) -> String {
    let q = |s: &str| format!("\"{}\"", s.replace('"', "'"));
    let mut out = String::from(
        "id,kind,scenario,variable,at,expected,got,verdict,gate,source,note,seed_min,seed_max,seed_sd,seed_n\n",
    );
    for o in outcomes {
        let got = o.got.map_or_else(String::new, |v| format!("{v}"));
        let (lo, hi, sd, n) = o.spread.map_or_else(
            || (String::new(), String::new(), String::new(), String::new()),
            |s| {
                (
                    format!("{}", s.min),
                    format!("{}", s.max),
                    format!("{}", s.sd),
                    format!("{}", s.n),
                )
            },
        );
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{},{:?},{},{},{},{},{},{}",
            q(&o.id),
            o.kind,
            q(&o.scenario),
            q(&o.variable),
            q(&o.at),
            q(&o.expected),
            got,
            o.verdict.name(),
            o.gate,
            q(&o.source),
            q(&o.note),
            lo,
            hi,
            sd,
            n,
        );
    }
    out
}

/// Formats a number for the report with about four significant digits.
#[must_use]
pub fn fmt_num(v: f64) -> String {
    let a = v.abs();
    if a == 0.0 {
        "0".into()
    } else if a >= 1000.0 {
        format!("{v:.0}")
    } else if a >= 10.0 {
        format!("{v:.1}")
    } else if a >= 0.1 {
        format!("{v:.3}")
    } else {
        format!("{v:.2e}")
    }
}

/// The report as Markdown: a summary line, then one table per kind, misses first.
#[must_use]
pub fn report_markdown(outcomes: &[Outcome], curves: &[CurveComparison]) -> String {
    let count = |v: Verdict| outcomes.iter().filter(|o| o.verdict == v).count();
    let failing = outcomes.iter().filter(|o| o.fails_build()).count();
    let mut s = String::from("# Validation report\n\n");
    let _ = write!(
        s,
        "{} checks: {} pass, {} miss ({} failing the build), {} known miss, {} fixed, {} not evaluated.\n\n",
        outcomes.len(),
        count(Verdict::Pass),
        count(Verdict::Miss),
        failing,
        count(Verdict::KnownMiss),
        count(Verdict::Fixed),
        count(Verdict::NotEvaluated),
    );
    for kind in ["target", "response", "parameter"] {
        let mut rows: Vec<&Outcome> = outcomes.iter().filter(|o| o.kind == kind).collect();
        if rows.is_empty() {
            continue;
        }
        rows.sort_by_key(|o| (o.verdict, o.id.clone()));
        let _ = write!(
            s,
            "## {}s\n\n| Verdict | Id | Scenario | Variable | At | Expected | Sim | Gate | Source | Note |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n",
            match kind {
                "target" => "Target",
                "response" => "Response",
                _ => "Parameter",
            }
        );
        for o in rows {
            let show = |v: f64| {
                if kind == "response" {
                    format!("{:+.0}%", v * 100.0)
                } else {
                    fmt_num(v)
                }
            };
            let got = o.got.map_or_else(
                || "-".into(),
                |v| match o.spread {
                    Some(sp) => {
                        let sd = if kind == "response" {
                            format!("{:.0}%", sp.sd * 100.0)
                        } else {
                            fmt_num(sp.sd)
                        };
                        format!(
                            "{} ± {sd} ({} to {}, n {})",
                            show(v),
                            show(sp.min),
                            show(sp.max),
                            sp.n
                        )
                    }
                    None => show(v),
                },
            );
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | {} | {} | {} | {:?} | {} | {} |",
                o.verdict.name(),
                o.id,
                o.scenario,
                o.variable,
                o.at,
                o.expected,
                got,
                o.gate,
                o.source,
                o.note.replace('|', "/"),
            );
        }
        s.push('\n');
    }
    if !curves.is_empty() {
        s.push_str("## Curves (information)\n\nShape NRMSE divides each series by its own maximum first: field curves are at stand density, so a lone pot plant is larger and only shape and timing compare.\n\n| Curve | Variable | Points | NRMSE | Shape NRMSE | Note |\n| --- | --- | --- | --- | --- | --- |\n");
        let pct = |v: Option<f64>| v.map_or_else(|| "-".into(), |v| format!("{:.0}%", v * 100.0));
        for c in curves {
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | {} | {} |",
                c.curve.id,
                c.curve.y.variable,
                c.curve.points.len(),
                pct(c.nrmse),
                pct(c.shape_nrmse),
                c.note
            );
        }
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(kind: &'static str) -> Outcome {
        Outcome {
            id: format!("{kind}_1"),
            kind,
            scenario: "single_pot".into(),
            variable: "pods".into(),
            at: "final".into(),
            expected: "1 to 2".into(),
            got: Some(1.5),
            verdict: Verdict::Pass,
            gate: Gate::Info,
            source: "s".into(),
            note: String::new(),
            spread: None,
        }
    }

    #[test]
    fn a_seed_mean_needs_half_the_runs_and_reports_the_range() {
        let states = [Some(1.0), Some(3.0), None, Some(2.0)];
        let m = seed_mean(&states, |v: &Option<f64>| {
            v.map(|v| (v, String::new()))
                .ok_or_else(|| "none".to_string())
        })
        .unwrap();
        assert!((m.value - 2.0).abs() < 1e-15);
        let sp = m.spread().unwrap();
        assert_eq!((sp.min, sp.max, sp.n), (1.0, 3.0, 3));
        assert!((sp.sd - 1.0).abs() < 1e-15);
        assert!(m.note.contains("3 of 4 seed runs"), "{}", m.note);
        let none = seed_mean(&states, |_: &Option<f64>| Err("no pods".into())).unwrap_err();
        assert!(none.contains("0 of 4"), "{none}");
    }

    #[test]
    fn the_report_has_one_section_per_kind() {
        let md = report_markdown(
            &[outcome("target"), outcome("response"), outcome("parameter")],
            &[],
        );
        for title in ["## Targets", "## Responses", "## Parameters"] {
            assert_eq!(md.matches(title).count(), 1, "{title} in\n{md}");
        }
    }

    #[test]
    fn known_misses_must_name_an_m0_check() {
        let mut gated = outcome("target");
        gated.gate = Gate::M0;
        let info = Outcome {
            id: "info_1".into(),
            ..outcome("target")
        };
        let entry = |id: &str| files::KnownMiss {
            id: id.into(),
            until: "WP12".into(),
            why: String::new(),
        };
        let known = KnownMisses {
            misses: vec![entry("target_1"), entry("info_1"), entry("typo")],
        };
        let stale = stale_known_misses(&known, &[gated, info]);
        assert_eq!(stale.len(), 2, "{stale:?}");
        assert!(stale[0].contains("info_1") && stale[1].contains("typo"));
    }

    #[test]
    fn an_unevaluated_m0_check_fails_the_build() {
        let mut o = outcome("target");
        o.gate = Gate::M0;
        o.verdict = Verdict::NotEvaluated;
        assert!(o.fails_build());
        o.gate = Gate::Info;
        assert!(!o.fails_build());
    }
}
