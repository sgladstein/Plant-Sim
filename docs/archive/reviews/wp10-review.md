# WP10 review: export and CLI (PR #31)

Reviewed at commit 300269d on branch claude/project-thread-ur2udi, 2026-11-05.
Read in full: crates/ps-cli/src/main.rs, crates/ps-cli/tests/cli.rs, crates/ps-export/src/lib.rs,
the ps-world and ps-arch/ps-physio diffs since WP9 (a2cfc55), the four response scenarios,
decision 0015, docs/cli.md, the golden files and CI changes.

## Checks
All green locally: fmt, clippy -D warnings, 174 tests passed (0 failed), check_deps (14 crates),
check_determinism (13 sim crates).

## Done-when
"The four §8.3 scenarios run from one command and write CSV": `four_response_scenarios_from_one_command`
runs control, drought_from_flowering, heat_at_flowering and shade_50 for 4 days through the binary and
checks the daily, organ and hash CSVs and the log. Met. `snapshot_resume_and_replay_give_the_run_hash`
closes the loop through the CLI. Golden files for linux-x86_64 are committed and CI checks them on Linux
and Windows.

## WP9 items folded in
All seven: `allocation::cut` with a `removed` ledger term and no resorption (debt leaves with the organ,
which is right); pruning the collar or hypocotyl is an error; current-tick actions are checked before
they are logged; §3.2 sentence; tracing warnings on version mismatch; dates validated in `validate`;
the extra-plant test compares soil 0 too.

## Chained daily hash
`day_hash` chains the previous day's hash (day 0: model version, scenario, catalog) with tick, soils,
plants, and the log entries and history rows added since. Everything the old full hash covered is still
covered through the chain, at constant cost per day. The cursors `scripted`, `hashed_log`, `hashed_rows`
are derived state and are rightly left out. Sound.

## Scenario events
Treatments as `Scenario.events`, applied before log actions of the same tick, never entering the log,
with their own cursor: this keeps "scenario + log is the truth" intact and replay cannot double-apply.
Good design. Two small gaps are below (items 2 and 3).

## Findings (none blocking)
1. **Decision number collision.** PR #32 (viewer thread, ps-session) also adds docs/decisions/0015.
   Whoever merges second renumbers; I have told the viewer thread to take 0016 when it rebases on #31.
2. **A missing golden file passes CI.** `hash --check-golden` prints the hashes and exits 0 when the
   platform file is absent. That is how Windows gets bootstrapped, but it also means a deleted or
   renamed Linux file passes silently. Make a missing file an error and add `--allow-missing` for the
   Windows job until its file is committed (then drop the flag).
3. **Events are not checked statically.** `validate` only checks their sort order; an event naming an
   unknown soil or a fraction of 1.5 fails at tick time as a run error. Run the static part of `check`
   (everything but the prune-existence test) over `events` in `validate`.
4. **Two hashes on the console.** `run` and `replay` print `state_hash()` (the full hash) while the
   golden files hold the chained daily hash. A reader will compare them and be confused. Print the last
   daily hash with the full hash, labelled, or drop the full one from the CLI.
5. **Future actions that fail at apply time abort the run.** `schedule` cannot check a Prune's organ in
   advance, so `apply_due` returns an error mid-run. Fine for the CLI; the session layer (PR #32) tries
   actions on a copy first, which covers the UI. Note it in the world card.
6. **Control duplicates single_pot.** Same seed, pot and weather, so the two golden files will always be
   equal apart from the scenario text in the hash seed. Fine; say so in the response README or the
   scenario comment so nobody wonders why control has no golden file.

## The first response numbers (for WP11 and WP12, not for this PR)
- **Heat cuts seed weight 21% against 50 to 71%.** The pod-set temperature factor is evaluated at the
  daily mean (`pod_set_temperature(mean_t)`, CROPGRO quadratic with optimum 22 to 25 and zero at 36).
  A 32/20 heat wave has a mean of about 26, which sits on the plateau. The plan (§3.11) says daytime
  temperature above about 32 drives pollen failure. For WP12: evaluate the heat side of the response on
  the daily maximum (or add a separate pollen-failure term on Tmax), keep the cold side on the mean.
  Gross and Kigel 1994 found both high day and high night temperatures reduce set, so a Tmax term will
  get into the 50 to 71% range with the existing 6-day wave.
- **Shade raises pod count (18 against 16) and cuts root mass sixfold.** Both are the sink limitation
  already on the WP12 list seen from another side. The pod-set carbon factor saturates when the
  parent internode's reserve is above the 0.1 g C/g DM reference, which it is in both control and shade
  because the plant discards 60% of potential gross assimilation anyway. So shade costs no pods, and
  what it costs comes out of the lowest-priority sink, the roots. When WP12 adds sinks (stem thickening,
  a root target fraction) and pod set responds to supply rather than a saturated reserve, both numbers
  should move the right way. Treat "shade reduces pod number" and "root fraction stays above about 10%
  under shade" as two more WP11 checks.
- **Drought cuts seed weight 19%.** Direction right; magnitude to judge once the pod/seed split is in
  the report (§8.3 says pod number falls more than seed size).

## Verdict
Approve. The CLI and export are complete and tested end to end, the chained hash is sound, the events
design is the right one, and all WP9 items landed. Items 1 and 2 are worth doing before WP11 depends on
the golden check; the rest can ride with WP11.
