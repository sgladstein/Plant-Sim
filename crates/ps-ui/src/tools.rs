//! Garden tools, the seed library and a new-run dialog (design plan §2, WP25).
//!
//! The UI never edits sim state (§5.8): every tool queues a [`PlayerAction`] on [`Tools`]
//! and the data source turns it into a sim action. Actions apply at the tick on screen.
//! A harvest or a cut is offered by the inspector for the selected organ; the tools
//! section stands a stake or places an obstacle at the selected plant's container. The
//! seed library lists harvested lots, filled by the data source. The new-run dialog
//! restarts the sim with another scenario, weather, latitude or sowing date.

use bevy::prelude::Resource;
use bevy_egui::egui;
use ps_view::{Garden, Selection};

/// A player action on the garden, in garden terms (pot index on screen, organ index).
#[derive(Clone, Debug, PartialEq)]
pub enum PlayerAction {
    /// Pick a pod or ear; its seeds go to the seed library.
    Harvest {
        /// Pot (plant) index in the garden.
        pot: usize,
        /// Organ index in the plant.
        organ: usize,
    },
    /// Take pollen from the plant's open flowers to cross another plant with (WP29).
    CollectPollen {
        /// Pot (plant) index in the garden.
        pot: usize,
    },
    /// Put the collected pollen on an open flower: its pod's seeds get that father.
    Pollinate {
        /// Pot (plant) index of the flower's plant in the garden.
        pot: usize,
        /// The flower's organ index in the plant.
        organ: usize,
        /// Pot (plant) index of the pollen plant.
        pollen: usize,
    },
    /// Cut an organ and everything it carries.
    Prune {
        /// Pot (plant) index in the garden.
        pot: usize,
        /// Organ index in the plant.
        organ: usize,
    },
    /// Water the pot's soil now.
    Water {
        /// Pot (plant) index in the garden.
        pot: usize,
        /// Water, mm over the soil surface.
        mm: f64,
    },
    /// Stand a stake or string in the pot's soil.
    AddSupport {
        /// Pot (plant) index in the garden.
        pot: usize,
        /// Position from the soil's center, cm (east, north).
        offset_cm: [f64; 2],
        /// Height, cm.
        height_cm: f64,
        /// Diameter, cm.
        diameter_cm: f64,
    },
    /// Place a box that shades the pot's plants.
    PlaceObstacle {
        /// Pot (plant) index in the garden.
        pot: usize,
        /// Lower corner from the soil's center and surface, cm (east, north, up).
        min_cm: [f64; 3],
        /// Upper corner, cm.
        max_cm: [f64; 3],
        /// Share of light passing through, 0 to 1.
        transmittance: f64,
    },
}

/// Where an obstacle stands relative to the pot.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Side {
    /// A board or wall to the south: shades most of the day at mid latitudes.
    #[default]
    South,
    /// To the east: shades the morning.
    East,
    /// To the west: shades the afternoon.
    West,
    /// To the north: shades almost nothing in the northern hemisphere.
    North,
    /// A horizontal cloth overhead.
    Overhead,
}

impl Side {
    /// All sides, in menu order.
    pub const ALL: [Self; 5] = [
        Self::South,
        Self::East,
        Self::West,
        Self::North,
        Self::Overhead,
    ];

    /// Menu label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::South => "Wall to the south",
            Self::East => "Wall to the east",
            Self::West => "Wall to the west",
            Self::North => "Wall to the north",
            Self::Overhead => "Cloth overhead",
        }
    }
}

/// The obstacle the tools section will place.
#[derive(Clone, Debug, PartialEq)]
pub struct ObstacleDraft {
    /// Where it stands.
    pub side: Side,
    /// Distance from the soil's center to its near face, cm (the cloth's height above
    /// the soil when overhead).
    pub distance_cm: f64,
    /// Width across the pot, cm.
    pub width_cm: f64,
    /// Height of a wall, cm.
    pub height_cm: f64,
    /// Share of light passing through, 0 to 1.
    pub transmittance: f64,
}

impl Default for ObstacleDraft {
    fn default() -> Self {
        Self {
            side: Side::South,
            distance_cm: 40.0,
            width_cm: 100.0,
            height_cm: 80.0,
            transmittance: 0.0,
        }
    }
}

/// Thickness of a wall or cloth, cm.
const THICKNESS_CM: f64 = 3.0;

impl ObstacleDraft {
    /// The box (lower and upper corner from the soil's center and surface, cm).
    #[must_use]
    pub fn corners(&self) -> ([f64; 3], [f64; 3]) {
        let (d, w, h, t) = (
            self.distance_cm.max(0.0),
            0.5 * self.width_cm.max(1.0),
            self.height_cm.max(1.0),
            THICKNESS_CM,
        );
        match self.side {
            Side::South => ([-w, -d - t, 0.0], [w, -d, h]),
            Side::North => ([-w, d, 0.0], [w, d + t, h]),
            Side::East => ([d, -w, 0.0], [d + t, w, h]),
            Side::West => ([-d - t, -w, 0.0], [-d, w, h]),
            Side::Overhead => ([-w, -w, d], [w, w, d + t]),
        }
    }
}

/// One harvested lot, as the seed library shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SeedRow {
    /// Day of the harvest.
    pub day: u32,
    /// Where it came from, for example "Pot 1 plant 1".
    pub source: String,
    /// Species id.
    pub species: String,
    /// Seeds or grains.
    pub seeds: u32,
    /// Dry mass, g.
    pub mass_g: f64,
    /// Whether the seeds were ripe.
    pub mature: bool,
    /// Their parents, for example "selfed" or "× Plant 3" (WP29).
    pub parents: String,
}

/// The new-run dialog's choices. The data source fills the lists and starts the run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NewRun {
    /// Whether the dialog is open.
    pub open: bool,
    /// Scenario files, relative to the data directory.
    pub scenarios: Vec<String>,
    /// Weather choices: preset names, then files under `weather/`.
    pub weathers: Vec<String>,
    /// Selected scenario index.
    pub scenario: usize,
    /// Selected weather index; `None` keeps the scenario's own.
    pub weather: Option<usize>,
    /// Latitude, degrees north.
    pub latitude_deg: f64,
    /// Sowing (start) month, 1 to 12.
    pub month: u32,
    /// Sowing (start) day of the month.
    pub day: u32,
    /// Set by the Start button; the data source clears it.
    pub start: bool,
    /// Set when the selected scenario changes, so the source reloads its defaults.
    pub reload_defaults: bool,
}

/// Tool settings, queued actions and the seed library.
#[derive(Clone, Debug, Resource)]
pub struct Tools {
    /// Actions queued this frame, taken by the data source.
    pub queue: Vec<PlayerAction>,
    /// Stake height, cm.
    pub stake_height_cm: f64,
    /// Stake diameter, cm (under 1 cm draws as a string on a frame).
    pub stake_diameter_cm: f64,
    /// Stake position from the soil's center, cm (east, north).
    pub stake_offset_cm: [f64; 2],
    /// Water per click, mm.
    pub water_mm: f64,
    /// The obstacle to place.
    pub obstacle: ObstacleDraft,
    /// Harvested lots, oldest first.
    pub seed_library: Vec<SeedRow>,
    /// Pollen collected for a hand cross: the pot (plant) index it came from and its
    /// name (WP29).
    pub pollen: Option<(usize, String)>,
    /// The new-run dialog.
    pub new_run: NewRun,
    /// Whether the selected plant can take actions (the sim, not the demo plants).
    pub enabled: bool,
    /// Whether the breeding bench window is open (WP30).
    pub breeding: bool,
}

impl Default for Tools {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            stake_height_cm: 200.0,
            stake_diameter_cm: 1.5,
            stake_offset_cm: [5.0, 0.0],
            water_mm: 10.0,
            obstacle: ObstacleDraft::default(),
            seed_library: Vec::new(),
            pollen: None,
            new_run: NewRun {
                latitude_deg: 45.0,
                month: 5,
                day: 15,
                ..NewRun::default()
            },
            enabled: false,
            breeding: false,
        }
    }
}

/// The tools section of the right panel.
#[allow(clippy::too_many_lines)]
pub(crate) fn section(
    ui: &mut egui::Ui,
    tools: &mut Tools,
    selection: &Selection,
    garden: &Garden,
) {
    egui::CollapsingHeader::new("Garden tools")
        .default_open(true)
        .show(ui, |ui| {
            if !tools.enabled {
                ui.weak("Tools need the simulation (not the --demo plants).");
                return;
            }
            pollen_line(ui, tools);
            let Some(pot) = selection.picked.map(|p| p.pot) else {
                ui.weak("Click a plant or its soil to choose where the tools act.");
                return;
            };
            let name = garden
                .pots
                .get(pot)
                .map(|p| p.label.as_str())
                .filter(|l| !l.is_empty())
                .unwrap_or("the selected plant's soil");
            ui.label(format!("At {name}"));
            ui.horizontal(|ui| {
                if ui.button("Water").clicked() {
                    tools.queue.push(PlayerAction::Water {
                        pot,
                        mm: tools.water_mm,
                    });
                }
                ui.add(
                    egui::DragValue::new(&mut tools.water_mm)
                        .range(1.0..=100.0)
                        .suffix(" mm"),
                );
            });
            ui.add_space(4.0);
            ui.strong("Stake or string");
            egui::Grid::new("stake").num_columns(2).show(ui, |ui| {
                ui.label("Height");
                ui.add(
                    egui::DragValue::new(&mut tools.stake_height_cm)
                        .range(10.0..=400.0)
                        .suffix(" cm"),
                );
                ui.end_row();
                ui.label("Diameter");
                ui.add(
                    egui::DragValue::new(&mut tools.stake_diameter_cm)
                        .range(0.2..=10.0)
                        .speed(0.1)
                        .suffix(" cm"),
                )
                .on_hover_text("Under 1 cm is a string hung from a frame");
                ui.end_row();
                ui.label("East, north");
                ui.horizontal(|ui| {
                    for v in &mut tools.stake_offset_cm {
                        ui.add(egui::DragValue::new(v).range(-150.0..=150.0).suffix(" cm"));
                    }
                });
                ui.end_row();
            });
            if ui
                .button("Stand it")
                .on_hover_text("Climbing plants in this soil can twine on it")
                .clicked()
            {
                tools.queue.push(PlayerAction::AddSupport {
                    pot,
                    offset_cm: tools.stake_offset_cm,
                    height_cm: tools.stake_height_cm,
                    diameter_cm: tools.stake_diameter_cm,
                });
            }
            ui.add_space(4.0);
            ui.strong("Obstacle");
            let o = &mut tools.obstacle;
            egui::ComboBox::from_id_salt("obstacle-side")
                .selected_text(o.side.label())
                .show_ui(ui, |ui| {
                    for s in Side::ALL {
                        ui.selectable_value(&mut o.side, s, s.label());
                    }
                });
            egui::Grid::new("obstacle").num_columns(2).show(ui, |ui| {
                ui.label(if o.side == Side::Overhead {
                    "Height"
                } else {
                    "Distance"
                });
                ui.add(
                    egui::DragValue::new(&mut o.distance_cm)
                        .range(0.0..=500.0)
                        .suffix(" cm"),
                );
                ui.end_row();
                ui.label("Width");
                ui.add(
                    egui::DragValue::new(&mut o.width_cm)
                        .range(5.0..=500.0)
                        .suffix(" cm"),
                );
                ui.end_row();
                if o.side != Side::Overhead {
                    ui.label("Height");
                    ui.add(
                        egui::DragValue::new(&mut o.height_cm)
                            .range(5.0..=500.0)
                            .suffix(" cm"),
                    );
                    ui.end_row();
                }
                ui.label("Light through");
                ui.add(egui::Slider::new(&mut o.transmittance, 0.0..=0.9).fixed_decimals(2));
                ui.end_row();
            });
            if ui
                .button("Place it")
                .on_hover_text("Shades the plants in this soil from now on")
                .clicked()
            {
                let (min_cm, max_cm) = o.corners();
                tools.queue.push(PlayerAction::PlaceObstacle {
                    pot,
                    min_cm,
                    max_cm,
                    transmittance: o.transmittance,
                });
            }
        });
}

/// The pollen in hand, if any, with a button to drop it.
fn pollen_line(ui: &mut egui::Ui, tools: &mut Tools) {
    let Some((_, from)) = &tools.pollen else {
        return;
    };
    let text = format!("Pollen from {from}");
    ui.horizontal(|ui| {
        ui.label(text).on_hover_text(
            "Select an open flower on another plant and choose Pollinate in the inspector",
        );
        if ui.small_button("Drop").clicked() {
            tools.pollen = None;
        }
    });
}

/// The seed library section of the right panel.
pub(crate) fn seed_library(ui: &mut egui::Ui, tools: &Tools) {
    egui::CollapsingHeader::new(format!("Harvested seeds ({})", tools.seed_library.len()))
        .default_open(!tools.seed_library.is_empty())
        .show(ui, |ui| {
            if tools.seed_library.is_empty() {
                ui.weak("Harvest a pod or ear from the inspector to save its seeds here.");
                return;
            }
            let seeds: u32 = tools.seed_library.iter().map(|r| r.seeds).sum();
            let mass: f64 = tools.seed_library.iter().map(|r| r.mass_g).sum();
            ui.label(format!("{seeds} seeds, {mass:.1} g in all"));
            egui::Grid::new("seeds")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Day");
                    ui.strong("From");
                    ui.strong("Seeds");
                    ui.strong("Mass");
                    ui.strong("Parents");
                    ui.end_row();
                    for r in tools.seed_library.iter().rev() {
                        ui.label(r.day.to_string());
                        ui.label(&r.source).on_hover_text(format!(
                            "{}{}",
                            r.species,
                            if r.mature { ", ripe" } else { ", picked green" }
                        ));
                        ui.label(r.seeds.to_string());
                        ui.label(format!("{:.2} g", r.mass_g));
                        ui.label(&r.parents);
                        ui.end_row();
                    }
                });
        });
}

/// The new-run dialog.
pub(crate) fn new_run_window(ctx: &egui::Context, run: &mut NewRun) {
    let mut open = run.open;
    egui::Window::new("New run")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            egui::Grid::new("new-run").num_columns(2).show(ui, |ui| {
                ui.label("Scenario");
                let current = run.scenarios.get(run.scenario).cloned().unwrap_or_default();
                egui::ComboBox::from_id_salt("scenario")
                    .selected_text(current)
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        for (i, s) in run.scenarios.iter().enumerate() {
                            if ui.selectable_label(i == run.scenario, s).clicked() {
                                run.scenario = i;
                                run.reload_defaults = true;
                            }
                        }
                    });
                ui.end_row();
                ui.label("Weather");
                let text = run
                    .weather
                    .and_then(|i| run.weathers.get(i))
                    .map_or("the scenario's own", String::as_str)
                    .to_owned();
                egui::ComboBox::from_id_salt("weather")
                    .selected_text(text)
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(run.weather.is_none(), "the scenario's own")
                            .clicked()
                        {
                            run.weather = None;
                        }
                        for (i, w) in run.weathers.iter().enumerate() {
                            if ui.selectable_label(run.weather == Some(i), w).clicked() {
                                run.weather = Some(i);
                            }
                        }
                    });
                ui.end_row();
                ui.label("Latitude");
                ui.add(
                    egui::Slider::new(&mut run.latitude_deg, -60.0..=60.0)
                        .suffix("° N")
                        .fixed_decimals(1),
                );
                ui.end_row();
                ui.label("Sowing date");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("month")
                        .selected_text(MONTHS[(run.month.clamp(1, 12) - 1) as usize])
                        .show_ui(ui, |ui| {
                            for (m, name) in (1..=12).zip(MONTHS) {
                                ui.selectable_value(&mut run.month, m, name);
                            }
                        });
                    ui.add(egui::DragValue::new(&mut run.day).range(1..=days_in(run.month)));
                });
                ui.end_row();
            });
            ui.add_space(6.0);
            ui.weak("Starts the scenario again from its first hour; the current run is closed.");
            if ui.button("Start").clicked() {
                run.start = true;
            }
        });
    run.open = open && !run.start;
}

/// Month names.
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Days in a month of a non-leap year.
#[must_use]
pub fn days_in(month: u32) -> u32 {
    match month {
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obstacles_stand_on_their_side() {
        let mut o = ObstacleDraft::default();
        let (lo, hi) = o.corners();
        // South: north face at −distance, standing on the soil.
        assert!((hi[1] + 40.0).abs() < 1e-9 && lo[2] == 0.0 && hi[2] > 0.0);
        o.side = Side::Overhead;
        let (lo, hi) = o.corners();
        assert!((lo[2] - 40.0).abs() < 1e-9 && hi[2] > lo[2]);
        for s in Side::ALL {
            o.side = s;
            let (lo, hi) = o.corners();
            assert!(lo.iter().zip(&hi).all(|(a, b)| a < b), "{s:?}");
        }
    }
}
