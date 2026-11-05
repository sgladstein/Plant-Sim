# 0013: Herb architecture details (WP8)

- **Date:** 2026-11-05
- **Status:** accepted

## Context

WP8 builds the bush bean's architecture (design plan §3.4, §3.7, §3.10, §3.11) on the
organ graph and the WP7 physiology. The plan leaves many details open: how the seed is
represented, how a new organ gets its first carbon, how roots are lumped, how the
light index of §3.11 shade senescence is measured, when branching stops, and what
happens after maturity. The first 90-day run also showed that with sinks bounded by
their expansion curves, a well-lit plant fixes far more carbon than it can use: by day
75 the plant held 30 g C of reserve against 17 g C of structure, a reserve
concentration of about 1.4 g C per g DM, which no plant tissue can hold.

## Decision

1. **Seed.** The seed is four organs at sowing: the collar (organ kind `Seed`; the
   embryo's root pole, where every seed-borne root attaches), the hypocotyl (not yet
   expanding), and two cotyledons on the hypocotyl holding the mobilizable reserve
   (`seed_mass × carbon fraction × mobilizable fraction`). The non-mobilizable 30% of
   the seed mass is structure, split 20/40/40 over the collar and the two cotyledons.
   Germination starts the hypocotyl's expansion curve (duration `internode_expansion`,
   potential length sowing depth plus `hypocotyl_above`), which lifts the cotyledons to
   the surface. Cotyledons and the collar carry no nitrogen, so no maintenance.
2. **Primordia.** A new organ starts with 2% of its potential mass (`PRIMORDIUM`, "~"),
   moved from its parent's reserve to its structure (at most half the parent's
   reserve). Without it a new organ of near-zero mass holds almost no reserve and
   needs days to start growing, because growth is a share of the organ's own reserve
   (decision 0009). The move leaves the carbon stock unchanged.
3. **Main stem and branches.** Node 1 is the epicotyl with two opposite unifoliate
   leaves and two buds; each later node is an internode, a trifoliate leaf and (on the
   main stem) a bud, one per phyllochron of air thermal time. The main stem makes
   `nodes_before_terminal` nodes, then a terminal raceme. Leaf size and internode
   length follow the profiles in the species file. Branches have `branch_nodes` nodes
   at `branch_angle` toward their subtending leaf, bend up by `shoot_gravitropism` per
   node, and end in a raceme.
4. **Bud fate.** Daily, a dormant main-stem bud at rank `r ≤ N − 2` activates with
   probability `bud_activation × L × F × (1 − exp(−(n − r)/apical_dominance_nodes))`,
   with `L` its leaf's light index, `F` its node's reserve concentration over
   `bud_reserve_reference` (capped at 1), `n` the main stem's current node count (the
   dominance term is 1 once the main stem has terminated). Branching stops at
   flowering (R1); dormant buds above rank `N − 2` then become racemes.
5. **Flowers and pods.** Flowers open only from R1, `flowers_per_raceme` over
   `raceme_flowering_days`. A flower closes after `flower_lifespan` days and sets a pod
   with probability `pod_set_base × carbon × temperature × drought`, where carbon and
   temperature are the means over its open days of the raceme's node reserve factor
   and the CROPGRO pod-set response to the daily mean temperature, and drought is the
   lowest FTSW while open over `drought_abortion_ftsw`, capped at 1. Past
   `seed_fill_delay` on its expansion clock a pod makes its seeds: each of
   `seeds_per_pod` ovules sets with probability `ovule_set`, at least one.
6. **Light index.** §3.11's "below 10% of full-sun absorbed PAR" is measured as
   `0.5 × (mean direct sunlit probability over the day's sun-up hours + diffuse
   transmittance)` from the daily light coefficients; a fully expanded leaf below
   `shade_light_threshold` for `shade_senescence_days` days senesces.
7. **Sink feedback on photosynthesis.** A leaf's capacity is multiplied by
   `clamp(1 − (C − 0.10)/(0.30 − 0.10), 0, 1)` with `C` its reserve concentration in
   g C per g DM (end-product inhibition, as in L-PEACH; both thresholds "~" and species
   constants). Below the class table's lowest capacity node every flux is scaled in
   proportion. After this the same run holds 0.2 g C per g DM in its leaves at peak
   and spends its reserve by maturity.
8. **Roots.** Each seed-borne root type makes `count` axes once thermal time since
   germination passes its emergence time: the radicle down from the collar, basal
   whorls from the collar at `insertion_angle` from vertical (the second whorl offset
   by half a step), hypocotyl-borne roots along the buried hypocotyl at golden-angle
   azimuths. An axis is a chain of `root_segment_length` segments; a segment's
   expansion duration is its length over the elongation rate, so carbon limits final
   length, not timing. A finished tip adds the next segment, bent by
   `0.2 × gravitropism` toward down plus Gaussian noise (sd 0.15, purpose `Tropism`).
   At the pot wall a segment turns straight down, else along the wall, else the axis
   stops. Laterals are lumped: once an axis is past its apical zone it emits one
   cohort per segment, a single organ of the lateral type's full length standing for
   `segment / branch_spacing` identical roots (`multiplicity`), at `insertion_angle`
   from the parent and golden-angle azimuths, mirrored downward if aimed up.
   Hydrotropism and phototropism are not applied yet (weights are in the file).
9. **Root length in the soil.** Each root organ's length is spread evenly along its
   geometric extent, sampled every half soil cell; length outside the soil is dropped.
10. **Shedding.** The architecture sheds an organ through `allocation::shed` after
    moving its positive reserve (mobile carbon) to its parent.
11. **Maturity.** At R9 every leaf is shed, starvation is no longer checked, and all
    organ nitrogen is set to zero, so the harvest no longer respires. Development stops.

## Consequences

- WP8's done-when holds across eight weather years (seeds 1 to 8): 7 main-stem nodes,
  3 to 5 branches, 11 to 16 pods, 3.5 to 3.9 seeds per pod, 19 to 27 g dry mass, 39 cm.
- The sink feedback is a model change outside §3.8's text; it is the smallest rule
  that keeps reserves physical when sinks are full. WP12 should check it against
  measured bean leaf starch and decide whether larger sinks (stem thickening, §3.4
  secondary growth; deeper root growth) should take over part of its role.
- Stems carry no secondary thickening yet and the root system is about 4% of dry
  mass, low for bean (WP12). Fruit is about 85% of living dry mass at maturity because
  leaves have dropped.
- Lumped laterals are one organ per cohort, so a 90-day plant has about 600 root
  organs (about 200 of them cohorts) instead of tens of thousands. The viewer draws a cohort `multiplicity` times.
