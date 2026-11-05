# WP9 review: world, schedule, action log, snapshots, hash (PR #29)

Reviewed at commit a2cfc55 on branch claude/project-thread-ur2udi, 2026-11-05.
Files read in full: crates/ps-world/src/{lib.rs,hash.rs,scenario.rs}, tests/determinism.rs,
data/scenarios/{single_pot,two_pot_sun_shade}.ron, docs/decisions/0014, docs/models/world-and-schedule.md,
and the diffs to ps-arch, ps-physio, bush_bean.ron, STATUS.md and design-plan.md.

## Checks
All green locally: fmt, clippy -D warnings, cargo test --workspace (171 tests incl. 7 in ps-world,
12.6 s for the determinism suite), check_deps.py (12 crates), check_determinism.py (11 sim crates).

## Done-when
1. Three runs give one hash, every day: `three_runs_give_one_hash` (90 daily hashes, pods >= 5). Met.
2. Resume from a day-30 snapshot matches the day-90 hash: `resume_from_day_30_snapshot` goes through
   RON text, not a clone. Met.
Plus the §11.5 checklist: extra plant in another pot leaves plant 0's hash unchanged, rayon on one
thread matches, a replayed three-action log (water, shade cloth off, prune) matches the live session.

## What is right
- The hash is a serde Serializer over the same impls as the snapshot, with bits for floats and tags
  for Option, sequence length and variants. Anything a snapshot keeps is hashed; nothing is formatted.
- Future actions are excluded from the hash, so replay and live session agree. Good call.
- Scenario matches §6.2 exactly; validation catches duplicate soil ids, unknown references, bad ranges.
- Decision 0014 documents the per-soil hourly order (3,5,6 then 4 then 7 then 8-12) and why.
- WP8 fixes folded in as agreed: primordium built at construction cost with growth respiration in the
  ledger; sink feedback on gross only (transpiration scaled by the same factor); leaf lifespan counted
  from full expansion in both capacity and shedding; total_dry_mass and feedback_loss in DayReport.
- Q10 factor cache keyed by temperature bits is bit-identical to the old path.

## Findings (none blocking)
1. **Prune resorbs carbon from the cut material.** `Plant::prune` calls `allocation::shed`, which
   moves 30% of the subtree's structural carbon into the living ancestor and charges its debt there.
   That is right for senescence (remobilization before abscission) but wrong for a cut: the gardener
   removes the leaf, so its carbon leaves with it. Pruning currently gives the plant free carbon
   (about 0.07 g C per 0.5 g leaf) and makes "prune to boost" an exploit later. Fix: a `cut` path
   (or a resorption argument of 0 on `shed`) where structure and positive reserve go to a `removed`
   ledger term and negative reserve is forgiven, not charged. Same for Harvest when it arrives.
2. **Pruning the collar or hypocotyl silently succeeds.** `prune` returns Ok and does nothing, and the
   action is logged as applied. Return an error so the UI and the log tell the truth.
3. **Plan §3.2 still shows the hourly order 3,4,5,6,7.** Decision 0014 reorders to 3,5,6,4,7 within
   the hour. Add one sentence to §3.2 pointing at the decision, as was done for other amendments.
   No version bump needed since 0.1.0 has not shipped golden hashes.
4. **No model-version warning.** §6.3 says a save from another model version replays with a warning.
   `from_state`, `from_snapshot_ron` and `replay` compare nothing to `MODEL_VERSION` or to
   `scenario.model_version`. One `tracing::warn!` each (ps-world has no tracing dependency yet).
   Fine to land with the CLI in WP10.
5. **The daily hash depends on `duration_days`.** The scenario is hashed, so day 30 of a 90-day run
   hashes differently from day 30 of a 40-day run. Intentional and fine, but write it in the model
   card and in the WP10 golden-file notes, or people will "reproduce" with a shorter run and fail.
6. **Start date is validated late.** `Scenario::validate` checks the hour but month/day only fail in
   `World::new` through `DateHour::new`. Move the check into validate so `from_ron` rejects it.
7. **Small test strengthening.** `extra_plant_does_not_change_the_first` compares the plant hash only;
   add soil 0's grid and ledger hash. `one_thread_matches_many` has little to bite on until ps-light
   or the class table uses rayon; keep it, it is the regression net for WP21.

## Open for WP12 (carried)
feedback_loss about 0.6 from flowering on (sink limitation: no stem thickening, roots about 4% of DM),
Vcmax/Jmax chloroplast basis, planophile G, phloem K, light fixture regeneration.

## Verdict
Approve. Both done-when items are met by tests that resume through text and replay a real log. The one
model fix worth doing now is item 1 (prune should not resorb); the rest are documentation and
validation polish that can ride along with WP10.
