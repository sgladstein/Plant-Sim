# 0027: Plant-to-plant variation within a pure line

- **Date:** 2026-11-05
- **Status:** accepted

## Context
The owner saw multi-plant scenes in the viewer where every plant was perfectly spaced
and grew identically. On model 0.9.0 the 30-bean plot showed that this was built in, not
a drawing problem: every bean emerged in the same hour, every main stem faced the same
compass direction (`emerge` used a fixed azimuth of 90°), every seed weighed the same,
every leaf at a node had the same potential size, every seed sat exactly on its mark at
3 cm, and the plot soil started uniform. Per-plant dry mass had a coefficient of
variation (CV) of 0.1% at day 20, 3.8% at day 45 and 9% at day 69, and height 1%. The
only differences came from pod-set and bud-fate draws late in the season and from edge
plants getting more light. `Purpose::Germination` and `Purpose::Phyllotaxis` were
defined but never drawn, and all scenarios sow `SeedChoice::Default`, a single genotype.

In a real stand of one cultivar, plants differ from the start. The main causes, roughly
in order of effect:

1. Emergence time. Seeds of one lot germinate over several days. Hydrothermal-time
   models describe this by giving each seed its own threshold, normally distributed in
   the lot (Gummerson 1986; Bradford 2002). Deeper seeds take longer: CERES-Wheat adds
   10.2 °C·day per cm of depth (Ritchie 1991). Seedlings that come up early stay ahead,
   because competition for light favours the larger plant (Ross and Harper 1972; Weiner
   and Thomas 1986).
2. Seed size. Single seeds of one lot vary by about 10 to 15% in mass, and bigger seeds
   give bigger seedlings. In beans the unifoliate leaves are already formed in the embryo.
3. Microsite. Hand sowing misses the mark by a centimetre or two, and plot soil is
   patchy. Plant-scale soil nitrate commonly varies by 30 to 60% (CV) in field surveys.
4. Development noise. Leaf size and divergence angle scatter about the plant's
   programme from node to node.
5. Genetics. A bush-bean cultivar is an inbred pure line, so within-cultivar genetic
   variance is close to zero. That belongs to M3 breeding (§3.12), not here.

## Decision
Add non-heritable variation at the sources above. Every draw goes through ps-rng and is
keyed by (world seed, the plant's or soil's own entity, tick, purpose). A plant's draws
therefore do not change when other plants are added or removed (the implementation
thread asked for this).

- Species schema 7 adds a `variation` group, with value, unit and source for each field:
  `seed_mass_cv`, `seed_leaf_exponent`, `emergence_cv`, `emergence_per_depth`,
  `emergence_reference_depth`, `leaf_size_cv`, `divergence_sd`.
- `ps_arch::Individual` is drawn once in `Plant::sow` and stored on the plant (for the
  inspector and snapshots):
  - seed factor `f = 1 + cv·z` with `z ~ N(0,1)` held to ±2.5 (`Purpose::SeedSize`). It
    scales the sown seed's mass, which means its reserve, its fixed tissue and its seed
    N. It also scales the unifoliate potential area by `f^seed_leaf_exponent`. The seed
    size the plant itself produces (`seed_potential`) is unchanged.
  - emergence threshold `(E + k·(depth − depth_ref))·(1 + cv·z)`, at least `E/4`
    (`Purpose::Germination`).
  - seedling azimuth uniform on [0, 2π) (`Purpose::Phyllotaxis`). The cotyledons and
    unifoliates turn with it. A grass already drew its first blade's azimuth (WP23)
    and keeps that draw.
- Each new node's divergence angle is the phyllotaxis plus `N(0, divergence_sd)` (keyed
  by the node). Each leaf's potential area is multiplied by `1 + leaf_size_cv·z` (keyed
  by the leaf's organ id, `Purpose::LeafSize`). This applies to grass blades too.
- The scenario gets an optional `variation` (`ps_world::FieldVariation`). Its default,
  `GARDEN`, is used when the field is absent:
  - sowing scatter of 1.0 cm SD per axis in position and 0.5 cm in depth, with depth
    kept at 1 cm or more (`Purpose::Sowing`, keyed by plant id). A point that leaves
    the soil falls back to the mark.
  - plot soil nitrogen (nitrate, ammonium, humus) times a lognormal column field with
    mean 1 and CV 0.25. The field is drawn on a 30 cm lattice, bilinearly interpolated
    and renormalized to unit variance (`Purpose::SoilPatch`, keyed by the soil). Pots
    are mixed soil and stay uniform.
  - `FieldVariation::NONE` turns all of it off.
- New purpose codes: SeedSize 15, LeafSize 16, SoilPatch 17, Sowing 18.
- Model 0.10.0. All goldens regenerated.

Sowing position scatter started at 1.5 cm. At that value, 2 of 4 pole beans in
`m2/pole_trellis` missed the stake 5 cm away and lodged, because the vine capture is a
fixed 6 cm sweep (decision 0021). At 1.0 cm (a seed dibbed at a mark), 1 pole bean in 16
misses over world seeds 7 to 10. That is plausible for a garden, where some seedlings
have to be guided to the pole. A sweep that grows with the free length of the searching
tip would make a miss rarer still. That is a climbing-model follow-up.

## Consequences
- In the 30-bean plot on seed 7, emergence now spreads over days 7 to 13. Per-plant
  dry mass CV is 13% at day 20, 11% at day 45 and 16% at harvest (it was 0.1%, 3.8% and
  about 10%). Pods per plant range from 5 to 13 (CV 25%), seed mass from 3.6 to 11.6 g
  (CV 28%), and height CV is 5%. Published monoculture stands often reach 20 to 40%
  biomass CV at harvest. Light competition here is only mildly size-asymmetric, so the
  gap between early and late plants widens less than in a field. See the WP21 density
  follow-ups (bud outgrowth and leaf size do not yet respond to R:FR).
- Validation (single seed 7): 64 of 87 pass, 0 failing the build. ndfa_low_n_bed now
  passes (0.53). Nothing was re-tuned. The seed-mean effects are read from the
  multi-seed harness (decision 0026) once it lands.
- `ps-arch` test `counts_hold_across_world_seeds` now checks the §8.2 pod band on the
  mean over 8 seeds. Each single seed only has to fall within a plausible 4 to 30 pods,
  because one lone plant may now carry 22.
- Not modelled: seed vigour beyond size (ageing, damage), seed-to-seed differences in
  germination water potential (only the thermal threshold varies), soil water-holding
  or compaction patches, and herbivory and disease.
