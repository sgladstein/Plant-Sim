# Plant-Sim: master handoff

Written 2026-11-05 when the project moved to a new account. The reader is a new
coordinator (a Claude session or a person) who has **only this repository**: no chat
history, no project files, no memory. Everything the old project knew that still
matters is in this file or linked from it.

Read in this order: this file, then [docs/STATUS.md](STATUS.md), then
[CLAUDE.md](../CLAUDE.md), then the workstream handoffs in [docs/handoff/](handoff/)
for whatever you pick up.

---

## 1. The owner and the goals

- **Owner:** Scotttt (GitHub `sgladstein`). Windows 10 desktop: i5-6600K (4 cores, no
  SMT), 8 GB RAM, GeForce GTX 1070 8 GB. Desktop only; it does not need to run in a
  browser.
- **What it is:** a simulation-first plant growth sandbox. You watch plants grow from
  seed in sped-up time in a stylized 3D diorama (pots and plots with a soil cutaway),
  with the science visible through toggleable "lenses", and later breed them.
- **Three goals, equally weighted:** pretty to watch, scientifically believable
  (every number traceable to a source, validated against published data), and a game
  (sandbox, then experiments, then challenges, then breeding).
- **Claude writes all the code.** The owner reviews, looks, and decides. No real-plant
  experiments: calibration comes only from published studies and open data.
- **How the owner likes to work:**
  - Threads work autonomously; the owner drops requests and skims later.
  - The owner approves merges to the repo. During the overnight build they said "I
    approve all merges to the repo", and workstreams merged their own PRs on green CI.
    Ask the owner whether that standing approval carries over to the new account
    before relying on it.
  - Decisions that change goals or look go to the owner as one question with options
    and a recommendation. Everything else: do it, then say what you did.
  - On visuals the owner judges **form, framing and setting**, not color grading.
    "Mostly looks like changing colors or filters" was the verdict on a grading-only
    round. A stylized look is fine.

## 2. Owner decisions (answered 2026-11-05)

On 2026-11-05 the owner accepted every recommendation below and approved all merges on
this account: threads merge their own PRs on green CI, and model PRs after an
independent review. So: M0 is signed off; WP20's clip is signed off after WP20 is built
and re-rendered; the pole bean target stays 1.5 m by day 60 and 2 m by day 75; dense
wheat takes option (a), 7 to 3 light points per blade (a model change); the
v0.1.0-pre.1 pre-release stays; the look stays option C (A and B/#79 parked); the WP7
readings of decision 0009 are confirmed as implemented. New questions go to the owner
one at a time, with options and a recommendation.

| # | Decision | Context | Recommendation (accepted) |
| --- | --- | --- | --- |
| 1 | **Sign off M0** (headless core) | Every M0 gate passes (decision 0018, later rechecked); the M0 exit summary is §4 of this file and STATUS "M0 exit". | Sign off. |
| 2 | **Sun-vs-shade clip sign-off** (WP20, M1's designed moment) | Clip in [docs/archive/screens/sun-vs-shade-day4-90.mp4](archive/screens/sun-vs-shade-day4-90.mp4) and contact sheet `sun-vs-shade-contact-sheet.jpg` (re-encoded, smaller). That clip only shows the WP19 time-lapse; WP20's moments (cotyledon unfolding, phototropic lean, wilting and recovery, flowering cue) are not built yet. | Build WP20, re-render with `--timelapse --clean`, then ask. |
| 3 | **Pole bean height** | Owner chose 1.5 m by day 60, 2 m by day 75 (decision 0023). On four-seed means (model 0.10.0) the sim gives 1.38 m at d60 and 1.86 m at d75, with a wide seed spread; part of it is vines missing their stake. Confirm the target stands. | Keep the target; fix the capture sweep, then tune on seed means. |
| 4 | **Dense wheat speed** | `m2/wheat_stand` costs ~110 ms per sim-day, can't play live. (a) fewer light sample points per grass blade, or (b) one light sample per tiller. Both change results. | (a), 7 to 3 points per blade ([docs/perf/handoff.md](perf/handoff.md)). |
| 5 | **Does the public pre-release stay?** | v0.1.0-pre.1 is a public GitHub pre-release of the Windows build (owner approved publishing it). | Owner's call; keep it unless they object. |
| 6 | **Viewer look** | Owner picked **option C "fuller plants" with the natural style "for right now"** (merged, #78). A "illustrated" (branch `claude/project-thread-u7o4cj`) and B "painterly" (PR #79, branch `claude/project-thread-fml7eu`) are parked. | Keep C; close #79 only when the owner says. |
| 7 | WP7 readings (decision 0009) | Two readings of plan text: priority as share of an organ's own reserve per hour; water stress multiplies the stomatal slope. Never confirmed. | Confirm as implemented. |
| 8 | Merge approval on the new account | See §1. | Approved: all merges, model PRs after review. |

## 3. Where things are

| What | Where |
| --- | --- |
| Design plan v3 (complete, kept current with decisions) | [docs/design-plan.md](design-plan.md). §3 model, §3.2 step order, §7 backlog of 36 WPs with done-when, §8 validation, §10 architecture, §11 implementation guide. |
| Progress, per WP, plus every open follow-up | [docs/STATUS.md](STATUS.md) (read first every session) |
| Agent rules (non-negotiable sim rules, code style, pre-push checks) | [CLAUDE.md](../CLAUDE.md) |
| Decision records 0001 to 0034 | [docs/decisions/](decisions/) |
| Model cards, one per process | [docs/models/](models/) |
| Bevy pin and idioms | [docs/bevy-notes.md](bevy-notes.md) |
| CLI | [docs/cli.md](cli.md) |
| Performance budgets and numbers | [docs/performance.md](performance.md), [docs/perf/handoff.md](perf/handoff.md) |
| Validation data schema | [docs/validation-data.md](validation-data.md), data in `data/validation/`, harness in [validation/README.md](../validation/README.md) |
| Viewer look review (3 rounds) and renders | [docs/design/viewer-look-review.md](design/viewer-look-review.md), `docs/design/viewer-look/` |
| Workstream handoffs | [docs/handoff/](handoff/) |
| Archive of the old project files (earlier design versions, all WP reviews, research reports and notes, perf reviews and patches, screenshots, look-option renders) | [docs/archive/](archive/) (see §10) |

## 4. State of the project (2026-11-05, main at the handoff merge)

- **Model version 0.24.0** (`ps_world::MODEL_VERSION`; roots respond to dry and nitrate-rich soil, realism plan R2b, decision 0043, species schema 16; before it 0.23.0, seed growth ends at maturity, decision 0042; before that 0.22.0, phototropism, realism plan R1c, decision 0041, species schema 15; before that 0.21.0, starch storage, root exudation and seed-driven nitrogen, realism plan R1b, decision 0040, species schema 14; before that 0.20.0, root length and water uptake, realism plan R2a, issue #116, decision 0037, species schema 13; before that 0.19.0, stem and root thickening, realism plan R1a, decision 0036, species schema 12; before that 0.18.0, the coupled leaf-temperature solve converges to 0.05 K for speed, decision 0034 amendment; before that 0.17.0, the leaf energy balance, WP27, decision 0034, species schema 11; before that, broad leaves cast rays from 5 points, decision 0033; before that, the research re-tune: development trapezoid, Medlyn g1 5.79, per-organ construction cost keys, decision 0032; before it, the bean genome's 40 loci, decision 0031, grass blades sample light at 3 points, decision 0030, and pole beans find their stake, decision 0029). Species schema 15.
- **M0 (headless core, WP0 to WP12): done**, awaiting owner sign-off. One bush bean in
  a pot, hourly step, carbon is the sole authority on growth, deterministic (golden
  hashes Linux = Windows, debug = release), validation report passes every M0 gate.
- **M1 (two-pot visible proof, WP13 to WP20): mostly done** in the viewer: Bevy app,
  real sim on its own thread, 7 lenses, charts, inspector, capture, time-lapse.
  WP13 passed its frame-time gate on the GTX 1070 (app-side timing on a virtual
  display, not yet checked on a physical monitor; re-run on final main is
  advisable: numbers predate #78). Open: WP20 "the moments" (not started; its
  done-when is the owner's sun-vs-shade clip sign-off), and the experiment and
  model-card panels of WP17. WP25's viewer side (harvest and cut, garden tools, seed
  library, new-run dialog) is done (#76).
- **M2 (plot and sandbox, WP21 to WP27): WP21 to WP25 and WP27 (leaf energy balance)
  done**; WP26 (save/load UI, replay markers, more lenses, carbon flow arrows) open.
  Pole bean (twining), spring wheat, nitrogen with nodules, harvest/prune/obstacles,
  weather files, plant-to-plant variation all landed.
- **M3 (breeding) and M4 (tree, fields): not started.**
- **Validation (model 0.24.0, mean of four world seeds, decision 0026):** 96 checks,
  72 pass (0.23.0: 69 of 94; R2b brought mean_seed_mass, fine_root_share and
  harvest_index_tops back in, took density_leaf_area_per_plant out and added two root
  checks, decision 0043), every M0 gate passes; the pole bean height targets pass since decision 0029.
  One bean spans 21 to 39 g by world seed alone, so every target reads the seed mean.
  Don't tune "~" parameters on one seed. Main misses: thermal-time phenology, wheat
  stand height and yield,
  drought and heat seed size, shade height, nar_early, root_shoot_seed_fill.
- **Speed (4 cores, model 0.19.0):** single pot ~3.5 ms per sim-day (budget 4), 30-bean plot
  about 26 ms (budget 30); budgets re-baselined by the owner 2026-11-06 (decision 0039),
  provisional until timed on the owner's PC. Wheat stand ~110 ms (no budget). Viewer on the GTX 1070, real sim,
  Vulkan uncapped at 2352x1325: 6.55 ms mean, p99 9.61 ms (f4bec25; budget 16.7).
  It predates the #72 lawn and #78 fuller plants, which are untimed.

### Open PRs at handoff

| PR | What | State |
| --- | --- | --- |
| #79 | Viewer look option B painterly | **Parked**, owner decides; do not merge |
| this PR | This handoff and the archive | |

Merged during the handoff: #81 (multi-seed validation, decision 0026), #87 (lateral
roots stay in the soil, decision 0028, model 0.11.0; validation unchanged), #82 (perf
handoff), #83 (plant visuals research), #84 (viewer root-copy fix and viewer
handoff; #86 was a duplicate fix and was closed). Open issue #58: scale nitrogen
uptake capacity with root mass.

### Branches

Work branches are named `claude/project-thread-<id>`, one per old thread. After the
merges above, the only ones with unmerged work are `claude/project-thread-u7o4cj`
(option A illustrated, parked) and `claude/project-thread-fml7eu` (option B painterly,
PR #79, stacked on A). The rest can be deleted once their PRs are merged.

## 5. Architecture

Rust workspace, toolchain 1.97.1 (`rust-toolchain.toml`). Sim crates are engine-free
(no Bevy, egui, wgpu or GPU dependency; `scripts/check_deps.py` enforces it in CI).

| Crate | Role |
| --- | --- |
| ps-units | Unit newtypes; every physical quantity uses one |
| ps-rng | Keyed counter-based RNG (Philox4x32-10) keyed by (world seed, entity, tick, purpose) |
| ps-weather | Solar geometry, weather presets, daily-to-hourly, weather files |
| ps-graph | Organ graph arena, organ types, expansion curves, allometry |
| ps-species | Species and genome schema (RON, unit- and source-checked) |
| ps-soil | 3D bucket soil water, temperature, nitrogen, uptake, FTSW |
| ps-light | Voxel light model: adaptive grid, sun and sky rays, per-leaf coefficients, R:FR |
| ps-physio | Farquhar + Medlyn gas exchange, respiration, thermal time, phenology, carbon allocation |
| ps-arch | Growth forms: bush bean herb, pole bean vine, grass; roots; flowers, pods |
| ps-world | The world: step schedule (design plan §3.2), action log, snapshots, full-state hash |
| ps-session | Runs a World on its own thread; snapshots, time commands, actions |
| ps-export | CSV export |
| ps-validate | Validation harness against `data/validation/` |
| ps-cli | Headless runner: run, hash, bench, replay, validate |
| ps-bench | Benchmarks against the budgets |
| ps-mesh | Engine-free geometry: organ placement, tubes, templates, instancing |
| ps-view | Bevy plugin: diorama scene, scene sync, lenses, camera, sky |
| ps-ui | egui panels |
| ps-app | The `plant-sim` desktop binary |

Data lives in `data/` (species, scenarios, weather, validation). Every parameter has
value, unit and source; `"~"` means a guess to validate. Golden hashes live in
`tests/golden/<platform>/`.

The viewer reads only `ps_view::Garden` and `SkyState`; `ps-app/src/sim_source.rs`
fills them from `ps_session::Snapshot`. Bevy is pinned at 0.20.1 with bevy_egui
0.43.1. If the compiler disagrees with memory about a Bevy API, the compiler is right.

## 6. Build, run, test

```sh
# Sim checks (what CI runs; required before pushing, per CLAUDE.md)
cargo fmt --all --check
# Without wayland/X libraries (cloud containers) exclude the render crates:
cargo clippy --workspace --exclude ps-view --exclude ps-ui --exclude ps-app --all-targets --locked -- -D warnings
cargo test --workspace --exclude ps-view --exclude ps-ui --exclude ps-app --no-fail-fast --locked   # 5 to 8 min; CI uses nextest
python3 scripts/check_deps.py
python3 scripts/check_determinism.py

# Headless runs
cargo run --release -p ps-cli -- run data/scenarios/single_pot.ron --out out
# Goldens exist for three scenarios only (plot at 30 days):
B=target/release/ps-cli   # after cargo build --release -p ps-cli
$B hash data/scenarios/single_pot.ron data/scenarios/two_pot_sun_shade.ron --check-golden tests/golden
$B hash data/scenarios/plot_30_beans.ron --days 30 --check-golden tests/golden
# Regenerate after a model change: same commands with --write-golden tests/golden, then
#   cp tests/golden/linux-x86_64/*.csv tests/golden/windows-x86_64/
cargo run --release -p ps-cli -- validate          # ~2 min with 4 seeds; report in out/validation/
python3 validation/report.py out/validation        # plots and report.html
cargo run --release -p ps-bench --example sim_days 5   # ms per sim-day per scenario

# Viewer (Linux needs libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev)
cargo run --release -p ps-app -- --scenario data/scenarios/two_pot_sun_shade.ron
# Headless screenshot in a container (xvfb + lavapipe; run from the repo root)
xvfb-run -a -s "-screen 0 1600x900x24" env WGPU_BACKEND=vulkan \
  target/release/plant-sim --day 45.5 --lens light --screenshot x.png
```

Useful app flags: `--day D`, `--lens NAME`, `--screenshot FILE`, `--charts`,
`--inspect POT[:ORGAN]`, `--timelapse DIR --day D --to-day D --timelapse-hours 6-19`,
`--clean`, `--focus POT --orbit YAW,PITCH --zoom F`, `--bench-frames N --bench-out
X.csv`, `--window WxH`, `--scenario FILE`, `--data DIR`, `--paused`, `--speed H`,
`--demo` (stand-in plants), `--fps-cap N`. In the app, "New run…" picks a scenario,
weather preset or file, latitude and sowing date. See
[docs/handoff-viewer.md](handoff-viewer.md).

Practical notes:
- A binary built from an older checkout panics on newer species files (the schema is
  strict). Rebuild after pulling, and run each binary against its own checkout's `data/`.
- Run long tests and `validate` in the background. Render crates are excluded from the sim jobs; render.yml covers them.
- `target/debug` balloons to ~27 GB across feature changes; `cargo clean` when disk
  runs out.
- Force-push is often blocked for agents. To push a branch restarted from main:
  `git merge -s ours origin/<branch>` (tree unchanged), then a normal push.

### CI (`.github/workflows/`)

| Workflow | What |
| --- | --- |
| linux.yml | fmt, clippy pedantic, tests (nextest), golden hashes, deps/determinism checks, validation report (artifact `validation-report`), report-only `bench-summary` job |
| windows.yml | Build and sim tests on Windows; golden hashes must equal Linux |
| render.yml | Clippy and tests of the render crates; uploads `plant-sim-windows` (release exe built with `--features static-dxc`) |
| plantecophys.yml | Cross-check against the plantecophys R package; runs only when its fixture changes, and weekly |
| prerelease.yml | Manual: re-zips a green render run's `plant-sim-windows` artifact and publishes a GitHub pre-release with SHA-256s |

### Windows builds and releases

- Every shipped Windows build must use `--features static-dxc`. DX12 cold start
  falls from ~176 s (default build, f4bec25) to 5 to 9 s (static DXC, d5ab169). Building it locally needs the Visual Studio ATL
  component (atls.lib); the owner's PC doesn't have it, so use the CI artifact.
- Backend: wgpu picks Vulkan on the owner's PC, about 2x faster than DX12 there;
  `WGPU_BACKEND=dx12` is the fallback. Present mode defaults to mailbox (Bevy falls
  back to immediate, then FIFO), capped at twice the monitor refresh; `--fps-cap N`
  changes the cap, 0 turns it off. FIFO showed ~30 ms spikes every ~14 frames on the
  1070.
- To cut a release: dispatch `prerelease.yml` on main with `run_id` = the id of the
  latest green `render` run on main and `tag` = the next tag (v0.1.0-pre.2, ...).
  The GitHub MCP has no create-release tool; this workflow is the way.
- First release: v0.1.0-pre.1 = main d5ab169, zip SHA-256
  `fe288957d02b5491c560895227e5db478183d7ac2d415ff9b53a7a1696c03316`, plant-sim.exe
  SHA-256 `432a37eeb0159697df97ebaa5806b7e4be5294d0d81d040b5c499c9261010ebc`.
- Running on the owner's desktop: see [docs/handoff/gpu-desktop.md](handoff/gpu-desktop.md).
  Expect to get the owner's explicit approval for building, running and downloading
  on that machine at the start of each new session; it doesn't carry over between
  sessions or accounts.

## 7. The rules that keep the sim honest

These are in CLAUDE.md; the short version, because breaking one costs a day:

- f64, libm transcendentals, no HashMap iteration, all randomness via ps-rng keyed,
  fixed-chunk parallel reductions with in-order folds. Determinism is checked: Linux
  and Windows golden hashes must match bit for bit.
- Changing an equation, a default parameter or the step order: write a decision
  record, update the model card, bump the model minor version, regenerate goldens
  (`ps-cli hash ... --write-golden tests/golden`) on Linux, and let windows.yml
  confirm.
- Performance PRs are **bit-identical** (goldens unchanged). Anything that changes
  results is a model change and goes through the above.
- Validation targets are not widened to make a miss pass. A miss is either fixed by
  model work or recorded as an Info miss with the reason.
- Research claims carry tags: [F] read in a source, [K, to verify] from memory,
  [I] inference. Check [K] before putting a number in `data/`.

## 8. Workstreams (reusable thread roles)

The old project ran as one coordinator plus parallel threads. Re-create them as
needed; each handoff file is written as instructions for a fresh thread.

| Workstream | What it does | Handoff |
| --- | --- | --- |
| **Implementation** | Builds WPs in plan order, one WP per branch and PR, issue "WP<n>" as acceptance criteria, decision records, model bumps, goldens, updates STATUS. Messages Review when a PR is up. | [handoff/implementation.md](handoff/implementation.md) |
| **Review** | Independent review of every WP PR: re-runs the sim, checks against the plan and research, writes write-ups to `docs/reviews/<name>-review.md` (old ones in `docs/archive/reviews/`), sends fixes back to Implementation, posts a short verdict to the owner. Ran on a different model from Implementation for independence. | [handoff/review.md](handoff/review.md) |
| **Performance** | Benchmarks each WP against the budgets, finds bit-identical speedups, opens small non-conflicting PRs, times the viewer on the owner's GPU. | [perf/handoff.md](perf/handoff.md) |
| **Research** | Deep literature research before a milestone: parallel topic researchers, one synthesized report with a ranked change table per WP, findings become decision records. | [handoff/research.md](handoff/research.md) |
| **Viewer** | The Bevy crates (ps-mesh, ps-view, ps-ui, ps-app), M1 WPs and viewer sides of M2 WPs, CI render builds and releases. Never touches sim crates. | [handoff-viewer.md](handoff-viewer.md) |
| **Look review / look options** | Prototype a visual direction on its own branch, show before/after renders, owner picks. | [handoff/look-review.md](handoff/look-review.md), [handoff/painterly.md](handoff/painterly.md), [handoff/fuller-plants.md](handoff/fuller-plants.md), [handoff/visuals-research.md](handoff/visuals-research.md) |
| **GPU desktop** | A Claude Code session on the owner's Windows PC for real-GPU checks (screenshots, frame times). | [handoff/gpu-desktop.md](handoff/gpu-desktop.md) |

How they coordinated: Implementation messaged Review on each PR and Viewer when a sim
change affected the snapshot; Review sent fixes back to Implementation; Performance
sent findings to Implementation; Research sent WP-keyed findings to Implementation and
checks to Review. Only one thread edits STATUS for a given WP. Shared write-ups went to
a project folder that no longer exists; from now on put them in the repo: reviews in
`docs/reviews/`, new research in `docs/research/<topic>/` (as `docs/research/plant-visuals/` does). Leave
`docs/archive/` frozen.

## 9. Next steps, in order

**Implementation (sim):**
1. Vine capture sweep: capture sweeps a fixed 6 cm, so with sowing scatter about
   1 pole bean in 16 never finds its stake, and the seed-mean pole height partly
   measures those misses. Batch the other variation-review items into the same model
   bump ([handoff/implementation.md](handoff/implementation.md) step 2).
2. Revisit the transport/priority rule first (STATUS: WP22 and WP23 follow-ups, the
   WP7 sink-limitation question); re-tuning phloem conductance per species before that
   gets undone by it. Then re-tune "~" parameters on the four-seed mean; pole_height_day60,
   drought_shoot_mass and others sit at band edges since the leaf-angle fix. Fold
   research items 1 to 4 ([handoff/research.md](handoff/research.md): gm/Vcmax basis,
   a 7/25/33/42 °C development trapezoid, construction cost per organ, Medlyn g1)
   into the same model bump, since they move the same calibration.
   Validation follow-ups from #81: sort validate jobs by plant count × days,
   descending, and give each job a one-thread rayon pool when several run at once.
   (Roots through the pot wall: decision 0028. Roots drawn upward: viewer fix, #84.)
3. WP26: save, load and replay already exist; missing are a non-stopping
   `Session::save_ron`, the plot save/replay done-when test, and lens data (light
   share, organ N, per-organ carbon flux) ([handoff/implementation.md](handoff/implementation.md)
   step 3); the UI half is the viewer's. WP27 (leaf energy balance) is done (decision 0034).
   Then the realism specs R1 carbon sinks, R2 root system, R3 plant hydraulics, in that
   order (owner, 2026-11-05; `docs/research/realism-plan`), then M2 exit: 30 plants at 60 fps and
   under 30 ms per sim-day (decision 0039), density plateau (still weak: -43% vs -70%).
4. M3 (WP28 to WP32): genome, pollination, breeding bench, trait discovery,
   challenges. Research round 2 has the genetics. WP28 is bound by decision 0023
   items 4 to 6 (light linkage, two-tier mutation 1e-3 QTL / 1e-5 named, a seed-colour
   rule table); the review workstream checks WP28 against them.
5. Smaller open follow-ups are listed in STATUS "Review follow-ups" (self-shading
   within a light cell, transport vs priority, wheat stand too short, pod set reading
   N, uptake scaling, issue #58).

**Viewer:** build WP20 (the moments), then re-render the sun-vs-shade clip for the
owner's sign-off; re-run the WP13 frame timing on final main on the 1070 (#72 lawn and
#78 per-leaf petiole meshes are untimed); show `Plant::individual` (seed factor,
emergence, azimuth) in the inspector (decision 0027); the rest of WP17 (experiment
panel, model-card panel, assimilation and transpiration charts once the sim writes
them to the daily rows); then the look follow-ups (leaf pose by age needs a sim change; plate view as a screenshot
mode; canopy shade). Details in [handoff-viewer.md](handoff-viewer.md).

**Performance:** bit-identical speedups only (decision 0039 re-baselined the budgets to
4 ms pot and 30 ms plot); measure each realism step; time on the owner's PC when it is
connected; wheat after the owner's choice. Details in [perf/handoff.md](perf/handoff.md).

**Research:** WP27 leaf energy balance data; M3 genetics gaps (cM distances, seed-colour
epistasis); weed competition for challenges. Details in [handoff/research.md](handoff/research.md).

## 10. The archive (`docs/archive/`)

Copied from the old project's shared folder, which does not move to the new account.

| Folder | Contents |
| --- | --- |
| `design/` | Earlier design versions (v1, v2, v3 review and expansion, "final", architecture note). Superseded by docs/design-plan.md; kept for the reasoning behind choices. |
| `reviews/` | Every review write-up, 19 files: WP4 to WP12 (incl. WP11 data and harness), WP16 session, WP21 to WP25, WP7 fixes, light leaf angle, variation. Their open findings are already copied into STATUS "Review follow-ups". |
| `research/` | Round 1 `plant-simulation-science-review.md`, round 2 `m2-m3-bean-science.md`, per-topic notes, DSSAT source files (BSD-3), the validation data generator scripts. OpenSimRoot XML (GPLv3) is left out; see `research/source-files/README.md`. |
| `perf/` | Perf reviews 1 and 2 and the patches (all landed). |
| `screens/` | Viewer screenshots over time (JPEG, downscaled) and the sun-vs-shade clip (re-encoded). |
| `viewer/look-options/` | Before/after renders for option C fuller plants and option B painterly. |

The plant visuals research (report, renders, mockup patch) is in
`docs/research/plant-visuals/`.

## 11. What was lost in the move

- Chat history of every thread; this file and the handoffs replace it.
- Published claude.ai artifacts (the design plan page, review boards, M0 exit summary).
  Their content is in the repo (design plan, reviews, research, look review).
- The original OpenSimRoot XML files (refetch, see above) and full-size PNG screenshots
  (JPEG copies kept).
