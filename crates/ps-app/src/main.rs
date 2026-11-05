//! Plant-Sim desktop app: the two-pot screen (design plan §2.1, M1).
//!
//! Until ps-world exists the app is driven by [`demo_source`]: stand-in plants under
//! real weather and the real light model. Options:
//!
//! ```text
//! plant-sim [--day N] [--speed H] [--paused] [--lens NAME] [--screenshot FILE.png]
//! ```
//!
//! `--speed` is sim hours per second (1, 6, 24, 72, 168). `--lens` is one of light,
//! soil-water, plant-water, roots, carbon, phenology, shade. `--screenshot` renders a
//! few frames, saves the window and exits (for headless checks and bug reports).

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
use bevy::window::WindowResolution;
use ps_view::{Lens, PlantViewPlugin, ViewSet};

#[derive(Clone, Debug, Resource)]
struct Options {
    day: f64,
    speed: f64,
    paused: bool,
    lens: Lens,
    screenshot: Option<PathBuf>,
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
            "--help" | "-h" => {
                println!(
                    "plant-sim [--day N] [--speed H] [--paused] [--lens NAME] [--screenshot FILE.png]"
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

/// Waits for the scene to settle, saves one screenshot, then exits.
#[allow(clippy::needless_pass_by_value)]
fn screenshot_and_exit(
    mut commands: Commands,
    options: Res<Options>,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    *frames += 1;
    if *frames == 90
        && let Some(path) = &options.screenshot
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
    }
    if *frames == 130 {
        exit.write(AppExit::Success);
    }
}
