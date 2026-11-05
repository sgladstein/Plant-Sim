# 0012: Sparse root lists, one joint uptake per hour, and an uptake ceiling

- **Date:** 2026-11-05
- **Status:** accepted

## Context

WP5 gave `SoilGrid::ftsw` and `SoilGrid::take_up` one root length per grid cell per
plant, and `take_up` was called once per plant. The performance review measured 215 ms
per sim-day for 30 plants on a plot, ten times the whole plot budget: a plant's roots
fill a few percent of the grid, but every call walked all of it and summed it twice for
the ledger. Calling plants one after another also made the first plant drink first from
a shared cell. Separately, the research thread found that uptake needs a ceiling per unit
root length (about 0.03 cm³ of water per cm of root per day); in moist loam the WP5
coefficient alone allows about 14 times that.

## Decision

- Roots enter the soil as `RootCells`: `(cell, length)` pairs sorted by cell, one per
  cell, positive lengths. Sums over them equal the old dense sums.
- `take_up` takes every plant's request at once and computes all of them from the same
  soil state. Where the requests to one cell exceed its available water, they are all
  scaled by the same factor. Plant order no longer matters (tested by swapping it).
- Each cell's potential gains the ceiling `u_max · L`, with `u_max` a new species
  constant `root_uptake_max` (kg m⁻¹ h⁻¹; bean 1.25e-4, "~" until the research report
  cites it).
- `M2PerHour` and `KgPerMH` newtypes carry the two uptake parameters (WP5 review).

## Consequences

- The world (WP9) builds each plant's `RootCells` from its root segments and calls
  `take_up` once per hour, after every plant's transpiration demand is known.
- A cell whose water is oversubscribed does not hand its leftover to plants that asked
  less elsewhere; the shortfall is logged as before.
- Species schema 1 gains `root_uptake_max`.
