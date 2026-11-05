# Organ growth over time and organ size

## Purpose
Every leaf, stem segment, root, flower and pod is born at a moment and then expands
along a smooth S-shaped curve, fastest in its middle, until it reaches full size. That
curve is what lets the garden grow smoothly at any speed and replay exactly. How big
"full size" ends up being depends on whether the plant could pay for the growth: an
organ that went hungry while growing ends up smaller and never catches up. Size on
screen (leaf area, stem length and thickness) comes from the organ's dry mass.

## Equations
**Expansion curve** (Yin et al. 2003, design plan §3.13 and §11.4). With the organ's
expansion clock `t` (°C·day: thermal time times the expansion temperature multiplier,
kept per organ as `expansion_age`; decision 0011), duration `te` and time of maximum rate `tm` (`0 < tm < te`):

- `y(t) = (1 + (te − t)/(te − tm)) · (t/te)^(te/(te − tm))` for `0 ≤ t ≤ te`; `y = 0` before, `1` after.
- `dy/dt = (2te − tm)(te − t) / (te (te − tm)²) · (t/te)^(tm/(te − tm))`.
- Potential growth over a step: `ΔW_pot = W_pot,eff · (y(t + Δt) − y(t))`.

**No catch-up** (§3.9 step 5): `W_pot,eff = W_pot · (1 − U / D)`, where `U` is the organ's
cumulative unmet carbon demand and `D` its cumulative potential demand.

**Allometry.**
- Leaf area `A = SLA · W` (SLA per organ, set at initiation; shade raises it).
- Internode and root length `L = SL · W` (specific length per organ, set at initiation;
  shade raises internode SL), radius `r = sqrt(W / (ρ π L))`.
- Seeds, flowers, pods: prolate spheroid of density `ρ` and aspect `a = L/d`:
  `d = (6 W / (π ρ a))^(1/3)`, `L = a·d`.

Structural mass `W` never decreases; at death it is frozen and the caller books 30%
resorption to the parent and the rest as litter (§3.9 step 6).

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Curve duration `te`, peak `tm`, potential `W_pot` | per organ type | Cd, Cd, g DM | species file (WP4) |
| Specific leaf area | 200 to 300 cm²/g (0.02 to 0.03 m²/g) | m2/g | §3.16, §8.2 |
| Specific root length | ~70 | m/g | §3.16 |
| Stem tissue density (placeholder) | 1.5e5 | g/m3 | ~ |
| Root tissue density (placeholder) | 1.0e5 | g/m3 | ~ |
| Bulk organ density (placeholder) | 4.0e5 | g/m3 | ~ |
| Pod aspect, seed aspect (placeholder) | 8, 1.5 | - | ~ |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | thermal age, expansion clock | Cd | thermal time (WP7) |
| in | realized growth | g DM | allocation (WP7) |
| out | potential growth | g DM | allocation demand |
| out | length, radius, area | m, m, m2 | light, renderer, CSV |

## Encoded or emergent
Encoded: the curve shape and allometry. Emergent: final organ sizes, through unmet demand.

## Known limits
- Herb stems keep a fixed length-to-mass ratio from initiation; secondary thickening
  beyond that is booked by ps-arch (WP8).
- Leaf blades are flat areas; petiole length is not modeled separately.

## Validation
- `ps_graph::curve::tests::increments_integrate_to_potential`: 20 parameter sets, 1e-6 (§8.1).
- `ps_graph::curve::tests::fraction_is_monotone`, `rate_matches_numeric_derivative`, `rate_peaks_at_t_max`.
- `ps_graph::tests::structural_mass_never_shrinks`: random add, grow, kill sequences (WP3 done-when).
- `ps_graph::tests::unmet_demand_shrinks_effective_potential`.
- `ps_graph::allometry::tests::*`: volume and area identities.

## References
- Yin, X., Goudriaan, J., Lantinga, E. A., Vos, J., Spiertz, H. J. (2003). A flexible
  sigmoid function of determinate growth. Annals of Botany 91, 361–371.
- Godin, C., Caraglio, Y. (1998). A multiscale model of plant topological structures (MTG). J. Theor. Biol. 191, 1–46.
