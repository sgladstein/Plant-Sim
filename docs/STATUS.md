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
| 6 Light | #7 | done (PRs #17, #18); owner accepted 95% of leaves within 10% for M0, re-check in WP12 (decision 0008) |
| 7 Physiology | #8 | done (PR #20) |
| 8 Herb architecture and roots | #9 | next |
| 9 World | #10 | not started |
| 10 Export and CLI | #11 | not started |
| 11 Validation harness | #12 | not started |
| 12 Tuning and lock | #13 | not started |

## Last validation report
None yet (WP11).

## Open questions for the owner
- WP7 (decision 0009): two readings of the plan's text to confirm. Priority is the share of an organ's own available reserve it may spend per hour (taken literally, §3.9 would cap every organ below its potential), and the water stress factor multiplies the stomatal slope.
- Set `main` as the repository's default branch in GitHub settings (the agent cannot change repo settings).
- WP13 (Bevy smoke test on the GTX 1070) should run around sessions 3 or 4 of M0.

## Review follow-ups (fold into a later PR)
- WP4 review: cross-field validation in `Species::validate` (t_base < t_opt < t_max, emergence < flowering < maturity, growth temperature ordering, finite tropisms, `#rrggbb` colors); the discrete-trait rounding rule in the species model card; `specific_stem_length` likely 10–20 cm/g (WP12). (The typed `Traits` view landed with WP7.)
- WP5 review (/mnt/project-files/reviews/wp5-review.md): derive the lateral fraction from a diffusivity (4e-5 m² h⁻¹ gives 0.1 at 2 cm) so 5 cm cells are right; per-column surface radiation so canopy shade dries only sunlit columns; `M2PerHour` and `M3` newtypes for `take_up` and `cell_volume`; interpolate the soil temperature lag; add "infiltration can fill a lower cell through a saturated one in the same hour" to known limits.

- WP6 review (/mnt/project-files/reviews/wp6-review.md), for WP12: correlated normals for sibling leaflets in the ray tracer fixture, a Monte Carlo diffuse reference drawn from the overcast radiance, a planophile G for bean, per-leaf normals for the beam, and a const table for the sky sectors.

- Performance: the light speedups and 32 cells per axis landed (decision 0010), and class-table lookups cache their node PAR values. Sparse root lists and one joint uptake call per hour landed (decision 0012). Before WP21: reusable scratch buffers in `allocation::hourly`, one class table per distinct (params, stress) at plot scale, and a look at `ExpansionCurve::increment`. Optional: cache the footprints `LightGrid::fit` computes.

- WP7 review and research findings: done in decision 0011 (expansion clock, implicit phloem transport, phenology from germination, debt charged to the ancestor, maintenance 0.218 g C/g N/day). Open for WP12: phloem conductance K now sets how far carbon reaches per hour; mature leaves import like any organ; priority is a weak lever; Vcmax/Jmax (Pimentel 2011) are chloroplast-based, so without gm Rubisco-limited A is about 20% high (add gm or use Ci-based Vcmax near 70).
- Research findings for WP8 (from the research thread; full report to come): bean roots are a tap root (2.36 cm/day), two whorls of about 4 basal roots (about 4 cm/day from day 3, gravitropism graded by whorl) and up to 10 hypocotyl-borne roots (about 1 cm/day, 85°, no gravitropism); laterals 0.77 cm/day for 6–10 days, about 6 cm, 7–12 per cm, 75°. Specific root length per root type from diameter (tap 0.9, basal 0.68, hypocotyl 0.64, lateral 0.3 mm) and tissue density 0.094 g/cm³, not one 70 m/g. Uptake capped near 0.03 cm³ water per cm root per day. Bean trifoliates are probably distichous (about 180°), not 137.5°. Baseline pod set 0.2–0.55 of flowers even unstressed. Shade avoidance lengthens internodes only, capped at 1.5–2×.

## Notes for the next session
- WP8: `leaf_capacity` takes the thermal age at full expansion; with the paced expansion clock (decision 0011) record the thermal age when `is_complete(expansion_age)` first becomes true, not the curve duration. Root segments become `ps_soil::RootCells` per plant.
- WP8 consumes ps-physio: build a `ClassTable` per plant-hour, read leaf rates with `lookup(par, leaf_capacity(...))`, pass income to `allocation::hourly`, run `starvation_deaths` in daily step 4, report `phenology::Events`. Leaf lifespan and shade senescence (daily step 4) belong to WP8.
- WP12: regenerate the light ray tracer fixture from a sim-exported 60-day plant and re-run `single_plant_matches_ray_tracer`; tighten its bounds if every leaf passes (decision 0008).
- Local checks: see CLAUDE.md "Session start".
- `cargo nextest` runs in CI; locally `cargo test --workspace` is equivalent.
