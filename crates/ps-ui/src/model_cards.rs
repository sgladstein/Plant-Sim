//! The model-card side panel (design plan §5.7, WP17).
//!
//! Every process has a model card in `docs/models/`. The app carries the same markdown
//! (compiled in, so a shipped build needs no docs folder) and shows it in a left side
//! panel opened from any lens legend or inspector field.

use bevy::prelude::Resource;
use bevy_egui::egui;

macro_rules! cards {
    ($($file:literal),* $(,)?) => {
        /// Every model card: its repo path and its markdown.
        pub const CARDS: &[(&str, &str)] = &[
            $((concat!("docs/models/", $file), include_str!(concat!("../../../docs/models/", $file))),)*
        ];
    };
}

cards!(
    "carbon-allocation.md",
    "climbing.md",
    "gas-exchange.md",
    "grass.md",
    "herb-architecture.md",
    "light.md",
    "nitrogen.md",
    "organ-growth.md",
    "plant-variation.md",
    "random-numbers.md",
    "respiration.md",
    "soil-and-water.md",
    "species-and-genome.md",
    "thermal-time-and-phenology.md",
    "weather-and-sun.md",
    "world-and-schedule.md",
);

/// The markdown of the card at `path` (for example `docs/models/light.md`).
#[must_use]
pub fn card(path: &str) -> Option<&'static str> {
    CARDS.iter().find(|(p, _)| *p == path).map(|(_, md)| *md)
}

/// A card's title: its first `# ` heading, or its file name.
#[must_use]
pub fn title(path: &str) -> String {
    card(path)
        .and_then(|md| md.lines().find_map(|l| l.strip_prefix("# ")))
        .map_or_else(
            || path.rsplit('/').next().unwrap_or(path).to_owned(),
            |t| t.trim().to_owned(),
        )
}

/// Which card the side panel shows, if any.
#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct ModelCards {
    /// The open card's path, for example `docs/models/light.md`.
    pub open: Option<String>,
}

impl ModelCards {
    /// Opens a card, or closes it if it is already open.
    pub fn toggle(&mut self, path: &str) {
        if self.open.as_deref() == Some(path) {
            self.open = None;
        } else {
            self.open = Some(path.to_owned());
        }
    }
}

/// A clickable "Model card: …" link that opens the card in the side panel.
pub(crate) fn link(ui: &mut egui::Ui, cards: &mut ModelCards, path: &str) {
    let shown = cards.open.as_deref() == Some(path);
    let text = format!("Model card: {}", title(path));
    if ui
        .selectable_label(shown, text)
        .on_hover_text(path)
        .clicked()
    {
        cards.toggle(path);
    }
}

/// The side panel's contents: a card picker, then the open card.
pub(crate) fn panel(ui: &mut egui::Ui, cards: &mut ModelCards) {
    let Some(open) = cards.open.clone() else {
        return;
    };
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("model-card")
            .selected_text(title(&open))
            .width(ui.available_width() - 60.0)
            .show_ui(ui, |ui| {
                for (path, _) in CARDS {
                    if ui.selectable_label(open == *path, title(path)).clicked() {
                        cards.open = Some((*path).to_owned());
                    }
                }
            });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("Close").clicked() {
                cards.open = None;
            }
        });
    });
    ui.weak(&open);
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt("model-card-text")
        .show(ui, |ui| match card(&open) {
            Some(md) => markdown(ui, md),
            None => {
                ui.label("This card is not in the app.");
            }
        });
}

/// One block of a card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A heading and its level (1 for `#`).
    Heading(usize, String),
    /// A paragraph, its lines joined.
    Paragraph(String),
    /// A quoted paragraph (`> `).
    Quote(String),
    /// A list item and its indent level.
    Bullet(usize, String),
    /// A table's rows (the separator row dropped), header first.
    Table(Vec<Vec<String>>),
    /// A fenced code block.
    Code(String),
}

/// Splits a model card into blocks. Handles what the cards use: headings, paragraphs,
/// quotes, bullets with continuation lines, pipe tables and fenced code.
#[must_use]
pub fn parse(md: &str) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let mut code: Option<String> = None;
    // True while the last block may take continuation lines.
    let mut open = false;
    for line in md.lines() {
        if let Some(c) = code.as_mut() {
            if line.trim_start().starts_with("```") {
                out.push(Block::Code(code.take().unwrap_or_default()));
            } else {
                if !c.is_empty() {
                    c.push('\n');
                }
                c.push_str(line);
            }
            continue;
        }
        let t = line.trim();
        if t.starts_with("```") {
            code = Some(String::new());
            open = false;
            continue;
        }
        if t.is_empty() {
            open = false;
            continue;
        }
        if let Some(rest) = t.strip_prefix('#') {
            let level = 1 + rest.chars().take_while(|&c| c == '#').count();
            out.push(Block::Heading(
                level,
                rest.trim_start_matches('#').trim().to_owned(),
            ));
            open = false;
            continue;
        }
        if t.starts_with('|') {
            let cells: Vec<String> = t
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_owned())
                .collect();
            let separator = cells
                .iter()
                .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')));
            if let Some(Block::Table(rows)) = out.last_mut().filter(|_| open) {
                if !separator {
                    rows.push(cells);
                }
            } else {
                out.push(Block::Table(vec![cells]));
            }
            open = true;
            continue;
        }
        if let Some(rest) = t.strip_prefix("> ").or_else(|| t.strip_prefix('>')) {
            if let Some(Block::Quote(q)) = out.last_mut().filter(|_| open) {
                q.push(' ');
                q.push_str(rest.trim());
            } else {
                out.push(Block::Quote(rest.trim().to_owned()));
            }
            open = true;
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if let Some(rest) = t
            .strip_prefix("- ")
            .or_else(|| t.strip_prefix("* "))
            .or_else(|| numbered(t))
        {
            out.push(Block::Bullet(indent / 2, rest.to_owned()));
            open = true;
            continue;
        }
        if let Some(Block::Bullet(_, s) | Block::Paragraph(s) | Block::Quote(s)) =
            out.last_mut().filter(|_| open)
        {
            s.push(' ');
            s.push_str(t);
        } else {
            out.push(Block::Paragraph(t.to_owned()));
            open = true;
        }
    }
    if let Some(c) = code {
        out.push(Block::Code(c));
    }
    out
}

/// `1. text` to `text`.
fn numbered(t: &str) -> Option<&str> {
    let digits = t.chars().take_while(char::is_ascii_digit).count();
    (digits > 0)
        .then(|| t[digits..].strip_prefix(". "))
        .flatten()
}

/// Inline text: `code` in monospace, `**bold**` and `[text](link)` shown as text.
fn inline(ui: &egui::Ui, text: &str, base: &egui::TextFormat) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let code_fmt = egui::TextFormat {
        font_id: egui::FontId::monospace(base.font_id.size * 0.95),
        background: ui.visuals().extreme_bg_color,
        color: egui::Color32::from_rgb(214, 222, 200),
        ..base.clone()
    };
    let bold_fmt = egui::TextFormat {
        color: ui.visuals().strong_text_color(),
        ..base.clone()
    };
    for (k, part) in text.split('`').enumerate() {
        if k % 2 == 1 {
            job.append(part, 0.0, code_fmt.clone());
            continue;
        }
        let part = links_as_text(part);
        for (b, piece) in part.split("**").enumerate() {
            let fmt = if b % 2 == 1 { &bold_fmt } else { base };
            job.append(piece, 0.0, fmt.clone());
        }
    }
    job.wrap.max_width = ui.available_width();
    job
}

/// `[text](target)` to `text`.
fn links_as_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find("](").map(|c| open + c) else {
            break;
        };
        let Some(end) = rest[close..].find(')').map(|e| close + e) else {
            break;
        };
        out.push_str(&rest[..open]);
        out.push_str(&rest[open + 1..close]);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

fn markdown(ui: &mut egui::Ui, md: &str) {
    let base = egui::TextFormat {
        font_id: egui::TextStyle::Body.resolve(ui.style()),
        color: ui.visuals().text_color(),
        ..Default::default()
    };
    for (n, block) in parse(md).into_iter().enumerate() {
        match block {
            Block::Heading(1, t) => {
                ui.heading(t);
            }
            Block::Heading(_, t) => {
                ui.add_space(6.0);
                ui.strong(egui::RichText::new(t).size(base.font_id.size * 1.15));
            }
            Block::Paragraph(t) => {
                ui.label(inline(ui, &t, &base));
            }
            Block::Quote(t) => {
                let weak = egui::TextFormat {
                    color: ui.visuals().weak_text_color(),
                    italics: true,
                    ..base.clone()
                };
                ui.label(inline(ui, &t, &weak));
            }
            Block::Bullet(level, t) => {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(8.0 + 14.0 * f32::from(u8::try_from(level).unwrap_or(u8::MAX)));
                    ui.label("•");
                    ui.label(inline(ui, &t, &base));
                });
            }
            Block::Table(rows) => {
                egui::ScrollArea::horizontal()
                    .id_salt(("card-table", n))
                    .show(ui, |ui| {
                        egui::Grid::new(("card-grid", n))
                            .striped(true)
                            .spacing([8.0, 2.0])
                            .show(ui, |ui| {
                                for (r, row) in rows.iter().enumerate() {
                                    for cell in row {
                                        let mut job = inline(ui, cell, &base);
                                        job.wrap.max_width = 220.0;
                                        if r == 0 {
                                            ui.strong(job.text.clone());
                                        } else {
                                            ui.label(job);
                                        }
                                    }
                                    ui.end_row();
                                }
                            });
                    });
            }
            Block::Code(c) => {
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(6)
                    .corner_radius(4)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(c).monospace());
                    });
            }
        }
        ui.add_space(3.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_card_in_docs_is_in_the_app() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/models");
        let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| unreachable!("{e}"))
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| {
                std::path::Path::new(n)
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("md"))
                    && n != "TEMPLATE.md"
            })
            .map(|n| format!("docs/models/{n}"))
            .collect();
        on_disk.sort();
        let compiled: Vec<String> = CARDS.iter().map(|(p, _)| (*p).to_owned()).collect();
        assert_eq!(compiled, on_disk, "add new cards to the cards! list");
    }

    #[test]
    fn every_lens_names_a_card_the_app_has() {
        for l in ps_view::Lens::ALL {
            if let Some(info) = l.info() {
                assert!(card(info.model_card).is_some(), "{}", info.model_card);
            }
        }
    }

    #[test]
    fn cards_parse_into_headings_tables_and_bullets() {
        let md = card("docs/models/plant-variation.md").unwrap_or_else(|| unreachable!());
        let blocks = parse(md);
        assert_eq!(
            blocks[0],
            Block::Heading(1, "Plant-to-plant variation".into())
        );
        assert!(matches!(&blocks[1], Block::Quote(q) if q.ends_with("for a developer.")));
        let table = blocks
            .iter()
            .find_map(|b| match b {
                Block::Table(rows) => Some(rows),
                _ => None,
            })
            .unwrap_or_else(|| unreachable!());
        assert_eq!(table[0], ["Name", "Value", "Unit", "Source"]);
        assert!(table.iter().all(|r| r.len() == 4));
        assert!(blocks.iter().any(
            |b| matches!(b, Block::Bullet(0, t) if t.starts_with("Seed factor") && t.ends_with("· unifoliate_area`."))
        ));
        assert_eq!(
            title("docs/models/plant-variation.md"),
            "Plant-to-plant variation"
        );
    }

    #[test]
    fn inline_links_become_text() {
        assert_eq!(
            links_as_text("see [the plan](../design-plan.md) §5"),
            "see the plan §5"
        );
        assert_eq!(numbered("12. twelve"), Some("twelve"));
        assert_eq!(numbered("2026 was"), None);
    }
}
