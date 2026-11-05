# Plant realism review: what is scripted, what is missing, what to build next

Written 2026-11-05 by the "Plan plant realism upgrades" thread, at model 0.13.0 (main
51049c9). This is **step 1** of the process the owner set: brainstorm, research and
review, then agree the top priorities with the owner, and only then write a full spec
for each chosen item. Nothing here is a work package yet, and no sim code changed.

Inputs: the Project Q&A discussion of 2026-11-05, the to-do list in the project files
(`coordination/model-todo.md`), every model card's Known limits, design plan §2, §3,
§7 and §9, docs/STATUS.md, the research notes in docs/archive/research/ and
docs/research/, a read of the root, allocation and allometry code, and fresh runs of
`single_pot`, `response/control`, `response/drought_from_flowering`, `m2/pole_trellis`
and `m2/wheat_spaced` (world seed of each scenario, one seed only).

Tags: **[M]** measured in those runs, **[F]** read in a source (abstract or the
archived research notes), **[K, to verify]** from knowledge, to be checked by the
Science research thread before a spec relies on it, **[I]** inference or arithmetic.

---

## 0. Summary

1. **The biggest realism gap isn't on the to-do list. It's missing carbon sinks.**
   From flowering on, the bean throws away about 70% of the sugar its leaves could
   make (the pole bean 79%, wheat 57%) [M], because nothing is left to grow into. Stems stop
   gaining mass at about day 60, roots never thicken, and there is no starch store in
   stems. So a stress (shade, drought, heat) barely costs the plant anything after
   flowering, and plants in a plot hardly compete. Stem and root thickening is the
   must-have on the to-do list, and it is also what fixes this.
2. **The root system is about 100 times too short early on.** At day 20 the sim bean has 1 m
   of root. Measured beans in sand have 120 to 170 m of basal and hypocotyl-borne root
   alone at 21 days [F]. Root to shoot is 0.05 to 0.07 through seed fill against 0.08
   to 0.30 published, and wheat's is 0.02 to 0.03 [M]. Thickening alone won't fix this:
   the fine roots (second-order laterals) and early root growth are missing too.
3. **Roots sense nothing.** The design plan says dry cells slow root elongation, but
   the code doesn't do it [M]. There is no hydrotropism, no extra branching in wet or
   N-rich soil, and no response to compaction or waterlogging.
4. **Water inside the plant is a single step from soil to leaf.** There is no water
   potential anywhere, not even in the soil. WP20 (#99) now draws wilting from the
   stress factor and a lean from per-leaf light, but in the sim there is no turgor and
   no phototropism, so the plant's shape and water use don't change with either.
5. **Level of detail: the idea works, but pots and plots need less of it than
   expected.** A whole bean season takes about 0.2 s to fast-forward in a pot and
   about 2 s in a 30-bean plot [I from the perf numbers], so week, month and season
   steps don't need a simplified model at those scales. The mechanism that does need
   two levels is plant hydraulics at plot scale. Fields (M4) need the planned patch
   model. The one hard rule is that fidelity must never depend on where the camera
   points, or replays and challenges would stop matching (§4).
6. **Recommended order** (§5): carbon sinks (thickening and storage) first, then root
   system realism, then plant hydraulics after WP27. Phototropism is a cheap extra
   that would make the WP20 lean real. Two decisions for the owner are in §6.

---

## 1. Audit: scripted, emergent or physically resolved

"Scripted" means set by the species file and a fixed rule. "Responsive" means a
scripted rule whose rate or size reacts to the plant's state. "Emergent" means it comes
out of the model's state and wasn't written in. "Resolved" means a physical flux is
computed through space.

| Process | Today | Class |
| --- | --- | --- |
| Light interception | Per-leaf rays through a voxel grid, per leaf angle, 24 sun positions and 16 sky sectors | Resolved |
| Photosynthesis and transpiration | Farquhar plus Medlyn per light class, each leaf scaled by its own light | Resolved per leaf (leaf temperature = air until WP27) |
| Sugar inside the plant | Diffusion along the organ tree, conductance `K·πr²/l`, solved exactly each hour | Resolved (diffusion, not pressure flow) |
| Soil water | 3D bucket grid: infiltration, drainage, lateral flow, evaporation | Resolved in the soil (no water potential, no upward flow) |
| Water inside the plant | Transpiration is pulled from root cells in the same hour. Stress is `min(1, FTSW/0.4)`, applied to the stomatal slope and leaf expansion | Lumped (no resistance, no storage, no delay) |
| Soil nitrogen | Mineralization, nitrification, leaching with water, per-cell uptake | Resolved in the soil |
| Nitrogen inside the plant | One plant pool tops up growing organs; mining from shaded leaves | Lumped pool, with per-organ N as state |
| Shoot skeleton | Node timing, node count, leaf and internode profiles, branch length, flowers per raceme | Scripted |
| Branching | Bud outgrowth from light, node carbon and apical dominance; topping releases buds | Emergent within scripted probabilities |
| Shade response | R:FR and daily light stretch internodes (up to 2×), thin leaves (SLA up to 1.4×), cut pod set | Responsive |
| Pod set, seed number | Carbon, light, heat and drought at the flower | Emergent within scripted probabilities |
| Organ final size | Shrinks with carbon shortfall | Emergent |
| Leaf death | Lifespan, 5 days of deep shade, starvation, frost, N mining | Emergent |
| Root skeleton | Root types, counts, max lengths, spacing and angles from OpenSimRoot bean | Scripted |
| Root growth | Thermal time; carbon via the priority rule; drought raises root priority | Responsive (speed and amount only) |
| Root direction | Previous direction + 0.2 × gravity weight + noise; pot walls deflect | Scripted with noise |
| Stem and root thickness | Fixed by the specific length at initiation: every bean stem is 4.6 mm across, the hypocotyl 2.4 mm, the tap root 0.68 mm, from day 1 to harvest [M] | Scripted, never changes |
| Phenology | Thermal time thresholds | Scripted (no stress or photoperiod effect) |

The shoot has real plasticity through light sensing and carbon-gated branching. Roots
only change speed and amount. The asymmetry Scott spotted is real.

### Numbers from the runs [M]

Bush bean, `single_pot` (fertilized pot, world seed 7):

| Day | Stage | Total g | Root g | Stem g | Leaf g | Fruit g | Root:shoot | Root length | Sugar discarded |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | emerged | 0.3 | 0.04 | 0.11 | 0.06 | 0 | 0.22 | 1.0 m | 6% |
| 40 | vegetative | 3.7 | 0.17 | 0.92 | 2.53 | 0 | 0.05 | 14 m | 45% |
| 50 | pod set | 7.5 | 0.37 | 1.72 | 5.12 | 0.08 | 0.05 | 40 m | 72% |
| 60 | seed fill | 10.5 | 0.58 | 1.91 | 5.68 | 1.90 | 0.06 | 67 m | 71% |
| 80 | seed fill | 17.4 | 1.09 | 1.92 | 5.71 | 8.31 | 0.07 | 136 m | 67% |
| 89 | mature | 20.4 | 1.29 | 1.92 | 0 | 11.07 | 0.10 | 165 m | 0% |

- Stem mass is flat from day 60 (1.91 to 1.92 g). Leaf mass is flat through seed fill
  in this fertilized pot, so no leaf senesces until all drop at maturity.
- The root axes (tap, basal and hypocotyl-borne) are 7.1 m and 0.24 g at maturity; the
  lateral cohorts carry 158 m and 1.05 g. Whole-system SRL is 128 m/g.
- Reference pot plant (Pinzón 2024, the `root_shoot_seed_fill` source): root 2.4 g
  against shoot 3.1 g plus pods 8.3 g at R8 [F, data/validation/targets.ron]. The sim
  root is about half of that at maturity and a quarter at day 60.
- Drought from flowering: root:shoot reaches 0.17 at maturity, but from the same low
  base, and stems stop at 1.64 g.

Pole bean (`m2/pole_trellis`): root:shoot 0.02 to 0.05 until maturity, 79% of the
sugar discarded at day 75. Spaced wheat (`m2/wheat_spaced`): root:shoot 0.02 to 0.03,
26 m of root at maturity, stem mass flat from day 75.

The "sugar discarded" column is the daily `feedback_loss`: the share of potential gross
assimilation removed by the sink feedback because the leaves' reserves are full.

---

## 2. Five findings that set the priorities

### F1. The plant is short of sinks, so stress after flowering costs it little
About 70% of potential assimilation is discarded through seed fill [M]. Real bean
leaves do down-regulate photosynthesis when sinks are short, but nowhere near that much.
The sugar is missing from five places a real bean puts it:
- stem secondary thickening (cambium makes wood and bark; the stem base keeps widening
  through pod fill) [K, to verify for bean magnitude];
- root secondary growth on the tap and basal roots (OpenSimRoot bean gives radial
  growth rates, with the tap root thickening most) [F];
- starch stored in stems and roots, then moved to seeds during fill (in wheat the
  stem's soluble sugars give 10 to 50% of grain mass under drought) [K, to verify];
- fine roots, which are cheap per metre but numerous;
- root exudation, about two thirds of root respiration in the OpenSimRoot bean set [F].

Consequences already logged elsewhere: shade and drought act mostly through pod
number, heat doesn't shrink seeds, competition in a plot is weak because "from
flowering on the plants are sink-limited, so shade costs little growth"
(world-and-schedule card), and density misses (STATUS, WP21 review). Filling the
missing sinks is likely to move several of those validation misses at once [I].

### F2. Roots are underbuilt, early and late
- 1 m at day 20 against 120 to 170 m of basal plus hypocotyl-borne root at 21 days in
  sand culture (Miller et al. 2003, quoted in the OpenSimRoot bean file; the file's
  author calls it "rather a lot", so take the lower end) [F].
- The nitrogen card already says drawn root length is a tenth of a real bean's at day
  40, and that a 25× uptake factor and a 4 cm reach stand in for the roots the sim
  doesn't draw (issue #58).
- There are no second-order (fine) laterals. These are 0.1 mm across with an SRL of
  about 1,350 m/g, so they add most of the length for little carbon [F, archive roots
  note §1].
- Root axes never thicken, so the tap root stays at 0.68 mm while a real one reaches
  several mm, with taper [K, to verify]. That also leaves roots short of mass.

So the root fix is two things: **more fine-root length early** (cheap carbon, large
uptake effect) and **thickening** (carbon sink and looks). Thickening alone would raise
root mass but not root length, and length is what drives uptake.

### F3. Roots don't respond to the soil
Elongation reads soil temperature through thermal time and nothing else
(`ps-arch/src/roots.rs::root_elongation`) [M]. Design plan §3.7 says "cells at wilting
point slow elongation", but that isn't implemented: a plan-versus-code gap.
Literature ranks the root responses like this [K, to verify]:
- **strongest and best documented:** local lateral proliferation in nitrate or
  phosphate patches (Drew 1975; Hodge 2004), and branching only on the wet side of a
  root (hydropatterning, Bao et al. 2014) or stopping where the tip crosses air
  (xerobranching, Orman-Ligeza et al. 2018);
- **strong:** elongation slows in dry soil (less than shoot growth does, so dry plants
  shift to roots; Sharp et al.) and in hard soil, where roots also thicken (Bengough et
  al. 2011);
- **real but weak in soil:** hydrotropism, bending toward wetter soil (Dietrich et al.
  2017 on the MIZ1 mechanism). It is much weaker than gravitropism in most crops.

So the to-do item "hydrotropism" is real, but foraging (branching where it pays) gives
more of the realism, and players will see it more.

### F4. No water potential anywhere
Soil holds water by volume, with no retention curve. The plant has no resistance or
storage. Stress reacts at once to root-zone FTSW. Consequences:
- Wilting is not a physical state. WP20 (#99) draws the "wilting and recovery" moment
  from the stress factor, which follows root-zone FTSW at once, with no turgor and no
  delay.
- Water in a wet patch can't make up for dry soil elsewhere through the plant's
  plumbing. The proportional split does some of that, but not by hydraulics.
- Root traits that breeders select in bean, like xylem size and axial conductance
  (Strock et al. 2020) or deep against shallow basal roots, can't matter in M3
  breeding [F].

### F5. Two designed moments are drawn by the viewer, not grown by the sim
- "Leaves and stems lean toward the light": WP20 (#99) poses the shoot toward its
  brighter side when drawing it, but the sim has no phototropism, so the lean never
  changes where leaves are or what light they get.
- "Wilting under drought and visible recovery": drawn from the stress factor; no turgor
  in the sim (F4).
- Topping pays off poorly because branches have no buds of their own (WP25 follow-up).
- Doc inconsistency: herb-architecture.md says leaves don't senesce from the base
  during seed fill, but nitrogen.md and its test say they do since WP24. Both are partly
  right: basal drop happens only when the plant is N-short. A fertilized pot keeps all
  its leaves until maturity, while real beans strip N from leaves to seeds whatever the
  fertilizer (the "self-destruction" of Sinclair and de Wit 1975) [K, to verify]. The herb
  card should be corrected in the next model PR that touches it.

---

## 3. Candidate list, wide

Ratings: **Needed** = a believability gap a curious player or a scientist would
notice, or one that blocks a planned feature. **Wanted** = clear value, not urgent.
**Later** = fine idea, depends on something else or is a big build. **Skip** = not for
now.

Cost is per plant per sim-day against the budgets (single pot under 2 ms, 30-bean plot
under 20 ms; both are over today, at about 2.2 and 24.5 ms). "Tiers?" says whether the
item needs a detailed and a simplified version (§4). Every item that changes an equation
bumps the model minor version and regenerates the golden hashes, so that column only
lists step-order changes.

### A. Carbon sinks and structure

| # | Item | Realism and game value | Minimal model | Cost | Tiers? | Step order | Rating |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A1 | **Stem secondary thickening** | Fixes stem mass, adds taper (base thicker than tip), absorbs surplus sugar, makes stress cost growth. Visible in every frame | Pipe model (Shinozaki 1964): each stem segment's target cross-section ∝ leaf area above it (+ a fruit-load term). The shortfall is a sink at priority 0.4 (already in §3.9). Length is frozen once elongation ends; extra mass widens the radius | One tree pass a day for leaf area above each segment, one more demand term per segment per hour: negligible | No | None (part of hourly step 12) | **Needed** (owner: must) |
| A2 | **Root secondary growth** | Tap and basal roots widen with taper, and roots gain mass. Bean data exist (Strock et al. 2017: lines with less secondary growth have 27 to 32% more root length under low P) | OpenSimRoot radial growth rates per root class (tap ×4, basal ×0.7, hypocotyl-borne ×0.6, laterals 0.1 to 0.3), as a sink at priority 0.4 | Negligible | No | None | **Needed** |
| A3 | **Stored starch (reserve pool) in stems and roots, moved to seeds in fill** | Seeds fill from storage when leaves fail: drought and heat after flowering hit seed size realistically. Wheat grain gets its stem sugar | A storage term per stem and root organ: refill at priority 0.2 (in §3.9, never built), released when the local concentration falls | Negligible | No | None | **Needed** (goes with A1) |
| A4 | Root exudation | Carbon to soil, a cost of roots | A share of root respiration (OpenSimRoot about 2/3) booked to the ledger | Negligible | No | None | Wanted (fold into A2) |
| A5 | Stem mechanics: bending under load, wheat lodging, how a vine falls | Pods weigh branches down; wheat lodges in rain and wind. Becomes possible once A1 gives real radii | Cantilever bending from radius and modulus (Niklas 1992); wheat lodging after Baker et al. 1998 [K] | Small daily | No | None | Later (after A1, with WP27's wind) |
| A6 | Wind thickening (thigmomorphogenesis) | Rubbed or wind-shaken bean stems grow shorter and thicker (Jaffe 1973, on bean) [K] | Multiplier on A1's target from wind | Negligible | No | None | Later |

A note for A1: phloem conductance is `K·πr²/l`, so thicker stems would also carry
sugar more easily. The pole bean already sits near the top of its final-height band, and
its height depends on a raised conductance. The spec has to decide whether conductance
follows the whole radius or only a phloem ring [I].

### B. Roots

| # | Item | Realism and game value | Minimal model | Cost | Tiers? | Step order | Rating |
| --- | --- | --- | --- | --- | --- | --- | --- |
| B1 | **Early root vigor**: basal and hypocotyl-borne root rates and lengths re-read against OpenSimRoot and Miller 2003 | Length right in the first month, when it matters most | Parameter and allocation re-tune; root length per day against a target curve | None | No | None | **Needed** |
| B2 | **Second-order laterals** | Most of the real root length; makes uptake follow real root length, so the 25× factor and the 4 cm reach (issue #58) can go | Second-order cohorts on first-order cohorts, or a fine-root length per soil cell grown from each cohort (no organs). The second is cheaper and is the "hybrid" root model idea (Dupuy et al. 2010) [K] | Cohorts: +50 to 100% root organs, so allocation cost up a similar share; per-cell field: small | No (pick one form for all scales) | None | **Needed** |
| B3 | **Local branching plasticity (foraging)** | Roots fill wet and N-rich patches. Shows in the rhizobox glass; patchy plot soil (decision 0027) gets a visible response; "put the fertilizer where the roots are" becomes a lesson | Lateral cohort density × f(local water) × g(local nitrate), bounded; skip branching where the parent cell is near wilting point | Negligible | No | None | **Needed** |
| B4 | Elongation response to dry and hard soil | Roots slow in dry or compacted soil, then recover. Fixes the §3.7 plan-versus-code gap | Elongation × f(cell water) × h(soil strength); hard soil also thickens the tip | Negligible | No | None | **Needed** (water part); compaction needs C5 |
| B5 | Hydrotropism | Tips bend toward wetter cells | One more term in the direction rule, from the water gradient of neighbouring cells, weight well below gravity | Negligible | No | None | Wanted (cheap, so fold into B3/B4) |
| B6 | Waterlogging (low oxygen) | Over-watered pots are a classic beginner mistake: roots stop, then die, and the plant wilts in wet soil | Air-filled porosity per cell from saturation; elongation and uptake drop to zero below about 10% air; roots die after some days | Negligible | No | None | Wanted (high game value) |
| B7 | Fine-root death and turnover | Roots turn over; carbon to soil | Age-based death of fine laterals | Negligible | No | None | Later |
| B8 | N uptake from real root length | Removes the 25× fudge, so plot nitrogen fixation can rise from 18% toward 30 to 60% | Follows from B1 and B2 | None | No | None | **Needed** (part of B2) |

### C. Water

| # | Item | Realism and game value | Minimal model | Cost | Tiers? | Step order | Rating |
| --- | --- | --- | --- | --- | --- | --- | --- |
| C1 | **Soil water retention (water potential per cell)** | Needed for C2 and B4; enables a soil water-potential lens | van Genuchten curve per texture (Carsel and Parrish 1988 table) [K]; keep the bucket flow | Small | No | None | **Needed if C2** |
| C2 | **Plant hydraulics**: root radial and axial conductance, stem conductance, leaf water potential, stomata closing on leaf water potential | Real flow through stems and roots; wilting from turgor loss; uptake shifts to wet zones on its own; breedable root xylem traits (Strock et al. 2020); split-root and drip-irrigation experiments | Doussan et al. 1998 hydraulic network on the organ tree: given transpiration, one linear solve gives every water potential and each root segment's uptake. It is the same tree shape as today's sugar solve, so it can reuse it. Stomata read the previous hour's leaf potential (Tuzet et al. 2003), so no iteration [K] | About one more tree solve per hour: roughly 1 ms per plant-day at 830 organs [I], so 30 ms single-thread or 8 ms on 4 cores for a plot | **Yes** (§4) | **Changes hourly steps 5 to 7** (decision record) | **Needed** (owner: wanted if no large downside) |
| C3 | Water stored in stems and leaves (capacitance) | Delays and smooths stress; matters for trees, little for beans | One capacitance per organ in the C2 solve | Small | With C2 | With C2 | Later (M4 tree) |
| C4 | Upward water flow (capillary rise), slower drainage | Soil dries from below as well as above; nitrate below the roots comes back (nitrogen card) | Darcy flux between cells from C1 potentials, bounded per hour | Medium (soil grid pass; plot hydrology is 11 ms already) | Maybe | Inside step 4 | Later |
| C5 | Soil bulk density and strength (compaction, tillage) | Paths, raised beds and digging as player tools; root response B4 | Bulk density per cell; strength from density and water (an empirical curve, for example Whalley et al.) [K] | Negligible | No | None | Wanted (with B4) |
| C6 | Bean leaf folding in heat and drought (paraheliotropism) | Beans visibly fold their leaflets at midday under stress, cutting light and water loss (Berg and Hsiao 1986) [K]. Lovely to watch, and the research thread already flagged it | Leaf angle from stress and light each hour; light reads the angle | Light coefficients are daily, so this needs an hourly angle correction | No | Touches light (§3.2 step 3) | Wanted |

### D. Nitrogen

| # | Item | Realism and game value | Minimal model | Cost | Tiers? | Step order | Rating |
| --- | --- | --- | --- | --- | --- | --- | --- |
| D1 | Seed-driven N stripping from leaves, even when fertilized | Leaves yellow from the base during fill in every plant; capacity falls, so the sink problem also eases from the source side | Seed N demand draws on leaf N at a rate cap (CROPGRO NMOBMX, in research m2-m3 notes) [F] | None | No | None | **Needed** (small, goes with A3) |
| D2 | Pod set reads N | N-short beans shed pods, as real ones do (nitrogen card) | One more factor in pod set | None | No | None | Wanted |
| D3 | N transport along the stem | N lens shows real gradients; a nitrogen-short branch is local | Second tree diffusion (or xylem nitrate with water from C2 plus phloem amino acids) | One tree solve per hour | Yes (pool = simple tier) | New hourly step after 12 | Later (value mainly in a lens) |

### E. Shoot development

| # | Item | Realism and game value | Minimal model | Cost | Tiers? | Step order | Rating |
| --- | --- | --- | --- | --- | --- | --- | --- |
| E1 | **Phototropism** | Plants lean toward the window or out of shade, and the lean changes the light they get. WP20 draws a lean; this makes it real | Daily bend of new internodes toward the light-weighted direction from the daily coefficients (§3.4 already specifies it) | Negligible | No | None | **Needed** (cheap) |
| E2 | Branches on branches, buds and racemes on branches, a branch taking over as leader | Topping and pruning pay off as in a garden (WP25 follow-up) | Bud sites on branch nodes with the same fate rule | More organs | No | None | Wanted |
| E3 | Drought speeds development | Dry beans mature earlier (a known bean response) [K, to verify] | Stress term on thermal-time accumulation from flowering | None | No | None | Wanted |
| E4 | Heat shortens seed fill, so seeds are smaller | Fixes the two heat seed checks | Fill duration in thermal time with a high-temperature cut [K, to verify] | None | No | None | Wanted |
| E5 | A spread of leaf angles; leaves tracking the sun | Fuller-looking canopies; better light | Drawn angle per leaf (decision 0025 note) | None | No | None | Later |
| E6 | Wheat photoperiod and vernalization | Winter wheat and sowing-date experiments | Standard CERES-style factors [K] | None | No | None | Later (when winter crops come) |
| E7 | Wheat grain number from ear growth; stem sugar to grain | Fixes grains per ear (13 against 30 to 40) | Grains set from ear mass at anthesis; A3 for stem sugar | None | No | None | Wanted (wheat) |

### F. Not now
Disease and pests, phosphorus and mycorrhiza, salinity, CO2 enrichment beyond today's
parameter, allelopathy. Each is a whole model. Phosphorus is the most interesting
later, because bean root breeding (shallow basal roots, less root secondary growth) is
largely about phosphorus (Lynch's work) [F for Strock 2017].

---

## 4. Level of detail: design sketch

Scott's rule: fully flow-resolved when zoomed in on a few plants at low speed;
simplified for big plots, fast-forward and turn-based steps, as long as the outcomes
stay similar.

### 4.1 Where it is needed
- **Fast-forward at pot and plot scale is already fast.** A single pot costs about
  2.2 ms per sim-day, so a 90-day season fast-forwards in 0.2 s. A 30-bean plot at
  about 25 ms per sim-day takes about 2.3 s, and a wheat stand at about 110 ms takes
  15 s for 140 days [I from docs/performance.md and STATUS]. Week, month and season
  steps don't need a simplified model at these scales. Only wheat at live speed is
  tight.
- **Most items in §3 are cheap enough to run the same way everywhere**: thickening,
  storage, root plasticity, tropisms, waterlogging, phenology. One version means no
  equivalence to test.
- **Two levels earn their keep for plant hydraulics (C2) at plot scale**, and later
  for N transport (D3).
- **Fields and forests (M4)** need the patch-cohort model already planned (§3.15,
  WP34 and WP36). This scheme is the same idea one level down.

### 4.2 The hard rule: fidelity can't depend on the camera
The sim is deterministic: world seed, model version and action log decide the result.
Challenges are scored from the replay (§3.14). If zooming in switched a plant to the
detailed model, the result would depend on where the player looked, and a replay
without the camera path would give a different harvest. Three ways to keep the rule:

1. **Fidelity is fixed by scale, at run start** (recommended). Pots and small plots
   (up to a set plant count) always run detailed; large plots, fields and the M3
   breeding bench always run simplified. The scenario records it, so replays match.
   For a closer look at one plant in a big plot, the viewer runs a **display-only
   detailed shadow** of that plant: same soil and light inputs, detailed hydraulics, its
   flows drawn in the lenses, but it never feeds back into the sim. Because the two
   levels agree (§4.4), what the shadow shows matches the authoritative plant closely.
2. Fidelity follows zoom and speed, and every switch is written to the action log as
   an action. Replays match, but the harvest then depends on how you watched, and
   challenges would need to pin fidelity.
3. Detailed everywhere, and pay for it with performance work. Possible for C2 (about
   8 ms per plot sim-day on 4 cores [I]), but it eats the margin the plot budget
   doesn't have.

### 4.3 How the two levels are built
The pattern is the one the light model already uses: an expensive geometric solve once a
day, then a cheap hourly step from its results.
- **Detailed (hydraulics):** the full root-stem-leaf network solve every hour.
- **Simplified:** once a day, solve the network once and reduce it to two numbers per
  plant, the root system's total conductance and the share of uptake each soil cell
  provides under uniform soil (Couvreur et al. 2012, the "standard uptake fraction")
  [K]. Each hour, leaf water potential and per-cell uptake come from those numbers and
  the cells' water potentials: as cheap as today's proportional uptake. The
  simplification is exact when the soil is equally wet everywhere and approximate when
  it isn't; Couvreur adds a compensation term for that.
- **Prefer simplifications with no state of their own.** Hydraulics without
  capacitance is quasi-steady: water potentials follow from structure and soil each
  hour. So switching levels has nothing to convert, and a switch can't break the water
  ledger. N transport (D3) has state (N per organ); its simple level is today's pool,
  and the conversion is "sum" one way and "the current top-up rule" the other, which
  conserves N exactly.

### 4.4 How to test that outcomes stay similar
A new harness next to `ps-cli validate` (the fidelity report):
- **Outcome vector**, per scenario on the four-seed means already used by validation:
  total and organ dry mass, seed yield, pods, height, water use and water use
  efficiency, root mass by depth, N uptake, flowering and maturity dates, and daily
  minimum leaf water potential.
- **Pass rule**: each outcome's difference between levels is within the larger of 5%
  and half the seed-to-seed range of the detailed level. In words: switching
  levels moves a result less than a different seed does. Every validation gate must
  pass at both levels.
- **Hard cases on purpose**: drought from flowering, a wet-and-dry split pot, a dense
  plot, and patchy plot soil. These are where Couvreur's approximation is weakest.
- **Switch test**: run detailed to day 30, simplified to day 60, detailed to the end.
  Ledgers close at 1e-9 and outcomes pass the same rule.
- **Determinism**: golden hashes per level. Both levels stay bit-exact on one
  platform.
- Runs in CI as a report next to validation, and fails the build like a gate.

---

## 5. Recommended priorities and order

Proposed packages, named R1 and so on so they don't collide with the WP numbers.
Accepted ones get WP numbers and issues when specced.

| Order | Package | Contains | Size | Why here |
| --- | --- | --- | --- | --- |
| 1 | **R1 Carbon sinks** | A1 stem thickening, A2 root secondary growth, A3 storage and remobilization, A4 exudation, D1 seed-driven N stripping; fix the herb card's senescence line | L | The must-have. Fixes F1, which is behind several validation misses and the weak plot competition (an M2 exit target). Changes the carbon budget a lot, so it comes before more tuning |
| 2 | **R2 Root system** | B1 early vigor, B2 fine laterals, B8 uptake from root length (closes #58), B3 foraging, B4 dry-soil response, B5 hydrotropism | L (two PRs: R2a length and uptake, R2b plasticity) | Fixes F2 and F3. R2a after R1, because thickening and root length share the root carbon |
| 3 | **R3 Plant hydraulics** | C1 soil retention, C2 network and stomata on leaf potential, turgor for wilting, the simplified level and the fidelity report (§4) | L | Fixes F4 and gives WP20's wilting a turgor value to draw. After WP27, because both change how stomata and transpiration are set, and doing hydraulics first would mean redoing it |
| any time | **R4 Phototropism** | E1 | S | Cheap; makes WP20's drawn lean real. Could go with R1 |
| later | R5 Soil extras | B6 waterlogging, C5 compaction, C4 upward flow | M | High game value (overwatering, raised beds) once R2 and R3 exist |
| later | R6 Shoot extras | E2 branches on branches, E3 drought phenology, E4 heat on seed fill, E7 wheat grain number, D2 pod set reads N, C6 leaf folding | M each | Triage again after R1, since R1 may move the heat and drought checks |
| later | R7 N transport | D3 with two levels | M | Mostly lens value |

### Fit with current work
- **Implementation: sim model changes** has the re-tune (model 0.15.0, after
  genetics 0.14.0) almost done, then WP27. R1 would follow WP27 as the next model change.
  R1 shifts the carbon budget, so the re-tune's seed-mean tuning will need a pass
  again after R1; the re-tune isn't wasted, because its items (g1, temperature trapezoid)
  are physiology, not compensation.
- R1 and R2 don't change the step order. R3 changes hourly steps 5 to 7 (water status
  from the previous hour's leaf potential, uptake from the network solve). That needs a
  decision record and a design plan §3.2 edit.
- **Performance**: R1 and R2 cost little, except B2 if fine roots become organs (the
  spec should pick the per-cell form). R3 needs Performance to time the network solve
  before the spec fixes the plot fidelity threshold.
- **Science research** data needed for the specs: bean stem and hypocotyl diameter
  over the season, stem share of shoot mass, a root length time series, bean root
  radial and axial conductances and the leaf turgor-loss point, and the hydropatterning
  and nitrate-patch response sizes. Most are "[K, to verify]" above.
- **Viewer**: R1 gives stems and roots real taper; R3 gives a turgor value per leaf
  for wilting; R4 makes the drawn lean real. Each needs only the existing snapshot fields plus a
  per-leaf turgor value.
- **M3 breeding**: R2 and R3 make root traits (basal root angle, secondary growth,
  xylem conductance) real breedable traits, which is among the best-documented bean
  breeding work [F].

---

## 6. Decisions for the owner

1. **Which priorities get full specs.** Recommended: R1 carbon sinks, R2 root system
   and R3 plant hydraulics, with R4 phototropism folded into R1. The order above.
2. **The level-of-detail rule** (§4.2). Recommended: fidelity fixed by scale at run start,
   plus a display-only detailed view of one plant in a big plot. The alternative is
   switching with zoom and speed, logged in the action log, so replays still match but
   results depend on how the run was watched.

---

## 7. Sources

- Strock, Morrow de la Riva and Lynch 2017, Reduction in root secondary growth as a
  strategy for phosphorus acquisition, Plant Physiol 176:691–703,
  [doi:10.1104/pp.17.01583](https://doi.org/10.1104/pp.17.01583) (PubMed abstract) [F].
- Strock, Burridge, Niemiec, Brown and Lynch 2020, Root metaxylem and architecture
  phenotypes integrate to regulate water use under drought stress, Plant Cell Environ
  44:49–67, [doi:10.1111/pce.13875](https://doi.org/10.1111/pce.13875) (PubMed
  abstract) [F].
- OpenSimRoot bean (Carioca) parameter set, Postma et al. 2017 New Phytol 215:1274;
  values and Miller et al. 2003 quote as read in docs/archive/research/notes/roots.md [F].
- Pinzón 2024 and Acosta-Díaz 2007 root:shoot values as recorded in
  data/validation/targets.ron [F].
- CROPGRO-Drybean N mobilization (NMOBMX) as read in
  docs/archive/research/m2-m3-bean-science.md [F].
- To verify: Shinozaki et al. 1964 (pipe model); Doussan, Pagès and Vercambre 1998
  (root hydraulic architecture); Couvreur et al. 2012 HESS (macroscopic uptake from
  hydraulic architecture); Tuzet, Perrier and Leuning 2003 (stomata and leaf water
  potential); van Genuchten 1980 and Carsel and Parrish 1988 (retention); Drew 1975 and
  Hodge 2004 (root proliferation); Bao et al. 2014 (hydropatterning); Orman-Ligeza et
  al. 2018 (xerobranching); Dietrich et al. 2017 (hydrotropism); Bengough et al. 2011
  (elongation, drought and impedance); Dupuy, Gregory and Bengough 2010 (continuous root
  models); Sinclair and de Wit 1975 (self-destruction); Jaffe 1973 (bean
  thigmomorphogenesis); Berg and Hsiao 1986 (bean leaf movement); Niklas 1992 and Baker
  et al. 1998 (stem mechanics and lodging) [K, to verify].
