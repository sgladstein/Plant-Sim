# Plant-to-plant variation

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Beans of one variety are close to genetically identical, yet no two plants in a row look
alike. Each seed is a little bigger or smaller and comes up a day earlier or later. It
lands a centimetre off the mark and a little deeper or shallower, in a patch of richer or
poorer soil, and its seedling faces its own way. Early, big seedlings get ahead and stay
ahead. The sim gives every plant these differences, drawn from its own dice, so the same
garden seed always grows the same garden.

## Equations
Each `z` is a standard normal held to ±2.5, from ps-rng keyed by (world seed, entity,
tick, purpose). `scatter(z, cv) = max(0.25, 1 + cv·z)`.

- Seed factor (at sowing, plant key, `SeedSize`): `f = scatter(z, seed_mass_cv)`. The
  sown seed mass is `f · seed_mass` (reserve, fixed tissue, seed N). The unifoliate
  potential area is `f^seed_leaf_exponent · unifoliate_area`.
- Emergence threshold (plant key, `Germination`):
  `E_i = max(E/4, (E + k_d·(d − d_ref)) · scatter(z, emergence_cv))`, with `E` the
  species' `phenology.emergence` (°C·day), `d` the sowing depth (cm) and `k_d`
  `emergence_per_depth` (°C·day/cm).
- Seedling azimuth (plant key, `Phyllotaxis`): `θ_0 ~ U[0, 2π)`. The cotyledons sit at
  `θ_0, θ_0 + π`, and the unifoliates and the main stem's first node turn with them.
- Divergence (each new node, `Phyllotaxis`): `Δθ = phyllotaxis + divergence_sd · z`.
- Leaf size (each new leaf or blade, its organ id, `LeafSize`):
  `A_pot = profile area · scatter(z, leaf_size_cv)`.
- Sowing (ps-world, plant id, `Sowing`): position `mark + σ_p·(z_x, z_y)`, depth
  `max(min(1 cm, d), d + σ_d·z_d)`. A point outside the soil falls back to the mark.
- Soil patches (ps-world, plot soils only, `SoilPatch`): standard normals `g_j` on a
  lattice `L` apart, bilinear weights `w_j` at each column centre,
  `G = Σ w_j g_j / sqrt(Σ w_j²)` (unit variance), factor
  `exp(σ G − σ²/2)` with `σ² = ln(1 + cv²)`. The factor multiplies the column's
  nitrate, ammonium and humus N at every depth.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| seed_mass_cv | 0.12 bean, 0.15 wheat | - | ~ single seeds of a lot, 10 to 15% (bean), 15 to 20% (wheat) |
| seed_leaf_exponent | 0.5 | - | ~ preformed primary leaves scale with seed size (seed-size vigour) |
| emergence_cv | 0.15 | - | ~ hydrothermal time, per-seed thresholds normally distributed (Gummerson 1986; Bradford 2002) |
| emergence_per_depth | 12 bean, 10.2 wheat | Cd/cm | ~ bean; CERES-Wheat 10.2 (Ritchie 1991) |
| emergence_reference_depth | 3 | cm | the depth the species' emergence sum was set for |
| leaf_size_cv | 0.1 | - | ~ |
| divergence_sd | 10 | deg | ~ 5 to 15 degrees in measured phyllotaxis |
| sowing_position_sd_cm | 1.0 | cm | ~ dibbed by hand at a mark (scenario `variation`) |
| sowing_depth_sd_cm | 0.5 | cm | ~ (scenario `variation`) |
| soil_n_cv | 0.25 | - | ~ field soil nitrate CV is commonly 30 to 60% at plant scale; kept lower |
| soil_patch_cm | 30 | cm | ~ patch size |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | world seed, plant id, soil id, sowing tick | - | scenario, world |
| in | `variation` group | see above | species file |
| in | `variation` | see above | scenario file (`FieldVariation`, default `GARDEN`) |
| out | `Plant::individual` (seed factor, emergence, azimuth) | -, Cd, rad | inspector, snapshot |
| out | scaled seed, thresholds, leaf areas, angles | | ps-arch |
| out | collar position, column N | m, g N | ps-arch, ps-soil |

## Encoded or emergent
Encoded: the size of each source of difference. Emergent: the size hierarchy. Early and
large seedlings shade their neighbours and keep their lead, and pod number and seed
yield spread wider than mass because pod set responds to the plant's own carbon.

## Known limits
- Only the thermal threshold to emergence varies. The water-potential threshold of
  germination and seed ageing or damage do not.
- Soil patches change nitrogen only, not water holding, texture or compaction. Pots are
  uniform.
- Pure lines have no genetic variance. Genotype sampling (`SeedChoice::Sample`) is M3.
- A vine sown more than about 6 cm from its stake lodges (decision 0021's fixed sweep),
  so sowing scatter makes a rare miss.
- Biomass CV at harvest (about 16% in the 30-bean plot) is below the 20 to 40% common in
  field stands, because size-asymmetric light competition is still weak (WP21 density
  follow-ups).

## Validation
- `ps_arch::tests::individuals_are_keyed_by_plant`,
  `deeper_seeds_emerge_later_and_no_variation_means_the_species_value`,
  `scatter_is_bounded`.
- `ps-world` `tests/determinism.rs`: `plot_plants_differ_and_keep_their_draws` (a
  plant's draws do not change when the plot is cut to 5 plants, NONE puts seeds on
  their marks), `plot_soil_nitrogen_is_patchy` (column CV 0.1 to 0.45, mean within 20%,
  pots uniform).
- `ps-arch` `tests/ninety_days.rs::counts_hold_across_world_seeds` (pod band on the
  8-seed mean).
- Golden hashes (`tests/golden/`) fix the draws bit for bit.

## References
- Bradford KJ (2002). Applications of hydrothermal time to quantifying and modeling seed
  germination and dormancy. Weed Science 50: 248–260.
- Gummerson RJ (1986). The effect of constant temperatures and osmotic potentials on the
  germination of sugar beet. J. Exp. Bot. 37: 729–741.
- Ritchie JT (1991). Wheat phasic development. In Hanks and Ritchie (eds), Modeling Plant
  and Soil Systems, Agronomy 31: 31–54 (CERES-Wheat emergence 40 + 10.2 °C·day per cm).
- Ross MA, Harper JL (1972). Occupation of biological space during seedling
  establishment. J. Ecol. 60: 77–88.
- Weiner J, Thomas SC (1986). Size variability and competition in plant populations.
  Oikos 47: 211–222.
- Decision 0027.
