//! The real data source (WP16, viewer side): a ps-world scenario running on the
//! ps-session sim thread, drawn from its snapshots.
//!
//! Each frame the app forwards the UI's time commands to the session and reads the latest
//! snapshot (an `Arc` clone; the sim never waits on the renderer). When the snapshot's
//! `seq` changes, each plant's skeleton is rebuilt from its organ graph with ps-arch's
//! attachment fractions and root multiplicities, so the shapes match
//! `ps_arch::geometry::layout`. Between ticks the view blends the previous hour's
//! skeleton into the current one, so growth is smooth (one hour behind the sim).
//!
//! Every lens reads sim values: absorbed PAR from each leaf's light coefficients under
//! the hour's weather and the pot's shade cloth, the red to far-red ratio, reserve
//! concentration, thermal age, root age, the plant's water stress factor, and FTSW on the
//! vertical soil slice through the plant.
//!
//! The charts panel reads the snapshot's daily rows (the same rows the CSV export
//! writes), rebuilt only when a day is added.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use bevy::math::Vec3;
use bevy::prelude::{ResMut, Resource};
use ps_graph::OrganKind;
use ps_mesh::Skeleton;
use ps_session::{Session, SessionOptions, Snapshot, TimeCommand as SimCommand};
use ps_ui::{Chart, Charts, Series, TimeCommand, TimeControl};
use ps_units::{Hours, WPerM2};
use ps_view::{Garden, Lens, PotShape, PotSize, PotView, SkyState, SoilFace};
use ps_weather::{HourWeather, WeatherSeries};
use ps_world::{Catalog, DayRow, PlantEntry, Scenario, SoilEntry, SoilSpec, World};

/// The scenario the app runs when none is given.
pub const DEFAULT_SCENARIO: &str = "scenarios/two_pot_sun_shade.ron";

/// Finds the `data/` directory: `explicit`, else the first `data/scenarios` found walking
/// up from the working directory and from the executable's directory.
///
/// # Errors
/// When no data directory is found.
pub fn find_data_dir(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p.to_path_buf());
    }
    let mut starts = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    {
        starts.push(dir);
    }
    for start in starts {
        for dir in start.ancestors() {
            let data = dir.join("data");
            if data.join("scenarios").is_dir() {
                return Ok(data);
            }
        }
    }
    bail!("no data/ directory found; run from the repository or pass --data DIR")
}

/// Loads a scenario file and builds its world.
///
/// # Errors
/// I/O, parse or world construction errors.
pub fn load_world(data_dir: &Path, scenario: &Path) -> Result<(World, Scenario)> {
    let path = if scenario.is_absolute() || scenario.exists() {
        scenario.to_path_buf()
    } else {
        data_dir.join(scenario)
    };
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading scenario {}", path.display()))?;
    let scenario = Scenario::from_ron(&text).with_context(|| format!("{}", path.display()))?;
    let catalog = Catalog::load(data_dir, &scenario).context("loading species and weather")?;
    let world = World::new(scenario.clone(), catalog).context("building the world")?;
    Ok((world, scenario))
}

/// What the viewer knows about one soil container.
struct PotInfo {
    label: String,
    /// Center in the garden, sim frame, m (pots re-centered on the garden's middle).
    center: [f32; 2],
    size: PotSize,
    /// Share of open-sky radiation reaching the pot.
    radiation_fraction: f64,
}

/// One plant's skeletons: the hour before the current tick and the current tick.
struct PlantFrames {
    tick: u64,
    previous: Skeleton,
    current: Skeleton,
}

/// The sim-driven source.
#[derive(Resource)]
pub struct SimSource {
    session: Session,
    weather: WeatherSeries,
    pots: BTreeMap<u32, PotInfo>,
    plants: BTreeMap<u32, PlantFrames>,
    seq: u64,
    snapshot: std::sync::Arc<Snapshot>,
    /// Resume at this speed once the startup step (`--day`) has finished.
    resume_speed: Option<f64>,
    /// The tick the startup step goes to; the source reports busy until it is reached.
    startup_tick: u64,
    /// The sim hour at which day 0 starts (minus the scenario's start hour).
    day_start_hour: f64,
}

impl SimSource {
    /// Starts the sim thread on `world`, advances to `start_day` as fast as possible, then
    /// runs at `speed` unless `paused`.
    ///
    /// # Errors
    /// If the sim thread cannot start.
    pub fn start(
        world: World,
        scenario: &Scenario,
        start_day: f64,
        speed: f64,
        paused: bool,
    ) -> Result<Self> {
        let weather = world.weather().clone();
        let fractions: BTreeMap<u32, f64> = world
            .state()
            .soils
            .iter()
            .map(|e| (e.id, e.radiation_fraction))
            .collect();
        let pots = pot_infos(&world.state().soils, scenario, &fractions);
        // `--day` counts from midnight of the first day, so 45.5 is noon on day 45
        // whatever hour the scenario starts.
        let hours = (start_day * 24.0 - f64::from(scenario.start.hour))
            .round()
            .min(world.end_tick() as f64);
        let session = Session::spawn(
            world,
            SessionOptions {
                speed,
                paused: true,
                ..SessionOptions::default()
            },
        )?;
        let mut resume_speed = None;
        if hours >= 1.0 {
            session.time(SimCommand::Step(hours));
            if !paused {
                resume_speed = Some(speed);
            }
        } else if !paused {
            session.time(SimCommand::SetSpeed(speed));
        }
        let snapshot = session.latest();
        let startup_tick = snapshot.tick + if hours >= 1.0 { hours as u64 } else { 0 };
        Ok(Self {
            session,
            weather,
            pots,
            plants: BTreeMap::new(),
            seq: 0,
            snapshot,
            resume_speed,
            startup_tick,
            day_start_hour: -f64::from(scenario.start.hour),
        })
    }
}

fn pot_infos(
    soils: &[SoilEntry],
    scenario: &Scenario,
    fractions: &BTreeMap<u32, f64>,
) -> BTreeMap<u32, PotInfo> {
    let positions: BTreeMap<u32, (f64, f64)> = scenario
        .soils
        .iter()
        .map(|s| match s {
            SoilSpec::Pot { id, position_m, .. } => (*id, *position_m),
        })
        .collect();
    let n = positions.len().max(1) as f64;
    let mean_x = positions.values().map(|p| p.0).sum::<f64>() / n;
    let mean_y = positions.values().map(|p| p.1).sum::<f64>() / n;
    soils
        .iter()
        .map(|e| {
            let (nx, ny, nz) = e.grid.dims();
            let cell = e.grid.cell_size().0 as f32;
            let p = positions.get(&e.id).copied().unwrap_or((0.0, 0.0));
            let f = fractions.get(&e.id).copied().unwrap_or(1.0);
            let light = if f >= 0.999 {
                "full sun".to_owned()
            } else {
                format!("{:.0}% shade", (1.0 - f) * 100.0)
            };
            (
                e.id,
                PotInfo {
                    label: format!("Pot {}: {light}", e.id),
                    center: [(p.0 - mean_x) as f32, (p.1 - mean_y) as f32],
                    size: PotSize {
                        // Every container is a cylindrical pot for now (`SoilSpec::Pot`).
                        shape: PotShape::Round,
                        width: nx as f32 * cell,
                        depth: ny as f32 * cell,
                        soil_depth: nz as f32 * cell,
                    },
                    radiation_fraction: f,
                },
            )
        })
        .collect()
}

fn to_sim(c: TimeCommand) -> SimCommand {
    match c {
        TimeCommand::TogglePause => SimCommand::TogglePause,
        TimeCommand::SetSpeed(s) => SimCommand::SetSpeed(s),
        TimeCommand::Step(h) => SimCommand::Step(h),
        TimeCommand::Seek(h) => SimCommand::Seek(h),
    }
}

/// A plant's skeleton relative to its pot's surface center.
fn skeleton_of(p: &PlantEntry, size: PotSize) -> Skeleton {
    let plant = &p.plant;
    let attach: Vec<f64> = plant.meta.iter().map(|m| m.attach).collect();
    let mult: Vec<f64> = plant.meta.iter().map(|m| m.multiplicity).collect();
    let c = plant.collar_position;
    let origin = [
        c[0] - f64::from(size.width) * 0.5,
        c[1] - f64::from(size.depth) * 0.5,
        c[2],
    ];
    Skeleton::from_arena_with(&plant.arena, origin, &attach, &mult)
}

/// FTSW on the vertical west-east slice of the grid through the plant's collar.
fn soil_slice(soil: &SoilEntry, collar_y: f64) -> SoilFace {
    let g = &soil.grid;
    let (nx, ny, nz) = g.dims();
    let cell = g.cell_size().0;
    let y = if cell > 0.0 {
        ((collar_y / cell).floor().max(0.0) as usize).min(ny.saturating_sub(1))
    } else {
        0
    };
    let mut values = Vec::with_capacity(nx * nz);
    for z in 0..nz {
        for x in 0..nx {
            let i = (z * ny + y) * nx + x;
            values.push(g.cell_ftsw(i) as f32);
        }
    }
    SoilFace {
        columns: nx,
        rows: nz,
        values,
    }
}

fn absorbed(p: &PlantEntry, i: usize, hour: usize, w: &HourWeather, fraction: f64) -> f64 {
    let sin_e = if w.sun.is_up() {
        w.sun.direction()[2]
    } else {
        0.0
    };
    ps_light::absorbed_par(
        &p.plant.arena.light[i],
        hour,
        WPerM2(w.direct.0 * fraction),
        WPerM2(w.diffuse.0 * fraction),
        sin_e,
        p.plant.params.g_function,
        p.plant.params.absorptance,
    )
    .mean()
    .0
}

/// A chart line's quantity: label, color, and the value from a daily row.
type Quantity = (&'static str, [f32; 3], fn(&DayRow) -> f64);

/// The charts of §5.4 that the daily rows carry, with each plant's lines in its pot's
/// line style (`pot_of` maps a plant id to its pot index).
#[must_use]
pub fn charts_from(history: &[DayRow], pot_of: &BTreeMap<u32, usize>) -> Vec<Chart> {
    let leaf = [0.3, 0.6, 0.15];
    let charts: [(&str, &str, Vec<Quantity>); 6] = [
        (
            "Dry mass by organ",
            "g",
            vec![
                ("Leaf", leaf, |r| r.report.leaf_mass),
                ("Stem", [0.55, 0.6, 0.2], |r| r.report.stem_mass),
                ("Root", [0.75, 0.6, 0.4], |r| r.report.root_mass),
                ("Pods", [0.85, 0.45, 0.2], |r| r.report.fruit_mass),
            ],
        ),
        (
            "Leaf area",
            "cm²",
            vec![("Leaf area", leaf, |r| r.report.leaf_area * 1e4)],
        ),
        (
            "Height",
            "cm",
            vec![("Height", [0.6, 0.75, 0.45], |r| r.report.height * 100.0)],
        ),
        (
            "Root-zone soil water",
            "FTSW",
            vec![("FTSW", [0.3, 0.55, 0.85], |r| r.env.ftsw)],
        ),
        (
            "Water shortfall",
            "g/day",
            vec![("Shortfall", [0.85, 0.5, 0.3], |r| {
                r.report.shortfall * 1000.0
            })],
        ),
        (
            "Pods",
            "count",
            vec![("Pods", [0.85, 0.45, 0.2], |r| f64::from(r.report.pods))],
        ),
    ];
    charts
        .into_iter()
        .map(|(title, unit, quantities)| {
            let mut series = Vec::new();
            for (&plant, &pot) in pot_of {
                for (label, color, value) in &quantities {
                    series.push(Series {
                        label: (*label).to_owned(),
                        color: *color,
                        pot,
                        points: history
                            .iter()
                            .filter(|r| r.plant == plant)
                            .map(|r| [f64::from(r.day), value(r)])
                            .collect(),
                    });
                }
            }
            Chart {
                title: title.to_owned(),
                unit: unit.to_owned(),
                series,
            }
        })
        .collect()
}

fn season(month: u32) -> &'static str {
    match month {
        3..=5 => "spring",
        6..=8 => "summer",
        9..=11 => "autumn",
        _ => "winter",
    }
}

/// Lens values for one plant, by organ.
fn organ_values(
    p: &PlantEntry,
    tick: u64,
    day_weather: &[Option<HourWeather>; 24],
    weather: Option<&HourWeather>,
    fraction: f64,
    high_speed: bool,
) -> BTreeMap<Lens, Vec<f32>> {
    let a = &p.plant.arena;
    let n = a.len();
    let hour_of_day = (tick % 24) as usize;
    let mut light = vec![f32::NAN; n];
    let mut rfr = vec![f32::NAN; n];
    let mut carbon = vec![f32::NAN; n];
    let mut age = vec![f32::NAN; n];
    let mut roots = vec![f32::NAN; n];
    for id in a.ids() {
        let i = id.index();
        let kind = a.kind[i];
        age[i] = a.thermal_age[i].0 as f32;
        let w = a.structural(id).0;
        if a.alive[i] && w > 0.0 {
            carbon[i] = (a.reserve[i].0 / w) as f32;
        }
        if matches!(kind, OrganKind::Leaf | OrganKind::Cotyledon) && a.alive[i] {
            rfr[i] = a.light[i].rfr as f32;
            let par = if high_speed {
                day_weather
                    .iter()
                    .enumerate()
                    .filter_map(|(k, hw)| hw.map(|hw| absorbed(p, i, k, &hw, fraction)))
                    .sum::<f64>()
                    / 24.0
            } else {
                weather.map_or(0.0, |w| absorbed(p, i, hour_of_day, w, fraction))
            };
            light[i] = par as f32;
        }
        if kind == OrganKind::Root {
            roots[i] = tick.saturating_sub(a.birth[i].0) as f32 / 24.0;
        }
    }
    BTreeMap::from([
        (Lens::Light, light),
        (Lens::ShadeSignal, rfr),
        (Lens::Carbon, carbon),
        (Lens::Phenology, age),
        (Lens::RootsXray, roots),
    ])
}

/// Forwards time commands, then fills the garden, the sky and the top bar from the
/// latest snapshot.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn drive(
    mut src: ResMut<SimSource>,
    mut time_ui: ResMut<TimeControl>,
    mut garden: ResMut<Garden>,
    mut sky: ResMut<SkyState>,
    mut charts: ResMut<Charts>,
) {
    for c in std::mem::take(&mut time_ui.commands) {
        // A user command overrides the pending resume after the startup step.
        src.resume_speed = None;
        src.session.time(to_sim(c));
    }
    let snap = src.session.latest();
    if snap.seq != src.seq {
        src.seq = snap.seq;
        if !snap.busy
            && snap.latest_tick >= src.startup_tick
            && let Some(speed) = src.resume_speed.take()
        {
            src.session.time(SimCommand::SetSpeed(speed));
        }
        let tick = snap.tick;
        for p in &snap.plants {
            let Some(size) = snap
                .soils
                .get(p.soil)
                .and_then(|s| src.pots.get(&s.id))
                .map(|i| i.size)
            else {
                continue;
            };
            let new = skeleton_of(p, size);
            match src.plants.get_mut(&p.id) {
                Some(f) if f.tick == tick => f.current = new,
                Some(f) if f.tick + 1 == tick => {
                    f.previous = std::mem::replace(&mut f.current, new);
                    f.tick = tick;
                }
                _ => {
                    src.plants.insert(
                        p.id,
                        PlantFrames {
                            tick,
                            previous: new.clone(),
                            current: new,
                        },
                    );
                }
            }
        }
        src.snapshot = snap;
    }
    let snap = std::sync::Arc::clone(&src.snapshot);
    let tick = snap.tick;
    let frac = (snap.hour - tick as f64).clamp(0.0, 1.0) as f32;
    let high_speed = !snap.paused && snap.speed > ps_ui::HIGH_SPEED;

    let day0 = tick - tick % 24;
    let day_weather: [Option<HourWeather>; 24] =
        core::array::from_fn(|k| src.weather.at(Hours(day0 + k as u64)).copied());
    let weather = snap
        .weather
        .or_else(|| src.weather.at(Hours(tick)).copied());

    // Sky: the hour's sun, or the day's noon sun at mean light in high-speed mode (§2.2).
    if let Some(w) = weather {
        let (sun, global) = if high_speed {
            let noon = day_weather[12].unwrap_or(w);
            let mean = day_weather
                .iter()
                .flatten()
                .map(|w| w.global.0)
                .sum::<f64>()
                / 24.0;
            (noon.sun.direction(), (mean * 2.0) as f32)
        } else {
            (w.sun.direction(), w.global.0 as f32)
        };
        sky.sun_direction = Vec3::new(sun[0] as f32, sun[1] as f32, sun[2] as f32);
        sky.global = global;
        sky.cloud_fraction = w.cloud_fraction as f32;
    }

    let mut pots = Vec::with_capacity(snap.plants.len());
    for p in &snap.plants {
        let Some(soil) = snap.soils.get(p.soil) else {
            continue;
        };
        let (Some(info), Some(frames)) = (src.pots.get(&soil.id), src.plants.get(&p.id)) else {
            continue;
        };
        let skeleton = Skeleton::lerp(&frames.previous, &frames.current, frac);
        pots.push(PotView {
            label: info.label.clone(),
            center: info.center,
            size: info.size,
            skeleton,
            water_stress: p.plant.stress.0 as f32,
            organ_values: organ_values(
                p,
                tick,
                &day_weather,
                weather.as_ref(),
                info.radiation_fraction,
                high_speed,
            ),
            soil_water: soil_slice(soil, p.plant.collar_position[1]),
        });
    }
    garden.pots = pots;
    garden.stand_in_lenses.clear();

    if charts.rows != snap.history.len() || charts.pots.len() != garden.pots.len() {
        let pot_of: BTreeMap<u32, usize> = snap
            .plants
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, i))
            .collect();
        charts.charts = charts_from(&snap.history, &pot_of);
        charts.pots = garden.pots.iter().map(|p| p.label.clone()).collect();
        charts.day_start_hour = src.day_start_hour;
        charts.rows = snap.history.len();
    }

    let date = snap.date;
    time_ui.hour = snap.hour;
    time_ui.latest_hour = snap.latest_tick as f64;
    time_ui.end_hour = snap.end_tick as f64;
    time_ui.speed = snap.speed;
    time_ui.paused = snap.paused;
    time_ui.busy = snap.busy || snap.latest_tick < src.startup_tick;
    time_ui.date = format!(
        "{}-{:02}-{:02} {:02}:00",
        date.year, date.month, date.day, date.hour
    );
    season(date.month).clone_into(&mut time_ui.season);
    let mut line = weather.map_or_else(String::new, |w| {
        format!(
            "{:.1} °C · {:.0} W/m² · {:.0}% cloud{}",
            w.t_air.0,
            w.global.0,
            w.cloud_fraction * 100.0,
            if w.rain.0 > 0.0 { " · rain" } else { "" }
        )
    });
    if let Some(e) = &snap.error {
        line = format!("Sim stopped: {e}");
    } else if let Some(r) = &snap.rejected {
        line = format!("{line} · {r}");
    }
    time_ui.weather = line;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> (World, Scenario) {
        let data = find_data_dir(None).unwrap_or_else(|e| unreachable!("{e}"));
        load_world(&data, Path::new(DEFAULT_SCENARIO)).unwrap_or_else(|e| unreachable!("{e:#}"))
    }

    #[test]
    fn skeletons_match_ps_arch_layout() {
        let (mut w, _) = world();
        w.run_days(20).unwrap_or_else(|e| unreachable!("{e}"));
        for p in &w.state().plants {
            let size = PotSize {
                shape: PotShape::Round,
                width: 0.2,
                depth: 0.2,
                soil_depth: 0.2,
            };
            let s = skeleton_of(p, size);
            let layout =
                ps_arch::geometry::layout(&p.plant.arena, &p.plant.meta, p.plant.collar_position);
            assert_eq!(s.len(), layout.len());
            let shift = Vec3::new(-0.1, -0.1, 0.0);
            for (o, (b, t)) in s.organs.iter().zip(&layout) {
                let b = Vec3::new(b[0] as f32, b[1] as f32, b[2] as f32) + shift;
                let t = Vec3::new(t[0] as f32, t[1] as f32, t[2] as f32) + shift;
                assert!((o.base - b).length() < 1e-4, "base {:?} vs {b:?}", o.base);
                assert!((o.tip() - t).length() < 1e-4, "tip {:?} vs {t:?}", o.tip());
            }
        }
    }

    #[test]
    fn charts_have_one_point_per_day_and_plant() {
        let (mut w, _) = world();
        w.run_days(6).unwrap_or_else(|e| unreachable!("{e}"));
        let pot_of: BTreeMap<u32, usize> = w
            .state()
            .plants
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, i))
            .collect();
        let charts = charts_from(&w.state().history, &pot_of);
        assert_eq!(charts.len(), 6);
        let days = w.state().history.len() / pot_of.len();
        assert!(days >= 5);
        for c in &charts {
            assert!(!c.series.is_empty() && !c.unit.is_empty());
            for s in &c.series {
                assert_eq!(s.points.len(), days, "{} {}", c.title, s.label);
                assert!(s.points.windows(2).all(|p| p[0][0] < p[1][0]));
                assert!(s.points.iter().all(|p| p[1].is_finite()));
            }
        }
        // Pots get their own line styles.
        let pots: std::collections::BTreeSet<usize> =
            charts[0].series.iter().map(|s| s.pot).collect();
        assert_eq!(pots.len(), pot_of.len());
    }

    #[test]
    fn pots_are_centered_and_sized_from_the_grid() {
        let (w, scenario) = world();
        let f = BTreeMap::new();
        let pots = pot_infos(&w.state().soils, &scenario, &f);
        assert_eq!(pots.len(), 2);
        let xs: Vec<f32> = pots.values().map(|p| p.center[0]).collect();
        assert!((xs[0] + xs[1]).abs() < 1e-6);
        assert!(
            pots.values()
                .all(|p| p.size.width > 0.15 && p.size.soil_depth > 0.15)
        );
    }
}
