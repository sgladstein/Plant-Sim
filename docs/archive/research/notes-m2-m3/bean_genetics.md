# Common bean (Phaseolus vulgaris) genetics for the M3 breeding mechanic

Tags: [F] read this session (abstract, full text or tool extract); [K, to verify] prior knowledge, not confirmed this session; [I] inference for the game design.
Design context: design-plan-v3 §3.12 (diploid, 20-40 loci, additive + dominance, unlinked, mutation 1e-3 per locus per gamete), §3.11 (selfing default, hand-pollination records pollen parent), §7 WP28-32 (F2 variance > F1 variance; selected lines measurably different after 5 generations), §6.4 challenge file.

## 1. Major genes usable as named loci

### Takeaway
Bean has a well-documented set of Mendelian loci with strong visual phenotypes (seed-coat colour and pattern, flower colour, growth habit, pod shattering, pod strings) that make good named, collectible loci. Several of them sit on the same chromosomes: fin and Ppd on Pv01, St and I on Pv02, and P with phaseolin and seed size on Pv07. That gives a real case for limited linkage.

### Cited Findings
- **fin / PvTFL1y (growth habit, determinacy).** PvTFL1y (Phvul.001G189200) is on the lower arm of Pv01. Indeterminate (Fin, type IV) is dominant over determinate (fin, type I). [F] — [González et al. 2016, Front. Plant Sci. 7:1940](https://www.frontiersin.org/articles/10.3389/fpls.2016.01940/full). The causal gene was first identified in [Repinski et al. 2012, TAG](https://link.springer.com/doi/10.1007/s00122-012-1808-8) [K, to verify: title and author list not read this session].
- **Ppd (photoperiod).** The major photoperiod-sensitivity locus Ppd is PHYTOCHROME A3 (PHYA3), an orthologue of soybean E3, on chromosome 1. Loss-of-function alleles occur in both the Andean and Mesoamerican gene pools. Fin is linked to Ppd on chromosome 1 and may also add to flowering time. Most Andean cultivars are photoperiod-sensitive; many Mesoamerican and determinate lines are day-neutral. Candidate flowering-time QTL: DTF-4.1 (PvCOL2, Pv04), DTF-9.1 / 9.4 / 9.5 (Pv09; AP3/FD, an E1 orthologue, AGL8/FUL). The paper reports additive and epistatic effects. [F] — [González et al. 2021, Front. Plant Sci. 11:599462](https://www.frontiersin.org/articles/10.3389/fpls.2020.599462/full)
- **Ppd dominance and the second locus Hr.** The dominant Ppd allele confers photoperiod sensitivity; ppd/ppd is insensitive (day-neutral). The second locus, Hr, changes the strength of the response and has no significant effect in ppd/ppd plants (this is epistasis). [F] — [CGSpace record, "Analysis of genes controlling photoperiod sensitivity in common bean using DNA markers"](https://cgspace.cgiar.org/items/561662d5-7b68-4922-a817-696cee55312b) (author and year not shown on the page; [K, to verify]: Gu et al. 1998).
- **Wild-type day length.** Wild forms need days shorter than 12 h to flower. [F] — [González et al. 2016](https://www.frontiersin.org/articles/10.3389/fpls.2016.01940/full)
- **Recent photoperiod genetics.** A CONSTANS-like gene loss relaxed the obligate short-day requirement, and photoperiod adaptation arose in parallel in the two domestications (Weller lab). Only the titles were seen. [F, titles only] — [figshare UTAS: "Ancient relaxation of an obligate short-day requirement in common bean through loss of CONSTANS-like gene function"](https://figshare.utas.edu.au/articles/journal_contribution/Ancient_relaxation_of_an_obligate_short-day_requirement_in_common_bean_through_loss_of_CONSTANS-like_gene_function/23002079); [figshare UTAS: "Parallel origins of photoperiod adaptation following dual domestications of common bean"](https://figshare.utas.edu.au/articles/journal_contribution/Parallel_origins_of_photoperiod_adaptation_following_dual_domestications_of_common_bean/22999553)
- **P (Pigment), the master colour switch.** P is dominant for seed colour. Recessive pp gives white seeds and white flowers, whatever the other colour genes are. P is a bHLH transcription factor (clade B, subclass IIIf). Ten race-specific P alleles each cause white seed: white seed arose by convergent evolution. P was the first marker found associated with a quantitative trait (seed size), and GWAS confirmed the P–seed-weight association. [F] — [McClean et al. 2018, New Phytologist (PubMed 29897103)](https://www.pubmed.gov/29897103/); epistasis rule [F] — [Plestenjak et al. 2025, Front. Plant Sci. 15:1523745](https://www.frontiersin.org/articles/10.3389/fpls.2024.1523745/full). P is on Pv07 [K, to verify]. The classic P–seed-size linkage is Sax (1923) [K, to verify].
- **Other seed-coat genes**, from Plestenjak et al. 2025 citing Bassett 2007, McClean 2018, Parker 2024 and others [F] — [Plestenjak et al. 2025](https://www.frontiersin.org/articles/10.3389/fpls.2024.1523745/full):
  - C: a MYB, PvMYB113. Controls colour and pattern. Closely linked modifiers Prp, Prpi and Gy sit within the C locus, and Cc heterozygotes change the pattern.
  - J: suspected MYB; primary colour, and the distribution of the coloured area.
  - B and G: colour intensity.
  - Rk: possibly a MYB; colour intensity.
  - V: flavonoid 3′,5′-hydroxylase; colour intensity, and purple versus white or pink flowers.
  - M, Pr, Acc and R: patterns (pinto, striped, mottled).
  - T: a WD40. Dominant T gives a fully coloured seed and flower; tt gives a partly coloured seed.
  - Bip (PvMYC1), Fib and Z (MYB113): the distribution of the coloured area in partly coloured seed.
  - The paper notes multiple alleles at many loci and much epistasis, without giving the rules.
- **Chromosome positions of the colour genes** [K, to verify]: C on Pv08; V on Pv06; T on Pv09; J on Pv10; G on Pv04; B on Pv02; Asp (seed-coat shininess, recessive asp = matte) on Pv07, near P. None of these was confirmed this session.
- **I gene (BCMV resistance).** On Pv02, at the end of the short arm. [F] — [Blair et al. 2018, PLoS ONE 13:e0189597](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn). Dominant I gives hypersensitive resistance to BCMV; with BCMNV it causes temperature-dependent "black root" necrosis [K, to verify].
- **Phaseolin (Phs, seed protein, linked to seed size)** is on Pv07; chalcone synthase (Chs) is on Pv02. [F] — [Blair et al. 2018](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn)
- **Pod shattering, PvMYB26 (Pv05).** Pod indehiscence (qPD5.1-Pv) maps to Pv05, with PvMYB26 (Phvul.005G157600) about 11 kb from the association peak. It is Mendelian and recessive: a BC4/F2 fit 3:1 dehiscent to indehiscent, and the domesticated allele works only when homozygous. Pod twisting is multifactorial. Other candidate shattering genes:
  - PvIND (Pv02), next to St
  - PvPdh1 (Pv03)
  - a MYB52-like gene (Pv04)
  - FUL-like genes (Pv09)
  - an NST1 / GmSHAT1-5 orthologue (Pv10)

  [F] — [Di Vittori et al. 2020, bioRxiv 10.1101/2020.04.02.021972](https://www.biorxiv.org/content/10.1101/2020.04.02.021972v1) (the published version is in J. Exp. Bot. 2021 [K, to verify]).
- **St (pod string)** is on Pv02 and co-maps with PvIND. [F] — [Di Vittori et al. 2020](https://www.biorxiv.org/content/10.1101/2020.04.02.021972v1). Dominant St gives a stringy pod; stringless (st) is recessive and was selected in snap beans [K, to verify].
- **Yellow wax pod (y, recessive)** [K, to verify; chromosome not found this session]. Seed dormancy, and the hard seed coat (impermeability) of wild beans, is a domestication trait with QTL [K, to verify; no source read this session].

### Inferences
- [I] A good first set of named loci for the game:
  - fin (habit, Pv01)
  - Ppd (Pv01, linked to fin)
  - P (white versus coloured, master epistatic switch, Pv07)
  - V (purple versus white flower, colour intensity)
  - C (pattern)
  - T (full versus partial colour)
  - St (string, Pv02)
  - PvMYB26 / shattering (Pv05)
  - I (virus resistance, Pv02, if disease ever exists)
  - Asp (shine)
  - y (wax pod)

  Ten or eleven named loci plus 10-25 anonymous QTL fits the 20-40-locus budget.
- [I] Seed-coat colour needs epistasis, which §3.12's additive-plus-dominance model does not express: P is a gate (pp means white whatever the other loci are), and T gates the pattern loci. Recommend a separate "visual trait" layer with a small rule table (gate, then base colour, then intensity, then pattern) that is not part of the additive physiological parameters. Colour has no physiological effect apart from the P–seed-size linkage.
- [I] Habit is already "the habit locus" in §3.4. Model fin as a single locus with complete dominance (Fin_ indeterminate, fin fin determinate). The F2 from bush × pole then segregates 3 pole : 1 bush, a believable first lesson in Mendelian ratios.

### Gaps
- Chromosome positions for V, C, T, J, G, B, Rk, Z, Bip, Asp and y were not confirmed this session; they are prior knowledge only. Check the [LIS / LegumeInfo bean gene list](https://www.legumeinfo.org/) or Bassett's "List of genes – Phaseolus vulgaris" (BIC Annual Report) before shipping.
- The exact epistasis table for seed-coat colour was not found in full. Bassett's reviews (2007) and Parker et al. 2024 are the primary sources.

## 2. Quantitative traits: heritability, QTL, G×E

### Takeaway
Broad-sense heritability is high for 100-seed weight (about 0.7-0.9) and days to maturity, moderate to high but variable for days to flowering (0.2-0.98 across studies), and lower and more environment-dependent for yield and pods per plant. Root angle is highly heritable and polygenic, with many small to moderate QTL. Seed weight is negatively correlated with pods per plant and seeds per pod, which supports §3.12's trade-off coming from physiology.

### Cited Findings
- **257 Kenyan accessions, 4 seasons.** Broad-sense H² was 0.89 for 100-seed weight, 0.77 for days to flowering and 0.61 for grain yield. Days to flowering and days to maturity correlated at r = 0.73. 100-seed weight correlated with pods per plant at r = −0.66 and with seeds per pod at r = −0.65. Yield ranged from 221 to 4,642 kg/ha (mean 1,267). [F, abstract] — [Wanjohi et al. 2023, J. Plant Breed. Crop Sci.](https://consensus.app/papers/details/50b6faee21d55a6f9fc0e807902b3635/?utm_source=claude_desktop)
- **100 Ethiopian landraces.** H² was 0.67 for days to 50% flowering, 0.87 for days to 90% maturity, 0.90 for pod length and 0.68 for 100-seed weight. GCV/PCV (%) was 11.7/19.4 for plant height and 12.7/15.4 for 100-seed weight. [F, abstract] — [Mesera et al. 2022, Adv. Agric.](https://consensus.app/papers/details/ecf67ba559e5510194be9fb6f151d40d/?utm_source=claude_desktop)
- **Sekoru, Ethiopia, two locations.** H² was low (0.20) for days to 50% flowering and high (0.935) for dry biomass. G×location was significant for most traits except plant height, biomass and 100-seed weight. [F, abstract] — [Demessie et al. 2024](https://consensus.app/papers/details/67ca5c50d370589cb2e685bed4f66280/?utm_source=claude_desktop)
- **Indian French bean.** H² was 0.98 for days to 50% flowering and 0.77 for seeds per pod. [F, abstract] — [Bora et al. 2026, Indian J. Agric. Res.](https://consensus.app/papers/details/86eb30e316755d198595769d295cc59e/?utm_source=claude_desktop)
- **Kashmir accessions, 4 locations.** H² was above 0.60 for all traits except pod length, with high expected genetic gain (above 30% of the mean) for days to maturity, pods per plant, seeds per pod, 100-seed weight and yield per plant. [F, abstract] — [Razvi et al. 2018, Legume Res.](https://consensus.app/papers/details/2dc06d10f74f52e49ab50187735cff25/?utm_source=claude_desktop)
- **Flowering-time QTL.** Flowering time in Andean beans is controlled by several QTL (on Pv01, Pv04 and Pv09, among others) with significant QTL × environment and epistatic effects. [F] — [González et al. 2021](https://www.frontiersin.org/articles/10.3389/fpls.2020.599462/full). Flowering time and vegetative growth make a major contribution to plant production. [F, title and summary] — [González et al. 2016](https://www.frontiersin.org/articles/10.3389/fpls.2016.01940/full)
- **Basal root growth angle (BRGA), Lynch lab.** In DOR364 × G19833 RILs (86 F5.7 lines), BRGA ranged from −18.7° to 56.7° and shallow basal root length from 0.42 to 2.63 m per plant. There were 16 QTL for root gravitropic traits across the 11 linkage groups and 6 for P acquisition efficiency; at least 3 root QTL co-located with P-efficiency QTL. In the field, shallow roots correlated with growth and P uptake. [F, abstract] — [Liao et al. 2004, Funct. Plant Biol.](https://consensus.app/papers/details/655e44d0e83557cf9abd28653ad4d176/?utm_source=claude_desktop)
- **Root angle in 10 lines.** High broad-sense heritability with high genetic advance for root growth angle and 100-seed weight. [F, abstract] — [Ibram et al. 2025, J. Appl. Genet.](https://consensus.app/papers/details/e32ee535859f52cda8140d4d129e74f5/?utm_source=claude_desktop)
- **OSU5446 × RR6950 RILs.** One QTL for basal root angle and one for taproot diameter. Root-rot resistance QTL each explained 5-22% of genetic variance. [F, abstract] — [Hagerty et al. 2015, Crop Sci. 55:1969](https://consensus.app/papers/details/284c8f84dfe15d6895a9eb1008d7d60c/?utm_source=claude_desktop)
- **Adventitious rooting under low P.** 19 QTL on 8 of the 11 linkage groups together explained 19-61% of phenotypic variance. Two major QTL (B2 and B9) explained 61% together. Transgressive segregation was observed. [F, abstract] — [Ochoa et al. 2006, Crop Sci.](https://consensus.app/papers/details/36d5e1f9b63354c689353082eb0f36eb/?utm_source=claude_desktop)
- **Root-angle heritability as a number** [K, to verify]: Lynch-lab work, e.g. Bonser et al. 1996 and Basu et al. 2007, reports moderate heritability for basal root angle, with strong plasticity to P supply. I found no exact h² figure this session.
- **Photosynthesis.** Leaf photosynthetic rate and Vcmax vary among bean genotypes, typically by about 10-30% within cultivated germplasm [K, to verify; no source read this session].

### Inferences
- [I] Suggested heritable-parameter targets, as H² in a uniform pot environment; the game environment is controlled, so H² can be at the high end:

  | Parameter | Suggested H² | Loci |
  | --- | --- | --- |
  | seed size | 0.8 | 2-3 loci, one linked to P on Pv07 |
  | phenology thresholds | 0.7 | Ppd major locus + 1-2 minor; Ppd only matters under long days |
  | root depth tendency (BRGA) | 0.6 | 2-3 loci |
  | branching | 0.5 | |
  | leaf size | 0.6 | |
  | SLA | 0.5 | |
  | Vcmax | 0.4 | |
  | seeds per pod | 0.5 | |

  Yield and pods per plant should not be heritable parameters. They are emergent, and their low realized H² will appear on its own through environmental noise and the trade-offs. This matches §3.12's "trade-offs from physiology".
- [I] The seed-weight versus pods-per-plant correlation (about −0.65) is a good validation target for emergent trade-offs on the bench.
- [I] G×E arises naturally from the process model (photoperiod × Ppd; water × root depth). No explicit G×E term is needed.

### Gaps
- No primary narrow-sense h² (parent–offspring or realized) for bean seed weight was read this session. All values above are broad-sense, from germplasm panels.
- No source was read for genotypic variation in Vcmax in bean.

## 3. Linkage

### Takeaway
Bean has 11 chromosome pairs, a genome of about 514-587 Mb and a genetic map of about 1,100-1,200 cM (about 100 cM per chromosome). Biologically meaningful linkages exist (fin–Ppd on Pv01, P–phaseolin/seed size on Pv07, St–I on Pv02). Optional light linkage (chromosome plus cM position) would add realism cheaply and does not threaten "F2 variance > F1 variance".

### Cited Findings
- **Reference genome G19833 v1.0.** 514 Mb, n = 11 (Pv01-Pv11), mostly acrocentric. Pv09 is telocentric with large rDNA blocks. [F] — [Blair et al. 2018, PLoS ONE](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn)
- **Map length.** The refined BAT93 × Jalo EEP558 map is 1,097.5 cM (the initial SNP map was 1,762.5 cM). Linkage groups range from 46.5 cM (Pv09) to 147 cM (Pv01), averaging 99.8 cM. The mean recombination rate is 2.13 cM/Mb, ranging from 1.24 cM/Mb (Pv09) to 2.87 cM/Mb (Pv02). Recombination is suppressed in pericentromeric regions. [F] — [Blair et al. 2018](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn)
- **LD decay.** LD falls to r² = 0.1 within about 20 cM (Andean) and about 30 cM (Mesoamerican), "fairly typical of a self-pollinating species". [F] — [Blair et al. 2018](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn)
- **Other maps.** 1,196 cM over 11 linkage groups [F] — [Hagerty et al. 2015](https://consensus.app/papers/details/284c8f84dfe15d6895a9eb1008d7d60c/?utm_source=claude_desktop). 1,175 cM [F] — [Ochoa et al. 2006](https://consensus.app/papers/details/36d5e1f9b63354c689353082eb0f36eb/?utm_source=claude_desktop).
- **Known linkages:**
  - fin with Ppd (Pv01) [F] — [González et al. 2021](https://www.frontiersin.org/articles/10.3389/fpls.2020.599462/full)
  - P with seed weight [F] — [McClean et al. 2018](https://www.pubmed.gov/29897103/); phaseolin is on Pv07 [F] — [Blair 2018](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn)
  - St with PvIND (Pv02) [F] — [Di Vittori 2020](https://www.biorxiv.org/content/10.1101/2020.04.02.021972v1); I is also on Pv02 [F] — [Blair 2018](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn)
  - C with Prp, Prpi and Gy (tightly linked within the C locus) [F] — [Plestenjak 2025](https://www.frontiersin.org/articles/10.3389/fpls.2024.1523745/full)
- **Reference genome paper.** "A reference genome for common bean and genome-wide analysis of dual domestications" (Schmutz et al. 2014, Nature Genetics) gives the assembly as about 521 Mb of a genome estimated at about 587 Mb [K, to verify; FAO AGRIS record title seen only](https://agris.fao.org/search/en/providers/122439/records/6798de73d6a63682f045e850).

### Inferences
- [I] Recommendation: give each locus a `chromosome: u8 (1..=11)` and `position_cm: f64`. At meiosis, draw crossovers per chromosome as a Poisson process with mean = chromosome length / 100 cM (Haldane, no interference), from ps-rng keyed by (seed, plant id, tick, "meiosis", chromosome). This costs a few dozen RNG draws per gamete.
- [I] Loci on different chromosomes, or more than 50 cM apart, then assort independently, so the §3.12 behaviour is the special case.
- [I] Use about 100 cM per chromosome (Pv09 about 50 cM, Pv01 about 150 cM).
- [I] Linkage makes repulsion-phase combinations (e.g. bush habit + photoperiod-insensitive when fin and Ppd come from different parents) harder to recover. That is a believable breeding puzzle.
- [I] If linkage is adopted it needs a decision record, because §3.12 says "Loci are unlinked". Adding it in WP28 means only one model version is affected.
- [I] An alternative: keep loci unlinked in M3 and put only the two showcase linkages (fin–Ppd at about 10-20 cM, P–seed size at about 10 cM) in a later WP. The exact cM distances were not found; see Gaps.

### Gaps
- The cM distances fin–Ppd and P–seed-size QTL were not read this session.

## 4. Reproductive biology: selfing, outcrossing, hand-pollination

### Takeaway
Bean is predominantly self-pollinating: anthers dehisce in the closed keel before the flower opens. Natural outcrossing ranges from under 1% in pollinator-poor sites to 5-10% or more where large bees are active (one Californian study reported much higher rates). Hand crosses succeed in roughly 40-60% of pollinations, and published reports of hooking methods quote 70-90% pod set with 4-6 seeds per pod. Pure lines, a uniform F1, a segregating F2, then single-seed descent to F5-F6 is the correct model.

### Cited Findings
- **California, 1989-1991.** Weighted mean outcrossing was 6.9% across about 120,000 seedlings (purple-hypocotyl marker). Line means ranged from 4.4% to 10.2%, and single families from 0% to 78%. Environment interactions were strong. [F, abstract] — [Ibarra-Pérez et al. 1997, Crop Sci.](https://consensus.app/papers/details/a1238c7e29ed57e99f510c1b32c68164/?utm_source=claude_desktop)
- **Irvine, CA, 1985.** Outcrossing averaged 19.3% for five lines and 66.8% for 'FM 53'. It was 18.8% at the early planting and 47.0% at the late planting. [F, abstract] — [Wells et al. 1988, Crop Sci.](https://consensus.app/papers/details/f71d65843c315bb184af85c5ef9f6ed0/?utm_source=claude_desktop)
- **Asturias, Spain.** No outcrossing was detected; the rate was below 0.74% at 95% confidence. [F, abstract] — [Ferreira et al. 2000, Euphytica](https://consensus.app/papers/details/d3ed065bd50b556c84c4bf3c6aff2a97/?utm_source=claude_desktop)
- **Hand-pollination methods compared in the glasshouse** [F] — [Usher & Redden 1986, Qld J. Agric. Anim. Sci. 43:115](https://era.dpi.qld.gov.au/id/eprint/13349/1/QJAAS_43%5B2%5D_1986_pp115-117_usher%26redden.pdf):
  - With emasculation: 143 F1 hybrids from 340 pollinations (0.43 per pollination), and 3 selfs.
  - Without emasculation (pollinated stigma inserted through a keel incision): 0.62 hybrids per pollination, but 51 selfs, and pod set about 50%.
  - The paper cites Buishand (1956) and Bliss (1980): 70-90% pod set and 4-6 seeds per pod with the "hooking" method, where the pollen-coated stigma of a male flower is hooked onto the exposed stigma of the female flower.
  - Hybrids were identified with a dominant marker on the male side: the female had white flowers and seed, the male coloured.
- **How a cross is done** [K, to verify]: the female bud is emasculated about 1 day before anthesis (keel opened, anthers removed with forceps). Crosses take about 3-5 weeks from pollination to mature seed in the glasshouse.
- **Inbreeding depression and heterosis.** Little inbreeding depression and modest heterosis in bean [K, to verify]. Blair et al. 2018 call bean an "inbreeding crop" and observe 0.5% heterozygous calls in F11 RILs [F] — [Blair et al. 2018](https://escholarship.org/content/qt042809xf/qt042809xf.pdf?t=shj4cn).

### Inferences
- [I] Default: selfing with probability 1.0 in pots (no pollinators indoors). An optional "bees" scenario flag could give an outcrossing rate of 0.01-0.07 per flower among neighbouring plants, keyed by ps-rng.
- [I] Hand-pollination action:
  - success probability 0.5 (range 0.4-0.7)
  - on success, 4 seeds per pod (range 3-6)
  - require the target flower to be a bud 1 day before opening
  - optionally, a 5% chance of an accidental self if the player skips emasculation. This makes the dominant-marker check (a white female × coloured male gives a coloured F1) a meaningful in-game tool, exactly as real breeders use it.
- [I] Starting seeds should be fully homozygous pure lines, with small among-line variation in the species allele distributions and none within a line. Then an F1 is uniform (variance ≈ environmental only) and the F2 segregates, which meets WP28's "F2 variance exceeds F1 variance" by construction.
- [I] Expected F2 variance = V_E + V_A + V_D (with F1 variance ≈ V_E). Heterozygosity halves with each selfing generation; F5 is about 94% homozygous and F6 about 97%.

### Gaps
- No primary source was read for the exact emasculation timing or the cross-to-seed duration.
- No quantitative data on inbreeding depression or heterosis percentages were found this session.

## 5. Mutation rate

### Takeaway
Real spontaneous visible-mutation rates are about 1e-5 to 1e-6 per locus per gamete, so the design's 1e-3 is 100-1,000 times too high for believability. It is defensible as "game time compression" only if mutations have small effects.

### Cited Findings
- I found no bean-specific spontaneous mutation-rate source this session. Classic per-locus rates for visible mutations in plants (maize, Stadler) are about 1e-6 to 1e-5 per gamete [K, to verify]. Per-generation mutational variance is V_M ≈ 1e-3 × V_E for quantitative traits across organisms (Lynch & Walsh; Houle et al. 1996) [K, to verify].

### Inferences
- [I] At 30 loci, 1e-3 per locus per gamete gives about 6% of seeds carrying a new mutation (30 × 2 × 1e-3).
- [I] Recommendation: either
  - keep 1e-3 for anonymous QTL with small Gaussian effects (SD about 0.1-0.2 of the locus allelic SD), framing it as the polygenic mutational variance V_M ≈ 1e-3 V_E, which is realistic in aggregate; or
  - for named Mendelian loci (fin, P, V, St), use 1e-5, or no mutation at all, so a white-seeded pure line does not spontaneously throw coloured seeds in an ordinary game.

  Document this as a game approximation in the model card.

### Gaps
- No primary citation was read for mutation rates; verify Houle 1996 (Genetics 143:1467) and the plant visible-mutation rates.

## 6. Response to selection

### Takeaway
With R = h²S and bean seed-size h² around 0.6-0.9, five generations of selecting the top 10-20% of plants for seed size should move the mean by several phenotypic SDs within a segregating population. This easily satisfies "measurably different after five generations", provided the starting population segregates (an F2 or a mixture), not a pure line.

### Cited Findings
- High heritability with high genetic advance as a percentage of the mean (above 30%) is reported for 100-seed weight, days to maturity, pods per plant and seeds per pod. [F, abstract] — [Razvi et al. 2018](https://consensus.app/papers/details/2dc06d10f74f52e49ab50187735cff25/?utm_source=claude_desktop)
- Additive gene action predominates for most yield-component traits. [F, abstract] — [Jan et al. 2026](https://consensus.app/papers/details/2dbb7702fb625d6db5de00d71469eca5/?utm_source=claude_desktop); [Kuma 2024](https://consensus.app/papers/details/b851c0f81b445b6dbeee17731846912d/?utm_source=claude_desktop)
- Genetic advance: GA = k·σP·h², with k = 2.06 for 5% selection intensity [K, to verify; standard formula, Johnson et al. 1955].
- In recurrent selection, bean realized gains per cycle for yield are a few percent [K, to verify; no source read].

### Inferences
- [I] A worked example. Bench population: an F2 of a small-seeded (25 g / 100 seeds) × large-seeded (50 g) cross, with h² ≈ 0.7. Selecting the top 2 of 10 plants (i ≈ 1.4) gives R ≈ 1.4 × 0.7 × σP per generation, roughly 1 σP per generation, shrinking as variance is fixed. After 5 generations the selected line should differ from an unselected or reverse-selected line by more than 3 σP. A t-test on 10 vs 10 plants is significant.
- [I] WP acceptance test: run a divergent-selection test (top 2 of 10 versus bottom 2 of 10, 5 generations, fixed seeds) and assert the line means differ by more than 2 pooled SD. Under inbreeding, selection response plateaus once lines are fixed; this is realistic and a teachable moment.

### Gaps
- No primary realized-heritability or selection-experiment data for bean seed size were read this session.

## 7. Existing breeding simulators and games

### Takeaway
AlphaSimR is the de facto standard stochastic breeding-program simulator. Its genome model (chromosomes with genetic maps, additive, dominance and epistatic QTL, meiosis with crossovers) is the reference design to imitate.

### Cited Findings
- AlphaSimR is an R package for "flexible modelling of breeding programmes" from the Highlander lab, Roslin Institute. It is on CRAN (versions 1.6.0 and 2.0.0 listed). [F, page titles] — [Roslin AlphaSimR page](https://vet.ed.ac.uk/roslin/research/divisions/quantitative-biology/highlander-lab/software/alphasimr); [rdocumentation AlphaSimR](https://www.rdocumentation.org/packages/AlphaSimR)
- AlphaSimR simulates chromosomes with a genetic map (Morgans), crossovers by a gamma or Poisson-type model, and additive, dominance and GxE trait architectures (Gaynor, Gorjanc & Hickey 2021, G3 11:jkaa017) [K, to verify]. QU-GENE / QuLine (Podlich & Cooper 1998; Wang et al. 2003) simulated inbred-line breeding including SSD and pedigree methods [K, to verify]. I could not verify a "Mendel's Garden" game this session.

### Inferences
- [I] Mirror AlphaSimR's model in Rust: a founder population with allele frequencies, QTL effects drawn from a Gamma or Normal distribution, dominance degree per locus, and meiosis with Haldane crossovers.
- [I] Tests for the game's genetics (an inference about method): compare F2 segregation ratios and the selection-response numbers against a small AlphaSimR script offline as a cross-check.

### Gaps
- No education-game sources were read; any claims about breeding games remain unverified.
