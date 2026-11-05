//! egui panels (design plan §5.1, §5.2, §5.5; WP17 groundwork).
//!
//! - Top bar: date, season and clock, pause, speed multipliers, step buttons and the
//!   replay scrubber. The bar never changes time itself: it queues [`TimeCommand`]s that
//!   the time owner (the demo driver now, the sim thread in WP16) applies, so the UI
//!   never edits sim state (§5.8).
//! - Right panel: the lens list and the active lens's legend, which is always visible
//!   while a lens is on, with its unit, explanation, encoded-or-emergent tag and model
//!   card.
//! - A custom theme so the panels match the diorama rather than a debug tool (§4).

use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use ps_view::{CameraRig, Garden, Lens, LensInfo, Origin, Scale};

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

/// The UI plugin. Add after `PlantViewPlugin`.
pub struct PlantUiPlugin;

impl Plugin for PlantUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<TimeControl>()
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

#[allow(clippy::needless_pass_by_value)]
fn panels(
    mut contexts: EguiContexts,
    mut time: ResMut<TimeControl>,
    mut lens: ResMut<Lens>,
    mut rig: ResMut<CameraRig>,
    garden: Res<Garden>,
    mut themed: Local<bool>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if !*themed {
        theme(ctx);
        *themed = true;
    }
    let mut root = egui::Ui::new(
        ctx.clone(),
        "plant-sim-root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let top = egui::Panel::top("time")
        .show(&mut root, |ui| top_bar(ui, &mut time, &garden))
        .response
        .rect
        .height();
    let right = egui::Panel::right("lenses")
        .resizable(false)
        .exact_size(300.0)
        .show(&mut root, |ui| lens_panel(ui, &mut lens, &garden))
        .response
        .rect
        .width();
    let bottom = egui::Panel::bottom("help").show(&mut root, |ui| {
        ui.horizontal(|ui| {
            ui.weak("Drag to orbit · right-drag to pan · wheel to zoom · F to reframe");
            if !rig.auto_frame && ui.button("Reframe").clicked() {
                rig.reset_requested = true;
            }
        });
    });
    let insets = [0.0, top, right, bottom.response.rect.height()];
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

fn top_bar(ui: &mut egui::Ui, time: &mut TimeControl, garden: &Garden) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let day = (time.hour / 24.0).floor();
        ui.heading(format!("Day {day:.0}"));
        ui.label(format!("{} · {}", time.date, time.season));
        ui.separator();
        ui.label(&time.weather);
        ui.separator();
        let labels: Vec<&str> = garden.pots.iter().map(|p| p.label.as_str()).collect();
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

fn lens_panel(ui: &mut egui::Ui, lens: &mut Lens, garden: &Garden) {
    ui.add_space(6.0);
    ui.heading("Lenses");
    for l in Lens::ALL {
        let name = l.info().map_or("Off (natural colors)", |i| i.name);
        ui.radio_value(lens, l, name);
    }
    if let Some(info) = lens.info() {
        ui.separator();
        legend(ui, &info, garden.stand_in_lenses.contains(lens));
    }
}

fn legend(ui: &mut egui::Ui, info: &LensInfo, stand_in: bool) {
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
    ui.weak(format!("Model card: {}", info.model_card));
    if stand_in {
        ui.add_space(4.0);
        ui.colored_label(
            egui::Color32::from_rgb(230, 180, 90),
            "Stand-in data: this lens shows placeholder values until the full simulation (ps-world) drives the viewer.",
        );
    }
}
