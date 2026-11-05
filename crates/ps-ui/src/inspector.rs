//! The inspector (design plan §5.3, WP17).
//!
//! Clicking an organ shows its state with units and one generated "why" line; clicking a
//! pot's soil shows the plant's totals. The data source fills [`Inspector`] for the
//! current [`Selection`]; the panel draws it, adds the active lens's value for the organ
//! (so a lens can be checked against the numbers), and names each field's model card on
//! hover.

use bevy::prelude::Resource;
use bevy_egui::egui;
use ps_view::{Garden, Lens, Picked, Selection};

/// One row of the inspector.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Field {
    /// Name, for example "Structural mass".
    pub label: String,
    /// Formatted value.
    pub value: String,
    /// Unit, for example "g DM"; empty for counts and names.
    pub unit: String,
    /// The model card the value comes from, for example "docs/models/carbon.md".
    pub card: String,
}

impl Field {
    /// A field.
    #[must_use]
    pub fn new(label: &str, value: String, unit: &str, card: &str) -> Self {
        Self {
            label: label.to_owned(),
            value,
            unit: unit.to_owned(),
            card: card.to_owned(),
        }
    }
}

/// What the inspector shows, written by the data source.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct Inspector {
    /// The selection and snapshot sequence the contents were built for, so the source
    /// rebuilds only when either changes.
    pub built_for: Option<(Picked, u64)>,
    /// Heading, for example "Leaf 14 (trifoliate, main stem)".
    pub title: String,
    /// Rows.
    pub fields: Vec<Field>,
    /// One generated sentence on why the organ is the way it is.
    pub why: String,
}

/// The inspector section of the right panel.
pub(crate) fn section(
    ui: &mut egui::Ui,
    inspector: &Inspector,
    selection: &mut Selection,
    garden: &Garden,
    lens: Lens,
) {
    let Some(picked) = selection.picked else {
        ui.weak(
            "Click a leaf, stem, root or pod to inspect it; click the soil for the whole plant.",
        );
        return;
    };
    ui.horizontal(|ui| {
        ui.heading("Inspector");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button("Close")
                .on_hover_text("Clear the selection")
                .clicked()
            {
                selection.picked = None;
            }
        });
    });
    if inspector.built_for.map(|(p, _)| p) != Some(picked) {
        ui.weak("…");
        return;
    }
    ui.strong(&inspector.title);
    if !inspector.why.is_empty() {
        ui.label(egui::RichText::new(&inspector.why).italics());
    }
    ui.add_space(4.0);
    egui::Grid::new("inspector")
        .num_columns(2)
        .striped(true)
        .spacing([10.0, 3.0])
        .show(ui, |ui| {
            for f in &inspector.fields {
                ui.label(&f.label);
                let text = if f.unit.is_empty() {
                    f.value.clone()
                } else {
                    format!("{} {}", f.value, f.unit)
                };
                let r = ui.label(text);
                if !f.card.is_empty() {
                    r.on_hover_text(format!("Model card: {}", f.card));
                }
                ui.end_row();
            }
            if let (Some(info), Some(organ)) = (lens.info(), picked.organ)
                && let Some(v) = garden
                    .pots
                    .get(picked.pot)
                    .and_then(|p| p.organ_value(lens, organ))
            {
                ui.strong(format!("{} (lens)", info.name));
                ui.strong(format!("{v:.3} {}", info.unit))
                    .on_hover_text(format!("Model card: {}", info.model_card));
                ui.end_row();
            }
        });
}
