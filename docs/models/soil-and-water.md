# Soil and water

## Purpose
Rain and watering soak in from the top, as fast as the soil lets them; the rest ponds
on the surface for a while and then runs off. Water moves up and down from wet soil to
dry and under gravity (R5a): a soaked bed drains over days, not hours, a dried surface
draws water up from below, and a pot with holes keeps a wet layer at its bottom, since
water leaves only through saturated soil at its holes. Water spreads sideways a little,
the sun dries the surface, and roots draw water from where they are: each root takes
what its own pull and the soil's wetness allow, so wet soil gives more (plant-hydraulics
card). How much water a soil holds also sets how tightly it holds it, its water
potential, and how easily water moves through it, its conductivity. Soil can be
compacted or loosened (R5c): a plough pan, a raised bed, a trampled path or a dug strip
changes how much air the soil holds, how fast water passes, and how hard it is for a
root to push through; dug soil settles back with the water that falls on it.

## Equations
Per cell of volume `V` (cubic, edge `Δx`), water mass `W` (kg); capacities
`W_x = θ_x V ρ_w` for saturation `s`, field capacity `fc`, wilting point `wp`, air-dry `ad`.

1. **Infiltration** (R5a, decision 0046): water in (mm) × `Δx²` per active column, plus
   what ponded there last hour; at most `K_s · 1 h` of it enters (loam 10.4 mm, sand
   297 mm, clay 2 mm an hour), filling cells top-down to `W_fc`, then to `W_s`. What
   does not enter or does not fit ponds on the column up to the pond store (5 mm on a
   plot, 25 mm in a pot, the space below its rim); the rest is runoff. A strip of
   columns `x0 ≤ x < x1` (a split pot's watered half) can get water of its own on top
   (R2b, decision 0043). Scheduled watering to field capacity counts the ponded water
   as given. Water that passes a cell on its way down carries nitrate as a mixing cell
   (Godwin and Jones 1991): the cell sends the share `p / (W + w_in)` of its nitrate
   plus what came in from above, `p` the water passing on and `w_in` the water coming
   in; the deepest cell the water reaches keeps the rest. In a grid with structure
   (equation 13) each cell's own `K_s` and saturation apply: the top cell takes at most
   its `K_s · 1 h`, and water filling a deeper cell passes every cell above it, no
   more than `K_s · 1 h` of the cell's class entering any cell below the top in the
   hour, so water perches on a dense layer and the rest ponds (R5c, decision 0049).
2. **Vertical flow** (R5a, decision 0046; `ps_soil::flow`): in each column, with heads
   `h` in metres of water (`ψ / ρg`, negative is suction), the downward flux across the
   face between cells `i` and `i + 1` is `q = K_f ((h_i − h_{i+1}) / Δz + 1)`, `K_f`
   the arithmetic mean of the two cells' `K(θ)` (equation 11). Between cells of
   different density classes (equation 13) it is
   `K_f = K_s,h · (K_i/K_s,i + K_{i+1}/K_s,{i+1}) / 2`, `K_s,h` the harmonic mean of
   their saturated conductivities, so a dense layer limits the flow through it as
   layers in series do (R5c). One backward-Euler step
   an hour with `K` and the retention slope `C = dθ/dh` taken at the start (one Picard
   step): `C_i Δz (h'_i − h_i) / Δt = q_in − q_out`, a tridiagonal system solved by
   the Thomas algorithm. The water then moves by the solved fluxes themselves, so each
   column conserves water exactly. A saturated cell's head is `−h_e + (θ − θ_s) / S_s`
   with a numerical specific storage `S_s` = 0.01 m⁻¹, so a perched water table has a
   pressure; a step that leaves a cell above `θ_s + S_s (n_z + 1) Δz` (more pressure
   than the column's own height of water) pushes the excess up, and out of the top to
   the pond. An hour whose single step would change any cell by more than 0.1 m³ m⁻³
   is split into 2, 4, … up to 64 equal steps. The solver reads the curve's head,
   slope and conductivity (equations 10 and 11) from a table on 4,096 even steps of
   relative saturation, from half the air-dry content to saturation: the head by cubic
   Hermite interpolation on its exact slope (within 1e-7 of the curve, so a
   hydrostatic profile set from the curve stays put), the slope and conductivity
   linearly (within 1e-3); the curve's powers had been most of the soil's cost.
   Columns are independent: on 256 or more active columns they are solved in parallel
   in fixed chunks of 32 and put back in column order, so any thread count gives the
   same state. The base:
   - a plot drains freely, `q = K_N` (unit gradient), and the water never returns;
   - a pot with holes is a seepage face: `q = K_N (h_N / (Δz/2) + 1)` only while
     `h_N > −Δz/2` (the base saturated), else nothing, so a soaked pot keeps its bottom
     near saturation;
   - a pot without holes passes nothing;
   - a pot in a saucer drains into it; while the saucer holds water at depth `d` the
     base is `q = K_b ((h_N − d) / (Δz/2) + 1)` either way (`K_b` the mean of `K_N` and
     `K_s`), each column taking at most what is left, in column order; past its depth
     the saucer spills (drained). A saucer does not evaporate.

   Nitrate moves with each face's net water over the hour at the upwind cell's
   start-of-hour concentration, each cell sending at most its store over all the water
   it sends; what leaves the base leaches (into a saucer too: its water returns
   without nitrate).
3. **Lateral**: for each horizontal neighbor pair, `f = a (W_i − W_j)`; `W_i −= f`, `W_j += f`,
   with `a = min(0.25, 0.1 (0.02 m / Δx)²)`, a diffusivity of 4e-5 m² h⁻¹ (decision 0019):
   0.1 in 2 cm pot cells, 0.016 in 5 cm plot cells. A watertight divider (a split
   pot's, R2b) removes the pairs across columns `nx/2 − 1` and `nx/2`.
4. **Evaporation**: water standing on a column evaporates first, at the full rate
   `E_p = 0.5 · R_soil · 3600 / λ · Δx²`, at most the pond (R5b, decision 0048); the top
   cell then gives `(E_p − E_pond) · clamp((W − W_ad)/(W_fc − W_ad), 0, 1)`, at most
   `W − W_ad`; `λ = 2.45 MJ/kg`.
5. **Soil temperature** of layer at depth `z`: `T = T̄₂₄ + e^{−z/D} (T_air(t − 24z/(2πD)) − T̄₂₄)`.
6. **FTSW** over root-occupied cells: `Σ L_i (W_i − W_wp) / Σ L_i (W_fc − W_wp)` in [0, 1];
   1 with no roots.
   It no longer sets a stress factor (R3b, decision 0044): it weights root carbon
   priority (carbon-allocation card), with the plant's stomatal factor.
7. *(Removed in R3b: the stress factor `min(1, FTSW / 0.4)` on stomata and expansion;
   the leaves' water potential sets both now, plant-hydraulics card.)*
8. **Uptake**, one call per hour for all plants in the grid (decision 0012), from each
   plant's network flows (R3b, decision 0044; `SoilGrid::take_flows`, equations in the
   plant-hydraulics card). Every plant's flows come from the same soil state, so plant
   order does not matter. A cell asked for more than its water above air-dry scales
   every withdrawal from it by the same factor; a plant's deposits (water its roots
   leak into drier cells) are scaled by its granted share, so its net uptake is that
   share of its demand. The shortfall is logged. Roots are sparse `(cell, length)`
   lists sorted by cell.
9. *(Removed in R3b: the per-root potential `min(k L/V (W − W_wp)⁺, u_max L, ...)` and
   its hourly ceiling `u_max` (decisions 0012, 0037). A root now draws past the wilting
   point down to air-dry, as its rhizosphere factor and the plant's potential allow.)*
10. **Water potential** (R3a; air entry R5a, decision 0046; `ps_soil::Retention`, read
    by the vertical flow and the plants' root networks). van Genuchten (1980) with the
    residual content set to 0 and the air entry of Ippisch, Vogel and Bastian (2006):
    `θ = θ_s` while `|ψ| ≤ h_e` (2 cm of water, 0.196 kPa), and
    `θ(ψ) = θ_s [1 + (|ψ| / ψ_s)^n]^(−m) / S_c` beyond, `m = 1 − 1/n`,
    `S_c = [1 + (h_e / ψ_s)^n]^(−m)`; inverted in closed form,
    `ψ = −ψ_s ((S S_c)^(−1/m) − 1)^(1/n)` with `S = θ/θ_s` (`−h_e` at saturation). Per
    texture, `n` and the suction scale `ψ_s = 1/α` are solved so that
    `θ(−33 kPa) = fc` and `θ(−1500 kPa) = wp`: an outer bisection on `n` for the
    wilting-point anchor around an inner bisection on `ln ψ_s` for the field-capacity
    anchor, each halving until its midpoint stops moving. The flow step's slope is
    `dθ/dψ = θ_s m n S (1 − y) / |ψ|` with `y = (S S_c)^(1/m)`.
    Published tables (Carsel and Parrish 1988) don't put their −33 kPa points at these
    field capacities, and keeping a published `n` while solving `θ_r` gave clay a
    negative `θ_r`, sand one equal to its wilting point and loam one above air-dry
    (docs/research/realism-plan/numbers/README.md). A cell's potential is
    `SoilGrid::water_potential`; the lens gets each layer's potential at the layer's
    mean water content (`SoilGrid::layer_water_potentials`). The anchor solve runs
    once per texture per process (`Texture::retention` caches it). The plant's root
    network reads each root cell's potential (R3b).
11. **Hydraulic conductivity** (R3c, decision 0045; air entry R5a, decision 0046;
    `Retention::flow_terms`, read by the vertical flow and the root network's soil
    term). Mualem (1976) on the same curve with Ippisch et al.'s scaling,
    `K(θ) = K_s · S^½ [(1 − (1 − y)^m) / (1 − (1 − S_c^(1/m))^m)]²`, 1 at saturation;
    the potential, its slope and the conductivity share `y`, three powers in all. With
    the anchored curves `K / K_s` is about 3.6e-4 at field capacity and 2.4e-8 at the
    wilting point in loam (sand 2.5e-5 and 7.8e-10, clay 4.0e-3 and 5.2e-7), four to
    five times the values without the air entry: Mualem's integral no longer counts
    pores larger than the air entry allows, which with `n` near 1.2 had cut `K` to a
    sixth of `K_s` within 3 cm of saturation.

12. **Aeration** (R5b, decision 0048; after evaporation each hour). A cell's air-filled
    porosity is `a = θ_s − θ`, and its oxygen deficit
    `d = clamp((a_c − a) / (a_c − a_0), 0, 1)`: 0 at `a_c` = 0.10 m³ m⁻³ or more (the
    classic limit for root function, Grable and Siemer 1968), 1 at `a_0` = 0.02 or
    less (CROPGRO's minimum pore space for roots, `PORMIN`, in its dry bean and soybean
    files). Its hypoxic hours `H` add `d` each hour and go back to 0 in an hour with
    `d = 0`. The cell is hypoxic once `H ≥ 6` (the oxygen left in wet soil is used up
    within hours; "~"), and its hypoxia is then `h = d`, else 0. Roots read `h` and
    `H` (herb-architecture and plant-hydraulics cards); nitrogen reads `h` (nitrogen
    card).

13. **Structure** (R5c, decision 0049; `ps_soil::structure`). A grid without structure
    is uniform at its texture's bulk density `ρ0` (loam 1300, sand 1550, clay 1250
    kg m⁻³) and runs as before. Once a scenario's density zone, a dig or a trampled path
    sets a density, every cell carries its bulk density `ρ`, the density it settles to
    `ρ_set`, the density it was loosened to `ρ_l`, and the water `R` that has reached
    its column's surface since then (rain and watering, mm).
    - *Hydraulic class*: the cell's density rounded to 1% steps of `ρ0` (factors 0.6 to
      1.4). Saturation `θ_s = θ_s0 − (ρ − ρ0) / ρ_p`, `ρ_p` = 2650 kg m⁻³, at least
      `θ_fc + 0.02`; field capacity and wilting point stay, and the retention curve is
      anchored through them again (equation 10). Saturated conductivity
      `K_s = K_s0 (φ_e / φ_e0)^4` with the effective porosity `φ_e = θ_s − θ_fc` (Ahuja
      et al. 1984). Loam at 1.2 `ρ0` (a plough pan, 1560 kg m⁻³): `θ_s` 0.352, 8% air
      at field capacity, `K_s` 0.45 mm h⁻¹ (a 23rd of 10.4); at 1.15 (a trampled path):
      0.376, 1.27 mm h⁻¹; at 0.85 (a raised bed): 0.524, 41 mm h⁻¹; freshly dug at
      0.667: 0.613, 137 mm h⁻¹. Each class and its flow table are built once per process
      and shared; the potential, conductivity, aeration (equation 12), nitrogen's water
      factor and the root network read each cell's class.
    - *Settling* (WEPP's rainfall consolidation, Alberts et al. 1995, eq. 7.7.9):
      `ρ = ρ_l + (ρ_set − ρ_l) R / (R + R_0)`, `R_0` = 10 mm, so dug soil closes half
      its gap after 10 mm of water and nine tenths after 90.
    - *Dig* (a region, to a depth): `ρ = min(ρ, 0.667 ρ0)` (WEPP's fully disturbed soil,
      eq. 7.7.1), `ρ_set = min(ρ_set, ρ0)`: dug soil settles back to the texture's
      density, so a pan dug through is gone. *Compact* (a trampled path):
      `ρ ≥ 1.15 ρ0`, and it stays. A scenario's density zone sets `ρ`, `ρ_set` and
      `ρ_l` to its factor times `ρ0`.
    - *Strength*, the penetration resistance, in the effective-stress form of Whalley
      et al. (2007): `Q = Q_fc (S |ψ| / σ_fc)^a · 10^(b (ρ − ρ0) / 1000)`, `S` the
      relative saturation `θ / θ_s` and `ψ` the matric potential of the cell's class,
      `σ_fc = (θ_fc / θ_s0) · 33 kPa` (the texture's own soil at field capacity),
      `Q_fc` = 0.5 MPa, `a` = 0.4, `b` = 2.0 per 1000 kg m⁻³ (all "~"; Whalley's
      coefficients were not reachable). Loam at field capacity reads 0.5 MPa at `ρ0`,
      1.3 at 1.15 and 1.8 at 1.2; at the wilting point about 1.6 at `ρ0`. Roots read
      `Q` against the reference `Q_ref`, the same expression at `ρ0` for the cell's
      water (herb-architecture card): the species' elongation rates were measured in
      ordinary soil.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Field capacity loam, sand, clay | 0.27, 0.15, 0.40 | m3/m3 | design plan §3.6 |
| Wilting point loam, sand, clay | 0.12, 0.05, 0.22 | m3/m3 | design plan §3.6 |
| Saturation loam, sand, clay | 0.45, 0.40, 0.50 | m3/m3 | ~ (decision 0007) |
| Air-dry loam, sand, clay | 0.05, 0.02, 0.10 | m3/m3 | ~ (decision 0007) |
| Lateral fraction at 2 cm | 0.1 | per hour | design plan §3.6 |
| Lateral reference cell | 0.02 | m | decision 0019 (fraction scales as `(0.02/Δx)²`, cap 0.25) |
| Evaporation coefficient | 0.5 | - | ~ (decision 0007) |
| Damping depth | 0.10 | m | ~ (typical moist loam) |
| Cell size, pots | 0.02 | m | design plan §3.6 |
| Cell size, plots | 0.05 | m | design plan §3.6 (decision 0019) |
| Retention `n`, `α` (loam) | 1.2150, 0.0314 | -, 1/cm | solved from fc, wp and saturation with the air entry (R3a, R5a); `ψ_s` 3.12 kPa |
| Retention `n`, `α` (sand) | 1.2885, 0.0964 | -, 1/cm | solved (R3a, R5a); `ψ_s` 1.02 kPa |
| Retention `n`, `α` (clay) | 1.1650, 0.0095 | -, 1/cm | solved (R3a, R5a); `ψ_s` 10.31 kPa |
| Air-entry suction `h_e` | 2 | cm of water | ~ (Ippisch et al. 2006; R5a spec §3.2) |
| Mualem pore connectivity `l` | 0.5 | - | Mualem 1976 |
| Saturated conductivity `K_s` (loam, sand, clay) | 24.96, 712.8, 4.8 | cm/d | Carsel and Parrish 1988 class means (R3c) |
| Specific storage `S_s` | 0.01 | 1/m | ~ (numerical; real soils ~1e-5) |
| Largest change in one flow step | 0.1 | m3/m3 | ~ (sub-steps, at most 64 an hour) |
| Flow table steps | 4,096 | - | numerical (from half the air-dry content to saturation) |
| Pond store, plot | 5 | mm | ~ (R5a spec §3.2) |
| Pond store, pot | 25 | mm | ~ (the space below a pot's rim) |
| Aeration limit `a_c` | 0.10 | m3/m3 | Grable and Siemer 1968 ("~", R5b spec §4) |
| Anoxic air-filled porosity `a_0` | 0.02 | m3/m3 | CROPGRO `PORMIN`, dry bean and soybean species files (R5b) |
| Hypoxia onset | 6 | full-deficit hours | ~ (R5b) |
| Bulk density `ρ0` loam, sand, clay | 1300, 1550, 1250 | kg/m3 | design plan §3.6 (nitrogen card) |
| Particle density `ρ_p` | 2650 | kg/m3 | the usual mineral value (R5c) |
| Density class step | 0.01 | of `ρ0` | ~ (numerical; 1% of a loam's density moves `θ_s` by 0.005) |
| Least structural air `θ_s − θ_fc` | 0.02 | m3/m3 | ~ (R5c) |
| `K_s` exponent on `φ_e` | 4 | - | Ahuja et al. 1984 [K, to verify] |
| Dug density | 0.667 | of `ρ0` | WEPP eq. 7.7.1 (Alberts et al. 1995) |
| Trampled density | 1.15 | of `ρ0` | ~ (R5c spec §5) |
| Settling water `R_0` | 10 | mm | WEPP eq. 7.7.9 (Alberts et al. 1995) |
| Strength at field capacity `Q_fc` | 0.5 | MPa | ~ (Ge et al. 2019: about 0.3 wet, over 1 dry) |
| Strength stress exponent `a` | 0.4 | - | ~ (about 3-fold from field capacity to wilting) |
| Strength density slope `b` | 2.0 | per 1000 kg/m3 | ~ (a loam pan at 1.6 Mg m⁻³ near 2 MPa; Bengough et al. 2011) |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | rain + irrigation | mm | weather, actions |
| in | radiation at the soil surface | W/m2 | light |
| in | air temperature | C | weather |
| in | each plant's flow to or from each root cell, water demand | kg | plant hydraulics |
| out | FTSW | - | root carbon priority |
| out | uptake per cell, shortfall | kg | plant, ledger |
| out | layer temperature | C | germination, roots |
| out | water potential per cell, per layer | kPa | the root network (R3b); lens (R3a) |
| out | water ledger (its storage counts ponded and saucer water) | kg | world ledger |
| out | ponded water, saucer water | mm | lens (R5a) |
| in | pot drainage: holes, none, saucer depth | - | scenario (R5a) |
| out | hypoxia and hypoxic hours per cell | -, h | roots, nitrogen (R5b) |
| in | a pot's drainage changed by `SetDrainage` | - | actions (R5b) |
| in | density zones; `Dig` and `Compact` regions | - | scenario, actions (R5c) |
| out | bulk density, penetration resistance and its reference per cell | kg/m3, MPa | roots (R5c) |

## Encoded or emergent
Encoded: all of the above. Emergent at plant level: drought stress timing, which
depends on how fast roots explore the pot and how much leaf area transpires.

## Known limits
Water moves up and down by Darcy's law but sideways by the earlier diffusion rule
(decision 0019), so a wet patch spreads sideways no faster in sand than in clay, nor
slower through a dense zone than a loose one (R5c). One
linearized step an hour: within the hour after a soaking a column can be 0.1 to 0.2
m³ m⁻³ off a finely stepped solution (worst in sand, whose wetting front crosses a
pot in minutes), within 0.02 after six hours. Infiltration fills top-down as before,
capped at `K_s` an hour: it ignores how much faster dry soil drinks at first, so heavy
rain runs off clay sooner than it would. Water above field capacity drains slowly: a
soaked loam column is still near 0.36 after two days and about 0.33 after a week, far
above the −33 kPa field capacity of 0.27, which the curve reaches only after weeks; the
done-when that asked for field capacity in 1 to 3 days is read as the end of rapid
drainage instead (decision 0046). A soaked loam pot with holes rests near saturation
(about 0.43): its 24 cm of height can hold only 24 cm of suction. Water rising to an
evaporating surface carries nitrate with it and leaves it there, and the next watering
carries most of it back down: a bare loam column refilled each night for 40 days ends
with 40% more nitrate in its top 2 cm. A face's conductivity is the arithmetic mean
of its two cells' (equation 11), which overstates flow into a dry cell from a wet one,
as into a dried surface over wet soil (the harmonic or upstream mean would give less);
bare loam's evaporation lands a little under Ritchie's two stages (24 mm in 20 days
against about 28), so it does not show, but it is the first thing to check if
evaporation ever reads high (R5a review). A saucer feeds its pot's columns in index
order, so when it is nearly empty the low-index columns drink first. No hysteresis. Air-dry soil reads far
below any plant's reach (loam about −88 MPa, sand −36, clay −178), since the curve has
no residual content to level off at; that only matters for the dry surface layer.
Evaporation ignores humidity and wind; saucer water does not evaporate, and ponded water
evaporates only where nothing shades the soil (R5b). Aeration reads only the
air-filled porosity: no oxygen diffusion, no respiration that uses the oxygen faster in
warm soil, and the same limits in every texture, although a clay's fine pores hold
less air at the same suction. A soaked garden-loam pot with holes stays under 10% air
for days after heavy rain (its perched table), as real field soil in a container
does; potting mixes, which keep 15 to 25% air when drained, are not modelled (R5b). In a plot the
surface radiation is one value for the whole plot, so the gaps between rows dry no
faster than the soil under the canopy (decision 0019). A root's radial conductance is
linear in its length in a cell, with no resistance in the soil between roots beyond the
rhizosphere drying factor, so crowded roots in one cell don't compete for its water
until the cell itself runs dry (R3b). Water a root gives back to a cell is not capped
at saturation; it needs a root wetter than the cell, so it only reaches a saturated
cell by gravity, in amounts far below the next hour's drainage.
Structure (R5c) has no cracks, biopores or worm channels, so a dense layer is uniformly
hard and a pan at 1.2 times the loam's density stops nearly every root in a season
that keeps it moist but not wet; real pans are crossed through old root channels.
Soil shrinks and swells not at all, frost and drying do not loosen it, and roots and
earthworms do not open it. The strength coefficients and the conductivity exponent
are guesses to verify; strength keeps no memory of how the soil was wetted. Dug soil
settles by the water reaching its surface only, not by its own weight or by time.

## Validation
- Water flow (R5a done-whens): `ps_soil::tests::soaked_loam_drains_over_days` (a soaked
  40 cm loam column sheds more than 15 mm on day 1, under 5 mm a day by day 3, still
  some on day 8; closure every hour), `a_column_at_equilibrium_stays_put` (the
  hydrostatic profile holds to 1e-9 for 48 h), `water_rises_from_a_water_table` (a sand
  column over free water comes to the curve's content at each cell's height within
  0.002; a dried top over wet loam is fed from below), `pots_keep_a_wet_bottom` (a
  soaked pot with holes keeps its bottom above 0.98 of saturation at hours 27 and 71; a
  pot without holes loses nothing; a saucer catches the outflow and feeds a drier pot
  back), `the_column_solver_conserves_water` (1e-12 every hour for every base, from
  saturated over dry), `the_flow_table_follows_the_curve` (head within 1e-7, slope and
  conductivity within 1e-3, every texture), `water_ponds_then_runs_off`.
- `ps_soil::tests::random_hours_conserve`: property test, random rain, radiation,
  temperature and uptake; closure every hour, water within saturation plus what a
  saturated cell holds under pressure.
- `infiltration_fills_from_the_top`, `ps_soil::nitrogen::tests::soaking_in_carries_nitrate_down`
  (the mixing-cell share, no deeper than the water goes; 40 days of sun and nightly
  refills leave the top layer under twice its nitrate),
  `lateral_redistribution_smooths_and_conserves`, `lateral_fraction_scales_as_a_diffusivity`, `evaporation_takes_from_top_only_and_stops_dry`,
  `a_strip_waters_its_columns_only` (R2b),
  `ftsw_and_stress`, `flows_are_granted_down_to_air_dry` and `joint_flows_share_cells_fairly`
  (R3b's network flows: air-dry floor, deposits, a shared cell, plant order),
  `deeper_soil_is_damped_and_lagged`,
  `pot_mask_area_is_close_to_circle`.
- Water potential (R3a): `ps_soil::retention::tests::anchors_hold_for_every_texture`
  (−33 kPa at field capacity and −1500 kPa at the wilting point to 1e-9, both ways),
  `parameters_match_the_realism_plan`, `potential_falls_as_the_soil_dries` (monotone,
  `−h_e` at saturation), `inverse_is_exact`, `finite_down_to_air_dry`,
  `air_entry_and_slope` (flat to `h_e`, continuous there, the slope a finite
  difference's to 1e-5); and `ps_soil::tests::pot_water_potential_follows_the_water`
  (a pot dried from the top reads drier the higher the layer).
- Structure (R5c): `a_structured_grid_at_the_textures_density_runs_as_before` (bit for
  bit over 96 h, strength at its reference),
  `density_moves_saturation_and_conductivity_and_keeps_the_anchors` (every class from
  0.67 to 1.3 keeps its field capacity and wilting point; a pan at 1.2 holds 7.5 to 9%
  air at field capacity and conducts 20 to 40 times less),
  `strength_rises_with_density_and_dryness` (monotone in both from field capacity to
  wilting; a pan at 1.2 reads 1.6 to 2 MPa at field capacity),
  `dug_soil_settles_and_trampled_soil_stays` (half the gap after 10 mm; the state
  round-trips), `water_perches_on_a_pan` (rain on a pan at 1.25 wets the layer above
  it, leaves the pan short of air for more than a day and the soil below drier). The
  plot experiments are in `ps-world/tests/compaction.rs` (herb-architecture card).

## References
- Sinclair, T. R., Ludlow, M. M. (1986). Influence of soil water supply on the plant water balance of four tropical grain legumes. Aust. J. Plant Physiol. 13, 329–341 (FTSW).
- Ritchie, J. T. (1998). Soil water balance and plant water stress. In Understanding Options for Agricultural Production (bucket models).
- van Genuchten, M. Th. (1980). A closed-form equation for predicting the hydraulic conductivity of unsaturated soils. Soil Sci. Soc. Am. J. 44, 892–898 (retention curve).
- Carsel, R. F., Parrish, R. S. (1988). Developing joint probability distributions of soil water retention characteristics. Water Resour. Res. 24, 755–769 (saturated conductivities; their retention parameters are not used).
- Mualem, Y. (1976). A new model for predicting the hydraulic conductivity of unsaturated porous media. Water Resour. Res. 12, 513–522.
- Ippisch, O., Vogel, H.-J., Bastian, P. (2006). Validity limits for the van Genuchten–Mualem model and implications for parameter estimation and numerical simulation. Adv. Water Resour. 29, 1780–1789 (air entry).
- Richards, L. A. (1931). Capillary conduction of liquids through porous mediums. Physics 1, 318–333.
- Celia, M. A., Bouloutas, E. T., Zarba, R. L. (1990). A general mass-conservative numerical solution for the unsaturated flow equation. Water Resour. Res. 26, 1483–1496 (mass-conservative stepping).
- Godwin, D. C., Jones, C. A. (1991). Nitrogen dynamics in soil-plant systems. In Hanks, J., Ritchie, J. T. (eds.), Modeling Plant and Soil Systems, Agronomy Monograph 31, 287–321 (nitrate carried by water as a mixing cell).
- Veihmeyer, F. J., Hendrickson, A. H. (1931). The moisture equivalent as a measure of the field capacity of soils. Soil Sci. 32, 181–193 (field capacity after 2 to 3 days of drainage).
- Allen, R. G., et al. (1998). FAO-56 (latent heat; Priestley–Taylor scale for the evaporation coefficient).
- Grable, A. R., Siemer, E. G. (1968). Effects of bulk density, aggregate size, and soil water suction on oxygen diffusion, redox potentials, and elongation of corn roots. Soil Sci. Soc. Am. J. 32, 180–186 (about 10% air-filled porosity for root function; R5b).
- Ahuja, L. R., Naney, J. W., Green, R. E., Nielsen, D. R. (1984). Macroporosity to characterize spatial variability of hydraulic conductivity and effects of land management. Soil Sci. Soc. Am. J. 48, 699–702 (`K_s` from effective porosity; R5c).
- Alberts, E. E., et al. (1995). Soil component. In USDA Water Erosion Prediction Project (WEPP) model documentation, chapter 7 (bulk density after tillage, eq. 7.7.1, and its rainfall consolidation, eq. 7.7.9; R5c).
- Whalley, W. R., To, J., Kay, B. D., Whitmore, A. P. (2007). Prediction of the penetrometer resistance of soils with models with few parameters. Geoderma 137, 370–377 (the effective-stress form; R5c).
- Bengough, A. G., McKenzie, B. M., Hallett, P. D., Valentine, T. A. (2011). Root elongation, water stress, and mechanical impedance: a review of limiting stresses and beneficial root tip traits. J. Exp. Bot. 62, 59–68 (R5c).
- Ge, Hawkesford, Rosolem, Mooney, Ashton, Evans, Whalley (2019). Multiple abiotic stress, nitrate availability and the growth of wheat. Soil Tillage Res. 191, 171–184 (field soil drying from about 0.3 to over 1 MPa; impeded wheat roots wider; R5c).
- Boote, K. J., Jones, J. W., Hoogenboom, G. CROPGRO species files `BNGRO048.SPE` and `SBGRO048.SPE` (DSSAT CSM, `PORMIN` 0.02, the minimum pore space for roots; R5b).
