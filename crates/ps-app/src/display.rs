//! Display pacing and window fit (WP13 hardware results, decision in docs/bevy-notes.md).
//!
//! On the owner's GTX 1070, FIFO vsync spiked to about 30 ms every ~14 frames (a present
//! cadence beating against 60 Hz), while mailbox ran smoothly but uncapped at about
//! 250 fps. So the default is mailbox (Bevy falls back to immediate, then FIFO) with a
//! frame limiter at the monitor's refresh rate. The window also shrinks to fit the
//! monitor, since 1600×900 at 175% scaling is wider than a 2560-pixel display.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::window::{Monitor, PrimaryMonitor, PrimaryWindow};

/// Share of the monitor the window may cover at most.
const FIT: f32 = 0.92;

/// The frame limiter: holds each frame until its slot.
#[derive(Debug, Resource)]
pub struct FrameCap {
    /// Requested cap, frames per second; `None` uses the monitor's refresh rate.
    pub fps: Option<f64>,
    /// Off entirely (vsync present modes, or `--fps-cap 0`).
    pub off: bool,
    interval: Option<Duration>,
    next: Option<Instant>,
    frames_without_rate: u32,
}

impl FrameCap {
    /// A limiter at `fps`, or at the monitor's rate when `None`.
    #[must_use]
    pub fn new(fps: Option<f64>, off: bool) -> Self {
        Self {
            fps,
            off,
            interval: None,
            next: None,
            frames_without_rate: 0,
        }
    }

    /// The cap in effect, frames per second, once known.
    #[must_use]
    pub fn effective_fps(&self) -> Option<f64> {
        self.interval.map(|d| 1.0 / d.as_secs_f64())
    }
}

/// The slot after `next` for a frame that finished at `now`: one interval on, or a fresh
/// start if the frame overran by more than an interval (so a hitch does not cause a burst
/// of catch-up frames).
#[must_use]
pub fn next_slot(next: Instant, now: Instant, interval: Duration) -> Instant {
    let due = next + interval;
    if now > due + interval {
        now + interval
    } else {
        due
    }
}

/// Logical window size that fits `fit` of a monitor of `monitor_px` physical pixels at
/// `scale`, keeping the aspect ratio of `want`.
#[must_use]
pub fn fit_window(want: Vec2, monitor_px: UVec2, scale: f32, fit: f32) -> Vec2 {
    if scale <= 0.0 || want.x <= 0.0 || want.y <= 0.0 {
        return want;
    }
    let max = monitor_px.as_vec2() * fit / scale;
    let k = (max.x / want.x).min(max.y / want.y).min(1.0);
    (want * k).floor()
}

/// Sleeps out the rest of the frame's slot (in `Last`).
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn limit_frame(
    mut cap: ResMut<FrameCap>,
    monitors: Query<&Monitor, With<PrimaryMonitor>>,
) {
    if cap.off {
        return;
    }
    if cap.interval.is_none() {
        let hz = cap.fps.or_else(|| {
            monitors
                .iter()
                .next()
                .and_then(|m| m.refresh_rate_millihertz)
                .map(|mhz| f64::from(mhz) / 1000.0)
        });
        // Monitors may not be reported on the first frames, and some report no rate;
        // assume 60 Hz after a moment.
        let hz = match hz.filter(|h| *h >= 10.0) {
            Some(hz) => hz,
            None if cap.frames_without_rate >= 60 => 60.0,
            None => {
                cap.frames_without_rate += 1;
                return;
            }
        };
        cap.interval = Some(Duration::from_secs_f64(1.0 / hz));
        info!("frame cap: {hz:.1} fps");
    }
    let Some(interval) = cap.interval else {
        return;
    };
    let now = Instant::now();
    let next = cap.next.unwrap_or(now);
    if next > now {
        let wait = next - now;
        // Sleep most of the wait; spin the last 2 ms, since sleep can overshoot by a
        // millisecond or more on Windows and miss the slot.
        if let Some(sleep) = wait.checked_sub(Duration::from_millis(2)) {
            std::thread::sleep(sleep);
        }
        while Instant::now() < next {
            std::hint::spin_loop();
        }
    }
    cap.next = Some(next_slot(next, Instant::now(), interval));
}

/// Present when the user chose the window size with `--window`; the size is then kept.
#[derive(Debug, Resource)]
pub struct ExplicitWindowSize;

/// Shrinks the window to fit the primary monitor, once the monitor is known.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn fit_to_monitor(
    explicit: Option<Res<ExplicitWindowSize>>,
    mut done: Local<bool>,
    monitors: Query<&Monitor, With<PrimaryMonitor>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if *done || explicit.is_some() {
        return;
    }
    let (Some(m), Ok(mut w)) = (monitors.iter().next(), windows.single_mut()) else {
        return;
    };
    *done = true;
    let want = Vec2::new(w.resolution.width(), w.resolution.height());
    let fit = fit_window(
        want,
        UVec2::new(m.physical_width, m.physical_height),
        m.scale_factor as f32,
        FIT,
    );
    if fit != want {
        info!(
            "window {}x{} does not fit the {}x{} monitor at {:.0}%; using {}x{}",
            want.x,
            want.y,
            m.physical_width,
            m.physical_height,
            m.scale_factor * 100.0,
            fit.x,
            fit.y
        );
        w.resolution.set(fit.x, fit.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_fits_a_scaled_monitor() {
        // 1600x900 logical at 175% is 2800x1575 physical: too wide for 2560x1600.
        let f = fit_window(Vec2::new(1600.0, 900.0), UVec2::new(2560, 1600), 1.75, 0.92);
        assert!(f.x * 1.75 <= 2560.0 * 0.92 + 1.0);
        assert!((f.x / f.y - 16.0 / 9.0).abs() < 0.01);
        // A window that fits is unchanged.
        let g = fit_window(Vec2::new(1280.0, 720.0), UVec2::new(2560, 1440), 1.0, 0.92);
        assert_eq!(g, Vec2::new(1280.0, 720.0));
    }

    #[test]
    fn slots_advance_evenly_and_reset_after_a_hitch() {
        let t0 = Instant::now();
        let dt = Duration::from_millis(16);
        assert_eq!(next_slot(t0, t0, dt), t0 + dt);
        let late = t0 + Duration::from_millis(100);
        assert_eq!(next_slot(t0, late, dt), late + dt);
    }
}
