# Plant hydraulics

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Water runs from the soil into the roots, up the stem and out through the leaves, pulled
by the leaves' transpiration. Each organ has a water potential: near the soil's at
night, and falling through the day as the leaves draw harder than the roots and stem can
pass water on. When a leaf's potential falls far enough its stomata close, and its cells
lose turgor: the leaf wilts and stops growing until water comes back. Roots in wet soil
can feed the whole plant while the rest of the pot dries, and at night roots carry a
little water from wet soil into dry soil (hydraulic redistribution). Realism plan R3b,
decision 0044.

## Equations
The organs are the nodes of a tree; the organ tree's parent links are its axial edges.
Water potential `ψ` is in MPa. In the solve each node holds its total potential,
`ψ_total = ψ_p + ρ g z` (`ρ g` = 9.80665e-3 MPa m⁻¹, `z` the organ's mid height, m,
negative below ground); organs report the pressure part `ψ_p`.

1. **Radial links.** Each root organ links to every soil cell it runs through (its spans,
   built with the root cells each day, half-cell sampling as the root cells):
   `k_r = Lp_r · 2π r L · f_dry(θ)` (kg h⁻¹ MPa⁻¹ after × 3.6e6), `r` the organ's radius
   (m), `L` its length in the cell (all the roots a cohort stands for, m). `Lp_r`
   falls linearly from the young to the thickened value with the organ's secondary
   share of its structure. `f_dry = clamp((θ − θ_ad) / (θ_wp − θ_ad), 0, 1)`: the soil
   next to the root stops conducting as it dries from the wilting point to air-dry.
   Each soil cell is a fixed total potential for the hour, its R3a matric potential
   plus gravity at the cell's center.
2. **Axial edges** (parent to child, the child's conductance, kg h⁻¹ MPa⁻¹):
   - a root organ: `m · K_x · G / l` with `m` the roots a cohort stands for and `l` the
     length of one; on a main axis `K_x` rises log-linearly from the tip's value to
     the mature value (tap root or basal) over `d_m` from the tip,
     `K_x = K_tip (K_mature / K_tip)^clamp(d / d_m, 0, 1)` with `d` the distance of the
     segment's base from the tip; a lateral or fine root takes the tip's value.
     `G = W / W_primary ≥ 1`, the organ's structure over its primary structure, is
     `(r / r_primary)²`: secondary xylem adds conducting area as the root thickens
     (R1a), as the pipe model does for stems;
   - an internode: `K_s · f_x · π r² / l · 3600`, so stem thickening (R1a) raises it;
   - a leaf (petiole and blade): `K_leaf · A`, with `K_leaf` in mmol m⁻² s⁻¹ MPa⁻¹
     converted by 18.015e-6 kg mmol⁻¹ × 3600 s h⁻¹ and `A` the leaf's area;
   - every other organ (buds, flowers, pods, seeds, cotyledons, nodules): 1 kg h⁻¹
     MPa⁻¹. They carry no flow, so only their potential, which follows the parent's,
     matters. Every edge is at least 1e-9.
3. **Sinks.** Each leaf draws its hour's transpiration `E_i` (kg), from gas exchange
   and the leaf energy balance (WP27).
4. **Solve.** At every organ, `Σ_j k_ij (ψ_j − ψ_i) + Σ_c k_r,ic (ψ_c − ψ_i) − E_i = 0`
   (Kirchhoff, no water storage). The soil cells are fixed, so the system is a forest
   of trees: one elimination from the highest organ id down and one substitution back
   up, O(organs), the same solve as the phloem transport
   (`ps_physio::hydraulics::solve_tree`). Each root cell's flow is
   `q_c = Σ_i k_r,ic (ψ_c − ψ_i)`: positive is uptake, negative is water a root leaks
   into drier soil. `Σ_c q_c = Σ_i E_i` to round-off.
5. **Joint uptake** (one call per soil per hour; `SoilGrid::take_flows`). Every plant's
   flows are computed against the soil as it stands before any is taken, so plant
   order does not matter. A cell asked for more than its water above air-dry scales
   every withdrawal from it by the same factor. A plant's deposits are scaled by its
   granted share of what it asked, so its net uptake is that share of its demand. The
   water ledger closes at 1e-9 every hour (debug builds assert it).
6. **No working link.** Before roots reach the soil, or when every root cell is
   air-dry, the plant takes nothing and every organ reads the root zone's potential.
7. **Root-zone potential**: the radial-conductance-weighted mean of the root cells'
   matric potentials, reported as the predawn soil potential and in the lens.
8. **Stomata** (Tuzet et al. 2003): the Medlyn slope is multiplied by
   `f_ψ = (1 + e^(s_f ψ_f)) / (1 + e^(s_f (ψ_f − ψ_leaf)))`, from the previous hour's
   leaf-area-weighted mean leaf potential, rounded to steps of 1/50 so that plants of a
   plot can share one light-class table (gas-exchange card). It replaces R2a's FTSW and
   supply factors on stomata. The one-hour lag removes the iteration between stomata
   and potential.
9. **Turgor**: `T(ψ) = clamp((ψ − ψ_tlp) / (ψ_0 − ψ_tlp), 0, 1)`, per organ from the
   last hour's solve; a leaf reads its parent node's potential, since it grows at its
   base, fed by the xylem before the blade's own resistance. The lens shows every
   organ's `T` and the leaves' area-weighted mean (how wilted the plant looks).
10. **Stored growth.** A growing leaf, internode or cotyledon's expansion clock runs at
    `T` times its other factors (cold, decision 0011), so a midday dip in turgor delays
    expansion instead of cutting the organ short: it catches up when turgor returns in
    the evening and at night. Leaves of ryegrass and rape do this (Baca Cabrera et al.
    2020, Lu et al. 2022): their daily growth stayed the same when hot dry air cut
    their daytime growth. A drought that holds turgor down for days still costs size,
    through the carbon the closed stomata don't fix and the days of growth lost at the
    season's end.
11. **Thickening** (R1a, decision 0037): the day's water factor on stem and root
    thickening is the leaves' mean turgor at 04:00 to 05:00 (predawn): stems and roots
    thicken mostly at night, when their tissues rehydrate.

## Parameters
Species file constants (schema 17). Wheat takes the bean's values: no wheat source yet.

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| `root_radial_conductivity` (`Lp_r`, young and fine) | 2e-7 | m s⁻¹ MPa⁻¹ | Fiscus and Markhart 1979, whole bean root systems, 0.8 to 6e-7 |
| `root_radial_conductivity_thickened` | 5e-8 | m s⁻¹ MPa⁻¹ | ~ suberized over young root about 0.39 in pine |
| `root_axial_conductance_tip` (`K_tip`; laterals, fine roots) | 2e-11 | m⁴ s⁻¹ MPa⁻¹ | Barrios-Gómez 2012 tap-root apex metaxylem by Hagen–Poiseuille × 0.4 |
| `root_axial_conductance_tap` (mature) | 2e-9 | m⁴ s⁻¹ MPa⁻¹ | Barrios-Gómez 2012 tap-root base metaxylem |
| `root_axial_conductance_basal` (mature) | 1e-10 | m⁴ s⁻¹ MPa⁻¹ | Massas 2022, wild *P. vulgaris* basal root 8.9e-11 |
| `root_axial_maturation` (`d_m`) | 20 | cm | ~ |
| `stem_xylem_conductivity` (`K_s`) | 9.8 | kg m⁻¹ s⁻¹ MPa⁻¹ | ~ calibrated so a 5 mm stem has Holste 2006's k_h 2.9e-5 kg m s⁻¹ MPa⁻¹ |
| `xylem_area_fraction` (`f_x`) | 0.15 | - | ~ only the product with `K_s` matters |
| `leaf_hydraulic_conductance` (`K_leaf`) | 8.7 | mmol m⁻² s⁻¹ MPa⁻¹ | back-calculated from Haverroth 2025, 5 to 15 |
| `tuzet_sensitivity` (`s_f`) | 4 | MPa⁻¹ | fit to bean closure from −0.4 to −0.6 MPa and about −1.2 MPa, 3 to 6 |
| `tuzet_reference` (`ψ_f`) | −1.0 | MPa | the same fit |
| `turgor_loss_point` (`ψ_tlp`) | −0.9 | MPa | Zlatev and Stoyanov 2005, well watered, −0.6 to −1.3; Haverroth 2025 −0.83 |
| `turgor_full` (`ψ_0`) | −0.2 | MPa | R3b spec §4.4 |
| Stomatal factor steps | 50 | per unit | decision 0044 (class-table sharing) |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | each leaf's transpiration this hour | kg | gas exchange, leaf energy balance |
| in | root spans: (root organ, cell, length) | -, -, m | daily root rebuild (geometry) |
| in | soil water content and potential per cell | m³ m⁻³, kPa | soil-and-water card (R3a) |
| in | organ sizes, heights and secondary share | m, m, - | organ growth, thickening |
| out | flow to or from each root cell | kg | soil joint uptake |
| out | every organ's water potential | MPa | next hour's stomata and expansion; lens |
| out | stomatal factor | - | gas exchange (next hour) |
| out | organ turgor, the leaves' mean turgor | - | expansion clock, thickening; lens (wilting) |
| out | root-zone potential; predawn leaf and soil potential, daily low leaf potential | MPa | lens, daily CSV |

## Encoded or emergent
Encoded: the conductances, the Tuzet and turgor curves, the one-hour lag, stored growth.
Emergent: the daily course of leaf potential (near the soil's before dawn, lowest in the
early afternoon); where the plant takes its water (wet cells give more, a split pot's
watered half carries the plant); hydraulic redistribution at night; how far a tall vine's
top falls below its base; wilting in a dry pot and the recovery after watering; the
stomata closing through a drought before the soil reaches its wilting point.

## Known limits
- No water storage in the plant (no capacitance): potentials follow transpiration
  within the hour, with no buffering by stem or leaf water.
- No cavitation or xylem vulnerability: `K_leaf` and `K_x` don't fall as the potential
  drops, so a very dry plant follows the soil down (to several MPa in an air-dry pot)
  instead of disconnecting.
- No stomatal memory after severe stress: real bean stomata take up to 8 days to reopen
  after −1.6 to −2.5 MPa (Trujillo 2013); here they reopen in the hour the potential
  recovers.
- Radial conductance is the same in both directions (no rectification), so roots in dry
  soil leak water at the rate they would take it up; in the split pot about 18% of what
  the roots take goes back into drier soil at night. Real roots pass water outward
  less readily, so this is likely on the high side. It keeps a drying half wetter, so
  R2b's dry-soil rules act less (`split_pot_foraging` 1.25).
- A pod or seed draws no water; fruit growth reads no turgor.
- Roots don't sense potential yet: their dry-soil rules (R2b) read relative available
  water.
- The stem is less negligible than the research numbers expected: a 1.1 m pole bean's
  stem takes about 0.18 MPa at noon, against 0.5 in its roots and up to 0.5 in a
  leaf. `K_s` is not tuned.
- Wheat's values are the bean's.
- The cost is a network solve per plant per hour; R3c's simplified level is the
  answer for large plots and fast-forward.

## Validation
- `ps_physio::hydraulics::tests`: `tuzet_is_one_at_zero_and_half_near_the_reference`,
  `turgor_runs_from_the_loss_point_to_full`, and the tree solve:
  `two_nodes_match_the_series_resistances`, `flows_balance_and_dry_soil_takes_water_back`
  (redistribution), `no_flow_means_soil_potential_everywhere`,
  `a_tree_without_roots_has_no_solution`.
- `ps_arch::tests::rhizosphere_and_stomatal_steps`.
- `ps-arch/tests/hydraulics.rs` (bush bean, 24 cm loam pot, watered nightly):
  `leaf_water_potential_follows_the_day`: on the brightest third of days 35 to 60,
  predawn leaf potential within 0.2 MPa of the root zone's and the daily low between
  −0.4 and −1.1 MPa (R3b spec §4.5); at 02:00 net flow equals demand to 1e-9 and every
  organ lies within 0.05 MPa of the root cells' range.
  `a_split_pot_draws_on_its_wet_half`: the R2b split pot run to day 60; in hours when
  the east half's roots are below 30% of their available water, more than half of the
  uptake comes from the watered west half.
- `ps-world/tests/wilting.rs`: in the wilt-and-recover scenario the dry pot's leaves
  fall below the turgor loss point and their turgor reaches 0, and turgor is back above
  0.5 within 6 hours of the watering (Kim and Lee-Stadelmann 1984).
- The water ledger's 1e-9 closure every hour in debug builds, the network's own
  closure (`Σ q = Σ E`) in debug builds, and every M0 validation gate (decision 0044).

## References
- Tuzet, A., Perrier, A., Leuning, R. (2003). A coupled model of stomatal conductance, photosynthesis and transpiration. Plant Cell Environ. 26, 1097–1116.
- Doussan, C., Pagès, L., Vercambre, G. (1998). Modelling of the hydraulic architecture of root systems: an integrated approach to water absorption. Ann. Bot. 81, 213–223.
- Bean sources of the parameter table and the recovery check, as the R3 research notes
  give them (docs/research/realism-plan/numbers/README.md, R3 parameters): Fiscus and
  Markhart 1979 (whole bean root systems), Steudle and Brinckmann 1989 (single young
  roots of *P. coccineus*), Barrios-Gómez 2012 (tap-root metaxylem), Massas 2022 (basal
  root axial conductance), Holste 2006 (bean stem conductance), Haverroth 2025 (leaf
  conductance and vulnerability), Zlatev and Stoyanov 2005 (turgor loss point), Kim and
  Lee-Stadelmann 1984 (recovery after rewatering), Trujillo 2013 (slow recovery after
  severe stress).
- Baca Cabrera, J. C., Hirl, R. T., Zhu, J., Schäufele, R., Schnyder, H. (2020). Atmospheric CO2 and VPD alter the diel oscillation of leaf elongation in perennial ryegrass: compensation of hydraulic limitation by stored-growth. New Phytol. 227, 1776–1789. doi:10.1111/nph.16639.
- Lu, Z., et al. (2022). Potassium regulates diel leaf growth of *Brassica napus* by coordinating the rhythmic carbon supply and water balance. J. Exp. Bot. 73, 3686–3698. doi:10.1093/jxb/erac060.
