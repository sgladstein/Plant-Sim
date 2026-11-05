# Review of PR #24: physiology revisions after the WP7 review (decision 0011)

Branch head e67c842 (plus a no-change merge commit). Checks green: fmt, clippy pedantic, 131 tests across the workspace (ps-physio 19 unit + 1 integration), check_deps (9 crates), check_determinism.

## Backward-Euler transport on the tree: correct
- The system `W_i C_i' + sum_j k_ij (C_i' - C_j') = R_i^+` is the implicit step of `W dC/dt = sum k (C_j - C_i)`, since `W_i C_i = R_i`. The matrix is a tree Laplacian plus a positive diagonal: symmetric, diagonally dominant, an M-matrix, so the solution is a weighted average of the old concentrations (monotone) and the elimination never divides by zero as long as every pivot node has positive W, which `max(W, 1e-6)` guarantees.
- Ordering: `edges()` yields living (parent, child) pairs in child id order, and parents are created before children, so eliminating in descending child order finishes every subtree before its root, and substituting in ascending order has each parent's value final before its children. The debug assert `pa < ch` guards the invariant. Dead organs are excluded by `edges()` and `alive_ids()`, so a living child of a dead parent is solved as a root. Correct.
- Carbon is moved as edge fluxes at the new concentrations, so conservation holds to rounding, and an organ with a non-negative reserve ends at `W_i C_i' >= 0`; a debtor keeps its debt and still receives. Both claims in decision 0011 hold.
- Determinism: fixed id order, no maps, no parallelism. The property test (conservation, monotone range) and the chain test (carbon reaches four edges in one hour, monotone profile) cover the behaviour that mattered. Good replacement for the cap; the counter is indeed unnecessary.
- One remark for the model card: with the implicit step, `K` alone sets how far carbon travels per hour (k/W of 0.07 to 0.35 per hour for a 2 mm, 4.5 cm internode against 0.1 to 0.5 g organs at K = 500), so source-to-sink gradients persist over hours rather than vanishing in one. That is the behaviour the plan wants; WP12 tunes K against it. The card says this already in the "open for WP12" note; it belongs in Known limits too.

## Expansion clock: correct and better than my proposal in one detail
Thermal age and expansion age are separate, so leaf aging (capacity decline) still runs on thermal time while expansion runs on the paced clock. Note for WP8: `leaf_capacity(age, expanded, lifespan)` takes the thermal age at full expansion as `expanded`; with the paced clock that moment varies, so WP8 should record the thermal age when `is_complete(expansion_age)` first becomes true instead of using the curve duration.

## Phenology from germination: fine. Thresholds 320 and 820 are "~" and documented.

## Debt to the ancestor: fine, ledger closes, litter never negative; test present.

## Maintenance 0.218 g C per g N per day: correct
CLM's base rate 2.525e-6 g C per g N per s (from Ryan 1991) is 0.218 per day; the plan's 0.015 was indeed a per-dry-mass figure misapplied per gram of nitrogen. For a bean stem at about 1.5% N this gives about 0.003 g C per g DM per day at 20 C, within the McCree and Penning de Vries range (0.003 to 0.006). Good catch by the research thread.

## Minor
- My harmonic-cap remark was wrong when degrees differ; the implementer's counterexample is right. Moot now.
- The vcmax and jmax note (Pimentel 2011 values fitted with mesophyll conductance, so Rubisco-limited A is about 20% high at 420 ppm without gm) is the kind of thing WP12 must handle; it is correctly parked in decision 0011.

Verdict: merge.
