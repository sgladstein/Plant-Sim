# WP11 harness review: ps-validate and `ps-cli validate` (PR #39)

Reviewed at commit 568cf30 on branch claude/project-thread-ur2udi, 2026-11-05.
Read in full: crates/ps-validate/src/{lib.rs,files.rs,derived.rs,logistic.rs}, the ps-cli validate
command and its test, the ps-export daily_number addition, validation/report.py, known_misses.ron,
decision 0017, validation/README.md, the CI job.

## Checks
fmt and clippy -D warnings clean for the sim and CLI crates (the Bevy crates cannot build in this
container; CI covers them). Tests, check_deps and check_determinism: see the end of the file.

## Done-when
"The report runs in CI and fails the build when a target is missed": the `validation` CI job runs
`ps-cli validate` in release, plots, puts report.md in the job summary and uploads the artifact; the
test `validate_fails_on_a_missed_target` drives the binary with a fixture that must miss and checks the
exit code, the stderr line and the report, then shows a listed known miss passing. Met.

## What is right
- Verdicts in Rust from the same serde types, plots in Python: one parser, exact state access.
- Every derived variable in docs/validation-data.md is implemented as written (thermal time from day 0
  with (Tmin+Tmax)/2, NAR over the 8-row window, shares over 3 days, HI_tops, mean internode over
  living Stem roles at Final). `At::Stage(s)` uses "first row at or past s", which is more robust than
  the doc's "is s" and gives the same answer.
- Cited rule = same sign and within a factor of two, as §8.3 says. FallsMoreThan requires both a fall
  and a larger fall. RelChange is inclusive.
- Known-miss list with "fixed" reporting is the right way to keep CI green through WP12 without
  hiding regressions.
- Replicate plants averaged per scenario (decision 0017 item 2) is the right answer to the 7% pod
  quantum; it also makes the WP12 8-pot scenarios meaningful.

## Findings (none blocking)
1. **The M0 exit logistic check is not in the data.** The plan's exit list says "biomass curve logistic
   with R² above 0.95". The harness has `Expect::LogisticR2` and the fit (grid search, closed-form K,
   recovers a known curve) but targets.ron has no target that uses it, so the exit criterion is not
   checked. Add `biomass_logistic` on `total_dry_mass_g` with `LogisticR2(min_r2: 0.95)`, gate M0.
2. **Schema drift from the contract.** `LogisticR2`, `Curve.scenario`, the `day_of_<Stage>` variables,
   `height_cm`, `leaf_area_cm2` and `harvest_index` exist in the harness but not in
   docs/validation-data.md, which the data file calls the contract and says any change bumps
   `schema`. Update the doc in this PR (no bump needed for additions, say so in the doc).
3. **One missing replicate voids a target.** `value()` returns Err if any plant lacks the value (a
   drought pot that never reaches Mature, a pot with zero pods for seeds_per_pod). With 8 replicate
   pots under stress that will happen. Average over the plants that have the value, require at least
   half of them, and put the count in the note.
4. **Unknown ids in known_misses.ron are silent.** A typo in an id means the miss it was meant to cover
   fails the build while the entry never fires; an entry for a deleted target lingers forever. Report
   "known miss id X matches no check" and treat it as a build failure.
5. **known_misses.ron is already stale.** maturity_thermal_time_b10 and heat_pods became Info in the
   data thread's PR #36, so the maturity entry is now covering an Info check. Harmless; drop it when
   #36 is in.
6. **Curves at stand density.** DSSAT per-plant curves come from 25-52 plants per m²; the lone pot plant
   will be 2-4x larger, so their NRMSE will read 100%+ forever. Either plot them normalised to their
   own maximum (shape and timing, as the data doc says they are for) or label the NRMSE column
   "shape only". Pinzón and Maldonado are per plant and can stay absolute.
7. **Small things.** `interpolate` assumes x ascending; thermal-time x is non-decreasing and the
   search guarantees x1 > x0, so it is safe, but a one-line debug_assert would document it. The CI job
   installs matplotlib directly instead of `pip install -r validation/requirements.txt`; use the file so
   the two cannot drift. `report.py` reads `plant == "0"` for the organ plot; fine for M0, but name the
   plant in the title.

## On the WP12 state you described
Flowering 440 and maturity 930 b8 from germination give days 43 and 86: inside the gates, and the
flowering sum now sits mid-spread of the four sources. Rain shelter, PAR-driven SLA and internode
response, Tmax plus a pre-anthesis pollen window and heat on seeds per pod all match what the data
asked for. Two cautions before you lock:
- Shade mass -33% at seed 7 but -24% and -29% at seeds 11 and 31 means the gate is not robustly met;
  the WP11 harness only runs seed 7. Run the gated set at three seeds in WP12 (a `--seed` option on
  validate, or three scenario copies) and require all three to pass before golden hashes are
  recorded, or the lock will be to a lucky seed.
- Root:shoot 0.035 against the new 0.08-0.30 Info target: this is the one remaining structural gap
  (no stem thickening, roots last in priority). If it is not fixed in WP12, write it into the M0 exit
  review as a known limit rather than leaving it Info.

## Verdict
Approve. Item 1 should land before the M0 exit review, since without it the exit list is not fully
checked; items 2 to 4 are small and belong in this PR or the WP12 one.

## Check results
166 tests passed (0 failed) across ps-validate, ps-cli, ps-export and the sim crates; check_deps OK
(19 crates); check_determinism OK (15 sim crates). End-to-end `ps-cli validate` on this branch: 69
checks, 44 pass, 20 miss (0 failing the build), 4 known miss, 1 not evaluated, exit 0. The 20 Info
misses are the ones the data review predicted (flowering thermal sums, heat seed mass, shade pods,
roots, height, SLA, root to shoot).
