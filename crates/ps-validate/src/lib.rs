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
//! [`derived`]. Values are taken from plant 0 of the scenario.

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
    /// "target", "response" or "curve".
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
}

impl Outcome {
    /// Whether this outcome fails the build: a miss on an M0 target that is not a known
    /// miss.
    #[must_use]
    pub fn fails_build(&self) -> bool {
        self.verdict == Verdict::Miss && self.gate == Gate::M0
    }
}

/// The runs a validation needs: scenario name to final world state.
pub type Runs = BTreeMap<String, WorldState>;

/// The rows of plant 0, in day order.
#[must_use]
pub fn plant_rows(state: &WorldState) -> Vec<&DayRow> {
    state.history.iter().filter(|r| r.plant == 0).collect()
}

/// The value of `variable` at `at` in a run, or why there is none.
///
/// # Errors
/// A sentence saying why the value is not available.
pub fn value(state: &WorldState, variable: &str, at: &At) -> Result<f64, String> {
    let rows = plant_rows(state);
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
        return derived::mean_internode_cm(state);
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
            let expected = t.expect.describe(&t.unit);
            let base = |got, verdict, note| Outcome {
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
            };
            if t.gate == Gate::Later {
                return base(
                    None,
                    Verdict::NotEvaluated,
                    "later: scenario not written yet".into(),
                );
            }
            let Some(state) = runs.get(&t.scenario) else {
                return base(
                    None,
                    Verdict::NotEvaluated,
                    format!("no run for scenario {}", t.scenario),
                );
            };
            let got = if let Expect::LogisticR2 { .. } = t.expect {
                logistic::series(state, "day", &t.variable).map(|(x, y)| logistic::fit(&x, &y).r2)
            } else {
                value(state, &t.variable, &t.at)
            };
            match got {
                Ok(v) => {
                    let verdict = settle(t.expect.holds(v), &t.id, known, &mut note);
                    base(Some(v), verdict, note)
                }
                Err(why) => base(None, Verdict::NotEvaluated, why),
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
            let base = |got, verdict, note| Outcome {
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
            };
            if r.gate == Gate::Later {
                return base(
                    None,
                    Verdict::NotEvaluated,
                    "later: scenario not written yet".into(),
                );
            }
            let (Some(t), Some(b)) = (runs.get(&r.scenario), runs.get(&r.baseline)) else {
                return base(None, Verdict::NotEvaluated, "a scenario has no run".into());
            };
            let change = |var: &str| -> Result<f64, String> {
                let vt = value(t, var, &r.at)?;
                let vb = value(b, var, &r.at)?;
                if vb == 0.0 {
                    return Err(format!("{var} is zero in the baseline"));
                }
                Ok((vt - vb) / vb.abs())
            };
            let got = match change(&r.variable) {
                Ok(c) => c,
                Err(why) => return base(None, Verdict::NotEvaluated, why),
            };
            let pass = if let ResponseExpect::FallsMoreThan(other) = &r.expect {
                match change(other) {
                    Ok(o) => got < 0.0 && got < o,
                    Err(why) => return base(Some(got), Verdict::NotEvaluated, why),
                }
            } else {
                r.expect.holds(got)
            };
            let verdict = settle(pass, &r.id, known, &mut note);
            base(Some(got), verdict, note)
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
            let Some(state) = runs.get(&scenario) else {
                return CurveComparison {
                    curve: c.clone(),
                    sim: vec![None; c.points.len()],
                    nrmse: None,
                    note: format!("no run for scenario {scenario}"),
                };
            };
            let x_var =
                if c.x.variable == "days_since_sowing" || c.x.variable == "days_after_sowing" {
                    "day"
                } else {
                    c.x.variable.as_str()
                };
            let series = logistic::series(state, x_var, &c.y.variable);
            let (sim, note) = match &series {
                Ok((x, y)) => (
                    c.points.iter().map(|p| interpolate(x, y, p.x)).collect(),
                    String::new(),
                ),
                Err(e) => (vec![None; c.points.len()], e.clone()),
            };
            let pairs: Vec<(f64, f64)> = c
                .points
                .iter()
                .zip(&sim)
                .filter_map(|(p, s)| s.map(|s| (p.y, s)))
                .collect();
            #[allow(clippy::cast_precision_loss)] // point counts are small
            let nrmse = if pairs.is_empty() {
                None
            } else {
                let n = pairs.len() as f64;
                let mean = pairs.iter().map(|p| p.0).sum::<f64>() / n;
                let mse = pairs.iter().map(|p| (p.1 - p.0) * (p.1 - p.0)).sum::<f64>() / n;
                (mean != 0.0).then(|| libm::sqrt(mse) / mean.abs())
            };
            CurveComparison {
                curve: c.clone(),
                sim,
                nrmse,
                note,
            }
        })
        .collect()
}

fn interpolate(x: &[f64], y: &[f64], at: f64) -> Option<f64> {
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
    let mut out =
        String::from("id,kind,scenario,variable,at,expected,got,verdict,gate,source,note\n");
    for o in outcomes {
        let got = o.got.map_or_else(String::new, |v| format!("{v}"));
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{},{:?},{},{}",
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
            if kind == "target" {
                "Target"
            } else {
                "Response"
            }
        );
        for o in rows {
            let got = o.got.map_or_else(
                || "-".into(),
                |v| {
                    if kind == "response" {
                        format!("{:+.0}%", v * 100.0)
                    } else {
                        fmt_num(v)
                    }
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
        s.push_str("## Curves (information)\n\n| Curve | Variable | Points | NRMSE | Note |\n| --- | --- | --- | --- | --- |\n");
        for c in curves {
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | {} |",
                c.curve.id,
                c.curve.y.variable,
                c.curve.points.len(),
                c.nrmse
                    .map_or_else(|| "-".into(), |v| format!("{:.0}%", v * 100.0)),
                c.note
            );
        }
        s.push('\n');
    }
    s
}
