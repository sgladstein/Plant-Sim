# 0015: Export, CLI and scenario treatments (WP10)

- **Date:** 2026-11-05
- **Status:** accepted

## Context

WP10 asks for scenario files, `ps-cli run`, `hash`, `bench` and `replay`, and CSV
time series and per-organ tables, with the four §8.3 scenarios running from one
command. Two of those scenarios need a treatment at a set time (drought and heat at
flowering), which the WP9 scenario could not express. The WP9 review asked for
pruning to remove carbon instead of resorbing it, for warnings on a model version
mismatch, and for date checks in scenario validation.

## Decision

1. **The four scenarios** are `data/scenarios/response/`: `control` (the `single_pot`
   set-up), `drought_from_flowering`, `heat_at_flowering` and `shade_50`. All three
   treatments are applied to the control's pot, seed and weather. The drought and heat
   start on the control's flowering day, which is day 33. The heat is 32/20 °C for six
   days. The shade is 50% from sowing. §8.3's drought-allocation row is checked on the
   drought run, and its lower-shaded-leaves row on every run. Dense planting, pruning
   and nitrogen are M2.
2. **Treatments are part of the scenario.** Two optional fields were added, so old
   files still parse and the schema stays 1:
   - `heat_wave` replaces the preset's own hot spell.
   - `events` is a list of actions sorted by tick. Events apply before log actions of
     the same tick. They never enter the player's log, so replaying a log never
     applies them twice.
   A treatment that waits for a stage (for example "at flowering, whenever that is")
   is left for when a challenge needs it.
3. **Daily rows** carry the sim day and the day's environment (`DayEnv`): the hourly
   minimum and maximum air temperature, radiation after the shade cloth, rain,
   root-weighted FTSW and soil water. The report adds leaf, stem and seed mass, so
   the §8 ratios can be computed from the CSV alone.
4. **CSV** has a header row with units in the column names. Numbers are written in
   Rust's shortest round-trip form. Stages and roles are written as their enum names.
   `run` writes `daily.csv`, `organs.csv` (every organ ever built, at the end of the
   run), `hashes.csv` and `log.ron` into `OUT/<scenario stem>/`. Scenarios given to
   one command run on parallel threads; each world is independent.
5. **Golden hashes** live at `tests/golden/<os>-<arch>/<stem>.csv`, one `day,hash` row
   per day. They cover `single_pot` and `two_pot_sun_shade`. CI runs
   `ps-cli hash --check-golden` on Linux and Windows. When a platform has no file, the
   command prints the hashes instead of failing, which is how the Windows file is
   first recorded. Debug and release builds give the same hashes. The scenario is part
   of the hash, so `--days` changes every day's hash, and a 40-day run's day-30 hash is
   not a 90-day run's day-30 hash.
6. **Cutting** goes through `allocation::cut`. Nothing is resorbed, and each cut
   organ's structure and reserve (a debt included) leave the plant as the ledger's
   `removed` term. Pruning the collar or hypocotyl is an error. Actions given for the
   current tick are checked against the state before they are logged, so a rejected
   action never enters the log.
7. **Version warnings.** `World::new` warns (tracing) when the scenario's
   `model_version` differs from the build's, and `World::from_state` does the same for
   a save.
8. **Model version** stays 0.1.0. These are the first golden hashes, so there is no
   earlier set to invalidate. From here on, CLAUDE.md's version bump rule applies.

## Consequences

- WP11 reads `daily.csv` and `organs.csv`. The reference data thread uses the column
  names above (docs/validation-data.md).
- A first look at the four runs, for WP11 and WP12:
  - Heat cuts seed weight by 21% against the cited 50 to 71%.
  - Drought cuts it by 19%.
  - Shade cuts total mass by 35%, but pods go up (18 against 16) and root mass falls
    sixfold.
