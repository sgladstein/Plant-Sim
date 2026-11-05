# Spec R2: root system (length, fine roots, uptake, plasticity)

Status: spec, written 2026-11-05 after the owner chose R1, R2 and R3 (review:
[README.md](README.md)). Nothing is built. Tags as in the review ([M], [F],
[K, to verify], [I]).

## 1. Why

- The root system is far too short early: 1.0 m at day 20 and 2.7 m at day 30 in
  single_pot, against 120 to 170 m of basal plus hypocotyl-borne root at 21 days in sand
  (Miller et al. 2003, quoted in the OpenSimRoot bean file, whose author calls it
  "rather a lot") [M, F]. The first lateral cohorts only appear between days 20 and 30
  [M].
- There are no second-order (fine) laterals. These are 0.1 mm across, about 1,350 m/g,
  and add length, though in bean pots and fields first-order laterals carry most of it
  (second-order share 0.1 to 0.4, [numbers](numbers/README.md)) [F].
- The nitrogen uptake works around the missing length with a 25× capacity factor and a
  4 cm reach (nitrogen card, issue #58). A constant factor is wrong in time: too little
  root early, too much late.
- Roots sense nothing but temperature. The design plan's "cells at wilting point slow
  elongation" (§3.7) isn't implemented [M, `ps-arch/src/roots.rs::root_elongation`].

## 2. Packages and order

| Package | Contains | Size | Depends on |
| --- | --- | --- | --- |
| R2a Length and uptake | Early vigor (lateral timing and rates), second-order cohorts, uptake from real root length for water and N (drop the 25× factor and the 4 cm reach, closes #58) | M | R1a (root carbon is shared with thickening) |
| R2b Plasticity | Dry-soil slowing, xerobranching, lateral proliferation in wet and N-rich cells, hydropatterning of lateral direction, hydrotropism | M | R2a |

Compaction (needs soil strength state) and waterlogging (needs slower drainage, since
the bucket cascade empties a saturated cell within the hour) go to R5 with the soil
work. Each package bumps the model minor version and regenerates goldens. No §3.2 step
order change: root elongation stays daily step 5 and reads the soil state it already
has.

## 3. R2a: length and uptake

### 3.1 Early vigor
Compare each root type's elongation, branching start and lateral spacing with the
OpenSimRoot bean set (archive roots note §1 tables) and fix what differs. Likely causes
to check, in this order:
1. Lateral cohorts form only behind an apical unbranched zone and only after a parent
   segment completes. Check that the zone length and the segment expansion duration
   match OpenSimRoot's lateral emergence (laterals on the tap root within days of
   germination [K, to verify]).
2. Basal and hypocotyl-borne root rates (OpenSimRoot 2.4 to 4 cm/day for basals) [F].
3. Early root carbon: the root priority `0.6 + 0.4(1 − FTSW)` against the 20% of
   allocation OpenSimRoot gives roots after day 5 [F]. If the root share in the first
   month stays far below 20% after 1 and 2, raise the base priority; record it.

Target: a root length time course inside the band of §5, tuned on four-seed means.

### 3.2 Second-order laterals as cohorts
One more cohort level, using the existing cohort machinery:
- Each first-order cohort organ gets one child **fine-root cohort** once it completes,
  with multiplicity `m₂ = m₁ · L₁ / s₂` (first-order length over the second-order
  spacing) and per-root length `ℓ₂` from the fine-lateral type.
- Its length is spread over the soil cells its parent passes through, in proportion to
  the parent's length per cell (the parent's cells are already known), so it adds root
  length density where the first-order laterals are without new geometry.
- It is an organ like any other: carbon from the tree, construction cost, N, death,
  maintenance. Its diameter (0.1 mm) sets its carbon per metre:
  `ρ_tissue · π (d/2)² · c_f`, with OpenSimRoot's tissue density 0.094 g/cm³ [F].
- Cost: one organ per first-order cohort, so about +35% root organs at maturity (242
  cohorts today) [M, I]. A per-cell fine-root field (no organs) is cheaper, but it would
  need its own carbon and N bookkeeping outside the tree, so cohorts are the better fit.

New root type in the species files: `fine_lateral` (diameter 0.1 mm, spacing, length
cap, elongation rate) from OpenSimRoot [F].

### 3.3 Uptake from real root length
- **Water.** Today `min(k · L/V · (W − W_p), u_max · L, available)` per cell. With 10 to
  100 times more `L` early, keep `u_max` (already the CERES value of 0.03 cm³ of water
  per cm of root per day), re-fit `k` on seed means, and add saturation in root length
  density, `L/V ÷ (L/V + 0.5 cm cm⁻³)` [F, archive roots note §4; the saturating form is
  an inference there].
- **Nitrogen.** Remove the 25× capacity factor and the 4 cm reach: uptake capacity per
  cell from the cell's own root length, with CROPGRO's RTNO3 per unit length (decision
  0020 second review round names it) [F].
- Keep the joint-uptake rule (start-of-hour state, the same scale factor for an
  over-requested cell), so plant order still doesn't matter.

### 3.4 Done when (R2a)
- Root length per plant inside the §5 bands on four-seed means; the first lateral
  laterals on the tap root appear 3 to 5 days after germination (Vieira 2008; Arthikala
  2018 [F]). The day-21 gap is mostly first-order laterals on basal and adventitious
  axes, so tune lateral density, length and timing first [I].
- `root_shoot_seed_fill` passes (with R1).
- The 25× factor and the root reach are gone; plot Ndfa moves from 18% toward the 30 to
  60% band, and `ndfa_low_n_bed` still passes.
- Every M0 gate passes; the water ledger closes at 1e-9.

## 4. R2b: plasticity

Every rule reads the soil cell a root tip or segment is in, at the daily root step. No
new random draws; ties are broken in a fixed order, so determinism holds.

| Response | Rule | Source |
| --- | --- | --- |
| Dry soil slows elongation | Tip elongation × `f_w = clamp((θ_rel − θ_stop)/(θ_full − θ_stop), 0, 1)` with `θ_rel` the cell's relative available water, `θ_stop` 0 and `θ_full` 0.3, with a floor of 0.3 on `f_w` (root tips keep full elongation near −1.6 MPa). After R3, switch to water potential with the same floor | Sharp 2004 (maize), Yamaguchi 2010 (soybean) [F, [numbers](numbers/README.md)]; `θ_full` ~ |
| Xerobranching | No lateral cohort forms on a parent segment whose cell is below `θ_rel` 0.1 when the cohort would form, and it does not form later (the effect looked irreversible in barley) | Orman-Ligeza et al. 2018 [F] |
| Proliferation in N-rich cells | Lateral and fine-root cohort length × `1 + a_N · clamp((NO₃_cell − NO₃_mean)/NO₃_mean, 0, 2)`, `a_N` 1 (so up to 3×), carbon still limiting | Drew 1975 (barley laterals several-fold in a nitrate patch); Hodge 2004 [K, to verify] |
| Proliferation in wet cells | The same form with relative available water against the root-zone mean, `a_W` 0.5 | [K, to verify]; ~ |
| Hydropatterning | A lateral cohort's direction is chosen from the existing golden-angle candidates as the first one whose tip cell holds at least the median available water of the candidates, else today's rule | Bao et al. 2014 [K, to verify] |
| Hydrotropism | Direction rule gains `w_h · ĝ`, where `ĝ` is the unit gradient of relative available water over the six neighbour cells (zero if they are equal), `w_h` 0.05, well below the gravity term 0.2 · w_g | Dietrich et al. 2017 (MIZ1); weak in soil [K, to verify]; weight ~ |

Parameters go in the species files with these values and "~" sources until Science
research confirms the response sizes.

### 4.1 Done when (R2b)
- **Split pot** (new scenario: one half watered, the other left to dry from day 20):
  root length in the wet half at least 1.5× the dry half by day 45, and uptake from the
  wet half higher [I].
- **Nitrate patch** (new scenario: a fertilized 6 cm cube at 10 cm depth): lateral and
  fine-root length in the patch at least 1.5× the same cube in the control (expect 1.5 to
  4; Zhang 1999, Linkohr 2002 [F]; no legume data).
- **Uniform soil**: the root system is within 5% of R2a's (the plasticity rules do
  nothing where nothing differs).
- Drought from flowering: root:shoot still rises (existing checks), and root length
  shifts deeper (Info).
- Every M0 gate passes.

## 5. Validation targets to add (Science research to confirm)

| Id (proposed) | Scenario | Variable | Band | Source |
| --- | --- | --- | --- | --- |
| `root_length_day21` | single_pot | root length per plant | 20 to 150 m, aim 40 to 90 | Miller et al. 2003, 20 L sand pots, 40 to 90 m (low P) to 78 to 228 m (high P) [F]; Polania 2009, 35 to 62 m at 32 days [F] |
| `root_length_flowering` | single_pot | root length at R1 | 60 to 180 m | Trindade and Araújo 2014 (93 to 180 m in 4 kg pots); Araújo 2004 (75 to 116 m at 45 days) [F] |
| `fine_root_share` | single_pot | share of root length in second-order laterals at R1 | 0.1 to 0.4 | Miller 2003 (second-order negligible at 43 days in the field); soybean 39% (Jin 2013) [F]. Defined by root order, not diameter |
| `split_pot_foraging` | m2/split_pot (new) | wet:dry root length at day 45 | at least 1.5 | [I] |
| `nitrate_patch_proliferation` | m2/nitrate_patch (new) | patch:control root length | at least 1.5 | Zhang 1999, Linkohr 2002 [F]; Drew 1975 [K] |

Data asks for Science research: a bean root length time course in a pot or the field,
fine-root share of length, bean lateral emergence timing, and response sizes for nitrate
patch proliferation and xerobranching in dicots.

## 6. Cost

- R2a: about +35% root organs; allocation and the hourly loops scale with organ count
  (allocation was 0.65 ms per sim-day at 191 organs [F, docs/performance.md]). Dropping
  the 4 cm reach removes the reached-cell pass. Net expected +5 to +10% on single_pot
  and the plot [I]; the Performance thread should measure before the PR.
- R2b: a few cell reads per tip and per new cohort per day: negligible.
- Snapshots grow about 30% (more organs).
- Same cost at every scale: no simplified level needed.

## 7. Interactions and risks

- **More root length means more water and N uptake capacity**, so drought sets in later
  in a pot and the drought response checks may move. Re-tune after R2a on seed means.
- **The joint uptake in a plot** sees much denser roots; check over-requested cells
  stay rare (the shortfall log).
- **Viewer.** Fine-root cohorts are many and thin: draw them as a haze or as short
  lines, not as tubes, and keep the copy cap. Tell the viewer thread in the PR.
- **Genetics (M3).** Basal root angle, lateral density, fine-root length and the
  proliferation weights become breedable traits with real trade-offs (shallow basal
  roots for topsoil nutrients, deep for water; Lynch's bean work) [F for Strock 2017].
- **Model cards to update:** herb-architecture (roots), soil-and-water (uptake law),
  nitrogen (uptake capacity, no reach), grass (fine roots on crown roots), plus the
  decision record.

## 8. Tests

- Fine-root cohorts conserve carbon and N; their length lands only in cells their parent
  occupies.
- Uptake capacity is linear in root length below the saturation and saturates above it.
- Each plasticity rule: a one-root fixture with a dry or N-rich neighbour cell turns,
  branches or stops as the rule says, and does nothing in uniform soil.
- Determinism: the same plant twice is bit-identical; adding a plant to a plot doesn't
  change another plant's roots except through shared soil.
- The split-pot and nitrate-patch scenarios through `ps-cli validate`.
