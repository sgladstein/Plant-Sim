# Herb architecture and roots

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
This is how a bush bean is put together. The seed swells, a hook pushes up through the
soil carrying two seed leaves, and a pair of simple leaves opens. Then the stem adds a
three-part leaf every few warm days until it ends in a flower cluster. Buds in the leaf
angles may grow into side branches if they get light and sugar. Flowers turn into pods
or drop, depending on heat, drought and how well fed the plant is. Underground, a tap
root and whorls of side roots spread through the pot.

## Equations
Thermal time `τ` is the species three-point response (°C·day; see
thermal-time-and-phenology). Daily steps run in §3.2 order: phenology, bud fate,
initiation, senescence, root elongation, then light and root-cell rebuild.

- **Primordium:** a new organ starts with `m₀ = min(0.02 · m_pot, 0.5 · R_parent / 0.45)`
  g DM taken from its parent's reserve.
- **Nodes:** an axis adds a node each phyllochron `φ` of air thermal time. Main-stem
  trifoliate at rank `r` of `n`: potential area `A = A_pot · min(f₁ + (1 − f₁) min(1, (r − 1)/2), f_n + (1 − f_n) min(1, (n − r)/2))`.
  Main-stem internode `k` of `N` (k ≥ 2): length `ℓ = ℓ_pot · (g₁ + (g_N − g₁)(k − 2)/(N − 2))`; the epicotyl has `ℓ_pot`.
  Branches use the branch fractions. Potential mass is area over SLA or length over
  specific stem length.
- **Shade avoidance:** at red:far-red `ρ` of the newest leaf below the node,
  `s(ρ, max) = 1 + (max − 1) · clamp((0.7 − ρ)/(0.7 − 0.2), 0, 1)` multiplies internode
  length (max 1.5) and SLA (max 1.3).
- **Bud activation (daily):** `p = a · L · min(1, C_node / C_ref) · (1 − exp(−(n − r)/d))`
  for dormant main-stem buds at `r ≤ N − 2` before flowering.
- **Light index (daily):** `L = 0.5 · (mean over sun-up hours of P_sunlit + T_diffuse)`.
- **Pod set:** `p = p₀ · mean(F_C) · mean(F_T) · min(1, FTSW_min / 0.3)`, with
  `F_C = min(1, C_node / C_ref)` and `F_T` the CROPGRO response: 0 at or outside
  `[T_min, T_max]`, 1 on `[T_o1, T_o2]`, `1 − x²` on the ramps.
- **Seeds per pod:** `Binomial(n_ovules, p_ovule)`, at least 1.
- **Root segment:** length `Δ = min(2 cm, ℓ_max − ℓ)`, expansion duration
  `Δ / (e · scale / (T_opt − T_base))` °C·day, potential `Δ · m / SRL`,
  `SRL = 1 / (ρ_root π d² / 4)`.
- **Root direction:** `u' = unit(u + 0.2 · w_g · (0, 0, −1) + ε)`, `ε ~ N(0, 0.15²)`
  per component.
- **Lateral cohort:** one per 2 cm of parent past its apical zone, multiplicity
  `m = 2 cm / spacing`, length `ℓ_max` of the lateral type.
- **Sink feedback on leaf capacity:** `f(C) = clamp(1 − (C − 0.10)/0.20, 0, 1)` (see
  gas-exchange).

## Parameters
All values, units and sources are in `data/species/bush_bean.ron`. The main ones:

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| nodes_before_terminal | 7 | - | "~" design plan §3.16 |
| phyllochron | 55 | °C·day | "~" research review |
| leaf_area_potential | 133 | cm² | "~" research review |
| unifoliate_area | 20 | cm² | "~" |
| internode_length_potential | 4.5 | cm | "~" Kumar 2025 |
| branch_nodes, branch_angle | 3, 45 | -, ° | "~" |
| bud_activation | 0.6 | per day | "~" |
| apical_dominance_nodes | 2 | nodes | "~" |
| flowers_per_raceme, raceme_flowering_days | 6, 10 | -, day | "~" §3.11 |
| pod_set_base | 0.4 | - | Binnie and Clifford 1981 |
| pod_set_t (min, opt, opt, max) | 7, 22, 25, 36 | °C | CROPGRO bean |
| seeds_per_pod, ovule_set | 5, 0.75 | -, - | "~" Kumar 2025 |
| Root types | tap, two basal whorls of 4, 10 hypocotyl-borne, laterals | see file | OpenSimRoot bean (values only) |
| root_segment_length | 2 | cm | "~" (decision 0013) |
| PRIMORDIUM | 0.02 | - | "~" code constant (decision 0013) |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | hourly weather, soil temperature and FTSW | °C, - | ps-weather, ps-soil |
| in | next day's sun path | unit vectors | world |
| out | organs, positions (base, tip per organ), roles, multiplicities | m | graph, renderer |
| out | root length per soil cell | m | ps-soil uptake |
| out | leaf points | m, m² | ps-light |
| out | daily report (counts, masses, height) | - | tests, CSV |

## Encoded or emergent
Encoded: node timing, the number of main-stem nodes, branch length, leaf and internode
profiles, flowers per raceme, the probabilities of bud activation and pod set, and
root type geometry. Emergent: how many buds actually branch (light, carbon and apical
dominance), how many flowers set pods (the carbon, heat and drought they meet), final
organ sizes (carbon shortfall), which leaves die in shade, and where roots end up.

## Known limits
- No secondary thickening of stems yet, so stems are light; the root system is about
  4% of dry mass, low for bean (WP12).
- No phototropism or hydrotropism yet; roots bend only by gravity and noise.
- Laterals are lumped into cohorts and have no laterals of their own.
- Leaves do not senesce from the base during seed fill (§3.11; M2 with nitrogen).
- At maturity all leaves drop at once.
- Branches carry no axillary buds or racemes except their terminal one.

## Validation
- `tests/ninety_days.rs::ninety_day_counts_are_in_the_bean_ranges` (WP8 done-when,
  world seed 7): main-stem nodes 6 to 8, branches 3 to 6, pods 10 to 20 (§3.16), 19
  seed-borne root axes, more than 50 lateral cohorts and 10 m of root.
- `counts_hold_across_world_seeds`: the same ranges for world seeds 1 to 8.
- `dry_soil_holds_the_seed`, `same_seed_same_plant` (bit-identical arena),
  `roots_stay_in_the_pot`.
- Debug builds assert the carbon ledger every hour (WP7) through all of these runs.
- `ps_arch::tests::*`: profiles, pod-set temperature, sink feedback, deflection.

## References
- Allen, M. T., Prusinkiewicz, P., DeJong, T. M. (2005). Using L-systems for modeling
  source-sink interactions, architecture and physiology of growing trees: the L-PEACH
  model. New Phytologist 166: 869-880.
- Binnie, R. C., Clifford, P. E. (1981). Flower and pod production in Phaseolus
  vulgaris. Journal of Agricultural Science 97: 397-402.
- Boote, K. J., et al. CROPGRO-Drybean species file (DSSAT).
- Schnepf, A., et al. (2018). CRootBox: a structural-functional modelling framework for
  root systems. Annals of Botany 121: 1033-1053.
- Postma, J. A., et al. (2017). OpenSimRoot: widening the scope and application of root
  architectural models. New Phytologist 215: 1274-1286.
