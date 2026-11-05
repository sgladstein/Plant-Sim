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
