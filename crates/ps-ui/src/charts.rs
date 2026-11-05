//! The charts panel (design plan §5.4, WP17).
//!
//! Small multiples of the daily time series, one line per plant and quantity, pots
//! overlaid in different line styles (solid for the first pot, dashed for the second).
//! The data source fills [`Charts`] from the same daily rows the CSV export writes; the
//! panel only draws them. Clicking a chart queues a seek to the end of that day, and a
//! vertical line marks the day on screen.

use bevy::prelude::Resource;
use bevy_egui::egui;

use crate::{TimeCommand, TimeControl};

/// One line: a quantity for one plant.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Series {
    /// Legend label, for example "Leaf".
    pub label: String,
    /// Linear RGB.
    pub color: [f32; 3],
    /// Index of the pot (0, 1, ...): sets the line style.
    pub pot: usize,
    /// (day, value) points in day order.
    pub points: Vec<[f64; 2]>,
}

/// One chart.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chart {
    /// Title, for example "Dry mass by organ".
    pub title: String,
    /// Unit of the y axis, for example "g".
    pub unit: String,
    /// Lines.
    pub series: Vec<Series>,
}

/// Everything the charts panel draws, written by the data source.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct Charts {
    /// Whether the panel is open.
    pub open: bool,
    /// Charts, left to right.
    pub charts: Vec<Chart>,
    /// Pot labels, by pot index, for the line-style legend.
    pub pots: Vec<String>,
    /// The sim hour at which day 0 starts (negative when the scenario starts after
    /// midnight), to turn a clicked day into a seek.
    pub day_start_hour: f64,
    /// Number of daily rows the charts were built from, so the source rebuilds them only
    /// when a day is added.
    pub rows: usize,
}

impl Charts {
    /// The sim hour at the end of `day`, clamped to the replay range.
    #[must_use]
    pub fn seek_hour(&self, day: f64, latest_hour: f64) -> f64 {
        (self.day_start_hour + (day.floor() + 1.0) * 24.0).clamp(0.0, latest_hour.max(0.0))
    }

    /// The day shown at sim hour `hour`.
    #[must_use]
    pub fn day_at(&self, hour: f64) -> f64 {
        (hour - self.day_start_hour) / 24.0
    }
}

/// A round axis maximum at or above `v`: 1, 2 or 5 times a power of ten.
#[must_use]
pub fn nice_max(v: f64) -> f64 {
    if !v.is_finite() || v <= 0.0 {
        return 1.0;
    }
    let p = 10f64.powf(v.log10().floor());
    for m in [1.0, 2.0, 5.0, 10.0] {
        if v <= m * p * (1.0 + 1e-9) {
            return m * p;
        }
    }
    10.0 * p
}

/// The point of `points` (sorted by x) whose x is nearest `x`.
#[must_use]
pub fn nearest(points: &[[f64; 2]], x: f64) -> Option<[f64; 2]> {
    let i = points.partition_point(|p| p[0] < x);
    let after = points.get(i);
    let before = i.checked_sub(1).and_then(|j| points.get(j));
    match (before, after) {
        (Some(b), Some(a)) => Some(if x - b[0] <= a[0] - x { *b } else { *a }),
        (Some(p), None) | (None, Some(p)) => Some(*p),
        (None, None) => None,
    }
}

/// A short number for axis labels and tooltips.
fn short(v: f64) -> String {
    let a = v.abs();
    if a >= 100.0 {
        format!("{v:.0}")
    } else if a >= 10.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    }
}

fn color32(rgb: [f32; 3]) -> egui::Color32 {
    egui::Color32::from(egui::Rgba::from_rgb(rgb[0], rgb[1], rgb[2]))
}

/// Most lines a chart's hover tooltip lists (a plot has dozens of plants).
const TOOLTIP_LINES: usize = 12;

/// Pot line styles: solid, dashed, dotted.
fn line(painter: &egui::Painter, points: &[egui::Pos2], stroke: egui::Stroke, pot: usize) {
    if points.len() < 2 {
        return;
    }
    match pot {
        0 => {
            painter.add(egui::Shape::line(points.to_vec(), stroke));
        }
        1 => {
            painter.extend(egui::Shape::dashed_line(points, stroke, 6.0, 4.0));
        }
        _ => {
            painter.extend(egui::Shape::dotted_line(points, stroke.color, 4.0, 1.2));
        }
    }
}

/// The panel body: a legend row and the charts side by side.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn panel(ui: &mut egui::Ui, charts: &Charts, time: &mut TimeControl) {
    ui.horizontal(|ui| {
        ui.strong("Charts");
        ui.weak("per day; click a chart to jump the replay to that day");
        // Many plants (a plot): legend the two named line styles and the rest as one.
        let shown = if charts.pots.len() > 3 {
            2
        } else {
            charts.pots.len()
        };
        for (i, label) in charts.pots.iter().enumerate().take(shown) {
            ui.separator();
            let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 12.0), egui::Sense::hover());
            let y = rect.center().y;
            line(
                ui.painter(),
                &[egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.6, egui::Color32::from_gray(220)),
                i,
            );
            ui.label(if label.is_empty() {
                format!("Plant {}", i + 1)
            } else {
                label.clone()
            });
        }
        if charts.pots.len() > shown {
            ui.separator();
            let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 12.0), egui::Sense::hover());
            let y = rect.center().y;
            line(
                ui.painter(),
                &[egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.6, egui::Color32::from_gray(220)),
                2,
            );
            ui.label(format!("{} more plants", charts.pots.len() - shown));
        }
    });
    if charts.charts.is_empty() {
        ui.weak("No days simulated yet.");
        return;
    }
    let n = charts.charts.len() as f32;
    let gap = 8.0;
    let width = ((ui.available_width() - gap * (n - 1.0)) / n).max(120.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for chart in &charts.charts {
            ui.vertical(|ui| {
                ui.set_width(width);
                one_chart(ui, chart, charts, time, width);
            });
        }
    });
}

#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
fn one_chart(
    ui: &mut egui::Ui,
    chart: &Chart,
    charts: &Charts,
    time: &mut TimeControl,
    width: f32,
) {
    ui.label(egui::RichText::new(format!("{} ({})", chart.title, chart.unit)).small());
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 120.0), egui::Sense::click());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    painter.rect_filled(rect, 4.0, visuals.extreme_bg_color);

    let last_day = chart
        .series
        .iter()
        .filter_map(|s| s.points.last())
        .map(|p| p[0])
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let y_max = nice_max(
        chart
            .series
            .iter()
            .flat_map(|s| s.points.iter())
            .map(|p| p[1])
            .fold(0.0_f64, f64::max),
    );
    let plot = rect.shrink2(egui::vec2(6.0, 8.0));
    let to_screen = |p: [f64; 2]| {
        egui::pos2(
            plot.left() + plot.width() * (p[0] / last_day) as f32,
            plot.bottom() - plot.height() * (p[1] / y_max).clamp(0.0, 1.05) as f32,
        )
    };
    // Grid: a midline and the axis maximum.
    let grid = egui::Stroke::new(1.0, visuals.faint_bg_color);
    painter.hline(plot.x_range(), plot.center().y, grid);
    painter.hline(plot.x_range(), plot.top(), grid);
    let weak = visuals.weak_text_color();
    let font = egui::FontId::proportional(10.0);
    painter.text(
        plot.left_top() + egui::vec2(2.0, 1.0),
        egui::Align2::LEFT_TOP,
        short(y_max),
        font.clone(),
        weak,
    );
    painter.text(
        plot.right_bottom(),
        egui::Align2::RIGHT_BOTTOM,
        format!("day {last_day:.0}"),
        font.clone(),
        weak,
    );

    for s in &chart.series {
        let pts: Vec<egui::Pos2> = s.points.iter().map(|p| to_screen(*p)).collect();
        line(
            &painter,
            &pts,
            egui::Stroke::new(1.6, color32(s.color)),
            s.pot,
        );
    }

    // The day on screen.
    let today = charts.day_at(time.hour);
    if (0.0..=last_day).contains(&today) {
        let x = to_screen([today, 0.0]).x;
        painter.vline(
            x,
            plot.y_range(),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(236, 240, 228)),
        );
    }

    // Hover: values on the nearest day; click: seek there.
    if let Some(pos) = response.hover_pos() {
        let day = (f64::from((pos.x - plot.left()) / plot.width()) * last_day)
            .clamp(0.0, last_day)
            .round();
        let x = to_screen([day, 0.0]).x;
        painter.vline(x, plot.y_range(), egui::Stroke::new(1.0, weak));
        if response.clicked() {
            time.commands
                .push(TimeCommand::Seek(charts.seek_hour(day, time.latest_hour)));
        }
        response.on_hover_ui_at_pointer(|ui| {
            ui.strong(format!("Day {day:.0}"));
            let mut lines = 0;
            for s in &chart.series {
                if let Some(p) = nearest(&s.points, day).filter(|p| (p[0] - day).abs() < 0.5) {
                    lines += 1;
                    if lines > TOOLTIP_LINES {
                        continue;
                    }
                    let pot = match charts.pots.get(s.pot) {
                        Some(l) if !l.is_empty() => l.clone(),
                        _ => format!("Plant {}", s.pot + 1),
                    };
                    ui.colored_label(
                        color32(s.color),
                        format!("{} {}: {} {}", pot, s.label, short(p[1]), chart.unit),
                    );
                }
            }
            if lines > TOOLTIP_LINES {
                ui.weak(format!("and {} more", lines - TOOLTIP_LINES));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_maxima_round_up() {
        assert!((nice_max(0.0) - 1.0).abs() < 1e-12);
        assert!((nice_max(0.7) - 1.0).abs() < 1e-12);
        assert!((nice_max(1.3) - 2.0).abs() < 1e-12);
        assert!((nice_max(37.0) - 50.0).abs() < 1e-9);
        assert!((nice_max(50.0) - 50.0).abs() < 1e-9);
        assert!((nice_max(512.0) - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn nearest_point_by_day() {
        let pts = [[0.0, 1.0], [1.0, 2.0], [3.0, 4.0]];
        assert_eq!(nearest(&pts, 1.4), Some([1.0, 2.0]));
        assert_eq!(nearest(&pts, 2.6), Some([3.0, 4.0]));
        assert_eq!(nearest(&pts, -5.0), Some([0.0, 1.0]));
        assert_eq!(nearest(&pts, 9.0), Some([3.0, 4.0]));
        assert_eq!(nearest(&[], 1.0), None);
    }

    #[test]
    fn a_clicked_day_seeks_to_its_end() {
        let c = Charts {
            day_start_hour: -6.0,
            ..Charts::default()
        };
        // Day 2 ends at 3 × 24 − 6 = 66 sim hours.
        assert!((c.seek_hour(2.3, 1000.0) - 66.0).abs() < 1e-12);
        assert!((c.seek_hour(2.3, 50.0) - 50.0).abs() < 1e-12);
        assert!((c.day_at(66.0) - 3.0).abs() < 1e-12);
    }
}
