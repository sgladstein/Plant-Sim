//! The breeding bench window (design plan §2, WP30): the seed library, the next
//! generation's plan and the last generation's results.
//!
//! The UI only describes and requests (§5.8): the data source owns the library file,
//! runs the bench off the UI thread and fills [`Breeding`] back in.

use bevy::prelude::{ResMut, Resource};
use bevy_egui::{EguiContexts, egui};

use crate::Tools;

/// The most pots a bench holds.
pub const MAX_POTS: usize = 10;

/// A seed coat (or a plant's seeds) as the table draws it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Coat {
    /// Pigment color, sRGB.
    pub color: [u8; 3],
    /// Ground under a pattern, sRGB.
    pub ground: [u8; 3],
    /// Share of the coat in the pigment color, 0 to 1.
    pub coverage: f32,
    /// Pattern name ("self", "mottled", ...).
    pub pattern: String,
    /// Flower color, sRGB.
    pub flower: [u8; 3],
}

/// One family in the library list: seeds of one mother plant with the same parents.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibraryRow {
    /// Library ids, ascending (one for a founder).
    pub ids: Vec<u64>,
    /// Species id.
    pub species: String,
    /// Generations since a founder.
    pub generation: u32,
    /// Parents in words: "founder", "#12 selfed", "#12 × #15".
    pub parents: String,
    /// Mean dry mass, mg.
    pub mass_mg: f64,
    /// Its coat, when the species has a colour table.
    pub coat: Option<Coat>,
    /// Where it came from: "line", "founder", "Bench 3, day 81".
    pub origin: String,
}

/// One plant of the last generation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResultRow {
    /// Pot, from 0.
    pub pot: usize,
    /// Library id of the seed sown.
    pub seed: u64,
    /// Father's pot when the plan crossed it.
    pub crossed_with: Option<usize>,
    /// Days to emergence (VE), first flower (R1) and maturity (R9).
    pub stages: [Option<u32>; 3],
    /// Tallest shoot, cm.
    pub height_cm: f64,
    /// Largest leaf area, cm².
    pub leaf_area_cm2: f64,
    /// Largest dry mass, g.
    pub dry_mass_g: f64,
    /// Pods picked.
    pub pods: u32,
    /// Seeds picked.
    pub seeds: u32,
    /// Seed yield, g.
    pub seed_yield_g: f64,
    /// Mean seed mass, mg.
    pub mean_seed_mg: f64,
    /// Its flower and the coat of the seeds it bore.
    pub coat: Option<Coat>,
    /// Library ids of its seeds.
    pub saved: Vec<u64>,
}

/// A grown generation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchResults {
    /// The plan's label.
    pub label: String,
    /// Days the bench ran.
    pub days: u32,
    /// One row per pot.
    pub rows: Vec<ResultRow>,
}

/// A trait the results chart can plot.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Trait {
    /// Seed yield.
    #[default]
    Yield,
    /// Mean seed mass.
    SeedMass,
    /// Height.
    Height,
    /// Days to first flower.
    Flowering,
    /// Pods.
    Pods,
    /// Dry mass.
    DryMass,
}

impl Trait {
    /// Every trait, in menu order.
    pub const ALL: [Self; 6] = [
        Self::Yield,
        Self::SeedMass,
        Self::Height,
        Self::Flowering,
        Self::Pods,
        Self::DryMass,
    ];

    /// Menu name with its unit.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Yield => "Seed yield (g)",
            Self::SeedMass => "Mean seed mass (mg)",
            Self::Height => "Height (cm)",
            Self::Flowering => "Days to first flower",
            Self::Pods => "Pods",
            Self::DryMass => "Dry mass (g)",
        }
    }

    /// The row's value; `None` when it never got there.
    #[must_use]
    pub fn of(self, r: &ResultRow) -> Option<f64> {
        match self {
            Self::Yield => Some(r.seed_yield_g),
            Self::SeedMass => (r.seeds > 0).then_some(r.mean_seed_mg),
            Self::Height => Some(r.height_cm),
            Self::Flowering => r.stages[1].map(f64::from),
            Self::Pods => Some(f64::from(r.pods)),
            Self::DryMass => Some(r.dry_mass_g),
        }
    }
}

/// The breeding bench's state and requests.
#[derive(Clone, Debug, Resource)]
pub struct Breeding {
    /// The library, newest last.
    pub library: Vec<LibraryRow>,
    /// Where the library file is, for the window's footer.
    pub library_path: String,
    /// Species that can found lines.
    pub species: Vec<String>,
    /// The species chosen for founders.
    pub founder_species: usize,
    /// How many founders to draw.
    pub founder_count: u32,
    /// The next generation's seeds, one per pot.
    pub plan: Vec<u64>,
    /// Per pot: the pot whose pollen it takes, if crossed.
    pub fathers: Vec<Option<usize>>,
    /// Last day of the bench.
    pub max_days: u32,
    /// The last generation.
    pub results: Option<BenchResults>,
    /// What the results chart plots.
    pub chart: Trait,
    /// Whether a generation is growing.
    pub running: bool,
    /// What the last request did.
    pub status: String,
    /// What the window asked for this frame (set by the window, taken by the source).
    pub request: Option<Request>,
}

/// A request from the breeding window to the data source.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// Draw founders of the chosen species.
    Founders,
    /// Add the chosen species' own line.
    Line,
    /// Bring the garden's harvested seeds into the library.
    ImportGarden,
    /// Grow the plan.
    Grow,
}

impl Default for Breeding {
    fn default() -> Self {
        Self {
            library: Vec::new(),
            library_path: String::new(),
            species: Vec::new(),
            founder_species: 0,
            founder_count: 5,
            plan: Vec::new(),
            fathers: Vec::new(),
            max_days: 120,
            results: None,
            chart: Trait::Yield,
            running: false,
            status: String::new(),
            request: None,
        }
    }
}

impl Breeding {
    /// Puts seed `id` in the next free pot; false when the bench is full or it is there.
    pub fn sow(&mut self, id: u64) -> bool {
        if self.plan.len() >= MAX_POTS || self.plan.contains(&id) {
            return false;
        }
        self.plan.push(id);
        self.fathers.push(None);
        true
    }

    /// Takes pot `pot` off the plan; crosses to it go, later pots move up.
    pub fn unsow(&mut self, pot: usize) {
        if pot >= self.plan.len() {
            return;
        }
        self.plan.remove(pot);
        self.fathers.remove(pot);
        for f in &mut self.fathers {
            *f = match *f {
                Some(p) if p == pot => None,
                Some(p) if p > pot => Some(p - 1),
                other => other,
            };
        }
    }

    /// The plan's crosses as (mother pot, father pot).
    #[must_use]
    pub fn crosses(&self) -> Vec<(usize, usize)> {
        self.fathers
            .iter()
            .enumerate()
            .filter_map(|(m, f)| f.map(|f| (m, f)))
            .collect()
    }
}

/// "#5", or "#177–#184 (8)" for a family.
fn ids(ids: &[u64]) -> String {
    match ids {
        [] => String::new(),
        [one] => format!("#{one}"),
        [first, .., last] => format!("#{first}–#{last} ({})", ids.len()),
    }
}

fn color(c: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

/// A seed swatch: the ground with the pigment over `coverage` of it.
fn swatch(ui: &mut egui::Ui, coat: Option<&Coat>) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(22.0, 14.0), egui::Sense::hover());
    let Some(c) = coat else {
        ui.painter().rect_stroke(
            rect,
            3.0,
            (1.0, egui::Color32::GRAY),
            egui::StrokeKind::Inside,
        );
        return response;
    };
    let p = ui.painter();
    p.rect_filled(rect, 5.0, color(c.ground));
    let mut pigment = rect;
    pigment.set_width(rect.width() * c.coverage.clamp(0.0, 1.0));
    p.rect_filled(pigment, 5.0, color(c.color));
    p.circle_filled(
        rect.right_top() + egui::vec2(-3.0, 3.0),
        2.5,
        color(c.flower),
    );
    response.on_hover_text(format!("{} coat; dot: flower", c.pattern))
}

/// The breeding bench window, opened from the top bar.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn window_system(
    mut contexts: EguiContexts,
    mut tools: ResMut<Tools>,
    breeding: Option<ResMut<Breeding>>,
) -> bevy::prelude::Result {
    let Some(mut b) = breeding else {
        return Ok(());
    };
    if !tools.breeding {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let mut open = true;
    egui::Window::new("Breeding bench")
        .open(&mut open)
        .default_width(720.0)
        .default_height(560.0)
        .show(ctx, |ui| window(ui, &mut b));
    tools.breeding = open;
    Ok(())
}

fn window(ui: &mut egui::Ui, b: &mut Breeding) {
    ui.label(
        "Grow up to ten saved seeds side by side in identical pots, cross them, and keep \
         the seeds of the plants you like for the next generation.",
    );
    if !b.status.is_empty() {
        ui.weak(&b.status);
    }
    ui.separator();
    let seeds: usize = b.library.iter().map(|r| r.ids.len()).sum();
    egui::CollapsingHeader::new(format!("Seed library ({seeds} seeds)"))
        .default_open(true)
        .show(ui, |ui| library(ui, b));
    ui.separator();
    plan(ui, b);
    if b.results.is_some() {
        ui.separator();
        results(ui, b);
    }
}

fn library(ui: &mut egui::Ui, b: &mut Breeding) {
    ui.horizontal(|ui| {
        let name = b
            .species
            .get(b.founder_species)
            .cloned()
            .unwrap_or_default();
        egui::ComboBox::from_id_salt("founder-species")
            .selected_text(name)
            .show_ui(ui, |ui| {
                for (i, s) in b.species.iter().enumerate() {
                    ui.selectable_value(&mut b.founder_species, i, s);
                }
            });
        ui.add(egui::DragValue::new(&mut b.founder_count).range(1..=20));
        if ui
            .button("Add founders")
            .on_hover_text("Pure lines drawn from the species' variation")
            .clicked()
        {
            b.request = Some(Request::Founders);
        }
        if ui
            .button("Add its line")
            .on_hover_text("The species file's own pure line, the garden's default seed")
            .clicked()
        {
            b.request = Some(Request::Line);
        }
        if ui
            .button("Import garden harvest")
            .on_hover_text("Pods picked in the garden join the library with their parents")
            .clicked()
        {
            b.request = Some(Request::ImportGarden);
        }
    });
    if b.library.is_empty() {
        ui.weak("No seeds yet. Add founders or import the garden's harvest.");
        return;
    }
    let mut sow = None;
    egui::ScrollArea::vertical()
        .id_salt("library")
        .max_height(200.0)
        .show(ui, |ui| {
            egui::Grid::new("library-grid")
                .num_columns(7)
                .striped(true)
                .show(ui, |ui| {
                    for h in ["", "Seed", "Species", "Gen", "Parents", "Mass", "From"] {
                        ui.strong(h);
                    }
                    ui.end_row();
                    for r in b.library.iter().rev() {
                        let next = r.ids.iter().copied().find(|id| !b.plan.contains(id));
                        let full = b.plan.len() >= MAX_POTS;
                        if ui
                            .add_enabled(next.is_some() && !full, egui::Button::new("Sow").small())
                            .on_hover_text(
                                "Put one of these seeds in the next free pot of the bench",
                            )
                            .clicked()
                        {
                            sow = next;
                        }
                        ui.horizontal(|ui| {
                            swatch(ui, r.coat.as_ref());
                            ui.label(ids(&r.ids));
                        });
                        ui.label(&r.species);
                        ui.label(r.generation.to_string());
                        ui.label(&r.parents);
                        ui.label(format!("{:.0} mg", r.mass_mg));
                        ui.weak(&r.origin);
                        ui.end_row();
                    }
                });
        });
    if let Some(id) = sow {
        b.sow(id);
    }
    if !b.library_path.is_empty() {
        ui.weak(format!("Saved in {}", b.library_path));
    }
}

fn plan(ui: &mut egui::Ui, b: &mut Breeding) {
    ui.strong(format!(
        "Next generation ({} of {MAX_POTS} pots)",
        b.plan.len()
    ));
    if b.plan.is_empty() {
        ui.weak("Sow seeds from the library, or keep seeds from the results below.");
    }
    let mut remove = None;
    let n = b.plan.len();
    egui::Grid::new("plan-grid").num_columns(4).show(ui, |ui| {
        for pot in 0..n {
            let id = b.plan[pot];
            let row = b.library.iter().find(|r| r.ids.contains(&id));
            ui.label(format!("Pot {}", pot + 1));
            ui.horizontal(|ui| {
                swatch(ui, row.and_then(|r| r.coat.as_ref()));
                ui.label(format!("#{id} {}", row.map_or("", |r| r.species.as_str())));
            });
            let father = &mut b.fathers[pot];
            let text = father.map_or_else(
                || "its own (selfs)".to_owned(),
                |f| format!("pot {}", f + 1),
            );
            egui::ComboBox::from_id_salt(("father", pot))
                .selected_text(format!("Pollen: {text}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(father, None, "its own (selfs)");
                    for f in (0..n).filter(|&f| f != pot) {
                        ui.selectable_value(father, Some(f), format!("pot {}", f + 1));
                    }
                });
            if ui.small_button("Remove").clicked() {
                remove = Some(pot);
            }
            ui.end_row();
        }
    });
    if let Some(pot) = remove {
        b.unsow(pot);
    }
    ui.horizontal(|ui| {
        ui.label("Up to");
        ui.add(
            egui::DragValue::new(&mut b.max_days)
                .range(30..=200)
                .suffix(" days"),
        );
        let label = if b.running {
            "Growing…"
        } else {
            "Grow generation"
        };
        if ui
            .add_enabled(!b.running && !b.plan.is_empty(), egui::Button::new(label))
            .on_hover_text(
                "Grows every pot to maturity in its own world (a few seconds), picks every \
                 pod and saves the seeds to the library",
            )
            .clicked()
        {
            b.request = Some(Request::Grow);
        }
        if !b.plan.is_empty() && ui.button("Clear").clicked() {
            b.plan.clear();
            b.fathers.clear();
        }
    });
}

#[allow(clippy::too_many_lines)]
fn results(ui: &mut egui::Ui, b: &mut Breeding) {
    let Some(res) = b.results.clone() else {
        return;
    };
    ui.horizontal(|ui| {
        ui.strong(format!("{}: {} days", res.label, res.days));
        egui::ComboBox::from_id_salt("bench-trait")
            .selected_text(b.chart.label())
            .show_ui(ui, |ui| {
                for t in Trait::ALL {
                    ui.selectable_value(&mut b.chart, t, t.label());
                }
            });
    });
    bars(ui, &res.rows, b.chart);
    let mut keep = None;
    egui::ScrollArea::horizontal()
        .id_salt("results")
        .show(ui, |ui| {
            egui::Grid::new("results-grid")
                .num_columns(13)
                .striped(true)
                .show(ui, |ui| {
                    for h in [
                        "Pot",
                        "Seed",
                        "Coat",
                        "VE",
                        "R1",
                        "R9",
                        "Height",
                        "Dry mass",
                        "Pods",
                        "Seeds",
                        "Yield",
                        "Seed mass",
                    ] {
                        ui.strong(h);
                    }
                    ui.strong("");
                    ui.end_row();
                    let day =
                        |d: Option<u32>| d.map_or_else(|| "–".to_owned(), |d| format!("d{d}"));
                    for r in &res.rows {
                        let cross = r
                            .crossed_with
                            .map_or_else(String::new, |f| format!(" × {}", f + 1));
                        ui.label(format!("{}{cross}", r.pot + 1));
                        ui.label(format!("#{}", r.seed));
                        swatch(ui, r.coat.as_ref());
                        for d in r.stages {
                            ui.label(day(d));
                        }
                        ui.label(format!("{:.0} cm", r.height_cm));
                        ui.label(format!("{:.1} g", r.dry_mass_g));
                        ui.label(r.pods.to_string());
                        ui.label(r.seeds.to_string());
                        ui.label(format!("{:.1} g", r.seed_yield_g));
                        ui.label(format!("{:.0} mg", r.mean_seed_mg));
                        let next = r.saved.iter().copied().find(|id| !b.plan.contains(id));
                        if ui
                            .add_enabled(
                                next.is_some() && b.plan.len() < MAX_POTS,
                                egui::Button::new("Keep a seed").small(),
                            )
                            .on_hover_text("Sow one of this plant's seeds in the next generation")
                            .clicked()
                        {
                            keep = next;
                        }
                        ui.end_row();
                    }
                });
        });
    if let Some(id) = keep {
        b.sow(id);
    }
}

/// One bar per pot for `t`, labeled by pot.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn bars(ui: &mut egui::Ui, rows: &[ResultRow], t: Trait) {
    let values: Vec<Option<f64>> = rows.iter().map(|r| t.of(r)).collect();
    let max = values.iter().flatten().copied().fold(0.0_f64, f64::max);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().min(560.0), 110.0),
        egui::Sense::hover(),
    );
    let p = ui.painter();
    p.rect_filled(rect, 4.0, egui::Color32::from_rgb(24, 22, 20));
    if rows.is_empty() {
        return;
    }
    let w = rect.width() / rows.len() as f32;
    let base = rect.bottom() - 16.0;
    let top = rect.top() + 14.0;
    let font = egui::FontId::proportional(11.0);
    for (i, (r, v)) in rows.iter().zip(&values).enumerate() {
        let x = rect.left() + w * (i as f32 + 0.5);
        p.text(
            egui::pos2(x, base + 2.0),
            egui::Align2::CENTER_TOP,
            format!("{}", r.pot + 1),
            font.clone(),
            egui::Color32::GRAY,
        );
        let Some(v) = *v else {
            p.text(
                egui::pos2(x, base - 2.0),
                egui::Align2::CENTER_BOTTOM,
                "–",
                font.clone(),
                egui::Color32::GRAY,
            );
            continue;
        };
        let h = if max > 0.0 {
            ((v / max) as f32) * (base - top)
        } else {
            0.0
        };
        let fill = r
            .coat
            .as_ref()
            .map_or(egui::Color32::from_rgb(122, 150, 112), |c| color(c.flower));
        p.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x - w * 0.3, base - h),
                egui::pos2(x + w * 0.3, base),
            ),
            2.0,
            fill,
        );
        p.text(
            egui::pos2(x, base - h - 1.0),
            egui::Align2::CENTER_BOTTOM,
            if v >= 100.0 {
                format!("{v:.0}")
            } else {
                format!("{v:.1}")
            },
            font.clone(),
            egui::Color32::from_gray(220),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsowing_moves_crosses_with_their_pots() {
        let mut b = Breeding::default();
        for id in [10, 11, 12] {
            assert!(b.sow(id));
        }
        assert!(!b.sow(11), "a seed is sown once");
        b.fathers[2] = Some(1);
        b.fathers[0] = Some(2);
        b.unsow(1);
        assert_eq!(b.plan, [10, 12]);
        // Pot 3 took pot 2's pollen; pot 2 is gone, so it selfs. Pot 1's father moved up.
        assert_eq!(b.fathers, [Some(1), None]);
        assert_eq!(b.crosses(), [(0, 1)]);
        for id in 20..40 {
            b.sow(id);
        }
        assert_eq!(b.plan.len(), MAX_POTS);
    }

    #[test]
    fn traits_skip_what_never_happened() {
        let r = ResultRow {
            seeds: 0,
            mean_seed_mg: 0.0,
            ..ResultRow::default()
        };
        assert_eq!(Trait::SeedMass.of(&r), None);
        assert_eq!(Trait::Flowering.of(&r), None);
        assert_eq!(Trait::Pods.of(&r), Some(0.0));
    }
}
