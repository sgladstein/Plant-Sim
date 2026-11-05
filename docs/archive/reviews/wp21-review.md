# WP21 review: plot soil and shared plot light (PR #50, decision 0019)

Reviewed at commit 3297869 on `claude/project-thread-ur2udi` against `origin/main` (WP12,
model 0.2.0). Reviewer: Fable review thread. Date: 2026-11-05.

## Verdict

**Approve.** The plot soil, the shared light grid and the parallel stepping are correct,
tested and documented. The per-voxel light rejection is well argued and the numbers back
it. The speed target is not met (the done-when says under 20 ms per sim-day for 30
plants), and the decision record says so plainly; this reviewer measured 20.9 ms over the
first 60 days on 4 cores and 41.7 ms on one thread, so the gap on 90 days is the late
canopy. The density response is the real M2 model gap, and the cause is not what the
decision says (see finding 1).

## What I checked

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| clippy, sim and CLI crates, `-D warnings` | clean (CI covers the Bevy crates; they do not build in this container) |
| `cargo test` on the sim and CLI crates | all pass, including the three new plot tests and the two new light tests |
| `scripts/check_deps.py`, `scripts/check_determinism.py` | pass |
| golden hashes, `single_pot`, `two_pot_sun_shade` (90 d), `plot_30_beans` (`--days 30`) | match |
| `ps-cli validate` | 70 checks: 54 pass, 16 Info misses, 0 failing, 0 known |
| pots unchanged: `daily.csv` and `organs.csv` of both pot scenarios from a `main` build against this branch | byte-identical, as decision 0019 claims |
| thread independence: `a_plot_is_independent_of_the_thread_count` plus `RAYON_NUM_THREADS=1` bench | same hashes |
| `ps-cli bench plot_30_beans --days 60` | 20.9 ms per sim-day on 4 cores, 41.7 ms on 1 |

A note on the golden check: running `plot_30_beans` for its full 90 days against the
30-day golden fails "at line 2", because `duration_days` is in the chained hash. That is
by design (world-and-schedule.md says so), but the error text should say the golden was
made for 30 days. See finding 7.

## The five points the implementer asked about

**Per-voxel light rejection.** Agree. The table in 0019 is the right experiment: the same
two ray-tracer fixtures, four methods, plant total and per-leaflet counts. Per-leaf rays on
the 3 cm plot grid beat every voxel variant on both fixtures. The cost argument also holds
at bean scale (about one leaflet per occupied voxel). Two requests: keep the scratch
implementation reachable (a tag or a branch named in 0019) so the numbers can be rerun when
the tree needs voxels at M4, and pin the plot-grid result in a test, which the PR already
does (`plot_grid_light_against_ray_tracer`). The plan's §3.5 text ("at M2 the coefficients
move to per-voxel transmittance") should be amended to point at 0019 when the plan is next
edited, so WP21's done-when is not read as unmet.

**Lateral fraction as a diffusivity.** Correct. `f = 0.1 (0.02/Δx)²` is `D Δt/Δx²` with
`D = 4e-5 m² h⁻¹`, which is at the low end of unsaturated soil diffusivities (1e-4 to 1e-2
cm² s⁻¹ is 3.6e-5 to 3.6e-3 m² h⁻¹) and so defensible for a loam near field capacity. The
cap of 0.25 is exactly the explicit 2-D stability limit for four neighbors (a cell can
give away at most all of its excess in one hour, never go negative). Pots are bit-identical
(the test compares bits). Fine.

**Day split.** Sound. `day()` is literally the two halves joined with `coeffs = None`, so
pots cannot drift. `day_before_light` does everything up to the layout and root cells;
`day_after_light` does the light rebuild, nitrogen, the report and the daily resets. The
one coupling to watch is that `light_inputs()` (world side) and `leaflets()` (inside
`rebuild_light`) must produce the same leaflet list in the same order, since the shared
coefficients are matched by position. Today both call `leaflets()` and nothing between
them changes geometry, so it holds, and the `debug_assert_eq!` on the lengths catches a
count mismatch in debug. A stronger guard is cheap: return the coefficient count from
`day_after_light` or have `rebuild_light` take `(points, owner)` from the world so the
list is computed once. Not blocking.

**One G per plot from the first plant.** Acceptable for one species per plot. It is
another reason to resolve the open G question from decision 0018 (G from the drawn leaf
normals per direction) before WP23 adds a second species, because then G becomes per leaf
and the "first plant" rule disappears on its own. The model card says this limit; good.

**Weak density response.** Agree it is the M2 gap, but the diagnosis in 0019 ("sink
limitation and narrow crowns") misses the main cause. See finding 1.

## Findings

Ordered by weight. None blocks the merge.

### 1. Branching does not respond to density; that is why leaf area per plant barely falls

From the validation runs of `m2/density_5` (16 plants, 4.9 per m²) and `m2/density_35`
(49 plants, 34 per m²), means over plants:

| At day 60 | density_5 | density_35 | change |
| --- | --- | --- | --- |
| branches | 5.8 | 5.7 | 0 |
| leaves | 25.2 | 15.3 | −39% |
| leaf area per leaf, cm² | 84 | 87 | 0 |
| leaf area per plant, m² (max) | 0.215 | 0.161 | −25% |
| total dry mass, g (final) | 28.2 | 18.9 | −33% |
| pods, seed mass (final) | 11.3, 10.6 g | 6.2, 5.4 g | −45%, −49% |
| feedback_loss at day 40 | 0.47 | 0.29 | more source-limited |
| stand LAI at peak | 1.05 | 5.5 | |

Mass, pods and seed respond to density in the right direction and roughly the right size
(DSSAT field beans at 25 to 52 per m² carry 4.5 to 4.9 pods per plant). The dense plants
are less sink-limited (feedback_loss 0.29 against 0.47), so shade is costing them
assimilate. What does not move is architecture: every plant makes the same 5.8 branches
and the same 85 cm² leaves at both densities. Real beans at 30 per m² make one or two
branches and smaller leaves; the −70% in Clavijo Michelangeli 2019 is mostly fewer and
smaller leaves per plant, not shorter-lived ones. Two consequences:

- The stand LAI at 34 per m² is 5.5. The M2 exit says "leaf area per unit ground plateaus
  with density as in §8.3", and a bean canopy plateaus around 3 to 4. This is the number
  to watch, more than the per-plant −70%.
- WP24 nitrogen will shorten shaded leaves' lives and should help, but it does not touch
  bud outgrowth. The cheaper lever is the R:FR signal the light model already computes per
  leaf (`ShadeSignal.rfr`): bud outgrowth at a node should scale with the shade factor at
  that node (phytochrome-mediated apical dominance), and leaf potential area should fall
  with the shade signal rather than only SLA rising. Both are species parameters with
  sources in the M2/M3 research notes. Suggested home: WP24 or a small WP22 item, with the
  density check promoted to an M2 gate once branching responds.

### 2. The density scenarios' plot sizes make the stand edges dominate

`density_5` is 4 × 4 plants on 1.8 m with the outer row 22.5 cm from the edge, and
`density_35` is 7 × 7 on 1.2 m. In both, most plants are edge plants with open ground on
one side, and the light grid stops 10 cm beyond the outermost leaves, so a border plant
sees full sky on that side. A field measurement is from an interior plant in a large
stand. Either read the density variables from the interior plants only (center 2 × 2 and
3 × 3), which the harness could do with a plant filter on the target, or state in the
response note that the sim value is a whole-plot mean and so an underestimate of the
response. The same applies to `neighbors_in_a_plot_take_each_others_light`, which already
picks plant 14 (the center); good.

### 3. Soil surface radiation is one value per plot

Listed as not done in 0019, and correctly in the model card known limits. Worth noting
why it matters more now than in the pots: a 150 × 120 plot with LAI about 2 has half of
its surface in sun between rows during the first 40 days, and the single-value rule dries
the shaded soil under the plants as fast as the gaps. With daily watering to field
capacity it has no effect on these scenarios, which is why it can wait. It should land
before WP26's weather-file runs, where rain-fed plots will be the point.

### 4. Plot dimensions are rounded silently to 5 cm cells

`soil_grid` rounds `width_cm / 5` to the nearest cell. A plot of 123 cm becomes 125 cm,
and the irrigation and surface area use the rounded grid. Either reject sides that are not
multiples of 5 in `validate`, or say in the schema doc that sides are rounded to the cell.
The same rounding on depth (40 → 8 cells) is fine.

### 5. Sowing offsets are checked against the grid, not the plot

`sow` rejects a sowing point outside the soil, which is right. For plots the error text
prints the point in soil-frame metres; printing the offset in cm and the plot size would
make a misplaced row easier to fix. Cosmetic.

### 6. Rayon error selection

`par_iter_mut().map(...).collect::<Result<_, _>>()` keeps the plant order for the values
(indexed parallel iterator, ordered collect), which is what the determinism test checks.
When two plants fail in the same hour, which error is returned depends on the schedule.
Errors abort the run, so this is harmless, but a comment at the site saying so would save
the next reader the question.

### 7. Golden check error text

A golden made with `--days 30` fails a run of the full scenario "at line 2". The message
should say the golden file holds 30 days and the run was asked for 90, or the check should
take its day count from the golden file. Small, and it would have saved this reviewer a
rerun.

### 8. Documentation

- `docs/design-plan.md` §3.5 and §7 WP21 still promise per-voxel coefficients at M2. Add a
  one-line pointer to decision 0019 when the plan is next edited.
- `light.md` says the plot grid's bottom is "at the ground". The code puts it at
  `min(0, lowest leaf)`, so a leaf drawn below ground would still be inside. Fine; say so.
- The model card for world-and-schedule should list `g_function` of the first plant as
  the plot's G under parameters, not only under known limits, so a WP23 reader finds it.

## Speed

Measured here: 20.9 ms per sim-day over 60 days on 4 cores and 41.7 ms on one thread,
against 28.8 and 59 ms over 90 days in 0019. So the first two months are at target and
the last month, with 30 full canopies, is over it. The callgrind split in 0019 (light 26%,
allocation 19%, class tables 11%, gas exchange 9%, hash 4%) points at two cheap wins
without a model change: skip ray tracing for leaflets whose light index was zero for
three days (dead or buried leaves still trace 40 × 7 rays), and share one class-table
lookup per plant per hour instead of per leaf. The perf thread owns this; the sim thread
also needs 60 fps with this world, so the budget there is tighter than 20 ms.

## For the record

- Model 0.3.0; pots byte-identical to 0.2.0; hash changed only through the version string
  and the new `shared_light` field. Verified.
- Validation at WP21: 70 checks, 54 pass, 16 Info misses, 0 known; the new miss is the
  density check at Info, as 0019 says.
- New tests: `the_plot_grid_has_fixed_cells_and_starts_at_the_ground`,
  `neighbors_in_a_plot_shade_each_other`, `plot_grid_light_against_ray_tracer`,
  `lateral_fraction_scales_as_a_diffusivity`, `a_plot_is_independent_of_the_thread_count`,
  `neighbors_in_a_plot_take_each_others_light`, `plots_outside_the_size_range_are_rejected`.
- Windows CI now checks the 30-day plot golden too.
