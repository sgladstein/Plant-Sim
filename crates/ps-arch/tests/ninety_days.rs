//! WP8 done-when: a 90-day run produces node, branch, pod and root counts inside the
//! §3.16 ranges. One bush bean in a 24 cm loam pot, watered to field capacity daily,
//! under the temperate-summer preset at 45° N from 15 May. The driver stands in for the
//! world loop of WP9 and runs the §3.2 steps in order. The count tests grow the plant at
//! the world pot's non-limiting 300 ppm nitrate (decisions 0020 and 0047): WP8 came
//! before the nitrogen model.

use ps_arch::{DayReport, HourCtx, Plant};
use ps_soil::{SoilGrid, SoilHour, Texture};
use ps_species::Species;
use ps_units::{Celsius, Degrees, Hours, KPa, Meters, Millimeters, Ppm, WPerM2};
use ps_weather::{DateHour, Preset, WeatherSeries};

const DAYS: u32 = 90;

/// The world pot's non-limiting nitrogen control: 300 ppm nitrate (decision 0020).
const FED: ps_soil::SoilNitrogen = ps_soil::SoilNitrogen {
    no3_ppm: 300.0,
    ..ps_soil::SoilNitrogen::GARDEN
};

/// Water to bring the pot back to volumetric water `theta`, mm over its surface.
fn deficit(soil: &SoilGrid, theta: f64) -> Millimeters {
    let kg: f64 = (0..soil.len())
        .filter(|&i| soil.is_active(i))
        .map(|i| (theta - soil.theta(i)).max(0.0) * soil.cell_volume() * 1000.0)
        .sum();
    Millimeters(kg / soil.surface_area().max(1e-12))
}

fn run(days: u32, seed: u64) -> (Plant, Vec<DayReport>) {
    run_watered(days, seed, Texture::Loam.hydraulics().field_capacity)
}

/// As [`run`], at 300 ppm nitrate ([`FED`]).
fn run_fed(days: u32, seed: u64) -> (Plant, Vec<DayReport>) {
    run_soil(days, seed, Texture::Loam.hydraulics().field_capacity, &FED)
}

/// Runs one plant for `days`, watering the pot back to volumetric water `theta` each
/// night through its surface.
fn run_watered(days: u32, seed: u64, theta: f64) -> (Plant, Vec<DayReport>) {
    run_soil(days, seed, theta, &ps_soil::SoilNitrogen::GARDEN)
}

/// As [`run_watered`], with the pot's soil nitrogen `n`.
fn run_soil(
    days: u32,
    seed: u64,
    theta: f64,
    n: &ps_soil::SoilNitrogen,
) -> (Plant, Vec<DayReport>) {
    let r = run_env(days, seed, theta, n, |_| {}, &[]);
    (r.0, r.1)
}

/// As [`run_soil`], with the species edited by `edit` and these light obstacles (in
/// the soil frame); also returns the PAR the leaves absorbed over the run, mol.
fn run_env(
    days: u32,
    seed: u64,
    theta: f64,
    n: &ps_soil::SoilNitrogen,
    edit: impl Fn(&mut Species),
    obstacles: &[ps_light::Obstacle],
) -> (Plant, Vec<DayReport>, f64) {
    let mut species =
        Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    edit(&mut species);
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
    soil.set_nitrogen(n);
    let (nx, ny, _) = soil.dims();
    #[allow(clippy::cast_precision_loss)]
    let center = [nx as f64 * cell * 0.5, ny as f64 * cell * 0.5, -0.03];
    let mut plant = Plant::sow(&species, &traits, seed, 1, center, Hours(0)).unwrap();
    plant.obstacles = obstacles.to_vec();
    let mut reports = Vec::new();
    let mut absorbed = 0.0;
    for tick in 0..u64::from(days) * 24 {
        let h = wx.at(Hours(tick)).unwrap();
        let ctx = HourCtx {
            weather: h,
            hour: usize::try_from(tick % 24).unwrap(),
            tick: Hours(tick),
            co2: Ppm(420.0),
            patm: KPa(101.3),
        };
        plant.water_demand(&ctx, &soil).unwrap();
        let lai = plant.report().leaf_area / soil.surface_area().max(1e-9);
        // Watered through the surface, so the water soaking in carries back down the
        // nitrate the day's evaporation drew up (R5a).
        let water_in = if tick > 0 && tick % 24 == 0 {
            deficit(&soil, theta)
        } else {
            Millimeters(0.0)
        };
        soil.step_hour(&SoilHour {
            water_in,
            surface_radiation: WPerM2(h.global.0 * libm::exp(-0.5 * lai)),
            t_air: h.t_air,
            strip: None,
        });
        let flows = plant.water_flows(&soil);
        let uptake = soil
            .take_flows(&[ps_soil::PlantFlows {
                demand: flows.demand,
                flows: &flows.flows,
            }])
            .unwrap();
        plant.grow_hour(&ctx, &soil, uptake.plants.first()).unwrap();
        let (area, par) = plant.absorbed_par(&ctx);
        absorbed += area.0 * par.0 * 3600.0 * 1e-6;
        if tick % 24 == 23 {
            let mut sun = [None; 24];
            for (k, s) in sun.iter_mut().enumerate() {
                let next = wx.at(Hours(tick + 1 + k as u64)).unwrap_or(h);
                *s = next.sun.is_up().then(|| next.sun.direction());
            }
            plant.nitrogen_alone(&mut soil, Hours(tick + 1)).unwrap();
            let rep = plant.day(&soil, &sun, Hours(tick + 1)).unwrap();
            reports.push(rep);
        }
    }
    (plant, reports, absorbed)
}

#[test]
fn ninety_day_counts_are_in_the_bean_ranges() {
    let (plant, reports) = run_fed(DAYS, 7);
    for r in reports.iter().step_by(5) {
        println!(
            "day {:>3} {:?} nodes {} branches {} leaves {} area {:.0} cm2 racemes {} flowers {} pods {} seeds {} roots {}+{} {:.1} m {:.2} g dm {:.2} g fruit {:.2} g h {:.0} cm fb {:.2} N {:.0} mg up {:.1} fix {:.1} mine {:.1} unmet {:.1} st {:.2} nod {:.0} mg",
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
            r.height * 100.0,
            r.feedback_loss,
            r.nitrogen * 1e3,
            r.n_uptake * 1e3,
            r.n_fixed * 1e3,
            r.n_mined * 1e3,
            r.n_unmet * 1e3,
            r.n_status,
            r.nodule_mass * 1e3
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
    // §3.16 gives 10 to 20 pods a plant.
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
    let mut pods = Vec::new();
    for seed in 1..=8 {
        let (plant, reports) = run_fed(DAYS, seed);
        let last = reports.last().unwrap();
        let reserve: f64 = plant.arena.reserve.iter().map(|r| r.0).sum();
        println!(
            "seed {seed}: nodes {} branches {} pods {} seeds {} flowers {} dm {:.1} total {:.1} fruit {:.1} root {:.2} g {:.0} m h {:.0} cm reserve {reserve:.2} g C, peak feedback loss {:.2}, mature day {:?}",
            last.main_nodes,
            last.branches,
            last.pods,
            last.seeds,
            last.flowers,
            last.dry_mass,
            last.total_dry_mass,
            last.fruit_mass,
            last.root_mass,
            last.root_length,
            last.height * 100.0,
            reports.iter().map(|r| r.feedback_loss).fold(0.0, f64::max),
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
        // Each plant now grows from its own seed (decision 0027), so single plants
        // spread wider than the target band; one plant need only be plausible.
        bad.extend(
            (!(4..=30).contains(&last.pods)).then(|| format!("seed {seed}: pods {}", last.pods)),
        );
        pods.push(f64::from(last.pods));
        // Nothing grows from maturity (decision 0042): the seeds keep the mass they had.
        if let Some(m) = reports
            .iter()
            .find(|r| r.stage == Some(ps_physio::Stage::Mature))
        {
            bad.extend(((last.seed_mass - m.seed_mass).abs() > 1e-12).then(|| {
                format!(
                    "seed {seed}: seed mass {} at maturity, {} at the end",
                    m.seed_mass, last.seed_mass
                )
            }));
        }
    }
    // §3.16's 10 to 20 holds for the mean over seeds, as the validation report reads
    // its bands (decision 0026).
    #[allow(clippy::cast_precision_loss)]
    let mean = pods.iter().sum::<f64>() / pods.len() as f64;
    if !(10.0..=20.0).contains(&mean) {
        bad.push(format!("mean pods {mean:.1}"));
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

/// Writes the leaflets of a 60-day plant for the light ray tracer fixture
/// (`validation/fixtures/light_raytrace.py --leaves`, decision 0008). Run with
/// `PS_LEAVES_OUT=path cargo test -p ps-arch --test ninety_days -- --ignored`.
#[test]
#[ignore = "writes a fixture file; run by hand"]
fn export_leaves_for_ray_tracer() {
    use std::fmt::Write as _;
    let (plant, _) = run(60, 7);
    let mut out = String::from("id,x,y,z,area_m2,nx,ny,nz\n");
    for (i, p) in plant.light_points().iter().enumerate() {
        writeln!(
            out,
            "{i},{:.6},{:.6},{:.6},{:.8},{:.6},{:.6},{:.6}",
            p.center[0], p.center[1], p.center[2], p.area.0, p.normal[0], p.normal[1], p.normal[2]
        )
        .unwrap();
    }
    let path = std::env::var("PS_LEAVES_OUT").unwrap_or_else(|_| "plant_leaves.csv".into());
    std::fs::write(&path, out).unwrap();
}

/// WP24 done-when: bottom-up senescence is emergent (decision 0020). In the garden
/// pot, the main-stem leaves the plant mines and drops during seed fill sit lower on
/// the stem than the ones still green when it matures.
#[test]
fn seed_fill_drops_leaves_from_the_bottom_up() {
    use ps_graph::OrganKind;
    use ps_physio::Stage;
    let (plant, reports) = run(DAYS, 7);
    let hour_of = |stage: Stage| {
        let r = reports
            .iter()
            .find(|r| r.stage.is_some_and(|s| s >= stage))
            .unwrap();
        u64::from(r.day) * 24
    };
    let (fill, mature) = (hour_of(Stage::SeedFill), hour_of(Stage::Mature));
    let a = &plant.arena;
    // A leaf's node: the internodes below it.
    let node = |id: ps_graph::OrganId| {
        let mut n = 0_u32;
        let mut cur = a.parent[id.index()];
        while let Some(p) = cur {
            n += u32::from(a.kind[p.index()] == OrganKind::Internode);
            cur = a.parent[p.index()];
        }
        f64::from(n)
    };
    let (mut dropped, mut kept) = (Vec::new(), Vec::new());
    for id in a.ids() {
        let i = id.index();
        if a.kind[i] != OrganKind::Leaf || a.axis_order[i] != 0 || a.birth[i].0 > fill {
            continue;
        }
        match a.death[i] {
            Some(d) if d.0 <= fill => {}
            Some(d) if d.0 < mature => {
                assert!(plant.meta[i].mined, "leaf {i} dropped unmined");
                dropped.push(node(id));
            }
            _ => kept.push(node(id)),
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    println!("dropped at nodes {dropped:?}, kept {kept:?}");
    assert!(!dropped.is_empty() && !kept.is_empty());
    assert!(
        mean(&dropped) < mean(&kept),
        "dropped {dropped:?} kept {kept:?}"
    );
}

/// R1b (decision 0040): in a fertilized pot the seeds' nitrogen still comes first
/// from the plant's own organs, so mining runs through seed fill, though the soil holds
/// plenty (before R1b it ran only when the soil fell short).
#[test]
fn seed_fill_mines_first_in_a_fertilized_pot() {
    use ps_physio::Stage;
    let (_, reports) = run_fed(DAYS, 7);
    let before: f64 = reports
        .iter()
        .filter(|r| r.stage.is_some_and(|s| s < Stage::SeedFill))
        .map(|r| r.n_mined)
        .sum();
    let fill: Vec<&DayReport> = reports
        .iter()
        .filter(|r| r.stage == Some(Stage::SeedFill))
        .collect();
    let mined: f64 = fill.iter().map(|r| r.n_mined).sum();
    let uptake: f64 = fill.iter().map(|r| r.n_uptake).sum();
    println!("mined before fill {before:.4} g, in fill {mined:.4} g; uptake in fill {uptake:.4} g");
    assert!(fill.len() > 10);
    assert!(
        before < 0.01,
        "a fed plant mines little before seed fill: {before}"
    );
    assert!(
        mined > 10.0 * before && mined > 0.3 * (mined + uptake),
        "mined {mined} against uptake {uptake}"
    );
}

/// R1c done-when (decision 0041): a plant beside a low wall leans away from it, toward
/// the open sky, and its leaves absorb more light than with phototropism off; in the
/// open it stays nearly upright.
#[test]
fn a_plant_beside_a_wall_leans_away_and_gains_light() {
    let fc = Texture::Loam.hydraulics().field_capacity;
    let n = ps_soil::SoilNitrogen::GARDEN;
    // An opaque wall 40 cm tall, 6 cm east of the stem (the pot's center is at 0.12 m).
    let wall = ps_light::Obstacle::with_transmittance([0.18, -0.5, 0.0], [0.20, 0.74, 0.4], 0.01);
    let days = 45;
    let (on, _, lit_on) = run_env(days, 7, fc, &n, |_| {}, &[wall]);
    let (off, _, lit_off) = run_env(
        days,
        7,
        fc,
        &n,
        |s| s.constants.phototropism_weight.value = 0.0,
        &[wall],
    );
    let (open, _, _) = run_env(days, 7, fc, &n, |_| {}, &[]);
    let offset = |p: &Plant| {
        let tip = p.positions[p.axes[0].apex.index()].1;
        [
            tip[0] - p.collar_position[0],
            tip[1] - p.collar_position[1],
            tip[2],
        ]
    };
    let (a, b, c) = (offset(&on), offset(&off), offset(&open));
    println!(
        "main tip offset east, north, height: wall {a:.3?}, wall without phototropism {b:.3?}, open {c:.3?}; PAR absorbed {lit_on:.2} against {lit_off:.2} mol"
    );
    assert!(a[0] < -0.03, "leans west, away from the wall: {a:?}");
    assert!(b[0].abs() < 1e-9, "upright without phototropism: {b:?}");
    assert!(lit_on > 1.02 * lit_off, "{lit_on} against {lit_off}");
    assert!(
        c[0].hypot(c[1]) < 0.02,
        "a lone plant in the open stays nearly upright: {c:?}"
    );
}
