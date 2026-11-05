//! WP8 done-when: a 90-day run produces node, branch, pod and root counts inside the
//! §3.16 ranges. One bush bean in a 24 cm loam pot, watered to field capacity daily,
//! under the temperate-summer preset at 45° N from 15 May. The driver stands in for the
//! world loop of WP9 and runs the §3.2 steps in order.

use ps_arch::{DayReport, HourCtx, Plant};
use ps_soil::{SoilGrid, SoilHour, Texture};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, Meters, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, Preset, WeatherSeries};

const DAYS: u32 = 90;

fn run(days: u32, seed: u64) -> (Plant, Vec<DayReport>) {
    run_watered(days, seed, Texture::Loam.hydraulics().field_capacity)
}

/// Runs one plant for `days`, refilling the pot to volumetric water `theta` each night.
fn run_watered(days: u32, seed: u64, theta: f64) -> (Plant, Vec<DayReport>) {
    let species = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    let traits = species
        .phenotype(&species.default_genotype())
        .traits()
        .unwrap();
    let start = DateHour::new(2026, 5, 15, 0).unwrap();
    let wx: WeatherSeries = ps_weather::preset_series(
        Preset::TemperateSummer,
        Degrees(45.0),
        start,
        days + 1,
        seed,
    );
    let cell = 0.02;
    let mut soil = SoilGrid::new_pot(
        Meters(0.24),
        Meters(0.24),
        Meters(cell),
        Texture::Loam,
        Celsius(15.0),
    )
    .unwrap();
    soil.fill_to(theta);
    let (nx, ny, _) = soil.dims();
    #[allow(clippy::cast_precision_loss)]
    let center = [nx as f64 * cell * 0.5, ny as f64 * cell * 0.5, -0.03];
    let mut plant = Plant::sow(&species, &traits, seed, 1, center, Hours(0)).unwrap();
    let mut reports = Vec::new();
    for tick in 0..u64::from(days) * 24 {
        let h = wx.at(Hours(tick)).unwrap();
        let ctx = HourCtx {
            weather: h,
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
        });
        let uptake = soil.take_up(&[plant.uptake_request(demand)]).unwrap();
        plant.grow_hour(&ctx, &soil, uptake.plants.first()).unwrap();
        if tick % 24 == 23 {
            let mut sun = [None; 24];
            for (k, s) in sun.iter_mut().enumerate() {
                let next = wx.at(Hours(tick + 1 + k as u64)).unwrap_or(h);
                *s = next.sun.is_up().then(|| next.sun.direction());
            }
            let rep = plant.day(&soil, &sun, Hours(tick + 1)).unwrap();
            reports.push(rep);
            soil.fill_to(theta);
        }
    }
    (plant, reports)
}

#[test]
fn ninety_day_counts_are_in_the_bean_ranges() {
    let (plant, reports) = run(DAYS, 7);
    for r in reports.iter().step_by(5) {
        println!(
            "day {:>3} {:?} nodes {} branches {} leaves {} area {:.0} cm2 racemes {} flowers {} pods {} seeds {} roots {}+{} {:.1} m {:.2} g dm {:.2} g fruit {:.2} g h {:.0} cm",
            r.day,
            r.stage,
            r.main_nodes,
            r.branches,
            r.leaves,
            r.leaf_area * 1e4,
            r.racemes,
            r.flowers,
            r.pods,
            r.seeds,
            r.root_axes,
            r.lateral_cohorts,
            r.root_length,
            r.root_mass,
            r.dry_mass,
            r.fruit_mass,
            r.height * 100.0
        );
    }
    let last = reports.last().unwrap();
    let max_pods = reports.iter().map(|r| r.pods).max().unwrap();
    println!("final {last:?}; peak pods {max_pods}");
    assert!(
        (6..=8).contains(&last.main_nodes),
        "main nodes {}",
        last.main_nodes
    );
    assert!(
        (3..=6).contains(&last.branches),
        "branches {}",
        last.branches
    );
    assert!((10..=20).contains(&last.pods), "pods {}", last.pods);
    // §3.16 gives no root count; the seed-borne axes are the species' type counts.
    assert_eq!(last.root_axes, 19);
    assert!(
        last.lateral_cohorts > 50,
        "lateral cohorts {}",
        last.lateral_cohorts
    );
    assert!(last.root_length > 10.0, "root length {}", last.root_length);
    let _ = plant;
}

#[test]
fn counts_hold_across_world_seeds() {
    let mut bad = Vec::new();
    for seed in 1..=8 {
        let (plant, reports) = run(DAYS, seed);
        let last = reports.last().unwrap();
        let reserve: f64 = plant.arena.reserve.iter().map(|r| r.0).sum();
        println!(
            "seed {seed}: nodes {} branches {} pods {} seeds {} flowers {} dm {:.1} fruit {:.1} root {:.2} g {:.0} m h {:.0} cm reserve {reserve:.2} g C, mature day {:?}",
            last.main_nodes,
            last.branches,
            last.pods,
            last.seeds,
            last.flowers,
            last.dry_mass,
            last.fruit_mass,
            last.root_mass,
            last.root_length,
            last.height * 100.0,
            reports
                .iter()
                .find(|r| r.stage == Some(ps_physio::Stage::Mature))
                .map(|r| r.day),
        );
        bad.extend(
            (!(6..=8).contains(&last.main_nodes))
                .then(|| format!("seed {seed}: main nodes {}", last.main_nodes)),
        );
        bad.extend(
            (!(3..=6).contains(&last.branches))
                .then(|| format!("seed {seed}: branches {}", last.branches)),
        );
        bad.extend(
            (!(10..=20).contains(&last.pods)).then(|| format!("seed {seed}: pods {}", last.pods)),
        );
    }
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn dry_soil_holds_the_seed() {
    let wilting = Texture::Loam.hydraulics().wilting_point;
    let (plant, reports) = run_watered(10, 7, wilting);
    assert!(
        reports
            .iter()
            .all(|r| r.stage == Some(ps_physio::Stage::Sown))
    );
    assert_eq!(plant.arena.len(), 4, "no organs beyond the seed");
}

#[test]
fn same_seed_same_plant() {
    let (a, ra) = run(40, 11);
    let (b, rb) = run(40, 11);
    assert_eq!(ra, rb);
    let bits = |p: &Plant| {
        let mut v = Vec::new();
        p.arena.visit_bits(|x| v.push(x));
        v
    };
    assert_eq!(bits(&a), bits(&b));
}

#[test]
fn roots_stay_in_the_pot() {
    let (plant, _) = run(50, 3);
    let a = &plant.arena;
    let mut outside = 0.0;
    let mut total = 0.0;
    for id in a.alive_of(ps_graph::OrganKind::Root) {
        let len = a.size[id.index()].length.0;
        total += len;
        if plant.positions[id.index()].1[2] > 1e-9 {
            outside += len;
        }
    }
    assert!(total > 1.0, "root length {total}");
    assert!(outside == 0.0, "{outside} m of roots above the soil");
    // Root length the soil sees is at least 95% of the total (lateral cohorts near the
    // wall may poke past it).
    let seen = plant.roots.total().0;
    assert!(seen > 0.95 * total, "{seen} of {total} m in soil cells");
}
