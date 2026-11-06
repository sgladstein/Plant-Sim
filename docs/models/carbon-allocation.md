# Carbon allocation

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Leaves make sugar; every other part of the plant has to get it from them. Sugar flows
from where it is plentiful to where it is scarce, and each growing
part takes what it can. A part that keeps running short ends up smaller, and a part that
cannot even pay its upkeep for three days dies and drops. Shaded lower branches starving
and falling off is not scripted; it happens this way. Stems and main roots also keep a
starch store: they fill it when sugar is plentiful and give it back when it runs short,
and once the seeds start filling it empties into them.

## Equations
Per plant, hourly steps 8 to 12 of §3.2, crate `ps-physio::allocation`. Organ `i` has
reserve `R` (g C), structure `W` (g DM) and concentration `C = R / max(W_p, 1e-6)`,
with `W_p = W − W_s` its primary structure: secondary (wood) mass `W_s` from R1a
thickening holds no phloem sap (decision 0036), so it neither dilutes the sugar
concentration nor raises the reserve floor.

1. Maintenance (step 8): non-leaf organs pay `Rm` (respiration card) from `R`. Roots
   then exude `e · Rm` from what positive reserve they have left, never into debt
   (`e` = `exudation_fraction`, 0.4; R1b, decision 0040); it leaves the plant as the
   ledger's `exudation`.
2. Income (step 9): each leaf adds its net assimilation (negative at night).
   Starvation signal: an organ whose `R < 0` now counts one more starving hour; any
   other organ resets to zero.
3. Transport (step 10), one backward-Euler diffusion step per hour on the tree of
   living parent-child edges (decision 0011). Conductance `k = K · π r_p² / l` of the
   segment the edge runs through, with `r_p = r √(W_p / W)` its primary radius
   (thickening widens a segment but not its phloem; decision 0036) (the child if it is a stem or root segment, else the
   parent; at least 0.5 mm radius and 5 mm length). New concentrations solve
   `W_p,i C_i' + Σ_j k_ij (C_i' − C_j') = max(R_i, 0)` for every organ, exactly, by one
   leaf-to-root elimination and one root-to-leaf substitution (the Thomas algorithm on
   a tree, O(n)). Carbon moves as edge fluxes `k (C_p' − C_c')`. The step is
   unconditionally stable and monotone (no concentration leaves its neighbors' range,
   so transport never takes a reserve below zero), carbon reaches past the first edge
   in one hour, and a debt (negative reserve) stays with its organ.
4. Thermal time (step 11): organ thermal age advances (thermal-time card), and so does
   the organ's expansion clock, by thermal time times the expansion temperature
   multiplier (decision 0011), times, for a growing leaf, internode, tendril or
   cotyledon, its turgor from the last hour's water potential (R3b, decision 0044;
   plant-hydraulics card: stored growth), and for a root tip its soil's dryness
   (R2b).
5. Growth (step 12). First, a starch store short of sugar releases (R1b, decision
   0040; item 8 below). Then, for organs still expanding: potential `ΔW = potential ×
   (y(a + da) − y(a))` from the beta curve on the expansion clock `a`, times the
   plant's nitrogen factor for leaves, internodes, tendrils and cotyledons. Cold and
   low turgor slow the clock, so they delay expansion instead of cutting the organ
   short. Demand
   `D = ΔW · 0.45 · 1.25` g C. Available `A = max(0, R − 0.05 W_p)`. Spent
   `S = min(D, p · A)` with priority `p` (seeds in pods 1.0, pods 0.9, roots
   0.6 + 0.4 (1 − min(FTSW, f_ψ)), everything else 0.7, with `f_ψ` the stomatal
   factor of the gas-exchange card, R3b). Unmet `D − S` is booked; `S / 1.25` becomes
   structure and the rest is growth respiration. A growing root draws on its own
   reserve and on its parent root's (or the seed's), `S = min(D, p (A + A_parent))`, own
   first (tip feeding, R2a, decision 0037): a queued root segment starts as a tiny
   primordium, and phloem unloads at the tip from the root behind it.
   Then secondary growth (R1a, decision 0036; organ-growth card), in the same hour from
   what is left: each stem or root organ with a thickening potential `ΔW_s` spends
   `S_s = min(ΔW_s · 0.45 · cc, 0.4 · max(0, R − 0.05 W_p))` at priority 0.4; `S_s / cc`
   becomes secondary structure (wider, same length) and the rest growth respiration.
   Unmet secondary demand is not booked: wood is a sink that takes only surplus.
   Since R2a the potential is set once a day for the coming day (daily step 5b,
   decision 0037), scaled by the day's share of the expanding shoot's carbon demand
   that was met (leaves and internodes, `Plant::carbon_met`), so wood is the first sink
   to yield when carbon is short.
   Last, the store refills (item 8).
   From physiological maturity nothing in step 12 runs: no organ grows, thickens,
   releases or refills, so late pods keep the seed mass they have (R1d, decision 0042;
   CROPGRO stops seed growth at R7).
6. No catch-up: demand always follows the species potential, and a shortfall is lost, so
   a fully expanded organ ends at `(1 − unmet / potential demand) × potential`.
7. Starvation (daily step 4): an organ with at least `24 × starvation_window` starving
   hours dies; a stem segment takes its whole subtree with it (an axis dies with its
   basal internode). 30% of each dead organ's primary (not woody) structural carbon
   moves to the nearest
   living ancestor's reserve, which also pays the dead organ's debt (a negative reserve
   is carbon already respired, so litter is never negative; decision 0011). The rest of
   its structure and any positive reserve become litter. With no living ancestor
   everything goes to litter. Every death goes through `allocation::shed`, which always
   kills the whole subtree (WP8): a starving organ, a senescing leaf, a closed flower or
   a dropped cotyledon. When the architecture sheds an organ (age, shade, frost,
   flower closing, cotyledon drop) its positive reserve, which is mobile, first moves
   to its parent; only structure is split into resorbed and litter (decision 0013).
   Starvation is not checked after maturity, when tissue is dry and no longer
   respires (no organ pays maintenance after maturity; decision 0020).
   A cut (pruning; harvest later) goes through `allocation::cut` instead: nothing is
   resorbed, and each cut organ's structure and reserve (a debt included) leave the
   plant as `removed` (WP10, WP9 review).
8. Starch store (R1b, decision 0040). Stem segments and main (order 0) root segments
   hold starch `S` (g C), apart from the reserve: it does not move and does not count
   in `C`. Capacity `S_max = s_max W c_f` (`s_max` 0.25 for bean stems and roots, 0.28
   for wheat stems, 0 for wheat roots). In step 12, before the organ grows: if
   `C < C_off` (0.06), release `min(k_r S, (C_off − C) W_p)` into `R` (`k_r` 0.02 h⁻¹).
   After its growth, once it has finished expanding and if `C > C_on` (0.15): store
   `min(k_s (S_max − S), 0.2 · max(0, R − 0.05 W_p))` (`k_s` 0.05 h⁻¹, priority 0.2).
   From seed fill (R5) the store stops refilling and releases at least `k_f S` an hour
   whatever `C` (`k_f` 0.08 d⁻¹ / 24). At death the store joins the reserve, so it is
   booked like reserve. Starch counts in the stock and, as `S / c_f`, in dry mass.
9. Ledger: `stock = Σ_living (R + S + 0.45 W)` must equal
   `start + added + income − maintenance − growth respiration − litter − removed −
   fixation − exudation`; debug builds assert it every hour at 1e-9 relative.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| carbon_fraction | 0.45 | g C g DM⁻¹ | design plan §6.1 |
| construction_cost_leaf, _stem, _root, _fruit | 1.25 each | g C g C⁻¹ | "~" design plan §3.8. Per organ class from schema 10 (decision 0032); CROPGRO's composition costs (leaf 1.39, stem 1.29, root 1.25, seed 1.36) are held back until the transport and priority revisit, because they weaken the carbon-limited shade internode response below its M0 gate |
| reserve_floor | 0.05 | g C g DM⁻¹ | "~" design plan §3.9 step 4 |
| resorption_fraction | 0.3 | 1 | "~" design plan §3.9 step 6 |
| starvation_window | 3 | day | "~" design plan §3.9, §3.16 |
| phloem_conductance K | 500 | g DM m⁻¹ h⁻¹ | "~" decision 0009 |
| priorities | 1.0, 0.9, 0.7, 0.6 + 0.4 (1 − FTSW) | 1 | design plan §3.9 step 2 |
| secondary priority | 0.4 | 1 | design plan §3.9 step 2; decision 0036 |
| store refill priority | 0.2 | 1 | design plan §3.9 step 2 (reserve refill); decision 0040 |
| storage_capacity, root_storage_capacity | 0.25, 0.25 bean; 0.28, 0 wheat | g g DM⁻¹ | "~" bean (0.15 to 0.30); wheat Davidson and Chevalier 1992; R1 numbers |
| storage_on, storage_off | 0.15, 0.06 | g C g DM⁻¹ | "~" decision 0040 (spec 0.12, 0.06) |
| storage_refill_rate, storage_release_rate | 0.05, 0.02 | h⁻¹ | "~" R1 spec §4.4 |
| storage_fill_release_rate | 0.08 | d⁻¹ | "~" Waters et al. 1980; decision 0040 |
| exudation_fraction | 0.4 | 1 | R1 numbers (0.3 to 0.5): Warembourg 2003, Nguyen 2003, Nielsen 2001 |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | leaf net assimilation | g C h⁻¹ | gas exchange |
| in | organ temperatures, FTSW, nitrogen factor | °C, 1, 1 | weather, soil, nitrogen |
| in | organ turgor (expansion clock), stomatal factor (root priority) | 1, 1 | plant hydraulics |
| out | organ reserves, starch stores, structural growth, unmet demand, starving hours, deaths | g C, g DM | organ graph, architecture, lenses |
| out | exudation | g C | carbon ledger, daily CSV (soil carbon is not tracked) |

## Encoded or emergent
Encoded: the priority numbers (the root term is the one encoded piece of functional
balance and is labeled as such in the UI). Emergent: source-to-sink gradients, starving
shaded leaves and branches, drought shifting growth toward roots, fruit load starving
late leaves, smaller leaves after a dull week.

## Known limits
- Transport is diffusion of sugar concentration, not pressure-driven mass flow, so it
  fades with distance along an axis; how far carbon reaches in an hour is set by the
  conductance `K` alone, a WP12 tuning parameter (at K = 500, `k/W` is about 0.07 to
  0.35 per hour for a 2 mm, 4.5 cm internode against 0.1 to 0.5 g organs). If tuning cannot make it plausible, the
  three-pool fallback of §3.9 is pre-decided.
- Mature leaves import carbon like any other organ when their reserve is lower than
  their neighbors' (real mature leaves mostly do not), which makes shaded leaves
  starve later. Revisit in WP12 if shaded leaves do not drop.
- Priority is the share of an organ's own available reserve it spends per hour, so at
  an hourly step it is a weak lever (decision 0009). If fruit does not starve late
  leaves in WP12, make priority shape the transport to the receiving organ.
- Secondary thickening is a sink at priority 0.4 (decision 0036), scaled by the
  previous day's shoot carbon (decision 0037). It keeps building wood through seed
  fill when the plant has sugar to spare, so a pot bean's stem mass hardly falls in
  seed fill (−1% against −10 to −35% in the field, where petioles count with the stem;
  decision 0040).
- A lone pot plant still has more sugar than its seeds can take in seed fill (about
  41% of its potential gross assimilation is lost to the sink feedback in model 0.23.0;
  the target is at most 35%).
- Exuded carbon leaves the model: there is no soil carbon pool or rhizosphere.
- Growth stops at once at maturity (decision 0042): a pod still filling is cut short,
  and the reserves left in the plant stay there. Under drought and heat the mean seed
  grows larger rather than smaller, likely because fewer pods share those reserves.
- Root tip feeding lets a growing root spend its parent's reserve at its own priority,
  so a fast-growing root system can draw on the reserve of the root behind it within
  the hour; there is no unloading limit at the tip (decision 0037).
- No functional balance on light: shaded plants keep the root share of a sunlit one
  (shade_root_mass passes at about −30%, but the root share of total mass rises a
  little in shade, where Poorter et al. 2012 find it falls). Scaling root growth by the
  shoot's carbon was tried in R2a and cut nitrogen and water uptake more than it
  helped the shoot (decision 0037).
- Secondary mass is phloem-free: thickening raises neither an organ's conductance nor
  its capacity to hold sugar. Real secondary phloem grows with the cambium, so old
  stems carry more than the model lets them.
- Leaf lifespan and shade senescence are daily step 4 too, but arrive with the
  architecture module (WP8).

## Validation
`ps_physio::alloc_tests`: `seed_reserve_feeds_growth_and_ledger_closes`,
`starving_leaf_dies_after_the_window` (dies at the third daily step, hour 72),
`unmet_demand_shrinks_final_size` (three days without carbon end 10% or more smaller,
and the final size matches the reduced potential within 5%),
`cold_delays_expansion_without_loss`, `starved_debt_is_charged_to_the_ancestor`,
`inputs_must_cover_every_organ`, `implicit_transport_crosses_a_chain_in_one_hour`, and the
property tests `transport_conserves_and_clamps` and
`implicit_transport_is_conservative_and_monotone` (random trees),
`secondary_growth_is_a_low_priority_sink` (R1a),
`root_tip_draws_on_its_parent` and `shedding_does_not_resorb_wood` (R2a),
`store_fills_holds_and_releases` (refill and release conserve carbon to 1e-12 per hour,
the store stays within 0 and `S_max`, release only below `C_off`),
`roots_exude_a_share_of_maintenance` and `filling_empties_the_store` (R1b),
`nothing_grows_after_maturity` (R1d; with `ninety_days`, which checks that seed mass
does not change after maturity);
`ps_graph::tests::store_moves_carbon_and_returns_it_at_death`. Validation report:
`stem_share_peak`, `stem_decline_fill`, `feedback_loss_fill`, `root_shoot_seed_fill`,
`mean_seed_mass`, `harvest_index_tops`.

## References
- Allen, M. T., Prusinkiewicz, P. and DeJong, T. M. (2005). Using L-systems for modeling
  source–sink interactions, architecture and physiology of growing trees: the L-PEACH
  model. New Phytologist 166.
- Thornley, J. H. M. (1972). A balanced quantitative model for root:shoot ratios in
  vegetative plants. Annals of Botany 36.
