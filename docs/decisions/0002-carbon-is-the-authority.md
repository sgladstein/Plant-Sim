# 0002: Carbon is the sole authority on growth, with per-organ reserves

- **Date:** 2026-11-05
- **Status:** accepted (adversarial review, owner)

## Context
Architecture-driven plant models can "grow" organs the plant cannot pay for, which
breaks the science lenses and makes shade and drought responses scripted, not emergent.

## Decision
Carbon decides how much grows; architecture modules decide only where and in what
shape. Reserves live per organ and move between neighbors by conductance-weighted
exchange (§3.9). Light and shade signals set bud activation and sink priority but never
create biomass. Nitrogen joins as a second authority in M2.

If after two tuning sessions in M0 per-organ transport cannot be made stable or
plausible, transport collapses to three pools (each shoot axis, the root system, the
reproductive organs) with the same demand and priority rules; per-organ reserves stay
as the starvation signal. This fallback is decided in advance.

## Consequences
- Self-pruning of shaded branches, root shift under drought and fruit-load effects are emergent.
- The carbon ledger must close every hour (1e-9 relative in debug builds).
- Allocation tuning is a known risk (R2), time-boxed with the fallback above.
