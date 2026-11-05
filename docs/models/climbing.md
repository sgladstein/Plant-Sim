# Climbing (vines)

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
A pole bean is a bush bean whose stem never stops to flower at its tip. Its growing tip
slowly circles, about once every two hours, and when it brushes a stake or a string it
wraps around it and climbs, always turning anticlockwise seen from above. Beans have no
tendrils: the stem itself does the twining. Thin supports are easy to catch; a post
thicker than a hand is not. Without anything to climb, the vine holds itself up for a
short while, then falls over and sprawls in a heap, and pods lying on damp soil rot.

## Equations
Code: `ps_arch::vine`, with the hooks in `ps_arch::shoot` and `ps_arch::geometry`.
Decision 0021.

1. **Vine.** A plant is a vine when its `habit` trait is 0.5 or more.
2. **Indeterminate main stem.** A node every phyllochron, as for the herb, until
   `τ_since_flower ≥ vegetative_after_flowering` or `nodes = max_main_nodes`; no
   terminal raceme. Buds above rank `nodes_before_terminal − 2` become racemes at first
   flower, and each later node gets a raceme in its axil.
3. **Internodes.** Main-stem internode `k ≥ 2`: `ℓ = ℓ_pot (g₁ + (g_N − g₁) min(k − 2, R)/R)`
   with `R = internode_ramp_nodes`.
4. **Capture**, daily before new nodes, for each searching axis with tip `p`: over the
   supports whose height span holds `p_z`, the gap `g = |p_xy − c_xy| − d/2`; the nearest
   with `g ≤ r_sweep` is caught with probability `P(d) = 1` for `d ≤ d_sure`,
   `(d_max − d)/(d_max − d_sure)` up to `d_max`, 0 beyond (stream `Twining`).
5. **Coil.** A twining internode of length `s` from base `b`, around axis `c` at radius
   `R = d/2 + r_stem` (`r_stem = √(1/(π · SSL · ρ_stem))`) and inclination `η`:
   `φ = atan2(b_y − c_y, b_x − c_x) + s cos η / R`, tip `= (c_x + R cos φ, c_y + R sin φ, b_z + s sin η)`.
   `φ` increases: anticlockwise seen from above, a right-handed helix.
6. **Topping.** When a twining tip passes the support's top, new internodes hang
   `0.4 rad` below level.
7. **Lodging.** A free axis sums the potential length of its new internodes; past
   `self_support_length` it falls: each internode from the axis' base to its apex turns
   to one random azimuth at elevation `−asin(clamp((z − h_lodge)/ℓ, 0, 1))`, walking up
   from the base height `z`. A falling main stem takes every branch with it, and new
   branches start fallen.
8. **Lodged growth.** A new internode turns by up to ±0.8 rad from the last one (stream
   `Tropism`), or back toward the collar (±0.4 rad) once the tip is more than
   `heap_radius` from it, dipping to the lodge height as in 7.
9. **Pod rot**, daily step 4, vines only: a pod whose foot is no higher than its length
   is shed with probability `pod_rot_rate` (stream `PodSet`).

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| habit (pole bean) | 1 | 0 determinate / 1 indeterminate | botany: *Fin_* is the pole type (González 2016) |
| max_main_nodes | 30 | - | ~ research notes: cap 25 to 35 |
| vegetative_after_flowering | 260 | Cd | CROPGRO BNGRO048.ECO FL-VS 13 PTD × 20 Cd |
| internode_ramp_nodes | 8 | - | ~ CROPGRO YVSHT |
| internode_length_potential (pole bean) | 12 | cm | ~ tuned to the height target |
| internode_last_fraction (pole bean) | 1.6 | - | ~ tuned to the height target |
| specific_stem_length (pole bean) | 120 | cm/g | ~ a 2.6 mm twining stem |
| flowering, maturity (pole bean) | 575, 1150 | Cd | CROPGRO CUL EM-FL 34/26 × 440; ~ |
| circumnutation_radius | 6 | cm | ~ 3 to 10 (Sothern 1998) |
| support_diameter_sure, support_diameter_max | 5, 20 | cm | Darwin 1875 |
| helix_inclination | 60 | deg | Rutjens 2025 (35 to 75) |
| self_support_length | 40 | cm | ~ |
| heap_radius | 15 | cm | ~ |
| lodge_height | 2 | cm | ~ |
| pod_rot_rate | 0.02 | 1/day | ~ set to the unstaked yield band |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | supports (foot, height, diameter) | m | scenario |
| in | tip positions (yesterday's layout) | m | geometry |
| in | thermal time since first flower | Cd | phenology |
| out | internode directions and coils | - | geometry, light, viewer |
| out | racemes on new nodes | - | flowering |
| out | rotten pods | g DM | carbon and nitrogen ledgers (litter) |

## Encoded or emergent
Encoded: capture by distance and thickness, the helix, the fall at a fixed free length,
the heap, pod rot on the soil.

Emergent: height on a support, the extra stem a climber builds, the canopy of a fallen
vine, and its yield loss together with pod rot.

## Known limits
- A staked vine reaches 1.28 m at day 60 and 1.71 m at the end, short of the 1.5 m by
  day 60 the research recommends: from first flower the pods take the carbon its upper
  internodes need (decision 0021).
- Most of the unstaked yield loss is the encoded pod rot. In the sim a fallen vine's
  leaves are lit better, not worse, than a staked vine's.
- Branches do not climb; they stay herb-like (3 nodes, a terminal raceme).
- Supports do not shade and are not obstacles; vines do not twine around each other.
- The coil does not tighten or slip; grown organs move only when a vine falls.
- Handedness does not mirror in the southern hemisphere, as in real beans (to verify).

## Validation
- `ps_arch::vine::tests`: capture probability against Darwin's stakes (0.5, 1.5, 5 cm
  caught, 25 cm not); the coil is anticlockwise from above, keeps its radius and rises
  at its inclination; the lodge dip.
- `ps-validate/tests/pole_bean.rs::staked_pole_beans_climb_and_unstaked_ones_lodge`
  (WP22 done-when): every staked main stem twines (10 or more coiled internodes) and ends
  above 1.5 m; unstaked ones lodge, stay under 30 cm and yield 30 to 70% less.
- `ps-validate/tests/pole_bean.rs::bush_beans_ignore_supports`.
- Validation report: `pole_height_day60` (Info, misses at 1.28 m), `pole_height_final`
  (Info, 1.71 m in 1.5 to 2.1), `pole_unstaked_seed_yield` (Info, −53% in −30 to −70%).

## References
- Darwin C. (1875) The Movements and Habits of Climbing Plants.
- Millet B. et al. (1988); Hou et al. (1997): circumnutation periods of *Phaseolus*.
- Rutjens et al. (2025) climbing-bean FSPM (twining polygon, inclination, internode
  ranges).
- Onyenedum et al. (2025): coils lock when gelatinous fibres form.
- DSSAT CROPGRO-Drybean BNGRO048 (ECO FL-VS, CUL EM-FL).
- IITA PROPAS, low-cost staking of beans; Reckling M. et al. (2011).
- Research notes m2-m3-bean-science.md (decisions 1 and 2).
