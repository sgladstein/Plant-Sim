//! CSV export of a world (design plan §6.6, WP10): the daily time series, the
//! per-organ table, and the daily hashes. Plain text with a header row, comma
//! separated, `.` decimal point, numbers in Rust's shortest round-trip form so a value
//! read back is bit-identical. Column names carry their units.

use std::fmt::Write as _;

use ps_world::{DayRow, WorldState};

/// Columns of [`daily_csv`], in order.
pub const DAILY_COLUMNS: [&str; 56] = [
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
    "soil_mineral_n_g",
    "plant_n_g",
    "n_uptake_g",
    "n_fixed_g",
    "n_mined_g",
    "n_unmet_g",
    "n_status",
    "nodule_mass_g",
    "fixation_c_g",
    "assimilation_gc",
    "transpiration_kg",
    "hypocotyl_diameter_mm",
    "tap_root_diameter_mm",
    "fine_root_length_m",
    "storage_mass_g",
    "stem_structural_mass_g",
    "root_structural_mass_g",
    "exudation_gc",
    "leaf_psi_predawn_mpa",
    "soil_psi_predawn_mpa",
    "leaf_psi_min_mpa",
    "leaf_turgor_predawn",
    "root_conductance_kg_h_mpa",
    "uptake_depth_m",
];

/// Columns of [`organs_csv`], in order.
pub const ORGAN_COLUMNS: [&str; 25] = [
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
    "nitrogen_g",
    "secondary_g",
    "storage_gc",
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
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
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
        e.soil_mineral_n_g,
        r.nitrogen,
        r.n_uptake,
        r.n_fixed,
        r.n_mined,
        r.n_unmet,
        r.n_status,
        r.nodule_mass,
        r.fixation_carbon,
        r.assimilation,
        r.transpiration,
        r.hypocotyl_diameter * 1e3,
        r.tap_root_diameter * 1e3,
        r.fine_root_length,
        r.storage_mass,
        r.stem_structural_mass,
        r.root_structural_mass,
        r.exudation,
        r.leaf_psi_predawn,
        r.soil_psi_predawn,
        r.leaf_psi_min,
        r.leaf_turgor_predawn,
        r.root_conductance,
        r.uptake_depth,
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
                "{},{},{:?},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
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
                a.nitrogen[i].0,
                a.secondary(ps_graph::OrganId(u32::try_from(i).unwrap_or(u32::MAX)))
                    .0,
                a.storage(ps_graph::OrganId(u32::try_from(i).unwrap_or(u32::MAX)))
                    .0,
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
        "soil_mineral_n_g" => e.soil_mineral_n_g,
        "plant_n_g" => r.nitrogen,
        "n_uptake_g" => r.n_uptake,
        "n_fixed_g" => r.n_fixed,
        "n_mined_g" => r.n_mined,
        "n_unmet_g" => r.n_unmet,
        "n_status" => r.n_status,
        "nodule_mass_g" => r.nodule_mass,
        "fixation_c_g" => r.fixation_carbon,
        "assimilation_gc" => r.assimilation,
        "transpiration_kg" => r.transpiration,
        "hypocotyl_diameter_mm" => r.hypocotyl_diameter * 1e3,
        "tap_root_diameter_mm" => r.tap_root_diameter * 1e3,
        "fine_root_length_m" => r.fine_root_length,
        "storage_mass_g" => r.storage_mass,
        "stem_structural_mass_g" => r.stem_structural_mass,
        "root_structural_mass_g" => r.root_structural_mass,
        "exudation_gc" => r.exudation,
        "leaf_psi_predawn_mpa" => r.leaf_psi_predawn,
        "soil_psi_predawn_mpa" => r.soil_psi_predawn,
        "leaf_psi_min_mpa" => r.leaf_psi_min,
        "leaf_turgor_predawn" => r.leaf_turgor_predawn,
        "root_conductance_kg_h_mpa" => r.root_conductance,
        "uptake_depth_m" => r.uptake_depth,
        _ => return None,
    })
}
