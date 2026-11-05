//! Plant-Sim desktop app: the two-pot screen (design plan §2.1, M1).
//!
//! Until ps-world exists the app is driven by [`demo_source`]: stand-in plants under
//! real weather and the real light model. Options:
//!
//! ```text
//! plant-sim [--day N] [--speed H] [--paused] [--lens NAME] [--screenshot FILE.png]
//!           [--present auto|fifo|mailbox|immediate] [--frame-latency N]
//! ```
//!
//! `--speed` is sim hours per second (1, 6, 24, 72, 168). `--lens` is one of light,
//! soil-water, plant-water, roots, carbon, phenology, shade. `--screenshot` waits until
//! the GPU pipelines have compiled and the scene has settled, saves the window and exits
//! (for headless checks and bug reports). `--present` and `--frame-latency` set the
//! swap chain's present mode (default auto vsync) and maximum queued frames (default
//! the driver's), for frame-pacing tests.

// Converting sim f64 and counts to f32 for display is intended.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

mod demo_source;

use std::path::PathBuf;

use anyhow::{Result, bail};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{PresentMode, WindowResolution};
use ps_view::{Lens, PlantViewPlugin, Ready, ViewSet};

#[derive(Clone, Debug, Resource)]
struct Options {
    day: f64,
    speed: f64,
    paused: bool,
    lens: Lens,
    screenshot: Option<PathBuf>,
    present: PresentMode,
    frame_latency: Option<std::num::NonZero<u32>>,
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

fn parse_args() -> Result<Options> {
    let mut o = Options {
        day: 0.0,
        speed: 24.0,
        paused: false,
        lens: Lens::Off,
        screenshot: None,
        present: PresentMode::AutoVsync,
        frame_latency: None,
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
            "--screenshot" => o.screenshot = Some(PathBuf::from(value()?)),
            "--present" => o.present = parse_present(&value()?)?,
            "--frame-latency" => {
                let n = anyhow::Context::context(value()?.parse::<u32>(), "--frame-latency")?;
                o.frame_latency = std::num::NonZero::new(n);
            }
            "--help" | "-h" => {
                println!(
                    "plant-sim [--day N] [--speed H] [--paused] [--lens NAME] [--screenshot FILE.png] [--present auto|fifo|mailbox|immediate|novsync] [--frame-latency N]"
                );
                std::process::exit(0);
            }
            other => bail!("unknown option {other:?} (try --help)"),
        }
    }
    Ok(o)
}

fn main() -> Result<()> {
    let options = parse_args()?;
    let paused = options.paused || options.screenshot.is_some();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Plant-Sim".into(),
            resolution: WindowResolution::new(1600, 900),
            present_mode: options.present,
            desired_maximum_frame_latency: options.frame_latency,
            ..default()
        }),
        ..default()
    }))
    .add_plugins((PlantViewPlugin, ps_ui::PlantUiPlugin))
    .insert_resource(demo_source::DemoSource::new(
        options.day,
        options.speed,
        paused,
    ))
    .insert_resource(options.lens)
    .insert_resource(options.clone())
    .add_systems(Update, demo_source::drive.in_set(ViewSet::Source));
    if options.screenshot.is_some() {
        app.add_systems(Update, screenshot_and_exit);
    }
    app.run();
    Ok(())
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
    time: Res<Time<Real>>,
    mut shot: Local<Shot>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(path) = &options.screenshot else {
        return;
    };
    if ready.ready {
        shot.frames_ready += 1;
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
