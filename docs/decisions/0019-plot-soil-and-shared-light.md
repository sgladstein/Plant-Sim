# 0019: Plot soil and shared light (WP21)

- **Date:** 2026-11-05
- **Status:** accepted

## Context
WP21 (design plan §7, M2) asks for three things:

- a plot soil block from 1 × 1 to 3 × 3 m with 5 cm cells;
- a merged light grid, so plants in a plot shade each other;
- per-voxel light coefficients (§3.5: "at M2 the coefficients move from per-leaf rays
  to per-voxel transmittance").

It is done when 30 plants run in under 20 ms per sim-day.

Until now each pot had its own soil grid and each plant its own light grid, fitted to
its own leaves. Nothing coupled two plants except water in a shared pot.

## Decision
Model version 0.2.0 → 0.3.0. Golden hashes are regenerated, and a 30-plant plot is added
to them.

### Plot soil
- **Scenario.** A new soil kind, `SoilSpec::Plot { id, width_cm, length_cm, depth_cm,
  texture, position_m }`. Sides must be 100 to 300 cm and the depth at least 10 cm.
- **Grid.** The plot is an unmasked box (`SoilGrid::new_box`) of 5 cm cells. Its bottom
  drains freely into a deep layer that never returns water, as the plan says.
- **Lateral flow as a diffusivity.** The lateral fraction of 0.1 per hour (§3.6) was set
  for 2 cm cells. The same spread in a 5 cm cell needs a smaller fraction. It is now
  `f = 0.1 × (0.02 m / Δx)²`, capped at 0.25. That is a diffusivity of 4e-5 m² h⁻¹, as
  the WP5 review asked. Pots (2 cm) still get exactly 0.1, so they are unchanged. A 5 cm
  plot gets 0.016.

### Shared light: one grid per plot, per-leaf rays
Every plant on a plot casts its leaves into one light grid, and each leaf traces its own
rays through that grid (`ps_light::rebuild_plot`). The grid has:

- 3 cm cells, at most 96 per horizontal axis;
- a 10 cm margin around the plants;
- its bottom at the ground (z = 0).

A pot plant keeps its own fitted grid, as in M0.

Per-voxel coefficients were built and measured, and they are not used. All four methods
were tried on the two ray-tracer fixtures (decision 0018): the synthetic WP6 shoot (57
leaflets) and the 60-day sim plant (74 leaflets). The table gives the plant-total error,
then the leaflets within 0.10 of the ray tracer.

| Method | Synthetic shoot | Sim plant |
| --- | --- | --- |
| Voxel transmittance, layer sweep, 5 cm | −6.7%, 45 of 57 | +6.6%, 39 of 74 |
| Voxel transmittance, traced from face centers, 5 cm | −6.0%, 47 of 57 | +8.6%, 35 of 74 |
| Voxel transmittance on the 3 cm plot grid | −3.7%, 41 of 57 | +4.5%, 48 of 74 |
| **Per-leaf rays on the 3 cm plot grid (chosen)** | **−0.8%, 51 of 57** | **+5.6%, 51 of 74** |
| Per-leaf rays, own fitted grid (M0, pots) | +1.7%, 56 of 57 | +9.9%, 49 of 74 |

The reasons for the choice:

- **Accuracy.** A leaf that reads its voxel's transmittance cannot tell the top of the
  voxel from the bottom. Interpolating optical depth between voxels darkens the canopy
  (the Jensen effect, since the mean of exp is above the exp of the mean). Interpolating
  transmittance instead blurs each shadow by one layer.
- **Speed.** A bean canopy has about one leaflet per occupied voxel. So voxels ×
  directions costs no less than leaves × directions, and the voxel version spends rays on
  empty air.
- **The plan's cost argument still holds for M4.** A tree has hundreds of leaves per
  voxel, and per-voxel light can come back then.

The scratch implementation is not committed. The numbers above are its record.

### One G per plot
G is still one number per plot, taken from the first plant's species. Mixed plots get the
first species' G until WP23 brings a second species with a different leaf angle. The open
G question from decision 0018 is unchanged by this decision.

### Stepping
- **Hourly.** Within a soil, the plants are stepped in parallel with rayon and collected
  in plant order: water demand, then the soil's joint uptake, then hourly growth. The
  result is bit-identical for any thread count (tested with 1 and 4 threads).
- **Daily.** A plant's day is split at the light rebuild. `Plant::day_before_light`
  runs the daily steps up to the layout and root cells. Then the plot's leaves are
  gathered in plant order, the shared grid's coefficients are split back per plant, and
  `Plant::day_after_light` finishes the day. `Plant::day` is the two halves joined, so
  pots are unchanged.

### Scenarios
- `plot_30_beans.ron`: 30 bush beans in 5 rows of 6, 25 cm × 24 cm apart (16.7 per m²),
  on a 150 × 120 × 40 cm loam plot watered to field capacity every morning. Its first
  30 days are in the golden hashes, and CI checks them on Linux and Windows.
- `m2/density_5.ron`: 4 × 4 plants 45 cm apart on 180 × 180 cm (4.9 per m²).
- `m2/density_35.ron`: 7 × 7 plants 17 cm apart on 120 × 120 cm (34 per m²).
- The density check (`density_leaf_area_per_plant`, Clavijo Michelangeli 2019) moves
  from Later to Info.

## Consequences
- **Pots did not change.** The daily rows and organ tables of `single_pot` and
  `two_pot_sun_shade` are byte-identical to model 0.2.0. Their hashes changed only
  because the version string and the new `shared_light` field are hashed.
- **Neighbors shade each other**, but competition is weak. Plants in the 30-bean plot
  have a mean light index of 0.39, against 0.56 for the same plant alone. Their mass
  hardly moves (about 24 g against 23 g).
  - From flowering on, the plants are sink-limited (decision 0018). So less light does
    not mean less growth.
  - Crowns are narrow, about 7 cm in radius, so rows 24 cm apart barely close.
  - **Density is an Info miss.** Leaf area per plant falls 25% from 5 to 34 plants per
    m², against the cited −70%. This is the main M2 model gap, and an M2 exit criterion.
    WP24's leaf nitrogen and remobilization should make shaded leaves cost more and die
    sooner. A wider-spreading architecture would also help.
- **Speed: not yet met.** The 30-bean plot takes 28.8 ms per sim-day on 4 cores and 59 ms
  on one thread, against the 20 ms target. A callgrind profile over 60 days (23.5 G
  instructions) shows where the time goes:
  - light coefficients, 26%;
  - hourly growth, 22% (of which carbon allocation is 19%);
  - water demand, 13% (class tables 10.7%, the gas exchange solve 9%);
  - the state hash, 4%.

  The perf thread is asked for a bit-identical speedup in this PR or a follow-up.
- **Not done in WP21:**
  - Pots placed close together still have separate light grids. Merging overlapping
    pots' grids was a WP9 note and is left for when a scenario needs it.
  - Soil surface radiation is still one value per soil (leaf area over the whole
    plot), not per column. Canopy shade does not yet dry the sunlit gaps between rows
    faster.
  - The viewer draws a plot as a round pot. The viewer thread owns that change.
