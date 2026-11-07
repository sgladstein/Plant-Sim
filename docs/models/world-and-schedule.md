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
- Root experiments (R2b, decision 0043): `dividers` lists pots with a watertight divider
  across their center (north to south); `Irrigation::ToFieldCapacityStrip` waters only
  the columns whose centers lie between two east offsets, by their own deficit to field
  capacity; `nitrate_patches` multiply the starting nitrate of the cells whose centers
  lie strictly inside a cube (center east, north and deep from the soil's center, edge
  in cm). Before the plants' daily steps the world computes each soil's mean available
  water and nitrate once and hands it to its plants for the root rules.
- Plots (decision 0019): plants of one soil step in parallel and are collected in plant
  order. Each day a plot's leaves go into one shared light grid (`ps_light::rebuild_plot`)
  between `Plant::day_before_light` and `Plant::day_after_light`.
- Water-network level (R3c, decision 0045; plant-hydraulics card): each plant runs
  its water network detailed or simplified, a per-plant state in the snapshot and the
  hash. The scenario's `fidelity` sets the level at sowing: `Auto` (the default) is
  detailed when the scenario sows at most `MAX_DETAILED` (8) seeds at the start and
  simplified above, `Detailed` or `Simplified` forces one, and `Locked(level)` forces
  one and refuses every `SetFidelity` action (a challenge's `fidelity:
  Locked(level)` sets it). The `SetFidelity { plants, level }` action (an empty list
  for every plant) changes the level from the next hour; the viewer writes it as the
  camera and speed change, so a replay of the log is exact. It is never a challenge
  rule break.
- Soil surface radiation: `global × fraction × exp(−0.5 × LAI)`, with LAI the soil's
  plants' leaf area over the soil surface area.
- Watering to field capacity: `mm = max(0, Σ_cells max(0, θ_fc V − W) − P) / A_surface`
  (kg over m² is mm), `P` the water still ponded on the surface (R5a); a strip counts
  its own columns' cells and pond.
- Pot drainage (R5a, decision 0046; soil-and-water card equation 2): a scenario pot's
  `drainage` is `Holes` (the default: water leaves only through saturated soil at the
  base), `NoHoles`, or `Saucer(depth_mm: d)` (0 to 100 mm: holes over a saucer that
  feeds the pot back). Plots always drain freely.
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
| `MAX_DETAILED` | 8 | plants | "~" fidelity plan, for Performance to time (decision 0045) |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | scenario | RON | data/scenarios |
| in | species files, weather file | RON, CSV | data/ (kept in the catalog) |
| in | supports (stakes, strings) | cm | the scenario (WP22) |
| in | scenario events, heat wave, shade cloth, rain shelter | tick, kind; °C; - | the scenario's treatments (decisions 0015, 0018) |
| in | actions | tick, kind | player, scripts, replay |
| out | daily rows per plant, with the day's weather and soil water, and the soil's drained, run-off and evaporated water and leached and mineralized nitrogen since the start (decision 0047) | - | `ps-export` CSV (decision 0015) |
| out | full-state hash per day | u64 | golden tests |
| out | snapshot | RON | saves, the viewer (WP16) |
| out | lens data per hour (not state; WP26) | see below | `World::lens`, `ps_session::Snapshot::lens` |

### Lens data (WP26)
With `World::set_lens_recording(true)` (the session turns it on), each hourly step
also records, beside the state and never in it:
- per plant: living leaf area; the leaf-area-weighted mean absorbed PAR, read from the
  same light coefficients the gas exchange reads; the PAR an unshaded horizontal leaf
  would absorb under the hour's sky after the soil's shade cloth, `(I_b + I_d) x 0.5 x
  4.57 x absorptance`; their ratio (relative light, 0 at night, above 1 for leaves
  facing a low sun); and the plant's share of the PAR its soil's plants absorb
  (area x mean, summed in plant order);
- per organ: the hour's phloem flux from its parent into it (negative for an export),
  copied out of the transport solve; its temperature (a leaf's own from the energy
  balance, WP27; air for the rest of the shoot; the soil layer for roots and the
  seed); and
  its nitrogen per structural dry mass after the hour; and its water potential (the
  pressure part, without gravity) and relative turgor from the hour's network solve
  (R3b, plant-hydraulics card); per plant also the leaves' area-weighted mean turgor
  (how wilted the plant looks) and the root zone's soil potential; and the plant's
  water-network level and each soil layer's share of its uptake in evenly wet soil
  (R3c; at the simplified level the organ potentials are the day summary's
  estimates). Organs added at the
  day's end have no potential until the next hour's solve, so these lists can be
  shorter than the organ arena;
- per soil: each layer's temperature, and each layer's water potential at its mean
  water content (R3a; soil-and-water card). A cell's own potential comes from the grid
  in the state (`SoilGrid::water_potential`), so the lens does not copy every cell's
  each hour.

Recording only copies values out of the step or reads the state after it, so the hash
and the goldens are the same with it on or off (`ps-session/tests/save.rs`). It is never
saved: a loaded or restored world has none until it steps an hour.

A save (`ps_session::Session::save_ron`) is the world's state as RON, taken between two
hours without stopping the session. It holds the timeline's whole log, so a save made
while scrubbed back resumes the recorded future. Loaded and finished, it reaches the
session's hashes, as does a replay of the session's log from the scenario (the WP26
done-when, on the 30-bean plot with every WP25 action).

## Encoded or emergent
Encoded: the step order, the watering rule, shade as a radiation multiplier, and
pruning as a cut whose carbon leaves the plant.
Emergent: plants in one soil compete for water through the joint uptake (each plant's
root network, from the same soil state, R3b), and plants on a plot also compete for
light through the shared grid.

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
