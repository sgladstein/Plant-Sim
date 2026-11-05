# 0035: Pollination: selfing by default, hand crosses, seed genotypes at seed set

- **Date:** 2026-11-05
- **Status:** accepted

## Context
WP29 (issue #109) adds pollination to the garden. Design plan §3.11 says flowers
self-pollinate by default, and a hand-pollination action records a cross (the pollen
parent's id) on the flower, whose seeds then carry recombined genotypes (§3.12). WP28
supplied:

- the breeding engine: `Species::meiosis` and `Species::seed_genotype`;
- the genotype stored on each plant (`PlantEntry::genotype`, decision 0031 item 10);
- the meiosis key, set in decision 0031 item 7. A seed's gametes are keyed by:
  - the world seed;
  - the seed organ's entity, `(plant << 32) | organ`;
  - the tick its genotype is fixed;
  - the side: egg or pollen.

  The `SeedInPod` organ is made at seed set.

Common bean is self-pollinating: the anthers shed onto the stigma in the bud, before the
flower opens. Breeders make a cross by emasculating a bud and dusting its stigma with
pollen from another plant's open flower.

## Decision
1. **Selfing is the default.** Every seed of a flower left alone has its mother as
   father. Natural outcrossing is not modelled. It is low in common bean (a few percent
   at most where bees work the flowers, [K, to verify]), and a breeder's line should
   breed true unless crossed.
2. **The hand-pollination action** is `Pollinate { plant, organ, pollen }`.
   - It puts plant `pollen`'s pollen on an open flower `organ` of plant `plant`.
   - The flower is taken as emasculated in time. The sim's open flower stands for the
     bud a breeder would cross.
   - The pollen parent is stored on the flower and passed to its pod when the pod sets
     (`Flower::pollen` and `Pod::pollen`, both omitted from snapshots and the hash
     while `None`).
   - Pod set follows the usual rule (carbon, heat, shade, drought; decision 0018), with
     no penalty for handling. Real hand crosses set fewer pods than selfed flowers, so a
     handling factor is a candidate parameter once there is data.
3. **The action is checked** when it is applied:
   - The organ is a living open flower that is not yet hand-pollinated. A flower takes
     one cross.
   - The pollen plant exists and has an open flower that tick. Pollen is not stored.
   - The two species cross (`Species::can_cross`). Their genomes must have the same
     layout:
     - every locus has the same trait, symbol, tier, chromosome and position;
     - the chromosome lengths are the same.

     The rule exists so an allele means the same thing in both parents. Bush × pole
     bean is allowed; bean × wheat is not. Wheat has no flower organs, so it can neither
     receive nor give pollen.
4. **Seed genotype at seed set.**
   - A seed's genotype is the mother's egg plus the father's pollen. Each parent's
     meiosis uses its own species, through `Species::cross_genotype`. For a selfed seed
     this equals `seed_genotype`.
   - The key follows decision 0031 item 7: (world seed, `(mother << 32) | seed organ`,
     the tick the `SeedInPod` organ was made, side). Seeds are made when the pod passes
     its seed-fill delay, so in the sim that tick is when the genotype is fixed. The sim
     does not resolve the hours between pollination and seed fill.
   - The genotype is computed when it is needed, not stored on the plant.
     - It is a pure function of state that never changes: both parents' genotypes, the
       seed organ, its birth tick, and the cross on its pod.
     - Storing it would add state without adding information, and would change every
       bean run's hash.
     - `World::seed_genotype(plant, organ)` reads it for a seed still in its pod, for the
       inspector and the WP31 lens.
     - A harvest writes it into the seed lot.
5. **Seed lots list their seeds.** `SeedLot::seed_genotypes` holds one `SeedGenotype`
   per seed, in organ order: the seed organ, `mother`, `father` (plant ids in this world)
   and the genotype. This meets the done-when: a cross records both parents in each
   seed. A seed's species is its mother's, the lot's `species`. Pedigrees across worlds
   (the breeding bench) are WP30's. A library seed takes the file its habit matches
   (decision 0038 item 7).
6. **Seed size stays maternal.** A seed fills to its mother's heritable `seed_mass`
   from her carbon. Its own genotype sets seed size only when it grows up and bears
   seeds itself. This settles decision 0031 item 11's open seed-set rule. In legumes,
   seed size depends largely on the mother: the seed coat and pod are her tissue and
   she supplies the assimilate ([K, to verify]).
7. **No model version bump** (agreed with the model-changes thread):
   - Runs without a cross keep their trajectories (`daily.csv`, `organs.csv`) and the
     golden hashes.
   - Runs that harvest seeds change their hash, because their lots now carry the seed
     records.

## Not adopted now
- Natural outcrossing by pollinators, pollen competition and mixed paternity in one pod.
- Xenia (pollen effects on the mother's tissue) and self-incompatibility. Neither applies
  to common bean.
- Sowing a harvested seed's genotype: that is the WP30 seed library and bench.
- A hand-pollination tool in the viewer: a UI follow-up for the panels thread.

## Consequences
- New action `ActionKind::Pollinate`, added as the enum's last variant so older logs keep
  their variant codes.
- New types and fields:
  - `Flower::pollen`, `Pod::pollen`;
  - `ps_arch::SeedOrigin`, `Plant::seed_origin`, `Plant::check_pollinate`,
    `Plant::pollinate`, `Plant::has_open_flower`;
  - `Harvested::origins`;
  - `Species::can_cross`, `Species::cross_genotype`;
  - `SeedLot::seed_genotypes`, `SeedGenotype`, `World::seed_genotype`.
- Done-when: `ps_world` `a_cross_records_both_parents_in_each_seed`.
  - Two sampled bush-bean lines flower together, and every open flower on plant 0 gets
    plant 1's pollen. Every seed of plant 0 names mother 0 and father 1.
  - Each F1 seed pairs the mother's allele with the father's at every locus (920 of
    920).
  - Plant 1's seeds name plant 1 twice.
  - The lot's records equal those read on the plant before the harvest.
  - The run replays from its log to the same hashes.
- `a_selfed_heterozygote_segregates_seed_by_seed`: a selfed heterozygote's 23 seeds are
  all different, and all 32 of its heterozygous loci throw both homozygotes and the
  heterozygote.
- `pollination_is_checked` covers the action's checks. `ps_species`
  `crosses_need_one_genome_layout` covers `can_cross`, and checks that a bush × pole F1
  carries Fin fin and climbs.
