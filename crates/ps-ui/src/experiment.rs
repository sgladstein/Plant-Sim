//! Experiment mode (design plan §2.5 and §5.6, WP17).
//!
//! Two to four pots, sown the same day under the same sky, with one variable differing
//! between them: shade, the amount of water, how often it comes, or the pot's size. An
//! experiment card states a question to answer with the charts. The dialog only edits
//! an [`ExperimentSetup`]; the data source builds the scenario, starts the run and
//! writes the lab notebook. The charts panel overlays the pots, so the comparison is on
//! screen as the run goes.

use bevy::prelude::Resource;
use bevy_egui::egui;

/// What differs between the pots.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Variable {
    /// Share of open-sky light that reaches the pot, under shade cloth.
    #[default]
    Light,
    /// Water per watering, mm.
    WaterAmount,
    /// Days between waterings.
    WaterInterval,
    /// Pot diameter (and depth), cm.
    PotSize,
}

impl Variable {
    /// All variables, in menu order.
    pub const ALL: [Self; 4] = [
        Self::Light,
        Self::WaterAmount,
        Self::WaterInterval,
        Self::PotSize,
    ];

    /// Menu name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Light => "Light (shade cloth)",
            Self::WaterAmount => "Water per watering",
            Self::WaterInterval => "Days between waterings",
            Self::PotSize => "Pot size",
        }
    }

    /// Short name for the notebook and pot labels.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::WaterAmount => "water_mm",
            Self::WaterInterval => "water_every_days",
            Self::PotSize => "pot_cm",
        }
    }

    /// Unit of a level.
    #[must_use]
    pub const fn unit(self) -> &'static str {
        match self {
            Self::Light => "% of full sun",
            Self::WaterAmount => "mm",
            Self::WaterInterval => "days",
            Self::PotSize => "cm",
        }
    }

    /// Allowed range of a level.
    #[must_use]
    pub const fn range(self) -> (f64, f64) {
        match self {
            Self::Light => (5.0, 100.0),
            Self::WaterAmount => (0.0, 40.0),
            Self::WaterInterval => (1.0, 14.0),
            Self::PotSize => (10.0, 40.0),
        }
    }

    /// The value every pot gets when this variable is held fixed.
    #[must_use]
    pub const fn fixed(self) -> f64 {
        match self {
            Self::Light => 100.0,
            Self::WaterAmount => 10.0,
            Self::WaterInterval => 2.0,
            Self::PotSize => 20.0,
        }
    }

    /// Starting levels for `n` pots, spread over a useful range.
    #[must_use]
    pub fn default_levels(self, n: usize) -> Vec<f64> {
        let spread: &[f64] = match self {
            Self::Light => &[100.0, 50.0, 25.0, 75.0],
            Self::WaterAmount => &[10.0, 3.0, 20.0, 6.0],
            Self::WaterInterval => &[1.0, 4.0, 7.0, 2.0],
            Self::PotSize => &[20.0, 12.0, 30.0, 16.0],
        };
        let mut v: Vec<f64> = spread.iter().copied().take(n.clamp(2, 4)).collect();
        v.sort_by(|a, b| b.total_cmp(a));
        v
    }

    /// A pot's label for a level, for example "50% sun" or "3 mm".
    #[must_use]
    pub fn label(self, level: f64) -> String {
        match self {
            Self::Light => format!("{level:.0}% sun"),
            Self::WaterAmount => format!("{level:.0} mm every 2 days"),
            Self::WaterInterval => format!("10 mm every {level:.0} days"),
            Self::PotSize => format!("{level:.0} cm pot"),
        }
    }
}

/// One experiment card (design plan §6.5), read from `data/experiments/`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CardRow {
    /// Card id.
    pub id: String,
    /// The question.
    pub prompt: String,
    /// What it varies and the levels it starts with.
    pub variable: Variable,
    /// Levels, one per pot.
    pub levels: Vec<f64>,
    /// Species id.
    pub species: String,
    /// Latitude, degrees north, if the card sets one.
    pub latitude_deg: Option<f64>,
    /// Chart titles to watch.
    pub watch: Vec<String>,
}

/// What the dialog describes.
#[derive(Clone, Debug, PartialEq)]
pub struct ExperimentSetup {
    /// The card it came from, if any.
    pub card: Option<String>,
    /// The question, from the card or typed.
    pub prompt: String,
    /// Species id.
    pub species: String,
    /// What differs between pots.
    pub variable: Variable,
    /// One level per pot, two to four.
    pub levels: Vec<f64>,
    /// Weather choice (preset name or `weather/<file>.csv`).
    pub weather: String,
    /// Latitude, degrees north.
    pub latitude_deg: f64,
    /// Days to run.
    pub days: u32,
    /// World seed.
    pub world_seed: u64,
    /// Chart titles worth watching, from the card.
    pub watch: Vec<String>,
}

impl Default for ExperimentSetup {
    fn default() -> Self {
        Self {
            card: None,
            prompt: String::new(),
            species: "bush_bean".into(),
            variable: Variable::Light,
            levels: Variable::Light.default_levels(2),
            weather: "temperate_summer".into(),
            latitude_deg: 45.0,
            days: 90,
            world_seed: 42,
            watch: Vec::new(),
        }
    }
}

impl ExperimentSetup {
    /// The setup a card describes, keeping this setup's weather, seed and length.
    #[must_use]
    pub fn from_card(&self, card: &CardRow) -> Self {
        Self {
            card: Some(card.id.clone()),
            prompt: card.prompt.clone(),
            species: card.species.clone(),
            variable: card.variable,
            levels: card.levels.clone(),
            latitude_deg: card.latitude_deg.unwrap_or(self.latitude_deg),
            watch: card.watch.clone(),
            ..self.clone()
        }
    }
}

/// The experiment running now, shown in the right panel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Running {
    /// The question.
    pub prompt: String,
    /// "Light (shade cloth)" and so on.
    pub variable: String,
    /// One label per pot, in pot order.
    pub pots: Vec<String>,
    /// The lab notebook file.
    pub notebook: String,
    /// Chart titles worth watching.
    pub watch: Vec<String>,
}

/// The experiment dialog's state, the running experiment and requests to the source.
#[derive(Clone, Debug, Default, Resource)]
pub struct Experiment {
    /// Whether the dialog is open.
    pub open: bool,
    /// Cards from `data/experiments/`.
    pub cards: Vec<CardRow>,
    /// Species ids from `data/species/`.
    pub species: Vec<String>,
    /// Weather choices.
    pub weathers: Vec<String>,
    /// What the dialog describes.
    pub setup: ExperimentSetup,
    /// Set by the Start button; the data source clears it.
    pub start: bool,
    /// Set by the Record results button; the data source clears it.
    pub record: bool,
    /// The experiment on screen, if the run is one.
    pub running: Option<Running>,
    /// What the last start or record did.
    pub status: String,
}

/// The experiment dialog.
pub(crate) fn window(ctx: &egui::Context, ex: &mut Experiment) {
    let mut open = ex.open;
    egui::Window::new("Experiment")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(420.0)
        .show(ctx, |ui| dialog(ui, ex));
    ex.open = open && !ex.start;
}

#[allow(clippy::too_many_lines)] // One row per control.
fn dialog(ui: &mut egui::Ui, ex: &mut Experiment) {
    ui.label("Pots sown the same day under the same sky, with one thing different. Watch the charts to see what it does.");
    ui.add_space(4.0);
    if !ex.cards.is_empty() {
        ui.strong("Start from a question");
        let mut chosen = None;
        for card in &ex.cards {
            let on = ex.setup.card.as_deref() == Some(card.id.as_str());
            if ui.selectable_label(on, &card.prompt).clicked() {
                chosen = Some(card.clone());
            }
        }
        if let Some(card) = chosen {
            ex.setup = ex.setup.from_card(&card);
        }
        ui.separator();
    }
    let s = &mut ex.setup;
    egui::Grid::new("experiment")
        .num_columns(2)
        .spacing([10.0, 4.0])
        .show(ui, |ui| {
            ui.label("Species");
            egui::ComboBox::from_id_salt("ex-species")
                .selected_text(s.species.replace('_', " "))
                .width(200.0)
                .show_ui(ui, |ui| {
                    for sp in &ex.species {
                        ui.selectable_value(&mut s.species, sp.clone(), sp.replace('_', " "));
                    }
                });
            ui.end_row();
            ui.label("What differs");
            let before = s.variable;
            egui::ComboBox::from_id_salt("ex-variable")
                .selected_text(s.variable.name())
                .width(200.0)
                .show_ui(ui, |ui| {
                    for v in Variable::ALL {
                        ui.selectable_value(&mut s.variable, v, v.name());
                    }
                });
            if s.variable != before {
                s.levels = s.variable.default_levels(s.levels.len());
                s.card = None;
                s.watch.clear();
            }
            ui.end_row();
            ui.label("Pots");
            let mut n = s.levels.len();
            ui.add(egui::Slider::new(&mut n, 2..=4));
            if n != s.levels.len() {
                let more = s.variable.default_levels(4);
                s.levels.resize(n, 0.0);
                for (i, l) in s.levels.iter_mut().enumerate() {
                    if *l <= 0.0 && s.variable != Variable::WaterAmount {
                        *l = more[i.min(3)];
                    }
                }
            }
            ui.end_row();
            let (lo, hi) = s.variable.range();
            for (i, level) in s.levels.iter_mut().enumerate() {
                ui.label(format!("Pot {i}"));
                ui.add(
                    egui::Slider::new(level, lo..=hi)
                        .suffix(format!(" {}", s.variable.unit()))
                        .fixed_decimals(0),
                );
                ui.end_row();
            }
            ui.label("Weather");
            egui::ComboBox::from_id_salt("ex-weather")
                .selected_text(&s.weather)
                .width(200.0)
                .show_ui(ui, |ui| {
                    for w in &ex.weathers {
                        ui.selectable_value(&mut s.weather, w.clone(), w);
                    }
                });
            ui.end_row();
            ui.label("Latitude");
            ui.add(
                egui::Slider::new(&mut s.latitude_deg, -60.0..=60.0)
                    .suffix("° N")
                    .fixed_decimals(1),
            );
            ui.end_row();
            ui.label("Days");
            ui.add(egui::Slider::new(&mut s.days, 20..=150));
            ui.end_row();
            ui.label("World seed");
            ui.add(egui::DragValue::new(&mut s.world_seed));
            ui.end_row();
        });
    ui.add_space(4.0);
    ui.label("Question (optional)");
    ui.text_edit_singleline(&mut s.prompt);
    ui.add_space(4.0);
    ui.weak(format!(
        "Held the same in every pot: {}. Each plant still grows from its own seed, so small differences are not the variable's doing.",
        held_same(s.variable)
    ));
    ui.add_space(4.0);
    if ui.button("Start experiment").clicked() {
        ex.start = true;
    }
}

/// What every pot shares when `v` is the variable.
#[must_use]
pub fn held_same(v: Variable) -> String {
    let mut out = Vec::new();
    if v != Variable::Light {
        out.push("full sun".to_owned());
    }
    out.push(match v {
        Variable::WaterAmount => {
            format!("watering every {:.0} days", Variable::WaterInterval.fixed())
        }
        Variable::WaterInterval => format!("{:.0} mm each watering", Variable::WaterAmount.fixed()),
        _ => format!(
            "{:.0} mm every {:.0} days",
            Variable::WaterAmount.fixed(),
            Variable::WaterInterval.fixed()
        ),
    });
    if v != Variable::PotSize {
        out.push(format!("a {:.0} cm pot", Variable::PotSize.fixed()));
    }
    out.push("loam".to_owned());
    out.join(", ")
}

/// The running experiment in the right panel: the question, the pots and the notebook.
pub(crate) fn section(ui: &mut egui::Ui, ex: &mut Experiment) {
    let Some(r) = &ex.running else {
        return;
    };
    ui.heading("Experiment");
    if !r.prompt.is_empty() {
        ui.label(egui::RichText::new(&r.prompt).italics());
    }
    ui.label(format!("Varies: {}", r.variable));
    if !r.watch.is_empty() {
        ui.label(format!("Watch: {}", r.watch.join(", ")));
    }
    for (i, p) in r.pots.iter().enumerate() {
        let style = ["solid", "dashed", "dotted", "long-dashed"][i.min(3)];
        ui.weak(format!("Pot {i}: {p} ({style} line in the charts)"));
    }
    if ui
        .button("Record results")
        .on_hover_text(
            "Write the daily and organ tables for the day on screen next to the lab notebook",
        )
        .clicked()
    {
        ex.record = true;
    }
    ui.weak(format!("Notebook: {}", r.notebook));
    if !ex.status.is_empty() {
        ui.weak(&ex.status);
    }
    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_levels_fit_the_range_and_count() {
        for v in Variable::ALL {
            let (lo, hi) = v.range();
            assert!((lo..=hi).contains(&v.fixed()), "{v:?}");
            for n in 2..=4 {
                let l = v.default_levels(n);
                assert_eq!(l.len(), n);
                assert!(l.iter().all(|x| (lo..=hi).contains(x)), "{v:?} {l:?}");
                assert!(l.windows(2).all(|w| w[0] > w[1]), "{v:?} {l:?}");
            }
        }
    }

    #[test]
    fn held_same_never_names_the_variable() {
        assert_eq!(
            held_same(Variable::PotSize),
            "full sun, 10 mm every 2 days, loam"
        );
        assert_eq!(
            held_same(Variable::WaterInterval),
            "full sun, 10 mm each watering, a 20 cm pot, loam"
        );
        assert_eq!(
            held_same(Variable::Light),
            "10 mm every 2 days, a 20 cm pot, loam"
        );
    }

    #[test]
    fn a_card_sets_the_variable_and_keeps_the_weather() {
        let base = ExperimentSetup {
            weather: "overcast".into(),
            ..ExperimentSetup::default()
        };
        let card = CardRow {
            id: "dry".into(),
            prompt: "Q".into(),
            variable: Variable::WaterAmount,
            levels: vec![10.0, 2.0],
            species: "bush_bean".into(),
            latitude_deg: Some(60.0),
            watch: Vec::new(),
        };
        let s = base.from_card(&card);
        assert_eq!(s.variable, Variable::WaterAmount);
        assert_eq!(s.levels, [10.0, 2.0]);
        assert_eq!(s.weather, "overcast");
        assert!((s.latitude_deg - 60.0).abs() < 1e-12);
        assert_eq!(s.card.as_deref(), Some("dry"));
    }
}
