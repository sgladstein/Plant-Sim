# 0004: Philox4x32-10 with a mixed key layout for ps-rng

- **Date:** 2026-11-05
- **Status:** accepted (implementer; the plan left the algorithm open: "Philox or ChaCha8")

## Context
The plan requires a stateless RNG keyed by (world seed, entity id, tick, purpose) and
names Philox or ChaCha8. The key is 64 + 64 + 64 + 32 bits plus a draw index, more than
either generator's native key and counter.

## Decision
Philox4x32-10, hand-written (about 30 lines, no dependency), checked against the
Random123 known-answer vectors. The world seed is the 64-bit Philox key. Entity and tick
are mixed by SplitMix64 into 64 bits of the counter; purpose is combined separately;
the draw index is the low 32 bits. Exact layout in docs/models/random-numbers.md.

## Consequences
- Draws are pure functions; no RNG state in snapshots.
- Collision of two (entity, tick) pairs has probability about 2⁻⁶⁴; accepted.
- The layout is part of the model: changing it changes every golden hash and bumps the model version.
- Purpose codes must never be renumbered or reused.
