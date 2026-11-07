use super::*;
use proptest::prelude::*;

const T20: Celsius = Celsius(20.0);

fn dry_hour() -> SoilHour {
    SoilHour {
        water_in: Millimeters(0.0),
        surface_radiation: WPerM2(0.0),
        t_air: T20,
        strip: None,
    }
}

fn column(texture: Texture, nz: usize) -> SoilGrid {
    SoilGrid::new_box(1, 1, nz, Meters(0.02), texture, T20).unwrap()
}

fn closed(texture: Texture, nz: usize) -> SoilGrid {
    let mut g = column(texture, nz);
    g.set_drainage(Drainage::NoHoles);
    g
}

/// Runs `hours` dry hours, asserting closure every hour; returns each hour's drainage.
fn drain_hours(g: &mut SoilGrid, hours: usize) -> Vec<f64> {
    (0..hours)
        .map(|_| {
            let l = g.step_hour(&dry_hour());
            assert!(l.relative_residual() < 1e-9, "{l:?}");
            l.drained.0
        })
        .collect()
}

/// R5a done-when: a soaked loam plot drains over days, not hours. From saturation, a
/// 40 cm column of 5 cm cells sheds tens of mm on the first day, under 5 mm a day by
/// the third and still a little after a week; the bucket rules had it at field
/// capacity within eight hours. Every hour closes.
#[test]
fn soaked_loam_drains_over_days() {
    let mut g = SoilGrid::new_box(1, 1, 8, Meters(0.05), Texture::Loam, T20).unwrap();
    let hy = Texture::Loam.hydraulics();
    g.fill_to(hy.saturation);
    let start = g.stored_water();
    let hours = drain_hours(&mut g, 24 * 8);
    // kg over 25 cm² is mm × 0.0025.
    let mm = |d: usize| hours[24 * d..24 * (d + 1)].iter().sum::<f64>() / 0.0025;
    assert!(mm(0) > 15.0, "day 1 {}", mm(0));
    assert!(
        mm(2) < 5.0 && mm(1) > mm(2),
        "days 2, 3: {} {}",
        mm(1),
        mm(2)
    );
    assert!(mm(7) > 0.1 && mm(7) < mm(2), "day 8 {}", mm(7));
    // Still wetter than field capacity: the slow tail of drainage.
    let mean = (0..8).map(|i| g.theta(i)).sum::<f64>() / 8.0;
    assert!(mean > hy.field_capacity && mean < hy.saturation, "{mean}");
    let out: f64 = hours.iter().sum();
    assert!(((start - g.stored_water()).0 - out).abs() < 1e-12);
}

/// A closed column resting at the hydrostatic profile (the potential rising by the
/// height) stays put.
#[test]
fn a_column_at_equilibrium_stays_put() {
    for texture in [Texture::Loam, Texture::Sand, Texture::Clay] {
        let mut g = closed(texture, 10);
        let r = texture.retention();
        for z in 0..10 {
            // −50 kPa at the top, wetter by 0.2 kPa for each 2 cm cell down.
            #[allow(clippy::cast_precision_loss)]
            let psi = KPa(-50.0 + 0.02 * flow::KPA_PER_M_WATER * z as f64);
            g.water[z] = g.capacity(r.theta(psi));
        }
        let before: Vec<f64> = (0..10).map(|i| g.theta(i)).collect();
        drain_hours(&mut g, 48);
        for (i, b) in before.iter().enumerate() {
            assert!((g.theta(i) - b).abs() < 1e-9, "{texture:?} cell {i}");
        }
    }
}

/// R5a done-when: water rises from a water table against gravity. A sand column over
/// free water at its base comes to the hydrostatic profile, each cell at the content
/// the retention curve gives its height above the water; and a dried top layer over a
/// wet column is fed from below.
#[test]
fn water_rises_from_a_water_table() {
    let texture = Texture::Sand;
    let r = texture.retention();
    let col = flow::Column {
        dz: 0.02,
        tables: flow::Tables::Uniform(flow::FlowTable::for_texture(texture)),
    };
    let mut theta = vec![texture.hydraulics().wilting_point; 15];
    let mut s = flow::Scratch::default();
    let mut taken = 0.0;
    for _ in 0..24 * 20 {
        let h = col.hour(
            &mut theta,
            3600.0,
            flow::Bottom::Head {
                head: 0.0,
                limit: 1.0,
            },
            &mut s,
        );
        taken -= h.bottom;
    }
    assert!(taken > 0.0);
    for (z, t) in theta.iter().enumerate() {
        // Height of the cell's center above the base.
        #[allow(clippy::cast_precision_loss)]
        let height = 0.02 * (14.5 - z as f64);
        let expect = r.theta(KPa(-height * flow::KPA_PER_M_WATER));
        assert!((t - expect).abs() < 2e-3, "cell {z}: {t} vs {expect}");
    }
    // A dried top layer over a wet closed column gets water back from below.
    let mut g = closed(Texture::Loam, 10);
    g.water[0] = g.capacity(Texture::Loam.hydraulics().wilting_point);
    let dry = g.theta(0);
    drain_hours(&mut g, 24);
    assert!(g.theta(0) > dry + 0.01, "{} vs {dry}", g.theta(0));
}

/// R5a done-when: a pot with holes keeps its bottom near saturation for at least a day
/// after a soaking (a perched water table); a pot without holes loses nothing; a pot in
/// a saucer catches what drains and feeds it back.
#[test]
fn pots_keep_a_wet_bottom() {
    let soak = SoilHour {
        water_in: Millimeters(60.0),
        ..dry_hour()
    };
    let pot = |d: Drainage| {
        let mut g = SoilGrid::new_pot(Meters(0.12), Meters(0.24), Meters(0.02), Texture::Loam, T20)
            .unwrap();
        g.set_drainage(d);
        g
    };
    let hy = Texture::Loam.hydraulics();
    let bottom = |g: &SoilGrid| {
        let (nx, ny, nz) = g.dims();
        let i = (nz - 1) * nx * ny + (ny / 2) * nx + nx / 2;
        g.theta(i)
    };
    let mut holes = pot(Drainage::Holes);
    let mut out = 0.0;
    for h in 0..72 {
        let l = holes.step_hour(&(if h < 3 { soak } else { dry_hour() }));
        assert!(l.relative_residual() < 1e-9, "{l:?}");
        out += l.drained.0;
        if h == 27 || h == 71 {
            assert!(
                bottom(&holes) > 0.98 * hy.saturation,
                "hour {h}: {}",
                bottom(&holes)
            );
        }
    }
    assert!(out > 0.0);
    let mut sealed = pot(Drainage::NoHoles);
    let before = sealed.stored_water();
    let mut applied = 0.0;
    for h in 0..24 {
        let l = sealed.step_hour(&(if h < 3 { soak } else { dry_hour() }));
        assert_eq!(l.drained, KgWater::ZERO);
        applied += l.applied.0 - l.runoff.0;
    }
    assert!(((sealed.stored_water() - before).0 - applied).abs() < 1e-12);
    let mut saucer = pot(Drainage::Saucer {
        depth: Millimeters(20.0),
    });
    for h in 0..6 {
        let l = saucer.step_hour(&(if h < 3 { soak } else { dry_hour() }));
        assert!(l.relative_residual() < 1e-9, "{l:?}");
    }
    assert!(saucer.saucer_water().0 > 0.0);
    // Dry the pot down to field capacity and fill the saucer 10 mm deep: it feeds the
    // base back.
    saucer.saucer = KgWater(10.0 * saucer.surface_area());
    let held = saucer.saucer_water();
    for i in 0..saucer.len() {
        if saucer.is_active(i) {
            saucer.water[i] = saucer.capacity(hy.field_capacity);
        }
    }
    drain_hours(&mut saucer, 24);
    assert!(saucer.saucer_water() < held);
    assert!(
        bottom(&saucer) > hy.field_capacity + 0.05,
        "{}",
        bottom(&saucer)
    );
}

/// The column solver's table follows the retention curve from air-dry to saturation:
/// the head to 1e-7, the slope and conductivity to 1e-3.
#[test]
fn the_flow_table_follows_the_curve() {
    let rel = |a: f64, b: f64| ((a - b) / b).abs();
    for texture in [Texture::Loam, Texture::Sand, Texture::Clay] {
        let hy = texture.hydraulics();
        let r = texture.retention();
        let table = flow::FlowTable::for_texture(texture);
        let mut worst = [0.0_f64; 3];
        for i in 0..1000 {
            let theta = hy.air_dry + (hy.saturation - hy.air_dry) * f64::from(i) / 1000.0;
            let (psi, slope, kr) = r.flow_terms(theta);
            let (h, c, k) = table.terms(theta);
            worst[0] = worst[0].max(rel(h, psi.0 / flow::KPA_PER_M_WATER));
            worst[1] = worst[1].max(rel(c, slope * flow::KPA_PER_M_WATER));
            worst[2] = worst[2].max(rel(k, kr * hy.saturated_conductivity));
        }
        println!("{texture:?}: head, slope, conductivity within {worst:?}");
        assert!(
            worst[0] < 1e-7 && worst[1] < 1e-3 && worst[2] < 1e-3,
            "{texture:?} {worst:?}"
        );
    }
}

/// The column solver conserves water to 1e-12 every hour, whatever it does inside.
#[test]
fn the_column_solver_conserves_water() {
    for texture in [Texture::Loam, Texture::Sand, Texture::Clay] {
        let hy = texture.hydraulics();
        let col = flow::Column {
            dz: 0.02,
            tables: flow::Tables::Uniform(flow::FlowTable::for_texture(texture)),
        };
        let mut s = flow::Scratch::default();
        for bottom in [
            flow::Bottom::Free,
            flow::Bottom::Seepage,
            flow::Bottom::Closed,
            flow::Bottom::Head {
                head: 0.01,
                limit: 0.002,
            },
        ] {
            // Saturated over dry: the hardest start.
            let mut theta = vec![hy.wilting_point; 12];
            theta[..4].fill(hy.saturation);
            for _ in 0..24 {
                let before: f64 = theta.iter().sum::<f64>() * 0.02;
                let h = col.hour(&mut theta, 3600.0, bottom, &mut s);
                let after: f64 = theta.iter().sum::<f64>() * 0.02;
                assert!(
                    (before - after - h.bottom - h.backed_up).abs() < 1e-12 * before,
                    "{texture:?} {bottom:?}"
                );
                assert!(h.substeps <= flow::MAX_SUBSTEPS);
                assert!(
                    theta.iter().all(|t| *t > 0.0),
                    "{texture:?} {bottom:?} {theta:?}"
                );
                if let flow::Bottom::Head { limit, .. } = bottom {
                    assert!(-h.bottom <= limit * (1.0 + 1e-12));
                }
            }
        }
    }
}

#[test]
fn infiltration_fills_from_the_top() {
    let mut g = column(Texture::Loam, 10);
    g.fill_to(Texture::Loam.hydraulics().wilting_point);
    // 2 mm on a 2 cm cell: 0.0008 kg, less than one cell's room to field capacity.
    let mut l = WaterLedger::default();
    g.infiltrate(Millimeters(2.0), None, &mut l);
    assert!((l.applied.0 - 0.0008).abs() < 1e-15);
    assert!(g.theta(0) > Texture::Loam.hydraulics().wilting_point);
    assert!((g.theta(1) - Texture::Loam.hydraulics().wilting_point).abs() < 1e-15);
    assert_eq!(l.runoff, KgWater::ZERO);
    assert_eq!(g.ponded_water(), KgWater::ZERO);
}

/// R5a: at most the saturated conductivity's depth soaks in an hour; the rest ponds up
/// to the surface's store and the excess runs off. Ponded water soaks in later.
#[test]
fn water_ponds_then_runs_off() {
    // Clay takes 4.8 cm a day, 2 mm an hour; a plot ponds 5 mm.
    let mut g = column(Texture::Clay, 10);
    g.fill_to(Texture::Clay.hydraulics().wilting_point);
    let mut l = WaterLedger::default();
    g.infiltrate(Millimeters(20.0), None, &mut l);
    let mm = |kg: KgWater| kg.0 / 4e-4;
    assert!((mm(g.ponded_water()) - PLOT_POND_MM).abs() < 1e-12);
    assert!(
        (mm(l.runoff) - (20.0 - 2.0 - PLOT_POND_MM)).abs() < 1e-9,
        "{l:?}"
    );
    let pond = g.ponded_water();
    let l = g.step_hour(&dry_hour());
    assert!(l.relative_residual() < 1e-9);
    assert!(g.ponded_water() < pond);
    // A saturated column takes nothing: all of it ponds or runs off.
    let mut g = column(Texture::Clay, 3);
    g.fill_to(Texture::Clay.hydraulics().saturation);
    let mut l = WaterLedger::default();
    g.infiltrate(Millimeters(12.0), None, &mut l);
    assert!((mm(l.runoff) - (12.0 - PLOT_POND_MM)).abs() < 1e-9, "{l:?}");
    // A pot holds more below its rim.
    let mut p =
        SoilGrid::new_pot(Meters(0.1), Meters(0.06), Meters(0.02), Texture::Clay, T20).unwrap();
    p.fill_to(Texture::Clay.hydraulics().saturation);
    let mut l = WaterLedger::default();
    p.infiltrate(Millimeters(20.0), None, &mut l);
    assert_eq!(l.runoff, KgWater::ZERO);
    assert_eq!(p.pond_capacity(), Millimeters(POT_POND_MM));
}

#[test]
fn lateral_redistribution_smooths_and_conserves() {
    let mut g = SoilGrid::new_box(5, 1, 1, Meters(0.02), Texture::Loam, T20).unwrap();
    g.fill_to(0.15);
    g.water[0] = g.capacity(0.25);
    let before = g.total_water();
    let spread0 = g.theta(0) - g.theta(4);
    for _ in 0..200 {
        g.redistribute_laterally();
    }
    assert!((g.total_water() - before).0.abs() < 1e-15);
    assert!(g.theta(0) - g.theta(4) < 0.2 * spread0);
}

/// A strip's water lands on its own columns only (a split pot's watered half, R2b).
#[test]
fn a_strip_waters_its_columns_only() {
    let mut g = SoilGrid::new_box(4, 1, 2, Meters(0.02), Texture::Loam, T20).unwrap();
    g.set_divider();
    g.set_drainage(Drainage::NoHoles);
    let dry = Texture::Loam.hydraulics().wilting_point;
    g.fill_to(dry);
    let l = g.step_hour(&SoilHour {
        strip: Some(Strip {
            water: Millimeters(5.0),
            x0: 0,
            x1: 2,
        }),
        ..dry_hour()
    });
    // 5 mm over two 2 cm columns is 2 × 2e-3 kg.
    assert!((l.applied.0 - 4e-3).abs() < 1e-15);
    assert!(g.theta(0) > dry + 1e-3 && g.theta(1) > dry + 1e-3);
    // The unwatered half's water only moved within its own columns.
    let east = |g: &SoilGrid| g.theta(2) + g.theta(3) + g.theta(6) + g.theta(7);
    assert!((east(&g) - 4.0 * dry).abs() < 1e-12);
    assert!(l.relative_residual() < 1e-12);
}

#[test]
fn evaporation_takes_from_top_only_and_stops_dry() {
    let mut g = closed(Texture::Sand, 4);
    g.fill_to(0.35);
    let hour = SoilHour {
        surface_radiation: WPerM2(800.0),
        ..dry_hour()
    };
    let start = g.stored_water();
    let l = g.step_hour(&hour);
    // Wet soil: 0.5 × 800 × 3600 / 2.45e6 = 0.588 kg m-2 h-1 (0.59 mm).
    assert!((l.evaporated.0 / 4e-4 - 0.5 * 800.0 * 3600.0 / 2.45e6).abs() < 1e-9);
    let mut evaporated = l.evaporated.0;
    for _ in 0..2000 {
        evaporated += g.step_hour(&hour).evaporated.0;
    }
    // The top dries out, the layers below feed it by capillary flow, and the column
    // loses exactly what evaporated.
    assert!(
        (g.theta(0) - Texture::Sand.hydraulics().air_dry).abs() < 1e-3,
        "{}",
        g.theta(0)
    );
    assert!(((start - g.stored_water()).0 - evaporated).abs() < 1e-12);
    let night = g.step_hour(&dry_hour());
    assert_eq!(night.evaporated, KgWater::ZERO);
}

#[test]
fn ftsw_and_stress() {
    let mut g = column(Texture::Loam, 2);
    let roots = RootCells::from_pairs(vec![(1, Meters(2.0)), (0, Meters(1.0)), (1, Meters(1.0))]);
    assert_eq!(roots.cells(), &[(0, Meters(1.0)), (1, Meters(3.0))]);
    assert!((g.ftsw(&roots).unwrap() - 1.0).abs() < 1e-12);
    g.fill_to(Texture::Loam.hydraulics().wilting_point);
    assert!(g.ftsw(&roots).unwrap().abs() < 1e-12);
    // Top cell at FC, bottom at WP: weighted 1 / (1 + 3).
    g.water[0] = g.capacity(Texture::Loam.hydraulics().field_capacity);
    assert!((g.ftsw(&roots).unwrap() - 0.25).abs() < 1e-12);
    assert!((stress_factor(0.2, 0.4) - 0.5).abs() < 1e-15);
    assert!((stress_factor(0.6, 0.4) - 1.0).abs() < 1e-15);
    assert!(stress_factor(0.0, 0.4).abs() < 1e-15);
    assert!(matches!(
        g.ftsw(&RootCells::from_pairs(vec![(2, Meters(1.0))])),
        Err(SoilError::RootCell { cell: 2, cells: 2 })
    ));
    let none = RootCells::from_pairs(vec![(0, Meters(0.0))]);
    assert!(none.cells().is_empty());
    assert!((g.ftsw(&none).unwrap() - 1.0).abs() < 1e-15);
}

fn plant(demand: f64, flows: &[(usize, KgWater)]) -> PlantFlows<'_> {
    PlantFlows {
        demand: KgWater(demand),
        flows,
    }
}

/// R3b: a plant's network flows are granted as asked while the cells hold the water,
/// a withdrawal stops at air-dry, and water given back to a dry cell lands there.
#[test]
fn flows_are_granted_down_to_air_dry() {
    let hy = Texture::Loam.hydraulics();
    let mut g = column(Texture::Loam, 2);
    g.fill_to(hy.field_capacity);
    let w1 = g.water(1);
    let flows = [(0, KgWater(2e-5)), (1, KgWater(-1e-5))];
    let j = g.take_flows(&[plant(1e-5, &flows)]).unwrap();
    let u = &j.plants[0];
    assert!((u.actual.0 - 1e-5).abs() < 1e-18);
    assert!(u.shortfall.0.abs() < 1e-18);
    assert_eq!(u.per_cell, flows.to_vec());
    assert!(((g.water(1) - w1).0 - 1e-5).abs() < 1e-18);
    assert!(j.ledger.relative_residual() < 1e-9);
    // Asking for more than the cell holds: the cell goes to air-dry, not below, and the
    // plant's deposit is cut by the same share, so its net uptake is that share of its
    // demand.
    g.fill_to(hy.field_capacity);
    let avail = (g.water(0) - g.capacity(hy.air_dry)).0;
    let big = [(0, KgWater(4.0 * avail)), (1, KgWater(-2.0 * avail))];
    let j = g.take_flows(&[plant(2.0 * avail, &big)]).unwrap();
    let u = &j.plants[0];
    assert!((g.theta(0) - hy.air_dry).abs() < 1e-12);
    assert!((u.actual.0 - 0.5 * avail).abs() < 1e-12 * avail);
    assert!((u.shortfall.0 - 1.5 * avail).abs() < 1e-12 * avail);
    assert!(j.ledger.relative_residual() < 1e-9);
    assert!(matches!(
        g.take_flows(&[plant(0.0, &[(2, KgWater(1e-6))])]),
        Err(SoilError::RootCell { cell: 2, cells: 2 })
    ));
}

/// Decision 0012 with R3b's flows: plants share a cell from the same state, so their
/// order does not matter, and together they never take more than it holds.
#[test]
fn joint_flows_share_cells_fairly() {
    let hy = Texture::Loam.hydraulics();
    let mut a = column(Texture::Loam, 2);
    a.fill_to(hy.field_capacity);
    let mut b = a.clone();
    let avail = (a.water(0) - a.capacity(hy.air_dry)).0;
    let p = [(0, KgWater(avail)), (1, KgWater(1e-6))];
    let q = [(0, KgWater(avail))];
    let ja = a.take_flows(&[plant(avail, &p), plant(avail, &q)]).unwrap();
    let jb = b.take_flows(&[plant(avail, &q), plant(avail, &p)]).unwrap();
    assert_eq!(ja.plants[0], jb.plants[1]);
    assert_eq!(ja.plants[1], jb.plants[0]);
    assert_eq!(a.water(0), b.water(0));
    // Both asked for all of cell 0; each gets half of it.
    assert!((ja.plants[1].actual.0 - 0.5 * avail).abs() < 1e-12 * avail);
    assert!((a.theta(0) - hy.air_dry).abs() < 1e-12);
    assert!(ja.ledger.relative_residual() < 1e-9);
}

#[test]
fn deeper_soil_is_damped_and_lagged() {
    let mut g = column(Texture::Loam, 10);
    let mut top = Vec::new();
    let mut deep = Vec::new();
    for day in 0..3 {
        for h in 0..24 {
            let t =
                20.0 + 8.0 * libm::sin(2.0 * core::f64::consts::PI * (f64::from(h) - 9.0) / 24.0);
            g.step_hour(&SoilHour {
                t_air: Celsius(t),
                ..dry_hour()
            });
            if day == 2 {
                top.push(g.layer_temperature(0).0);
                deep.push(g.layer_temperature(9).0);
            }
        }
    }
    let range = |v: &[f64]| {
        v.iter().copied().fold(f64::MIN, f64::max) - v.iter().copied().fold(f64::MAX, f64::min)
    };
    assert!(
        range(&deep) < 0.3 * range(&top),
        "{} vs {}",
        range(&deep),
        range(&top)
    );
    let argmax = |v: &[f64]| {
        v.iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0
    };
    assert_ne!(argmax(&deep), argmax(&top));
}

#[test]
fn pot_mask_area_is_close_to_circle() {
    let g =
        SoilGrid::new_pot(Meters(0.20), Meters(0.20), Meters(0.02), Texture::Loam, T20).unwrap();
    assert_eq!(g.dims(), (10, 10, 10));
    let circle = core::f64::consts::PI * 0.01;
    assert!(
        (g.surface_area() / circle - 1.0).abs() < 0.1,
        "{}",
        g.surface_area()
    );
    assert!(
        SoilGrid::new_pot(Meters(0.01), Meters(0.2), Meters(0.02), Texture::Loam, T20).is_err()
    );
}

/// R3a: a pot at field capacity reads −33 kPa in every active cell and layer, nothing
/// outside the pot, and a pot dried from the top reads drier the higher the layer.
#[test]
fn pot_water_potential_follows_the_water() {
    let mut g =
        SoilGrid::new_pot(Meters(0.20), Meters(0.20), Meters(0.02), Texture::Loam, T20).unwrap();
    let outside = (0..g.len()).find(|&i| !g.is_active(i)).unwrap();
    assert_eq!(g.water_potential(outside), None);
    for i in (0..g.len()).filter(|&i| g.is_active(i)) {
        let psi = g.water_potential(i).unwrap();
        assert!((psi.0 + 33.0).abs() < 1e-9, "cell {i}: {psi:?}");
    }
    let layers = g.layer_water_potentials();
    assert_eq!(layers.len(), 10);
    assert!(
        layers.iter().all(|p| (p.0 + 33.0).abs() < 1e-9),
        "{layers:?}"
    );
    // Evaporation dries the top layer only.
    for _ in 0..48 {
        g.step_hour(&SoilHour {
            surface_radiation: WPerM2(600.0),
            ..dry_hour()
        });
    }
    let layers = g.layer_water_potentials();
    assert!(layers[0].0 < -100.0, "{layers:?}");
    // The pot dries from the top: capillary flow feeds the surface from below, so every
    // layer dries, the top most.
    assert!(layers.windows(2).all(|w| w[0].0 < w[1].0), "{layers:?}");
    assert!(layers[9].0 < -33.0 && layers[9].0 > -60.0, "{layers:?}");
}

proptest! {
    /// Random weather in a pot: every hour closes to 1e-9, water stays within [0,
    /// saturation], give or take the little a saturated cell holds under pressure.
    #[test]
    fn random_hours_conserve(
        hours in proptest::collection::vec((0.0f64..30.0, 0.0f64..900.0, -5.0f64..40.0, 0.0f64..0.002), 1..80),
        texture in prop_oneof![Just(Texture::Loam), Just(Texture::Sand), Just(Texture::Clay)],
    ) {
        let mut g = SoilGrid::new_pot(Meters(0.12), Meters(0.10), Meters(0.02), texture, T20).unwrap();
        let cells: Vec<usize> = (0..g.len()).filter(|&i| i % 3 == 0 && g.is_active(i)).collect();
        let sat = texture.hydraulics().saturation;
        for (mm, rad, t, demand) in hours {
            let l = g.step_hour(&SoilHour { water_in: Millimeters(mm), surface_radiation: WPerM2(rad), t_air: Celsius(t), strip: None });
            prop_assert!(l.relative_residual() < 1e-9, "{l:?}");
            // Two plants drawing their demand evenly from every third cell.
            #[allow(clippy::cast_precision_loss)]
            let each = KgWater(demand / cells.len() as f64);
            let flows: Vec<(usize, KgWater)> = cells.iter().map(|&i| (i, each)).collect();
            let u = g.take_flows(&[plant(demand, &flows), plant(demand, &flows)]).unwrap();
            prop_assert!(u.ledger.relative_residual() < 1e-9);
            for i in 0..g.len() {
                let th = g.theta(i);
                // A saturated cell holds a little more under pressure (specific storage).
                prop_assert!(th >= -1e-15 && th <= sat + 0.002, "cell {i} theta {th}");
            }
        }
    }
}

#[test]
fn lateral_fraction_scales_as_a_diffusivity() {
    let at = |cell: f64| {
        SoilGrid::new_box(2, 2, 1, Meters(cell), Texture::Loam, T20)
            .unwrap()
            .lateral_fraction()
    };
    // Exactly the §3.6 value at 2 cm, so pots are unchanged.
    assert_eq!(at(0.02).to_bits(), LATERAL_FRACTION.to_bits());
    // 4e-5 m² h⁻¹ over (5 cm)² = 0.016 in a plot.
    assert!((at(0.05) - 0.016).abs() < 1e-15);
    // Capped for stability in fine grids.
    assert!((at(0.005) - 0.25).abs() < 1e-15);
}

/// R5b (decision 0048): a cell's oxygen deficit rises linearly from 0 at 10% air to 1
/// at 2%; its hypoxic hours sum the deficit and it is hypoxic once they reach 6, then
/// as hypoxic as it is short of air. A cell with air enough starts again from 0.
#[test]
fn a_saturated_cell_turns_hypoxic_after_hours_and_recovers_when_it_drains() {
    let sat = Texture::Loam.hydraulics().saturation;
    let mut g = closed(Texture::Loam, 3);
    g.fill_to(sat - 0.06);
    assert!((g.oxygen_deficit(0) - 0.5).abs() < 1e-12);
    g.fill_to(sat - AERATION_CRITICAL);
    assert!(g.oxygen_deficit(0).abs() < 1e-12);
    g.fill_to(sat);
    for h in 1..=8 {
        let l = g.step_hour(&dry_hour());
        assert!(l.relative_residual() < 1e-9);
        let full = f64::from(h);
        assert!(
            (g.hypoxic_hours(2) - full).abs() < 1e-9,
            "{}",
            g.hypoxic_hours(2)
        );
        let expected = if full >= HYPOXIA_ONSET_HOURS {
            1.0
        } else {
            0.0
        };
        assert!((g.hypoxia(2) - expected).abs() < 1e-12, "hour {h}");
    }
    g.fill_to(Texture::Loam.hydraulics().field_capacity);
    g.step_hour(&dry_hour());
    assert!(g.hypoxic_hours(2).abs() < 1e-12);
    assert!(g.hypoxia(2).abs() < 1e-12);
}

/// R5b: water standing on the soil evaporates first, at the full rate; the soil under
/// it loses nothing while it lasts, and every hour closes.
#[test]
fn standing_water_evaporates_before_the_soil() {
    let sat = Texture::Clay.hydraulics().saturation;
    let mut g = closed(Texture::Clay, 3);
    g.fill_to(sat);
    let mut l = WaterLedger::default();
    g.infiltrate(Millimeters(2.0), None, &mut l);
    let soil = g.total_water();
    let hour = SoilHour {
        surface_radiation: WPerM2(800.0),
        ..dry_hour()
    };
    let l = g.step_hour(&hour);
    assert!(l.relative_residual() < 1e-9);
    // 0.588 mm of the 2 mm pond goes, the soil keeps its water.
    let mm = |kg: KgWater| kg.0 / 4e-4;
    assert!((mm(l.evaporated) - 0.5 * 800.0 * 3600.0 / 2.45e6).abs() < 1e-9);
    assert!((mm(g.ponded_water()) - (2.0 - mm(l.evaporated))).abs() < 1e-9);
    assert!((g.total_water() - soil).0.abs() < 1e-12);
}

/// Cells `z0 ≤ z < z1` of every column.
fn layers(g: &SoilGrid, z0: usize, z1: usize) -> Vec<usize> {
    let (nx, ny, _) = g.dims();
    (z0 * nx * ny..z1 * nx * ny).collect()
}

fn wet_hour(mm: f64) -> SoilHour {
    SoilHour {
        water_in: Millimeters(mm),
        ..dry_hour()
    }
}

/// R5c: the texture's own density is the texture's class exactly, so a grid whose cells
/// were all set to it runs bit for bit as one without structure.
#[test]
fn a_structured_grid_at_the_textures_density_runs_as_before() {
    for texture in [Texture::Loam, Texture::Sand, Texture::Clay] {
        let c = structure::soil_class(texture, 0);
        assert_eq!(c.hydraulics, texture.hydraulics());
        assert_eq!(c.retention, texture.retention());
        let mut plain =
            SoilGrid::new_pot(Meters(0.12), Meters(0.2), Meters(0.02), texture, T20).unwrap();
        let mut set = plain.clone();
        set.set_density(&layers(&set, 0, 10), 1.0);
        assert!(set.has_structure() && !plain.has_structure());
        for h in 0..96 {
            let hour = if h % 24 == 6 {
                wet_hour(30.0)
            } else {
                dry_hour()
            };
            let (a, b) = (plain.step_hour(&hour), set.step_hour(&hour));
            assert_eq!(a, b, "{texture:?} hour {h}");
        }
        for i in 0..plain.len() {
            assert_eq!(
                plain.water(i).to_bits(),
                set.water(i).to_bits(),
                "{texture:?} {i}"
            );
            assert_eq!(
                plain.hypoxic_hours(i).to_bits(),
                set.hypoxic_hours(i).to_bits()
            );
        }
        let middle = set.index(Cell { x: 3, y: 3, z: 2 });
        let s = set.strength(middle).unwrap();
        assert_eq!(s.resistance, s.reference);
    }
}

/// R5c: denser soil holds less air and drains slower, and keeps its field capacity and
/// wilting point; looser soil the reverse.
#[test]
fn density_moves_saturation_and_conductivity_and_keeps_the_anchors() {
    let base = Texture::Loam.hydraulics();
    let mut last_ks = f64::INFINITY;
    for level in [-33, -15, 0, 15, 20, 30] {
        let c = structure::soil_class(Texture::Loam, level);
        let h = c.hydraulics;
        let d = base.bulk_density * (1.0 + 0.01 * f64::from(level));
        assert!((h.bulk_density - d).abs() < 1e-9, "{level}");
        let sat = (base.saturation - (d - base.bulk_density) / 2650.0)
            .max(base.field_capacity + structure::MIN_STRUCTURAL_AIR);
        assert!(
            (h.saturation - sat).abs() < 1e-12,
            "{level}: {}",
            h.saturation
        );
        assert!(h.saturated_conductivity < last_ks, "{level}");
        last_ks = h.saturated_conductivity;
        let r = c.retention;
        assert!(
            (r.psi(base.field_capacity).0 + 33.0).abs() < 1e-6,
            "{level}"
        );
        assert!(
            (r.psi(base.wilting_point).0 + 1500.0).abs() < 1e-4,
            "{level}"
        );
    }
    // A pan at 1.2 times a loam's density keeps 8% air at field capacity, short of the
    // 10% roots need (R5b), and conducts about 30 times less.
    let pan = structure::soil_class(Texture::Loam, 20).hydraulics;
    let air = pan.saturation - pan.field_capacity;
    assert!((0.075..0.09).contains(&air), "{air}");
    let ratio = base.saturated_conductivity / pan.saturated_conductivity;
    assert!((20.0..40.0).contains(&ratio), "{ratio}");
}

/// R5c spec §8: strength rises with density at the same water, and with dryness at the
/// same density; the reference is the same water at the texture's density.
#[test]
fn strength_rises_with_density_and_dryness() {
    let mut g = SoilGrid::new_box(1, 1, 6, Meters(0.05), Texture::Loam, T20).unwrap();
    for (z, f) in [0.7, 0.85, 1.0, 1.1, 1.2, 1.3].into_iter().enumerate() {
        g.set_density(&[z], f);
    }
    let hy = Texture::Loam.hydraulics();
    let mut last_dry = [0.0; 6];
    // At field capacity and drier: every class is short of saturation there (wetter, a
    // dense cell can be saturated and soft where a loose one is not).
    for theta in [hy.field_capacity, 0.2, 0.15, hy.wilting_point] {
        g.fill_to(theta);
        let q: Vec<Strength> = (0..6).map(|i| g.strength(i).unwrap()).collect();
        for z in 1..6 {
            assert!(q[z].resistance.0 > q[z - 1].resistance.0, "{theta} {z}");
            assert!(q[z].resistance.0 > last_dry[z], "{theta} {z}");
            last_dry[z] = q[z].resistance.0;
        }
        if (theta - hy.field_capacity).abs() < 1e-12 {
            // The texture's own soil at field capacity reads the anchor; a 1.2 pan
            // (1.56 Mg m⁻³) about 1.8 MPa.
            assert!((q[2].resistance.0 - 0.5).abs() < 1e-9, "{:?}", q[2]);
            assert!((1.6..2.0).contains(&q[4].resistance.0), "{:?}", q[4]);
        }
    }
    assert!(
        SoilGrid::new_box(1, 1, 1, Meters(0.05), Texture::Loam, T20)
            .unwrap()
            .strength(0)
            .is_none()
    );
}

/// R5c: dug soil settles back with the water that reaches it, half the gap in the first
/// 10 mm (WEPP); a trampled path stays; digging through a pan breaks it up.
#[test]
fn dug_soil_settles_and_trampled_soil_stays() {
    let mut g = SoilGrid::new_box(2, 1, 4, Meters(0.05), Texture::Loam, T20).unwrap();
    let base = Texture::Loam.hydraulics().bulk_density;
    g.set_density(&[4, 5], 1.25); // a pan in layer 2
    g.dig(&[0, 2, 4]); // column 0, through the pan
    g.compact(&[1, 3]); // column 1's top two layers
    let dug = base * structure::DUG_FACTOR;
    assert!((g.bulk_density(0).0 - dug).abs() < 1e-9);
    assert!((g.bulk_density(4).0 - dug).abs() < 1e-9);
    assert!((g.settled_density(4).0 - base).abs() < 1e-9, "pan broken");
    assert!(
        (g.settled_density(5).0 - 1.25 * base).abs() < 1e-9,
        "pan left"
    );
    assert!((g.bulk_density(1).0 - 1.15 * base).abs() < 1e-9);
    g.step_hour(&wet_hour(10.0));
    let half = dug + 0.5 * (base - dug);
    assert!(
        (g.bulk_density(0).0 - half).abs() < 1e-9,
        "{:?}",
        g.bulk_density(0)
    );
    for _ in 0..9 {
        g.step_hour(&wet_hour(10.0));
    }
    let r = 100.0 / 110.0;
    assert!((g.bulk_density(2).0 - (dug + r * (base - dug))).abs() < 1e-9);
    assert!(
        (g.bulk_density(1).0 - 1.15 * base).abs() < 1e-9,
        "trampled stays"
    );
    assert!((g.bulk_density(6).0 - base).abs() < 1e-12, "untouched");
    // Saved and loaded, the classes come back from the densities.
    let saved = ron::to_string(&g).unwrap();
    let back: SoilGrid = ron::from_str(&saved).unwrap();
    assert_eq!(back, g);
    for i in 0..g.len() {
        assert_eq!(back.hydraulics_at(i), g.hydraulics_at(i), "{i}");
    }
}

/// R5c: water perches on a pan after rain and the pan stays short of air, while every
/// hour's water closes.
#[test]
fn water_perches_on_a_pan() {
    let mut pan = SoilGrid::new_box(1, 1, 10, Meters(0.05), Texture::Loam, T20).unwrap();
    let mut flat = pan.clone();
    pan.set_density(&[3], 1.25);
    for g in [&mut pan, &mut flat] {
        for h in 0..72 {
            let hour = if h < 4 { wet_hour(15.0) } else { dry_hour() };
            let l = g.step_hour(&hour);
            assert!(l.relative_residual() < 1e-9, "{l:?}");
        }
    }
    // Three days on, the layer above the pan is still 2% wetter, the pan short of air,
    // and the soil below drier.
    assert!(
        pan.theta(2) > flat.theta(2) + 0.015,
        "{} {}",
        pan.theta(2),
        flat.theta(2)
    );
    assert!(pan.air_porosity(3) < 0.02, "{}", pan.air_porosity(3));
    assert!(pan.hypoxic_hours(3) > 24.0, "{}", pan.hypoxic_hours(3));
    assert!(
        pan.theta(5) < flat.theta(5) - 0.01,
        "{} {}",
        pan.theta(5),
        flat.theta(5)
    );
}
