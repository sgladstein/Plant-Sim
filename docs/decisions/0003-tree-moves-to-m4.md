# 0003: The tree moves to M4; M2 growth forms are pole bean and a grass

- **Date:** 2026-11-05
- **Status:** accepted (owner, on the v3 review's recommendation)

## Context
A tree needs perennial storage, secondary growth, dormancy and multi-year runs, which is
a large step from an annual herb. M2 should prove the engine generalizes at modest cost.

## Decision
M2 adds the pole bean (the bush bean file with the habit locus set to indeterminate,
plus a tendril module) and a grass (tillering, intercalary growth, fibrous roots).
The tree is built in M4 alongside fields and forests.

## Consequences
- ps-arch's GrowthForm trait must fit herb, vine and grass by M2; tree pressure on the trait comes later.
- Scope creep control (R5): no tree work before M4.
