# WP7 review: physiology (PR #20, branch head 1b9cd4e)

Reviewed on the PR branch. Files: crates/ps-physio (gas.rs, classes.rs, allocation.rs, phenology.rs, thermal.rs, respiration.rs, tests), the plantecophys fixture scripts, the new CI job, decision 0009, four model cards, the species schema additions and the typed `Traits` view.

## Checks (all green on the branch)
- fmt, clippy pedantic, cargo test --workspace (125 tests; ps-physio 14 unit + 1 integration), check_deps (8 crates), check_determinism. The new `plantecophys` CI job has not run yet on this branch at review time.

## Done-when (plan section 7, WP7)
- Gas exchange within 2% of the plantecophys fixture over the section 8.1 grid: met, 1008 cases, worst 1.18%. See the honesty answer below.
- Allocation on hand-built graphs (starvation, deficit, priority): met (`seed_reserve_feeds_growth_and_ledger_closes`, `starving_leaf_dies_after_the_window` at exactly 72 h, `unmet_demand_shrinks_final_size`, transport conservation property test).
- Carbon ledger closes at 1e-9 relative every hour and after deaths, asserted in debug builds as CLAUDE.md requires.

## Answers to the four points

### 1. Priority as min(demand, priority x available) (decision 0009 item 8)
Agree with the reading. The literal plan text caps every organ at 70% of potential forever, which was never the intent; priority is meant to bias competition under scarcity and vanish when carbon is abundant. The implementation does exactly that: with full reserves every organ grows at potential, and under scarcity a low-priority organ spends a smaller share of its own reserve per hour, keeps a higher concentration, and so exports more to its neighbours. One consequence to record: the strength of the effect depends on the hourly step (70% per hour of a reserve is still almost all of it within a day), so priority is a weak lever. If WP12 finds fruit does not starve late leaves as section 3.9 promises, the fix is to make priority also shape the transport (conductance times priority of the receiving end), not to return to the literal formula. Plan amendment: section 3.9 step 4 should read `realized = min(demand, priority x max(0, R - 0.05 W))`.

### 2. Demand follows the species potential (item 9)
Agree. The reduced potential as the base compounds the loss and can lock an organ at zero; the species potential gives exactly `(1 - unmet / cumulative demand) x potential` and no catch-up, and the beta curve's increments sum to the potential (WP3 property test), so the identity holds. Correct.

### 3. Transport stability cap 0.5 min(W/degree) (item 10)
Correct as a stability condition: the explicit update of concentration `C_i` is a convex combination when `sum_j k_ij <= W_i`, and the cap guarantees it with margin. Two refinements, one of which matters:
- **Small organs are throttled by their own mass.** A new leaf of structural mass W can import at most `0.5 W dC / degree` per hour, so its growth is exponential at roughly `0.5 dC / (0.45 x 1.25)` per hour, about 9% per hour at a 0.1 g C/g DM gradient and 2% per hour at 0.02. Whether that binds depends on the initiation mass WP8 gives new organs and on the gradients the plant actually has. If it binds, every organ loses potential in its first days and, under no-catch-up, ends permanently smaller, which WP12 would then "tune" with higher K to no effect, because the cap, not K, is the limit. Two cheap mitigations: (a) use the harmonic cap `k <= W_i W_j / ((W_i + W_j) max(deg_i, deg_j))`, which is the exact one-hour relaxation of a two-node pair, is never smaller than the current cap and lets a tiny organ equilibrate with its parent in one step without overshoot; (b) add a counter of "hours where the cap bound" per organ to the inspector and CSV so WP12 can see it. Do (b) now and (a) if the counter is nonzero on the bean.
- Conductance uses the child segment's radius and length, falling back to the parent, else minimum values. For a leaf on an internode this makes the leaf-internode edge as conductive as the internode itself; fine, but the model card should say so.

### 4. Is the 1.18% honest?
Yes, with the caveat the implementer already states. I checked the Python mirror against what I know of plantecophys 1.4-6: Bernacchi 2001 constants (Kc 404.9 / 79430, Ko 278.4 / 36380, Gamma* 42.75 / 37830, O2 210 x Patm/100), the peaked Arrhenius form and the defaults (58550 / 200000 / 629.26 and 29680 / 200000 / 631.88), R = 8.314, vpdmin 0.5, the 1.57 conversion with g0 divided by it, Rd = Rd0 Q10^((T - 25)/10), the MAESTRA quadratics, the low-light rule and the hyperbolic minimum with 1 - 1e-4 all match my recollection of photosyn.R. The remaining 1.18% is explained, not mysterious: at exact co-limitation the hyperbolic minimum with theta = 0.9999 returns 0.99 of the true minimum, so a plain minimum is up to 1% high there, plus the small effect of Rd being subtracted after the smooth minimum. The result is a real 2% pass against plantecophys' equations; the CI job turns it into a pass against plantecophys' output. Optional: adopt the same hyperbolic minimum in Rust, which would let the test tolerance drop to about 1e-6 and make it a far stronger regression check. The plan's section 3.8 does not forbid it.

## Other findings (no bugs)

1. **Medium, plan-level (my plan, not the implementation): the temperature multiplier double-counts cold.** Demand = beta-curve increment in thermal time x the expansion multiplier (zero at 5 C, one at 25 to 30 C). Thermal time already slows development in the cold, and under no-catch-up the multiplier then throws away the remaining increment permanently. At 15 C the organ develops at 7 Cd/day and loses half of each increment; a temperate summer with 12 C nights loses a sizeable share of potential every night. WP12 will see small plants and reach for `growth_t_min`. Options: drop the multiplier when the curve runs in thermal time (the plan's own cardinal temperatures already carry the effect), or apply it to the pace of the curve (advance the organ's expansion clock by `dt x factor`) so cold delays rather than destroys growth. I recommend the second; it keeps the species constants and the plan's intent that expansion is more cold-sensitive than development. It is an equation change: decision record, model card, model minor bump. Flagging now so WP8 wires the organ clock accordingly.

2. **Low: flowering and maturity count thermal time from sowing, including the time a seed waits in dry or cold soil.** A seed that waits two weeks for water then flowers two weeks "earlier" after emergence. Crop models (CROPGRO, APSIM) count phases from emergence or germination. Suggest counting flowering and maturity from germination, with the bean thresholds reduced by the emergence 70 Cd, and recording it in decision 0009 item 12. Plan amendment to section 3.11.

3. **Low: a starved organ's negative reserve is booked as negative litter.** The ledger closes, but a debt vanishing at death is a small unphysical carbon source. Alternative: charge the deficit to the nearest living ancestor's reserve (the plant did burn that carbon). Worth a line in known limits either way.

4. **Low: light-class table.** Linear-in-PAR between log-spaced nodes with 4 capacity nodes is a good upgrade over snapping; the quoted 3% worst case in A should be a test (compare table lookup to a direct solve over a random PAR x capacity sample) so a later node change cannot silently worsen it.

5. **Nits.** `priority()` gives `Nodule` the root formula, fine for M2. `HourInputs.temperature` falls back to 20 C for a missing entry; prefer a length check with a `GraphError`, since a silently wrong temperature is hard to find. `Traits` view and the schema constants for the temperature responses close the WP4 follow-up nicely; the model card for species was updated.

## Plan consistency
- Steps 6 to 12 of section 3.2 are in order. Water stress acts in exactly two places (Medlyn slope term and expansion demand), as section 3.2 step 5 requires. Leaves pay maintenance through Rd and not through the nitrogen formula (decision 0009 item 7), which avoids the double count the plan did not notice.
- Plan amendments to make (I will send them to the implementation thread to fold into docs/design-plan.md): section 3.9 step 4 formula; section 8.1 light metric wording (from WP6); section 3.11 phase start (finding 2) if accepted; section 3.9 step 1 temperature multiplier (finding 1) once decided.
