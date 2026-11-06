//! The run debrief: week by week, what held each plant back.
//!
//! Each pressure is a 0-to-1 shortfall read from the daily table against the species'
//! own thresholds, so the debrief says what the model did, not a separate opinion:
//!
//! - **Water:** one minus the leaves' predawn turgor (R3b, decision 0044): how much
//!   water stress the plant carries through the night, which slows its stomata and its
//!   growth the next day; at least the share of the day's transpiration demand the
//!   soil could not supply, `shortfall / (transpiration + shortfall)`; and while
//!   flowers are open, at least `1 − FTSW / drought_abortion_ftsw`, the soil-water
//!   deficit that aborts flowers in the model (read at the end of the day, where the
//!   model takes the day's minimum, so a little milder). A midday turgor dip in wet
//!   soil is not a pressure: the plant grows it back that night.
//! - **Heat** and **cold:** while any flower is open, one minus the model's pod-set
//!   response at the day's peak temperature (the quadratic ramps of ps-arch
//!   `pod_set_temperature`), since that is what costs beans their pods. Otherwise one
//!   minus the growth trapezoid at the day's mean of `t_min` and `t_max` (the model
//!   applies it hourly, so the mean is an approximation).
//! - **Light:** one minus the day's radiation at the soil over a full-sun reference
//!   (data/game/debrief.ron), times the share of photosynthesis the sink feedback
//!   leaves (light cannot limit a plant with nowhere to put the sugar). Shade cast by
//!   neighbours is not in this number.
//! - **Nitrogen:** one minus the plant's nitrogen factor on expansion.
//! - **Sinks:** the share of potential photosynthesis lost because the plant had
//!   nowhere to put the sugar (the sink feedback, decision 0013).
//!
//! Sinks are a symptom more than a cause (a plant whose pods aborted in a heat wave
//! has nowhere to put its sugar), so a week's top factor and the season ranking use the
//! five environmental factors; sinks stay in the weekly numbers only. The
//! season ranks factors that hit in a bloom week first, by their bloom-week peak, then
//! the rest by their worst week, since one bad week in bloom can cost a season's pods.
//! Bloom is a flower opened within the last flower lifespan, and the debrief stops at
//! maturity.

use ps_physio::phenology::Stage;
use ps_species::Param;
use ps_world::{DayRow, WorldState};
use serde::{Deserialize, Serialize};

/// Reference values for the debrief, from data/game/debrief.ron.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "DebriefRefs")]
pub struct DebriefRefs {
    /// Daily global radiation of a clear summer day, MJ m⁻² d⁻¹.
    pub full_sun_radiation: Param,
    /// Weekly mean pressure above which a factor is named as holding the plant back.
    pub notable_pressure: Param,
}

impl Default for DebriefRefs {
    fn default() -> Self {
        let p = |value: f64, unit: &str| Param {
            value,
            unit: unit.into(),
            source: "~".into(),
        };
        Self {
            full_sun_radiation: p(25.0, "MJ/m2/d"),
            notable_pressure: p(0.15, "-"),
        }
    }
}

/// What can hold a plant back.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Factor {
    /// Roots could not meet transpiration demand.
    Water,
    /// Too hot for growth or pod set.
    Heat,
    /// Too cold for growth or pod set.
    Cold,
    /// Too little light at the soil.
    Light,
    /// Short of nitrogen.
    Nitrogen,
    /// More sugar than places to put it.
    Sinks,
}

impl Factor {
    /// The environmental factors, which can be a week's top factor.
    pub const PRIMARY: [Self; 5] = [
        Self::Water,
        Self::Heat,
        Self::Cold,
        Self::Light,
        Self::Nitrogen,
    ];

    /// All factors, in display order.
    pub const ALL: [Self; 6] = [
        Self::Water,
        Self::Heat,
        Self::Cold,
        Self::Light,
        Self::Nitrogen,
        Self::Sinks,
    ];

    /// A short plain-language name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Water => "water",
            Self::Heat => "heat",
            Self::Cold => "cool weather",
            Self::Light => "light",
            Self::Nitrogen => "nitrogen",
            Self::Sinks => "too few sinks",
        }
    }

    /// One sentence on what it means for the plant.
    #[must_use]
    pub fn explain(self) -> &'static str {
        match self {
            Self::Water => {
                "The roots could not keep up with the leaves; stomata closed and flowers dropped."
            }
            Self::Heat => {
                "Too hot: growth slowed, and in bloom, pollen fails and young pods abort."
            }
            Self::Cold => "Below the temperatures beans like best: growth and development slowed.",
            Self::Light => "Too little light reached the plant to make the sugar it could use.",
            Self::Nitrogen => "Short of nitrogen: leaves grew smaller and paler.",
            Self::Sinks => "the leaves made more sugar than the plant could use.",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// One week of one plant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeekDebrief {
    /// Week number from emergence, from 1.
    pub week: u32,
    /// First and last scenario day of the week.
    pub days: (u32, u32),
    /// Most advanced stage reached in the week.
    pub stage: Option<Stage>,
    /// Whether any flower was open during the week.
    pub bloom: bool,
    /// Mean pressure of each factor, in [`Factor::ALL`] order, 0 to 1.
    pub pressure: [f64; 6],
    /// The factor that held the plant back most, if any passed the notable threshold.
    pub top: Option<Factor>,
    /// Pods gained over the week (negative when pods aborted or were picked).
    pub pods_gained: i64,
    /// Dry mass gained, g.
    pub mass_gained_g: f64,
}

/// One plant's season.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlantDebrief {
    /// Plant id.
    pub plant: u32,
    /// Species id.
    pub species: String,
    /// Weeks since emergence.
    pub weeks: Vec<WeekDebrief>,
    /// Season mean pressure per factor, in [`Factor::ALL`] order.
    pub season: [f64; 6],
    /// Worst week's pressure per factor, in [`Factor::ALL`] order.
    pub peak: [f64; 6],
    /// Factors notable in most weeks: the season's setting rather than an event.
    pub backdrop: Vec<Factor>,
    /// Environmental factors that held it back, by worst week, notable ones only.
    pub ranked: Vec<Factor>,
    /// A few sentences for the debrief screen.
    pub summary: Vec<String>,
}

/// A run's debrief.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Debrief {
    /// One entry per plant that emerged, by id.
    pub plants: Vec<PlantDebrief>,
}

struct Temps {
    growth: [f64; 4],
    pod_set: [f64; 4],
    /// FTSW below which flowers abort.
    abort_ftsw: f64,
    /// Days a flower stays open.
    flower_days: usize,
}

/// The model's pod-set temperature response (mirrors ps-arch
/// `Params::pod_set_temperature`): 1 on the plateau, `1 − x²` on each ramp, 0 beyond.
fn pod_set_response(t: [f64; 4], x: f64) -> f64 {
    let [lo, o1, o2, hi] = t;
    if x <= lo || x >= hi {
        0.0
    } else if x < o1 {
        let d = (o1 - x) / (o1 - lo);
        1.0 - d * d
    } else if x > o2 {
        let d = (x - o2) / (hi - o2);
        1.0 - d * d
    } else {
        1.0
    }
}

/// Trapezoid response: 0 at or beyond `t[0]` and `t[3]`, 1 between `t[1]` and `t[2]`.
fn trapezoid(t: [f64; 4], x: f64) -> f64 {
    if x <= t[0] || x >= t[3] {
        0.0
    } else if x < t[1] {
        (x - t[0]) / (t[1] - t[0])
    } else if x <= t[2] {
        1.0
    } else {
        (t[3] - x) / (t[3] - t[2])
    }
}

fn day_pressure(row: &DayRow, bloom: bool, temps: &Temps, full_sun: f64) -> [f64; 6] {
    let r = &row.report;
    let e = &row.env;
    let mut p = [0.0; 6];
    let demand = r.transpiration + r.shortfall;
    let mut water = if demand > 0.0 {
        r.shortfall / demand
    } else {
        0.0
    };
    water = water.max(1.0 - r.leaf_turgor_predawn);
    if bloom && temps.abort_ftsw > 0.0 {
        water = water.max(1.0 - e.ftsw / temps.abort_ftsw);
    }
    p[Factor::Water.index()] = water.clamp(0.0, 1.0);
    if bloom {
        // The model's pod-set response: the day's peak through quadratic ramps
        // (ps-arch `Params::pod_set_temperature`).
        let t = temps.pod_set;
        let lost = 1.0 - pod_set_response(t, e.t_max_c);
        if e.t_max_c > t[2] {
            p[Factor::Heat.index()] = lost;
        } else if e.t_max_c < t[1] {
            p[Factor::Cold.index()] = lost;
        }
    } else {
        let t = temps.growth;
        let mean = 0.5 * (e.t_min_c + e.t_max_c);
        let lost = 1.0 - trapezoid(t, mean);
        if mean > t[2] {
            p[Factor::Heat.index()] = lost;
        } else if mean < t[1] {
            p[Factor::Cold.index()] = lost;
        }
    }
    if full_sun > 0.0 {
        // Light only limits a plant that has somewhere to put the sugar: scale by the
        // share of photosynthesis the sink feedback leaves.
        let short = (1.0 - e.radiation_mj / full_sun).clamp(0.0, 1.0);
        p[Factor::Light.index()] = short * (1.0 - r.feedback_loss.clamp(0.0, 1.0));
    }
    p[Factor::Nitrogen.index()] = (1.0 - r.n_status).clamp(0.0, 1.0);
    p[Factor::Sinks.index()] = r.feedback_loss.clamp(0.0, 1.0);
    p
}

fn temps(state: &WorldState, species: &str) -> Option<Temps> {
    let c = &state.catalog.species.get(species)?.constants;
    Some(Temps {
        growth: [
            c.growth_t_min.value,
            c.growth_t_opt_low.value,
            c.growth_t_opt_high.value,
            c.growth_t_max.value,
        ],
        pod_set: [
            c.pod_set_t_min.value,
            c.pod_set_t_opt_low.value,
            c.pod_set_t_opt_high.value,
            c.pod_set_t_max.value,
        ],
        abort_ftsw: c.drought_abortion_ftsw.value,
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        flower_days: c.flower_lifespan.value.round().max(1.0) as usize,
    })
}

fn ranked(pressure: &[f64; 6], notable: f64) -> Vec<Factor> {
    let mut f: Vec<Factor> = Factor::PRIMARY
        .into_iter()
        .filter(|f| pressure[f.index()] >= notable)
        .collect();
    f.sort_by(|a, b| pressure[b.index()].total_cmp(&pressure[a.index()]));
    f
}

fn week_span(weeks: &[WeekDebrief], factor: Factor, notable: f64) -> Vec<(u32, u32)> {
    let mut spans: Vec<(u32, u32)> = Vec::new();
    for w in weeks
        .iter()
        .filter(|w| w.pressure[factor.index()] >= notable)
    {
        match spans.last_mut() {
            Some(s) if s.1 + 1 == w.week => s.1 = w.week,
            _ => spans.push((w.week, w.week)),
        }
    }
    spans
}

fn spans_text(spans: &[(u32, u32)]) -> String {
    spans
        .iter()
        .map(|(a, b)| {
            if a == b {
                format!("week {a}")
            } else {
                format!("weeks {a} to {b}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn weeks_over(weeks: &[WeekDebrief], f: Factor, notable: f64) -> usize {
    weeks
        .iter()
        .filter(|w| w.pressure[f.index()] >= notable)
        .count()
}

fn is_chronic(weeks: &[WeekDebrief], f: Factor, notable: f64) -> bool {
    #[allow(clippy::cast_precision_loss)]
    let over = weeks_over(weeks, f, notable) as f64;
    #[allow(clippy::cast_precision_loss)]
    let n = weeks.len() as f64;
    over >= CHRONIC_SHARE * n
}

/// A season-long factor that never hit while flowers were open: the setting, not an
/// event. One that hit in bloom is always an event, however long it lasted.
fn is_backdrop(weeks: &[WeekDebrief], bloom_peak: &[f64; 6], f: Factor, notable: f64) -> bool {
    bloom_peak[f.index()] < notable && is_chronic(weeks, f, notable)
}

/// Share of weeks above the notable pressure from which a factor is called a backdrop
/// of the whole season rather than an event.
const CHRONIC_SHARE: f64 = 0.6;

fn summary(
    weeks: &[WeekDebrief],
    peak: &[f64; 6],
    bloom_peak: &[f64; 6],
    ranked: &[Factor],
    notable: f64,
) -> Vec<String> {
    let mut out = Vec::new();
    let events: Vec<Factor> = ranked
        .iter()
        .copied()
        .filter(|f| !is_backdrop(weeks, bloom_peak, *f, notable))
        .collect();
    let backdrop: Vec<Factor> = ranked
        .iter()
        .copied()
        .filter(|f| is_backdrop(weeks, bloom_peak, *f, notable))
        .collect();
    if events.is_empty() {
        out.push(
            "No single week stood out: nothing in its surroundings hit this plant hard.".into(),
        );
    }
    let bloom_weeks: Vec<WeekDebrief> = weeks.iter().filter(|w| w.bloom).cloned().collect();
    for f in events.iter().take(3) {
        let i = f.index();
        // A factor that hit in bloom is told by its bloom weeks only.
        let (pct, spans, bloom) = if bloom_peak[i] >= notable {
            (
                bloom_peak[i],
                week_span(&bloom_weeks, *f, notable),
                ", during bloom",
            )
        } else {
            (peak[i], week_span(weeks, *f, notable), "")
        };
        let pct = (pct * 100.0).round();
        out.push(format!(
            "{}: up to {pct:.0}% in {}{bloom}. {}",
            capitalise(f.name()),
            spans_text(&spans),
            f.explain()
        ));
    }
    if !backdrop.is_empty() {
        let names: Vec<&str> = backdrop.iter().map(|f| f.name()).collect();
        out.push(format!(
            "All season, mildly: {}. That is the setting, not something you did.",
            names.join(" and ")
        ));
    }
    out
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}

/// A week's worst event: the top environmental factor above `notable` that is not part
/// of the season's backdrop.
#[must_use]
pub fn week_event(w: &WeekDebrief, backdrop: &[Factor], notable: f64) -> Option<(Factor, f64)> {
    Factor::PRIMARY
        .into_iter()
        .filter(|f| !backdrop.contains(f))
        .map(|f| (f, w.pressure[f as usize]))
        .filter(|(_, p)| *p >= notable)
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// The debrief of a run, from its daily table.
#[must_use]
#[allow(clippy::too_many_lines)] // One pass per plant: bloom days, weeks, ranking.
pub fn debrief(state: &WorldState, refs: &DebriefRefs) -> Debrief {
    let full_sun = refs.full_sun_radiation.value;
    let notable = refs.notable_pressure.value;
    let mut plants = Vec::new();
    for p in &state.plants {
        let Some(temps) = temps(state, &p.species) else {
            continue;
        };
        let all: Vec<&DayRow> = state.history.iter().filter(|r| r.plant == p.id).collect();
        // Bloom on a day: a flower opened within the last flower lifespan. The daily
        // `flowers` column counts flowers opened so far. The model sets pods for every
        // flower that closes, whatever the stage, so this is wider than Flowering and
        // PodSet.
        let open: Vec<bool> = (0..all.len())
            .map(|i| {
                let before = i
                    .checked_sub(temps.flower_days)
                    .map_or(0, |j| all[j].report.flowers);
                all[i].report.flowers > before
            })
            .collect();
        // From emergence to maturity: after maturity nothing more can be lost.
        let rows: Vec<(&DayRow, bool)> = all
            .iter()
            .zip(&open)
            .filter(|(r, _)| r.report.stage.is_some_and(|s| s >= Stage::Emerged))
            .take_while(|(r, _)| r.report.stage.is_some_and(|s| s < Stage::Mature))
            .map(|(r, b)| (*r, *b))
            .collect();
        if rows.is_empty() {
            continue;
        }
        let mut weeks: Vec<WeekDebrief> = Vec::new();
        let mut prev_pods = 0_i64;
        let mut prev_mass = rows[0].0.report.total_dry_mass;
        for (i, chunk) in rows.chunks(7).enumerate() {
            let mut sum = [0.0; 6];
            for (r, bloom) in chunk {
                let d = day_pressure(r, *bloom, &temps, full_sun);
                for (s, p) in sum.iter_mut().zip(d) {
                    *s += p;
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let n = chunk.len() as f64;
            let pressure = sum.map(|s| s / n);
            let last = chunk[chunk.len() - 1].0;
            let top = ranked(&pressure, notable).first().copied();
            let pods = i64::from(last.report.pods);
            weeks.push(WeekDebrief {
                week: u32::try_from(i + 1).unwrap_or(u32::MAX),
                days: (chunk[0].0.day, last.day),
                stage: chunk.iter().filter_map(|(r, _)| r.report.stage).max(),
                bloom: chunk.iter().any(|(_, b)| *b),
                pressure,
                top,
                pods_gained: pods - prev_pods,
                mass_gained_g: last.report.total_dry_mass - prev_mass,
            });
            prev_pods = pods;
            prev_mass = last.report.total_dry_mass;
        }
        let mut season = [0.0; 6];
        for w in &weeks {
            for (s, p) in season.iter_mut().zip(w.pressure) {
                *s += p;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let nw = weeks.len() as f64;
        let season = season.map(|s| s / nw);
        let mut peak = [0.0_f64; 6];
        for w in &weeks {
            for (m, p) in peak.iter_mut().zip(w.pressure) {
                *m = m.max(p);
            }
        }
        // Pods are made in bloom, so a factor that hit while flowers were open outranks
        // one that did not, whatever their sizes.
        let mut bloom_peak = [0.0_f64; 6];
        for w in weeks.iter().filter(|w| w.bloom) {
            for (m, p) in bloom_peak.iter_mut().zip(w.pressure) {
                *m = m.max(p);
            }
        }
        let mut rank = ranked(&peak, notable);
        // Bloom factors first, by their bloom-week peak; then the rest by their peak.
        let key = |f: &Factor| {
            let b = bloom_peak[f.index()];
            if b >= notable {
                (true, b)
            } else {
                (false, peak[f.index()])
            }
        };
        rank.sort_by(|a, b| {
            let (ka, kb) = (key(a), key(b));
            kb.0.cmp(&ka.0).then(kb.1.total_cmp(&ka.1))
        });
        let summary = summary(&weeks, &peak, &bloom_peak, &rank, notable);
        let backdrop: Vec<Factor> = rank
            .iter()
            .copied()
            .filter(|f| is_backdrop(&weeks, &bloom_peak, *f, notable))
            .collect();
        plants.push(PlantDebrief {
            plant: p.id,
            species: p.species.clone(),
            weeks,
            season,
            peak,
            backdrop,
            ranked: rank,
            summary,
        });
    }
    Debrief { plants }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pod_set_response_matches_the_model() {
        let t = [7.0, 22.0, 25.0, 36.0];
        assert!((pod_set_response(t, 23.0) - 1.0).abs() < 1e-12);
        assert!((pod_set_response(t, 30.5) - 0.75).abs() < 1e-12);
        assert!(pod_set_response(t, 36.0).abs() < 1e-12);
    }

    #[test]
    fn trapezoid_shape() {
        let t = [5.0, 25.0, 30.0, 40.0];
        assert!((trapezoid(t, 5.0)).abs() < 1e-12);
        assert!((trapezoid(t, 15.0) - 0.5).abs() < 1e-12);
        assert!((trapezoid(t, 27.0) - 1.0).abs() < 1e-12);
        assert!((trapezoid(t, 35.0) - 0.5).abs() < 1e-12);
        assert!((trapezoid(t, 45.0)).abs() < 1e-12);
    }

    #[test]
    fn shipped_refs_parse() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/game/debrief.ron");
        let text = std::fs::read_to_string(path).expect("read");
        let r: DebriefRefs = ron::from_str(&text).expect("parse");
        assert_eq!(r.full_sun_radiation.unit, "MJ/m2/d");
        assert!(!r.full_sun_radiation.source.is_empty());
    }
}
