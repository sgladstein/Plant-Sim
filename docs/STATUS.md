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
| 8 Herb architecture and roots | #9 | done (PR #27; decision 0013) |
| 9 World | #10 | done (PR #29; decision 0014) |
| 10 Export and CLI | #11 | done (PR #31; decision 0015, docs/cli.md) |
| 11 Validation harness | #12 | done (PR #39; decision 0017, validation/README.md) |
| 12 Tuning and lock | #13 | done (this PR; decision 0018). Model 0.2.0; every M0 check passes in four weather years; known-miss list empty |

**M0 exit (§7):** the validation report passes every M0 gate, the 90-day run is deterministic (golden hashes, Linux = Windows, debug = release), frost kills the plant for good, and the known limits are in decision 0018 and the model cards. The owner signs off M0.

## M1 groundwork (parallel to M0, viewer thread)
| WP | State |
| --- | --- |
| 13 Lock the engine | Bevy 0.20.1 + bevy_egui 0.43.1 pinned (docs/bevy-notes.md); waiting on the owner's screenshot from the GTX 1070 (CI artifact `plant-sim-windows`) |
| 14 Mesh generation (ps-mesh) | done (PR #23) |
| 15 Scene sync and look | started: ps-view draws two rhizobox pots, tubes and instanced organs, 2% rebuild rule, sky from ps-weather, auto-framing orbit camera, high-speed daily-mean light |
| 16 Sim thread | done: sim side PR #32 (decision 0016); viewer side (this PR): ps-app runs the scenario on ps-session by default and draws its snapshots (`--demo` keeps the stand-ins). Skeletons use ps-arch's attach and multiplicity (matches `geometry::layout`, tested); every lens reads sim values |
| 17 UI | started: ps-ui top bar (speeds, steps, replay scrubber) and lens panel with legends |
| 18 Lenses | framework and all seven M1 lenses, all on sim data since WP16: absorbed PAR from each leaf's light coefficients under the hour's weather and shade cloth, R:FR, reserve concentration, thermal age, root age, the stress factor, FTSW on the soil slice through the plant |

The viewer reads `ps_view::Garden` and `SkyState` only; `ps-app/src/sim_source.rs` fills them from `ps_session::Snapshot`. Lateral root cohorts are drawn as up to 8 copies at golden-angle turns about the parent root. Still to do: pots render as boxes the size of the soil grid.

## Last validation report
`cargo run --release -p ps-cli -- validate`; CI uploads it as the `validation-report` artifact of the linux workflow (WP11). At WP12 (model 0.2.0, world seed 7): 70 checks: 54 pass, 15 Info misses, 1 check gated Later (M2 density) not evaluated, 0 known misses. The misses are listed in decision 0018.

## Open questions for the owner
- WP7 (decision 0009): two readings of the plan's text to confirm. Priority is the share of an organ's own available reserve it may spend per hour (taken literally, §3.9 would cap every organ below its potential), and the water stress factor multiplies the stomatal slope.
- WP12 light re-check (decisions 0008, 0018): on a 60-day sim plant, the light model is 10% too bright on the plant total with the species G of 0.5, and 49 of 74 leaflets are within 0.10. The sim draws every blade tilted about 19°, and G = 0.5 assumes spherical leaf angles. With G = 0.65 the total is within 2%, and 60 of 74 leaflets are within 0.10. Fixing it is a model change that needs re-tuning. Either make G follow the leaf angles the sim draws, varying with direction (area-weighted |n·d|, the reviewer's preference), or draw a spread of leaf angles. The suggestion is M2, with WP21.
- Set `main` as the repository's default branch in GitHub settings (the agent cannot change repo settings).
- WP13 (Bevy smoke test on the GTX 1070) passed its frame-time gate on 2026-11-05 with stand-in plants: Vulkan 2.7 ms per frame, DX12 5.8 ms (budget 16 ms). Re-check on the real sim. The viewer thread has the follow-ups (default backend, slow cold-start screenshots, see-through side panel).

## Review follow-ups (fold into a later PR)
- WP12 review (/mnt/project-files/reviews/wp12-review.md): an ignored test or weekly CI job that runs the M0 gates at world seeds 11, 23 and 31 (today only `seeds.sh`-style local runs check them).
- Viewer (PR #38): at day 60 of the full-sun pot at least one root tip sits outside the soil grid on the +x side, through the pot wall. Roots need clamping or deflecting at the pot boundary (ps-arch root elongation).
- WP4 review: cross-field validation in `Species::validate` (t_base < t_opt < t_max, emergence < flowering < maturity, growth temperature ordering, finite tropisms, `#rrggbb` colors); the discrete-trait rounding rule in the species model card; `specific_stem_length` likely 10–20 cm/g (WP12). (The typed `Traits` view landed with WP7.)
- WP5 review (/mnt/project-files/reviews/wp5-review.md): derive the lateral fraction from a diffusivity (4e-5 m² h⁻¹ gives 0.1 at 2 cm) so 5 cm cells are right; per-column surface radiation so canopy shade dries only sunlit columns; `M2PerHour` and `M3` newtypes for `take_up` and `cell_volume`; interpolate the soil temperature lag; add "infiltration can fill a lower cell through a saturated one in the same hour" to known limits.

- WP6 review (/mnt/project-files/reviews/wp6-review.md), for WP12: correlated normals for sibling leaflets in the ray tracer fixture, a Monte Carlo diffuse reference drawn from the overcast radiance, a planophile G for bean, per-leaf normals for the beam, and a const table for the sky sectors.

- Performance: the light speedups and 32 cells per axis landed (decision 0010), and class-table lookups cache their node PAR values. Sparse root lists and one joint uptake call per hour landed (decision 0012). Before WP21: reusable scratch buffers in `allocation::hourly`, one class table per distinct (params, stress) at plot scale, and a look at `ExpansionCurve::increment`. Optional: cache the footprints `LightGrid::fit` computes.

- WP7 review and research findings: done in decision 0011 (expansion clock, implicit phloem transport, phenology from germination, debt charged to the ancestor, maintenance 0.218 g C/g N/day). Open for WP12: phloem conductance K now sets how far carbon reaches per hour; mature leaves import like any organ; priority is a weak lever; Vcmax/Jmax (Pimentel 2011) are chloroplast-based, so without gm Rubisco-limited A is about 20% high (add gm or use Ci-based Vcmax near 70).
- WP8 review, folded into WP9 (decision 0013 items 2, 7, 12, 13): primordia are built at construction cost with growth respiration in the ledger; the sink feedback acts on gross assimilation only; leaf lifespan counts from full expansion; the daily report carries total dry mass and the feedback loss. With these, `pod_set_base` is 0.45 (inside Binnie and Clifford's 0.2 to 0.55) and world seeds 1 to 8 give 10 to 18 pods. **For WP12:** from flowering on the feedback discards about 60% of potential gross assimilation, so the plant is sink-limited (no stem thickening, roots about 4% of DM, Vcmax about 20% high without gm).
- WP8 performance (perf thread): the Q10 cache patch landed with WP9. Light rebuild is about two thirds of the daily cost: 2.72 ms per day single-threaded against a 2 ms budget, 1.94 ms on 4 cores. Owner options if it matters: fewer sky sample points, or rebuild light every other day. Smaller items: zeroed scratch Vecs, footprint caching. WP9 world: 1.96 ms/day for one pot and 3.40 ms/day for two, on 4 cores. The daily hash is chained, so it costs the same every day. Free speedup for later: step pots with separate soils and light grids in parallel (rayon over pots, ordered collect).
- WP8 for WP12 (decision 0013): stems have no secondary thickening (§3.4) and roots are about 4% of dry mass, low for bean; the leaf sink feedback (0.10 to 0.30 g C/g DM) keeps reserves physical and should be checked against bean leaf starch; no phototropism or hydrotropism yet; branches have no axillary racemes; all leaves drop at maturity; the research review says shade avoidance changes internode length only, while §3.4 also raises SLA up to 1.3×.

## Notes for the next session
- WP9 (decision 0014, model card world-and-schedule): `ps_world::World` steps scenarios from `data/scenarios/`, keeps the action log, writes RON snapshots and a full-state hash per day. For the viewer: `WorldState.plants[i].plant` carries `positions` (base, tip per organ, soil-grid frame) and `meta` (role, attach, multiplicity); `World::snapshot_ron`/`from_snapshot_ron` round-trip bit for bit.
- WP10: golden hashes for linux-x86_64 and windows-x86_64 are committed and identical (the Windows CI run reproduced Linux day for day). CI fails on a mismatch or a missing file.
- WP11/WP12, first response runs (decision 0015): heat at flowering cuts seed weight by 21% (cited 50 to 71%), drought from flowering by 19%, shade 50% cuts total mass by 35% but pods rise and root mass falls sixfold.
- WP16 (sim thread) runs in its own thread; the viewer thread does the viewer side.
- WP12 (decision 0018): the heat and drought response scenarios start on the control's flowering day (day 43). Move them with any change to the phenology thresholds. The validation run is about 20 s in release. Regenerate the sim-plant light fixture with `PS_LEAVES_OUT=... cargo test -p ps-arch --test ninety_days -- --ignored`, then `python3 validation/fixtures/light_raytrace.py --leaves FILE`.
- Next: M2 sim packages, WP21 then WP24 (docs/design-plan.md §7). Research notes for them are in the project files (research/m2-m3-bean-science.md).
- Local checks: see CLAUDE.md "Session start".
- `cargo nextest` runs in CI; locally `cargo test --workspace` is equivalent.
