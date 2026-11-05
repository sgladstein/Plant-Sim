# World and schedule

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
The world is the clock and the garden. Every simulated hour it brings the weather, lets
the soil take in rain and watering, lets each plant drink, breathe and grow, and once a
day lets plants make new leaves, buds, flowers and roots. Everything you do (watering,
shading, pruning) is written into a log with the hour it happened, so a garden can be
replayed exactly.

## Equations
- Clock hour of tick `t`: `(start_hour + t) mod 24`; day: `(start_hour + t) div 24`.
- Shade cloth: radiation seen by a soil and its plants `= fraction × radiation`.
- Daily step 0 (decision 0020): for each soil, its plants' joint nitrogen uptake from
  the same soil state, then each plant's nitrogen step, in plant order; then the
  daily plant steps. Each soil's starting nitrogen comes from the scenario's
  `nitrogen` map (default garden soil); the `Fertilize` action adds g N m⁻² of nitrate
  or ammonium to its top layer.
- Supports (WP22, decision 0021): the scenario's `supports` list stands vertical
  stakes or strings in a soil (offset from its center, height, diameter). At sowing
  each plant takes those of its soil into its own frame; vines climb them (model card
  climbing).
- Rain shelter: rain reaching a soil `= rain_fraction × rain` (scenario `rain_fraction`
  per soil, default 1; decision 0018). Irrigation and action water are not affected.
- Plots (decision 0019): plants of one soil step in parallel and are collected in plant
  order. Each day a plot's leaves go into one shared light grid (`ps_light::rebuild_plot`)
  between `Plant::day_before_light` and `Plant::day_after_light`.
- Soil surface radiation: `global × fraction × exp(−0.5 × LAI)`, with LAI the soil's
  plants' leaf area over the soil surface area.
- Watering to field capacity: `mm = Σ_cells max(0, θ_fc V − W) / A_surface` (kg over m²
  is mm).
- Hash: for each serialized word `w_k` (k = 0, 1, …):
  `h ← s(h ⊕ s(w_k + k))`, final `s(h ⊕ n)`, with `s` the splitmix64 finalizer.
  Daily hash `H_d = hash(H_{d−1}, tick, soils, plants, actions and rows new since d−1)`,
  `H_{−1} = hash(model version, scenario, catalog)`.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Irrigation hour | 6 | clock hour | decision 0014 |
| Air pressure | 101.325 | kPa | sea level (no elevation in M0) |
| Pot cell size | 2 | cm | design plan §3.6 |
| Plot cell size | 5 | cm | design plan §3.6 (decision 0019) |
| Plot sides | 100 to 300 | cm | design plan §7 WP21 |
| Plot depth, minimum | 10 | cm | decision 0019 |
| Support height, diameter, most | 500, 100 | cm | input limits (decision 0021) |
| Initial soil temperature | 15 | °C | "~" until the air history fills |
| Soil-surface extinction | 0.5 | - | "~" Beer's law with G = 0.5 |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | scenario | RON | data/scenarios |
| in | species files, weather file | RON, CSV | data/ (kept in the catalog) |
| in | supports (stakes, strings) | cm | the scenario (WP22) |
| in | scenario events, heat wave, shade cloth, rain shelter | tick, kind; °C; - | the scenario's treatments (decisions 0015, 0018) |
| in | actions | tick, kind | player, scripts, replay |
| out | daily rows per plant, with the day's weather and soil water | - | `ps-export` CSV (decision 0015) |
| out | full-state hash per day | u64 | golden tests |
| out | snapshot | RON | saves, the viewer (WP16) |

## Encoded or emergent
Encoded: the step order, the watering rule, shade as a radiation multiplier, and
pruning as a cut whose carbon leaves the plant.
Emergent: plants in one soil compete for water through the joint uptake, and plants on
a plot also compete for light through the shared grid.

## Known limits
- Plants on a plot shade each other. Plants in pots do not, even in the same pot or in
  pots side by side (merging their grids is left for when a scenario needs it).
- Competition on a plot is weak: from flowering on the plants are sink-limited, so
  shade costs little growth. Leaf area per plant falls 25% from 5 to 34 plants per m²,
  against about 70% in the field at model 0.3.0 (decision 0019). With nitrogen (model
  0.4.0, decision 0020) it falls 43%.
- No elevation or wind.
- A plot's light uses the first plant's species G for every plant.
- Snapshots are RON text, about 740 KB for one plant at day 90.
- The scenario is part of the hash: a shorter run of the same scenario has different
  daily hashes from the start.
- Treatments start at fixed ticks, not at a stage.
- Actions are checked when given; a prune scheduled for later can still fail when it
  comes due (the organ died meanwhile), which stops the run. The interactive session
  probes actions on a copy first (decision 0016).

## Validation
- `ps_world::tests` in `crates/ps-world/tests/determinism.rs`:
  `three_runs_give_one_hash` (WP9 done-when 1), `resume_from_day_30_snapshot`
  (done-when 2), `extra_plant_does_not_change_the_first`, `one_thread_matches_many`,
  `replay_of_the_action_log_matches_the_session` (§11.5 checklist),
  `actions_in_the_past_are_rejected`, `shade_cloth_slows_the_shaded_plant`.
- `ps_world::hash::tests::*`: bit-pattern hashing and shape tags.
- `pruning_removes_carbon_and_refuses_the_hypocotyl`, `scenario_events_and_heat_waves_apply`,
  `invalid_start_dates_are_rejected`, `plots_outside_the_size_range_are_rejected`.
- Plots (WP21): `a_plot_is_independent_of_the_thread_count` (1 and 4 threads give one
  hash), `neighbors_in_a_plot_take_each_others_light` (the center plant of
  `plot_30_beans` gets less than 85% of the light it gets alone at day 45).
- Golden daily hashes: `tests/golden/<os>-<arch>/`, checked in CI with
  `ps-cli hash --check-golden tests/golden`, including the first 30 days of
  `plot_30_beans`.

## References
- Vigna, S. (2014). splitmix64 (public domain reference implementation).
