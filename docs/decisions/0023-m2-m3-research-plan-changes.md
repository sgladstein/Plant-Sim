# 0023: Plan changes from the M2 and M3 research

- **Date:** 2026-11-05
- **Status:** accepted. The owner approved all six recommendations of the M2/M3 follow-up research report on 2026-11-05.

## Context
The follow-up research report (project files: research/m2-m3-bean-science.md, published as
"M2 and M3 Bean Science") ends with six decisions for the owner. Two were already built
on their recommended option: twining in WP22 (decision 0021) and spring wheat in WP23
(decision 0022). Four change M3 (WP28), which has not started. This record fixes all six
so the design plan and later packages follow them.

## Decision
1. **Stem twining, not tendrils (WP22; plan §3.3, §3.4).** Common bean is a stem twiner.
   The tip circumnutates, captures a support in its sweep, and coils round it
   anticlockwise seen from above (a right-handed helix). There is no tendril search.
   Built in WP22 (decision 0021). The `Tendril` organ kind stays in `OrganKind` for now,
   unused. Removing it changes a serialized enum and the golden hashes, so it goes with
   the next package that bumps the model anyway.
2. **WP22 target: at least 1.5 m by day 60 and 2 m by day 75 at a 20 °C mean**, in place
   of "fills a 2 m trellis by day 60". At 20 °C the stem makes about 13.5 nodes by day 60,
   and 2 m would need 15 cm internodes, the top of the published range (Rutjens 2025;
   'Kentucky Wonder' 1.5 to 2.1 m). Built in WP22 (decision 0021).
3. **Spring wheat is the WP23 grass**, with the module general enough that perennial
   ryegrass is a parameter swap plus perenniality. Built in WP23 (decision 0022).
4. **Light linkage in WP28 (plan §3.12).** Each locus has `chromosome` (1 to 11) and
   `position_cm`. Meiosis draws Haldane crossovers, Poisson with mean length / 100 cM per
   chromosome (about 100 cM each; BAT93 × Jalo map 1,097.5 cM, Pv01 147, Pv09 46.5; Blair
   2018), on a stream keyed by (world seed, plant id, tick, `Meiosis`, chromosome).
   Unlinked loci are the special case of separate chromosomes or 50 cM or more apart.
   The named loci that really share chromosomes are placed there: *fin* with *Ppd* on
   Pv01, *St* with *I* on Pv02, *P* with phaseolin and seed size on Pv07. Their distances
   were not found and start as "~" placeholders of 10 to 20 cM.
5. **Two-tier mutation in WP28 (plan §3.12, §6.1).** Anonymous QTL mutate at 1e-3 per locus
   per gamete with small effects (SD 0.1 to 0.2 of the allelic SD), framed as mutational
   variance about 1e-3 of the environmental variance (Houle 1996, [K, to verify]). Named
   Mendelian loci mutate at 1e-5 (visible-mutation rates in plants are 1e-5 to 1e-6). The
   model card calls this a game approximation.
6. **A separate seed-colour rule table in WP28 (plan §3.12, §6.1).** Seed-coat and flower
   colour come from a visual-trait layer, a small rule table outside the additive
   physiological map: the *P* gate (white or coloured) → *T* (full or partial) → *C*/*J*/*V*
   base colour → *B*/*G*/*Rk* intensity → pattern loci (Plestenjak 2025; McClean 2018). It
   drives only `appearance`. P's tie to seed size is carried by its chromosome position, not
   the table. Bassett 2007 and Parker 2024 are to be read for the full table before WP28.

## Consequences
- The design plan's §3.3, §3.4, §3.12 and §7 rows 22, 23 and 28 now say this.
- WP28 adds `chromosome` and `position_cm` to each locus, the `Meiosis` RNG purpose, a
  per-tier mutation rate in the species schema, and the visual-trait table. That is one
  model version and one golden regeneration. Adding linkage later would have cost a second.
- The F2-variance-exceeds-F1 done-when still holds with linkage. A repulsion-phase cross
  (bush and day-neutral from *fin* × *Ppd* parents) becomes a believable breeding puzzle.
- The research report's other rows (density sweep, nitrogen design, competition lens,
  challenges) are recommendations for their packages and need no owner decision.
