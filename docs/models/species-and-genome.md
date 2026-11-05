# Species and genome

## Purpose
A species is a recipe: how a bean builds leaves, how fast its roots grow, how it
responds to heat or shade. Some parts of the recipe are inherited and vary from seed to
seed (leaf size, branching, photosynthetic capacity); others are fixed chemistry that
every plant shares. In this first version every bush bean seed is identical, so
differences you see between plants come from where they grow, not from their genes.

## Equations
For heritable parameter `k` with species default `d_k`, bounds `[min_k, max_k]` and the
loci `L_k` that affect it, a plant with allele values `(a_l, b_l)` has

`p_k = clamp(d_k + Σ_{l ∈ L_k} [e_l (a_l + b_l)/2 + δ_l |a_l − b_l|], min_k, max_k)`

with additive effect `e_l` and dominance deviation `δ_l` (decision 0006). Alleles of a
sampled seed are drawn from `N(0, σ_l)` through ps-rng (purpose `GenotypeSample`).
In M0, `σ_l = 0` for every locus, so `p_k = d_k`.

## Parameters
All parameters, with values, units and sources, are in `data/species/bush_bean.ron`,
`data/species/pole_bean.ron` and `data/species/spring_wheat.ron` (schema 3: the
`climbing` group, WP22, decision 0021; every species carries it and only vines read it;
schema 4: the optional `grass` group, WP23, decision 0022, present exactly when the
growth form is `Grass`; schema 5: the constant `apical_hold_nodes`, WP25, decision 0024);
the expected units are in `crates/ps-species/src/schema.rs`. The loader rejects a wrong
unit, an empty source, an unknown or missing key, a non-finite value, and a default
outside its bounds.

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Mutation rate | 0.001 | per locus per gamete | design plan §3.12 (used from M3) |
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
- No linkage; loci assort independently.
- Phenology thresholds are not yet heritable (decision 0006).
- The dominance form is provisional until breeding (M3).

## Validation
- `ps_species::tests::bean_file_loads` (WP4 done-when 1).
- `ps_species::tests::wrong_unit_is_a_clear_error`, `missing_source_is_a_clear_error` (WP4 done-when 2).
- `ps_species::tests::identity_mapping`, `nonzero_variance_varies_and_stays_in_bounds`.
- `ps_species::tests::every_parameter_has_unit_and_source`, `round_trip_through_ron`.
- `ps_arch::tests::root_types_convert_to_model_units` (root type conversion).

## References
- Falconer, D. S., Mackay, T. F. C. (1996). Introduction to Quantitative Genetics, 4th ed. (additive and dominance effects).
