# 0014: World schedule, snapshots and the full-state hash (WP9)

- **Date:** 2026-11-05
- **Status:** accepted

## Context

WP9 joins weather, soils and plants into one world stepped on the §3.2 schedule, with
an action log, snapshots and a full-state hash (§3.14, §6.2, §6.3, §11.5). The plan
leaves open how the hash covers the state, what a snapshot holds, how watering and
shade cloth enter the hour, and where the hourly steps of several plants in one soil
meet.

## Decision

1. **Hourly order per soil.** Actions due this tick apply first. Then, for each soil
   in scenario order: every plant's light, water status and gas exchange from the
   soil as it stands (steps 3, 5 and 6), the soil's water step with rain, scheduled
   irrigation and action water (step 4), one joint uptake for all the soil's plants
   (step 7), then each plant's carbon steps (8 to 12). Step 4 runs after the demand
   is computed and before uptake, so uptake draws on the hour's infiltrated water;
   the stress factor uses the FTSW at the start of the hour.
2. **Daily step** after the hour whose clock hour is 23 (sim-midnight): every plant's
   daily steps with the next day's sun path, one summary row per plant, then the
   full-state hash.
3. **Radiation fraction** (shade cloth) scales global, direct and diffuse radiation
   for a soil and its plants.
4. **Irrigation** happens at 06:00 local solar time on days `0, n, 2n, …` from the
   scenario start: a fixed depth (`Schedule`) or enough water to bring every cell to
   field capacity (`ToFieldCapacity`). `Water` actions add their depth at the next
   hourly step.
5. **Hash.** A serde serializer feeds every number of the state, as its bit pattern,
   to a splitmix64-based mixer in serialization order, with tags for options,
   sequence lengths and enum variants. It covers the model version, scenario,
   catalog (species files and weather text), tick, soils with their ledgers, plants,
   the applied part of the action log, the daily rows and the earlier hashes. Actions
   scheduled for later ticks are not state and are left out, so a replay that knows
   its whole log from the start hashes like the live session. Hourly scratch (a
   plant's leaf income between its two hourly calls) and the constant sky table are
   not serialized and are rebuilt.
6. **Snapshots** are the same state as RON text. Rust prints `f64` in the shortest
   form that parses back to the same bits, so a snapshot resumes bit for bit; the
   test resumes from text, not from a clone. Snapshots are taken at day boundaries.
   Infinite values never appear in the state (the daily minimum temperature starts at
   `f64::MAX`).
7. **Model version** `0.1.0` lives in ps-world as `MODEL_VERSION`, and the state
   records it.
8. **Actions in M0:** `Sow`, `Water`, `SetRadiationFraction`, `SetIrrigation` and
   `Prune` (an organ and its subtree are shed through `allocation::shed`). Harvest of
   single organs, fertilizer, hand pollination and obstacles come with the
   milestones that need them.

## Consequences

- The §11.5 checklist runs as tests in `crates/ps-world/tests/determinism.rs`: three
  runs, a day-30 snapshot resume to day 90, an extra plant in another pot leaving the
  first plant's hash unchanged, rayon on one thread, and a replayed action log. The
  grep check is `scripts/check_determinism.py` in CI.
- Golden hash files per platform (§11.6) are not committed yet: the Windows file has
  to come from a Windows run. WP10 adds them with the CLI that prints them.
- Plants in one soil interact only through water. Light interaction between plants
  (one grid per merged pot group) is M1 (§3.5).
