//! Challenges and the run debrief (design plan §2.6, §6.4; WP32).
//!
//! The window lists the challenges, lets the player plan what the rules allow ahead
//! of time (the sowing day, fertilizer doses), and starts one. While it runs, a panel
//! above the lenses shows the score so far, the budget left and any broken rule. At the
//! last day the result card shows the medal and the debrief: week by week, what held
//! each plant back. The data source fills every string and number here; this module
//! only draws them and records requests.

use bevy::prelude::{ResMut, Resource};
use bevy_egui::{EguiContexts, egui};

use crate::Tools;

/// Fertilizer doses a plan can hold.
pub const MAX_DOSES: usize = 3;

/// One challenge in the list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChallengeRow {
    /// File stem.
    pub id: String,
    /// Title.
    pub title: String,
    /// The task.
    pub brief: String,
    /// A nudge toward the science.
    pub hint: String,
    /// What the player may do, in words.
    pub rules: String,
    /// Bronze, silver and gold, with the score's label.
    pub medals: String,
    /// Sowing window in scenario days, when the player sows.
    pub sow_window: Option<(u32, u32)>,
    /// Nitrogen budget, g N, when the player fertilizes ahead.
    pub nitrogen_budget: Option<f64>,
    /// The player's best so far, in words.
    pub best: Option<String>,
}

/// One planned fertilizer dose.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Dose {
    /// Scenario day.
    pub day: u32,
    /// Nitrogen, g N.
    pub grams: f64,
}

/// The plan for the chosen challenge.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    /// Sowing day, when the challenge has a sowing window.
    pub sow_day: u32,
    /// Fertilizer doses.
    pub doses: Vec<Dose>,
}

/// A challenge in progress.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Live {
    /// Challenge id.
    pub id: String,
    /// Title.
    pub title: String,
    /// The task.
    pub brief: String,
    /// The hint.
    pub hint: String,
    /// The score so far, in words ("7 pods").
    pub score: String,
    /// The next medal and what it needs, in words.
    pub next_medal: String,
    /// Budgets left, one line each.
    pub budgets: Vec<String>,
    /// Broken rules, one line each.
    pub breaks: Vec<String>,
    /// Day on screen and last day.
    pub day: (u32, u32),
}

/// One week of the debrief strip.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeekCell {
    /// Week number.
    pub week: u32,
    /// The factor that held the plant back most, or `None` for a good week.
    pub top: Option<String>,
    /// How hard, 0 to 1.
    pub pressure: f32,
    /// Its color, sRGB.
    pub color: [u8; 3],
    /// Hover text: days, stage, pods gained and every factor.
    pub hover: String,
}

/// One plant's debrief.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlantStory {
    /// Plant name.
    pub name: String,
    /// The week strip.
    pub weeks: Vec<WeekCell>,
    /// Sentences, worst first.
    pub summary: Vec<String>,
}

/// The result card.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChallengeResult {
    /// Title.
    pub title: String,
    /// Final score in words.
    pub score: String,
    /// The seasons behind a mean score, in words; empty for one season.
    pub seasons: String,
    /// Medal name, or `None`.
    pub medal: Option<String>,
    /// Its color, sRGB.
    pub medal_color: [u8; 3],
    /// Whether this beat the player's best.
    pub new_best: bool,
    /// Broken rules.
    pub breaks: Vec<String>,
    /// One story per plant.
    pub plants: Vec<PlantStory>,
    /// Legend: factor name and color.
    pub legend: Vec<(String, [u8; 3])>,
    /// Where the result was written.
    pub saved: String,
}

/// The challenge window's state.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct Challenges {
    /// Challenges, in menu order.
    pub rows: Vec<ChallengeRow>,
    /// The chosen row.
    pub chosen: Option<usize>,
    /// The plan for it.
    pub plan: Plan,
    /// Set by Start; the data source clears it.
    pub start: bool,
    /// Set by Try again on the result card.
    pub retry: bool,
    /// The running challenge.
    pub live: Option<Live>,
    /// The last result, shown until closed.
    pub result: Option<ChallengeResult>,
    /// Whether the result card is open.
    pub result_open: bool,
    /// A message from the data source.
    pub status: String,
    /// Sim day to open a started challenge at (the `--day` flag); 0 from the window.
    pub start_day: f64,
}

impl Challenges {
    /// Chooses row `i` and resets the plan to the row's defaults.
    pub fn choose(&mut self, i: usize) {
        self.chosen = Some(i);
        let Some(row) = self.rows.get(i) else { return };
        self.plan = Plan {
            sow_day: row.sow_window.map_or(0, |w| w.0),
            doses: row
                .nitrogen_budget
                .map(|b| vec![Dose { day: 20, grams: b }])
                .unwrap_or_default(),
        };
    }

    /// Nitrogen the plan spends, g N.
    #[must_use]
    pub fn planned_nitrogen(&self) -> f64 {
        self.plan.doses.iter().map(|d| d.grams).sum()
    }
}

fn color(c: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

/// The panel above the lenses while a challenge runs.
pub(crate) fn section(ui: &mut egui::Ui, c: &Challenges) {
    let Some(live) = &c.live else { return };
    ui.heading(&live.title);
    ui.label(egui::RichText::new(&live.brief).italics());
    ui.horizontal(|ui| {
        ui.strong(&live.score);
        ui.weak(format!("day {} of {}", live.day.0, live.day.1));
    });
    ui.label(&live.next_medal);
    for b in &live.budgets {
        ui.label(b);
    }
    for b in &live.breaks {
        ui.colored_label(egui::Color32::from_rgb(230, 140, 110), b);
    }
    ui.collapsing("Hint", |ui| ui.label(&live.hint));
    ui.separator();
}

/// The challenge list, the plan and the result card.
pub(crate) fn window_system(
    mut contexts: EguiContexts,
    mut tools: ResMut<Tools>,
    mut c: ResMut<Challenges>,
) -> bevy::prelude::Result {
    let ctx = contexts.ctx_mut()?;
    if tools.challenges {
        let mut open = true;
        egui::Window::new("Challenges")
            .open(&mut open)
            .collapsible(false)
            .default_width(460.0)
            .show(ctx, |ui| list(ui, &mut c));
        tools.challenges = open && !c.start;
    }
    if c.result_open && c.result.is_some() {
        let mut open = true;
        egui::Window::new("Season debrief")
            .open(&mut open)
            .collapsible(false)
            .default_width(560.0)
            .show(ctx, |ui| result(ui, &mut c));
        c.result_open = open && !c.retry;
    }
    Ok(())
}

fn list(ui: &mut egui::Ui, c: &mut Challenges) {
    ui.label(
        "A scored season: the same model as the sandbox, with a goal, a limit and a medal. \
         Scores are recomputed from the replay, so anyone with the save gets the same number.",
    );
    ui.add_space(4.0);
    let mut chosen = None;
    for (i, row) in c.rows.iter().enumerate() {
        let on = c.chosen == Some(i);
        let text = match &row.best {
            Some(b) => format!("{}   (best: {b})", row.title),
            None => row.title.clone(),
        };
        if ui
            .selectable_label(on, egui::RichText::new(text).strong())
            .clicked()
        {
            chosen = Some(i);
        }
    }
    if let Some(i) = chosen {
        c.choose(i);
    }
    let Some(row) = c.chosen.and_then(|i| c.rows.get(i)).cloned() else {
        if !c.status.is_empty() {
            ui.separator();
            ui.label(&c.status);
        }
        return;
    };
    ui.separator();
    ui.label(&row.brief);
    ui.weak(&row.rules);
    ui.label(format!("Medals: {}", row.medals));
    ui.collapsing("Hint", |ui| ui.label(&row.hint));
    if let Some((lo, hi)) = row.sow_window {
        ui.horizontal(|ui| {
            ui.label("Sow on day");
            ui.add(egui::Slider::new(&mut c.plan.sow_day, lo..=hi));
        });
    }
    if let Some(budget) = row.nitrogen_budget {
        ui.label(format!("Fertilizer plan (nitrate, {budget} g N to spend)"));
        let mut remove = None;
        for (k, d) in c.plan.doses.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label("Day");
                ui.add(egui::DragValue::new(&mut d.day).range(0..=89));
                ui.label("g N");
                ui.add(
                    egui::DragValue::new(&mut d.grams)
                        .range(0.0..=budget)
                        .speed(0.01)
                        .fixed_decimals(2),
                );
                if ui.small_button("Remove").clicked() {
                    remove = Some(k);
                }
            });
        }
        if let Some(k) = remove {
            c.plan.doses.remove(k);
        }
        if c.plan.doses.len() < MAX_DOSES && ui.small_button("Add a dose").clicked() {
            c.plan.doses.push(Dose {
                day: 40,
                grams: 0.0,
            });
        }
        let spent = c.planned_nitrogen();
        if spent > budget + 1e-9 {
            ui.colored_label(
                egui::Color32::from_rgb(230, 140, 110),
                format!("{spent:.2} g planned: over the budget, so no medal"),
            );
        } else {
            ui.weak(format!("{spent:.2} of {budget} g planned"));
        }
    }
    ui.add_space(4.0);
    if ui.button("Start").clicked() {
        c.start = true;
    }
    if !c.status.is_empty() {
        ui.label(&c.status);
    }
}

fn result(ui: &mut egui::Ui, c: &mut Challenges) {
    let Some(r) = &c.result else { return };
    ui.heading(&r.title);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&r.score).size(20.0).strong());
        match &r.medal {
            Some(m) => {
                ui.label(
                    egui::RichText::new(format!("● {m}"))
                        .size(20.0)
                        .color(color(r.medal_color)),
                );
            }
            None => {
                ui.weak("no medal");
            }
        }
        if r.new_best {
            ui.label(egui::RichText::new("new best").italics());
        }
    });
    if !r.seasons.is_empty() {
        ui.weak(&r.seasons);
    }
    for b in &r.breaks {
        ui.colored_label(egui::Color32::from_rgb(230, 140, 110), b);
    }
    for p in &r.plants {
        ui.separator();
        ui.strong(format!("What held {} back, week by week", p.name));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for w in &p.weeks {
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(26.0, 22.0), egui::Sense::hover());
                let base = egui::Color32::from_rgb(60, 56, 50);
                let fill = if w.top.is_some() {
                    base.lerp_to_gamma(color(w.color), 0.35 + 0.65 * w.pressure.clamp(0.0, 1.0))
                } else {
                    base
                };
                ui.painter().rect_filled(rect, 3.0, fill);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    w.week.to_string(),
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(236, 232, 224),
                );
                resp.on_hover_text(&w.hover);
            }
        });
        for s in &p.summary {
            ui.label(format!("• {s}"));
        }
    }
    ui.horizontal_wrapped(|ui| {
        for (name, c) in &r.legend {
            ui.label(egui::RichText::new("■").color(color(*c)));
            ui.weak(name);
        }
        ui.weak("(grey: nothing out of the ordinary; hover a week for its numbers)");
    });
    if !r.saved.is_empty() {
        ui.weak(&r.saved);
    }
    if ui.button("Try again").clicked() {
        c.retry = true;
    }
}
