//! Seed library and breeding bench (design plan §2.6 and §7 WP30; decision 0038).
//!
//! The **seed library** ([`SeedLibrary`]) is the player's store of saved seeds, kept
//! across worlds as one RON file. Each seed carries its species, its genotype, the
//! library ids of the seeds its mother and father grew from, its generation, where it
//! was picked and the look of its coat. Founders are pure lines: the species file's own
//! line or lines drawn from the species population. Seeds harvested in any world join
//! the library through [`SeedLibrary::add_lots`]. A plant sown from the library
//! (`SeedChoice::Saved`) keeps the seed's id, so its own seeds can name it as a parent.
//!
//! The **breeding bench** ([`BenchPlan`], [`run_generation`]) grows up to ten library
//! seeds in ten pots of one headless world. The world steps its plants in parallel; a
//! pot builds its own light grid, so the pots never shade each other. Flowers self
//! unless the plan crosses two pots. The bench stops once every plant is mature (or at
//! the plan's last day), picks every pod, measures each plant ([`BenchRow`]) and adds the
//! seeds to the library with their parents. A generation is deterministic: the plan, the
//! library seeds and the model version fix it, whatever the thread count.

mod bench;
mod library;

pub use bench::{
    BenchConditions, BenchCross, BenchPlan, BenchRow, Generation, MAX_POTS, grow, measure,
    run_generation,
};
pub use library::{LibrarySeed, Looks, Origin, SeedLibrary};

use ps_world::WorldError;

/// Errors from the library and the bench.
#[derive(Debug, thiserror::Error)]
pub enum BreedError {
    /// The world rejected the bench scenario or a step.
    #[error(transparent)]
    World(#[from] WorldError),
    /// A species or a colour table is missing or unusable.
    #[error("species: {0}")]
    Species(String),
    /// The plan is unusable.
    #[error("bench plan: {0}")]
    Plan(String),
    /// A library or table file could not be read or parsed.
    #[error("{path}: {why}")]
    File {
        /// The file.
        path: String,
        /// The error.
        why: String,
    },
}
