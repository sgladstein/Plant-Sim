# WP6 review: light (PR #17, branch head 43a0edf, not yet merged)

Reviewed on the PR branch. Files: crates/ps-light (lib.rs, tests.rs, tests/single_plant.rs, fixtures), validation/fixtures/light_raytrace.py, docs/models/light.md, docs/decisions/0008-light-model-details.md.

## Checks (all green on the branch)
- cargo fmt --check, clippy pedantic, cargo test --workspace (108 tests; 7 unit + 2 integration in ps-light), check_deps (7 crates), check_determinism.

## Done-when (plan section 7, WP6)
- "Uniform canopy within 2% of Beer's law": met, LAI 0.5 to 6 at three elevations.
- "Single plant within 10% per leaf of the Python ray tracer fixture": 55 of 57 leaflets within 0.10 absolute, worst 0.142; plant total +2.9% against the 3% target. The test pins today's numbers rather than claiming the target. Honest and the right call. My recommendation on the open question is at the end.

## Answers to the three questions

### 1. The one-blade extinction k = -ln(1 - G dL)/dL
Correct for what it claims, and a genuine improvement over "area at the center". Derivation checks: a leaf covering fraction dL of a cell's horizontal section, with mean projection G, blocks a vertical ray with probability G dL, so the optical depth per layer must be -ln(1 - G dL); dividing by dL gives k per unit LAD, and the limit dL to 0 is G (Beer). Two caveats to record in the model card, neither blocking:
- It is exact for vertical rays only. For a slanted ray the path through the layer is s/cos(theta) and the code gives transmittance (1 - G dL)^(1/cos theta) where the one-blade answer is 1 - G dL/cos(theta). Same to first order; for big leaflets and low sun the model is brighter than the blade. This is one reason the residuals are all positive.
- Two leaves in one cell are still combined as a Poisson product. Where sibling leaflets overlap, the true answer is "the upper one blocks", and the product is too bright. This is the known limit you named, and it explains why leaves 7 and 8 (siblings in one trifoliate) are the misses.

Every one of the seven largest errors is positive (+0.08 to +0.14), and the plant total is +2.9%. The model has a consistent bright bias from the two effects above, not noise. Worth stating in the card, because WP12 may need to know which way to lean.

### 2. The per-leaf metric: 0.10 absolute on the mean of direct and diffuse
I agree relative error is the wrong gate for shaded leaves, and that the plan's "10% per leaf" was written loosely. Two refinements:
- Averaging five sun positions with the diffuse coefficient can hide errors of opposite sign in different directions. Keep the index as the gate, but print the worst per-direction error per leaf as a diagnostic, so a change that fixes the mean while breaking a direction is visible.
- A mixed tolerance "within max(0.05 absolute, 10% relative)" is the usual compromise: tight for well-lit leaves, bounded for shaded ones. With it, today's result would be about 50 of 57, so I would not adopt it as the gate now, but report it.

The plan text in section 8.1 should be updated to the metric actually used (absolute 0.10 on the daily light index, relative on the area-weighted total). I will make that edit in the plan when the owner decides the gate.

### 3. Is the synthetic fixture fair?
It is harsh but legitimate, and harsher than a real bean in one way that matters: each leaflet gets an independent spherical normal, so a near-vertical leaflet (leaf 2, normal z = 0.14) sits beside horizontal siblings inside one cell layer. Real trifoliate leaflets are roughly coplanar and mostly within about 30 degrees of horizontal (bean is planophile). Disc overlap is only slightly exaggerated: real leaflets touch too. Suggestions:
- Make sibling normals correlated (one leaf normal per trifoliate plus a small per-leaflet tilt) and keep the spherical distribution across leaves. That is both more realistic and a fairer test of the voxel assumption.
- Add a true Monte Carlo diffuse reference: random sky directions drawn from the overcast radiance, not the model's own 16 sector centers. Today the diffuse comparison only validates the ray marcher, not the two-band sky discretization. Cheap in Python.
- Regenerate from the WP8 export as planned.

## Findings

1. **Medium: diffuse and beam use different leaf-orientation conventions.** `absorbed_par` gives the sunlit part G times the beam flux normal to the sun (random leaf orientation), but gives the diffuse part the full horizontal diffuse irradiance times transmittance (a horizontal leaf). Under the same G convention, an unobstructed leaf of random orientation under a standard overcast sky receives (12/7) G = 0.857 of the horizontal diffuse irradiance, not 1.0; with the current two-band sector means and 1/sin(beta) weighting it comes out at 0.74, so the band discretization also loses 14%. Diffuse PAR is 17% high today, and overcast days are diffuse. Fix: compute a second set of sector weights for absorption, w_abs,s = G times the integral of L(beta) cos(beta) over the band (radiance times solid angle, no sin(beta) projection), normalized by the hemisphere's horizontal irradiance, so they sum to 0.857 for G = 0.5; use the cos-weighted mean elevation for those sectors; absorbed diffuse = sum of w_abs,s T_s times I_dif. Keep the current `diffuse` coefficient (sums to 1) for lenses. This is an equation change: model card, test and a model minor bump. Do it before WP7 consumes the output. Longer term, since `LeafPoint` carries the normal, per-leaf |n . d| could replace G for the beam, which would also let heliotropism matter.

2. **Low: rays that start outside the grid are counted as fully lit.** `path_sum` returns 0 when `locate(from)` fails instead of advancing the ray to the grid boundary. The bounding box pads x and y by the leaf radius but not z, and the ring sample points of a tilted leaf extend up to 0.71 r below its center. If a caller omits the stem base from `extra_bounds`, a low tilted leaf's lower samples can fall under the grid and be reported unshaded. Pad z by r in `fit`, or clip the ray to the box in `path_sum` (both are a few lines).

3. **Low: G = 0.5 for bean.** Spherical is the plan's default and the fixture is consistent with it, but bean canopies are planophile, so G near zenith is more like 0.7 to 0.8 and lower at low sun. The species file already carries `g_function`; WP12 should check it against the plan's section 8 targets. A direction-dependent G is a later option.

4. **Nits.** `sky_sectors()` integrates with 512 midpoint steps every call; it is cheap but could be a `const` table with the integral done offline and a test that regenerates it. `coefficients` allocates the footprint per leaf per day; fine at M0 scale. The plan says one ray per leaf; decision 0008 records seven, which is the right trade.

## Recommendation on the open question
Accept the pinned gate (95% of leaflets within 0.10, worst below 0.15, total within 3%) for M0, re-measure on the WP8 export, and decide the final gate at WP12. Reason: the two misses are sibling leaflets whose combined light is right; the error is in which leaflet gets it, which barely moves whole-plant carbon. Fix finding 1 first, because it changes the hourly absorbed PAR by more than the per-leaf misses do.
