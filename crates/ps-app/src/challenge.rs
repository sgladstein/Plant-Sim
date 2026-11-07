//! Challenges on the app side (design plan §2.6, §6.4; WP32).
//!
//! - The list comes from `data/challenges/*.ron` through `ps_game`.
//! - Start turns the player's plan (sowing day, fertilizer doses) into actions scheduled
//!   on the world before the run starts, so they sit in the action log like any other
//!   action and the replay scores them.
//! - While the run goes, the live panel reads the run on screen once per sim day: the
//!   score so far, the budget left, any broken rule.
//! - On the last day, `Challenge::judge` scores the run on screen and `ps_game::debrief`
//!   explains it. A challenge is scored over several seasons (world seeds), so a
//!   background thread replays the log on the others; the card shows the mean when they
//!   are done, and only then is the medal given and the best kept. The best run of each challenge, with its action log, is kept in
//!   `<captures>/challenge_scores.ron`, so the score can be replayed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use bevy::prelude::{Commands, Res, ResMut, Resource};
use ps_game::{Challenge, DebriefRefs, Factor, Medal, Metric, Outcome};
use ps_ui::{
    ChallengeResult, ChallengeRow, Challenges, Charts, Experiment, Inspector, Live, Plan,
    PlantStory, WeekCell,
};
use ps_view::Selection;
use ps_world::{Action, ActionKind, SeedChoice, Sowing, World, WorldState};
use serde::{Deserialize, Serialize};

use crate::capture::{CaptureConfig, state_on_screen};
use crate::sim_source::SimSource;
use crate::tools::RunSettings;

/// Hour of the day planned actions apply.
const PLAN_HOUR: u64 = 8;

/// The other seasons' scores, or why they could not be run.
type SeasonScores = Result<Vec<f64>, String>;

/// The challenge being played.
#[derive(Clone, Debug, Resource)]
pub struct ActiveChallenge {
    challenge: Challenge,
    refs: DebriefRefs,
    /// (tick, days of history) last read, so the live panel updates once a sim day.
    seen: (u64, usize),
    finished: bool,
    /// The first season's outcome and log, waiting for the other seasons.
    first: Option<(Outcome, Vec<Action>)>,
    /// Filled by the thread scoring the other seasons.
    others: Arc<Mutex<Option<SeasonScores>>>,
}

/// The best run of one challenge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Best {
    /// Score.
    pub score: f64,
    /// Medal.
    pub medal: Option<Medal>,
    /// Model version it was played on.
    pub model_version: String,
    /// The run's action log: replaying it on the challenge gives the score.
    pub log: Vec<Action>,
}

fn scores_path(captures: &Path) -> PathBuf {
    captures.join("challenge_scores.ron")
}

fn read_scores(captures: &Path) -> BTreeMap<String, Best> {
    std::fs::read_to_string(scores_path(captures))
        .ok()
        .and_then(|t| ron::from_str(&t).ok())
        .unwrap_or_default()
}

fn fmt_score(c: &Challenge, v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{v:.0} {}", c.score.label)
    } else {
        format!("{v:.2} {}", c.score.label)
    }
}

fn medal_name(m: Medal) -> &'static str {
    match m {
        Medal::Bronze => "Bronze",
        Medal::Silver => "Silver",
        Medal::Gold => "Gold",
    }
}

fn medal_color(m: Option<Medal>) -> [u8; 3] {
    match m {
        Some(Medal::Gold) => [232, 196, 80],
        Some(Medal::Silver) => [200, 204, 210],
        Some(Medal::Bronze) => [205, 140, 90],
        None => [150, 145, 135],
    }
}

/// Debrief colors, one per factor.
#[must_use]
pub fn factor_color(f: Factor) -> [u8; 3] {
    match f {
        Factor::Water => [86, 150, 224],
        Factor::Heat => [228, 96, 70],
        Factor::Cold => [168, 150, 230],
        Factor::Light => [232, 200, 90],
        Factor::Nitrogen => [150, 190, 80],
        Factor::Sinks => [170, 140, 110],
    }
}

fn rules_text(c: &Challenge) -> String {
    let r = &c.rules;
    let mut parts: Vec<String> = Vec::new();
    let allowed: Vec<String> = r
        .allowed
        .iter()
        .map(|a| format!("{a:?}").to_lowercase())
        .collect();
    parts.push(format!("Allowed: {}.", allowed.join(", ")));
    if let Some(w) = r.water_budget {
        parts.push(format!("Water budget {:.0} L.", w.0));
    }
    if let Some(n) = r.nitrogen_budget {
        parts.push(format!("Nitrogen budget {} g.", n.0));
    }
    if let Some((a, b)) = r.sow_window_days {
        parts.push(format!("Sow between day {a} and {b}."));
    }
    parts.join(" ")
}

/// The challenge list with each one's best.
#[must_use]
pub fn load_rows(data: &Path, captures: &Path) -> Vec<ChallengeRow> {
    let best = read_scores(captures);
    let mut files: Vec<PathBuf> = std::fs::read_dir(data.join("challenges"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "ron"))
        .collect();
    files.sort();
    files
        .iter()
        .filter_map(|p| {
            let c = std::fs::read_to_string(p)
                .map_err(anyhow::Error::from)
                .and_then(|t| Challenge::from_ron(&t).map_err(anyhow::Error::from));
            match c {
                Ok(c) => Some(c),
                Err(e) => {
                    bevy::log::warn!("skipping challenge {}: {e:#}", p.display());
                    None
                }
            }
        })
        .map(|c| {
            let m = c.medals;
            ChallengeRow {
                id: c.id.clone(),
                title: c.title.clone(),
                brief: c.brief.clone(),
                hint: c.hint.clone(),
                rules: rules_text(&c),
                medals: format!(
                    "bronze {}, silver {}, gold {}",
                    fmt_score(&c, m.bronze),
                    fmt_score(&c, m.silver),
                    fmt_score(&c, m.gold)
                ),
                sow_window: c.rules.sow_window_days,
                nitrogen_budget: c
                    .rules
                    .nitrogen_budget
                    .filter(|_| c.rules.allowed.contains(&ps_game::ActionType::Fertilize))
                    .map(|n| n.0),
                best: best.get(&c.id).map(|b| {
                    let medal = b
                        .medal
                        .map_or(String::new(), |m| format!(", {}", medal_name(m)));
                    format!("{}{medal}", fmt_score(&c, b.score))
                }),
            }
        })
        .collect()
}

/// The plan as actions: the sowing and the fertilizer doses.
fn planned(c: &Challenge, scenario: &ps_world::Scenario, plan: &Plan) -> Vec<Action> {
    let mut out = Vec::new();
    if c.rules.sow_window_days.is_some() {
        out.push(Action {
            tick: u64::from(plan.sow_day) * 24 + PLAN_HOUR,
            kind: ActionKind::Sow(Sowing {
                soil: scenario.soils.first().map_or(0, ps_world::SoilSpec::id),
                species: c
                    .rules
                    .sow_species
                    .clone()
                    .unwrap_or_else(|| "bush_bean".into()),
                seed: SeedChoice::Default,
                depth_cm: 3.0,
                offset_cm: None,
            }),
        });
    }
    if c.rules.allowed.contains(&ps_game::ActionType::Fertilize) {
        let soil = scenario.soils.first().map_or(0, ps_world::SoilSpec::id);
        let area = ps_game::soil_area(scenario, soil).map_or(1.0, |a| a.0);
        for d in plan.doses.iter().filter(|d| d.grams > 0.0) {
            out.push(Action {
                tick: u64::from(d.day) * 24 + PLAN_HOUR,
                kind: ActionKind::Fertilize {
                    soil,
                    form: ps_soil::nitrogen::NitrogenForm::Nitrate,
                    g_per_m2: d.grams / area,
                },
            });
        }
    }
    out.sort_by_key(|a| a.tick);
    out
}

/// Builds the challenge's world with the plan scheduled and starts it.
///
/// # Errors
/// When a file is missing or bad, or the plan does not fit the world.
pub fn launch(
    data: &Path,
    id: &str,
    plan: &Plan,
    start_day: f64,
    speed: f64,
    paused: bool,
) -> Result<(SimSource, ActiveChallenge)> {
    let c = ps_game::load_challenge(data, id)?;
    let scenario = ps_game::load_scenario(data, &c)?;
    let actions = planned(&c, &scenario, plan);
    let catalog = ps_game::catalog_for(data, &c, &scenario, &actions)?;
    let mut world = World::new(scenario.clone(), catalog).context("building the world")?;
    for a in actions {
        world.schedule(a).context("scheduling the plan")?;
    }
    let src = SimSource::start(world, &scenario, start_day, speed, paused)?;
    let refs = ps_game::load_debrief_refs(data)?;
    Ok((
        src,
        ActiveChallenge {
            challenge: c,
            refs,
            seen: (u64::MAX, usize::MAX),
            finished: false,
            first: None,
            others: Arc::default(),
        },
    ))
}

fn live(c: &Challenge, state: &WorldState) -> Live {
    let (breaks, usage) = c.check_log(&state.scenario, &state.log);
    let score = c.measure(state);
    let next = [
        (Medal::Bronze, c.medals.bronze),
        (Medal::Silver, c.medals.silver),
        (Medal::Gold, c.medals.gold),
    ]
    .into_iter()
    .find(|(m, _)| c.medal_for(score) < Some(*m));
    let next_medal = match next {
        Some((m, t)) => format!("Next: {} at {}", medal_name(m), fmt_score(c, t)),
        None => "On course for gold".into(),
    };
    let next_medal = if matches!(c.score.metric, Metric::PeakSum(_)) {
        next_medal
    } else {
        format!("{next_medal} (scored on the last day)")
    };
    let next_medal = if c.seasons > 1 {
        format!(
            "{next_medal}. The medal goes to your plan's mean over {} seasons.",
            c.seasons
        )
    } else {
        next_medal
    };
    let mut budgets = Vec::new();
    if let Some(b) = c.rules.water_budget {
        let soil = state
            .scenario
            .soils
            .first()
            .map_or(0, ps_world::SoilSpec::id);
        let area = ps_game::soil_area(&state.scenario, soil).map_or(0.0, |a| a.0);
        budgets.push(format!(
            "Water left: {:.2} of {:.0} L (1 mm on this pot is {:.3} L)",
            (b.0 - usage.water.0).max(0.0),
            b.0,
            area
        ));
    }
    if let Some(b) = c.rules.nitrogen_budget {
        budgets.push(format!(
            "Nitrogen used: {:.2} of {} g",
            usage.nitrogen.0, b.0
        ));
    }
    let day = state.history.last().map_or(0, |r| r.day);
    Live {
        id: c.id.clone(),
        title: c.title.clone(),
        brief: c.brief.clone(),
        hint: c.hint.clone(),
        score: if c.seasons > 1 {
            format!("This season so far: {}", fmt_score(c, score))
        } else {
            format!("So far: {}", fmt_score(c, score))
        },
        next_medal,
        budgets,
        breaks: breaks
            .iter()
            .map(|b| format!("Rule broken, no medal: {b}"))
            .collect(),
        day: (day, state.scenario.duration_days.saturating_sub(1)),
    }
}

fn story(snap: &ps_session::Snapshot, p: &ps_game::PlantDebrief, notable: f64) -> PlantStory {
    let weeks = p
        .weeks
        .iter()
        .map(|w| {
            let event = ps_game::week_event(w, &p.backdrop, notable);
            let numbers: Vec<String> = Factor::ALL
                .iter()
                .zip(w.pressure)
                .map(|(f, v)| format!("{} {:.0}%", f.name(), v * 100.0))
                .collect();
            let stage = w.stage.map_or_else(String::new, |s| format!(", {s:?}"));
            #[allow(clippy::cast_possible_truncation)]
            let pressure = event.map_or(0.0, |(_, v)| v as f32);
            WeekCell {
                week: w.week,
                top: event.map(|(f, _)| f.name().to_owned()),
                pressure,
                color: event.map_or([60, 56, 50], |(f, _)| factor_color(f)),
                hover: format!(
                    "Week {} (days {} to {}{stage}): {}, pods {:+}, dry mass {:+.1} g\n{}",
                    w.week,
                    w.days.0,
                    w.days.1,
                    event.map_or("nothing out of the ordinary", |(f, _)| f.name()),
                    w.pods_gained,
                    w.mass_gained_g,
                    numbers.join(", ")
                ),
            }
        })
        .collect();
    PlantStory {
        name: crate::inspect::plant_name(snap, p.plant),
        weeks,
        summary: p.summary.clone(),
    }
}

/// The result card for the run on screen: its debrief now, the score once the other
/// seasons are in ([`settle`]).
fn finish(
    a: &ActiveChallenge,
    o: &Outcome,
    state: &WorldState,
    snap: &ps_session::Snapshot,
) -> ChallengeResult {
    let c = &a.challenge;
    let d = ps_game::debrief(state, &a.refs);
    let mut legend: Vec<(String, [u8; 3])> = Factor::PRIMARY
        .iter()
        .map(|f| (f.name().to_owned(), factor_color(*f)))
        .collect();
    legend.dedup();
    let mut r = ChallengeResult {
        title: c.title.clone(),
        breaks: o.breaks.iter().map(ToString::to_string).collect(),
        plants: d
            .plants
            .iter()
            .map(|p| story(snap, p, a.refs.notable_pressure.value))
            .collect(),
        legend,
        ..ChallengeResult::default()
    };
    if c.seasons > 1 {
        r.score = format!(
            "This season {}; scoring your plan over {} seasons…",
            fmt_score(c, o.score),
            c.seasons
        );
        r.medal_color = medal_color(None);
    }
    r
}

/// Fills the score, the medal and the best on the result card from the full outcome,
/// and keeps the best run when `save`.
fn settle(c: &Challenge, o: &Outcome, log: &[Action], captures: &Path, r: &mut ChallengeResult) {
    if o.season_scores.len() > 1 {
        let each: Vec<String> = o.season_scores.iter().map(|v| fmt_score(c, *v)).collect();
        r.seasons = format!(
            "Mean of {} seasons, each a new draw of weather and chance: {}. The season you \
             watched is the first; the debrief below is that season. Harvests and pruning count \
             only in that one; the plan (sowing, water, fertilizer) runs in all.",
            o.season_scores.len(),
            each.join(", ")
        );
    }
    r.score = fmt_score(c, o.score);
    r.medal = o.medal.map(|m| medal_name(m).to_owned());
    r.medal_color = medal_color(o.medal);
    let mut scores = read_scores(captures);
    let better = |old: &Best| {
        let ranked = o.breaks.is_empty();
        ranked
            && (old.medal.is_none() && o.medal.is_some()
                || if c.score.higher_is_better {
                    o.score > old.score
                } else {
                    o.score < old.score
                })
    };
    r.new_best = o.breaks.is_empty() && scores.get(&c.id).is_none_or(better);
    if r.new_best {
        scores.insert(
            c.id.clone(),
            Best {
                score: o.score,
                medal: o.medal,
                model_version: o.model_version.clone(),
                log: log.to_vec(),
            },
        );
        let path = scores_path(captures);
        let write = std::fs::create_dir_all(captures)
            .map_err(anyhow::Error::from)
            .and_then(|()| {
                ron::ser::to_string_pretty(&scores, ron::ser::PrettyConfig::default())
                    .map_err(anyhow::Error::from)
            })
            .and_then(|t| std::fs::write(&path, t).map_err(anyhow::Error::from));
        r.saved = match write {
            Ok(()) => format!("Best run and its action log saved to {}", path.display()),
            Err(e) => format!("Could not save the best run: {e:#}"),
        };
    }
}

/// Starts scoring seasons 2 and on in the background.
fn score_others(data: &Path, a: &ActiveChallenge, log: &[Action]) {
    let (data, c, log, slot) = (
        data.to_path_buf(),
        a.challenge.clone(),
        log.to_vec(),
        a.others.clone(),
    );
    std::thread::spawn(move || {
        let r = ps_game::score_other_seasons(&data, &c, &log).map_err(|e| format!("{e:#}"));
        if let Ok(mut s) = slot.lock() {
            *s = Some(r);
        }
    });
}

/// Starts challenges, keeps the live panel current and scores the last day.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn challenge(
    mut commands: Commands,
    settings: Res<RunSettings>,
    config: Res<CaptureConfig>,
    src: Option<Res<SimSource>>,
    active: Option<ResMut<ActiveChallenge>>,
    mut ui: ResMut<Challenges>,
    mut ex: ResMut<Experiment>,
    mut selection: ResMut<Selection>,
    mut inspector: ResMut<Inspector>,
    mut charts: ResMut<Charts>,
    mut tools: ResMut<ps_ui::Tools>,
) {
    if ui.rows.is_empty() {
        ui.rows = load_rows(&settings.data, &config.dir);
    }
    if ui.start || ui.retry {
        ui.start = false;
        ui.retry = false;
        let Some(row) = ui.chosen.and_then(|i| ui.rows.get(i)).cloned() else {
            ui.status = "Choose a challenge first.".into();
            return;
        };
        let start_day = std::mem::take(&mut ui.start_day);
        match launch(
            &settings.data,
            &row.id,
            &ui.plan,
            start_day,
            settings.speed,
            start_day > 0.0,
        ) {
            Ok((src, active)) => {
                commands.insert_resource(src);
                commands.insert_resource(active);
                commands.remove_resource::<crate::experiment::OpenNotebook>();
                ex.running = None;
                ui.status = String::new();
                ui.result = None;
                ui.result_open = false;
                ui.live = Some(Live {
                    title: row.title.clone(),
                    brief: row.brief.clone(),
                    hint: row.hint.clone(),
                    score: "Starting…".into(),
                    ..Live::default()
                });
                selection.picked = None;
                *inspector = Inspector::default();
                charts.rows = usize::MAX;
                tools.seed_library.clear();
                tools.challenges = false;
            }
            Err(e) => {
                tools.challenges = true;
                ui.status = format!("Could not start: {e:#}");
            }
        }
        return;
    }
    let (Some(src), Some(mut a)) = (src, active) else {
        return;
    };
    if a.first.is_some() {
        let done = a.others.lock().ok().and_then(|mut s| s.take());
        if let Some(done) = done
            && let Some((o, log)) = a.first.take()
            && let Some(r) = ui.result.as_mut()
        {
            match done {
                Ok(more) => {
                    let o = o.with_seasons(&a.challenge, &more);
                    settle(&a.challenge, &o, &log, &config.dir, r);
                    ui.rows = load_rows(&settings.data, &config.dir);
                }
                Err(e) => r.score = format!("Could not score the other seasons: {e}"),
            }
        }
    }
    let snap = src.snapshot();
    let seen = (snap.tick, snap.history.len());
    let at_end = snap.tick >= src.end_tick();
    let score_now = at_end && !a.finished;
    if seen.1 == a.seen.1 && !score_now {
        return;
    }
    a.seen = seen;
    let state = state_on_screen(&src);
    ui.live = Some(live(&a.challenge, &state));
    if score_now {
        a.finished = true;
        let o = a.challenge.judge(&state);
        ui.result = Some(finish(&a, &o, &state, snap));
        ui.result_open = true;
        if a.challenge.seasons > 1 {
            score_others(&settings.data, &a, &state.log);
            a.first = Some((o, state.log.clone()));
        } else if let Some(r) = ui.result.as_mut() {
            settle(&a.challenge, &o, &state.log, &config.dir, r);
            ui.rows = load_rows(&settings.data, &config.dir);
        }
    }
}

/// A new run or an experiment ends the challenge.
#[allow(clippy::needless_pass_by_value)]
pub fn end_on_new_run(
    mut commands: Commands,
    tools: Res<ps_ui::Tools>,
    ex: Res<Experiment>,
    active: Option<Res<ActiveChallenge>>,
    mut ui: ResMut<Challenges>,
) {
    if active.is_some() && (tools.new_run.start || ex.start) {
        commands.remove_resource::<ActiveChallenge>();
        ui.live = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> PathBuf {
        crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"))
    }

    #[test]
    fn every_challenge_lists_and_its_plan_schedules() {
        let data = data();
        let dir = std::env::temp_dir().join(format!("ps-ch-{}", std::process::id()));
        let rows = load_rows(&data, &dir);
        assert!(rows.len() >= 3);
        for row in &rows {
            let c = ps_game::load_challenge(&data, &row.id).unwrap_or_else(|e| unreachable!("{e}"));
            let sc = ps_game::load_scenario(&data, &c).unwrap_or_else(|e| unreachable!("{e}"));
            let mut ui = Challenges {
                rows: rows.clone(),
                ..Challenges::default()
            };
            ui.choose(rows.iter().position(|r| r.id == row.id).unwrap_or(0));
            let actions = planned(&c, &sc, &ui.plan);
            let (breaks, _) = c.check_log(&sc, &actions);
            assert!(
                breaks.is_empty(),
                "{}: the default plan breaks {breaks:?}",
                row.id
            );
            let catalog = ps_game::catalog_for(&data, &c, &sc, &actions)
                .unwrap_or_else(|e| unreachable!("{e}"));
            let mut w = World::new(sc, catalog).unwrap_or_else(|e| unreachable!("{e}"));
            for a in actions {
                w.schedule(a).unwrap_or_else(|e| unreachable!("{e}"));
            }
        }
    }
}
