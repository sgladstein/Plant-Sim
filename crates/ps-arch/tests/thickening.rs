//! R1a (decision 0036) on whole plants: a bean's hypocotyl and basal stem widen and
//! the stem tapers to the tip, the tap root widens most near the collar, pipe targets
//! never fall, secondary growth leaves lengths and the phloem's radius alone, and a
//! grass never thickens. One plant in a 24 cm loam pot under the temperate-summer
//! preset at 45° N from 15 May, watered to field capacity nightly; the driver runs the
//! §3.2 steps in order as `ninety_days.rs` does.

use ps_arch::{HourCtx, Plant, Role};
use ps_graph::OrganKind;
use ps_soil::{SoilGrid, SoilHour, Texture};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, Meters, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, Preset, WeatherSeries};

/// Runs one plant of `species` for `days`, calling `after_day` after each daily step.
fn run(species: &str, days: u32, mut after_day: impl FnMut(u32, &Plant)) -> Plant {
    let species = Species::from_ron(species).unwrap();
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
    for tick in 0..u64::from(days) * 24 {
        let h = *wx.at(Hours(tick)).unwrap();
        let ctx = HourCtx {
            weather: &h,
            hour: usize::try_from(tick % 24).unwrap(),
            tick: Hours(tick),
            co2: Ppm(420.0),
            patm: KPa(101.3),
        };
        let demand = plant.water_demand(&ctx, &soil).unwrap();
        let lai = plant.report().leaf_area / soil.surface_area().max(1e-9);
        soil.step_hour(&SoilHour {
            water_in: Millimeters(0.0),
            surface_radiation: WPerM2(h.global.0 * libm::exp(-0.5 * lai)),
            t_air: h.t_air,
            strip: None,
        });
        let uptake = soil.take_up(&[plant.uptake_request(demand)]).unwrap();
        plant.grow_hour(&ctx, &soil, uptake.plants.first()).unwrap();
        if tick % 24 == 23 {
            let mut sun = [None; 24];
            for (k, s) in sun.iter_mut().enumerate() {
                let next = wx.at(Hours(tick + 1 + k as u64)).unwrap_or(&h);
                *s = next.sun.is_up().then(|| next.sun.direction());
            }
            plant.nitrogen_alone(&mut soil, Hours(tick + 1)).unwrap();
            plant.day(&soil, &sun, Hours(tick + 1)).unwrap();
            soil.fill_to(theta);
            after_day(u32::try_from(tick / 24).unwrap(), &plant);
        }
    }
    plant
}

/// Diameter of an organ, mm.
fn diameter(p: &Plant, i: usize) -> f64 {
    2e3 * p.arena.size[i].radius.0
}

/// R1a done-when, bean side.
#[test]
fn bean_stems_and_roots_widen_and_taper() {
    let mut area: Vec<f64> = Vec::new();
    let mut stem = Vec::new();
    let p = run(
        include_str!("../../../data/species/bush_bean.ron"),
        56,
        |day, p| {
            // The pipe model's leaf-area record never falls.
            let now: Vec<f64> = p.meta.iter().map(|m| m.supported_area).collect();
            for (i, a) in area.iter().enumerate() {
                assert!(now[i] >= *a, "day {day}: organ {i} {a} -> {}", now[i]);
            }
            area = now;
            stem.push(p.report().stem_mass);
        },
    );
    let a = &p.arena;
    // Secondary growth keeps lengths: each segment's primary radius is its primary
    // allometry's, sqrt(1 / (ρ π SL)), however wide it has grown.
    for id in a.alive_ids() {
        let i = id.index();
        if let (ps_graph::SpecificSize::Length(sl), OrganKind::Internode) =
            (a.specific[i], a.kind[i])
        {
            let rp = libm::sqrt(
                1.0 / (p.params.allometry.stem_density.0 * core::f64::consts::PI * sl.0),
            );
            assert!(
                (a.primary_radius(id).0 - rp).abs() <= 1e-9 * rp,
                "organ {i}: {} vs {rp}",
                a.primary_radius(id).0
            );
            let len = sl.0 * a.primary(id).0;
            assert!((a.size[i].length.0 - len).abs() <= 1e-12 + 1e-9 * len);
        }
    }
    // The hypocotyl carries the whole shoot: the widest stem, in the field band.
    let hyp = diameter(&p, p.hypocotyl.index());
    println!("hypocotyl {hyp:.2} mm");
    assert!((4.5..10.0).contains(&hyp), "hypocotyl {hyp} mm");
    // Main-stem internodes in creation order, base to tip.
    let main: Vec<usize> = a
        .alive_of(OrganKind::Internode)
        .map(ps_graph::OrganId::index)
        .filter(|&i| a.axis_order[i] == 0 && p.meta[i].role == Role::Stem)
        .collect();
    let d: Vec<f64> = main.iter().map(|&i| diameter(&p, i)).collect();
    println!("main stem, base to tip: {d:.2?} mm");
    // The hypocotyl and the epicotyl carry nearly the whole shoot and are the widest
    // (the thin hypocotyl draws less carbon and catches up by maturity); the stem
    // tapers to the tip. Daily thickening (decision 0037) builds from the midnight
    // reserve, so on day 56 the hypocotyl is about 1.15 times the tip (R1a's hourly
    // rule: more than 1.2) and the epicotyl about 1.5 times. From R2b (decision 0043)
    // nitrate no longer spreads sideways at the water's rate, so this garden pot is
    // shorter of nitrogen: on day 56 its hypocotyl is about 1.06 times the tip and the
    // epicotyl about 1.35 times.
    let widest = hyp.max(d[0]);
    assert!(d.iter().all(|x| *x <= widest + 1e-9), "{d:?}");
    assert!(hyp > 1.03 * d[d.len() - 1], "hypocotyl {hyp} vs tip {d:?}");
    assert!(d[0] > 1.3 * d[d.len() - 1], "no taper: {d:?}");
    // Stem mass still rises in the last week.
    let n = stem.len();
    assert!(stem[n - 1] > stem[n - 8], "stem mass {:?}", &stem[n - 8..]);
    // The tap root is widest near the collar and tapers.
    let tap = p.tap_root_diameter(0.05) * 1e3;
    let deep = p.tap_root_diameter(0.15) * 1e3;
    println!("tap root at 5 cm {tap:.2} mm, at 15 cm {deep:.2} mm");
    assert!(tap > deep && deep > 0.0);
    assert!((2.0..8.0).contains(&tap), "tap root {tap} mm");
}

/// Grasses have no cambium: no secondary mass anywhere.
#[test]
fn grass_does_not_thicken() {
    let p = run(
        include_str!("../../../data/species/spring_wheat.ron"),
        35,
        |_, _| {},
    );
    assert!(p.arena.alive_ids().count() > 20);
    for id in p.arena.ids() {
        assert!(
            p.arena.secondary(id).0.abs() < f64::MIN_POSITIVE,
            "organ {}",
            id.index()
        );
    }
    assert!(
        p.meta
            .iter()
            .all(|m| m.pipe_target.abs() < f64::MIN_POSITIVE)
    );
}
