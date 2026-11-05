# Species and genome

## Purpose
A species is a recipe: how a bean builds leaves, how fast its roots grow, how it
responds to heat or shade. Some parts of the recipe are inherited and vary from seed to
seed (leaf size, branching, photosynthetic capacity); others are fixed chemistry that
every plant shares. In this version every bush bean of a variety has the same genes, as
in a real pure line. The differences you see between plants come from the seed each one
grew from, when it came up, and where it grows (the `variation` group, decision 0027;
see plant-variation.md), not from its genes.

## Equations
For heritable parameter `k` with species default `d_k`, bounds `[min_k, max_k]` and the
loci `L_k` that affect it, a plant with allele values `(a_l, b_l)` has

`p_k = clamp(d_k + Σ_{l ∈ L_k} [e_l (a_l + b_l)/2 + δ_l |a_l − b_l|], min_k, max_k)`

with additive effect `e_l` and dominance deviation `δ_l` (decision 0006). Alleles of a
sampled seed are drawn from `N(0, σ_l)` through ps-rng (purpose `GenotypeSample`).
In M0, `σ_l = 0` for every locus, so `p_k = d_k`.

### Meiosis, mutation and seed colour (WP28 groundwork, decision 0023; not yet called by the sim)
`ps_species::genetics` and `ps_species::seed_colour` hold the breeding engine. Nothing
in the step calls them yet, so model results are unchanged; pollination and seed set
(WP29) will.

- **Linkage.** A locus may carry `chromosome` (1 to 11) and `position_cm`; without them
  it is unlinked to every other locus. In a gamete each unlinked locus takes the
  maternal or paternal allele on a fair coin. On each chromosome a fair coin picks the
  starting strand and crossovers follow a Poisson process with one per 100 cM and no
  interference (Haldane), up to that chromosome's entry in `chromosome_lengths_cm`
  (100 cM each by default; BAT93 × Jalo is 1,097.5 cM over 11 chromosomes, from Pv09
  at 46.5 to Pv01 at 147, Blair 2018). A locus takes the strand after the
  number of crossovers before its position, so two loci `d` cM apart recombine with
  `r = (1 − e^(−2d/100)) / 2`.
- **Mutation, two tiers.** After recombination, each anonymous QTL mutates with
  probability `mutation_rate` by a step `N(0, (mutation_effect_sd · σ_l)²)`
  (0.15 allele SD by default); each named Mendelian locus (`tier: Named(states: n)`,
  allele states coded 0 to n − 1, 0 the recessive or null allele) mutates with
  probability `named_mutation_rate` (1e-5 by default) to one of its other states,
  uniformly. A locus with `σ_l = 0` therefore never changes, so the M0 pure lines breed
  true. These rates are a game approximation (Houle 1996 frames mutational variance near
  1e-3 of the environmental variance, [K, to verify]).
- **Seeds.** A seed's genotype is (egg, pollen): the egg from a meiosis of the mother,
  the pollen from a meiosis of the pollen parent (the mother again when selfed).
- **Keys.** A meiosis is keyed by (world seed, the seed's ovule entity, tick of
  fertilization) and its side (egg or pollen). Recombination draws use purpose
  `Recombination` (code 10, reserved since WP1; it is decision 0023's "Meiosis"
  purpose) with a fixed block of draws per side and chromosome; mutation draws use
  `Mutation` with a fixed slot per side and locus. Decision 0023 keyed by plant id and
  chromosome; keying by the ovule instead keeps sibling seeds formed in one tick, and
  the two gametes of a selfed seed, independent. Adding a locus never shifts another's
  draws.
- **Seed colour.** A table of layers evaluated in order; in each, the first rule whose
  conditions hold (`Carries(locus, state)`: at least one allele; `Homozygous(locus,
  state)`: both; `Expressed(locus, state)`: the higher code, since codes rise with
  dominance, so an allelic series such as V > v^lae > v reads directly) sets some of
  pigment colour, pattern ground, pattern name, coverage (the pigmented share of the
  coat), shine and flower colour, and may stop the evaluation (the *P* gate). It drives only appearance. The bean table itself is
  still to be written from Bassett 2007, Parker 2024, Plestenjak 2025 and McClean 2018.

## Parameters
All parameters, with values, units and sources, are in `data/species/bush_bean.ron`,
`data/species/pole_bean.ron` and `data/species/spring_wheat.ron` (schema 3: the
`climbing` group, WP22, decision 0021; every species carries it and only vines read it;
schema 4: the optional `grass` group, WP23, decision 0022, present exactly when the
growth form is `Grass`; schema 5: the constant `apical_hold_nodes`, WP25, decision 0024; schema 6: `g_function`
removed, since each leaf's own angle sets its light interception, decision 0025);
the expected units are in `crates/ps-species/src/schema.rs`. The loader rejects a wrong
unit, an empty source, an unknown or missing key, a non-finite value, and a default
outside its bounds.

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Mutation rate (QTL) | 0.001 | per locus per gamete | design plan §3.12 (used from M3) |
| `named_mutation_rate` (optional) | 1e-5 if absent | per locus per gamete | decision 0023 |
| `mutation_effect_sd` (optional) | 0.15 if absent | allele SD | decision 0023 (0.1 to 0.2) |
| `chromosome_lengths_cm` (optional, 11 values) | 100 each if absent | cM | decision 0023; Blair 2018 |
| Loci | one per heritable parameter | - | M0 identity mapping |
| Allele SD | 0 | trait unit | M0 identity mapping |

Root types (WP8, decision 0013) are a list of CRootBox-style entries, each with a name,
an origin (`Radicle`, `Basal`, `Hypocotyl` or `Lateral`), a branching order, and
`count`, `emergence` (°C·day after germination), `elongation` (cm per day at the
optimum temperature), `max_length`, `branch_spacing`, `apical_zone` (cm),
`insertion_angle` (°), `diameter` (mm) and tropism weights. The loader requires exactly
one radicle, unique names, order 0 for seed-borne types and order above 0 for laterals,
and a parent order for every lateral order. Specific root length follows from diameter
and `root_density`.

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | species file | - | data/species/*.ron |
| in | world seed, plant entity | - | scenario, world |
| out | genotype, phenotype, typed `Traits` view (`Phenotype::traits`) | trait units | architecture, physiology |

## Encoded or emergent
Encoded: the mapping. Trade-offs between traits are not tabulated here; they arise from
physiology (§3.12).

## Known limits
- The shipped species files place no loci on chromosomes and have no named loci yet, so
  every locus is unlinked; the meiosis engine is built but not called by the sim (WP29).
- One genetic length for every chromosome; no crossover interference.
- Named-locus mutation picks any other state with equal chance (no forward/back bias).
- Phenology thresholds are not yet heritable (decision 0006).
- The dominance form is provisional until breeding (M3).

## Validation
- `ps_species::tests::bean_file_loads` (WP4 done-when 1).
- `ps_species::tests::wrong_unit_is_a_clear_error`, `missing_source_is_a_clear_error` (WP4 done-when 2).
- `ps_species::tests::identity_mapping`, `nonzero_variance_varies_and_stays_in_bounds`.
- `ps_species::tests::every_parameter_has_unit_and_source`, `round_trip_through_ron`.
- `ps_arch::tests::root_types_convert_to_model_units` (root type conversion).
- `ps_species::genetics_tests::recombination_follows_the_map` (Haldane r at 10, 20, 100 cM; 0.5 unlinked and across chromosomes), `meiosis_is_keyed`, `adding_a_locus_keeps_other_draws`, `m0_species_breeds_true`.
- `ps_species::genetics_tests::f2_variance_exceeds_f1` (WP28 done-when 2 on a 12-locus test genome), `two_tier_mutation`, `linkage_map_is_validated`, `optional_genome_keys_round_trip`.
- `ps_species::genetics_tests::seed_colour_table_evaluates_in_layers`, `seed_colour_table_is_validated`.

## References
- Blair 2018, BAT93 × Jalo map length, as cited in decision 0023 and the M2/M3 research report (full citation in that report).
- Haldane, J. B. S. (1919). The combination of linkage values and the calculation of distances between the loci of linked factors. J. Genet. 8: 299-309.
- Houle, D., Morikawa, B., Lynch, M. (1996). Comparing mutational variabilities. Genetics 143: 1467-1483.
- Falconer, D. S., Mackay, T. F. C. (1996). Introduction to Quantitative Genetics, 4th ed. (additive and dominance effects).
