# Spec R3: plant hydraulics, with a detailed and a simplified level

Status: spec, written 2026-11-05 after the owner chose R1, R2 and R3 (review:
[README.md](README.md)). Owner decision on fidelity (2026-11-05): **the level follows
zoom and speed for now**, to be evaluated and changed later if it is a problem. Nothing
is built. Tags as in the review ([M], [F], [K, to verify], [I]).

## 1. Why

- Water inside the plant is one step from soil to leaf: no resistance, no water
  potential, no delay. Stress is `min(1, FTSW/0.4)` over the root zone, at once
  (soil-and-water card).
- There is no water potential in the soil either: water is held by volume between field
  capacity and wilting point.
- So wilting is not a physical state (WP20 draws it from the stress factor), uptake
  can't shift to wet soil through the plant's plumbing, and root traits breeders select
  in bean, like metaxylem size and axial conductance (Strock et al. 2020), can't matter
  [F].

R3 adds water potential in the soil and in every organ, flow through roots and stems,
stomata that close as leaf water potential falls, and a turgor value per leaf for
wilting. It comes in two levels with similar outcomes.

## 2. Packages and order

| Package | Contains | Size | Depends on |
| --- | --- | --- | --- |
| R3a Soil water potential | Retention curve per texture, anchored on today's field capacity and wilting point; a soil water-potential lens value | S | none |
| R3b Hydraulic network (detailed level) | Doussan-style network on the organ tree, stomata on leaf water potential (Tuzet), turgor-driven expansion and per-leaf turgor | L | WP27 (energy balance), R2a (real root length), R3a |
| R3c Simplified level, switching and the fidelity report | Couvreur-style reduced model from a daily network solve, the `SetFidelity` action and the zoom-and-speed rule, the fidelity report in CI | M | R3b |

R3b changes what hourly steps 5 to 7 do (§5), so it needs a decision record, a model
bump and a design plan §3.2 edit. R3a changes nothing the sim computes (it only adds a
derived value), so its goldens should stay unchanged; check that.

## 3. R3a: soil water potential

van Genuchten (1980) retention per texture:
`θ(ψ) = θ_r + (θ_s − θ_r) / [1 + (α|ψ|)^n]^(1 − 1/n)`.

The published texture tables (Carsel and Parrish 1988 [K, to verify]) don't put loam's
−33 kPa point at the sim's field capacity of 0.27. So **anchor the curve on the sim's
own values**: per texture, keep `θ_s` (saturation, already in the soil params) and a
literature `n`, and solve `α` and `θ_r` so that `θ(−33 kPa) = FC` and
`θ(−1500 kPa) = WP`. Water contents and every existing result stay as they are; only
ψ is new. Invert the curve per cell when ψ is needed (closed form).

Done when: ψ is −33 kPa at field capacity and −1.5 MPa at wilting point for each
texture to 1e-9, it is monotone in θ, and the goldens are unchanged.

## 4. R3b: the hydraulic network (detailed level)

### 4.1 Network
Nodes are the organs. Water potential ψ (MPa) includes gravity (`ψ_total = ψ_p + ρ g z`,
about 0.0098 MPa/m).

- **Axial edges** along the organ tree (parent to child):
  roots `K_x,type / l`; stems `K_s · A_x / l`, with the xylem area `A_x` a fraction of the
  stem cross-section, so R1a's thickening raises stem conductance (that is the point of
  the pipe model); petiole and leaf `K_leaf · area`.
- **Radial links** from each root segment or cohort to each soil cell it occupies:
  `k_r = Lp_r,type · 2π r L_c · f_dry(θ_rel)`, with `L_c` the organ's length in that
  cell (already known per cell) and `f_dry` a rhizosphere drying factor that falls to 0
  near the residual water content.
- **Leaves** are flux nodes: each takes its transpiration `E_i` (from gas exchange, this
  hour).

Each organ node obeys `Σ_j k_ij (ψ_j − ψ_i) + Σ_c k_r,ic (ψ_c − ψ_i) − E_i = 0`. Soil
cells are fixed potentials for the hour, so the matrix stays tree-structured: one
leaf-to-root elimination and one substitution, the same O(n) solve as today's sugar
transport (`ps-physio/src/allocation.rs::transport`) [M]. Reuse that solver.

Outputs per hour: ψ per organ, uptake per (organ, cell) `k_r,ic (ψ_c − ψ_i)`, and so
uptake per cell. Uptake into a dry cell can be negative: the root leaks water into dry
soil (hydraulic redistribution). That is real, and the water ledger stays closed.

### 4.2 Stomata and growth
- **Stomata (Tuzet et al. 2003 [K, to verify]):** the Medlyn slope is multiplied by
  `f_ψ = (1 + e^(s_f ψ_f)) / (1 + e^(s_f (ψ_f − ψ_leaf)))`, with `ψ_leaf` from the
  **previous hour's** solve. The one-hour lag removes the iteration between stomata and
  water potential. In this level `f_ψ` replaces the FTSW stress factor on stomata.
- **Expansion:** the expansion stress for a growing organ becomes a turgor term,
  `f_exp = clamp((ψ_i − ψ_tlp) / (ψ_0 − ψ_tlp), 0, 1)` with its own last-hour ψ.
- **Wilting:** each leaf's `turgor = clamp((ψ_leaf − ψ_tlp) / (ψ_0 − ψ_tlp), 0, 1)`
  goes in the snapshot (outside the hash, like WP26's lens data). The viewer draws
  wilting from it instead of the stress factor.
- **WP27 coupling:** the energy balance needs transpiration, and transpiration needs the
  stomata. With the lag, the hourly order is: stomatal factor from last hour's ψ, gas
  exchange plus energy balance (WP27), then the network solve with this hour's `E`.

### 4.3 Uptake in a plot
Each plant solves its own network against the start-of-hour soil potentials (as the
joint uptake reads the start-of-hour state today), plants in parallel. Where the plants'
uptakes from one cell exceed what it holds, all are scaled by the same factor and the
shortfall is logged, as today. Plant order does not matter.

### 4.4 Parameters (species and soil files)

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| van Genuchten `n` per texture | loam 1.56, sand 2.68, clay 1.09 | - | Carsel and Parrish 1988 [K, to verify] |
| `root_radial_conductivity` per root type (`Lp_r`) | ~1e-7 (fine) to ~2e-8 (thickened) | m s⁻¹ MPa⁻¹ | [K, to verify]; thickening (R1a) lowers it |
| `root_axial_conductance` per root type (`K_x`) | from metaxylem diameter by Hagen–Poiseuille | m⁴ s⁻¹ MPa⁻¹ | Strock et al. 2020 measured bean metaxylem [F]; values to extract |
| `stem_xylem_conductivity` (`K_s`), `xylem_area_fraction` | to set | kg m⁻¹ s⁻¹ MPa⁻¹, - | [K, to verify] |
| `leaf_hydraulic_conductance` (`K_leaf`) | about 5 to 15 | mmol m⁻² s⁻¹ MPa⁻¹ | [K, to verify] for bean |
| Tuzet `s_f`, `ψ_f` | about 3, −1.0 | MPa⁻¹, MPa | [K, to verify] |
| `turgor_loss_point` (`ψ_tlp`), `ψ_0` | about −1.0, −0.2 | MPa | [K, to verify] for bean |

`root_axial_conductance` and `root_radial_conductivity` become heritable traits for M3
(Strock et al. 2020: low-conductance bean lines conserved soil water for later in the
season) [F].

### 4.5 Done when (R3b)
- **Diurnal cycle:** on a sunny, well-watered day, predawn leaf ψ is within 0.1 MPa of
  the root-zone soil ψ, and midday leaf ψ falls to −0.4 to −0.8 MPa [K, to verify].
- **Wilting and recovery** (`data/scenarios/viewer/wilt_and_recover.ron`, WP20): during
  the dry-down, midday leaf ψ drops below `ψ_tlp` and turgor reaches 0; after watering,
  turgor is back above 0.5 within 6 sim-hours [K, to verify against bean rewatering
  data].
- **Compensation:** in the R2b split pot, more than half of the uptake comes from the wet
  half once the dry half falls below 30% FTSW.
- Every M0 gate passes; the water ledger closes at 1e-9 every hour.

## 5. Step order (R3b)

Today (§3.2): 5 water status (FTSW) → 6 gas exchange → 7 uptake (proportional split).
With R3b at the detailed level:

5. Plant water status: the stomatal and expansion factors from the **previous hour's**
   water potentials (the seed, at the first hour, starts at the soil ψ).
6. Gas exchange, with WP27's energy balance: assimilation, transpiration `E` per leaf.
7. Water uptake: the network solve with this hour's `E`; ψ per organ, uptake per cell.

The step numbers stay; their content changes. Decision record, §3.2 edit, model bump,
new goldens.

## 6. R3c: the simplified level and switching

### 6.1 Simplified level (Couvreur et al. 2012 [K, to verify])
Once a day, after the root rebuild (daily step 5), each plant runs one network solve
with uniform soil ψ and reduces it to:
- `K_rs`, the whole root system's conductance (collar flux per unit ψ drop);
- `SUF_c`, the standard uptake fraction of each soil cell (sums to 1);
- per leaf, a path conductance from the collar, `K_path,i`.

Each hour:
- `ψ_seq = Σ_c SUF_c ψ_c` (the root zone's effective soil potential);
- `ψ_collar = ψ_seq − E_total / K_rs`;
- uptake per cell `U_c = E_total · SUF_c + K_rs · SUF_c · (ψ_c − ψ_seq)` (the
  compensation term: wet cells give more than their share);
- `ψ_leaf,i = ψ_collar − E_i / K_path,i`.

That is O(cells with roots) per plant per hour, the cost of today's proportional uptake.
It is exact when the soil is equally wet everywhere, and close when it isn't. The
compensation term is where the hard cases (one wet patch, drip irrigation) differ most.
With no water storage (no capacitance) the level has no state of its own, so switching
in either direction converts nothing and can't break the water ledger.

### 6.2 Switching (owner's choice: follow zoom and speed)
- A per-plant state `HydraulicsLevel { Detailed, Simplified }`, in the snapshot and the
  hash.
- A new action `SetFidelity { plants, level }`, taking effect at the next hour boundary,
  written to the action log like every action. Replays re-issue it, so a replay matches
  the session (WP16's done-when stays true).
- **The viewer issues it** from a rule: detailed for the plants in the camera's focus
  (up to `max_detailed`, 8, ~) while the speed is at or below `detailed_speed_max` (1
  sim-day per second, ~); simplified otherwise. To avoid flicker, the view must be
  stable for 2 wall seconds before a switch is sent.
- **Headless runs** (ps-cli, validation, the M3 bench) take the level from the scenario:
  `fidelity: Detailed | Simplified | Auto`, where `Auto` is detailed for pots and plots
  up to `max_detailed` plants. Validation runs both levels (§7).
- **Challenge hook** (in case following zoom becomes a problem): a challenge file can
  set `fidelity: Locked(level)`, and then the session ignores `SetFidelity` actions.
  Cheap to add now; off by default.

### 6.3 Done when (R3c)
- The fidelity report (§7) passes on the scenario set.
- Switch test: detailed to day 30, simplified to day 60, detailed to the end; ledgers
  close at 1e-9, outcomes pass the report's rule, and the run's replay matches its hash.
- A session that zooms in and out replays to the same hash headless.

## 7. The fidelity report

A new `ps-cli fidelity` next to `validate`, run in CI like validation and failing the
build like a gate.
- **Scenarios:** single_pot, response/drought_from_flowering, the R2b split pot,
  plot_30_beans, m2/plot_low_n (patchy soil), m2/wheat_stand.
- **Outcomes on four-seed means:** total and organ dry mass, seed yield, pods, height,
  water use and water use efficiency, root mass by depth, N uptake, flowering and
  maturity dates, and the daily minimum leaf ψ.
- **Pass rule:** for every outcome, `|detailed − simplified|` is at most the larger of 5%
  and half the seed-to-seed range of the detailed level. Every validation gate passes at
  both levels.
- **Bias check** (because following zoom lets a player pick the level): across the
  scenario set, the simplified level must not be systematically higher or lower on seed
  yield (the mean signed difference within ±2%) [I]. If it is, a player could gain by
  watching closely or by not watching.
- Expected size (owner asked 2026-11-05): about 1 to 3% in typical runs, maybe 5 to 10%
  in the hard cases before the compensation term is tuned [I]; the report measures it.

## 8. Cost

- Detailed level: about one more tree solve per plant per hour, plus the radial links
  (one per organ and cell). Estimated about 1 ms per plant per sim-day at 830 organs,
  so +40 to 50% on single_pot (2.2 ms today) and about 8 ms per sim-day for 30 plants
  on 4 cores [I]. Performance should time a prototype solve before R3b's PR fixes
  `max_detailed`.
- Simplified level: one solve per plant per day plus O(cells) per hour, about today's
  uptake cost: a few percent [I].
- R3a: negligible.

## 9. Interactions and risks

- **WP27** must land first: both change how stomata and transpiration are set. The
  energy balance research already found g1 had to rise to 5.79 for the leaf temperature
  checks (re-tune); the Tuzet factor will need a calibration pass against those checks.
- **FTSW stress** stays in the simplified level's expansion term only if R3c can't match
  turgor otherwise; prefer the same turgor rule at both levels (from `ψ_leaf,i`).
- **Validation checks on drought** (drought_shoot_mass, drought_root_shoot, the seed
  size checks) will move. Re-tune on seed means after R3b.
- **Viewer:** wilting from per-leaf turgor (snapshot field), a soil and plant water
  potential lens, and the zoom-and-speed rule that sends `SetFidelity`. Viewer thread
  work, after R3b and R3c.
- **Model cards:** soil-and-water (retention, network uptake), gas-exchange (Tuzet
  factor), carbon-allocation (turgor expansion), a new plant-hydraulics card, and
  world-and-schedule (`SetFidelity`, levels in the hash).

## 10. Tests

- R3a: the anchors hold per texture; ψ(θ) is monotone and its inverse is exact.
- Network: a straight two-segment root in uniform soil matches the analytic solution;
  water is conserved per hour at 1e-12; zero transpiration gives ψ = soil ψ everywhere
  (plus gravity).
- Simplified level: in uniform soil it matches the detailed solve at 1e-9; in a two-cell
  wet/dry fixture it is within the report's tolerance.
- Switching: a `SetFidelity` mid-run replays bit for bit; the level is in the hash.
- Ninety-day: the §4.5 targets, through `ps-cli validate`, and the fidelity report.
