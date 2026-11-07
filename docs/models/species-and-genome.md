# Species and genome

## Purpose
A species is a recipe: how a bean builds leaves, how fast its roots grow, how it
responds to heat or shade. Some parts of the recipe are inherited and vary from seed to
seed (leaf size, branching, photosynthetic capacity); others are fixed chemistry that
every plant shares. A species file describes one pure line, as a bean variety is: every
`Default` seed has the same genes. The differences between such plants come from the
seed each one grew from, when it came up, and where it grows (the `variation` group,
decision 0027; see plant-variation.md). A `Sample` seed is a different pure line drawn
from the species population, and crosses between lines segregate (decision 0031).

## Equations
For heritable parameter `k` with species default `d_k`, bounds `[min_k, max_k]` and the
loci `L_k` that affect it, a plant with allele values `(a_l, b_l)` has

`p_k = clamp(d_k + Σ_{l ∈ L_k} [e_l (a_l + b_l)/2 + δ_l |a_l − b_l|], min_k, max_k)`

with additive effect `e_l` and dominance deviation `δ_l` (decision 0006). A `Default`
seed is homozygous for each locus's `line` allele (0 unless set), so a QTL adds nothing
and `p_k = d_k`. The pole bean's line carries Fin Fin, which sets its habit to 1 through
the fin locus (effect 1, dominance 0.5).

A `Sample` seed is a pure-line founder (decision 0031): each locus gets one value, put on
both homologues, drawn through ps-rng (purpose `GenotypeSample`, four draws per locus):
`line + N(0, σ_l)` for a QTL, and a state from the locus's `frequencies` for a named
locus. `σ_l = CV_G · d_k / √n_k` over the trait's `n_k` loci, so the trait's genetic SD
across founders is `CV_G · d_k`. The genotype is stored with the plant.

### Meiosis, mutation and seed colour (decisions 0023, 0031 and 0035)
`ps_species::genetics` and `ps_species::seed_colour` hold the breeding engine.
Pollination and seed set (WP29, below) call meiosis; the viewer and the genotype lens
(WP31) will call the colour table.

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
  the pollen from a meiosis of the pollen parent (the mother again when selfed), each
  with its own species' map and rates (`Species::cross_genotype`).
- **Keys.** A meiosis is keyed by (world seed, the seed's own organ entity
  `(mother << 32) | organ`, the tick the seed organ was made at seed set) and its side
  (egg or pollen). Recombination draws use purpose
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
  coat), shine and flower colour, and may stop the evaluation (the *P* gate). It drives
  only appearance. The bean table is `data/genetics/bean_seed_colour.ron`: P gate,
  flower (tt whitens it; otherwise V, v^lae, v), hue (C^R red, cc buff, j pale, V with
  or without B, rk kidney reds, then G and B browns), pattern (C^mot mottle on buff),
  partly coloured patterns under tt (from Z, Bip and J), and shine (Asp).

## Parameters
All parameters, with values, units and sources, are in `data/species/bush_bean.ron`,
`data/species/pole_bean.ron` and `data/species/spring_wheat.ron` (schema 3: the
`climbing` group, WP22, decision 0021; every species carries it and only vines read it;
schema 4: the optional `grass` group, WP23, decision 0022, present exactly when the
growth form is `Grass`; schema 5: the constant `apical_hold_nodes`, WP25, decision 0024; schema 6: `g_function`
removed, since each leaf's own angle sets its light interception, decision 0025;
schema 9: per-locus `line`, `frequencies` and a required `source`, visual-only named
loci without a `trait`, and the genome's `chromosome_lengths_cm`, decision 0031;
schema 12: the `thickening` group, the `root_secondary_rate` table and each root
type's `secondary_factor` table, decision 0036. A table is a list of `(x, y)` points
with an x unit, a unit and a source, read by linear interpolation and held level past
its ends; the loader checks its units, that x rises and that every value is finite and not
negative; schema 13: `root_reach` removed, since nitrogen uptake reads each cell's own
root length, and the beans gain a `fine_lateral` root type of order 2, decision 0037;
schema 14: the `storage` group (starch store and root exudation) and
`nitrogen.seed_fill_mining_first`, whose rates and shares the loader keeps within 0 and
1 (a day's fill release within 24) with release below refill, decision 0040;
schema 15: `phototropism_weight` (0 to 1) and `phototropism_max_per_day` (0 to 90°),
decision 0041; schema 16: the root plasticity constants `root_dry_full`,
`root_dry_floor`, `xerobranching_threshold`, `hydrotropism_weight` (0 to 1),
`root_proliferation_nitrate`, `root_proliferation_water` (0 to 5) and `hydropatterning`
(0 or 1), decision 0043; schema 17: the plant hydraulics constants
`root_radial_conductivity` and `_thickened`, `root_axial_conductance_tip`, `_tap` and
`_basal`, `root_axial_maturation`, `stem_xylem_conductivity`, `xylem_area_fraction`
(above 0, at most 1), `leaf_hydraulic_conductance`, `tuzet_sensitivity`,
`tuzet_reference` (negative), `turgor_loss_point` and `turgor_full` (loss point below
full, full at most 0), all positive where not said otherwise, replacing
`ftsw_threshold`, `root_uptake_rate` and `root_uptake_max`, decision 0044); schema
18: `hydraulic_compensation` (at least 0; the simplified water network's `K_comp` over
`K_rs`, decision 0045);
the expected units are in `crates/ps-species/src/schema.rs`. The loader rejects a wrong
unit, an empty source, an unknown or missing key, a non-finite value, and a default
outside its bounds.

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Mutation rate (QTL) | 0.001 | per locus per gamete | design plan §3.12 (used from M3) |
| `named_mutation_rate` (optional) | 1e-5 if absent | per locus per gamete | decision 0023 |
| `mutation_effect_sd` (optional) | 0.15 if absent | allele SD | decision 0023 (0.1 to 0.2) |
| `chromosome_lengths_cm` (optional, 11 values) | 100 each if absent | cM | decision 0023; Blair 2018 |
| Loci (beans) | 24 QTL over 15 traits, 16 named | - | decision 0031; docs/research/m3-breeding-and-weeds |
| Allele SD (bean QTL) | `CV_G · default / √n`, CV_G 0.08 to 0.20 | trait unit | docs/research/m3-breeding-and-weeds, breeding_quantitative ("~") |
| Dominance (bean QTL) | +0.15 vigour traits, −0.05 seeds per pod, 0 otherwise | trait unit per allele difference | decision 0031 ("~") |
| Chromosome lengths (beans) | Pv01 147, Pv09 46.5, others 100 | cM | Blair 2018; "~" for the 100s |
| Loci (wheat) | one per heritable parameter, zero variance | - | M0 identity mapping |

The bean named loci (state codes rise with dominance), with chromosome and position in
cM; the full sources are in each locus's `source`:

| Locus | States | Pv | cM | Acts on |
| --- | --- | --- | --- | --- |
| fin | fin, Fin | 01 | 77 | habit |
| Ppd | ppd, Ppd | 01 | 85 | nothing yet (photoperiod not modelled) |
| I | i, I | 02 | 1 | nothing yet (no disease) |
| B | b, B | 02 | 2 | colour |
| St | st, St | 02 | 15 | nothing yet (no pod strings) |
| Rk | rk, Rk | 02 | 30 | colour |
| Z | z, z^sel, Z | 03 | 60 | pattern |
| G | g, G | 04 | 50 | colour |
| V | v, v^lae, V | 06 | 10 | flower, colour |
| Asp | asp, Asp | 07 | 75 | shine |
| Phs | S, T | 07 | 91 | marker |
| P | p, p^gri, P | 07 | 100 | colour gate |
| C | c, C^mot, C, C^R | 08 | 40 | colour, pattern |
| T | t, T | 09 | 15 | pattern gate, flower |
| Bip | bip, bip^ana, Bip | 10 | 40 | pattern |
| J | j, J | 10 | 67 | colour, pattern |

The seed-mass QTL SW7 sits on Pv07 at 90 cM, 10 cM from P (decision 0023).

Root types (WP8, decision 0013) are a list of CRootBox-style entries, each with a name,
an origin (`Radicle`, `Basal`, `Hypocotyl` or `Lateral`), a branching order, and
`count`, `emergence` (°C·day after germination), `elongation` (cm per day at the
optimum temperature), `max_length`, `branch_spacing`, `apical_zone` (cm),
`insertion_angle` (°), `diameter` (mm) and tropism weights. The loader requires exactly
one radicle, unique names, order 0 for seed-borne types and order above 0 for laterals,
and a parent order for every lateral order. Specific root length follows from diameter
and `root_density`.

### Pollination and seed set (WP29, decision 0035)
- Flowers self: each seed's father is its mother. Natural outcrossing is not modelled.
- `Pollinate { plant, organ, pollen }` hand-pollinates an open flower with another
  plant's pollen. The flower is taken as emasculated in time, and every seed of its pod
  has `pollen` as father. Pod set follows the usual rule (no handling penalty).
- Checks: the organ is a living open flower that is not yet crossed; the pollen plant
  has an open flower that tick; the two species cross (`Species::can_cross`: the same
  loci with the same trait, symbol, tier, chromosome and position, and the same
  chromosome lengths). Bush and pole bean cross; bean and wheat do not.
- A seed's genotype is fixed by its key and both parents, so it is computed when needed
  (`World::seed_genotype` for a seed in its pod) and written to the seed lot at harvest
  (`SeedLot::seed_genotypes`: organ, mother, father, genotype). In the world's lot its
  species is its mother's. When the seed is saved to the seed library, it takes the bean
  file that matches its habit (breeding-bench.md, decision 0038 item 7).
- Seed size stays maternal: a seed fills to its mother's heritable `seed_mass`.

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | species file | - | data/species/*.ron |
| in | world seed, plant entity | - | scenario, world |
| out | genotype, phenotype, typed `Traits` view (`Phenotype::traits`) | trait units | architecture, physiology |
| in | hand-pollination (`ActionKind::Pollinate`) | - | player, scenario events |
| out | seed parents and genotypes (`SeedLot::seed_genotypes`, `World::seed_genotype`) | - | seed library, inspector |

## Encoded or emergent
Encoded: the mapping. Trade-offs between traits are not tabulated here; they arise from
physiology (§3.12).

## Known limits
- Pollination is selfing or one hand cross per flower: no pollinator outcrossing, pollen
  competition or mixed paternity in a pod, and no handling penalty on a crossed flower's
  pod set (decision 0035).
- Seed size is the mother's heritable value; the embryo's own genotype does not change it.
- No crossover interference. Only Pv01 and Pv09 have published lengths; the rest are 100 cM.
- Most positions and every allele frequency are "~" placements; only Rk–B (28 cM),
  P–Phs (9 cM) and B–I (very tight) are published distances.
- Ppd, I and St have no physiological effect until photoperiod, disease and pod strings
  exist. The direction of St (which allele is stringless) conflicts between sources.
- No per-plant micro-environment on heritable parameters; realized h² is to be measured
  on the WP30 bench (decision 0031).
- Named-locus mutation picks any other state with equal chance (no forward/back bias).
- Phenology thresholds are not yet heritable (decision 0006).
- The dominance form is provisional until breeding (M3).

## Validation
- `ps_species::tests::bean_file_loads` (WP4 done-when 1).
- `ps_species::tests::wrong_unit_is_a_clear_error`, `missing_source_is_a_clear_error` (WP4 done-when 2).
- `ps_species::tests::default_seed_is_the_pure_line`, `sampled_seeds_differ`, `nonzero_variance_varies_and_stays_in_bounds`.
- `ps_world` `sampled_seeds_differ_measurably` (WP28 done-when 1: leaf area at day 30 moves 14% and 47% from the `Default` twins).
- `ps_species::tests::every_parameter_has_unit_and_source`, `round_trip_through_ron`.
- `ps_arch::tests::root_types_convert_to_model_units` (root type conversion).
- `ps_species::genetics_tests::recombination_follows_the_map` (Haldane r at 10, 20, 100 cM; 0.5 unlinked and across chromosomes), `meiosis_is_keyed`, `adding_a_locus_keeps_other_draws`, `pure_lines_breed_true`, `habit_segregates_three_to_one` (bush × pole F2 is 3 : 1 climbing).
- `ps_world` `a_cross_records_both_parents_in_each_seed` (WP29 done-when: every F1 seed
  names mother and father and pairs their alleles at every locus; selfed seeds name the
  mother twice; the run replays to the same hashes), `a_selfed_heterozygote_segregates_seed_by_seed`
  (23 seeds, all distinct; all 32 heterozygous loci segregate), `pollination_is_checked`.
- `ps_species::genetics_tests::crosses_need_one_genome_layout` (bush × pole cross, bean ×
  wheat does not; the F1 is Fin fin and climbs).
- `ps_species::genetics_tests::f2_variance_exceeds_f1` (WP28 done-when 2 on a 12-locus test genome), `two_tier_mutation`, `linkage_map_is_validated`, `optional_genome_keys_round_trip`.
- `ps_species::genetics_tests::seed_colour_table_evaluates_in_layers`, `seed_colour_table_is_validated`, `expressed_reads_an_allelic_series`, `lines_and_frequencies_are_validated`.
- `ps_species::genetics_tests::bean_seed_colour_table`: black (P C J G B V), dark brown violet, dark brown, yellow brown, dark red kidney, Prim yellow with a pink flower, grey-white, and the tt patterns (self, virgarcus, bipunctata, anasazi, sellatus) against the research rows.

## References
- Blair 2018, BAT93 × Jalo map length, as cited in decision 0023 and the M2/M3 research report (full citation in that report).
- Haldane, J. B. S. (1919). The combination of linkage values and the calculation of distances between the loci of linked factors. J. Genet. 8: 299-309.
- Houle, D., Morikawa, B., Lynch, M. (1996). Comparing mutational variabilities. Genetics 143: 1467-1483.
- Falconer, D. S., Mackay, T. F. C. (1996). Introduction to Quantitative Genetics, 4th ed. (additive and dominance effects).
