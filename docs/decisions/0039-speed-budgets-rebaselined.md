# 0039: Speed budgets re-baselined

- **Date:** 2026-11-06
- **Status:** accepted (owner, 2026-11-06)

## Context

The design plan set two sim speed budgets: the single bush bean in a pot under
2 ms per sim-day (M0 exit) and the 30-bean plot under 20 ms per sim-day (WP21 and
M2 exit). They were set before the leaf energy balance (WP27, decision 0034), stem
and root thickening (decision 0036) and the realism plan (R1 to R3), each of which
adds per-organ work. The bit-identical speedups are close to exhausted (see
docs/perf/handoff.md). In the 4-core cloud container at model 0.19.0 the single
pot runs at about 3.5 ms per sim-day and the plot at about 26 ms. Meeting the old
numbers would have needed result-changing cuts (fewer light points per leaf, sky
light every few days, a cheaper hash) made only to hit a number.

The owner was offered: (A) re-baseline the budgets, (B) keep the old budgets and
accept result-changing cuts, (C) rely on the multi-fidelity plan
(docs/research/realism-plan/fidelity-levels.md) for big plots and fast-forward.
The owner chose A and C.

## Decision

- Single plant (bush bean in a pot): under **4 ms** per sim-day.
- Plot (30 bush beans): under **30 ms** per sim-day.
- Both are provisional until they are timed on the owner's PC (i5-6600K, 4 cores).
  The cloud container numbers stand in until then.
- Big plots and fast-forward are covered by the multi-fidelity plan: the simplified
  mode runs there, provided its outcomes stay close to the full model. The full
  model is held to the budgets above, not the simplified one.
- The performance workstream stops pursuing result-changing speedups whose only
  purpose was the old 2 ms and 20 ms budgets. Bit-identical speedups continue.
  A result-changing speedup can still be proposed for another reason (for example
  the viewer frame budget), with the implementation or review workstream's
  agreement.
- The viewer frame budget (M1, under 16.7 ms on the GTX 1070) is unchanged.

## Consequences

- design-plan §7 M0 exit, WP21 and M2 exit point here for the sim speed budgets.
- docs/performance.md and docs/perf/handoff.md carry the new budgets.
- The shelved result-changing patches (sky light every third day, class-table
  row interpolation) stay shelved; they are recorded in docs/perf/handoff.md.
- Each new realism step still gets a cost measurement; if the full model drifts
  over 4 ms or 30 ms, that is reported, and the remedy is a bit-identical cut,
  the simplified mode, or an owner decision.
