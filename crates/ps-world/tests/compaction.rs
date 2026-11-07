//! R5c (decision 0049) done-when: compacted soil. A bush bean in a 1 m loam plot,
//! 60 cm deep, watered 6 mm every other morning.
//! - With a plough pan from 15 to 20 cm (`m2/plough_pan`, 1.2 times the loam's bulk
//!   density) it has more root above the pan and far less in and below it than in the
//!   same plot without the pan (`m2/plough_pan_control`), and its axes in the pan are
//!   thicker than in the same pan without the thickening response.
//! - In a raised bed (`m2/raised_bed`, the top 30 cm loosened to 0.85) it roots deeper
//!   than in a plot compacted to 1.15 throughout (`m2/compacted_plot`).
//! - Digging a strip of the compacted plot on day 20 (`m2/dug_strip`) changes the soil
//!   only in the strip and moves root growth into it.
//! - Runs with digging and trampling replay and resume bit for bit.

use std::path::{Path, PathBuf};

use ps_graph::OrganKind;
use ps_world::{Action, ActionKind, Catalog, Scenario, SoilZone, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn load(name: &str) -> (Scenario, Catalog) {
    let text = std::fs::read_to_string(data().join(format!("scenarios/{name}.ron"))).unwrap();
    let s = Scenario::from_ron(&text).unwrap();
    let c = Catalog::load(&data(), &s).unwrap();
    (s, c)
}

/// The world after `days` of the scenario (its whole run if `None`).
fn run(name: &str, days: Option<u32>) -> World {
    run_with(name, days, |_| {})
}

/// [`run`] with the species' constants changed by `change` first.
fn run_with(name: &str, days: Option<u32>, change: impl Fn(&mut Catalog)) -> World {
    let (mut s, mut c) = load(name);
    if let Some(d) = days {
        s.duration_days = d;
    }
    change(&mut c);
    let mut w = World::new(s, c).unwrap();
    w.run_to_end().unwrap();
    w
}

/// One tenth of a living root organ: its depth and distance east of the plot's
/// center at the piece's middle, m, its length, m, its radius, m, and its axis order.
struct Piece {
    depth: f64,
    east: f64,
    length: f64,
    radius: f64,
    order: u8,
}

/// The first plant's living roots in tenths along each organ from base to tip.
fn pieces(w: &World) -> Vec<Piece> {
    let soil = &w.state().soils[0].grid;
    let (nx, _, _) = soil.dims();
    #[allow(clippy::cast_precision_loss)]
    let center = 0.5 * nx as f64 * soil.cell_size().0;
    let p = &w.state().plants[0].plant;
    let a = &p.arena;
    let mut out = Vec::new();
    for i in 0..a.len() {
        if a.kind[i] != OrganKind::Root || !a.alive[i] {
            continue;
        }
        let (base, tip) = p.positions[i];
        for k in 0..10 {
            let t = (f64::from(k) + 0.5) / 10.0;
            out.push(Piece {
                depth: -(base[2] + (tip[2] - base[2]) * t),
                east: base[0] + (tip[0] - base[0]) * t - center,
                length: a.size[i].length.0 / 10.0,
                radius: a.size[i].radius.0,
                order: a.axis_order[i],
            });
        }
    }
    out
}

/// Root length, m, of the pieces `keep` selects.
fn length(p: &[Piece], keep: impl Fn(&Piece) -> bool) -> f64 {
    p.iter().filter(|x| keep(x)).map(|x| x.length).sum()
}

/// The length-weighted mean radius, mm, of the axes (order 0) between two depths, m.
fn axis_radius(p: &[Piece], top: f64, bottom: f64) -> f64 {
    let axes: Vec<&Piece> = p
        .iter()
        .filter(|x| x.order == 0 && x.depth >= top && x.depth < bottom)
        .collect();
    let l: f64 = axes.iter().map(|x| x.length).sum();
    1000.0 * axes.iter().map(|x| x.length * x.radius).sum::<f64>() / l
}

#[test]
fn a_plough_pan_holds_roots_above_it_and_thickens_them_in_it() {
    // The thickening is read against the same pan with the response off: in the pan
    // the axes that get there are a selection, so the control's radius would not do.
    let (pan, flat, plain) = std::thread::scope(|t| {
        let pan = t.spawn(|| {
            let w = run("m2/plough_pan", None);
            // The pan is short of air at field capacity, but every tip that reaches it
            // keeps its own waterlogging clock: none dies there this season.
            let a = &w.state().plants[0].plant.arena;
            let lost = (0..a.len())
                .filter(|&i| a.kind[i] == OrganKind::Root && a.axis_order[i] == 0 && !a.alive[i])
                .count();
            assert_eq!(lost, 0);
            pieces(&w)
        });
        let flat = t.spawn(|| pieces(&run("m2/plough_pan_control", None)));
        let plain = t.spawn(|| {
            pieces(&run_with("m2/plough_pan", None, |c| {
                for sp in c.species.values_mut() {
                    sp.constants.root_impedance_thickening.value = 0.0;
                }
            }))
        });
        (
            pan.join().unwrap(),
            flat.join().unwrap(),
            plain.join().unwrap(),
        )
    });
    let above = |p: &[Piece]| length(p, |x| x.depth < 0.15);
    let below = |p: &[Piece]| length(p, |x| x.depth >= 0.15);
    let (r_pan, r_plain, r_flat) = (
        axis_radius(&pan, 0.15, 0.20),
        axis_radius(&plain, 0.15, 0.20),
        axis_radius(&flat, 0.15, 0.20),
    );
    println!(
        "day 70 root length above 15 cm: pan {:.1} m, no pan {:.1} m; at 15 cm and below: \
         {:.1} m and {:.1} m; axis radius in the pan's layer {r_pan:.3} mm, {r_plain:.3} \
         without thickening, {r_flat:.3} without the pan",
        above(&pan),
        above(&flat),
        below(&pan),
        below(&flat)
    );
    assert!(above(&pan) > above(&flat));
    assert!(below(&pan) < 0.5 * below(&flat));
    assert!(r_pan > 1.15 * r_plain);
}

#[test]
fn a_raised_bed_roots_deeper_than_compacted_soil() {
    let bed = pieces(&run("m2/raised_bed", None));
    let hard = pieces(&run("m2/compacted_plot", None));
    let deepest = |p: &[Piece]| p.iter().map(|x| x.depth).fold(0.0, f64::max);
    let deep = |p: &[Piece]| length(p, |x| x.depth >= 0.20);
    println!(
        "day 70 deepest root: raised bed {:.1} cm, compacted {:.1} cm; below 20 cm {:.2} m and {:.2} m",
        100.0 * deepest(&bed),
        100.0 * deepest(&hard),
        deep(&bed),
        deep(&hard)
    );
    assert!(deepest(&bed) > deepest(&hard) + 0.05);
    assert!(deep(&bed) > 1.0);
    assert!(deep(&bed) > 10.0 * deep(&hard));
}

#[test]
fn digging_a_strip_changes_the_soil_and_the_roots_there() {
    let dug = run("m2/dug_strip", Some(45));
    let hard = run("m2/compacted_plot", Some(45));
    // The soil differs only inside the strip, 5 to 25 cm east and 30 cm deep, and is
    // still looser there than the trampled soil around it.
    let (g, h) = (&dug.state().soils[0].grid, &hard.state().soils[0].grid);
    let strip = ps_world::zone_cells(
        g,
        &SoilZone {
            soil: 0,
            east_cm: Some((5.0, 25.0)),
            north_cm: None,
            depth_cm: (0.0, 30.0),
        },
    );
    assert!(strip.len() > 100);
    for i in 0..g.len() {
        let (a, b) = (g.bulk_density(i).0, h.bulk_density(i).0);
        if strip.binary_search(&i).is_ok() {
            assert!(a < 0.95 * b, "cell {i}: {a} against {b}");
        } else {
            assert!(
                (a - b).abs() < 1e-12,
                "cell {i} outside the strip: {a} against {b}"
            );
        }
    }
    // More root in the strip; the rest of the root system much as without digging.
    let (pd, ph) = (pieces(&dug), pieces(&hard));
    let in_strip = |x: &Piece| (0.05..0.25).contains(&x.east) && x.depth < 0.30;
    let out = |p: &[Piece]| length(p, |x| !in_strip(x));
    let (sd, sh) = (length(&pd, in_strip), length(&ph, in_strip));
    println!(
        "day 45 root length in the strip: dug {sd:.2} m, undug {sh:.2} m; elsewhere {:.2} m and {:.2} m",
        out(&pd),
        out(&ph)
    );
    assert!(sd > 2.0 * sh);
    assert!((out(&pd) / out(&ph) - 1.0).abs() < 0.2);
}

/// A session that digs and tramples replays from its log and resumes from a snapshot
/// taken after both, each giving the same daily hashes.
#[test]
fn digging_and_trampling_replay_and_resume_bit_for_bit() {
    let (mut s, c) = load("m2/compacted_plot");
    s.duration_days = 30;
    let zone = |east: (f64, f64)| SoilZone {
        soil: 0,
        east_cm: Some(east),
        north_cm: Some((-20.0, 20.0)),
        depth_cm: (0.0, 25.0),
    };
    let mut session = World::new(s.clone(), c.clone()).unwrap();
    session.run_days(12).unwrap();
    session.apply(ActionKind::Dig(zone((-30.0, -5.0)))).unwrap();
    session.run_until(12 * 24 + 7).unwrap();
    session
        .apply(ActionKind::Compact(zone((-20.0, -10.0))))
        .unwrap();
    session.run_days(4).unwrap();
    let snapshot = session.snapshot_ron().unwrap();
    session.run_to_end().unwrap();
    let log: Vec<Action> = session.state().log.clone();
    assert_eq!(log.len(), 2);
    let replayed = World::replay(s, c, &log, session.tick()).unwrap();
    assert_eq!(replayed.state().hashes, session.state().hashes);
    assert_eq!(replayed.state_hash(), session.state_hash());
    let mut resumed = World::from_snapshot_ron(&snapshot).unwrap();
    resumed.run_to_end().unwrap();
    assert_eq!(resumed.state().hashes, session.state().hashes);
    assert_eq!(resumed.state_hash(), session.state_hash());
}
