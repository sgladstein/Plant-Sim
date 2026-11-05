# 0009: Physiology details the plan left open

- **Date:** 2026-11-05
- **Status:** accepted (implementer); items 4 and 8 read the plan's text in a way the
  owner may want to confirm. Item 10 and the flowering and maturity part of item 12 are
  superseded by decision 0011, which also changes how temperature enters item 9.

## Context
§3.8 to §3.11 and §11.4 give the processes and most constants of WP7. Building them
raised the questions below. Each answer is the smallest one that keeps the plan's intent
and its validation targets.

## Decision
1. **Fixture without R.** CRAN is unreachable from the build environment, so the
   gas-exchange fixture comes from `validation/fixtures/photosyn_reference.py`, an
   independent implementation of the equations plantecophys 1.4-6 `Photosyn()` uses. The
   CI job `plantecophys` runs plantecophys itself on the same grid and fails on any
   difference above 1e-6 relative. Until that job has passed once, the fixture is
   "plantecophys equations", not "plantecophys output".
2. **Temperature responses.** §11.4 says "the Medlyn 2002 parameter set unless a species
   card says otherwise". The bean file now carries the peaked-Arrhenius parameters
   (Medlyn 2002 form) with plantecophys' defaults as "~" values
   (Vcmax 58550 / 200000 / 629.26, Jmax 29680 / 200000 / 631.88), and Rd Q10 1.92. They
   are guesses for bean until WP12. Rubisco kinetics are Bernacchi et al. 2001 constants
   in code (fixed physiology, §3.12).
3. **H2O/CO2 ratio 1.57**, as plantecophys hard-wires (Medlyn 2011 writes 1.6). With 1.6
   the conductance would differ from the reference by 1.9%, at the edge of the 2% target.
4. **Water stress on the Medlyn slope.** The stress factor multiplies the
   `(1 + g1/√D) A / Ca` term, so full stress leaves `g0`. The plan says the factor acts on
   "stomatal conductance" without saying where.
5. **Below light compensation**, `Ci = Ca` and `gs = g0`, as plantecophys and MAESTRA do.
   The coupled solution there differs by a fraction of a µmol m⁻² s⁻¹ but is 40% off in
   relative terms, which a 2% relative check cannot tolerate near zero.
6. **Light classes as an interpolated table.** 17 PAR nodes (0 and 16 log-spaced from 1
   to 2000) × 4 capacity nodes (1.0, 0.8, 0.6, 0.4), read linearly in PAR and capacity:
   68 solves per plant-hour instead of the plan's 32, and up to 3.0% error in A instead of
   the tens of percent of snapping to a class. Capacity nodes exist because leaf aging
   (§3.8) makes leaves in one PAR class differ.
7. **Leaf maintenance is Rd.** Leaves pay maintenance through day respiration in gas
   exchange (and `−Rd` at night), so the nitrogen-based maintenance of §3.8 applies to
   every other organ. Charging both would count leaf respiration twice.
8. **How priority enters growth.** §3.9 step 4 reads
   `realized = min(demand × priority, max(0, R − 0.05 W))`. Taken literally, a leaf with
   priority 0.7 would never get more than 70% of its demand even when swimming in
   carbon, so every organ would end smaller than its potential. The implementation uses
   `realized = min(demand, priority × max(0, R − 0.05 W))`: priority is the share of its
   own available reserve an organ may spend in an hour. High-priority sinks draw their
   reserves down faster, which steepens the gradient toward them, so they also attract
   more transport.
9. **Demand follows the species potential.** Hourly demand is the curve increment of the
   species potential, not of the reduced potential. Using the reduced one compounds the
   reduction, and an organ that got nothing early would lock at zero forever. With the
   species potential, a fully expanded organ ends at exactly the plan's
   `(1 − unmet / cumulative potential demand) × potential`, and there is still no
   catch-up.
10. **Transport stability.** One explicit Jacobi pass is unstable when an edge's
    conductance exceeds an organ's mass per hour (carbon sloshes back and forth each
    hour). Each edge's conductance is capped at `0.5 · min(W / degree)` of its two ends,
    which makes every hourly update a convex combination. Conductance is
    `K π r² / l` of the segment the edge runs through, with `K = 500 g DM m⁻¹ h⁻¹` ("~":
    moves a leaf's hourly export through a 2 mm stem at a 0.03 g C/g DM gradient).
11. **Starvation details.** The starving-hour counter updates after maintenance and
    income, so a leaf's night respiration counts. A dead organ's reserve (negative when
    it starved) goes to litter with the non-resorbed structure, and resorption goes to
    the nearest living ancestor.
12. **Phenology.** Emergence thermal time counts from germination (§3.10), flowering and
    maturity from sowing (§3.11). Trifoliate, pod set and seed fill are organ events the
    architecture reports. Water stress does not yet change the pace of development; the
    plan's "and stress" stays open until WP12 shows whether it is needed.
13. **Schema 1 gains constants** `rd_q10`, `vcmax_*`, `jmax_*` and `phloem_conductance`,
    and ps-species gains a typed `Traits` view of the phenotype (WP4 review). No species
    file has been released, so the schema number stays 1.

## Consequences
- Gas exchange meets the 2% target with a 1.2% worst case; the remaining difference is
  plantecophys' smooth minimum of Ac and Aj.
- Model minor version: first physiology, no golden hashes yet.
- WP12 tunes the "~" values; the three-pool transport fallback of §3.9 stays available.
