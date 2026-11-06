//! WP26 done-when: "replay of a saved plot matches its hash". A session on the 30-bean
//! plot takes every WP25 action (obstacle, support, cut, harvest), is saved while it
//! runs, and keeps running to the end. The save, loaded and finished headless, and a
//! replay of the session's log from the scenario, both reach the session's hashes.
//! Also: lens data never changes a hash.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ps_graph::OrganKind;
use ps_session::{Session, SessionOptions, Snapshot, TimeCommand};
use ps_world::{ActionKind, Catalog, ObstacleSpec, Scenario, SupportSpec, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn load(name: &str, days: u32) -> (Scenario, Catalog) {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.duration_days = days;
    let c = Catalog::load(&data(), &s).unwrap();
    (s, c)
}

fn step(session: &Session, hours: f64) -> u64 {
    session.time(TimeCommand::Step(hours));
    session.wait_idle().unwrap()
}

/// The first living organ of `kind` on any plant, newest first within a plant.
fn find(snap: &Snapshot, kind: OrganKind) -> Option<(u32, u32)> {
    snap.plants
        .iter()
        .find_map(|e| e.plant.arena.alive_of(kind).last().map(|id| (e.id, id.0)))
}

#[test]
fn saved_plot_resumes_and_replays_to_the_session_hash() {
    // Bush beans set their first pods in the sixth week (decision 0018).
    const DAYS: u32 = 52;
    let (s, c) = load("plot_30_beans.ron", DAYS);
    let world = World::new(s.clone(), c.clone()).unwrap();
    let session = Session::spawn(world, SessionOptions::default()).unwrap();

    // Day 12: a board wall along the north edge and a stake by the middle row.
    step(&session, 12.0 * 24.0 + 9.0);
    session
        .act_and_wait(ActionKind::PlaceObstacle(ObstacleSpec {
            soil: 0,
            min_cm: (-75.0, 62.0, 0.0),
            max_cm: (75.0, 64.0, 60.0),
            transmittance: 0.0,
        }))
        .unwrap();
    session
        .act_and_wait(ActionKind::AddSupport(SupportSpec {
            soil: 0,
            offset_cm: (0.0, 6.0),
            height_cm: 120.0,
            diameter_cm: 1.5,
        }))
        .unwrap();
    // Day 30: cut a leaf.
    step(&session, 18.0 * 24.0);
    let (plant, leaf) = find(&session.latest(), OrganKind::Leaf).expect("a leaf by day 30");
    session
        .act_and_wait(ActionKind::Prune { plant, organ: leaf })
        .unwrap();
    // Then a day at a time until some plant has a pod, and pick it.
    let mut pod = None;
    while pod.is_none() {
        let t = step(&session, 24.0);
        assert!(t < u64::from(DAYS - 2) * 24, "no pod by tick {t}");
        pod = find(&session.latest(), OrganKind::Pod);
    }
    let (plant, organ) = pod.unwrap();
    let harvest_tick = session
        .act_and_wait(ActionKind::Harvest { plant, organ })
        .unwrap();
    // Actions apply at the start of the hour they are logged for.
    step(&session, 5.0);
    assert_eq!(session.latest().seed_library.len(), 1);

    // Save while running on the wall clock, mid-day, and keep running.
    session.time(TimeCommand::SetSpeed(2000.0));
    std::thread::sleep(Duration::from_millis(30));
    let saved = session.save_ron().unwrap();
    assert!(saved.tick > harvest_tick);
    std::thread::sleep(Duration::from_millis(30));
    session.time(TimeCommand::TogglePause);
    let after = session.wait_idle().unwrap();
    assert!(after > saved.tick, "the save stopped the sim ({after})");
    // Finish the session.
    let end = step(&session, 1e9);
    assert_eq!(end, u64::from(DAYS) * 24);
    let live = session.stop().unwrap();
    assert_eq!(live.state().log.len(), 4);

    // The save, loaded and finished headless.
    let mut loaded = World::from_snapshot_ron(&saved.ron).unwrap();
    assert_eq!(loaded.tick(), saved.tick);
    assert_eq!(loaded.state_hash(), saved.hash);
    loaded.run_to_end().unwrap();
    assert_eq!(loaded.state().hashes, live.state().hashes);
    assert_eq!(loaded.state_hash(), live.state_hash());

    // A replay of the session's log from the scenario.
    let replay = World::replay(s, c, &live.state().log, live.tick()).unwrap();
    assert_eq!(replay.state().hashes, live.state().hashes);
    assert_eq!(replay.state_hash(), live.state_hash());
}

/// A save taken while scrubbed back keeps the recorded future as scheduled actions, and
/// the loaded world catches up to the session.
#[test]
fn save_while_scrubbed_back_keeps_the_timeline() {
    let (s, c) = load("single_pot.ron", 6);
    let session = Session::spawn(
        World::new(s.clone(), c.clone()).unwrap(),
        SessionOptions::default(),
    )
    .unwrap();
    step(&session, 70.0);
    session
        .act_and_wait(ActionKind::Water { soil: 0, mm: 10.0 })
        .unwrap();
    step(&session, 50.0);
    session.time(TimeCommand::Seek(40.0));
    session.wait_idle().unwrap();
    let saved = session.save_ron().unwrap();
    assert_eq!(saved.tick, 40);
    step(&session, 1e9);
    let live = session.stop().unwrap();
    let mut loaded = World::from_snapshot_ron(&saved.ron).unwrap();
    assert_eq!(loaded.state().log.len(), 1, "the later action is kept");
    loaded.run_to_end().unwrap();
    assert_eq!(loaded.state_hash(), live.state_hash());
}

/// Lens data is recorded beside the state: the same run with and without it has the
/// same hashes, and the values hang together.
#[test]
fn lens_data_never_changes_the_hash() {
    let (s, c) = load("plot_30_beans.ron", 22);
    let mut with = World::new(s.clone(), c.clone()).unwrap();
    with.set_lens_recording(true);
    let mut without = World::new(s, c).unwrap();
    // To noon of day 21, then compare.
    let noon = 21 * 24 + 12;
    with.run_until(noon).unwrap();
    without.run_until(noon).unwrap();
    assert!(without.lens().is_none());
    assert_eq!(with.state().hashes, without.state().hashes);
    assert_eq!(with.state_hash(), without.state_hash());

    let lens = with.lens().expect("recorded").clone();
    assert_eq!(lens.tick, noon - 1);
    assert_eq!(lens.plants.len(), 30);
    let share: f64 = lens.plants.iter().map(|p| p.light_share).sum();
    assert!((share - 1.0).abs() < 1e-9, "shares sum to {share}");
    for (p, e) in lens.plants.iter().zip(&with.state().plants) {
        assert_eq!(p.plant, e.id);
        let n = e.plant.arena.len();
        assert_eq!(p.phloem_import.len(), n);
        assert_eq!(p.organ_temperature.len(), n);
        assert_eq!(p.organ_n.len(), n);
        assert!(p.leaf_area.0 > 0.0);
        assert!(
            p.relative_light > 0.0 && p.relative_light < 2.0,
            "plant {} relative light {}",
            p.plant,
            p.relative_light
        );
        // Carbon moves somewhere at noon, and a root grows at its soil layer's
        // temperature.
        assert!(p.phloem_import.iter().any(|f| f.0.abs() > 0.0));
        let root = e.plant.arena.alive_of(OrganKind::Root).next().unwrap();
        let t = p.organ_temperature[root.index()];
        assert!(
            lens.soils[0]
                .layer_temperature
                .iter()
                .any(|l| l.0.to_bits() == t.0.to_bits())
        );
        let leaf = e.plant.arena.alive_of(OrganKind::Leaf).next().unwrap();
        assert!(p.organ_n[leaf.index()] > 0.0);
    }
    // One water potential per soil layer, none wetter than saturation (R3a).
    let psi = &lens.soils[0].layer_water_potential;
    assert_eq!(psi.len(), lens.soils[0].layer_temperature.len());
    assert!(psi.iter().all(|p| p.0 <= 0.0 && p.0.is_finite()), "{psi:?}");
    // The lens reads each plant's own light, so plants differ. (Whether a crowd
    // shades its interior is the model's business, tested on the state in
    // ps-world's neighbors_in_a_plot_take_each_others_light; the canopy is still
    // open at day 21.)
    let first = lens.plants[0].relative_light;
    assert!(
        lens.plants
            .iter()
            .any(|p| p.relative_light.to_bits() != first.to_bits())
    );

    // Stepping without recording drops nothing from the state; a clone keeps the data.
    let mut copy = with.clone();
    copy.set_lens_recording(false);
    assert!(copy.lens().is_none());
    assert!(with.lens().is_some());
}
