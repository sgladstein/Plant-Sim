# Grass shoot

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
A grass such as wheat grows from a crown at the soil surface. Each new leaf unrolls
out of the rolled sheath of the one before. Buds at the crown nodes become tillers,
side shoots that grow their own leaves, until the stem starts to shoot up at jointing
or the crowd of neighbours shades the crown. Then the youngest tillers die. Every
shoot that is left ends in an ear on a long stalk, and the ear fills grain.

## Equations
Code: `ps_arch::grass`. Decision 0022.

1. **Node** on an axis with `N` leaves, at rank `k`, every phyllochron `P`:
   - **Internode:**
     - For `k ≤ N − E`: `ℓ = ℓ_crown`.
     - Otherwise: `ℓ = ℓ_pot (f₁ + (1 − f₁)(j − 1)/(E − 1)) · s_shade`, with
       `j = k − (N − E)`.
     - An elongating internode's expansion clock starts at `−P`.
   - **Sheath:** `ℓ_sh = ℓ_sh1 + (ℓ_shN − ℓ_sh1)(k − 1)/(N − 1)`, along the axis.
   - **Blade area:**
     - For `k < N`: `A = A_max (b₁ + (1 − b₁) min(1, (k − 1)/(N − 2)))`.
     - The flag leaf (`k = N`): `A = A_max b_flag`.
     - The blade sits at the sheath's top, at elevation `e₀`, azimuth advancing by the
       phyllotaxis (185°).
     - A plant's first blade has a uniform random azimuth on its `Tropism` stream; a
       tiller's first blade is `φ_t` (40°) round from its parent's.
2. **Tiller site** at rank `r` of an axis of order `o < o_max`, on a crown node:
   - It is decided once, when the axis reaches `r + d` leaves.
   - It fills with probability `p = s_f · min(1, C_node / C_ref)`.
   - It stays empty if `RFR_node < RFR_stop`, or if the tiller would have fewer than
     2 leaves.
   - A tiller has `N_t = N_axis − r − d + 1` leaves. It leans `θ_t` out on the
     site's azimuth.
3. **Jointing:** when the main stem's first elongating internode starts to grow
   (`nodes − 1 > N − E`), no site fills any more. Tillers with fewer than `L_min`
   leaves are shed.
4. **Heading:** after leaf `N`, the next phyllochron adds a peduncle (`ℓ_ped`) and an
   ear (`Pod`, chaff `W_ear` over `τ_ear`).
5. **Grain set:** once the ear is `τ_lag` old and the plant has reached anthesis, the
   number of grains is `n ~ Binomial(n_fl, ovule_set · h · min(1, C_ear / C_ref))`, at
   least 1. `h` is the mean pod-set temperature factor of the last 7 daily maxima.
6. **Blade light points:** three strips of `A/3`.
   - Width `w = √(A/(0.75 a))`, length `L = a w`.
   - Strip `j` has elevation `e₀ − 0.6 (j + ½)/3`.
   - Each strip's normal is perpendicular to it in its vertical plane, facing up.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| phyllochron | 85 | Cd | ~ spring sowings (Baker 1980); WALTer 99 |
| final leaf number (`nodes_before_terminal`) | 9 | - | ~ 8 to 9 for spring wheat |
| crown_internode_length | 0.3 | cm | ~ |
| elongating_internodes | 4 | - | ~ |
| internode_length_potential, first fraction | 18, 0.2 | cm, - | ~ |
| peduncle_length | 30 | cm | ~ |
| sheath_length_first, sheath_length_last | 3, 15 | cm | ~ elong-Wheat |
| leaf_area_potential, blade_first/flag_fraction | 30, 0.15, 0.8 | cm², - | ~ elong-Wheat widths |
| blade_aspect, blade_elevation | 18, 40 | -, deg | ~; WALTer |
| tiller_delay | 2 | phyllochrons | Klepper 1982 (n + 2 or 3) |
| site_filling | 0.69 | - | Neuteboom 1989 |
| tiller_stop_rfr | 1.1 | - (proxy) | ~ WALTer/Evers 0.35 measured; set on the proxy for 2 to 3 ears per plant at 200 per m² |
| tiller_max_order, tiller_survive_leaves, tiller_angle | 2, 2, 15 | -, -, deg | ~ |
| tiller_azimuth, phyllotaxis | 40, 185 | deg | WALTer |
| maturity | 1700 | Cd | ~ APSIM spring types 1700 to 1900 |
| florets per ear, floret set | 50, 0.85 | - | ~ |
| ear chaff, ear growth, lag, grain fill | 0.45 g, 200, 180, 550 | g, Cd | ~; APSIM-Wheat |
| phloem_conductance (wheat) | 1500 | g/m/h | ~ as the pole bean (decision 0021) |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | thermal time | Cd | phenology |
| in | red:far-red and absorbed PAR at a node | -, mol m⁻² d⁻¹ | light |
| in | node and ear reserves | g C | carbon allocation |
| out | internodes, sheaths, blades, tillers, peduncles, ears, grains | organs | organ graph, light, viewer |
| out | tiller deaths at jointing | g DM | carbon and nitrogen ledgers |

## Encoded or emergent
**Encoded:**
- the tiller timetable (site delay, the stop at jointing, the death of small tillers);
- the red:far-red threshold;
- the profiles of internodes, sheaths and blades.

**Emergent:**
- how many sites fill, which depends on carbon;
- when the shade threshold bites in a stand, which depends on its leaf area;
- height, which depends on carbon reaching the elongating internodes;
- grains per ear and grain mass.

## Known limits
- Grasses have no cambium, so stems and roots never thicken (`secondary_growth` 0 in
  the species file; decision 0036). A culm's width is its primary allometry's.
- A dense stand stays too short (0.56 m against 0.7 to 1.0) and yields too little. Its
  culms elongate while the plants are short of sugar, and the sim does not yet treat
  the growing stem as the main sink at jointing.
- Ears set about a third of the grains a field ear does.
- The red:far-red threshold is set on the sim's simple shade signal, not on a measured
  ratio. The signal is read at the blade over the bud rather than at the crown, so the
  threshold sits high (1.1) and a lone plant's own blades cost it half a tiller.
- The crown sits 1 cm above the soil; a real wheat crown is 1 to 2 cm below it.
- All crown roots appear as one cohort, not one set per tiller.
- No vernalization, no photoperiod response and no lodging. Leaves grow from their
  base only in the sense that a blade appears from the sheath; blade growth itself is
  the usual expansion curve.

## Validation
- `ps_arch::grass::tests`: blade profile and elongating nodes; blade light points keep
  the area and face up.
- `ps-validate/tests/wheat.rs`:
  - `spaced_wheat_tillers_heads_and_fills_grain` (WP23 done-when): 3 to 10 tillers,
    0.7 to 1.0 m, one ear per living shoot, sheaths paired with blades, no nodules,
    more than 1 g of grain;
  - `a_dense_stand_tillers_less` (at day 50, the stand has fewer than 70% of the spaced
    plants' tillers; more than 150 of the 200 plants have their own first-blade
    azimuth).
- Validation report (all Info):
  - `wheat_tillers_spaced`: misses at 3.75 (4 to 12);
  - `wheat_tillers_stand`: 1.9 in 1 to 5;
  - `wheat_ears_stand`: 2.9 in 2 to 3;
  - `wheat_height_spaced`: 0.78 m in 0.7 to 1.0;
  - `wheat_height_stand`: misses at 0.56 m;
  - `wheat_density_tillers`: −50% in −30 to −90%;
  - `wheat_maturity_day`: 104 d in 100 to 130;
  - `wheat_peak_leaf_area_stand`: 0.0215 m² per plant (GAI 4.3) in 0.02 to 0.03;
  - `wheat_shoot_mass_stand`: misses at 3.7 g (5 to 9);
  - `wheat_grain_stand`: misses at 1.16 g (1.5 to 3).

## References
- Lecarpentier C. et al. (2019) WALTer. Annals of Botany 123: 961-975.
- elong-Wheat parameters (OpenAlea wheatfspm); CPlantBox wheat.xml.
- Klepper B., Rickman R.W., Peterson C.M. (1982) Quantitative characterization of
  vegetative development in small cereal grains. Agronomy Journal 74: 789-792.
- Neuteboom J.H., Lantinga E.A. (1989) Tillering potential and relationship between
  leaf and tiller production in perennial ryegrass. Annals of Botany 63: 265-270.
- Evers J.B. et al. (2006) Cessation of tillering in spring wheat in relation to light
  interception and red:far-red ratio. Annals of Botany 97: 649-658.
- Research notes m2-m3-bean-science.md (decision 5, the spring wheat table).
