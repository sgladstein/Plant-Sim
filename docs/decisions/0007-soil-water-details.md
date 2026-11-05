# 0007: Soil water details the plan left open

- **Date:** 2026-11-05
- **Status:** accepted (implementer; design plan §3.6 gives the processes, not every constant)

## Context
§3.6 specifies field capacity and wilting point per texture, the order of processes,
"drainage moves excess down one cell per hour", "lateral redistribution moves 10% of any
gradient per hour", and "evaporation takes a fraction of radiation reaching the soil
surface ... reduced as it dries". It does not give saturation, the evaporation fraction,
the dryness reduction, how infiltration treats water beyond field capacity, the uptake
rate's units, or how soil temperature lags air temperature.

## Decision
- Saturation: loam 0.45, sand 0.40, clay 0.50; air-dry: 0.05, 0.02, 0.10 (all "~").
- Infiltration fills each column top-down to field capacity, then top-down to saturation;
  water that still does not fit is runoff. Drainage then moves all excess above field
  capacity one cell down per hour (Jacobi), and the bottom layer's excess leaves the pot.
- Lateral redistribution moves 10% of the water-mass difference between each pair of
  active horizontal neighbors per hour, computed from one state (Jacobi).
- Evaporation potential = 0.5 × radiation at the soil surface / latent heat, reduced
  linearly from field capacity to air-dry in the top layer, and never below air-dry.
- Uptake coefficient `root_uptake_rate` is a species constant in m² h⁻¹ (added to schema 1);
  root length density × coefficient is the hourly share of a cell's available water.
- Soil temperature per layer = 24-hour mean air temperature + exp(−z/D) × (air
  temperature lagged by 24 z / (2π D) hours − mean), with damping depth D = 0.1 m.

## Consequences
- The analytic drainage case is exact: a saturated column of n cells drains for n hours.
- Drainage is faster than real soils (no conductivity curve); acceptable for pots in M0,
  listed under known limits in the model card.
