# 0028: Lateral root cohorts keep their tips in the soil

- **Date:** 2026-11-05
- **Status:** accepted

## Context
The owner saw roots in the viewer leaving the soil, some going straight up out of the
surface. Two causes, one on each side:

- **Sim.** A lateral cohort is one straight organ the full length of its root type
  (6.1 cm for the bean), placed at the insertion angle with no check against the soil.
  Root axis segments go through the pot-wall rule (decision 0013); laterals did not. At
  the end of single_pot, 68 of 218 lateral cohorts (31%) ended outside the 24 cm pot,
  through the wall or below the bottom. No root organ ever rose above the surface.
- **Viewer.** ps-mesh draws up to 8 copies of a cohort turned about the parent axis. For
  a near-horizontal parent (basal roots, axes running along the wall), that sends about
  half the copies upward. The viewer thread fixes this: it turns copies about the
  vertical and clamps them to the pot.

## Decision
1. A lateral cohort's tip must lie in active soil. The direction is chosen in this order:
   - the drawn direction (insertion angle at the next golden-angle turn, mirrored down);
   - else the pot-wall rule of an axis segment (straight down, then along the wall
     either way);
   - else the next seven golden-angle turns, each mirrored down.

   If none fits, the drawn direction stays (not seen in the test scenarios).
2. Model 0.10.0 → 0.11.0, goldens regenerated.

## Consequences
- No lateral tip ends outside the pot in single_pot (was 68), or in pole_trellis (was 37).
  Test: `ps-export/tests/roots_in_soil.rs`.
- Root length moves from the pot's outer cells into its interior. Water and N uptake
  shift slightly. In watered, fed pots growth hardly changes: single_pot ends at the same
  organ counts, with dry mass different in the fifth digit, and the pole bean's day-60
  height is unchanged.
- Validation at 0.11.0 on four-seed means: 65 of 87 pass, the same checks as at 0.10.0.
  Every M0 check passes.
