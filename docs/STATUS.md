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
| 8 Herb architecture and roots | #9 | done (this PR; decision 0013) |
| 9 World | #10 | next |
| 10 Export and CLI | #11 | not started |
| 11 Validation harness | #12 | not started |
| 12 Tuning and lock | #13 | not started |

## M1 groundwork (parallel to M0, viewer thread)
| WP | State |
| --- | --- |
| 13 Lock the engine | Bevy 0.20.1 + bevy_egui 0.43.1 pinned (docs/bevy-notes.md); waiting on the owner's screenshot from the GTX 1070 (CI artifact `plant-sim-windows`) |
| 14 Mesh generation (ps-mesh) | done (PR #23) |
| 15 Scene sync and look | started: ps-view draws two rhizobox pots, tubes and instanced organs, 2% rebuild rule, sky from ps-weather, auto-framing orbit camera, high-speed daily-mean light |
| 16 Sim thread | not started; ps-app's `demo_source` stands in (time commands from the UI, no action log). The WP9 snapshot will carry organ positions and root cohort Meta; read those instead of `Skeleton::from_arena` placement |
| 17 UI | started: ps-ui top bar (speeds, steps, replay scrubber) and lens panel with legends |
| 18 Lenses | framework and all seven M1 lenses; Light and Shade signal run the real ps-light on stand-in plants; the rest are stand-in data, flagged in the legend |

The viewer reads `ps_view::Garden` and `SkyState` only. Swapping the stand-in for ps-world means filling those from the snapshot (WP16).

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
- WP8 for WP12 (decision 0013): stems have no secondary thickening (§3.4) and roots are about 4% of dry mass, low for bean; the leaf sink feedback (0.10 to 0.30 g C/g DM) keeps reserves physical and should be checked against bean leaf starch; no phototropism or hydrotropism yet; branches have no axillary racemes; all leaves drop at maturity; the research review says shade avoidance changes internode length only, while §3.4 also raises SLA up to 1.3×.

## Notes for the next session
- WP9: `ps_arch::Plant` runs one plant's §3.2 steps. Per hour: `water_demand` for every plant, `SoilGrid::step_hour`, one `take_up` over every plant's `uptake_request`, then `grow_hour` with each plant's `Uptake`. Per day: `day(soil, next day's sun path, tick)`. `tests/ninety_days.rs::run_watered` is a one-plant version of that loop. The snapshot should carry `Plant::positions` (base, tip per organ, soil-grid frame) and `Meta` (role, attach, multiplicity) for the viewer (agreed with the viewer thread).
- WP12: regenerate the light ray tracer fixture from a sim-exported 60-day plant and re-run `single_plant_matches_ray_tracer`; tighten its bounds if every leaf passes (decision 0008).
- Local checks: see CLAUDE.md "Session start".
- `cargo nextest` runs in CI; locally `cargo test --workspace` is equivalent.
