# Seed library and breeding bench

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
The seed library keeps every seed you save. Each one carries its genes, both parents
and its generation, so a line can be traced back to the founders it came from. The
breeding bench grows up to ten of those seeds side by side in identical pots, measures
each plant the way a breeder would, and saves the seeds they give. You pick the parents
of the next generation from those numbers.

## Equations
The bench adds no plant equation. Each plant grows by the same model as in the garden.
What is new is how seeds move between worlds and what is measured (decision 0038).

- **Founders.** A founder is a pure line, homozygous at every locus. The species line is
  `Species::default_genotype`. A sampled founder is `Species::sample_genotype` keyed by
  (world seed, library id); see species-and-genome.md.
- **Species file.** Each saved seed names the species file it grows by (decision 0038
  item 7).
  - The habit class is "climbing" when `habit` is 0.5 or more.
  - The seed keeps its mother's file while its habit class matches that file's own line.
  - Otherwise it takes the first file, in id order, that shares the mother's genome
    layout (`Species::can_cross`) and whose line has the seed's class.
  - So a Fin fin F1 on a bush mother grows as a pole bean, and a fin fin F2 seed on that
    F1 grows as a bush bean.
- **Sowing.** `SeedChoice::Saved { id, genotype }`. The plant's phenotype is computed
  from that genotype, and the plant keeps `id` as `PlantEntry::seed_id`.
- **Seeds.** Each picked seed's genotype, mother and father come from the world's seed
  lots (decision 0035). Its library entry records:
  - `mother` and `father`: the library ids of the seeds those plants grew from (the same
    id twice when selfed);
  - `generation`: the mother seed's generation plus one, or 1 when the mother was not
    sown from the library;
  - `mass_g`: lot seed mass divided by lot seed count, in g;
  - `coat`: the colour table evaluated on the mother's genotype. The seed coat is
    maternal tissue.
- **Crosses.** For a plan cross (m, f), every flower open on m on a day f has an open
  flower takes f's pollen (`ActionKind::Pollinate`). m's earlier and later flowers self.
  A cross between files that do not share a genome layout is refused before the bench
  starts.
- **Stop.** The bench steps whole days until every plant has reached R9 or `max_days`
  is reached. It then picks every pod that has a living seed.
- **Measures** (`BenchRow`), from the daily reports:
  - first day at or past VE, R1 and R9 (days after sowing);
  - the largest `height` × 100 (cm), `leaf_area` × 10⁴ (cm²) and living structural
    `dry_mass` (g) over the run;
  - pods and seeds picked;
  - seed yield (g), and mean seed mass = yield / seeds × 1000 (mg).

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| pots on the bench | 1 to 10 | - | design plan §7 WP30 |
| pot | 40 cm across, 40 cm deep, loam | cm | WP22 trellis pot (decision 0021) |
| watering | to field capacity every morning | - | WP22 trellis pot |
| stake | 200 cm high, 1.5 cm across, 5 cm east of the seed | cm | WP22 trellis pot |
| site and date | 45° N, sown 15 May 06:00, temperate summer preset, full sun, 420 ppm CO2 | - | ~ (the default garden) |
| sowing depth | 3 | cm | scenarios' default |
| `max_days` | 120 | days | ~ (a pole bean matures by about day 100 on the bench) |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | seed library file | - | player's save folder |
| in | bench plan: seeds, crosses, world seed, `max_days`, conditions | - | player, UI |
| in | colour tables | - | data/genetics/*.ron |
| in | species files a seed may take | - | data/species/*.ron |
| out | one `BenchRow` per pot | days, cm, cm², g, mg | results card, `bench.csv` |
| out | new library seeds with parents | - | seed library |

## Encoded or emergent
- **Encoded:** the bench layout and conditions, the stopping rule and the measures.
- **Emergent:** every difference between plants. It comes from their genotypes through
  the plant model, and from each plant's own non-heritable variation (decision 0027).
  Examples:
  - a climbing F1 out-growing its bush mother, whichever way it was crossed;
  - seed mass varying between founder lines;
  - F2 segregation.

## Known limits
- One plant per pot and ten pots at most. A field-scale trial with many plants per line
  is not modelled.
- A sown seed's reserve comes from its genes' seed-mass value, not from the actual mass
  of the saved seed. Small saved seeds do not yet give weaker seedlings.
- The bush and pole bean files differ in more than the habit gene: flowering and
  maturity times, internode lengths and stem build. No genes carry those differences, so
  they switch with habit as one block. A climbing seed gets the pole bean's
  timing in full, and a bush seed gets the bush bean's.
- On a crossed mother, flowers that open while the father has none still self. Both
  pots are sown on the same day, and a bush bean flowers about ten days before a pole
  bean, so a bush mother crossed with a pole father sets mostly selfed seed early on.
- Founder k drawn with world seed S has the same genes as plant k sown with `Sample` in
  a world with seed S, because both draws use the same key.
- Sowing a saved seed does not use it up.
- Plants that never mature (dead, or too slow) keep the bench running to `max_days`.
- Seeds are picked when the bench stops. Pods that shattered or rotted earlier are not
  modelled.

## Validation
- `ps_breed` `a_ten_seed_generation_fills_the_library`. Ten lines, all measured; every
  seed is in the library with its mother, generation and coat; mean seed mass differs by
  more than 10% across lines; the library survives a round trip through RON.
- `ps_breed` `a_bench_cross_gives_an_f1_and_its_f2_segregates`:
  - bush and pole are crossed both ways on one bench;
  - the F1 seeds are Fin fin, name both parents and grow by the pole file;
  - the reciprocal F1s differ in height by less than 15%;
  - both F1s reach at least 80% of the pole line's height and more than twice the bush
    line's;
  - F2 seeds segregate at fin, and each takes the file that matches its habit.

  By hand (release, world seed 11), the reciprocal F1s reach 192.9 and 193.6 cm, flower
  on day 52 and mature on day 102. Their 72 F2 seeds are 53 pole and 19 bush.
- `ps_breed` `generations_repeat_and_garden_seeds_join_the_library`: the same plan gives
  the same hash, and garden picks import with their mother's id.
- `ps_breed` `plans_and_libraries_are_checked`.
- `ps-bench --example breed_generation`: the WP30 done-when wall time. A ten-seed
  generation takes 4.6 to 5.2 s in release on 4 cores (budget 10 s), with the same hash
  on 1 to 4 threads.

## References
- Design plan §2.5, §2.6 and §7 (WP30); decisions 0021, 0031, 0035 and 0038.
