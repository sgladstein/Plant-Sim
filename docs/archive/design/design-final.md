# Plant Growth Simulation — Design Plan

*2026-11-05. Revised after adversarial review (v1 → v2 changelog at the end). Target machine: Windows PC, GeForce GTX 1070. Architecture in §10.*

## 1. Vision

A simulation-first plant growth sandbox: watch plants grow from seed in sped-up time, where form emerges from real, inspectable rules (light, water, nutrients, carbon) rather than scripted animation. Three goals at once:

- **Beautiful to watch:** a cozy, stylized 3D diorama with emergent plant shapes.
- **Scientifically believable:** established plant-science models, with internals visible through toggleable lenses and charts. "As realistic as possible" within real-time limits, with realism defined as measurable tests (§8).
- **Fun to play:** sandbox → challenges → seed collection and breeding.

Scale grows over time: one plant → small plot → fields and forests.

### Non-goals (for now)
Photorealism; web distribution; multiplayer/monetization/mobile; agronomic decision-support accuracy.

## 2. Core experience

### 2.1 The diorama view
- A soil block with plant(s) on top; orbit, pan, zoom.
- **Roots shown rhizobox-style:** the soil block is a thin-ish slab (as in real root experiments) so most roots are visible through the glass front, plus an **X-ray lens** that fades soil to show the full 3D root system.
- Sun follows latitude and season; day/night; weather (rain, clouds, temperature; wind sway visual only at first).
- **Auto-framing camera:** the plant changes size ~1000× (mm seedling to 1–2 m), so the camera tracks plant bounds by default, with manual override.

### 2.2 Time controls
- Continuous simulation with speed multipliers (1 sim-hour/s up to ~1 sim-week/s).
- Pause; step buttons **+1 hour, +1 day, +1 week, +1 season**.
- **High-speed rendering mode:** above ~1 sim-day/s, show daily-averaged lighting and smoothed stress visuals to avoid strobing day/night and flickering wilt.
- Replay/scrubbing comes from the action log + organ growth curves (§3.9), not stored video.

### 2.3 Science lenses (toggleable)
- **Light:** absorbed PAR per leaf; sunlit vs. shaded fraction.
- **Water:** soil water content in the slab; plant water stress tint.
- **Carbon:** source/sink flows, per-organ reserves, organ biomass.
- **Nutrients (M2+):** soil N; leaf N; deficiency yellowing.
- **Phenology/age:** organ age and plant stage.
- **Charts:** biomass by organ, leaf area, height, photosynthesis, transpiration, water/carbon balance.
- **Inspector:** click any organ to see its state with units.

### 2.4 Fun-to-watch moments (designed, not accidental)
- Wilting and visible recovery within sim-hours of watering.
- Stems visibly bending toward light; branches flushing on the sunny side.
- Flowering as an event with a camera cue.
- Side-by-side comparisons (same seed, different environment) as a core mode.

### 2.5 Player actions (sandbox)
Plant seed; water / irrigation schedule; fertilize (M2); prune (M2); harvest; set latitude/season/weather presets (drought, shade, heat wave); place obstacles/trellis (M2).

### 2.6 Progression (later)
- **Challenges (M3):** e.g. most fruit under drought; scored from sim outputs; pinned to a model version and seed.
- **Seeds and breeding (M3):** seeds from mature plants; crossing via a genotype→phenotype system (§3.8); seed library; trait discovery.

## 3. Simulation model

### 3.1 Principles (locked decisions)
1. **Carbon (and later nitrogen) is the single authority on how much grows.** Architecture models decide only *where* growth goes and *what shape* it takes. Light/shade signals act as bud activation probabilities and sink-strength weights, never as biomass. (LIGNUM / L-PEACH style FSPM split.)
2. **Carbon is local:** reserves are stored per organ/branch module and moved with transport-resistance (distance) weighting, so shaded branches can run a deficit and self-prune emergently. Fallback rule if needed: a branch with negative net carbon balance for N days dies.
3. **Headless sim, separate from rendering.**
4. **Determinism (§3.10):** action log + world seed + model version is the source of truth.
5. **Engine shared, architecture modules per growth form** (herb/bush, grass, vine, tree), all sharing the organ graph, physiology, and environment.
6. **Organs grow as functions of time** (birth time + expansion curve) so rendering, interpolation, and replay are smooth.
7. **Every quantity has units**; data files are unit-tagged and versioned.

### 3.2 Plant structure
- An MTG-like **organ graph**: internodes, nodes/buds, leaves (blade + petiole), root segments, flowers, fruits.
- Each organ stores: id, parent, type, birth time, expansion curve params, dimensions, structural biomass, **carbon reserve**, **nitrogen content** (reserved from M0 even before the N model), local light coefficients, water status.
- Buds: activate / stay dormant / die based on light signal, local carbon availability, and apical dominance (a signal from active tips down the axis).
- **Shade avoidance:** a red:far-red proxy from the light model increases internode elongation and specific leaf area under neighbor shade. (Needed so "shaded plants get taller and thinner" is real.)
- **Tropisms:** phototropism toward the light-weighted direction; gravitropism; roots also hydrotropism.
- **Secondary growth is a carbon sink:** woody archetype uses pipe-model targets; herbs use stem allometry from allocated carbon.

### 3.3 Architecture modules
- **Herb/bush (M0–M1):** sympodial or monopodial branching with axillary buds that respond to light; e.g. bush bean or tomato-like.
- **Tree (M2):** self-organizing bud fate in the Palubicki spirit but without space-colonization markers; driven by the light signal and carbon.
- **Grass/cereal (M2+):** tillering from the base, intercalary growth.
- **Vine (M2+):** tendril search and attachment to supports.
- **Roots (all):** CRootBox-style parametric root types (tap, laterals, order-based branching, tropisms), growth limited by allocated carbon and soil conditions.

### 3.4 Light (environment)
- **Turbid-medium voxel model** (Beer–Lambert with a leaf-angle G-function, RATP/Sinoquet style):
  - Direct beam traced along the sun direction; diffuse light from ~6–16 sky sectors.
  - Each leaf split into sunlit and shaded fractions.
- **Multi-rate:** recompute per-leaf exposure coefficients when geometry changes (daily); multiply by hourly above-canopy radiation (direct/diffuse split). Avoids hourly ray tracing and the averaging bias of a single shade index.
- Also produces the R:FR proxy for shade avoidance.
- **Validation:** compare against an offline ray-traced reference for a single plant.
- GPU acceleration is allowed for rendering, but the sim's light model must be reproducible (§3.10).

### 3.5 Soil and water
- **Coarse 3D soil grid** (~2–5 cm cells): water content, temperature, nitrogen (M2), texture.
- Water: rain/irrigation infiltration, bucket-style drainage between cells with lateral redistribution, soil evaporation, root uptake weighted by root length density per cell. Known accuracy limit documented (no full Richards equation).
- **Water stress via fraction of transpirable soil water (FTSW, Sinclair)**, applied in one place (stomatal conductance + expansion growth) to avoid double counting.
- A retention curve (van Genuchten/Campbell) is added only if water potential is shown in the UI.

### 3.6 Physiology
- **Coupled leaf gas exchange from day one:** Farquhar–von Caemmerer–Berry photosynthesis + Medlyn stomatal conductance, giving consistent photosynthesis and transpiration (and water-use efficiency). Separate sunlit/shaded leaf calculations. Validate against fixtures generated with the R package plantecophys.
- **Respiration:** maintenance scaled by nitrogen content and temperature (Ryan 1991, Q10); growth respiration as a construction cost per gram.
- **Carbon allocation:** local reserves → sinks by sink strength (organ demand from expansion curves) with transport-resistance weighting; phenology shifts sink priorities (e.g. fruits become strong sinks). Root:shoot balance arises from where carbon is needed (water-limited growth raises root sink activity) and is labeled as encoded where it's a rule.
- **Phenology:** thermal time with species base temperature; optional photoperiod; **germination requires soil temperature and moisture thresholds**. Stages: germination, vegetative, flowering, fruiting, senescence; perennial dormancy/leaf fall (M2).
- **Death and shedding:** carbon-starved organs/branches die and drop. Litter decomposition and nutrient return is M2+.

### 3.7 Time stepping
- Hourly physiology and environment step; daily architecture step (bud decisions, light coefficient update).
- **Weather:** hourly from presets or files; daily weather files disaggregated to hourly (sinusoidal temperature, radiation from solar geometry).
- Skip buttons run headless as fast as possible.
- **Performance budgets per milestone** in ms per sim-day (e.g. M1 single plant: < 5 ms per sim-day to allow ~1 sim-week/s with headroom; M2 plot: < 20 ms per sim-day for 30 plants).

### 3.8 Species, genotype and phenotype
- **Genotype:** diploid loci with alleles (dominance, some pleiotropy).
- **Phenotype:** bounded heritable parameters (branching, leaf size, Vcmax, root depth tendency, phenology thresholds…) computed from the genotype by mapping functions.
- **Fixed physiological constants** kept separate from heritable parameters.
- **Trade-offs come from physiology costs**, not lookup tables: e.g. high Vcmax costs N and maintenance respiration; deep roots cost carbon.
- Species files are human-readable, unit-tagged and schema-versioned. The genotype→phenotype layer exists in the data format from M0, even before breeding.
- Real vs. invented species stays open: start with real-ish archetypes calibrated against literature; invented ones are new genomes.

### 3.9 Organ growth as a function of time
- Every organ has a birth time and an expansion curve (beta function over thermal time, as in GreenLab/ADEL), scaled by realized carbon supply.
- The renderer evaluates organ size at fractional time → smooth time-lapse, no popping, and replay scrubbing.

### 3.10 Determinism
- **Source of truth:** world seed + model version + **action log** (player actions timestamped in sim ticks). Snapshots are only a cache for fast loading and scrubbing.
- **Random streams:** counter-based RNG keyed by (world seed, entity id, tick, purpose), so adding a plant doesn't perturb others.
- Fixed-order reductions for any parallel sums.
- **Guarantee level:** bit-exact on the same build and platform; cross-build/platform within tolerances. Sim math runs on CPU; GPU used for rendering (and only for sim math later if it is made reproducible).
- Golden-run hash test in CI.

### 3.11 Scale and LOD
- Plant and plot: full organ model.
- Fields/forests (M4): aggregate canopy/cohort models with procedural geometry at distance.
- **Promotion is a plausible regeneration, not the true history.** Canonical conserved quantities (carbon and N per organ class, leaf area index, height, root mass per soil layer) must be preserved through demote→promote, tested. If that proves too hard, M4 detail is visual-only.

## 4. Visual design
- Stylized 3D: botanical illustration meets cozy indie game; soft lighting, gentle palette.
- Stems as tapered tubes; leaves as instanced templates oriented/scaled by sim; flowers/fruit as templates.
- Physiological cues: wilting (droop), N deficiency (yellowing), senescence (browning), leaf drop, ripening colors.
- Lenses recolor with consistent, colorblind-safe scales.
- Must run smoothly on a GTX 1070 at 1080p/1440p for a plot of 30 plants.

## 5. UI
Top bar (date/season, speed, step buttons); left tools; right lenses + inspector; bottom collapsible charts; save/load; CSV export; screenshot/time-lapse capture.

## 6. Data formats
- Species/genome files: human-readable, unit-tagged, schema-versioned.
- Saves: model version + world seed + action log (+ optional snapshot cache); migration policy: saves from older model versions load in "replay with old model unavailable → re-simulate with warning" mode.
- Challenges pin the model version.
- CSV export of time series and per-organ tables.

## 7. Milestones (re-cut)

### M0 — Headless core (no 3D)
- One bush-bean-like annual (branching that responds visibly to light).
- Organ graph with units; carbon authority + local reserves; coupled FvCB + Medlyn; respiration; thermal-time phenology to flowering; germination thresholds.
- Turbid-medium light with daily coefficients × hourly radiation; coarse 3D soil bucket with FTSW stress.
- Genotype→phenotype data format; action log + keyed RNG; determinism golden-run hash.
- Conservation checks (water balance closure, carbon accounting).
- CSV export and plotted charts.
- **Exit:** biomass, leaf area and height curves within literature ranges; conservation checks pass; deterministic.

### M1 — Visible proof
- Tube-and-leaf renderer with orbit and auto-framing; expansion-curve interpolation.
- Play/pause/speed and +hour/+day/+week steps; high-speed rendering mode.
- Light lens, X-ray root lens, inspector, charts panel.
- Actions: water, shade preset, drought preset.
- **Two pots side by side, same seed, different environment** (sun vs. shade, or wet vs. drought). This screen proves the vision.

### M2 — Plot and sandbox
- 5–30 plants, competition for light and soil; tree and one more growth form.
- Nitrogen model, fertilize, prune, obstacles/trellis, weather files.
- Save/load UI, latitude/season controls, water and carbon lenses, seed fill and harvest.
- Perennial dormancy and leaf fall; litter decomposition.

### M3 — Seeds, breeding, challenges
Seed library, crossing/mutation, trait discovery, challenges with scoring, replay timeline.

### M4 — Fields and forests
Aggregate LOD, large terrain, promotion/demotion with conservation tests (or visual-only detail).

## 8. Validation ("realism" as measurement)
1. **Process level:** each model vs. a reference implementation or dataset (FvCB vs. plantecophys (R); light vs. ray-traced reference; soil water vs. an analytic infiltration case).
2. **Conservation:** water balance closure and carbon accounting checked every step in debug builds.
3. **Whole plant:** comparison to published growth series for the chosen species.
4. **Emergent behaviors,** each tagged **encoded** (rule written in) or **emergent** (falls out of other rules):
   - Shaded plant: taller, thinner, larger specific leaf area (encoded via shade avoidance; magnitude emergent).
   - Drought: higher root:shoot ratio (partly encoded; label honestly).
   - Shaded lower branches self-prune (emergent from local carbon).
   - Biomass curve roughly sigmoidal (emergent).
5. **Regression:** golden-run and determinism tests in CI, critical since an AI writes the code.

## 9. Risks
- Realism vs. performance → multi-rate stepping, light coefficients, budgets per milestone, LOD.
- Parameter explosion → literature defaults, genotype→phenotype bounds, costs-based trade-offs.
- Model drift from AI-written code → validation suite and golden runs.
- Scope creep → milestone exits are concrete and testable.

## 10. Architecture (from architecture review)

**Decision: Rust.** A pure-Rust simulation workspace with no engine dependency, a **Bevy** renderer pinned to one version, **egui (bevy_egui + egui_plot)** for UI and charts. Python and R generate validation reference data and do analysis; nothing shipped depends on them.

### Why
- **Compile-time safety catches AI mistakes:** types, ownership, exhaustive matching, clippy with warnings as errors.
- **Determinism is controllable:** no implicit fused multiply-add or fast-math; `libm` for transcendental functions; fixed-order parallel reductions; ordered maps only in the sim.
- **Everything is code or text:** no editor or scene files; species, scenarios, saves in RON/TOML/JSON via serde. M0 and most of M1 can be built and tested headlessly in a Linux container.
- **Bevy covers M1's rendering:** instanced leaves, runtime mesh updates for growing stems, stylized shading, cameras, screenshots. Runs on wgpu (DX12/Vulkan), fine on a GTX 1070.
- **egui fits the inspector, lens panels, and time-series charts.**

### Options compared (summary)
| Option | Verdict |
|---|---|
| Rust sim + Bevy + egui | **Chosen.** Best safety, determinism, text-only workflow, headless testing. Risk: Bevy API churn every ~3–4 months. |
| Rust sim + Godot 4.6 (gdext) | **Runner-up.** Better stylized rendering, UI widgets, stability; but needs the editor and export templates, scene files are less AI-friendly. |
| C# + Godot .NET | Fast iteration, but weaker determinism control and compile-time safety. |
| Rust + raw wgpu + egui | Months of extra renderer work. |
| C++ + bgfx/raylib + ImGui/ImPlot | Great charts, weaker safety for AI-written code. |
| Unity / Unreal | Editor-centric, can't develop in a container; overkill. |
| Python core + viewer | Too slow for organ-graph walks; weakest safety. |

### Stack
- **Rust stable, one Cargo workspace (edition 2024).**
- **Sim:** structs-of-arrays organ graph with integer IDs (arena, not Bevy's ECS); rayon with fixed chunking and in-order reductions; f64; `libm`; counter-based RNG keyed by (world seed, entity, tick, purpose).
- **Units:** lightweight newtypes (e.g. `GramsC`, `MolM2S`, `Celsius`); data files carry units, checked on load.
- **Renderer:** Bevy, reading a read-only sim snapshot each frame; mesh generation in an engine-free crate.
- **UI:** bevy_egui + egui_plot with a custom cozy theme.
- **Testing/CI:** cargo-nextest, proptest (conservation properties), insta (snapshot tests), criterion and iai-callgrind (instruction-count perf budgets stable in CI), cargo-deny, clippy pedantic. GitHub Actions on Linux (fmt, clippy, tests, golden-run hash, validation comparison) and Windows (build, sim tests, Windows golden hash, release packaging). Best-effort headless render smoke test via software Vulkan.
- **Python/R role:** `validation/` scripts generate committed reference fixtures (FvCB + Medlyn via R plantecophys, Beer–Lambert, infiltration); Rust tests compare with tolerances. pandas/matplotlib reports compare CLI CSV output to published growth series. Optional PyO3 bindings for parameter sweeps and calibration.

### Workspace layout
```
crates/
  ps-units    unit newtypes                                   M0
  ps-rng      keyed counter-based RNG                         M0
  ps-graph    organ graph arena, organ types, expansion curves M0
  ps-species  species/genome schema, genotype→phenotype       M0
  ps-weather  presets, daily→hourly, solar geometry           M0
  ps-light    turbid-medium voxel light, sky sectors, R:FR    M0
  ps-soil     3D bucket soil grid, FTSW, uptake               M0
  ps-physio   FvCB+Medlyn, respiration, phenology, allocation M0
  ps-arch     growth-form modules (herb first)                M0
  ps-world    scheduler, action log, snapshots, state hash, conservation checks  M0
  ps-export   CSV time series and per-organ tables            M0
  ps-cli      headless runner: scenario → CSV / hash / bench  M0
  ps-py       optional PyO3 bindings                          M0+
  ps-mesh     tube/leaf generation, interpolation (engine-free) M1
  ps-view     Bevy plugin: scene sync, lenses, camera, high-speed mode  M1
  ps-ui       egui panels, inspector, charts, tools           M1
  ps-app      binary; two-pot comparison mode                 M1
data/species, data/scenarios   validation/   tests/golden/   benches/
```
**Rule:** `ps-world` and everything below it never depends on Bevy (enforced in CI). The renderer only reads state and sends actions through the action log, so saves and replays behave the same with or without graphics.

### Risks and mitigations
1. **Bevy API churn:** pin one version; upgrade only between milestones; keep Bevy code in three thin crates.
2. **AI agents mix up Bevy versions:** keep `docs/bevy-notes.md` for the pinned version; rely on the compiler.
3. **Determinism traps:** `libm`, ordered reductions, ban HashMap iteration in sim, no `target-cpu=native`, hash full state every sim-day in golden tests. Golden hashes stored per platform.
4. **Rendering can't be tested in the cloud:** geometry logic tested headlessly in `ps-mesh`; manual check on the GTX 1070 at each milestone exit.
5. **egui may look plain:** custom theme and fonts; fall back to Bevy UI for chrome in M2 if needed.
6. **Compile times:** Bevy dynamic linking in dev, fast linker, small sim crates.

**Switch to the runner-up (Godot via gdext) if** visual polish or UI needs outgrow Bevy + egui, Bevy upgrades cost more than about a week per milestone, or a human artist joins and wants an editor. The sim crates would stay unchanged.

*Unverified at review time: exact Bevy 0.19 release date and matching bevy_egui version; gdext's official Godot 4.6 support; reliability of headless Bevy rendering on software Vulkan.*

## Changelog v1 → v2
- Carbon is the sole authority on growth; architecture only places it (C1). Local per-organ reserves enable self-pruning (C2).
- Light: turbid-medium voxel model with direct/diffuse, sunlit/shaded leaves, daily coefficients × hourly radiation (C3).
- Added shade avoidance via an R:FR proxy; tests tagged encoded vs. emergent (C4).
- Determinism designed: action log, keyed RNG, CPU sim, guarantee level (C5).
- M1 split into M0 headless core + M1 visible two-pot proof; many features deferred to M2 (C6).
- Water stress via FTSW in one place; coarse 3D soil grid (I1). Coupled FvCB + Medlyn (I2). Secondary growth as carbon sink; pipe model only for trees (I3).
- Architecture modules per growth form; first species changed from sunflower to bush bean (I4).
- Organ expansion curves for smooth rendering and replay (I5).
- Genotype→phenotype layer with physiology-cost trade-offs (I6).
- LOD promotion honesty + conservation tests (I7). Data formats and versioning (I8). Validation section (I9).
- Watchability: auto-framing, anti-strobe mode, rhizobox + X-ray roots, designed feedback moments (I10).
- Minor: N-based maintenance respiration, germination thresholds, weather disaggregation, perf budgets.
