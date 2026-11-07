use super::*;
use crate::{SoilHour, Texture, WaterLedger};
use ps_units::{Celsius, Meters, Millimeters, WPerM2};

fn hour(rain_mm: f64, t: f64) -> SoilHour {
    SoilHour {
        water_in: Millimeters(rain_mm),
        surface_radiation: WPerM2(0.0),
        t_air: Celsius(t),
        strip: None,
    }
}

fn column(nz: usize, t: f64) -> SoilGrid {
    let mut g = SoilGrid::new_box(1, 1, nz, Meters(0.05), Texture::Loam, Celsius(t)).unwrap();
    g.set_nitrogen(&SoilNitrogen::GARDEN);
    g
}

#[test]
fn garden_soil_reads_back_its_concentrations() {
    let g = column(4, 20.0);
    for i in 0..4 {
        assert!((g.no3_ppm(i) - 20.0).abs() < 1e-12);
    }
    // 5 cm loam cell: 125 cm³ × 1.3 g/cm³ = 0.1625 kg, so 3.25 mg nitrate N.
    assert!((g.no3(0).0 - 3.25e-3).abs() < 1e-15);
    assert!((g.organic_n(0).0 - 0.1625).abs() < 1e-15);
    assert!(g.has_rhizobia());
}

#[test]
fn humus_mineralizes_and_ammonium_nitrifies_at_the_stated_rates() {
    // 30 °C at field capacity: both factors are 1.
    let mut g = column(1, 30.0);
    let (org, nh4) = (g.organic_n(0).0, g.nh4(0).0);
    g.step_hour(&hour(0.0, 30.0));
    let m = org * MINERALIZATION_RATE / 24.0;
    assert!(((org - g.organic_n(0).0) / m - 1.0).abs() < 1e-9);
    let n = (nh4 + m) * NITRIFICATION_RATE / 24.0;
    assert!((g.nh4(0).0 / (nh4 + m - n) - 1.0).abs() < 1e-9);
    // Cold soil does nothing.
    let mut cold = column(1, -2.0);
    let before = cold.clone();
    cold.step_hour(&hour(0.0, -2.0));
    assert_eq!(cold.organic_n(0), before.organic_n(0));
}

#[test]
fn factors_have_their_shapes() {
    assert!((temperature_factor(30.0) - 1.0).abs() < 1e-15);
    assert!((temperature_factor(15.0) - 0.25).abs() < 1e-15);
    assert!(temperature_factor(-5.0).abs() < f64::MIN_POSITIVE);
    assert!((temperature_factor(40.0) - 1.2).abs() < 1e-15);
    let (wp, fc, sat) = (0.12, 0.27, 0.45);
    assert!(water_factor(wp, wp, fc, sat).abs() < f64::MIN_POSITIVE);
    assert!((water_factor(0.5 * (wp + fc), wp, fc, sat) - 1.0).abs() < 1e-12);
    assert!((water_factor(fc, wp, fc, sat) - 1.0).abs() < 1e-12);
    assert!((water_factor(sat, wp, fc, sat) - 0.5).abs() < 1e-12);
}

/// Heavy rain on a column at field capacity drains through it: nitrate moves down and
/// out of the bottom, ammonium and humus stay, and the ledger closes every hour.
#[test]
fn rain_leaches_nitrate_and_the_ledger_closes() {
    let mut g = column(4, 20.0);
    let nh4_top = g.nh4(0);
    let mut water = WaterLedger::default();
    for h in 0..48 {
        let rain = if h < 6 { 10.0 } else { 0.0 };
        water.accumulate(&g.step_hour(&hour(rain, 20.0)));
        let l = g.nitrogen_ledger();
        assert!(l.relative_residual(g.total_nitrogen()) < 1e-12, "{l:?}");
    }
    assert!(water.drained.0 > 0.0);
    let l = *g.nitrogen_ledger();
    assert!(l.leached.0 > 0.0);
    assert!(
        g.no3_ppm(0) < g.no3_ppm(3),
        "top {} bottom {}",
        g.no3_ppm(0),
        g.no3_ppm(3)
    );
    // Ammonium only changes by mineralization and nitrification, not by water.
    assert!(g.nh4(0).0 < nh4_top.0 + 1e-6);
}

/// Nitrate moves sideways with the water that moves, at the concentration of the cell
/// the water leaves, and otherwise only diffuses slowly (R2b, decision 0043).
#[test]
fn nitrate_moves_with_the_water_and_diffuses_slowly() {
    // Equal water: diffusion alone, 5.1e-3 of the difference an hour between 2 cm cells.
    let mut g = SoilGrid::new_box(3, 1, 1, Meters(0.02), Texture::Loam, Celsius(0.0)).unwrap();
    g.set_drainage(crate::Drainage::NoHoles);
    g.set_nitrogen(&SoilNitrogen::NONE);
    g.no3[0] = GramsN(1e-3);
    g.n_ledger.start = g.total_nitrogen();
    for _ in 0..20 {
        g.step_hour(&hour(0.0, 0.0));
    }
    let moved = 1.0 - g.no3(0).0 / 1e-3;
    assert!(moved > 0.05 && moved < 0.12, "{moved}");
    assert!(g.no3(2).0 < 0.01e-3, "{:?}", g.no3);
    assert!((g.total_nitrogen().0 - 1e-3).abs() < 1e-15);
    // A wet cell with nitrate next to a drier one: in the first hour the nitrate moves
    // in the same share as the water (plus diffusion).
    let mut g = SoilGrid::new_box(2, 1, 1, Meters(0.02), Texture::Loam, Celsius(0.0)).unwrap();
    g.set_nitrogen(&SoilNitrogen::NONE);
    g.fill_to(0.15);
    g.water[0] = g.capacity(0.25);
    g.no3[0] = GramsN(1e-3);
    g.n_ledger.start = g.total_nitrogen();
    let w0 = g.water(0).0;
    g.redistribute_laterally();
    let water_share = 1.0 - g.water(0).0 / w0;
    let n_share = 1.0 - g.no3(0).0 / 1e-3;
    let diffusion = g.nitrate_diffusion_fraction();
    assert!(
        (n_share - water_share - diffusion).abs() < 1e-12,
        "{n_share} {water_share}"
    );
    assert!((g.total_nitrogen().0 - 1e-3).abs() < 1e-15);
}

/// A divider stops water and nitrate between the halves (a split pot, R2b).
#[test]
fn a_divider_stops_water_and_nitrate() {
    let mut g = SoilGrid::new_box(4, 1, 1, Meters(0.02), Texture::Loam, Celsius(0.0)).unwrap();
    g.set_nitrogen(&SoilNitrogen::NONE);
    g.set_divider();
    assert_eq!(g.divider(), Some(2));
    g.fill_to(0.15);
    g.water[0] = g.capacity(0.25);
    g.no3[0] = GramsN(1e-3);
    let east = (g.water(2), g.water(3));
    for _ in 0..300 {
        g.redistribute_laterally();
    }
    assert_eq!((g.water(2), g.water(3)), east);
    assert_eq!((g.no3(2), g.no3(3)), (GramsN(0.0), GramsN(0.0)));
    assert!(
        (g.theta(0) - g.theta(1)).abs() < 1e-6,
        "the west half evens out"
    );
}

#[test]
fn fertilizer_goes_to_the_top_layer() {
    let mut g = SoilGrid::new_box(2, 2, 3, Meters(0.05), Texture::Loam, Celsius(20.0)).unwrap();
    g.set_nitrogen(&SoilNitrogen::NONE);
    // 10 g N per m² over 0.01 m² of surface is 0.1 g.
    g.fertilize(NitrogenForm::Nitrate, 10.0);
    g.fertilize(NitrogenForm::Ammonium, 5.0);
    assert!((g.no3(0).0 - 0.025).abs() < 1e-15);
    assert!((g.nh4(3).0 - 0.0125).abs() < 1e-15);
    assert!(g.no3(4).0.abs() < f64::MIN_POSITIVE);
    assert!((g.nitrogen_ledger().added.0 - 0.15).abs() < 1e-15);
}

/// Uptake follows root length and soil N, stops at the residual floors, never takes more
/// than the demand, and shares a cell between plants without regard to their order.
#[test]
fn nitrogen_uptake_is_limited_and_fair() {
    let mut g = column(2, 20.0);
    let a = RootCells::from_pairs(vec![(0, Meters(1.0))]);
    let b = RootCells::from_pairs(vec![(0, Meters(3.0)), (1, Meters(1.0))]);
    let req = |r| NitrogenRequest {
        demand: GramsN(1e-4),
        roots: r,
        capacity_per_length: 6e-4,
        hypoxic_factor: 1.0,
    };
    // A small demand is met exactly.
    let u = g.take_up_nitrogen(&[req(&a)]).unwrap();
    assert!((u[0].total().0 - 1e-4).abs() < 1e-18);
    // Unlimited demand: capacity · (1 − e^(−0.08 ppm)) per form.
    let mut h = column(2, 20.0);
    let big = NitrogenRequest {
        demand: GramsN(1.0),
        ..req(&a)
    };
    let u = h.take_up_nitrogen(&[big]).unwrap();
    let want = 6e-4 * (1.0 - libm::exp(-0.08 * 20.0));
    assert!(
        (u[0].no3.0 - want).abs() < 1e-15,
        "{} vs {want}",
        u[0].no3.0
    );
    // Order does not matter.
    let (mut p, mut q) = (column(2, 20.0), column(2, 20.0));
    let big_a = NitrogenRequest { roots: &a, ..big };
    let big_b = NitrogenRequest { roots: &b, ..big };
    let ab = p.take_up_nitrogen(&[big_a, big_b]).unwrap();
    let ba = q.take_up_nitrogen(&[big_b, big_a]).unwrap();
    assert_eq!(ab[0], ba[1]);
    assert_eq!(ab[1], ba[0]);
    // Long roots drain a cell to its floors and no further.
    let long = RootCells::from_pairs(vec![(0, Meters(1e4))]);
    let mut s = column(1, 20.0);
    for _ in 0..5 {
        s.take_up_nitrogen(&[NitrogenRequest {
            roots: &long,
            ..big
        }])
        .unwrap();
    }
    assert!((s.no3_ppm(0) - NO3_FLOOR_PPM).abs() < 1e-9);
    assert!(s.nitrogen_ledger().relative_residual(s.total_nitrogen()) < 1e-12);
}

/// Water soaking in carries nitrate down as it passes a cell (a mixing cell), so the
/// days' evaporation, which draws water and nitrate up, and the nights' watering back to
/// field capacity do not pump the nitrate into the top layer (R5a).
#[test]
fn soaking_in_carries_nitrate_down() {
    let fc = Texture::Loam.hydraulics().field_capacity;
    let mut g = SoilGrid::new_box(1, 1, 10, Meters(0.02), Texture::Loam, Celsius(20.0)).unwrap();
    g.set_drainage(crate::Drainage::NoHoles);
    g.set_nitrogen(&SoilNitrogen::NONE);
    g.fill_to(fc);
    g.no3[0] = GramsN(1e-3);
    g.water[0] = g.capacity(fc - 0.05);
    g.n_ledger.start = g.total_nitrogen();
    // 6 mm on a top cell 1 mm short of field capacity: what passes on (the rest of
    // the column is at field capacity, so the cell also fills toward saturation)
    // takes its share of the water the cell held plus the 6 mm that came in.
    let held_mm = g.water(0).0 / 4e-4;
    let mut l = WaterLedger::default();
    g.infiltrate(Millimeters(6.0), None, &mut l);
    let passed_mm = 6.0 - (g.water(0).0 / 4e-4 - held_mm);
    assert!(passed_mm > 1.0 && passed_mm < 5.0, "{passed_mm}");
    let sent = 1.0 - g.no3(0).0 / 1e-3;
    assert!((sent - passed_mm / (held_mm + 6.0)).abs() < 1e-9, "{sent}");
    // The rest soaks into the second cell, which keeps what it carried; none deeper.
    assert!(g.no3(1).0 > 0.0, "{:?}", g.no3);
    assert!(g.no3(2).0.abs() < f64::MIN_POSITIVE, "{:?}", g.no3);
    assert!((g.total_nitrogen().0 - 1e-3).abs() < 1e-15);
    // Forty days of sun on the bare surface and a refill each night: the top layer
    // gains some (140%), as salts gather where water evaporates, not all of it.
    let mut g = SoilGrid::new_box(1, 1, 10, Meters(0.02), Texture::Loam, Celsius(20.0)).unwrap();
    g.set_drainage(crate::Drainage::NoHoles);
    g.set_nitrogen(&SoilNitrogen::NONE);
    g.fill_to(fc);
    for i in 0..10 {
        g.no3[i] = GramsN(1e-4);
    }
    g.n_ledger.start = g.total_nitrogen();
    for h in 0..40 * 24 {
        let deficit: f64 = (0..10)
            .map(|i| (g.capacity(fc) - g.water(i)).0.max(0.0))
            .sum();
        let sun = (6..18).contains(&(h % 24));
        g.step_hour(&SoilHour {
            water_in: Millimeters(if h % 24 == 0 { deficit / 4e-4 } else { 0.0 }),
            surface_radiation: WPerM2(if sun { 500.0 } else { 0.0 }),
            ..hour(0.0, 20.0)
        });
    }
    assert!(g.no3(0).0 < 2e-4, "top {:?}", g.no3);
    assert!((g.total_nitrogen().0 - 1e-3).abs() < 1e-15);
}

/// R5b (decision 0048): a waterlogged cell loses nitrate to the air at
/// `DENITRIFICATION_RATE · f_T` an hour once it is hypoxic, and stops nitrifying; the
/// nitrogen ledger closes with the new line. A cell at field capacity loses none.
#[test]
fn a_waterlogged_cell_denitrifies_and_the_ledger_closes() {
    let mut g = column(2, 30.0);
    g.set_drainage(crate::Drainage::NoHoles);
    g.fill_to(Texture::Loam.hydraulics().saturation);
    let start = g.no3(1).0;
    let mut nitrified = 0.0;
    for h in 1..=48 {
        g.step_hour(&hour(0.0, 30.0));
        let l = g.nitrogen_ledger();
        assert!(l.relative_residual(g.total_nitrogen()) < 1e-12, "{l:?}");
        if h == 5 {
            nitrified = l.nitrified.0;
        }
    }
    let l = *g.nitrogen_ledger();
    assert!(l.denitrified.0 > 0.0);
    assert!(
        (l.nitrified.0 - nitrified).abs() <= 1e-15 * nitrified.abs(),
        "hypoxic soil nitrifies"
    );
    // 30 °C (f_T = 1): 42 hypoxic hours at 2% of the nitrate an hour, the first 6
    // before the cell turned hypoxic.
    let expected = start * libm::pow(1.0 - DENITRIFICATION_RATE, 43.0);
    assert!(
        (g.no3(1).0 / expected - 1.0).abs() < 0.05,
        "{} {expected}",
        g.no3(1).0
    );
    let mut fc = column(2, 30.0);
    for _ in 0..48 {
        fc.step_hour(&hour(0.0, 30.0));
    }
    assert!(fc.nitrogen_ledger().denitrified.0.abs() < 1e-15);
}
