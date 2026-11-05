//! Plant-Sim desktop app: the two-pot screen (design plan §2.1, M1).
//!
//! The app runs a ps-world scenario on the ps-session sim thread ([`sim_source`]); with
//! `--demo`, or when no data directory is found, it shows [`demo_source`]'s stand-in
//! plants instead. Options:
//!
//! ```text
//! plant-sim [--day N] [--speed H] [--paused] [--lens NAME] [--screenshot FILE.png]
//!           [--present auto|fifo|mailbox|immediate|novsync] [--frame-latency N]
//!           [--bench-frames N] [--bench-out FILE.csv]
//!           [--scenario FILE.ron] [--data DIR] [--demo] [--look natural|illustrated|painterly]
//!           [--backdrop garden|plate] [--view YAW,PITCH]
//!           [--window WxH] [--fps-cap N]
//! ```
//!
//! The default present mode is mailbox with a frame cap at twice the monitor's refresh rate
//! (`--fps-cap N` overrides it, 0 turns it off; FIFO and auto vsync are never capped).
//! `--window WxH` sets the window size in physical pixels (clamped to fit the monitor);
//! without it the 1600x900 logical default shrinks to fit the monitor.
//!
//! `--day` starts the view that many days in (the sim steps there as fast as it can).
//! `--scenario` is a path, or relative to the data directory (default
//! `scenarios/two_pot_sun_shade.ron`); `--data` overrides the search for `data/`.
//!
//! `--speed` is sim hours per second (1, 6, 24, 72, 168). `--lens` is one of light,
//! soil-water, plant-water, roots, carbon, phenology, shade. `--charts` opens the
//! charts panel. `--inspect POT[:ORGAN]` selects a plant or an organ for the inspector.
//! `--captures DIR` is where the Screenshot and Export CSV buttons write (default
//! `captures`). `--timelapse DIR` renders time-lapse frames offline from `--day` to
//! `--to-day` (default the scenario's end), one per sim hour within `--timelapse-hours`
//! (default 6-19), then exits; see `capture.rs`. `--clean` hides the panels and shows
//! only a caption, for clips. `--screenshot` waits until
//! the GPU pipelines have compiled and the scene has settled, saves the window and exits
//! (for headless checks and bug reports). `--present` and `--frame-latency` set the
//! swap chain's present mode (default auto vsync) and maximum queued frames (default
//! the driver's), for frame-pacing tests. `--bench-frames N` records N frame times after
//! the pipelines are ready and a 120-frame warm-up, writes them to `--bench-out`
//! (default frame-times.csv) with a summary line (mean, p50, p95, p99, max), and exits.

// Converting sim f64 and counts to f32 for display is intended.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

mod bench;
mod capture;
mod demo_source;
mod display;
mod inspect;
mod sim_source;
mod tools;

use std::path::PathBuf;

use anyhow::{Result, bail};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{PresentMode, WindowResolution};
use ps_view::{Lens, PlantViewPlugin, Ready, ViewSet};

// Command-line switches are independent flags.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Resource)]
struct Options {
    day: f64,
    speed: f64,
    paused: bool,
    lens: Lens,
    charts: bool,
    inspect: Option<ps_view::Picked>,
    /// Camera yaw and pitch, degrees (`--view`), for screenshots from a set angle.
    view: Option<(f32, f32)>,
    captures: PathBuf,
    timelapse: Option<PathBuf>,
    to_day: Option<f64>,
    timelapse_hours: (u32, u32),
    clean: bool,
    screenshot: Option<PathBuf>,
    present: PresentMode,
    frame_latency: Option<std::num::NonZero<u32>>,
    present_name: String,
    bench_frames: Option<u32>,
    bench_out: PathBuf,
    demo: bool,
    look: ps_view::toon::Style,
    backdrop: ps_view::look::Backdrop,
    scenario: PathBuf,
    data: Option<PathBuf>,
    fps_cap: Option<f64>,
    window: (u32, u32),
    window_explicit: bool,
}

fn parse_present(name: &str) -> Result<PresentMode> {
    Ok(match name {
        "auto" => PresentMode::AutoVsync,
        "fifo" => PresentMode::Fifo,
        "mailbox" => PresentMode::Mailbox,
        "immediate" => PresentMode::Immediate,
        "novsync" => PresentMode::AutoNoVsync,
        other => bail!("unknown present mode {other:?}"),
    })
}

/// `H0-H1`, hours of the day, for example `6-19`.
fn parse_hours(s: &str) -> Result<(u32, u32)> {
    let parsed = s
        .split_once('-')
        .and_then(|(a, b)| Some((a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?)));
    match parsed {
        Some((a, b)) if a <= b && b < 24 => Ok((a, b)),
        _ => bail!("--timelapse-hours takes H0-H1 within 0-23, got {s}"),
    }
}

/// `POT` or `POT:ORGAN`, for example `1` (the second pot's plant) or `0:12`.
fn parse_pick(s: &str) -> Result<ps_view::Picked> {
    let mut parts = s.splitn(2, ':');
    let pot = parts.next().unwrap_or_default().parse::<usize>();
    let organ = parts.next().map(str::parse::<usize>).transpose();
    match (pot, organ) {
        (Ok(pot), Ok(organ)) => Ok(ps_view::Picked { pot, organ }),
        _ => bail!("--inspect takes POT or POT:ORGAN, got {s}"),
    }
}

fn parse_lens(name: &str) -> Result<Lens> {
    Ok(match name {
        "off" => Lens::Off,
        "light" => Lens::Light,
        "soil-water" => Lens::SoilWater,
        "plant-water" => Lens::PlantWater,
        "roots" => Lens::RootsXray,
        "carbon" => Lens::Carbon,
        "phenology" => Lens::Phenology,
        "shade" => Lens::ShadeSignal,
        other => bail!("unknown lens {other:?}"),
    })
}

/// Parses `--view YAW,PITCH` (degrees).
fn parse_view(v: &str) -> Result<(f32, f32)> {
    let parsed = v
        .split_once(',')
        .and_then(|(a, b)| Some((a.parse::<f32>().ok()?, b.parse::<f32>().ok()?)));
    anyhow::Context::with_context(parsed, || {
        format!("--view takes YAW,PITCH in degrees, got {v}")
    })
}

// One flat match arm per flag.
#[allow(clippy::too_many_lines)]
fn parse_args() -> Result<Options> {
    let mut o = Options {
        day: 0.0,
        speed: 24.0,
        paused: false,
        lens: Lens::Off,
        charts: false,
        inspect: None,
        view: None,
        captures: PathBuf::from("captures"),
        timelapse: None,
        to_day: None,
        timelapse_hours: (6, 19),
        clean: false,
        screenshot: None,
        present: PresentMode::Mailbox,
        frame_latency: None,
        present_name: "mailbox".into(),
        bench_frames: None,
        bench_out: PathBuf::from("frame-times.csv"),
        demo: false,
        look: ps_view::toon::Style::default(),
        backdrop: ps_view::look::Backdrop::default(),
        scenario: PathBuf::from(sim_source::DEFAULT_SCENARIO),
        data: None,
        fps_cap: None,
        window: (1600, 900),
        window_explicit: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value =
            || anyhow::Context::with_context(args.next(), || format!("{a} needs a value"));
        match a.as_str() {
            "--day" => o.day = anyhow::Context::context(value()?.parse::<f64>(), "--day")?,
            "--speed" => o.speed = anyhow::Context::context(value()?.parse::<f64>(), "--speed")?,
            "--paused" => o.paused = true,
            "--lens" => o.lens = parse_lens(&value()?)?,
            "--charts" => o.charts = true,
            "--inspect" => o.inspect = Some(parse_pick(&value()?)?),
            "--clean" => o.clean = true,
            "--view" => o.view = Some(parse_view(&value()?)?),
            "--captures" => o.captures = PathBuf::from(value()?),
            "--timelapse" => o.timelapse = Some(PathBuf::from(value()?)),
            "--to-day" => {
                o.to_day = Some(anyhow::Context::context(
                    value()?.parse::<f64>(),
                    "--to-day",
                )?);
            }
            "--timelapse-hours" => o.timelapse_hours = parse_hours(&value()?)?,
            "--screenshot" => o.screenshot = Some(PathBuf::from(value()?)),
            "--present" => {
                o.present_name = value()?;
                o.present = parse_present(&o.present_name)?;
            }
            "--bench-frames" => {
                o.bench_frames = Some(anyhow::Context::context(
                    value()?.parse::<u32>(),
                    "--bench-frames",
                )?);
            }
            "--bench-out" => o.bench_out = PathBuf::from(value()?),
            "--demo" => o.demo = true,
            "--look" => {
                let v = value()?;
                o.look = anyhow::Context::with_context(ps_view::toon::Style::parse(&v), || {
                    format!("--look takes natural, illustrated or painterly, got {v}")
                })?;
            }
            "--backdrop" => {
                let v = value()?;
                o.backdrop =
                    anyhow::Context::with_context(ps_view::look::Backdrop::parse(&v), || {
                        format!("--backdrop takes garden or plate, got {v}")
                    })?;
            }
            "--fps-cap" => {
                o.fps_cap = Some(anyhow::Context::context(
                    value()?.parse::<f64>(),
                    "--fps-cap",
                )?);
            }
            "--window" => {
                let v = value()?;
                let parsed = v
                    .split_once(['x', 'X'])
                    .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
                    .filter(|(w, h)| *w >= 320 && *h >= 240);
                o.window = anyhow::Context::with_context(parsed, || {
                    format!("--window wants WxH, at least 320x240, got {v:?}")
                })?;
                o.window_explicit = true;
            }
            "--scenario" => o.scenario = PathBuf::from(value()?),
            "--data" => o.data = Some(PathBuf::from(value()?)),
            "--frame-latency" => {
                let n = anyhow::Context::context(value()?.parse::<u32>(), "--frame-latency")?;
                o.frame_latency = std::num::NonZero::new(n);
            }
            "--help" | "-h" => {
                println!(
                    "plant-sim [--day N] [--speed H] [--paused] [--lens NAME] [--screenshot FILE.png] [--present auto|fifo|mailbox|immediate|novsync] [--frame-latency N] [--bench-frames N] [--bench-out FILE.csv] [--scenario FILE.ron] [--data DIR] [--demo] [--look natural|illustrated|painterly] [--backdrop garden|plate] [--view YAW,PITCH] [--window WxH] [--fps-cap N]"
                );
                std::process::exit(0);
            }
            other => bail!("unknown option {other:?} (try --help)"),
        }
    }
    Ok(o)
}

#[allow(clippy::too_many_lines)] // Plain wiring of options into the app.
fn main() -> Result<()> {
    let options = parse_args()?;
    let paused = options.paused || options.screenshot.is_some() || options.timelapse.is_some();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Plant-Sim".into(),
            resolution: WindowResolution::new(options.window.0, options.window.1),
            present_mode: options.present,
            desired_maximum_frame_latency: options.frame_latency,
            ..default()
        }),
        ..default()
    }))
    .add_plugins((PlantViewPlugin, ps_ui::PlantUiPlugin))
    .insert_resource(options.lens)
    .insert_resource(options.look)
    .insert_resource(match options.view {
        Some((yaw, pitch)) => ps_view::CameraRig {
            yaw: yaw.to_radians(),
            pitch: pitch.to_radians(),
            ..default()
        },
        None => ps_view::CameraRig::default(),
    })
    .insert_resource(options.backdrop)
    .insert_resource(ps_view::Selection {
        picked: options.inspect,
    })
    .insert_resource(ps_ui::CaptureControl {
        clean: options.clean,
        ..default()
    })
    .insert_resource(ps_ui::Charts {
        open: options.charts,
        ..default()
    })
    .insert_resource(options.clone());
    let sim = if options.demo {
        None
    } else {
        match start_sim(&options, paused) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("plant-sim: {e:#}\nplant-sim: showing the stand-in demo plants instead");
                None
            }
        }
    };
    let stem = options.scenario.file_stem().map_or_else(
        || "plant-sim".to_owned(),
        |s| s.to_string_lossy().into_owned(),
    );
    app.insert_resource(capture::CaptureConfig {
        dir: options.captures.clone(),
        stem,
    })
    .add_systems(Update, capture::capture_requests.after(ViewSet::Source));
    if let Some((sim, data, scenario)) = sim {
        let mut ui_tools = ps_ui::Tools::default();
        tools::init_new_run(&mut ui_tools.new_run, &data, &scenario, &options.scenario);
        app.insert_resource(ui_tools)
            .insert_resource(tools::RunSettings {
                data,
                speed: options.speed,
            });
        if let Some(dir) = &options.timelapse {
            std::fs::create_dir_all(dir)?;
            let to = options.to_day.unwrap_or(f64::MAX / 48.0);
            let ticks = capture::timelapse_ticks(
                options.day,
                to,
                options.timelapse_hours,
                sim.day_start_hour(),
                sim.end_tick(),
            );
            info!("time-lapse: {} frames to {}", ticks.len(), dir.display());
            app.insert_resource(capture::Timelapse::new(dir.clone(), ticks))
                .add_systems(Update, capture::timelapse.before(ViewSet::Source));
        }
        app.insert_resource(sim).add_systems(
            Update,
            (
                tools::new_run,
                tools::apply_tools,
                sim_source::drive,
                inspect::inspect,
            )
                .chain()
                .in_set(ViewSet::Source),
        );
    } else {
        app.insert_resource(demo_source::DemoSource::new(
            options.day,
            options.speed,
            paused,
        ))
        .add_systems(Update, demo_source::drive.in_set(ViewSet::Source));
    }
    let vsync = matches!(options.present, PresentMode::Fifo | PresentMode::AutoVsync);
    let cap_off = vsync || options.fps_cap.is_some_and(|f| f <= 0.0);
    if options.window_explicit {
        app.insert_resource(display::ExplicitWindowSize(UVec2::new(
            options.window.0,
            options.window.1,
        )));
    }
    app.insert_resource(display::FrameCap::new(options.fps_cap, cap_off))
        .add_systems(Update, display::fit_to_monitor)
        .add_systems(Last, display::limit_frame);
    if let Some(frames) = options.bench_frames {
        app.insert_resource(bench::BenchConfig {
            frames: frames.max(1),
            out: options.bench_out.clone(),
            present: options.present_name.clone(),
            latency: options
                .frame_latency
                .map_or_else(|| "default".to_owned(), |n| n.to_string()),
        })
        .add_systems(Last, bench::record);
    }
    if options.screenshot.is_some() {
        app.add_systems(Update, screenshot_and_exit);
    }
    app.run();
    Ok(())
}

/// Loads the scenario and starts the sim thread.
fn start_sim(
    options: &Options,
    paused: bool,
) -> Result<(sim_source::SimSource, PathBuf, ps_world::Scenario)> {
    let data = sim_source::find_data_dir(options.data.as_deref())?;
    let (world, scenario) = sim_source::load_world(&data, &options.scenario)?;
    let src = sim_source::SimSource::start(world, &scenario, options.day, options.speed, paused)?;
    Ok((src, data, scenario))
}

/// Frames rendered after the pipelines are ready before the screenshot, so the camera
/// has eased onto the plants.
const SETTLE_AFTER_READY: u32 = 60;
/// Give up if the scene is not ready after this many seconds.
const SCREENSHOT_TIMEOUT_S: f32 = 180.0;

/// Screenshot progress.
#[derive(Default)]
struct Shot {
    frames_ready: u32,
    requested: bool,
    saved: std::sync::Arc<std::sync::atomic::AtomicBool>,
    frames_since_saved: u32,
}

/// Waits until the pipelines are compiled and the scene has settled, saves one
/// screenshot, then exits once the file is written.
#[allow(clippy::needless_pass_by_value)]
fn screenshot_and_exit(
    mut commands: Commands,
    options: Res<Options>,
    ready: Res<Ready>,
    time_ui: Res<ps_ui::TimeControl>,
    time: Res<Time<Real>>,
    mut shot: Local<Shot>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(path) = &options.screenshot else {
        return;
    };
    // The sim may still be stepping to --day; wait until it has arrived.
    if ready.ready && !time_ui.busy {
        shot.frames_ready += 1;
    } else {
        shot.frames_ready = 0;
    }
    if !shot.requested && shot.frames_ready >= SETTLE_AFTER_READY {
        shot.requested = true;
        let saved = shot.saved.clone();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()))
            .observe(
                move |_: On<bevy::render::view::screenshot::ScreenshotCaptured>| {
                    saved.store(true, std::sync::atomic::Ordering::Relaxed);
                },
            );
    }
    if shot.saved.load(std::sync::atomic::Ordering::Relaxed) {
        // A few more frames so the save observer has certainly run.
        shot.frames_since_saved += 1;
        if shot.frames_since_saved > 5 {
            exit.write(AppExit::Success);
        }
    } else if time.elapsed_secs() > SCREENSHOT_TIMEOUT_S {
        error!(
            "screenshot: scene not ready after {SCREENSHOT_TIMEOUT_S} s ({} pipelines waiting)",
            ready.waiting
        );
        exit.write(AppExit::error());
    }
}
