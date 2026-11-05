//! CSV export of a world (design plan §6.6, WP10): the daily time series, the
//! per-organ table, and the daily hashes. Plain text with a header row, comma
//! separated, `.` decimal point, numbers in Rust's shortest round-trip form so a value
//! read back is bit-identical. Column names carry their units.

use std::fmt::Write as _;

use ps_world::{DayRow, WorldState};

/// Columns of [`daily_csv`], in order.
pub const DAILY_COLUMNS: [&str; 32] = [
    "day",
    "plant",
    "soil",
    "stage",
    "days_since_sowing",
    "main_nodes",
    "branches",
    "leaves",
    "leaf_area_m2",
    "racemes",
    "flowers",
    "pods",
    "seeds",
    "root_axes",
    "lateral_cohorts",
    "root_length_m",
    "root_mass_g",
    "leaf_mass_g",
    "stem_mass_g",
    "fruit_mass_g",
    "seed_mass_g",
    "dry_mass_g",
    "total_dry_mass_g",
    "feedback_loss",
    "height_m",
    "shortfall_kg",
    "t_min_c",
    "t_max_c",
    "radiation_mj_m2",
    "rain_mm",
    "ftsw",
    "soil_water_kg",
];

/// Columns of [`organs_csv`], in order.
pub const ORGAN_COLUMNS: [&str; 22] = [
    "plant",
    "organ",
    "kind",
    "role",
    "parent",
    "axis_order",
    "alive",
    "birth_h",
    "death_h",
    "structural_g",
    "reserve_gc",
    "length_m",
    "radius_m",
    "area_m2",
    "base_x",
    "base_y",
    "base_z",
    "tip_x",
    "tip_y",
    "tip_z",
    "multiplicity",
    "light_index",
];

fn header(cols: &[&str]) -> String {
    let mut s = cols.join(",");
    s.push('\n');
    s
}

fn soil_id(state: &WorldState, plant: u32) -> u32 {
    state
        .plants
        .get(plant as usize)
        .and_then(|p| state.soils.get(p.soil))
        .map_or(u32::MAX, |s| s.id)
}

fn daily_row(out: &mut String, state: &WorldState, row: &DayRow) {
    let r = &row.report;
    let e = &row.env;
    let stage = r.stage.map_or_else(String::new, |s| format!("{s:?}"));
    // Writing to a String cannot fail.
    let _ = writeln!(
        out,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        row.day,
        row.plant,
        soil_id(state, row.plant),
        stage,
        r.day,
        r.main_nodes,
        r.branches,
        r.leaves,
        r.leaf_area,
        r.racemes,
        r.flowers,
        r.pods,
        r.seeds,
        r.root_axes,
        r.lateral_cohorts,
        r.root_length,
        r.root_mass,
        r.leaf_mass,
        r.stem_mass,
        r.fruit_mass,
        r.seed_mass,
        r.dry_mass,
        r.total_dry_mass,
        r.feedback_loss,
        r.height,
        r.shortfall,
        e.t_min_c,
        e.t_max_c,
        e.radiation_mj,
        e.rain_mm,
        e.ftsw,
        e.soil_water_kg,
    );
}

/// The daily time series: one row per day per plant (§3.2 daily step 7).
#[must_use]
pub fn daily_csv(state: &WorldState) -> String {
    let mut out = header(&DAILY_COLUMNS);
    for row in &state.history {
        daily_row(&mut out, state, row);
    }
    out
}

/// The per-organ table now: every organ ever built, living or dead, for every plant.
#[must_use]
pub fn organs_csv(state: &WorldState) -> String {
    let mut out = header(&ORGAN_COLUMNS);
    for p in &state.plants {
        let plant = &p.plant;
        let a = &plant.arena;
        for i in 0..a.len() {
            let meta = plant.meta.get(i);
            let (base, tip) = plant.positions.get(i).copied().unwrap_or_default();
            let size = a.size[i];
            let parent = a.parent[i].map_or_else(String::new, |x| x.0.to_string());
            let death = a.death[i].map_or_else(String::new, |h| h.0.to_string());
            let role = meta.map_or_else(String::new, |m| format!("{:?}", m.role));
            let _ = writeln!(
                out,
                "{},{},{:?},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                p.id,
                i,
                a.kind[i],
                role,
                parent,
                a.axis_order[i],
                a.alive[i],
                a.birth[i].0,
                death,
                a.structural_all()[i].0,
                a.reserve[i].0,
                size.length.0,
                size.radius.0,
                size.area.0,
                base[0],
                base[1],
                base[2],
                tip[0],
                tip[1],
                tip[2],
                meta.map_or(1.0, |m| m.multiplicity),
                meta.map_or(1.0, |m| m.light_index),
            );
        }
    }
    out
}

/// The daily hashes: `day,hash` with the hash as 16 hex digits.
#[must_use]
pub fn hashes_csv(state: &WorldState) -> String {
    let mut out = String::from("day,hash\n");
    for (d, h) in state.hashes.iter().enumerate() {
        let _ = writeln!(out, "{d},{h:016x}");
    }
    out
}

/// The number in one column of a daily row, by [`DAILY_COLUMNS`] name; the stage is
/// its ordinal (0 = sown, 7 = mature, −1 before sowing). `None` for an unknown name.
#[must_use]
pub fn daily_number(state: &WorldState, row: &DayRow, column: &str) -> Option<f64> {
    let r = &row.report;
    let e = &row.env;
    Some(match column {
        "day" => f64::from(row.day),
        "plant" => f64::from(row.plant),
        "soil" => f64::from(soil_id(state, row.plant)),
        "stage" => r.stage.map_or(-1.0, |s| f64::from(s as u8)),
        "days_since_sowing" => f64::from(r.day),
        "main_nodes" => f64::from(r.main_nodes),
        "branches" => f64::from(r.branches),
        "leaves" => f64::from(r.leaves),
        "leaf_area_m2" => r.leaf_area,
        "racemes" => f64::from(r.racemes),
        "flowers" => f64::from(r.flowers),
        "pods" => f64::from(r.pods),
        "seeds" => f64::from(r.seeds),
        "root_axes" => f64::from(r.root_axes),
        "lateral_cohorts" => f64::from(r.lateral_cohorts),
        "root_length_m" => r.root_length,
        "root_mass_g" => r.root_mass,
        "leaf_mass_g" => r.leaf_mass,
        "stem_mass_g" => r.stem_mass,
        "fruit_mass_g" => r.fruit_mass,
        "seed_mass_g" => r.seed_mass,
        "dry_mass_g" => r.dry_mass,
        "total_dry_mass_g" => r.total_dry_mass,
        "feedback_loss" => r.feedback_loss,
        "height_m" => r.height,
        "shortfall_kg" => r.shortfall,
        "t_min_c" => e.t_min_c,
        "t_max_c" => e.t_max_c,
        "radiation_mj_m2" => e.radiation_mj,
        "rain_mm" => e.rain_mm,
        "ftsw" => e.ftsw,
        "soil_water_kg" => e.soil_water_kg,
        _ => return None,
    })
}
