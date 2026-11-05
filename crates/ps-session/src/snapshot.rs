//! What the sim thread publishes, and the double buffer it publishes through.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use ps_weather::{DateHour, HourWeather};
use ps_world::{Action, DayRow, ObstacleSpec, PlantEntry, SeedLot, SoilEntry, SupportSpec};

/// One published view of the world: everything the renderer and the UI read.
///
/// A snapshot is a copy, so reading it never touches the sim. It is `Send + Sync +
/// Clone`; the renderer holds an `Arc<Snapshot>` for as long as it likes.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    /// Publication counter, starting at 1 for the first snapshot. A reader that sees
    /// the same `seq` twice can skip its update.
    pub seq: u64,
    /// The world's tick: the state is the world after `tick` hours.
    pub tick: u64,
    /// The hour to draw, `tick` plus the part of the next hour already due while running
    /// (always below `tick + 1`). Organs grow as functions of time, so the renderer can
    /// draw at this fractional hour.
    pub hour: f64,
    /// The latest tick simulated so far (the end of the replay scrubber).
    pub latest_tick: u64,
    /// The scenario's last tick.
    pub end_tick: u64,
    /// Requested speed, sim hours per real second.
    pub speed: f64,
    /// Speed actually reached over the last half second, sim hours per real second.
    pub achieved_speed: f64,
    /// True while paused (also true at the scenario's end and after a step finishes).
    pub paused: bool,
    /// True while a `Step` or a `Seek` is still being worked through.
    pub busy: bool,
    /// Calendar date and clock hour of `tick`.
    pub date: DateHour,
    /// Open-sky weather of the hour starting at `tick` (before shade cloth), if the
    /// series covers it.
    pub weather: Option<HourWeather>,
    /// Soils, in scenario order.
    pub soils: Vec<SoilEntry>,
    /// Plants by id.
    pub plants: Vec<PlantEntry>,
    /// Daily summary rows so far, in day then plant order (charts read these).
    pub history: Arc<[DayRow]>,
    /// The action log: every action in the timeline being shown, applied or pending
    /// (the scrubber's markers).
    pub actions: Arc<[Action]>,
    /// Harvested seed lots, in harvest order (WP25).
    pub seed_library: Vec<SeedLot>,
    /// Supports stood by actions so far, in action order (not the scenario's own).
    pub added_supports: Vec<SupportSpec>,
    /// Obstacles placed by actions so far, in action order.
    pub added_obstacles: Vec<ObstacleSpec>,
    /// The chained daily hash of the last full day, if a day has ended.
    pub day_hash: Option<u64>,
    /// Why the last action was rejected, until the next action is accepted.
    pub rejected: Option<String>,
    /// A step failed and the world stopped. Seeking back clears it.
    pub error: Option<String>,
}

/// The double buffer between the sim thread and its readers.
///
/// The sim thread builds the next snapshot (the back buffer) without holding any
/// lock, then swaps it in as the front with one pointer store. Readers clone the front
/// `Arc`; the lock is held only for that clone, never across sim work.
#[derive(Debug)]
pub struct SnapshotBuffer {
    front: Mutex<Arc<Snapshot>>,
    seq: AtomicU64,
}

impl SnapshotBuffer {
    pub(crate) fn new(first: Snapshot) -> Self {
        let seq = first.seq;
        Self {
            front: Mutex::new(Arc::new(first)),
            seq: AtomicU64::new(seq),
        }
    }

    /// Swaps in `next` and returns the previous front, so the sim thread (not the
    /// render thread) usually drops it.
    pub(crate) fn publish(&self, next: Snapshot) -> Arc<Snapshot> {
        let seq = next.seq;
        let next = Arc::new(next);
        let old = {
            let mut front = self.front.lock().unwrap_or_else(PoisonError::into_inner);
            std::mem::replace(&mut *front, next)
        };
        self.seq.store(seq, Ordering::Release);
        old
    }

    /// The latest snapshot.
    #[must_use]
    pub fn latest(&self) -> Arc<Snapshot> {
        Arc::clone(&self.front.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// The latest snapshot's `seq`, without taking the lock.
    #[must_use]
    pub fn seq(&self) -> u64 {
        self.seq.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync_clone<T: Send + Sync + Clone + 'static>() {}

    #[test]
    fn snapshot_crosses_threads() {
        assert_send_sync_clone::<Snapshot>();
        assert_send_sync_clone::<Arc<SnapshotBuffer>>();
    }
}
