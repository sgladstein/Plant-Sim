# Spec R7: nitrogen moving along the stem

Status: spec, written 2026-11-05 (owner's order: R5 soil, R6 branching, then this). The
owner asked for a simple and a detailed level. Nothing is built. Tags as in the review
([M], [F], [K, to verify], [I]).

## 1. Why

Organ N is real state since WP24, but inside the plant N is one pool: uptake and mining
add to it, and growing organs top up from it each hour (nitrogen card steps 12 to 15)
[F]. So N reaches a growing leaf at the top the same hour a root takes it up, and a
branch can't run short on its own. Real plants move nitrate up in the xylem with the
transpiration stream and move amino acids in the phloem with the sugar, from old and
senescing leaves to growing organs and seeds [K]. A delay and a distance matter: a
shaded branch with few transpiring leaves gets less fresh nitrate, and seeds far from
senescing leaves fill a little later.

Game value is mostly a **nitrogen lens that shows the flow** (where nitrate goes, which
leaves export, where it piles up), and local yellowing on a starved branch. Whole-plant
growth changes little, which is why it comes last.

## 2. Model (detailed level)

### 2.1 State
- Each organ splits its N into structural (as today, `N_i`) and **mobile** N `M_i`
  (nitrate and amino acids in transit), new column `mobile_n: Vec<GramsN>`.
- The plant pool is gone at this level: `pool = Σ M_i` is a derived value for reports.

### 2.2 Xylem: nitrate up with water
- The day's uptake (daily step 0, unchanged) lands in each root organ's `M_i` in
  proportion to its uptake by cell (each organ's root length in each cell).
- Each hour, after gas exchange, a share of the roots' mobile N moves to the
  transpiring leaves in proportion to each leaf's transpiration that hour,
  `F_leaf,i = x · Σ_roots M · E_i / E_total` (`x` 0.3 per hour, ~). It needs only the
  per-leaf transpiration the sim already has, so R7 does not depend on R3. With R3's
  detailed hydraulics, the flux can follow the network's water fluxes instead.

### 2.3 Phloem: amino acids with the sugar
- One more diffusion step on the organ tree each hour, with the same O(n) backward-Euler
  solver as sugar transport (`ps-physio/src/allocation.rs::transport`), on the
  concentration `M_i / W_i`, with edge conductance `k_N = r_N · k_C` (the sugar edge
  conductance times a ratio, ~).
- Mining (step 15) moves leaf N into that leaf's `M_i` instead of the pool, so
  remobilized N travels to seeds through the tree.

### 2.4 Use
- **Top-up (step 12)** reads the organ's own `M_i` plus what reaches it, instead of the
  pool. A growing organ that finds too little mobile N grows less, as the N gate does
  now.
- **Fixation** adds to the nodule's `M_i`, so nodule N enters at the roots.

## 3. Simple level

Today's pool model. Both levels run on the same R3c switch, `SetFidelity`, so a plant's
hydraulics and nitrogen are always at the same level, and the owner's "follow zoom and
speed" rule covers both.

Switching:
- **detailed → simple:** `pool = Σ M_i`, each `M_i` set to 0. Exact.
- **simple → detailed:** the pool is spread to organs in proportion to the growing
  organs' deficits (today's top-up rule), the rest to the root organs. Exact in total.

Both directions conserve N exactly, so the N ledger can't break across a switch.

## 4. Step order

A new hourly step between today's 10 (sugar transport) and 12 (growth): xylem delivery,
then phloem diffusion of mobile N. The step 12 top-up then reads organ-local N. That is a
§3.2 change: decision record, model bump, goldens.

## 5. Parameters

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| `xylem_n_release` (`x`) | 0.3 | h⁻¹ | ~ |
| `phloem_n_ratio` (`r_N`) | 1 | - | ~ [K, to verify: amino acid vs sucrose transport in phloem] |

## 6. Done when

- **Fidelity report** (R3c): total N uptake, seed N and yield within the report's rule
  (5% or half the seed spread) between levels, and no bias.
- **Local shortage:** in a fixture where one branch's leaves are shaded to a low
  transpiration share, that branch's young leaves have measurably lower N per area than
  the sunlit branch's, at the detailed level only.
- **Bottom-up senescence** still happens in seed fill (the WP24 test passes at both
  levels).
- The N ledger closes every day, including across a switch.

## 7. Cost and risks

- One more tree solve per plant per hour plus a leaf loop: about the cost of today's
  sugar transport [I], so +5 to 15% per plant-day at the detailed level and nothing at
  the simple level. Performance should measure it.
- If the fidelity report shows the levels disagree in N-short plots (patchy soil), the
  simple level may need a small delay term (a one-day lag on the pool). Decide from the
  report.
- **Viewer/UI:** a nitrogen flow lens (per-organ N import and export, like WP26's
  carbon import data) and an N column in the inspector.
- **Model cards:** nitrogen (transport, two levels), carbon-allocation (shared solver),
  world-and-schedule (the level switch covers N).

## 8. Tests

- Xylem delivery conserves N and splits by transpiration share.
- Phloem diffusion of N conserves N at 1e-12 and is monotone (no negative `M_i`).
- A switch in each direction conserves N exactly and replays bit for bit.
- The local-shortage fixture of §6.
