//! The reference data files (docs/validation-data.md) as serde types. Unknown fields
//! are ignored, so the data files can carry more than the harness reads.

use std::path::Path;

use ps_physio::Stage;
use serde::{Deserialize, Serialize};

use crate::ValidateError;

/// When a variable is read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum At {
    /// The last day of the run.
    Final,
    /// A sim day (days after sowing).
    Day(u32),
    /// The largest value over the run.
    Max,
    /// The first day at or past a stage.
    Stage(Stage),
}

impl At {
    /// Short text for reports.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Final => "final".into(),
            Self::Day(d) => format!("day {d}"),
            Self::Max => "max".into(),
            Self::Stage(s) => format!("at {s:?}"),
        }
    }
}

/// Whether a check can fail the build.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gate {
    /// An M0 target: a miss fails the build unless it is a listed known miss.
    M0,
    /// Information only: reported and plotted, never fails.
    Info,
    /// Needs a scenario that does not exist yet; skipped.
    Later,
}

/// What a whole-plant target expects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expect {
    /// Inside `[min, max]`.
    Range {
        /// Lower bound.
        min: f64,
        /// Upper bound.
        max: f64,
    },
    /// Within `rel_tol` of `value`, relative.
    Approx {
        /// Reference value.
        value: f64,
        /// Relative tolerance.
        rel_tol: f64,
    },
    /// A logistic fit to the variable's time course has at least this R² (§8.3 pass
    /// rule for the biomass curve).
    LogisticR2 {
        /// Smallest R² that passes.
        min_r2: f64,
    },
}

impl Expect {
    /// Whether `v` meets the expectation.
    #[must_use]
    pub fn holds(&self, v: f64) -> bool {
        match *self {
            Self::Range { min, max } => v >= min && v <= max,
            Self::Approx { value, rel_tol } => (v - value).abs() <= rel_tol * value.abs(),
            Self::LogisticR2 { min_r2 } => v >= min_r2,
        }
    }

    /// Short text for reports.
    #[must_use]
    pub fn describe(&self, unit: &str) -> String {
        match *self {
            Self::Range { min, max } => format!("{min} to {max} {unit}"),
            Self::Approx { value, rel_tol } => {
                format!("{value} {unit} ± {:.0}%", rel_tol * 100.0)
            }
            Self::LogisticR2 { min_r2 } => format!("logistic R² ≥ {min_r2}"),
        }
    }
}

/// One whole-plant target (§8.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    /// Unique id.
    pub id: String,
    /// Scenario name (file stem).
    pub scenario: String,
    /// Variable: a daily.csv column or a derived name.
    pub variable: String,
    /// When it is read.
    pub at: At,
    /// What is expected.
    pub expect: Expect,
    /// Unit of the expectation (the variable's own unit).
    pub unit: String,
    /// Basis, such as a thermal-time base temperature.
    #[serde(default)]
    pub basis: Option<String>,
    /// Source keys.
    pub sources: Vec<String>,
    /// Gate.
    pub gate: Gate,
    /// Note.
    #[serde(default)]
    pub note: String,
}

/// `targets.ron`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "ValidationTargets")]
pub struct TargetFile {
    /// Schema version.
    pub schema: u32,
    /// Species id.
    pub species: String,
    /// Offsets added to every scenario's world seed: each scenario runs once per offset,
    /// and every target, response and curve reads the mean over those runs (review of
    /// WP25). Offset 0 is the scenario as written. Empty means one run.
    #[serde(default)]
    pub seed_offsets: Vec<u64>,
    /// Targets.
    pub targets: Vec<Target>,
}

/// What a response expects of `(treatment − baseline) / |baseline|`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResponseExpect {
    /// Below zero.
    Decrease,
    /// Above zero.
    Increase,
    /// Inside `[min, max]`.
    RelChange {
        /// Lower bound.
        min: f64,
        /// Upper bound.
        max: f64,
    },
    /// Falls, and by more (relatively) than the named variable.
    FallsMoreThan(String),
    /// The §8.3 rule: same sign as the cited change and within a factor of two of it.
    Cited {
        /// The cited relative change.
        rel_change: f64,
    },
}

impl ResponseExpect {
    /// Whether a relative change `c` meets the expectation (`FallsMoreThan` is checked
    /// by the caller, which knows the other variable).
    #[must_use]
    pub fn holds(&self, c: f64) -> bool {
        match *self {
            Self::Decrease | Self::FallsMoreThan(_) => c < 0.0,
            Self::Increase => c > 0.0,
            Self::RelChange { min, max } => c >= min && c <= max,
            Self::Cited { rel_change } => {
                c * rel_change > 0.0
                    && c.abs() >= 0.5 * rel_change.abs()
                    && c.abs() <= 2.0 * rel_change.abs()
            }
        }
    }

    /// Short text for reports.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Decrease => "decrease".into(),
            Self::Increase => "increase".into(),
            Self::RelChange { min, max } => {
                format!("{:+.0}% to {:+.0}%", min * 100.0, max * 100.0)
            }
            Self::FallsMoreThan(o) => format!("falls more than {o}"),
            Self::Cited { rel_change } => format!(
                "{:+.0}% (×½ to ×2: {:+.0}% to {:+.0}%)",
                rel_change * 100.0,
                rel_change * 50.0,
                rel_change * 200.0
            ),
        }
    }
}

/// Whether a response is encoded in the model or emergent (§8.3 tag column).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tag {
    /// The mechanism is a rule in the model; the magnitude is the check.
    Encoded,
    /// Partly a rule in the model.
    PartlyEncoded,
    /// Nothing encodes it.
    Emergent,
}

/// One treatment response (§8.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Unique id.
    pub id: String,
    /// Treatment scenario.
    pub scenario: String,
    /// Baseline scenario.
    pub baseline: String,
    /// Variable.
    pub variable: String,
    /// When it is read (in both runs).
    pub at: At,
    /// What is expected.
    pub expect: ResponseExpect,
    /// Source keys.
    pub sources: Vec<String>,
    /// Gate.
    pub gate: Gate,
    /// Encoded or emergent.
    pub tag: Tag,
    /// Note.
    #[serde(default)]
    pub note: String,
}

/// `responses.ron`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "ValidationResponses")]
pub struct ResponseFile {
    /// Schema version.
    pub schema: u32,
    /// Species id.
    #[serde(default)]
    pub species: String,
    /// Responses.
    pub responses: Vec<Response>,
}

/// A curve axis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    /// Variable name (a daily.csv column or derived name; x is a day count or a thermal
    /// time).
    pub variable: String,
    /// Unit.
    pub unit: String,
}

/// One digitized curve.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Curve {
    /// Schema version.
    pub schema: u32,
    /// Unique id.
    pub id: String,
    /// Source key.
    pub source: String,
    /// Scenario to compare with; `single_pot` if absent.
    #[serde(default)]
    pub scenario: Option<String>,
    /// x axis.
    pub x: Axis,
    /// y axis.
    pub y: Axis,
    /// Points.
    pub points: Vec<Point>,
    /// How the points were taken.
    #[serde(default)]
    pub method: String,
    /// Growing conditions of the reference.
    #[serde(default)]
    pub conditions: String,
    /// Basis.
    #[serde(default)]
    pub basis: Option<String>,
    /// Gate (curves are information only; `Later` skips the curve).
    #[serde(default = "info")]
    pub gate: Gate,
}

fn info() -> Gate {
    Gate::Info
}

/// One point of a curve.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// x.
    pub x: f64,
    /// y.
    pub y: f64,
    /// Standard deviation of y, if reported.
    #[serde(default)]
    pub sd: Option<f64>,
}

/// One check of a species-file default against the literature.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParamCheck {
    /// A heritable key, a constants field, or `section.field`.
    pub parameter: String,
    /// What is expected.
    pub expect: Expect,
    /// Unit.
    pub unit: String,
    /// Basis.
    #[serde(default)]
    pub basis: Option<String>,
    /// Source keys.
    pub sources: Vec<String>,
    /// Gate.
    pub gate: Gate,
    /// Note.
    #[serde(default)]
    pub note: String,
}

/// `parameters.ron`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "ParameterChecks")]
pub struct ParameterFile {
    /// Schema version.
    pub schema: u32,
    /// Species id.
    pub species: String,
    /// Checks.
    pub checks: Vec<ParamCheck>,
}

/// One known miss.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnownMiss {
    /// Target or response id.
    pub id: String,
    /// The work package expected to fix it.
    pub until: String,
    /// Why it misses.
    pub why: String,
}

/// `validation/known_misses.ron`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KnownMisses {
    /// The list.
    pub misses: Vec<KnownMiss>,
}

fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ValidateError> {
    let err = |why: String| ValidateError::File {
        file: path.display().to_string(),
        why,
    };
    let text = std::fs::read_to_string(path).map_err(|e| err(e.to_string()))?;
    ron::from_str(&text).map_err(|e| err(e.to_string()))
}

fn read_opt<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, ValidateError> {
    if path.exists() {
        read(path).map(Some)
    } else {
        Ok(None)
    }
}

/// Everything under the validation data directory.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data {
    /// Targets, if the file exists.
    pub targets: Option<TargetFile>,
    /// Responses, if the file exists.
    pub responses: Option<ResponseFile>,
    /// Parameter checks, if the file exists.
    pub parameters: Option<ParameterFile>,
    /// Curves, sorted by path.
    pub curves: Vec<Curve>,
}

impl Data {
    /// Reads `targets.ron`, `responses.ron` and `curves/*.ron` from `dir`; missing files
    /// are empty.
    ///
    /// # Errors
    /// [`ValidateError::File`] for a file that exists but does not parse.
    pub fn load(dir: &Path) -> Result<Self, ValidateError> {
        let mut paths = Vec::new();
        walk(&dir.join("curves"), &mut paths)?;
        paths.sort();
        let mut curves = Vec::new();
        for p in paths {
            curves.push(read(&p)?);
        }
        Ok(Self {
            targets: read_opt(&dir.join("targets.ron"))?,
            responses: read_opt(&dir.join("responses.ron"))?,
            parameters: read_opt(&dir.join("parameters.ron"))?,
            curves,
        })
    }

    /// The seed offsets every scenario runs with (see [`TargetFile::seed_offsets`]):
    /// `[0]` when none are listed.
    #[must_use]
    pub fn seed_offsets(&self) -> Vec<u64> {
        match &self.targets {
            Some(t) if !t.seed_offsets.is_empty() => t.seed_offsets.clone(),
            _ => vec![0],
        }
    }

    /// Every scenario the data refers to, sorted, without repeats.
    #[must_use]
    pub fn scenarios(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if let Some(t) = &self.targets {
            v.extend(
                t.targets
                    .iter()
                    .filter(|t| t.gate != Gate::Later)
                    .map(|t| t.scenario.clone()),
            );
        }
        if let Some(r) = &self.responses {
            for x in r.responses.iter().filter(|x| x.gate != Gate::Later) {
                v.push(x.scenario.clone());
                v.push(x.baseline.clone());
            }
        }
        for c in self.curves.iter().filter(|c| c.gate != Gate::Later) {
            v.push(c.scenario.clone().unwrap_or_else(|| "single_pot".into()));
        }
        v.sort();
        v.dedup();
        v
    }
}

/// Every `.ron` file under `dir`, recursively; nothing if `dir` does not exist.
fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), ValidateError> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir).map_err(|e| ValidateError::File {
        file: dir.display().to_string(),
        why: e.to_string(),
    })?;
    for e in entries.filter_map(Result::ok) {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out)?;
        } else if p.extension().is_some_and(|x| x == "ron") {
            out.push(p);
        }
    }
    Ok(())
}

/// Reads the known-miss list; an absent file is an empty list.
///
/// # Errors
/// [`ValidateError::File`] for a file that exists but does not parse.
pub fn known_misses(path: &Path) -> Result<KnownMisses, ValidateError> {
    Ok(read_opt(path)?.unwrap_or_default())
}
