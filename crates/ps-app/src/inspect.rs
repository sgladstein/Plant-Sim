//! Fills the inspector (design plan §5.3, WP17) from the latest snapshot.
//!
//! An organ shows the §3.3 fields with units, and one "why" line generated from its
//! state. A plant shows its stage and the latest daily row: totals, root-zone soil water
//! and the water shortfall. Each field names the model card it comes from.

use std::fmt::Write as _;

use bevy::prelude::{Res, ResMut};
use ps_graph::{OrganId, OrganKind};
use ps_ui::{Field, Inspector, PlayerAction, Tools};
use ps_view::Selection;
use ps_world::{DayRow, PlantEntry};

use crate::sim_source::SimSource;

const GROWTH: &str = "docs/models/organ-growth.md";
const CARBON: &str = "docs/models/carbon-allocation.md";
const LIGHT: &str = "docs/models/light.md";
const PHENOLOGY: &str = "docs/models/thermal-time-and-phenology.md";
const WATER: &str = "docs/models/soil-and-water.md";
const ARCH: &str = "docs/models/herb-architecture.md";
const VARIATION: &str = "docs/models/plant-variation.md";
const GENOME: &str = "docs/models/species-and-genome.md";

/// `FirstTrifoliate` to "first trifoliate".
fn words(debug: &str) -> String {
    let mut s = String::new();
    for (i, c) in debug.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push(' ');
        }
        s.extend(c.to_lowercase());
    }
    s
}

fn capitalized(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// Rebuilds the inspector when the selection or the snapshot changes.
#[allow(clippy::needless_pass_by_value)]
pub fn inspect(
    src: Res<SimSource>,
    selection: Res<Selection>,
    tools: Res<Tools>,
    mut inspector: ResMut<Inspector>,
) {
    let Some(picked) = selection.picked else {
        if inspector.built_for.is_some() {
            *inspector = Inspector::default();
        }
        return;
    };
    let snap = src.snapshot();
    let key = (picked, snap.seq);
    if inspector.built_for == Some(key) {
        return;
    }
    let Some(plant) = src
        .plant_of_pot(picked.pot)
        .and_then(|id| snap.plants.iter().find(|p| p.id == id))
    else {
        *inspector = Inspector::default();
        return;
    };
    let fraction = src.radiation_fraction(picked.pot);
    let name = |id: u32| plant_name(snap, id);
    let (title, fields, why) = if let Some(i) = picked.organ {
        let (title, mut fields, why) = organ(plant, i, src.day_start_hour(), fraction);
        fields.extend(parentage(plant, i, name));
        (title, fields, why)
    } else {
        let row = snap.history.iter().rev().find(|r| r.plant == plant.id);
        let (title, mut fields, why) = whole_plant(
            plant,
            row,
            src.pot_label(picked.pot),
            src.soil_center(picked.pot),
        );
        let lens = snap
            .lens
            .as_deref()
            .and_then(|l| l.plants.iter().find(|x| x.plant == plant.id));
        if let Some(l) = lens {
            fields.extend(competition(l));
        }
        (title, fields, why)
    };
    let pollen = tools.pollen.as_ref().map(|(pot, n)| (*pot, n.as_str()));
    let actions = picked
        .organ
        .map(|i| actions(plant, picked.pot, i, pollen))
        .unwrap_or_default();
    *inspector = Inspector {
        built_for: Some(key),
        title,
        fields,
        why,
        actions,
    };
}

/// "Plant N", numbered as the snapshot lists the plants (the seed library's names).
#[must_use]
pub fn plant_name(snap: &ps_session::Snapshot, id: u32) -> String {
    snap.plants
        .iter()
        .position(|p| p.id == id)
        .map_or_else(|| format!("Plant id {id}"), |i| format!("Plant {}", i + 1))
}

/// Who pollinated a flower, a pod's flower or a seed (WP29).
pub fn parentage(p: &PlantEntry, i: usize, name: impl Fn(u32) -> String) -> Vec<Field> {
    let id = OrganId(u32::try_from(i).unwrap_or(u32::MAX));
    let father = |pollen: Option<u32>| pollen.map_or_else(|| "itself (selfed)".to_owned(), &name);
    if let Some(origin) = p.plant.seed_origin(id) {
        return vec![
            Field::new("Mother", name(p.id), "", GENOME),
            Field::new("Father", father(origin.pollen), "", GENOME),
        ];
    }
    if let Some(f) = p.plant.flowers.iter().find(|f| f.id == id) {
        let by = f
            .pollen
            .map_or_else(|| "not yet: it selfs if left".to_owned(), &name);
        return vec![Field::new("Pollinated by", by, "", GENOME)];
    }
    if let Some(pod) = p.plant.pods.iter().find(|x| x.id == id) {
        return vec![Field::new("Pollen from", father(pod.pollen), "", GENOME)];
    }
    Vec::new()
}

/// What the player can do to organ `i` of the plant in pot `pot` (WP25, WP29): pick a
/// living pod or ear, cut a living shoot organ (the sim refuses the collar and
/// hypocotyl), take pollen from an open flower, or put `pollen` (the pot it came from
/// and its name) on an open flower of another plant.
#[must_use]
pub fn actions(
    p: &PlantEntry,
    pot: usize,
    i: usize,
    pollen: Option<(usize, &str)>,
) -> Vec<(String, PlayerAction)> {
    let id = OrganId(u32::try_from(i).unwrap_or(u32::MAX));
    let mut out = Vec::new();
    let open = p
        .plant
        .flowers
        .iter()
        .any(|f| f.id == id && p.plant.arena.alive.get(i) == Some(&true));
    if open {
        out.push((
            "Collect pollen".to_owned(),
            PlayerAction::CollectPollen { pot },
        ));
    }
    if let Some((from, name)) = pollen
        && from != pot
        && p.plant.check_pollinate(id).is_ok()
    {
        out.push((
            format!("Pollinate with {name}'s pollen"),
            PlayerAction::Pollinate {
                pot,
                organ: i,
                pollen: from,
            },
        ));
    }
    if p.plant.check_harvest(id).is_ok() {
        let what = if p.plant.params.grass.is_some() {
            "Harvest ear"
        } else {
            "Harvest pod"
        };
        out.push((what.to_owned(), PlayerAction::Harvest { pot, organ: i }));
    }
    if p.plant.check_prune(id).is_ok() && p.plant.arena.kind[i] != OrganKind::Root {
        out.push(("Cut here".to_owned(), PlayerAction::Prune { pot, organ: i }));
    }
    out
}

/// Title, fields and why line for organ `i`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn organ(
    p: &PlantEntry,
    i: usize,
    day_start_hour: f64,
    radiation_fraction: f64,
) -> (String, Vec<Field>, String) {
    let a = &p.plant.arena;
    if i >= a.len() {
        return (format!("Organ {i}"), Vec::new(), String::new());
    }
    let id = OrganId(i as u32);
    let kind = a.kind[i];
    let meta = p.plant.meta.get(i);
    let role = meta.map_or_else(
        || words(&format!("{kind:?}")),
        |m| words(&format!("{:?}", m.role)),
    );
    let axis = match (kind, a.axis_order[i]) {
        (OrganKind::Root, 0) => "tap root".to_owned(),
        (OrganKind::Root, n) => format!("root order {n}"),
        (_, 0) => "main stem".to_owned(),
        (_, n) => format!("branch order {n}"),
    };
    let title = format!("{} {i} ({axis})", capitalized(&role));
    let day = |h: u64| ((h as f64 - day_start_hour) / 24.0).floor();
    let mut f = Vec::new();
    f.push(Field::new(
        "Born",
        format!("day {:.0}", day(a.birth[i].0)),
        "",
        GROWTH,
    ));
    if let Some(d) = a.death[i] {
        f.push(Field::new(
            "Died",
            format!("day {:.0}", day(d.0)),
            "",
            GROWTH,
        ));
    }
    f.push(Field::new(
        "Thermal age",
        format!("{:.0}", a.thermal_age[i].0),
        "°C·day",
        PHENOLOGY,
    ));
    let expanded = a.curve[i].fraction(a.expansion_age[i]);
    f.push(Field::new(
        "Expansion",
        format!("{:.0}", expanded * 100.0),
        "% of final size",
        GROWTH,
    ));
    let size = a.size[i];
    if size.length.0 > 0.0 {
        f.push(Field::new(
            "Length",
            format!("{:.1}", size.length.0 * 100.0),
            "cm",
            GROWTH,
        ));
    }
    if size.radius.0 > 0.0 {
        f.push(Field::new(
            "Radius",
            format!("{:.2}", size.radius.0 * 1000.0),
            "mm",
            GROWTH,
        ));
    }
    if size.area.0 > 0.0 {
        f.push(Field::new(
            "Area",
            format!("{:.1}", size.area.0 * 1e4),
            "cm²",
            GROWTH,
        ));
    }
    let mass = a.structural(id).0;
    f.push(Field::new(
        "Structural mass",
        format!("{mass:.4}"),
        "g DM",
        CARBON,
    ));
    f.push(Field::new(
        "Carbon reserve",
        format!("{:.4}", a.reserve[i].0),
        "g C",
        CARBON,
    ));
    if mass > 0.0 {
        f.push(Field::new(
            "Reserve concentration",
            format!("{:.3}", a.reserve[i].0 / mass),
            "g C/g DM",
            CARBON,
        ));
    }
    f.push(Field::new(
        "Nitrogen",
        format!("{:.4}", a.nitrogen[i].0),
        "g N",
        "docs/models/respiration.md",
    ));
    let asked = a.potential_demand[i].0;
    let unmet = a.unmet[i].0;
    f.push(Field::new(
        "Unmet demand",
        format!("{unmet:.4}"),
        "g C",
        CARBON,
    ));
    let leafy = matches!(kind, OrganKind::Leaf | OrganKind::Cotyledon);
    let light = &a.light[i];
    if leafy {
        f.push(Field::new(
            "Sunlit fraction",
            format!("{:.2}", light.sunlit_fraction),
            "0 to 1",
            LIGHT,
        ));
        f.push(Field::new(
            "Diffuse light",
            format!("{:.0}", light.diffuse * 100.0),
            "% of open sky",
            LIGHT,
        ));
        f.push(Field::new(
            "Red:far-red",
            format!("{:.2}", light.rfr),
            "ratio",
            LIGHT,
        ));
    }
    if let Some(m) = meta.filter(|_| leafy) {
        f.push(Field::new(
            "Light index (yesterday)",
            format!("{:.2}", m.light_index),
            "0 to 1",
            ARCH,
        ));
    }
    if let Some(m) = meta.filter(|m| kind == OrganKind::Root && m.multiplicity > 1.0) {
        f.push(Field::new(
            "Roots in cohort",
            format!("{:.0}", m.multiplicity),
            "roots",
            ARCH,
        ));
    }
    let stress = p.plant.stress.0;
    f.push(Field::new(
        "Water stress factor",
        format!("{stress:.2}"),
        "(1 = unstressed)",
        WATER,
    ));

    // Why: the most telling one or two facts.
    let mut why: Vec<String> = Vec::new();
    if let Some(d) = a.death[i] {
        why.push(format!("Shed on day {:.0}.", day(d.0)));
    } else if a.starving_hours[i] > 0 {
        why.push(format!(
            "Starving: its reserve has been below zero for {} h; it dies if that goes on.",
            a.starving_hours[i]
        ));
    }
    if a.alive[i] {
        if leafy {
            let shade = if radiation_fraction < 0.999 {
                format!(
                    ", in a pot under {:.0}% shade cloth",
                    (1.0 - radiation_fraction) * 100.0
                )
            } else {
                String::new()
            };
            why.push(format!(
                "Sees {:.0}% of the open sky's diffuse light{shade}.",
                light.diffuse * radiation_fraction * 100.0
            ));
        }
        if expanded < 0.99 {
            let mut s = format!(
                "Still expanding: {:.0}% of its final size",
                expanded * 100.0
            );
            if asked > 0.0 && unmet / asked > 0.05 {
                let _ = write!(
                    s,
                    ", short of carbon (got {:.0}% of what it asked for)",
                    (1.0 - unmet / asked) * 100.0
                );
            }
            s.push('.');
            why.push(s);
        }
        if stress < 0.95 {
            why.push(format!(
                "Dry soil is closing the stomata (water stress factor {stress:.2})."
            ));
        }
        if why.is_empty() {
            why.push("Fully grown and in carbon balance.".to_owned());
        }
    }
    why.truncate(2);
    (title, f, why.join(" "))
}

/// What sets this plant apart from its siblings (decision 0027): the seed it grew
/// from, how much warmth it needed to come up, where it was sown and which way its
/// seedling faced. `soil_center` is the soil frame's surface center (m), so the sowing
/// spot reads from the middle of the pot or bed.
#[must_use]
pub fn individual(p: &PlantEntry, soil_center: [f64; 2]) -> Vec<Field> {
    let ind = &p.plant.individual;
    let pos = p.plant.collar_position;
    // The sim's azimuth is counterclockwise from east; a compass bearing is clockwise
    // from north.
    let bearing = (90.0 - ind.azimuth.0.to_degrees()).rem_euclid(360.0);
    let mut f = vec![
        Field::new(
            "Seed size",
            format!("{:.0}", ind.seed_factor * 100.0),
            "% of the variety's mean",
            VARIATION,
        ),
        Field::new(
            "Seed mass",
            format!("{:.3}", p.plant.params.seed_mass.0),
            "g DM",
            VARIATION,
        ),
        Field::new(
            "Emergence needs",
            format!("{:.0}", ind.emergence.0),
            "°C·day",
            VARIATION,
        ),
        Field::new(
            "Sown at",
            format!(
                "{:+.1}, {:+.1}",
                (pos[0] - soil_center[0]) * 100.0,
                (pos[1] - soil_center[1]) * 100.0
            ),
            "cm E, N",
            VARIATION,
        ),
        Field::new(
            "Sowing depth",
            format!("{:.1}", -pos[2] * 100.0),
            "cm",
            VARIATION,
        ),
        Field::new(
            "Seedling faces",
            format!("{bearing:.0}"),
            "° from north",
            VARIATION,
        ),
    ];
    if let Some(d) = p.plant.emerged_day {
        f.push(Field::new(
            "Emerged",
            format!("day {d} after sowing"),
            "",
            PHENOLOGY,
        ));
    }
    f
}

/// The plant's light this hour against the open sky and its neighbours (WP26).
#[must_use]
pub fn competition(l: &ps_session::PlantLens) -> Vec<Field> {
    vec![
        Field::new(
            "Absorbed light (hour)",
            format!("{:.0}", l.absorbed_par.0),
            "µmol/m²/s per leaf area",
            LIGHT,
        ),
        Field::new(
            "Share of open sky",
            format!("{:.2}", l.relative_light),
            "0 to 1",
            LIGHT,
        ),
        Field::new(
            "Share of its soil's light",
            format!("{:.0}", l.light_share * 100.0),
            "% of all plants there",
            LIGHT,
        ),
    ]
}

/// Title, fields and why line for the whole plant, from its latest daily row.
#[must_use]
#[allow(clippy::too_many_lines)] // One line per field.
pub fn whole_plant(
    p: &PlantEntry,
    row: Option<&DayRow>,
    pot: &str,
    soil_center: [f64; 2],
) -> (String, Vec<Field>, String) {
    let stage = words(&format!("{:?}", p.plant.phenology.stage));
    let title = format!("Plant {} ({pot})", p.id);
    let mut f = vec![
        Field::new(
            "Species",
            p.species.clone(),
            "",
            "docs/models/species-and-genome.md",
        ),
        Field::new("Stage", stage.clone(), "", PHENOLOGY),
        Field::new(
            "Water stress factor",
            format!("{:.2}", p.plant.stress.0),
            "(1 = unstressed)",
            WATER,
        ),
    ];
    f.extend(individual(p, soil_center));
    let Some(r) = row else {
        return (
            title,
            f,
            format!("At the {stage} stage; no full day simulated yet."),
        );
    };
    let d = &r.report;
    f.extend([
        Field::new(
            "Latest full day",
            format!("day {}", r.day),
            "",
            "docs/models/world-and-schedule.md",
        ),
        Field::new("Height", format!("{:.1}", d.height * 100.0), "cm", ARCH),
        Field::new("Leaves", format!("{}", d.leaves), "count", ARCH),
        Field::new(
            "Leaf area",
            format!("{:.0}", d.leaf_area * 1e4),
            "cm²",
            GROWTH,
        ),
        Field::new("Dry mass", format!("{:.2}", d.dry_mass), "g", CARBON),
        Field::new("  leaves", format!("{:.2}", d.leaf_mass), "g", CARBON),
        Field::new("  stems", format!("{:.2}", d.stem_mass), "g", CARBON),
        Field::new("  roots", format!("{:.2}", d.root_mass), "g", CARBON),
        Field::new(
            "  pods and seeds",
            format!("{:.2}", d.fruit_mass),
            "g",
            CARBON,
        ),
        Field::new("Pods", format!("{}", d.pods), "count", ARCH),
        Field::new("Root length", format!("{:.1}", d.root_length), "m", ARCH),
        Field::new(
            "Root-zone soil water",
            format!("{:.2}", r.env.ftsw),
            "FTSW",
            WATER,
        ),
        Field::new(
            "Net assimilation",
            format!("{:.2}", d.assimilation),
            "g C today",
            CARBON,
        ),
        Field::new(
            "Water taken up",
            format!("{:.1}", d.transpiration * 1000.0),
            "g today",
            WATER,
        ),
        Field::new(
            "Water shortfall",
            format!("{:.1}", d.shortfall * 1000.0),
            "g today",
            WATER,
        ),
        Field::new(
            "Radiation",
            format!("{:.1}", r.env.radiation_mj),
            "MJ/m² today",
            "docs/models/weather-and-sun.md",
        ),
    ]);
    let why = if r.env.ftsw < 0.3 {
        format!(
            "At the {stage} stage. The soil is drying (FTSW {:.2}), so the plant is closing its stomata.",
            r.env.ftsw
        )
    } else if d.feedback_loss > 0.2 {
        format!(
            "At the {stage} stage. It makes more sugar than it can use: {:.0}% of today's potential photosynthesis was held back.",
            d.feedback_loss * 100.0
        )
    } else {
        format!(
            "At the {stage} stage, with {:.0} cm² of leaves and {:.1} g of dry mass.",
            d.leaf_area * 1e4,
            d.dry_mass
        )
    };
    (title, f, why)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every numeric field has a unit (WP17: "every inspector field a unit").
    fn all_numbers_have_units(fields: &[Field]) {
        for f in fields {
            if f.value.trim().parse::<f64>().is_ok() {
                assert!(!f.unit.is_empty(), "{} has no unit", f.label);
            }
            assert!(!f.card.is_empty(), "{} has no model card", f.label);
        }
    }

    #[test]
    fn pods_offer_a_harvest_and_the_collar_offers_nothing() {
        let data = crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"));
        let (mut w, _) = crate::sim_source::load_world(
            &data,
            std::path::Path::new(crate::sim_source::DEFAULT_SCENARIO),
        )
        .unwrap_or_else(|e| unreachable!("{e:#}"));
        w.run_days(70).unwrap_or_else(|e| unreachable!("{e}"));
        let p = &w.state().plants[0];
        let a = &p.plant.arena;
        let pod = (0..a.len())
            .find(|&i| a.kind[i] == OrganKind::Pod && a.alive[i])
            .unwrap_or_else(|| unreachable!("no living pod by day 70"));
        let offered = actions(p, 0, pod, None);
        assert!(
            offered
                .iter()
                .any(|(_, x)| *x == PlayerAction::Harvest { pot: 0, organ: pod })
        );
        assert!(actions(p, 0, p.plant.collar.index(), None).is_empty());
    }

    #[test]
    fn every_organ_and_plant_inspects_with_units() {
        let data = crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"));
        let (mut w, _) = crate::sim_source::load_world(
            &data,
            std::path::Path::new(crate::sim_source::DEFAULT_SCENARIO),
        )
        .unwrap_or_else(|e| unreachable!("{e:#}"));
        w.run_days(40).unwrap_or_else(|e| unreachable!("{e}"));
        for p in &w.state().plants {
            for i in 0..p.plant.arena.len() {
                let (title, fields, why) = organ(p, i, 0.0, 0.5);
                assert!(!title.is_empty() && !fields.is_empty());
                assert!(
                    p.plant.arena.death[i].is_some() || !why.is_empty(),
                    "{title}"
                );
                all_numbers_have_units(&fields);
            }
            let row = w.state().history.iter().rev().find(|r| r.plant == p.id);
            let (_, fields, why) = whole_plant(p, row, "Pot", [0.1, 0.1]);
            assert!(fields.len() > 10 && !why.is_empty());
            all_numbers_have_units(&fields);
        }
    }

    #[test]
    fn each_plant_shows_its_own_seed() {
        let data = crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"));
        let (w, _) = crate::sim_source::load_world(
            &data,
            std::path::Path::new(crate::sim_source::DEFAULT_SCENARIO),
        )
        .unwrap_or_else(|e| unreachable!("{e:#}"));
        let shown: Vec<Vec<Field>> = w
            .state()
            .plants
            .iter()
            .map(|p| individual(p, [0.1, 0.1]))
            .collect();
        for fields in &shown {
            let labels: Vec<&str> = fields.iter().map(|f| f.label.as_str()).collect();
            assert_eq!(
                labels,
                [
                    "Seed size",
                    "Seed mass",
                    "Emergence needs",
                    "Sown at",
                    "Sowing depth",
                    "Seedling faces"
                ]
            );
            all_numbers_have_units(fields);
        }
        assert_ne!(
            shown[0][0].value, shown[1][0].value,
            "two plants, one seed size"
        );
    }

    #[test]
    fn an_open_flower_takes_another_plants_pollen() {
        let data = crate::sim_source::find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"));
        let (mut w, _) = crate::sim_source::load_world(
            &data,
            std::path::Path::new(crate::sim_source::DEFAULT_SCENARIO),
        )
        .unwrap_or_else(|e| unreachable!("{e:#}"));
        let open = |w: &ps_world::World, k: usize| {
            let p = &w.state().plants[k];
            p.plant
                .flowers
                .iter()
                .map(|f| f.id.index())
                .find(|&i| p.plant.check_pollinate(OrganId(i as u32)).is_ok())
        };
        for _ in 0..90 {
            if open(&w, 0).is_some() && w.state().plants[1].plant.has_open_flower() {
                break;
            }
            w.step_hour().unwrap_or_else(|e| unreachable!("{e}"));
            for _ in 0..23 {
                w.step_hour().unwrap_or_else(|e| unreachable!("{e}"));
            }
        }
        let flower = open(&w, 0).unwrap_or_else(|| unreachable!("no open flower"));
        let name = |id: u32| format!("Plant {}", id + 1);
        let p = &w.state().plants[0];
        assert_eq!(
            parentage(p, flower, name)[0].value,
            "not yet: it selfs if left"
        );
        let offered = actions(p, 0, flower, Some((1, "Plant 2")));
        let labels: Vec<&str> = offered.iter().map(|(l, _)| l.as_str()).collect();
        assert!(labels.contains(&"Collect pollen"), "{labels:?}");
        assert!(
            labels.contains(&"Pollinate with Plant 2's pollen"),
            "{labels:?}"
        );
        // Its own pollen is not offered back to it.
        assert!(actions(p, 0, flower, Some((0, "Plant 1"))).len() < offered.len());
        let pollen = w.state().plants[1].id;
        w.apply(ps_world::ActionKind::Pollinate {
            plant: p.id,
            organ: flower as u32,
            pollen,
        })
        .unwrap_or_else(|e| unreachable!("{e}"));
        // Logged actions apply as the next hour steps.
        w.step_hour().unwrap_or_else(|e| unreachable!("{e}"));
        let p = &w.state().plants[0];
        assert_eq!(parentage(p, flower, name)[0].value, "Plant 2");
        all_numbers_have_units(&parentage(p, flower, name));
    }

    #[test]
    fn debug_names_become_words() {
        assert_eq!(words("FirstTrifoliate"), "first trifoliate");
        assert_eq!(capitalized("tap root"), "Tap root");
    }
}
