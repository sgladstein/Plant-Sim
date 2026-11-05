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
- Soil surface radiation: `global × fraction × exp(−0.5 × LAI)`, with LAI the soil's
  plants' leaf area over the soil surface area.
- Watering to field capacity: `mm = Σ_cells max(0, θ_fc V − W) / A_surface` (kg over m²
  is mm).
- Hash: for each serialized word `w_k` (k = 0, 1, …):
  `h ← s(h ⊕ s(w_k + k))`, final `s(h ⊕ n)`, with `s` the splitmix64 finalizer.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Irrigation hour | 6 | clock hour | decision 0014 |
| Air pressure | 101.325 | kPa | sea level (no elevation in M0) |
| Pot cell size | 2 | cm | design plan §3.6 |
| Initial soil temperature | 15 | °C | "~" until the air history fills |
| Soil-surface extinction | 0.5 | - | "~" Beer's law with G = 0.5 |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | scenario | RON | data/scenarios |
| in | species files, weather file | RON, CSV | data/ (kept in the catalog) |
| in | actions | tick, kind | player, scripts, replay |
| out | daily rows per plant | - | CSV export (WP10) |
| out | full-state hash per day | u64 | golden tests |
| out | snapshot | RON | saves, the viewer (WP16) |

## Encoded or emergent
Encoded: the step order, the watering rule, shade as a radiation multiplier.
Emergent: plants in one pot compete for water through the joint uptake; nothing else
couples them in M0.

## Known limits
- Plants in different pots, or in the same pot, do not shade each other yet (M1 merges
  light grids of overlapping pots).
- No elevation, wind or soil nitrogen.
- Snapshots are RON text, a few hundred kilobytes for one plant at day 90.

## Validation
- `ps_world::tests` in `crates/ps-world/tests/determinism.rs`:
  `three_runs_give_one_hash` (WP9 done-when 1), `resume_from_day_30_snapshot`
  (done-when 2), `extra_plant_does_not_change_the_first`, `one_thread_matches_many`,
  `replay_of_the_action_log_matches_the_session` (§11.5 checklist),
  `actions_in_the_past_are_rejected`, `shade_cloth_slows_the_shaded_plant`.
- `ps_world::hash::tests::*`: bit-pattern hashing and shape tags.

## References
- Vigna, S. (2014). splitmix64 (public domain reference implementation).
