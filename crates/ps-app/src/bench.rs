//! `--bench-frames N`: frame-time measurement without a profiler (WP13 hardware checks).
//!
//! After the pipelines are ready and a warm-up, records N frame times (real time between
//! frames, so it includes vsync waits), writes them as CSV, logs a summary line with the
//! adapter, backend, present mode, frame latency and frame cap, then exits.

use std::fmt::Write as _;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use ps_view::Ready;

/// Frames skipped after the scene is ready, so caches and the camera have settled.
pub const WARMUP_FRAMES: u32 = 120;

/// What to measure, from the command line.
#[derive(Clone, Debug, Resource)]
pub struct BenchConfig {
    /// Frames to record.
    pub frames: u32,
    /// CSV output path.
    pub out: PathBuf,
    /// Present mode, for the summary.
    pub present: String,
    /// Maximum frame latency, for the summary.
    pub latency: String,
}

/// Summary statistics over frame times, ms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub n: usize,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
}

/// Nearest-rank percentile of sorted values (`q` in 0..=1).
fn percentile(sorted: &[f64], q: f64) -> f64 {
    let rank = (q * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

/// Statistics of `ms`; `None` when empty.
#[must_use]
pub fn stats(ms: &[f64]) -> Option<Stats> {
    if ms.is_empty() {
        return None;
    }
    let mut sorted = ms.to_vec();
    sorted.sort_by(f64::total_cmp);
    Some(Stats {
        n: ms.len(),
        mean: ms.iter().sum::<f64>() / ms.len() as f64,
        p50: percentile(&sorted, 0.50),
        p95: percentile(&sorted, 0.95),
        p99: percentile(&sorted, 0.99),
        max: sorted[sorted.len() - 1],
    })
}

/// The CSV: one row per frame, then summary rows as comments.
#[must_use]
pub fn csv(ms: &[f64], summary: &str) -> String {
    let mut s = String::from("frame,ms\n");
    for (i, v) in ms.iter().enumerate() {
        let _ = writeln!(s, "{i},{v:.4}");
    }
    let _ = writeln!(s, "# {summary}");
    s
}

#[derive(Default)]
pub(crate) struct Recorder {
    after_ready: u32,
    ms: Vec<f64>,
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn record(
    config: Res<BenchConfig>,
    ready: Res<Ready>,
    time: Res<Time<Real>>,
    adapter: Option<Res<RenderAdapterInfo>>,
    cap: Option<Res<crate::display::FrameCap>>,
    windows: Query<&Window>,
    mut rec: Local<Recorder>,
    mut exit: MessageWriter<AppExit>,
) {
    if !ready.ready {
        return;
    }
    rec.after_ready += 1;
    if rec.after_ready <= WARMUP_FRAMES {
        return;
    }
    rec.ms.push(time.delta_secs_f64() * 1000.0);
    if rec.ms.len() < config.frames as usize {
        return;
    }
    let Some(s) = stats(&rec.ms) else {
        return;
    };
    let gpu = adapter.map_or_else(
        || "unknown adapter".to_owned(),
        |a| format!("{} ({:?})", a.name, a.backend),
    );
    let size = windows.iter().next().map_or_else(String::new, |w| {
        format!(" {}x{}", w.physical_width(), w.physical_height())
    });
    let cap = cap
        .and_then(|c| if c.off { None } else { c.effective_fps() })
        .map_or_else(|| "off".to_owned(), |f| format!("{f:.0}"));
    let summary = format!(
        "{gpu}{size} present={} latency={} cap={cap} frames={} mean={:.2} p50={:.2} p95={:.2} p99={:.2} max={:.2} ms",
        config.present, config.latency, s.n, s.mean, s.p50, s.p95, s.p99, s.max
    );
    info!("bench: {summary}");
    println!("{summary}");
    match std::fs::write(&config.out, csv(&rec.ms, &summary)) {
        Ok(()) => info!("bench: wrote {}", config.out.display()),
        Err(e) => error!("bench: could not write {}: {e}", config.out.display()),
    }
    exit.write(AppExit::Success);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_of_known_values() {
        let ms: Vec<f64> = (1..=100).map(f64::from).collect();
        let s = stats(&ms).unwrap_or_else(|| unreachable!());
        assert_eq!(s.n, 100);
        assert!((s.mean - 50.5).abs() < 1e-12);
        assert!((s.p50 - 50.0).abs() < 1e-12);
        assert!((s.p95 - 95.0).abs() < 1e-12);
        assert!((s.p99 - 99.0).abs() < 1e-12);
        assert!((s.max - 100.0).abs() < 1e-12);
        assert!(stats(&[]).is_none());
    }

    #[test]
    fn csv_has_header_rows_and_summary() {
        let text = csv(&[1.0, 2.5], "x");
        assert_eq!(text, "frame,ms\n0,1.0000\n1,2.5000\n# x\n");
    }
}
