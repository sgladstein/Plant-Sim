# Carbon allocation

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Leaves make sugar; every other part of the plant has to get it from them. Sugar flows
from where it is plentiful to where it is scarce, and each growing
part takes what it can. A part that keeps running short ends up smaller, and a part that
cannot even pay its upkeep for three days dies and drops. Shaded lower branches starving
and falling off is not scripted; it happens this way.

## Equations
Per plant, hourly steps 8 to 12 of §3.2, crate `ps-physio::allocation`. Organ `i` has
reserve `R` (g C), structure `W` (g DM) and concentration `C = R / max(W, 1e-6)`.

1. Maintenance (step 8): non-leaf organs pay `Rm` (respiration card) from `R`.
2. Income (step 9): each leaf adds its net assimilation (negative at night).
   Starvation signal: an organ whose `R < 0` now counts one more starving hour; any
   other organ resets to zero.
3. Transport (step 10), one backward-Euler diffusion step per hour on the tree of
   living parent-child edges (decision 0011). Conductance `k = K · π r² / l` of the
   segment the edge runs through (the child if it is a stem or root segment, else the
   parent; at least 0.5 mm radius and 5 mm length). New concentrations solve
   `W_i C_i' + Σ_j k_ij (C_i' − C_j') = max(R_i, 0)` for every organ, exactly, by one
   leaf-to-root elimination and one root-to-leaf substitution (the Thomas algorithm on
   a tree, O(n)). Carbon moves as edge fluxes `k (C_p' − C_c')`. The step is
   unconditionally stable and monotone (no concentration leaves its neighbors' range,
   so transport never takes a reserve below zero), carbon reaches past the first edge
   in one hour, and a debt (negative reserve) stays with its organ.
4. Thermal time (step 11): organ thermal age advances (thermal-time card), and so does
   the organ's expansion clock, by thermal time times the expansion temperature
   multiplier (decision 0011).
5. Growth (step 12), for organs still expanding: potential `ΔW = potential ×
   (y(a + da) − y(a))` from the beta curve on the expansion clock `a`, times the
   water stress factor for leaves, internodes, tendrils and cotyledons. Cold slows the
   clock, so it delays expansion instead of cutting the organ short. Demand
   `D = ΔW · 0.45 · 1.25` g C. Available `A = max(0, R − 0.05 W)`. Spent
   `S = min(D, p · A)` with priority `p` (seeds in pods 1.0, pods 0.9, roots
   0.6 + 0.4 (1 − FTSW), everything else 0.7). Unmet `D − S` is booked; `S / 1.25` becomes
   structure and the rest is growth respiration.
6. No catch-up: demand always follows the species potential, and a shortfall is lost, so
   a fully expanded organ ends at `(1 − unmet / potential demand) × potential`.
7. Starvation (daily step 4): an organ with at least `24 × starvation_window` starving
   hours dies; a stem segment takes its whole subtree with it (an axis dies with its
   basal internode). 30% of each dead organ's structural carbon moves to the nearest
   living ancestor's reserve, which also pays the dead organ's debt (a negative reserve
   is carbon already respired, so litter is never negative; decision 0011). The rest of
   its structure and any positive reserve become litter. With no living ancestor
   everything goes to litter.
8. Ledger: `stock = Σ_living (R + 0.45 W)` must equal
   `start + added + income − maintenance − growth respiration − litter`; debug builds
   assert it every hour at 1e-9 relative.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| carbon_fraction | 0.45 | g C g DM⁻¹ | design plan §6.1 |
| construction_cost | 1.25 | g C g C⁻¹ | "~" design plan §3.8 |
| reserve_floor | 0.05 | g C g DM⁻¹ | "~" design plan §3.9 step 4 |
| resorption_fraction | 0.3 | 1 | "~" design plan §3.9 step 6 |
| starvation_window | 3 | day | "~" design plan §3.9, §3.16 |
| phloem_conductance K | 500 | g DM m⁻¹ h⁻¹ | "~" decision 0009 |
| priorities | 1.0, 0.9, 0.7, 0.6 + 0.4 (1 − FTSW) | 1 | design plan §3.9 step 2 |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | leaf net assimilation | g C h⁻¹ | gas exchange |
| in | organ temperatures, FTSW, stress | °C, 1, 1 | weather, soil |
| out | organ reserves, structural growth, unmet demand, starving hours, deaths | g C, g DM | organ graph, architecture, lenses |

## Encoded or emergent
Encoded: the priority numbers (the root term is the one encoded piece of functional
balance and is labeled as such in the UI). Emergent: source-to-sink gradients, starving
shaded leaves and branches, drought shifting growth toward roots, fruit load starving
late leaves, smaller leaves after a dull week.

## Known limits
- Transport is diffusion of sugar concentration, not pressure-driven mass flow, so it
  fades with distance along an axis; how far carbon reaches in an hour is set by the
  conductance `K`, a WP12 tuning parameter. If tuning cannot make it plausible, the
  three-pool fallback of §3.9 is pre-decided.
- Mature leaves import carbon like any other organ when their reserve is lower than
  their neighbors' (real mature leaves mostly do not), which makes shaded leaves
  starve later. Revisit in WP12 if shaded leaves do not drop.
- Priority is the share of an organ's own available reserve it spends per hour, so at
  an hourly step it is a weak lever (decision 0009). If fruit does not starve late
  leaves in WP12, make priority shape the transport to the receiving organ.
- Secondary thickening and reserve refill sinks (§3.9 priorities 0.4 and 0.2) are not
  separate sinks yet (WP8).
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
`implicit_transport_is_conservative_and_monotone` (random trees).

## References
- Allen, M. T., Prusinkiewicz, P. and DeJong, T. M. (2005). Using L-systems for modeling
  source–sink interactions, architecture and physiology of growing trees: the L-PEACH
  model. New Phytologist 166.
- Thornley, J. H. M. (1972). A balanced quantitative model for root:shoot ratios in
  vegetative plants. Annals of Botany 36.
