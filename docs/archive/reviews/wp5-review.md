# WP5 review: soil and water (PR #16, merged as 4bcec06)

Reviewed on main after the merge. Files: crates/ps-soil (lib.rs, tests.rs), docs/models/soil-and-water.md, docs/decisions/0007-soil-water-details.md, plus the `root_uptake_rate` constant added to the species schema and bean file.

## Checks (all green on main)
- cargo fmt --check, cargo clippy --workspace --all-targets -D warnings (pedantic), cargo test --workspace (99 tests, 10 in ps-soil including one property test), scripts/check_deps.py (6 crates), scripts/check_determinism.py.

## Done-when (plan section 7, WP5)
- "A wet column drains to field capacity with closure under 1e-9": yes, for loam, sand and clay, every hour and cumulatively (`wet_column_drains_to_field_capacity`).
- "The analytic case in section 8.1 matches": yes. A saturated column of n cells loses exactly one cell's excess per hour for n hours, drainage time within 5%, outflow exact to 1e-12.
- Everything section 3.6 lists is present: 3D grid, pot mask, infiltration, drainage, lateral redistribution, evaporation, soil temperature, root uptake, FTSW and the stress factor. Section 3.2 step order 4, 5, 7 is respected.

## What is good
- Every flux is booked in a `WaterLedger` and debug builds assert closure at 1e-9 relative, as CLAUDE.md requires. The property test throws random rain, radiation, temperature and uptake at a pot and checks closure and that water stays within [0, saturation].
- Drainage and lateral redistribution are Jacobi passes (computed from one state, then applied), so results do not depend on cell visiting order. Sums are in index order. No HashMap, libm for exp and round.
- Lateral redistribution is stable: a cell gives at most 10% of the difference to each of four neighbours, so it can never go below its neighbours or above saturation.
- The uptake request returns the shortfall without a second penalty, exactly as section 3.2 step 7 asks.
- Decision 0007 records every constant the plan left open (saturation, air-dry, evaporation coefficient, damping depth, uptake rate units) and the model card lists them with sources.
- `visit_bits` exposes all state for the golden hash, including the air temperature history.

## Findings (no bugs; suggestions in priority order)

1. **Lateral redistribution fraction depends on cell size.** "10% of the gradient per hour" is a finite-difference diffusion step: fraction = D dt / dx². At 2 cm cells it implies D = 4e-5 m²/h. M2 plots use 5 cm cells; keeping 0.1 there makes lateral flow about six times too fast. Suggest replacing `LATERAL_FRACTION` by a diffusivity constant and computing the fraction from the cell size, or at least recording the dependency in the model card now so WP21 does not inherit it silently.

2. **Evaporation takes one radiation value for the whole surface.** WP6 will know how much of the pot the canopy shades per column. Consider letting `SoilHour` carry per-top-cell radiation (a slice) or an optional override, so a leaf shading half the pot dries only the sunlit half. Not needed for M0 correctness; cheap to add before callers exist.

3. **Public API takes raw f64 for a dimensioned quantity.** `take_up(..., rate_m2_per_h: f64)` and `cell_volume() -> f64` are raw where CLAUDE.md asks for ps-units newtypes at interfaces. A small `M2PerHour` and `M3` in ps-units, or passing the species constant as a typed struct, would close the gap. FTSW, theta and the stress factor are dimensionless, so f64 is right for those.

4. **Soil temperature.** The damping depth is fixed at 0.1 m for all textures and moistures; dry sand is nearer 0.05 m. Fine as "~" for M0. The 24-entry air history uses `Vec::remove(0)` once an hour, which is harmless at this size. One small point: the phase lag is rounded to whole hours, so layers at 0.03 to 0.1 m all get the same lag; a linear interpolation between the two neighbouring hours would make the lens-visible temperature profile smoother for little cost.

5. **Known limits to add to the model card.** Infiltration can fill a lower cell through a saturated upper cell in the same hour (a bucket shortcut); evaporation ignores humidity and wind (already listed) and uses one radiation value for the surface; lateral mixing is cell-size dependent (finding 1).

## Plan consistency
- Field capacity and wilting point per texture, the 2 cm pot cell, the free-draining pot bottom, the FTSW definition and the 0.4 threshold as a species parameter all match sections 3.6 and 3.2.
- The uptake formula matches section 3.6 (root length density × available water × species rate), with the hourly fraction capped at 1.
- Adding `root_uptake_rate` to the species schema is the right place for it; decision 0006 was updated to say 40 constants.
- No equation or step order from the plan changed, so no model version bump is needed.
