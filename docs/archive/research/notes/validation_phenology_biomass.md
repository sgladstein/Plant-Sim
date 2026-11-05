# Bush-bean phenology and biomass reference data (primary sources)

Compiled 2026-11-05. Access notes: the sandbox's curl egress is blocked for every publisher host tried
(redalyc, scielo, dialnet, udenar, wiley, tandfonline, core, ucr). All full texts below were read through
WebFetch's text extraction or SciSpace. No PDF could be saved locally, so **no figure could be opened as an
image or digitized**. `scratchpad/downloads/phen/` exists but is empty.

Labels used below:
- **[TABLE]**: number printed in a table of the paper
- **[TEXT]**: number stated in the paper's prose or abstract
- **[FIG-TEXT]**: number printed as a data label inside a figure, recovered from the PDF text layer
- **[DERIVED]**: computed by me from published numbers (the formula is given)
- **[NOT FOUND]**: not in the paper, or not readable through the routes I had

---

## 1. Pinzón-Sandoval, Carvalho Silva & Marchiori 2024

- **Citation:** Pinzón-Sandoval, E.H., Carvalho Silva, W., Marchiori, P.E.R. (2024). Phenology, mass accumulation patterns and growing degree days in common bean. *Revista de Ciencias Agrícolas* 41(3): e3239.
- **DOI:** 10.22267/rcia.20244103.239. This is the form on the journal landing page and in the "Cite" line. The PDF header prints 10.22267/rcia.20244102.3239, an internal inconsistency.
- **URLs read:**
  - https://revistas.udenar.edu.co/index.php/rfacia/article/view/9029 (landing page)
  - https://dialnet.unirioja.es/descarga/articulo/10214422.pdf (full text, read via text extraction)
  - The SciELO Colombia copy, http://www.scielo.org.co/scielo.php?script=sci_arttext&pid=S0120-01352024000300004, was blocked by robots.txt.
- **Licence:** CC BY-NC 4.0, stated on the journal landing page. No licence was found in the PDF text.
- **Cultivar:** 'BRSMG Marte', a red-seeded common bean. The paper does not state the growth habit. BRSMG Marte is generally a type II cultivar, but I did not read that in this paper, so check before treating it as type I.
- **Site:** greenhouse, Universidade Federal de Lavras, Lavras, MG, Brazil; 21°14′43″S, 44°59′59″ (printed as "E", which must be W); 919 m altitude.
- **Setup [TEXT]:**
  - 45 pots per period, 4 seeds sown per pot, thinned to 1 plant per pot.
  - 5 plants harvested weekly from germination until R8.
  - Organs were separated into root, stem, leaves and pods, but results are reported as root, "aerial part/shoots (stem + leaves)" and pods. **Leaf and stem are not reported separately.**
  - Drying: 65 °C for about 72 h.
- **Periods:** period 1 = April–June 2023 (autumn); period 2 = July–September 2023 (winter). Exact sowing dates are [NOT FOUND].
- **Base temperature (critical):** **Tb = 8.3 °C**, quoted from the paper: "The base temperature used was 8.3 °C" (citing Barrios-Gómez & López-Castañeda 2009). GDD = Tmean − Tb, with Tmean the daily mean of 30-min readings. No upper threshold is used.

### Table 1. Greenhouse climate by month [TABLE]
```
month,Tmin_C,Tmax_C,Tavg_C,RH_minmax_pct,RH_avg_pct,PAR_avg_umol_m2_s
Apr,14.9,31.1,20.8,71.1-85.5,78.1,398.1
May,8.7,32.8,19.3,65.7-97.2,75.2,417.4
Jun,8.4,30.8,18.1,68.1-88.7,74.3,436.9
Jul,7.1,32.4,18.8,68.1-75.6,72.6,456.9
Aug,10.3,35.9,20.6,51.9-83.0,70.6,428.5
Sep,13.3,35.8,23.3,47.2-91.0,68.2,395.9
```

### Phenology: days and GDD in each stage interval [TABLE/FIG-TEXT]
These values are durations of each interval, not cumulative totals. Each row is the time to reach the stage from the previous stage, starting at V0. The columns sum exactly to the published totals.
```
stage,P1_days,P1_GDD,P2_days,P2_GDD
V1,11,145.8,17,150.7
V2,6,78.9,7,80.8
V3,18,186.6,17,186.2
V4,5,56.3,5,55.8
R5,6,55.9,4,51.2
R6,8,80.9,7,93.5
R7,7,70.7,6,70.4
R8,14,128.4,10,120.0
TOTAL_V0-R8,75,803.5,73,808.6
```

Phase subtotals [TEXT]:
- Vegetative V1–V4: 40 d / 467.6 GDD (period 1) and 46 d / 473.5 GDD (period 2).
- Reproductive R5–R8: 35 d / 335.9 GDD (period 1) and 27 d / 335.1 GDD (period 2). Elsewhere the text gives 329.9 and 330.0.

Cumulative values from sowing to each stage [DERIVED, cumulative sums of the table above]:
```
stage,P1_cumDays,P1_cumGDD,P2_cumDays,P2_cumGDD
V1,11,145.8,17,150.7
V2,17,224.7,24,231.5
V3,35,411.3,41,417.7
V4,40,467.6,46,473.5
R5,46,523.5,50,524.7
R6,54,604.4,57,618.2
R7,61,675.1,63,688.6
R8,75,803.5,73,808.6
```

### Logistic fits Y = a / (1 + exp(−b·(GDD − c))), Y in g per plant, x in accumulated GDD (Tb 8.3) [TABLE 2 = period 1, TABLE 3 = period 2]
"Shoots" means stem + leaves. Every fit is reported with R² = 0.9.
```
period,variable,a,b,c
1,FM_root,21.2483,0.03156,280.51
1,FM_shoots,13.108,0.012,263.17
1,FM_total,34.2026,0.01953,272.38
1,DM_root,2.4150,0.03874,272.38
1,DM_shoots,3.1459,0.0099,347.64
1,DM_total,5.2226,0.01615,300.82
1,FM_pods,27.6291,0.02861,690.16
1,DM_pods,8.8839,0.03952,734.42
2,FM_root,12.059,0.0330,295.73
2,FM_shoots,8.7482,0.01441,250.32
2,FM_total,21.5399,0.02150,286.66
2,DM_root,1.2501,0.0302,301.65
2,DM_shoots,1.6972,0.01373,285.80
2,DM_total,3.0293,0.01795,298.88
2,FM_pods,18.5806,0.0538,711.01
2,DM_pods,5.7600,0.1117,786.50
```
- The total mass fitted here is root + shoot only, and **excludes pods**. The paper never defines it explicitly. The evidence is that a_total (5.22 g in period 1) is about root + shoot, and that the text reports the vegetative total of 5.1 g and pods of 8.37 g separately.
- Note that the paper's root parameters give root ≥ shoot dry mass, which is unusual for pot-grown plants (roots were washed from pots).

### Dry mass maxima stated in the text [TEXT]
```
period,DM_root_max_g,DM_aerial(leaf+stem)_max_g,DM_total_veg_g(at GDD),DM_pods_g(at repro GDD)
1,2.5,2.7,5.1 (467.6),8.37 (329.9 reproductive-phase GDD)
2,1.2,1.7,3.1 (473.5),5.54 (330.0)
```
- Fresh mass maxima [TEXT]: root 21.8 g and aerial 12.71 g (period 1); root 11.4 g and aerial 8.9 g (period 2).
- The text's percentage reductions are inconsistent. It gives "40%" and also "60.7%" for the drop in vegetative total dry mass, and 33.8% / 64.6% for pods.

### Per-date dry mass series
**[NOT FOUND as numbers].** The weekly per-organ points exist only in Fig. 4 (dry mass by organ vs GDD) and Fig. 5 (total dry mass vs GDD). Neither could be rendered, so **nothing was digitized**.

As a stand-in, here are the published logistic fits evaluated at each stage's cumulative GDD [DERIVED from Tables 2–3, g per plant]:
```
period,stage,cumGDD,DM_root,DM_shoot(leaf+stem),DM_total_veg,DM_pods
1,V1,145.8,0.018,0.376,0.395,0.000
1,V2,224.7,0.329,0.719,1.182,0.000
1,V3,411.3,2.404,2.053,4.472,0.000
1,V4,467.6,2.414,2.411,4.892,0.000
1,R5,523.5,2.415,2.677,5.083,0.002
1,R6,604.4,2.415,2.916,5.184,0.052
1,R7,675.1,2.415,3.028,5.210,0.777
1,R8,803.5,2.415,3.112,5.221,8.340
2,V1,150.7,0.013,0.230,0.198,0.000
2,V2,231.5,0.134,0.546,0.696,0.000
2,V3,417.7,1.214,1.459,2.708,0.000
2,V4,473.5,1.243,1.577,2.903,0.000
2,R5,524.7,1.249,1.636,2.978,0.000
2,R6,618.2,1.250,1.680,3.020,0.000
2,R7,688.6,1.250,1.690,3.027,0.000
2,R8,808.6,1.250,1.696,3.029,5.310
```
- These curves are only as good as the fits: R² is rounded to 0.9 and the per-date data are not reproduced.
- The pod fits are very steep, so pod dry mass is about 0 until after R7 and nearly all pod mass appears in the last ~130 GDD.

---

## 2. Cavalcante et al. 2020 (BBCH thermal sum)

- **Citation:** Cavalcante, A.G., Lemos, L.B., Meirelles, F.C., Cavalcante, A.C.P., Aquino, L.A. de (2020). Thermal sum and phenological descriptions of growth stages of the common bean according to the BBCH scale. *Annals of Applied Biology* 176(3): 342–349.
- **DOI:** 10.1111/aab.12571
- **URLs read:**
  - Repository record: https://repositorio.unesp.br/entities/publication/d9989983-be6f-49a4-bca2-bf2cfabae3ff/full. It is marked open access ("green"), but no file is attached.
  - Wiley: inaccessible.
  - **What I actually read:** the first author's doctoral thesis, Chapter 2, "Soma térmica e descrição fenológica de acordo com a escala BBCH para o feijoeiro". Its footnote says the chapter "refere-se ao artigo publicado na Revista Annals of Applied Biology, 2020". Thesis: Cavalcante, A.G. (2020), *Fenologia, atributos produtivos, qualitativos e influência do armazenamento em cultivares de feijoeiro especial e de exportação*, Tese (Doutorado), UNESP/FCAV Jaboticabal. http://hdl.handle.net/11449/202139, PDF: https://repositorio.unesp.br/bitstreams/3dd90c34-e24b-4e0c-9b93-8e8c28948610/download
  - The published article may differ slightly from the thesis chapter.
- **Licence:** the article is Wiley standard (record lists Wiley terms). The thesis licence was not checked.
- **Cultivars:**
  - Type I (determinate): BRS Ártico, BRS Embaixador, IPR Garça.
  - Type II: BRS Executivo, BRS FS 305.
  - Results are pooled by growth habit.
- **Site:** field, UNESP-FCAV Jaboticabal, SP, Brazil; 21°14′33″S, 48°17′10″W; 565 m altitude; Köppen Aw; 2018/2019 agricultural year. Sowing date [NOT FOUND].
- **Base temperature:** **Tb = 10 °C** for the whole cycle [TEXT]. Method of Arnold (1960), using daily Tmax and Tmin. The equation image was unreadable. No upper threshold is stated.

### Thermal sum per stage interval, °C·day [FIG-TEXT, Fig. 3]
The values are printed as data labels on the bars, and each column sums exactly to the stated total.
```
stage,TypeI_GDD,TypeII_GDD
V1_emergence,92.5,92.9
V2_primary_leaves,27.7,28.1
V3_1st_trifoliate,106.0,109.2
V4_3rd_trifoliate,52.3,52.8
R5_flower_buds,85.0,139.1
R6_flowering,66.8,65.0
R7_pod_formation,56.4,54.2
R8_grain_filling,82.7,89.5
R9_maturation,257.4,260.0
STa_total,826.8,890.8
```
- Cycle length [TEXT]: 75 days (type I) and 85 days (type II), for 826.8 and 890.8 °C·day respectively.
- **Days per stage [FIG-TEXT, ambiguous assignment]:**
  - Text-layer order, type I: "3 … 10 11 3 9 6 5 10 18 75"
  - Text-layer order, type II: "3 … 10 10 5 11 6 5 8 27 85"
  - The nine values sum exactly to 75 and 85. The label-to-stage mapping is not certain from extracted text.
  - The mapping most consistent with the GDD/day rate (about 9–11 °C·d per day) is below. Verify against the figure if it can be opened.
```
stage,TypeI_days(best-guess),TypeII_days(best-guess)
V1,10,10
V2,3,3
V3,11,10
V4,3,5
R5,9,11
R6,6,6
R7,5,5
R8,10,8
R9,18,27
total,75,85
```
- Cumulative sums [DERIVED]:
  - Type I GDD: V1 92.5, V2 120.2, V3 226.2, V4 278.5, R5 363.5, R6 430.3, R7 486.7, R8 569.4, R9 826.8.
  - Type II GDD: V1 92.9, V2 121.0, V3 230.2, V4 283.0, R5 422.1, R6 487.1, R7 541.3, R8 630.8, R9 890.8.
- The thesis's Table 1 holds the full BBCH 00–99 descriptions, which I did not reproduce. No biomass data are in this chapter.

---

## 3. Dapaah, McKenzie & Hill 1999 (pinto bean, Canterbury, NZ)

- **Citation:** Dapaah, H.K., McKenzie, B.A., Hill, G.D. (1999). Effects of irrigation and sowing date on phenology and yield of pinto beans (*Phaseolus vulgaris* L.) in Canterbury, New Zealand. *New Zealand Journal of Crop and Horticultural Science* 27(4): 297–305.
- **DOI:** 10.1080/01140671.1999.9514109
- **URLs read:**
  - SciSpace record: https://scispace.com/papers/effects-of-irrigation-and-sowing-date-on-phenology-and-yield-4coasw55bo
  - Full text: https://scispace.com/pdf/effects-of-irrigation-and-sowing-date-on-phenology-and-yield-4coasw55bo.pdf
  - Abstract: https://informahealthcare.com/doi/abs/10.1080/01140671.1999.9514109
- **Licence:** not stated. SciSpace flags it open access (T&F).
- **Cultivar:** 'Othello' (pinto). Its habit is not stated in what I read; Othello is generally described as type III.
- **Site:** field, Lincoln University, Canterbury, NZ, on Wakanui silt loam; 60 plants/m².
- **Sowing dates:** 1994/95: 27 Oct and 24 Nov. 1995/96: 1, 15 and 29 Nov and 13 Dec. Each sowing had nil and full irrigation.
- **Base temperature:** **Tb = 7 °C** [TEXT].
  - Derived from the E–F development-rate regression Y = −2.44 + 0.35T (r² = 0.99) and applied to all phases.
  - TT = Σ((Tmax + Tmin)/2 − 7).
  - Irrigation had little effect on phase durations, so the values are pooled over irrigation.

### Table 2. Thermal time, °C·day above 7 °C [TABLE]
```
phase,27Oct94,24Nov94,SEM94,01Nov95,15Nov95,29Nov95,13Dec95,SEM95
S-E,57.6,69.7,0.56,66.7,67.7,110.4,83.9,1.43
E-F,296.4,313.9,1.53,280.3,292.3,313.6,338.1,3.88
F-Pd,109.6,69.2,1.42,88.6,57.9,75.5,78.3,0.93
Pd-Pm,373.5,451.9,5.10,368.5,409.9,377.9,372.1,2.50
Pm-H,173.9,119.6,4.24,143.2,102.6,109.9,63.8,9.66
S-H,1010.2,1025.0,1.46,954.6,930.4,987.4,936.3,11.04
```
- Means [TEXT, abstract]: E–F 306, F–podding 79, podding–physiological maturity 392, S–H 974 °C·day. The text also gives S–E ≈ 76 and Pm–H ≈ 119.
- Days [TEXT]: E–F took 36 d for October sowings and 28 d for mid-November sowings, at mean temperatures of 15.1 °C and 17.4 °C. No full days-per-phase table [NOT FOUND].
- Table 1 in the paper gives monthly Tmax/Tmin and rainfall for both seasons (captured, not reproduced here).
- Related sources:
  - Dapaah's PhD thesis (Lincoln Univ., 1997), per the AGRIS abstract: with **Tb = 10 °C**, flowering 204 and maturity 659 °C·day; RUE 1.20 g DM per MJ intercepted PAR.
  - Companion growth paper: Dapaah, McKenzie & Hill (2000), *J. Agric. Sci.* 134: 33–43. It reports total DM at harvest of 453–579 g/m² (1994/95) and 371–461 g/m² (1995/96). Leaf and stem DM peaked about 24 d after flowering and fell to about 73% (leaf) and 85% (stem) of maximum by maturity [TEXT]. The organ time series is in Figs. 4–5 only [NOT DIGITIZED].

---

## 4. Ventura Elías (published 1991; the OJS record is dated 2016)

- **Citation:** Ventura Elías, R. (1991). Fenología y fenometría de una variedad y una línea de frijol (*Phaseolus vulgaris*) en la zona occidental de El Salvador. *Agronomía Mesoamericana* 2: 56–60. CENTA, El Salvador. **The plan's "2016" is the OJS upload date; the volume is 1991.**
- **DOI:** 10.15517/am.v2i0.25224
- **URLs read:**
  - https://archivo.revistas.ucr.ac.cr/index.php/agromeso/article/view/25224
  - PDF: https://archivo.revistas.ucr.ac.cr/index.php/agromeso/article/download/25224/25486
- **Licence:** CC BY-NC-ND 4.0, per the journal footer.
- **Cultivars:** 'Rojo de Seda' (variety) and 'APN 83' (line).
- **Sites:** field, Ahuachapán department, western El Salvador; second-season (postrera) sowing, roughly Sept–Dec, after maize.
  - 1987: Joya del Zapote, Turín, 615 m.
  - 1988: Salitrero, Atiquizaya, 630 m.
  - 1989: Izcaquilío, Atiquizaya, 640 m.
  - Spacing 0.90 × 0.25 m.
- **Base temperature:** **18.9 °C** in the abstract and conclusions; the Cuadro 7 caption says "Base 18 °C" (inconsistent).
  - Heat units follow González (1985), which uses Tmax, Tb, the daily range and K = 0.25.
  - Because of this high Tb and unusual method, the "heat units" are **not comparable** with standard Tb 8–10 °C GDD.

### Mean of both genotypes and 3 years [TEXT, abstract and results]
```
stage,DAS,heat_units(Tb18.9)
emergence,4.5,12.89
guide(branch)_formation,18.3,61.09
flowering,28.7,89.08
pod_formation,31.8,99.31
pod_filling,37.3,114.97
maturity,54.2,157.74
```

### Cuadro 7. Heat units per genotype (cumulative from sowing; caption says base 18 °C) [TABLE]
```
stage,Rojo_de_Seda,APN_83
emergence,12.89,12.89
guide_formation,64.41,57.78
flowering,91.33,88.44
pod_formation,99.31,99.31
pod_filling,112.77,117.18
maturity,153.24,162.25
```
- Days after sowing per genotype: [NOT FOUND] (only the pooled means above).
- Rojo de Seda is reported as about 5.5% earlier.
- Phenometry (plant height, green leaves, stems, flowers and pods over time, measured 3× per week) is said to be in Cuadros 1–2, but those tables were not in the extracted text [NOT FOUND].
- **No dry weight or leaf area measured.** Cuadro 8 has decadal rainfall vs crop water demand (ETM), not reproduced here.

---

## 5. Baldinotti, Conceição et al. 2026 (listed in the plan as "Conceição 2026")

- **Citation:** Baldinotti, H. da S., Conceição, M.P. da, Kondo, Y.R., Ferreira, E.P. de B., Hungria, M., Kaschuk, G. (2026). Interaction among growth cycle, photosynthesis, and nodule enzymatic activity in *Phaseolus vulgaris* L. inoculated with *Rhizobium tropici* CIAT 899. *Annals of Applied Biology* (issue 2, published online about 30 Jun 2026). **Conceição is the second author; the first author is Baldinotti.**
- **DOI:** 10.1111/aab.70142
- **URLs read:**
  - https://www.citedrive.com/en/discovery/interaction-among-growth-cycle-photosynthesis-and-nodule-enzymatic-activity-in-iphaseolus-vulgarisi-l-inoculated-with-irhizobium-tropicii-scpciatscp-/
  - SciSpace record: https://scispace.com/papers/interaction-among-growth-cycle-photosynthesis-and-nodule-llcwlzokpkc7
  - **Abstract only.** No open-access full text was found; Wiley was unreachable.
- **Licence:** not found.
- **Conditions:** greenhouse. Treatments: uninoculated without N; N-fertilized; inoculated with *R. tropici* CIAT 899.
- **Cultivars and cycle lengths [TEXT, abstract]:**
```
cultivar,cycle_class,cycle_days
BRS FC104,super-early,65
BRS Pontal,intermediate,87
BRS Vereda,long,93
```
- Other abstract numbers: GOGAT at R6 was 6.8 (inoculated), 5.3 (control) and 4.8 (N-fertilized) µmol NADH min⁻¹. Petiole N-ureides under N fertilization were 37.7 (Pontal), 15.2 (FC104) and 17.0 (Vereda) nmol g⁻¹.
- Shoot and root biomass were measured **only at V4 and R6**, so this is not a time series. The values are [NOT FOUND] (full text paywalled).

---

## 6. Other bean growth time series found

### 6a. Maldonado & Corchuelo 1993: ICA-Cerinza (type I, determinate) vs ICA-Tundama (type II)
- **Citation:** Maldonado, G., Corchuelo, G. (1993). Dinámica de crecimiento de dos variedades de frijol (*Phaseolus vulgaris* L.). I. Análisis de crecimiento por periodos fijos. *Agronomía Colombiana* 10(2): 114–121.
- **URLs read:**
  - Text copy: https://distantreader.org/stacks/journals/agrocol/agrocol-21269.pdf
  - Journal page: https://revistas.unal.edu.co/index.php/agrocol/article/view/21269
  - The text is OCR.
- **Licence:** journal is open access (DOAJ-listed); specific licence not checked.
- **Conditions:** field, Mosquera (Cundinamarca, Colombia), 2640 m altitude, **mean temperature 14 °C**, last quarter of 1992. Spacing 0.50 × 0.12 m (166,000 plants/ha). Fertilizer 50-50-50 NPK.
- **Sampling:** 3 plants per plot × 4 reps, every 7 d from 15 to 78 days after emergence (DAE). The study covers the vegetative phase.

Cuadro 1. Total leaf area per plant and LAI, mean of 12 plants [TABLE]:
```
DAE,Cerinza_LA_cm2,Cerinza_LAI,Tundama_LA_cm2,Tundama_LAI
15,77.99,0.129,63.15,0.105
22,118.09,0.196,111.19,0.186
29,172.52,2.287*,166.73,0.277
36,309.87,0.516,336.82,0.561
43,449.77,0.749,468.99,0.781
50,706.81,1.178,712.12,1.186
57,1153.45,1.922,1202.47,2.004
64,1727.47,2.879,1897.89,3.163
71,1876.76,3.127,2362.32,3.937
78,2924.69,4.874,3183.73,5.306
```
\* 2.287 is almost certainly a misprint for 0.287 (LA/LAI ratio).

- Dry weights (total, stem, leaf) are in Fig. 1 only [NOT DIGITIZED].
- The OCR'd quadratic fits are garbled; for example, Cerinza total dry weight PST = 3.56 − 0.29t + 0.006t² (R² 0.98). **Do not use these without checking the original.**
- Mean RGR [TEXT]: 0.0455 g g⁻¹ d⁻¹ (Cerinza) and 0.0437 g g⁻¹ d⁻¹ (Tundama).

### 6b. Medeiros et al. 2000, PAB 35(9): 1733–1742 (Carioca 80-SH, Campinas; type III, so less relevant)
- **DOI:** 10.1590/S1678-3921.pab2000.v35.5954. Read at https://apct.sede.embrapa.br/pab/en/article/download/5954/3058/20846
- **Conditions:** Tb 10 °C. Sown 16 Aug 1994, emerged 26 Aug, harvested 15 Nov 1994.
- **Values [TEXT]:**
  - Shoot DM 157 g/m² at 298 GDD (25 DAE).
  - 725 g/m² at harvest (82 DAE, 1170 GDD), 25 plants/m².
  - Field crop at 28 plants/m²: maximum 602 g/m² at 67 DAE.
  - Total DM plateaued at about 950–1050 GDD.
- Tables 3–4 (organ DM, stage GDD) were not recoverable from the text layer [NOT FOUND].

### 6c. Identified but not readable here
- Lépiz Ildefonso, Chavarín Espinoza & Rodríguez Guzmán (2015), "Fenología y producción de materia seca en variedades de frijol arbustivo de diferente hábito de crecimiento", *e-CUCBA* No. 3, DOI 10.32870/e-cucba.v0i3.30.
  - It includes **'Peruano Mostaza', type I**, with dry-matter sampling at the start of each stage and vegetative vs reproductive DM.
  - Abstract [TEXT]: cycle 88–93 DAS; Peruano Mostaza's maximum crop growth rate was 9.13 g m⁻² d⁻¹ (3rd trifoliate to pre-flowering).
  - The e-cucba.cucba.udg.mx PDF is blocked by robots.txt. **This is the best candidate for a tabulated type I series; worth retrieving manually.**
- Neri-Luna et al. (2019), "Phenology and biomass distribution pattern in two varieties of common bean", ECORFAN *J. Natural & Agricultural Sciences*, DOI 10.35429/JNAS.2019.19.6.18.27.
  - Covers 'Black Valentine' and 'Black Wax' snap beans (bush), with per-organ fresh/dry weight and leaf area every 10 DAP.
  - Full text not attempted or reached.
- Andrade et al. (2009), *Acta Sci. Agron.* (Ouro Negro, BRS MG Talismã; samples at 12, 22, 36, 51, 65 and 76 DAE, organ biomass). These cultivars are indeterminate. Not read.
