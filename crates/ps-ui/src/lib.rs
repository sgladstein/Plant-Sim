//! egui panels (design plan §5.1, §5.2, §5.5; WP17 groundwork).
//!
//! - Top bar: date, season and clock, pause, speed multipliers, step buttons and the
//!   replay scrubber. The bar never changes time itself: it queues [`TimeCommand`]s that
//!   the time owner (the demo driver now, the sim thread in WP16) applies, so the UI
//!   never edits sim state (§5.8).
//! - Right panel: the lens list and the active lens's legend, which is always visible
//!   while a lens is on, with its unit, explanation, encoded-or-emergent tag and model
//!   card.
//! - Under the lenses: the inspector for the clicked organ or plant (§5.3).
//! - Bottom panel: the charts (§5.4), toggled from the top bar.
//! - Left panel: the model card (§5.7) opened from a lens legend or an inspector value.
//! - A custom theme so the panels match the diorama rather than a debug tool (§4).

mod charts;
mod inspector;
mod model_cards;
mod tools;

pub use charts::{Chart, Charts, Series, nearest, nice_max};
pub use inspector::{Field, Inspector};
pub use model_cards::{Block, CARDS, ModelCards, card, parse, title};
pub use tools::{NewRun, ObstacleDraft, PlayerAction, SeedRow, Side, Tools, days_in};

use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use ps_view::{CameraRig, Garden, Lens, LensInfo, Origin, Scale, Selection};

/// Speed multipliers in sim hours per real second (§2.2).
pub const SPEEDS: [(&str, f64); 5] = [
    ("1 h/s", 1.0),
    ("6 h/s", 6.0),
    ("1 day/s", 24.0),
    ("3 days/s", 72.0),
    ("1 week/s", 168.0),
];

/// Step buttons, in sim hours (§2.2). A season is 91 days.
pub const STEPS: [(&str, f64); 4] = [
    ("+1 h", 1.0),
    ("+1 day", 24.0),
    ("+1 week", 168.0),
    ("+1 season", 91.0 * 24.0),
];

/// Above this speed (sim hours per second) the view switches to high-speed rendering:
/// daily mean light and smoothed stress (§2.2).
pub const HIGH_SPEED: f64 = 24.0;

/// A request from the UI to the owner of sim time.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeCommand {
    /// Pause or resume.
    TogglePause,
    /// Run at this many sim hours per real second.
    SetSpeed(f64),
    /// Advance this many sim hours as fast as possible, then show the result.
    Step(f64),
    /// Jump the replay to this sim hour (0 to the latest hour reached).
    Seek(f64),
}

/// What the top bar shows, written by the time owner, plus the queue of commands the
/// UI emits.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct TimeControl {
    /// The sim hour on screen (fractional while running).
    pub hour: f64,
    /// The latest hour simulated so far (the end of the replay scrubber).
    pub latest_hour: f64,
    /// The last hour the scenario covers.
    pub end_hour: f64,
    /// Current speed, sim hours per real second.
    pub speed: f64,
    /// True while paused.
    pub paused: bool,
    /// True while the time owner is still working through a step or a seek.
    pub busy: bool,
    /// Date and clock, for example "2026-06-02 14:00".
    pub date: String,
    /// Season name.
    pub season: String,
    /// One line of weather.
    pub weather: String,
    /// Commands queued this frame.
    pub commands: Vec<TimeCommand>,
}

impl TimeControl {
    /// True when the view should use high-speed rendering.
    #[must_use]
    pub fn high_speed(&self) -> bool {
        !self.paused && self.speed > HIGH_SPEED
    }
}

/// Capture requests from the top bar, handled by the app, and its last status line.
#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct CaptureControl {
    /// Save a screenshot this frame.
    pub screenshot: bool,
    /// Export the CSV tables this frame.
    pub export_csv: bool,
    /// What the last capture did, shown in the bottom bar.
    pub status: String,
    /// Clip mode: hide the panels and show only a caption (day, date, pot labels).
    pub clean: bool,
}

/// The UI plugin. Add after `PlantViewPlugin`.
pub struct PlantUiPlugin;

impl Plugin for PlantUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<TimeControl>()
            .init_resource::<Charts>()
            .init_resource::<Inspector>()
            .init_resource::<CaptureControl>()
            .init_resource::<Tools>()
            .init_resource::<ModelCards>()
            .add_systems(EguiPrimaryContextPass, panels)
            .add_systems(PreUpdate, block_camera_under_panels);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn block_camera_under_panels(wants: Option<Res<EguiWantsInput>>, mut rig: ResMut<CameraRig>) {
    rig.input_blocked = wants.is_some_and(|w| w.wants_pointer_input() || w.is_pointer_over_area());
}

/// Warm charcoal panels with a sage accent, opaque so the scene never shows through text
/// (visual design review, 2026-11-05).
fn theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    let panel = egui::Color32::from_rgb(34, 31, 28);
    let sage = egui::Color32::from_rgb(122, 150, 112);
    v.panel_fill = panel;
    v.window_fill = panel;
    v.extreme_bg_color = egui::Color32::from_rgb(24, 22, 20);
    v.faint_bg_color = egui::Color32::from_rgb(42, 39, 35);
    v.window_corner_radius = egui::CornerRadius::same(8);
    v.selection.bg_fill = egui::Color32::from_rgb(78, 98, 70);
    v.selection.stroke.color = egui::Color32::from_rgb(236, 240, 228);
    v.hyperlink_color = sage;
    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(52, 48, 43);
    v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(66, 62, 55);
    v.widgets.active.weak_bg_fill = egui::Color32::from_rgb(78, 98, 70);
    ctx.set_visuals(v);
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn panels(
    mut contexts: EguiContexts,
    mut time: ResMut<TimeControl>,
    mut lens: ResMut<Lens>,
    mut rig: ResMut<CameraRig>,
    garden: Res<Garden>,
    mut charts: ResMut<Charts>,
    mut selection: ResMut<Selection>,
    inspector: Res<Inspector>,
    mut capture: ResMut<CaptureControl>,
    mut tools: ResMut<Tools>,
    mut cards: ResMut<ModelCards>,
    moments: Res<ps_view::Moments>,
    mut themed: Local<bool>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if !*themed {
        theme(ctx);
        *themed = true;
    }
    if capture.clean {
        caption(ctx, &time, &garden, moments.banner.as_deref());
        if rig.insets.iter().any(|v| v.abs() > 0.0) {
            rig.insets = [0.0; 4];
        }
        return Ok(());
    }
    let mut root = egui::Ui::new(
        ctx.clone(),
        "plant-sim-root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let top = egui::Panel::top("time")
        .show(&mut root, |ui| {
            top_bar(
                ui,
                &mut time,
                &garden,
                &mut charts.open,
                &mut capture,
                &mut tools,
            );
        })
        .response
        .rect
        .height();
    let right = egui::Panel::right("lenses")
        .resizable(false)
        .exact_size(300.0)
        .show(&mut root, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                lens_panel(ui, &mut lens, &garden, &mut cards);
                ui.separator();
                inspector::section(
                    ui,
                    &inspector,
                    &mut selection,
                    &garden,
                    *lens,
                    &mut tools,
                    &mut cards,
                );
                ui.separator();
                tools::section(ui, &mut tools, &selection, &garden);
                ui.separator();
                tools::seed_library(ui, &tools);
            });
        })
        .response
        .rect
        .width();
    let left = if cards.open.is_some() {
        egui::Panel::left("model-card")
            .resizable(true)
            .default_size(420.0)
            .show(&mut root, |ui| model_cards::panel(ui, &mut cards))
            .response
            .rect
            .width()
    } else {
        0.0
    };
    let bottom = egui::Panel::bottom("help").show(&mut root, |ui| {
        if charts.open {
            ui.add_space(4.0);
            charts::panel(ui, &charts, &mut time);
            ui.separator();
        }
        ui.horizontal(|ui| {
            ui.weak("Drag to orbit · right-drag to pan · wheel to zoom · F to reframe · P to save a screenshot");
            if !capture.status.is_empty() {
                ui.separator();
                ui.weak(&capture.status);
            }
            if !rig.auto_frame && ui.button("Reframe").clicked() {
                rig.reset_requested = true;
            }
        });
    });
    if tools.new_run.open {
        tools::new_run_window(ctx, &mut tools.new_run);
    }
    if let Some(text) = &moments.banner {
        moment_toast(ctx, text, top + 10.0);
    }
    let insets = [left, top, right, bottom.response.rect.height()];
    if rig
        .insets
        .iter()
        .zip(insets)
        .any(|(a, b)| (a - b).abs() > 0.5)
    {
        rig.insets = insets;
    }
    Ok(())
}

/// A moment's banner (WP20's flowering cue), centered under the top bar.
fn moment_toast(ctx: &egui::Context, text: &str, y: f32) {
    egui::Area::new("moment".into())
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, y))
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(34, 31, 28, 220))
                .corner_radius(6)
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("✿ {text}"))
                            .size(18.0)
                            .color(MOMENT_COLOR),
                    );
                });
        });
}

/// The banner color: a soft flower pink on the dark panels.
const MOMENT_COLOR: egui::Color32 = egui::Color32::from_rgb(240, 214, 232);

/// The clip caption: day and date top left, pot labels under it, and a moment's banner
/// while one runs.
fn caption(ctx: &egui::Context, time: &TimeControl, garden: &Garden, moment: Option<&str>) {
    egui::Area::new("caption".into())
        .fixed_pos(egui::pos2(18.0, 14.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(34, 31, 28, 200))
                .corner_radius(6)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    let day = (time.hour / 24.0).floor();
                    ui.label(
                        egui::RichText::new(format!("Day {day:.0}"))
                            .size(22.0)
                            .strong()
                            .color(egui::Color32::from_rgb(240, 236, 226)),
                    );
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 196, 186),
                        format!("{} · {}", time.date, time.season),
                    );
                    let labels: Vec<&str> = garden
                        .pots
                        .iter()
                        .map(|p| p.label.as_str())
                        .filter(|l| !l.is_empty())
                        .collect();
                    ui.colored_label(
                        egui::Color32::from_rgb(214, 222, 200),
                        labels.join("   |   "),
                    );
                    if let Some(text) = moment {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("✿ {text}"))
                                    .size(17.0)
                                    .color(MOMENT_COLOR),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    }
                });
        });
}

fn top_bar(
    ui: &mut egui::Ui,
    time: &mut TimeControl,
    garden: &Garden,
    charts_open: &mut bool,
    capture: &mut CaptureControl,
    tools: &mut Tools,
) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let day = (time.hour / 24.0).floor();
        ui.heading(format!("Day {day:.0}"));
        ui.label(format!("{} · {}", time.date, time.season));
        ui.separator();
        ui.label(&time.weather);
        ui.separator();
        let labels: Vec<&str> = garden
            .pots
            .iter()
            .map(|p| p.label.as_str())
            .filter(|l| !l.is_empty())
            .collect();
        ui.weak(labels.join("  |  "));
    });
    ui.horizontal(|ui| {
        let play = if time.paused { "▶ Play" } else { "⏸ Pause" };
        if ui.button(play).clicked() {
            time.commands.push(TimeCommand::TogglePause);
        }
        ui.separator();
        for (label, speed) in SPEEDS {
            let selected = (time.speed - speed).abs() < 1e-9;
            if ui.selectable_label(selected, label).clicked() {
                time.commands.push(TimeCommand::SetSpeed(speed));
            }
        }
        ui.separator();
        for (label, hours) in STEPS {
            if ui.button(label).clicked() {
                time.commands.push(TimeCommand::Step(hours));
            }
        }
        ui.separator();
        ui.toggle_value(charts_open, "Charts");
        if ui
            .button("Screenshot")
            .on_hover_text("Save the view as a PNG (P)")
            .clicked()
        {
            capture.screenshot = true;
        }
        if ui
            .button("Export CSV")
            .on_hover_text("Write the daily and organ tables for the day on screen")
            .clicked()
        {
            capture.export_csv = true;
        }
        if tools.enabled
            && ui
                .button("New run…")
                .on_hover_text(
                    "Start again with another scenario, weather, latitude or sowing date",
                )
                .clicked()
        {
            tools.new_run.open = true;
        }
        ui.separator();
        if time.busy {
            ui.colored_label(egui::Color32::from_rgb(200, 210, 170), "Simulating…");
        } else if time.high_speed() {
            ui.colored_label(
                egui::Color32::from_rgb(230, 200, 120),
                "High-speed view: daily means",
            );
        }
    });
    ui.horizontal(|ui| {
        ui.label("Replay");
        let mut day = time.hour / 24.0;
        let latest = (time.latest_hour / 24.0).max(0.0);
        let width = ui.available_width() - 8.0;
        ui.spacing_mut().slider_width = width.max(100.0);
        let r = ui.add(
            egui::Slider::new(&mut day, 0.0..=latest.max(1e-3))
                .show_value(false)
                .clamping(egui::SliderClamping::Always),
        );
        if r.changed() {
            time.commands.push(TimeCommand::Seek(day * 24.0));
        }
    });
    ui.add_space(2.0);
}

/// Linear RGB to an egui color.
fn color32(rgb: [f32; 3]) -> egui::Color32 {
    egui::Color32::from(egui::Rgba::from_rgb(rgb[0], rgb[1], rgb[2]))
}

fn lens_panel(ui: &mut egui::Ui, lens: &mut Lens, garden: &Garden, cards: &mut ModelCards) {
    ui.add_space(6.0);
    ui.heading("Lenses");
    for l in Lens::ALL {
        let name = l.info().map_or("Off (natural colors)", |i| i.name);
        ui.radio_value(lens, l, name);
    }
    if let Some(info) = lens.info() {
        ui.separator();
        legend(ui, &info, garden.stand_in_lenses.contains(lens), cards);
    }
}

fn legend(ui: &mut egui::Ui, info: &LensInfo, stand_in: bool, cards: &mut ModelCards) {
    ui.strong(info.quantity);
    ui.label(format!("Colors: {}. Unit: {}", info.colors, info.unit));
    let width = ui.available_width().min(270.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 16.0), egui::Sense::hover());
    let n = 48_u8;
    for k in 0..n {
        let t0 = f32::from(k) / f32::from(n);
        let t1 = f32::from(k + 1) / f32::from(n);
        let v = info.min + (info.max - info.min) * (t0 + t1) * 0.5;
        let c = info.color(v).unwrap_or([0.5; 3]);
        let r = egui::Rect::from_min_max(
            egui::pos2(rect.left() + rect.width() * t0, rect.top()),
            egui::pos2(rect.left() + rect.width() * t1 + 0.5, rect.bottom()),
        );
        ui.painter().rect_filled(r, 0.0, color32(c));
    }
    ui.horizontal(|ui| {
        ui.label(format!("{}", info.min));
        if let Scale::Diverging { center } = info.scale {
            ui.weak(format!("· white = {center} ·"));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("{}", info.max));
        });
    });
    ui.add_space(4.0);
    ui.label(info.explanation);
    ui.add_space(4.0);
    let tag = match info.origin {
        Origin::Encoded => "Encoded: set directly by a model equation.",
        Origin::Emergent => "Emergent: arises from interacting processes.",
    };
    ui.weak(tag);
    model_cards::link(ui, cards, info.model_card);
    if stand_in {
        ui.add_space(4.0);
        ui.colored_label(
            egui::Color32::from_rgb(230, 180, 90),
            "Stand-in data: this lens shows placeholder values until the full simulation (ps-world) drives the viewer.",
        );
    }
}
