//! Meiosis, mutation and fertilization (design plan §3.12, decision 0023, WP28).
//!
//! A gamete takes, at each locus, the maternal or the paternal allele of its parent.
//! Unlinked loci choose independently. Loci on one chromosome follow a Haldane
//! crossover process: crossovers fall as a Poisson process along the linkage group,
//! one per Morgan (100 cM) on average and without interference, so two loci `d` cM
//! apart recombine with probability `r = (1 − e^(−2d/100)) / 2`. Mutation then acts
//! on the gamete in two tiers: anonymous QTL at `mutation_rate` by a Gaussian step of
//! `mutation_effect_sd` allele SDs, named Mendelian loci at `named_mutation_rate` to
//! another allele state.
//!
//! Every draw is keyed. A gamete is identified by the seed it will make (its ovule
//! entity), the tick of fertilization and its side (egg or pollen), so the sibling
//! seeds of one pod, and the two gametes of one selfed seed, are independent. Within
//! the `Recombination` key each chromosome owns a fixed block of draws, and within the
//! `Mutation` key each locus owns a fixed slot, so adding a locus never shifts the
//! draws of another.
//!
//! Nothing in the sim calls this yet: pollination and seed set (WP29) will. Model card:
//! docs/models/species-and-genome.md.

use ps_rng::{Purpose, RngKey};
use serde::{Deserialize, Serialize};

use crate::{Genotype, LocusTier, Species};

/// Chromosomes in the haploid set of common bean (2n = 22).
pub const CHROMOSOMES: u8 = 11;

/// Genetic length of a chromosome when the species file gives none, cM (decision
/// 0023: about 100 cM each; BAT93 × Jalo is 1,097.5 cM over 11, Blair 2018).
pub const DEFAULT_CHROMOSOME_LENGTH_CM: f64 = 100.0;

/// Mutation rate of a named Mendelian locus when the species file gives none, per
/// locus per gamete (decision 0023: visible-mutation rates in plants are 1e-5 to 1e-6).
pub const DEFAULT_NAMED_MUTATION_RATE: f64 = 1e-5;

/// SD of a QTL mutation step when the species file gives none, in allele SDs
/// (decision 0023: 0.1 to 0.2).
pub const DEFAULT_MUTATION_EFFECT_SD: f64 = 0.15;

/// Draws reserved for each block of a meiosis key.
const BLOCK: u32 = 1 << 20;
/// Blocks per gamete side: block 0 for the unlinked loci, block `c` for chromosome `c`.
const BLOCKS_PER_SIDE: u32 = 16;
/// Draws reserved per locus in the mutation key.
const MUTATION_SLOT: u32 = 4;
/// Upper bound on crossovers per chromosome per meiosis (mean is length / 100 cM).
const MAX_CROSSOVERS: usize = 64;

/// Which gamete of a seed is being made.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GameteSide {
    /// The egg, from the seed's mother (the plant that bears it).
    Egg,
    /// The pollen grain, from the pollen parent (the mother itself when selfed).
    Pollen,
}

impl GameteSide {
    const fn index(self) -> u32 {
        match self {
            GameteSide::Egg => 0,
            GameteSide::Pollen => 1,
        }
    }
}

/// The key of one meiosis.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MeiosisKey {
    /// World seed.
    pub world_seed: u64,
    /// The seed this gamete goes into: its ovule's organ entity (plant id packed with
    /// organ id), unique within the world.
    pub entity: u64,
    /// Tick of fertilization.
    pub tick: u64,
    /// Egg or pollen.
    pub side: GameteSide,
}

impl MeiosisKey {
    fn key(self, purpose: Purpose) -> RngKey {
        RngKey::new(self.world_seed, self.entity, self.tick, purpose)
    }
}

/// A haploid set of allele values, one per locus in genome order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gamete {
    /// Allele values.
    pub alleles: Vec<f64>,
}

/// Haldane's map function: the recombination fraction between two loci `d_cm` apart
/// on one chromosome.
#[must_use]
pub fn haldane_r(d_cm: f64) -> f64 {
    0.5 * (1.0 - libm::exp(-2.0 * d_cm.max(0.0) / 100.0))
}

/// The allele state code of a named locus's allele value: the value rounded and
/// clamped to `0..states`.
#[must_use]
pub fn allele_state(value: f64, states: u8) -> u8 {
    let top = f64::from(states.saturating_sub(1));
    let v = libm::round(value).clamp(0.0, top);
    // v is a whole number in [0, 255].
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let state = v as u8;
    state
}

impl Genotype {
    /// A fully homozygous genotype (an inbred line) with these allele values.
    #[must_use]
    pub fn homozygous(values: &[f64]) -> Self {
        Self {
            alleles: values.iter().map(|&v| (v, v)).collect(),
        }
    }

    /// The seed of an egg and a pollen grain: `(maternal, paternal)` per locus.
    #[must_use]
    pub fn fertilize(egg: &Gamete, pollen: &Gamete) -> Self {
        debug_assert_eq!(egg.alleles.len(), pollen.alleles.len());
        Self {
            alleles: egg
                .alleles
                .iter()
                .zip(&pollen.alleles)
                .map(|(&a, &b)| (a, b))
                .collect(),
        }
    }
}

impl Species {
    /// One meiosis of `parent`: recombination, then mutation (decision 0023).
    #[must_use]
    pub fn meiosis(&self, parent: &Genotype, key: MeiosisKey) -> Gamete {
        let paternal = self.recombine(parent, key);
        let mut alleles: Vec<f64> = parent
            .alleles
            .iter()
            .zip(&paternal)
            .map(|(&(a, b), &p)| if p { b } else { a })
            .collect();
        self.mutate(&mut alleles, key);
        Gamete { alleles }
    }

    /// A seed of `mother` fertilized by `father` (pass the mother twice for a selfed
    /// seed). `entity` is the seed's ovule entity and `tick` the tick of fertilization.
    #[must_use]
    pub fn seed_genotype(
        &self,
        mother: &Genotype,
        father: &Genotype,
        world_seed: u64,
        entity: u64,
        tick: u64,
    ) -> Genotype {
        let key = |side| MeiosisKey {
            world_seed,
            entity,
            tick,
            side,
        };
        let egg = self.meiosis(mother, key(GameteSide::Egg));
        let pollen = self.meiosis(father, key(GameteSide::Pollen));
        Genotype::fertilize(&egg, &pollen)
    }

    /// For each locus, whether the gamete takes the paternal allele.
    fn recombine(&self, parent: &Genotype, key: MeiosisKey) -> Vec<bool> {
        let loci = &self.genome.loci;
        let n = loci.len().min(parent.alleles.len());
        let mut paternal = vec![false; n];
        let rng = key.key(Purpose::Recombination);
        let base = key.side.index() * BLOCKS_PER_SIDE;

        // Unlinked loci: one fair coin each, at a fixed index of block 0.
        for (i, l) in loci.iter().take(n).enumerate() {
            if l.chromosome.is_none() {
                let index = u32::try_from(i).unwrap_or(u32::MAX).min(BLOCK - 1);
                paternal[i] = rng.stream_at(base * BLOCK + index).uniform() < 0.5;
            }
        }

        // Linked loci: a fair coin for the starting strand, then Poisson crossovers.
        let mut crossovers: Vec<f64> = Vec::with_capacity(8);
        for c in 1..=CHROMOSOMES {
            let on_c = |l: &&crate::Locus| l.chromosome == Some(c);
            if !loci.iter().take(n).any(|l| on_c(&l)) {
                continue;
            }
            let length = self.genome.chromosome_length_cm(c);
            let mut s = rng.stream_at((base + u32::from(c)) * BLOCK);
            let start_paternal = s.uniform() < 0.5;
            crossovers.clear();
            let mut x = 0.0;
            while crossovers.len() < MAX_CROSSOVERS {
                // Exponential gap with mean 100 cM; 1 - u is in (0, 1].
                x += -100.0 * libm::log(1.0 - s.uniform());
                if x >= length {
                    break;
                }
                crossovers.push(x);
            }
            for (i, l) in loci.iter().take(n).enumerate() {
                if l.chromosome == Some(c) {
                    let at = l.position_cm.unwrap_or(0.0);
                    let before = crossovers.iter().filter(|&&p| p < at).count();
                    paternal[i] = start_paternal ^ (before % 2 == 1);
                }
            }
        }
        paternal
    }

    /// Two-tier mutation of a gamete in place.
    fn mutate(&self, alleles: &mut [f64], key: MeiosisKey) {
        let g = &self.genome;
        let rng = key.key(Purpose::Mutation);
        let base = key.side.index() * (BLOCK * BLOCKS_PER_SIDE);
        for (i, (l, v)) in g.loci.iter().zip(alleles.iter_mut()).enumerate() {
            let slot = u32::try_from(i)
                .unwrap_or(u32::MAX)
                .saturating_mul(MUTATION_SLOT)
                .min(BLOCK * BLOCKS_PER_SIDE - MUTATION_SLOT);
            let mut s = rng.stream_at(base + slot);
            let u = s.uniform();
            match l.tier() {
                LocusTier::Qtl => {
                    if u < g.mutation_rate.value {
                        *v += s.normal(0.0, g.mutation_effect_sd() * l.allele_sd);
                    }
                }
                LocusTier::Named { states } => {
                    if u < g.named_mutation_rate() && states >= 2 {
                        let old = allele_state(*v, states);
                        let step = s.below(u32::from(states - 1));
                        // step < states - 1, so the sum is below 2 * states.
                        let new = (u32::from(old) + 1 + step) % u32::from(states);
                        *v = f64::from(new);
                    }
                }
            }
        }
    }
}
