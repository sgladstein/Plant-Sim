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

## Session start
- Read docs/STATUS.md first. It says which WP is done, which is in progress, and what is open.
- Before pushing: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `python3 scripts/check_deps.py`, `python3 scripts/check_determinism.py`.
