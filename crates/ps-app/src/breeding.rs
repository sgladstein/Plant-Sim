//! The breeding bench's data source (WP30): owns the seed library file, answers the
//! window's requests and grows generations on a worker thread with `ps_breed`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use anyhow::{Context, Result};
use bevy::prelude::{Res, ResMut, Resource};
use ps_breed::{BenchCross, BenchPlan, Generation, LibrarySeed, Looks, Origin, SeedLibrary};
use ps_species::Species;
use ps_species::seed_colour::SeedLook;
use ps_ui::{BenchResults, Breeding, Coat, LibraryRow, Request, ResultRow};

use crate::sim_source::SimSource;

/// Founders are keyed by (this, library id), so each draw differs from the last.
const FOUNDER_WORLD_SEED: u64 = 1;

/// A finished generation from the worker: the library with its seeds, and the rows.
type Grown = Result<(SeedLibrary, Generation, String), String>;

/// The library and the bench job, beside the window's [`Breeding`].
#[derive(Resource)]
pub struct Bench {
    data: PathBuf,
    path: PathBuf,
    library: SeedLibrary,
    /// False when the file on disk could not be read: changes are not written over it.
    writable: bool,
    species: BTreeMap<String, Species>,
    looks: Looks,
    /// Garden lots already imported, and the tick of that import (a lower tick is a
    /// new run, whose lots start again from 0).
    imported: (usize, u64),
    /// The growing generation (a `Mutex` only to make the resource `Sync`).
    job: Option<Mutex<Receiver<Grown>>>,
}

impl Bench {
    /// Loads the species files, colour tables and the library at
    /// `<captures>/seed_library.ron` (an empty one when there is no file).
    ///
    /// # Errors
    /// A species file or colour table that cannot be read.
    pub fn load(data: &Path, captures: &Path) -> Result<(Self, Breeding)> {
        let species = ps_breed::load_species(data)?;
        let looks = Looks::load(data, &species)?;
        let path = captures.join("seed_library.ron");
        let mut ui = Breeding {
            species: species.keys().map(|k| k.replace('_', " ")).collect(),
            library_path: path.display().to_string(),
            ..Breeding::default()
        };
        ui.founder_species = ui
            .species
            .iter()
            .position(|s| s == "pole bean")
            .unwrap_or_default();
        let (library, writable) = match std::fs::read_to_string(&path) {
            Ok(text) => match SeedLibrary::from_ron(&text) {
                Ok(lib) => (lib, true),
                Err(e) => {
                    ui.status = format!(
                        "Could not read {}: {e}. Changes are not saved.",
                        path.display()
                    );
                    (SeedLibrary::new(), false)
                }
            },
            Err(_) => (SeedLibrary::new(), true),
        };
        let bench = Self {
            data: data.to_owned(),
            path,
            library,
            writable,
            species,
            looks,
            imported: (0, 0),
            job: None,
        };
        ui.library = library_rows(&bench.library);
        Ok((bench, ui))
    }

    fn save(&self) -> Result<()> {
        if !self.writable {
            return Ok(());
        }
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&self.path, self.library.to_ron()?)
            .with_context(|| format!("writing {}", self.path.display()))
    }

    fn handle(
        &mut self,
        request: Request,
        ui: &mut Breeding,
        src: Option<&SimSource>,
    ) -> Result<String> {
        let id = self
            .species
            .keys()
            .nth(ui.founder_species)
            .cloned()
            .unwrap_or_default();
        match request {
            Request::Founders | Request::Line => {
                let sp = self
                    .species
                    .get(&id)
                    .with_context(|| format!("no species {id}"))?;
                let added = if request == Request::Line {
                    vec![self.library.add_line(&id, sp, &self.looks)?]
                } else {
                    self.library.add_founders(
                        &id,
                        sp,
                        ui.founder_count,
                        FOUNDER_WORLD_SEED,
                        &self.looks,
                    )?
                };
                self.save()?;
                Ok(format!(
                    "Added {} {} seed(s): {}",
                    added.len(),
                    id.replace('_', " "),
                    ids(&added)
                ))
            }
            Request::ImportGarden => {
                let src = src.context("the garden is not running the sim")?;
                let saved = src.save()?;
                let world = ps_world::World::from_snapshot_ron(&saved.ron)?;
                let (mut from, tick) = self.imported;
                if saved.tick < tick || world.state().seed_library.len() < from {
                    from = 0;
                }
                let added = self.library.add_lots(
                    world.state(),
                    from,
                    "Garden",
                    &self.species,
                    &self.looks,
                )?;
                self.imported = (world.state().seed_library.len(), saved.tick);
                self.save()?;
                Ok(if added.is_empty() {
                    "No new harvest in the garden. Pick ripe pods with the Harvest tool first."
                        .to_owned()
                } else {
                    format!("Imported {} garden seed(s): {}", added.len(), ids(&added))
                })
            }
            Request::Grow => {
                let label = format!("Bench {}", next_bench(&self.library));
                let plan = BenchPlan {
                    label: label.clone(),
                    seeds: ui.plan.clone(),
                    crosses: ui
                        .crosses()
                        .into_iter()
                        .map(|(mother, father)| BenchCross { mother, father })
                        .collect(),
                    world_seed: next_bench(&self.library),
                    max_days: ui.max_days,
                    conditions: ps_breed::BenchConditions::default(),
                };
                plan.check(&self.library)?;
                let scenario = plan.scenario(&self.library)?;
                let catalog = ps_world::Catalog::load(&self.data, &scenario)?;
                let looks = self.looks.clone();
                let kin = self.species.clone();
                let mut lib = self.library.clone();
                let (tx, rx) = channel();
                std::thread::Builder::new()
                    .name("breeding-bench".into())
                    .spawn(move || {
                        let out = ps_breed::run_generation(&mut lib, &plan, &catalog, &kin, &looks)
                            .map(|g| (lib, g, plan.label))
                            .map_err(|e| e.to_string());
                        let _ = tx.send(out);
                    })?;
                self.job = Some(Mutex::new(rx));
                ui.running = true;
                Ok(format!("Growing {label}…"))
            }
        }
    }

    fn poll(&mut self, ui: &mut Breeding) {
        let Some(rx) = self.job.as_mut().and_then(|m| m.get_mut().ok()) else {
            return;
        };
        let got = match rx.try_recv() {
            Ok(got) => got,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("the bench thread stopped".to_owned()),
        };
        self.job = None;
        ui.running = false;
        match got {
            Ok((lib, g, label)) => {
                self.library = lib;
                ui.status = match self.save() {
                    Ok(()) => format!("{label}: {} days, {} seeds saved", g.days, g.saved.len()),
                    Err(e) => format!("{label} grew, but the library was not saved: {e:#}"),
                };
                ui.results = Some(results(&label, &g));
                ui.plan.clear();
                ui.fathers.clear();
                ui.library = library_rows(&self.library);
            }
            Err(e) => ui.status = format!("The bench failed: {e}"),
        }
    }
}

/// Answers the window's request and collects a finished generation.
#[allow(clippy::needless_pass_by_value)]
pub fn breeding(mut ui: ResMut<Breeding>, mut bench: ResMut<Bench>, src: Option<Res<SimSource>>) {
    bench.poll(&mut ui);
    let Some(request) = ui.request.take() else {
        return;
    };
    ui.status = match bench.handle(request, &mut ui, src.as_deref()) {
        Ok(s) => s,
        Err(e) => format!("{e:#}"),
    };
    ui.library = library_rows(&bench.library);
}

/// One more than the highest "Bench N" the library has seeds from.
fn next_bench(lib: &SeedLibrary) -> u64 {
    lib.seeds
        .iter()
        .filter_map(|s| match &s.origin {
            Origin::Harvest { run, .. } => run.strip_prefix("Bench ")?.parse::<u64>().ok(),
            _ => None,
        })
        .max()
        .unwrap_or(0)
        + 1
}

fn ids(ids: &[u64]) -> String {
    match ids {
        [] => String::new(),
        [one] => format!("#{one}"),
        [first, .., last] => format!("#{first} to #{last}"),
    }
}

/// "#3 selfed", "#3 × #7", or what a seed without parents is.
fn parents(s: &LibrarySeed) -> String {
    match (s.mother, s.father) {
        (Some(m), Some(f)) if s.crossed => format!("#{m} × #{f}"),
        (Some(m), _) if !s.crossed => format!("#{m} selfed"),
        (None, Some(f)) => format!("garden plant × #{f}"),
        (Some(m), None) => format!("#{m} × garden plant"),
        _ => match s.origin {
            Origin::Line => "species line".to_owned(),
            Origin::Founder { .. } => "founder".to_owned(),
            Origin::Harvest { .. } => "garden plant".to_owned(),
        },
    }
}

fn origin(s: &LibrarySeed) -> String {
    match &s.origin {
        Origin::Line => "line".to_owned(),
        Origin::Founder { world_seed } => format!("founder, draw {world_seed}"),
        Origin::Harvest {
            run, day, mature, ..
        } => {
            let green = if *mature { "" } else { " (green)" };
            format!("{run}, day {day}{green}")
        }
    }
}

/// The library as families, oldest first: seeds of one mother plant in one run with the
/// same parents share a row (a founder is a row of its own).
pub fn library_rows(lib: &SeedLibrary) -> Vec<LibraryRow> {
    let mut rows: Vec<(String, LibraryRow, f64)> = Vec::new();
    for s in &lib.seeds {
        let parents = parents(s);
        let origin = origin(s);
        let key = match &s.origin {
            Origin::Harvest { .. } => format!("{}|{}|{parents}|{origin}", s.species, s.generation),
            _ => format!("#{}", s.id),
        };
        if let Some((_, row, mass)) = rows.iter_mut().find(|(k, ..)| *k == key) {
            row.ids.push(s.id);
            *mass += s.mass_g;
            continue;
        }
        let row = LibraryRow {
            ids: vec![s.id],
            species: s.species.replace('_', " "),
            generation: s.generation,
            parents,
            mass_mg: 0.0,
            coat: s.coat.as_ref().map(coat),
            origin,
        };
        rows.push((key, row, s.mass_g));
    }
    rows.into_iter()
        .map(|(_, mut row, mass)| {
            #[allow(clippy::cast_precision_loss)] // a family is a few hundred seeds
            let n = row.ids.len() as f64;
            row.mass_mg = mass / n * 1000.0;
            row
        })
        .collect()
}

fn results(label: &str, g: &Generation) -> BenchResults {
    BenchResults {
        label: label.to_owned(),
        days: g.days,
        rows: g
            .rows
            .iter()
            .map(|r| ResultRow {
                pot: r.pot,
                seed: r.seed,
                crossed_with: r.crossed_with,
                stages: [r.emergence_day, r.flowering_day, r.maturity_day],
                height_cm: r.height_cm,
                leaf_area_cm2: r.leaf_area_cm2,
                dry_mass_g: r.dry_mass_g,
                pods: r.pods,
                seeds: r.seeds,
                seed_yield_g: r.seed_yield_g,
                mean_seed_mg: r.mean_seed_mg,
                coat: r.look.as_ref().map(coat),
                saved: r.saved.clone(),
            })
            .collect(),
    }
}

/// `#rrggbb` as sRGB bytes; grey when it is not one.
fn hex(s: &str) -> [u8; 3] {
    let s = s.trim_start_matches('#');
    let byte = |i: usize| {
        s.get(i..i + 2)
            .and_then(|b| u8::from_str_radix(b, 16).ok())
            .unwrap_or(128)
    };
    [byte(0), byte(2), byte(4)]
}

#[allow(clippy::cast_possible_truncation)] // a share from 0 to 1
fn coat(l: &SeedLook) -> Coat {
    Coat {
        color: hex(&l.seed_coat),
        ground: hex(&l.ground),
        coverage: l.coverage as f32,
        pattern: l.pattern.clone(),
        flower: hex(&l.flower),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> PathBuf {
        crate::sim_source::find_data_dir(None).expect("data dir")
    }

    #[test]
    fn hex_colors_parse_and_bad_ones_are_grey() {
        assert_eq!(hex("#ff8000"), [255, 128, 0]);
        assert_eq!(hex("nope"), [128, 128, 128]);
    }

    #[test]
    fn founders_join_the_library_file_and_a_bench_grows() {
        let dir = std::env::temp_dir().join(format!("ps-app-breeding-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (mut bench, mut ui) = Bench::load(&data(), &dir).expect("load");
        assert!(ui.species.iter().any(|s| s == "bush bean"));
        assert_eq!(ui.species[ui.founder_species], "pole bean");
        ui.founder_count = 2;
        ui.request = Some(Request::Founders);
        let r = ui.request.take().expect("request");
        let msg = bench.handle(r, &mut ui, None).expect("founders");
        assert!(msg.starts_with("Added 2 pole bean"), "{msg}");
        assert!(dir.join("seed_library.ron").exists());
        let rows = library_rows(&bench.library);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].parents, "founder");
        assert!(rows[0].coat.is_some(), "beans have a colour table");

        // Mother pot 0 takes pot 1's pollen; a short bench keeps the test quick.
        assert!(ui.sow(rows[0].ids[0]) && ui.sow(rows[1].ids[0]));
        ui.fathers[0] = Some(1);
        ui.max_days = 30;
        bench
            .handle(Request::Grow, &mut ui, None)
            .expect("grow starts");
        assert!(ui.running);
        while ui.running {
            std::thread::sleep(std::time::Duration::from_millis(50));
            bench.poll(&mut ui);
        }
        assert!(!ui.running, "{}", ui.status);
        let res = ui.results.as_ref().expect("results");
        assert_eq!(res.label, "Bench 1");
        assert_eq!(res.rows.len(), 2);
        assert_eq!(res.rows[0].crossed_with, Some(1));
        assert!(ui.plan.is_empty());
        // The picked seeds join as families: one row per mother and parentage.
        let seeds: usize = ui.library.iter().map(|r| r.ids.len()).sum();
        assert_eq!(seeds, bench.library.seeds.len());
        if seeds > 2 {
            assert!(ui.library.len() < seeds, "{} rows", ui.library.len());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bench_labels_count_up_from_the_library() {
        let mut lib = SeedLibrary::new();
        assert_eq!(next_bench(&lib), 1);
        lib.seeds.push(LibrarySeed {
            id: 0,
            species: "bean".into(),
            genotype: ps_species::Genotype {
                alleles: Vec::new(),
            },
            mother: Some(3),
            father: Some(3),
            crossed: false,
            generation: 2,
            mass_g: 0.3,
            coat: None,
            origin: Origin::Harvest {
                run: "Bench 4".into(),
                plant: 0,
                organ: 9,
                day: 80,
                mature: false,
            },
        });
        assert_eq!(next_bench(&lib), 5);
        assert_eq!(parents(&lib.seeds[0]), "#3 selfed");
        assert_eq!(origin(&lib.seeds[0]), "Bench 4, day 80 (green)");
    }
}
