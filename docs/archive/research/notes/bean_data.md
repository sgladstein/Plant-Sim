# Common bean (Phaseolus vulgaris) validation data and botany check for a bush-bean simulator

Scope note: about 20 tool calls. DSSAT CROPGRO-Drybean genotype files were downloaded and read directly (saved under /tmp/claude-0/-home-claude-Plant-Sim/b9667701-e1b8-5d1a-aab2-5cb6e2180076/scratchpad/downloads/bean/: BNGRO048.CUL, BNGRO048.ECO, BNGRO048.SPE). The GitHub API for DSSAT/dssat-csm-data is blocked in this session, so I could not list the Drybean experiment files (*.BNX/*.BNA/*.BNT). Guessed names such as UFGA8201.BNX returned 404 on raw.githubusercontent.com.

## 1. Concrete validation data sets (time series, phenology, multi-environment)

### Takeaway
The best openly available bean parameter and calibration set is DSSAT CROPGRO-Drybean (CRGRO048). Its cultivar, ecotype and species files are readable as raw files and hold phenology, phyllochron, leaf size, SLA, seed size, seeds per pod and internode-length-per-node tables for both Type I (determinate) and Type II-IV habits. For snap bush beans in a temperate climate, the Oregon State CROPTIME degree-day model for 'Provider' is a ready-made field phenology target. I found no open per-organ weekly time-series dataset; digitizing figures is still the route.

### Cited Findings
- **DSSAT CROPGRO-Drybean species, ecotype and cultivar files** are available at https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Data/Genotype/BNGRO048.CUL (also .ECO and .SPE). They include generic habit cultivars: 990001 "Andean Habit 1", 990005 "Meso Amer. Hab.1" and 990002-990007 for the other habits, plus calibrated cultivars such as Seafarer (MESDET), Manitou, Redkloud, Canadian Wonder and Isabella (ANDDET), and Porrillo Sintetico, BAT 477 and Carioca (MESIND). — [BNGRO048.CUL](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Data/Genotype/BNGRO048.CUL)
  - Ecotypes: MESDET "MESO AMER TYPE 1" and ANDDET "ANDEAN TYPE 1". In both, FL-VS (photothermal days from first flower to the last main-stem leaf) = 7.0 and RHGHT (height relative to the standard internode-per-node table) = 0.00. This is how DSSAT encodes main-stem termination in Type I. Indeterminate ecotypes have FL-VS 9 to 10 and RHGHT 11 to 13. For all ecotypes TRIFL (main-stem leaf appearance rate) = 0.34 leaves per thermal day, with a calibration range of 0.32 to 0.36. EM-V1 = 3.0 thermal days. PL-EM = 2.4 (Mesoamerican) to 3.4 (Andean) thermal days. — [BNGRO048.ECO](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Data/Genotype/BNGRO048.ECO)
  - Cardinal temperatures for vegetative development (TB, TO1, TO2, TM) are 5, 27, 37, 45 °C. Early reproductive development uses 5, 24, 34, 44 °C and late reproductive development uses 6, 25, 37, 60 °C. — [BNGRO048.SPE](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Data/Genotype/BNGRO048.SPE)
  - Cultivar coefficients for Type I generic lines:
    - Andean Habit 1: EM-FL 22.6 photothermal days; SLAVR 305 cm²/g; SIZLF 133 cm² (maximum full trifoliate leaf size); WTPSD 0.60 g maximum per seed; SDPDV 3.5 seeds per pod; SFDUR 15; PODUR 10.
    - Meso Amer. Hab.1: EM-FL 27.0; SLAVR 310 cm²/g; SIZLF 133 cm²; WTPSD 0.23 g; SDPDV 5.2 seeds per pod.
    - Across all cultivars: SLAVR 250 to 350 cm²/g, SIZLF 133 to 180 cm², WTPSD 0.22 to 0.66 g, SDPDV 3.0 to 5.5. Seafarer (a Michigan navy bean): WTPSD 0.22 g, SDPDV 5.0. Redkloud (kidney bean): WTPSD 0.55 g, SDPDV 3.5. — [BNGRO048.CUL](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Data/Genotype/BNGRO048.CUL)
  - Species file allocation:
    - Leaf partitioning (YLEAF) is 0.46, 0.56, 0.60, 0.55, 0.45, 0.40, 0.30 at V-stage 0, 1.8, 4, 6.5, 8.5, 9.5, 12. Stem partitioning (YSTEM) is 0.15 to 0.59 at the same stages.
    - SLAMAX 900 and SLAMIN 255 cm²/g.
    - Internode length per node (YVSHT, m) is 0.044, 0.026, 0.044, 0.10, 0.10, 0.11, 0.12, 0.12 at V-stage 0, 2.67, 4.25, 6.75, 8, 9.58, 10, 16.
    - Reference leaf area (YVREF) is 30, 60, 110, 280, 1000, 1800 cm² per plant at V-stage 0, 1.83, 2.8, 4.3, 6.8, 9.6.
    - Leaf protein at initiation (PROLFI) is 0.24. — [BNGRO048.SPE](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Data/Genotype/BNGRO048.SPE)
- **Oregon State University CROPTIME snap bush bean phenology model**:
  - Field data: Willamette Valley, Oregon, 2013 to 2015 (Corvallis and Aurora).
  - Method: single-sine degree-days, thresholds 40 °F (4.44 °C) and 90 °F (32.2 °C).
  - 'Provider' (4 data sets): 608 DD to first trifoliate, 1094 DD to first open flowers, 1681 DD to harvest (CV 8.45%); 60.5 days to harvest.
  - '5630' (9 data sets): 601, 1148, 1630 DD; 58 days.
  - 'Sahara' (4 data sets): 526, 1248, 1805 DD; 63.5 days.
  - Harvest means pods 3.5 to 4.0 inches long.
  - Licence is not stated. — [OSU CROPTIME bean models](https://instar.biossys.oregonstate.edu/CROPTIME/bean_varieties_models.pdf)
- **Harvard Dataverse, CIAT: "Replication Data for: Genomic prediction of agronomic traits in common bean under environmental stress"** (Keller et al.), https://doi.org/10.7910/DVN/XCD67U.
  - Contents: 481 Andean elite lines; 12 trials from 2013 to 2018 in Palmira and Darien, Colombia; irrigated, drought and low-phosphorus treatments.
  - Variables: days to flowering, days to physiological maturity, 100-seed weight, yield (BLUEs), meteorological data, and a SNP VCF.
  - It has no organ time series, but it is good for drought response ranges and the spread of phenology and seed size.
  - Licence is not stated on the index page I saw; CIAT Dataverse items are usually CC0 or CC-BY (unverified). — [ICAR index of the Dataverse record](https://krishi.icar.gov.in/ohs-2.3.1/index.php/record/view/833544)

### Inferences
- **Phyllochron (inference):** in CROPGRO a "thermal day" is one day at the optimum temperature, which is TO1 − TB = 27 − 5 = 22 °C·day. TRIFL = 0.34 leaves per thermal day therefore means about 22 / 0.34 ≈ **65 °C·day per main-stem node (base 5 °C)**, with a calibration range of 61 to 69. Re-expressed on our base of 8 °C it is about 19 / 0.34 ≈ 56 °C·day (approximate, because the bases differ). This matches the plan's guess of 50 to 70 °C·day.
- **Main-stem node count (inference):** the YVREF and YLEAF tables run to V-stage 9 to 12. With FL-VS = 7 photothermal days for Type I (about 2.4 more leaves after first flower at 0.34 per day), a Type I main stem ends with roughly 6 to 9 trifoliate nodes. Our 6 to 8 is consistent.
- **Unit conversion (inference):** the CROPTIME values are almost certainly in °F-days (uspest.org works in °F). Converted to °C-days on a 4.4 °C base, 'Provider' reaches first flower at about 608 °C·day and snap harvest at about 934 °C·day. That is a direct temperate snap-bean target for the plan: flowering at about 65% of sowing-to-harvest thermal time, and snap harvest comes long before dry-seed maturity.
- The DSSAT internode table (YVSHT, about 0.03 to 0.12 m per node) is for the indeterminate standard. For Type I, RHGHT = 0 suppresses height growth after termination in DSSAT. It still gives a usable per-node internode profile: short internodes near the base, about 4 to 5 cm, then 10 to 12 cm.
- Licence (unverified): the DSSAT CSM source repository is distributed under DSSAT's open licence (I believe BSD-3-style), but I could not fetch LICENSE.md; it returned 404 at the path I tried. Verify before shipping parameter values.

### Gaps
- I could not list or fetch the DSSAT Drybean experiment files (*.BNX/*.BNT/*.BNA with LAI, organ mass and V-stage time series) because GitHub API access is blocked here. The next step is to request repository access (add_repo for DSSAT/dssat-csm-data) or browse https://github.com/DSSAT/dssat-csm-data/tree/develop/Drybean.
- I found no public phenotyping-platform (LemnaTec) bean time-series dataset with a confirmed URL in this pass.
- BeanCAP field data were not checked.

## 2. Botany corrections for the plan (§3.4, §3.10, §3.11, §3.16)

### Takeaway
Most of the plan's botany holds: epigeal germination, opposite unifoliates, alternate trifoliates and a Type I terminal raceme. The 137.5° phyllotaxis claim is **unverified and probably wrong for bean**; I found no bean-specific source either way (see Gaps). Pod set is too optimistic: bean sheds **45 to 80%** of flowers and young pods. Racemes carry **3 to 8 flowers** (occasionally more). Mature trifoliate size in DSSAT, **133 to 180 cm²**, is above the plan's 50 to 100 cm².

### Cited Findings
- CIAT growth habit definitions:
  - **Type I** (determinate): reproductive terminal bud, erect or prostrate stems, no terminal guide, pods spread along the stem.
  - **Type II**: vegetative terminal bud, erect, short guide or none.
  - **Type III**: prostrate, little climbing, pods concentrated at the base.
  - **Type IV**: strong climber, long guide.
  
  These are adapted from Singh 1982 (Annu. Rept. Bean Improv. Coop. 25:92-94) and Hall 1991 (Compendium of Bean Diseases). The development stages are V2 = primary leaves, V3/V4 = trifoliates, R5 = raceme. — [UPRM Bean Improvement Center, Growth Habit](https://www.bic.uprm.edu/wp-content/uploads/2018/04/GrowthHabit.pdf)
- Leaves are alternate, pinnately trifoliolate, with pulvini on the petiolules. The terminal leaflet's stalk is longer than the laterals'. Leaflets are 4 to 16 cm long and 2.5 to 11 cm broad. Stipules are about 4 mm and stipels 1 to 3 mm. Inflorescences are **axillary racemes of 3 to 8 flowers (occasionally more)**, rarely longer than the subtending petiole. Pods are 10 to 15 cm long and 1 to 1.5 cm broad, with **4 to 10 seeds**. Seeds are 0.9 to 2 cm long. This is compiled from Flora of China, CIAT 1986 *Morphology of the Common Bean Plant* and Gentry 1969. — [U. Michigan Climbers: Phaseolus vulgaris (Santanna & Burnham)](https://climbers.lsa.umich.edu/wp-content/uploads/2013/06/PhasvulgFABAFINAL.pdf)
- Bush types are 20 to 60 cm tall. Pods are 8 to 20 cm long with 4 to 8 seeds. Snap pods are harvested about **8 to 10 days after flowering**, while the seeds are small. — [Wikipedia: Phaseolus vulgaris](https://en.wikipedia.org/wiki/Phaseolus_vulgaris) (secondary source)
- **Flower and pod abscission:** across seven cultivars, **45 to 80% of flowers and immature pods were shed**. The earliest-opened flowers were most likely to make mature pods. Removing opened flowers for up to 15 days from the start of anthesis only slightly reduced seed yield, and sometimes increased it, which points to compensation. — [Binnie & Clifford 1981, J. Agric. Sci. 97:397-402](https://www.cambridge.org/core/journals/journal-of-agricultural-science/article/flower-and-pod-production-in-phaseolus-vulgaris/69D11A53ED2B8241E73FBCDA5BB4B8DC)
- Seed-size classes from Singh et al. 1991, as cited by Lei et al. 2020:
  - Small, under 25 g per 100 seeds: Mesoamerican.
  - Medium, 25 to 40 g per 100 seeds.
  - Large, over 40 g per 100 seeds: Andean.
  - Observed subgroup means across two environments: Mesoamerican 26 to 29 g, Andean 45 to 50 g, admixed 39 to 41 g per 100 seeds. — [Lei et al. 2020, Front. Genet. 11:698](https://www.frontiersin.org/journals/genetics/articles/10.3389/fgene.2020.00698/full)
- DSSAT maximum seed weight: Mesoamerican Type I 0.23 g; Andean Type I 0.60 g; range 0.22 to 0.66 g. — [BNGRO048.CUL](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Data/Genotype/BNGRO048.CUL)
- On 137.5°: the golden angle belongs to **spiral** phyllotaxis. Distichous and decussate arrangements are distinct patterns with 180° and 90° divergence. — [Kuhlemeier 2017, Curr. Biol.](https://consensus.app/papers/details/955be0798265544e8fd44d203afd2b84/?utm_source=claude_desktop)

### Inferences
- **Pod set (inference):** a 45 to 80% shed rate means a per-flower pod-set probability of about **0.2 to 0.55 under normal conditions**, before any heat or drought penalty. Early flowers on proximal racemes should be favoured, for example by a carbon-priority rule. The plan's rule (reserve-driven set plus heat and drought abortion) is the right mechanism. Its baseline needs calibrating so that 10 to 20 pods per plant come from many more flowers. With about 5 flowers per raceme at a 0.3 to 0.4 set rate, that means roughly 8 to 15 productive racemes per plant, branches included.
- **Phyllotaxis (inference):** in the field, the trifoliates of common bean look two-ranked or nearly so, and opposite primary leaves are rotated about 90° from the cotyledons. I could not confirm a published divergence angle. Make the angle a species parameter: test 180° (distichous) against a spiral angle, and settle it with CIAT 1986 (Debouck & Hidalgo) or photographs before fixing it. Whichever is right, **137.5° should not be stated as "botany"** without a source.
- **Seeds per pod:** keep 3 to 6 for bush dry beans (DSSAT SDPDV 3.0 to 5.5), but allow up to 6 to 8 ovules per pod for snap types. A 0.3 to 0.5 g seed suits Andean and kidney-type or snap seed. Mesoamerican black or navy seed is about 0.2 to 0.25 g. A species file should allow 0.2 to 0.6 g.
- **Leaf area per trifoliate:** DSSAT SIZLF of 133 to 180 cm² is the *maximum* full trifoliate. The plant average of 0.12 m² over 20 leaves (Kumar 2025 in plan §8.2) is about 60 cm². Our 50 to 100 cm² is a fair *mean* across nodes, but the potential size at the best nodes should reach about 130 to 180 cm².
- **SLA:** DSSAT SLAVR of 250 to 350 cm²/g under standard conditions sits slightly above the plan's 200 to 300 cm²/g. Widen the range to 200 to 350.
- **Snap bean harvest:** a garden player harvests snap pods about 8 to 10 days after flowering, well before seed fill. The game needs a "green pod harvest" endpoint, and the 'Provider' CROPTIME numbers above fit it.

### Gaps
- I found no source giving a quantitative leaf divergence angle for P. vulgaris. Source to fetch: Debouck & Hidalgo 1986, *Morphology of the common bean plant* (CIAT, on CGSpace via https://alliancebioversityciat.org/publications-data/morphology-common-bean-plant-phaseolus-vulgaris).
- Petiole lengths, internode lengths by node for a specific bush cultivar, and the number of racemes per plant were not found as tabulated numbers. The DSSAT YVSHT table is the only per-node length profile found.
- Leaf lifespan and leaf expansion duration in °C·day were not found in this pass.

## 3. Shade avoidance, paraheliotropism, drought, heat and density responses

### Takeaway
Bean shows a strong, well-documented far-red internode response, mostly in seedlings under end-of-day far-red treatment: stem elongation 2.9 to about 3 times controls, with no change in leaf initiation rate. In bush beans, far-red lengthens the stem only through internode length, not through node number. That supports the plan's encoding (internode multiplier, no change in phyllochron) and suggests a maximum multiplier of 1.5 is conservative.

### Cited Findings
- **Pinto bean seedlings, Downs et al. 1957:** low-energy far-red from incandescent light at the end of the day made internodes elongate to about **3 or more times** normal length. The effect was fully reversible with red. Bush and pole varieties responded alike. — [Downs et al. 1957, Bot. Gaz.](https://consensus.app/papers/details/3be7f34a65c95a8c8b4cecfbe2e67819/?utm_source=claude_desktop)
- **Pinto bean, Nakata & Lockhart 1966:** daily supplementary far-red after 8 h of daylight for 12 days gave stem elongation **about 2.9 times** that under red. Cell division and cell elongation were promoted equally. There was **virtually no difference in leaf initiation rate or internode growth duration**. — [Nakata et al. 1966, Am. J. Bot.](https://consensus.app/papers/details/f90f4423a6315d32a31123307f124770/?utm_source=claude_desktop)
- **Bush cv. 'Celina' vs pole cv. 'Kentucky Wonder', Hanyu et al. 1996:** supplementary far-red that lowered Pfr/P increased stem length, stem dry weight and the stem share of plant dry mass. The stem-length increase was large while Pfr/P stayed above 0.77 and small below 0.77. **In the bush type, stems lengthened only through internode length; in the pole type, node number also rose.** — [Hanyu et al. 1996, Environ. Control Biol.](https://consensus.app/papers/details/d735d28f599753f6a8ab762e4a239ef4/?utm_source=claude_desktop)
- **Phaseolus and Glycine, Buck et al. 1985:** internode elongation responds both to lower Pfr/P, acting on the internode itself, and to the irradiance and length of the day extension, acting through the leaf. The response to end-of-day far-red diminished in successive internodes. — [Buck et al. 1985, Plant Cell Physiol.](https://consensus.app/papers/details/f2ddffdeb6d350b298515cf86988ccac/?utm_source=claude_desktop)
- **Paraheliotropism sources (not fetched; PMC returned a captcha):**
  - Yu & Berg, "Control of paraheliotropism in two Phaseolus species", Plant Physiol. (PMC159699). — [scholarworks.uni.edu record](https://scholarworks.uni.edu/facpub/4373)
  - Article: https://pmc.ncbi.nlm.nih.gov/articles/PMC159699

### Inferences
- **WP12 shade number:** use Hanyu 1996 (bush bean, field-like Pfr/P levels) as the single study, with Nakata 1966 or Downs 1957 as the upper bound. A cap of 1.5 to 2 times on internode length under natural canopy shade is plausible. The roughly 3-fold values come from seedlings under extreme end-of-day far-red. Keep the phyllochron unchanged under shade, as Nakata 1966 supports.
- **Pfr/P threshold (inference, needs mapping):** Hanyu's sensitive zone at Pfr/P above 0.77 suggests the strongest response is at mild shade. Turning that into an R:FR threshold requires the Pfr/P–R:FR hyperbola from the paper; the plan's R:FR < 0.7 trigger cannot be checked directly.
- Drought and density data (Keller et al. Dataverse; Clavijo Michelangeli 2019, already in the plan) give response ranges for phenology and yield, not organ-level time series.

### Gaps
- No quantitative paraheliotropism data (leaflet angle against light or water potential) were extracted, because PMC was blocked.
- No new quantitative heat-abortion or drought data set beyond those already in plan §8.3 was found in this pass.
- No snap bean ('Provider', 'Blue Lake 274') organ dry-mass time series was found.
