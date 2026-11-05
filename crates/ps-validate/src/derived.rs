//! Derived variables: values computed from the daily rows (and, for one, the organ
//! table) rather than read from a single column.
//!
//! | Name | Definition |
//! | --- | --- |
//! | `thermal_time_cd_b<T>` | Σ max(0, (t_min + t_max)/2 − T) over the days so far, °C·day |
//! | `sla_cm2_per_g` | leaf area / leaf mass, cm² g⁻¹ |
//! | `root_shoot_ratio` | root mass / (leaf + stem + fruit mass) |
//! | `vegetative_shoot_mass_g` | leaf + stem mass, g |
//! | `shoot_mass_g` | leaf + stem + fruit mass (living tops), g |
//! | `harvest_index_tops` | seed mass / living tops (DSSAT HIAD) |
//! | `days_since_emergence` | days since sowing minus its value on the first emerged row |
//! | `nar_g_m2_day` | (total mass now − 7 days ago) / (7 × mean leaf area over those 8 days), g m⁻² d⁻¹ |
//! | `leaf_share_new_veg`, `stem_share_new_veg`, `root_share_new_veg` | share of the last 3 days' new leaf + stem + root mass |
//! | `day` | the sim day (days after sowing) |
//! | `mean_seed_mass_g` | seed mass / seeds, g |
//! | `seeds_per_pod` | seeds / pods |
//! | `harvest_index` | seed mass / (total dry mass ever built − living root mass) |
//! | `leaf_area_per_leaf_cm2` | leaf area / living leaves, cm² |
//! | `height_cm` | height × 100 |
//! | `leaf_area_cm2` | leaf area × 10⁴ |
//! | `mean_internode_cm` | mean length of living stem internodes at the end, cm |
//! | `day_of_<Stage>` or `day_of(<Stage>)` | first day at or past the stage, days after sowing |

use ps_arch::Role;
use ps_physio::Stage;
use ps_world::{DayRow, WorldState};

/// Parses a stage name as written in the Rust enum.
#[must_use]
pub fn parse_stage(name: &str) -> Option<Stage> {
    Some(match name {
        "Sown" => Stage::Sown,
        "Germinated" => Stage::Germinated,
        "Emerged" => Stage::Emerged,
        "FirstTrifoliate" => Stage::FirstTrifoliate,
        "Flowering" => Stage::Flowering,
        "PodSet" => Stage::PodSet,
        "SeedFill" => Stage::SeedFill,
        "Mature" => Stage::Mature,
        _ => return None,
    })
}

/// `day_of_<Stage>` / `day_of(<Stage>)`: `Ok(None)` if `variable` is not a stage date.
///
/// # Errors
/// An unknown stage name, or a stage the run never reaches.
pub fn stage_day(rows: &[&DayRow], variable: &str) -> Result<Option<f64>, String> {
    let name = if let Some(s) = variable.strip_prefix("day_of_") {
        s
    } else if let Some(s) = variable
        .strip_prefix("day_of(")
        .and_then(|s| s.strip_suffix(')'))
    {
        s
    } else {
        return Ok(None);
    };
    let stage = parse_stage(name).ok_or_else(|| format!("unknown stage {name:?}"))?;
    rows.iter()
        .find(|r| r.report.stage.is_some_and(|s| s >= stage))
        .map(|r| Some(f64::from(r.day)))
        .ok_or_else(|| format!("the run never reaches {name}"))
}

fn ratio(num: f64, den: f64, what: &str) -> Result<f64, String> {
    if den > 0.0 {
        Ok(num / den)
    } else {
        Err(format!("{what} is zero"))
    }
}

/// The value of a column or derived variable on row `i` (thermal time sums rows `0..=i`).
///
/// # Errors
/// An unknown variable or a zero denominator.
pub fn row_value(
    state: &WorldState,
    rows: &[&DayRow],
    i: usize,
    variable: &str,
) -> Result<f64, String> {
    let row = rows[i];
    let r = &row.report;
    if let Some(base) = variable.strip_prefix("thermal_time_cd_b") {
        let b: f64 = base
            .parse()
            .map_err(|_| format!("bad base temperature in {variable}"))?;
        return Ok(rows[..=i]
            .iter()
            .map(|x| ((x.env.t_min_c + x.env.t_max_c) * 0.5 - b).max(0.0))
            .sum());
    }
    let shoot = |r: &ps_arch::DayReport| r.leaf_mass + r.stem_mass + r.fruit_mass;
    match variable {
        "day" => Ok(f64::from(row.day)),
        "sla_cm2_per_g" => ratio(r.leaf_area * 1e4, r.leaf_mass, "leaf mass"),
        "root_shoot_ratio" => ratio(r.root_mass, shoot(r), "shoot mass"),
        "vegetative_shoot_mass_g" => Ok(r.leaf_mass + r.stem_mass),
        "shoot_mass_g" => Ok(shoot(r)),
        "harvest_index_tops" => ratio(r.seed_mass, shoot(r), "shoot mass"),
        "mean_seed_mass_g" => ratio(r.seed_mass, f64::from(r.seeds), "seed count"),
        "seeds_per_pod" => ratio(f64::from(r.seeds), f64::from(r.pods), "pod count"),
        "harvest_index" => ratio(r.seed_mass, r.total_dry_mass - r.root_mass, "shoot mass"),
        "leaf_area_per_leaf_cm2" => ratio(r.leaf_area * 1e4, f64::from(r.leaves), "leaf count"),
        "height_cm" => Ok(r.height * 100.0),
        "leaf_area_cm2" => Ok(r.leaf_area * 1e4),
        "days_since_emergence" => {
            let e = rows
                .iter()
                .find(|x| x.report.stage.is_some_and(|s| s >= Stage::Emerged))
                .ok_or("not emerged")?;
            if row.day < e.day {
                return Err("not emerged yet".into());
            }
            Ok(f64::from(r.day) - f64::from(e.report.day))
        }
        "nar_g_m2_day" => {
            if i < 7 {
                return Err("needs 7 earlier days".into());
            }
            let window = &rows[i - 7..=i];
            #[allow(clippy::cast_precision_loss)] // 8 rows
            let la = window.iter().map(|x| x.report.leaf_area).sum::<f64>() / window.len() as f64;
            ratio(
                r.total_dry_mass - rows[i - 7].report.total_dry_mass,
                7.0 * la,
                "leaf area",
            )
        }
        "leaf_share_new_veg" | "stem_share_new_veg" | "root_share_new_veg" => {
            if i < 3 {
                return Err("needs 3 earlier days".into());
            }
            let p = &rows[i - 3].report;
            let (dl, ds, dr) = (
                r.leaf_mass - p.leaf_mass,
                r.stem_mass - p.stem_mass,
                r.root_mass - p.root_mass,
            );
            let part = match variable {
                "leaf_share_new_veg" => dl,
                "stem_share_new_veg" => ds,
                _ => dr,
            };
            ratio(part, dl + ds + dr, "new vegetative mass")
        }
        _ => ps_export::daily_number(state, row, variable)
            .ok_or_else(|| format!("unknown variable {variable:?}")),
    }
}

/// A species-file default by name (`vcmax25`, `pod_set_base`) or `section.field`
/// (`phenology.t_base`): the entry's `default` or `value`.
///
/// # Errors
/// If the parameter is not found or has no number.
pub fn species_parameter(species: &ron::Value, name: &str) -> Result<f64, String> {
    use ron::Value;
    fn key_is(k: &Value, name: &str) -> bool {
        matches!(k, Value::String(s) if s == name)
    }
    fn find<'a>(v: &'a Value, name: &str) -> Option<&'a Value> {
        match v {
            Value::Map(m) => {
                for (k, x) in m.iter() {
                    if key_is(k, name) {
                        return Some(x);
                    }
                }
                m.iter().find_map(|(_, x)| find(x, name))
            }
            Value::Seq(xs) => xs.iter().find_map(|x| find(x, name)),
            Value::Option(Some(x)) => find(x, name),
            _ => None,
        }
    }
    fn number(v: &Value) -> Option<f64> {
        match v {
            Value::Number(n) => Some(n.into_f64()),
            Value::Map(m) => m
                .iter()
                .find(|(k, _)| key_is(k, "default") || key_is(k, "value"))
                .and_then(|(_, x)| number(x)),
            _ => None,
        }
    }
    let mut parts = name.split('.');
    let first = parts.next().unwrap_or_default();
    let mut at = find(species, first).ok_or_else(|| format!("{first} not in the species file"))?;
    for p in parts {
        at = match at {
            Value::Map(m) => m
                .iter()
                .find(|(k, _)| key_is(k, p))
                .map(|(_, x)| x)
                .ok_or_else(|| format!("{name} not in the species file"))?,
            _ => return Err(format!("{name} not in the species file")),
        };
    }
    number(at).ok_or_else(|| format!("{name} has no value"))
}

/// Mean length of the living stem internodes of plant 0 now, cm.
///
/// # Errors
/// If the plant has no living stem internodes.
pub fn mean_internode_cm(state: &WorldState) -> Result<f64, String> {
    let p = state.plants.first().ok_or("no plant")?;
    let a = &p.plant.arena;
    let lengths: Vec<f64> = (0..a.len())
        .filter(|&i| a.alive[i] && p.plant.meta.get(i).is_some_and(|m| m.role == Role::Stem))
        .map(|i| a.size[i].length.0 * 100.0)
        .collect();
    #[allow(clippy::cast_precision_loss)] // internode counts are small
    let n = lengths.len() as f64;
    ratio(lengths.iter().sum(), n, "the stem internode count")
}
