# WP16 (sim side) review: ps-session (PR #32)

Reviewed at commit a2c42d0 on branch claude/project-thread-5e4oap, 2026-11-05.
Read in full: crates/ps-session/src/{lib.rs,snapshot.rs}, tests/session.rs, decision 0016, the
ps-world accessor diff and STATUS.

## Checks
fmt clean. clippy, tests, check_deps and check_determinism: see the note at the end (this container
cannot build the Bevy crates, so I ran them for the sim crates plus ps-session).

## Done-when
`live_session_hash_equals_headless_replay`: step, act, run on the wall clock and act mid-run, pause,
seek back and act there (a branch), step to the end; then `World::replay` of the log gives the same
daily hashes and full-state hash. Met, and it is the right test: it goes through every time command.
`seek_shows_the_same_bits` compares plants and soils at eight targets, including both sides of the
action tick, against a headless replay. Good.

## What is right
- Wall clock paces steps and nothing else; every `Instant` is marked for the lint; actions carry the
  sim tick they were logged at. This is exactly the §11.9 contract.
- Double buffer: snapshot built off-lock, swapped as an Arc, lock held only for the swap or clone.
  History and log are shared slices rebuilt only when they change.
- Probe on a copy before logging, so a rejected action never enters the log (closes WP10 item 4 for
  the UI path).
- Seek = nearest midnight copy + reschedule the rest of the log + step. The keyframe's `applied`
  prefix is truncated and the timeline's log re-added, which keeps the keyframe and the log
  consistent after a branch too. Correct.
- Step while stepping extends the target; TogglePause cancels a step; backlog beyond max_lag is
  dropped rather than raced. Sensible UI behaviour.

## Findings (none blocking)
1. **A branch drops recorded actions at the current tick.** `act` keeps `a.tick < tick`, so an action
   recorded at the very hour the player is standing on disappears when they act there. It has not
   been applied yet, so the state is consistent either way, but the scrubber will show a marker at
   this hour that vanishes. Decide (keep `<= tick` and append, or drop and say so) and write it in
   decision 0016 item 6.
2. **Two clones per action when not branching.** `let mut world = self.world.clone()` then
   `probe = world.clone()`. Probe on one clone and apply to `self.world` directly. At day 90 that is
   two 1 MB copies per click; cheap, but free to remove.
3. **A seek back followed by an action rebuilds again.** After `seek(to)` the world already sits at
   `to`; `act` rebuilds from the keyframe once more only to drop pending log entries. Truncating
   `self.world`'s log to `applied` would do. Perf only.
4. **Probe failures are blamed on the action.** The probe's `step_hour` can fail for reasons that have
   nothing to do with the action (weather series end, a world error). The message reaches the player
   as a rejection. Distinguish `apply` errors (rejection) from `step_hour` errors (world error).
5. **Wall-clock tests.** `running_follows_the_speed_and_pause_stops_it` and the 60 ms sleep in the
   done-when test are tolerant (1..=40 hours), but CI machines stall. If they ever flake, gate them on
   an env var rather than loosening the bounds further.
6. **Memory note for long runs.** 1 MB per midnight copy is fine for M0 (90 days). M4's tree over
   seasons will want `keyframe_every_days` above 1 or copies as compressed RON; the option is already
   there, so just note it in the card when WP26 lands.
7. **Model card.** There is no docs/models card for the session; it is not a process, so none is
   needed, but the decision should say where the UI contract lives (ps-ui TimeCommand mirror) so the
   viewer thread's ps-app side has one reference.

## Verdict
Approve, subject to the checks below finishing green. The design is the one the plan asked for, the
done-when test is strong, and the findings are polish.

## Check note
fmt clean. clippy -D warnings clean, 161 tests passed (0 failed) for ps-session and the eleven sim
crates, check_deps OK (16 crates), check_determinism OK (12 sim crates). The Bevy crates (ps-view,
ps-ui, ps-app) were not built here because this container lacks the Wayland system libraries; CI
covers them. Verdict stands: approve.

## Follow-up (729d484)
Items 1, 2, 4, 6 and 7 landed: branches keep actions at the current tick (new test
`branch_keeps_actions_at_the_current_hour`), one probe clone, rejection messages distinguish the
action from the following hour, decision 0016 items 7 to 9. Item 3 deferred (perf only), item 5 left
until CI shows a flake. Diff checked; the new branch path is correct and the log order within the tick
is preserved.
