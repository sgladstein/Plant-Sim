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
- **Shoot direction and phototropism (R1c, decision 0041):** a new internode of a bean
  axis starts from its parent's direction `u`, turns toward the light its apex saw,
  then toward vertical: `u₁ = rotate(u → v̂, min(θ_max · d, k_p · ∠(u, v̂)))`,
  `u' = unit(u₁ + g · (0, 0, 1))`, with `g = 0.3` (shoot gravitropism), `k_p = 0.3`,
  `θ_max = 10°` per day and `d` the days of light seen since the axis' last node.
  `v = Σ_hours (I_b,h T_h ŝ_h + I_d,h Σ_s a_s T_s d̂_s)` is the vector irradiance at the
  apex tip summed hour by hour since the last node: `I_b` the beam normal to the sun,
  `I_d` the horizontal diffuse irradiance, `T` the transmittance from the tip toward
  each hour's sun and each of 16 sky sectors (light card), `a_s` a sector's radiance
  times solid angle. Its horizontal part is the light difference across the stem. Only
  new internodes turn; grown ones keep their direction. A twining vine's internode
  follows its support, and a grass does not bend. Nodes come 3 to 5 days apart, so the
  daily bound allows 30 to 50° per node while `k_p · ∠` is at most 27° for light within
  90°: the bound does not bind today, and `k_p` alone sets the response (30° a day gives
  bit-identical runs; review of decision 0041). Under a fixed light direction at angle
  `α` from vertical a stem settles at about `α · k_p / (k_p + g) = α/2`.
- **Shade response (decision 0018):** the newest leaf below the node that has seen a
  whole day gives two signals, red:far-red `ρ` and its running mean daily absorbed PAR
  `D` (mol m⁻² day⁻¹; `D ← D + (D_yesterday − D)/3`). The response index is
  `x = max(clamp((0.7 − ρ)/(0.7 − 0.2), 0, 1), clamp((D_on − D)/(D_on − D_full), 0, 1))`
  with `D_on = 30`, `D_full = 8`, and `s(max) = 1 + (max − 1) · x`. It multiplies
  internode length (max 2.6, Lopes 1987's 30% light point; was 2.0, decision 0037),
  and the internode's specific length by the same factor, so a shaded internode is
  longer and thinner at the same potential mass; SLA (max 1.4); and pod set (to 0.5 at
  full shade; was 0.2, decision 0037).
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
  `D = Δ / (e · scale / (T_opt − T_base))` °C·day, potential `Δ · m / SRL`,
  `SRL = 1 / (ρ_root π d² / 4)`. Each daily step, an axis whose tip has started
  expanding queues the next segment at the tip's planned end with its expansion clock
  at `−(D_tip − age_tip)`, so it starts growing the moment the tip finishes and the
  tip moves at the root's elongation rate rather than one segment per daily step
  (R2a, decision 0037). The axis length `ℓ` counts finished segments at the length
  they reached; segment starts are restated from those lengths each day.
- **Root tip feeding (R2a):** a growing root organ draws on its own reserve and on its
  parent root's (or the seed's) reserve above the floor, at its priority: phloem
  unloads at the tip from the root behind it, and a primordium's own reserve alone held
  it to a few percent of growth per hour.
- **Root direction:** `u' = unit(u + 0.2 · w_g · (0, 0, −1) + ε)`, `ε ~ N(0, 0.15²)`
  per component.
- **Lateral cohort:** one per 2 cm of parent past its apical zone, multiplicity
  `m = 2 cm / spacing`, length `ℓ_max` of the lateral type. Its direction is the
  insertion angle off the parent at the next golden-angle turn, mirrored downward. The
  cohort is one straight organ, so its tip must lie in active soil, as an axis segment
  must: the drawn direction, else the pot-wall rule (straight down, then along the wall
  either way), else the next seven golden-angle turns (decision 0028), else level
  toward the plant's collar; where none fits (a pot's bottom edge) the cohort is not
  made (decision 0037).
- **Fine-root cohort (R2a, decision 0037):** once a first-order cohort finishes, it
  gets one second-order cohort of the fine-lateral type standing for its fine roots,
  `m₂ = m₁ (ℓ₁ − a₁) / s₁` (realized length past the lateral's apical zone `a₁` over the
  fine-root spacing `s₁`), attached in the middle of that stretch, built like a lateral
  cohort. Fine roots are 0.1 mm across (about 1,350 m/g), live in the cells next to
  their lateral and do not thicken.
- **Root plasticity (R2b, decision 0043).** Every rule reads soil cells at the daily
  root step and draws no random numbers.
  - *Dry soil slows elongation:* each growing root organ's expansion clock runs at
    `f_w = max(floor, clamp(θ/θ_full, 0, 1))` for the day, with `θ` the relative
    available water of its tip's cell, `θ_full` 0.3 and floor 0.3 (root tips keep
    elongating near −1.6 MPa). Set after the day's layout; hourly step 11 multiplies
    the organ's thermal time by it.
  - *Xerobranching:* no lateral or fine-root cohort forms where the parent's cell is
    below `θ` 0.1, and none forms there later.
  - *Proliferation:* a cohort stands for `m · F` roots, `F` the mean over its straight
    path (sampled every half cell) of `(1 + a_N r_N)(1 + a_W r_W)`,
    `r = clamp((v − v̄)/v̄, 0, 2)` for the cell's nitrate (mg N/kg) and `θ`, with `v̄` the
    larger of the soil's mean over its active cells and the root-length-weighted mean
    over the plant's root cells; `a_N` 1, `a_W` 0.5. The soil's mean keeps the roots'
    own depletion from making fresh soil look rich; the roots' mean keeps nitrate that
    the water carried to them, and left there, from looking rich to the roots already
    in it. Carbon still limits: `F` raises the cohort's potential, not its supply.
  - *Hydropatterning:* a cohort's direction is the first of the eight golden-angle
    candidates whose tip cell holds at least their median `θ`, where they differ;
    otherwise the rule above.
  - *Hydrotropism:* an axis tip's direction gains `w_h ĝ`, `ĝ` the unit gradient of `θ`
    over the six neighbors of its cell (zero where they are equal), `w_h` 0.05.
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
| shade_internode_factor, shade_sla_factor, shade_pod_set_factor | 2.6, 1.4, 0.5 | - | Lopes et al. 1987 (30% light); Cossu et al. 2021 (decision 0037) |
| pod_set_t (min, opt, opt, max) | 7, 22, 25, 36 | °C | CROPGRO bean |
| seeds_per_pod, ovule_set | 5, 0.75 | -, - | "~" Kumar 2025 |
| Root types | tap, two basal whorls of 4, 10 hypocotyl-borne, laterals, fine laterals | see file | OpenSimRoot bean (values only; fine laterals and lateral spacing 0.5 cm / apical zone 4 cm from carioca.xml, decision 0037) |
| root_segment_length | 2 | cm | "~" (decision 0013) |
| shoot_gravitropism | 0.3 | - | "~" (decision 0013) |
| phototropism_weight, phototropism_max_per_day | 0.3, 10 | -, ° | "~" R1 spec §5; the daily bound does not bind at today's phyllochron (decision 0041) |
| PRIMORDIUM | 0.02 | - | "~" code constant (decision 0013) |
| root_dry_full, root_dry_floor | 0.3, 0.3 | - | "~" R2b spec; floor from Sharp 2004, Yamaguchi 2010 (decision 0043) |
| xerobranching_threshold | 0.1 | - | "~" Orman-Ligeza 2018 (barley) |
| root_proliferation_nitrate, root_proliferation_water | 1.0, 0.5 | - | "~" Drew 1975; Zhang 1999 and Linkohr 2002 (Arabidopsis, 1.5 to 4 times); water a guess |
| hydropatterning, hydrotropism_weight | 1 (on), 0.05 | -, - | Bao 2014; "~" Dietrich 2017 |

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
- Root length reaches 56 to 93 m at flowering in the pot (60 to 180 m published) and
  root:shoot 0.13 to 0.18 at seed fill (0.08 to 0.30), but fine roots hold only about
  8% of root length at flowering against 10 to 40% (Info), since a lateral cohort
  gets its fine roots only once it has finished (decision 0037).
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
- Drought from flowering raises root:shoot by 23 to 54%, as published, but cuts shoot
  mass 57 to 70% against about 26% (Info): once stomata follow what the roots can
  supply (decision 0037), drought closes them harder than the published pots show.
- Stems lean toward the light (R1c): next to a low wall a pot plant leans away from
  it and its leaves catch a few percent more light, while a lone plant in the open stays
  nearly upright. Next to a tall wall leaning gains no sky (decision 0041). Under an
  open sky a tilted branch's apex sees light from nearly straight up, so the rule adds a
  second upward pull on top of gravitropism: pot and plot branch internodes stand 27°
  from vertical against 34° without it, with tips 20 to 25% closer to the collar. Branch
  angles have not been checked against bean data. Leaves do
  not turn their blades toward the sun (no heliotropism). Roots bend by gravity, noise
  and, weakly, toward wetter cells (R2b).
- Laterals are lumped into cohorts; their fine roots are one more cohort each, in the
  cells next to the lateral, without geometry of their own (decision 0037).
- A cohort is one straight organ, so proliferation raises the roots it stands for along
  its whole path by the path's mean richness: a lateral that only crosses a patch
  proliferates in part, and outside the patch too (decision 0043). A nitrate patch the
  tap root runs through gains about 1.5 times the lateral length of the same cube
  without it by day 45 (1.3 to 1.7 over six seeds), at the low end of the Arabidopsis
  1.5 to 4; no legume number exists.
- The dry-soil rules act only once soil is dry: in the split pot the unwatered half
  falls to about 0.4 of available water by day 45, so wet:dry root length is about 1.5
  (1.3 to 1.6; 1.1 without the rules). Root growth reads relative available water, not
  water potential, until R3.
- In soil that starts uniform the plant still makes differences (it depletes the
  cells it roots in, and water carries nitrate to them), so the rules act a little:
  over six seeds of the watered 300 ppm pot, root mass and total mass stay within 2% of
  the rules off, root length is 5 to 8% higher (more of the root carbon in laterals)
  (decision 0043).
- Leaves senesce from the base during seed fill through nitrogen mining (nitrogen
  card). Since R1b the seeds mine first whatever the soil holds (decision 0040), but a
  fertilized pot's lower leaves still end above the drop threshold, so it keeps its
  leaves to maturity; an N-limited plot drops about 16 of 23 in seed fill.
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
- `ps_arch::tests::*`: profiles, pod-set temperature, sink feedback, deflection;
  `an_apex_turns_toward_the_light_within_the_daily_bound` (R1c).
- `tests/ninety_days.rs::a_plant_beside_a_wall_leans_away_and_gains_light` (R1c
  done-when, world seed 7, day 45): beside a 40 cm wall 6 cm east of the stem the main
  stem's tip ends 8.5 cm west of the collar and the leaves absorb 4% more PAR than with
  phototropism off; the same plant in the open ends within 2 cm of upright.
- `ps_arch::roots::tests::*` (R2b): proliferation follows nitrate above the mean and is
  1 in uniform soil, a cohort's factor is the share of its path in rich soil, the water
  gradient points to the wet side and is zero in uniform soil.
- `tests/root_plasticity.rs` (R2b, world seed 7, 40 days): a split pot roots more in
  its watered half with the rules than without; laterals in a nitrate patch on the tap
  root's path outgrow those in the same cube of plain soil by more than 1.2 times.
- Validation (Info): `split_pot_foraging` (m2/split_pot) and
  `nitrate_patch_proliferation` (m2/nitrate_patch against m2/nitrate_patch_control).

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
- Bao, Y., Aggarwal, P., Robbins, N. E., et al. (2014). Plant roots use a patterning
  mechanism to position lateral root branches toward available water. PNAS 111:
  9319-9324.
- Dietrich, D., Pang, L., Kobayashi, A., et al. (2017). Root hydrotropism is controlled
  via a cortex-specific growth mechanism. Nature Plants 3: 17057.
- Drew, M. C. (1975). Comparison of the effects of a localised supply of phosphate,
  nitrate, ammonium and potassium on the growth of the seminal root system, and the
  shoot, in barley. New Phytologist 75: 479-490.
- Linkohr, B. I., Williamson, L. C., Fitter, A. H., Leyser, H. M. O. (2002). Nitrate and
  phosphate availability and distribution have different effects on root system
  architecture of Arabidopsis. Plant Journal 29: 751-760.
- Orman-Ligeza, B., Morris, E. C., Parizot, B., et al. (2018). The xerobranching
  response represses lateral root formation when roots are not in contact with water.
  Current Biology 28: 3165-3173.
- Sharp, R. E., Poroyko, V., Hejlek, L. G., et al. (2004). Root growth maintenance during
  water deficits: physiology to functional genomics. Journal of Experimental Botany
  55: 2343-2351.
- Yamaguchi, M., Valliyodan, B., Zhang, J., et al. (2010). Regulation of growth
  response to water stress in the soybean primary root. Plant, Cell and Environment 33:
  223-235.
- Zhang, H., Jennings, A., Barlow, P. W., Forde, B. G. (1999). Dual pathways for
  regulation of root branching by nitrate. PNAS 96: 6529-6534.
