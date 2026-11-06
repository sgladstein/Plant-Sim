# Respiration

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Living tissue burns some of its sugar just to stay alive (maintenance), and building new
tissue costs extra sugar on top of what ends up in it (growth respiration). Warm nights
cost a plant more carbon than cool ones. Roots also leak some sugar into the soil
(exudates), which feeds the microbes around them.

## Equations
- Maintenance, per non-leaf organ per hour (§3.2 hourly step 8):
  `Rm = r20 · N · Q10^((T − 20)/10) / 24` (g C), with `N` the organ's nitrogen (g N) and
  `T` its temperature (air for the shoot, soil layer for roots). Paid from the organ's
  own reserve.
- Leaves: their maintenance is the day respiration `Rd` inside gas exchange, so they are
  not charged `Rm` too (decision 0009). At night a leaf's net income is `−Rd`.
- Growth: of `G` g C spent on growth, `G / cost` becomes structure and
  `G (1 − 1/cost)` is respired; with `cost = 1.25` that is 20% (§3.8). Secondary
  growth (thickening, decision 0036) pays the same cost. Wood holds no nitrogen, so it
  adds nothing to `Rm`: a thickened stem's upkeep is its primary tissue's.
- Root exudation (R1b, decision 0040), right after each root pays `Rm`:
  `X = min(e · Rm, max(0, R))`, so it never runs a reserve into debt; booked as the
  carbon ledger's `exudation` and the daily column `exudation_gc`.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| maintenance_rate_20c | 0.218 | g C g N⁻¹ day⁻¹ | Ryan 1991, as CLM's base rate 2.525e-6 g C g N⁻¹ s⁻¹ (decision 0011) |
| maintenance_q10 | 2.0 | 1 | "~" design plan §3.8 |
| construction_cost_leaf, _stem, _root, _fruit | 1.25 each | g C g C⁻¹ | "~" design plan §3.8; per organ class from decision 0032 (see carbon-allocation.md) |
| exudation_fraction | 0.4 | 1 | R1 numbers 0.3 to 0.5: legume exudates about 2.7% of assimilated ¹⁴C (Warembourg 2003), rhizodeposition 5 to 10% (Nguyen 2003), bean root respiration about 20% of daytime assimilation (Nielsen 2001); OpenSimRoot has 0.67. Wheat takes the bean value |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | organ nitrogen, temperature | g N, °C | organ graph, weather and soil |
| out | maintenance and growth respiration | g C | organ reserve, carbon ledger |
| out | root exudation | g C | carbon ledger, daily CSV |

## Encoded or emergent
Encoded: the rates. Emergent: heat waves draining reserves, big plants needing more light
to break even, starving shaded branches.

## Known limits
- Organ nitrogen is real state since WP24 (docs/models/nitrogen.md): mined organs
  respire less. The sown seed and the cotyledons pay no maintenance, and nothing does
  after maturity (decision 0020).
- Nodules pay fixation respiration, 7.1 g C per g N fixed, booked as `fixation` in the
  carbon ledger.
- Leaf `Rd` scales with leaf capacity (aging), so old leaves respire less too.
- Exudation is a fixed share of root maintenance: it does not rise with the reserve,
  the soil's nutrients or drought, and the exuded carbon is not tracked in the soil.

## Validation
`ps_physio::tests::maintenance_and_growth_respiration`; carbon ledger closure in
`ps_physio::alloc_tests`; `roots_exude_a_share_of_maintenance` (R1b).

## References
- Amthor, J. S. (2000). The McCree–de Wit–Penning de Vries–Thornley respiration
  paradigms: 30 years later. Annals of Botany 86.
- Nguyen, C. (2003). Rhizodeposition of organic C by plants: mechanisms and controls.
  Agronomie 23.
- Ryan, M. G. (1991). Effects of climate change on plant respiration. Ecological
  Applications 1(2).
