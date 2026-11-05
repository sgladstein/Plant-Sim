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

## M2: plot and sandbox
| WP | Issue | State |
| --- | --- | --- |
| 21 Plot soil and shared light | #49 | done (PR #50; decision 0019). Model 0.3.0. Plots of 1 to 3 m with 5 cm cells, one shared light grid per plot with per-leaf rays (per-voxel light measured and rejected), parallel plant stepping, 30-bean plot in the golden hashes. Speed not yet met (the perf thread has it): reviewer 20.9 ms per sim-day over 60 days on 4 cores, target 20 ms |
| 24 Nitrogen | #52 | done (PR #54; review follow-ups, model 0.5.0) (decision 0020). Model 0.4.0, species schema 2. Soil humus, NH4 and NO3 per cell (mineralization, nitrification, leaching), joint daily uptake with a 4 cm root reach, organ N as state, hourly top-up from a plant pool, mining low-light leaves first, demand-driven fixation by nodule organs, fertilize action. Done-when: nodulated shoot 7.9x unnodulated in N-free sand at 45 days after emergence; leaves drop bottom-up in seed fill. single_pot and the response scenarios are fertilized (300 ppm NO3); garden soil is the default. Plot speed 24.6 ms per sim-day (main 19.4 on the same machine) |
| 22, 23, 25 to 27 | | open |

**M0 exit (§7):** the validation report passes every M0 gate, the 90-day run is deterministic (golden hashes, Linux = Windows, debug = release), frost kills the plant for good, and the known limits are in decision 0018 and the model cards. The owner signs off M0.

## M1 groundwork (parallel to M0, viewer thread)
| WP | State |
| --- | --- |
| 13 Lock the engine | Bevy 0.20.1 + bevy_egui 0.43.1 pinned (docs/bevy-notes.md); waiting on the owner's screenshot from the GTX 1070 (CI artifact `plant-sim-windows`) |
| 14 Mesh generation (ps-mesh) | done (PR #23) |
| 15 Scene sync and look | started: ps-view draws two rhizobox pots, tubes and instanced organs, 2% rebuild rule, sky from ps-weather, auto-framing orbit camera, high-speed daily-mean light |
| 16 Sim thread | done: sim side PR #32 (decision 0016); viewer side (this PR): ps-app runs the scenario on ps-session by default and draws its snapshots (`--demo` keeps the stand-ins). Skeletons use ps-arch's attach and multiplicity (matches `geometry::layout`, tested); every lens reads sim values |
| 17 UI | started: ps-ui top bar (speeds, steps, replay scrubber), lens panel with legends, charts panel (§5.4: dry mass by organ, leaf area, height, root-zone FTSW, water shortfall, pods; pots overlaid as solid and dashed lines; click to seek; `--charts`), inspector (§5.3: click an organ for its §3.3 fields with units, model card on hover, a generated why line and the active lens's value; click the soil for plant totals; `--inspect POT[:ORGAN]`). Not yet: experiment panel, model card panel, assimilation and transpiration charts (not in the daily rows) |
| 19 Capture | started: Screenshot and Export CSV buttons (P key; `captures/<scenario>-dayDDD-HHh`, CSVs from ps-export for the state on screen), offline time-lapse frames (`--timelapse DIR --day D --to-day D --timelapse-hours 6-19`, one frame per sim hour, same clip on any machine), `--clean` for clips. Seed library waits for a harvest action in the sim |
| 18 Lenses | framework and all seven M1 lenses, all on sim data since WP16: absorbed PAR from each leaf's light coefficients under the hour's weather and shade cloth, R:FR, reserve concentration, thermal age, root age, the stress factor, FTSW on the soil slice through the plant |

The viewer reads `ps_view::Garden` and `SkyState` only; `ps-app/src/sim_source.rs` fills them from `ps_session::Snapshot`. Lateral root cohorts are drawn as up to 8 copies at golden-angle turns about the parent root. Pots are drawn round (`PotShape::Round`): a terracotta back half lined with soil, a curved glass front half, and a saucer; the soil-water slice wraps onto the glass. The rhizobox shape remains for the demo scene.

## Last validation report
`cargo run --release -p ps-cli -- validate`; CI uploads it as the `validation-report` artifact of the linux workflow (WP11). At WP24 after review (model 0.5.0, world seed 7): 73 checks: 58 pass, 15 Info misses, 0 known misses (the two new ones are the Ndfa targets: garden plot 18% misses 30 to 60%, the 5 ppm bed passes at 56%). The M0 misses are listed in decision 0018. Decision 0020 lists what changed: density now passes (-43% against -70%), as do heat pods and heat seeds per pod; drought shoot mass now misses (-64%); shade internodes +23% against a 20% gate.

## Open questions for the owner
- WP7 (decision 0009): two readings of the plan's text to confirm. Priority is the share of an organ's own available reserve it may spend per hour (taken literally, §3.9 would cap every organ below its potential), and the water stress factor multiplies the stomatal slope.
- WP12 light re-check (decisions 0008, 0018; still open after WP21, which keeps one G per plot): on a 60-day sim plant, the light model is 10% too bright on the plant total with the species G of 0.5, and 49 of 74 leaflets are within 0.10. The sim draws every blade tilted about 19°, and G = 0.5 assumes spherical leaf angles. With G = 0.65 the total is within 2%, and 60 of 74 leaflets are within 0.10. Fixing it is a model change that needs re-tuning. Either make G follow the leaf angles the sim draws, varying with direction (area-weighted |n·d|, the reviewer's preference), or draw a spread of leaf angles. The suggestion is M2, with WP21.
- Set `main` as the repository's default branch in GitHub settings (the agent cannot change repo settings).
- WP13 (Bevy smoke test on the GTX 1070) passed its frame-time gate on 2026-11-05 with stand-in plants: Vulkan 2.7 ms per frame, DX12 5.8 ms (budget 16 ms). Re-check on the real sim. The viewer thread has the follow-ups (default backend, slow cold-start screenshots, see-through side panel).

## Review follow-ups (fold into a later PR)
- WP21 review (/mnt/project-files/reviews/wp21-review.md), still open after WP24:
  - Density is still short (-43% against -70%). Branches and leaf size do not respond to density, and stand LAI is 5.5 at 34 per m² against a real plateau of 3 to 4. The lever is bud outgrowth and leaf potential area scaling with the R:FR shade signal. Read the density variables from interior plants, not edge plants (a plant filter on the target).
  - Plot sides that are not multiples of 5 cm are rounded silently: reject them in `validate`, or document the rounding.
  - The golden mismatch text should say the golden holds 30 days.
  - Per-column soil surface radiation before WP26.
  - light.md: the plot grid bottom is `min(0, lowest leaf)`. world-and-schedule: list the first plant's G as a plot parameter.
  - Compute the light inputs and leaflet list once a day. Settle the G question in decision 0018.
- WP24 (decision 0020), for later packages:
  - Pod set should read nitrogen. N-short plants set as many pods as fed ones and fill smaller seeds.
  - Leaf aging still multiplies capacity next to leaf N.
  - The root reach and the 25x uptake capacity stand in for root length the sim does not draw.
  - Uptake capacity should scale with root mass (or drawn plus undrawn length), not a constant 25x: drawn length is a tenth of a real bean's at day 40 and near all of it at maturity. The factor should then return to CROPGRO's RTNO3 and plot Ndfa rise from 18%. Mineralization (0.03 g N per m2 a day on the plot) is low against gardens (0.05 to 0.15). Issue to follow.
  - WP24 review (/mnt/project-files/reviews/wp24-review.md), folded in as model 0.5.0 (decision 0020 amendment). Still open: thin shade leaves carry the same N per mass as sun leaves (real ones carry somewhat more); the density pass comes mostly from topsoil N running out; fixation is 34% of the N-free plant's carbon; more N giving less mass at maturity (Rd scales with capacity while the sink feedback throttles A).
  - The soil water has no upward flow.
- WP12 review (/mnt/project-files/reviews/wp12-review.md): an ignored test or weekly CI job that runs the M0 gates at world seeds 11, 23 and 31 (today only `seeds.sh`-style local runs check them).
- Viewer (PR #38): at day 60 of the full-sun pot at least one root tip sits outside the soil grid on the +x side, through the pot wall. Roots need clamping or deflecting at the pot boundary (ps-arch root elongation).
- WP4 review: cross-field validation in `Species::validate` (t_base < t_opt < t_max, emergence < flowering < maturity, growth temperature ordering, finite tropisms, `#rrggbb` colors); the discrete-trait rounding rule in the species model card; `specific_stem_length` likely 10–20 cm/g (WP12). (The typed `Traits` view landed with WP7.)
- WP5 review (/mnt/project-files/reviews/wp5-review.md): (the lateral fraction from a diffusivity landed with WP21); per-column surface radiation so canopy shade dries only sunlit columns; `M2PerHour` and `M3` newtypes for `take_up` and `cell_volume`; interpolate the soil temperature lag; add "infiltration can fill a lower cell through a saturated one in the same hour" to known limits.

- WP6 review (/mnt/project-files/reviews/wp6-review.md), for WP12: correlated normals for sibling leaflets in the ray tracer fixture, a Monte Carlo diffuse reference drawn from the overcast radiance, a planophile G for bean, per-leaf normals for the beam, and a const table for the sky sectors.

- Performance: the light speedups and 32 cells per axis landed (decision 0010), and class-table lookups cache their node PAR values. Sparse root lists and one joint uptake call per hour landed (decision 0012). Before WP21: reusable scratch buffers in `allocation::hourly`, one class table per distinct (params, stress) at plot scale, and a look at `ExpansionCurve::increment`. Footprints are now computed once a day (`LightGrid::fit_with_footprints`). M0 speed (§7 exit, under 2 ms per sim-day): after WP12 tuning the single pot took 2.17 ms; the bit-identical speed fix brought it to 1.67 ms on 4 cores (2.34 single-thread), two pots 2.9 ms. Met; golden hashes unchanged. WP21 plot speed (30 plants under 20 ms per sim-day): a bit-identical follow-up (shared class tables, plot footprint reuse, dense uptake totals, parallel after-light step, daily hash off the critical path) took plot_30_beans from 28.8 to about 20.1 ms on 4 cores (19.7 to 20.7 over 5 runs, 51 ms single-thread): at the target, not reliably under. See docs/performance.md for what is left.

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
- WP21 (decision 0019): `SoilSpec::Plot` in scenarios; the world steps a soil's plants in parallel (ordered collect), and a plot's daily light runs between `Plant::day_before_light` and `Plant::day_after_light`. Pots are byte-identical to 0.2.0. The viewer still draws a plot as a round pot (`ps-app/src/sim_source.rs` only got a compile fix).
- WP24 (decision 0020, docs/models/nitrogen.md): `nitrogen` map per soil in scenarios (default garden soil), `Fertilize` action, `Role::Nodule` organs, 9 new daily CSV columns and `nitrogen_g` in the organ CSV. Daily step 0 is `World::daily_nitrogen` (joint uptake per soil, then `Plant::nitrogen_day`); `Plant::nitrogen_alone` does the same for a plant driven without a world.
- Next: the remaining M2 packages in plan order (docs/design-plan.md §7), and the WP21 speed target if the perf follow-up has not met it. Research notes are in the project files (research/m2-m3-bean-science.md).
- Local checks: see CLAUDE.md "Session start".
- `cargo nextest` runs in CI; locally `cargo test --workspace` is equivalent.
