# Review: plant-to-plant variation within a pure line (PR #80, decision 0027)

Reviewed at commit 7fe4480 on `claude/project-thread-huhdoy` against `origin/main` (model 0.9.0).
Reviewer: Fable review thread. Date: 2026-11-05.

## Verdict

**Approve.** The diagnosis is right (the plants were identical by construction: one emergence
hour, one azimuth, one seed mass, exact marks, uniform soil), the sources added are the ones
the literature ranks first (emergence time, seed size, microsite, development noise), every
draw is keyed to the plant's or soil's own entity so a plant does not change when its
neighbours do, and the done-when numbers reproduce here: in the 30-bean plot the final mass
CV is 16%, pods 5 to 13 (CV 25%), seed 28%, height 5%. Genetics correctly stays in M3. The
model card is complete, every parameter has a unit and a source, and the "~" marks are honest.

## What I checked

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| clippy, sim and CLI crates, `-D warnings` | clean (CI covers the Bevy crates) |
| `cargo test` on the sim and CLI crates | all pass, including the five new tests and `counts_hold_across_world_seeds` |
| `scripts/check_deps.py`, `scripts/check_determinism.py` | pass (ps-world now depends on ps-rng, allowed) |
| golden hashes (pots 90 d, plot 30 d) | match |
| `ps-cli validate` | 87 checks: 64 pass, 23 Info misses, 0 failing; `ndfa_low_n_bed` passes at 0.53; every M0 gate passes |
| plot spread | recomputed from `plot_30_beans` daily.csv: mass CV 13% (d20), 11% (d45), 16% (final); pods 8.9 ± 25%, 5 to 13; seed 6.2 g ± 28%; height 5% |

## Science values

All reasonable for a first set, and the sources say what they are:

- Seed mass CV 0.12 (bean) and 0.15 (wheat): within-lot single-seed CVs are 10 to 15% and
  15 to 20%. Fine.
- Emergence CV 0.15 on hydrothermal time plus 12 °C·d per cm (bean), 10.2 (CERES wheat):
  gives a 7-to-13-day emergence window for the plot, about a day wider than a warm garden
  sowing shows (most of a lot comes up over 3 to 5 days). If it looks too spread in the
  viewer, 0.10 is the number to try; it is not worth touching before the multi-seed harness.
- The `f^0.5` seed-to-unifoliate rule is a sensible partial effect. Note that the seed
  factor scales reserve, fixed tissue and seed N but not the embryo's root axis; fine.
- Leaf size CV 0.1 and divergence SD 10°: right order. The divergence scatter also breaks
  the leaf fences the WP23 review found for wheat (185° plus noise), which is a bonus.
- Soil N CV 0.25 on a 30 cm lattice: conservative against field surveys (30 to 60%), and the
  lognormal with mean 1 keeps the scenario's mean soil. Good.

## Findings, none blocking

1. **Soil patch field.** The construction is correct: bilinear weights on lattice normals,
   renormalised by `sqrt(Σ w²)` so every cell has unit variance, then `exp(σG − σ²/2)` with
   `σ² = ln(1 + cv²)` for mean 1. Two notes. The renormalisation makes the field's variance
   uniform but its correlation length a little shorter than the lattice at cell centres
   between nodes; harmless. And because the lattice draws come from one stream in lattice
   order, the field changes when the plot's size changes, which the doc says; a per-node key
   (soil, ix, iy) would make a patch stay put when a plot is enlarged. Worth doing when the
   viewer lets a player resize a bed, not now.
2. **Entity keys use three conventions.** Organ streams key `(entity << 32) | organ`, the
   individual keys `(entity << 32) | 0xFFFF_FFFF`, sowing scatter keys the raw plant id, and
   the soil field keys `SOIL_ENTITY | soil`. They cannot collide because the purposes
   differ, but `random-numbers.md` should list the four so the next purpose does not land on
   one by accident.
3. **`leaf_scatter` keys by "the next organ id"** (`arena.len()`). Unique per leaf today, but
   it depends on the leaf being allocated right after the call; if an internode is ever
   created in between, the draw silently moves to another organ's id (still unique, still
   deterministic, but the comment would be wrong). Key by the node instead, which the caller
   has.
4. **The stake miss (1 in 16).** Agree it is the fixed 6 cm sweep's problem and the growing
   sweep is the fix. Two more: a gardener sows a pole bean at the pole, so sowing scatter
   should not move a seed away from a support it was sown within 5 cm of (clamp the scatter
   toward the support, or let a sowing carry `at_support: Some(id)`); and the row scenarios
   with strings now have plants 1 to 2.5 cm off the string line, which is realistic and will
   cost a little height through the helix. Fine to leave both for the climbing follow-up.
5. **`counts_hold_across_world_seeds`**: the single-seed band widened to 4 to 30 pods and the
   §8.2 band moved to the 8-seed mean. Right, and consistent with decision 0026. The
   single-seed band is now wide enough that it only catches a broken plant, which is what it
   is for.
6. **Height CV 5%** is low against stands (about 10%): internode potentials are still
   deterministic, and the emergence spread is the only thing moving height. A small CV on
   the internode length potential (keyed per internode, like leaf size) would be the matching
   term. Later.
7. **The gap to field CVs (16% against 20 to 40% at harvest)** is correctly attributed to
   weak size-asymmetric light competition. Two of the WP21 follow-ups (bud outgrowth and leaf
   size responding to R:FR) are what would widen it, and the light model's within-cell
   "who is on top" limit also damps the advantage of the taller plant. All logged elsewhere.

## For the record

- Model 0.10.0, species schema 7 (`variation` group), scenario `variation: FieldVariation`
  defaulting to `GARDEN`; purposes 15 to 18. Goldens regenerated.
- Validation on seed 7: 63 → 64 passes; only `ndfa_low_n_bed` changes (MISS → pass).
- The 4-seed means and any re-tuning wait for the multi-seed harness (decision 0026).
