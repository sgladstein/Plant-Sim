# 0024: Harvest, obstacles, supports mid-run, weather files, and the apical hold (WP25)

- **Date:** 2026-11-05
- **Status:** accepted. The owner chose to strengthen apical dominance (2026-11-05). Setting `apical_hold_nodes` to 0 and `apical_dominance_nodes` back to 2 reproduces model 0.7.0's bean exactly (daily and organ tables byte-identical).

## Context
WP25 (design plan §7, M2) adds the prune and harvest actions, obstacles, weather file
import and latitude and season controls. It is done when "pruning releases branching
visibly". Prune existed since M0. The UI parts (the tools, the weather import dialog,
the latitude and season controls) belong to the viewer; this record covers the sim.

Topping a bean in model 0.7.0 released nothing that its untopped twin was not about to
grow anyway. Apical dominance decayed over 2 nodes from the apex
(`1 − exp(−(n − r)/2)`), so a bud two nodes below a growing apex already broke at about
a third of the full rate, and both twins had grown the same buds within days.

## Decision
1. **Harvest** (`ActionKind::Harvest { plant, organ }`): picks a pod or ear with what it
   carries, as a prune does (the material leaves the plant through the carbon and
   nitrogen ledgers). Its seed count and seed dry mass go to the world's seed library
   (`WorldState::seed_library`, one `SeedLot` per harvest: day, plant, species, organ,
   seeds, mass, and whether the plant was mature). Anything but a pod or an ear is
   refused.
2. **Obstacles** (`Scenario::obstacles` and `ActionKind::PlaceObstacle`): axis-aligned
   boxes per soil, corners in cm from the soil's center and surface, with a
   transmittance through the thinnest side. The light grid spans their corners and
   gains their optical depth in proportion to cell overlap (docs/models/light.md).
   They shade pots and plots alike and are grey, so they do not change R:FR. Every
   plant in the soil carries them, including plants sown later.
3. **Supports mid-run** (`ActionKind::AddSupport`): a stake, pole or string added to a
   soil joins every plant there and every later sowing. Vines capture it as they
   capture scenario supports (decision 0021). This is the build-a-trellis action.
4. **Weather files**: the hourly and daily readers existed since WP2. A synthetic
   hourly file (data/weather/sample_hourly_45n.csv, from
   scripts/make_sample_weather.py) and m2/single_pot_weather_file prove a file drives
   a full run. Latitude and sowing date were already scenario inputs, so the season
   and latitude controls are UI.
5. **Apical hold** (species schema 5, constant `apical_hold_nodes`): the buds of the top
   `h` nodes below a growing main-stem apex stay dormant, and dominance decays over
   `apical_dominance_nodes` below that:
   `1 − exp(−max(0, n − r − h)/d)`. A cut apex counts as terminated at once. The beans
   use `h = 1.5`, `d = 0.7` (both "~"). A small sweep chose them: `h = 2` with `d` of
   0.5, 0.7 or 1 set too few pods on the standalone 90-day bean or pushed the unstaked
   pole row's yield loss past −70% (WP22's done-when); `h = 1.5, d = 0.7` keeps both and
   the 20-seed pod sweep. This is the classic bean decapitation response
   (Cline 1991 [K, to verify]): the buds of the nodes under the apex grow out within
   days of its removal.
6. **Hashes.** The new state fields are skipped when empty, so a run without
   obstacles, added supports or harvests serializes as before. The bean's numbers move
   because of the hold. Model 0.7.0 → 0.8.0, species schema 4 → 5, goldens
   regenerated.

## Consequences
- Done-when: four days after topping at 4 nodes (the bush bean on day 30, the pole bean
  on day 22), every bud below the cut has grown a branch, while the untopped twin still
  holds some of them dormant (`ps-validate/tests/actions.rs`).
- A topped plant ends with fewer branches than an untopped one: each released bud makes
  one 3-node branch with no buds of its own, and the nodes above the cut are gone.
  Branches on branches, and a branch taking over as leader after topping, are
  follow-ups.
- The bush bean keeps 6 branches, but grows bigger: 32.8 g at maturity instead of
  25.0, 18 pods instead of 12 and 15.9 g of seed instead of 10.6, all still inside their
  bands. The cause is not traced; later branches come out higher, in better light, and
  early low branches no longer compete with the main stem's leaves. Every M0 check
  passes. Validation goes from 67 to 66 passes of 87:
  - `nar_early` now misses (6.9);
  - `heat_seeds_per_pod` now misses (−7%);
  - `drought_shoot_mass` now passes (−45%);
  - `pole_unstaked_seed_yield` stays in band at −67% (−34% before), near its edge.
- The viewer needs tools for harvest, obstacles and supports, a seed library view, and
  the weather file dialog. The sim side is the actions above.
