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
  per axis, largest `E`. Cell edge `s = max(1.2 E / 48, 5 mm)`; cells per axis
  `n_a = min(⌈1.2 E_a / s⌉, 48)`, centered on the box.
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
- **Red:far-red.** `R:FR = clamp(1.2 · exp(−0.5 · LAI_above), 0.2, 1.2)`, with
  `LAI_above = Σ (LAD_c − LAD_own,c) ℓ_c` straight up from the leaf center.

Hourly (§3.2 hourly step 3), `absorbed_par`, per unit leaf area:

- `I_dif,abs = T_dif · I_dif · f_PAR · q · α`
- `I_beam,abs = G · I_dir / max(sin β, 0.05) · f_PAR · q · α` (horizontal direct
  irradiance converted to flux normal to the beam, times the mean projection `G`)
- sunlit part (fraction `T_dir,h`): `I_dif,abs + I_beam,abs`; shaded part: `I_dif,abs`.

`I_dir`, `I_dif`: horizontal direct and diffuse global irradiance (W m⁻²); `β`: solar
elevation; `α`: leaf PAR absorptance; `f_PAR`, `q` below.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| G-function, spherical leaf angles | 0.5 | 1 | Ross 1981; species parameter (`g_function` in schema) |
| Max cells per axis | 48 | 1 | design plan §3.5 |
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
| out | 24 direct coefficients, diffuse coefficient, R:FR | 1 | `LightCoeffs` on each leaf |
| out | absorbed PAR, sunlit and shaded | µmol m⁻² s⁻¹ | gas exchange (WP7) |

## Encoded or emergent
Encoded: Beer's-law attenuation through foliage, one-leaf opacity within a footprint,
the overcast sky shape, the R:FR proxy. Emergent: self-shading inside the canopy, the
light gradient from top to bottom, sunflecks moving across the day, the plant's light
interception as it grows, and shade between neighboring plants.

## Known limits
- Leaves are discs; real bean leaflets are ovate. Stems and pods do not cast shade yet.
- Within a cell, the model does not know which leaf is on top: two leaflets that overlap
  inside one cell layer shade each other partly instead of one shading the other fully.
  That is where most per-leaf error against the ray tracer comes from.
- No scattering: light reflected or transmitted by leaves is ignored, so deep shade is a
  little too dark. Fine for M0; revisit if gas exchange in deep shade looks low.
- Coefficients are computed once a day; leaf movement within the day is ignored.
- One plant's grid in M0. Merging grids for neighbors is part of the world (WP9).
- Cost: leaves × 40 directions × 7 origins × about 50 cells. A 300-leaf plant is about
  4 million cell visits per day, a few milliseconds.

## Validation
- `ps_light::tests::uniform_canopy_matches_beers_law`: 40 × 40 × 12 cells, LAI 0.5–6,
  elevations 45–90°, absorbed fraction within 2% of `1 − exp(−G LAI / sin β)`.
- `single_plant` (crates/ps-light/tests/single_plant.rs) against the Monte Carlo ray
  tracer fixture (`validation/fixtures/light_raytrace.py`, seed 20261105, 4000 rays per
  leaflet, 57 leaflets with random spherical normals, 5 sun positions and the 16 sky
  directions). The per-leaf light index is the mean of the direct coefficients and the
  diffuse coefficient. Measured: plant total +2.9% (target 3%, met); 55 of 57 leaflets
  within 0.10 absolute (target all 57, missed: worst 0.142); RMS 0.047. See decision 0008.
- `isolated_leaf_is_fully_lit`, `leaf_under_another_is_shaded`, `grid_respects_limits`,
  `sky_weights_sum_to_one`, `rfr_proxy`, `absorbed_par_splits_sunlit_and_shaded`,
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
