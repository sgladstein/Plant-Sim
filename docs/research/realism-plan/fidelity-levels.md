# Spec: fidelity levels (detailed, simplified, aggregate)

Status: spec, written 2026-11-05 at the owner's request ("the multi-fidelity idea should
be clarified if it is left vague"). Nothing is built. It replaces the sketch in the
review's §4 and gathers the fidelity parts of R3c (§6 and §7 of
[r3-plant-hydraulics.md](r3-plant-hydraulics.md)) and R7 §3 in one place. Where this file
and those specs differ, this file wins. Tags as in the review ([M], [F], [K, to verify],
[I]).

## 1. The rule in one paragraph

The sim runs every plant at one of three levels:
- **Detailed**: every flow resolved organ by organ.
- **Simplified**: the same plants and organs, but water and nitrogen move through
  per-plant summaries.
- **Aggregate**: M4 fields and forests, where plants become patch cohorts.

The level is part of the sim state. It changes only through logged actions, so a replay
always gives the same result. In live play the viewer picks the level from what the
player is looking at and how fast time runs (the owner's choice: "follow zoom and speed
for now"). Headless runs take it from the scenario. Two levels never have to match bit
for bit, but they must give similar outcomes, and a test in CI proves that (§5).

## 2. What each level computes

Most processes run the same way at the detailed and simplified levels, because they are
cheap. Only two get a second form at the organ scale: plant hydraulics (R3) and nitrogen
transport (R7). The aggregate level is a different model (WP34), and every process
needs a stated aggregate form.

| Process | Detailed | Simplified | Aggregate (M4, per patch cohort) |
| --- | --- | --- | --- |
| Light | Per-leaf rays on the plot grid (today) | same | Two-leaf canopy (sunlit, shaded) with height-sorted layers, same gas-exchange code (§3.15) |
| Photosynthesis, stomata | Per leaf, Tuzet factor on that leaf's ψ | same code, per leaf, ψ from the plant summary | same code, per leaf class |
| Carbon allocation, sugar transport | Organ tree solve (today) | same | Organ classes (leaf, stem, root by layer, seed, storage), priority rule on class demands |
| R1 thickening, storage, exudation | Per organ | same | Stem and root class mass carry a secondary fraction; one storage pool per class; exudation as a fraction of root respiration |
| R2 root length, fine roots, plasticity | Per organ and cohort | same | Root mass by soil layer × specific root length; new root mass split across layers by the same wet and N-rich weights |
| R3 hydraulics | Network solve every hour (R3b) | Couvreur summary: `K_rs`, `SUF` per cell, `K_path` per leaf, one solve per day (R3c) | Per cohort: `K_rs` and `SUF` per layer from root length, `ψ_leaf` per leaf class (§2.1) |
| R4 phototropism | Per new internode | same | none (lean has no cohort effect) |
| R5 soil water flow | Richards column solve per soil column | same (soil is shared, never per plant) | Same solver on one column per patch, coarser layers |
| R5 waterlogging, compaction | Per cell | same | Per layer, same thresholds |
| R6 branching, topping | Bud fate per axis | same | Branch count is a cohort statistic; topping and pruning act on the patch as a mass removal and a branching bonus |
| N uptake, fixation | Per cell from root length | same | Per layer from root length |
| R7 N inside the plant | Mobile N per organ, xylem and phloem steps | Today's single pool with top-up and mining | One pool per cohort, top-up by organ class |

So at the organ scale, "simplified" means **hydraulics and nitrogen transport only**.
Everything else is the same code at both levels, so it needs no equivalence test.

### 2.1 R3 at field scale (settle before R3 is built)

The aggregate hydraulics is the simplified level with organs replaced by layers. Per
cohort, once a day:
- `L_z`: root length in each soil layer z (from root mass by layer × specific root
  length).
- `K_rs = Σ_z L_z · k_eff,z`, where `k_eff` is an effective conductance per unit root
  length that folds in axial limits. It is fitted from organ-level runs, `K_rs / L` by
  depth.
- `SUF_z = L_z · k_eff,z / K_rs`.
- `K_shoot` per unit leaf area from `K_leaf` and the stem conductance (stem resistance is
  small, see the R3 numbers note).

Each hour:
- `ψ_seq = Σ_z SUF_z ψ_z`.
- `ψ_leaf,class = ψ_seq − E_class / K_plant,class` for the sunlit and shaded classes.
- The Tuzet factor and turgor use the same functions and parameters as the organ level.
- Uptake per layer uses Couvreur's compensation term with the same `K_comp`.

This has no state of its own, so promotion and demotion convert nothing for water. For
it to work, **R3 must be built with three things now**:
1. The Tuzet factor and turgor are plain functions of `(ψ_leaf, parameters)` in
   ps-physio, not tied to the organ arena, so a leaf class can call them.
2. The simplified level writes daily outputs per plant: `K_rs`, total root length, and
   `SUF` summed by soil layer. WP34 fits `k_eff,z` from these on the validation
   scenarios.
3. `K_comp` is a parameter (already in R3 after the numbers check), shared by all levels.

Nothing else in R3 changes. The aggregate form for R7 is today's pool per cohort.

## 3. When the level switches

### 3.1 Between detailed and simplified (organ scale, R3c and R7)

Hydraulics and nitrogen share one switch: a plant is entirely detailed or entirely
simplified.

| Situation | Level |
| --- | --- |
| Headless runs (ps-cli, validation, the M3 bench) | The scenario's `fidelity: Detailed \| Simplified \| Auto`. `Auto` is detailed for pots and plots up to `max_detailed` plants (8, ~), simplified above |
| Live play, plants in focus, speed at or below `detailed_speed_max` (24 sim h per real s, the default speed, ~) | Detailed, up to `max_detailed` plants |
| Live play, plants out of focus | Simplified |
| Speed above `detailed_speed_max` | Simplified for every plant |
| **Step buttons** (the "turn-based" steps: advance N hours as fast as possible, then pause) | Simplified for every plant during the step. The focused plants return to detailed when the step ends |
| **Plants in one experiment** (two-pot bench, experiment cards) | Always share one level, so a comparison is never confounded by the level. Focus on one plant of the experiment brings in all of them, up to `max_detailed`; beyond that the whole experiment stays simplified |
| A challenge file with `fidelity: Locked(level)` | That level; `SetFidelity` actions are refused (off by default) |

Focus means the plants whose bases are inside the camera's view at close range, nearest
first. Its exact geometry is a viewer setting, not sim state.

**Mechanics:**
- The viewer decides, and the session writes `SetFidelity { plants, level }` to the
  action log.
- The switch takes effect at the next hour boundary.
- To avoid flicker, the view and speed must be stable for 2 wall seconds before the
  viewer sends a switch. A step-button switch is sent at once, with the step.
- Seek and rewind (WP26) replay the logged switches like any action.

**State across the switch:**
- **Hydraulics:** nothing to convert, because neither level stores water in the plant.
  The water ledger can't break.
- **Nitrogen:**
  - Detailed → simplified: `pool = Σ mobile_n`, and each organ's `mobile_n` is set
    to 0.
  - Simplified → detailed: the pool goes to the growing organs in proportion to their
    deficits (today's top-up rule), and the rest to the root organs.

  Both directions are exact in total, and the N ledger closes across the switch at 1e-12.
- **Detailed-only diagnostics** (per-organ flows, per-organ ψ) are not state. After a
  switch to simplified, the lenses show the simplified estimates (`ψ` from `K_path`,
  flows from the daily solve scaled to the hour) with a "simplified" badge in the
  inspector, so a lens never goes blank.

### 3.2 Between the organ scale and aggregate (M4, WP36)

- **Demotion:** when the camera leaves a promoted patch, or when a field run starts.
  The plants' organs sum into the cohort's organ classes. Root mass by layer, N per
  class, storage, LAI and height carry over exactly. Branch count and the experiment
  history become cohort statistics.
- **Promotion:** when the camera comes close to a patch at or below
  `detailed_speed_max`. Plants are regenerated, as a plausible regeneration and not the
  true history (§3.15). A promoted plant starts at the simplified level and then follows
  §3.1.
  - The draws use ps-rng with `Purpose::Promote`, keyed by the patch id and the tick.
  - The regenerated plants hit the conserved class totals exactly; the generator
    rescales them to do so.
- **Mechanics:** both are logged actions (`Promote { patch }`, `Demote { patch }`) with
  the same 2-second stability rule, so replays match.
- **Authority:** a promoted patch is authoritative, so actions on its plants count, only
  if WP36's round-trip test passes. If it fails, promotion is display-only: the
  regenerated plants are drawn, and the cohort stays the authoritative state. That is
  the design plan's existing fallback.

## 4. Determinism and golden hashes

- **Source of truth is unchanged:** world seed, model version and action log. The level
  of each plant, and the patch or organ form of each area, is in the state and in
  `state_hash`.
- **Each level is bit-exact on its own** on one build and platform. Two levels are not
  bit-equal to each other; they are only similar (§5).
- **The viewer's rule is not sim state.** `max_detailed`, `detailed_speed_max`, the
  focus geometry and the 2-second rule live in ps-app. Changing them changes which
  actions a live session writes, not the sim, so it bumps no model version and changes
  no golden. A change to it is still recorded in a decision, because it changes play.
- **A replay needs the log's fidelity actions.** A replay that dropped them would run
  at the scenario's level and give a different (similar) harvest. Saves already carry
  the full log (WP26), so this holds.
- **Per-plant levels and parallelism:** plants at different levels cost different
  amounts. The fixed chunking and in-order combination stay as they are. Joint soil
  uptake reads start-of-hour state and combines plants in id order whatever their
  level, so one plant's level never changes another plant's numbers except through
  shared soil, as today.
- **Golden runs** (tests/golden/<platform>/):
  - Existing goldens run at their scenario's `Auto` level: single_pot and
    two_pot_sun_shade detailed, plot_30_beans simplified.
  - Add `single_pot_simplified` (the same pot at the other level).
  - Add `fidelity_switch`: single_pot with scripted `SetFidelity` actions at days 30
    and 60.
  - At M4, add a promote and demote golden.
  - A change to either level's equations bumps the model minor version and
    regenerates all of them, as today.
- **Session test (WP16, WP26):** a session that zooms in and out, steps and seeks
  replays headless to the same hash.

## 5. How "outcomes stay similar" is tested

A new `ps-cli fidelity` report, run in CI next to `validate`, failing the build like a
gate. It has two parts: organ-scale levels (R3c onward) and aggregate (WP34 onward).

### 5.1 Detailed against simplified
- **Scenarios:** single_pot, response/drought_from_flowering, the R2b split pot,
  plot_30_beans (forced detailed), m2/plot_low_n (patchy soil), m2/wheat_stand, and from
  R5 the waterlogged pot.
- **Outcomes on four-seed means:**
  - total and organ dry mass, seed yield, pods, height;
  - water use and water use efficiency;
  - root mass by depth;
  - N uptake, seed N;
  - flowering and maturity dates;
  - daily minimum leaf ψ.
- **Pass rule:** for each outcome, `|detailed − simplified|` is at most the larger of 5%
  and half the seed-to-seed range of the detailed level. Every validation gate passes at
  both levels.
- **Bias check:** across the scenario set, the mean signed difference in seed yield is
  within ±2%. Because the player picks the level by watching, a systematic gap would
  make watching (or not watching) a strategy.
- **Switch test:** detailed to day 30, simplified to day 60, detailed to the end.
  Ledgers close, outcomes pass the same rule, and the replay matches its hash.
- **Step test:** a season played with the step buttons (simplified throughout) against
  the same season at live speed with the plant in focus (detailed throughout). Same
  rule. This is the case the owner named: fast and turn-based play must reach about the
  same harvest.
- **Watched against unwatched:** a two-pot experiment where a scripted camera watches one
  pot. The two pots stay at one level (§3.1). The test checks the experiment rule
  directly.

### 5.2 Organ scale against aggregate (WP34, WP36)
- **WP34's done-when** stands: patch yield within 10% of the organ-level plot for the
  same scenario. This spec adds water use, the drought response (yield loss against the
  watered control) and N uptake to the same 10% rule, on plot_30_beans and
  m2/plot_low_n run as a patch.
- **WP36's done-when** stands: a demote and promote round trip preserves carbon and N
  per organ class, LAI, height and root mass per layer within 5%. With §3.2's rescaling
  the class totals are exact, so the 5% applies to what is regenerated (LAI, height,
  root mass by layer).
- **Bias check** as in §5.1, for promoted against unpromoted patches.

### 5.3 What happens on a failure
- **Simplified fails a hard case:** tune `K_comp` first, the knob the R3 numbers note
  names. If it still fails, add a lag term (R7 §7) or use the detailed level in fewer situations.
- **Simplified is biased:** that is a gameplay bug. Fix it before the release, or lock
  scored challenges to one level with the existing hook.
- **Aggregate fails:** M4 detail becomes display-only (§3.2), as the design plan already
  allows.

## 6. Fit with the design plan

- **§3.15 (scale and level of detail)** gains the simplified level between "plant and
  plot: full organ model" and "fields and forests". Its promotion text gains the logged
  actions and exact class totals.
- **§7 M4:**
  - WP34's aggregate hydraulics and N take their forms from §2.
  - WP34 consumes R3's per-plant `K_rs` and `SUF` outputs to fit `k_eff,z`.
  - WP36 uses the same switching mechanics as §3.1.
  - Neither WP's done-when changes; §5.2 only adds outcomes to WP34's.
- **Decision log** gets an entry when R3c lands: "fidelity follows zoom and speed; step
  buttons run simplified; experiments share one level".
- **Order:**
  - The detailed and simplified split arrives with R3c, and R7 reuses it.
  - §2.1's three requirements go into R3's PR.
  - The aggregate level arrives with WP34, and promotion with WP36.

## 7. Parameters

| Name | Value | Unit | Where | Source |
| --- | --- | --- | --- | --- |
| `max_detailed` | 8 | plants | viewer and `Auto` | ~, Performance to time R3b before fixing it |
| `detailed_speed_max` | 24 | sim h per real s | viewer | ~ (the default speed, 1 sim-day per second) |
| Switch stability | 2 | wall s | viewer | ~ |
| `K_comp` | `K_rs` | m³ s⁻¹ MPa⁻¹ | species | Couvreur 2012 [F] |
| `k_eff,z` | fitted | per unit root length | species, M4 | fitted at WP34 [I] |

## 8. Open points

- **Owner choice: what a promoted field plant can do** (§3.2). If WP36 passes, a player
  can walk into a field, prune or water single plants, and it counts. If it fails, those
  plants are only for looking. The design plan already makes this depend on the test, so
  no decision is needed now unless the owner wants actions on field plants regardless
  (which would need M4 to drop the 5% rule).
- **Everything else** is technical and set here with "~" values that Performance and the
  fidelity report will check.
