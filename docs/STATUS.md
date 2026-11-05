# Status

Read this first at the start of every session (CLAUDE.md).

## Milestone
M0: headless core. Backlog: docs/design-plan.md §7; one GitHub issue per WP (#1 = WP0 ... #13 = WP12).

| WP | Issue | State |
| --- | --- | --- |
| 0 Repo scaffold | #1 | done (PR #14) |
| 1 Units and RNG | #2 | done (PR #14) |
| 2 Weather and sun | #3 | done (PR #14) |
| 3 Organ graph | #4 | done (PR #14) |
| 4 Species schema | #5 | done (PR #15) |
| 5 Soil and water | #6 | done (PR #16) |
| 6 Light | #7 | done (this PR); owner accepted 95% of leaves within 10% for M0, re-check in WP12 (decision 0008) |
| 7 Physiology | #8 | done (this PR) |
| 8 Herb architecture and roots | #9 | next |
| 9 World | #10 | not started |
| 10 Export and CLI | #11 | not started |
| 11 Validation harness | #12 | not started |
| 12 Tuning and lock | #13 | not started |

## Last validation report
None yet (WP11).

## Open questions for the owner
- WP7 (decision 0009): two readings of the plan's text to confirm. Priority is the share of an organ's own available reserve it may spend per hour (taken literally, §3.9 would cap every organ below its potential), and the water stress factor multiplies the stomatal slope.
- WP7: the CI job `plantecophys` proves the gas-exchange fixture against plantecophys in R; the fixture was generated without R.
- Set `main` as the repository's default branch in GitHub settings (the agent cannot change repo settings).
- WP13 (Bevy smoke test on the GTX 1070) should run around sessions 3 or 4 of M0.

## Review follow-ups (fold into a later PR)
- WP4 review: cross-field validation in `Species::validate` (t_base < t_opt < t_max, emergence < flowering < maturity, growth temperature ordering, finite tropisms, `#rrggbb` colors); the discrete-trait rounding rule in the species model card; `specific_stem_length` likely 10–20 cm/g (WP12). (The typed `Traits` view landed with WP7.)
- WP5 review (/mnt/project-files/reviews/wp5-review.md): derive the lateral fraction from a diffusivity (4e-5 m² h⁻¹ gives 0.1 at 2 cm) so 5 cm cells are right; per-column surface radiation so canopy shade dries only sunlit columns; `M2PerHour` and `M3` newtypes for `take_up` and `cell_volume`; interpolate the soil temperature lag; add "infiltration can fill a lower cell through a saturated one in the same hour" to known limits.

- WP6 review (/mnt/project-files/reviews/wp6-review.md), for WP12: correlated normals for sibling leaflets in the ray tracer fixture, a Monte Carlo diffuse reference drawn from the overcast radiance, a planophile G for bean, per-leaf normals for the beam, and a const table for the sky sectors.

- Performance review (/mnt/project-files/perf/perf-review-1.md): light rebuild is 31 ms/day for 300 leaves vs the 2 ms whole-plant budget. Apply /mnt/project-files/perf/wp6-light-speedups.patch (own-footprint range check, rays stop at the occupied box, rayon over leaves with ordered collect; bit-identical) on top of #18; decision record for MAX_CELLS_PER_AXIS 32 (27% faster, fixture total +1.7%). Before WP8: sparse (cell, length) root lists in ps-soil and one joint uptake call per hour.

## Notes for the next session
- WP8 consumes ps-physio: build a `ClassTable` per plant-hour, read leaf rates with `lookup(par, leaf_capacity(...))`, pass income to `allocation::hourly`, run `starvation_deaths` in daily step 4, report `phenology::Events`. Leaf lifespan and shade senescence (daily step 4) belong to WP8.
- WP12: regenerate the light ray tracer fixture from a sim-exported 60-day plant and re-run `single_plant_matches_ray_tracer`; tighten its bounds if every leaf passes (decision 0008).
- Local checks: see CLAUDE.md "Session start".
- `cargo nextest` runs in CI; locally `cargo test --workspace` is equivalent.
