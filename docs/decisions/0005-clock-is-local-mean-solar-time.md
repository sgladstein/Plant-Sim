# 0005: The sim clock is local mean solar time; no longitude or time zone

- **Date:** 2026-11-05
- **Status:** accepted (implementer; the plan's scenario format has latitude only)

## Context
Solar geometry needs a time reference. The scenario file (§6.2) gives latitude and a
local start date and hour, but no longitude or time zone.

## Decision
Clock hours in the sim are local mean solar time: 12:00 is when the mean sun crosses
the meridian. The equation of time is applied, so true solar noon drifts by up to
about ±16 minutes over the year, as it does for a real sundial. There is no daylight
saving time. Hourly weather files are read as the same clock.

## Consequences
- Sun position depends only on latitude, date and hour; validated against pvlib SPA at longitude 0, UTC.
- A weather file recorded in civil time is off by the site's longitude offset from its zone meridian
  (up to about an hour); acceptable for M0. If real-site weather becomes important, add
  `longitude_deg` and `utc_offset_h` to the scenario with a schema bump.
