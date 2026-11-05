use super::*;
use proptest::prelude::*;
use ps_units::{KgPerMH, M2PerHour};

const T20: Celsius = Celsius(20.0);

fn dry_hour() -> SoilHour {
    SoilHour {
        water_in: Millimeters(0.0),
        surface_radiation: WPerM2(0.0),
        t_air: T20,
    }
}

fn column(texture: Texture, nz: usize) -> SoilGrid {
    SoilGrid::new_box(1, 1, nz, Meters(0.02), texture, T20).unwrap()
}

/// WP5 done-when: a wet column drains to field capacity with closure under 1e-9, and
/// matches the analytic case of §8.1. With all excess moving one cell per hour, a
/// saturated column of n cells loses (θs − θfc)·V each hour for exactly n hours.
#[test]
fn wet_column_drains_to_field_capacity() {
    for texture in [Texture::Loam, Texture::Sand, Texture::Clay] {
        let n = 10;
        let mut g = column(texture, n);
        let hy = texture.hydraulics();
        g.fill_to(hy.saturation);
        let start = g.total_water();
        let excess_per_cell = (hy.saturation - hy.field_capacity) * g.cell_volume() * WATER_DENSITY;
        let mut total = WaterLedger {
            storage_before: start,
            ..WaterLedger::default()
        };
        let mut drained_by_hour = Vec::new();
        for _ in 0..2 * n {
            let l = g.step_hour(&dry_hour());
            assert!(l.relative_residual() < 1e-9, "{texture:?}: {l:?}");
            total.accumulate(&l);
            drained_by_hour.push(l.drained.0);
        }
        assert!(total.relative_residual() < 1e-9);
        // Every cell at field capacity.
        for i in 0..g.len() {
            assert!(
                (g.theta(i) - hy.field_capacity).abs() < 1e-12,
                "{texture:?} cell {i}"
            );
        }
        // Analytic outflow: excess per cell, each hour, for n hours, then nothing.
        for (h, d) in drained_by_hour.iter().enumerate() {
            let expect = if h < n { excess_per_cell } else { 0.0 };
            assert!(
                (d - expect).abs() < 1e-12,
                "{texture:?} hour {h}: {d} vs {expect}"
            );
        }
        // Drainage time (99.9% of excess out) within 5% of n hours.
        let target = 0.999 * excess_per_cell * f64::from(u32::try_from(n).unwrap());
        let mut cum = 0.0;
        let hours = drained_by_hour
            .iter()
            .position(|d| {
                cum += d;
                cum >= target
            })
            .unwrap()
            + 1;
        assert!((f64::from(u32::try_from(hours).unwrap()) / 10.0 - 1.0).abs() <= 0.05);
        assert!(((start - g.total_water()).0 - total.drained.0).abs() < 1e-12);
    }
}

#[test]
fn infiltration_fills_from_the_top() {
    let mut g = column(Texture::Loam, 10);
    g.fill_to(Texture::Loam.hydraulics().wilting_point);
    // 2 mm on a 2 cm cell: 0.0008 kg, less than one cell's room to field capacity.
    let l = g.step_hour(&SoilHour {
        water_in: Millimeters(2.0),
        ..dry_hour()
    });
    assert!((l.applied.0 - 0.0008).abs() < 1e-15);
    assert!(g.theta(0) > Texture::Loam.hydraulics().wilting_point);
    assert!((g.theta(1) - Texture::Loam.hydraulics().wilting_point).abs() < 1e-15);
    assert_eq!(l.runoff, KgWater::ZERO);
}

#[test]
fn excess_runs_off_when_saturated() {
    let mut g = column(Texture::Clay, 3);
    g.fill_to(Texture::Clay.hydraulics().saturation);
    let l = g.step_hour(&SoilHour {
        water_in: Millimeters(5.0),
        ..dry_hour()
    });
    assert!((l.runoff.0 - l.applied.0).abs() < 1e-15);
    assert!(l.relative_residual() < 1e-9);
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

#[test]
fn evaporation_takes_from_top_only_and_stops_dry() {
    let mut g = column(Texture::Sand, 4);
    let hour = SoilHour {
        surface_radiation: WPerM2(800.0),
        ..dry_hour()
    };
    let l = g.step_hour(&hour);
    // Wet soil: 0.5 × 800 × 3600 / 2.45e6 = 0.588 kg m-2 h-1 (0.59 mm).
    assert!((l.evaporated.0 / 4e-4 - 0.5 * 800.0 * 3600.0 / 2.45e6).abs() < 1e-9);
    let below = g.theta(1);
    for _ in 0..500 {
        g.step_hour(&hour);
    }
    assert!((g.theta(0) - Texture::Sand.hydraulics().air_dry).abs() < 1e-3);
    assert!(
        (g.theta(1) - below).abs() < 1e-15,
        "lower layer must not evaporate"
    );
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

fn one(demand: f64, roots: &RootCells, max: f64) -> PlantUptake<'_> {
    PlantUptake {
        demand: KgWater(demand),
        roots,
        rate: M2PerHour(1e-5),
        max_per_length: KgPerMH(max),
    }
}

#[test]
fn uptake_is_proportional_and_limited() {
    let mut g = column(Texture::Loam, 2);
    let roots = RootCells::from_pairs(vec![(0, Meters(1.0))]);
    let j = g.take_up(&[one(1e-5, &roots, 1.0)]).unwrap();
    let u = &j.plants[0];
    assert!((u.actual.0 - 1e-5).abs() < 1e-18);
    assert_eq!(u.per_cell.len(), 1);
    assert!(j.ledger.relative_residual() < 1e-9);
    // Huge demand: limited to the potential, never below wilting point.
    for _ in 0..10_000 {
        let j = g.take_up(&[one(1.0, &roots, 1.0)]).unwrap();
        assert!(j.plants[0].shortfall.0 > 0.0);
    }
    assert!(g.theta(0) >= Texture::Loam.hydraulics().wilting_point - 1e-12);
    // The per-length ceiling binds in wet soil.
    g.fill_to(Texture::Loam.hydraulics().field_capacity);
    let j = g.take_up(&[one(1.0, &roots, 1e-6)]).unwrap();
    assert!((j.plants[0].actual.0 - 1e-6).abs() < 1e-18);
}

/// Decision 0012: plants share a cell from the same state, so their order does not
/// matter, and together they never take more than its available water.
#[test]
fn joint_uptake_shares_cells_fairly() {
    let mut a = column(Texture::Loam, 2);
    a.fill_to(Texture::Loam.hydraulics().field_capacity);
    let mut b = a.clone();
    let wp = a.capacity(Texture::Loam.hydraulics().wilting_point);
    let avail = (a.water(0) - wp).0;
    let p = RootCells::from_pairs(vec![(0, Meters(1e4)), (1, Meters(1.0))]);
    let q = RootCells::from_pairs(vec![(0, Meters(1e4))]);
    let ja = a.take_up(&[one(1.0, &p, 1.0), one(1.0, &q, 1.0)]).unwrap();
    let jb = b.take_up(&[one(1.0, &q, 1.0), one(1.0, &p, 1.0)]).unwrap();
    assert_eq!(ja.plants[0], jb.plants[1]);
    assert_eq!(ja.plants[1], jb.plants[0]);
    assert_eq!(a.water(0), b.water(0));
    // Both asked for all of cell 0; each gets half of it.
    assert!((ja.plants[1].actual.0 - 0.5 * avail).abs() < 1e-12 * avail);
    assert!((a.water(0) - wp).0.abs() < 1e-12 * avail);
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

proptest! {
    /// Random weather in a pot: every hour closes to 1e-9, water stays within [0, saturation].
    #[test]
    fn random_hours_conserve(
        hours in proptest::collection::vec((0.0f64..30.0, 0.0f64..900.0, -5.0f64..40.0, 0.0f64..0.002), 1..80),
        texture in prop_oneof![Just(Texture::Loam), Just(Texture::Sand), Just(Texture::Clay)],
    ) {
        let mut g = SoilGrid::new_pot(Meters(0.12), Meters(0.10), Meters(0.02), texture, T20).unwrap();
        let roots = RootCells::from_pairs((0..g.len()).filter(|i| i % 3 == 0).map(|i| (i, Meters(0.5))).collect());
        let sat = texture.hydraulics().saturation;
        for (mm, rad, t, demand) in hours {
            let l = g.step_hour(&SoilHour { water_in: Millimeters(mm), surface_radiation: WPerM2(rad), t_air: Celsius(t) });
            prop_assert!(l.relative_residual() < 1e-9, "{l:?}");
            let u = g.take_up(&[one(demand, &roots, 1e-3), one(demand, &roots, 1e-4)]).unwrap();
            prop_assert!(u.ledger.relative_residual() < 1e-9);
            for i in 0..g.len() {
                let th = g.theta(i);
                prop_assert!(th >= -1e-15 && th <= sat + 1e-12, "cell {i} theta {th}");
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
