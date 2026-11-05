# 0031: WP28 genome: 40 loci, pure lines, seed colour

- **Date:** 2026-11-05
- **Status:** proposed (WP28 step 2; step 1 was PR #92)

## Context
Step 1 (PR #92) built the breeding engine and changed no result. It added Haldane linkage,
meiosis, two-tier mutation, crosses and a seed-colour rule evaluator, all in
`ps_species::genetics` and `ps_species::seed_colour`. Step 2 fills the bean species files
with real loci and fixes the choices decision 0023 left open.

Inputs:
- the WP28 research: `docs/research/m3-breeding-and-weeds/notes/genetics_map_colour.md` (named
  loci, distances, the colour pipeline) and `breeding_quantitative.md` in the same folder (heritabilities,
  genotypic CVs, crossing, mutation);
- the step 1 review (`reviews/wp28-step1-review.md` in the project files).

## Decision
1. **Forty loci per bean file: 24 anonymous QTL and 16 named loci.**
   - **QTL (15 traits).** Loci per trait follow the research table:
     - vcmax25 2, jmax25 1, rd25 1, medlyn_g1 2;
     - leaf_area_potential 2, specific_leaf_area 2, internode_length_potential 2;
     - nodes_before_terminal 1, leaf_lifespan 1;
     - seed_mass 3, seeds_per_pod 2, flowers_per_raceme 1;
     - root_elongation_scale 1, bud_activation 2, specific_stem_length 1.
   - **Named loci.** fin, Ppd, I, B, St, Rk, Z, G, V, Asp, Phs, P, C, T, Bip and J.
   - **Left out.** Disease (Co-, bc-), pod (y, MYB26) and sterility (ms-2) loci wait until
     the sim has those processes.
2. **A species file describes a pure line.**
   - Each locus has a `line` allele. A `Default` seed carries that allele on both
     homologues. A locus without `line` defaults to 0, which is the species default.
   - The bush and pole bean files share one genome. Both set `habit` default to 0, and the
     pole bean's line carries Fin Fin.
   - `fin` has effect 1 and dominance 0.5, so Fin Fin and Fin fin give habit 1 and fin fin
     gives 0.
   - A bush × pole cross therefore climbs in F1 and splits 3 : 1 in F2.
3. **Founders are pure lines.**
   - `SeedChoice::Sample` draws one value per locus and puts it on both homologues. Bean is
     an inbreeder; a heterozygous founder would segregate on selfing.
   - A QTL takes `line + N(0, allele_sd)`, with `allele_sd = CV_G · default / √n`. CV_G
     comes from the research table: 0.10 for photosynthesis, 0.15 to 0.20 for g1, leaf
     size, internode length, seed mass and seeds per pod.
   - A named locus takes a state drawn from its `frequencies`. All frequencies are "~".
4. **Dominance.**
   - +0.15 on leaf_area_potential, internode_length_potential and flowers_per_raceme
     (vigour components). This gives heterosis through physiology, never from a table.
   - −0.05 on seeds_per_pod (the research reports negative heterosis).
   - 0 elsewhere, and 0 on seed_mass (heterosis is at most 6%).
5. **Named loci without a modelled process carry no trait.**
   - Ppd has no photoperiod response yet. I and St have no disease or pod strings.
     Phs is a marker. The colour genes affect only appearance.
   - Allele state codes rise with dominance, so allelic series are read with `Expressed`:
     - P: p, p^gri, P
     - V: v, v^lae, V
     - Z: z, z^sel, Z
     - Bip: bip, bip^ana, Bip
   - The C complex is one four-state locus: c, C^mot, C, C^R. The code order is an assumed
     dominance [I].
6. **Placements and lengths.**
   - Chromosome lengths are 147 cM for Pv01 and 46.5 cM for Pv09 (Blair 2018), and
     100 cM for the rest.
   - Named positions follow the research table:
     - fin 77 and Ppd 85 (8 cM apart [I]);
     - I 1, B 2, St 15 and Rk 30 on Pv02 (Rk 28 cM from B, published);
     - Asp 75, the seed-mass QTL SW7 90, Phs 91 and P 100 on Pv07 (P–Phs 9 cM, published);
     - Bip 40 and J 67 on Pv10.
   - QTL placements are [I].
   - Every locus now carries its own `source`. This answers review item 2: a locus records
     the source of its placement, effect and spread together, rather than making
     `position_cm` a separate `Param`.
7. **Meiosis key (amends decision 0023 §4).**
   - A gamete is keyed by (world seed, the seed's own organ entity `(plant << 32) | organ
     id`, the tick its genotype is fixed, egg or pollen) under purpose `Recombination`
     (code 10). Each chromosome owns a fixed block of draws.
   - The pod id alone, or the plant id as in 0023, would give same-tick siblings identical
     gametes.
   - WP29 fixes a seed's genotype when the `SeedInPod` organ is created at seed fill. It
     keys by that organ (review item 3).
8. **Wording fix to decision 0023.**
   - Loci 50 cM apart are not unlinked under Haldane: r = 0.32 at 50 cM, and r reaches 0.5
     only asymptotically.
   - "Unlinked" means on different chromosomes, or without a chromosome (review item 4).
9. **The seed-colour table** is `data/genetics/bean_seed_colour.ron`.
   - Its layers are:
     1. P gate
     2. flower (tt and V)
     3. hue (C^R, cc, j, V with B, rk, G and B)
     4. pattern (C^mot mottle)
     5. partly coloured (the tt × Z × Bip × J table)
     6. shine (Asp)
   - Each rule cites its row in the research.
   - It drives only appearance. Nothing in the step reads it, so it is not in the hash.
     The viewer and the WP31 genotype lens will.
10. **The genotype is stored on each plant** (`PlantEntry::genotype`). WP29 needs it to
    make seeds and WP31 to show genotypes. It is part of the state hash.
11. **Not adopted now.**
    - **A per-plant micro-environment term on heritable parameters.** Decision 0027
      already adds non-heritable variation to outcomes (seed size, emergence, leaf and angle
      scatter, patchy soil). Realized h² should first be measured by parent-offspring
      regression on the WP30 bench.
    - **Heritable flowering time and basal root angle.** Both change the heritable schema
      and phenology code; they go to a later package with the Ppd photoperiod response.
    - **Maternal control of seed size.** This is a WP29 seed-set rule.

## Consequences
- Species schema 9 (the locus keys `line`, `frequencies` and `source`; `source` is
  required on every locus; a visual-only named locus may omit `trait`). Model 0.14.0.
- Golden hashes regenerate because the catalog and plant entries are hashed. Every
  `Default` seed's trajectory is unchanged: `daily.csv` and `organs.csv` are byte-identical
  to main for single_pot, pole_trellis and wheat_spaced over 40 days. Only `hashes.csv`
  differs.
- WP28 done-when:
  - Two `Sample` seeds differ measurably: `ps_world` `sampled_seeds_differ_measurably`.
    Leaf area at day 30 moves 14% and 47% from the `Default` twins with the same plant ids.
  - F2 variance exceeds F1 variance: `ps_species` `f2_variance_exceeds_f1`, and the 3 : 1
    habit split in `habit_segregates_three_to_one`.
- WP29 builds pollination and seed set on `Species::seed_genotype`, keyed as in item 7.
