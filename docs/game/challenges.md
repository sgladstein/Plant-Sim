# Challenges and the run debrief (WP32, sim side)

The owner chose this as the first game slice on 2026-11-06: a few scored challenges
and a debrief that says what held each plant back. Neither changes a model result:
a challenge is a scenario, rules on the action log and a metric read from the daily
table, and the debrief reads that same table against the species' own thresholds.

Crate: `ps-game` (headless; depends on ps-world, ps-species, ps-physio, ps-units).
CLI: `ps-cli challenge ID [LOG] [--out DIR]`.

## Challenge files

`data/challenges/<id>.ron`, schema 1:

| Field | Meaning |
| --- | --- |
| `scenario` | path under `data/scenarios/` without `.ron`; it pins the model version and world seed |
| `score` | `PeakSum(column)` (each plant's highest daily value, summed: harvesting pods does not lower a pod count) or `FinalSum(column)`; a label; the direction |
| `rules` | allowed action types, and optional water budget (kg = L, by can), nitrogen budget (g N), sowing window (scenario days) and most sowings |
| `medals` | bronze, silver and gold thresholds in score units |
| `seasons` | seasons scored (default 1): the log is replayed on this many world seeds and the medal goes to the mean |

A score is recomputed from the replay (`ps_game::play`: scenario plus log, run to the
last day, then `Challenge::judge`), so anyone with the log gets the same number. A run
with a broken rule, or scored before the last day, gets its score but no medal. A run
on a model version other than the scenario's pin is flagged.

### Seasons

One season is one draw: each flower's set is a chance, and the weather is generated from
the world seed, so a single run can swing a pod count by several pods. On 0.25.0 (R3b)
thirsty summer on seed 7 gave 2 pods for watering daily in bloom and gold for watering
every 4 days, the reverse of the lesson. So each shipped challenge is scored over
8 seasons: the log replays on the scenario's world seed and the next seven up
(`ps_game::season_scenario`), and the score is the mean of the eight. Actions on one
particular organ (harvest, prune, pollinate) are dropped from seasons 2 and on
(`ps_game::season_log`), since another seed grows other organs; the plan (sowing,
watering, fertilizer) carries over, so a harvest counts in one season of the eight; the
result card says so. The first season is the scenario as written, and it
is the one the viewer shows and the debrief explains. The other seven run on threads,
about 2 s in a release build.

## The first three challenges

Medal thresholds come from strategy sweeps of the 8-season mean (world seeds 7 to 14),
on model 0.25.0 (R3b) and again on 0.26.0 (R3c, decision 0045), where thirsty summer's
medals rose a pod or more (5 / 7 / 8.5, from 4 / 6 / 7.5) and the others held.
Single-seed sweeps on 0.23.0 to 0.25.0 are in the git history of this file; the
review's eight-seed sweep is in the project files (reviews/wp32-challenges-137-review.md
and reviews/wp32-seasons-143-review.md). Even the mean of 8 moves from one strategy to
its neighbour (by about a pod in the heat wave, whose seasons spread about 3 pods), so
thresholds sit where a sound strategy lands, with gold just under the best plan found.
Every challenge locks the water network at its detailed level (`fidelity:
Some(Locked(Detailed))`, decision 0045), so the viewer's level switches never change a
score.

| Challenge | Setting | Player controls | Sweep, mean of 8 seasons (0.26.0) | Medals (B / S / G) |
| --- | --- | --- | --- | --- |
| Thirsty summer | one pot under a rain shelter, no irrigation, starts at field capacity | the can, 6 L for the season | no water 2.1 pods; 6 L spread daily over the season 2.6; from day 35 or day 40, every 1 to 7 days, 6.4 to 8.5 (daily from day 40 7.5; every 4 days from day 40, the fixture, 8.4); days 42 to 70 every 2 to 5 days 7.9 to 9.5 (every 2 days 9.0, every 5 days 9.5) | 5 / 7 / 8.5 pods (0.25.0: 4 / 6 / 7.5) |
| Dodge the heat wave | heat wave days 55 to 61 (37 / 24 °C), watered daily | sowing day 0 to 40, one seed | sown days 0 to 8 (blooms into the heat) 2.9 to 4.9 pods; days 12 and 16 5.3 and 6.5; days 20 to 26 7.8 to 9.0; days 28 to 40 8.4 to 11.1 (day 28, the fixture, 10.8; day 32 11.1) | 5 / 7.5 / 9.5 pods, unchanged |
| Hungry soil | 5 ppm nitrate, no rhizobia, watered daily | 0.5 g N of fertilizer | none 0.7 g seed; all on day 45 3.4; all on day 35 5.6; all at sowing 6.3; one dose on day 15, 20 or 25 7.5, 8.1 and 7.1; two or three doses from day 10 to 45 7.0 to 7.6 (days 15, 30 and 45, the fixture, 7.2; days 15, 25 and 35 7.6) | 5 / 7 / 7.9 g seed, unchanged |

On the mean the lessons hold: water in bloom beats water spread thin by about five pods,
sowing after day 25 beats blooming into the heat by six, and feeding before and through
early bloom beats feeding late by four grams.

On 0.25.0 daily watering in bloom earned only bronze (5.4 pods for days 40 to 75): pod
abortion still reads the root zone's lowest FTSW of the day, which under any bloom
schedule sits at 0.25 to 0.45 most afternoons against the 0.5 threshold (decision 0044),
so larger doses every 3 or 4 days did better. On 0.26.0 most bloom schedules gain one to
two pods (daily from day 40 5.4 to 7.5, every 2 days from day 42 6.9 to 9.0, the
fixture 7.3 to 8.4), and larger, less frequent doses still lead.

The tests in `crates/ps-game/tests/challenges.rs` replay a good and a poor strategy for
each challenge (tests/fixtures/challenges) and check the good one beats the poor one and
earns a medal, and that every challenge scenario pins the current model. When a model
bump moves these numbers, re-run the sweeps and reset the medals here and in
`data/challenges/`.

## The debrief

`ps_game::debrief(state, refs)` splits each plant's season into weeks from emergence.
Each week gets a 0-to-1 pressure per factor, averaged over its days:

| Factor | Daily pressure | Source of the threshold |
| --- | --- | --- |
| Water | `1 − predawn leaf turgor` (R3b: the stress the plant carries through the night; a midday dip in wet soil is grown back that night), at least `shortfall / (transpiration + shortfall)` (demand the soil could not meet); while any flower is open, at least `1 − FTSW / drought_abortion_ftsw` (end-of-day FTSW; the model uses the day's minimum) | the model's water network; species `turgor_loss_point`, `turgor_full`, `drought_abortion_ftsw` |
| Heat, cool weather | while any flower is open, one minus the model's pod-set response (quadratic ramps) at the day's peak; otherwise one minus the growth trapezoid at the day's mean | species `pod_set_t_*`, `growth_t_*` |
| Light | one minus radiation at the soil over a full-sun reference, times one minus the sink-feedback loss | `data/game/debrief.ron` (guess, 25 MJ m⁻² d⁻¹) |
| Nitrogen | one minus the nitrogen factor on expansion | the model's `n_status` |
| Sinks | the sink-feedback loss (decision 0013) | the model's `feedback_loss` |

Reading rules:

- A week's top factor is the worst of the five environmental factors above the notable
  pressure (0.15, a guess in `data/game/debrief.ron`). Sinks are a symptom (a plant that
  lost its pods to heat has nowhere to put sugar), so they stay in the weekly numbers
  and never top a week or reach the summary.
- The debrief runs from emergence to maturity. A day is in bloom when a flower opened
  within the last flower lifespan (3 days for the bush bean; the daily `flowers` column
  counts flowers opened so far), which runs well into seed fill.
- The season ranks factors that hit in a bloom week first, by their bloom-week peak
  (the number quoted, with only the bloom weeks named), then the rest by their worst
  week, since one bad week in bloom can cost the season's pods. A factor that hit in
  bloom is never called backdrop. A factor notable in 60% or more of the weeks is reported as the season's
  backdrop ("All season, mildly: cool weather and light"), not as an event.
- "During bloom" is added when the quoted spans are bloom weeks.

Known limits: outside bloom the temperature pressure uses the daily mean, while the model steps
hourly; light ignores shade from neighbours; the debrief names pressures, it does not
count the pods each one cost; weekly means dilute a short heat wave that straddles two
weeks. On `response/heat_at_flowering` heat leads the summary for 7 of 8 plants; on
`response/drought_from_flowering` water leads for all 8. A frost kill is not reported
yet.

## Viewer

- **Challenges…** in the top bar opens the list (best score per challenge), the brief,
  rules, medals and hint, and a plan step for what the rules let the player decide ahead:
  the sowing day (a slider over the window) and up to three nitrate doses within the
  nitrogen budget. Start schedules the plan on the world before the run, so it sits in
  the action log and the replay scores it.
- While the run goes, a panel above the lenses shows the score so far, the next medal,
  budgets left and any broken rule. It updates once a sim day. Watering uses the
  ordinary Water tool; the panel converts the pot's mm to litres.
- At the last day the **Season debrief** card shows the season on screen while the
  other seven seasons score in the background ("scoring your plan over 8 seasons…"),
  then the mean, each season's score, and the medal. The best is kept only then. The
  card has a week strip per
  plant coloured by that week's worst event (season-long backdrop factors stay grey;
  hover a week for every number) and the summary sentences. Try again replays the
  challenge with the same plan.
- The best ranked run of each challenge, with its action log, is kept in
  `<captures>/challenge_scores.ron`.
- Flags: `--challenge ID [--sow-day N] [--day D]` starts one (and opens the debrief
  when `--day` is the last day); `--challenges` opens the list.

Not done yet: tools are not greyed out when a challenge forbids them (a forbidden
action shows as a broken rule and costs the medal), and the fourth challenge.
