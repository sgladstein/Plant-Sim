# 0026: Validation reads the mean of four seed runs

- **Date:** 2026-11-05
- **Status:** accepted (review of WP25; a harness change, not a model change).

## Context
`ps-cli validate` ran each scenario once, at its world seed (7). The review of WP25 ran
`single_pot` on eight other seeds: the same bean, under the same weather and parameters,
ends between 21 and 39 g and between 8 and 17 pods, because a few dozen pod-set draws at
flowering decide its sink. Checks that flip when one draw flips (`heat_seeds_per_pod`,
`drought_shoot_mass` at WP25) were reading noise, and so was any tuning done on seed 7
alone.

## Decision
1. `targets.ron` lists `seed_offsets: [0, 1, 2, 3]`. Every scenario runs once at
   `world_seed + offset` for each offset (seeds 7 to 10 for most scenarios).
2. A target's value is the mean over the runs of each run's plant mean. At least half
   the runs must have a value. The logistic shape check is fitted per run and averaged.
3. A response compares the treatment's seed mean with the baseline's. Its reported range
   is the per-seed change, seed k against seed k.
4. Curves compare the mean over the runs at each point.
5. The report shows the mean, the standard deviation, the range and the number of runs;
   `results.csv` gains `seed_min`, `seed_max`, `seed_sd` and `seed_n`. Only the first run (offset 0) writes `runs/<scenario>/` for the plots.
6. Runs go through a pool of one worker per core instead of one thread per run.
7. Tuning a "~" parameter reads seed means, never one seed.

## Consequences
- Four bean runs give the mass mean within about ±10% (seed SD about 20%).
- `validate` takes 2 min 7 s on 4 cores here, up from 34 s; wheat_stand dominates.
- At model 0.9.0: 87 checks, 64 pass on seed means against 63 on seed 7 alone.
  `ndfa_low_n_bed`, `heat_seeds_per_pod` and `emergence_thermal_time_b7` pass on the
  mean. `maturity_thermal_time_b8_3` and `wheat_peak_leaf_area_stand` miss on it, each
  at a band edge. Every M0 check passes.
- The golden hashes are untouched: they hash the scenarios as written.
