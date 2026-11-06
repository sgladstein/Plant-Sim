# Organ growth over time and organ size

## Purpose
Every leaf, stem segment, root, flower and pod is born at a moment and then expands
along a smooth S-shaped curve, fastest in its middle, until it reaches full size. That
curve is what lets the garden grow smoothly at any speed and replay exactly. How big
"full size" ends up being depends on whether the plant could pay for the growth: an
organ that went hungry while growing ends up smaller and never catches up. Size on
screen (leaf area, stem length and thickness) comes from the organ's dry mass.

Stems and roots of a bean also grow wider after they stop growing longer (secondary
growth, R1a, decision 0036): a stem widens toward the cross-section its leaves need (the
pipe model), and a root widens with its age, most near its base. The widening is extra
mass at a fixed length, so a stem keeps its height and gets thicker, and stems and
roots taper from base to tip. Grasses have no cambium and never widen.

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
- Internode and root length from the primary mass, `L = SL · (W − W_s)` (specific
  length per organ, set at initiation; shade raises internode SL), radius from the whole
  mass, `r = sqrt(W / (ρ π L))`. `W_s` is the secondary part of the structural mass
  (R1a); with `W_s = 0` this is the one-mass rule bit for bit. The primary radius
  `r_p = r · sqrt((W − W_s) / W)` carries the phloem (carbon-allocation card).
- Seeds, flowers, pods: prolate spheroid of density `ρ` and aspect `a = L/d`:
  `d = (6 W / (π ρ a))^(1/3)`, `L = a·d`.

Structural mass `W` never decreases; at death it is frozen and the caller books 30%
resorption to the parent and the rest as litter (§3.9 step 6).

**Secondary growth** (R1a, decision 0036; `ps_arch::thicken`). Potentials per hour,
set once a day for the coming day at daily step 5b (after root elongation; decision
0037, approved by the owner) and built each hour by allocation at priority 0.4 from
the organ's reserve (carbon-allocation card):

- Stems (pipe model, Shinozaki et al. 1964): daily after initiation, each living
  internode's `A_max ← max(A_max, ΣA)`, the leaf area above it (pipes stay after their
  leaves die), and target `X* = h_L · A_max + h_F · ΣF` (`ΣF` fruit dry mass above).
  Once the internode has finished expanding,
  `ΔW_s = min(X* − π r², g_max · 2π r · 24 h) / 24 · L · ρ_stem · f_T · s` per hour (at
  least 0): a day's widening toward the target, spread over the day.
- Roots (OpenSimRoot bean): `ΔW_s = 2π r L · ½ ḋ(age) · k(x) · 1 h · ρ_root · f_T · s`,
  where `ḋ` is the species' diameter growth rate by the segment's own age, `k(x)` the
  root type's factor by the segment's distance `x` from its root's base (mid-segment),
  and `L` includes a lateral cohort's multiplicity. The rate `½ ḋ(age) · k(x)` is read
  at the daily step for each living segment (`Plant::set_wood_rates`).
- `f_T` is the expansion temperature multiplier at the mean air temperature of the day
  just ended (the soil layers' daily mean follows it), `s` that day's mean expansion
  stress factor (water times nitrogen) times the share of the expanding shoot's carbon
  demand that was met that day (leaves and internodes, `Plant::carbon_met`, R2a,
  decision 0037): wood is the first sink to yield when carbon is short. Nothing
  thickens before germination, after maturity, in dead organs, or in a species with
  `secondary_growth` 0.
- R1a computed these potentials every hour. Thickness changes too slowly to show
  within a day, so R2a computes them once a day (decision 0037); the hourly build
  stays, since one draw a day from the midnight reserve starved wood.

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
| `secondary_growth` | bean 1, wheat 0 | - | botany: grasses have no cambium |
| `pipe_area_per_leaf_area` `h_L` | 3e-4 | m2/m2 | ~ whole-stem Huber value 1.5e-4 to 3.1e-4 from field stem diameters and LAI (R1 numbers) |
| `pipe_area_per_fruit_mass` `h_F` | 0 | m2/g | ~ off |
| `stem_radial_rate_max` `g_max` | 0.1 | mm/day | ~ cap; bean needs 0.02 to 0.05 on average |
| `root_secondary_rate` `ḋ(age)` | 0 at day 0 to 0.003 cm/day by day 29 (adds 0.1 cm in 43 days) | cm/day | OpenSimRoot bean growthrates.xml |
| root `secondary_factor` `k(x)` | tap 4 to 20 cm, 2 from 40 cm; basal 0.7 to 20 cm, 0.4 from 40 cm; hypocotyl-borne 0.6 to 0.2 at 50 cm; laterals 0.1 | - | OpenSimRoot bean growthrates.xml |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | thermal age, expansion clock | Cd | thermal time (WP7) |
| in | realized growth | g DM | allocation (WP7) |
| out | potential growth | g DM | allocation demand |
| out | length, radius, area | m, m, m2 | light, renderer, CSV |
| out | secondary mass | g DM | organs.csv `secondary_g`; daily `hypocotyl_diameter_mm`, `tap_root_diameter_mm` |

## Encoded or emergent
Encoded: the curve shape and allometry, the pipe target and the root rate tables.
Emergent: final organ sizes, through unmet demand; stem and root diameters and taper,
which carbon decides.

## Known limits
- The hypocotyl's primary tissue is thin (a watery seedling hypocotyl), so it draws
  less carbon than the epicotyl and lags it in diameter until late in the season; both
  reach the pipe target by maturity.
- Secondary tissue carries no nitrogen requirement and pays no maintenance (both are
  per nitrogen); wood does hold some nitrogen.
- No secondary phloem: transport keeps the primary radius (R1a rule).
- One root rate table per species (OpenSimRoot's basal whorls start 2 days later).
- The thickening potential is set once a day: a segment's age, its rate, the
  temperature factor and the stress factor are each read once, from the day before, so
  the factor at the day's mean temperature stands in for the mean of the hourly
  factors and wood answers to stress a day late (decision 0037).
- The pipe target follows leaf area and fruit only: wind sway and self-weight do not
  thicken a stem, and there is no bark or heartwood.
- Leaf blades are flat areas; petiole length is not modeled separately.

## Validation
- `ps_graph::curve::tests::increments_integrate_to_potential`: 20 parameter sets, 1e-6 (§8.1).
- `ps_graph::curve::tests::fraction_is_monotone`, `rate_matches_numeric_derivative`, `rate_peaks_at_t_max`.
- `ps_graph::tests::structural_mass_never_shrinks`: random add, grow, kill sequences (WP3 done-when).
- `ps_graph::tests::unmet_demand_shrinks_effective_potential`.
- `ps_graph::allometry::tests::*`: volume and area identities;
  `secondary_mass_widens_at_fixed_length` (no secondary mass is the old rule bit for
  bit). `ps_graph::tests::secondary_growth_keeps_length_and_widens`.
- `ps-arch/tests/thickening.rs`: a 56-day bean's hypocotyl at 4.5 to 10 mm, the stem
  tapers, pipe targets never fall, primary radii and lengths unchanged by thickening,
  the tap root tapers; a wheat plant has no secondary mass.
- Validation (Info): `basal_stem_diameter`, `tap_root_diameter`, `stem_share_peak`.

## References
- Yin, X., Goudriaan, J., Lantinga, E. A., Vos, J., Spiertz, H. J. (2003). A flexible
  sigmoid function of determinate growth. Annals of Botany 91, 361–371.
- Godin, C., Caraglio, Y. (1998). A multiscale model of plant topological structures (MTG). J. Theor. Biol. 191, 1–46.
- Shinozaki, K., Yoda, K., Hozumi, K., Kira, T. (1964). A quantitative analysis of plant
  form: the pipe model theory. Japanese Journal of Ecology 14, 97–105.
- Postma, J. A., Kuppe, C., et al. (2017). OpenSimRoot: widening the scope and
  application of root architectural models. New Phytologist 215, 1274–1286 (bean
  parameter set growthrates.xml).
- Strock, C. F., De La Riva, L. M., Lynch, J. P. (2018). Reduction in root secondary
  growth as a strategy for phosphorus acquisition. Plant Physiology 176, 691–703.
