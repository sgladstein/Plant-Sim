# Light

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
Works out how much sunlight and skylight reaches each leaf. Leaves higher up or out on
the edge are lit; leaves under others are shaded, and the light that gets through
foliage is redder in far-red, which the plant reads as "a neighbor is shading me".

## Equations
Daily (§3.2 daily step 6), in crate `ps-light`:

- **Grid.** Bounding box of all leaf footprints (and any extra points), extent `E_a`
  per axis, largest `E`. Cell edge `s = max(1.2 E / 32, 5 mm)`; cells per axis
  `n_a = min(⌈1.2 E_a / s⌉, 32)`, centered on the box.
- **Plot grid** (decision 0019). All plants on a plot share one grid with fixed cells
  of 3 cm (larger if the widest axis would need more than 96), a 10 cm margin around
  the leaves, and its bottom at the ground. `n_a = ⌊span_a / s⌋ + 1`. Each leaf still
  traces its own rays; per-voxel transmittance was measured and is not used.
- **Obstacles** (WP25, decisions 0024 and 0025). A box with transmittance `T` through
  its thinnest side `t` has `κ = −ln(max(T, 10⁻⁶)) / t` (m⁻¹). A ray from `p` in
  direction `d` gains `κ · ℓ_box`, the length of the ray inside the box (slab method),
  wherever the box is. Obstacles stay out of the grid, which is fitted to the leaves
  only. They add optical depth only, not leaf area, so they do not change the
  red:far-red proxy.
- **Footprint.** Leaf `i` of area `A_i` (m²) is a horizontal disc of radius
  `r_i = √(A_i/π)` at its center height, sampled on a square lattice of spacing at most
  `s/2`; each lattice point carries `A_i / N_points`. Leaf area density in cell `c`:
  `LAD_c = Σ_i A_i,c / s³` (m² m⁻³).
- **Projection** (decision 0025). Leaf `i` with blade normal `n_i` intercepts a ray in
  direction `d` in proportion to `G_i(d) = |n_i · d|`, its own G-function value toward
  `d` (both faces count). A spread of leaf angles gives the mean `G` of Ross (1981);
  spherical leaves average 0.5.
- **Extinction per leaf, cell and direction.** `ΔL_i,c = LAD_i,c · s` (the fraction of
  the cell layer leaf `i` covers, ≤ 1);
  `k_i,c(d) = −ln(1 − G_i(d) ΔL_i,c) / ΔL_i,c`, which is `G_i(d)` as `ΔL → 0`.
  Optical depth density `D_c(d) = Σ_i k_i,c(d) LAD_i,c` (m⁻¹), kept for each of the
  day's directions (the hours the sun is up and the 16 sky sectors) in the cells a leaf
  touches.
- **Ray.** From point `p` in unit direction `d`, the optical depth is
  `τ = Σ_cells (D_c(d) − D_own,c(d)) · ℓ_c + Σ_boxes κ ℓ_box`, with `ℓ_c` the path
  length in each cell (Amanatides–Woo traversal) and `D_own` the leaf's own footprint.
- **Ray origins.** A broad leaf (a bean leaflet): five points, the leaf center and four
  points at radius `r/√2` in the blade plane (the leaf normal) at 90° spacing (decision
  0033; seven points at 60° before, cut for plot speed). A grass
  blade is three strips, each sampled at three points along it, its center and `±ℓ/3`
  (the centers of its thirds), with `ℓ` the strip's length (decision 0030).
- **Direct coefficient.** For each hour `h` with the sun above the horizon,
  `T_dir,h = mean over origins of exp(−τ(p, d_sun,h))`; zero at night.
- **Diffuse coefficient.** `T_dif = Σ_s w_s · mean over origins of exp(−τ(p, d_s))`
  over 16 sectors: two bands (0–45°, 45–90°) × eight azimuths at 22.5° + k·45°. The
  standard overcast sky (CIE, Moon and Spencer) has radiance `L(β) ∝ (1 + 2 sin β)/3`;
  a band's weight is `∫ L sin β cos β dβ` over the band, and its direction is the
  weight-averaged elevation (30.50° and 62.01°). Weights sum to 1.
- **Diffuse absorbed by a leaf** (decision 0025). Per unit leaf area, absorbed diffuse
  flux is `∫ L |n · d| dΩ`, not the horizontal irradiance `∫ L sin β dΩ`. Each sector
  gets an absorption weight `a_s = ∫_band L cos β dβ / 8 ÷ ∫_hemisphere L sin β cos β dβ`
  (sum 12/7 for the overcast sky), and
  `T_dif,abs = Σ_s a_s · |n · d'_s| · mean over origins of exp(−τ(p, d_s))`. Here `d'_s`
  has the sector's azimuth and the elevation whose sine is the band's radiance-weighted
  mean `sin β` (`w_s / a_s`), so a horizontal leaf in the open gets exactly 1. A
  spherical leaf distribution would get `(12/7) · 0.5 = 0.857`. The rays use the 16
  sector directions `d_s` of `T_dif`.
- **Beam projection.** For each hour the sun is up, `P_h = |n · d_sun,h|`. A leaf of
  several leaflets keeps the sunlit-weighted mean `Σ T_dir P / Σ T_dir`, so its mean
  absorbed beam is exact.
- **Red:far-red.** `R:FR = clamp(1.2 · exp(−0.5 · LAI_above), 0.2, 1.2)`, with
  `LAI_above = Σ (LAD_c − LAD_own,c) ℓ_c` straight up from the leaf center.

Hourly (§3.2 hourly step 3), `absorbed_par`, per unit leaf area:

- `I_dif,abs = T_dif,abs · I_dif · f_PAR · q · α`
- `I_beam,abs = P_h · I_dir / max(sin β, 0.05) · f_PAR · q · α` (horizontal direct
  irradiance converted to flux normal to the beam, times the leaf's projection toward
  the sun)
- sunlit part (fraction `T_dir,h`): `I_dif,abs + I_beam,abs`; shaded part: `I_dif,abs`.

`I_dir`, `I_dif`: horizontal direct and diffuse global irradiance (W m⁻²); `β`: solar
elevation; `α`: leaf PAR absorptance; `f_PAR`, `q` below.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Max cells per axis | 32 | 1 | decision 0010 (plan §3.5 said 48) |
| Plot cell edge | 0.03 | m | decision 0019 |
| Plot max cells per horizontal axis | 96 | 1 | decision 0019 (2.9 m of canopy and margin at 3 cm; a wider canopy gets larger cells) |
| Plot margin | 0.10 | m | decision 0019 |
| Min cell edge | 0.005 | m | design plan §3.5 |
| Margin per side | 0.10 | fraction of extent | design plan §3.5 (20% total) |
| Sky sectors | 16 (2 × 8) | 1 | design plan §3.5 |
| Ray origins per leaf | 7 | 1 | decision 0008, "~" |
| PAR fraction of global radiation, `f_PAR` | 0.5 | 1 | "~"; 0.45–0.50 in the literature (Monteith and Unsworth 2013) |
| Photons per joule of PAR, `q` | 4.57 | µmol J⁻¹ | McCree 1972 (daylight) |
| Smallest `sin β` for the beam | 0.05 | 1 | "~", avoids division blow-up at sunrise |
| R:FR in open light | 1.2 | 1 | design plan §3.5 |
| R:FR extinction | 0.5 | per unit LAI | design plan §3.5, "~" |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | leaf center, area, blade normal | m, m², 1 | organ graph (ps-graph) |
| in | hourly sun direction | unit vector | ps-weather solar position |
| in | direct and diffuse irradiance | W m⁻² | ps-weather hourly series |
| out | 24 direct coefficients, 24 beam projections, diffuse coefficient (lenses), absorbed-diffuse coefficient, R:FR | 1 | `LightCoeffs` on each leaf |
| out | absorbed PAR, sunlit and shaded | µmol m⁻² s⁻¹ | gas exchange (WP7) |

## Encoded or emergent
Encoded: Beer's-law attenuation through foliage, one-leaf opacity within a footprint,
the overcast sky shape, the R:FR proxy. Emergent: self-shading inside the canopy, the
light gradient from top to bottom, sunflecks moving across the day, the plant's light
interception as it grows, and shade between neighboring plants.

## Known limits
- Leaves are discs; real bean leaflets are ovate. Stems and pods do not cast shade yet.
- A grass blade strip is sampled along its length only, not across its width
  (decision 0030); the blade is a few mm to 2 cm wide, under one light cell.
- Obstacles are grey: under a shade cloth, red:far-red stays as it is, as it does under
  a neutral cloth. They shade leaves only: the soil surface's radiation ignores them.
- Within a cell, the model does not know which leaf is on top: two leaflets that overlap
  inside one cell layer shade each other partly instead of one shading the other fully
  (leaves in one cell combine as a Poisson product). That is where most per-leaf error
  against the ray tracer comes from.
- The one-blade extinction is exact for vertical rays only: a slanted ray gets
  `(1 − G ΔL)^(1/sin β)` where one blade would give `1 − G ΔL / sin β`.
- Both effects make the model too bright, not noisy: on the 60-day sim plant the
  plant total is +3.8% against the ray tracer (decision 0025). If tuning needs a
  direction, the model overestimates light inside the canopy.
- A sector's projection `|n · d'_s|` is taken at one direction per sector, and its rays
  use the irradiance-weighted band elevation. For an erect blade whose plane crosses a
  band, `|n · d|` at the sector's mean direction under-reads its mean over the sector,
  so wheat's diffuse is a few percent low; sub-sample the band if it matters.
- A leaf of several leaflets keeps one sunlit-weighted beam projection per hour, exact
  for its mean absorbed beam but averaged before the light response of photosynthesis,
  which is not linear.
- Each leaf intercepts by its drawn angle, so the light is only as right as the drawn
  angles: every bean leaflet tilts about 19° and does not track the sun, so
  heliotropism is not modelled yet (decision 0025).
- Per leaf, 11 of a 60-day plant's 74 leaflets are off by more than 0.10 (worst 0.26),
  mostly the grid limit above (who is on top within a cell).
- No scattering: light reflected or transmitted by leaves is ignored, so deep shade is a
  little too dark. Fine for M0; revisit if gas exchange in deep shade looks low.
- Coefficients are computed once a day; leaf movement within the day is ignored.
- A plot shares one grid for all its plants; each leaf keeps its own angle, so a wheat
  blade and a bean leaflet intercept differently in one plot. Pots keep a
  grid each, so pots side by side do not shade each other.
- At bean scale, per-voxel light is no cheaper and less accurate than per-leaf rays, so
  the plan's M2 per-voxel coefficients are deferred to the tree (M4; decision 0019).
- Cost: leaves × up to 40 directions × 7 origins × about 30 cells. Rays stop when they
  leave the box of occupied cells, and leaves run in parallel in chunks of 16, each
  chunk direction by direction (ordered collect, so the result does not depend on the
  thread count). The grid keeps one optical depth per occupied cell and direction, in
  one run per direction; empty cells point at a slot of zeros. A 300-leaf plant takes
  about 3.3 ms on 4 cores (2.8 ms with one G before decision 0025); a 60-leaflet bean
  about 1 ms.

## Validation
- `ps_light::obstacle::tests`: a 1 cm wall at 10% transmittance gives `−ln 0.1` of
  optical depth along a ray across 10 cm cells; a ray beside it is clear; a cloth 2 m
  above a 10 cm grid still shades it, and a slanted ray gets its slant length.
- `ps_light::tests::uniform_canopy_matches_beers_law`: 40 × 40 × 12 cells, LAI 0.5–6,
  elevations 45–90°, absorbed fraction within 2% of `1 − exp(−G LAI / sin β)`, both for
  spherical leaf normals (`G = 0.5`) and for horizontal leaves (`G = sin β`).
- `a_tilted_leaf_shades_by_its_projection`, `projection_is_the_cosine_to_the_ray`: a
  blade tilted 60° passes half a noon sun, an edge-on blade all of it.
- `single_plant` (crates/ps-light/tests/single_plant.rs) against the Monte Carlo ray
  tracer fixture (`validation/fixtures/light_raytrace.py`, seed 20261105, 4000 rays per
  leaflet, 57 leaflets with random spherical normals, 5 sun positions and the 16 sky
  directions). The per-leaf light index is the mean of the direct coefficients and the
  diffuse coefficient. With five ray origins per leaflet (decision 0033): plant total
  +2.99% (the gate is under 3%, so it passes at its edge); all 57 leaflets within 0.10 absolute (worst 0.093); 51 of 57
  within max(0.05, 10% relative), reported but not gated. With seven origins it was
  +2.8% and worst 0.076 (decision 0025). With one G of 0.5 it was
  +1.7%, 56 of 57, worst 0.122 (decisions 0008, 0010). The test also prints each leaf's
  worst single sun direction.
- `sim_plant_against_ray_tracer` (same file): the WP12 re-check on a 60-day sim plant
  (74 leaflets, `plant60_*`, exported by the ps-arch test `export_leaves_for_ray_tracer`,
  traced with `light_raytrace.py --leaves`). With five ray origins (decision 0033) the
  plant total is +4.4%, 61 of 74 leaflets are within 0.10 (the gate is 82%, 61 leaflets)
  and the worst is 0.27; with seven, +3.8%, 63 of 74 and 0.26 (decision 0025). With one G of 0.5 it was +9.9%, 49 of 74 and 0.32: the sim draws every blade
  tilted about 19°, a planophile canopy, and G = 0.5 assumes spherical leaf angles
  (decision 0018).
- `plot_grid_light_against_ray_tracer` (same file): both fixtures lit through the 3 cm
  plot grid, five origins (decision 0033). Synthetic shoot: −0.5%, 54 of 57 within 0.10.
  Sim plant: −0.35%, 60 of 74 (seven origins: −0.7% and 54 of 57; −0.8% and 61 of 74;
  +5.6% and 51 of 74 with one G, decision 0019).
- `ps_light::plot::tests`: `the_plot_grid_has_fixed_cells_and_starts_at_the_ground`,
  `neighbors_in_a_plot_shade_each_other`.
- `isolated_leaf_is_fully_lit` (also: a horizontal leaf in the open absorbs exactly the
  horizontal diffuse irradiance), `leaf_under_another_is_shaded`, `grid_respects_limits`,
  `sky_weights_sum_to_one` (also checks the absorption weights sum to 12/7),
  `ray_from_below_the_grid_enters_it`, `bounded_traversal_is_bit_identical`, `rfr_proxy`, `absorbed_par_splits_sunlit_and_shaded`,
  `fixture_sky_directions_match_sky_sectors`.

## References
- Amanatides, J. and Woo, A. (1987). A fast voxel traversal algorithm for ray tracing.
  Eurographics.
- McCree, K. J. (1972). Test of current definitions of photosynthetically active
  radiation against leaf photosynthesis data. Agricultural Meteorology 10.
- Monteith, J. L. and Unsworth, M. H. (2013). Principles of Environmental Physics, 4th ed.
- Moon, P. and Spencer, D. E. (1942). Illumination from a non-uniform sky. Illuminating
  Engineering 37.
- Nilson, T. (1971). A theoretical analysis of the frequency of gaps in plant stands.
  Agricultural Meteorology 8.
- Ross, J. (1981). The Radiation Regime and Architecture of Plant Stands.
- Sinoquet, H. et al. (2001). RATP: a model for simulating the spatial distribution of
  radiation absorption, transpiration and photosynthesis within canopies. Plant, Cell and
  Environment 24.
