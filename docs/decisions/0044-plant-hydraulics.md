# 0044: Water moves through each plant as a network (R3b)

- **Date:** 2026-11-06
- **Status:** accepted (the realism plan's R3b, owner-approved queue)

## Context
Since WP7 a plant's water status was one number, the FTSW of its root zone: stomata
closed as FTSW fell, uptake was split across root cells in proportion to root length,
and growth and thickening read a stress factor from FTSW. Nothing in the plant had a
water potential, so a dry pot could not wilt in the afternoon and recover at night, a
plant could not draw on its wet roots while others sat in dry soil, and a tall vine's
top was no drier than its base. R3a (PR #140, no model change) gave each soil cell a
matric potential. The realism plan's R3b
(`docs/research/realism-plan/r3-plant-hydraulics.md` §4 and §5, numbers in
`docs/research/realism-plan/numbers/README.md`) adds the plant's half: a network from
the soil cells through the roots, stem and leaves, with done-whens for the day cycle of
leaf potential, wilting and recovery, and a split pot that lives on its wet half.

## Decision
1. **Network** (`ps_arch::hydraulics`, plant-hydraulics card equations 1 to 7). The
   organs are nodes and the organ tree's parent links the axial edges; each root organ
   links radially to every soil cell its spans run through,
   `k_r = Lp_r · 2π r L · f_dry`. Soil cells are fixed total potentials for the hour
   (matric plus gravity at the cell's center); organs report pressure potential. The
   system is a forest of trees, solved by one elimination from the highest organ id
   down and one substitution back up (`ps_physio::hydraulics::solve_tree`), O(organs).
   Its net flow equals the hour's demand to round-off (asserted in debug builds against
   the gross flow, since radial flows of both signs can cancel).
2. **Joint uptake** (`SoilGrid::take_flows`, replacing `take_up` and its proportional
   split). Every plant's flows are computed against the soil as it stands after step 4,
   in parallel and independent of plant order. A cell asked for more than its water
   above air-dry scales every withdrawal from it by the same factor; a plant's
   deposits (water leaked into dry cells) are scaled by its granted share, so its net
   uptake is that share of its demand and the ledger closes.
3. **Conductances.** As the spec, with three choices of ours:
   - a root's axial conductance rises log-linearly from the tip value to the mature
     value over 20 cm (`root_axial_maturation`, a guess) instead of the spec's three
     point values;
   - it is multiplied by `G = W / W_primary ≥ 1`, which is `(r / r_primary)²`, so
     secondary xylem in a thickened root (R1a) conducts as stem thickening does. Without
     it the tap root's base carried a 1.1 m pole bean's whole transpiration through its
     primary metaxylem and the root axial drop dominated the plant (−0.5 MPa of the
     −1.26 MPa at the lowest leaf at noon on day 50);
   - organs that carry no flow (buds, flowers, pods, seeds, cotyledons, nodules) get a
     conductance of 1 kg h⁻¹ MPa⁻¹ and every edge a floor of 1e-9, so the solve never
     divides by zero.
4. **Stomata** read Tuzet's factor of the previous hour's leaf-area-weighted mean leaf
   potential, in steps of 1/50 so that plants of a plot still share one light-class
   table (the table key holds the factor's bits). It replaces R2a's FTSW and supply
   factors on stomata.
5. **Turgor and stored growth.** Turgor is `clamp((ψ − ψ_tlp) / (ψ_0 − ψ_tlp), 0, 1)`
   from the last hour's solve; a leaf reads its parent node's potential, since it grows
   at its base. A growing leaf, internode or cotyledon's **expansion clock** runs at its
   turgor times its other factors, so a midday dip delays expansion and the organ
   catches up in the evening and at night (stored growth: Baca Cabrera et al. 2020,
   doi:10.1111/nph.16639; Lu et al. 2022, doi:10.1093/jxb/erac060). The spec put the
   turgor term on the expansion increment; see below.
6. **Thickening** reads the leaves' mean turgor at 04:00 to 05:00 (predawn) as its daily
   water factor, times the day's N factor and carbon met. The hourly stress sum now
   holds the N factor only.
7. **What FTSW still does.** Root carbon priority reads the smaller of FTSW and the
   stomatal factor; pod abortion and nitrogen fixation keep FTSW (their sources give
   soil water, not plant potential).
8. **Season debrief** (challenges): water's pressure is the larger of the shortfall
   share and 1 − the predawn turgor, plus the FTSW clause in bloom. The daily CSV gains
   `leaf_turgor_predawn`.
9. **Species schema 17**: the eleven constants of the card's parameter table, the
   bean's values for all three species (wheat's unchecked).
10. **Step order** (§3.2): the step numbers stay and their content changes, as the spec
    §5 says: 5 reads the previous hour's potentials, 6 is gas exchange with WP27's
    energy balance, 7 is the network solve and the joint uptake against the soil after
    step 4.
11. Van Genuchten retention is now built once per texture (a `OnceLock` per curve):
    the network reads every root cell's potential every hour.

### Tried and not kept
Four world seeds of the validation suite unless noted.
- **Turgor on the expansion increment** (the spec's `f_exp`), each organ's own: leaf
  area −40% and total mass −21% against 0.24.0. The leaves' midday turgor in a watered
  pot is about 0.2 to 0.3, so the increment rule cut every day's growth by most of the
  light hours.
- **The same, leaves reading their base node:** pot dry mass −11%, leaf area −13%, pole
  bean 1.31 m at day 60 and 1.60 m at day 75 (target 1.5 and 2), stem and tap root
  4.65 and 3.13 mm. Variants of the increment rule all left the pole bean at 1.27 to
  1.42 m at day 60: `Lp_r` × 4 or × 6 (1.38), a 0.4 floor on the factor (1.42), `ψ_tlp`
  −1.2 (1.38), `K_leaf` 15 (1.27), `s_f` 12 (1.28), `K_s` × 3 (1.35). Turning turgor off
  gave 1.59.
- **The clock without the root xylem gain or predawn thickening:** 69 of 96 checks,
  pole bean 1.46 m, basal stem 4.68 mm and tap root 3.15 mm: with the day's mean stress
  on thickening, the midday dip thinned every stem and root.
- **No Tuzet factor** (`tuzet_sensitivity` 0): rejected by species validation, which
  requires a positive slope; not pursued.

## Consequences
- Model 0.25.0, species schema 17; every golden hash moves.
- Done-whens (single seed):
  - day cycle (`ps-arch/tests/hydraulics.rs`, days 35 to 60, brightest third): predawn
    leaf potential −0.04 to −0.08 MPa against the root zone's −0.033; daily low −0.64
    to −0.82 MPa;
  - split pot (to day 60): in hours when the east roots are below 30% of their
    available water, 1.149 kg came from the west half and 0.186 kg from the east;
    over the run the roots gave 0.640 kg back to drier cells against 3.612 kg taken
    (hydraulic redistribution, emergent);
  - wilting (`ps-world/tests/wilting.rs`): lowest leaf potential in the dry-down −4.80
    MPa, turgor 0; turgor 1.0 within 6 hours of the watering.
- Validation on four-seed means: **96 checks, 71 pass, every M0 gate passes** (0.24.0:
  70). In: `wheat_height_spaced` 0.689 → 0.753 m, `density_leaf_area_per_plant` −30% →
  −36%. Out: `stem_share_peak` 0.357 → 0.346 (band from 0.35). Further from their
  bands: `split_pot_foraging` 1.495 → 1.248 (seeds 1.15 to 1.39) and
  `nitrate_patch_proliferation` +48.8% → +34.1%: the split pot's east half stays wetter,
  plausibly fed by the roots' redistribution, so the dry-soil rules (which read relative
  available water) act less.
- Headline numbers, 0.24.0 → 0.25.0: total dry mass at maturity 25.28 → 26.36 g, peak
  leaf area 0.175 → 0.191 m², pods 11 → 11.5, seed per plant 7.31 → 7.49 g, mean seed
  0.189 → 0.187 g, harvest index 0.435 → 0.422, basal stem 5.62 → 5.44 mm, tap root
  3.82 → 3.71 mm, root length at flowering 73.7 → 71.9 m. Drought from flowering: pods
  −74% → −76%, root:shoot +36% → +31%, shoot mass −65% → −67% (still a miss). Shade
  total mass −49% → −46% (gate −50%), shade internode +28% → +32%, heat seeds −33% →
  −31%. Flowering (day 43.75) and maturity (day 79) unchanged. Pole bean 1.51 m at day
  60 and 2.04 m at day 75.
- At noon on day 50 a pole bean's potential runs from −0.057 MPa in the soil to −0.544
  at the stem base, −0.72 at the top and −1.26 at the lowest leaf: the roots' axial
  drop dominates; the stem takes about 0.18 MPa.
- The viewer still draws wilting from the plant's stress pair, whose first entry is now
  the stomatal factor; it should read the lens's `leaf_turgor` (a watered leaf's mean
  turgor at noon is about 0.2 to 0.3, so the drawing needs its own scale).
- Challenges, re-swept over world seeds 7 to 14 (docs/game/challenges.md): thirsty
  summer's gold 10 → 9 pods, and its test's bloom strategy waters every 4 days from day
  40 (over the seeds daily doses average 5.4 pods and 4-day doses 7.3; on seed 7 the
  daily schedule draws 2); the heat wave's medals 9 / 12 / 15 → 8 / 10 / 12 (sowing
  from day 20 makes 9 to 12 pods on seed 7); hungry soil unchanged. Pod abortion still
  reads the day's lowest root-zone FTSW, which in bloom under 6 L sits at 0.25 to 0.45
  most afternoons, against the 0.5 threshold. Moving pod abortion to the plant's water
  status (predawn turgor or the stomatal factor) is a candidate for R3c's calibration.
- Cost (one container, 4 cores, best of 3, two rounds, against the 0.24.0 binary):
  single_pot 3.5 to 4.2 → 5.6 to 6.4 ms per sim-day (+50%), plot_30_beans (30 days) 12.4
  to 13.0 → 16.2 to 17.6 ms (+30%). Callgrind on single_pot (one thread): the network is
  27% of instructions; each root cell's retention potential (two `libm::pow` an hour)
  9%, rebuilding the edge list 6%, the tree solve 2.4%. The pot is over its 4 ms budget
  at the detailed level; R3c's simplified level is the plan for plots and fast-forward,
  and the performance thread can take the bit-identical savings (cache each cell's
  potential when its water has not changed, keep the edge list between hours).
- Open: no capacitance, cavitation or stomatal memory (card's known limits); R2b's
  dry-soil rules still read relative available water; wheat's values are the bean's;
  R3c adds the simplified level and switching.
