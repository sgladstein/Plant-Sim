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

- **Primordium:** a new organ starts with `m₀ = min(0.02 · m_pot, 0.5 · R_parent / (0.45 · 1.25))`
  g DM built from its parent's reserve at the construction cost 1.25.
- **Nodes:** an axis adds a node each phyllochron `φ` of air thermal time. Main-stem
  trifoliate at rank `r` of `n`: potential area `A = A_pot · min(f₁ + (1 − f₁) min(1, (r − 1)/2), f_n + (1 − f_n) min(1, (n − r)/2))`.
  Main-stem internode `k` of `N` (k ≥ 2): length `ℓ = ℓ_pot · (g₁ + (g_N − g₁)(k − 2)/(N − 2))`; the epicotyl has `ℓ_pot`.
  Branches use the branch fractions. Potential mass is area over SLA or length over
  specific stem length.
- **Shade response (decision 0018):** the newest leaf below the node that has seen a
  whole day gives two signals, red:far-red `ρ` and its running mean daily absorbed PAR
  `D` (mol m⁻² day⁻¹; `D ← D + (D_yesterday − D)/3`). The response index is
  `x = max(clamp((0.7 − ρ)/(0.7 − 0.2), 0, 1), clamp((D_on − D)/(D_on − D_full), 0, 1))`
  with `D_on = 30`, `D_full = 8`, and `s(max) = 1 + (max − 1) · x`. It multiplies
  internode length (max 2.0), and the internode's specific length by the same factor,
  so a shaded internode is longer and thinner at the same potential mass; SLA (max
  1.4); and pod set (max 0.2).
- **Bud activation (daily):** `p = a · L · min(1, C_node / C_ref) · (1 − exp(−max(0, n − r − h)/d))`
  for dormant main-stem buds at `r ≤ N − 2` before flowering. The buds of the top `h`
  nodes under a growing apex stay dormant (WP25, decision 0024). The dominance term is 1
  once the main stem has terminated or its apex has been cut, so topping releases them.
- **Light index (daily):** `L = 0.5 · (mean over sun-up hours of P_sunlit + T_diffuse)`.
- **Pod set (decision 0018):** a flower closing after `n` open days sets with
  `p = p₀ · mean(F_C) · H · mean(F_L) · min(1, FTSW_min / 0.5)`, where
  `F_C = min(1, C_node / 0.22)` (its own reserve reference, so pod number follows the
  carbon supply), `F_L = s(0.2)` the shade term above, and
  `H = min(mean over the 7 days before opening of F_T(T_max), mean over the open days of F_T(T_max))`:
  heat counts while the pollen develops and while the flower is open, at the day's
  peak, and the flower is as fertile as its worse stage (so an ordinary warm day is
  not counted twice). `F_T` is the CROPGRO response: 0 at or outside `[T_min, T_max]`, 1 on
  `[T_o1, T_o2]`, `1 − x²` on the ramps.
- **Seeds per pod:** `Binomial(n_ovules, p_ovule · H)`, at least 1: heat also lowers
  the share of ovules that set.
- **Root segment:** length `Δ = min(2 cm, ℓ_max − ℓ)`, expansion duration
  `Δ / (e · scale / (T_opt − T_base))` °C·day, potential `Δ · m / SRL`,
  `SRL = 1 / (ρ_root π d² / 4)`.
- **Root direction:** `u' = unit(u + 0.2 · w_g · (0, 0, −1) + ε)`, `ε ~ N(0, 0.15²)`
  per component.
- **Lateral cohort:** one per 2 cm of parent past its apical zone, multiplicity
  `m = 2 cm / spacing`, length `ℓ_max` of the lateral type. Its direction is the
  insertion angle off the parent at the next golden-angle turn, mirrored downward. The
  cohort is one straight organ, so its tip must lie in active soil, as an axis segment
  must: the drawn direction, else the pot-wall rule (straight down, then along the wall
  either way), else the next seven golden-angle turns (decision 0028).
- **Sink feedback on gross assimilation:** `A' = f(C) (A + Rd) − Rd`,
  `f(C) = clamp(1 − (C − 0.10)/0.20, 0, 1)` (provisional; see gas-exchange).
- **Leaf death:** at thermal age `expanded + lifespan`, after 5 days below light index
  0.1, at frost, at starvation, or at maturity. A leaf freezes when its own lowest
  temperature of the day (energy balance, WP27) is below `leaf_frost_kill`, so on a
  clear, calm night exposed leaves can freeze in air just above the threshold while
  covered ones survive (owner decision 2026-11-05, decision 0034). A killing frost of
  the air (the day's minimum below the threshold) also stops every apex and kills buds
  and flowers, so the plant does not regrow (§7 M0 exit). Pods, seeds, stems and roots
  are spared: they stop growing only because the leaves are gone.

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
| apical_dominance_nodes, apical_hold_nodes | 0.7, 1.5 | nodes | "~" (decision 0024; Cline 1991 for the bean decapitation response) |
| flowers_per_raceme, raceme_flowering_days | 6, 10 | -, day | "~" §3.11 |
| pod_set_base | 0.42 | - | Binnie and Clifford 1981 (0.2 to 0.55) |
| pod_set_reserve_reference | 0.22 | g C / g DM | "~" (decision 0018) |
| drought_abortion_ftsw | 0.5 | - | CROPGRO SWFAC 0.5 |
| shade_dli_onset, shade_dli_full | 30, 8 | mol m⁻² day⁻¹ | "~" Poorter et al. 2019 range; calibrated (decision 0018) |
| shade_internode_factor, shade_sla_factor, shade_pod_set_factor | 2.0, 1.4, 0.2 | - | Lopes et al. 1987; Cossu et al. 2021 |
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
profiles, flowers per raceme, the probabilities of bud activation and pod set, the
maximum shade multipliers, and root type geometry. Emergent: how many buds actually
branch (light, carbon and apical dominance), how many flowers set pods (the carbon,
light, heat and drought they meet), how much a plant responds to shade (the light its
leaves actually absorb), final organ sizes (carbon shortfall), which leaves die in
shade, and where roots end up.

## Known limits
- No secondary thickening of stems or roots yet, so stems are light and the root
  system is about 4% of dry mass (root:shoot 0.04 at seed fill against 0.08 to 0.30
  published): a known limit at the M0 exit (WP12).
- From flowering on the plant is short of sinks: the sink feedback discards about 60%
  of potential gross assimilation, even at half light. So a source stress (shade,
  drought, heat) acts mainly through pod number, which pod set ties to carbon, light,
  water and heat, and only a little through filling. Leaves keep their capacity
  through seed fill (no nitrogen remobilization yet), one likely reason (M2).
- Heat lowers seed set but not seed size: `heat_seed_size` and `heat_seed_mass` stay
  short of the published -8 to -34% and -50 to -71% (Info). A 32 °C heat wave scores
  F_T 0.6, against 0.8 to 0.93 on an ordinary 28 to 30 °C day, so the CROPGRO curve
  separates a heat wave only weakly from summer; seeds fall 8 to 31% across four
  weather years.
- Drought raises root:shoot by 16 to 39%, but from a base 2 to 8 times too low.
- No phototropism or hydrotropism yet; roots bend only by gravity and noise.
- Laterals are lumped into cohorts and have no laterals of their own.
- Leaves do not senesce from the base during seed fill (§3.11; M2 with nitrogen).
- At maturity all leaves drop at once.
- Branches carry no axillary buds or racemes except their terminal one.

## Validation
- `tests/ninety_days.rs::ninety_day_counts_are_in_the_bean_ranges` (WP8 done-when,
  world seed 7): main-stem nodes 6 to 8, branches 3 to 6, pods 10 to 20 (§3.16), 19
  seed-borne root axes, more than 50 lateral cohorts and 10 m of root.
- `counts_hold_across_world_seeds` takes pods 6 to 20 (the §8.2 range) for seeds 1 to 8.
- The WP11 validation report (`ps-cli validate`): the §8.3 shade, drought, heat and
  frost responses, gated for M0. `ps-cli/tests/cli.rs::a_frost_night_kills_the_leaves_for_good`.
  `ps-arch/tests/leaf_energy.rs::a_clear_calm_night_frosts_exposed_leaves_but_not_the_growing_points`.
- `dry_soil_holds_the_seed`, `same_seed_same_plant` (bit-identical arena),
  `roots_stay_in_the_pot`.
- Debug builds assert the carbon ledger every hour (WP7) through all of these runs.
- `ps_arch::tests::*`: profiles, pod-set temperature, sink feedback, deflection.

## References
- Allen, M. T., Prusinkiewicz, P., DeJong, T. M. (2005). Using L-systems for modeling
  source-sink interactions, architecture and physiology of growing trees: the L-PEACH
  model. New Phytologist 166: 869-880.
- Cossu, M., Sirigu, A., Deligios, P. A., et al. (2021). Yield response and physiological
  adaptation of green bean to photovoltaic greenhouses. Frontiers in Plant Science 12:
  655851.
- Lopes, N. F., Oliva, M. A., Gomes, M. M. S., et al. (1987). Crescimento, morfologia,
  partição de assimilados e produção de matéria seca do feijão (Phaseolus vulgaris L.)
  submetido a três densidades do fluxo radiante e dois regimes hídricos. Revista Ceres
  34(192): 110-124.
- Poorter, H., et al. (2019). A meta-analysis of plant responses to light intensity for
  70 traits ranging from molecules to whole plant performance. New Phytologist 223:
  1073-1105.
- Binnie, R. C., Clifford, P. E. (1981). Flower and pod production in Phaseolus
  vulgaris. Journal of Agricultural Science 97: 397-402.
- Boote, K. J., et al. CROPGRO-Drybean species file (DSSAT).
- Schnepf, A., et al. (2018). CRootBox: a structural-functional modelling framework for
  root systems. Annals of Botany 121: 1033-1053.
- Postma, J. A., et al. (2017). OpenSimRoot: widening the scope and application of root
  architectural models. New Phytologist 215: 1274-1286.
