//! R5b (decision 0048) done-when: waterlogging. The bush bean in `m2/waterlogged_pot`,
//! a pot with its holes sealed under a rain shelter, is soaked on day 45 and kept wet
//! to day 48; on day 49 its holes are opened. Against `m2/waterlogged_pot_control`
//! (holes, watered to field capacity each morning) it loses most of its fine roots,
//! wilts at midday while its soil is wet, loses nitrate to the air, and roots again,
//! slowly, as the pot drains. The control loses none of its roots.

use std::path::{Path, PathBuf};

use ps_world::{ActionKind, Catalog, DayRow, Fidelity, PotDrainage, Scenario, SoilSpec, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// Turgor loss point of the bush bean, `MPa` (`data/species/bush_bean.ron`).
const TURGOR_LOSS: f64 = -0.9;

/// The plant's daily rows over the scenario.
fn rows(name: &str) -> Vec<DayRow> {
    rows_at(name, None)
}

/// The plant's daily rows over the scenario, at a water-network level if given.
fn rows_at(name: &str, fidelity: Option<Fidelity>) -> Vec<DayRow> {
    rows_with(name, |s| {
        if let Some(f) = fidelity {
            s.fidelity = f;
        }
    })
}

/// The plant's daily rows over the scenario as `edit` leaves it.
fn rows_with(name: &str, edit: impl FnOnce(&mut Scenario)) -> Vec<DayRow> {
    let text = std::fs::read_to_string(data().join(format!("scenarios/{name}.ron"))).unwrap();
    let mut scenario = Scenario::from_ron(&text).unwrap();
    edit(&mut scenario);
    let days = u64::from(scenario.duration_days);
    let catalog = Catalog::load(&data(), &scenario).unwrap();
    let mut w = World::new(scenario, catalog).unwrap();
    w.run_until(days * 24).unwrap();
    w.state().history.clone()
}

fn day(rows: &[DayRow], d: u32) -> &DayRow {
    rows.iter().find(|r| r.day == d).unwrap()
}

#[test]
fn a_waterlogged_pot_loses_its_fine_roots_wilts_and_recovers_slowly() {
    let wet = rows("m2/waterlogged_pot");
    let dry = rows("m2/waterlogged_pot_control");
    let fine = |r: &[DayRow], d| day(r, d).report.fine_root_length;
    let roots = |r: &[DayRow], d| day(r, d).report.root_length;
    // Most fine roots die within the four wet days; the control's keep growing.
    println!(
        "fine roots day 44 → 49: waterlogged {:.1} → {:.1} m, control {:.1} → {:.1} m",
        fine(&wet, 44),
        fine(&wet, 49),
        fine(&dry, 44),
        fine(&dry, 49)
    );
    assert!(fine(&wet, 49) < 0.5 * fine(&wet, 44));
    assert!(fine(&dry, 49) > fine(&dry, 44));
    // Tips stop in the soaked soil while the control's grow; the fine roots live
    // through the first day short of air (under 48 full-deficit hours) and are gone
    // after the second.
    println!(
        "root length day 45 → 46: waterlogged {:.1} → {:.1} m, control {:.1} → {:.1} m",
        roots(&wet, 45),
        roots(&wet, 46),
        roots(&dry, 45),
        roots(&dry, 46)
    );
    assert!(roots(&wet, 46) < 1.01 * roots(&wet, 45));
    assert!(roots(&dry, 46) > 1.05 * roots(&dry, 45));
    assert!(fine(&wet, 46) > 0.9 * fine(&wet, 45));
    assert!(fine(&wet, 48).abs() < 1e-12);
    // Midday wilting while the soil is wet: the waterlogged plant's leaves fall below
    // the turgor loss point, the control's stay above it.
    let low = |r: &[DayRow]| {
        (45..=49)
            .map(|d| day(r, d).report.leaf_psi_min)
            .fold(f64::MAX, f64::min)
    };
    let wet_ftsw = (45..=49)
        .map(|d| day(&wet, d).env.ftsw)
        .fold(f64::MAX, f64::min);
    println!(
        "days 45 to 49: lowest midday leaf ψ {:.2} against {:.2} MPa, soil FTSW at least {wet_ftsw:.2}",
        low(&wet),
        low(&dry)
    );
    assert!(
        wet_ftsw > 0.99,
        "the waterlogged soil dried: FTSW {wet_ftsw}"
    );
    assert!(low(&wet) < TURGOR_LOSS);
    assert!(low(&dry) > TURGOR_LOSS);
    // Nitrate lost to the air in the wet pot only.
    let last = wet.last().unwrap().day;
    assert!(day(&wet, last).env.denitrified_n_total_g > 0.0);
    assert!(day(&dry, last).env.denitrified_n_total_g.abs() < 1e-15);
    // Slow recovery: new roots after the soil at the collar has air again, but by the
    // end the plant has well under the control's root. Since the pot drains the day
    // its holes open (decision 0050) its laterals and axis tips outlive the soak, so it
    // ends at about two thirds of the control's root (a fifth before).
    let least = (50..=last).map(|d| roots(&wet, d)).fold(f64::MAX, f64::min);
    println!(
        "root length: least after the soak {least:.1} m, day {last} {:.1} m (control {:.1} m)",
        roots(&wet, last),
        roots(&dry, last)
    );
    assert!(roots(&wet, last) > 1.1 * least);
    assert!(roots(&wet, last) < 0.75 * roots(&dry, last));
}

/// The simplified water network (decision 0045) makes a new summary within the day
/// once the root cells' hypoxia moves (decision 0048), so it wilts on the same days as
/// the detailed one. With only the daily summary it missed the first day's wilting
/// (−0.77 against −1.01 `MPa` on day 45).
#[test]
fn a_simplified_waterlogged_pot_wilts_like_the_detailed_one() {
    let detailed = rows("m2/waterlogged_pot");
    let simple = rows_at("m2/waterlogged_pot", Some(Fidelity::Simplified));
    for d in 45..=53 {
        let (a, b) = (
            day(&detailed, d).report.leaf_psi_min,
            day(&simple, d).report.leaf_psi_min,
        );
        println!("day {d}: lowest leaf ψ detailed {a:.3}, simplified {b:.3} MPa");
        assert!((a - b).abs() < 0.1, "day {d}: {a} against {b}");
    }
}

/// The waterlogged pot in sand, sealed as in `m2/waterlogged_pot` and with its holes
/// open throughout (Code review of #150). Sand drains through holes, so there the
/// done-when's "the same plant in a pot with holes does not" holds as written: the
/// sealed pot loses most of its fine roots and wilts, the pot with holes keeps its fine
/// roots and its leaves. (A 24 cm loam pot stays below 10% air at its base after a soak
/// even with holes, decision 0048.)
#[test]
fn in_sand_a_pot_with_holes_drains_and_keeps_its_roots() {
    let sand = |s: &mut Scenario| {
        if let Some(SoilSpec::Pot { texture, .. }) = s.soils.first_mut() {
            *texture = ps_soil::Texture::Sand;
        }
    };
    let sealed = rows_with("m2/waterlogged_pot", sand);
    let holes = rows_with("m2/waterlogged_pot", |s| {
        sand(s);
        if let Some(SoilSpec::Pot { drainage, .. }) = s.soils.first_mut() {
            *drainage = PotDrainage::Holes;
        }
        s.events
            .retain(|e| !matches!(e.kind, ActionKind::SetDrainage { .. }));
    });
    let fine = |r: &[DayRow], d| day(r, d).report.fine_root_length;
    let low = |r: &[DayRow]| {
        (45..=49)
            .map(|d| day(r, d).report.leaf_psi_min)
            .fold(f64::MAX, f64::min)
    };
    println!(
        "sand, fine roots day 44 → 49: sealed {:.2} → {:.2} m, holes {:.2} → {:.2} m; lowest leaf ψ {:.2} and {:.2} MPa",
        fine(&sealed, 44),
        fine(&sealed, 49),
        fine(&holes, 44),
        fine(&holes, 49),
        low(&sealed),
        low(&holes)
    );
    assert!(fine(&sealed, 49) < 0.5 * fine(&holes, 49));
    assert!(fine(&holes, 49) > fine(&holes, 44));
    assert!(low(&sealed) < TURGOR_LOSS);
    assert!(low(&holes) > TURGOR_LOSS);
}
