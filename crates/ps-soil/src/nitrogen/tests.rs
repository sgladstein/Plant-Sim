use super::*;
use crate::{SoilHour, Texture, WaterLedger};
use ps_units::{Celsius, Meters, Millimeters, WPerM2};

fn hour(rain_mm: f64, t: f64) -> SoilHour {
    SoilHour {
        water_in: Millimeters(rain_mm),
        surface_radiation: WPerM2(0.0),
        t_air: Celsius(t),
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

#[test]
fn nitrate_spreads_sideways_with_the_water() {
    let mut g = SoilGrid::new_box(3, 1, 1, Meters(0.05), Texture::Loam, Celsius(0.0)).unwrap();
    g.set_nitrogen(&SoilNitrogen::NONE);
    g.no3[0] = GramsN(1e-3);
    g.n_ledger.start = g.total_nitrogen();
    for _ in 0..200 {
        g.step_hour(&hour(0.0, 0.0));
    }
    assert!(g.no3(2).0 > 1e-4, "{:?}", g.no3);
    assert!((g.total_nitrogen().0 - 1e-3).abs() < 1e-15);
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
        reach: Meters(0.0),
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

/// The root reach spreads each root cell's length evenly over the active cells within
/// it, keeps the total length, and does nothing below half a cell.
#[test]
fn root_reach_spreads_length_over_nearby_cells() {
    let g = SoilGrid::new_box(5, 5, 5, Meters(0.02), Texture::Loam, Celsius(20.0)).unwrap();
    let center = g.index(Cell { x: 2, y: 2, z: 2 });
    let corner = g.index(Cell { x: 0, y: 0, z: 0 });
    let roots = RootCells::from_pairs(vec![(center, Meters(2.7)), (corner, Meters(0.8))]);
    assert_eq!(g.reached_cells(&roots, Meters(0.009)), roots);
    let r = g.reached_cells(&roots, Meters(0.02));
    let total: f64 = r.cells().iter().map(|c| c.1.0).sum();
    assert!((total - 3.5).abs() < 1e-12);
    // The center's 27 neighbors get 0.1 m each; the corner's 8 get 0.1 m each.
    let at = |c: Cell| {
        let i = g.index(c);
        r.cells().iter().find(|x| x.0 == i).map_or(0.0, |x| x.1.0)
    };
    assert!((at(Cell { x: 3, y: 3, z: 3 }) - 0.1).abs() < 1e-12);
    assert!((at(Cell { x: 1, y: 1, z: 1 }) - 0.2).abs() < 1e-12);
    assert!(at(Cell { x: 4, y: 4, z: 4 }).abs() < 1e-12);
}

/// The original form of [`SoilGrid::reached_cells`]: a list of the active cells in each
/// root cell's cube, then the shares added on a dense grid. The summed-volume form must
/// give the same bits.
fn reached_cells_reference(g: &SoilGrid, roots: &RootCells, reach: Meters) -> RootCells {
    let r = libm::round((reach.0 / g.cell_size().0).max(0.0)).min(8.0) as usize;
    if r == 0 {
        return roots.clone();
    }
    let (nx, ny, nz) = g.dims();
    let mut length = vec![0.0_f64; g.len()];
    let mut touched = vec![false; g.len()];
    for &(i, l) in roots.cells() {
        let c = g.cell_of(i);
        let mut near = Vec::new();
        for z in c.z.saturating_sub(r)..=(c.z + r).min(nz - 1) {
            for y in c.y.saturating_sub(r)..=(c.y + r).min(ny - 1) {
                for x in c.x.saturating_sub(r)..=(c.x + r).min(nx - 1) {
                    let j = g.index(Cell { x, y, z });
                    if g.is_active(j) {
                        near.push(j);
                    }
                }
            }
        }
        if near.is_empty() {
            continue;
        }
        let share = l.0 / near.len() as f64;
        for &j in &near {
            if touched[j] {
                length[j] += share;
            } else {
                touched[j] = true;
                length[j] = share;
            }
        }
    }
    RootCells::from_pairs(
        (0..g.len())
            .filter(|&j| touched[j] && length[j] > 0.0)
            .map(|j| (j, Meters(length[j])))
            .collect(),
    )
}

/// In a round pot (inactive corners) with overlapping reaches, the summed-volume
/// reach matches the original cell by cell, bit for bit.
#[test]
fn root_reach_matches_the_reference_bit_for_bit() {
    let g = SoilGrid::new_pot(
        Meters(0.2),
        Meters(0.25),
        Meters(0.02),
        Texture::Loam,
        Celsius(20.0),
    )
    .unwrap();
    let n = g.len();
    let pairs: Vec<(usize, Meters)> = (0..400usize)
        .map(|k| {
            (
                (k * 7919 + 13) % n,
                Meters(0.001 + 0.37 * ((k * 31 % 17) as f64)),
            )
        })
        .collect();
    let roots = RootCells::from_pairs(pairs);
    for reach in [0.009, 0.02, 0.04, 0.07, 0.2] {
        let a = g.reached_cells(&roots, Meters(reach));
        let b = reached_cells_reference(&g, &roots, Meters(reach));
        assert_eq!(a.cells().len(), b.cells().len(), "reach {reach}");
        for (x, y) in a.cells().iter().zip(b.cells()) {
            assert_eq!(x.0, y.0);
            assert_eq!(
                x.1.0.to_bits(),
                y.1.0.to_bits(),
                "reach {reach} cell {}",
                x.0
            );
        }
    }
}
