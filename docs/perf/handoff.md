# Performance work: handoff

This page is for whoever picks up performance work, without access to the
project's earlier chats. It covers what the budgets are, how to measure, the
rules for speedups, what has been done, and what is open. The live numbers are
in docs/performance.md, and the STATUS perf line says where each budget stands.

## Budgets (docs/design-plan.md §7)

| Scope | Budget | State after the 2026-11-05 bit-identical pass (4-core Xeon container about 15% slower than earlier ones) |
| --- | --- | --- |
| One bush bean in a pot (M0) | under 2 ms per sim-day | 2.06 to 2.19 ms (main 2.66 in the same container): at the budget, likely under it on the earlier containers |
| 30-bean plot (M2) | under 20 ms per sim-day | 22.7 ms (main 27.2 to 29.1 in the same container), about 2 ms over after scaling |
| Viewer frame (M1) | under 16.7 ms on the owner's GTX 1070 at 2560 × 1440 | 6.55 ms mean, p99 9.61 (Vulkan uncapped, 2352 × 1325, real sim, f4bec25); re-time after #72 and #78 |

There are no budgets for the pole beans (about 4 ms per plant per sim-day) or the
wheat stand (about 110 ms per sim-day for about 200 plants on 1 m²). The wheat
stand cannot play live at a sim-week per second.

The owner's PC is an i5-6600K (4 cores, no SMT) with 8 GB RAM and a GTX 1070.

## How to measure

- **Whole-sim speed:** `cargo run --release -p ps-bench --example sim_days 5`.
  It prints ms per sim-day (fastest and median) and the state hash for each
  committed scenario. The CI linux workflow runs it in the `bench-summary` job
  and writes the table into the job summary. That job is report-only and can
  never fail the build, because shared runners are too noisy to gate on.
- **Hot paths:** `cargo bench -p ps-bench` (criterion; benches are light, soil,
  physio, mesh and world).
- **Where the instructions go:** callgrind works in the cloud container (perf
  does not). Build a small example that runs one scenario, then run
  `RAYON_NUM_THREADS=1 valgrind --tool=callgrind ...` and use
  `callgrind_annotate --inclusive=yes`. Callgrind overcounts memset and memcpy
  (rep stos is counted per byte), so check wall time for anything allocation-heavy.
- **Where wall time goes:** add temporary `Instant` laps around the stages in
  `World::step_hour_on_worker` (water status, class tables, demand, soil step,
  uptake, hourly growth, daily step) and sum them over a run. Remove them before
  committing.
- **Comparing two commits:** build `sim_days` on each and copy the binaries out.
  The binary reads `data/` from the source tree at run time, so check out the
  matching commit before running each binary.

## Rules for speedups

Performance PRs here are **bit-identical**: the three golden state hashes
(single_pot, two_pot_sun_shade, plot_30_beans) must not change. A change that
alters results is a model change. It belongs to the implementation work, needs a
decision record, and needs the owner's call.

Techniques that kept bits identical:

- Keep the same order of floating-point operations, and accumulate densely in the
  order a stable sort would have added the values.
- Adding +0 is exact as long as the running sum is never −0. That lets a missing
  term become a branch-free +0.
- Share the results of pure functions keyed by the bits of their inputs (the
  class tables).
- Overlap work that doesn't change results, such as computing the daily hash on
  a snapshot while the next day steps.

`state_hash` covers every daily hash, so equal end hashes on the three scenarios
are the check. The workspace golden tests check the same thing in CI.

## Done (all bit-identical)

| PR | What |
| --- | --- |
| #19, #22, #28 | ps-bench crate and benches |
| #47 | M0 under 2 ms after WP12: unrolled ray traversal, footprint reuse, class-table temperature terms, full Q10 cache |
| #53 | 30-bean plot from 28.8 to about 20 ms: shared class tables, plot footprint reuse, dense uptake totals, parallel after-light step, daily hash off the critical path |
| #60 | Root reach and joint uptake after WP24 (nitrogen): dense accumulation instead of sorting |
| #62 | Hourly soil drainage and lateral exchange (soil step 20.6 to 17.5 µs per hour), and the CI bench summary |
| #64, #70 | Pole and wheat scenarios added to the CI speed table |
| this PR | mimalloc in the binaries, ray tails cut by per-direction shadows, per-axis footprints, own-cell index, parallel daily nitrogen with a summed-volume reach, lean soil storage sums: single_pot -20%, plot -17%, wheat_stand -14% |

docs/performance.md has the details of each, including measured floors.

## Things tried that were slower (don't retry)

- Branchless axis choice in the ray loop (an indexed `t_max` array): 35% slower than
  the unrolled branches.
- Per-layer horizontal bounds checked at each z step of a ray: saves 7% of steps on
  the plot and costs as much. The sheared shadow (done) replaced it.
- Shadow bins narrower than `cell · (1 + |slope|)`: fewer steps, more build time.

- Overlapping the soil step with the demand pass (`rayon::join`).
- Parallel class-table builds and parallel water status: a parallel section's
  wake-up costs more than the work.
- A branch-free drain (it divides for every cell, and most cells have no excess).
- Splitting the nitrogen transform into a rate pass and an update pass, and a
  branch-free `water_factor`. The transform is bound by its four divisions per cell.
- Precomputed lateral pair masks.
- For the plot's hourly growth section, which scales only 1.8× on 4 cores: spinner
  jobs to keep rayon workers awake, `with_max_len(1)`, and running the daily hash
  on its own thread. Rayon wake-up does cost about 24 µs per section in the cloud
  VM, but removing it didn't make the section faster.

## Open work, in order

1. **Done 2026-11-05: profile the light fit after #75** (decision 0025); see the
   pass in docs/performance.md. Was: The fit now builds
   footprint depth per direction (sun-up hours plus 16 sky sectors, about 31). It
   costs about 6 to 7% on the single pot and the plot. Look for bit-identical cuts
   in the logs per distinct leaf area density and in the per-direction adds.
2. **Plot over budget** (now 22.7 ms in a slower container, about 2 ms over). The
   biggest bit-identical item left is the hourly growth section's overhead, about
   2.7 ms per sim-day of wake-up and scheduling across 2,160 short parallel
   sections (spinner jobs were tried before and did not help). Was 24.5 ms against 20. Remaining cost per sim-day: daily
   light and growth about 9.7 ms, hourly growth about 8.5 ms, demand 1.9, soil 1.7,
   uptake 0.7. The daily hash is a latency-bound chain, about 3.6 ms of serial
   time, and runs overlapped. Going clearly under 20 ms probably needs an owner
   decision: fewer rays per leaf, or a cheaper hash definition (new golden hashes).
3. **Wheat stand: open choice for the owner.** It runs at about 110 ms per sim-day,
   so it can't play live at a sim-week per second (that needs about 0.8 s of sim
   per wall second). Light rays are about 43% of instructions, hourly growth about
   33%, and the hash about 6%. No bit-identical cut is big enough. The options are:
   - (a) fewer sample points per leaf for grass blades (today 7 points per leaf ×
     (sun-up hours + 16 sky sectors) rays each); or
   - (b) one light sample per tiller instead of per blade.

   **Recommendation: (a).** Narrow blades don't need 7 points across, and it keeps
   per-blade light and leaf angle, which decision 0025 now depends on. Going to 3
   points cuts the stand's ray work by about 4/7, about a quarter of total run time.
   Either option changes results, so it needs a decision record, new golden hashes,
   and a re-run of the wheat validation targets.
4. **Viewer on the 1070:** time single_pot, plot_30_beans and m2/wheat_stand
   after the fuller-plants look (#78, the owner's pick). Use 2560 × 1440, Vulkan
   Mailbox, vsync off, and 1,200 frames, and report mean and p99 frame time and GPU
   busy %. #78 gives every living bean leaf its own petiole mesh, so the plot gains
   thousands of draws. If that matters, merge each plant's petioles into one mesh.
   The painterly option (#79) adds one full-screen pass of about 69 texture reads
   per pixel, estimated at 2 to 3 ms per frame. Half-resolution Kuwahara is the
   fallback.
5. **Validation run time:** `ps-cli validate` runs every scenario at 4 seeds
   (about 2 minutes on 4 cores; the wheat stand dominates). Watch the CI
   validation job's duration.

## Viewer numbers on the owner's PC (GTX 1070)

- Stand-in plants, two pots at 2560 × 1440, vsync off, 1,200 frames: Vulkan mean
  2.70 ms (p99 2.94), GPU about 51% busy; DX12 mean 5.78 ms, GPU about 98%.
- After the new look (main 58807c6), still stand-in plants: Vulkan Mailbox mean
  4.0 ms, p99 4.4 ms, no spikes. Vulkan Fifo shows 31 ms spikes about every 14 frames from present
  cadence. DX12 Fifo p99 is 32.7 ms.
- Real sim (f4bec25, Vulkan uncapped, 2352 × 1325): mean 6.55 ms, p99 9.61 ms.
  This is the number to compare against the 16.7 ms budget.
- Cold start: Vulkan about 5 s, DX12 about 180 s (pipeline compilation).
  Recommended default: Vulkan with Mailbox, falling back to Fifo, and DX12 only
  without Vulkan.
- Not yet timed: the #72 lawn and the #78 fuller plants (see open item 4).

## Practical notes

- In the cloud container, `cargo test --workspace` (excluding ps-app, ps-view and
  ps-ui) takes more than 20 minutes. Run it in the background. Run clippy locally
  with the same excludes; render.yml covers the render crates.
- Before pushing, run the CLAUDE.md checks: fmt, clippy pedantic, tests,
  check_deps and check_determinism.
- **Pushing a restarted branch:** after a PR merges, restart the work branch from
  main. If force-push is blocked, run `git merge -s ours origin/<branch>` (check
  that the tree is unchanged), then do a normal push.
- **Rayon in a VM:** waking sleeping workers costs about 24 to 75 µs per parallel
  section, and a short `par_iter` after an idle gap can be slower than serial.
  Measure before parallelizing small work.
- **Deferred hash:** never wait on it with a blocking `recv` inside the pool;
  `finish_hash` spins on `try_recv` with `rayon::yield_now`, or the pool can deadlock.
- **History:** the early perf reviews (WP6 to WP8) and their patches, all landed,
  are archived under docs/archive/ by the handoff PR.
