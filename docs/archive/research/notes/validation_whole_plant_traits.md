# Whole-plant and leaf trait reference data: bush bean (Phaseolus vulgaris)

Compiled 2026-11-05. Every number below was read from the source named next to it, using WebFetch on the publisher page, SciSpace full-text extraction, or the PubMed/PMC tool. Nothing comes from memory.

Status tags:
- **[TABLE]**: read from a table in the paper.
- **[TEXT]**: read from the abstract or body text.
- **[SECONDARY]**: a value that one paper reports about another paper.
- **[NOT FOUND]**: could not be located.
- **[DIFFERS]**: does not match the design plan's quote.

No figures were digitised, because the figure images could not be fetched.

**Download note.** No files could be saved to `scratchpad/downloads/traits/`. The agent proxy refused direct (curl) connections to scielo.br, minio.scielo.br, arcjournals.org, ps.ueb.cas.cz, apct.sede.embrapa.br, core.ac.uk and redalyc.org (CONNECT 403). The directory exists but is empty. All reading was done through WebFetch, SciSpace or PubMed. Treat any text quoted here as untrusted third-party data.

---

## 1. Kumar 2025, "Growth performance and yield contributing traits of Kharif French bean varieties under different dates of sowing": [NOT FOUND]

Searches run:
- WebSearch (standard and extended) on the exact title, the quoted phrase fragments, and author + year + journal-family combinations.
- SciSpace semantic search, twice.
- Consensus, which was rate-limited and then hit its monthly quota.

None returned a paper with this title. Papers that were found but are **not** this one:
- Thakare et al. 2025, *Int J Res Agron* 8(5):764-767, DOI 10.33545/2618060X.2025.v8.i5j.2982. Rabi season, Parbhani, India. Varieties Varun, Phule Viraj and Phule Rajma; spacing 45 × 10 cm. Yield and economics only: no height, leaf or dry-mass data. Read via WebFetch.
- Lad & Bhagat 2022, *Int J Farm Sci*, DOI 10.5958/2250-0499.2022.00003.9. Rajmash, kharif 2015, Pune, four sowing dates (20 Jun to 20 Aug). From the abstract only:
  - GRB-902 had the maximum plant height (55.68 cm), primary branches (2.85) and seed yield per plant (19.60 g).
  - GRB-702 had 19.38 g seed yield per plant.
  - Full text was not accessed.

**None of the plan's values could be verified:** height 35 cm, 5.5 branches, 20 leaves, leaf area 0.12 m², dry mass 21.8 g/plant, 11.8 pods/plant, 3.7 seeds/pod, 8.8 g seed/plant. No DAS time series was obtained. Next step: ask the plan author for the DOI or journal of the Kumar 2025 paper.

---

## 2. Gebeyehu 2019, "Evaluation of growth performance of common bean genotypes during two phenological phases": [NOT FOUND]

WebSearch (standard and extended) and SciSpace did not find this title or author. The closest matches are two Hawassa (Ethiopia) growth-analysis papers. Both measure SLA at two samplings, so the plan may have mixed up the citation.
- **Desalew 2021**: see section 3a. Its highest first-sampling SLA is 228.49 cm²/g (Ibbado), which is near the plan's "~240 cm²/g highest genotype".
- **Mohammed 2018**: see section 3b. SLA 284 to 341 cm²/g.

**The plan's "~240 cm²/g" is not verified for Gebeyehu 2019.**

---

## 3. Merga 2021, "Evaluation of some haricot bean cultivars for growth characters and dry matter production": [NOT FOUND under this author]

No paper by Merga with this title was found. A 2021 paper with a near-identical title exists (3a). A 2018 paper matches the plan's NAR "up to 41 mg/dm²/day" (3b). Both are given below in full.

### 3a. Desalew 2021 (closest title match)

**Citation.** Desalew F. (2021). Evaluating Haricot Bean Cultivars (*Phaseolus vulgaris* L.) for Growth and Dry Matter Production. *International Journal of Research Studies in Agricultural Sciences* 7(9):13-16.
- DOI: 10.20431/2454-6224.0709003
- URL read: https://arcjournals.org/pdfs/ijrsas/v7-i9/3.pdf (reached via WebFetch after the DOI redirect)
- Licence: Creative Commons Attribution (stated in the paper)
- Sole author: Fikir Desalew, Holetta Agricultural Research Center. **There is no co-author named Merga.**

**Conditions.**
- Site: Hawassa University College of Agriculture, Ethiopia; 6°42′N, 38°29′E; 1650 m a.s.l.
- Climate: mean annual rainfall 900 mm. Mean annual temperatures are printed as "max 13 °C, min 27 °C", which looks transposed.
- Design: RCBD with 3 reps; plots of 5 rows × 2 m; spacing 40 × 10 cm; 50 kg/ha DAP.
- Sown 9 Apr 2019.
- Destructive samples: 10 May and 23 May 2019, i.e. **20 and 33 days after emergence**. Three plants per plot; oven-dried 48 h at 72 °C.
- Cultivars: Hawassa Dume, Ibbado, SER-119, SER-125, Nassir. Growth habit is not stated.

**Table 2: SLA (cm²/g)** [TABLE]
```
cultivar,SLA_20DAE,SE,SLA_33DAE,SE
Hawassa Dume,205.52,4.33,267.02,78.01
Ibbado,228.49,4.61,209.12,75.76
SER-119,198.91,22.84,320.35,50.24
SER-125,201.18,1.95,228.53,6.19
Nassir,218.17,10.02,256.67,4.14
```
Range: 198.9 to 228.5 at 20 DAE; 209.1 to 320.4 at 33 DAE.

**Table 1: LAR (cm²/g)** [TABLE]
```
cultivar,LAR_20DAE,SE,LAR_33DAE,SE
Hawassa Dume,154.02,3.87,149.37,34.42
Ibbado,151.22,0.98,126.53,5.48
SER-119,142.54,16.71,195.02,32.75
SER-125,138.60,2.98,130.43,4.34
Nassir,155.45,5.4,158.61,3.55
```

**Table 3: NAR and RGR, interval 20 to 33 DAE (the table does not state the interval)** [TABLE]
```
cultivar,NAR_mg_dm-2_d-1,SE,RGR_mg_g-1_d-1,SE
Hawassa Dume,263.09,10.15,92.91,3.14
Ibbado,184.69,7.35,52.60,5.86
SER-119,222.18,1.02,85.85,1.56
SER-125,205.42,2.73,77.21,0.37
Nassir,212.78,1.08,85.69,1.53
```

Caveats:
- **NAR of 185 to 263 mg/dm²/d is physically implausible**: it is about 5 to 10 times typical crop values. It is probably a unit or calculation error. The paper itself cites "near 80 mg/dm²/day" for early growth (Wallace & Munger 1965).
- **[DIFFERS]**: the plan's "up to 41" does not appear anywhere in this paper.
- Not reported: LAI, CGR, and tables of leaf area, plant height or dry mass. The paper says these were measured but gives no values.

### 3b. Mohammed 2018 (matches the plan's NAR of about 41 to 44)

**Citation.** Mohammed S. (2018). Haricot Bean (*Phaseolus vulgaris* L.) Varieties Growth Analysis at Hawassa University, Ethiopia. *Journal of Natural Sciences Research* 8(13):63-66.
- No DOI is printed.
- URL read: https://www.iiste.org/Journals/index.php/JNSR/article/download/43521/44836
- Licence: not stated in the text.

**Conditions.**
- Site: Hawassa; 07°03′N, 38°28′E; 1708 m a.s.l. Daily maximum 27 °C, minimum 13 °C. Off-season planting in 2018.
- Design: RCBD with 4 reps; plots 2 × 2 m with 5 rows; spacing 40 × 10 cm; 50 kg/ha DAP.
- Samples: **29 and 43 days after emergence**, 3 plants from the centre rows; dried at 70 °C for 48 h.
- Varieties: Ibado, Omo-95, Red Wolayita.
- None of the variety differences was significant (ANOVA, Tables 2 and 4).

**Table 1: LAR and SLA (cm²/g); mean ± pooled SE** [TABLE]
```
variety,LAR_29DAE,LAR_43DAE,SLA_29DAE,SLA_43DAE
Ibado,200.98,147.28,312.46,300.85
Omo-95,171.57,159.09,294.46,340.65
Red Wolayita,174.58,162.17,284.12,335.92
pooled_SE,15.05,6.65,27.61,14.75
grand_mean(Table2),182.38,156.18,297.01,325.81
```

**Table 3: RGR, NAR (interval 29 to 43 DAE) and biomass at the 2nd sampling** [TABLE]
```
variety,RGR_mg_g-1_d-1,NAR_mg_dm-2_d-1,biomass_g
Ibado,49.81,29.47,19.14
Omo-95,52.53,33.34,16.85
Red Wolayita,72.81,43.55,16.76
pooled_SE,8.04,5.77,1.78
grand_mean(Table4),58.39,35.46,17.58
CV_%,27.57,32.56,20.3
```

Caveats:
- The biomass basis is unclear: it could be per plant or per 3-plant sample.
- **[DIFFERS]**: the maximum NAR here is **43.55** mg/dm²/d, not 41, and this paper is Mohammed 2018, not Merga 2021.
- Not reported: CGR and LAI.

---

## 4. Pimentel et al. 2011, "Características fotossintéticas de *Phaseolus vulgaris* L."

**Citation.** Pimentel C., Oliveira R.F. de, Ribeiro R.V., Santos M.G. dos, Machado E.C. (2011). *Hoehnea* 38(2):273-280.
- DOI: 10.1590/S2236-89062011000200007
- URLs read: https://www.scielo.br/j/hoehnea/a/n6qW3pF9NWGmd435Dn6t8zq/abstract/?lang=en and the full Portuguese HTML at https://www.scielo.br/j/hoehnea/a/n6qW3pF9NWGmd435Dn6t8zq/?lang=pt
- Licence: CC BY 4.0, as shown on the SciELO page.
- Table 1 and Figures 1 to 4 are images only. The image fetch and the PDF text layer both failed, so **Table 1 was not read directly**. All values below come from the abstract and the results text.

**Plant material and conditions** [TEXT]
- Cultivar: **cv. Carioca**, described as *P. vulgaris* var. *mexicanus*.
- Growth: greenhouse, 8 L pots, 1 plant per pot. Substrate was organic matter and vermiculite (3:1). Greenhouse temperature ran from a 12 °C night minimum to a 28 °C day maximum.
- Fertiliser: N 20, P2O5 90, K2O 30 kg/ha, plus 40 kg N/ha top-dressed at 25 DAE.
- Stage: measured at pre-flowering, **40 to 45 days after planting**. Leaf: central leaflet of the 5th trifoliate, the youngest fully expanded.
- Instrument: LI-6400XT with a 6 cm² fluorometer chamber. Chamber at **25 ± 1 °C**, VPD below 2.0 kPa, measured 10:00 to 11:30.
- A/PPFD curve: started at 1500 µmol m⁻² s⁻¹ and stepped down by 50 per step. A/Ci curve: PPFD 700 µmol m⁻² s⁻¹; Ca stepped 380 → 50 → 380 → 1500.
- **n = 4 curves, each on a different plant** (completely randomised design; SNK test at 5%).

**Values** [TEXT]
```
parameter,value,unit,source
Vcmax,89.3,umol m-2 s-1,abstract+results (Table 1 cited)
Vomax,24.7,umol m-2 s-1,abstract
Jmax,159.8,umol m-2 s-1,abstract+results
Rd (from A/Ci fit; day respiration, not a dark measurement),0.88,umol m-2 s-1,abstract+results
gm,0.47,mol m-2 s-1,abstract+results
A at Ca 380 (Amax),~20,umol m-2 s-1,results
A at saturating Ci (>=600),~25,umol m-2 s-1,results/Fig 4
gs max,~0.4,mol m-2 s-1,results
Fv/Fm,0.78 +/- 0.02,-,results
ETR max,~120,umol m-2 s-1,results/Fig 1
light saturation of A and ETR,>800,umol m-2 s-1 PPFD,results/Fig 1
```
The plan's Vcmax 89.3, Jmax 159.8, Rd 0.88 and gm 0.47 are **verified [TEXT]**.

**Ci or Cc basis: NOT STATED.**
- The methods say the parameters were computed "conforme modelos matemáticos propostos por Farquhar et al. (1980), ajustados por Bernacchi et al. (2001)", with methods from Long & Bernacchi (2003) and simultaneous fluorescence.
- The text also says that older models assume Cc = Ci, which "não é exato". gm was estimated, but its method is not described beyond the Long & Bernacchi 2003 reference. Harley 1992 is not cited.
- **The values of Kc, Ko and Γ\* are not given in the text.** The only lead is the Bernacchi 2001 citation.
- **Not verified:** whether Bernacchi et al. (2001) constants are Ci-based. That is the background knowledge behind the next bullet and was not checked against that paper here.
- On that assumption, Vcmax 89.3 is most likely on a Ci basis (gm assumed infinite), with gm estimated separately. This is an inference and is not confirmed; Table 1 (an image) may settle it.
- **Recommendation:** treat Vcmax 89.3 as Ci-based (apparent) until Table 1 or the authors confirm otherwise.

**Discussion comparisons** [SECONDARY]
- The authors say their Vcmax and Jmax are above the 109-species C3 means of **64 and 134**.
- They give **Vcmax 49 to 91 and Jmax 96 to 165 µmol m⁻² s⁻¹** as Wullschleger (1993)'s values "para o feijoeiro comum", and say the values resemble *P. vulgaris* var. *aborigineus* in Wullschleger (1993).
- They give a gm range of 0.16 to 0.39 mol m⁻² s⁻¹ for herbaceous species (Flexas et al. 2008).
- They cite Amax of about 15 µmol m⁻² s⁻¹ and gs of 0.32 mol m⁻² s⁻¹ for var. *aborigineus* (Wentworth et al. 2006).

---

## 5. Wullschleger 1993

**Citation.** Wullschleger S.D. (1993). Biochemical limitations to carbon assimilation in C3 plants: a retrospective analysis of the A/Ci curves from 109 species. *Journal of Experimental Botany* 44(5):907-920.
- DOI: 10.1093/jxb/44.5.907
- URLs read: https://academic.oup.com/jxb/article/44/5/907/525226 (abstract only; full text is a PDF behind a paywall) and https://scispace.com/papers/biochemical-limitations-to-carbon-assimilation-in-c3-plants-32vit11f8t (abstract and summary).
- Licence: not open access.

**Values** [TEXT, abstract]
```
group,Vcmax_umol_m-2_s-1,Jmax_umol_m-2_s-1
all 109 species (164 curves),64 (range ~6-194),134 (range ~17-372)
annuals (herbaceous),75,154
woody perennials (SciSpace summary),44,97
```
The plan's 75 and 154 are **verified** from the OUP abstract, which reads: "For annuals, Vcmax and Jmax averaged 75 and 154 µmol m⁻² s⁻¹".

**Phaseolus vulgaris rows in Table 1: [NOT FOUND]**; the table is paywalled. The only figure available is Pimentel 2011's report that Wullschleger lists common bean at Vcmax 49 to 91 and Jmax 96 to 165 µmol m⁻² s⁻¹ [SECONDARY]. Source papers and Ci basis for each row are unknown. Note that Wullschleger's analysis was Ci-based by design (A/Ci curves), but this was not confirmed from the full text here.

---

## 6. Bush bean Amax and leaf dark respiration, extra sources

### 6a. McKenzie-Gopsill et al. 2020: field, temperate (Canada)

**Citation.** McKenzie-Gopsill A.G., Amirsadeghi S., Fillmore S., Swanton C.J. (2020). Duration of Weed Presence Influences the Recovery of Photosynthetic Efficiency and Yield in Common Bean (*Phaseolus vulgaris* L.). *Frontiers in Agronomy* 2:593570.
- DOI: 10.3389/fagro.2020.593570
- URL read: https://www.frontiersin.org/journals/agronomy/articles/10.3389/fagro.2020.593570/full
- Licence: not shown in the fetched text. Frontiers is normally CC BY, but this was not verified.

**Conditions** [TEXT]
- Site: Harrington, Prince Edward Island, Canada (46°21′N, 63°09′W), 2019.
- Cultivar: **CDC Panther**. The paper calls it **indeterminate**, so it is not a determinate bush type.
- Sown 4 Jul 2019, 45 cm rows, 47 plants/m². RCBD with 4 reps.
- Instrument: LI-6800F, measured 08:00 to 11:00. Leaf at **25 °C**, CO2 400 µmol/mol, VPD 1.5 kPa.
- **PPFD matched to ambient, so not necessarily saturating.** Leaf: the youngest fully expanded unshaded trifoliate, 2 plants per plot, weekly from V1 to R4.

**Values** [TEXT]
- A of plants weeded at V3 and V4 (close to weed-free): **27.0 and 28.7 µmol CO2 m⁻² s⁻¹** (Fig. 4B text). The weed-free control A appears only in figures.
- Vcmax in weed-free plants: rose **from 76.5 to 155.4** µmol m⁻² s⁻¹ during vegetative growth, then fell **to 93.3** in reproductive stages (Fig. 9 text; n = 64 A/Ci curves).
- Leaf dark respiration was not reported.

### 6b. Pimentel 2011 (section 4)

Greenhouse, cv. Carioca, Brazil. A at Ca 380 was about 20 µmol m⁻² s⁻¹ (PPFD near saturation, above 800), and about 25 at saturating Ci. Rd 0.88 is a value fitted from the A/Ci curve, not a dark measurement.

### 6c. Azcón-Bieto 1986 (dark respiration, partial)

**Citation.** Azcón-Bieto J. (1986). Effect of Oxygen on the Contribution of Respiration to the CO2 Compensation Point in Wheat and Bean Leaves. *Plant Physiology*. Read via SciSpace full-text extraction (untrusted summary).
- Cultivar: Hawkesbury Wonder (Australia).
- Rn (dark CO2 efflux, 30 min after lights off, 25 °C) in **1-day-old, still-expanding** leaves: **4.1 µmol m⁻² s⁻¹ at 21% O2** and **3.9 at 2% O2**.
- The text says Rn falls quickly as the leaf matures. The mature-leaf values are in Table IV, which was not obtained.
- **Not suitable as a mature-leaf Rd.**

### 6d. Mature-leaf dark respiration from a temperate field or greenhouse: [NOT FOUND]

Candidate papers were identified but their full text was not accessible:
- Parsons et al. 1997, *Photosynthetica* 33:265-279, DOI 10.1023/A:1006800909651. Gives P. vulgaris light and dark respiration and Amax; the PDF is a scan with no text layer.
- Costa et al. 1988, *Pesq. Agropec. Bras.* Greenhouse, cv. Negrito 897; photosynthesis and dark respiration. The download redirect could not be fetched.
- Jifon & Wolfe 2005, *JASHS*. Red kidney 'Redkloud', described as strongly determinate, grown at 26/15 °C in Ithaca NY. Light-saturated An at PPF 1200 was measured, but no absolute values appeared in the extracted text.

---

## Summary against the plan

```
item,plan_value,status,found_value,source
Kumar height/branches/leaves/LA/DM/pods/seeds/seed mass,35/5.5/20/0.12/21.8/11.8/3.7/8.8,NOT FOUND,-,-
Gebeyehu SLA highest,~240 cm2/g,NOT FOUND (paper),nearest: 228.5 (Desalew 2021 Tbl2 20DAE),Desalew 2021
Merga NAR early,up to 41 mg dm-2 d-1,NOT FOUND (paper); DIFFERS,43.55 max (Mohammed 2018 Tbl3); Desalew 2021 gives 185-263 (implausible),Mohammed 2018 / Desalew 2021
Pimentel Vcmax,89.3,VERIFIED [TEXT]; basis not stated,89.3,abstract/results
Pimentel Jmax,159.8,VERIFIED [TEXT],159.8,abstract/results
Pimentel Rd,0.88,VERIFIED [TEXT] (A/Ci-derived),0.88,abstract/results
Pimentel gm,0.47,VERIFIED [TEXT],0.47 mol m-2 s-1,abstract/results
Wullschleger annual means,75/154,VERIFIED [TEXT],75/154,OUP abstract
Wullschleger P. vulgaris rows,-,NOT FOUND (paywalled),49-91 / 96-165 [SECONDARY via Pimentel],Pimentel 2011
Amax temperate field,-,found,27.0-28.7 at ambient PPFD 25C 400ppm,McKenzie-Gopsill 2020
Rd mature leaf,-,NOT FOUND,-,-
```
