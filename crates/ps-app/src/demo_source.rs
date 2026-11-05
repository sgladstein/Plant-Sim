//! The data source until ps-world exists: two stand-in bean plants (sun and 50% shade)
//! under real weather from ps-weather, with leaf light from the real ps-light model.
//!
//! What is real: the weather series (temperate summer preset), the sun's path, and the
//! light lens and shade signal, which run `ps_light::rebuild_daily` and
//! `ps_light::absorbed_par` on the stand-in plants' leaves. What is stand-in: the plants
//! themselves (`ps_mesh::demo`), soil water, plant water stress and reserves. The legend
//! says so for the stand-in lenses.
//!
//! Time follows the UI's [`TimeCommand`]s; nothing here is the sim's clock or action log.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use bevy::prelude::*;
use ps_graph::{LightCoeffs, OrganArena, OrganKind};
use ps_light::LeafPoint;
use ps_mesh::demo::{DemoParams, demo_bean, demo_origin};
use ps_mesh::{InstanceOptions, Skeleton, Template, TemplateKey, build_template, instances};
use ps_ui::{TimeCommand, TimeControl};
use ps_units::{Degrees, M2, WPerM2};
use ps_view::{Garden, Lens, PotSize, PotView, SkyState, SoilFace};
use ps_weather::{DateHour, HourWeather, Preset, WeatherSeries, preset_series};

/// Scenario length, days.
pub const DAYS: u32 = 90;
/// Site latitude, degrees north.
pub const LATITUDE: f64 = 45.0;
/// Leaf PAR absorptance ("~").
const ABSORPTANCE: f64 = 0.85;
/// Days between waterings in the stand-in soil water.
const WATERING_DAYS: f64 = 5.0;
/// Soil face grid for the soil water lens.
const FACE_COLUMNS: usize = 16;
const FACE_ROWS: usize = 18;

struct PotSource {
    label: &'static str,
    params: DemoParams,
    /// Share of the open-sky radiation that reaches the plant (shade cloth).
    light: f64,
    center: [f32; 2],
    cache: BTreeMap<u64, (OrganArena, Skeleton)>,
    light_cache: Option<(u64, Vec<(usize, LightCoeffs)>)>,
}

impl PotSource {
    fn at(&mut self, hour: u64) -> &(OrganArena, Skeleton) {
        if self.cache.len() > 8 {
            // Keep only the hours near the present.
            let keep: Vec<u64> = self.cache.keys().rev().take(4).copied().collect();
            self.cache.retain(|k, _| keep.contains(k) || *k + 2 >= hour);
        }
        let params = self.params;
        self.cache.entry(hour).or_insert_with(|| {
            let arena = demo_bean(params, hour);
            let skeleton = Skeleton::from_arena(&arena, demo_origin(0.0, 0.0));
            (arena, skeleton)
        })
    }
}

/// The demo driver's state.
#[derive(Resource)]
pub struct DemoSource {
    weather: WeatherSeries,
    start: DateHour,
    pots: Vec<PotSource>,
    clock: f64,
    latest: f64,
    speed: f64,
    paused: bool,
}

impl DemoSource {
    /// A sun-versus-shade pair starting `start_day` days after sowing.
    #[must_use]
    pub fn new(start_day: f64, speed: f64, paused: bool) -> Self {
        let start = DateHour::new(2026, 5, 15, 0).unwrap_or(DateHour {
            year: 2026,
            month: 5,
            day: 15,
            hour: 0,
        });
        let weather = preset_series(Preset::TemperateSummer, Degrees(LATITUDE), start, DAYS, 7);
        let pot = |label, params, light, x| PotSource {
            label,
            params,
            light,
            center: [x, 0.0],
            cache: BTreeMap::new(),
            light_cache: None,
        };
        let end = f64::from(DAYS) * 24.0;
        let clock = (start_day * 24.0).clamp(0.0, end);
        Self {
            weather,
            start,
            pots: vec![
                pot("Left: full sun", DemoParams::SUN, 1.0, -0.21),
                pot("Right: 50% shade", DemoParams::SHADE, 0.5, 0.21),
            ],
            clock,
            latest: clock,
            speed,
            paused,
        }
    }

    fn end() -> f64 {
        f64::from(DAYS) * 24.0
    }

    fn apply(&mut self, c: TimeCommand) {
        let end = Self::end();
        match c {
            TimeCommand::TogglePause => self.paused = !self.paused,
            TimeCommand::SetSpeed(s) => {
                self.speed = s;
                self.paused = false;
            }
            TimeCommand::Step(h) => {
                self.clock = (self.clock.floor() + h).min(end);
                self.paused = true;
            }
            TimeCommand::Seek(h) => self.clock = h.clamp(0.0, self.latest),
        }
        self.latest = self.latest.max(self.clock);
    }
}

/// The blade center of each template, in template units (cached).
fn template_centroid(t: Template) -> Vec3 {
    static CENTROIDS: OnceLock<BTreeMap<Template, Vec3>> = OnceLock::new();
    let map = CENTROIDS.get_or_init(|| {
        Template::ALL
            .iter()
            .map(|&t| {
                let m = build_template(TemplateKey::new(t));
                let n = m.positions.len().max(1) as f32;
                let sum: Vec3 = m.positions.iter().map(|p| Vec3::from(*p)).sum();
                (t, sum / n)
            })
            .collect()
    });
    map.get(&t).copied().unwrap_or(Vec3::ZERO)
}

/// Leaves as the light model sees them, with their organ index.
fn leaf_points(skeleton: &Skeleton) -> (Vec<usize>, Vec<LeafPoint>) {
    let mut ids = Vec::new();
    let mut points = Vec::new();
    for inst in instances(skeleton, InstanceOptions::default()) {
        let Some(p) = skeleton.get(inst.organ) else {
            continue;
        };
        if !matches!(p.kind, OrganKind::Leaf | OrganKind::Cotyledon) || p.area <= 0.0 {
            continue;
        }
        let c =
            inst.translation + inst.rotation * (template_centroid(inst.key.template) * inst.scale);
        let n = (inst.rotation * Vec3::Z).normalize_or(Vec3::Z);
        ids.push(inst.organ.index());
        points.push(LeafPoint {
            center: [f64::from(c.x), f64::from(c.y), f64::from(c.z)],
            area: M2(f64::from(p.area)),
            normal: [f64::from(n.x), f64::from(n.y), f64::from(n.z)],
        });
    }
    (ids, points)
}

/// Mean absorbed PAR of a leaf in one hour, µmol m⁻² s⁻¹, with `light` the share of open
/// radiation reaching the pot.
fn absorbed(c: &LightCoeffs, hour: usize, w: &HourWeather, light: f64) -> f64 {
    let sin_e = w.sun.elevation.0.to_radians().sin();
    ps_light::absorbed_par(
        c,
        hour,
        WPerM2(w.direct.0 * light),
        WPerM2(w.diffuse.0 * light),
        sin_e,
        ABSORPTANCE,
    )
    .mean()
    .0
}

fn season(month: u32) -> &'static str {
    match month {
        3..=5 => "spring",
        6..=8 => "summer",
        9..=11 => "autumn",
        _ => "winter",
    }
}

/// Stand-in soil water on the glass face: roots dry the cells they pass through between
/// waterings; FTSW returns to 1 every few days.
fn soil_face(skeleton: &Skeleton, size: PotSize, hour: f64) -> SoilFace {
    let mut root_len = vec![0.0_f32; FACE_COLUMNS * FACE_ROWS];
    for p in skeleton
        .organs
        .iter()
        .filter(|p| p.alive && p.kind == OrganKind::Root)
    {
        let mid = 0.5 * (p.base + p.tip());
        let col = ((mid.x / size.width + 0.5) * FACE_COLUMNS as f32).floor();
        let row = (-mid.z / size.soil_depth * FACE_ROWS as f32).floor();
        if (0.0..FACE_COLUMNS as f32).contains(&col) && (0.0..FACE_ROWS as f32).contains(&row) {
            root_len[row as usize * FACE_COLUMNS + col as usize] += p.length;
        }
    }
    let since_water = (hour / 24.0) % WATERING_DAYS;
    // Spread each cell's roots a little to its neighbors, then dry by root length.
    let mut values = Vec::with_capacity(root_len.len());
    for r in 0..FACE_ROWS {
        for c in 0..FACE_COLUMNS {
            let mut rl = 0.0;
            for (dr, dc, w) in [
                (0, 0, 1.0),
                (-1, 0, 0.5),
                (1, 0, 0.5),
                (0, -1, 0.5),
                (0, 1, 0.5),
            ] {
                let (Some(rr), Some(cc)) = (r.checked_add_signed(dr), c.checked_add_signed(dc))
                else {
                    continue;
                };
                if rr < FACE_ROWS && cc < FACE_COLUMNS {
                    rl += w * root_len[rr * FACE_COLUMNS + cc];
                }
            }
            // Evaporation dries the top rows a little too.
            let surface = if r < 2 { 0.06 } else { 0.0 };
            let dry = (f64::from(rl) * 6.0 + surface) * since_water;
            values.push((1.0 - dry).clamp(0.0, 1.0) as f32);
        }
    }
    SoilFace {
        columns: FACE_COLUMNS,
        rows: FACE_ROWS,
        values,
    }
}

/// Advances time, then fills the garden, the sky and the top bar.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn drive(
    mut demo: ResMut<DemoSource>,
    mut time_ui: ResMut<TimeControl>,
    mut garden: ResMut<Garden>,
    mut sky: ResMut<SkyState>,
    lens: Res<Lens>,
    time: Res<Time>,
) {
    for c in std::mem::take(&mut time_ui.commands) {
        demo.apply(c);
    }
    let end = DemoSource::end();
    if !demo.paused {
        demo.clock = (demo.clock + f64::from(time.delta_secs()) * demo.speed).min(end);
        if demo.clock >= end {
            demo.paused = true;
        }
    }
    demo.latest = demo.latest.max(demo.clock);
    let high_speed = !demo.paused && demo.speed > ps_ui::HIGH_SPEED;

    let clock = demo.clock;
    let h0 = clock.floor().max(0.0) as u64;
    let frac = (clock - clock.floor()) as f32;
    let hour_of_day = (h0 % 24) as usize;
    let tick = ps_units::Hours(h0);
    let weather = demo.weather.at(tick).copied();
    let day0 = h0 - h0 % 24;
    let day_weather: [Option<HourWeather>; 24] =
        core::array::from_fn(|k| demo.weather.at(ps_units::Hours(day0 + k as u64)).copied());
    let sun_path: [Option<[f64; 3]>; 24] = core::array::from_fn(|k| {
        day_weather[k]
            .filter(|w| w.sun.is_up())
            .map(|w| w.sun.direction())
    });
    let day_mean_global = day_weather
        .iter()
        .flatten()
        .map(|w| w.global.0)
        .sum::<f64>()
        / 24.0;

    // Sky: the hour's sun, or the day's noon sun at mean light in high-speed mode (§2.2).
    if let Some(w) = weather {
        let noon = demo
            .weather
            .at(ps_units::Hours(day0 + 12))
            .copied()
            .unwrap_or(w);
        let (sun, global) = if high_speed {
            (noon.sun.direction(), (day_mean_global * 2.0) as f32)
        } else {
            (w.sun.direction(), w.global.0 as f32)
        };
        sky.sun_direction = Vec3::new(sun[0] as f32, sun[1] as f32, sun[2] as f32);
        sky.global = global;
        sky.cloud_fraction = w.cloud_fraction as f32;
    }

    let needs_light = matches!(*lens, Lens::Light | Lens::ShadeSignal);
    let mut pots = Vec::with_capacity(demo.pots.len());
    for src in &mut demo.pots {
        let (a_arena, a) = src.at(h0).clone();
        let b = src.at(h0 + 1).1.clone();
        let skeleton = Skeleton::lerp(&a, &b, frac);
        let n = skeleton.len();
        let size = PotSize::RHIZOBOX;

        let mut light = vec![f32::NAN; n];
        let mut rfr = vec![f32::NAN; n];
        if needs_light {
            let day = h0 / 24;
            if src.light_cache.as_ref().is_none_or(|(d, _)| *d != day) {
                // Rebuild once a day from the plant at noon, as the sim does (§3.2 daily step 6).
                let (_, noon) = src.at(day * 24 + 12).clone();
                let (ids, points) = leaf_points(&noon);
                let (_, coeffs) =
                    ps_light::rebuild_daily(&points, &[], &sun_path, &ps_light::sky_sectors());
                src.light_cache = Some((day, ids.into_iter().zip(coeffs).collect()));
            }
            if let (Some((_, coeffs)), Some(w)) = (&src.light_cache, weather) {
                for (i, c) in coeffs {
                    if *i >= n {
                        continue;
                    }
                    let par = if high_speed {
                        // Daily mean absorbed PAR (§2.2 high-speed mode).
                        day_weather
                            .iter()
                            .enumerate()
                            .filter_map(|(k, hw)| hw.map(|hw| absorbed(c, k, &hw, src.light)))
                            .sum::<f64>()
                            / 24.0
                    } else {
                        absorbed(c, hour_of_day, &w, src.light)
                    };
                    light[*i] = par as f32;
                    rfr[*i] = c.rfr as f32;
                }
            }
        }

        // Stand-in reserves: leaves in good light hold more sugar; everything else sits
        // near the 5% safety reserve and dips at night.
        let night = weather.is_none_or(|w| !w.sun.is_up());
        let carbon: Vec<f32> = skeleton
            .organs
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let base = if night { 0.035 } else { 0.055 };
                let lift = if p.kind == OrganKind::Leaf {
                    0.03 * (light[i] / 800.0).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                base + lift * src.light as f32
            })
            .collect();
        let phenology: Vec<f32> = skeleton.organs.iter().map(|p| p.thermal_age).collect();
        let root_age: Vec<f32> = skeleton
            .organs
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if p.kind == OrganKind::Root {
                    (clock - a_arena.birth.get(i).map_or(clock, |b| b.0 as f64)) as f32 / 24.0
                } else {
                    f32::NAN
                }
            })
            .collect();
        let soil_water = soil_face(&skeleton, size, clock);
        let wet: Vec<f32> = soil_water
            .values
            .iter()
            .copied()
            .filter(|v| *v < 0.999)
            .collect();
        let ftsw = if wet.is_empty() {
            1.0
        } else {
            wet.iter().sum::<f32>() / wet.len() as f32
        };
        // Stress begins below 40% transpirable water (Sinclair and Ludlow style threshold).
        let water_stress = (ftsw / 0.4).clamp(0.0, 1.0);

        let mut organ_values = BTreeMap::new();
        organ_values.insert(Lens::Light, light);
        organ_values.insert(Lens::ShadeSignal, rfr);
        organ_values.insert(Lens::Carbon, carbon);
        organ_values.insert(Lens::Phenology, phenology);
        organ_values.insert(Lens::RootsXray, root_age);
        pots.push(PotView {
            label: src.label.to_owned(),
            center: src.center,
            size,
            skeleton,
            water_stress,
            first_flower: None,
            organ_values,
            soil_water,
            container: true,
            supports: Vec::new(),
            obstacles: Vec::new(),
        });
    }
    garden.pots = pots;
    garden.stand_in_lenses = vec![
        Lens::SoilWater,
        Lens::PlantWater,
        Lens::Carbon,
        Lens::Phenology,
        Lens::RootsXray,
    ];

    let date = demo.start.plus_hours(h0);
    time_ui.hour = clock;
    time_ui.latest_hour = demo.latest;
    time_ui.end_hour = end;
    time_ui.speed = demo.speed;
    time_ui.paused = demo.paused;
    time_ui.date = format!(
        "{}-{:02}-{:02} {:02}:00",
        date.year, date.month, date.day, date.hour
    );
    season(date.month).clone_into(&mut time_ui.season);
    time_ui.weather = weather.map_or_else(String::new, |w| {
        format!(
            "{:.1} °C · {:.0} W/m² · {:.0}% cloud{}",
            w.t_air.0,
            w.global.0,
            w.cloud_fraction * 100.0,
            if w.rain.0 > 0.0 { " · rain" } else { "" }
        )
    });
}
