# 0034: Leaf energy balance (WP27)

- **Date:** 2026-11-05
- **Status:** accepted (owner decisions 2026-11-05: leaf frost reads leaf temperature,
  leaves only; Medlyn g1 5.79 first, decision 0032)

## Context
Until now a leaf was at air temperature (design plan §3.8: "an M2 upgrade"). WP27 (#105)
adds the leaf energy balance, so that sunlight, sky longwave, wind and transpiration set
each leaf's temperature, and that temperature drives its gas exchange and frost. The
research thread's package (`docs/research/wp27-leaf-energy-balance`, #93) gives the
formulation, the parameters, the acceptance cases and a Python prototype; it found that
with g1 = 4 a well-watered bean leaf on a hot dry day sat at or above air temperature,
against every field baseline, which is why g1 = 5.79 (decision 0032) came first.

## Decision
1. **Weather gains wind** (`HourWeather.wind`, m s⁻¹ at 2 m). Hourly CSV files keep their
   optional `wind_m_s` column; daily files take an optional daily mean; presets and files
   without wind use 2 m s⁻¹ (FAO-56). A daily mean is spread over the day with DSSAT's
   HWIND (night 0.3 of the noon peak, a sine by day), which keeps the daily mean. Cloud
   fraction, so far display-only, now sets the sky's longwave emissivity (Brutsaert 1975
   with the Unsworth and Monteith cloud correction); night hours of an hourly file keep
   the last daylight hour's value.
2. **The balance** (`ps-physio::energy`), per one-sided leaf area, both faces exchanging
   heat and longwave: absorbed shortwave from absorbed PAR and the species' NIR
   absorptance; longwave with the leaf's sky view (its diffuse-sky transmittance);
   forced convection `1.4 × 0.00328 √(u/d)` plus Leuning free convection per face;
   stomata in series with each face's boundary layer (`g_bw = 1.075 g_bh`), faces in
   parallel with the species' abaxial share of conductance. Solved by the linearised
   Penman–Monteith form, then two Newton steps on the full balance.
3. **Coupled class table.** Each of the 68 nodes solves gas exchange and the energy
   balance together for a horizontal leaf in the open at 0.5 m in the hour's air and
   wind: stomata see the leaf temperature and the leaf-to-air deficit. The root of
   `T_eb(gs(T)) − T` is found by a fixed-point step from a first guess (the neighboring
   nodes' answers), clamped secant steps until bracketed, then Illinois, until 0.01 K or
   8 gas-exchange solves (the research recommended 1e-3 K; 0.01 K costs under 0.05% in A
   and saves about a third of the solves). A dark hour solves only the PAR-0 node and
   copies it (a lookup at PAR 0 reads the same bits). The table key adds the energy
   parameters; the wind is the soil's, as the rest of the hour's weather.
4. **Per leaf.** Each leaf part (sunlit, shaded) solves its own temperature and
   transpiration from its own absorbed PAR, its sky view, its own wind
   (`u2 · ln(z/0.02)/ln(2/0.02) · √τ` at its height `z`, floored at 0.1 m s⁻¹) and the
   conductance it read (scaled with assimilation by the sink feedback). Its water
   demand is that transpiration, at least 0. Assimilation and day respiration stay the
   table's.
5. **What reads leaf temperature:** FvCB kinetics, Rd, stomata, transpiration, and leaf
   frost: a leaf dies when its own lowest temperature of the day is below
   `leaf_frost_kill` (owner decision). Growing points, buds and flowers still freeze on
   the day's minimum air temperature, and thermal time, leaf expansion, stem maintenance
   and the pod-set heat response stay on air.
6. **Species schema 11:** `leaf_width` (bean 0.08 m, wheat 0.015 m), `leaf_nir_absorptance`
   0.20, `leaf_emissivity` 0.97, `abaxial_conductance_share` (bean 0.875, wheat 0.5),
   each with a source. Model 0.16.0 → 0.17.0, goldens regenerated.

## Consequences
- WP27 done-when (`ps_physio::energy::tests`): hot still day −1.17 K (pass −7 to +1); dry
  air and wind −0.31 to −1.33 K (pass < 0); closed stomata +5.44 K, 6.6 K above case 1;
  clear calm night −1.51 K (pass −3 to −0.5); irrigated hot wheat −1.83 to −1.97 K (pass
  < 0); soil uptake equals the plant's demand to 1e-9 every hour of 30 days
  (`ps-arch/tests/leaf_energy.rs`).
- Radiation frost is emergent: air at −1.5 °C on a clear, calm night freezes the exposed
  leaves of a 25-day bean (3 of 5) and spares the inner ones and the apices.
- Validation on four-seed means at 0.17.0: 87 checks, **67 pass** (0.16.0: 68), every
  M0 check passes. One Info check slips: `harvest_index_tops` 0.415 → 0.392 (band 0.4 to
  0.6). Sunlit leaves in a temperate summer run a few kelvin above air, nearer the
  optimum of Vcmax and Jmax, and transpire less with the boundary layer in series, so
  the pot plant makes more: total dry mass at maturity 23.4 → 24.7 g, pods 10.5 → 13.0;
  with seed number fixed at pod set, seed size takes it (0.259 → 0.235 g per seed), the
  known missing seed abortion rule (decision 0032), which also lowers the harvest index.
  Responses keep their verdicts and move a few points: drought seed yield −49% →
  −44%, drought shoot mass −46% → −43%, drought root:shoot +26% → +23% (less water
  used), heat pods −20% → −24%, shade pods −56% → −53%.
- Speed, this container, interleaved runs: plot_30_beans 32.0 to 33.4 → 34.2 to 34.9 ms
  per sim-day (about +5%), wheat_stand 17.4 to 17.5 → 17.4 to 18.0 s (+1 to 2%),
  single_pot 0.29 → 0.36 to 0.40 s. A lit coupled table costs about 80 µs (68 nodes,
  about 1.7 gas-exchange solves each) against 12 µs uncoupled; dark hours solve one
  node. A soil's tables are now built in parallel, one per distinct key (collect keeps
  their order, so the bits are the same); the timings include it.
- Known limits (model card): assimilation carries the reference leaf's temperature, not
  each leaf's own; the near infrared is not enriched in shade; no CO2 boundary layer; a
  leaf short of water does not warm for the water it did not get; one leaf width per
  species; leaves do not fold away from the sun.
- The daily rows (`DayReport`, daily.csv) gain `assimilation_gc`, the day's net leaf
  assimilation after leaf day respiration and the sink feedback, and `transpiration_kg`,
  the water the soil gave the plant (demand less shortfall), for the viewer's WP17
  charts. The plant's day totals are state, so they are in the hash.
- The research's next check stands: a validation target for midday gs of well-watered
  field bean (0.4 to 0.8 mol m⁻² s⁻¹) once it has a source.
