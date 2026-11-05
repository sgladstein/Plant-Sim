//! Save and load (design plan §6.4, WP26) and the replay scrubber's action markers
//! (§5.5).
//!
//! The UI only raises requests on [`SaveLoad`]; the data source saves the running
//! session without stopping it, lists the saves on disk and loads one.

use bevy::prelude::Resource;
use bevy_egui::egui;

/// One save on disk, for the load dialog.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SaveRow {
    /// Folder name, for example `two_pot_sun_shade-day040-12h`.
    pub name: String,
    /// One line about it: day, date, model version.
    pub summary: String,
}

/// Save and load requests and the load dialog's list.
#[derive(Clone, Debug, Default, Resource)]
pub struct SaveLoad {
    /// Save the running session this frame.
    pub save: bool,
    /// Whether the load dialog is open.
    pub open: bool,
    /// Saves on disk, newest name last; the source fills it when the dialog opens.
    pub saves: Vec<SaveRow>,
    /// Set when the dialog opens so the source refreshes `saves`.
    pub refresh: bool,
    /// The save to load, set by the dialog; the source clears it.
    pub load: Option<String>,
    /// What the last save or load did.
    pub status: String,
}

/// An action on the replay scrubber.
#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    /// Sim hour it applied at.
    pub hour: f64,
    /// What it was, for the hover text.
    pub label: String,
}

/// The load dialog.
pub(crate) fn load_window(ctx: &egui::Context, s: &mut SaveLoad) {
    let mut open = s.open;
    egui::Window::new("Load a save")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(380.0)
        .show(ctx, |ui| {
            if s.saves.is_empty() {
                ui.weak("No saves yet. Save adds one for the moment on screen.");
            }
            egui::ScrollArea::vertical()
                .max_height(320.0)
                .show(ui, |ui| {
                    for row in s.saves.iter().rev() {
                        let r = ui
                            .selectable_label(false, &row.name)
                            .on_hover_text("Load this save; the current run is closed");
                        ui.weak(&row.summary);
                        if r.clicked() {
                            s.load = Some(row.name.clone());
                        }
                    }
                });
            if !s.status.is_empty() {
                ui.separator();
                ui.weak(&s.status);
            }
        });
    s.open = open && s.load.is_none();
}

/// Draws the markers over a slider's rect, `latest` hours wide, and returns the hover
/// text of the marker under the pointer, if any.
pub(crate) fn markers(
    ui: &egui::Ui,
    rect: egui::Rect,
    latest: f64,
    markers: &[Marker],
) -> Option<String> {
    if latest <= 0.0 || markers.is_empty() {
        return None;
    }
    let painter = ui.painter();
    let color = egui::Color32::from_rgb(230, 190, 110);
    let pointer = ui.ctx().pointer_hover_pos();
    let mut hovered: Vec<&str> = Vec::new();
    // The slider's handle insets its track by about its radius.
    let inset = rect.height() * 0.5;
    let (left, right) = (rect.left() + inset, rect.right() - inset);
    for m in markers {
        #[allow(clippy::cast_possible_truncation)] // A fraction in [0, 1].
        let t = (m.hour / latest).clamp(0.0, 1.0) as f32;
        let x = left + (right - left) * t;
        let top = egui::pos2(x, rect.top() - 1.0);
        painter.line_segment(
            [top, egui::pos2(x, rect.top() + rect.height() * 0.35)],
            egui::Stroke::new(2.0, color),
        );
        if let Some(p) = pointer
            && (p.x - x).abs() < 4.0
            && rect.expand(4.0).contains(p)
        {
            hovered.push(&m.label);
        }
    }
    (!hovered.is_empty()).then(|| hovered.join("\n"))
}
