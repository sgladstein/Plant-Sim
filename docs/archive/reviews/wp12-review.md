# WP12 review: M0 tuning and lock (PR #43)

Reviewed at commit 557cfea on branch claude/project-thread-ur2udi, 2026-11-05.
Read in full: decision 0018; the ps-arch diffs (lib.rs, params.rs, shoot.rs); the species file,
scenario, world, ps-units and schema changes; the light re-check test and fixture script; the
herb-architecture, phenology and world cards; STATUS; targets.ron and known_misses.ron.

## Checks
fmt and clippy -D warnings clean; 171 tests passed (0 failed) across the sim and CLI crates;
check_deps OK (19 crates); check_determinism OK (15 sim crates). `ps-cli validate` and the golden hash
check: see the end of the file. The Bevy crates were not built here (container limit; CI covers them).

## Done-when
Plan §7 M0 exit list against decision 0018 and the harness: all §8.2 ranges, the logistic R² (now a
gated target), the shade, drought, heat and frost rules, 1e-9 ledgers (debug asserts through every
test), one hash across runs with Linux and Windows files committed, under 2 ms per sim-day (perf thread).
Every M0 gate passes at seeds 7, 11, 23 and 31, which answers the WP11 caution. Met, subject to item 1.

## Findings

### 1. Shaded internodes lose half their mass; the decision and the card say they keep it. (fix before lock)
`internode()` in shoot.rs sets `specific = stem_specific_length × e²` and
`potential = length / specific` with `length = base × e`, so the potential mass is `base/(SL × e)`:
divided by the elongation factor, half at full shade. Decision 0018 item "Low-light shade response",
the code comment and the herb-architecture card all say "its specific length rises by the same factor,
so its potential mass stays the same", which is `specific = SL × e`. One of the two is wrong. The
physiology in the card (longer and thinner at the same mass) is the better one: under shade bean
shifts allocation toward stems (Lopes 1987 stem fraction rises), and a rule that makes shaded stems
half-price pushes the stem fraction the other way and buys the −46% shade mass partly with free
elongation. Fix the code to `SL × e`, re-run the four seeds, regenerate the goldens. If the e² form is
what you want, say so and why in the decision and the card, and expect the shade mass gate to move.
Either way the two must agree before 0.2.0 is locked.

### 2. Heat is counted twice on ordinary warm days. (note, not blocking)
`H = mean F_T(Tmax) over 7 pre-anthesis days × mean F_T(Tmax) over open days`. With the CROPGRO
ramp (optimum to 25, zero at 36) a normal 28 to 30 °C summer afternoon gives F_T 0.8 to 0.93, so the
product takes 20 to 35% off pod set in the control every day, which pod_set_base then absorbed (0.45 to
0.42). A single mean over the whole window (pre-anthesis plus open days) keeps the pollen-development
window and does not square the ordinary-day penalty. Moving to Tmax is right (CROPGRO's own function
runs on daytime temperature); the product form is the only quibble.

### 3. The light model card was not updated. (fix in this PR)
docs/models/light.md still carries the WP6 synthetic-shoot result. The sim-plant numbers (G 0.5:
+10% total, 48 of 74 within 0.10, worst 0.31; G 0.65: −1.5%, 58 of 74, worst 0.22) belong in its
Validation and Known limits, with the 19° tilt explanation. CLAUDE.md: a PR that changes a model's
measured accuracy updates the card.

### 4. The WP8 pod range was loosened. (note)
`counts_hold_across_world_seeds` went from pods 10..=20 (the §3.16 bean range used as the WP8
done-when) to 6..=20, because seeds 1 to 8 now give 8 to 17. The §8.2 range is 6 to 20, so the test
is consistent with the data, but the exit review should know a done-when bound moved in the tuning
PR and that pods fell from 16 to 11 at seed 7.

### 5. Small things
- The 4-seed robustness check is manual. Add an ignored test or a weekly CI job that runs the gated
  set at seeds 11, 23 and 31, so the claim in 0018 stays true as the model moves.
- Drought and heat start at a hard-coded tick 1032; 0018 notes this. A stage-triggered event would
  remove the coupling, but that is M1 work.
- Frost kills leaves, apices, buds and flowers; pods and roots survive. Fine for "no recovery", but
  say so in the card.
- `sim_plant_against_ray_tracer` pins a known 5 to 15% error. Good as a regression net; the comment
  already says it does not claim the target is met. Keep it that way.

## The light re-check (decision 0008 follow-up)
The sim draws every leaflet at about 19° from horizontal and the species G of 0.5 assumes spherical
angles, so the turbid model under-shades by about 10% on the plant total and only 65% of leaflets are
within 0.10. Three things to tell the owner:
- The 10% was absorbed by the WP12 calibration, so any fix re-opens the tuning. That is a fair reason
  to leave it for WP21 (M2), as 0018 proposes.
- Of the two fixes, make G follow the drawn leaf normals per direction (area-weighted mean of |n·d|
  per plant or per voxel). It makes the light model consistent with whatever geometry is drawn, so leaf
  angles can later be set for looks or for heliotropism without breaking the physics. Drawing a spread
  of angles to match G = 0.5 fits the plant to the model, which is backwards, although a spread of
  angles would also look more natural.
- The per-leaf error (35% of leaflets off by more than 0.10, worst 0.31 at G 0.5) is a real limit of
  one voxel grid on a dense single plant, not only a G problem: at G 0.65 still 22% miss. This feeds
  the "lower leaves die in shade" rule. It should be an explicit M0 known limit, not only an owner
  question.

## Are the Info misses honest known limits?
Yes, with three additions. The list in 0018 (root:shoot 0.04, sink limitation and no N
remobilization, heat on seed set but not size, drought not shrinking seeds, shade height +34% not
+99%, shade roots −14% not −30%, thermal-time papers disagreeing) states each miss with its number
and a cause, which is what an exit review needs. Add:
1. **Which gates test a calibration and which a prediction.** shade_internode and shade_sla are
   Encoded: their factors (2.0, 1.4) and the DLI onset and full values were set from Lopes 1987 and
   Cossu 2021, the same papers that gate them. They prove the encoding works, not the model. The
   predictions are the Emergent ones: shade total mass −46%, drought pods −53%, drought root:shoot
   +20%, heat seeds −18%, the logistic R² 0.996, and the §8.2 ranges. Say this in the exit summary.
2. **Drought root:shoot passes on a base that is 3 to 5 times too low** (0.04 to 0.048). The sign is
   right; the magnitude means nothing until roots are fixed.
3. **The light model's per-leaf accuracy on a real plant** (above).

## Verdict
Approve once item 1 is resolved (code or documentation, with the four seeds re-run and goldens
regenerated if the code changes) and item 3 is done. Items 2, 4 and 5 can ride into M1. The M0 exit
summary for the owner should carry the three additions above alongside 0018's list.

## Check results
`ps-cli validate` on this branch: 70 checks, 53 pass, 16 Info misses, 0 failing the build, 0 known
misses, 1 not evaluated (M2 density), exit 0. Golden hashes: single_pot and two_pot_sun_shade match
tests/golden/linux-x86_64. All as decision 0018 states.

## Follow-up (de854e9): approved
Item 1 fixed: specific length rises by e, so a shaded internode keeps its potential mass (checked in
the diff). Item 3: heat is now `H = min(mean pre-anthesis F_T, mean open-day F_T)`, the worse window
decides and an ordinary warm day counts once; a sound choice (a single mean made the 32 °C wave read
+1%). Items 2 and 5: light.md carries the sim-plant numbers and the per-leaf error as an M0 known limit;
the card says frost spares pods and roots; the seed-sweep job is a STATUS follow-up. All four seeds
still pass every gate (pods 10 to 13, shade mass −37 to −49%, shade internodes +28 to +34%, heat seeds
−8 to −31%); goldens and the plant60 fixture regenerated; 0018 has the calibrated-versus-predicted
note. Verdict: approve; merge when CI is green.
