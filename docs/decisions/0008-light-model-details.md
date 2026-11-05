# 0008: Light model details the plan left open

- **Date:** 2026-11-05
- **Status:** accepted (implementer), except the per-leaf validation miss, which is open
  with the owner

## Context
§3.5 says leaf area density is "accumulated from each leaf's area at its center", one
ray per leaf per direction from the leaf center, Beer's law with G, a "standard overcast
sky weighting" for 16 sectors, and validation of a single plant within 10% per leaf and
3% on the total against an offline Monte Carlo ray tracer. Built literally, a mature
bean leaflet (25–40 cm²) is several times the 1–2 cm cell, so all its area lands in one
cell and casts a small, very dark spot; and a leaf in the same cell as its neighbors
shades itself. Against the ray tracer the literal model was 9% too bright on the plant
total and up to 0.22 off on single leaves.

## Decision
1. **Footprint.** A leaf's area is spread over a horizontal disc of its own area at its
   center height (lattice spacing at most half a cell). This replaces "area at its
   center".
2. **Self-exclusion.** A leaf's own footprint is subtracted wherever its rays cross it,
   so an isolated leaf is fully lit while other leaves in the same cells still count.
3. **One leaf is opaque, not a cloud.** Per leaf and cell, the extinction coefficient is
   `k = −ln(1 − G ΔL)/ΔL`, with `ΔL` the fraction of the cell layer the leaf covers
   (Nilson's positive binomial with one leaf per layer). Small leaves recover Beer's
   `k = G`, so the uniform canopy still matches Beer's law within 2%; a ray through a
   whole leaflet is blocked with probability `G` instead of `1 − exp(−G)`. This took the
   plant total from +5.6% to +2.9%.
4. **Seven ray origins per leaf**, the center and a ring at `r/√2`, in the plane of the
   blade (so `LeafPoint` gains the leaf normal). The coefficient is the mean
   transmittance. This took the worst leaf from 0.22 to 0.14. It costs 7× the plan's
   ray count, still a few milliseconds a day for 300 leaves.
5. **Sky.** CIE standard overcast radiance `(1 + 2 sin β)/3`; band weights are its
   horizontal irradiance integrated over 0–45° and 45–90°; each band's ray elevation is
   the weight-averaged elevation (30.50°, 62.01°); azimuths 22.5° + k·45°.
6. **PAR.** 0.5 of global radiation, 4.57 µmol J⁻¹ (McCree). Beam per unit leaf area is
   `G · I_dir,horizontal / max(sin β, 0.05)`.
7. **Validation metric.** The per-leaf quantity is a light index: the mean of the direct
   coefficients over five sun positions and the diffuse coefficient. "Within 10%" is
   read as 0.10 absolute on that index, because relative error is unbounded for deeply
   shaded leaves. The plant total is relative, weighted by leaf area.
8. **Fixture.** Until WP8 can export a 60-day plant, the ray tracer uses a synthetic
   bean-like shoot: 19 trifoliate leaves (57 leaflet discs) with random spherical
   normals. Regenerate it from a sim export in WP8 or WP12.

## Result and the open part
- Plant total: +2.9% (target 3%, met, with little margin).
- Per leaf: 55 of 57 within 0.10, worst 0.142, RMS 0.047 (target: all within 0.10,
  **missed**). Both misses are leaflets of one trifoliate that overlap inside one cell
  layer, where a grid cannot tell which leaflet is on top.
- The test pins today's result (95% of leaves within 0.10, worst below 0.15, total
  within 3%) so it cannot regress unnoticed. It does not claim the target is met.
- Options for the owner: accept 95% of leaves within 0.10 as the M0 target; or keep the
  target and add a leaf-to-leaf correction for close neighbors (explicit disc tests
  between leaves sharing a cell) in WP12; or re-measure on the WP8 plant before deciding.

## Consequences
- Model minor version: first light model, no golden hashes yet.
- `LightGrid` stores G and an optical depth grid; `coefficients` reads G from the grid.
- Organ normals must reach ps-light from ps-graph `Orientation` (WP8 wiring).
