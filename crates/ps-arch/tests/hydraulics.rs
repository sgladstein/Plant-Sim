//! R3b (decision 0044) on a whole plant: the water network. One bush bean in a 24 cm
//! loam pot under the temperate-summer preset at 45° N from 15 May, sown at the
//! center and watered back to field capacity every night.
//!
//! - Day cycle: on sunny days in the flowering weeks, predawn leaf water potential is
//!   within 0.2 `MPa` of the root zone's soil potential and the midday low is −0.4 to
//!   −1.1 `MPa` (R3b spec §4.5, R3 numbers change 4).
//! - At night the network moves water only between cells: its net flow is the leaves'
//!   small night transpiration, and every organ sits between the wettest and driest
//!   root cell's potential.
//! - Compensation: in a split pot whose east half is left to dry from day 20 (the R2b
//!   split pot, run to day 60 so the east half gets that dry), once the east half's
//!   roots are below 30% of their available water, more than half of the uptake comes
//!   from the watered west half.

use ps_arch::{HourCtx, Plant};
use ps_soil::{PlantFlows, SoilGrid, SoilHour, SoilNitrogen, Strip, Texture};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, Meters, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, Preset, WeatherSeries};

const CELL: f64 = 0.02;

#[derive(Default)]
struct Log {
    /// (day, predawn leaf ψ, predawn root-zone soil ψ, daily low leaf ψ, daily light MJ).
    days: Vec<(u32, f64, f64, f64, f64)>,
    /// Night hours: (net flow, demand, lowest organ ψ, highest organ ψ, driest cell ψ,
    /// wettest cell ψ), `MPa` and kg.
    nights: Vec<(f64, f64, f64, f64, f64, f64)>,
    /// Uptake from the west and east halves in hours when the east roots are below
    /// 30% of their available water, kg.
    halves: (f64, f64),
    /// Split pot: water the roots gave back to drier cells and took from wetter ones
    /// over the run, kg (hydraulic redistribution against gross uptake).
    given: f64,
    taken: f64,
}

/// FTSW of the plant's roots east of the divider.
fn east_ftsw(p: &Plant, soil: &SoilGrid) -> f64 {
    let (nx, _, _) = soil.dims();
    let (mut a, mut b) = (0.0, 0.0);
    for &(i, l) in p.roots.cells() {
        if soil.cell_of(i).x >= nx / 2 {
            a += l.0 * soil.cell_ftsw(i);
            b += l.0;
        }
    }
    if b > 0.0 { a / b } else { 1.0 }
}

/// Logs a night hour: the network's net flow and demand, and the organ and cell
/// potentials it sits between.
fn log_night(log: &mut Log, plant: &Plant, flows: &ps_arch::WaterFlows, soil: &SoilGrid) {
    let retention = soil.retention();
    let net: f64 = flows.flows.iter().map(|f| f.1.0).sum();
    let psi = plant.water_potentials();
    let alive: Vec<f64> = plant
        .arena
        .alive_ids()
        .map(|id| psi[id.index()].0)
        .collect();
    let cells: Vec<f64> = flows
        .flows
        .iter()
        .map(|&(c, _)| retention.psi(soil.theta(c)).to_mpa().0)
        .collect();
    let lo = |v: &[f64]| v.iter().copied().fold(f64::MAX, f64::min);
    let hi = |v: &[f64]| v.iter().copied().fold(f64::MIN, f64::max);
    log.nights.push((
        net,
        flows.demand.0,
        lo(&alive),
        hi(&alive),
        lo(&cells),
        hi(&cells),
    ));
}

/// Logs a split-pot hour: water given back and taken, and, when the east roots are
/// dry, the uptake from each half.
fn log_split(log: &mut Log, uptake: &ps_soil::Uptake, soil: &SoilGrid, dry_east: bool) {
    let (nx, _, _) = soil.dims();
    for &(c, q) in &uptake.per_cell {
        if q.0 < 0.0 {
            log.given -= q.0;
        } else {
            log.taken += q.0;
        }
        if dry_east {
            if soil.cell_of(c).x < nx / 2 {
                log.halves.0 += q.0;
            } else {
                log.halves.1 += q.0;
            }
        }
    }
}

/// The 24 cm loam pot at field capacity, with a divider for the split pot.
fn pot(split: bool) -> SoilGrid {
    let mut soil = SoilGrid::new_pot(
        Meters(0.24),
        Meters(0.24),
        Meters(CELL),
        Texture::Loam,
        Celsius(15.0),
    )
    .unwrap();
    soil.fill_to(Texture::Loam.hydraulics().field_capacity);
    soil.set_nitrogen(&SoilNitrogen::GARDEN);
    if split {
        soil.set_divider();
    }
    soil
}

/// Runs one bush bean for `days`; with `split`, a divider and only the west half
/// watered from day 20.
fn run(split: bool, days: u32) -> Log {
    let species = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    let traits = species
        .phenotype(&species.default_genotype())
        .traits()
        .unwrap();
    let start = DateHour::new(2026, 5, 15, 0).unwrap();
    let wx: WeatherSeries =
        ps_weather::preset_series(Preset::TemperateSummer, Degrees(45.0), start, days + 1, 7);
    let h = Texture::Loam.hydraulics();
    let mut soil = pot(split);
    let (nx, ny, _) = soil.dims();
    #[allow(clippy::cast_precision_loss)]
    let center = [nx as f64 * CELL * 0.5, ny as f64 * CELL * 0.5, -0.03];
    let mut plant = Plant::sow(&species, &traits, 7, 1, center, Hours(0)).unwrap();
    // The pot's field-capacity deficit, mm over its surface.
    let deficit = |soil: &SoilGrid| {
        let kg: f64 = (0..soil.len())
            .filter(|&i| soil.is_active(i))
            .map(|i| (h.field_capacity - soil.theta(i)).max(0.0) * soil.cell_volume() * 1000.0)
            .sum();
        Millimeters(kg / soil.surface_area().max(1e-12))
    };
    let west_deficit = |soil: &SoilGrid| {
        let mut kg = 0.0;
        for i in (0..soil.len()).filter(|&i| soil.is_active(i) && soil.cell_of(i).x < nx / 2) {
            kg += (h.field_capacity - soil.theta(i)).max(0.0) * soil.cell_volume() * 1000.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let area = (nx / 2 * ny) as f64 * CELL * CELL;
        Millimeters(kg / area)
    };
    let mut log = Log::default();
    let mut light = 0.0;
    for tick in 0..u64::from(days) * 24 {
        let w = *wx.at(Hours(tick)).unwrap();
        let hour = usize::try_from(tick % 24).unwrap();
        let ctx = HourCtx {
            weather: &w,
            hour,
            tick: Hours(tick),
            co2: Ppm(420.0),
            patm: KPa(101.3),
        };
        light += w.global.0 * 3600.0 * 1e-6;
        plant.water_demand(&ctx, &soil).unwrap();
        let lai = plant.report().leaf_area / soil.surface_area().max(1e-9);
        let strip = (split && tick >= 20 * 24 && hour == 0).then(|| Strip {
            water: west_deficit(&soil),
            x0: 0,
            x1: nx / 2,
        });
        // Watered through the surface, so the water soaking in carries back down the
        // nitrate the day's evaporation drew up (R5a).
        let water_in = if tick > 0 && hour == 0 && (!split || tick < 20 * 24) {
            deficit(&soil)
        } else {
            Millimeters(0.0)
        };
        soil.step_hour(&SoilHour {
            water_in,
            surface_radiation: WPerM2(w.global.0 * libm::exp(-0.5 * lai)),
            t_air: w.t_air,
            strip,
        });
        let flows = plant.water_flows(&soil);
        let dry_east = split && tick >= 20 * 24 && east_ftsw(&plant, &soil) < 0.3;
        if !split && hour == 2 && !flows.flows.is_empty() {
            log_night(&mut log, &plant, &flows, &soil);
        }
        let uptake = soil
            .take_flows(&[PlantFlows {
                demand: flows.demand,
                flows: &flows.flows,
            }])
            .unwrap();
        if split {
            log_split(&mut log, &uptake.plants[0], &soil, dry_east);
        }
        plant.grow_hour(&ctx, &soil, uptake.plants.first()).unwrap();
        if hour == 23 {
            let r = plant.report();
            if r.leaf_area > 0.0 {
                log.days.push((
                    plant.day,
                    r.leaf_psi_predawn,
                    r.soil_psi_predawn,
                    r.leaf_psi_min,
                    light,
                ));
            }
            light = 0.0;
            let mut sun = [None; 24];
            for (k, s) in sun.iter_mut().enumerate() {
                let next = wx.at(Hours(tick + 1 + k as u64)).unwrap_or(&w);
                *s = next.sun.is_up().then(|| next.sun.direction());
            }
            plant.nitrogen_alone(&mut soil, Hours(tick + 1)).unwrap();
            plant.day(&soil, &sun, Hours(tick + 1)).unwrap();
        }
    }
    log
}

#[test]
fn leaf_water_potential_follows_the_day() {
    let log = run(false, 60);
    // Sunny days (the brightest third) from day 35 to 60, when the canopy is full.
    let mut late: Vec<_> = log.days.iter().filter(|d| d.0 >= 35).copied().collect();
    late.sort_by(|a, b| b.4.total_cmp(&a.4));
    let sunny = &late[..late.len() / 3];
    assert!(!sunny.is_empty());
    for &(day, pd, soil, low, _) in sunny {
        println!("day {day}: predawn {pd:.3}, soil {soil:.3}, low {low:.3}");
        assert!(
            (pd - soil).abs() <= 0.2,
            "day {day}: predawn {pd} against soil {soil}"
        );
        assert!((-1.1..=-0.4).contains(&low), "day {day}: midday low {low}");
    }
    // At night the plant's net uptake is its night transpiration, and every organ
    // sits between its wettest and driest root cell.
    assert!(!log.nights.is_empty());
    for &(net, demand, lo, hi, dry, wet) in &log.nights {
        assert!(
            (net - demand).abs() <= 1e-12 + 1e-9 * demand,
            "{net} against {demand}"
        );
        assert!(
            lo >= dry - 0.05 && hi <= wet + 0.05,
            "organs {lo}..{hi}, soil {dry}..{wet}"
        );
    }
}

#[test]
fn a_split_pot_draws_on_its_wet_half() {
    let log = run(true, 60);
    let (west, east) = log.halves;
    println!(
        "uptake once the east half is dry: west {west:.3} kg, east {east:.3} kg; \
         over the run the roots gave back {:.3} kg of {:.3} kg taken",
        log.given, log.taken
    );
    assert!(west + east > 0.0, "the east half never fell below 30% FTSW");
    assert!(west > 0.5 * (west + east), "west {west}, east {east}");
}
