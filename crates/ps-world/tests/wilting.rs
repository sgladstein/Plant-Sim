//! R3b (decision 0044) done-when: wilting and recovery. In the WP20 wilt-and-recover
//! scenario the right pot's watering stops on day 30: during the dry-down its leaves'
//! midday water potential falls below the turgor loss point and their turgor reaches 0.
//! It is watered again on day 55 at 07:00 (tick 1327), and its leaf turgor is back
//! above 0.5 within 6 hours (Kim and Lee-Stadelmann 1984, moderate stress).

use std::path::{Path, PathBuf};

use ps_graph::OrganKind;
use ps_world::{Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// Turgor loss point of the bush bean, `MPa` (`data/species/bush_bean.ron`).
const TURGOR_LOSS: f64 = -0.9;
/// The tick the dry pot is watered again.
const WATERED: u64 = 1327;

#[test]
fn a_dry_pot_wilts_and_recovers_within_hours() {
    let text =
        std::fs::read_to_string(data().join("scenarios/viewer/wilt_and_recover.ron")).unwrap();
    let scenario = Scenario::from_ron(&text).unwrap();
    let catalog = Catalog::load(&data(), &scenario).unwrap();
    let mut w = World::new(scenario, catalog).unwrap();
    w.run_until(30 * 24).unwrap();
    w.set_lens_recording(true);
    // Lowest leaf turgor and leaf water potential of the dry pot's plant (plant 1)
    // during the dry-down, and the highest leaf turgor in the 6 hours after watering.
    let (mut low_turgor, mut low_psi, mut after) = (f64::MAX, f64::MAX, f64::MIN);
    // The watered plant's leaf turgor in the same 6 hours, for the printout.
    let mut control = f64::MIN;
    while w.tick() < WATERED + 6 {
        w.step_hour().unwrap();
        let tick = w.tick() - 1;
        let lens = w.lens().unwrap();
        let dry = lens.plants.iter().find(|p| p.plant == 1).unwrap();
        let wet = lens.plants.iter().find(|p| p.plant == 0).unwrap();
        if tick < WATERED {
            let arena = &w.state().plants[1].plant.arena;
            // Leaves added at the day's end have no potential until the next solve.
            for id in arena.alive_of(OrganKind::Leaf) {
                if let Some(p) = dry.organ_water_potential.get(id.index()) {
                    low_psi = low_psi.min(p.0);
                }
            }
            low_turgor = low_turgor.min(dry.leaf_turgor);
        } else {
            after = after.max(dry.leaf_turgor);
            control = control.max(wet.leaf_turgor);
        }
    }
    println!(
        "dry-down: lowest leaf ψ {low_psi:.2} MPa, lowest leaf turgor {low_turgor:.2}; \
         after watering: {after:.2} (watered plant {control:.2})"
    );
    assert!(low_psi < TURGOR_LOSS, "lowest leaf ψ {low_psi}");
    assert!(low_turgor <= 1e-9, "lowest leaf turgor {low_turgor}");
    assert!(after > 0.5, "leaf turgor within 6 h of watering {after}");
}
