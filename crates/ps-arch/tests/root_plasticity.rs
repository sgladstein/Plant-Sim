//! R2b (decision 0043) on a whole plant: roots forage. One bush bean in a 24 cm loam
//! pot under the temperate-summer preset at 45° N from 15 May, sown at the center.
//! A split pot (a divider, the west half watered every night, the east half left to dry
//! from day 15) grows more root west than east, and more so with the rules than
//! without; a nitrate patch on the tap root's path draws laterals.

use ps_arch::{HourCtx, Plant, params::RootPlasticity};
use ps_soil::{Cell, SoilGrid, SoilHour, SoilNitrogen, Strip, Texture};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, Meters, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, Preset, WeatherSeries};

const CELL: f64 = 0.02;

/// The pot's cells 6 to 14 cm deep within 4 cm of its axis: the patch.
fn patch(soil: &SoilGrid) -> Vec<usize> {
    (0..soil.len())
        .filter(|&i| {
            let c = soil.cell_of(i);
            soil.is_active(i)
                && (4..8).contains(&c.x)
                && (4..8).contains(&c.y)
                && (3..7).contains(&c.z)
        })
        .collect()
}

enum Setup {
    /// Divider; the east half is not watered from day 15.
    Split,
    /// The patch holds this many times the garden's nitrate.
    Patch(f64),
}

/// Runs one bush bean for `days`; `plastic` false turns the R2b rules off.
fn run(setup: &Setup, plastic: bool, days: u32) -> (Plant, SoilGrid) {
    let species = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    let traits = species
        .phenotype(&species.default_genotype())
        .traits()
        .unwrap();
    let start = DateHour::new(2026, 5, 15, 0).unwrap();
    let wx: WeatherSeries =
        ps_weather::preset_series(Preset::TemperateSummer, Degrees(45.0), start, days + 1, 7);
    let h = Texture::Loam.hydraulics();
    let mut soil = SoilGrid::new_pot(
        Meters(0.24),
        Meters(0.24),
        Meters(CELL),
        Texture::Loam,
        Celsius(15.0),
    )
    .unwrap();
    soil.fill_to(h.field_capacity);
    soil.set_nitrogen(&SoilNitrogen::GARDEN);
    let (nx, ny, _) = soil.dims();
    match setup {
        Setup::Split => soil.set_divider(),
        Setup::Patch(f) => {
            let cells = patch(&soil);
            soil.scale_nitrate_cells(&cells, *f);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let center = [nx as f64 * CELL * 0.5, ny as f64 * CELL * 0.5, -0.03];
    let mut plant = Plant::sow(&species, &traits, 7, 1, center, Hours(0)).unwrap();
    if !plastic {
        plant.params.root_plasticity = RootPlasticity::default();
    }
    // Night watering of the west half: its own field-capacity deficit, mm over it.
    let west_deficit = |soil: &SoilGrid| {
        let mut kg = 0.0;
        for i in (0..soil.len()).filter(|&i| soil.is_active(i) && soil.cell_of(i).x < nx / 2) {
            kg += (h.field_capacity - soil.theta(i)).max(0.0) * soil.cell_volume() * 1000.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let area = (nx / 2 * ny) as f64 * CELL * CELL;
        Millimeters(kg / area)
    };
    for tick in 0..u64::from(days) * 24 {
        let w = *wx.at(Hours(tick)).unwrap();
        let ctx = HourCtx {
            weather: &w,
            hour: usize::try_from(tick % 24).unwrap(),
            tick: Hours(tick),
            co2: Ppm(420.0),
            patm: KPa(101.3),
        };
        let demand = plant.water_demand(&ctx, &soil).unwrap();
        let lai = plant.report().leaf_area / soil.surface_area().max(1e-9);
        let strip =
            (matches!(setup, Setup::Split) && tick >= 15 * 24 && tick % 24 == 0).then(|| Strip {
                water: west_deficit(&soil),
                x0: 0,
                x1: nx / 2,
            });
        soil.step_hour(&SoilHour {
            water_in: Millimeters(0.0),
            surface_radiation: WPerM2(w.global.0 * libm::exp(-0.5 * lai)),
            t_air: w.t_air,
            strip,
        });
        let uptake = soil.take_up(&[plant.uptake_request(demand)]).unwrap();
        plant.grow_hour(&ctx, &soil, uptake.plants.first()).unwrap();
        if tick % 24 == 23 {
            let mut sun = [None; 24];
            for (k, s) in sun.iter_mut().enumerate() {
                let next = wx.at(Hours(tick + 1 + k as u64)).unwrap_or(&w);
                *s = next.sun.is_up().then(|| next.sun.direction());
            }
            plant.nitrogen_alone(&mut soil, Hours(tick + 1)).unwrap();
            plant.day(&soil, &sun, Hours(tick + 1)).unwrap();
            if matches!(setup, Setup::Patch(_)) || tick < 15 * 24 {
                soil.fill_to(h.field_capacity);
            }
        }
    }
    (plant, soil)
}

/// Root length (axis order `min_order` and up) in each soil cell.
fn lengths(p: &Plant, soil: &SoilGrid, min_order: u8) -> Vec<(usize, f64)> {
    ps_arch::geometry::root_cells_from(&p.arena, &p.positions, soil, min_order)
        .cells()
        .iter()
        .map(|(i, l)| (*i, l.0))
        .collect()
}

/// Root length west of the divider over that east of it.
fn west_east(p: &Plant, soil: &SoilGrid) -> f64 {
    let (nx, _, _) = soil.dims();
    let (mut w, mut e) = (0.0, 0.0);
    for (i, l) in lengths(p, soil, 0) {
        if soil.cell_of(i).x < nx / 2 {
            w += l;
        } else {
            e += l;
        }
    }
    w / e
}

#[test]
fn a_split_pot_roots_more_in_its_wet_half() {
    let (on, soil_on) = run(&Setup::Split, true, 40);
    let (off, soil_off) = run(&Setup::Split, false, 40);
    let (r_on, r_off) = (west_east(&on, &soil_on), west_east(&off, &soil_off));
    println!("west/east root length: rules {r_on:.2}, none {r_off:.2}");
    // The east half dried.
    let (nx, _, _) = soil_on.dims();
    let east = soil_on.index(Cell {
        x: nx - 3,
        y: 6,
        z: 2,
    });
    assert!(soil_on.cell_ftsw(east) < 0.8, "{}", soil_on.cell_ftsw(east));
    assert!(r_on > 1.15 * r_off, "{r_on} vs {r_off}");
}

#[test]
fn laterals_proliferate_in_a_nitrate_patch() {
    let in_patch = |p: &Plant, soil: &SoilGrid| {
        let cells = patch(soil);
        lengths(p, soil, 1)
            .into_iter()
            .filter(|(i, _)| cells.binary_search(i).is_ok())
            .map(|(_, l)| l)
            .sum::<f64>()
    };
    let (rich, soil_rich) = run(&Setup::Patch(10.0), true, 40);
    let (plain, soil_plain) = run(&Setup::Patch(1.0), true, 40);
    let (a, b) = (in_patch(&rich, &soil_rich), in_patch(&plain, &soil_plain));
    println!("lateral length in the patch: {a:.2} m against {b:.2} m");
    assert!(a > 1.2 * b, "{a} vs {b}");
}
