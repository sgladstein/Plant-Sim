# 0016: The sim thread, its snapshots and scrubbing (WP16)

- **Date:** 2026-11-05
- **Status:** accepted

## Context

WP16 puts the sim on its own thread. The renderer reads a double-buffered snapshot,
the top bar's time commands (pause, speed, step, seek) drive time, and actions reach the
world through the action log only (§2.2, §5.8, §11.9). The done-when is that the
session's hash equals a headless replay's. The plan leaves open where the thread lives,
what a snapshot carries, how a seek into the past works before the WP26 scrubber, and
what an action taken in the past means.

## Decision

1. **A new engine-free crate, ps-session**, owns the thread. It depends on ps-world and
   ps-weather only, so the headless CLI and tests can drive a session too; ps-app wraps
   it in a Bevy resource. It keeps every rule of the sim crates except the wall clock:
   `Instant` paces the steps and nothing else, each such line marked for the
   determinism lint.
2. **Double buffer.** The sim thread builds the next `Snapshot` without a lock and swaps
   it in as an `Arc` (one pointer store under a mutex held only for the swap). Readers
   clone the `Arc`. Snapshots carry the tick, a fractional draw hour, the latest tick,
   speed (requested and achieved), pause and busy flags, date, open-sky weather, the
   soils and plants (with ps-arch positions and organ metadata), the daily rows and the
   action log (both shared `Arc` slices, rebuilt only when they change), the last daily
   hash, the last rejection and any step error. Publishing is at most once per 8 ms
   while busy. Measured in release on the two-pot scenario: copying soils and plants
   takes 4 µs on day 10 and 75 µs on day 90.
3. **Time commands** mirror `ps_ui::TimeCommand`. `SetSpeed` resumes. `Step(h)` runs
   `round(h)` hours (at least one, stopping at the scenario end) as fast as possible,
   publishing as it goes, then pauses; pressing a step again while stepping extends the
   target. `TogglePause` during a step cancels it. When the sim cannot keep up, it
   drops backlog beyond 250 ms of requested time rather than racing to catch up.
4. **Actions** apply at the sim's current tick through `World::apply`. Each is first
   tried on a copy for one hour, so an invalid action is rejected (reported in the
   snapshot and to `act_and_wait`) and never enters the log.
5. **Seeking.** The thread keeps a copy of the state at every midnight (tick 0
   included). A seek back restores the nearest copy at or before the target, re-adds
   the rest of the log and steps forward, so the scrubber shows bit for bit what the
   session computed. A seek forward within the recorded timeline steps there. Running
   from the past replays the recorded timeline and becomes live past the latest tick.
   About 1 MB of state per copy at day 90, so 90 days of copies stay under about 100 MB;
   `keyframe_every_days` thins them for longer runs.
6. **Acting in the past branches.** An action while behind the latest tick discards the
   recorded future (later actions and midnight copies) and the latest tick becomes the
   current one, as when a save is loaded and changed. The WP26 scrubber can add a
   confirmation.

## Consequences

- The session's world is the same `World` a headless run uses; `Session::stop` returns
  it, and `tests/session.rs` replays its log headlessly to the same full-state and daily
  hashes, including after wall-clock running, a seek back and a branch.
- ps-world gains three read accessors (`weather`, `weather_now`, `end_tick`); no step
  changes, so the model version stays 0.1.0.
- Viewer integration (WP16's ps-app side): a `Session` resource, map `ps_ui::TimeCommand`
  one to one, read `Snapshot::plants` into `ps_view::Garden`.
