# 0041: Phototropism of new internodes (R1c)

- **Date:** 2026-11-06
- **Status:** accepted (owner approved the realism plan's R1 to R3 on 2026-11-05; R1c is
  `docs/research/realism-plan/r1-carbon-sinks.md` §5)

## Context
Before R1c a bean's new internode started from its parent's direction and turned 0.3 of
the way to vertical (shoot gravitropism, decision 0013), whatever the light. Stems grew
straight up next to a wall, and the viewer drew a lean from the leaves' light asymmetry
(WP20) that the sim did not have. Design plan §3.4 asks for phototropism as small daily
rotations of new internodes toward the light, bounded per day. The R1 spec (§5) sets the
light vector, `k_p` 0.3 and `θ_max` 10° a day (the research thread's numbers: 10° is
conservative, 20 to 30° a day is defensible from seedling bending in sunflower and pea;
no bean stem kinetics were found).

## Decision
1. **The light an apex sees.** At daily step 6, after the light rebuild, each growing
   shoot apex of a bean (an axis that has not terminated, with a living apex) casts one
   ray from its tip toward each hour's sun and each of the 16 sky sectors through the
   day's grid: every leaf in the cells crossed, no own footprint, and every obstacle.
   It keeps `D_h = T_h ŝ_h` for each hour and `S = Σ_s a_s T_s d'_s`, with `a_s` the
   sector's radiance times solid angle and `d'_s` its flux-weighted direction (the
   light card's absorption weights). A plot computes them on its shared grid
   (`ps_light::apex_light`, `Plant::apex_points`).
2. **Summed hour by hour.** Each hour the apex adds the vector irradiance at its tip,
   `I_b,h D_h + I_d,h S`, with `I_b` the beam normal to the sun (horizontal direct over
   `max(sin β, 0.05)`, as for leaves) and `I_d` the horizontal diffuse. This is the
   spec's `v` with the hour's real radiation. The spec weights sky sectors by `w_s`
   (their share of horizontal irradiance); the radiance times solid angle is used
   instead, because the horizontal part of the vector irradiance is the light
   difference across a vertical stem (`∫ L (d · n) dΩ` for a side facing `n`). Under an
   open overcast sky `S = (0, 0, 1)`.
3. **The turn.** At daily step 3 a new internode starts from its parent's direction
   `u`, turns toward `v̂` (what the apex saw since the axis' last node) by
   `min(θ_max · d, k_p · ∠(u, v̂))` in the plane of the two (`d` the days of light seen,
   so the bound is per day), then gravitropism adds `0.3 · (0, 0, 1)` as before. The
   sum starts again at each new node. Grown internodes keep their direction. A twining
   vine's internode still follows its support (decision 0021); a free vine's turns.
4. **Grass does not bend.** A wheat apex sits at the crown until stem elongation and the
   culms grow straight; wheat's keys are 0 ("unused by the grass") and casts no rays.
5. **Parameters** (species schema 15): `phototropism_weight` 0.3 and
   `phototropism_max_per_day` 10° for both beans, from the spec, checked 0 to 1 and 0 to
   90°. The daily bound does not bind at today's phyllochron (review): nodes come 3 to 5
   days apart, so it allows 30 to 50° per node while `k_p · ∠` is at most 27° for light
   within 90°, and 30° a day gives bit-identical runs (16 pot seeds, 4 plot seeds). So
   `k_p` alone sets the response: under a fixed light direction `α` from vertical a stem
   settles at about `α · k_p / (k_p + 0.3) = α/2`.
6. No change to the §3.2 step list: the rays are part of daily step 6, the hourly sum of
   hourly step 3, the turn of daily step 3.

### Tried and not kept
- **A taller wall in the done-when test.** Beside an opaque wall 1.5 m tall, 11 cm east
  of the pot's center, the plant leans 5 to 16 cm away by days 40 to 70 but gains no light (−0
  to −3%, seeds 7 and 11): the wall still hides most of the eastern sky from a few cm
  further on, and the leaning branches crowd each other. Leaning pays where moving a few
  cm changes what the leaves see: beside a 40 cm wall 6 cm east, the leaves absorb more
  PAR than with phototropism off on every one of eight seeds by day 45 (+0.4 to +12%).
  The test uses that wall.

## Consequences
- Model 0.22.0, species schema 15; every golden hash moves.
- Validation on four-seed means: **94 checks, 70 pass, every M0 gate passes** (0.21.0:
  70). `stem_share_peak` now passes (0.349 → 0.354, band from 0.35);
  `harvest_index_tops` slips out (0.410 → 0.374, band from 0.40). Pole heights at day
  60, 75 and final are unchanged (the main stem twines up its stake). Shade internodes
  +27% (−12% to +43% over the seeds), shade total mass −46%, shade pods −45%; heat_seeds'
  weakest seed −21%.
- **A lone pot plant hardly leans** (single_pot, world seed 7: the main stem's tip is
  1.2 mm from above the collar at days 50 and 70, 34 to 35 cm up; the summed sun path
  and the open sky are close to vertical), so its yield changes by chance, not by
  geometry: leaf area and carbon fixed stay within 0.3% of 0.21.0 to day 45, then pod
  set draws go the other way (7 pods against 9 at day 55). Over world seeds 7 to 22
  against phototropism off, seed mass per plant −0.50 ± 0.32 g (mean ± s.e., 12 of 16
  lower), assimilation −0.52 ± 0.37 g C, harvest index tops 0.401 against 0.409: a
  possible small cost in the lone pot, at the band edge (review). Seeds 7 to 10, the
  validation's four, are the low tail (0.374), which takes `harvest_index_tops` out of
  its band.
- **In the open it mostly straightens branches** (review). Branch internodes stand
  27.2° from vertical against 33.6° with phototropism off in the pot (27.3° against
  33.6° in the plot), and pot internode tips sit 20 to 25% closer to the collar. Under an
  open sky a tilted branch's apex sees light from nearly straight up, so the rule adds a
  second upward pull on top of the calibrated 0.3 gravitropism. That is physically fair,
  but it reshapes the canopy and is the likeliest cause of the pot's small seed loss.
  Branch angles are not yet checked against bean data; lowering `shoot_gravitropism` is
  the other option.
- Main stems lean only beside obstacles: their tips move 0.2 to 3.2 mm in the pot and 1
  to 2 mm in the plot (at most 6.5 mm), and edge plants do not lean outward. Free vines
  turn: `pole_unstaked_seed_yield` goes −51% → −59%, still in band.
- Beside a 40 cm wall 6 cm east (single_pot, world seed 7): the main stem's tip ends
  8.2 cm west at day 50, and the plant has fixed 7% more carbon (3.38 against 3.17 g C)
  and weighs 5.0 against 4.6 g. Figure: `/mnt/project-files/figures/r1c/r1c_wall_lean.png`.
- Speed: within the container's noise (plot_30_beans 34.0 to 36.6 ms per sim-day with
  phototropism, 34.1 to 35.6 without; single_pot 3.5 to 3.9 against 3.1 to 3.9).
- **Viewer.** The sim now leans stems beside obstacles, so the WP20 drawn lean
  (`ps_mesh::moments::lean_from_light`) should be turned down near obstacles, or the
  sim's lean subtracted, or a plant beside a wall leans twice. Switching it off would
  remove the only lean visible in open plots.
- Follow-ups: turn the growing internode daily so the per-day bound means what it says;
  check branch angles against bean data; leaf heliotropism (blades turning toward the
  sun, light follow-ups of decision 0025); a phototropism rate for bean stems if one is
  found (10° a day is a guess). Small points from review: a plant with no light grid
  keeps yesterday's apex light, and a ray's partial path through its starting cell
  (about 10 cm in the plot) could tilt the vector in dense cells; nothing shows either
  at today's lean sizes.

## References
- Spec R1 §5 and the R1 numbers (`docs/research/realism-plan/numbers/README.md`):
  seedling bending of sunflower and pea, about 25° in 2 h.
