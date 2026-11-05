# Plant growth simulation: design plan v3 (complete)

2026-11-05 · Scotttt (owner)

This is the single, complete plan. It merges the v2 design plan with the v3 review and expansion and applies the decisions of 2026-11-05: the tree moves to M4, M2 adds pole bean and a grass, hand-pollination is the M3 breeding action, and no real-plant experiment is planned. It is written for an implementer starting cold: a Claude session with this doc, the repo, and nothing else. Values marked \~ are starting points to be checked against the validation targets in §8.

## 1. Vision, goals and non-goals

A simulation-first plant growth sandbox. You watch plants grow from seed in sped-up time, and their form emerges from real, inspectable rules (light, water, nitrogen, carbon) rather than scripted animation. Three goals carry equal weight:

- **Beautiful to watch.** A cozy, stylized 3D diorama with emergent plant shapes; botanical illustration meets a cozy indie game.
- **Scientifically believable.** Established plant-science models (Farquhar photosynthesis, Medlyn stomata, turbid-medium light, bucket soil water, source-sink carbon allocation, thermal-time phenology), with the internals visible through toggleable lenses and charts. "As realistic as possible" is defined as measurable tests (§8), not a feeling.
- **Fun to play.** Sandbox and experiments first, then scored challenges, then seed collection and breeding as the long-term hook.

Scale grows over the milestones: one plant in a pot, two pots side by side, a plot of up to 30 plants, then fields and forests at a coarser level of detail.

Target machine: Windows PC, GeForce GTX 1070, 1080p or 1440p. Desktop only. Claude writes all the code; the owner reviews, plays, and decides.

### Non-goals (for now)

- Photorealism.
- Web or shareable-link distribution.
- Multiplayer, monetization, mobile.
- Agronomic decision-support accuracy. The model is calibrated to literature ranges for a bush bean archetype, not to a named cultivar's yield.
- Real-plant experiments by the owner. All calibration data comes from published studies.

### How to read this document

§2 is the experience. §3 is the model, written as algorithms with step order and starting numbers. §4 to §6 are visuals, UI and data formats. §7 is the backlog: milestones broken into work packages with done-when criteria. §8 says how realism is measured. §9 lists risks. §10 is the architecture decision. §11 is the implementation guide for the session that starts the code. §12 records every decision made so far so none is reopened by accident.

## 2. Core experience

### 2.1 The diorama view

- A soil block with one or more plants on top. Orbit, pan and zoom with the mouse; the camera auto-frames the plant's bounding box by default because a plant grows about a thousandfold in size from seed to maturity. Manual camera overrides auto-framing until reset.
- Roots are shown rhizobox-style: the soil block is a slab with a glass front, so most roots are visible through the glass, and an X-ray lens fades the soil to show the whole 3D root system.
- Sun position follows latitude, date and hour; day and night; weather (rain, cloud, temperature, humidity). Wind is a visual sway only, no mechanics.
- In M1 the block is two pots side by side (same seed, one variable different). In M2 it is a plot. In M4 it is terrain.

### 2.2 Time controls

- Continuous simulation with speed multipliers: 1 sim-hour per second, 6 hours, 1 day, 3 days, 1 week per second.
- Pause, and step buttons: +1 hour, +1 day, +1 week, +1 season. Steps run headless as fast as possible, then the view refreshes.
- High-speed rendering mode engages automatically above 1 sim-day per second: lighting shows the daily mean rather than the hour, stress visuals are smoothed, and lens quantities switch to daily totals, so there is no day-night strobing or wilt flicker.
- Replay scrubber from day 0 to now, driven by the action log and snapshots (§3.14), not stored video.

### 2.3 Designed moments

These are the things a viewer should see happen on their own, and each has a visual and a test:

- The seed splits, the hypocotyl hook straightens, the cotyledons unfold, the first leaves open (§3.10).
- Leaves and stems lean toward the light; branches flush on the sunny side.
- Wilting under drought and visible recovery within sim-hours of watering.
- Shaded lower leaves yellow and drop while upper leaves thrive.
- Flowering as an event with a camera cue; pods swell and fill; the plant senesces at maturity.
- Side-by-side pots diverge visibly within a sim-week.

### 2.4 Player actions

| Action | Milestone | Effect in the model |
| --- | --- | --- |
| Sow a seed at a location | M1 | Creates a seed organ in a soil cell (§3.10) |
| Water (can) or set an irrigation schedule | M1 | Adds water to the top soil cells at the action's tick |
| Choose radiation fraction, latitude, season, CO2, and weather presets (temperate summer, overcast, heat wave, drought, frost night) | M1 | Changes the weather and sun inputs |
| Harvest pods | M1 | Removes pod organs; seeds go to the seed library |
| Fertilize | M2 | Adds mineral nitrogen to soil cells |
| Prune | M2 | Removes an internode's subtree; apical dominance is released, lower buds activate (emergent) |
| Place obstacles and trellises | M2 | Occluders in the light grid; attachment targets for vines |
| Import a weather file | M2 | Hourly CSV drives the weather |
| Hand-pollinate a flower with another plant's pollen | M3 | A cross; otherwise flowers self-pollinate |
| Define and run a challenge | M3 | Scenario with a scored metric and a pinned model version |

Every action is appended to the action log with its sim tick. The renderer never changes sim state directly.

### 2.5 Experiment mode

The two-pot screen generalizes into experiment mode, the sandbox's first goal-shaped activity and the main science tool.

- Two to four pots, same seed, one variable differing: radiation fraction, irrigation schedule, CO2, latitude, sowing date, or a weather preset.
- A lab notebook records the setup, pins the model version and world seed, overlays the pots' charts, and exports CSV plus a comparison plot when the run ends.
- Experiment cards offer prompts before scored challenges exist: "Make the shaded plant catch up", "Find the irrigation that gives the most pods with the least water", "Grow a bean at 60° north". No scoring, a stated question and the charts to answer it.
- Seeds from any mature plant can be saved to the seed library from M1 on, so nothing grown before breeding exists is lost.

### 2.6 Progression

- **Sandbox (M1).** Watch, water, change the environment, compare pots.
- **Plot (M2).** Plant a mixed plot, prune, fertilize, build a trellis, import your town's weather.
- **Breeding (M3).** Save seeds, hand-pollinate crosses, grow lines on the breeding bench, discover traits that come from physiology (§3.12).
- **Challenges (M3).** Scored scenarios pinned to a model version and seed, reproducible by replay.
- **Fields and forests (M4).** Zoom out to a field or a stand of trees over years; zoom in to promote a patch to full detail.

## 3. Simulation model

### 3.1 Principles (locked)

1. **Carbon is the single authority on how much grows.** Architecture modules decide only where growth goes and what shape it takes. Light and shade signals set bud activation probabilities and sink priorities; they never create biomass. (LIGNUM and L-PEACH style functional-structural split.) Nitrogen joins as a second authority in M2.
2. **Carbon is local.** Reserves are stored per organ and moved between neighboring organs by a conductance-weighted exchange, so a shaded branch can run a deficit and die on its own. Self-pruning is emergent, not scripted.
3. **Headless sim, separate from rendering.** The sim produces state; the renderer reads a snapshot and sends actions through the action log.
4. **Deterministic.** World seed, model version and action log are the source of truth (§3.14).
5. **One engine, architecture modules per growth form** (herb, vine, grass, tree), all sharing the organ graph, physiology, light and soil.
6. **Organs grow as functions of time**: a birth time plus an expansion curve, so rendering, interpolation and replay are smooth.
7. **Every quantity has a unit.** Code uses unit newtypes; data files carry units and are checked on load.
8. **A fixed step order.** Hourly and daily processes run in the order of §3.2, always.

### 3.2 Step schedule

Hourly steps run 24 times per sim-day. The daily step runs once, at sim-midnight, after the 24th hourly step.

Hourly step, in order (in the code, steps 3, 5 and 6 run from the soil as it stands before step 4, so step 7 draws on the hour's infiltrated water; decision 0014):

1. Weather for the hour: air temperature, humidity (converted to vapor pressure deficit), global radiation split into direct and diffuse, rain, cloud fraction.
2. Solar geometry: sun elevation and azimuth from latitude, day of year and hour.
3. Light: absorbed PAR per leaf = the stored daily coefficient for this hour's sun position × this hour's direct radiation, plus the diffuse coefficient × diffuse radiation. Yields sunlit fraction and the red:far-red proxy per leaf.
4. Soil water: infiltrate rain and irrigation into the top cells, drain cells above field capacity downward, redistribute laterally, evaporate from exposed topsoil, update soil temperature.
5. Plant water status: fraction of transpirable soil water (FTSW) over root-occupied cells, weighted by root length density. One stress factor results and is used in exactly two places: stomatal conductance and expansion growth.
6. Gas exchange per light class (§3.8): coupled Farquhar photosynthesis and Medlyn conductance give assimilation and transpiration per unit leaf area per class. Each leaf copies its class result scaled by its area and sunlit or shaded fraction.
7. Water uptake: remove transpired water from soil cells in proportion to root length density and water above wilting point. A shortfall is logged; no second penalty is applied because the stress factor already reduced conductance.
8. Maintenance respiration per organ, paid from the organ's own reserve.
9. Carbon income: each leaf adds its hourly net assimilation to its own reserve.
10. Transport: one relaxation pass of reserve exchange between neighboring organs (§3.9).
11. Thermal time: each organ and the plant accumulate degree-hours with the development temperature response (§3.11; a trapezoid since decision 0032).
12. Growth: each organ computes potential growth for the hour from its expansion curve, takes what its reserve can pay after growth respiration, updates dimensions and structural mass, and carries unmet demand forward (§3.9).
13. Ledger: water and carbon balance accumulators update; in debug builds a relative mismatch above 1e-9 panics.

From M2 (WP24, decision 0020), nitrogen adds to the hourly steps: step 4 also mineralizes and nitrifies soil nitrogen and moves nitrate with the drained and laterally moved water; nodules fix nitrogen, paid from their reserve, just before step 8; and after step 12 growing organs are topped up with nitrogen from the plant's pool. An organ diluted to its final nitrogen concentration does not grow that hour.

Daily step, in order (from M2, a step 0 runs first: each soil's joint nitrogen uptake, then each plant's nitrogen supply, mining, fixation target, nodulation, leaf drop and nitrogen status; decision 0020):

1. Phenology: stage transitions from thermal time thresholds and stress (§3.11).
2. Bud fate: each bud activates, stays dormant, or dies, from its light signal, its parent internode's reserve, and apical dominance.
3. Organ initiation: new internodes, leaves, root segments, flowers and pods are created with birth time = now and their expansion curves set from the species file and the plant's current state (shade avoidance modifies internode and leaf targets here).
4. Senescence and shedding: organs past their lifespan, below their light threshold for 5 days, or carbon-negative for the species' starvation window die; 30% of their structural carbon is resorbed to the parent before they drop; the rest becomes litter.
5. Root elongation and branching from the root carbon allocated during the day (§3.7).
6. Light geometry rebuild: voxel grid resized to plant bounds, leaf area density recomputed, per-leaf coefficients for the next day's 24 sun positions and the diffuse sky recomputed (§3.5).
7. Output: one daily summary row to the time series; full-state hash appended to the golden-run log.

### 3.3 Plant structure and organ state

An MTG-like organ graph stored as structs-of-arrays indexed by integer ids, with parent pointers and per-node children lists kept in creation order so iteration is deterministic. Organ types: seed, cotyledon, internode, bud, leaf (blade plus petiole), root segment, flower, pod, seed-in-pod, and in M2 nodule. (Decision 0023: common bean twines, so there is no tendril organ; a twining internode carries its support and coil instead. `OrganKind::Tendril` stays unused until the next model bump removes it.)

| Field | Unit | Notes |
| --- | --- | --- |
| id, parent id, organ type, axis order | - | Axis order 0 is the main stem or tap root |
| birth time | sim hours | Start of the expansion curve |
| thermal age | °C·day | Drives expansion and lifespan |
| expansion curve parameters | - | Beta-function shape, duration in °C·day, potential final size |
| length, radius or area | m, m, m² | Rendered quantities; derived from structural mass through species allometry |
| structural mass | g dry matter | Never shrinks except by resorption at death |
| carbon reserve | g C | Local pool; may go negative briefly, which is the starvation signal |
| nitrogen content | g N | Stored from M0; drives maintenance respiration now and photosynthetic capacity in M2 |
| unmet demand | g C | Cumulative shortfall; shrinks potential final size |
| light coefficients | - | 24 direct transmittances for the day, one diffuse coefficient, sunlit fraction, red:far-red proxy |
| water status | - | Plant-level stress factor copied per organ for lenses and inspector |
| orientation | unit vector | Base direction and phyllotactic angle; updated by tropisms at the daily step |

Buds carry: dormancy state, light signal (yesterday's absorbed PAR at the bud's position relative to full sun), apical dominance signal (decays with distance from the nearest active tip along the axis), and the fate roll's RNG purpose key.

### 3.4 Architecture modules

Each module implements one trait (§11) with daily bud-fate and organ-initiation rules. All share phyllotaxis, tropisms and shade avoidance.

- **Herb, determinate (M0): bush bean.** Epigeal germination with two cotyledons, a pair of unifoliate leaves, then alternate trifoliate leaves. The main stem produces a species-set number of nodes (\~6 to 8) and terminates in a raceme. Axillary buds on lower nodes make branches (\~3 to 6 per plant); upper nodes make racemes. Bud activation probability rises with the bud's light signal and its parent internode's reserve concentration and falls with apical dominance.
- **Vine, indeterminate (M2): pole bean.** The same species file with the habit locus flipped: the main stem keeps producing nodes, internodes are longer, and a tendril module searches a cone around the tip for a support within reach, attaches, and wraps. Without a support the stem flops and shades itself (emergent penalty). (WP22, decision 0021: common bean twines rather than using tendrils, so the module is a circumnutation sweep, capture and helical coil; the flop penalty in the sim comes mostly from pods rotting on the soil.)
- **Grass (M2): spring wheat** (decision 0023). Tillering from basal buds, intercalary growth (leaves elongate from the base), fibrous root system with many first-order roots and no tap root. Built so perennial ryegrass is a parameter swap plus perenniality (decision 0022).
- **Tree (M4).** Self-organizing bud fate driven by the light signal and local carbon (Palubicki spirit, no space-colonization markers); pipe-model secondary growth as a carbon sink; perennial storage in wood and roots; dormancy and leaf fall by photoperiod and temperature; frost hardiness.
- **Roots (all).** CRootBox-style parametric root types (§3.7).

Shared rules:

- **Phyllotaxis:** species angle (bean: alternate, \~137.5° after the unifoliates) applied to the parent's azimuth.
- **Tropisms:** phototropism bends new internodes toward the light-weighted direction from the daily coefficients; gravitropism pulls shoots up and roots down; roots also follow hydrotropism toward wetter neighbor cells. Applied as small daily rotations of the organ's orientation, bounded per day.
- **Shade avoidance:** when the red:far-red proxy at an initiating node is below 0.7, the new internode's potential length is multiplied by up to 1.5 and the new leaf's specific leaf area by up to 1.3 (linear in the proxy). This is encoded; the magnitude of the whole-plant response is emergent.
- **Secondary growth:** herbs use stem allometry from allocated carbon (radius from supported leaf area). Trees use pipe-model targets as a sink.

### 3.5 Light

Turbid-medium voxel model (Beer's law with a leaf-angle G-function, RATP and Sinoquet style), multi-rate: geometry coefficients daily, radiation hourly.

- **Grid.** An axis-aligned box around all shoot organs with 20% margin, at most 32 cells per axis (amended from 48 by decision 0010) and no cell smaller than 5 mm. A seedling gets millimeter cells; a mature bean gets 3 to 4 cm cells. Rebuilt in the daily step.
- **Leaf area density** per cell is accumulated from each leaf's area at its center. Extinction along a path uses the G-function; spherical distribution (G = 0.5) is the default and a species parameter.
- **Direct beam.** For each of the 24 hourly sun positions, march a ray from each leaf's center toward the sun through the grid, accumulating optical depth. The transmittance is the probability the leaf is sunlit that hour. Below the horizon it is zero.
- **Diffuse.** 16 sky sectors (two elevation bands × eight azimuths) with a standard overcast sky weighting; one ray per sector per leaf; the diffuse coefficient is the weighted mean transmittance.
- **Sunlit and shaded fractions.** Each leaf is split by its direct transmittance; the two fractions feed different light classes in gas exchange.
- **Red:far-red proxy.** 1.2 × exp(-0.5 × leaf area index above the leaf), clipped to \[0.2, 1.2\].
- **Cost.** Leaves × 40 rays × \~50 cells per ray; a 300-leaf plant is about 600,000 cell visits per day, under a millisecond. At M2 the coefficients move from per-leaf rays to per-voxel transmittance (each voxel stores 24 direct and one diffuse transmittance; leaves read their voxel), which scales as voxels × directions regardless of plant count. (Decision 0019 measured per-voxel coefficients at WP21 and kept per-leaf rays on a shared plot grid; per-voxel light may return for trees at M4.)
- **Multiple plants.** Pots use independent grids unless their boxes overlap, in which case they merge for that day. A plot uses one grid. Obstacles and trellises are opaque or partly transmitting cells.
- **Validation.** Uniform canopy against Beer's law within 2%; single plant against an offline Monte Carlo ray tracer within 10% per leaf and 3% on the total.

### 3.6 Soil and water

- **Grid.** 3D cells, 2 cm in pots (M0, M1) and 5 cm in plots (M2). A pot is a mask: cells outside it are inactive. Pot bottoms drain freely; a plot's bottom drains to a deep layer that never returns water. Default M1 pot: 20 cm diameter, 20 cm deep, loam.
- **Texture** gives field capacity and wilting point by volume: loam \~0.27 and \~0.12, sand \~0.15 and \~0.05, clay \~0.40 and \~0.22. Transpirable water per cell is the difference.
- **Infiltration** fills cells from the top until field capacity; **drainage** moves excess down one cell per hour; **lateral redistribution** moves 10% of any gradient per hour; **evaporation** takes a fraction of radiation reaching the soil surface from the top layer only, reduced as it dries.
- **Uptake.** Potential uptake per cell = root length density × available water × a species uptake rate; actual uptake is the lesser of demand and total potential, distributed in proportion.
- **FTSW** over root-occupied cells weighted by root length density. Stress factor = 1 above FTSW 0.4, falling linearly to 0 at 0 (the 0.4 threshold is a species parameter).
- **Soil temperature** follows air temperature with a depth-dependent lag (daily mean plus a damped diurnal cycle); drives germination and root elongation.
- **Nitrogen (M2).** Mineral nitrogen pool per cell; mineralization from an organic matter pool at a temperature- and moisture-dependent rate; leaching with drainage; root uptake by Michaelis-Menten in concentration × root length. Leaf nitrogen per area drives Vcmax linearly. Nodules (bean) are root-attached organs that cost \~10 to 15% of assimilate when active and fix nitrogen when soil mineral nitrogen is low.
- **Known limits**, written in the model card: no matric potential, no hysteresis, no capillary rise. A retention curve is added only if water potential is ever shown in a lens.

### 3.7 Roots

- Root types in the CRootBox style: tap root (order 0), first-order laterals, second-order laterals; grasses replace the tap root with many order-0 roots from the crown. Each type has potential elongation rate (cm per day at optimum temperature), maximum length, inter-branch distance, insertion angle, apical unbranched zone, and tropism weights.
- Root growth is a sink in allocation: the root system's demand is the sum over tips of potential elongation × carbon per cm from specific root length (\~70 m per g for bean fine roots). Realized elongation scales with the carbon the root pool received that day.
- Soil temperature and cells at wilting point slow elongation; a tip entering an inactive cell (pot wall) deflects along the wall.
- Root length density per cell is updated daily and feeds uptake, FTSW weighting and the water lens.

### 3.8 Gas exchange and respiration

- **Coupled Farquhar–von Caemmerer–Berry photosynthesis with Medlyn stomatal conductance**, solved by Newton iteration on intercellular CO2, giving assimilation, conductance and transpiration together.
- **Per light class, not per leaf.** Each hour each plant bins its leaves' absorbed PAR into 16 log-spaced classes from 1 to 2,000 µmol m⁻² s⁻¹, separately for sunlit and shaded fractions. Leaf temperature equals air temperature in M0; a leaf energy balance is an M2 upgrade. The solver runs once per class; each leaf multiplies the class rate by its area and fraction. Cost: 30 plants × 32 classes × 24 hours ≈ 23,000 solves per sim-day, well under the M2 budget.
- **Leaf aging.** Capacity declines linearly with thermal age after full expansion to 40% at the end of lifespan; stands in for nitrogen remobilization until M2, when leaf nitrogen takes over.
- **Starting parameters for bean** (§3.16): Vcmax 89, Jmax 160, dark respiration 0.9 µmol m⁻² s⁻¹ at 25 °C, Medlyn g1 \~4 kPa^0.5, CO2 420 ppm adjustable.
- **Sink feedback (provisional, decision 0013)**: gross assimilation falls linearly to zero as the leaf's reserve concentration rises from 0.10 to 0.30 g C per g DM; the daily report logs the share lost so missing sinks show up.
- **Maintenance respiration**: 0.218 g C per g N per day at 20 °C (Ryan 1991; amended from 0.015, a per-dry-mass value, by decision 0011) with Q10 = 2, per organ, paid from the organ reserve. **Growth respiration**: 20% of carbon allocated to growth (construction cost 1.25).

### 3.9 Carbon allocation

Each organ has reserve R (g C), structural mass W (g dry matter) and concentration C = R / W. Each graph edge has conductance k proportional to cross-sectional area over length. Roots connect to the shoot through the hypocotyl.

Hourly:

1. **Demand.** Potential growth this hour = the increment of the organ's expansion curve over the hour on its expansion clock (thermal time × the temperature growth multiplier of §3.11, so cold delays growth rather than destroying it; decision 0011) × the species potential final size ×, for expanding organs, the water stress factor. Demand in carbon = potential growth × 0.45 g C per g dry matter × 1.25.
2. **Priority.** Seeds in pods 1.0, pods 0.9, expanding leaves and internodes 0.7, roots 0.6 + 0.4 × (1 - FTSW), secondary thickening 0.4, reserve refill 0.2. The root term is the one encoded piece of functional balance and is labeled as such in the UI.
3. **Transport.** One backward-Euler diffusion step per hour on the organ tree, solved exactly in O(n): flux from i to j = k\_ij × (C\_i' - C\_j') × dt at the end-of-step concentrations. Unconditionally stable and monotone, so transport never takes a reserve below zero (amended from one explicit pass by decision 0011). A crude Münch flow, enough to produce gradients from source leaves to sinks.
4. **Growth.** Realized growth = min(demand, priority × max(0, R - 0.05 W)) (amended by decision 0009: priority is the share of the available reserve an organ spends per hour, so carbon-rich organs grow at potential). Carbon used leaves R; growth respiration goes to the ledger; the rest becomes structural mass.
5. **Deficit.** Unmet demand accumulates in the organ; potential final size becomes (1 - unmet / cumulative potential demand) × the species value. No catch-up: a leaf that grew in shade ends smaller.
6. **Starvation.** An organ whose reserve has been negative after maintenance for the species starvation window (3 days for bean leaves and branches; 14 days for tree branches) dies at the next daily step; 30% of its structural carbon is resorbed to its parent, which also pays the dead organ's negative reserve (decision 0011); the rest is litter. An axis dies when its basal internode dies.

Emergent consequences this is designed to give: shaded lower leaves and branches starve and drop; drought shifts growth to roots; fruit load starves late leaves and ends vegetative growth; pruning releases buds below the cut.

**Fallback, decided in advance.** If after two tuning sessions in M0 the per-organ transport cannot be made stable or plausible, collapse transport to three pools (each shoot axis, the root system, the reproductive organs) with the same demand and priority rules; per-organ reserves stay as the starvation signal. The data model does not change.

### 3.10 Seed reserves and germination

- The plant starts as a seed of four organs (decision 0013): the collar where roots attach, a hypocotyl that starts expanding at germination, and two cotyledons holding the reserve. A bean seed is \~0.3 to 0.5 g dry mass, \~0.45 carbon, \~70% of it mobilizable. The two cotyledons hold that reserve and are the only carbon source until leaves export.
- Germination requires the seed's soil cell above 30% FTSW and soil temperature above base temperature for 24 hours; a dry or cold hour pauses the count rather than resetting it (decision 0029; the plan had 24 consecutive hours). Emergence follows at \~60 to 80 °C·day (about a week at 20 °C); the hypocotyl grows on cotyledon carbon until it reaches the surface.
- Cotyledons export carbon down the concentration gradient like any organ, senesce when their reserve falls below 10% of the start, and drop about two weeks after emergence. Unifoliate leaves open at emergence.

### 3.11 Phenology, temperature and reproduction

- **Thermal time**: a trapezoid, zero below the base, linear to an optimum plateau, falling to zero at the maximum (species cardinal temperatures, accumulated hourly). The bean uses 7 / 25 / 33 / 42 °C (decision 0032; the plan had a three-point response, 8 / 28 / 38 °C); wheat keeps a single optimum.
- **Stages** on the standard V and R scale: sowing, emergence, first trifoliate, further vegetative nodes, flowering, pod set, seed fill, maturity. Phase thresholds count thermal time from germination, so a seed waiting in dry or cold soil does not flower early (decision 0011). Greenhouse data put sowing to maturity near 800 °C·day; cycle lengths of 65 to 93 days; flowering near 40% of the cycle (§8.2).
- **Flowers** open over \~10 days per raceme. Each flower sets a pod with a probability driven by: the raceme's parent internode reserve over the previous three days (carbon), daytime temperature above \~32 °C (heat abortion through pollen failure), and FTSW below 0.3 (drought abortion). Pod number is the yield component drought and heat hit hardest in beans, and this is how the model reproduces it.
- **Pods** expand over \~20 days to full length, then **seeds** fill over \~20 days as the strongest sinks; 3 to 6 seeds per pod; final seed mass is a species parameter reduced by unmet demand.
- **Temperature on growth**: an expansion multiplier, zero at 5 °C, one from 25 to 30 °C, zero at 40 °C. Leaves die at -1 °C (frost preset). Heat waves abort flowers and raise respiration.
- **Senescence**: leaf lifespan \~600 °C·day; leaves below 10% of full-sun absorbed PAR for 5 days senesce; during seed fill leaves senesce from the base upward at a rate tied to seed demand (encoded in M0; emergent via nitrogen remobilization in M2). Whole-plant senescence at maturity ends the annual's life.
- **Pollination (M3)**: flowers self-pollinate by default. A hand-pollination action records a cross (pollen parent id) on the flower; its seeds then carry recombined genotypes (§3.12).

### 3.12 Species, genotype and phenotype

- **Genotype**: diploid, 20 to 40 loci by M3. Each heritable parameter is the sum of one to three loci; each locus has an additive effect and a dominance deviation. Loci are lightly linked (decision 0023): each has a chromosome (1 to 11) and a position in cM, and meiosis draws Haldane crossovers, Poisson with mean length / 100 cM per chromosome, keyed by (world seed, plant id, tick, "meiosis", chromosome). Unlinked loci are the special case. *fin* and *Ppd* share Pv01, *St* and *I* Pv02, and *P*, phaseolin and seed size Pv07.
- **Species as populations**: a species file defines allele distributions per locus, so two seeds of "bush bean" differ slightly. M0 ships the degenerate case: one locus per heritable parameter, zero variance, so the mapping is identity. Pole bean is the bush bean file with the habit locus set to indeterminate.
- **Phenotype**: bounded heritable parameters (branching, leaf size, specific leaf area, Vcmax, root depth tendency, phenology thresholds, seed size and number, habit) computed from the genotype by mapping functions. Fixed physiological constants (Rubisco kinetics, carbon fraction, construction cost) are separate and not heritable.
- **Mutation** (two tiers, decision 0023): anonymous QTL 1 in 1,000 per locus per gamete with small Gaussian changes (SD 0.1 to 0.2 of the allelic SD); named Mendelian loci 1 in 100,000. A game approximation, said so in the model card.
- **Seed and flower colour** (decision 0023): a visual-trait rule table outside the additive map (P gate, then T, then C/J/V base colour, B/G/Rk intensity and pattern loci) drives only `appearance`.
- **Trade-offs from physiology, never from a table**: higher Vcmax costs nitrogen and maintenance respiration; bigger leaves cost specific leaf area and shade the plant's own lower leaves; more seeds per pod means smaller seeds under a fixed pod supply; a deep tap root costs early shoot growth.
- Species files are human-readable, unit-tagged and schema-versioned (§6.1).

### 3.13 Organ growth as a function of time

Every organ has a birth time and a beta-function expansion curve over thermal time (GreenLab and ADEL style), scaled by its potential final size as reduced by unmet demand. The renderer evaluates organ size at fractional sim time, so time-lapse is smooth, nothing pops, and replay can scrub.

### 3.14 Determinism

- **Source of truth**: world seed + model version + action log (player actions timestamped in sim ticks). Snapshots are a cache for fast loading and scrubbing, never the truth.
- **Random streams**: counter-based RNG keyed by (world seed, entity id, tick, purpose), so adding a plant does not perturb another's draws.
- **Fixed-order reductions** for any parallel sums; ordered maps only in the sim; no HashMap iteration.
- **Guarantee**: bit-exact on the same build and platform; cross-platform within tolerance. Sim math on CPU in f64 with libm; the GPU only renders.
- **Golden-run hash test** in CI, per platform.

### 3.15 Scale and level of detail

- Plant and plot: full organ model.
- Fields and forests (M4): per patch (1 to 10 m²) and per species cohort, state is leaf area index, height, biomass by organ class and root mass by layer. Canopy photosynthesis uses the two-leaf (sunlit and shaded) model with the same gas-exchange code and parameters as the organ level. Patches compete through height-sorted light layering, gap-model style; soil is one column per patch.
- **Promotion is a plausible regeneration, not the true history.** Conserved quantities (carbon and nitrogen per organ class, leaf area index, height, root mass per layer) must survive a demote-promote round trip within 5%, tested. If that proves too hard, M4 detail is visual-only.

### 3.16 Starting parameters for bush bean

| Parameter | Starting value | Source or status |
| --- | --- | --- |
| Seed dry mass | 0.3 to 0.5 g | \~ typical for common bean |
| Base, optimum, maximum temperature | 8, 28, 38 °C | \~; check against §8.2 phenology data |
| Sowing to maturity | \~800 to 1,200 °C·day; 65 to 93 days | §8.2 |
| Main stem nodes before terminal raceme | 6 to 8 | \~ bush type |
| Branches per plant | 3 to 6 | §8.2 |
| Mature trifoliate leaf area | 50 to 100 cm² | \~ |
| Specific leaf area | 200 to 300 cm² per g | §8.2 |
| Plant height at maturity | 30 to 50 cm | §8.2 |
| Pods per plant; seeds per pod | 10 to 20; 3 to 6 | §8.2 |
| Total dry mass per plant | 20 to 40 g | §8.2 |
| Vcmax, Jmax, Rd at 25 °C | 89, 160, 0.9 µmol m⁻² s⁻¹ | measured on common bean, §8.2 |
| Medlyn g1 | \~4 kPa^0.5 | \~ herbaceous crop default |
| Leaf lifespan | \~600 °C·day | \~ |
| Specific root length | \~70 m per g | \~ |
| Starvation window | 3 days | \~ |
| Phyllotaxis | alternate, \~137.5° | botany |

## 4. Visual design

- **Style.** Stylized 3D: botanical illustration meets cozy indie game. Soft lighting (a smooth shading ramp plus a rim light), gentle palette, real branch structure, leaves simple enough to render thousands.
- **Geometry from the sim.** Stems are tapered tubes along internode chains with smooth joins; leaves are instanced templates (cotyledon, unifoliate, trifoliate with three leaflets and a midrib bend) scaled and oriented by sim state; flowers, pods and seeds are templates; roots are thin tubes. Everything is evaluated at fractional sim time from the expansion curves.
- **Physiological cues with lenses off.** Wilting (a droop parameter on the leaf template driven by the stress factor), nitrogen deficiency yellowing (M2), senescence browning, leaf drop, flowering, pod swelling and ripening colors. The plain view already tells the story.
- **Soil slab.** A glass front shows roots in the slab; soil layers are tinted by texture; the X-ray lens fades soil to 15% opacity.
- **Sky and sun** from solar geometry; day and night; cloud cover dims and softens; rain is a particle effect; wind is a vertex sway.
- **High-speed mode** above one sim-day per second: daily mean lighting, smoothed stress visuals.
- **Lenses** recolor and change opacity only, never geometry, so toggling never jumps. Scales are colorblind-safe (viridis sequential, blue-white-orange diverging) and legends are always visible while a lens is on.
- **Performance.** 60 fps at 1440p on a GTX 1070 for two plants (M1) and 30 plants (M2); stem meshes are rebuilt only when size changes by more than 2%; leaves are GPU-instanced; impostors beyond a distance in M4.
- **Theme.** A custom egui theme (fonts, rounded panels, muted colors) so the UI matches the diorama rather than looking like a debug tool.

## 5. UI and lenses

### 5.1 Layout

Top bar: date, season, speed control, step buttons, pause. Left: tools (sow, water, harvest; M2 adds fertilize, prune, obstacles). Right: lens panel with legends, and the inspector. Bottom: collapsible charts panel. Menu: save, load, CSV export, screenshot, time-lapse capture, experiment panel, seed library.

### 5.2 Lens catalogue

Every lens has a legend with units, a two-sentence explanation of what the colors mean, an encoded-or-emergent tag, and a link to the model card of the process it shows. In high-speed mode instantaneous quantities switch to daily totals or means.

| Lens | What is colored | Quantity and unit | Scale | Milestone |
| --- | --- | --- | --- | --- |
| Light | Leaves | Absorbed PAR, µmol m⁻² s⁻¹ (instant) or mol m⁻² day⁻¹ (high speed); hatching for the shaded fraction | Sequential, 0 to 2,000 | M1 |
| Soil water | Soil cells on the cut face and in X-ray | Fraction of transpirable soil water 0 to 1, field capacity and wilting point marked | Sequential blue | M1 |
| Plant water | Whole plant tint plus leaf droop | Stress factor 0 to 1 | Sequential | M1 |
| Roots X-ray | Soil fades to 15% opacity; roots by age or by today's uptake | days; g water per day | Sequential | M1 |
| Carbon | Organs by reserve concentration; flow arrows along edges sized by flux | g C per g dry matter, centered on the 5% safety reserve | Diverging | M1 (arrows M2) |
| Phenology | Organs by thermal age; stage badge on the plant | °C·day; stage name | Sequential | M1 |
| Shade signal | Leaves and internodes by red:far-red proxy | ratio 0.2 to 1.2 | Diverging around 0.7 | M1 |
| Temperature | Soil cells and an air gradient | °C | Diverging around the species optimum | M2 |
| Nitrogen | Soil cells and leaves | g N per m² soil; g N per m² leaf | Sequential | M2 |
| Competition | Each plant by the share of its light taken by neighbors | fraction 0 to 1 | Sequential | M2 |
| Genotype | Plants by the value of a chosen heritable trait | trait units | Sequential | M3 |

### 5.3 Inspector

Clicking an organ shows the §3.3 fields with units, plus one generated "why" line built from state, for example: "Shaded: 12% of full sun today. Reserve falling for 2 days. Drops in about 1 day if nothing changes." Hovering any field names the model card it comes from. Clicking a plant shows plant-level totals, stage, FTSW and today's carbon budget (income, respiration, growth, change in reserves).

### 5.4 Charts

Per plant, overlaid across pots in different line styles: dry mass by organ class, leaf area, height, daily assimilation and respiration, daily transpiration and uptake, root-zone soil water, and the carbon and water balance residuals (flat at zero; a visible honesty check). Clicking a chart point jumps the replay to that day. Charts read the same daily time series the CSV export writes.

### 5.5 Time controls

As in §2.2. The replay scrubber shows action markers (watering, presets) on the timeline.

### 5.6 Experiment mode and lab notebook

As in §2.5. The notebook entry is a small RON file next to the save: variable, levels per pot, model version, world seed, start and end dates, and the exported CSV paths. Experiment cards are data files too (§6.5), so new prompts need no code.

### 5.7 Model cards in the app

Each process has a model card (docs/models/\*.md in the repo) with the equation, parameters with units and sources, the encoded-or-emergent tag, known limits, and the validation test id. The app shows the same markdown in a side panel from any lens legend or inspector field. This is how "science on display" becomes learnable rather than decorative.

### 5.8 Visual rules

- Lenses change color and opacity only, never geometry.
- Legends are always visible while a lens is on; units everywhere.
- The glass front is the default; X-ray is a lens, not a mode.
- Nothing in the UI edits sim state directly; every control emits an action.

## 6. Data formats

All data files are RON (Rusty Object Notation) read through serde, human-readable, unit-tagged and schema-versioned. Every numeric parameter is a record with value, unit and source, so provenance survives edits and CI can check it.

### 6.1 Species file (data/species/bush\_bean.ron)

```ron
Species(
  schema: 1,
  id: "bush_bean",
  name: "Bush bean archetype",
  growth_form: Herb,
  constants: (            // not heritable
    carbon_fraction: (value: 0.45, unit: "g C / g DM", source: "typical"),
    construction_cost: (value: 1.25, unit: "g C / g C", source: "~"),
    g_function: (value: 0.5, unit: "-", source: "spherical leaf angles"),
  ),
  heritable: {            // phenotype parameters with bounds
    "vcmax25": (default: 89.0, min: 40.0, max: 140.0, unit: "umol/m2/s", source: "Pimentel 2011"),
    "nodes_before_terminal": (default: 7, min: 4, max: 30, unit: "-", source: "~ bush type"),
    "habit": (default: 0, min: 0, max: 1, unit: "0 determinate / 1 indeterminate", source: "botany"),
    "leaf_area_potential": (default: 75.0, min: 30.0, max: 150.0, unit: "cm2", source: "~"),
    // ...
  },
  genome: (               // M0: one locus per trait, zero variance (identity mapping)
    loci: [ (trait: "vcmax25", effect: 1.0, dominance: 0.0, allele_sd: 0.0), /* ... */ ],
    mutation_rate: (value: 0.001, unit: "per locus per gamete", source: "design"),
  ),
  phenology: ( t_base: (value: 8.0, unit: "C", source: "~"), t_opt: (value: 28.0, unit: "C", source: "~"),
               t_max: (value: 38.0, unit: "C", source: "~"), emergence: (value: 70.0, unit: "Cd", source: "~"),
               flowering: (value: 350.0, unit: "Cd", source: "~ 40% of 800"), maturity: (value: 850.0, unit: "Cd", source: "Pinzon-Sandoval 2024") ),
  roots: [ RootType(order: 0, elongation: (value: 2.0, unit: "cm/day", source: "~"), max_length: (value: 60.0, unit: "cm", source: "~"),
                    branch_spacing: (value: 1.0, unit: "cm", source: "~"), insertion_angle: (value: 70.0, unit: "deg", source: "~"),
                    gravitropism: 1.0, hydrotropism: 0.5), /* orders 1, 2 */ ],
  appearance: ( leaf_template: "trifoliate", leaf_color: "#4f8a3a", flower_color: "#f4f0e6", pod_template: "bean_pod" ),
)
```

The loader rejects a file whose unit string does not match the expected unit for that field, and a parameter without a source. "\~" is an accepted source meaning a guess to be validated.

### 6.2 Scenario file (data/scenarios/two\_pot\_sun\_shade.ron)

```ron
Scenario(
  schema: 1,
  model_version: "0.1.0",
  world_seed: 42,
  latitude_deg: 45.0,
  start: (year: 2000, month: 5, day: 15, hour: 6),
  weather: Preset("temperate_summer"),      // or File("data/weather/my_town_2019.csv")
  soils: [ Pot(id: 0, diameter_cm: 20.0, depth_cm: 20.0, texture: Loam, position_m: (0.0, 0.0)),
           Pot(id: 1, diameter_cm: 20.0, depth_cm: 20.0, texture: Loam, position_m: (0.6, 0.0)) ],
  radiation_fraction: { 0: 1.0, 1: 0.5 },      // per soil id
  irrigation: { 0: Schedule(every_days: 2, mm: 10.0), 1: Schedule(every_days: 2, mm: 10.0) },
  co2_ppm: 420.0,
  sowings: [ (soil: 0, species: "bush_bean", seed: Default, depth_cm: 3.0), (soil: 1, species: "bush_bean", seed: Default, depth_cm: 3.0) ],
  duration_days: 90,
)
```

Weather files are hourly CSV with columns: timestamp (ISO 8601, local), t\_air\_c, rh\_pct, radiation\_w\_m2, rain\_mm, wind\_m\_s (optional), cloud\_fraction (optional). Daily files are accepted and disaggregated. The sim clock, including scenario start hours and weather file timestamps, is local mean solar time: there is no longitude, time zone or daylight saving time, and 12:00 is when the mean sun crosses the meridian (decision record 0005).

### 6.3 Saves and the action log

A save is a directory: scenario.ron (the starting scenario), actions.ron (the action log), snapshots/ (optional cache, one file per sim-day or sparser), and notebook.ron for experiments. The action log is append-only:

```ron
[ Action(tick: 912, kind: Water(soil: 1, mm: 5.0)),
  Action(tick: 2400, kind: SetRadiationFraction(soil: 1, fraction: 0.5)),
  Action(tick: 4000, kind: Harvest(plant: 3, organ: 118)),
  Action(tick: 5100, kind: HandPollinate(flower: 231, pollen_plant: 2)) ]
```

Ticks are sim hours since scenario start. Loading replays the log from the scenario; snapshots only speed that up and are validated against the replay hash when present. A save from an older model version replays with a warning that results will differ; the old model is not shipped.

### 6.4 Challenge file (M3)

```ron
Challenge(
  schema: 1, id: "drought_pods", title: "Most pods on 150 mm of water",
  scenario: "data/scenarios/plot_loam_45n.ron", model_version: "0.3.0", world_seed: 7,
  constraints: [ MaxWaterMm(150.0), NoFertilizer ],
  score: Metric(column: "pods_total", at: EndOfRun, higher_is_better: true),
  duration_days: 100,
)
```

Scores are recomputed from the replay, so a score is reproducible by anyone with the save.

### 6.5 Experiment cards

```ron
ExperimentCard( id: "shade_catch_up", prompt: "Make the shaded plant catch up to the sunny one by day 60.",
                scenario: "data/scenarios/two_pot_sun_shade.ron", watch: ["dry_mass_total", "leaf_area"] )
```

### 6.6 CSV export

- **Daily time series** (one file per plant): day, date, stage, thermal\_time\_cd, dry\_mass\_total\_g, dry\_mass\_leaf\_g, dry\_mass\_stem\_g, dry\_mass\_root\_g, dry\_mass\_pod\_g, dry\_mass\_seed\_g, leaf\_area\_m2, height\_m, n\_leaves, n\_branches, n\_pods, assimilation\_g\_c, respiration\_g\_c, growth\_g\_c, reserve\_g\_c, transpiration\_kg, uptake\_kg, ftsw, absorbed\_par\_mol, carbon\_residual\_g\_c, water\_residual\_kg.
- **Per-organ table** (at the end or on demand): the §3.3 fields, one row per organ, with birth day and death day.
- **Soil table**: per cell water content and, in M2, nitrogen, at the end or on demand.

Column names are the contract between the sim, the charts panel and the validation scripts; changing one is a schema bump.

### 6.7 Versioning

Every file carries a schema integer. The sim carries a model\_version (semver). A change to any equation, parameter default or step order bumps the model's minor version and invalidates golden hashes; a change to data layout bumps the schema. Challenges and notebook entries pin both.

## 7. Milestones and work packages

Five milestones. Each has an exit that is a measurement, not an opinion. Work packages (WP) are in build order; each later package depends only on earlier ones, so each can be one or two agent sessions ending with green CI. Size is relative: S under a session, M one to two, L two to four. The implementer should open one GitHub issue per WP with its done-when as the acceptance criteria (§11.8).

### M0: headless core

One bush bean grows from seed to maturity in a headless run, lands inside the §8 targets under four environments, conserves water and carbon, and produces the same hash every run. No graphics. Everything builds and tests in a Linux container.

| WP | Work package | Crates | Size | Done when |
| --- | --- | --- | --- | --- |
| 0 | Repo scaffold and conventions: Cargo workspace, CI (fmt, clippy pedantic as errors, nextest), CLAUDE.md with the sim rules (§11.2), model card template in docs/models/, decision records in docs/decisions/, issue templates | all | S | CI green on the empty workspace; decision records for Rust and Bevy, carbon authority, tree to M4 committed |
| 1 | Units and RNG: unit newtypes with checked conversions; counter-based RNG keyed by (world seed, entity id, tick, purpose) | ps-units, ps-rng | S | Property tests: same key gives same stream; adding an entity does not change another's stream |
| 2 | Weather and sun: solar position and day length by latitude and date; direct and diffuse split from clearness; presets (temperate summer, overcast, heat wave, drought, frost night); daily-to-hourly disaggregation; hourly CSV reader | ps-weather | M | Sun position within 0.5° of a reference for 10 dates × 3 latitudes; preset daily radiation totals match |
| 3 | Organ graph: arena, organ types, parent and children lists in creation order, beta expansion curve, allometry | ps-graph | M | Property tests: structural mass never shrinks except at death; expansion curve integrates to the potential size |
| 4 | Species schema: RON with units and sources checked on load; genotype-to-phenotype identity mapping; the bush bean file | ps-species | M | Bean file loads; a wrong unit or missing source fails with a clear error |
| 5 | Soil and water: 3D grid, pot mask, infiltration, drainage, lateral redistribution, evaporation, soil temperature, root uptake, FTSW | ps-soil | M | A wet column drains to field capacity with closure under 1e-9; the analytic case in §8.1 matches |
| 6 | Light: adaptive grid, leaf area density, ray marching for 24 sun positions and 16 sky sectors, per-leaf coefficients, red:far-red proxy | ps-light | M | Uniform canopy within 2% of Beer's law; single plant within 10% per leaf of the Python ray tracer fixture |
| 7 | Physiology: coupled Farquhar and Medlyn solver with light classes; respiration; thermal time; phenology stages; the §3.9 allocation algorithm | ps-physio | L | Gas exchange within 2% of plantecophys fixtures over the test grid; allocation unit tests: a starving organ dies on schedule; unmet demand shrinks final size |
| 8 | Herb architecture and roots: determinate main stem, axillary branching, phyllotaxis, bud fate, shade avoidance, tropisms; CRootBox-style root types; seed and cotyledons; flowers, pods, seeds with abortion rules | ps-arch | L | A 90-day run produces node, branch, pod and root counts inside the §3.16 ranges |
| 9 | World: the §3.2 step schedule, action log, snapshot save and load, full-state hash, conservation ledger | ps-world | M | Three runs give one hash; a run resumed from a day-30 snapshot matches the uninterrupted run's day-90 hash |
| 10 | Export and CLI: scenario files; `ps-cli run`, `hash`, `bench`, `replay`; daily time series and per-organ CSV | ps-export, ps-cli | S | The four §8.3 scenarios run from one command and write CSV |
| 11 | Validation harness: Python scripts that generate fixtures (plantecophys via R, ray tracer, solar), read the CSV, compare to §8 targets and write a plot report committed as a CI artifact; a 3D matplotlib snapshot of the organ graph for eyeballing | validation/ | M | The report runs in CI and fails the build when a target is missed |
| 12 | Tuning and lock: adjust \~ parameters until all §8 M0 targets pass; record golden hashes for Linux and Windows | data/, tests/golden/ | L | Exit criteria below |

M0 exit, all measured by the WP11 report:

- Temperate summer scenario, 90 sim-days from sowing: total dry mass, pod count, height, peak leaf area, flowering date and maturity date inside the §8.2 ranges; biomass curve logistic with R² above 0.95.
- Shade scenario (50% radiation): internodes at least 20% longer, specific leaf area higher, total mass 30 to 50% lower.
- Drought scenario (irrigation withheld from day 30): root to shoot ratio higher, pod count down at least 30%, seed size down less than pod count.
- Frost-night scenario: leaves die, no recovery. Heat-wave scenario at flowering: pod set falls.
- Water and carbon ledgers close to 1e-9 relative every hour.
- One hash across three Linux runs; Linux and Windows hashes recorded, each stable.
- Under 2 ms per sim-day for the single plant, by instruction count on CI.
- A model card for every §3 process; every parameter has a source or "\~".

### M1: the two-pot visible proof

Two bean plants from the same seed grow side by side on the GTX 1070, one in sun and one in shade or one watered and one in drought, at up to a sim-week per second, with lenses, inspector and charts. The headless replay of a session gives the same hash as the session with graphics. The sim does not change in M1 except for bugs found by looking.

| WP | Work package | Crates | Size | Done when |
| --- | --- | --- | --- | --- |
| 13 | Lock the engine: pick the current stable Bevy and matching bevy\_egui on day one, write docs/bevy-notes.md, build a window-and-cube binary, have the owner run it on the GTX 1070. Do this during M0, not after | ps-app | S | A screenshot from the target machine |
| 14 | Mesh generation: tapered tubes with smooth joins; leaf templates with midrib bend and droop; flower, pod, cotyledon, seed templates; root tubes; evaluation at fractional sim time | ps-mesh | M | Headless tests: vertex and index counts, outward normals, no NaN at size zero, continuity between hours |
| 15 | Scene sync and look: diff the snapshot by organ id; instanced leaves; stems rebuilt only on >2% size change; soil slab with glass front and roots; sky and sun; stylized shading; auto-framing orbit camera; high-speed mode | ps-view | L | Two 90-day plants at 60 fps at 1440p on the 1070 |
| 16 | Sim thread: sim on its own thread, renderer reads a double-buffered snapshot; speed control, pause, step buttons; actions through the action log only | ps-app, ps-world | M | Session hash equals headless replay hash |
| 17 | UI: top bar, lens panel with legends, inspector with units and the why line, charts panel, two-pot layout, experiment panel, model card side panel | ps-ui | L | Every lens has a legend; every inspector field a unit; charts match the CSV |
| 18 | Lenses: light, soil water, plant water, roots X-ray, carbon, phenology, shade signal | ps-view, ps-ui | M | Each lens matches the CSV for a clicked organ |
| 19 | Capture and library: screenshot, time-lapse frames, CSV export, seed library (data only) | ps-app | S | Files land next to the save; a harvested seed appears in the library |
| 20 | The moments: cotyledon unfolding, phototropic lean, wilting and recovery, flowering cue; tune visuals, not the sim | ps-view | M | Owner signs off on a 60-second sun-versus-shade clip |

M1 exit: the two-pot runs look right to the owner and the pot differences are the ones M0 measured; 60 fps at 1440p for two plants; under 16 ms frame time at one sim-week per second in high-speed mode; session hash equals replay hash; a 60-second time-lapse clip exported.

### M2: plot and sandbox

| WP | Work package | Size | Done when |
| --- | --- | --- | --- |
| 21 | Plot soil block (1 × 1 to 3 × 3 m, 5 cm cells), merged light grid, per-voxel light coefficients (measured and not used; decision 0019) | M | 30 plants under 20 ms per sim-day |
| 22 | Vine module and pole bean: habit locus, indeterminate growth, circumnutation, capture and twining, trellis objects (decisions 0021, 0023) | L | A pole bean reaches 1.5 m by day 60 and 2 m by day 75 at a 20 °C mean; without a trellis it flops and yields less |
| 23 | Grass module and spring wheat: tillering, intercalary growth, fibrous roots (decisions 0022, 0023) | L | Tiller count and height inside a literature range chosen in WP23 (ranges and results in decision 0022) |
| 24 | Nitrogen: soil mineral N, mineralization, leaching, uptake, leaf N drives Vcmax, remobilization, nodules for legumes, fertilize action | L | Nodulated beans grow in N-free soil and unnodulated ones do not; bottom-up senescence now emergent |
| 25 | Prune and harvest actions, obstacles, weather file import UI, latitude and season controls | M | Pruning releases branching visibly |
| 26 | Save and load UI, replay scrubber with action markers, competition, temperature and nitrogen lenses, carbon flow arrows | M | Replay of a saved plot matches its hash |
| 27 | Leaf energy balance (leaf temperature from radiation, wind and transpiration) | M | Leaf-air temperature difference within literature range on a hot still day |

M2 exit: 30 plants at 60 fps and under 20 ms per sim-day; leaf area per unit ground plateaus with density as in §8.3; pruning releases branching; nodulation works; weather file import drives a run.

### M3: seeds, breeding and challenges

| WP | Work package | Size | Done when |
| --- | --- | --- | --- |
| 28 | Genome: 20 to 40 loci, additive plus dominance, light linkage (chromosome and cM per locus), two-tier mutation, a seed-colour rule table; species files as allele distributions (decision 0023) | M | Two seeds of one species differ measurably; F2 variance exceeds F1 variance |
| 29 | Pollination: selfing by default; hand-pollination action; recombination at seed set | S | A cross records both parents in each seed |
| 30 | Seed library and breeding bench: ten headless pots in parallel, results card with charts, parent selection | L | A ten-seed generation runs in under 10 s of wall time |
| 31 | Trait discovery: traits appear in the UI only after measured in a grown plant | M | Genotype lens shows only measured traits |
| 32 | Challenges: challenge files, scoring from the replay, local leaderboard, the first four challenges | M | One challenge playable start to finish with a reproducible score |

M3 exit: a cross between two lines shows F2 variation on the bench charts; one challenge is playable with a replay; selected lines are measurably different after five generations.

### M4: fields, forests and the tree

| WP | Work package | Size | Done when |
| --- | --- | --- | --- |
| 33 | Tree module: self-organizing bud fate, pipe-model secondary growth, perennial storage, dormancy and leaf fall, frost hardiness, years-long runs | L | A ten-year tree run with recognizable seasons and plausible height and crown |
| 34 | Aggregate level: patch cohorts, two-leaf canopy photosynthesis on the shared gas-exchange code, height-sorted competition, soil column per patch | L | Patch-level yield matches the organ-level plot within 10% for the same scenario |
| 35 | Rendering at distance: procedural plants from cohort statistics, instancing, impostors; terrain | L | One hectare at 60 fps |
| 36 | Promotion and demotion with conservation tests | M | Round trip preserves conserved quantities within 5%, or M4 detail declared visual-only |

M4 exit: a one-hectare field or a small mixed stand at 60 fps; zoom-in promotion passes the conservation test; a ten-year forest run.

## 8. Validation: realism as measurement

Four levels, each with a named reference and a tolerance. All data comes from published studies; no real-plant experiment is planned.

### 8.1 Process fixtures (unit tests)

| Process | Reference | Test grid | Tolerance |
| --- | --- | --- | --- |
| Coupled Farquhar and Medlyn gas exchange | R package plantecophys, Photosyn() | PAR 50 to 2,000 × temperature 10 to 40 °C × VPD 0.5 to 4 kPa × CO2 280, 420, 800 ppm | 2% on assimilation, conductance, transpiration |
| Solar position and day length | NOAA solar calculator or pvlib | 10 dates × 3 latitudes | 0.5° |
| Light interception, uniform canopy | Beer's law closed form | leaf area index 0.5 to 6, G = 0.5 | 2% on absorbed fraction |
| Light interception, single plant | Own Monte Carlo ray tracer in Python, committed with its seed | 60-day bean geometry exported from the sim | 0.10 absolute per leaf on the daily light index (95% of leaves for M0, decision 0008), 3% relative on the area-weighted plant total |
| Soil water | Closed-form drainage of a wet column to field capacity; mass closure | loam, sand, clay | closure 1e-9 relative; drainage time within 5% |
| Thermal time | Hand-computed hourly sums | 5 daily temperature traces | exact to rounding |
| Expansion curve | Integral of the beta function equals potential size | 20 parameter sets (property test) | 1e-6 |
| Allocation | Hand-built three-organ graphs | starvation, deficit, priority cases | exact to rounding |

### 8.2 Whole-plant targets for bush bean

The M0 temperate-summer run must land in these ranges. Cultivars differ, so targets are ranges, not points.

| Variable | Target | Source |
| --- | --- | --- |
| Sowing to maturity, thermal time | \~800 °C·day in a greenhouse, both seasons | [Pinzón-Sandoval 2024](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop) |
| Thermal sums by BBCH stage | Stage-by-stage check once the paper's table is digitized | [Cavalcante 2020](https://consensus.app/papers/details/f99d70d2ad125b88bd90b297eb6a7a26/?utm_source=claude_desktop) |
| Sowing to maturity, days | 65 (super-early) to 93 (long cycle) | [Conceição 2026](https://consensus.app/papers/details/c2e8163fd8c45570a67062d5bb53982d/?utm_source=claude_desktop) |
| Emergence, flowering, maturity in a warm climate, days after planting | 4.5, 28.7, 54.2 | [Ventura Elías 2016](https://consensus.app/papers/details/f12981865fff580d98042cf80f5d8496/?utm_source=claude_desktop) |
| Dry mass accumulation shape | Logistic for the plant and each organ class | [Pinzón-Sandoval 2024](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop) |
| Height, branches, leaves, leaf area, dry mass at harvest | 35 cm; 5.5 branches; 20 leaves; 0.12 m²; 21.8 g per plant | [Kumar 2025](https://consensus.app/papers/details/0acea9340180597f8465c3c495e4b046/?utm_source=claude_desktop) |
| Pods per plant, seeds per pod, seed mass per plant | 11.8; 3.7; 8.8 g | [Kumar 2025](https://consensus.app/papers/details/0acea9340180597f8465c3c495e4b046/?utm_source=claude_desktop) |
| Specific leaf area | \~240 cm² per g (highest genotype); declines with age | [Gebeyehu 2019](https://consensus.app/papers/details/59277c779b9d5689aab68ff3e5ddea22/?utm_source=claude_desktop), [Merga 2021](https://consensus.app/papers/details/b9a69a6d643158bba3217253856933f5/?utm_source=claude_desktop) |
| Net assimilation rate, early growth | up to 41 mg per dm² per day (4.1 g m⁻² day⁻¹) | [Merga 2021](https://consensus.app/papers/details/b9a69a6d643158bba3217253856933f5/?utm_source=claude_desktop) |
| Vcmax, Jmax, dark respiration, mesophyll conductance at 25 °C | 89.3, 159.8, 0.88 µmol m⁻² s⁻¹; 0.47 mol m⁻² s⁻¹ | [Pimentel 2011](https://consensus.app/papers/details/879c7fe0f71a538b8407a5c7943b21ef/?utm_source=claude_desktop); herbaceous annual means 75 and 154 in [Wullschleger 1993](https://consensus.app/papers/details/d5f18c7251575a80b536d7a8d4178160/?utm_source=claude_desktop) |

### 8.3 Response targets (the scenarios)

| Scenario | Expected response | Source | Tag |
| --- | --- | --- | --- |
| Drought from flowering | Pod number falls more than seed size; leaf area falls more than other variables; shoot mass, leaf area index and growth rate all fall | [Gomes](https://consensus.app/papers/details/b97819d779ef59e28cc7d3e5a87920d1/?utm_source=claude_desktop), [Guerrero-Domínguez](https://consensus.app/papers/details/1504985d05765a26ab566f8216abf698/?utm_source=claude_desktop) | Pod abortion encoded; magnitude emergent |
| Drought, allocation | Root to shoot ratio rises | general | Partly encoded (root priority term) |
| Heat at flowering (32 °C days for 5 to 7 days) | Pod and seed set fall through pollen failure; total seed weight down 50 to 71%, single seed weight down 8 to 34% | [Gross 1994](https://consensus.app/papers/details/d0e154d8f0035b178b28813744fe424a/?utm_source=claude_desktop), [Sharda 2026](https://consensus.app/papers/details/0f6080758ba55e87a0bba510b6043b75/?utm_source=claude_desktop) | Encoded abortion rule; the seed-weight numbers are the check |
| 50% shade | Longer internodes, higher specific leaf area, lower total mass | shade-avoidance literature; bean-specific number still to find | Encoded trigger, emergent magnitude |
| Dense planting (M2) | Leaf area and leaf nitrogen per area plateau at 20 plants per m² and above | [Clavijo Michelangeli 2019](https://consensus.app/papers/details/11b9dd4ab6fd51aa9d2da96ca8314961/?utm_source=claude_desktop) | Emergent |
| Lower shaded leaves | Drop after sustained shading | emergent from local carbon | Emergent |
| Pruning (M2) | Buds below the cut activate | emergent from apical dominance release | Emergent |
| Nitrogen-free soil (M2) | Nodulated plants grow, unnodulated do not | legume physiology | Emergent from the nodule cost and benefit |

Pass rule: a range target passes if the sim lands inside it; the biomass curve passes if a logistic fit has R² above 0.95; a response target passes if the direction matches and the magnitude is within a factor of two of the cited value. Tolerances tighten as better data is digitized.

### 8.4 Regression

Golden-run hashes per platform; the validation report as a CI artifact on every push; a weekly scheduled run of the full scenario set so drift shows even when nobody touched the sim.

### 8.5 Data gaps and how to fill them (all online)

- A per-organ weekly time series: digitize the figures behind [Pinzón-Sandoval 2024](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop) (WP11).
- Stage-by-stage thermal sums: digitize the table in [Cavalcante 2020](https://consensus.app/papers/details/f99d70d2ad125b88bd90b297eb6a7a26/?utm_source=claude_desktop) (WP11).
- DSSAT CROPGRO-Drybean cultivar and species files as a second reference for phenology and partitioning; confirm availability and license (WP11).
- A bean shade-avoidance number (internode length under reduced red:far-red); one study is enough (WP12).
- Bean root parameters: check CRootBox example parameter sets for a Phaseolus entry (WP8).
- Grass targets for WP23 are in decision 0022; tree targets for WP33 are chosen when that package starts.

## 9. Risk register

| # | Risk | Likelihood | Impact | Control |
| --- | --- | --- | --- | --- |
| R1 | Realism versus performance | High | High | Multi-rate stepping; daily light coefficients; gas exchange per light class; per-voxel coefficients at M2; budgets per milestone measured by instruction count; LOD at M4 |
| R2 | Allocation tuning spirals | Medium | High | Fallback to three pools decided in advance (§3.9); time-boxed to two tuning sessions |
| R3 | Parameter explosion and lost provenance | High | Medium | Literature defaults; genotype bounds; every parameter carries a source or "\~" and CI checks it; model cards per process |
| R4 | Model drift from AI-written code across sessions | High | High | Validation suite and golden runs in CI; model cards; decision records; CLAUDE.md rules; one issue per WP with done-when; this doc as the backlog |
| R5 | Scope creep | High | Medium | Milestone exits are numbers; a tree is M4, not M2; features not in a WP wait for the next milestone |
| R6 | The owner sees nothing for the whole of M0 | High | Medium | WP11 plots from the first package; a matplotlib 3D organ-graph snapshot; run the WP13 Bevy smoke test during M0 |
| R7 | Windows is the target but development is on Linux | Certain | Medium | Windows CI job builds, runs sim tests and records its own golden hash; only rendering checks need the GTX 1070 |
| R8 | Bevy API churn and version mismatch with bevy\_egui | High | Medium | Lock on day one of M1; docs/bevy-notes.md; upgrade only between milestones; Bevy code confined to three thin crates |
| R9 | Determinism traps | Medium | High | f64 and libm; ordered reductions; no HashMap iteration in the sim; no target-cpu=native; full-state hash every sim-day in golden tests; per-platform hashes |
| R10 | Linux and Windows hashes differ | Likely | Low | Expected; two golden hashes; same-platform determinism is the guarantee |
| R11 | Rendering cannot be tested in the cloud | Certain | Medium | Geometry logic tested headlessly in ps-mesh; software-Vulkan smoke test best effort; owner checks at each milestone exit |
| R12 | egui looks plain or egui\_plot falls short | Medium | Medium | Custom theme; egui\_plot covers M1; fall back to plotters-to-texture or Bevy UI for chrome in M2 |
| R13 | Compile times with Bevy | Medium | Low | Dynamic linking in dev; fast linker; small sim crates; Bevy only built in the M1 CI job |
| R14 | Real species name invites comparison the model cannot win | Medium | Low | Call it a bush bean archetype; show the ranges it is tuned to in its model card |
| R15 | Saves break across model versions | Certain over time | Low | Replay re-simulates with a warning; challenges pin the version; snapshots are a cache |
| R16 | The sandbox is boring between M1 and M3 | Medium | Medium | Experiment mode and cards in M1; seed library from M1 |
| R17 | Heat, frost and shade presets do nothing visible | Medium | Medium | Temperature growth multiplier, frost kill and heat abortion in M0; shade avoidance tested in M0 |
| R18 | Light-class binning hides per-leaf differences the lenses are meant to show | Low | Low | Lenses color by each leaf's own absorbed PAR; only the solver is shared; a per-leaf debug mode exists for validation |

Switch to the runner-up architecture (Godot 4 via gdext) if visual polish or UI needs outgrow Bevy plus egui, if Bevy upgrades cost more than about a week per milestone, or if a human artist joins and wants an editor. The sim crates stay unchanged in that case.

## 10. Architecture

**Decision: Rust.** A pure-Rust simulation workspace with no engine dependency, a Bevy renderer pinned to one version, and egui (bevy\_egui plus egui\_plot) for UI and charts. Python and R generate validation reference data and analysis; nothing shipped depends on them.

### 10.1 Why

- Compile-time safety catches AI mistakes: types, ownership, exhaustive matching, clippy with warnings as errors.
- Determinism is controllable: no implicit fused multiply-add or fast-math, libm for transcendental functions, fixed-order parallel reductions, ordered maps only in the sim.
- Everything is code or text: no editor or scene files; species, scenarios and saves are RON via serde. M0 and most of M1 build and test headlessly in a Linux container.
- Bevy covers M1's rendering: instanced leaves, runtime mesh updates, stylized shading, cameras, screenshots; wgpu on DX12 or Vulkan runs well on a GTX 1070.
- egui fits the inspector, lens panels and time-series charts.

### 10.2 Options compared

| Option | Verdict |
| --- | --- |
| Rust sim + Bevy + egui | Chosen. Best safety, determinism, text-only workflow, headless testing. Risk: Bevy API churn every 3 to 4 months. |
| Rust sim + Godot 4 (gdext) | Runner-up. Better stylized rendering and UI widgets, more stable; needs the editor and export templates; scene files are less AI-friendly. |
| C# + Godot .NET | Fast iteration; weaker determinism control and compile-time safety. |
| Rust + raw wgpu + egui | Months of extra renderer work. |
| C++ + bgfx or raylib + ImGui/ImPlot | Great charts; weaker safety for AI-written code. |
| Unity or Unreal | Editor-centric; cannot develop in a container; overkill. |
| Python core + viewer | Too slow for organ-graph walks; weakest safety. |

### 10.3 Stack

- Rust stable, one Cargo workspace, edition 2024, MSRV pinned in rust-toolchain.toml.
- Sim: structs-of-arrays organ graph with integer ids (an arena, not Bevy's ECS); rayon with fixed chunking and in-order reductions; f64; libm; counter-based RNG keyed by (world seed, entity, tick, purpose).
- Units: lightweight newtypes (GramsC, GramsDM, MolPerM2S, Celsius, Meters, Hours, DegreeDays); data files carry units, checked on load.
- Renderer: Bevy reading a read-only sim snapshot each frame; mesh generation in an engine-free crate.
- UI: bevy\_egui plus egui\_plot with a custom theme.
- Testing and CI: cargo-nextest, proptest (conservation and invariants), insta (snapshot tests of CSV heads and RON), criterion and iai-callgrind (instruction-count budgets stable in CI), cargo-deny, clippy pedantic. GitHub Actions on Linux (fmt, clippy, tests, golden hash, validation report) and Windows (build, sim tests, Windows golden hash, release packaging). Best-effort headless render smoke test via software Vulkan.
- Python and R: validation/ scripts generate committed fixtures (plantecophys via R, the ray tracer, solar positions); pandas and matplotlib compare CLI CSV output to the §8 targets and write the report. Optional PyO3 bindings (ps-py) for parameter sweeps.

### 10.4 Workspace layout

```text
crates/
  ps-units    unit newtypes                                        M0
  ps-rng      keyed counter-based RNG                              M0
  ps-graph    organ graph arena, organ types, expansion curves     M0
  ps-species  species and genome schema, genotype to phenotype     M0
  ps-weather  presets, hourly CSV, daily-to-hourly, solar geometry M0
  ps-light    turbid-medium voxel light, sky sectors, R:FR         M0
  ps-soil     3D bucket soil grid, FTSW, uptake, (N in M2)         M0
  ps-physio   FvCB+Medlyn, respiration, phenology, allocation      M0
  ps-arch     growth-form modules: herb (M0), vine, grass (M2), tree (M4)
  ps-world    scheduler, action log, snapshots, state hash, ledger M0
  ps-export   CSV time series and per-organ tables                 M0
  ps-cli      headless runner: run / hash / bench / replay         M0
  ps-py       optional PyO3 bindings                               M0+
  ps-mesh     tube and leaf generation, interpolation (engine-free) M1
  ps-view     Bevy plugin: scene sync, lenses, camera, high-speed  M1
  ps-ui       egui panels, inspector, charts, tools                M1
  ps-app      binary: two-pot mode, experiment mode                M1
data/species  data/scenarios  data/weather  data/cards  data/challenges
docs/models   docs/decisions  docs/bevy-notes.md
validation/   tests/golden/   benches/
```

### 10.5 Dependency rules

- ps-world and everything below it never depends on Bevy, egui, or anything with a GPU. Enforced in CI by a script that reads cargo metadata.
- The renderer reads state and sends actions through the action log only, so saves and replays behave the same with or without graphics.
- ps-mesh depends on ps-graph and glam only, so geometry is testable headlessly.
- Only ps-view, ps-ui and ps-app depend on Bevy, and they are thin.

## 11. Implementation guide for the next session

Written for a Claude session that starts with this document and an empty repository. Follow it in order. Where this guide and §3 disagree, §3 wins and this guide gets fixed.

### 11.1 First session, in order

1. Read §3, §7 (M0), §6 and this section. Do not read the v2 plan or the v3 review; this document supersedes both.
2. Create one GitHub issue per M0 work package (WP0 to WP12) titled "WP\<n>: \<name>", body = the WP row from §7 plus its done-when as a checklist. Label by milestone.
3. Do WP0 on a branch, open a PR, get CI green, merge. Every later WP is its own branch and PR.
4. Save a decision record (§11.8) for anything this document leaves open that you had to decide. Never decide silently.
5. Keep docs/STATUS.md current: which WP is done, which is in progress, what the last validation report said. The next session reads it first.

### 11.2 CLAUDE.md (commit this verbatim as the starting point)

```markdown
# Plant-Sim: rules for agents

The design is docs/design-plan.md. Section 3 is the model; section 7 the backlog; section 11 this guide's source.
One work package per branch and PR. Issue "WP<n>" is the acceptance criteria. CI must be green before merge.

## Simulation rules (non-negotiable)
- Crates ps-world and below never depend on bevy, egui, wgpu or anything with a GPU. CI checks cargo metadata.
- Sim math is f64. Transcendentals come from libm, never std. No target-cpu=native. No fast-math.
- No HashMap or HashSet iteration in the sim. Use Vec, BTreeMap, or indexmap with explicit ordering.
- All randomness goes through ps-rng keyed by (world seed, entity id, tick, purpose). No thread_rng, no global RNG.
- Parallel reductions use fixed chunking and in-order combination (rayon with explicit chunks, then a sequential fold).
- Every physical quantity is a unit newtype from ps-units. Raw f64 is allowed only inside a function body.
- Every parameter in data/ has value, unit and source. "~" is a valid source meaning a guess to validate.
- Step order is docs/design-plan.md section 3.2. Do not reorder without a decision record and a model version bump.
- Debug builds assert conservation (water and carbon) every hour at 1e-9 relative.
- Every process has a model card in docs/models/. A PR that adds or changes an equation updates the card.
- Changing an equation, a default parameter or the step order bumps the model minor version and regenerates golden hashes.

## Code style
- clippy::pedantic as errors; no unsafe; no panics in library code except debug_assert and unreachable!.
- Errors: thiserror in libraries, anyhow in binaries. Logging: tracing.
- Tests next to code; fixtures under tests/fixtures; golden hashes under tests/golden/<platform>/.
- Commit messages: imperative, one WP per PR, reference the issue.

## Bevy (M1 onward)
- The pinned version and its idioms are in docs/bevy-notes.md. Do not use APIs from other versions. If the compiler disagrees with your memory, the compiler is right.
```

### 11.3 Core types (sketch, Rust)

These are shapes, not final signatures. Keep them this simple.

```rust
// ps-units
#[derive(Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)] pub struct GramsC(pub f64);
pub struct GramsDM(pub f64); pub struct Celsius(pub f64); pub struct Meters(pub f64);
pub struct M2(pub f64); pub struct UmolPerM2S(pub f64); pub struct DegreeDays(pub f64); pub struct Hours(pub u64);

// ps-rng: counter-based, keyed. Philox or ChaCha8 seeded from a hash of the key.
pub struct RngKey { pub world_seed: u64, pub entity: u64, pub tick: u64, pub purpose: Purpose }
pub fn draw_f64(key: &RngKey, n: u32) -> f64;      // n-th draw for that key, stateless

// ps-graph
pub struct OrganId(pub u32);
pub enum OrganKind { Seed, Cotyledon, Internode, Bud, Leaf, Root, Flower, Pod, SeedInPod, Nodule, Tendril }
pub struct ExpansionCurve { pub duration: DegreeDays, pub t_max: DegreeDays, pub potential: f64 }
pub struct OrganArena {           // structs of arrays, all indexed by OrganId
    pub kind: Vec<OrganKind>, pub parent: Vec<Option<OrganId>>, pub children: Vec<Vec<OrganId>>,
    pub birth: Vec<Hours>, pub thermal_age: Vec<DegreeDays>, pub curve: Vec<ExpansionCurve>,
    pub structural: Vec<GramsDM>, pub reserve: Vec<GramsC>, pub nitrogen: Vec<f64>, pub unmet: Vec<GramsC>,
    pub light: Vec<LightCoeffs>, pub orientation: Vec<Orientation>, pub alive: Vec<bool>,
}

// ps-light
pub struct LightCoeffs { pub direct: [f64; 24], pub diffuse: f64, pub sunlit_fraction: f64, pub rfr: f64 }
pub fn rebuild_daily(shoot: &ShootGeometry, sun_path: &[SunDir; 24], sky: &SkySectors) -> Vec<LightCoeffs>;

// ps-soil
pub struct SoilGrid { /* cells, mask, texture, water, temperature, n_mineral, root_length_density */ }
impl SoilGrid { pub fn step_hour(&mut self, w: &HourWeather, uptake: &[CellUptake]) -> WaterLedger; pub fn ftsw(&self, rld: &RootLengthDensity) -> f64; }

// ps-physio
pub struct LightClass { pub par: UmolPerM2S, pub sunlit: bool, pub area: M2 }
pub struct GasExchange { pub a_net: UmolPerM2S, pub gs: f64, pub e: f64 }
pub fn solve_fvcb_medlyn(p: &LeafParams, env: &LeafEnv) -> GasExchange;   // Newton on Ci, bisection fallback
pub fn allocate_hour(arena: &mut OrganArena, edges: &Edges, ctx: &AllocCtx) -> CarbonLedger;

// ps-arch
pub trait GrowthForm {
    fn daily_bud_fate(&self, plant: &mut Plant, ctx: &DailyCtx);
    fn daily_initiate(&self, plant: &mut Plant, ctx: &DailyCtx);
    fn daily_senesce(&self, plant: &mut Plant, ctx: &DailyCtx);
}

// ps-world
pub enum Action { Sow{..}, Water{..}, SetRadiationFraction{..}, SetIrrigation{..}, Harvest{..}, Prune{..}, Fertilize{..}, HandPollinate{..}, PlaceObstacle{..} }
pub struct World { /* scenario, weather, soils, plants, tick, action log, ledgers */ }
impl World {
    pub fn from_scenario(s: &Scenario) -> World;
    pub fn apply(&mut self, a: Action);          // appends to the log and applies at the current tick
    pub fn step_hour(&mut self);                // section 3.2, hourly
    pub fn step_day(&mut self);                 // section 3.2, daily; called by step_hour at midnight
    pub fn snapshot(&self) -> Snapshot;         // read-only view for the renderer and for saves
    pub fn state_hash(&self) -> u64;            // xxhash of all sim state in fixed order
}
```

### 11.4 Numerics that must be right

- **Carbon units.** 1 µmol CO2 = 12 µg C. Hourly leaf income in g C = A\_net (µmol m⁻² s⁻¹) × area (m²) × 3600 × 12e-6.
- **FvCB.** Standard three-limitation form (Rubisco, RuBP regeneration, triose phosphate utilization optional) with Arrhenius temperature responses for Kc, Ko, Gamma-star and peaked Arrhenius for Vcmax and Jmax, using the Medlyn 2002 parameter set unless a species card says otherwise. Electron transport J from the non-rectangular hyperbola with curvature 0.7 and quantum yield 0.3 (electrons per photon, absorbed PAR).
- **Medlyn conductance.** gs = g0 + 1.6 (1 + g1 / sqrt(D)) A / Ca, with D in kPa, g0 \~0.01 mol m⁻² s⁻¹. Couple through Ci = Ca - 1.6 A / gs (boundary layer ignored in M0). Solve by Newton on Ci with a bisection fallback on \[0, Ca\], tolerance 1e-6 µmol mol⁻¹, max 50 iterations; return the limiting case flag for the inspector.
- **Transpiration.** E = gs × D / P (mol m⁻² s⁻¹), converted to kg per hour per leaf.
- **Expansion curve.** Beta function of Yin et al. 2003: y(t) = (1 + (te - t)/(te - tm)) × (t/te)^(te/(te - tm)) for 0 ≤ t ≤ te, where t is thermal age, te the duration and tm the time of maximum rate. Hourly potential growth = potential × (y(t + dt) - y(t)). Check: y(te) = 1.
- **Thermal time.** Hourly: max(0, f(T)) × 1/24 day, with f the three-point response normalized so f(T\_opt) = T\_opt - T\_base; this makes °C·day totals comparable to the literature sums at temperatures below the optimum.
- **Solar.** Declination, hour angle, elevation and azimuth from standard formulas; extraterrestrial radiation and a clearness index give the direct and diffuse split (Erbs-style correlation). Day length from the sunrise hour angle.
- **Transport clamp.** After computing all edge fluxes from the same concentrations (Jacobi, not Gauss-Seidel, for determinism and order-independence), scale down any organ's total outflow so its reserve cannot go below zero from transport alone.
- **Hash.** Hash every f64 as its bit pattern in a fixed traversal order (plants by id, organs by id, soil cells by index). Never hash floats after formatting.

### 11.5 Determinism checklist (run before closing WP9)

- [ ] Three consecutive runs of each scenario give one hash.
- [ ] A run with an extra plant added does not change the first plant's hash of its own state.
- [ ] A run with rayon forced to one thread matches the multi-threaded hash.
- [ ] Snapshot at day 30 and resume gives the day-90 hash of the uninterrupted run.
- [ ] The replay of a recorded action log gives the hash of the original session.
- [ ] grep finds no HashMap iteration, no thread\_rng, no std f64 transcendental calls in sim crates (a CI script does this).

### 11.6 Testing strategy

- **Unit tests** next to code for every equation, with the model card's example values as the test case.
- **Property tests** (proptest) for invariants: mass never shrinks except at death, reserves never negative after transport alone, ledgers close, expansion curve integrates to one.
- **Fixture tests** compare to committed CSVs generated by validation/ scripts. Fixtures are regenerated only by running the scripts; the PR that regenerates them explains why.
- **Golden tests** hash each scenario at every sim-day and compare to tests/golden/\<platform>/\<scenario>.txt. A model version bump regenerates them in the same PR.
- **Performance tests** with iai-callgrind on the single-plant scenario (instruction count, stable on CI) and criterion locally.
- **Snapshot tests** (insta) of the first five CSV rows and of loaded RON to catch schema drift.
- **Validation report** (WP11) runs on every push and weekly; it fails the build when a §8 target is missed after WP12.

### 11.7 CI

- linux.yml: fmt, clippy pedantic, cargo-deny, nextest for all sim crates, golden hashes, validation report as an artifact, dependency-rule script. Caches cargo registry and target.
- windows.yml: build, nextest for sim crates, Windows golden hashes, release packaging of ps-app from M1.
- render-smoke.yml (M1, best effort, allowed to fail): ps-app headless frame via software Vulkan.
- weekly.yml: the full scenario set and validation report on a schedule.
- Bevy is built only in the M1 jobs; sim jobs never compile it.

### 11.8 Documentation conventions

- **Decision records** in docs/decisions/NNNN-title.md: Context, Decision, Consequences, Date. Seed them from §12.
- **Model cards** in docs/models/\<process>.md: Purpose; Equations; Parameters (table: name, value, unit, source); Inputs and outputs with units; Encoded or emergent; Known limits; Validation test ids; References. The app shows these in a side panel (§5.7), so write them for a curious player, not only for a developer.
- **docs/STATUS.md**: current WP, last validation report summary, open questions for the owner.
- **docs/bevy-notes.md** (M1): the pinned versions, the idioms used, and the gotchas hit.

### 11.9 Bevy notes for M1 (to confirm against the pinned version on day one)

- Lock Bevy and bevy\_egui together on the first day of M1; check their compatibility table before writing code.
- Use the dynamic\_linking feature in dev profiles and a fast linker to keep iteration under a minute.
- The sim runs on its own std thread. Publish snapshots through a triple buffer (or an Arc swap); the render thread reads the latest without blocking. Actions flow back through a channel and are applied by the sim thread at its current tick.
- Leaves: one shared mesh per template with per-instance transform and color; Bevy batches identical mesh-material pairs, and a custom instanced material is the fallback if entity count hurts.
- Stems: rebuild the tube mesh in ps-mesh and replace the Mesh asset only when the organ's size changed by more than 2% since the last rebuild.
- Lenses are a material parameter (a per-instance color and an opacity), never a second mesh set.
- Charts: egui\_plot with the daily time series from the snapshot; downsample above 2,000 points.
- Screenshots and time-lapse frames via the screenshot API of the pinned version; write PNG next to the save.
- Windows uses the DX12 backend by default; test Vulkan too since the GTX 1070 supports both.

### 11.10 Anti-patterns to refuse

- Putting sim state in Bevy's ECS or letting a Bevy system mutate it.
- Using wall-clock time anywhere in the sim.
- f32, std transcendental functions, or HashMap iteration in sim crates.
- A parameter without a source, or a process without a model card.
- A new feature that is not a work package in §7; propose it for the next milestone instead.
- Fixing a failing validation target by widening the target instead of fixing the model, unless a decision record says why.

### 11.11 Suggested session plan for M0

| Session | Work | Output |
| --- | --- | --- |
| 1 | WP0, WP1 | Workspace, CI green, units and RNG, decision records, STATUS.md |
| 2 | WP2, WP3 | Weather and sun; organ graph and expansion curves |
| 3 | WP4, WP5 | Species schema and bean file; soil and water |
| 4 | WP6 | Light with the Python ray tracer fixture |
| 5 to 6 | WP7 | Gas exchange with plantecophys fixtures; respiration; phenology; allocation |
| 7 to 8 | WP8 | Herb architecture, roots, seed, reproduction |
| 9 | WP9, WP10 | World, action log, hash, CLI, CSV |
| 10 | WP11 | Validation harness and report |
| 11 to 13 | WP12 | Tuning to the §8 targets; golden hashes; M0 exit review with the owner |

Run WP13 (the Bevy smoke test on the owner's machine) during session 3 or 4 so the GPU path is proven before M1 starts.

## 12. Decision log and changelog

### 12.1 Decisions (do not reopen without the owner)

| Date | Decision | Why |
| --- | --- | --- |
| 2026-11-05 | All three goals (beautiful, believable, fun) carry equal weight; realism is measured, not felt | Owner's brief |
| 2026-11-05 | Desktop only, Windows, GTX 1070; no web requirement | Owner |
| 2026-11-05 | Scale roadmap: one plant, two pots, plot, fields and forests | Owner |
| 2026-11-05 | Real-time with speed control plus skip buttons; not turn-based | Owner |
| 2026-11-05 | Sandbox first, then challenges; breeding from seeds is the long-term hook | Owner |
| 2026-11-05 | Science on display and toggleable, as realistic as possible | Owner |
| 2026-11-05 | Stylized real 3D, orbitable diorama with a soil cutaway; no photorealism | Design discussion |
| 2026-11-05 | Temperate climate, four seasons, adjustable latitude and weather | Default accepted by owner |
| 2026-11-05 | Species are data (genome parameters) on one engine; detail scales with zoom | Design discussion |
| 2026-11-05 | First species: bush bean archetype (branches visibly, fast cycle, self-pollinating) | Adversarial review (sunflower barely branches) |
| 2026-11-05 | Carbon is the sole authority on growth; architecture only places it; per-organ reserves | Adversarial review |
| 2026-11-05 | M1 split into M0 headless core and M1 two-pot visible proof | Adversarial review |
| 2026-11-05 | Rust sim workspace, Bevy pinned, egui; Godot via gdext is the runner-up | Architecture review |
| 2026-11-05 | Gas exchange solved per light class, not per leaf | v3 review, finding F1 |
| 2026-11-05 | Allocation algorithm of §3.9 with a pre-decided three-pool fallback | v3 review, finding F2 |
| 2026-11-05 | No catch-up growth: unmet demand shrinks final organ size | v3 review, finding F5 |
| 2026-11-05 | Tree moves to M4; M2 growth forms are pole bean (habit locus) and a grass | Owner, on v3 recommendation |
| 2026-11-05 | Hand-pollination is the M3 breeding action; selfing is the default | Owner, on v3 recommendation |
| 2026-11-05 | No real-plant experiment; all calibration data from published studies | Owner |
| 2026-11-05 | Pole bean twines (no tendrils), 1.5 m by day 60 and 2 m by day 75; spring wheat is the grass; WP28 uses light linkage, two-tier mutation and a seed-colour rule table (decision 0023) | Owner, on the M2/M3 research recommendations |
| 2026-11-05 | This v3 complete plan supersedes the v2 plan and the v3 review doc | Owner |
| 2026-11-05 | Implementation is handed to a fresh session using §11 | Owner |

### 12.2 Review findings resolved in this plan

From the v3 review of the v2 plan. Listed so nobody re-finds them.

| # | Finding | Resolved in |
| --- | --- | --- |
| F1 | Per-leaf hourly gas exchange does not fit the plot budget | §3.8 light classes |
| F2 | Allocation rule named but not specified | §3.9 |
| F3 | Seed reserves missing | §3.10 |
| F4 | Reproduction thin: no pod set, abortion or seed fill | §3.11 |
| F5 | Expansion curve scaling ambiguous | §3.9 step 5 |
| F6 | Fixed voxel size wrong for seedlings and plots | §3.5 adaptive grid; per-voxel at M2 |
| F7 | Validation without datasets or tolerances | §8 |
| F8 | No step order | §3.2 |
| F9 | Temperature only drove phenology | §3.11 growth multiplier, frost, heat |
| F10 | Legume nitrogen fixation ignored | §3.6 nodules, WP24 |
| F11 | CO2 dropped as a variable | §2.5 experiment panel |
| F12 | Lenses had no units, scales or explanation | §5.2, §5.7 |
| F13 | Nothing to do between M1 and M3 | §2.5 experiment mode, seed library |
| F14 | Bevy version lock had no date | WP13, day one of M1 |
| F15 | No model documentation convention | §11.8 |

### 12.3 Changelog

- **v1** (2026-11-05): first design draft from the discussion.
- **v2** (2026-11-05): adversarial review worked in (carbon authority, local reserves, turbid-medium light, FTSW, coupled gas exchange, shade avoidance, determinism design, M0/M1 split, bush bean); architecture review worked in (Rust, Bevy, egui).
- **v3 review** (2026-11-05): fifteen findings; expanded model, work packages, validation, lenses, later milestones, risks.
- **v3 complete** (this document, 2026-11-05): v2 and the v3 review merged into one plan; owner decisions applied; data formats with examples; implementation guide; decision log.

## 13. Sources

Project documents superseded by this one:

- [Plant growth simulation: design plan (v2)](https://claude.ai/artifact/UkAzFjE4Y3BNRDpZM9QMyR)
- [Plant growth simulation: design review and expansion (v3)](https://claude.ai/artifact/X6Xa6Tjf6MRzNh71BDXKzx)
- [Plant growth simulation: options and priorities (research report)](https://claude.ai/artifact/NR9fpmSdNqFZCNpteHCeGf)

Studies behind the validation targets (abstracts read via Consensus; full texts to be opened when digitizing data in WP11):

- [Phenology, mass accumulation patterns and growing degree days in common bean](https://consensus.app/papers/details/65b57d5aad2e59e1a634d608222b4970/?utm_source=claude_desktop) (Pinzón-Sandoval et al., 2024)
- [Thermal sum and phenological descriptions of growth stages of the common bean according to the BBCH scale](https://consensus.app/papers/details/f99d70d2ad125b88bd90b297eb6a7a26/?utm_source=claude_desktop) (Cavalcante et al., 2020)
- [Interaction among growth cycle, photosynthesis, and nodule enzymatic activity in Phaseolus vulgaris inoculated with Rhizobium tropici CIAT 899](https://consensus.app/papers/details/c2e8163fd8c45570a67062d5bb53982d/?utm_source=claude_desktop) (Conceição et al., 2026)
- [Fenología y fenometría de una variedad y una línea de frijol en la zona occidental de El Salvador](https://consensus.app/papers/details/f12981865fff580d98042cf80f5d8496/?utm_source=claude_desktop) (Ventura Elías, 2016)
- [Growth performance and yield contributing traits of Kharif French bean varieties under different dates of sowing](https://consensus.app/papers/details/0acea9340180597f8465c3c495e4b046/?utm_source=claude_desktop) (Kumar et al., 2025)
- [Evaluation of growth performance of common bean genotypes during two phenological phases](https://consensus.app/papers/details/59277c779b9d5689aab68ff3e5ddea22/?utm_source=claude_desktop) (Gebeyehu, 2019)
- [Evaluation of some haricot bean cultivars for growth characters and dry matter production](https://consensus.app/papers/details/b9a69a6d643158bba3217253856933f5/?utm_source=claude_desktop) (Merga, 2021)
- [Características fotossintéticas de Phaseolus vulgaris L.](https://consensus.app/papers/details/879c7fe0f71a538b8407a5c7943b21ef/?utm_source=claude_desktop) (Pimentel et al., 2011)
- [Biochemical limitations to carbon assimilation in C3 plants: a retrospective analysis of the A/Ci curves from 109 species](https://consensus.app/papers/details/d5f18c7251575a80b536d7a8d4178160/?utm_source=claude_desktop) (Wullschleger, 1993)
- [Accumulation of biomass, physiological characteristics and grain yield of bean cultivars under irrigated and dry regimes](https://consensus.app/papers/details/b97819d779ef59e28cc7d3e5a87920d1/?utm_source=claude_desktop) (Gomes et al.)
- [Behavior of common bean plants subjected to two irrigation systems](https://consensus.app/papers/details/1504985d05765a26ab566f8216abf698/?utm_source=claude_desktop) (Guerrero-Domínguez et al.)
- [Differential sensitivity to high temperature of stages in the reproductive development of common bean](https://consensus.app/papers/details/d0e154d8f0035b178b28813744fe424a/?utm_source=claude_desktop) (Gross and Kigel, 1994)
- [Multi-trait phenotyping under reproductive-stage heat stress reveals heritable reproductive and yield indicators of tolerance in common bean](https://consensus.app/papers/details/0f6080758ba55e87a0bba510b6043b75/?utm_source=claude_desktop) (Sharda et al., 2026)
- [Influence of plant density and growth habit of common bean on leaf area development and N accumulation](https://consensus.app/papers/details/11b9dd4ab6fd51aa9d2da96ca8314961/?utm_source=claude_desktop) (Clavijo Michelangeli et al., 2019)

Models and software referenced by name (standard references; confirm versions and licenses when the work package that uses them starts): Farquhar, von Caemmerer and Berry 1980 (photosynthesis); Medlyn et al. 2002 (temperature responses) and 2011 (stomatal conductance); Yin et al. 2003 (beta growth function); Sinclair and Ludlow 1986 (FTSW); Sinoquet and co-workers' RATP (turbid-medium light); de Pury and Farquhar 1997 (two-leaf canopy); GreenLab and ADEL (organ expansion); LIGNUM and L-PEACH (carbon-driven functional-structural models); Palubicki et al. 2009 (self-organizing trees); CRootBox and CPlantBox (root architecture); plantecophys (R, gas-exchange fixtures); pvlib (solar position); DSSAT CROPGRO-Drybean (second reference model).
