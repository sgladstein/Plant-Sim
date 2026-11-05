//! The seed library: saved seeds with their pedigrees (WP30).

use std::collections::BTreeMap;
use std::path::Path;

use ps_species::seed_colour::{SeedColourTable, SeedLook};
use ps_species::{Genotype, Species};
use ps_world::WorldState;
use serde::{Deserialize, Serialize};

use crate::BreedError;

/// Where a library seed came from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Origin {
    /// The species file's own pure line (its `Default` seed).
    Line,
    /// A pure line drawn from the species population: `Species::sample_genotype` keyed
    /// by the world seed and the seed's library id.
    Founder {
        /// World seed of the draw.
        world_seed: u64,
    },
    /// Picked from a plant in a world: a bench generation or the garden.
    Harvest {
        /// The run's label (a bench plan's label, or the garden save's name).
        run: String,
        /// Mother plant id in that world.
        plant: u32,
        /// The seed's organ id on the mother.
        organ: u32,
        /// Sim day of the harvest.
        day: u32,
        /// Whether the mother had reached maturity (seeds picked earlier are green).
        mature: bool,
    },
}

/// One saved seed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibrarySeed {
    /// Library id, unique and increasing.
    pub id: u64,
    /// Species id; a seed's species is its mother's.
    pub species: String,
    /// Genotype, in the species' locus order.
    pub genotype: Genotype,
    /// Library id of the seed the mother grew from; `None` for a founder, or a mother
    /// that was not sown from the library.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mother: Option<u64>,
    /// Library id of the seed the father grew from (the mother's again when selfed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub father: Option<u64>,
    /// Whether the father was another plant (a hand cross, WP29).
    #[serde(default)]
    pub crossed: bool,
    /// Generations since a founder: 0 for a founder, one more than the mother's seed
    /// otherwise (1 when the mother was not sown from the library).
    pub generation: u32,
    /// Dry mass, g: the mean of its lot, or the species value for a founder.
    pub mass_g: f64,
    /// The look of its coat, which is the mother's tissue and so follows her genotype
    /// (its `flower` field is the mother's flower); `None` without a colour table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coat: Option<SeedLook>,
    /// Where it came from.
    pub origin: Origin,
}

/// Seed-colour tables by species id (data/genetics, decision 0031 item 9).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Looks(pub BTreeMap<String, SeedColourTable>);

impl Looks {
    /// Loads every table in `data_dir/genetics` (file-name order) for the species in
    /// `species` that the table lists; a missing directory gives no tables.
    ///
    /// # Errors
    /// [`BreedError::File`] for an unreadable or malformed table, and
    /// [`BreedError::Species`] for a table that does not fit a species it lists.
    pub fn load(data_dir: &Path, species: &BTreeMap<String, Species>) -> Result<Self, BreedError> {
        let dir = data_dir.join("genetics");
        let mut files: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(r) => r
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "ron"))
                .collect(),
            Err(_) => return Ok(Self::default()),
        };
        files.sort();
        let mut out = BTreeMap::new();
        for path in files {
            let file = |why: String| BreedError::File {
                path: path.display().to_string(),
                why,
            };
            let text = std::fs::read_to_string(&path).map_err(|e| file(e.to_string()))?;
            let table: SeedColourTable = ron::from_str(&text).map_err(|e| file(e.to_string()))?;
            for id in &table.species {
                if let Some(sp) = species.get(id) {
                    table
                        .validate(sp)
                        .map_err(|e| BreedError::Species(format!("{id}: {e}")))?;
                    out.insert(id.clone(), table.clone());
                }
            }
        }
        Ok(Self(out))
    }

    /// The look of a plant of `species` with genotype `g`, if a table covers it.
    #[must_use]
    pub fn of(&self, species_id: &str, species: &Species, g: &Genotype) -> Option<SeedLook> {
        self.0.get(species_id).map(|t| t.evaluate(species, g))
    }
}

/// The player's saved seeds, in id order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SeedLibrary {
    /// The next id to hand out.
    pub next_id: u64,
    /// Seeds, ids increasing.
    pub seeds: Vec<LibrarySeed>,
}

impl SeedLibrary {
    /// An empty library.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a library file and checks its ids increase.
    ///
    /// # Errors
    /// [`BreedError::File`] for malformed text or ids out of order.
    pub fn from_ron(text: &str) -> Result<Self, BreedError> {
        let bad = |why: String| BreedError::File {
            path: "seed library".into(),
            why,
        };
        let lib: Self = ron::from_str(text).map_err(|e| bad(e.to_string()))?;
        let ordered = lib.seeds.windows(2).all(|w| w[0].id < w[1].id);
        if !ordered || lib.seeds.last().is_some_and(|s| s.id >= lib.next_id) {
            return Err(bad("seed ids must increase and stay below next_id".into()));
        }
        Ok(lib)
    }

    /// The library as RON text.
    ///
    /// # Errors
    /// [`BreedError::File`] if serialization fails.
    pub fn to_ron(&self) -> Result<String, BreedError> {
        // One line per seed field: a genotype is one line of allele pairs.
        let pretty = ron::ser::PrettyConfig::default().depth_limit(4);
        ron::ser::to_string_pretty(self, pretty).map_err(|e| BreedError::File {
            path: "seed library".into(),
            why: e.to_string(),
        })
    }

    /// The seed with library id `id`.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&LibrarySeed> {
        self.seeds
            .binary_search_by_key(&id, |s| s.id)
            .ok()
            .map(|i| &self.seeds[i])
    }

    fn push(&mut self, mut seed: LibrarySeed) -> u64 {
        let id = self.next_id;
        seed.id = id;
        self.seeds.push(seed);
        self.next_id += 1;
        id
    }

    /// Adds the species file's own pure line (what a `Default` seed grows from).
    ///
    /// # Errors
    /// [`BreedError::Species`] if the species' phenotype cannot be built.
    pub fn add_line(
        &mut self,
        species_id: &str,
        species: &Species,
        looks: &Looks,
    ) -> Result<u64, BreedError> {
        let g = species.default_genotype();
        let seed = founder(species_id, species, g, Origin::Line, looks)?;
        Ok(self.push(seed))
    }

    /// Adds `n` pure lines drawn from the species population, each keyed by
    /// (`world_seed`, its library id).
    ///
    /// # Errors
    /// [`BreedError::Species`] if a phenotype cannot be built.
    pub fn add_founders(
        &mut self,
        species_id: &str,
        species: &Species,
        n: u32,
        world_seed: u64,
        looks: &Looks,
    ) -> Result<Vec<u64>, BreedError> {
        (0..n)
            .map(|_| {
                let g = species.sample_genotype(world_seed, self.next_id);
                let seed = founder(
                    species_id,
                    species,
                    g,
                    Origin::Founder { world_seed },
                    looks,
                )?;
                Ok(self.push(seed))
            })
            .collect()
    }

    /// Adds every seed of the world's harvested lots from index `from` on (WP25, WP29),
    /// with its parents: a plant sown from the library names the seed it grew from.
    /// Returns the new ids in lot and organ order. A caller that imports a running
    /// garden passes the number of lots it has already imported.
    ///
    /// # Errors
    /// [`BreedError::Species`] if a lot names a plant or species the world lacks.
    pub fn add_lots(
        &mut self,
        state: &WorldState,
        from: usize,
        run: &str,
        looks: &Looks,
    ) -> Result<Vec<u64>, BreedError> {
        let mut ids = Vec::new();
        for lot in state.seed_library.iter().skip(from) {
            let entry = |p: u32| {
                state
                    .plants
                    .get(p as usize)
                    .ok_or_else(|| BreedError::Species(format!("lot names plant {p}")))
            };
            let mother = entry(lot.plant)?;
            let species = state.catalog.species.get(&mother.species).ok_or_else(|| {
                BreedError::Species(format!("{} not in the world", mother.species))
            })?;
            let generation = mother
                .seed_id
                .and_then(|id| self.get(id))
                .map_or(1, |s| s.generation + 1);
            let coat = looks.of(&mother.species, species, &mother.genotype);
            #[allow(clippy::cast_precision_loss)] // a pod holds a handful of seeds
            let mass_g = lot.seed_mass_g / f64::from(lot.seeds.max(1));
            for g in &lot.seed_genotypes {
                let father = entry(g.father)?;
                let seed = LibrarySeed {
                    id: 0,
                    species: mother.species.clone(),
                    genotype: g.genotype.clone(),
                    mother: mother.seed_id,
                    father: father.seed_id,
                    crossed: g.father != g.mother,
                    generation,
                    mass_g,
                    coat: coat.clone(),
                    origin: Origin::Harvest {
                        run: run.to_owned(),
                        plant: lot.plant,
                        organ: g.organ,
                        day: lot.day,
                        mature: lot.mature,
                    },
                };
                ids.push(self.push(seed));
            }
        }
        Ok(ids)
    }
}

fn founder(
    species_id: &str,
    species: &Species,
    genotype: Genotype,
    origin: Origin,
    looks: &Looks,
) -> Result<LibrarySeed, BreedError> {
    let traits = species
        .phenotype(&genotype)
        .traits()
        .map_err(|e| BreedError::Species(format!("{species_id}: {e}")))?;
    Ok(LibrarySeed {
        id: 0,
        species: species_id.to_owned(),
        coat: looks.of(species_id, species, &genotype),
        genotype,
        mother: None,
        father: None,
        crossed: false,
        generation: 0,
        mass_g: traits.seed_mass,
        origin,
    })
}
