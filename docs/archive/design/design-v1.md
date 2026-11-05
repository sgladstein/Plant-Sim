# Plant Growth Simulation — Design Plan (v1 draft)

*Draft 2026-11-05. Architecture (language/engine) intentionally not decided here; see the architecture review.*

## 1. Vision

A simulation-first plant growth sandbox where you can watch plants grow from seed in sped-up time, and where how they look comes out of real, inspectable rules (light, water, nutrients, carbon) instead of scripted animation. It serves three goals at once:

- **Beautiful to watch:** a cozy, stylized 3D diorama with emergent plant shapes.
- **Scientifically believable:** grounded in established plant-science models, with the internals visible through toggleable overlays and charts. "As realistic as possible" within real-time limits.
- **Fun to play:** sandbox first, then challenges, then seed collection and breeding as the long-term hook.

Scale grows over time: one plant → a small plot → fields and forests.

### Non-goals (for now)
- Photorealistic rendering.
- Web/shareable-link distribution (desktop is fine).
- Multiplayer, monetization, mobile.
- Matching a specific crop model's yield predictions to agronomic accuracy (we aim for believable, calibrated-to-literature behavior, not a decision-support tool).

## 2. Core experience

### 2.1 The diorama view
- A block of soil with plant(s) on top, which the player can orbit, pan, and zoom.
- The front face of the soil block is a **cutaway**: roots are always visible growing through soil layers (topsoil, subsoil, maybe a rock/clay layer).
- Sky with a sun whose path follows latitude and season; day/night cycle.
- Weather: rain, clouds (reducing light), temperature, wind (visual sway at first).

### 2.2 Time controls
- Continuous real-time simulation with speed multipliers (e.g. 1 sim-hour/sec up to 1 sim-week/sec).
- Pause; step buttons: **+1 hour, +1 day, +1 week, +1 season**.
- Timeline scrubber to replay recorded growth (history snapshots) — stretch goal for M1, required by M3.
- Simulation must be deterministic given a seed and the same player actions, so replays and challenges are fair.

### 2.3 Science "lenses" (toggleable overlays)
Each lens recolors the scene or adds a panel:
- **Light:** per-leaf absorbed light (PAR) heatmap; shadow volume visualization.
- **Water:** soil moisture in the cutaway; plant water stress tint.
- **Nutrients:** soil nitrogen (and later P/K) in the cutaway; deficiency symptoms (yellowing) on leaves.
- **Carbon:** flow of sugars from leaves (sources) to growing organs (sinks); organ-level biomass.
- **Age / phenology:** organ age, growth stage (seedling, vegetative, flowering, fruiting, senescence).
- **Charts panel:** biomass by organ over time, photosynthesis rate, water use, leaf area index, height.
- **Inspector:** click any organ to see its state (age, size, light, carbon, water potential).

### 2.4 Player actions (sandbox)
- Plant seed (choose species / genome) at a location.
- Water (can, or set irrigation schedule); fertilize.
- Prune (click a branch to cut); harvest fruit/seed.
- Move sun latitude / set season / set weather presets (drought, shade, heat wave).
- Place obstacles (wall, trellis, rock) to see growth respond.

### 2.5 Progression (later milestones)
- **Challenges:** e.g. "Grow the most fruit under drought," "Fill the trellis in one season," "Keep a tree alive under a canopy." Scored by sim outputs.
- **Seeds and breeding:** mature plants produce seeds; crossing two plants recombines their genomes with mutation; a seed library; discovering traits (shade-tolerant, drought-hardy, fast-climbing).

## 3. Simulation model

### 3.1 Principles
- **Simulation is headless and separate from rendering.** The sim produces state; renderers and UI read it.
- **Deterministic:** seeded RNG, fixed internal timestep, no dependence on frame rate.
- **Generic engine, species as data:** one growth engine; each species is a parameter set ("genome").
- **Level of detail (LOD) by scale:** organ-level detail for a single plant / small plot; simplified (aggregate) models at field/forest scale.
- **Calibrate against literature:** each process has a reference model and test cases showing it behaves within known ranges.

### 3.2 Plant structure
- Plant represented as a **graph of organs** (an MTG-like topology): internodes, nodes/buds, leaves, roots segments, flowers, fruits.
- Each organ: id, parent, type, age, dimensions (length, radius, area), biomass, local light, local water status, carbon reserve.
- Growth happens via **buds** (apical and axillary) that activate, stay dormant, or die depending on light, carbon, and hormonal-like signals (apical dominance modeled as a signal from tips down the stem).
- Shoot architecture: a **self-organizing bud model** (Palubicki et al. 2009) whose rules are parameterized by species (phyllotaxis angle, branching angle, internode length, apical dominance strength, gravitropism/phototropism).
- Root architecture: parametric root model in the style of CRootBox/CPlantBox (tap/lateral roots, branching density, gravitropism, hydrotropism toward moist soil, growth slowed by compacted layers).
- Secondary growth: stems thicken following the pipe model (cross-section supports leaf area above).

### 3.3 Environment
- **Light:** sun position by time/latitude; direct + diffuse components; cloud cover.
  - Near term: a **voxel light grid** (shadow propagation) computing light per organ cheaply, including mutual shading among plants.
  - Upgrade path: ray-traced or GPU-based radiation for higher fidelity in close-up.
- **Soil:** 3D grid of voxels (or layered columns) with water content, nitrogen, temperature, and texture (sand/loam/clay affects infiltration and holding capacity).
  - Water: rain/irrigation infiltration, drainage between layers (simplified Richards or tipping-bucket), evaporation, root uptake.
  - Nitrogen: mineralization, uptake, leaching with drainage.
- **Atmosphere:** temperature, humidity/VPD, CO₂ (constant by default, adjustable for experiments).
- **Weather:** generated from presets or loaded from a weather file (daily or hourly).

### 3.4 Physiology
- **Photosynthesis:** per leaf, from absorbed PAR. Start with a light-response curve (non-rectangular hyperbola) with temperature and water-stress modifiers; upgrade to the Farquhar–von Caemmerer–Berry model coupled to stomatal conductance (Medlyn/Ball–Berry).
- **Respiration:** maintenance (proportional to biomass, temperature-dependent Q10) and growth respiration (cost per gram built).
- **Water:** transpiration from leaves driven by VPD and stomatal conductance; supply from root uptake; water stress index reduces stomatal conductance and expansion growth. Plant wilts visibly under stress.
- **Carbon allocation:** a plant-level (or per-branch) carbon pool distributed to organs by **sink strength** (GreenLab-style), modulated by phenology and stress (e.g. drought shifts allocation to roots — functional balance).
- **Nutrients:** nitrogen uptake by roots, nitrogen content per leaf affecting photosynthetic capacity.
- **Phenology:** thermal time (growing degree days) and optionally photoperiod drive stages: germination, vegetative, flowering, fruiting, senescence; perennials have dormancy and leaf fall.
- **Death and shedding:** shaded or carbon-starved branches and leaves die and drop (self-pruning), producing litter that can return nutrients.

### 3.5 Time stepping
- Fixed internal step (e.g. 1 sim-hour) for physiology and environment; slower processes (architecture, bud decisions) can run less often (e.g. daily).
- Fast-forward runs more steps per frame; at high speed the sim may run on a background thread with the renderer interpolating.
- Skip buttons (+day, +week) run the sim headless as fast as possible and then refresh the view.

### 3.6 Species / genome
- A species is a parameter set covering architecture (branching, phyllotaxis, internode length, leaf shape/size), physiology (max photosynthesis rate, shade tolerance, water-use efficiency, root:shoot tendency), phenology (GDD thresholds, life span, annual/perennial), and appearance (colors, leaf/flower templates).
- Starting set of 3–4 archetypes with clearly different strategies, roughly modeled on real plants:
  1. **Fast annual herb** (sunflower/bean-like).
  2. **Climbing vine** (needs support; tendrils seek trellis).
  3. **Grass/cereal** (tillering, fibrous roots).
  4. **Slow woody tree** (perennial, secondary growth, dormancy).
- Invented species are just new parameter sets; breeding mixes parameter sets with mutation and trade-off constraints (no free lunch: e.g. fast growth costs drought tolerance).

### 3.7 Scale and LOD
- **Plant scale (M1):** full organ graph, per-leaf light, per-voxel soil.
- **Plot scale (M2):** same model for ~5–30 plants; performance budget drives voxel resolution.
- **Field/forest scale (M4):** aggregate models (big-leaf/canopy Beer–Lambert, cohort-based forest-gap style models) with procedural geometry generated from aggregate state; zooming into a plant could "promote" it to full detail.

## 4. Visual design

- **Style:** stylized 3D — botanical illustration meets cozy indie game. Soft lighting, gentle palette, real branch structure, simple leaf meshes/cards, readable at a glance.
- Geometry comes from the sim: stems as tapered tubes (generalized cylinders), leaves as instanced templates scaled/oriented by sim state, flowers/fruit as templates.
- Visual cues of physiology: wilting (leaf droop), yellowing (N deficiency), browning (senescence), leaf drop, fruit ripening colors.
- Smooth animation between sim states (interpolate organ growth) so sped-up time looks like a time-lapse, not jumps.
- Lenses recolor using consistent, accessible color scales.
- Camera: orbit/zoom diorama; later, zoom out to plot/field.

## 5. UI

- Top bar: time/date/season, speed control, step buttons.
- Left: tools (plant, water, prune, fertilize, obstacles, weather).
- Right: lenses and the inspector.
- Bottom: collapsible charts panel.
- Save/load full sim state; export data (CSV) for external analysis; screenshot / time-lapse recording.

## 6. Milestones

### M1 — Single plant, real rules (the foundation)
- One plant of one species (fast annual herb) grows from seed to seed in the diorama.
- Organ graph, self-organizing shoot, simple parametric roots.
- Voxel light grid; layered soil water (tipping bucket); light-response photosynthesis; respiration; sink-based carbon allocation; thermal-time phenology.
- Time controls (play/pause/speed, +hour/+day/+week).
- Lenses: light, water, carbon; inspector; charts (biomass, height, photosynthesis).
- Actions: water, move sun latitude/season, drought/shade presets.
- Deterministic, save/load, CSV export.
- Validation: sanity tests (e.g. shaded plant grows taller and thinner; drought increases root:shoot ratio; biomass curve is sigmoidal).

### M2 — Small plot and sandbox
- 5–30 plants, 3–4 species, competition for light and soil.
- Prune, fertilize (add nitrogen model), obstacles/trellis, weather presets and weather file import.
- Perennial support (dormancy, leaf fall) for the tree archetype.
- Performance work.

### M3 — Seeds, breeding, challenges
- Seed production, seed library, crossing and mutation, trait discovery.
- Challenge mode with scoring and fixed seeds.
- Replay timeline.

### M4 — Fields and forests
- Aggregate LOD models; large terrain; procedural geometry at distance; promote to full detail on zoom.

## 7. Risks and open questions
- **Realism vs. performance:** per-leaf physiology at hourly steps for many plants could be expensive. Mitigation: LOD, multi-rate stepping, profiling budget per milestone.
- **Parameter explosion:** many parameters per species; risk of unrealistic combos. Mitigation: start from literature values; constrained genome with trade-offs.
- **Validation:** how do we know it's "realistic"? Need a test suite of qualitative behaviors plus comparisons to published curves.
- **Visual continuity:** discrete bud events can cause popping; need interpolation.
- **Real vs. invented species:** decision deferred; genome system supports both.
