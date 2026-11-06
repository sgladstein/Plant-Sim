# 0040: Starch storage, root exudation and seed-driven nitrogen (R1b)

- **Date:** 2026-11-06
- **Status:** accepted (owner approved the realism plan's R1 to R3 on 2026-11-05; R1b is
  `docs/research/realism-plan/r1-carbon-sinks.md` §4)

## Context
Before R1b a plant had one carbon pool per organ, its reserve, which moves in the
phloem and sets growth. Sugar that no sink could use stayed in the leaves and shut down
photosynthesis through the sink feedback (decision 0013): in `single_pot` about 60 to
70% of the potential gross assimilation was lost over seed fill. Stems never stored
starch, so they could not give it back to the seeds; roots leaked no carbon to the soil;
and nitrogen was mined from leaves only when the soil fell short, so a fertilized pot
kept every leaf green until maturity.

The R1 spec (§4) asked for a starch store in stems and main roots, root exudation as a
share of root maintenance, and seeds served first by mining from seed fill on. The
research thread's R1 numbers set `exudation_fraction` to 0.3 to 0.5 and the wheat
`storage_capacity` to 0.25 to 0.30.

## Decision
1. **A starch store** (`OrganArena` column `storage`, g C) in stem segments and in the
   segments of main (order 0) roots. It is carbon held apart from the reserve: it does
   not move in the phloem and does not count in the reserve concentration
   `C = R / W_p`. It is part of the plant's carbon stock, and of its dry mass as
   `S / c_f`. Lateral and fine roots store nothing.
2. **Release first, refill last**, inside hourly step 12 (no step reorder). Before an
   organ grows: below `C_off` the store gives back `min(k_r S, (C_off − C) W_p)`. After
   its primary and secondary growth: once the organ has finished expanding and
   `C > C_on`, the store is a sink for `k_s (S_max − S)` with
   `S_max = s_max W c_f`, met at priority 0.2 (§3.9's reserve-refill value) of the
   reserve above the growth floor. Two changes from the spec, both found in the re-tune:
   - **Refill only after expansion.** Starch is laid down in finished tissue; letting a
     growing internode fill its store took sugar from its own elongation (wheat stood
     about 5% shorter on four-seed means).
   - **`C_on` 0.15, not 0.12.** A growing organ's hourly growth is a share of its
     reserve above the floor, so any store that fills below the point where growth is
     saturated slows growth. At 0.12 the pot bean set 9.8 pods against 12.5 at 0.15 and
     wheat stood 3% shorter. 0.15 sits above the leaves' feedback onset (0.10): a stem
     stores sugar the leaves are already holding back.
3. **Seed fill empties the store.** From seed fill (stage R5) the store stops refilling
   and gives back `k_f S` an hour whatever the reserve, at `k_f = 0.08 d⁻¹`. This is
   not in the spec. With the concentration rule alone the stores never released in
   seed fill: the pot plant's reserve stays near 0.2 g C g⁻¹, far above `C_off`, because
   its seeds cannot use all its sugar. Stem reserves fall in seed fill whatever the
   source in the data the plan cites: bean stem starch −42% in 7 days of pod fill
   (Waters et al. 1980), stem sugars from about 7% to 1.6% of dry mass (Peña-Valdivia
   1991), wheat stem water-soluble carbohydrate from its peak 10 to 14 days after
   anthesis to under 5% at maturity (Davidson and Chevalier 1992). New parameter
   `storage_fill_release_rate`.
4. **Root storage has its own capacity.** `root_storage_capacity` 0.25 for both beans
   (the spec's "0.25 bean stem and root"), 0 for wheat: grass roots have no secondary
   tissue to hold starch (with a root store the spaced wheat plants also made 5% less
   grain, on one seed).
5. **At death the store joins the reserve** (`OrganArena::kill`), so a shed or starved
   organ books it like reserve: litter, or carried to the parent where the shedding
   rule moves reserve.
6. **Root exudation.** After paying maintenance `Rm` in step 8, each root organ exudes
   `e · Rm` from what positive reserve it has left (never into debt), booked as the
   ledger's new `exudation` line. `e = 0.4` for all three species. Nothing exudes after
   maturity.
7. **Seeds mine first.** From seed fill on (`seed_fill_mining_first`, schema 14), the
   filling seeds' nitrogen demand (their deficits plus their share of yesterday's
   growth, tracked hourly as `seed_growth`) is served first by mining, up to the day's
   mining capacity, at the existing rate and light weighting (nitrogen card step 15).
   The soil request is cut by that amount; the pool and the soil cover the rest, and
   mining covers any shortfall from the capacity left. This reorders sub-steps inside
   daily step 0, not the §3.2 step list.
8. **Outputs.** Daily rows: `stem_mass_g`, `root_mass_g`, `dry_mass_g` and
   `total_dry_mass_g` include starch; new `storage_mass_g`, `stem_structural_mass_g`,
   `root_structural_mass_g` (the old meanings) and `exudation_gc`. Organ rows gain
   `storage_gc`. New Info targets `stem_decline_fill` (single_pot, stem dry mass at
   maturity against its peak, −10 to −35%) and `feedback_loss_fill` (mean over seed
   fill, at most 0.35), with derived variables `stem_change_from_peak` and
   `feedback_loss_seed_fill`.

Species schema 14 adds the `storage` group (`storage_capacity`,
`root_storage_capacity`, `storage_on`, `storage_off`, `storage_refill_rate`,
`storage_release_rate`, `storage_fill_release_rate`, `exudation_fraction`) and
`nitrogen.seed_fill_mining_first`.

### Tried and not kept
- **The concentration rule alone** (spec §4.1): the stores filled to `S_max` and held
  it through seed fill in every bean scenario; stem mass rose to maturity.
- **A higher release threshold** (`C_off` 0.10, `C_on` 0.15) without the seed-fill
  release: still no release in the pot (stem change −0.4%).
- **Leaves drop at 30% of their mobile nitrogen** (`leaf_drop_mobile` 0.1 → 0.3), so
  that a fertilized pot sheds lower leaves in seed fill: three leaves dropped, but seed
  mass fell 12% and `harvest_index_tops` slipped below its band.

## Consequences
- Model 0.21.0, species schema 14; every golden hash moves.
- Validation on four-seed means (94 checks, two new): **70 pass, every M0 gate passes**
  (0.20.0: 68 of 92). `harvest_index_tops` now passes (0.395 → 0.410), and so do
  `pod_set_thermal_time_b10` and `density_leaf_area_per_plant`; `wheat_height_spaced`
  slips just out (0.704 → 0.687, band from 0.70) and `stem_share_peak` stays just out
  (0.306 → 0.349, band from 0.35). The pot bean carries 12.5 pods (10.8) and 10.5 g of
  seed (9.0); total dry mass 26.7 g (24.6). Shade gates: internodes +25% (−13% to +42% over
  the seeds), total mass −47%, pods −47%; heat_seeds' weakest seed is now −15% (−1.1%
  at 0.20.0).
- **Three R1b goals are not met.**
  - `stem_decline_fill` −0.7% against −10 to −35%. The stores empty in seed fill
    (0.51 to 0.07 g of starch in the default-seed pot's stems), but stem wood keeps
    growing from the pot plant's spare sugar (0.46 to 0.92 g), and the field figure
    includes petioles, which the sim counts with the leaf. In the 30-bean plot the stem
    falls about 4.5%.
  - A fertilized pot still keeps its leaves to maturity. Mining now runs through seed
    fill (0.27 g N mined against 0.37 g taken up in
    `seed_fill_mines_first_in_a_fertilized_pot`; 5 mg before seed fill), and the
    lowest leaves are down to 16 to 21% of their mobile nitrogen four days before
    maturity, above the 10% drop threshold.
  - `feedback_loss_fill` 0.44 against at most 0.35 (about 0.63 before; Info). The lone
    pot plant has more sugar than its seeds can take.

  Follow-ups: the stem band without petioles (research), whether cambial growth yields
  to the seeds in seed fill, and CROPGRO-style leaf senescence in proportion to the
  nitrogen mined (`SENRTE`, to verify).
- Exudation is up to about 2 mg C a day in the pot bean, 0.06 g over the season.
- `stem_mass` and `root_mass` now include starch, so they read higher than before by
  up to about 25% of structure; the structural columns keep the old numbers.

## References
- Davidson, D. J. and Chevalier, P. M. (1992). Storage and remobilization of
  water-soluble carbohydrates in stems of spring wheat. Crop Science 32.
- Nguyen, C. (2003). Rhizodeposition of organic C by plants. Agronomie 23.
- Peña-Valdivia, C. B. et al. (1991), stem sugars of bean through seed fill (R1 numbers,
  research thread).
- Sinclair, T. R. and de Wit, C. T. (1975). Photosynthate and nitrogen requirements for
  seed production by various crops. Science 189.
- Waters, S. P. et al. (1980), stem starch of bean through pod fill (R1 numbers,
  research thread).
- Warembourg, F. R. et al. (2003). Legume exudation of assimilated ¹⁴C (R1 numbers,
  research thread).
