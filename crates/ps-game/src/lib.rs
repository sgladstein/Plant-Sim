//! Game layer over the headless sim (design plan §2.6, §6.4, WP32): challenge files
//! scored from the replay, and the run debrief that says what held each plant back.
//!
//! Nothing here changes a model result. A challenge is a scenario, a set of rules on
//! the action log, and a metric read from the daily table; the debrief reads the same
//! table against the species' own thresholds.

pub mod debrief;

use std::path::Path;

use ps_units::{GramsN, KgWater, M2};
use ps_world::{
    Action, ActionKind, Catalog, DayRow, Fidelity, HydraulicsLevel, Scenario, SoilSpec, World,
    WorldState,
};
use serde::{Deserialize, Serialize};

pub use debrief::{Debrief, DebriefRefs, Factor, PlantDebrief, WeekDebrief, debrief, week_event};

/// Schema of challenge files.
pub const CHALLENGE_SCHEMA: u32 = 1;

/// Errors reading a challenge.
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    /// The RON did not parse.
    #[error("parsing a challenge: {0}")]
    Parse(#[from] ron::error::SpannedError),
    /// The schema is not one this build reads.
    #[error("challenge schema {found}, this build reads {CHALLENGE_SCHEMA}")]
    Schema {
        /// The file's schema.
        found: u32,
    },
    /// A file could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The path.
        path: String,
        /// The error.
        source: std::io::Error,
    },
    /// The scenario could not be parsed, loaded or run.
    #[error("the challenge's scenario: {0}")]
    World(#[from] ps_world::WorldError),
    /// Medal thresholds out of order.
    #[error(
        "challenge {0}: medal thresholds must run bronze, silver, gold in the score's direction"
    )]
    Medals(String),
    /// A season's replay panicked.
    #[error("a season's replay panicked")]
    Panicked,
}

/// A scored scenario (design plan §6.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Challenge")]
pub struct Challenge {
    /// File schema.
    pub schema: u32,
    /// Stable id, the file stem.
    pub id: String,
    /// Title shown on the card.
    pub title: String,
    /// The task, one or two sentences.
    pub brief: String,
    /// A nudge toward the science that decides it.
    pub hint: String,
    /// Scenario path under `data/scenarios/`, without `.ron`.
    pub scenario: String,
    /// What is scored.
    pub score: ScoreSpec,
    /// What the player may do.
    pub rules: Rules,
    /// Medal thresholds, in score units.
    pub medals: Medals,
    /// Seasons scored: the log is replayed on this many world seeds (the scenario's,
    /// then the next ones up) and the medal goes to the mean. One season's pod set is
    /// chance enough to swing a score by a few pods, so a single run can reward a poor
    /// plan; the mean rewards the plan.
    #[serde(default = "one_season")]
    pub seasons: u32,
    /// A water-network level the challenge runs at whatever the viewer asks
    /// (`Locked(level)`; R3c, decision 0045); the scenario's own if absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fidelity: Option<ChallengeFidelity>,
}

fn one_season() -> u32 {
    1
}

/// A challenge's hold on the water network's level.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeFidelity {
    /// Every plant at this level; `SetFidelity` actions are refused.
    Locked(HydraulicsLevel),
}

/// The score: a metric and its direction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreSpec {
    /// What to measure.
    pub metric: Metric,
    /// Short label with its unit, e.g. "pods".
    pub label: String,
    /// Whether more is better.
    pub higher_is_better: bool,
}

/// A number read from the run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Metric {
    /// Each plant's highest daily value of a daily-table column over the run, summed
    /// over plants. Peak, not final, so harvesting pods does not lower a pod count.
    PeakSum(String),
    /// The last day's value of a column, summed over plants.
    FinalSum(String),
}

/// What the player may do in a challenge.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    /// Action types allowed in the log; any other is a rule break.
    pub allowed: Vec<ActionType>,
    /// Most water the player may add by can over the run, kg (1 kg = 1 L).
    #[serde(default)]
    pub water_budget: Option<KgWater>,
    /// Most mineral nitrogen the player may add, g N.
    #[serde(default)]
    pub nitrogen_budget: Option<GramsN>,
    /// Sow actions must fall on these scenario days, inclusive.
    #[serde(default)]
    pub sow_window_days: Option<(u32, u32)>,
    /// Most seeds the player may sow.
    #[serde(default)]
    pub max_sowings: Option<u32>,
    /// The species the player sows; loaded with the scenario so the sow tool can use it.
    #[serde(default)]
    pub sow_species: Option<String>,
}

/// Kinds of action, for the rules.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionType {
    /// Sow a seed.
    Sow,
    /// Water by can.
    Water,
    /// Change the automatic watering.
    Irrigation,
    /// Change the shade cloth.
    Shade,
    /// Prune.
    Prune,
    /// Harvest a pod or ear.
    Harvest,
    /// Place an obstacle.
    Obstacle,
    /// Add a stake or trellis.
    Support,
    /// Fertilize.
    Fertilize,
    /// Hand-pollinate.
    Pollinate,
    /// The viewer's choice of water-network level (R3c): not the player's, so never a
    /// rule break.
    Fidelity,
}

impl ActionType {
    /// The type of an action.
    #[must_use]
    pub fn of(kind: &ActionKind) -> Self {
        match kind {
            ActionKind::Sow(_) => Self::Sow,
            ActionKind::Water { .. } => Self::Water,
            ActionKind::SetIrrigation { .. } => Self::Irrigation,
            ActionKind::SetRadiationFraction { .. } => Self::Shade,
            ActionKind::Prune { .. } => Self::Prune,
            ActionKind::Harvest { .. } => Self::Harvest,
            ActionKind::PlaceObstacle(_) => Self::Obstacle,
            ActionKind::AddSupport(_) => Self::Support,
            ActionKind::Fertilize { .. } => Self::Fertilize,
            ActionKind::Pollinate { .. } => Self::Pollinate,
            ActionKind::SetFidelity { .. } => Self::Fidelity,
        }
    }
}

/// Medal thresholds in score units: reaching one earns it.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Medals {
    /// Bronze.
    pub bronze: f64,
    /// Silver.
    pub silver: f64,
    /// Gold.
    pub gold: f64,
}

/// A medal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Medal {
    /// Third tier.
    Bronze,
    /// Second tier.
    Silver,
    /// Top tier.
    Gold,
}

/// One broken rule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RuleBreak {
    /// An action type the challenge does not allow, at this tick.
    NotAllowed {
        /// Tick of the action.
        tick: u64,
        /// Its type.
        action: ActionType,
    },
    /// More water than the budget.
    OverWaterBudget {
        /// Water added, kg.
        used: KgWater,
    },
    /// More nitrogen than the budget.
    OverNitrogenBudget {
        /// Nitrogen added, g N.
        used: GramsN,
    },
    /// A sowing of another species than the challenge's, at this tick.
    WrongSpecies {
        /// Tick of the sowing.
        tick: u64,
    },
    /// A sowing outside the window, at this tick.
    SowOutsideWindow {
        /// Tick of the sowing.
        tick: u64,
    },
    /// More sowings than allowed.
    TooManySowings {
        /// Sowings in the log.
        count: u32,
    },
    /// The run was scored before the scenario's last day.
    Unfinished {
        /// Days run.
        days: u32,
    },
}

impl std::fmt::Display for RuleBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAllowed { tick, action } => {
                write!(f, "day {}: {action:?} is not allowed here", tick / 24)
            }
            Self::OverWaterBudget { used } => write!(f, "used {:.1} L of water", used.0),
            Self::OverNitrogenBudget { used } => write!(f, "used {:.2} g of nitrogen", used.0),
            Self::SowOutsideWindow { tick } => {
                write!(f, "day {}: sown outside the window", tick / 24)
            }
            Self::TooManySowings { count } => write!(f, "{count} seeds sown"),
            Self::WrongSpecies { tick } => write!(f, "day {}: another species sown", tick / 24),
            Self::Unfinished { days } => write!(f, "only {days} days run"),
        }
    }
}

/// A scored run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    /// The score, in the challenge's units.
    pub score: f64,
    /// The medal earned; `None` below bronze or with any rule broken.
    pub medal: Option<Medal>,
    /// Broken rules; a run with any is not ranked.
    pub breaks: Vec<RuleBreak>,
    /// Water added by can, kg.
    pub water_used: KgWater,
    /// Mineral nitrogen added, g N.
    pub nitrogen_used: GramsN,
    /// Model version the run was simulated with.
    pub model_version: String,
    /// Whether it matches the scenario's pinned version (scores across versions differ).
    pub version_matches: bool,
    /// Each season's score, the scenario's own world seed first; `score` is their mean.
    pub season_scores: Vec<f64>,
}

impl Outcome {
    /// Adds the scores of the other seasons: `score` becomes the mean over all of them
    /// and the medal follows it.
    #[must_use]
    pub fn with_seasons(mut self, c: &Challenge, more: &[f64]) -> Self {
        self.season_scores.extend_from_slice(more);
        #[allow(clippy::cast_precision_loss)]
        let n = self.season_scores.len().max(1) as f64;
        self.score = self.season_scores.iter().sum::<f64>() / n + 0.0;
        self.medal = if self.breaks.is_empty() {
            c.medal_for(self.score)
        } else {
            None
        };
        self
    }
}

impl Challenge {
    /// Reads and checks a challenge file.
    ///
    /// # Errors
    /// [`GameError`] on bad RON, an unknown schema, or medals out of order.
    pub fn from_ron(text: &str) -> Result<Self, GameError> {
        let c: Self = ron::from_str(text)?;
        if c.schema != CHALLENGE_SCHEMA {
            return Err(GameError::Schema { found: c.schema });
        }
        let m = c.medals;
        let ordered = if c.score.higher_is_better {
            m.bronze <= m.silver && m.silver <= m.gold
        } else {
            m.bronze >= m.silver && m.silver >= m.gold
        };
        if !ordered {
            return Err(GameError::Medals(c.id));
        }
        Ok(c)
    }

    /// The medal a score earns, ignoring rules.
    #[must_use]
    pub fn medal_for(&self, score: f64) -> Option<Medal> {
        let reach = |t: f64| {
            if self.score.higher_is_better {
                score >= t
            } else {
                score <= t
            }
        };
        if reach(self.medals.gold) {
            Some(Medal::Gold)
        } else if reach(self.medals.silver) {
            Some(Medal::Silver)
        } else if reach(self.medals.bronze) {
            Some(Medal::Bronze)
        } else {
            None
        }
    }

    /// Checks a log against the rules. `scenario` gives soil areas for the budgets.
    #[must_use]
    pub fn check_log(&self, scenario: &Scenario, log: &[Action]) -> (Vec<RuleBreak>, Usage) {
        let r = &self.rules;
        let mut breaks = Vec::new();
        let usage = usage(scenario, log);
        let mut sowings = 0_u32;
        for a in log {
            let t = ActionType::of(&a.kind);
            if t != ActionType::Fidelity && !r.allowed.contains(&t) {
                breaks.push(RuleBreak::NotAllowed {
                    tick: a.tick,
                    action: t,
                });
            }
            if let ActionKind::Sow(s) = &a.kind {
                sowings += 1;
                if r.sow_species.as_ref().is_some_and(|id| *id != s.species) {
                    breaks.push(RuleBreak::WrongSpecies { tick: a.tick });
                }
                if let Some((lo, hi)) = r.sow_window_days {
                    let day = a.tick / 24;
                    if day < u64::from(lo) || day > u64::from(hi) {
                        breaks.push(RuleBreak::SowOutsideWindow { tick: a.tick });
                    }
                }
            }
        }
        if let Some(b) = r.water_budget
            && usage.water.0 > b.0 + 1e-9
        {
            breaks.push(RuleBreak::OverWaterBudget { used: usage.water });
        }
        if let Some(b) = r.nitrogen_budget
            && usage.nitrogen.0 > b.0 + 1e-9
        {
            breaks.push(RuleBreak::OverNitrogenBudget {
                used: usage.nitrogen,
            });
        }
        if let Some(m) = r.max_sowings
            && sowings > m
        {
            breaks.push(RuleBreak::TooManySowings { count: sowings });
        }
        (breaks, usage)
    }

    /// The metric on a run's daily table.
    #[must_use]
    pub fn measure(&self, state: &WorldState) -> f64 {
        metric(&self.score.metric, &state.history)
    }

    /// Scores a finished run: the rules on its action log, then the metric.
    #[must_use]
    pub fn judge(&self, state: &WorldState) -> Outcome {
        let (mut breaks, usage) = self.check_log(&state.scenario, &state.log);
        // Days run: the daily table's, or the hash list's when the table is empty (a
        // viewer's copy of the state carries the table but not the hashes).
        let days = state
            .history
            .last()
            .map_or(0, |r| r.day + 1)
            .max(u32::try_from(state.hashes.len()).unwrap_or(u32::MAX));
        if days < state.scenario.duration_days {
            breaks.push(RuleBreak::Unfinished { days });
        }
        let score = self.measure(state);
        let medal = if breaks.is_empty() {
            self.medal_for(score)
        } else {
            None
        };
        Outcome {
            score,
            medal,
            breaks,
            water_used: usage.water,
            nitrogen_used: usage.nitrogen,
            model_version: state.model_version.clone(),
            version_matches: state.model_version == state.scenario.model_version,
            season_scores: vec![score],
        }
    }
}

/// Reads `data/challenges/<id>.ron`.
///
/// # Errors
/// [`GameError`] on a missing or bad file.
pub fn load_challenge(data_dir: &Path, id: &str) -> Result<Challenge, GameError> {
    let path = data_dir.join("challenges").join(format!("{id}.ron"));
    Challenge::from_ron(&read(&path)?)
}

/// Reads the challenge's scenario, with the challenge's water-network lock applied.
///
/// # Errors
/// [`GameError`] on a missing or bad file.
pub fn load_scenario(data_dir: &Path, c: &Challenge) -> Result<Scenario, GameError> {
    let path = data_dir
        .join("scenarios")
        .join(format!("{}.ron", c.scenario));
    let mut scenario = Scenario::from_ron(&read(&path)?)?;
    if let Some(ChallengeFidelity::Locked(level)) = c.fidelity {
        scenario.fidelity = Fidelity::Locked(level);
    }
    Ok(scenario)
}

fn read(path: &Path) -> Result<String, GameError> {
    std::fs::read_to_string(path).map_err(|source| GameError::Io {
        path: path.display().to_string(),
        source,
    })
}

/// Replays a log on the challenge's scenario to its last day and scores it: the
/// reproducible score of design plan §6.4. The world returned is the first season's,
/// on the scenario's own seed; the score is the mean over [`Challenge::seasons`].
///
/// # Errors
/// [`GameError`] when the scenario cannot be loaded or run.
pub fn play(data_dir: &Path, c: &Challenge, log: &[Action]) -> Result<(World, Outcome), GameError> {
    let scenario = load_scenario(data_dir, c)?;
    let catalog = catalog_for(data_dir, c, &scenario, log)?;
    let w = replay_season(&scenario, &catalog, log, 0)?;
    let more = other_seasons(c, &scenario, &catalog, log)?;
    let o = c.judge(w.state()).with_seasons(c, &more);
    Ok((w, o))
}

/// The scores of seasons 2 and on, for a run whose first season is already scored
/// (the viewer plays the first on screen).
///
/// # Errors
/// [`GameError`] when the scenario cannot be loaded or run.
pub fn score_other_seasons(
    data_dir: &Path,
    c: &Challenge,
    log: &[Action],
) -> Result<Vec<f64>, GameError> {
    let scenario = load_scenario(data_dir, c)?;
    let catalog = catalog_for(data_dir, c, &scenario, log)?;
    other_seasons(c, &scenario, &catalog, log)
}

/// The scenario of season `k`: the world seed moved up by `k`, which redraws the
/// weather, the field and every chance in the plant (pod set among them).
#[must_use]
pub fn season_scenario(scenario: &Scenario, k: u32) -> Scenario {
    let mut s = scenario.clone();
    s.world_seed = s.world_seed.wrapping_add(u64::from(k));
    s
}

/// The log for season `k`. The plan carries over; actions on one particular organ
/// (pick this pod, prune that shoot) do not, because another season grows other
/// organs. Season 0 keeps everything.
#[must_use]
pub fn season_log(log: &[Action], k: u32) -> Vec<Action> {
    log.iter()
        .filter(|a| {
            k == 0
                || !matches!(
                    a.kind,
                    ActionKind::Prune { .. }
                        | ActionKind::Harvest { .. }
                        | ActionKind::Pollinate { .. }
                )
        })
        .cloned()
        .collect()
}

fn replay_season(
    scenario: &Scenario,
    catalog: &Catalog,
    log: &[Action],
    k: u32,
) -> Result<World, GameError> {
    let end = u64::from(scenario.duration_days) * 24;
    Ok(World::replay(
        season_scenario(scenario, k),
        catalog.clone(),
        &season_log(log, k),
        end,
    )?)
}

/// Seasons 1.. scored, one thread each, gathered in season order.
fn other_seasons(
    c: &Challenge,
    scenario: &Scenario,
    catalog: &Catalog,
    log: &[Action],
) -> Result<Vec<f64>, GameError> {
    let runs: Vec<Result<f64, GameError>> = std::thread::scope(|s| {
        let handles: Vec<_> = (1..c.seasons)
            .map(|k| {
                s.spawn(move || {
                    replay_season(scenario, catalog, log, k).map(|w| c.measure(w.state()))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or(Err(GameError::Panicked)))
            .collect()
    });
    runs.into_iter().collect()
}

/// The catalog a challenge run needs: the scenario's, plus the challenge's sowing
/// species and any species the log sows.
///
/// # Errors
/// [`GameError`] when a file is missing or bad.
pub fn catalog_for(
    data_dir: &Path,
    c: &Challenge,
    scenario: &Scenario,
    log: &[Action],
) -> Result<Catalog, GameError> {
    let mut catalog = Catalog::load(data_dir, scenario)?;
    if let Some(id) = &c.rules.sow_species {
        add_species(data_dir, &mut catalog, id)?;
    }
    add_sown_species(data_dir, &mut catalog, log)?;
    Ok(catalog)
}

fn add_species(data_dir: &Path, catalog: &mut Catalog, id: &str) -> Result<(), GameError> {
    if catalog.species.contains_key(id) {
        return Ok(());
    }
    let path = data_dir.join("species").join(format!("{id}.ron"));
    let sp = ps_species::Species::from_ron(&read(&path)?)
        .map_err(|e| GameError::World(ps_world::WorldError::Species(format!("{id}: {e}"))))?;
    catalog.species.insert(id.to_owned(), sp);
    Ok(())
}

/// Adds the species the log sows to a catalog (a challenge scenario may start empty).
///
/// # Errors
/// [`GameError`] when a species file is missing or bad.
pub fn add_sown_species(
    data_dir: &Path,
    catalog: &mut Catalog,
    log: &[Action],
) -> Result<(), GameError> {
    for a in log {
        if let ActionKind::Sow(s) = &a.kind {
            add_species(data_dir, catalog, &s.species)?;
        }
    }
    Ok(())
}

/// Reads `data/game/debrief.ron`, or the built-in guesses when it is missing.
///
/// # Errors
/// [`GameError`] when the file exists but does not parse.
pub fn load_debrief_refs(data_dir: &Path) -> Result<DebriefRefs, GameError> {
    let path = data_dir.join("game").join("debrief.ron");
    if !path.exists() {
        return Ok(DebriefRefs::default());
    }
    Ok(ron::from_str(&read(&path)?)?)
}

/// Resources the player's actions added.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Usage {
    /// Water by can, kg.
    pub water: KgWater,
    /// Mineral nitrogen, g N.
    pub nitrogen: GramsN,
}

/// A soil's surface area by id.
#[must_use]
pub fn soil_area(scenario: &Scenario, soil: u32) -> Option<M2> {
    scenario
        .soils
        .iter()
        .find(|s| s.id() == soil)
        .map(|s| match s {
            SoilSpec::Pot { diameter_cm, .. } => {
                let r = diameter_cm / 200.0;
                M2(std::f64::consts::PI * r * r)
            }
            SoilSpec::Plot {
                width_cm,
                length_cm,
                ..
            } => M2(width_cm * length_cm / 1e4),
        })
}

/// Water and nitrogen the log adds. Water mm over a soil's area is kg (1 mm on 1 m² is
/// 1 kg).
#[must_use]
pub fn usage(scenario: &Scenario, log: &[Action]) -> Usage {
    let mut water = 0.0;
    let mut nitrogen = 0.0;
    for a in log {
        match &a.kind {
            ActionKind::Water { soil, mm } => {
                water += mm * soil_area(scenario, *soil).map_or(0.0, |a| a.0);
            }
            ActionKind::Fertilize { soil, g_per_m2, .. } => {
                nitrogen += g_per_m2 * soil_area(scenario, *soil).map_or(0.0, |a| a.0);
            }
            _ => {}
        }
    }
    Usage {
        water: KgWater(water),
        nitrogen: GramsN(nitrogen),
    }
}

/// A metric over a daily table, plants in id order.
#[must_use]
pub fn metric(m: &Metric, history: &[DayRow]) -> f64 {
    let (col, peak) = match m {
        Metric::PeakSum(c) => (c.as_str(), true),
        Metric::FinalSum(c) => (c.as_str(), false),
    };
    // Per plant: running peak, or the value on its latest row. BTreeMap keeps the
    // summation order fixed.
    let mut per: std::collections::BTreeMap<u32, f64> = std::collections::BTreeMap::new();
    for row in history {
        let Some(v) = column(row, col) else { continue };
        let e = per.entry(row.plant).or_insert(v);
        *e = if peak { e.max(v) } else { v };
    }
    // `+ 0.0` turns an empty sum's −0.0 into 0.
    per.values().sum::<f64>() + 0.0
}

/// The columns a challenge can score, by daily-table name (the same names as the
/// CSV export).
#[must_use]
pub fn column(row: &DayRow, name: &str) -> Option<f64> {
    let r = &row.report;
    Some(match name {
        "pods" => f64::from(r.pods),
        "seeds" => f64::from(r.seeds),
        "flowers" => f64::from(r.flowers),
        "leaves" => f64::from(r.leaves),
        "leaf_area_m2" => r.leaf_area,
        "seed_mass_g" => r.seed_mass,
        "fruit_mass_g" => r.fruit_mass,
        "dry_mass_g" => r.dry_mass,
        "total_dry_mass_g" => r.total_dry_mass,
        "height_m" => r.height,
        "root_length_m" => r.root_length,
        "transpiration_kg" => r.transpiration,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn challenge(higher: bool, medals: Medals) -> Challenge {
        Challenge {
            schema: CHALLENGE_SCHEMA,
            id: "t".into(),
            title: String::new(),
            brief: String::new(),
            hint: String::new(),
            scenario: String::new(),
            score: ScoreSpec {
                metric: Metric::PeakSum("pods".into()),
                label: "pods".into(),
                higher_is_better: higher,
            },
            rules: Rules::default(),
            medals,
            seasons: 1,
            fidelity: None,
        }
    }

    /// R3c (decision 0045): the viewer's level switches are never the player's rule
    /// breaks, and a challenge's lock reaches its scenario.
    #[test]
    fn fidelity_is_not_a_rule_and_a_lock_reaches_the_scenario() {
        let mut c = challenge(
            true,
            Medals {
                bronze: 1.0,
                silver: 2.0,
                gold: 3.0,
            },
        );
        c.scenario = "single_pot".into();
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let scenario = load_scenario(&data, &c).expect("scenario");
        assert_eq!(scenario.fidelity, Fidelity::Auto);
        let log = [Action {
            tick: 30,
            kind: ActionKind::SetFidelity {
                plants: Vec::new(),
                level: HydraulicsLevel::Simplified,
            },
        }];
        let (breaks, _) = c.check_log(&scenario, &log);
        assert!(breaks.is_empty(), "{breaks:?}");
        c.fidelity = Some(ChallengeFidelity::Locked(HydraulicsLevel::Detailed));
        let locked = load_scenario(&data, &c).expect("scenario");
        assert_eq!(locked.fidelity, Fidelity::Locked(HydraulicsLevel::Detailed));
        let text = "Locked(Detailed)";
        let parsed: ChallengeFidelity = ron::from_str(text).expect("ron");
        assert_eq!(parsed, ChallengeFidelity::Locked(HydraulicsLevel::Detailed));
    }

    #[test]
    fn medals_follow_the_direction() {
        let c = challenge(
            true,
            Medals {
                bronze: 5.0,
                silver: 10.0,
                gold: 15.0,
            },
        );
        assert_eq!(c.medal_for(4.0), None);
        assert_eq!(c.medal_for(5.0), Some(Medal::Bronze));
        assert_eq!(c.medal_for(12.0), Some(Medal::Silver));
        assert_eq!(c.medal_for(15.0), Some(Medal::Gold));
        let low = challenge(
            false,
            Medals {
                bronze: 30.0,
                silver: 20.0,
                gold: 10.0,
            },
        );
        assert_eq!(low.medal_for(25.0), Some(Medal::Bronze));
        assert_eq!(low.medal_for(9.0), Some(Medal::Gold));
    }

    #[test]
    fn shipped_challenges_parse() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/challenges");
        let mut n = 0;
        for e in std::fs::read_dir(dir).expect("challenge dir") {
            let p = e.expect("entry").path();
            if p.extension().is_some_and(|x| x == "ron") {
                let text = std::fs::read_to_string(&p).expect("read");
                let c = Challenge::from_ron(&text).expect("parse");
                assert_eq!(Some(c.id.as_str()), p.file_stem().and_then(|s| s.to_str()));
                n += 1;
            }
        }
        assert!(n >= 3, "expected the first challenges, found {n}");
    }
}
