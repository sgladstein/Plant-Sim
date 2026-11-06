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
   If `cargo test -p ps-game` fails after the bump, re-sweep the medals as in docs/game/challenges.md.

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

## 2. Current state (2026-11-06, R1d PR)

**Model and data.** Model 0.23.0 (decision 0042: seed growth ends at maturity, bush seed fill 289 Cd), species schema 15. Before it 0.22.0 (decision 0041: phototropism, realism plan R1c), 0.21.0 (decision 0040: starch storage, root exudation and seed-driven nitrogen, realism plan R1b, schema 14), 0.20.0 (decision 0037: root length and water uptake, realism plan R2a, issue #116; the thickening potential is set daily, schema 13), 0.19.0 (decision 0036: stem and root thickening, realism plan R1a, schema 12), 0.18.0 (decision 0034 amendment: coupled tolerance 0.05 K, for speed; perf thread), 0.17.0 (decision 0034: the leaf energy balance, WP27, schema 11) and 0.16.0 (decision 0033: broad leaves at 5 light points) and schema 10 (decision 0032). The genetics thread's WP28 step 2 took 0.14.0, schema 9 and decision 0031.

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
- The R1d PR (decision 0042, model 0.23.0): from physiological maturity nothing grows (CROPGRO stops seed growth at R7), and the bush bean's seed fill is 289 Cd (SFDUR 15 photothermal days, was 350). Seed at the end now equals seed at maturity; over eight pot seeds seed at maturity 6.89 → 7.84 g, final 9.60 → 7.84 g.

**Validation at 0.23.0, seed means.** 69 of 94 pass, every M0 gate passes (0.22.0: 70; 0.21.0: 70; 0.20.0: 68 of 92). R1d took mean_seed_mass out (0.153 g against 0.16; 0.10 to 0.18 over the seeds) once seeds stopped filling after maturity; seed_mass_per_plant 7.26 g (was 9.52), harvest_index_tops 0.397. R1b brought pod_set_thermal_time_b10 and density_leaf_area_per_plant back inside their bands; wheat_height_spaced slips just out (0.687 against 0.7). R1c brought stem_share_peak in (0.354) and took harvest_index_tops out (0.374 against 0.4) by chance (16 seeds: −0.009 ± 0.006 against phototropism off). The misses:
- **Thermal-time phenology.** The seven checks against two literature bases that disagree with each other.
- **Wheat.** Five checks: the stand is too short and low-yielding, spaced plants tiller too little.
- **Drought and heat seed size** (drought_seed_size, drought_seeds_per_pod, heat_seed_mass, heat_seed_size): seed number does not adjust to carbon after pod set. At 0.23.0 the mean seed is larger under drought and heat (+19%, +11%), likely because fewer pods share the reserves. Needs a seed abortion rule.
- **drought_shoot_mass** (−62% against −26%): stomata now follow what the roots supply, and the drought pot closes them harder than published pots show. R3's root hydraulics revisit it.
- **fine_root_share** (0.082 at flowering, 0.1 to 0.4): a lateral cohort gets its fine roots only when it finishes. The rest of R2.
- **R1b's own targets (Info):** stem_decline_fill (−0.7% against −10 to −35%: the stores empty but stem wood keeps growing from spare sugar, and the field band includes petioles) and feedback_loss_fill (0.41 at 0.23.0 against at most 0.35: the lone pot plant has more sugar than its seeds can take). A fertilized pot still keeps every leaf to maturity (lowest leaves at 16 to 21% of their mobile N). Follow-ups in decision 0040: a stem band without petioles (research), whether cambial growth yields to the seeds in seed fill, CROPGRO-style senescence in proportion to N mined (`SENRTE`).
- **wheat_height_spaced** (0.687 against 0.7, since R1b: stem starch filling after expansion still costs a little height).
- **R1b review (decision 0040):** the pot's seed gain is the four validation seeds' (over seeds 7 to 14, 10.72 → 10.63 g); the stands lose 3 to 6% of seed. Seeds kept filling 8 to 10 days after maturity (late pods finished their 74 + 350 °C·day after the 965 °C·day maturity); fixed by R1d (decision 0042), which took feedback_loss_fill only from 0.43 to 0.41, so the pot's sink limit in seed fill has another cause. Exudation is 0.3 to 0.5% of assimilation against 2.7 to 10% cited.
- **Near-edge M0 passes (R2a review, at 0.23.0):** shade_internode +27% (seeds −12% to +43%), shade_total_mass −46% (gate −50%); heat_seeds' weakest seed −28%. Watch them on every model change.
- **R2a review notes for later work:** 38% of the pot's root length sits in the bottom 4 cm at maturity (R3 density saturation will matter there); A/E 3 to 5 mmol/mol, the low end for C3 (keep in view for R3); fine roots could start past a lateral's apical zone instead of when it finishes (R2b).
- **harvest_index_tops** (0.397 against 0.4 at 0.23.0), **mean_seed_mass** (0.153 g against 0.16 since R1d), **shade_height** (+31%), **nar_early** (7.5).

**Speed (perf thread, 4 cores).** single_pot about 2.3 ms per sim-day (target 2). plot_30_beans about 24.7 ms (M2 target 20). wheat_stand about 108 ms after #98. The perf thread has a bit-identical speed-up in flight.

## 3. Next steps, in order

1. **Realism specs, in order (owner, 2026-11-05): R1 carbon sinks, R2 root system, R3 plant hydraulics.** R1a, R2a (root length, with the water-uptake fix of #116), R1b (storage, exudation, seed-driven N) R1c (phototropism) and the R1d correction (seed growth ends at maturity) are done; the rest of R2 goes next, then R3 (`docs/research/realism-plan/`, r1-carbon-sinks.md, r2-root-system.md, r3-plant-hydraulics.md; PR #107). Each its own model version, decision record and PR; Code review before merging; the performance thread re-times after each merge.
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
