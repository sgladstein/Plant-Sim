# 0038: Seed library and breeding bench

- **Date:** 2026-11-05
- **Status:** accepted

## Context
WP30 (issue #125) is the M3 breeding loop: save seeds, grow lines on a bench, select
parents. Design plan §7 asks for a "seed library and breeding bench: ten headless pots in
parallel, results card with charts, parent selection". The done-when is that a ten-seed
generation runs in under 10 s of wall time. The M3 exit asks that a cross between two
lines show F2 variation on the bench charts, and that selected lines differ measurably
after five generations.

WP29 (decision 0035) made seeds carry genotypes and parents. Those genotypes live only in
one world's `SeedLot`s, and nothing could sow a seed of a given genotype.

## Decision
1. **A new sim crate, `ps-breed`.** It sits above `ps-world`, with no GPU dependency.
   The viewer builds the results card, the charts and the parent-selection UI on it
   (Viewer UI panels thread).
2. **The seed library outlives a world.** `SeedLibrary` is one RON file of
   `LibrarySeed`s with increasing ids. Each seed records:
   - its species (the mother's) and its genotype;
   - the library ids of the seeds its mother and father grew from;
   - whether it was crossed;
   - its generation: 0 for a founder, the mother's plus one otherwise;
   - its mass (the mean of its lot);
   - its coat look;
   - its origin: the species line, a sampled founder, or a harvest (run, plant, organ,
     day, mature).

   Founders are pure lines:
   - `add_line` adds the species file's own line;
   - `add_founders` draws lines from the species population, keyed by (world seed,
     library id).

   `add_lots` imports a world's harvested lots from a given index, so a running garden
   can import new picks without doubling the old ones. The garden's `SeedLot` list stays
   the world's own harvest record; the library is the player's store across worlds.
3. **Sowing a saved seed.** `SeedChoice::Saved { id, genotype }` sows a library seed.
   - The variant is appended, so existing scenarios keep their codes.
   - The plant keeps the id (`PlantEntry::seed_id`, omitted from snapshots and the hash
     while `None`), so its seeds name it as a parent.
   - The genotype is checked against the species: one finite pair per locus.
   - The plant grows from its genotype's phenotype, as a `Sample` seed does. Its seed
     reserve comes from the heritable `seed_mass`, not the saved seed's actual mass.
     Using the actual mass as a maternal effect on the seedling is a candidate for later.
4. **The bench is one world of up to ten pots.**
   - Pot `i` (soil `i`, 1 m apart) holds seed `i`. A pot builds its own light grid, so
     pots never shade each other, and the world steps the plants in parallel.
   - Default conditions are the WP22 trellis pot: 40 cm loam, watered to field capacity
     every morning, garden nitrogen, a 2 m stake 5 cm from the seed, full sun at 45° N,
     sown 15 May. A plan can change them.
   - Flowers self. A plan's `crosses` (mother pot, father pot) put the father's pollen
     on every flower the mother opens while he has an open flower. A pot is a mother in
     at most one cross.
   - The bench stops when every plant is mature, or at `max_days` (120 by default). It
     then picks every pod that has seeds.
   - A generation is fixed by the plan, the library seeds and the model version. Its end
     hash is the same on 1, 2 or 4 threads.
5. **What the bench measures** (`BenchRow`), as a breeder writes it down:
   - days to emergence (VE), first flower (R1) and maturity (R9);
   - the tallest shoot, the largest living leaf area and the largest living structural
     dry mass over the run;
   - pods and seeds picked, seed yield and mean seed mass;
   - the plant's look: its flower, and the coat of the seeds it bears;
   - the father's pot when crossed;
   - the library ids of its seeds.

   These are the trait readings WP31's discovery shows only after measurement.
6. **Seed-coat colour is maternal.** A seed's coat is its mother's tissue, so a library
   seed's `coat` is evaluated on the mother's genotype. An F2 seed wears its F1 mother's
   coat; the F2 colours show on the F3 seeds. Colour tables name the species they cover
   (`SeedColourTable::species`; the bean table lists bush and pole bean), and
   `Looks::load` matches them.
7. **No model version bump.**
   - Default, `Sample` and harvest-free runs are unchanged.
   - `SeedChoice::Saved` and `PlantEntry::seed_id` are new state that only bench and
     library sowings use.
   - The CI goldens stay byte-identical.

## Done-when
- **Wall time: 4.6 to 5.2 s** (`ps-bench --example breed_generation`, release build, 4-core
  container). Ten bush bean lines (the species line and nine founders) grow to maturity
  in 81 days and give 388 seeds. On one thread it takes 6.2 s with the same hash. CI
  prints it in the bench summary as report only, like the other wall times.
- `ps_breed` tests:
  - `a_ten_seed_generation_fills_the_library`: every plant measured, every seed in the
    library with its mother, father, generation and coat, and mean seed mass varying
    across lines;
  - `a_bench_cross_gives_an_f1_and_its_f2_segregates`: bush × pole on the bench gives
    Fin fin F1 seeds naming both parents; the selfed F1 climbs taller than the bush line,
    and its F2 seeds segregate at fin;
  - `generations_repeat_and_garden_seeds_join_the_library`;
  - `plans_and_libraries_are_checked`.

## Not adopted now
- Several seeds per pot, or a bench grid larger than ten pots.
- Seed size as a maternal effect on the seedling (item 3).
- Automatic truncation selection over generations. The player picks parents in the UI.
  A five-generation selection test for the M3 exit can use `run_generation` in a loop.

## Consequences
- New crate `ps-breed`.
- New CLI commands: `ps-cli founders LIBRARY SPECIES [--count N] [--world-seed S]
  [--line]` and `ps-cli breed LIBRARY PLAN`, which writes `bench.csv`.
- New in ps-world: `SeedChoice::Saved`, `PlantEntry::seed_id`, and the exported
  `SCENARIO_SCHEMA`.
- New in ps-species: `SeedColourTable::species`.
- New example `ps-bench --example breed_generation`.
- The model card is docs/models/breeding-bench.md.
