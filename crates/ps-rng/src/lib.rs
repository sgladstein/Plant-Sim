//! Keyed, stateless, counter-based random numbers.
//!
//! Every random draw in the sim is a pure function of
//! `(world seed, entity id, tick, purpose, n)`, where `n` is the index of the draw
//! within that key. There is no generator state to carry, save or advance, so:
//!
//! - the same key always gives the same stream, on any thread, in any order;
//! - adding a plant (a new entity) cannot shift another plant's draws;
//! - snapshots need no RNG state.
//!
//! The bit generator is Philox4x32-10 (Salmon et al. 2011, "Parallel random numbers:
//! as easy as 1, 2, 3"), checked against the Random123 known-answer vectors.
//! The world seed is the Philox key. The entity, tick and purpose are mixed into the
//! upper 96 bits of the counter, and `n` is the low 32 bits. See
//! docs/decisions/0004-counter-based-rng.md.

use serde::{Deserialize, Serialize};

/// Why a draw is being made. Each purpose is its own independent stream, so adding a
/// new kind of draw for an entity never changes the draws of an existing kind.
///
/// Codes are part of the model: renumbering one changes every golden hash. Add new
/// variants with new codes; never reuse a code.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Purpose {
    /// Weather noise (hourly disaggregation, cloud).
    Weather,
    /// Germination timing.
    Germination,
    /// Bud activation, dormancy or death.
    BudFate,
    /// Pod set or abortion of a flower.
    PodSet,
    /// Seeds per pod.
    SeedNumber,
    /// Phyllotaxis jitter.
    Phyllotaxis,
    /// Tropism jitter.
    Tropism,
    /// Root branching and insertion angle jitter.
    RootBranching,
    /// Genotype sampling from a species population.
    GenotypeSample,
    /// Recombination at meiosis.
    Recombination,
    /// Mutation at meiosis.
    Mutation,
    /// Light model jitter (ray offsets), if ever used.
    Light,
    /// A vine tip catching a support (WP22).
    Twining,
    /// Rot of an organ on the soil (WP22).
    Rot,
    /// Test-only purpose; never used by the model.
    Test,
}

impl Purpose {
    /// Stable numeric code mixed into the counter.
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Purpose::Weather => 1,
            Purpose::Germination => 2,
            Purpose::BudFate => 3,
            Purpose::PodSet => 4,
            Purpose::SeedNumber => 5,
            Purpose::Phyllotaxis => 6,
            Purpose::Tropism => 7,
            Purpose::RootBranching => 8,
            Purpose::GenotypeSample => 9,
            Purpose::Recombination => 10,
            Purpose::Mutation => 11,
            Purpose::Light => 12,
            Purpose::Twining => 13,
            Purpose::Rot => 14,
            Purpose::Test => 0xFFFF,
        }
    }
}

/// The full key of a random stream.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RngKey {
    /// World seed from the scenario.
    pub world_seed: u64,
    /// Entity id: a plant id, an organ id packed with its plant id, a soil id, etc.
    pub entity: u64,
    /// Sim tick (hours since scenario start) at which the draw is made.
    pub tick: u64,
    /// Why the draw is made.
    pub purpose: Purpose,
}

impl RngKey {
    /// Builds a key.
    #[must_use]
    pub const fn new(world_seed: u64, entity: u64, tick: u64, purpose: Purpose) -> Self {
        Self {
            world_seed,
            entity,
            tick,
            purpose,
        }
    }

    /// A stream over this key starting at draw 0.
    #[must_use]
    pub const fn stream(self) -> Stream {
        Stream { key: self, next: 0 }
    }
}

const PHILOX_M0: u32 = 0xD251_1F53;
const PHILOX_M1: u32 = 0xCD9E_8D57;
const PHILOX_W0: u32 = 0x9E37_79B9;
const PHILOX_W1: u32 = 0xBB67_AE85;

#[inline]
fn mulhilo(a: u32, b: u32) -> (u32, u32) {
    let p = u64::from(a) * u64::from(b);
    #[allow(clippy::cast_possible_truncation)]
    ((p >> 32) as u32, p as u32)
}

/// Philox4x32 with 10 rounds: a bijection of the 128-bit counter under a 64-bit key.
#[must_use]
pub fn philox4x32_10(counter: [u32; 4], key: [u32; 2]) -> [u32; 4] {
    let mut c = counter;
    let mut k = key;
    for round in 0..10 {
        if round > 0 {
            k[0] = k[0].wrapping_add(PHILOX_W0);
            k[1] = k[1].wrapping_add(PHILOX_W1);
        }
        let (hi0, lo0) = mulhilo(PHILOX_M0, c[0]);
        let (hi1, lo1) = mulhilo(PHILOX_M1, c[2]);
        c = [hi1 ^ c[1] ^ k[0], lo1, hi0 ^ c[3] ^ k[1], lo0];
    }
    c
}

/// `SplitMix64` finalizer: a strong 64-bit bijective mixer.
#[inline]
const fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[allow(clippy::cast_possible_truncation)]
fn split(x: u64) -> [u32; 2] {
    [x as u32, (x >> 32) as u32]
}

/// Counter words for a key and draw index.
fn counter_for(key: &RngKey, n: u32) -> [u32; 4] {
    // 96 bits of the counter carry (entity, tick, purpose). Entity and tick are mixed
    // separately and combined so that (e, t) and (t, e) differ.
    let h = mix64(mix64(key.entity ^ 0x6A09_E667_F3BC_C908) ^ key.tick.rotate_left(29));
    let [h0, h1] = split(h);
    [
        n,
        key.purpose.code() ^ h0.rotate_left(16),
        h1,
        mix32(h0 ^ key.purpose.code()),
    ]
}

#[inline]
fn mix32(x: u32) -> u32 {
    #[allow(clippy::cast_possible_truncation)]
    let y = mix64(u64::from(x)) as u32;
    y
}

/// The `n`-th 64-bit draw for a key. Stateless.
#[must_use]
pub fn draw_u64(key: &RngKey, n: u32) -> u64 {
    let out = philox4x32_10(counter_for(key, n), split(key.world_seed));
    u64::from(out[0]) | (u64::from(out[1]) << 32)
}

/// The `n`-th uniform draw in `[0, 1)` for a key, with 53 bits of precision. Stateless.
#[must_use]
pub fn draw_f64(key: &RngKey, n: u32) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let x = (draw_u64(key, n) >> 11) as f64;
    x * (1.0 / 9_007_199_254_740_992.0) // 2^-53
}

/// A cursor over one key's stream. Holds only the key and the next index, so it is
/// cheap to create where needed and never needs saving.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Stream {
    key: RngKey,
    next: u32,
}

impl Stream {
    /// The key of this stream.
    #[must_use]
    pub const fn key(&self) -> RngKey {
        self.key
    }

    /// Index of the next draw.
    #[must_use]
    pub const fn position(&self) -> u32 {
        self.next
    }

    fn advance(&mut self) -> u32 {
        let n = self.next;
        self.next = self.next.wrapping_add(1);
        n
    }

    /// Next raw 64-bit draw.
    pub fn next_u64(&mut self) -> u64 {
        let n = self.advance();
        draw_u64(&self.key, n)
    }

    /// Next uniform draw in `[0, 1)`.
    pub fn uniform(&mut self) -> f64 {
        let n = self.advance();
        draw_f64(&self.key, n)
    }

    /// Next uniform draw in `[lo, hi)`.
    pub fn uniform_range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.uniform()
    }

    /// True with probability `p` (clamped to [0, 1]).
    pub fn bernoulli(&mut self, p: f64) -> bool {
        self.uniform() < p.clamp(0.0, 1.0)
    }

    /// Next standard normal draw (Box-Muller with libm; consumes two uniforms).
    pub fn standard_normal(&mut self) -> f64 {
        // 1 - u is in (0, 1], so ln is finite.
        let u1 = 1.0 - self.uniform();
        let u2 = self.uniform();
        libm::sqrt(-2.0 * libm::log(u1)) * libm::cos(2.0 * core::f64::consts::PI * u2)
    }

    /// Next normal draw with mean `mu` and standard deviation `sd`.
    pub fn normal(&mut self, mu: f64, sd: f64) -> f64 {
        mu + sd * self.standard_normal()
    }

    /// Uniform integer in `0..n` (Lemire's multiply-shift; bias below 2^-32 for n < 2^32).
    pub fn below(&mut self, n: u32) -> u32 {
        let x = self.next_u64() >> 32;
        #[allow(clippy::cast_possible_truncation)]
        let r = ((x * u64::from(n)) >> 32) as u32;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Known-answer tests from Random123 (kat_vectors, philox4x32_10).
    #[test]
    fn philox_kat_zero() {
        assert_eq!(
            philox4x32_10([0, 0, 0, 0], [0, 0]),
            [0x6627_e8d5, 0xe169_c58d, 0xbc57_ac4c, 0x9b00_dbd8]
        );
    }

    #[test]
    fn philox_kat_ones() {
        assert_eq!(
            philox4x32_10([u32::MAX; 4], [u32::MAX; 2]),
            [0x408f_276d, 0x41c8_3b0e, 0xa20b_c7c6, 0x6d54_51fd]
        );
    }

    #[test]
    fn philox_kat_pi() {
        assert_eq!(
            philox4x32_10(
                [0x243f_6a88, 0x85a3_08d3, 0x1319_8a2e, 0x0370_7344],
                [0xa409_3822, 0x299f_31d0]
            ),
            [0xd16c_fe09, 0x94fd_cceb, 0x5001_e420, 0x2412_6ea1]
        );
    }

    #[test]
    fn golden_first_draws() {
        // Pins the key-to-counter layout. Changing it changes every golden hash.
        let k = RngKey::new(42, 7, 100, Purpose::BudFate);
        let got: Vec<u64> = (0..3).map(|n| draw_u64(&k, n)).collect();
        assert_eq!(
            got,
            vec![
                0x01cc_3b09_95f2_da36,
                0x1caf_ab42_e567_4522,
                0x7f00_aecc_281d_f079
            ]
        );
    }

    #[test]
    fn uniform_mean_and_range() {
        let mut s = RngKey::new(1, 2, 3, Purpose::Test).stream();
        let n = 100_000;
        let mut sum = 0.0;
        for _ in 0..n {
            let u = s.uniform();
            assert!((0.0..1.0).contains(&u));
            sum += u;
        }
        let mean = sum / f64::from(n);
        assert!((mean - 0.5).abs() < 0.005, "mean {mean}");
    }

    #[test]
    fn normal_moments() {
        let mut s = RngKey::new(9, 9, 9, Purpose::Test).stream();
        let n = 100_000;
        let (mut m1, mut m2) = (0.0, 0.0);
        for _ in 0..n {
            let z = s.standard_normal();
            m1 += z;
            m2 += z * z;
        }
        let mean = m1 / f64::from(n);
        let var = m2 / f64::from(n) - mean * mean;
        assert!(mean.abs() < 0.02, "mean {mean}");
        assert!((var - 1.0).abs() < 0.03, "var {var}");
    }

    #[test]
    fn below_in_range() {
        let mut s = RngKey::new(5, 5, 5, Purpose::Test).stream();
        let mut counts = [0u32; 6];
        for _ in 0..60_000 {
            counts[s.below(6) as usize] += 1;
        }
        for c in counts {
            assert!((9_000..11_000).contains(&c), "{counts:?}");
        }
    }

    fn purpose_strategy() -> impl Strategy<Value = Purpose> {
        prop_oneof![
            Just(Purpose::Weather),
            Just(Purpose::BudFate),
            Just(Purpose::PodSet),
            Just(Purpose::Mutation),
            Just(Purpose::Test),
        ]
    }

    proptest! {
        /// Done-when 1: the same key gives the same stream.
        #[test]
        fn same_key_same_stream(seed: u64, entity: u64, tick: u64, p in purpose_strategy(), len in 1usize..64) {
            let k = RngKey::new(seed, entity, tick, p);
            let mut a = k.stream();
            let mut b = k.stream();
            let xs: Vec<u64> = (0..len).map(|_| a.next_u64()).collect();
            let ys: Vec<u64> = (0..len).map(|_| b.next_u64()).collect();
            prop_assert_eq!(&xs, &ys);
            // Stateless access agrees with the cursor.
            for (n, x) in xs.iter().enumerate() {
                prop_assert_eq!(*x, draw_u64(&k, u32::try_from(n).unwrap()));
            }
        }

        /// Done-when 2: adding an entity does not change another entity's stream.
        /// Draws are a pure function of the key, so interleaving draws for a new entity,
        /// in any order and any amount, leaves the original entity's sequence unchanged.
        #[test]
        fn new_entity_does_not_perturb(seed: u64, a in 0u64..1000, extra in 1000u64..2000, tick: u64, interleave in 0usize..8) {
            let alone: Vec<f64> = {
                let mut s = RngKey::new(seed, a, tick, Purpose::BudFate).stream();
                (0..32).map(|_| s.uniform()).collect()
            };
            let mut s_a = RngKey::new(seed, a, tick, Purpose::BudFate).stream();
            let mut s_new = RngKey::new(seed, extra, tick, Purpose::BudFate).stream();
            let mut with_new = Vec::new();
            for _ in 0..32 {
                for _ in 0..interleave { let _ = s_new.uniform(); }
                with_new.push(s_a.uniform());
            }
            prop_assert_eq!(alone, with_new);
        }

        /// Different keys give different streams (no accidental aliasing of fields).
        #[test]
        fn fields_are_separated(seed: u64, e: u64, t: u64) {
            let base = draw_u64(&RngKey::new(seed, e, t, Purpose::BudFate), 0);
            prop_assert_ne!(base, draw_u64(&RngKey::new(seed ^ 1, e, t, Purpose::BudFate), 0));
            prop_assert_ne!(base, draw_u64(&RngKey::new(seed, e ^ 1, t, Purpose::BudFate), 0));
            prop_assert_ne!(base, draw_u64(&RngKey::new(seed, e, t ^ 1, Purpose::BudFate), 0));
            prop_assert_ne!(base, draw_u64(&RngKey::new(seed, e, t, Purpose::PodSet), 0));
            prop_assert_ne!(base, draw_u64(&RngKey::new(seed, e, t, Purpose::BudFate), 1));
        }
    }
}
