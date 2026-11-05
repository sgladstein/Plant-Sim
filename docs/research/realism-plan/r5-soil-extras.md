# Spec R5: soil extras (water flow, waterlogging, compaction)

Status: spec, written 2026-11-05. The owner chose the next round in this order: soil
extras (this file), then branches on branches ([r6-branching.md](r6-branching.md)), then
nitrogen moving along the stem ([r7-nitrogen-transport.md](r7-nitrogen-transport.md)).
Nothing is built. Tags as in the review ([M], [F], [K, to verify], [I]).

## 1. Why

- **Drainage is a bucket cascade** (soil-and-water card, step 2): every cell above field
  capacity passes its excess one cell down each hour, and the bottom cell drains at
  once. A soaked pot is back at field capacity within hours, where real soil takes one
  to three days (that slow drainage is what "field capacity" means) [K, to verify]. So
  overwatering is harmless, and water never moves up: no capillary rise, and nitrate
  below the roots can't come back (nitrogen card).
- **Roots never meet a lack of air.** Waterlogged soil has no air-filled pores, roots
  stop and die within days, and the plant wilts in wet soil. Common bean is sensitive to
  waterlogging [K, to verify]. In the sim a waterlogged cell lasts an hour.
- **Soil has no density or strength.** Hard soil slows roots and makes them thicker
  (Bengough et al. 2011 [K, to verify]), and gardeners dig, build raised beds and avoid
  walking on beds for that reason.

Game value: overwatering becomes the real beginner mistake it is; pot drainage, saucers,
digging, raised beds and paths become tools; the rhizobox glass shows roots bending
around a hard layer or dying back in a soggy bottom.

## 2. Packages and order

| Package | Contains | Size | Depends on |
| --- | --- | --- | --- |
| R5a Water flow | Darcy flow between cells from water potential (R3a) and unsaturated conductivity: slow drainage, capillary rise, a perched water table at a pot's bottom; pot drainage options | M | R3a (soil water potential) |
| R5b Waterlogging | Air-filled porosity per cell, hypoxic roots stop, lose conductance and die; denitrification in waterlogged cells | S | R5a, R3b (so a waterlogged plant wilts through its hydraulics) |
| R5c Compaction | Bulk density per cell, penetration resistance, root slowing and thickening; dig, raised bed and compact actions | M | R2b (the root elongation multiplier chain), R3a |

R5a changes what hourly step 4 computes but not the §3.2 order. R5b and R5c add inputs
to existing steps. Each bumps the model minor version and regenerates goldens.

## 3. R5a: water flow

### 3.1 Model
Per column, vertical flux between neighbouring cells from Darcy's law on total head
(Richards' equation, one linearized step per hour):

`q_{i→i+1} = K_{i+½} · ((ψ_i − ψ_{i+1}) / Δz + 1)` (downward positive, head in m),

with `K(θ)` from Mualem–van Genuchten using R3a's anchored retention parameters (`θ_r`
0, and `n` 1.17 to 1.29 after the research check, [numbers](numbers/README.md)) and a
saturated conductivity `K_s` per texture. With `n` that low, Mualem's `K` drops by orders
of magnitude just below saturation, so use the air-entry form of Ippisch et al. 2006
(a small air-entry head, ~ 2 cm) [K, to verify], and check the loam drainage target with
it before tuning `K_s`.

- **Solver:** backward Euler on each column with `K` taken at the start of the hour and
  the retention curve linearized there (one Picard step): a tridiagonal system, solved
  by the Thomas algorithm, O(cells) per column. Written in flux form, so water is
  conserved exactly; the hour is split into sub-steps only where a cell would pass
  saturation or residual water (bounded, so the cost stays fixed per hour).
- **Lateral flow** keeps today's diffusion step (decision 0019) for now.
- **Infiltration** stays top-down filling, but only as fast as the top cell's `K_s`
  allows; the rest ponds for the next hour (a small surface store), then runs off.
- **Bottom boundary:**
  - plot: free drainage (unit gradient), as today's deep layer that never returns water;
  - pot with holes: a seepage face. Water leaves only when the bottom cell is at
    saturation, so a pot holds a perched wet layer at its bottom, as real pots do
    [K, to verify];
  - pot without holes: no outflow;
  - pot in a saucer: outflow fills the saucer (a store up to its depth), and the saucer
    feeds the bottom cell back while it holds water.
- Scenario pots get `drainage: Holes | None | Saucer(depth_mm)` (default `Holes`).

### 3.2 Parameters

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| `K_s` loam, sand, clay | 0.25, 7.1, 0.05 | m day⁻¹ | Carsel and Parrish 1988 [K, to verify] |
| Mualem pore connectivity `l` | 0.5 | - | Mualem 1976 [K] |
| `n`, `α` per texture | as R3a (`θ_r` 0) | - | as R3a |
| Air-entry head `h_e` | 2 | cm | ~ (Ippisch et al. 2006 form) |
| Pond store before runoff | 5 | mm | ~ |

### 3.3 Done when (R5a)
- A loam column wetted to saturation drains to within 10% of field capacity in 1 to 3
  days (not hours) [K, to verify].
- With a water table at the bottom of a sand column, water rises against gravity to a
  height inside the analytic steady-state range.
- A pot with holes keeps its bottom cell near saturation after a heavy watering for at
  least a day; a pot in a full saucer keeps it wet.
- Water ledger closes at 1e-9 every hour, including the saucer and pond stores.
- The M0 gates pass after re-tuning; the drought scenarios still dry the root zone.

### 3.4 Cost
Pots: trivial. Plots: today's hydrology is 11 ms per sim-day for a 3 × 3 m plot (72,000
cells), a quarter of the plot budget [F, docs/performance.md]. A tridiagonal solve per
column per hour is about 24 × 72,000 × 20 flops a day, which may double that [I].
Performance should time a prototype before the PR. If it is too slow, two options
change results and need the owner: coarser vertical cells below the root zone, or the
flow step every two hours in plots.

## 4. R5b: waterlogging

- **Air-filled porosity** per cell, `a = θ_s − θ`. A cell with `a < a_crit` is
  hypoxic; `a_crit` 0.10 m³ m⁻³, the classic threshold for root function
  [K, to verify].
- **Roots in a hypoxic cell:** tips stop elongating (R2b's multiplier chain gains
  `f_O2 = 0`); radial conductivity `Lp_r` falls by `f_Lp` (0.3, ~) within hours, so the
  plant wilts in wet soil through its hydraulics (R3b) [K, to verify: anoxia closes
  aquaporins]; N uptake from the cell × 0.2 (~). Each root organ counts hypoxic hours;
  past `t_die` (48 h for fine roots, 120 h for axes, ~) it dies (litter, as today's
  death).
- **Recovery:** new laterals form on surviving axes once the cell drains (R2b rules).
- **Denitrification:** in a hypoxic cell, `ΔNO₃ = −k_dn · f_T · NO₃` per hour (`k_dn`
  0.02 h⁻¹, ~), booked as a new nitrogen ledger line (N lost to air). Overwatering then
  also wastes fertilizer [K, to verify].
- **Bean vs wheat:** the thresholds are species parameters; wheat tolerates
  waterlogging somewhat better than bean [K, to verify].

Done when: a pot without holes watered to saturation and kept wet for 4 days loses most
of its fine roots in the wet layer, wilts at midday while its soil is wet, and recovers
slowly after the water drains or is withheld; the same plant in a pot with holes does
not. Nitrogen ledger closes with the denitrification line.

## 5. R5c: compaction

- **State:** bulk density `BD` per cell (kg m⁻³), default per texture (the nitrogen card
  already lists 1300, 1550, 1250) [F]. Scenarios can set layers (a plough pan) or a
  raised bed (loose soil).
- **Effects on water:** `θ_s = 1 − BD / 2650` (R3a's `n` and `α` are re-solved for the
  cell's `θ_s`, keeping FC and WP; closed form, at a BD change only) and `K_s × (BD_ref / BD)^m` (~), so
  compacted soil holds less air and drains slower (feeds R5a and R5b).
- **Penetration resistance:** `Q(BD, ψ)` from an empirical pedotransfer function (for
  example Whalley et al. 2007, from bulk density and water potential [K, to verify]).
  Dry, dense soil is hardest.
- **Roots:** elongation × `f_Q = 1 / (1 + Q / Q_half)`, with `Q_half` about 1 MPa
  (elongation roughly halves near 1 MPa and nearly stops at 2 to 3 MPa) [K, to verify];
  a tip in hard soil grows thicker, `d × (1 + c_Q · Q)` (~), so it costs more carbon per
  cm; laterals form more densely behind an impeded tip (~) [K, to verify].
- **Actions:** `Dig { soil, region, depth }` loosens to `BD_loose` (0.85 × default, ~);
  `Compact { soil, region }` (a trampled path) raises to 1.15 × default (~). The viewer
  offers them as garden tools; both go in the action log.
- **Settling:** loosened soil settles back toward its default with rain, a fraction per
  wetting (~), so digging is not permanent [K, to verify].

Done when: a plot with a compacted layer at 15 cm shows fewer and thicker roots below it
and more above it; a raised bed (loose 30 cm) grows deeper roots than flat compacted
soil; digging a strip mid-run changes root growth only there; determinism and ledgers
hold.

## 6. Validation targets to add

| Id (proposed) | Scenario | Variable | Band | Source |
| --- | --- | --- | --- | --- |
| `loam_drain_days` | soil fixture | days from saturation to FC + 10% | 1 to 3 | [K, to verify] |
| `waterlogging_root_loss` | m2/waterlogged_pot (new) | fine-root length lost in the wet layer after 4 days | at least 50% | [K, to verify] |
| `waterlogging_yield` | m2/waterlogged_pot | seed yield against control | a published bean waterlogging response | data gap |
| `pan_root_shift` | m2/plough_pan (new) | root length above / below the pan, against no pan | above up, below down | [K, to verify] |

Data asks for Science research: drainage times for loam pots, bean waterlogging
responses (root death timing, yield), a penetration resistance pedotransfer function
with parameters for loam, sand and clay, and root elongation against penetration
resistance for a dicot.

## 7. Interactions and risks

- **Drought scenarios dry more slowly** once drainage is slow and capillary rise feeds
  the root zone; the drought responses must be re-checked and the scenarios may need a
  later start or less water.
- **Nitrate leaching** slows with slower drainage; the N checks (Ndfa, low-N bed) move.
- **Plot speed** (R5a): see §3.4.
- **Viewer:** the soil-water lens gains a water potential and an air-filled porosity
  view, hypoxic roots can be drawn browned, and the garden tools gain Dig and Compact.
- **Model cards:** soil-and-water (flow, boundaries, compaction), nitrogen
  (denitrification), herb-architecture (root responses), world-and-schedule (new
  actions, pot drainage).

## 8. Tests

- Water flow: closure at 1e-12 per hour per column; a uniform column at equilibrium
  stays put; the analytic steady infiltration and capillary rise cases.
- Waterlogging: a root fixture in a saturated cell stops, then dies at `t_die`.
- Compaction: `f_Q` monotone in BD and in dryness; Dig and Compact replay bit for bit.
