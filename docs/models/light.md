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
- **Footprint.** Leaf `i` of area `A_i` (m²) is a horizontal disc of radius
  `r_i = √(A_i/π)` at its center height, sampled on a square lattice of spacing at most
  `s/2`; each lattice point carries `A_i / N_points`. Leaf area density in cell `c`:
  `LAD_c = Σ_i A_i,c / s³` (m² m⁻³).
- **Extinction per leaf and cell.** `ΔL_i,c = LAD_i,c · s` (the fraction of the cell
  layer leaf `i` covers, ≤ 1);
  `k_i,c = −ln(1 − G ΔL_i,c) / ΔL_i,c`, which is `G` as `ΔL → 0`.
  Optical depth density `D_c = Σ_i k_i,c LAD_i,c` (m⁻¹).
- **Ray.** From point `p` in unit direction `d`, the optical depth is
  `τ = Σ_cells (D_c − D_own,c) · ℓ_c`, with `ℓ_c` the path length in each cell
  (Amanatides–Woo traversal) and `D_own` the leaf's own footprint.
- **Ray origins.** Seven points: the leaf center, and six points at radius `r/√2` in the
  blade plane (the leaf normal) at 60° spacing.
- **Direct coefficient.** For each hour `h` with the sun above the horizon,
  `T_dir,h = mean over origins of exp(−τ(p, d_sun,h))`; zero at night.
- **Diffuse coefficient.** `T_dif = Σ_s w_s · mean over origins of exp(−τ(p, d_s))`
  over 16 sectors: two bands (0–45°, 45–90°) × eight azimuths at 22.5° + k·45°. The
  standard overcast sky (CIE, Moon and Spencer) has radiance `L(β) ∝ (1 + 2 sin β)/3`;
  a band's weight is `∫ L sin β cos β dβ` over the band, and its direction is the
  weight-averaged elevation (30.50° and 62.01°). Weights sum to 1.
- **Diffuse absorbed by a random leaf.** Per unit leaf area of a randomly oriented
  leaf, absorbed diffuse flux is `G ∫ L dΩ`, not the horizontal irradiance `∫ L sin β dΩ`.
  Each sector gets an absorption weight `a_s = ∫_band L cos β dβ / 8 ÷ ∫_hemisphere L sin β
  cos β dβ` (sum 12/7 for the overcast sky), and
  `T_dif,abs = G Σ_s a_s · mean over origins of exp(−τ(p, d_s))`, which is
  `(12/7) G = 0.857` in the open for G = 0.5. The rays use the same 16 directions as
  `T_dif`.
- **Red:far-red.** `R:FR = clamp(1.2 · exp(−0.5 · LAI_above), 0.2, 1.2)`, with
  `LAI_above = Σ (LAD_c − LAD_own,c) ℓ_c` straight up from the leaf center.

Hourly (§3.2 hourly step 3), `absorbed_par`, per unit leaf area:

- `I_dif,abs = T_dif,abs · I_dif · f_PAR · q · α` (same random-orientation convention as
  the beam)
- `I_beam,abs = G · I_dir / max(sin β, 0.05) · f_PAR · q · α` (horizontal direct
  irradiance converted to flux normal to the beam, times the mean projection `G`)
- sunlit part (fraction `T_dir,h`): `I_dif,abs + I_beam,abs`; shaded part: `I_dif,abs`.

`I_dir`, `I_dif`: horizontal direct and diffuse global irradiance (W m⁻²); `β`: solar
elevation; `α`: leaf PAR absorptance; `f_PAR`, `q` below.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| G-function, spherical leaf angles | 0.5 | 1 | Ross 1981; species parameter (`g_function` in schema) |
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
| out | 24 direct coefficients, diffuse coefficient (lenses), absorbed-diffuse coefficient, R:FR | 1 | `LightCoeffs` on each leaf |
| out | absorbed PAR, sunlit and shaded | µmol m⁻² s⁻¹ | gas exchange (WP7) |

## Encoded or emergent
Encoded: Beer's-law attenuation through foliage, one-leaf opacity within a footprint,
the overcast sky shape, the R:FR proxy. Emergent: self-shading inside the canopy, the
light gradient from top to bottom, sunflecks moving across the day, the plant's light
interception as it grows, and shade between neighboring plants.

## Known limits
- Leaves are discs; real bean leaflets are ovate. Stems and pods do not cast shade yet.
- Within a cell, the model does not know which leaf is on top: two leaflets that overlap
  inside one cell layer shade each other partly instead of one shading the other fully
  (leaves in one cell combine as a Poisson product). That is where most per-leaf error
  against the ray tracer comes from.
- The one-blade extinction is exact for vertical rays only: a slanted ray gets
  `(1 − G ΔL)^(1/sin β)` where one blade would give `1 − G ΔL / sin β`.
- Both effects make the model too bright, not noisy: the seven largest per-leaf errors
  are all positive and the plant total is +2.9%. If WP12 tuning needs a direction, the
  model overestimates light inside the canopy.
- The absorbed-diffuse weights are exact band integrals, but their rays use the
  irradiance-weighted band elevations rather than the radiance-weighted ones.
- The beam uses the mean projection G, not each leaf's own normal; heliotropism cannot
  matter yet.
- G is one number for every direction. The sim's blades all tilt about 19°, so their
  true G is near 0.9 for a high sun and lower near the horizon. With G = 0.5 a 60-day
  plant gets about 10% too much light (decision 0018). The fix planned for WP21 is G
  per direction from the drawn leaf normals (area-weighted |n·d|).
- Per leaf, about a third of a 60-day plant's leaflets are off by more than 0.10 at
  G = 0.5, and a fifth at G = 0.65. Part of that is the grid limit above (who is on
  top within a cell). A known limit at the M0 exit.
- No scattering: light reflected or transmitted by leaves is ignored, so deep shade is a
  little too dark. Fine for M0; revisit if gas exchange in deep shade looks low.
- Coefficients are computed once a day; leaf movement within the day is ignored.
- A plot shares one grid and one G (the first plant's) for all its plants. Pots keep a
  grid each, so pots side by side do not shade each other.
- At bean scale, per-voxel light is no cheaper and less accurate than per-leaf rays, so
  the plan's M2 per-voxel coefficients are deferred to the tree (M4; decision 0019).
- Cost: leaves × 40 directions × 7 origins × about 30 cells. Rays stop when they leave
  the box of occupied cells, and leaves run in parallel (ordered collect, so the result
  does not depend on the thread count). A 300-leaf plant is about 19 ms of single-core
  work per day, about 5 ms on 4 cores; a 60-leaflet bean about 1 ms (decision 0010).

## Validation
- `ps_light::tests::uniform_canopy_matches_beers_law`: 40 × 40 × 12 cells, LAI 0.5–6,
  elevations 45–90°, absorbed fraction within 2% of `1 − exp(−G LAI / sin β)`.
- `single_plant` (crates/ps-light/tests/single_plant.rs) against the Monte Carlo ray
  tracer fixture (`validation/fixtures/light_raytrace.py`, seed 20261105, 4000 rays per
  leaflet, 57 leaflets with random spherical normals, 5 sun positions and the 16 sky
  directions). The per-leaf light index is the mean of the direct coefficients and the
  diffuse coefficient. Measured at 32 cells per axis: plant total +1.7% (target 3%,
  met); 56 of 57 leaflets within 0.10 absolute (target all 57; owner accepted 95% for
  M0, worst 0.122); 50 of 57 within max(0.05, 10% relative), reported but not gated.
  At 48 cells it was +2.9%, 55/57, worst 0.142 (decision 0010). The test also
  prints each leaf's worst single sun direction. See decision 0008.
- `sim_plant_against_ray_tracer` (same file): the WP12 re-check on a 60-day sim plant
  (74 leaflets, `plant60_*`, exported by the ps-arch test `export_leaves_for_ray_tracer`,
  traced with `light_raytrace.py --leaves`). With the species G of 0.5 the plant total
  is +9.9%, 49 of 74 leaflets are within 0.10 and the worst is 0.32. With G = 0.65 the
  total is −1.9%, 60 of 74 are within 0.10 and the worst is 0.23. The sim draws every
  blade tilted about 19° from horizontal, a planophile canopy, and G = 0.5 assumes
  spherical leaf angles, so the model shades the plant too little (decision 0018).
- `plot_grid_light_against_ray_tracer` (same file): both fixtures lit through the 3 cm
  plot grid. Synthetic shoot: −0.8%, 51 of 57 within 0.10. Sim plant: +5.6%, 51 of 74
  (decision 0019).
- `ps_light::plot::tests`: `the_plot_grid_has_fixed_cells_and_starts_at_the_ground`,
  `neighbors_in_a_plot_shade_each_other`.
- `isolated_leaf_is_fully_lit`, `leaf_under_another_is_shaded`, `grid_respects_limits`,
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
