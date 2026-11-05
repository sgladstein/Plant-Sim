# 0017: The validation harness (WP11)

- **Date:** 2026-11-05
- **Status:** accepted

## Context

WP11 asks for scripts that read the CSV, compare it to the §8 targets, and write a plot
report as a CI artifact, including a 3D snapshot of the organ graph. The report must fail
the build when a target is missed. The reference data is gathered in parallel, in
`data/validation/`, using the schema in docs/validation-data.md. Several targets are
known to miss until WP12 tunes the model (decision 0015 consequences), so a gate that
failed on every miss would keep CI red until then.

## Decision

1. **Verdicts in Rust, plots in Python.** `ps-cli validate` (crate `ps-validate`) reads
   the RON data with serde, runs the scenarios, evaluates every check, and writes
   `report.md`, `results.csv` and per-curve CSVs. `validation/report.py` only draws
   figures (responses over time, the day-45 organ graph in 3D, curves against the sim)
   and writes `report.html`. This keeps one parser for the data, gives exact access to
   the state (the organ table, stage dates), and keeps the Python free of any RON
   reader.
2. **Variables** are the `daily.csv` columns plus derived names (`ps_validate::derived`):
   - thermal time to any base (`thermal_time_cd_b<T>`)
   - SLA, root to shoot ratio, mean seed mass, seeds per pod, harvest index
   - leaf area per leaf, height in cm, mean internode length
   - stage dates (`day_of_<Stage>`)

   A value is the mean over the scenario's plants, which act as replicates: one plant's
   pod count moves in steps of one pod (about 7%), so a response scenario with one
   plant mostly measures sampling noise. Each plant has its own random streams (keyed
   by its id), so replicate pots differ the way seeds of one line do. The logistic
   shape check and the curves use the first plant. `at` is Final, Day(n), Max or
   Stage(s). Days count from sowing on day 0.
3. **Checks.**
   - Targets: Range, Approx, or LogisticR2 (the §8.3 shape rule, fitted by grid search
     with closed-form K).
   - Responses: the relative change `(treatment − baseline)/|baseline|`, checked
     against Decrease, Increase, RelChange, FallsMoreThan or Cited. Cited is the §8.3
     rule: same sign as the cited change and within a factor of two of it.
   - Curves are information only, and the report gives their NRMSE.
4. **Gates.** An M0-gated miss fails the build. An Info miss is reported only.
   `validation/known_misses.ron` lists ids that are expected to miss until a named work
   package. These are reported as "known miss" and do not fail the build. A listed id
   that starts passing is reported as "fixed" so it can be removed. WP12 aims to empty
   the list, and M0 exits with it empty.
5. **CI**: the `validation` job in linux.yml runs the harness in release, then the plot
   script (even after a failure), puts the Markdown report in the job summary, and
   uploads `validation-report`.

## Consequences

- A new reference value is a data change only, with no code change, unless it needs a
  new derived variable.
- Python 3.12 and matplotlib are the only extra CI dependencies.
- An M0 check that cannot be evaluated fails the build, and so does a known-miss entry
  that names no M0 check (a typo, or a check since moved to Info).
- Curves report a shape NRMSE beside the absolute one, as field curves are at stand
  density.
- Per-plant targets average over replicates. Stand-level targets (density, M2) will
  need a per-area aggregate.
