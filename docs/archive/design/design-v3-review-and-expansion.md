# Plant growth simulation: design review and expansion (v3)

Nov 5, 2026 · @Scotttt

This doc reviews the v2 design plan and expands it. The v2 plan stands; this companion adds detail the build needs and fixes the gaps found.

## 1. Review verdict

The v2 plan is sound in its big decisions and should not be reopened. Carbon as the single growth authority, local reserves, the headless-first M0, the two-pot M1, species as data, and Rust with a thin Bevy layer all hold up. The weaknesses are in what the plan leaves undefined: it names models but not the algorithms, step order, or numbers that an implementer (an AI agent working alone) has to pick. Those gaps are where drift and wasted milestones would come from. Each finding below is fixed in a later section of this doc.

| # | Finding | Severity | Why it matters | Fixed in |
| --- | --- | --- | --- | --- |
| F1 | Per-leaf hourly FvCB does not fit the M2 budget. 30 plants with roughly 5,000 leaves, sunlit and shaded, 24 hours a day is about 240,000 coupled gas-exchange solves per sim-day, which is seconds of CPU, not the 20 ms budgeted. | Critical | Either the budget or the physiology design breaks at M2. Deciding now avoids a rewrite. | §2.6: solve FvCB per light-and-temperature class, not per leaf; leaves inherit their class result |
| F2 | The carbon allocation rule is named (sink strength, transport resistance) but not specified. No equation, no priority order, no behavior when supply is short. | Critical | Allocation is the heart of the model and the hardest thing to tune. An unspecified rule gets reinvented every session. | §2.4: concrete two-level algorithm with a deficit carry-over |
| F3 | Seed reserves are missing. The plan starts growth at germination with no carbon source. A bean seedling lives for about a week on cotyledon reserves before leaves pay their way. | High | Without it the first days cannot be simulated and the biomass curve starts wrong. | §2.5 |
| F4 | Reproductive growth is thin. Pods and seeds are mentioned but there is no pod set, abortion under stress, or seed filling. M3 breeding and the drought challenge both depend on seed yield. | High | Yield is the number challenges score and the thing that makes seeds. | §2.7 |
| F5 | "Expansion curve scaled by realized carbon" is ambiguous: does a starved organ grow later, or end up smaller? | High | The two readings give different plants and different renderings. | §2.4: potential from the curve, realized by supply, unmet demand reduces final size, no catch-up |
| F6 | Voxel light at 2 to 5 cm is far too coarse for a 1 cm seedling leaf and too fine for a 1 m plot. Resolution must scale with the plant. | High | A fixed grid gives wrong early light and a slow late grid. | §2.3: grid sized from the plant bounding box, rebuilt daily |
| F7 | Validation names "literature ranges" without datasets, variables, or tolerances. | High | Exit criteria that cannot be measured do not gate anything. | §5 |
| F8 | No step schedule. Hourly and daily processes are listed but the order within a step (light, then water, then gas exchange, then allocation, then growth) is not written down, and order changes results. | Medium | Determinism and conservation checks need a fixed order. | §2.1 |
| F9 | Temperature only drives phenology. Frost kill, heat stress at flowering, and temperature limits on expansion growth are absent, so a heat wave preset does nothing visible. | Medium | A promised preset would be a no-op. | §2.7 |
| F10 | Bean is a legume and the plan reserves nitrogen but never mentions nodulation. | Medium | M2's nitrogen model would be wrong for the first species if it ignored fixation. | §7 M2 |
| F11 | CO2 as an adjustable variable was in v1 and dropped in v2. | Low | It is a cheap, visible science lever, and FvCB already takes it as an input. | §6 experiment panel |
| F12 | The science lenses have no units, scales or data sources written down, and no way for the player to learn what a lens means. | Medium | "Science on display" needs a legend and an explanation, or it is just colors. | §6 |
| F13 | The game loop before M3 is only watching. The two-pot screen is a good proof, but nothing says what a player does for the second hour. | Medium | Sandbox needs at least one goal-shaped activity early. | §6 experiment mode, §7 |
| F14 | Bevy and bevy\_egui versions were unverified at review time, and the plan has no rule for when to lock them. | Low | M0 needs no Bevy, so the lock can wait, but it needs a date. | §8 |
| F15 | No model documentation convention. For AI-written code, each process needs its equations, parameters and sources written next to the code or the next agent re-derives them. | Medium | Model drift risk was named without a concrete control. | §3 (M0 work package 0) |

## 2. Simulation model, expanded

This section turns the v2 model list into an algorithm an implementer can follow without inventing anything. Every number marked \~ is a starting value to be checked against the data in §5, not a fact.

### 2.1 Step schedule

The order inside a step is fixed, because order changes results and the conservation checks depend on it. Hourly steps run 24 times per sim-day. The daily step runs once, at sim-midnight, after the 24th hourly step.

Hourly step, in order:

1. Weather for the hour: air temperature, humidity (converted to vapor pressure deficit), global radiation split into direct and diffuse, rain, cloud fraction.
2. Solar geometry: sun elevation and azimuth from latitude, day of year and hour.
3. Light: absorbed PAR per leaf = stored daily coefficients for this hour's sun position × this hour's direct and diffuse radiation. Yields sunlit fraction and the red:far-red proxy per leaf.
4. Soil water: infiltrate rain and irrigation into the top cells, drain cells above field capacity downward, evaporate from exposed topsoil, update soil temperature.
5. Plant water status: fraction of transpirable soil water (FTSW) over root-occupied cells, weighted by root length density. One stress factor comes out of this and is used in exactly two places: stomatal conductance and expansion growth.
6. Gas exchange per light class (§2.6): coupled Farquhar photosynthesis and Medlyn conductance, giving assimilation and transpiration per unit leaf area. Each leaf copies its class result scaled by its own area.
7. Water uptake: remove transpired water from soil cells in proportion to root length density and cell water above wilting point. If the soil cannot supply the demand, the shortfall is logged; the stress factor already reduced conductance, so no second penalty is applied.
8. Maintenance respiration per organ, scaled by organ nitrogen and a Q10 temperature response, paid from the organ's own reserve.
9. Carbon income: each leaf adds its hourly net assimilation to its own reserve.
10. Transport: one relaxation pass of reserve exchange between neighboring organs (§2.4).
11. Thermal time: each organ and the plant accumulate degree-hours above the species base temperature, with a hot-side cutoff.
12. Growth: each organ computes its potential growth for the hour from its expansion curve, takes what its reserve can pay after growth respiration, updates dimensions and structural mass, and carries any unmet demand forward (§2.4).
13. Ledger: the water and carbon balance accumulators are updated; in debug builds a mismatch beyond 1e-9 relative panics.

Daily step, in order:

1. Phenology: stage transitions from thermal time thresholds (§2.7).
2. Bud fate: each bud activates, stays dormant, or dies, from its light signal, the reserve of its parent internode, and apical dominance (§2.4 fallback rules apply if reserves are not yet tuned).
3. Organ initiation: new internodes, leaves, root segments, flowers and pods are created with birth time = now and their expansion curves set from the species file and the plant's current state.
4. Senescence and shedding: organs past their lifespan, below their light threshold for 5 days, or carbon-negative for the species' starvation window die; part of their carbon is resorbed to the parent before they drop.
5. Root elongation and branching, from the root carbon allocated during the day (§2.9).
6. Light geometry rebuild: voxel grid resized to the plant bounds, leaf area density recomputed, per-leaf coefficients for the next day's 24 sun positions and the diffuse sky recomputed (§2.3).
7. Output: one row of daily summary to the time series, full state hash appended to the golden-run log.

### 2.2 Organ state

Every organ carries the fields below. Integer ids index structs-of-arrays; the graph is parent pointers plus a children list per node, kept in creation order so iteration is deterministic.

| Field | Unit | Notes |
| --- | --- | --- |
| id, parent id, organ type, axis order | - | Type is one of: seed, cotyledon, internode, bud, leaf (blade + petiole), root segment, flower, pod, seed-in-pod |
| birth time | sim hours | Also the start of the organ's expansion curve |
| thermal age | °C·day | Drives the expansion curve and lifespan |
| expansion curve parameters | - | Beta-function shape, duration in °C·day, potential final size |
| length, radius or area | m, m, m² | Rendered quantities; derived from structural mass through species allometry |
| structural mass | g dry matter | Cannot shrink except by resorption at death |
| carbon reserve | g C | Local pool; can go negative briefly, which is the starvation signal |
| nitrogen content | g N | Stored from M0; drives maintenance respiration now and photosynthetic capacity in M2 |
| unmet demand | g C | Cumulative shortfall; shrinks the potential final size (§2.4) |
| light coefficients | - | 24 direct transmittances for the day plus one diffuse coefficient, sunlit fraction, red:far-red proxy |
| water status | - | The plant-level stress factor copied per organ for the inspector and lenses |
| orientation | unit vector, radians | Base direction and phyllotactic angle, updated by tropisms at the daily step |

### 2.3 Light

The turbid-medium voxel model stays, with the resolution problem (F6) fixed by sizing the grid to the plant.

- Grid: an axis-aligned box around all shoot organs with 20% margin, divided into at most 48 cells per axis and no cells smaller than 5 mm. A seedling gets millimeter cells, a mature bean gets 2 to 3 cm cells. The grid is rebuilt in the daily step; coefficients are only valid for one day, which is also how often geometry changes matter.
- Leaf area density per cell is accumulated from each leaf's area and center. Extinction along a path uses Beer's law with a leaf angle G-function; spherical distribution (G = 0.5) is the default and a species parameter.
- Direct beam: for each of the 24 hourly sun positions, march a ray from each leaf's center toward the sun through the grid, accumulating optical depth. Transmittance gives the probability that the leaf is sunlit at that hour. Below the horizon the coefficient is zero.
- Diffuse: 16 sky sectors (two elevation bands × eight azimuths) with a standard overcast sky weighting; one ray per sector per leaf; the diffuse coefficient is the weighted mean transmittance.
- Red:far-red proxy: 1.2 × exp(-0.5 × leaf area index above the leaf), clipped to \[0.2, 1.2\]. Values below \~0.7 switch on shade avoidance (longer internodes, higher specific leaf area) in the architecture module.
- Cost: leaves × 40 rays × \~50 cells per ray. A 300-leaf plant is about 600,000 cell visits per day, well under a millisecond.
- Two pots on one screen use two independent grids unless their boxes overlap, in which case they merge for that day. This is also how a plot works in M2.
- Validation: a single-plant case compared with an offline Monte Carlo ray tracer (Python) to within 10% absorbed PAR per leaf, and total absorbed PAR against Beer's law for a horizontally uniform canopy to within 2%.

### 2.4 Carbon allocation algorithm

This replaces the unspecified sink-strength rule (F2) and settles the expansion curve ambiguity (F5).

Each organ has a reserve R (g C), structural mass W, and a reserve concentration C = R / W. Edges of the organ graph have a conductance k proportional to cross-sectional area divided by length, so thick short connections move carbon easily and long thin ones do not. Roots connect to the shoot through the hypocotyl.

Hourly:

1. Demand. Each organ's potential growth this hour is the derivative of its expansion curve times its (possibly reduced) potential final size, times the temperature growth multiplier (§2.7) and, for expanding organs, the water stress factor. Demand in carbon = potential growth × carbon fraction (0.45 g C per g dry matter) × construction cost (1.25, meaning 20% of carbon used is respired as growth respiration).
2. Priority. Organ types have a base priority: seeds in pods 1.0, pods 0.9, expanding leaves and internodes 0.7, roots 0.6, secondary thickening 0.4, reserve refill 0.2. Roots gain priority under water stress: root priority = 0.6 + 0.4 × (1 - FTSW). This is the one encoded piece of functional balance and is labeled as such in the UI.
3. Transport. One pass over edges: flux from i to j = k\_ij × (C\_i - C\_j) × dt, clamped so no reserve goes below zero from transport alone. This is a crude Münch flow and is enough to create gradients from source leaves to sinks.
4. Growth. Each organ realizes growth = min(demand × priority, max(0, R - safety reserve)) where the safety reserve is 5% of W. Carbon used leaves R; growth respiration goes to the ledger; the rest becomes structural mass.
5. Deficit. Unmet demand accumulates in the organ. Its potential final size is reduced to (1 - unmet / cumulative potential demand) of the species value. There is no catch-up: a leaf that grew in the shade ends smaller, as observed. Rendering interpolates the realized size, so the curve stays smooth.
6. Starvation. An organ whose reserve has been negative after maintenance for the species starvation window (3 days for bean leaves and branches) dies at the next daily step. 30% of its structural carbon is resorbed to its parent, the rest becomes litter. A whole axis dies when its basal internode dies.

What this gives for free: shaded lower leaves and branches starve and drop; drought shifts growth to roots through the priority term and through reduced shoot expansion; fruit load starves late leaves. Those are the §8 emergent tests.

Fallback, decided in advance: if after two tuning sessions in M0 the per-organ transport cannot be made stable or plausible, collapse transport to three pools (each shoot axis, the root system, the reproductive organs) with the same demand and priority rules. Per-organ reserves remain as the starvation signal. The data model does not change.

### 2.5 Seed reserves and germination

The plant starts as a seed organ, not at germination (F3).

- A bean seed is \~0.3 to 0.5 g dry mass, of which \~0.45 is carbon and \~70% of that is mobilizable. The two cotyledons are organs holding that reserve; they are the only carbon source until leaves export.
- Germination requires the seed's soil cell above 30% FTSW and soil temperature above the base temperature for 24 consecutive hours. Emergence follows at \~60 to 80 °C·day above an 8 °C base (about a week at 20 °C); the hypocotyl grows from cotyledon carbon until it reaches the soil surface.
- Cotyledons export carbon down their concentration gradient like any organ, senesce when their reserve is below 10% of the start, and drop about two weeks after emergence. Unifoliate leaves open at emergence and are the first photosynthetic organs.
- Watchability: the seed, the hypocotyl hook straightening, the cotyledons unfolding and the first leaves opening are the first on-screen events, and the camera's auto-framing starts at the seed.

### 2.6 Gas exchange and respiration

The Farquhar–von Caemmerer–Berry model coupled with Medlyn conductance stays, solved per light class rather than per leaf (F1).

- Each hour, each plant bins its leaves' absorbed PAR into 16 classes (log-spaced from 1 to 2,000 µmol m⁻² s⁻¹), separately for sunlit and shaded fractions. Leaf temperature equals air temperature in M0 (a leaf energy balance is an M2 upgrade). The coupled solver (Newton on intercellular CO2) runs once per class; each leaf multiplies the class rate by its area and sunlit or shaded fraction.
- Cost: 30 plants × 32 classes × 24 hours is about 23,000 solves per sim-day, under 25 ms. The 20 ms M2 budget becomes achievable; M0 raises the single-plant budget to 2 ms per sim-day.
- Leaf capacity declines with leaf thermal age after full expansion (linear to 40% at the end of lifespan), which stands in for nitrogen remobilization until M2.
- Starting parameters for bush bean, to be checked in §5: Vcmax at 25 °C \~90 µmol m⁻² s⁻¹, Jmax \~1.7 × Vcmax, dark respiration \~0.015 × Vcmax, Medlyn g1 \~4 kPa^0.5, ambient CO2 420 ppm (adjustable in the experiment panel).
- Maintenance respiration: 0.015 g C per g N per day at 20 °C, Q10 = 2, per organ. Growth respiration: 20% of carbon allocated to growth, as above.

### 2.7 Phenology, temperature and reproduction

- Thermal time uses a three-point response: zero below 8 °C, linear to an optimum at 28 °C, falling to zero at 38 °C (\~ values; cardinal temperatures are species parameters). Hourly accumulation avoids the daily-mean bias.
- Stages for a determinate bush bean follow the standard V and R scale: sowing, emergence, first trifoliate, further vegetative nodes, flowering (R5 or R6 depending on the scale), pod set, seed fill, maturity. Greenhouse measurements put sowing to maturity at about 800 °C·day \[1\]; cultivar cycle lengths run 65 to 93 days \[8\]. Flowering starts around 40% of the cycle.
- Determinate habit: the main stem produces a species-set number of nodes (\~6 to 8 for a bush type) and terminates in an inflorescence. Axillary buds on the lower nodes make branches (\~3 to 6 per plant \[4\]); upper ones make racemes.
- Flowers open over about 10 days per raceme. Each flower sets a pod with probability driven by three things: the raceme's parent internode reserve over the previous three days (carbon), daytime temperature above \~32 °C (heat abortion), and FTSW below 0.3 (drought abortion). Pod number is the yield component drought hits hardest in beans \[2\]\[9\], and this is how the model reproduces that.
- Pods expand over \~20 days to full length, then seeds fill over \~20 days as the strongest sinks. Three to six seeds per pod; final seed mass is a species parameter reduced by unmet demand. Total dry mass per plant of \~20 to 40 g with \~10 to 20 pods is the field range to land in \[4\].
- Temperature on growth (F9): expansion growth has its own multiplier, zero at 5 °C, one from 25 to 30 °C, zero at 40 °C. Leaves die at -1 °C (frost preset). Heat waves abort flowers and raise respiration; both are visible.
- Senescence: leaf lifespan \~600 °C·day; leaves below 10% of full-sun absorbed PAR for 5 days senesce; during seed fill, leaves senesce from the base upward at a rate tied to seed demand (encoded until the M2 nitrogen model makes it emergent). Whole-plant senescence at maturity ends the annual's life.

### 2.8 Soil and water in pots and plots

- The soil grid is 3D with 2 cm cells in M0 and M1. A pot is a mask over the grid: cells outside the pot are inactive. The pot bottom drains freely (holes); a plot's bottom boundary drains to a deep layer that never returns water.
- Texture gives field capacity and wilting point by volume (loam \~0.27 and \~0.12; sand \~0.15 and \~0.05; clay \~0.40 and \~0.22). Transpirable water per cell is the difference.
- Infiltration fills cells from the top until field capacity; drainage moves excess down one cell per hour; lateral redistribution moves 10% of any gradient per hour. Soil evaporation takes a fraction of radiation reaching the soil surface, from the top layer only, reduced as it dries.
- Root uptake potential per cell is root length density × available water × a species uptake rate; actual uptake is the lesser of demand and total potential, distributed in proportion.
- FTSW is computed over cells containing roots, weighted by root length density. The stress factor is 1 above FTSW 0.4 and falls linearly to 0 at 0.
- Soil temperature follows air temperature with a depth-dependent lag (daily mean plus damped diurnal cycle). It drives germination and root elongation.
- Known limits, documented in the model card: no matric potential, no hysteresis, no capillary rise. A retention curve is added only if water potential is shown in a lens.

### 2.9 Roots

- CRootBox-style root types: a tap root, first-order laterals, second-order laterals. Each type has potential elongation rate (cm per day at optimum temperature), maximum length, inter-branch distance, insertion angle, and tropism weights (gravitropism down, hydrotropism toward wetter neighbor cells).
- Root growth is a sink in the allocation: the root system's demand is the sum over tips of potential elongation × carbon per cm from specific root length (\~70 m per g for bean fine roots, \~ value). Realized elongation scales with the carbon the root pool actually received that day.
- Soil temperature and cells at wilting point slow elongation; a tip entering an inactive (pot wall) cell deflects along the wall, which is what real pot roots do.
- Root length density per cell is updated daily and feeds uptake, FTSW weighting and the water lens.

### 2.10 Starting parameters for bush bean

All values are starting points, with their validation target in §5.

| Parameter | Starting value | Source or status |
| --- | --- | --- |
| Seed dry mass | 0.3 to 0.5 g | \~ typical for common bean |
| Base, optimum, maximum temperature | 8, 28, 38 °C | \~; check against §5 phenology data |
| Sowing to maturity | \~800 to 1,200 °C·day; 65 to 93 days | \[1\], \[8\] |
| Main stem nodes before terminal raceme | 6 to 8 | \~ bush type |
| Branches per plant | 3 to 6 | \[4\] |
| Mature trifoliate leaf area | 50 to 100 cm² | \~ |
| Specific leaf area | 200 to 300 cm² per g | \[3\], \[6\] |
| Plant height at maturity | 30 to 50 cm | \[4\] |
| Pods per plant, seeds per pod | 10 to 20; 3 to 6 | \[4\] |
| Total dry mass per plant | 20 to 40 g | \[4\] |
| Vcmax, Jmax at 25 °C | \~90, \~150 µmol m⁻² s⁻¹ | 89 and 160 measured on common bean \[Pimentel 2011\] |
| Medlyn g1 | \~4 kPa^0.5 | \~ herbaceous crop default |
| Leaf lifespan | \~600 °C·day | \~ |
| Specific root length | \~70 m per g | \~ |

## 3. M0 scope and task list: headless core

M0 ends when one bush bean grows from seed to maturity in a headless run, lands inside the §5 targets under four environments, conserves water and carbon, and produces the same hash every run. No graphics. Everything builds and tests in a Linux container.

Work packages in build order. Each later package depends only on earlier ones, so each can be one or two agent sessions with a green CI at the end. Size is relative: S under a session, M one to two, L two to four.

| # | Work package | Crates | Size | Done when |
| --- | --- | --- | --- | --- |
| 0 | Repo scaffold and conventions: Cargo workspace, CI (fmt, clippy pedantic as errors, nextest), a CLAUDE.md with the sim rules (no Bevy below ps-world, no HashMap iteration in the sim, libm only, every quantity a unit newtype), a model card template in docs/models/ (equation, parameters with units and source, encoded or emergent, validation test id), decision records in docs/decisions/ | all | S | CI green on an empty workspace; the first two decision records (Rust and Bevy, carbon authority) committed |
| 1 | Units and RNG: unit newtypes with checked conversions; counter-based RNG keyed by (world seed, entity id, tick, purpose) | ps-units, ps-rng | S | Property tests: same key gives same stream; adding an entity does not change another's stream |
| 2 | Weather and sun: solar position and day length by latitude and date; direct and diffuse split from clearness; presets (temperate summer, overcast, heat wave, drought, frost night); daily-to-hourly disaggregation | ps-weather | M | Sun position within 0.5° of a NOAA reference for 10 test dates; daily radiation totals match the preset |
| 3 | Organ graph: arena, organ types, parent and children lists in creation order, beta expansion curve, allometry | ps-graph | M | Property tests: structural mass never shrinks except at death; expansion curve integrates to the potential size |
| 4 | Species schema: RON files with units checked on load; genotype-to-phenotype with one additive locus per heritable parameter as the first mapping; the bush bean file with a source for every number | ps-species | M | Bean file loads; a deliberately wrong unit fails to load with a clear error |
| 5 | Soil and water: 3D grid, pot mask, infiltration, drainage, lateral redistribution, evaporation, soil temperature, root uptake, FTSW | ps-soil | M | A single wet column drains to field capacity with closure under 1e-9; the analytic infiltration case in §5 matches |
| 6 | Light: adaptive grid, leaf area density, ray marching for 24 sun positions and 16 sky sectors, per-leaf coefficients, red:far-red proxy | ps-light | M | Uniform canopy matches Beer's law within 2%; single-plant case within 10% per leaf of the Python ray tracer fixture |
| 7 | Physiology: coupled Farquhar and Medlyn solver with light classes; maintenance and growth respiration; thermal time; phenology stages; the allocation algorithm of §2.4 | ps-physio | L | Gas exchange within 2% of plantecophys fixtures over the test grid; allocation unit tests: a starving organ dies on schedule, unmet demand shrinks final size |
| 8 | Herb architecture and roots: determinate main stem, axillary branching, phyllotaxy, bud fate, shade avoidance, tropisms; CRootBox-style root types | ps-arch | L | A 60-day run produces node, branch and root counts inside the §2.10 ranges |
| 9 | World: the step schedule of §2.1, action log, snapshot save and load, full-state hash, conservation ledger | ps-world | M | Three runs give one hash; a snapshot resumed at day 30 matches the uninterrupted run's hash at day 90 |
| 10 | Export and CLI: scenario files; `ps-cli run`, `hash`, `bench`; daily time series and per-organ CSV | ps-export, ps-cli | S | The four §5 scenarios run from one command and write CSV |
| 11 | Validation harness: Python scripts that generate the fixtures, read the CSV, compare to the §5 targets and write a plot report; committed as a CI artifact | validation/ | M | The report runs in CI and fails the build when a target is missed |
| 12 | Tuning and lock: adjust the \~ parameters until all §5 M0 targets pass; record golden hashes for Linux and Windows | data/, tests/golden/ | L | Exit criteria below met |

Exit criteria, all measured by the WP11 report:

- Temperate summer scenario, 90 sim-days from sowing: total dry mass, pod count, height, peak leaf area, flowering date and maturity date inside the §5 ranges.
- Shade scenario (50% radiation): internodes at least 20% longer, specific leaf area higher, total mass 30 to 50% lower than the summer run.
- Drought scenario (irrigation withheld from day 30): root to shoot ratio higher than the summer run, pod count down by at least 30%, seed size down by less than pod count.
- Frost-night scenario: leaves die, the plant does not recover. (Cheap to add and proves the temperature path.)
- Water and carbon ledgers close to 1e-9 relative every hour.
- One hash across three Linux runs; Linux and Windows hashes recorded, expected to differ, each stable.
- Under 2 ms per sim-day for the single plant on the CI runner, measured by instruction count.
- A model card exists for every process in §2, and the parameter file cites a source or marks the value as a guess.

## 4. M1 scope and task list: the two-pot visible proof

M1 ends when Scotttt can watch two bean plants from the same seed grow side by side on the GTX 1070, one in sun and one in shade or one watered and one in drought, at up to a sim-week per second, with lenses, an inspector and charts, and the headless replay of the same session gives the same hash as the one with graphics. Nothing in the sim changes in M1 except bugs found by looking at it.

| # | Work package | Crates | Size | Done when |
| --- | --- | --- | --- | --- |
| 13 | Lock the engine: pick the current stable Bevy and a matching bevy\_egui on day one of M1, write docs/bevy-notes.md for that version, build a window-and-cube binary, and have Scotttt run it on the GTX 1070 | ps-app | S | A screenshot from the target machine |
| 14 | Mesh generation: tapered tubes along internode chains with smooth joins; leaf templates (cotyledon, unifoliate, trifoliate with three leaflets, midrib bend and a droop parameter for wilting); flower and pod templates; root segments as thin tubes; evaluation at fractional sim time | ps-mesh | M | Headless tests: vertex and index counts, outward normals, no NaN at size zero, continuity between hours |
| 15 | Scene sync and look: diff the sim snapshot by organ id each frame, instanced leaves, stem meshes rebuilt only when size changes by more than 2%; soil slab with a glass front and visible roots; sky and sun from solar geometry; stylized shading (soft ramp plus rim light, gentle palette); auto-framing orbit camera; high-speed rendering mode above one sim-day per second | ps-view | L | Two 90-day plants render at 60 fps at 1440p on the 1070 |
| 16 | Sim thread: the sim runs on its own thread, the renderer reads a double-buffered snapshot; speed control, pause, and the +hour, +day, +week, +season buttons; actions go through the action log only | ps-app, ps-world | M | Hash of a recorded session equals the hash of its headless replay |
| 17 | UI: top bar (date, season, speed, steps), lens panel with legends, inspector with units, charts panel (biomass by organ, leaf area, height, assimilation, transpiration, water and carbon balance), two-pot layout, experiment panel (radiation %, irrigation schedule, CO2, latitude, presets) | ps-ui | L | Every lens has a legend; every inspector field has a unit |
| 18 | Lenses for M1: light, water (soil and plant), X-ray roots, carbon reserve, phenology (§6) | ps-view, ps-ui | M | Each lens matches the CSV for a clicked organ |
| 19 | Capture: screenshot and time-lapse frame export, CSV export from the app | ps-app | S | Files land next to the save |
| 20 | The moments: wilting and recovery, phototropic lean, cotyledon unfolding, flowering cue; tune the visuals, not the sim | ps-view | M | Scotttt signs off on a 60-second sun-versus-shade clip |

Exit criteria:

- The two-pot sun-versus-shade and wet-versus-drought runs look right to Scotttt and the differences between pots are the ones §3 measured.
- 60 fps at 1440p for two plants on the GTX 1070; under 16 ms frame time at one sim-week per second with the high-speed mode on.
- Session hash equals headless replay hash.
- Lenses have legends and units; the inspector shows the §2.2 fields.
- A 60-second time-lapse clip exported from the app. This is also the first thing worth showing anyone.

## 5. Validation data

Realism is measured at four levels, each with a named reference and a tolerance. The whole-plant level now has real bean data behind it; the gaps that remain are listed at the end.

### 5.1 Process fixtures (unit tests)

| Process | Reference | Test grid | Tolerance |
| --- | --- | --- | --- |
| Coupled Farquhar and Medlyn gas exchange | R package plantecophys, Photosyn() | PAR 50 to 2,000 × temperature 10 to 40 °C × VPD 0.5 to 4 kPa × CO2 280, 420, 800 ppm | 2% on assimilation, conductance and transpiration |
| Solar position and day length | NOAA solar calculator or pvlib | 10 dates × 3 latitudes | 0.5° |
| Light interception, uniform canopy | Beer's law closed form | leaf area index 0.5 to 6, G = 0.5 | 2% on absorbed fraction |
| Light interception, single plant | Own Monte Carlo ray tracer in Python, committed with its seed | 60-day bean geometry exported from the sim | 10% per leaf, 3% on the plant total |
| Soil water | Closed-form drainage of a wet column to field capacity; mass closure | loam, sand, clay | closure 1e-9 relative; drainage time within 5% |
| Thermal time | Hand-computed hourly sums with the three-point response | 5 daily temperature traces | exact to rounding |
| Expansion curve | Integral of the beta function equals potential size | 20 parameter sets (property test) | 1e-6 |

### 5.2 Whole-plant targets for bush bean

These are the ranges the M0 temperate-summer run must land in. Sources are field and greenhouse studies on common bean; cultivars differ, so the targets are ranges, not points.

| Variable | Target | Source |
| --- | --- | --- |
| Sowing to maturity, thermal time | \~800 °C·day in a greenhouse, both seasons | [Pinzón-Sandoval 2024](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop) |
| Thermal sums by BBCH stage | Use as the stage-by-stage check once the paper's table is digitized | [Cavalcante 2020](https://consensus.app/papers/details/f99d70d2ad125b88bd90b297eb6a7a26/?utm_source=claude_desktop) |
| Sowing to maturity, days | 65 (super-early) to 93 (long cycle) at greenhouse temperatures | [Conceição 2026](https://consensus.app/papers/details/c2e8163fd8c45570a67062d5bb53982d/?utm_source=claude_desktop) |
| Emergence, flowering, maturity in a warm climate, days after planting | 4.5, 28.7, 54.2 | [Ventura Elías 2016](https://consensus.app/papers/details/f12981865fff580d98042cf80f5d8496/?utm_source=claude_desktop) |
| Dry mass accumulation shape | Logistic (sigmoidal) for the plant and each organ class | [Pinzón-Sandoval 2024](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop) |
| Height, branches, leaves, leaf area, dry mass at harvest | 35 cm; 5.5 branches; 20 leaves; 0.12 m²; 21.8 g per plant (best sowing date) | [Kumar 2025](https://consensus.app/papers/details/0acea9340180597f8465c3c495e4b046/?utm_source=claude_desktop) |
| Pods per plant, seeds per pod, seed mass per plant | 11.8; 3.7; 8.8 g | [Kumar 2025](https://consensus.app/papers/details/0acea9340180597f8465c3c495e4b046/?utm_source=claude_desktop) |
| Specific leaf area | \~240 cm² per g (highest genotype); declines with age | [Gebeyehu 2019](https://consensus.app/papers/details/59277c779b9d5689aab68ff3e5ddea22/?utm_source=claude_desktop), [Merga 2021](https://consensus.app/papers/details/b9a69a6d643158bba3217253856933f5/?utm_source=claude_desktop) |
| Net assimilation rate, early growth | up to 41 mg per dm² per day (4.1 g m⁻² day⁻¹) | [Merga 2021](https://consensus.app/papers/details/b9a69a6d643158bba3217253856933f5/?utm_source=claude_desktop) |
| Vcmax, Jmax, dark respiration, mesophyll conductance at 25 °C | 89.3, 159.8, 0.88 µmol m⁻² s⁻¹; 0.47 mol m⁻² s⁻¹ | [Pimentel 2011](https://consensus.app/papers/details/879c7fe0f71a538b8407a5c7943b21ef/?utm_source=claude_desktop); herbaceous annual means 75 and 154 in [Wullschleger 1993](https://consensus.app/papers/details/d5f18c7251575a80b536d7a8d4178160/?utm_source=claude_desktop) |

### 5.3 Response targets (the four scenarios)

| Scenario | Expected response | Source | Tag |
| --- | --- | --- | --- |
| Drought from flowering | Pod number falls more than seed size; leaf area falls more than other growth variables; shoot mass, leaf area index and crop growth rate all fall | [Gomes](https://consensus.app/papers/details/b97819d779ef59e28cc7d3e5a87920d1/?utm_source=claude_desktop), [Guerrero-Domínguez](https://consensus.app/papers/details/1504985d05765a26ab566f8216abf698/?utm_source=claude_desktop) | Pod abortion encoded; magnitude emergent |
| Drought, allocation | Root to shoot ratio rises | general; magnitude from the sim | Partly encoded (root priority term) |
| Heat at flowering (32 °C days for 5 to 7 days) | Pod and seed set fall through pollen failure; total seed weight down 50 to 71%, single seed weight down 8 to 34% depending on tolerance | [Gross 1994](https://consensus.app/papers/details/d0e154d8f0035b178b28813744fe424a/?utm_source=claude_desktop), [Sharda 2026](https://consensus.app/papers/details/0f6080758ba55e87a0bba510b6043b75/?utm_source=claude_desktop) | Encoded abortion rule; the two seed-weight numbers are the check |
| 50% shade | Longer internodes, higher specific leaf area, lower total mass | shade-avoidance literature, no bean-specific number yet | Encoded trigger, emergent magnitude |
| Dense planting (M2) | Leaf area and leaf nitrogen per area plateau at 20 plants per m² and above | [Clavijo Michelangeli 2019](https://consensus.app/papers/details/11b9dd4ab6fd51aa9d2da96ca8314961/?utm_source=claude_desktop) | Emergent (competition) |
| Lower shaded leaves | Drop after sustained shading | emergent from local carbon | Emergent |

Pass rule: a range target passes if the sim lands inside it; the biomass curve passes if a logistic fit has R² above 0.95; a response target passes if the direction matches and the magnitude is within a factor of two of the cited value. Tighter tolerances come with better data.

### 5.4 Regression

Golden-run hashes per platform, the validation report as a CI artifact, and a weekly scheduled run of the full scenario set so drift shows up even when nobody touched the sim.

### 5.5 Gaps and how to fill them

- No per-organ weekly time series is in hand yet. The best candidate is the weekly destructive-harvest data behind [Pinzón-Sandoval 2024](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop); digitizing its figures is an M0 WP11 task.
- The DSSAT CROPGRO-Drybean cultivar and species files are a documented, public process model of the same crop and would make a good second reference for phenology and partitioning. Availability and license to be confirmed before relying on them.
- Shade avoidance in bean has no number attached yet. An M0 task is to find one internode-length-under-shade study, or accept direction-only for M0.
- Root architecture parameters for bean (elongation rates, branching density) are still starting guesses. CRootBox ships example parameter sets that can be checked for a Phaseolus entry.
- A windowsill bean. Scotttt growing a real bush bean next to the sim, with weekly photos and a ruler, is cheap, fun, and gives a height and leaf-count series for the exact conditions the sim is shown in. Optional but recommended.

## 6. UI and lenses

The v2 layout stands (top bar, tools left, lenses and inspector right, charts below). This section specifies the lenses so they teach rather than decorate (F12), and adds the experiment mode that gives the sandbox a goal before M3 (F13).

### 6.1 Lens catalogue

Every lens has a legend with units, a two-sentence explanation of what the colors mean, an encoded-or-emergent tag, and a link to the model card for the process it shows. Sequential scales use viridis; diverging scales use a blue-white-orange ramp; both are colorblind-safe. In high-speed mode (above one sim-day per second) instantaneous quantities switch to daily totals or means so the picture stops flickering.

| Lens | What is colored | Quantity and unit | Scale | Milestone |
| --- | --- | --- | --- | --- |
| Light | Leaves | Absorbed PAR, µmol m⁻² s⁻¹ (instant) or mol m⁻² day⁻¹ (high speed); hatching for the shaded fraction | Sequential, 0 to 2,000 | M1 |
| Soil water | Soil cells on the cut face and in X-ray | Fraction of transpirable soil water, 0 to 1, with field capacity and wilting point marked on the legend | Sequential blue | M1 |
| Plant water | Whole plant tint plus leaf droop | Stress factor 0 to 1 | Sequential | M1 |
| Roots X-ray | Soil fades to 15% opacity; roots by age or by today's uptake | Age in days, or uptake in g water per day | Sequential | M1 |
| Carbon | Organs by reserve concentration; later, flow arrows along the graph | g C per g dry matter, centered on the safety reserve | Diverging | M1 (arrows M2) |
| Phenology | Organs by thermal age; stage badge on the plant | °C·day; stage name | Sequential | M1 |
| Shade signal | Leaves and internodes by red:far-red proxy | ratio 0.2 to 1.2 | Diverging around 0.7 | M1 (cheap) |
| Temperature | Soil cells and air gradient | °C | Diverging around the species optimum | M2 |
| Nitrogen | Soil cells and leaves | g N per m² soil; g N per m² leaf | Sequential | M2 |
| Competition | Each plant by the share of its light taken by neighbors | fraction 0 to 1 | Sequential | M2 |

### 6.2 Inspector

Clicking an organ shows the §2.2 fields with units, plus one generated "why" line built from state, for example: "Shaded: 12% of full sun today. Reserve falling for 2 days. Drops in about 1 day if nothing changes." This line is what makes the local-carbon rule legible without a textbook. Hovering any field names the model card it comes from.

### 6.3 Charts

The bottom panel plots, per plant and overlaid across pots in different line styles: dry mass by organ class, leaf area, height, daily assimilation and respiration, daily transpiration and uptake, soil water in the root zone, and the carbon and water balance residuals (which should be flat at zero and are a visible honesty check). A click on a chart point jumps the replay to that day.

### 6.4 Time controls

Speeds: 1 sim-hour per second, 6, then 1 day, 3 days and 1 week per second. Step buttons: +hour, +day, +week, +season. A replay scrubber from day 0 to now, driven by the action log and snapshots. Above one day per second the high-speed rendering mode engages automatically and the lens quantities switch to daily values.

### 6.5 Experiment mode

This generalizes the two-pot screen and is the sandbox's first goal-shaped activity.

- Two to four pots, same seed, one variable differing: radiation fraction, irrigation schedule, CO2, latitude, sowing date, or a weather preset. The variable is chosen from the experiment panel.
- A lab notebook records the setup, pins the model version and world seed, and exports CSV and a plot of the comparison when the run ends.
- Experiment cards offer prompts before M3's scored challenges exist: "Make the shaded plant catch up", "Find the irrigation that gives the most pods with the least water", "Grow a bean at 60° north". No scoring, just a stated question and the charts to answer it.
- Any mature plant's seeds can be saved to a seed library from M1 on. The library is data only until M3, but it means nothing grown before breeding exists is lost.

### 6.6 Visual rules

- Lenses never change geometry, only color and opacity, so a lens can be toggled mid-run without a visual jump.
- Legends are always visible while a lens is on.
- The glass front of the soil slab is the default; X-ray is a lens, not a mode.
- Plant state that has a visual (wilt, yellowing, browning, leaf drop, flowering, pod fill, ripening) shows with lenses off, so the plain view already tells the story.

## 7. Later milestones in more detail

One reordering is recommended: the tree moves from M2 to M4. A tree is a hidden second project (years of sim time, perennial carbon storage, dormancy, wood, secondary growth, leaf fall) and it pays off in the forest, not the plot. M2 instead adds two cheap, very different growth forms that reuse the bean work.

### M2: plot and sandbox

- Plot: a soil block of 1 × 1 m to 3 × 3 m with 5 cm soil cells, 5 to 30 plants, one merged light grid. Light coefficients move from per-leaf rays to per-voxel transmittance (each voxel stores its transmittance for 24 sun positions and the diffuse sky; leaves read their voxel), which cuts the cost from leaves × rays to voxels × directions and makes 30 plants cheap.
- Second growth form, pole bean: the same species file with one habit locus flipped from determinate to indeterminate, plus the vine module (tendril search, trellis attachment, continued node production). This shows genotype-to-phenotype doing real work a year before breeding, and the trellis is the first obstacle type.
- Third growth form, a grass: tillering from the base, intercalary growth, fibrous roots. Chosen because it looks nothing like a bean and tests that the engine is general.
- Nitrogen: a mineral nitrogen pool per soil cell with mineralization from organic matter, leaching with drainage, and root uptake by concentration and root length. Leaf nitrogen per area drives Vcmax linearly, and nitrogen remobilization during seed fill makes bottom-up senescence emergent instead of encoded. Fertilize becomes an action. Bean is a legume (F10): nodules are a root-attached organ that costs carbon (\~10 to 15% of assimilate when active) and fixes nitrogen when soil nitrogen is low, so the fertilizer-versus-fixation trade-off is emergent and visible in the carbon lens.
- Prune and harvest: cutting an internode removes its subtree and releases apical dominance, so axillary buds below activate without a special rule. Harvesting pods removes sinks and extends leaf life (also emergent).
- Weather files: hourly CSV import with a documented column format, plus latitude and season controls. A handful of named real-climate presets ship with the app.
- Save and load UI, replay scrubber, the competition lens, soil temperature and nitrogen lenses.
- Exit: 30 plants at 60 fps on the GTX 1070 and under 20 ms per sim-day; leaf area per unit ground plateaus with density as in §5.3; pruning visibly releases branching; nodulated beans grow in nitrogen-free soil and unnodulated ones do not.

### M3: seeds, breeding and challenges

Genetics is designed so that breeding is a real search, not a slot machine, and so that every trade-off comes from physiology already in the model.

- Genome: diploid, 20 to 40 loci. Each heritable parameter is the sum of one to three loci; each locus has an additive effect and a dominance deviation. Loci are unlinked at first (independent assortment), which keeps the math simple and the outcomes rich enough.
- Species as populations: a species file defines allele distributions per locus, so two seeds of "bush bean" differ slightly. The M0 identity mapping is the degenerate case of this.
- Mutation: per locus per gamete at 1 in 1,000, with small Gaussian effect changes. Mutation is what makes long breeding programs keep finding new things.
- Pollination as a game action: bean self-pollinates, so a cross requires the player to hand-pollinate a flower with pollen from another plant. This is historically accurate, gives breeding a deliberate feel, and makes selfing the default that purifies lines.
- Trade-offs from physiology, never from a table: higher Vcmax costs nitrogen and maintenance respiration; bigger leaves cost specific leaf area and shade the plant's own lower leaves; more seeds per pod means smaller seeds under a fixed pod carbon supply; a deep tap root costs shoot growth early. The breeding bench shows a trait only after the player has grown and measured it, so discovery is earned.
- Breeding bench: grow up to ten seeds headless in parallel pots and get a results card with charts, then pick parents. The watchable diorama stays for the plants the player cares about; the bench is for throughput. A bean generation at one sim-week per second is about ten seconds, so the loop is fast either way.
- Seed library: each seed stores genotype, parent ids, the environment it grew in, and a viability that declines with storage time.
- Challenges: a challenge file names a scenario, a scored metric read from the CSV (pods, seed mass, water used, days to maturity), constraints (water budget, no fertilizer), a world seed and a pinned model version. Scores are reproducible by replay. First set: most pods under a 150 mm water budget; fill a trellis by day 60; a bean that matures at 60° north; the heaviest single seed.
- Exit: a cross between two lines shows F2 variation in the bench charts; one challenge is playable start to finish with a replay.

### M4: fields, forests and the tree

- Tree module: self-organizing bud fate driven by the light signal and local carbon, pipe-model secondary growth as a carbon sink, perennial storage in wood and roots, dormancy and leaf fall by photoperiod and temperature, frost hardiness. Years-long runs with seasons.
- Aggregate level: per patch (1 to 10 m²) and per species cohort, the state is leaf area index, height, biomass by organ class and root mass by soil layer. Canopy photosynthesis uses the two-leaf (sunlit and shaded) model with the same gas-exchange code path and parameters as the organ level, which is what makes the two levels agree. Patches compete through height-sorted light layering, gap-model style. Soil is one column per patch.
- Rendering at distance: procedural plants generated from cohort statistics with the same mesh templates, instanced, with impostors beyond a distance. Zooming in promotes a patch's cohort to organ-level plants that match the conserved quantities.
- Promotion and demotion test: carbon and nitrogen per organ class, leaf area index, height and root mass per layer preserved within 5% through a demote-promote round trip, or M4 detail is declared visual-only as v2 allows.
- Exit: a one-hectare field or a small mixed stand at 60 fps; zoom-in promotion with the conservation test passing; a ten-year forest run with recognizable seasons.

## 8. Risk register

The v2 risks (realism versus performance, parameter explosion, model drift, scope creep, Bevy churn, determinism traps, cloud rendering, egui looks, compile times) stand. These are additional, with the controls this doc adds.

| # | Risk | Likelihood | Impact | Control |
| --- | --- | --- | --- | --- |
| R1 | Allocation tuning spirals: per-organ transport is unstable or implausible and eats sessions | Medium | High | Fallback to three pools decided in advance (§2.4); time-boxed to two tuning sessions |
| R2 | Parameters lose their provenance as agents edit files | High | Medium | Every parameter carries a source field or the word guess; CI fails on a missing field; model cards per process |
| R3 | Scotttt sees nothing for the whole of M0 | High | Medium | WP11 plots from the first package onward; a Python 3D snapshot of the organ graph as a poor man's view; run the WP13 Bevy smoke test during M0, not after |
| R4 | Windows is the target but development is on Linux | Certain | Medium | Windows CI job builds and runs sim tests and records its own golden hash; only rendering checks need the GTX 1070 |
| R5 | Per-leaf light rays do not scale to a plot | Medium | Medium | Per-voxel coefficients at M2 (§7) |
| R6 | The tree module swallows M2 | High | High | Tree moved to M4; M2 growth forms are pole bean and a grass |
| R7 | Real species name invites comparison the model cannot win | Medium | Low | Call it a bush bean archetype, cite the ranges it is tuned to, show them in the model card |
| R8 | Linux and Windows hashes differ | Likely | Low | Expected; two golden hashes; same-platform determinism is the guarantee |
| R9 | Context loss between agent sessions re-derives decisions | High | Medium | CLAUDE.md with the rules; decision records; one GitHub issue per work package with its done-when; this doc as the backlog |
| R10 | Saves break across model versions | Certain over time | Low | Replay re-simulates with a warning; challenges pin the version; snapshots are a cache, never the truth |
| R11 | Bevy and bevy\_egui versions mismatch at M1 start | Medium | Low | Lock on day one of M1 (WP13), note the versions in docs/bevy-notes.md, upgrade only between milestones |
| R12 | egui charts are not good enough for the science panel | Low | Medium | egui\_plot covers M1; fall back to rendering plots to textures with plotters if needed |
| R13 | The sandbox is boring between M1 and M3 | Medium | Medium | Experiment mode and experiment cards in M1 (§6.5); seed library from M1 |
| R14 | Heat, frost and shade presets do nothing visible | Medium | Medium | Temperature growth multiplier, frost kill and heat abortion in M0 (§2.7); shade avoidance tested in M0 |

## 9. Open questions and what changed

Decisions that are Scotttt's, with a recommendation each. Everything else in this doc is a default the build will follow unless changed.

- [ ] Move the tree to M4 and make pole bean and a grass the M2 growth forms? Recommended yes (§7, R6).
- [ ] Hand-pollination as the breeding action in M3, with selfing as the default? Recommended yes; it is how beans work and it gives breeding a deliberate feel.
- [ ] Grow a real bush bean on a windowsill during M0 and M1 for a photo series to compare against? Optional, recommended.
- [ ] Merge this doc into the design plan once reviewed, or keep it as the companion? Recommended: merge after review so there is one plan.
- [ ] Start M0 work package 0 now, or wait for comments on both docs? Recommended: wait for a read of this doc, then start.

What changed from v2:

- Fifteen review findings, two of them critical: gas exchange cost at plot scale, and the unspecified allocation rule. Both fixed in §2.
- The simulation now has a fixed step schedule, an organ state table, a concrete allocation algorithm with a pre-decided fallback, seed reserves and germination, light-class gas exchange, reproduction with pod set and abortion, temperature effects beyond phenology, and a starting parameter table for bush bean.
- M0 and M1 are broken into 21 work packages in build order, each with a done-when, and the exit criteria are numbers.
- Validation now names datasets, variables, tolerances and tags, with real bean studies behind the whole-plant targets, and lists the data still missing.
- Lenses have units, scales and legends; the inspector has a generated why line; an experiment mode gives the sandbox a goal before M3.
- The tree moves to M4; M2 gains pole bean (one habit locus) and a grass; nodulation enters the nitrogen model; M3 genetics is specified; M4 reuses the gas-exchange code path at the aggregate level.
- Fourteen new risks with controls, several aimed at the fact that an AI writes the code across many sessions.

## 10. Sources

Project documents:

- [Plant growth simulation: design plan (v2)](https://claude.ai/artifact/UkAzFjE4Y3BNRDpZM9QMyR)
- [Plant growth simulation: options and priorities (research report)](https://claude.ai/artifact/NR9fpmSdNqFZCNpteHCeGf)

Studies cited (found through Consensus; abstracts read, full texts not yet opened):

- [Phenology, mass accumulation patterns and growing degree days in common bean](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop) (Pinzón-Sandoval et al., 2024, Revista de Ciencias Agrícolas)
- [Thermal sum and phenological descriptions of growth stages of the common bean according to the BBCH scale](https://consensus.app/papers/details/f99d70d2ad125b88bd90b297eb6a7a26/?utm_source=claude_desktop) (Cavalcante et al., 2020, Annals of Applied Biology)
- [Interaction among growth cycle, photosynthesis, and nodule enzymatic activity in Phaseolus vulgaris inoculated with Rhizobium tropici CIAT 899](https://consensus.app/papers/details/c2e8163fd8c45570a67062d5bb53982d/?utm_source=claude_desktop) (Conceição et al., 2026, Annals of Applied Biology)
- [Fenología y fenometría de una variedad y una línea de frijol en la zona occidental de El Salvador](https://consensus.app/papers/details/f12981865fff580d98042cf80f5d8496/?utm_source=claude_desktop) (Ventura Elías, 2016, Agronomía Mesoamericana)
- [Growth performance and yield contributing traits of Kharif French bean varieties under different dates of sowing](https://consensus.app/papers/details/0acea9340180597f8465c3c495e4b046/?utm_source=claude_desktop) (Kumar et al., 2025, International Journal of Research in Agronomy)
- [Evaluation of growth performance of common bean genotypes during two phenological phases](https://consensus.app/papers/details/59277c779b9d5689aab68ff3e5ddea22/?utm_source=claude_desktop) (Gebeyehu, 2019)
- [Evaluation of some haricot bean cultivars for growth characters and dry matter production](https://consensus.app/papers/details/b9a69a6d643158bba3217253856933f5/?utm_source=claude_desktop) (Merga, 2021, Advanced Journal of Plant Biology)
- [Características fotossintéticas de Phaseolus vulgaris L.](https://consensus.app/papers/details/879c7fe0f71a538b8407a5c7943b21ef/?utm_source=claude_desktop) (Pimentel et al., 2011)
- [Biochemical limitations to carbon assimilation in C3 plants: a retrospective analysis of the A/Ci curves from 109 species](https://consensus.app/papers/details/d5f18c7251575a80b536d7a8d4178160/?utm_source=claude_desktop) (Wullschleger, 1993, Journal of Experimental Botany)
- [Accumulation of biomass, physiological characteristics and grain yield of bean cultivars under irrigated and dry regimes](https://consensus.app/papers/details/b97819d779ef59e28cc7d3e5a87920d1/?utm_source=claude_desktop) (Gomes et al.)
- [Behavior of common bean plants subjected to two irrigation systems](https://consensus.app/papers/details/1504985d05765a26ab566f8216abf698/?utm_source=claude_desktop) (Guerrero-Domínguez et al.)
- [Differential sensitivity to high temperature of stages in the reproductive development of common bean](https://consensus.app/papers/details/d0e154d8f0035b178b28813744fe424a/?utm_source=claude_desktop) (Gross and Kigel, 1994, Field Crops Research)
- [Multi-trait phenotyping under reproductive-stage heat stress reveals heritable reproductive and yield indicators of tolerance in common bean](https://consensus.app/papers/details/0f6080758ba55e87a0bba510b6043b75/?utm_source=claude_desktop) (Sharda et al., 2026, Frontiers in Plant Science)
- [Influence of plant density and growth habit of common bean on leaf area development and N accumulation](https://consensus.app/papers/details/11b9dd4ab6fd51aa9d2da96ca8314961/?utm_source=claude_desktop) (Clavijo Michelangeli et al., 2019, Journal of Crop Improvement)

Reference software named in the plan and not yet opened in this review: plantecophys (R), CRootBox and CPlantBox, DSSAT CROPGRO-Drybean, pvlib. Confirm availability and license before the work packages that depend on them.
