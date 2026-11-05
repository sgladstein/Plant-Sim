# 0011: Physiology revisions after the WP7 review and the research findings

- **Date:** 2026-11-05
- **Status:** accepted

## Context

The WP7 design review (/mnt/project-files/reviews/wp7-review.md) and the first findings
of the plant-science research thread raised five points about the physiology merged in
PR #20. Each one changes an equation or a default parameter. No golden hashes exist yet,
so nothing is regenerated; the model minor version is bumped when it is introduced
(WP10).

## Decision

1. **The expansion temperature multiplier sets the pace of an organ's expansion clock
   instead of scaling its demand.** Each organ keeps `expansion_age`, advanced each hour
   by thermal time × the multiplier (0 at 5 °C, 1 from 25 to 30 °C, 0 at 40 °C). The
   expansion curve runs on that clock. Before, the curve ran on thermal age and demand
   was multiplied by the factor; with no catch-up (decision 0009) that destroyed part
   of every cool hour's growth for good (half of each increment at 15 °C), double-
   counting cold, which thermal time already slows. Now cold delays expansion and a
   fed organ still reaches its potential (`cold_delays_expansion_without_loss`).
   Water stress still scales demand, so drought loses growth permanently, as intended.

2. **Phloem transport is one backward-Euler diffusion step per hour, solved exactly on
   the organ tree.** Concentrations `C = R / W` solve
   `W_i C_i' + Σ_j k_ij (C_i' − C_j') = max(R_i, 0)` by one leaf-to-root elimination
   and one root-to-leaf substitution (the Thomas algorithm on a tree, O(n), in fixed id
   order, so deterministic). This replaces the explicit Jacobi pass, its stability cap
   `0.5 · min(W / degree)` and the outflow clamp of decision 0009. Reasons: the review
   showed the cap throttles a small organ by its own mass (about 9% growth per hour at a
   0.1 g C/g gradient), and lost hours are permanent under no-catch-up; the research
   thread found no plant model that uses one explicit pass per hour (L-PEACH uses
   Newton–Raphson, PiafMunch a stiff solver) and noted the explicit pass moves carbon
   one edge per hour. The implicit step is unconditionally stable and monotone (no
   concentration leaves its neighbors' range, so no reserve is driven negative by
   transport), conserves carbon by moving it as edge fluxes, and needs no cap. A debt
   (negative reserve) is excluded from the solve and stays with its organ, so it
   neither pulls carbon by itself nor spreads to neighbors; the organ still receives
   carbon as if empty. The per-organ cap counter the review proposed is not needed.

3. **Flowering and maturity count thermal time from germination, not sowing**, as crop
   models (CROPGRO, APSIM) count phases, so time a seed waits in dry or cold soil does
   not bring flowering forward. The bean thresholds become 320 and 820 °C·day: the
   sowing-based 350 and 850 less about 30 °C·day from sowing to germination in moist,
   warm soil ("~").

4. **A starved organ's negative reserve is charged to its nearest living ancestor**
   when it dies. The debt is carbon the organ already respired; booking it as negative
   litter was a small unphysical carbon source. With no living ancestor (the whole
   plant died) it still goes to litter.

5. **Maintenance respiration is 0.218 g C per g N per day at 20 °C** (Ryan 1991, the
   base rate CLM uses, 2.525e-6 g C g⁻¹ N s⁻¹). The plan's 0.015 is a per-dry-mass
   rate (McCree) applied per gram of nitrogen, so non-leaf organs paid about 15 times
   too little. Leaves are still not charged it (they pay `Rd`, decision 0009 item 7).

Also, not an equation change: `allocation::hourly` returns `PhysioError::InputLength`
when the temperature or income slice does not have one entry per organ, instead of
silently using 20 °C.

## Consequences

- `OrganArena` gains `expansion_age` (in the state hash). `allocation::hourly` returns
  `PhysioError`, which wraps `GraphError`.
- Plan amendments: §3.8 maintenance rate, §3.9 steps 1, 3, 4 (the decision 0009
  formula) and 6, §3.11 phase start, and the §8.1 light metric wording (decision 0008).
- Open for WP12, from the review and the research thread: the phloem conductance `K`
  now sets how far carbon reaches in an hour; mature leaves import like any organ;
  priority is a weak lever at an hourly step; Vcmax 89.3 and Jmax 160 (Pimentel 2011)
  were fitted with a mesophyll conductance, so without one Rubisco-limited A is about
  20% high at 420 ppm (add gm or use Ci-based values near 70).
