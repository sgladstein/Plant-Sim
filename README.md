# Plant-Sim

A simulation-first plant growth sandbox: watch plants grow from seed in sped-up time, with the science visible through toggleable lenses, and later breed them from seeds.

Start with [docs/design-plan.md](docs/design-plan.md). It is the complete v3 plan: experience, simulation model, data formats, milestones and work packages, validation targets, architecture, and the implementation guide (section 11).

- Progress: [docs/STATUS.md](docs/STATUS.md)
- Rules for agents: [CLAUDE.md](CLAUDE.md)
- Decisions: [docs/decisions/](docs/decisions/)
- Model cards: [docs/models/](docs/models/)

## Build

```sh
cargo test --workspace
```

Rust 1.97 (pinned in rust-toolchain.toml). M0 is headless; there is nothing to run yet beyond the tests.
