//! WP27 (decision 0034) on a whole plant: transpiration taken from the soil equals the
//! plant's total, leaves get their own temperatures, and a clear, calm night just above
//! the frost threshold kills exposed leaves by radiation frost while the growing
//! points, which read air temperature, live. One bush bean in a 24 cm loam pot under
//! the temperate-summer preset at 45° N from 15 May, watered to field capacity nightly;
//! the driver runs the §3.2 steps in order as `ninety_days.rs` does.

use ps_arch::{DayReport, HourCtx, Plant};
use ps_graph::OrganKind;
use ps_soil::{SoilGrid, SoilHour, Texture};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, Meters, MetersPerSecond, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, HourWeather, Preset, WeatherSeries};

/// What each hour checks.
struct Hourly {
    /// Hours with leaves.
    hours: u32,
    /// Lowest and highest leaf-minus-air difference seen, K.
    dt_range: (f64, f64),
}

/// The plant's network flows, taken from the soil alone.
fn take(plant: &mut Plant, soil: &mut SoilGrid) -> ps_soil::JointUptake {
    let flows = plant.water_flows(soil);
    soil.take_flows(&[ps_soil::PlantFlows {
        demand: flows.demand,
        flows: &flows.flows,
    }])
    .unwrap()
}

/// Runs one plant for `days`; `edit` may change each hour's weather.
fn run(
    days: u32,
    edit: impl Fn(u64, &HourWeather) -> HourWeather,
    mut after_day: impl FnMut(u32, &Plant),
) -> (Plant, Hourly) {
    let species = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    let traits = species
        .phenotype(&species.default_genotype())
        .traits()
        .unwrap();
    let start = DateHour::new(2026, 5, 15, 0).unwrap();
    let wx: WeatherSeries =
        ps_weather::preset_series(Preset::TemperateSummer, Degrees(45.0), start, days + 1, 7);
    let cell = 0.02;
    let theta = Texture::Loam.hydraulics().field_capacity;
    let mut soil = SoilGrid::new_pot(
        Meters(0.24),
        Meters(0.24),
        Meters(cell),
        Texture::Loam,
        Celsius(15.0),
    )
    .unwrap();
    soil.fill_to(theta);
    soil.set_nitrogen(&ps_soil::SoilNitrogen::GARDEN);
    let (nx, ny, _) = soil.dims();
    #[allow(clippy::cast_precision_loss)]
    let center = [nx as f64 * cell * 0.5, ny as f64 * cell * 0.5, -0.03];
    let mut plant = Plant::sow(&species, &traits, 7, 1, center, Hours(0)).unwrap();
    let mut seen = Hourly {
        hours: 0,
        dt_range: (f64::MAX, f64::MIN),
    };
    let mut taken_today = 0.0;
    for tick in 0..u64::from(days) * 24 {
        let h = edit(tick, wx.at(Hours(tick)).unwrap());
        let ctx = HourCtx {
            weather: &h,
            hour: usize::try_from(tick % 24).unwrap(),
            tick: Hours(tick),
            co2: Ppm(420.0),
            patm: KPa(101.3),
        };
        let demand = plant.water_demand(&ctx, &soil).unwrap();
        assert!(demand.0 >= 0.0, "hour {tick}: demand {}", demand.0);
        let lai = plant.report().leaf_area / soil.surface_area().max(1e-9);
        soil.step_hour(&SoilHour {
            water_in: Millimeters(0.0),
            surface_radiation: WPerM2(h.global.0 * libm::exp(-0.5 * lai)),
            t_air: h.t_air,
            strip: None,
        });
        let uptake = take(&mut plant, &mut soil);
        // Done-when 6: what the soil gives plus what it could not equals the plant's
        // demand, and the soil's ledger books exactly what the plant took.
        let got = &uptake.plants[0];
        // The network's flows close to the round-off of its terms (decision 0044): at
        // zero demand that is about 1e-16 kg, so the tolerance has an absolute floor.
        let scale = demand.0.max(1e-12);
        assert!(
            (got.actual.0 + got.shortfall.0 - demand.0).abs() <= 1e-12 + 1e-9 * scale,
            "hour {tick}: {} + {} vs {}",
            got.actual.0,
            got.shortfall.0,
            demand.0
        );
        assert!(
            (uptake.ledger.uptake.0 - got.actual.0).abs() <= 1e-9 * got.actual.0.max(1e-12),
            "hour {tick}: ledger {} vs plant {}",
            uptake.ledger.uptake.0,
            got.actual.0
        );
        taken_today += got.actual.0;
        let mut any = false;
        for id in plant.arena.alive_of(OrganKind::Leaf) {
            if plant.arena.size[id.index()].area.0 <= 0.0 {
                continue;
            }
            let t = plant.meta[id.index()]
                .t_leaf
                .expect("a leaf with area has a temperature");
            let d = t.0 - h.t_air.0;
            seen.dt_range = (seen.dt_range.0.min(d), seen.dt_range.1.max(d));
            any = true;
        }
        seen.hours += u32::from(any);
        plant.grow_hour(&ctx, &soil, uptake.plants.first()).unwrap();
        if tick % 24 == 23 {
            let mut sun = [None; 24];
            for (k, s) in sun.iter_mut().enumerate() {
                let next = wx.at(Hours(tick + 1 + k as u64)).unwrap_or(&h);
                *s = next.sun.is_up().then(|| next.sun.direction());
            }
            plant.nitrogen_alone(&mut soil, Hours(tick + 1)).unwrap();
            check_day(
                &plant.day(&soil, &sun, Hours(tick + 1)).unwrap(),
                taken_today,
            );
            taken_today = 0.0;
            soil.fill_to(theta);
            after_day(u32::try_from(tick / 24).unwrap(), &plant);
        }
    }
    (plant, seen)
}

/// The daily rows carry the day's transpiration (what the soil gave) and net
/// assimilation, for the viewer's charts.
fn check_day(rep: &DayReport, taken: f64) {
    assert!(
        (rep.transpiration - taken).abs() <= 1e-12 * taken.max(1e-12),
        "day {}: {} vs {taken}",
        rep.day,
        rep.transpiration
    );
    assert!(rep.assimilation.is_finite());
    if rep.leaf_area > 0.01 {
        assert!(rep.assimilation > 0.0 && rep.transpiration > 0.0, "{rep:?}");
    }
}

/// Done-when 6 and plausible leaf temperatures over a month of summer.
#[test]
fn transpiration_from_the_soil_equals_the_plant_total() {
    let (_, seen) = run(30, |_, h| *h, |_, _| {});
    println!(
        "{} hours with leaves; leaf minus air {:+.2} to {:+.2} K",
        seen.hours, seen.dt_range.0, seen.dt_range.1
    );
    assert!(seen.hours > 400);
    // Sunlit leaves in light wind warm a few kelvin; leaves on clear nights cool a few.
    assert!(seen.dt_range.0 > -6.0 && seen.dt_range.0 < 0.0);
    assert!(seen.dt_range.1 > 0.0 && seen.dt_range.1 < 12.0);
}

/// Saturation vapor pressure, kPa (the weather's Tetens form).
fn es(t: f64) -> f64 {
    0.6108 * libm::exp(17.27 * t / (t + 237.3))
}

/// The owner's decision (2026-11-05): leaf frost reads leaf temperature, growing
/// points air. Night 25 is clear and calm with the air at −1.5 °C, above the bean's
/// −2.2 °C frost kill: exposed leaves radiate below it and die; the apices live, so
/// the plant keeps growing.
#[test]
fn a_clear_calm_night_frosts_exposed_leaves_but_not_the_growing_points() {
    const NIGHT: u32 = 25;
    let cold = |tick: u64, h: &HourWeather| {
        let (day, hour) = (tick / 24, tick % 24);
        if day == u64::from(NIGHT) && hour <= 5 {
            let t = -1.5;
            HourWeather {
                t_air: Celsius(t),
                rh_pct: 85.0,
                vpd: KPa(es(t) * 0.15),
                global: WPerM2(0.0),
                direct: WPerM2(0.0),
                diffuse: WPerM2(0.0),
                cloud_fraction: 0.0,
                wind: MetersPerSecond(0.5),
                ..*h
            }
        } else {
            *h
        }
    };
    let mut leaves = Vec::new();
    let (plant, _) = run(NIGHT + 4, cold, |day, p| {
        leaves.push((day, p.report().leaves));
    });
    let at = |d: u32| leaves.iter().find(|(x, _)| *x == d).unwrap().1;
    println!("leaves by day: {leaves:?}");
    let (before, after) = (at(NIGHT - 1), at(NIGHT));
    assert!(after < before, "no leaf froze: {before} -> {after}");
    assert!(after > 0, "every leaf froze");
    assert!(
        plant.axes.iter().any(|a| !a.terminated),
        "the growing points froze at air temperature above the threshold"
    );
}
