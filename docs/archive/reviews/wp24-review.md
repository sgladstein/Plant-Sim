# WP24 review: nitrogen (PR #54, decision 0020)

Reviewed at commit 4563ca6 on `claude/project-thread-ur2udi` against `origin/main` (WP21 merged,
model 0.3.0). Reviewer: Fable review thread. Date: 2026-11-05.

## Verdict

**Approve, with two fixes to fold in** (a one-line bug and a wrong attribution in the decision
record) **and one framing change.** The soil pools, the uptake law, the plant pool and top-up,
mining, demand-driven fixation and nodulation are implemented as described, both ledgers close,
and the done-when holds: nodulated shoot 7.9× unnodulated at 45 days after emergence in
nitrogen-free sand, unnodulated plant N at or below its seed N, and seed-fill leaf drop from
nodes 2 to 5 upward. The model card is complete and every parameter has a unit and a source.

The two fixes:

1. **Filled seeds are mined.** `mine()` excludes `Seed` and `Nodule` but not `SeedInPod`. A seed
   that has finished filling is no longer "growing", so it gives up nitrogen down to the 0.02
   floor to feed other organs. Real seeds do not give nitrogen back, and CROPGRO never mines
   them. Exclude `SeedInPod` from mining (keep `seed_n_final` as the growth floor only).
2. **The shade-internode drop is not nitrogen.** Decision 0020 says internodes fell from +28% to
   +23% because "shaded plants are short of N sooner". I reran the validation with the uptake
   capacity raised 100× (nitrogen cannot limit): shade internodes are +24%. So nitrogen explains
   about one point of the five, and the rest comes from something else in this PR: the new
   capacity nodes (1.0, 0.7, 0.4, 0.1, with up to 4.6% interpolation error on A where aging
   leaves sit), the maintenance changes (seed and cotyledons no longer pay; nothing pays after
   maturity), or the aging × N product. The gate margin is now three points on an M0 exit check
   and its cause is unknown. Bisect it (class nodes first, since that is the cheapest to revert)
   and write the real cause into 0020.

## What I checked

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| clippy, sim and CLI crates, `-D warnings` | clean (CI covers the Bevy crates) |
| `cargo test` on the sim and CLI crates | all pass, including `nitrogen_free_sand_needs_nodules`, `seed_fill_drops_leaves_from_the_bottom_up` and the soil nitrogen tests |
| `scripts/check_deps.py`, `scripts/check_determinism.py` | pass |
| golden hashes (pots 90 d, plot 30 d) | match |
| `ps-cli validate` | 71 checks: 57 pass, 14 Info misses, 0 failing, 0 known; every M0 gate passes |
| debug ledgers | soil N hourly and plant N daily asserts at 1e-9 run in the debug test suite |
| diagnostic: validate with `uptake_capacity` × 100 | 56 pass, 15 miss; numbers in the findings |

## The four points the implementer asked about

### 1. 300 ppm nitrate in `single_pot` and the response scenarios: is it honest?

Yes, if it is named for what it is. The pot holds about 14 kg of soil, so 300 mg kg⁻¹ is
4.2 g of nitrate N against a season's plant uptake of 0.8 to 0.9 g (and about 1.8 g leached by
rain over the season, from the daily CSV). It is a **non-limiting control**, the equivalent of
running CROPGRO with the nitrogen switch off, and it also pushes the nodulation factor to its
0.1 floor so the M0 goldens do not depend on the nodule code. Both are good reasons. The
scenario comments say "fertilized pot"; say "nitrogen non-limiting (about 4 g N, three times a
heavy liquid-feed season)" instead, and add a line to decision 0018's calibrated-vs-predicted
note that the M0 gates are evaluated on non-limiting nitrogen. Then the garden-soil pot
(19.6 g, 0.17 g seeds, 69% Ndfa) is the predicted case and the player's default, which is the
right way round.

One check on the "non-limiting" claim: with the real capacity, the fed pot still logs 2 to
6 mg of unmet demand a day from day 30 to 75, about 10 to 20% of the daily need, because
uptake is bounded by drawn root length × capacity. It does not cost growth: with 100× capacity
the plant ends at 25.0 g, not 28.8 g, so the budget is sink-limited, not nitrogen-limited.
Record that number; it is the evidence the framing rests on. (Why more nitrogen gives less
mass is worth a look of its own: more leaf capacity, more Rd, same sinks.)

### 2. The 4 cm reach and the 10× capacity

Both are honest stand-ins and the card says so. They do not double count: the reach spreads
each root cell's length over the cube and conserves it (`share = L / n`), and the 10× scales
the length. Three things to note:

- The reach is 4 cm in a pot (2 cells) but 5 cm in a plot (`round(0.8) = 1` cell), so the
  cube is 10 cm across in pots and 15 cm in plots. Fine, but say it.
- Water uptake does not use the reach, so the same root draws water from 1 cell and nitrogen
  from 27 or 125. Defensible (nitrate moves to roots by mass flow and diffusion, water does
  not), but a reader will ask.
- **Stranded nitrate is the real consequence.** In `density_35` the plants are at N status 0.53
  on day 40 while 12 g of the soil's 16 g mineral N is still there, and 8 g is left at
  maturity. Roots reach 23 cm at day 45 in a 40 cm plot, so the bottom third is never touched,
  and with no upward flow nitrate never comes back. The density check now passes (−43%)
  largely for this reason: with 100× capacity it is −31%, and light alone gave −25% at WP21.
  So the pass is from top-soil exhaustion, which a field bean would answer with deeper roots.
  The honest note on the density row is "passes with nitrogen; the light-only response is
  about −30%". The WP21 branching point stands.

### 3. Leaf aging kept next to leaf N

Agree with keeping it for now, and the decision record is clear that the plan meant N to
replace it. Two comments. First, the trial that lost "a fifth of the mass" without aging is
not evidence that aging is needed; it says the sink-limited budget punishes extra capacity,
because `Rd` scales with capacity while assimilation is throttled by the reserve feedback.
That is the same signal as the 100× result above. The fix is in the feedback (apply it to `Rd`
too, or not at all), not in keeping two capacity declines. Second, the two now multiply: a
mined old leaf loses capacity twice. Until the feedback is revisited, consider lifting the
aging floor from 0.4 toward 0.7 so the overlap is smaller. Add the §3.8 wording ("stands in
for remobilization until M2") to the plan's amendments list.

### 4. Shade internodes at +23%

See fix 2 above. The attribution is wrong and the cause needs a bisect. If the class nodes are
the cause, two more nodes (1.0, 0.85, 0.7, 0.4, 0.25, 0.1) cost 34 more solves per plant-hour
and would bring the interpolation error back under 3%.

## Other findings

5. **Fixation cost in nitrogen-free sand** is about 40% of the plant's assimilate at day 50
   (39 mg C per day on a 2.4 g plant), against the plan's 10 to 15% for active nodules in
   garden soil. The 30-bean plot's 11% is in band, so this is only the extreme case, but the
   card should give both numbers.
6. **Mining weight by light index** uses the plant's brightest leaf as the reference
   (`light_index / top`), so in a uniformly shaded plant every leaf gets weight 0.1 to 1 by
   its rank. That is the intent (bottom-up order), fine; say in the card that it is relative.
7. **`nodulate` reads nitrate from the drawn root cells**, not the reached ones, while uptake
   uses the reach. Consistent either way; pick one and say so.
8. **Rain leaches a fed pot.** 1.8 g of the 4.2 g left with rain on a pot already at field
   capacity. Real; and it means the fed pots' nitrate is not constant over the season (3.8 g at
   day 20, 1.2 g at day 89). Worth one line in the scenario comment.
9. **`Fertilize` validation** accepts any `g_per_m2 ≥ 0`. A cap (say 50 g m⁻², 500 kg N ha⁻¹)
   would catch a unit slip in a saved action log.
10. **Speed**: 24.6 ms per sim-day on 4 cores against 19.4 on main and 20 target. Hourly
    `top_up_hour` walks every organ for every plant every hour, and `deficits()` allocates a
    Vec each call. Cheap wins: skip the top-up when the pool is zero and no organ is growing,
    and keep the growing list from the allocation pass. The perf thread owns it.

## Science check, briefly

- Soil rates: mineralization 1.5e-4 d⁻¹ at 30 °C gives about 1 mg N per day in the 14 kg pot
  at 20 °C, nitrification turns ammonium over in days. Both in the usual range.
- Uptake law is CROPGRO's NUPTAK shape; the floors and the 0.08 response are the SPE values.
- Fixation: 0.047 g N g⁻¹ d⁻¹ realized at day 50 in sand, at SNACTM. Cost 7.1 g C g⁻¹ N is
  the corrected RFIXN from the research notes.
- Ndfa 33% in the 30-bean plot and 53% at 35 per m², inside Palmero 2022's 30 to 60%.
- Leaf capacity from N per area with a 0.3 g m⁻² intercept is Kattge/Walker; `Na_ref` at the
  species SLA is 1.26 g m⁻², a reasonable bean value.

## For the record

- Model 0.4.0, species schema 2; scenario schema 1 with defaults. Pot goldens regenerated
  (nitrogen changes every day's state, so no byte-identity claim this time).
- Validation at WP24: 71 checks, 57 pass, 14 Info misses, 0 known. Now pass: density (−43%),
  heat pods (−17%), heat seeds per pod (−29%). Now miss: drought shoot mass (−64% vs −13 to
  −52%). Shade internodes +23% (gate 20%).
- Diagnostic run (uptake capacity × 100): 56 pass, 15 miss; single_pot 25.0 g; shade
  internode +24%; density −31%; drought shoot −57%.
