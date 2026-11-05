# 0022: Grass module and spring wheat (WP23)

- **Date:** 2026-11-05
- **Status:** accepted. Spring wheat follows the research recommendation; the owner's card is still open.

## Context
WP23 (design plan §7, M2) adds a grass module and a grass species file, covering
tillering, intercalary growth and fibrous roots. It is done when "tiller count and
height [are] inside a literature range to be chosen in WP23". The M2 research report
(research/m2-m3-bean-science.md, decision 5) recommends spring wheat. Wheat has the
deepest open parameter base (WALTer, elong-Wheat, CPlantBox), and a spring type needs
no vernalization. It should be built so that perennial ryegrass is a parameter swap.

## Decision
1. **One engine, a grass path.** A species whose growth form is `Grass` carries the new
   `grass` group (species schema 4, optional for other forms). `Plant` takes the grass
   path in emergence, bud fate, organ initiation and flowering. Phenology, carbon,
   light, water, nitrogen, roots and senescence are shared with the bean.
2. **Seed and emergence.** The grain keeps its reserve in one endosperm organ that
   stays in the soil (kind `Cotyledon`, role `Endosperm`, shed when spent or after 21
   days). The hypocotyl slot carries the coleoptile and subcrown internode to 1 cm
   above the soil. The first leaf unrolls at emergence.
3. **Nodes.** Each axis makes a node every phyllochron (85 °C·d). A node is an
   internode, a leaf sheath standing on it, and a blade at the sheath's top. Sheath
   lengths rise linearly with rank, from 3 cm to 15 cm. Blade areas rise from 15% of
   the largest to 100% at the second-to-last leaf; the flag leaf is at 80%. Blades
   start 40° above horizontal and arch down.
   - Below the top `elongating_internodes` (4), internodes are 3 mm crown internodes.
   - The top ones elongate from 20% to 100% of the potential (18 cm), each starting a
     phyllochron after its node appears (WALTer). Intercalary growth is therefore
     delayed stem growth, and the blades still unroll from the sheaths.
4. **Tillers.** Every crown node of an axis below the highest tiller order (2) has a
   tiller site in its axil.
   - A site's time comes `tiller_delay` (2) phyllochrons after its leaf (Klepper:
     T_n with leaf n + 2 or n + 3). It then fills once, with probability
     `site_filling` (0.69, Neuteboom) times the node's carbon factor, or stays empty
     for good. Each site is decided once, on its own stream (`BudFate`).
   - A tiller gets the leaves that let it head with its parent:
     `parent leaves − rank − delay + 1`.
   - **Light stops tillering:** no site fills when the red:far-red at its node is
     below `tiller_stop_rfr`. WALTer's 0.35 is a measured R:FR. The sim's proxy
     (`1.2 exp(−0.5 LAI above)`) counts only the leaf area straight above a leaf and
     misses the far-red that neighbours reflect sideways. At the crown of a
     200 per m² stand at jointing it reads 0.5 to 0.6, against 0.7 to 1.2 for a spaced
     plant, so the threshold is set on the proxy at 0.6 (~).
   - **Jointing:** once the main stem's first elongating internode starts to grow, no
     more sites fill, and tillers with fewer than 2 leaves die with what they carry
     (WALTer: regression from jointing, youngest first).
5. **Ear and grain.** After its last leaf, each axis makes a peduncle (30 cm) and an
   ear (kind `Pod`, role `Ear`, 0.45 g of chaff over 200 °C·d).
   - 180 °C·d after heading, each of 50 florets sets a grain (kind `SeedInPod`, role
     `Grain`). The chance is `ovule_set` (0.85) times the mean temperature factor of
     the last 7 days' maxima, times the ear's carbon factor. Each ear sets at least
     one grain.
   - Grains fill over 550 °C·d.
   - Anthesis (stage `Flowering`) is a thermal time (900 °C·d), as for the bean. Grain
     set waits for it.
6. **Roots.** Wheat uses the existing root types:
   - a seminal primary root (`Radicle`) and four seminal roots (`Basal`, 40° from
     vertical), at 2.2 cm a day up to 80 cm (CPlantBox);
   - one cohort of 12 crown roots (`Hypocotyl` origin, along the subcrown internode,
     from 300 °C·d);
   - laterals of 3 cm.
   There is no tap-root dominance and no nodulation (`nodule_activity` 0).
7. **Transport.** The wheat file uses a phloem conductance of 1500 g/m/h, as the pole
   bean does (decision 0021). At 500, the 15 to 20 cm culm internodes conduct too
   little as solid cylinders, and the ears set 13 grains instead of 20 (spaced plants).
8. **Reports.** `branches` counts living tillers for a grass, `pods` counts ears, and
   `seeds` counts grains.
9. **Version.** Model 0.6.0 → 0.7.0, species schema 4. The goldens were regenerated:
   the bean's daily and organ tables are unchanged, and only the state hash moves (new
   serialized fields).

## Consequences
- **Spaced plants** (m2/wheat_spaced: 4 pots of 40 cm, sown 10 April at 45 N):
  - They make 4.3 tillers and 5.3 ears and keep all their tillers.
  - They reach 0.76 m.
  - They head at about day 60, are mature by day 91, and fill 69 grains (3.1 g of
    11.1 g, harvest index 28%).
- **The stand** (m2/wheat_stand: 200 per m², rows 20 cm apart):
  - Plants make 2.0 tillers and 3.0 ears, so the density response is −52%.
  - They reach only 0.40 m. The stand is source-limited while its culms elongate,
    and an internode that misses its carbon stays short, with no catch-up.
  - Grain is 176 g per m², against 300 to 600 for a real crop.
  - `wheat_height_stand` is an Info miss.
- **Per ear**, grain number is low: 13 on spaced plants and 11 in the stand, against 30
  to 40 in the field. Roots are 2% of the plant, as the bean's are (decision 0018).
- **Targets chosen here** (all Info):
  - spaced plants: 3 to 10 living tillers;
  - the stand: 1 to 5 tillers;
  - height 0.7 to 1.0 m;
  - density response: −30 to −90%.
  The research table's "15 to 40+ tillers for spaced plants" describes long-season
  winter types and ryegrass, not a 9-leaf spring wheat.
- **The viewer** sees blades as `Leaf` organs with role `Blade` and an arching strip
  geometry (`grass::blade_points`); sheaths, peduncles, ears and grains have their own
  roles.
- **Follow-ups:**
  - stem elongation as a priority sink at jointing (the stand's height);
  - grains per ear from ear growth before anthesis (Fischer's spike-mass rule);
  - crown roots per tiller;
  - ryegrass, with mowing and regrowth;
  - a real R:FR model (horizontal reflection) to bring the threshold back to 0.35.
