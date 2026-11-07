# Implementation workstream: handoff

Written 2026-11-05 by the implementation thread (the sim crates, ps-world and below, plus ps-cli,
ps-validate, ps-export, ps-session and data/). It covers everything a fresh thread needs to
keep implementing the design plan from the repository alone.

## 1. How this workstream runs (instructions for a fresh thread)

**Read first, every session:** `CLAUDE.md` (the non-negotiable sim rules), then `docs/STATUS.md`
(which WP is done, in flight and open, plus every open follow-up), then the WP's row in
`docs/design-plan.md` §7 and the model cards it touches in `docs/models/`.

**One work package per branch and PR.**
- The backlog is `docs/design-plan.md` §7, in plan order. M0 and M2 WP21 to WP25 and WP27 are done. WP26 finishes M2; the realism specs R1 to R3 come next for this thread, then M3 (WP28 to WP32).
- Each WP gets a GitHub issue titled `WP<n>: ...`, holding its done-when as acceptance criteria. Create it if it is missing (WP26 is #89, WP27 #105). Close it with `state_reason: completed` when the PR merges.
- Review follow-ups that are not a WP get their own small PR, such as the light fix (#75), the validation harness (#81) and the root fix. Record them in STATUS.

**Every model change carries five things in the same PR:**
1. A decision record, `docs/decisions/NNNN-title.md`, from `TEMPLATE.md`. Number it as the next free number; 0034 is the latest (0035 is claimed by the genetics thread's WP29).
2. A model card update in `docs/models/` for every equation or default that changes.
3. A minor bump of `MODEL_VERSION` in `crates/ps-world/src/lib.rs`, plus `model_version` in every scenario under `data/scenarios/` (sed them all).
4. A species schema bump in `crates/ps-species/src/schema.rs`, if a species file key changes. Schema is at 11.
5. Regenerated goldens:
   ```sh
   B=target/release/ps-cli   # after cargo build --release -p ps-cli
   $B hash data/scenarios/single_pot.ron data/scenarios/two_pot_sun_shade.ron --write-golden tests/golden
   $B hash data/scenarios/plot_30_beans.ron --days 30 --write-golden tests/golden
   cp tests/golden/linux-x86_64/*.csv tests/golden/windows-x86_64/
   ```
   Windows goldens equal Linux ones. CI checks both platforms, so a mismatch is a determinism bug.
6. The challenges (`data/challenges/`, scenarios in `data/scenarios/challenges/`) pin the model too.
   If `cargo test -p ps-game` fails after the bump, re-sweep the medals as in docs/game/challenges.md (on the 8-season mean that `ps-cli challenge` prints).

**Before every push:**
```sh
cargo fmt --all --check
cargo clippy --workspace --exclude ps-view --exclude ps-ui --exclude ps-app --all-targets --locked -- -D warnings
cargo test --workspace --exclude ps-view --exclude ps-ui --exclude ps-app --no-fail-fast --locked
python3 scripts/check_deps.py
python3 scripts/check_determinism.py
$B hash data/scenarios/single_pot.ron data/scenarios/two_pot_sun_shade.ron --check-golden tests/golden
$B hash data/scenarios/plot_30_beans.ron --days 30 --check-golden tests/golden
$B validate        # writes out/validation/report.md and results.csv; about 2 minutes on 4 cores
```
ps-view, ps-ui and ps-app are excluded because the cloud container has no wayland/X libraries. CI's `windows-app` and `render` jobs build them.

**PR conventions.**
- PR body sections: Work package, What changed, Done-when evidence (validation counts, test names, timings), Checklist.
- Assign the owner (`sgladstein`).
- Merge by squash, with the full 40-character head SHA as `expectedHeadSha`, once CI is green.
- The owner approved all merges for this workstream. A new account should re-confirm that with the owner, not assume it.

**Reviews.**
- A separate review thread reviewed every WP and wrote `reviews/*.md` (now `docs/archive/reviews/`).
- A performance thread benchmarked each WP (`ps-cli bench`, criterion benches in `crates/ps-bench`).
- Their follow-ups are folded into STATUS under "Review follow-ups". When starting fresh without those threads, ask for an independent review of each model PR before merge, if you can.

**Validation (decision 0026).**
- `ps-cli validate` runs every scenario in `data/validation/targets.ron` at world seed + 0, 1, 2 and 3, and checks the seed mean.
- The report shows mean ± SD (min to max, n).
- **Tune "~" parameters (guesses) on seed means only, never on one seed.** One bean spans 21 to 39 g on the seed alone.
- M0 gates fail the build; everything else is Info.
- Targets, sources and bands are documented in `docs/validation-data.md` and `data/validation/`.

## 2. Current state (2026-11-07, R5b PR)

**Model and data.** Model 0.29.0 (decision 0048: waterlogged soil runs short of oxygen; hypoxic roots slow, wilt, lose nitrogen uptake and die, and regrow when the soil drains; hypoxic cells denitrify; standing water evaporates first; `SetDrainage`; realism plan R5b), species schema 19. Before it 0.28.0 (decision 0047: the daily rows carry the soil's drained, run-off and evaporated water and leached and mineralized nitrogen since the start; results unchanged from 0.27.0), 0.27.0 (decision 0046: soil water moves by Darcy's law, with pot drainage options, realism plan R5a), 0.26.0 (decision 0045: dry soil limits roots, and the water network has a simplified level, realism plan R3c), 0.25.0 (decision 0044: water moves through each plant as a network, realism plan R3b, schema 17), 0.24.0 (decision 0043: roots respond to dry and nitrate-rich soil, realism plan R2b, schema 16), 0.23.0 (decision 0042: seed growth ends at maturity, bush seed fill 289 Cd), 0.22.0 (decision 0041: phototropism, realism plan R1c), 0.21.0 (decision 0040: starch storage, root exudation and seed-driven nitrogen, realism plan R1b, schema 14), 0.20.0 (decision 0037: root length and water uptake, realism plan R2a, issue #116; the thickening potential is set daily, schema 13), 0.19.0 (decision 0036: stem and root thickening, realism plan R1a, schema 12), 0.18.0 (decision 0034 amendment: coupled tolerance 0.05 K, for speed; perf thread), 0.17.0 (decision 0034: the leaf energy balance, WP27, schema 11) and 0.16.0 (decision 0033: broad leaves at 5 light points) and schema 10 (decision 0032). The genetics thread's WP28 step 2 took 0.14.0, schema 9 and decision 0031.

**Merged today:**
- #87: lateral roots stay in the soil (decision 0028, model 0.11.0).
- #95: pole beans find their stake; variation-review items (decision 0029, model 0.12.0). Pole heights pass on seed means.
- #98: grass blades sample light at 3 points per strip (decision 0030, model 0.13.0), the owner's option (a) for dense wheat speed: about -12 to -15% on wheat_stand.
- #104: the re-tune (decision 0032, model 0.15.0): development trapezoid 7 / 25 / 33 / 42 °C with the calendar kept, Medlyn g1 5.79, per-organ construction cost keys (all 1.25 for now), pole internode_last_fraction 1.75.
- #110: broad leaves at 5 light points (decision 0033, model 0.16.0), the performance thread's plot-speed option 1.
- #114: leaf energy balance (decision 0034, model 0.17.0), wind, leaf frost on leaf temperature.
- #118: coupled leaf-temperature tolerance 0.05 K (decision 0034 amendment, model 0.18.0), perf thread.
- #119: R1a (decision 0036, model 0.19.0): stem and root thickening, a sink at priority 0.4.
- #129: R2a (decision 0037, model 0.20.0): roots queue behind the tip, tip feeding, fine-root cohorts, `u_max` over the transpiring hours (#116), stomata follow supply, nitrogen from each cell's own roots, wood yields first when carbon is short, the thickening potential set once a day, shade factors recalibrated.
- #133: R1b (decision 0040, model 0.21.0): a starch store in stems and main roots (release below `C_off`, refill above `C_on` 0.15 once expanded, emptied from seed fill at 0.08 a day), root exudation at 0.4 of maintenance, the seeds' nitrogen mined first from seed fill.
- #134: R1c (decision 0041, model 0.22.0): new bean internodes turn toward the light their apex saw (rays from each growing apex at the light rebuild, vector irradiance summed hourly; 0.3 of the angle; the 10° a day bound does not bind), then gravitropism. Main stems lean only beside obstacles; in the open it straightens branches (27° against 34° from vertical). The WP20 drawn lean should be turned down near obstacles, not switched off.
- #136: R1d (decision 0042, model 0.23.0): from physiological maturity nothing grows (CROPGRO stops seed growth at R7), and the bush bean's seed fill is 289 Cd (SFDUR 15 photothermal days, was 350). Seed at the end now equals seed at maturity; over eight pot seeds seed at maturity 6.89 → 7.84 g, final 9.60 → 7.84 g.
- #138: R2b (decision 0043, model 0.24.0, schema 16): dry soil slows root tips (floor 0.3), xerobranching below 0.1 of available water, lateral and fine-root cohorts proliferate (up to 3 times, path mean) where nitrate or water is above the larger of the soil's and the root zone's mean, hydropatterning, weak hydrotropism. Nitrate moves sideways with the water flux plus slow diffusion (was the water's exchange fraction, about 20 times too fast). Scenarios m2/split_pot, m2/nitrate_patch(_control), with dividers, strip watering and nitrate patches in the scenario format.
- #140: R3a (no model change): van Genuchten soil water potential per texture.
- #141: R3b (decision 0044, model 0.25.0, schema 17): each plant's organs are a water network solved every hour against the soil cells (radial links per root span, axial conductances from the organ sizes with a root xylem gain `(r/r_primary)²`, gravity); `SoilGrid::take_flows` replaces `take_up` (per-cell scaling at air-dry, deposits scaled by the plant's granted share); stomata read Tuzet's factor of the last hour's mean leaf potential (steps of 1/50, so class tables are still shared); leaf, internode and cotyledon expansion clocks run at their turgor (stored growth); thickening reads the leaves' predawn turgor; root carbon priority reads min(FTSW, stomatal factor); pod abortion and fixation keep FTSW. Debrief water pressure = max(shortfall share, 1 − predawn turgor). Daily CSV adds `leaf_turgor_predawn`. Tests: `ps-arch/tests/hydraulics.rs`, `ps-world/tests/wilting.rs`.
- The R3c PR (decision 0045, model 0.26.0, schema 18): a Gardner soil conductance in series with each radial link (`ps_arch::rhizosphere_conductance`, Mualem `K(θ)` from `Retention::psi_and_conductivity`, `Hydraulics::saturated_conductivity`), replacing `f_dry`; `HydraulicsLevel` per plant (in the hash) with a daily `WaterSummary` (K_rs, SUF per cell, drop per organ) built at both levels; the simplified hourly step (`water_flows_simplified`, Couvreur, `K_comp = hydraulic_compensation · K_rs`); `Scenario.fidelity` (`Auto` / `Detailed` / `Simplified` / `Locked`, `MAX_DETAILED` 8), `ActionKind::SetFidelity`, `ps_game::ActionType::Fidelity` (never a rule break) and `Challenge.fidelity`; `ps-cli fidelity` and `validate --fidelity`; CSV `root_conductance_kg_h_mpa`, `uptake_depth_m`; lens `hydraulics_level`, `uptake_by_layer`. New goldens single_pot_simplified and fidelity_switch. Tests: `ps-world/tests/fidelity.rs`, `ps-cli` fidelity unit tests.
- #146: R5a (decision 0046, model 0.27.0): soil water moves by Darcy's law (a Richards solve per column, 2 cm air entry, pot holes, no holes or a saucer, ponding), nitrate moves with the water (a mixing cell on infiltration), a flow table per texture and parallel plot columns.
- The R5a follow-up PR (decision 0047, model 0.28.0): soil water and nitrogen totals in the daily rows; `ninety_days.rs` waters through the surface, and its count tests grow the plant at 300 ppm nitrate, keeping §3.16's 10 to 20 pods.
- The R5b PR (decision 0048, model 0.29.0, schema 19): soil aeration (`SoilGrid::air_porosity`, `oxygen_deficit`, `hypoxic_hours`, `hypoxia`), root responses and death (`Plant::drown_roots`, `drowned` regrowth from the hypocotyl), radial and N-uptake factors, denitrification, pond evaporation, `SetDrainage`, the simplified summary's refresh on hypoxia, m2/waterlogged_pot and its control, `waterlogging_root_loss`, and the pot response scenarios under a rain shelter.

**Validation at 0.29.0, seed means.** 74 of 97 pass, every M0 gate passes (decision 0048); every pot response matches 0.28.0 under the new rain shelter, and the rained validation pot's seed 9 sets 17 pods instead of 5 (its pod-set draws changed).

**Validation at 0.27.0, seed means.** 72 of 96 pass, every M0 gate passes (decision 0046). split_pot_foraging (1.548), fine_root_share and density_leaf_area_per_plant pass again; ndfa_garden_plot slips out (0.24; the plot's plants take twice the soil nitrate), nitrate_patch_proliferation +16%, drought pods −75%, shade total mass −37%. The watered pot is unchanged on eight seeds (pods 11.6 → 11.1, seed 8.34 → 8.12 g; the four validation seeds' 6.8 g is their luck). Fidelity 111 of 111, bias −0.65%.

**Validation at 0.26.0, seed means.** 70 of 96 at `Auto`, 69 with every plant simplified, every M0 gate passes (decision 0045). The soil term alone (all detailed) keeps 71; at `Auto` density_leaf_area_per_plant slips to −34.97% (band from −35%), and simplified everywhere the pole bean is 1.495 m at day 60 (from 1.5). Seed per plant 7.49 → 8.09 g and harvest index 0.422 → 0.455 on four seeds, but over eight seeds the watered pot is unchanged (code review), drought pods −76% → −70%, drought shoot mass −67% → −62% (still a miss), split_pot_foraging 1.254, nitrate_patch_proliferation +37%. The fidelity report: 111 of 111 outcomes pass, seed-yield bias +1.7%; a four-seed bias has a standard error of about 3% (12 seeds: per-seed paired spread 17%), so the bias gate also needs twice its standard error.

**Validation at 0.25.0, seed means.** 71 of 96 pass, every M0 gate passes (decision 0044). R3b brought wheat_height_spaced (0.753 m) and density_leaf_area_per_plant (−36%) back in and took stem_share_peak out (0.346 against 0.35); split_pot_foraging fell to 1.248 and nitrate_patch_proliferation to +34.1% (the dry half dries more slowly as uptake moves to the wet half, and R2b's dry-soil rules read relative available water; code review ruled out redistribution); shade total mass −46%, heat seeds −31%, drought shoot mass −67% (still the miss below). Tried and not kept, with numbers, in decision 0044: turgor on the expansion increment (the spec's rule) cost 13 to 40% of leaf area and left the pole bean at 1.3 m at day 60.

**Validation at 0.24.0, seed means.** 70 of 96 pass, every M0 gate passes, the nearest shade total mass −49.1% against −50% (0.23.0: 69 of 94; 0.22.0: 70; 0.21.0: 70; 0.20.0: 68 of 92). R2b brought mean_seed_mass (0.189 g) and harvest_index_tops (0.435) back in and took density_leaf_area_per_plant out (−30% against −35%); fine_root_share 0.097 stays just out (band from 0.1). Its own checks both miss at the band's edge: nitrate_patch_proliferation +48.8% (seeds +40 to +64%; band from +50%), split_pot_foraging 1.495 (seeds 1.32 to 1.59; band from 1.5). At 0.23.0: R1d took mean_seed_mass out (0.153 g against 0.16; 0.10 to 0.18 over the seeds) once seeds stopped filling after maturity; seed_mass_per_plant 7.26 g (was 9.52), harvest_index_tops 0.397. R1b brought pod_set_thermal_time_b10 and density_leaf_area_per_plant back inside their bands; wheat_height_spaced slips just out (0.687 against 0.7). R1c brought stem_share_peak in (0.354) and took harvest_index_tops out (0.374 against 0.4) by chance (16 seeds: −0.009 ± 0.006 against phototropism off). The misses:
- **Thermal-time phenology.** The seven checks against two literature bases that disagree with each other.
- **Wheat.** Five checks: the stand is too short and low-yielding, spaced plants tiller too little.
- **Drought and heat seed size** (drought_seed_size, drought_seeds_per_pod, heat_seed_mass, heat_seed_size): seed number does not adjust to carbon after pod set. At 0.23.0 the mean seed is larger under drought and heat (+19%, +11%), likely because fewer pods share the reserves. Needs a seed abortion rule.
- **drought_shoot_mass** (−62% against −26%): stomata now follow what the roots supply, and the drought pot closes them harder than published pots show. R3's root hydraulics revisit it.
- **split_pot_foraging** (1.495 against at least 1.5, Info, R2b): without the rules 1.13. Code review: drying the east half from day 10 or 15 gives 1.51 and 1.48, so the rules' strength sets it, not late drying. R3's water potential (dry-soil rule and xerobranching) and compensation will move it.
- **nitrate_patch_proliferation** (+48.8% against at least +50%, Info, R2b; seeds +40 to +64%, six-seed 1.48 against 1.01 without the rules): it passed (+51%) until the review's diffusion fix; nitrate that diffuses at its real rate leaves the patch faster. A proliferation factor that can also fall below 1 in poorer cells (review) would sharpen the contrast and keep uniform soil neutral.
- **fine_root_share** (0.097 against 0.1 to 0.4; 0.086 at 0.23.0).
- **density_leaf_area_per_plant** (−30% against −35% to −140%, since R2b; it sat at −35% before).
- **R1b's own targets (Info):** stem_decline_fill (−0.7% against −10 to −35%: the stores empty but stem wood keeps growing from spare sugar, and the field band includes petioles) and feedback_loss_fill (0.41 at 0.23.0 against at most 0.35: the lone pot plant has more sugar than its seeds can take). A fertilized pot still keeps every leaf to maturity (lowest leaves at 16 to 21% of their mobile N). Follow-ups in decision 0040: a stem band without petioles (research), whether cambial growth yields to the seeds in seed fill, CROPGRO-style senescence in proportion to N mined (`SENRTE`).
- **wheat_height_spaced** (0.687 against 0.7, since R1b: stem starch filling after expansion still costs a little height).
- **R1b review (decision 0040):** the pot's seed gain is the four validation seeds' (over seeds 7 to 14, 10.72 → 10.63 g); the stands lose 3 to 6% of seed. Seeds kept filling 8 to 10 days after maturity (late pods finished their 74 + 350 °C·day after the 965 °C·day maturity); fixed by R1d (decision 0042), which took feedback_loss_fill only from 0.43 to 0.41, so the pot's sink limit in seed fill has another cause. Exudation is 0.3 to 0.5% of assimilation against 2.7 to 10% cited.
- **Near-edge M0 passes (R2a review; at 0.24.0):** shade_total_mass −49.1% (gate −50%), shade_internode +28% (seeds −13% to +42%); heat_seeds −33% (weakest seed −25%). Watch them on every model change.
- **R2a review notes for later work:** 38% of the pot's root length sits in the bottom 4 cm at maturity (R3 density saturation will matter there); A/E 3 to 5 mmol/mol, the low end for C3 (keep in view for R3); fine roots could start past a lateral's apical zone instead of when it finishes (R2b).
- **R3a (soil water potential, no model change):** `ps_soil::Retention` (van Genuchten, θ_r = 0, n and α solved from each texture's field capacity and wilting point), `Texture::retention`, `SoilGrid::water_potential` per cell and `layer_water_potentials` for the lens (`SoilLens::layer_water_potential`). The lens holds layers, not cells, so recording costs nothing per cell each hour; the viewer reads a cell's from the grid. The fit runs about 54 bisection steps per call, so R3b should take the curve once per soil per hour. Goldens unchanged.
- **R3b notes for later work (decision 0044):** the viewer draws wilting from the leaves' mean turgor (`ps_mesh::wilt_of_turgor`, none above 0.1; a watered leaf's mean turgor at a bright noon is about 0.1 to 0.4), not from `plant.stress.0`, which is now the stomatal factor. Drought delays expansion but no longer cuts an organ's final size through turgor. R3c's first change (code review): a soil conductance in series with the root's radial link, since `f_dry` keeps full conductance to the wilting point. No capacitance, cavitation or stomatal memory. At noon a 1.1 m pole bean's root axial drop is about 0.5 MPa of its 1.2; the stem about 0.18. R2b's dry-soil rules and xerobranching still read relative available water (spec: move to ψ). Wheat's hydraulic values are the bean's. Thirsty summer: daily bloom watering's pods depend on how far the root-weighted FTSW dips each afternoon (pod abortion still reads FTSW), see docs/game/challenges.md.
- **R5a notes for later work (decision 0046):** `ps_soil::flow` (`FlowTable` per texture, `Column::hour`, bases `Free`, `Seepage`, `Closed`, `Head`); `SoilGrid` gains `Drainage`, a pond per column and a saucer (all in the hash). Water soaking in carries nitrate down as a mixing cell; without it evaporation pumps nitrate into the top layer (the first run halved the plot's Ndfa). Bare-soil evaporation is now realistic and large, which halved thirsty summer's pods (medals 3 / 4.5 / 5.5; an owner decision card asks whether to keep 6 L). Thirsty-summer means move by up to 1.2 pods on 1e-5 changes in the soil curves (plans sit at the pod-abortion threshold). Cost +25 to 38% (pot and plot); the plant side's `psi_and_conductivity` could read the same table (changes results). Lateral flow still diffuses (decision 0019); ponded water evaporates first since R5b (saucer water still does not); the 1-to-3-day field-capacity done-when is read as the end of rapid drainage. R5b (waterlogging) done (decision 0048); R5c (compaction) next.
- **R5b notes for later work (decision 0048):** the oxygen deficit is graded from 10% to 2% air (on/off at 10% stopped every root in a rained loam pot for 11 days); `hypoxic_hours` add the deficit each hour and reset in an hour with none, so a cell that dips above 10% air for an hour starts its count again. Only an axis's tip segment can drown (all segments drowning killed a sealed pot's whole root system); dead laterals are not replaced, a dead fine-root cohort is (its lateral's counter resets). `k_dn` 0.02 h⁻¹ is the spec's guess: the rained validation pot (seed 9) loses 2.9 of 4.2 g of mineral N. Pods in a rained pot move by pod-set luck when hypoxia changes its organ ids. The viewer could show air porosity as a lens and drowned roots browned (viewer threads). R5c (compaction) next.
- **R3c notes for later work (decision 0045):** the simplified summary is built at midnight, so roots grown and cells dried since then are not seen until the next day; `K_comp` = `K_rs` (species `hydraulic_compensation` 1.0) is untuned; the viewer's zoom and speed rule for writing `SetFidelity` belongs to ps-app (viewer thread); nitrogen has one level (R7); soil water still moves between cells by the bucket rules, not by the new conductivity (soil extras could use it). The simplified pot's water step is about 12% of its instructions, almost all retention-curve powers: the hourly simplified step reads only potentials (`cell_potentials`), the detailed one potential and conductivity in one call. Pot cost (review, after #142) 5.7 to 5.9 ms detailed, 4.4 to 4.7 simplified, still over the ~4 ms budget. The three challenges lock at Detailed. Review follow-ups: drought leaves reach −34 MPa and no leaf dies of drying out; `Auto` counts only starting sowings; the bias gate trips beyond about ±5% at four seeds (pots dominate its SE).
- **R2b notes for later work (decision 0043):** in uniform soil root length is 7 to 9% above 0.23.0 (root mass +2 to 7%): the plant's own depletion and the nitrate water carries to its roots still make differences the rules answer. In the 300 ppm pots nitrate gathers in the rooted cells (about 2.3 times the rootless cells by day 45) since uptake saturates. Wheat's plasticity values are the bean's.
- **shade_height** (+31%), **nar_early** (7.5).

**Speed (perf thread, 4 cores).** single_pot about 2.3 ms per sim-day (target 2). plot_30_beans about 24.7 ms (M2 target 20). wheat_stand about 108 ms after #98. The perf thread has a bit-identical speed-up in flight.

## 3. Next steps, in order

1. **Realism specs, in order (owner, 2026-11-05): R1 carbon sinks, R2 root system, R3 plant hydraulics.** R1a, R2a (root length, with the water-uptake fix of #116), R1b (storage, exudation, seed-driven N), R1c (phototropism), the R1d correction (seed growth ends at maturity), R2b (root plasticity), R3a (soil water potential), R3b (the hydraulic network, decision 0044) and R3c (soil conductance, the simplified level, `SetFidelity` and the fidelity report, decision 0045) are done; soil extras are under way (R5a, soil water flow, decision 0046, and R5b, waterlogging, decision 0048, are done; R5c compaction next), then branches on branches and nitrogen along the stem (owner's order) (`docs/research/realism-plan/`, r1-carbon-sinks.md, r2-root-system.md, r3-plant-hydraulics.md; PR #107). Each its own model version, decision record and PR; Code review before merging; the performance thread re-times after each merge.
2. **Seed abortion after pod set** (decision 0032, not done), or a CROPGRO PODUR window that ends pod addition about 10 photothermal days after the first pod (decision 0042 review: ours runs 14 to 18, and the last cohorts end near-empty). The root of the drought and heat seed-size misses and of mean_seed_mass at its band edge. Also consider eight world seeds for single_pot and the response scenarios in validation: three model PRs in a row flipped a pot verdict on seeds 7 to 10 alone.
3. **Per-organ construction costs** (decision 0032): CROPGRO's leaf 1.39, stem 1.29, root 1.25, fruit 1.36 go in with the transport and priority revisit (decision 0009), since the leaf cost alone takes the shade-internode M0 gate to +17%.
4. **WP26, sim side** (its own thread): session save, the replay done-when test, lens data for competition, nitrogen, temperature and carbon flow.
5. **Light follow-ups (decision 0025).**
   - The within-cell "who is on top" limit is now the whole sim-plant ray-tracer error (+3.8%). Fix it before reading stand yields.
   - Heliotropism and a spread of leaf angles: every bean leaflet tilts about 19°. Put each leaf's blade normal in `Meta` for the viewer.
   - Soil-surface radiation under obstacles. A grass strip's footprint is still a disc (decision 0030).
6. **WP25 follow-ups.**
   - `Harvested::mature` per pod.
   - Bud eligibility below a cut.
   - Topping in flower and branch-as-leader.
   - The weather README note: hour labels are the hour start.
7. **Validation note.** `Day(60)` reads `days_since_sowing` 61, and pole_height_day60 passes by a thin margin; say so in the target notes or ask the owner.

## 4. Open owner questions (also in STATUS)

- Answered 2026-11-05 (HANDOFF §2): the WP7 readings (decision 0009) are confirmed; dense wheat speed took option (a), 3 light points per blade (decision 0030); pole bean targets stay 1.5 m at day 60 and 2 m at day 75.
- **WP13.** Re-check the GTX 1070 frame time on the real sim, not the stand-ins. It is viewer-side.

## 5. Gotchas

- **Disk.** The release and debug target dirs reach about 13 GB, and the container allowance is small. Keep one `CARGO_TARGET_DIR`, and delete old worktrees' targets.
- **Squash merges and one long-lived branch.** After a squash merge, sync the branch without a force-push:
  ```sh
  git fetch origin main
  TREE=$(git rev-parse origin/main^{tree})
  git merge -s ours --no-commit origin/main
  git read-tree -u --reset $TREE
  git commit -m "Sync with main after ... squash merge"
  ```
  Stash uncommitted work first. Don't push new commits while a PR from the branch is open, or they join that PR.
- **A main-branch binary panics on newer data.** The species schema is checked strictly. Run each binary against its own checkout's `data/`.
- **Validation is slow.** It takes about 2 minutes now with 4 seeds. Run it in the background. CI's `validation` job uploads the report as the `validation-report` artifact.
- **plantecophys.** That check moved out of the PR path (#77). It runs only when `crates/ps-physio/tests/fixtures/photosyn.csv` or `validation/fixtures/photosyn_*` change. `cargo test` checks the Rust code against that fixture on every PR.
- **Network.** The container's proxy blocked some open weather archives, so `data/weather/` holds a synthetic sample file. Replace it with a station file that has a clear licence.
- **Golden changes.** Any change to a default parameter, an equation, step order or RNG keys changes the goldens. If they change when you didn't intend a model change, you have a determinism bug. Look for HashMap iteration, std transcendentals or unkeyed RNG.
- **Scratch tools.** Seed sweeps were done with ad-hoc scripts, which were not kept. `ps-cli validate` with `seed_offsets` covers the main need. For a one-off sweep, edit the scenario's `world_seed` and run `ps-cli run`.

## 6. The natural-variation work (decision 0027, PR #80, merged)

Done:
- per-plant seed size, emergence threshold (with depth), seedling azimuth, node divergence and leaf-size scatter;
- sowing scatter (1.0 cm, 0.5 cm depth);
- patchy plot soil N (CV 0.25, 30 cm patches).

Unfinished (in STATUS under the variation review):
- the capture sweep and the support-aware scatter (next step 2);
- per-node lattice keys and leaf_scatter keys;
- emergence CV 0.10, if the 7 to 13 day emergence window looks too wide on seed means;
- per-internode length CV.

The entity-key conventions are now documented in `docs/models/random-numbers.md`.
