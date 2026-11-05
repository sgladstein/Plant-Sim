# Bean (Phaseolus vulgaris) quantitative genetics for the M3 breeding bench

Tags: [F] = I read the source this session (abstract, full text, publisher page or a SciSpace full-text extract). [K, to verify] = from memory, not confirmed this session. [I] = my inference for the game.
Inputs read: docs/archive/research/notes-m2-m3/bean_genetics.md, the "Bean genetics for the breeding bench" section of docs/archive/research/m2-m3-bean-science.md, and data/species/bush_bean.ron (heritable block and genome loci; every `allele_sd` is 0.0 today).
Semantics used below (from crates/ps-species/src/lib.rs `phenotype`): `value = clamp(default + Σ effect·(a+b)/2 + dominance·|a−b|)`. `allele_sd` is in trait units.
- With one locus and effect 1, a pure line (a = b) has genetic SD = `allele_sd`.
- A heterozygous random draw (what `sample_genotype` does today) has additive SD = `allele_sd/√2`.
- With n equal loci per trait, divide by √n.

## Takeaway

1. **Heritability.**
   - Narrow-sense h² in bean, from F3-on-F2 regressions and Design II/S1-family variance components, is:
     - high for 100-seed weight (0.57-0.86)
     - moderate for days to flowering (0.29-0.75)
     - moderate for seeds per pod (about 0.57)
     - low to moderate for days to maturity (0.26-0.63)
     - low and unstable for seed yield (0.09-0.75, often about 0.2, and not different from zero in an Ethiopian drought study)
     - low for pods per plant (about 0.2-0.5)
   - Broad-sense H² on line means in a panel of 112 lines is much higher: 0.87-0.98 for yield, biomass, harvest index and phenology. That is not single-plant heritability. [F]
   - Basal root growth angle (BRGA) has field H² of 0.60-0.67 (Burridge 2016). [F]
   - Narrow-sense h² for adventitious root traits is 0.10-0.51. [F]
2. **Genotypic variation.**
   - Photosynthetic capacity differs among cultivars by roughly 10-35%:
     - net CO₂ exchange 9-35% (Izhar & Wallace 1967)
     - area-based Vcmax 12% and A 11-30% between Jamapa and Calima
   - Stomatal conductance differs more: gs 44-79% between the same two genotypes, and leaf Δ¹³C spans more than 2‰ (about 31 ppm of ci) across 10 cultivars.
   - Leaf size and seed size vary enormously across germplasm:
     - leaf length 4.5-20.7 cm
     - 100-seed weight 3.5-96 g, over 4,274 accessions
   - BRGA in one RIL population ranges from −19° to +57° from horizontal; the parents are 2° and 36°. [F]
3. **Crossing.**
   - Emasculate an unopened bud, best about 1 day before anthesis. The pistil is fertile from about 3 days before anthesis [F: Inoue & Suzuki 1959], and pollen is shed inside the keel before opening.
   - Success varies widely: 17-23% pod set (600 pollinations, greenhouse) and 18% at Thika. Usher & Redden got 0.43 F1 plants per pollination. The hooking method is reported at 70-90%.
   - Crossed pods hold fewer seeds: 2-3, against 4-6 for selfed pods.
   - Seed reaches mass maturity about 35-45 days after flowering and harvest dryness about 58 days after flowering.
   - Fresh seed from 30 days after flowering (only 48% filled) gave 90% normal seedlings when sown undried. Seeds dried before about 25-30 days after flowering die. [F]
4. **Heterosis.**
   - Mid-parent heterosis for yield is common and large: 26-124% in single-plant or small-plot tests.
   - Better-parent heterosis is rarer: none was significant in Gutierrez & Singh 1985 (13 crosses), while 20 of 47 F1s beat the better parent by 24-92% in Singh & Urrea 1995.
   - Yield drops sharply from F1 to F2.
   - Seed size shows almost no heterosis: at most 6%, with inbreeding depression at most 2%.
   - Seeds per pod shows negative heterosis.
   - Biomass shows about 44% better-parent heterosis in one Indian line × tester study; harvest index shows about −13%. [F]
5. **Mutation.**
   - Per-locus spontaneous rates for visible or morphological loci in selfing crops are about 1e-7 to 1e-6 per gamete:
     - barley: below 8.9e-7 for morphological loci
     - barley chlorophyll loci: about 6e-7
   - Per-site rate in Arabidopsis: 7e-9.
   - Polygenic mutational heritability V_M/V_E ranges from 1e-4 to 5e-2, about 1e-2 for durum wheat yield components. No bean-specific rate exists. [F]

**Recommended game defaults** [I] (for pure-line founders in a uniform pot environment; see the per-trait table in Inferences):
- **Make founders pure lines.** Use a = b at every locus. `sample_genotype` currently draws a and b independently, so founders come out heterozygous.
- **Set `allele_sd` = σG/√n_loci**, with σG = CV_G × default:
  - CV_G about 0.10 for vcmax25, jmax25 and rd25
  - 0.15-0.20 for medlyn_g1
  - 0.15 for leaf_area_potential
  - 0.10 for specific_leaf_area
  - 0.15 for internode_length_potential
  - 0.20 for seed_mass
  - 0.15 for seeds_per_pod
  - 0.10 for leaf_lifespan and nodes_before_terminal
- **Target single-plant h²** (between lines in a uniform environment):
  - seed_mass 0.7
  - flowering thermal time 0.6
  - seeds_per_pod 0.5
  - leaf size, internode and SLA about 0.5
  - BRGA 0.6
  - Vcmax 0.35
  - g1 0.4
- **Emergent traits:** yield should come out at about 0.2-0.3, pods per plant at about 0.2-0.3, and harvest index at about 0.3-0.5.
- **The sim is deterministic,** so h² will be close to 1 unless a per-plant micro-environment term is added: σE = σG·√((1−h²)/h²), keyed in ps-rng.
- **Dominance and mutation.** Use small dominance (0 to +0.25·allele_sd) on vigour traits and 0 on seed_mass. Keep mutation at 1e-3 only for anonymous QTL with small effects; use 1e-5 or less for named loci.

## Heritability table

Notation: h² = narrow-sense; H² = broad-sense; r = realized. "F3/F2" = regression of F3 on F2 (h² plus part of the dominance variance).

| Trait | Estimate | Type, method | Material, sample, environment | Source |
|---|---|---|---|---|
| 100-seed weight | 0.57±0.04 to 0.80±0.04 | F3/F2 regression; realized gain in F3 about equal to expected | 9-parent diallel, F2 and F3 bulks, 2 sites Mexico + 2 Colombia (rainfed) | White, Ochoa & Ibarra 1994, J. Agric. Sci. [F, abstract + SciSpace extract] https://scispace.com/papers/inheritance-of-seed-yield-maturity-and-seed-weight-of-common-4ka899jcfi ; DOI 10.1017/S0021859600087451 [K, to verify] |
| Seed yield | 0.09±0.18 to 0.75±0.25 | F3/F2 regression. Expected gain from top-20% selection: 1.8-8.4% (Mexico), 6.5-28.1% (Colombia). Realized gain in F3: 0.4-7.4% and 2.9-15.7%, lower than expected | as above | White et al. 1994 [F] |
| Days to maturity | 0.26-0.34 (±0.09) | F3/F2 regression; realized gain 13.4% | as above | White et al. 1994 [F] |
| Seed yield / pods m⁻² / seeds pod⁻¹ / 100-seed wt | 0.21±0.13 / 0.20±0.13 / 0.57±0.13 / 0.74±0.15 | h², Design II, F2 entry means, unbiased by G×E | 200 F2 populations from 80 Middle-American bush cultivars; 2 locations, Colombia, 1983 | Nienhuis & Singh 1988, Plant Breed. 101:155 [F, SciSpace extract] https://scispace.com/papers/genetics-of-seed-yield-and-its-components-in-common-bean-30nnehyrvc |
| Yield / pods m⁻² / seeds m⁻² / 100-seed wt / maturity | 0.43±0.19 / 0.49±0.20 / 0.63±0.21 / 0.76±0.23 / 0.55±0.22 | h² (additive > non-additive for all traits); expected gain 9.2 / 12.8 / 21.0 / 17.6 / 4.6% | Andean large-seeded F2s, CIAT Colombia | Singh, Terán & Molina 1991, Plant Breed. [F, abstract] https://scispace.com/papers/genetics-of-seed-yield-and-its-components-in-common-beans-57pzlm8b65 |
| Yield / seed wt / maturity | 0.31-0.46 / 0.75-0.86 / 0.50-0.81 | Among 45 S1 families per cycle. Realized yield gain ≥15% per cycle; expected 7.9-13.2% (C1) and 3-4% (C2) | 2 recurrent-selection populations (Middle American; Andean × Middle American), 3 sites per year, Colombia, 1992-97 | Singh, Terán & Muñoz 1999, Crop Sci. 39:391 [F, SciSpace extract] https://scispace.com/papers/two-cycles-of-recurrent-selection-for-seed-yield-in-common-fiv81m8g01 |
| Days to first flower / physiological maturity / reproductive period | 0.29-0.75 / 0.31-0.63 / 0.01-0.51 | h², F3 progeny means (Hallauer & Miranda) | 3 early × late indeterminate crosses + reciprocals, 50 F3 lines each, Isabela PR, 2 planting dates | Cerna & Beaver 1990, Crop Sci. 30:1215 [F, SciSpace extract] https://scispace.com/papers/inheritance-of-early-maturity-of-indeterminate-dry-bean-53k9l91vwp |
| Days to first flower / maturity / pod-fill period | 0.59 / 0.33 / 0.26 | h², F3/F2 regression ("standard unit") | TMO 216 × CIAT 16-1, Morogoro, Tanzania, 1991 | Nchimbi-Msolla & Mduruma 2007, Tanz. J. Agric. Sci. [F, abstract] https://scispace.com/papers/estimate-of-heritability-for-maturity-characteristics-of-an-3yeiukj8nq |
| Seed yield | not different from 0 | h², variance components; high G×E | 7 and 25 cultivars, 2-3 environments (stress and non-stress), Ethiopia | Shenkut & Brick 2003, Euphytica [F, SciSpace extract] https://scispace.com/papers/traits-associated-with-dry-edible-bean-phaseolus-vulgaris-l-3w74itwwmo |
| Days to flowering, plant height, pods plant⁻¹, seeds pod⁻¹, biomass, 100-seed wt, root dry wt | all significant and > 0.40 (exact values not extracted) | h², low G×E for biomass and pods plant⁻¹ | as above | Shenkut & Brick 2003 [F] |
| Yield / biomass / harvest index / days to maturity / days to flower / pod-fill days | 0.90 / 0.93 / 0.92 / 0.96 / 0.98 / 0.94 | H² on line means across 5 environments (not single-plant) | 112 diverse inbred lines, New York, 1986-87 | Scully, Wallace & Viands 1991, JASHS 116:127 [F, abstract] https://scispace.com/papers/heritability-and-correlation-of-biomass-growth-rates-harvest-2wqm6axisy |
| 100-seed wt / DTF / yield | 0.89 / 0.77 / 0.61 | H², germplasm panel | 257 Kenyan accessions, 4 seasons | Wanjohi 2023 (already in bean_genetics.md) [F, abstract] |
| Plant height | GCV 11.7%, PCV 19.4% (H² about 0.36 derived = GCV²/PCV²) [I] | panel | 100 Ethiopian landraces | Mesera 2022 (bean_genetics.md) [F, abstract] |
| Basal root growth angle | H² 0.67 (Chokwe, 2 yr); 0.60 (Chokwe + Umbeluzi) | H², shovelomics in the field | 30 genotypes, Mozambique 2008-09 | Burridge, Jochua, Bucksch & Lynch 2016, Field Crops Res. 192:21, DOI 10.1016/j.fcr.2016.04.008 [F, PSU PDF] https://plantscience.psu.edu/research/labs/roots/publications/pdfs/2017-legume-shovelomics |
| Basal root whorl no. / basal root no. / hypocotyl-borne root no. / basal root length | 0.86 / 0.79 / 0.56 / 0.14 | H², as above | as above | Burridge et al. 2016 [F] |
| Adventitious root no. / biomass / length / SRL | 0.26-0.37 / 0.10-0.37 / 0.10-0.35 / 0.18-0.51 | h², by field or greenhouse and high or low P | 76-84 F5:8 RILs, G2333 × G19839 | Ochoa, Blair & Lynch 2006, Crop Sci. 46:1609, DOI 10.2135/cropsci2005.12-0446 [F, PSU PDF] |
| Root traits (unspecified, incl. root angle) | 0.43-0.67 | heritability from an NC II design | 8 parents + F2, Kenya, several environments | Levene Castiano et al. 2022, Plant Breed. [F, abstract] https://scispace.com/papers/combining-ability-of-common-bean-phaseolus-vulgaris-29t2rr4k |
| Seedling root phenotypes | repeatability 0.52-0.57 | repeatability, not h² | 577 genotypes, many environments | Strock et al. 2019, Field Crops Res. 237:53, DOI 10.1016/j.fcr.2019.04.012 [F, record page] |
| Leaf photosynthetic rate | No h² number found. Izhar & Wallace (F2-F3) call the NCE difference quantitative, "few genes", with some dominance for low NCE | genetic analysis | greenhouse, 5 varieties + segregating progeny | Izhar & Wallace 1967, Crop Sci. 7:457 [F, abstract] https://scispace.com/papers/studies-of-the-physiological-basis-for-yields-differences-4cdbr0hp6x |
| SLA, leaf area ratio, leaf weight ratio | "greater genetic expression" than the other 31 growth and photosynthesis variables (no number) | variance components | 29 accessions, Colombia (Corpoica) | Ligarreto 2013 [F, abstract] https://scispace.com/papers/componentes-de-variancia-en-variables-de-crecimiento-y-8mqse3qcg1 |

## Genotypic variation table

| Trait (game parameter) | Range or difference among genotypes | Material and conditions | Source |
|---|---|---|---|
| Net CO₂ exchange (vcmax25 / jmax25 proxy) | Michelite-62 exceeds Monroe by about 9%, and Perry Marrow, Steuben and Red Kidney by 22, 35 and 31% | greenhouse, single attached leaves | Izhar & Wallace 1967 [F, SciSpace extract] |
| Vcmax, Jmax, A, gs | Area-based Vcmax: Jamapa about 12% above Calima. Light-saturated Vlmax +22%. A at 400 ppm: 17.2 vs 15.4 µmol m⁻² s⁻¹ (+11%); at 200 ppm +30%. Protein-based Jmax 252 vs 138 (+83%) and protein-based Vcmax 125 vs 168 (protein basis, not comparable to area basis). gsw at 400 ppm 0.65 vs 0.38 (+71%); +44-79% across CO₂ levels | Jamapa (Mesoamerican) vs Calima (Andean); 3-week plants, greenhouse 26/20 °C, PPFD about 1000, LI-6800 at 25 °C | Ogolla Egesa, Vallejos & Begcy 2024, Front. Plant Sci. 15:1422814, DOI 10.3389/fpls.2024.1422814 [F, full text] |
| Vcmax25 / Jmax / Rd reference | 89.3 / 159.8 / 0.88 µmol m⁻² s⁻¹ (one cultivar; the game default) | Brazil | Pimentel et al. 2011, Hoehnea 38:273, DOI 10.1590/S2236-89062011000200007 [F, abstract] |
| CER, gs, chlorophyll, stomatal density | Andean accessions lower than Mesoamerican on CER, gs, stomatal density and chlorophyll. Wild Peruvian CER > landraces. No numbers extracted | 47 wild + 47 landraces, 2 field trials, Colombia | González, Lynch, Tohme et al. 1995, Crop Sci. 35:1468 [F, abstract] |
| Max A, gmax, SLA (medlyn_g1, specific_leaf_area) | "Large differences" between cultivars in maximum A and E. A correlated with leaf N and SLA. gmax varied widely, while the sensitivity of ln g to VPD was nearly uniform across cultivars (supports a heritable g1 with a fixed VPD response shape) | 12 cultivars, greenhouse, 30 °C leaf | Comstock & Ehleringer 1993, Aust. J. Plant Physiol. 20:669 [F, abstract] |
| Leaf size vs gs (leaf_area_potential, medlyn_g1) | Cultivars from hot and dry regions have significantly smaller leaves and crowns but higher gs and E per area (leaf size and gs inversely coupled) | 12 cultivars, greenhouse | Mencuccini & Comstock 1999, Aust. J. Plant Physiol. 26:115 [F, abstract] |
| Δ¹³C and leaf conductance (medlyn_g1) | Δ range > 2‰ among cultivars, equal to > 31 µL L⁻¹ difference in ci. Conductance differences persisted across stages and correlated with Δ | 10 cultivars, field | Ehleringer 1990, Plant Physiol. 93:1422 [F, abstract] |
| A, gs, WUE in landraces | gs and WUE most variable; A and light-use efficiency least variable. Determinate types: higher gs and ci, lower A | Croatian landraces | Lazarević et al. 2018, J. Elementology [F, abstract] |
| Leaf length / width (leaf_area_potential) | 4.5-20.7 cm / 3.4-17.5 cm | 4,274 accessions, Indian gene bank, 2 years | Rana et al. 2015, Euphytica 205:441 [F, abstract + extract] |
| 100-seed weight (seed_mass) | 3.5-96.3 g (mean 29.5 g) | as above | Rana et al. 2015 [F] |
| Seeds per pod (seeds_per_pod) | 2.1-9.6 | as above | Rana et al. 2015 [F] |
| Pods per plant | 4.2-49.6 (mean 16.3) | as above | Rana et al. 2015 [F] |
| Days to 50% flowering (flowering phenology) | 27-104 d (mean 55) | as above; includes photoperiod-sensitive types | Rana et al. 2015 [F] |
| Seed-weight GCV | 12.7% (PCV 15.4%) | 100 Ethiopian landraces | Mesera 2022 [F, abstract via bean_genetics.md] |
| BRGA (basal root insertion angle) | Parents: DOR364 35.9° (low P) and 36.6° (high P); G19833 2.1° (low P) and 23.6° (high P). RILs at low P: mean 12.0°, range −18.7° to 56.7°. RILs at high P: mean 13.1°, range −9.6° to 47.6°. Degrees from horizontal; negative = upward. BRGA QTL individually explain 9.9-15.9% | 86 F5.7 RILs, growth pouches, 6 days, 25/20 °C | Liao et al. 2004, Funct. Plant Biol. 31:959, DOI 10.1071/FP03255 [F, PSU PDF] |
| BRGA plasticity | Of 16 genotypes, 6 became shallower in low P, 1 steeper and 9 unchanged within 5 days | pouches + buffered sand | Bonser, Lynch & Snapp 1996, New Phytol. 132:281 [F, abstract + PSU summary; no degree values seen] |
| BRGA, field | Not obtained. Ho et al. 2005 (Funct. Plant Biol. 32:737) reports field BRGA of contrasting RILs [K, to verify] | | |
| Basal root whorl number | Major QTL Rwn7.1 R² 0.25 (DOR364 × G19833) and 0.59 (G2333 × G19839) | 2 RIL populations | Miguel, Brown, Blair & Lynch 2009, ASA poster [F] |

## Crossing table

| Item | Value | Source |
|---|---|---|
| Flower biology | Pollen is shed onto the stigma inside the closed keel before the flower opens (cleistogamous selfing) | [K, to verify]; consistent with Hoc & Amela García 1999 (protogynous: stigma receptive in the bud before the anthers dehisce, in wild var. aborigineus) [F] Rev. Biol. Trop. 47:59 |
| Anthesis duration | One day. Opens about 07:00; anthesis lasts 9-13 h, flower closes or withers the next day | Hoc & Amela García 1999 [F] (wild var.) |
| Pistil receptivity | Fertile from 3 days before anthesis. Set rises toward the day before anthesis | Inoue & Suzuki 1959, J. Jpn. Soc. Hortic. Sci. [F, abstract] (cv. Masterpiece) |
| Emasculation stage | Unopened bud, in the forenoon. Fold back the standard, remove the keel in pieces following the spiral (or the style breaks), remove the stamens; pollinate at once with pollen from a freshly opened flower, or hook its pollen-laden stigma (Buishand 1956) | TNAU Agritech, "Emasculation and pollination techniques in vegetables" [F] https://agritech.tnau.ac.in/crop_improvement/crop_imprv_emasculation_vegetables.html |
| Emasculation timing (hours) | About 1 day (about 12-24 h) before anthesis; the bud has a coloured corolla tip just past the calyx | [K, to verify]; no hour figure found for bean |
| Success, emasculated | 143 F1 hybrids from 340 pollinations (0.43 per pollination), 3 selfs | Usher & Redden 1986, Qld J. Agric. Anim. Sci. 43:115 [F, earlier session] |
| Success, not emasculated | 0.62 hybrids per pollination, but 51 selfs; about 50% pod set | Usher & Redden 1986 [F] |
| Success, hooking | 70-90% pod set, 4-6 seeds per pod (Buishand 1956, Bliss 1980, as cited) | Usher & Redden 1986 [F, citing] |
| Success, greenhouse snap bean | About 17% (Sept) and 23% (June) pod set from 600 emasculated pollinations; 15-28% by time of day (10:00, 12:00, 15:00; differences not significant). About 2.1-3.2 seeds per crossed pod. 17-23 °C, RH 47-77% | Carpentieri-Pípolo, Vizoni & Giroto 2001, Acta Sci. 23:1191 [F, full text] |
| Success, Kenya field | 18% crossing success at Thika vs 55% natural pod set. Humidity, temperature and technique all matter | van Rheenen 1978, E. Afr. Agric. For. J. 44:126, DOI 10.1080/00128325.1978.11662997 [F, abstract] |
| Alternative (heat-induced male sterility) | Plants given 35 °C for 72 h; flowers opening 8-12 days later are self-sterile. Crossing them without emasculation gives 75-100% pod set | JIRCAS research highlight 2000 [F] https://www.jircas.go.jp/en/publication/research_results/2000_28 |
| Soybean comparison | Emasculated 1-2 days before opening: 64% hybrid pods (23/36) | Vollmann et al. 1992, Die Bodenkultur 43 [F] |
| Pollination to physiological (mass) maturity | 43 days after flowering (seed moisture 52%), cv. Teebus, field | van de Venter et al. 1996, S. Afr. J. Plant Soil 13:47 [F, abstract + extract] |
| | 35 days after anthesis (2 cultivars, Brazil) | Ayres et al. 2021 [F, extract] |
| | 79-92 days after anthesis (Embrapa lines) | Bolina 2012 thesis [F, abstract]; this is an outlier and probably counts from sowing or first flower [I] |
| Pollination to harvest-dry seed | 58 days after flowering (12.7% moisture). Maximum quality and desiccation tolerance at 48 days after flowering (34% moisture) | van de Venter et al. 1996 [F] |
| Earliest plantable seed (immature) | Fresh seed taken at 30 days after flowering, 48% of final dry mass, gave 90.5% normal seedlings when sown fresh; only 3% if dried rapidly | van de Venter et al. 1996 [F] |
| | Seeds excised at 22 days do not germinate even after drying; at 32 days they germinate after a drying treatment | Dasgupta & Bewley 1982, Plant Physiol. 70:1224 [F, abstract] |
| | Germinability starts before the seed is half filled. Maximum normal germination is reached 2-23 days after mass maturity, and maximum desiccation tolerance 6-23 days after (30/24 and 27/21 °C) | Sanhewe & Ellis 1996, J. Exp. Bot. 47:949 [F, abstract + extract] |
| Natural outcrossing | <0.74% (Asturias); weighted mean 6.9% (California, about 120k seedlings, family range 0-78%); 19-67% (Irvine 1985) | already in bean_genetics.md [F] |
| | Autonomous self (bagged): 50% fruit set; open-pollinated: 89% (wild var. aborigineus) | Hoc & Amela García 1999 [F] |
| | Indehiscent-anther mutant: 94% outcrossing in the field | Wyatt 1984, JASHS 109 [F, abstract] |

## Heterosis and inbreeding

| Finding | Material | Source |
|---|---|---|
| Mid-parent yield heterosis of 27.8-47.3% in 6 of 13 crosses; no F1 significantly above the better parent; no heterosis for pods per plant; significant heterosis for seeds per pod all negative; one heterotic cross each showed inbreeding depression for yield and for seed weight | 10 bush lines, 13 crosses | Gutierrez & Singh 1985, Can. J. Plant Sci. 65:243 [F, abstract] |
| Mid-parent yield heterosis of 26.4-123.8% (31 of 47 crosses significant); 20 F1s beat the better parent by 23.7-91.8%; "sharp reduction in yield from F1 to F2"; more heterosis in inter-race crosses | 20 parents from 4 races, 47 F1/F2, Palmira and Popayán, Colombia | Singh & Urrea 1995, Euphytica 81:131 [F, SciSpace extract] |
| Better-parent heterosis for yield per plant from −31% to +114%; 12 of 28 crosses significantly positive | 8 × 8 diallel, Ethiopia | Ali, Tiruneh & Zelleke 2013, J. Appl. Biosci. [F, extract] |
| Best mid-parent heterosis for yield 15-23%; none negative | 6 cultivars, 15 F1s, greenhouse, Maringá, Brazil | Barelli et al. 2008, Acta Sci. Agron. [F, extract] |
| Mid-parent heterosis for yield up to 72-77%, pods per plant up to 66%, seed weight 6-16%; all yield heterosis positive | 7 parents, 21 F1s, greenhouse pots, Mato Grosso | da Silva et al. 2020 [F, extract] |
| Heterosis up to 94.3% and heterobeltiosis up to 80.8% for yield | 25 F1s (NC II), rainfed, Durango, Mexico | Robledo-Torres et al. 2022, Rev. Fitotec. Mex. [F, extract] |
| Large heterosis for yield and pods/seeds per plant; all F1s above mid-parent; 5 of 12 above the higher parent | 2 pinto × 2 kidney diallel, field | Foolad & Bassiri 1983, J. Agric. Sci. [F, extract] |
| Average better-parent heterosis +43.7% for biomass and −13.0% for harvest index | 21 F1s, line × tester, India | Gandhi et al. 2024, J. Food Legumes [F, abstract] |
| Seed-size heterosis ≤ 6%, inbreeding depression ≤ 2%; ≥ 10 additive factors; F2 obtained by selfing without hand work | wild × cultivated cross, greenhouse | Motto, Soressi & Salamini 1978, Genetica [F, extract] |
| Mid-parent heterosis for yield, pods/plant and height in 1 of 2 crosses with no F2 inbreeding effect; attributed to epistasis and some overdominance | 2 large-seeded crosses, 6 generations, Awassa | Kimani & Asfaw 2005 [F, abstract] |
| Andean × Mesoamerican RILs show outbreeding depression and strong epistasis | RIL population, 2 environments | Johnson & Gepts 2002, Euphytica [F, abstract] |
| Seed yield in F1 above parents and F2 shows dominance, but gene action is mostly additive | 4 populations, F1 and F2 | Abreu et al. 2002, Genet. Mol. Biol. [F, extract] |

Summary [I]:
- **Yield heterosis.** Typical mid-parent heterosis for yield or biomass in a space-planted F1 is about +20 to +50% (median about 30%). Wide crosses reach 100% or more. Better-parent heterosis is 0-25% on average.
- **F1 to F2.** With pure dominance, the F2 should keep about half the F1's excess over the mid-parent, a drop of about 10-25%. Single-plant, competition-free F1s inflate the figures.
- **Inbreeding depression** in a selfing pure line is effectively zero, because the species is already homozygous (Blair 2018: 0.5% heterozygous calls in F11 RILs; see bean_genetics.md).
- **Seed traits.** Seed size shows almost no heterosis. Seeds per pod shows negative heterosis.

## Mutation

| Finding | Source |
|---|---|
| Barley isogenic line: no mutants in 3,386,850 allele replications at 4 morphological loci, so the rate is < 8.85e-7 per locus per gamete per generation (95%). Enzyme loci < 3.56e-6 | Kahler, Allard & Miller 1984, Genetics 106:729 [F, abstract] https://scispace.com/papers/mutation-rates-for-enzyme-and-morphological-loci-in-barley-2ojlt2542l |
| Barley spontaneous chlorophyll mutants: frequency 1.6e-4 among 1.43 million seedlings, about 6.3e-4 per diploid genome per generation, about 6e-7 per locus per haploid genome | Jørgensen & Jensen, Hereditas [F, abstract] |
| Maize R locus: seed-colour mutation up to 1.82e-3 per gamete in one inbred stock ("Cornell"); zero in 10-20k gametes in others. Rates vary widely among alleles and stocks | Stadler 1948, Am. Nat. [F, abstract] |
| Arabidopsis: 7e-9 base substitutions per site per generation (MA lines, 30 generations) | Ossowski et al. 2010, Science 327:92 [F, abstract]; DOI 10.1126/science.1180677 [K, to verify] |
| Arabidopsis: fitness declined by < 1% per generation over 10 generations in 1,000 MA lines | Schultz, Lynch & Willis 1999, PNAS [F, abstract] |
| V_M/V_E (mutational heritability) ranges from about 1e-4 to 5e-2 across organisms, including crops | Lynch 1988, Genet. Res. 51:137 [F, abstract] |
| Durum wheat (MA, 7 generations, single-seed descent): V_M/V_E about 0.01 for heading date and yield components; U = 0.05 per haploid genome | Bataillon et al. 2001 [F, abstract] |
| Life-history traits have a median CV_M more than 6 times that of morphological traits | Houle, Morikawa & Lynch 1996, Genetics 143:1467 [F, abstract] |
| No spontaneous-mutation estimate for Phaseolus was found | — |

## Inferences

1. **Proposed loci per trait.** Pure-line founders. σG = CV_G·default; allele_sd = σG/√n when all n loci have effect 1 and founders have a = b. Each row cites the evidence behind its CV_G.

   | Parameter | Default | CV_G | σG | n loci | allele_sd per locus | Target h² | Dominance | Evidence |
   |---|---|---|---|---|---|---|---|---|
   | vcmax25 | 89.3 | 0.10 | 9 | 2 | 6.3 | 0.35 | 0 | NCE 9-35%, Vcmax_area 12% |
   | jmax25 | 159.8 | 0.10 | 16 | 2 (1 shared with vcmax25 [I]) | 11 | 0.35 | 0 | keeps J/V near 1.8 |
   | rd25 | 0.88 | 0.10 | 0.09 | 1 | 0.09 | 0.3 | 0 | — |
   | medlyn_g1 | 4.0 | 0.18 | 0.7 | 2 | 0.5 | 0.4 | 0 | Δ¹³C > 2‰, gs ±40-80% |
   | leaf_area_potential | 133 cm² | 0.15 | 20 | 2 | 14 | 0.5 | +0.2·sd | Rana leaf length; Mencuccini |
   | specific_leaf_area | 305 cm²/g | 0.10 | 30 | 1-2 | 21-30 | 0.5 | 0 | Ligarreto; Comstock |
   | internode_length_potential | 4.5 cm | 0.15 | 0.7 | 2 | 0.5 | 0.5 | +0.2·sd | height GCV 11.7% |
   | nodes_before_terminal | 7 | 0.10 | 0.7 | 1 | 0.7 | 0.5 | 0 | — |
   | leaf_lifespan | 600 Cd | 0.10 | 60 | 1 | 60 | 0.4 | 0 | no data |
   | seed_mass | 0.3 g | 0.20 | 0.06 | 3 (1 near P on Pv07) | 0.035 | 0.7 | 0 | h² 0.57-0.86; heterosis ≤ 6% |
   | seeds_per_pod (ovules) | 5 | 0.15 | 0.75 | 2 | 0.5 | 0.5 | ≤ 0 | h² 0.57; negative heterosis |
   | flowers_per_raceme | 6 | 0.15 | 0.9 | 1 | 0.9 | 0.4 | +0.2·sd | — |
   | root_elongation_scale | 1.0 | 0.10 | 0.1 | 1 | 0.1 | 0.4 | 0 | adventitious-root h² 0.1-0.5 |

   - **Seed-size range.** A single bench population (two parents) sees far less than the 3.5-96 g world range. For a breeding challenge, give starter lines distinct fixed values instead of only drawing from N(0, sd), e.g. a small-seeded line at about 0.2 g and a large-seeded line at about 0.5 g (within min 0.15 and max 0.8).
   - **Founder starting kit.** Six to ten named pure lines whose genotypes are drawn once from these SDs (or hand-set to contrasting extremes) give F2 transgressive segregation without any new mechanism.

2. **Environmental variance is needed.** The sim is deterministic. Identical pots give V_E ≈ 0, so realized h² ≈ 1 and an F1 has zero variance; WP28's "F2 > F1" passes trivially and the selection challenge becomes too easy.
   - **Fix:** a per-plant "micro-environment" multiplier on each heritable parameter, drawn from ps-rng keyed (seed, plant, 0, "microenv", trait) with σE = σG·√((1−h²)/h²). For example, seed_mass at h² 0.7 gives σE ≈ 0.65·σG.
   - **Real noise sources** the model may already have: sowing-seed size (maternal reserves), position on the bench, light gradients.
   - **Rule:** measure the realized h² of the bench by parent-offspring regression in a test, and tune σE until it matches the targets.

3. **Seed size is maternally determined.** The size of the seeds a plant bears is set mainly by the mother plant's genotype (pod and seed-coat tissue, assimilate supply), not the embryo's [K, to verify]. In the game, read seed_mass for seeds forming on plant X from X's phenotype. A plant's own phenotype then shows up in its seeds, the F1's seed size appears on the F1 plant, and an F2's segregation appears on F2 plants. This matches classical bean seed-size genetics (Motto 1978 measured F2 seed traits on F1 plants) [I].

4. **Heterosis without a heterosis table.** Positive dominance on vigour components (leaf area, internode length, flowers per raceme) of about 0.2·allele_sd per heterozygous locus, combined with physiology, should give an F1 vigour gain of about 10-30% when the parents differ at several loci. Set dominance ≤ 0 on seeds_per_pod and 0 on seed_mass to match the data. Then check emergent mid-parent heterosis for biomass and yield against the target of +10 to +50% and the F1-to-F2 loss of about half of that [I].

5. **Crossing mechanics.**
   - Hand cross on a bud that will open the next day (window: 1-3 days before anthesis).
   - Pod-set probability 0.4 for an emasculated cross by a novice; 0.25 under a hot or dry greenhouse penalty; 0.7 with "hooking" skill or an upgrade. Use the existing pod_set temperature response multiplicatively.
   - Seeds per crossed pod: 0.6 × the normal ovule set, giving about 2-3.
   - Accidental self when emasculation is skipped: about 0.5.
   - Natural outcrossing indoors: 0.
   - Seed is plantable fresh from about 300-350 Cd after pollination (about 30 days at 20 °C mean, base 8 °C ≈ 360 Cd) and as dry seed at seed_fill end + drying (about 45-58 days).
   - Drying an immature seed early (< about 25 days) kills it. That gives a "speed breeding" choice: sow green seed early and accept a smaller seed with less reserve [I].
   - The current seed_fill_delay (70 Cd) + seed_fill (330 Cd) = 400 Cd after pod start ≈ 33 days at 12 Cd per day, consistent with 35-45 days to mass maturity [I].

6. **Mutation check.** With μ = 1e-3 per locus per gamete and mutant effect SD c·allele_sd (c = 0.1-0.2), the variance added per generation per locus in a selfed line is about μ·c²·allele_sd². For 2 loci and c = 0.15 that is about 4.5e-5·allele_sd², so V_M/V_E is about 1e-5 to 1e-4. That sits at the bottom of Lynch's 1e-4 to 5e-2 range and below the durum value of about 1e-2.
   - Over a 5-generation game it is invisible, which is fine.
   - To make mutation visible in long games, raise c to 0.5 (V_M/V_E about 1e-3) rather than raising μ [I].
   - The per-locus real rate for visible loci (≤ 1e-6) supports 1e-5 or less for named loci, as decision 0023 already sets.

7. **Root angle.** BRGA is not in the heritable list today. Basal roots carry `insertion_angle` 80° from vertical (10° from horizontal) as a constant.
   - Propose a heritable `basal_insertion_angle` with default 70° from vertical (20° from horizontal; the RIL mean was 12-13°), σG about 10° (CV about 0.15 on the 0-90° scale), range 30-100° from vertical, h² 0.6, 2-3 loci.
   - Phosphorus plasticity (6 of 16 genotypes go shallower in low P) is G×E that emerges only if a P model exists. Leave it out [I].

8. **Phenology.** Flowering thermal time is not heritable in bush_bean.ron. Days to flowering is one of the most heritable and most-selected bean traits (h² 0.3-0.75; H² up to 0.98), and earliness is a classic breeding goal.
   - Make `phenology.flowering` heritable with CV_G 0.08 (about ±35 Cd, or ±3 days) and h² 0.6, plus the Ppd major locus already planned.
   - Days to maturity should follow, with lower h² (0.3-0.6) [I].

9. **Validation targets for WP28-32 tests** [I]. Run on a simulated F2 from two contrasting founders, with σE tuned:
   - (a) parent-offspring h² for seed_mass between 0.55 and 0.85
   - (b) for days to flowering, between 0.3 and 0.75
   - (c) emergent yield h² below 0.5
   - (d) seed mass and seeds per pod negatively correlated (r about −0.3 to −0.65)
   - (e) divergent selection on seed_mass, top 2 of 10 for 5 generations, separates the lines by > 2 pooled SD; the realized gain per generation of about 10-20% of the mean matches the expected 11.8-17.6% (Nienhuis & Singh 1988; Singh 1991)
   - (f) mid-parent heterosis for biomass in F1 between 0 and +50%

## Gaps

- **Single-plant narrow-sense h² in a greenhouse or pot environment.** Not found. All h² values come from field plots, and greenhouse pots would give higher values.
- **Vcmax25 and Jmax25 across a germplasm panel.** No genotypic CV was found for bean. Only pairwise comparisons were read (Izhar & Wallace 1967; Egesa 2024). The Portuguese 158-accession panel (Leitão 2021, Hortic. Res.) and Lazarević 2018 have A and gs data, but no numbers were extracted. CV_G 0.10 is an inference.
- **Bean SLA and individual leaf area across genotypes, in cm² g⁻¹ and cm².** Ligarreto 2013 and Trindade 2010 had no numbers in the accessible text.
- **BRGA across germplasm (not RILs), and Ho et al. 2005 field values.** Not read. Bonser 1996 angle values in degrees not seen.
- **Exact hours before anthesis for emasculation, and pollen-shed timing relative to opening.** No primary bean figure; the "evening before" timing is [K].
- **Inbreeding depression as numeric F1-to-F2 percentages.** None found for bean beyond "sharp reduction" (Singh & Urrea 1995) and "none" (Kimani & Asfaw 2005).
- **Spontaneous mutation rate in Phaseolus.** No estimate found.
- **DOIs.** Not checked for White 1994, Gutierrez & Singh 1985, Nienhuis & Singh 1988, Singh 1991/1995/1999 and Cerna & Beaver 1990. The SciSpace links above resolve to the records.
- **Seed size maternal control.** Asserted from memory [K]; confirm with a bean reciprocal-cross source before relying on it.

### Additional sources found but not read
- Bliss 1980, "Common bean", in Fehr & Hadley (eds.), *Hybridization of Crop Plants*, ASA, the standard crossing reference [K, to verify].
- Andrade-Aguilar, "El método de inserción", Rev. Fitotec. Mex., a pollination technique [F, title + abstract only].
