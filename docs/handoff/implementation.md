# Implementation workstream: handoff

Written 2026-11-05 by the implementation thread (the sim crates, ps-world and below, plus ps-cli,
ps-validate, ps-export, ps-session and data/). It covers everything a fresh thread needs to
keep implementing the design plan from the repository alone.

## 1. How this workstream runs (instructions for a fresh thread)

**Read first, every session:** `CLAUDE.md` (the non-negotiable sim rules), then `docs/STATUS.md`
(which WP is done, in flight and open, plus every open follow-up), then the WP's row in
`docs/design-plan.md` §7 and the model cards it touches in `docs/models/`.

**One work package per branch and PR.**
- The backlog is `docs/design-plan.md` §7, in plan order. M0 and M2 WP21 to WP25 are done. WP26 and WP27 finish M2; M3 (WP28 to WP32) follows.
- Each WP gets a GitHub issue titled `WP<n>: ...`, holding its done-when as acceptance criteria. Create it if it is missing; WP26 and WP27 have none yet. Close it with `state_reason: completed` when the PR merges.
- Review follow-ups that are not a WP get their own small PR, such as the light fix (#75), the validation harness (#81) and the root fix. Record them in STATUS.

**Every model change carries five things in the same PR:**
1. A decision record, `docs/decisions/NNNN-title.md`, from `TEMPLATE.md`. Number it as the next free number; 0030 is the latest.
2. A model card update in `docs/models/` for every equation or default that changes.
3. A minor bump of `MODEL_VERSION` in `crates/ps-world/src/lib.rs`, plus `model_version` in every scenario under `data/scenarios/` (sed them all).
4. A species schema bump in `crates/ps-species/src/schema.rs`, if a species file key changes. Schema is at 8.
5. Regenerated goldens:
   ```sh
   B=target/release/ps-cli   # after cargo build --release -p ps-cli
   $B hash data/scenarios/single_pot.ron data/scenarios/two_pot_sun_shade.ron --write-golden tests/golden
   $B hash data/scenarios/plot_30_beans.ron --days 30 --write-golden tests/golden
   cp tests/golden/linux-x86_64/*.csv tests/golden/windows-x86_64/
   ```
   Windows goldens equal Linux ones. CI checks both platforms, so a mismatch is a determinism bug.

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

## 2. Current state (2026-11-05, end of this thread)

**Model and data.** Model 0.11.0 if the root PR is merged (else 0.10.0). Species schema 7.

**Merged today:**
- #73: WP25 (harvest, obstacles, supports mid-run, weather files, apical hold; decision 0024).
- #75: leaf-angle light (each leaf intercepts by its own |n·d|, obstacles intersected analytically; decision 0025).
- #77: the plantecophys R check is its own workflow, which runs only when the photosynthesis fixture changes, weekly, or by hand. It installs R from apt and takes 30 s.
- #80: plant-to-plant variation (decision 0027, from a separate thread).
- #81: multi-seed validation (decision 0026).

**In flight: lateral root fix, PR #87 (decision 0028, model 0.11.0).**
- Lateral root cohorts now keep their tips in active soil. Before, 31% of them poked out of the pot wall or bottom.
- On branch `claude/project-thread-ur2udi`. Validation is unchanged by it: 65 of 87 on seed means.
- The viewer half of that bug, drawn copies of a cohort turned upward out of the soil, belongs to the viewer workstream: `ps-mesh` `axes.rs` `copy_transforms` should turn copies about vertical and clamp them to the pot.

**Validation at 0.10.0, seed means.** 65 of 87 pass, every M0 gate passes. The misses:
- **Thermal-time phenology.** Flowering, pod set and maturity checks miss against two different literature bases.
- **Pole bean.** Height is 1.38 m at day 60 (target 1.5) and 1.86 m at day 75 (target 2.0), with a wide seed spread.
- **Wheat stand.** Too short and low-yielding.
- **Drought and heat seed-size responses.**
- **Shade height.**
- **nar_early.**
- **root_shoot_seed_fill.**

**Speed (perf thread, 4 cores).** single_pot about 2.3 ms per sim-day (target 2). plot_30_beans about 24.7 ms (M2 target 20). wheat_stand about 110 ms.

## 3. Next steps, in order

1. **Merge #87 (root fix) if it is not merged yet.** Check CI first.
2. **Vine capture sweep (variation review).** Capture sweeps a fixed 6 cm, so with sowing scatter some pole beans never find their support and lodge. Make the sweep grow with free tip length. Don't let sowing scatter move a seed sown within 5 cm of a support. Batch the other variation-review items into the same model bump (patch lattice keyed by (soil, ix, iy), leaf_scatter keyed by node and leaf index, per-internode length CV). They are listed in STATUS. Re-read pole_height_day60 on seed means afterwards.
3. **WP26, sim side.**
   - Save/load already exists: `World::snapshot_ron` / `from_snapshot_ron`, `World::replay`, and ps-cli `replay`.
   - Missing:
     - a session-level save that doesn't stop the sim (`Session::save_ron`);
     - the done-when test, "replay of a saved plot matches its hash": plot_30_beans with WP25 actions (harvest, cut, obstacle, support), saved mid-run, loaded and finished, must match both the uninterrupted run and a replay of its log;
     - lens data: competition (each plant's light share, or absorbed against open-sky potential), nitrogen (organ N concentration and soil mineral N, both already in state), temperature (soil temperature now; leaf temperature after WP27), and per-organ carbon import and export for the flow arrows (a new derived field that the hash leaves out).
   - The UI half goes to the viewer workstream.
4. **WP27, leaf energy balance.** Leaf temperature from radiation, wind and transpiration, coupled to the gas-exchange solver. Done-when: the leaf-air difference is in the literature range on a hot still day. It changes almost every golden.
5. **Light follow-ups (decision 0025).**
   - The within-cell "who is on top" limit is now the whole sim-plant ray-tracer error (+3.8%). Fix it before reading stand yields.
   - Heliotropism and a spread of leaf angles: every bean leaflet tilts about 19°. Put each leaf's blade normal in `Meta` for the viewer.
   - Soil-surface radiation under obstacles.
6. **WP25 follow-ups.**
   - `Harvested::mature` per pod.
   - Bud eligibility below a cut.
   - Topping in flower and branch-as-leader.
   - The weather README note: hour labels are the hour start.
7. **Then M3 (WP28 to WP32).** Decision 0023 fixed the genome choices: two-tier mutation 1e-3 QTL / 1e-5 named, light linkage, a seed-colour rule table.

## 4. Open owner questions (also in STATUS)

- **WP7 (decision 0009).** Two readings of the plan's text to confirm:
  - priority is the share of an organ's own available reserve it may spend per hour;
  - the water stress factor multiplies the stomatal slope.
- **Dense wheat speed (111 ms per sim-day).** Fewer rays per leaf, or one light sample per tiller? Both are model choices, and the owner has not picked.
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
