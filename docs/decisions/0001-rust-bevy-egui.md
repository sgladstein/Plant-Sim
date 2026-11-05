# 0001: Rust simulation workspace, Bevy renderer, egui UI

- **Date:** 2026-11-05
- **Status:** accepted (owner)

## Context
The project needs a deterministic, testable simulation written entirely by AI agents,
a stylized 3D renderer for a Windows PC with a GTX 1070, and UI for lenses, an inspector
and charts. Options compared in design plan §10.2.

## Decision
A pure-Rust simulation workspace with no engine dependency; Bevy as the renderer,
pinned to one version on day one of M1 (WP13); egui (bevy_egui, egui_plot) for UI.
Godot 4 via gdext is the runner-up.

## Consequences
- Compile-time checks and clippy pedantic catch many agent mistakes.
- Determinism is controllable (f64, libm, ordered reductions, no HashMap iteration).
- Everything is code or text; M0 builds and tests headlessly in a Linux container.
- Bevy API churn is a risk (R8): pin, keep docs/bevy-notes.md, confine Bevy to three thin crates.
- Switch to Godot via gdext if the triggers in §9 fire; the sim crates do not change.
