# ps-cli

The headless runner (WP10, decision 0015). Run it from the repository root. Use a
release build for anything longer than a test.

```sh
cargo run --release -p ps-cli -- run data/scenarios/response/*.ron --out out
```

| Command | What it does |
| --- | --- |
| `run SCENARIO... [--out DIR] [--days N] [--snapshot-day D]` | Runs each scenario, all of them in parallel. Writes `daily.csv`, `organs.csv`, `hashes.csv` and `log.ron` to `DIR/<stem>/`, plus `snapshot_dayD.ron` if `--snapshot-day` is given. |
| `hash SCENARIO... [--days N] [--write-golden DIR] [--check-golden DIR]` | Prints the last daily hash. Writes the golden files, or checks them against `DIR/<os>-<arch>/<stem>.csv`. |
| `bench SCENARIO... [--days N] [--repeat N]` | Reports wall time per sim-day, the best of N runs. |
| `replay SCENARIO LOG [--until-tick T] [--out DIR]` | Replays a `log.ron` from the start and prints the hash. |
| `replay --snapshot FILE [--out DIR]` | Resumes a snapshot to the end of its scenario. |
| `challenge ID [LOG] [--out DIR]` | Replays a log (none: an empty log) on challenge `data/challenges/ID.ron`, prints the score (the mean over the challenge's seasons, with each season's score), medal, broken rules and the first season's weekly debrief. `--out` writes the CSVs and `debrief.ron` to `DIR/ID/`. See docs/game/challenges.md. |

`--data DIR` points at the data directory (default `data`). `--days N` shortens a run.
The scenario is hashed, so `--days` changes every hash.

The CSV columns are in `crates/ps-export/src/lib.rs` (`DAILY_COLUMNS`, `ORGAN_COLUMNS`).
Units are part of the column names. Day 0 is the scenario's first day, which is also
the sowing day.
