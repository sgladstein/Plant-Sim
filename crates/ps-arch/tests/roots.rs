//! R2a (decision 0037) on a whole plant: root axes grow segment after segment without
//! waiting for the daily step, fine-root cohorts sit on finished first-order laterals
//! with the multiplicity of the roots they stand for, and the plant's water supply
//! keeps up with its demand in a watered pot. One plant in a 24 cm loam pot under the
//! temperate-summer preset at 45° N from 15 May, watered to field capacity nightly.

use ps_arch::{HourCtx, Plant};
use ps_graph::OrganKind;
use ps_soil::{SoilGrid, SoilHour, Texture};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, Meters, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, Preset, WeatherSeries};

/// Runs one plant of `species` for `days`; returns it with the water it took up and the
/// water it asked for over the run, kg.
fn run(species: &str, days: u32) -> (Plant, f64, f64) {
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
    let (mut took, mut asked) = (0.0, 0.0);
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
        let u = &uptake.plants[0];
        took += u.actual.0;
        asked += u.actual.0 + u.shortfall.0;
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
        }
    }
    (plant, took, asked)
}

/// R2a done-when, bean side.
#[test]
fn bean_roots_branch_twice_and_keep_up() {
    let (p, took, asked) = run(include_str!("../../../data/species/bush_bean.ron"), 45);
    let a = &p.arena;
    let order = |i: usize| a.axis_order[i];
    let lateral = p.params.roots.iter().find(|r| r.order == 1).unwrap();
    let mut length = [0.0; 3];
    for id in a.alive_of(OrganKind::Root) {
        let i = id.index();
        length[usize::from(order(i)).min(2)] += a.size[i].length.0;
        if order(i) != 2 {
            continue;
        }
        // Each fine cohort hangs on a finished first-order cohort and stands for its
        // fine roots: m2 = m1 (L1 - apical zone) / spacing.
        let h = a.parent[i].unwrap().index();
        assert_eq!(order(h), 1);
        assert!(a.curve[h].is_complete(a.expansion_age[h]));
        let m1 = p.meta[h].multiplicity;
        let l1 = a.size[h].length.0 / m1;
        let m2 = m1 * (l1 - lateral.apical_zone.0) / lateral.branch_spacing.0;
        assert!(
            (p.meta[i].multiplicity - m2).abs() <= 1e-9 * m2,
            "{} vs {m2}",
            p.meta[i].multiplicity
        );
    }
    let total: f64 = length.iter().sum();
    let fine = length[2] / total;
    println!("root length {total:.1} m, by order {length:.1?}, fine share {fine:.3}");
    assert!(fine > 0.0 && fine < 0.4, "fine share {fine}");
    assert!(total > 20.0, "root length {total} m");
    // Axes: segments follow each other with no gap, and at most the tip waits.
    for ax in p
        .root_axes
        .iter()
        .filter(|x| p.params.roots[x.kind].order == 0)
    {
        let waiting = ax
            .segments
            .iter()
            .filter(|(id, _)| a.expansion_age[id.index()].0 < 0.0)
            .count();
        assert!(waiting <= 1);
        for w in ax.segments.windows(2) {
            let (i, s0) = (w[0].0.index(), w[0].1);
            // A growing segment counts the most it can reach: its length now plus the
            // rest of its potential.
            let c = &a.curve[i];
            let len = a.size[i].length.0
                + if c.is_complete(a.expansion_age[i]) {
                    0.0
                } else {
                    match a.specific[i] {
                        ps_graph::SpecificSize::Length(sl) => {
                            c.potential.0 * (1.0 - c.fraction(a.expansion_age[i])) * sl.0
                        }
                        _ => unreachable!(),
                    }
                };
            assert!((w[1].1 - (s0 + len)).abs() < 1e-9, "{:?}", ax.segments);
        }
    }
    // The watered pot meets nearly all of its demand.
    let share = took / asked;
    println!("water demand met {share:.3}");
    assert!(share > 0.95, "met {share}");
}
