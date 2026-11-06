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

A score is recomputed from the replay (`ps_game::play`: scenario plus log, run to the
last day, then `Challenge::judge`), so anyone with the log gets the same number. A run
with a broken rule, or scored before the last day, gets its score but no medal. A run
on a model version other than the scenario's pin is flagged.

## The first three challenges

Medal thresholds come from strategy sweeps on world seed 7: on model 0.23.0 (re-swept
after the PR #137 review), checked again on 0.24.0 (R2b, decision 0043), where only
hungry soil's medals moved. Pod counts are noisy from one strategy to the next (each
flower's set is a draw), so thresholds sit where a sound strategy usually lands, not at
the best single draw. The review's eight-seed sweep is in the project files
(reviews/wp32-challenges-137-review.md).

| Challenge | Setting | Player controls | Sweep (seed 7) | Medals (B / S / G) |
| --- | --- | --- | --- | --- |
| Thirsty summer | one pot under a rain shelter, no irrigation, starts at field capacity | the can, 6 L for the season | 0.23.0: 6 L spread over the whole season 2 to 5 pods; the same 6 L in bloom (days 40 to 75, every 1 to 4 days) 9 to 14. 0.24.0: no water 4, spread daily 1, bloom every 1 to 4 days 6 to 10 | 5 / 8 / 11 pods |
| Dodge the heat wave | heat wave days 55 to 61 (37 / 24 °C), watered daily | sowing day 0 to 40, one seed | 0.23.0: sown days 0 to 20 (blooms into the heat) 1 to 8 pods; days 22 to 40 6 to 18, mostly 11 to 18. 0.24.0: days 0 to 16 2 to 7; days 20 to 36 10 to 15; day 40 6 | 9 / 12 / 15 pods |
| Hungry soil | 5 ppm nitrate, no rhizobia, watered daily | 0.5 g N of fertilizer | 0.23.0: none 0.5 g seed; all at sowing 4.0; all on day 45 2.1; split or mid-season (days 25 to 35, or several doses from day 15 to 55) 4.9 to 7.4. 0.24.0: none 0.8; all on day 45 4.9; days 20 and 35 6.7; days 15, 30 and 45 7.3; all at sowing 8.0; days 25 and 40 8.2 | 5.5 / 7 / 8.5 g seed (0.23.0: 4.5 / 6 / 7) |

On 0.24.0 hungry soil pays more for every feeding (water now carries nitrate to the
roots, and roots proliferate where it lands), and on world seed 7 feeding all at sowing
does as well as splitting. Over world seeds 7 to 14 it does not: all at sowing averages
6.6 g (2.3 to 8.8), split or mid-season 7.3 to 7.6 g (5.8 to 9.6), all on day 45 3.2 g,
none 0.7 g. So the hint stands.

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
| Water | `shortfall / (transpiration + shortfall)`: demand the roots could not meet; while any flower is open, at least `1 − FTSW / drought_abortion_ftsw` (end-of-day FTSW; the model uses the day's minimum) | the model's own uptake; species `drought_abortion_ftsw` |
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
- At the last day the **Season debrief** card shows the score, medal, a week strip per
  plant coloured by that week's worst event (season-long backdrop factors stay grey;
  hover a week for every number) and the summary sentences. Try again replays the
  challenge with the same plan.
- The best ranked run of each challenge, with its action log, is kept in
  `<captures>/challenge_scores.ron`.
- Flags: `--challenge ID [--sow-day N] [--day D]` starts one (and opens the debrief
  when `--day` is the last day); `--challenges` opens the list.

Not done yet: tools are not greyed out when a challenge forbids them (a forbidden
action shows as a broken rule and costs the medal), and the fourth challenge.
