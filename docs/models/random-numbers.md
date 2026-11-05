# Random numbers

## Purpose
Plants are not identical. Whether a bud wakes up or a flower sets a pod involves chance.
The sim makes every such chance a pure function of *who* is rolling, *when*, and *why*,
so a garden replays exactly the same way every time from its seed, and adding a new
plant never changes what happens to the plants already there.

## Equations
For a key `(world_seed, entity, tick, purpose)` and draw index `n`:

- `h = mix64(mix64(entity ⊕ c) ⊕ rotl(tick, 29))`, with `mix64` the SplitMix64 finalizer
  and `c = 0x6A09E667F3BCC908`.
- counter = `[n, code(purpose) ⊕ rotl(h_lo, 16), h_hi, mix32(h_lo ⊕ code(purpose))]`
- key = `[lo32(world_seed), hi32(world_seed)]`
- `out = Philox4x32-10(counter, key)`; `u64 = out[0] | out[1] << 32`
- uniform in [0, 1) = `(u64 >> 11) × 2⁻⁵³`
- standard normal = Box–Muller on two uniforms with libm `sqrt`, `log`, `cos`.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| Philox rounds | 10 | - | Salmon et al. 2011 (recommended) |
| Purpose codes | see `ps_rng::Purpose::code` | - | design; never renumber |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | world seed | - | scenario file |
| in | entity id, tick, purpose | -, h, - | caller |
| out | u64, uniform f64, normal f64 | - | caller |

## Encoded or emergent
Encoded: independence of streams by key. Nothing biological lives here.

## Known limits
Entity and tick are compressed to 64 bits of the counter by a mixer, so two different
(entity, tick) pairs collide with probability about 2⁻⁶⁴ per pair. Purpose is kept
separately. The stream within one key is limited to 2³² draws.

## Validation
- `ps_rng::tests::philox_kat_*`: Random123 known-answer vectors.
- `ps_rng::tests::golden_first_draws`: pins the key layout.
- `ps_rng::tests::same_key_same_stream`, `new_entity_does_not_perturb`, `fields_are_separated`: property tests (WP1 done-when).
- `ps_rng::tests::uniform_mean_and_range`, `normal_moments`, `below_in_range`: distribution sanity.

## References
- Salmon, Moraes, Dror, Shaw (2011). Parallel random numbers: as easy as 1, 2, 3. SC11.
- Steele, Lea, Flood (2014). Fast splittable pseudorandom number generators (SplitMix).
