# Review workstream handoff (Fable review thread)

Written 2026-11-05 for the next coordinator. Everything here is reproducible from the repo
alone. The write-ups referenced below live in `docs/archive/reviews/` after the handoff
(copied from `/mnt/project-files/reviews/`).

## 1. What the review thread did

An independent reviewer, running in its own session, reviewed every work-package PR from the
implementation thread (and the viewer and variation threads) before merge, against
`docs/design-plan.md`, the WP's done-when, and the science. The owner approved all merges in
advance; the implementation thread merged on green CI after the review verdict and folded fixes
in, either in the same PR or the next. The reviewer never pushed to the repo; it sent findings
as messages and kept a write-up per review.

Reviews done (all approved, with fixes folded in by the implementer): WP4 to WP12 (M0,
including the WP7 fixes PR #24 and the WP11 data and harness reviews), WP16 (sim side), WP21
plot, WP24 nitrogen (+ PR #57 round), WP22 pole bean, WP23 wheat (+ #67 round), WP25 actions
(PR #73), light leaf-angle fix (PR #75, decision 0025), plant-to-plant variation (PR #80,
decision 0027), multi-seed harness (PR #81, decision 0026). Files: `wp<n>-review.md`,
`wp7-fixes-pr24-review.md`, `wp11-data-review.md`, `wp11-harness-review.md`,
`light-leaf-angle-review.md`, `variation-review.md`.

## 2. How to review a PR (reusable instructions for a fresh review thread)

Read `docs/STATUS.md`, the WP row in `docs/design-plan.md` §7, the PR body and the new
`docs/decisions/00nn-*.md`. Then, on the PR head commit:

```
cargo fmt --all --check
cargo clippy -p ps-validate -p ps-cli -p ps-export -p ps-world -p ps-arch -p ps-physio \
  -p ps-soil -p ps-light -p ps-weather -p ps-species -p ps-graph -p ps-rng -p ps-units \
  -p ps-session --all-targets --locked -- -D warnings
cargo test <same -p list> --locked
python3 scripts/check_deps.py
python3 scripts/check_determinism.py
cargo run -q --release --locked -p ps-cli -- hash data/scenarios/single_pot.ron \
  data/scenarios/two_pot_sun_shade.ron --check-golden tests/golden
cargo run -q --release --locked -p ps-cli -- hash data/scenarios/plot_30_beans.ron --days 30 \
  --check-golden tests/golden
cargo run -q --release --locked -p ps-cli -- validate --out <dir>
cargo run -q --release --locked -p ps-cli -- bench data/scenarios/plot_30_beans.ron --days 60
```

Notes: the Bevy crates (ps-mesh, ps-view, ps-ui, ps-app) do not build in a GPU-less container;
CI covers them. The plot golden is a 30-day golden, so pass `--days 30`. `validate` writes
`report.md`, `results.csv` and `runs/<scenario>/{daily,organs,organs_day45}.csv`; it takes
about 2.5 min on 4 cores with the 4-seed means. For a byte-identity claim ("the bean is
unchanged"), build `origin/main` in a `git worktree` with its own `CARGO_TARGET_DIR`, run
`ps-cli run <scenario> --out <dir>` on both and `cmp` the CSVs.

Then do what the checks cannot: pick the claim in the decision record that carries the most
weight and test it with a diagnostic run. The reviews that mattered most came from this step:
run the scenario under other world seeds (the WP25 review: a one-seed gain was pod-set noise);
flip a parameter 100× to see if it is really the cause (WP24: the shade-internode drop was not
nitrogen); patch one line and rerun (WP23: all wheat blades pointed east); read `organs.csv` to
see which organ starved (WP22: internodes starve while `feedback_loss` is 0.7). Always compare
against main, not against the PR's own claims.

Science checks per PR: every parameter in `data/` has value, unit and source ("~" is a guess to
validate); every equation change updates the model card in `docs/models/` and bumps the model
minor version with regenerated goldens; the step order (plan §3.2) is unchanged; no new
`HashMap` iteration, `thread_rng`, std transcendentals or raw f64 quantities in the sim crates.

Write-up format (`docs/archive/reviews/<name>-review.md`): Verdict (approve / approve with
fixes / request changes, with the one or two fixes named in bold), What I checked (table),
the implementer's questions answered, Findings ranked by weight, For the record (model
version, schema, validation counts, diagnostic numbers). Then send the implementer the verdict
and the fixes in under 2000 characters, and the owner a plain-language summary under 150
words with the file attached. The owner wants the one surprising finding, not the list.

## 3. Unresolved findings that matter most, ranked

1. **Transport / priority rule (WP7 open question, decision 0009).** Each organ spends only
   from its own reserve per hour and a shortfall is lost (no catch-up). Three species now miss
   height or size for this reason while `feedback_loss` is 0.3 to 0.8 (the plant discards up to
   70% of potential assimilation for lack of sinks): pole bean 1.38 m at day 60 (target 1.5),
   wheat stand 0.54 m (0.7 to 1.0), stand biomass 3.5 g (5 to 9). Revisit the rule once, before
   M2 exit, rather than raising `phloem_conductance` per species (bush 500, pole and wheat
   1500 already). Reviews: wp22, wp23. Owner's call because the bush bean then needs re-tuning.
2. **Seed-to-seed noise in a single plant.** One bean spans 21 to 39 g and 8 to 17 pods with
   the world seed alone (wp25-review). The 4-seed harness (0026) now reads means, but no "~"
   parameter tuned before 0026 has been re-checked on means, and the model puts all the
   variance in a few dozen Bernoulli pod-set draws at flowering. Re-tune on means; consider
   spreading pod set over the season.
3. **R:FR response of branching and leaf size** (wp21-review finding 1). Density response is
   weak because every plant makes the same 6 branches and 85 cm² leaves at 5 and 35 per m².
   Bud outgrowth and leaf potential area should scale with the per-leaf `ShadeSignal.rfr`.
   Also the lever for wheat tillering (the proxy `1.2 exp(−0.5 LAI above)` has no sideways
   far-red; derive R:FR from the sky-view coefficient instead; wp23). Promote the density
   check to an M2 gate once branching responds.
4. **Light within-cell limit.** Leaves in one cell shade each other partly instead of one
   fully (light.md known limits); the 60-day sim plant is +3.8% against the ray tracer with
   85% of leaflets within 0.10, and the 0008 gate (95%) holds only on the synthetic shoot.
   Follow-ups: leaf-angle spread (mean 20°, SD 15°) and heliotropism now that each leaf uses
   its own normal (light-leaf-angle-review).
5. **Vine capture sweep is a fixed 6 cm** (decision 0021). With sowing scatter (0027) one pole
   bean in 16 misses its stake and lodges, dragging the height mean. Fix: sweep grows with the
   free tip length, and sowing scatter should not move a seed away from a support it was sown
   at (variation-review).
6. **Nitrogen follow-ups**: uptake capacity should scale with root mass (issue #58); stranded
   nitrate below the root zone drives the density pass (wp24-review); fixation cost 40% of
   assimilate in sand (card should say so); pod set should read N.
7. **Smaller logged items**: soil surface radiation ignores obstacles; `Harvested::mature`
   should be per pod; topping after flowering releases nothing (pole bean at the trellis top);
   bud eligibility after a cut; soil-patch field should be keyed per lattice node; four
   RNG entity-key conventions to list in random-numbers.md; wheat crown 1 cm above soil;
   Fischer grain rule; interval tiller regression; G per direction now resolved by 0025.

## 4. Review in progress

None. PR #81 (multi-seed harness) was reviewed and approved at 15:14 UTC; it merges on green
CI. No other PR was open for review at handoff.

## 5. Standing agreements with the owner that affect reviews

- The owner approved all merges in advance (2026-11-05) and wants threads autonomous; the
  implementer merges on green after the review verdict.
- Decision 0008: the M0 light gate is 95% of leaves within 10% on the synthetic shoot.
- Decision 0023: twining not tendrils; pole bean target 1.5 m by day 60 and 2 m by day 75;
  spring wheat is the grass; light linkage, two-tier mutation and a seed-colour rule table bind
  WP28 (genome), which has not been built yet and should be reviewed against them.
