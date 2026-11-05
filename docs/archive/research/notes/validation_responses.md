# Bush-bean stress-response reference data (validation targets)

Compiled 2026-11-05. Every number comes from the primary paper (publisher page, SciELO, PMC full text, or the publisher abstract as indexed by SciSpace/Consensus). Nothing is from memory.
Relative change = (treatment − control) / control, unless the paper states its own %.

Source-quality tags:
- **[TABLE]**: read from a table in the full text.
- **[TEXT]**: stated in the full-text Results or Discussion.
- **[ABSTRACT]**: only the abstract was reachable (no full text).
- **[FIG-NOT-DIGITIZED]**: the value is in a figure only and was not digitized (WebFetch cannot read figure images).
- **[NOT FOUND]**: no reachable source gives it.

Access notes:
- Direct curl to publisher hosts is blocked by the sandbox proxy (403 CONNECT), so pages were read with WebFetch.
- PMC full texts came through the PubMed MCP. They are saved, as untrusted data, in `scratchpad/downloads/resp/`: vargas2021.txt, sharda2026.json, sharda2026.txt.

---

## A. Drought from flowering

### A1. Gomes et al. 2000 (starting paper)
- **Citation:** Gomes, A.A.; Araújo, A.P.; Rossiello, R.O.P.; Pimentel, C. (2000). Acumulação de biomassa, características fisiológicas e rendimento de grãos em cultivares de feijoeiro irrigado e sob sequeiro. *Pesquisa Agropecuária Brasileira* 35(10):1927–1937.
- **DOI:** 10.1590/S0100-204X2000001000003 (DOAJ). The Embrapa landing page lists 10.1590/S1678-3921.pab2000.v35.5979.
- **URLs read:** https://apct.sede.embrapa.br/pab/article/download/5979/3083/20871 (PDF), https://apct.sede.embrapa.br/pab/article/view/5979, https://doaj.org/article/2793d7c3d81247898c04cb53e0e5048a
- **Licence:** not stated.
- **Cultivars:** Xodó (type II), Carioca, Negro Argel (type III), line A320 (type III).
- **Design:** field, Seropédica RJ, Brazil. Sown 5 May 1992. Rainfed (123 mm total water) vs irrigated (252 mm; irrigation when soil tension fell below −0.03 MPa).
- **Timing:** the deficit was not imposed at a set stage. It built up progressively in the dry season. Shoot dry mass started to diverge at 28–35 DAS, full flowering was at 36–39 DAS, and pod formation began at 40–44 DAS, so the timing is a fair analogue of "withheld from about day 30". Harvest was at 98 DAS.

| Trait | Result | Tag |
|---|---|---|
| Pods/plant | The most affected yield component; reduced in all cultivars. Carioca about −60%. | [TEXT] |
| Seed yield | Carioca about −60%. Two cultivars about −16% and −48% (the text links these to Negro Argel and A320, but the Negro Argel loss was not significant). Reduced in all cultivars except Negro Argel. | [TEXT] |
| Seeds/pod | Reduced, but not significantly. | [TEXT] |
| 100-seed weight | Increased slightly, not significantly. Seed size is conserved. | [TEXT] |
| Crop growth rate | Carioca peak CGR about 4 g m⁻² d⁻¹ rainfed vs about 15 g m⁻² d⁻¹ irrigated, i.e. **about −73%**. Peak shifted from 49 to 56 DAS. | [TEXT] (Fig. 2) |
| LAI | Reduced by stress in all cultivars; peak at about 63 DAS (Carioca 56 DAS). No values given. | [FIG-NOT-DIGITIZED] (Fig. 3) |
| Shoot dry mass | Reduced in all cultivars. Carioca and Xodó peaked 7 days earlier under stress. | [FIG-NOT-DIGITIZED] (Fig. 1) |
| Leaf, stem and pod dry mass by date; leaf area duration | Table 1 exists, but its numbers are garbled in the PDF text layer. | [NOT FOUND] (Table 1) |
| Table 2 numbers (yield kg/ha, pods/plant, seeds/pod, 100-seed mass) | Garbled in the PDF text layer. The SciELO HTML returned server errors. | [NOT FOUND] (Table 2) |
| Stem dry mass decline from 70 DAS to the end, stressed vs irrigated | Negro Argel 24 vs 52%; Xodó 39 vs 52%; Carioca 43 vs 36%; A320 26 vs 30%. | [TEXT] |
| Root:shoot ratio | Not measured. | — |

### A2. Guerrero-Domínguez et al. 2023 (starting paper)
- **Citation:** Guerrero-Domínguez, L.; González-Pérez, B.L.; Jerez-Mompie, E.I.; Morales-Guevara, D.; Dell'Amico-Rodríguez, J.; Santa Cruz-Suárez, A. (2023). Behavior of common bean plants (*Phaseolus vulgaris* L.) subjected to two irrigation systems. *Revista Ciencias Técnicas Agropecuarias* 32(4): e06.
- **DOI:** not shown on the page.
- **URL read:** https://www.redalyc.org/journal/932/93276285006/html/
- **Licence:** not stated on the page (Redalyc hosting is typically CC BY-NC; not verified).
- **Cultivar:** Triunfo 70 (black bean).
- **Design:** concrete gutters of 1.56 m² filled with Ferralitic Red soil, semi-controlled conditions, INCA, Cuba, Jan–Apr 2021.
- **Treatments:** R100 (100% ETc) vs R50 (50% ETc). The deficit ran from about 5 days after sowing to the end, i.e. it was not flowering-specific. Soil moisture was about 32–33% vs 21–23% (dry-mass basis).

| Trait | Result | Tag |
|---|---|---|
| Pods/plant | Significantly lower in R50 | [FIG-NOT-DIGITIZED] (Fig. 7A) |
| Seeds/pod | No significant difference | [FIG-NOT-DIGITIZED] (Fig. 7B) |
| Seeds/plant | Significantly lower in R50 | [FIG-NOT-DIGITIZED] (Fig. 8A) |
| 100-seed mass | No significant difference | [FIG-NOT-DIGITIZED] (Fig. 8B) |
| Yield (g/plant) | Significantly lower in R50 | [FIG-NOT-DIGITIZED] (Fig. 8C) |
| Seed length, width, thickness | No significant difference | Fig. 9 |
| Stem and leaf dry mass | Lower in R50 at 28, 42 and 61 DAS (except stem at 42 DAS) | [FIG-NOT-DIGITIZED] (Fig. 4) |
| Leaf area | Equal at 28 DAS; lower in R50 at 42 and 61 DAS | [FIG-NOT-DIGITIZED] (Fig. 3) |
| LAI, CGR, root mass, root:shoot | Not measured | — |

The text gives no numbers. Magnitudes need figure digitizing from the PDF.

### A3. Kazai et al. 2019 (added; stress from first flower buds)
- **Citation:** Kazai, P.; Noulas, C.; Khah, E.; Vlachostergios, D. (2019). Yield and seed quality parameters of common bean cultivars grown under water and heat stress field conditions. *AIMS Agriculture and Food* 4(2):285–302.
- **DOI:** 10.3934/agrfood.2019.2.285
- **URL read:** https://aimspress.com/aimspress-data/aimsagri/2019/2/PDF/agrfood-04-02-285.pdf
- **Licence:** CC BY 4.0.
- **Cultivars:** Pyrgetos, Iro, Great Northern, Cannellino.
- **Design:** field, Larissa, Greece, 2014, drip irrigation.
- **Treatments:** NWT (300 mm) vs SWT (150 mm, 50%). The cut started 25 days after emergence, when the first flower buds were visible. Section 3.1 places it at 36 DAS. It lasted until the end of pod filling.
- **Confounder:** a heat wave near 40 °C on 25–27 June hit both treatments.

**Table 1 [TABLE]** (mean of 4 cultivars):

| Trait | NWT (control) | SWT (stressed) | Relative change |
|---|---|---|---|
| Seed yield (kg/ha) | 819 | 340 | **−58%** |
| Pods/plant | 8.7 | 4.1 | **−53%** |
| Seeds/pod | 2.2 | 2.0 | **−10%** (paper: −9.7%) |
| 100-seed weight (g) | 27 | 22 | **−18%** |
| Harvest index | 0.21 | 0.11 | −49% |
| Plant height (cm) | 51.5 | 48.1 | −7% |

- Cultivar range for pods/plant: Cannellino −8%, Great Northern −31%, Pyrgetos −80%, Iro −95%.
- Biomass and LAI were not reported.

### A4. Rezene et al. 2012 (added; drought initiated at flowering)
- **Citation:** Rezene, Y.; Gebeyehu, S.; Zelleke, H. (2012/2013). Morpho-physiological response to post-flowering drought stress in small red seeded common bean (*Phaseolus vulgaris* L.) genotypes. *Journal of Plant Studies* 2(1):42.
- **DOI:** 10.5539/jps.v2n1p42
- **URL read:** https://ccsenet.org/journal/index.php/jps/article/view/21338 (the PDF returned binary only)
- **Licence:** CC BY 4.0.
- **Design:** 49 genotypes in the field at Gofa, Ethiopia. Irrigation was withheld from flowering.
- **Results [ABSTRACT]** (mean of 49 genotypes):
  - seed yield **−59%** (genotype range −9% to −89%)
  - 100-seed weight **−19%**
  - seeds/plant −29%
  - seeds/pod −15%
  - harvest index −39%
  - LAI under drought: ECAB-0427 (tolerant) 2.8 vs REDWOLAITA (susceptible) 0.8. No control LAI given.
- Pods/plant: [NOT FOUND] in the abstract.

### A5. Hinkossa et al. 2013 (added; post-flowering drought)
- **Citation:** Hinkossa, A.; Gebeyehu, S.; Zelleke, H. (2013). Differential effects of post-flowering drought stress on growth and yield of the basic generations of two common bean populations. *STAR Journal of Science, Technology and Arts Research* 2(1):22–31.
- **URL read:** https://ajol.info/index.php/star/article/view/98839/88098
- **Results [TEXT]:**
  - pods/plant 31.1 → 25.6 (**−18%**, Roba-1 × SER-16) and 22.97 → 18.48 (−20%, Melka-Dima × SAB-623)
  - seed yield 3535 → 2598 kg/ha (−27%) and 3956 → 2697 kg/ha (−32%); abstract mean −28%
  - seed weight was not reported for this trial.
- This is a mild stress (yield only −28%).

### A6. Root:shoot ratio under drought (added)

| Study | Conditions | Root:shoot change | Other results | Tag |
|---|---|---|---|---|
| **Sofi, Djanaguiraman, Siddique, Prasad (2018)** *Indian J. Plant Physiol.* 23:796–809. DOI 10.1007/s40502-018-0429-x. URL: https://link.springer.com/article/10.1007/s40502-018-0429-x. Paywalled. | 150-cm columns for 48 d; water withheld for 41 d. | **+70%** | root biomass −29%; pod set −53%; pod weight −43%; root length −35%; rooting depth −14% | [ABSTRACT] |
| **Acosta-Díaz et al. (2007)** *Agricultura Técnica en México* 33(2):153–162. URL: http://www.scielo.org.mx/scielo.php?script=sci_arttext&pid=S0568-25172007000200005. CC BY-NC 3.0. | Greenhouse, 3-kg pots; Flor de Junio lines UAZ FJ 2, 3, 10 + landrace. SMV+SSR = 75% of ET replaced in the vegetative stage, then 50% in the reproductive stage. | **+42%** | shoot DW −26%; root DW −44%; yield −52%. Graded root DW: −29% (75% ET), −33% (50% ET), −44% (SMV+SSR); yield −41%, −46%, −52%. Field rainfed vs supplemented: shoot DW −34%, yield −48%. | [TEXT] (Cuadros 2–5 are images) |
| **Riyaz et al. (2025)** *Scientific Reports*. DOI 10.1038/s41598-025-04635-8. | 45 genotypes, column culture. Seedling to vegetative stage, so not reproductive. | **+89%** | shoot biomass −60%; root biomass −32% | [ABSTRACT] |

---

## B. Heat at flowering

### B1. Gross & Kigel 1994 (starting paper)
- **Citation:** Gross, Y.; Kigel, J. (1994). Differential sensitivity to high temperature of stages in the reproductive development of common bean (*Phaseolus vulgaris* L.). *Field Crops Research* 36(3):201–212.
- **DOI:** 10.1016/0378-4290(94)90112-0
- **URLs read:** https://cris.huji.ac.il/en/publications/differential-sensitivity-to-high-temperature-of-stages-in-the-rep/. ScienceDirect refused the fetch.
- **Licence:** subscription.
- **Cultivars:** two determinate lines, PI-271998 and BBL-47 (heat-sensitive).
- **Treatments:** control 22/17 °C; HT 32/27 °C for **1 or 5 days**, applied at successive flower-development stages.
- **Results [ABSTRACT]:**
  - Microsporogenesis-stage HT gave non-viable pollen and no anther dehiscence.
  - Near anthesis, HT reduced pollen-tube penetration and pod and seed set. The female-side effect peaked at anthesis.
  - Ovules nearest the peduncle failed most.
- Pod-set and seed-set percentages: [NOT FOUND] (full text unavailable).

### B2. Sharda et al. 2026 (starting paper; plan values confirmed)
- **Citation:** Multi-trait phenotyping under reproductive-stage heat stress reveals heritable reproductive and yield indicators of tolerance in common bean (*Phaseolus vulgaris* L.). *Frontiers in Plant Science* (2026).
- **DOI:** 10.3389/fpls.2026.1904240. PMID 42643273, PMC13503261.
- **URL read:** PMC full text via PubMed MCP (saved as downloads/resp/sharda2026.txt).
- **Licence:** Frontiers, CC BY (standard for the journal; not re-checked on the page).
- **Material:** 56 NBPGR accessions.
- **Design:** 4-kg pots, three plants per pot, Panjab University, Chandigarh, India; 2023 and 2024 seasons.
- **Treatment:**
  - At bud initiation, plants moved to a walk-in chamber.
  - Temperature was ramped +1 °C/day up to **32/22 °C (day/night)**, then held for **7 days** over flowering to podding.
  - Control 25/15 °C; 12-h photoperiod; 600 µmol m⁻² s⁻¹; 65–70% RH.

**Results** [TEXT/ABSTRACT; per-accession values are in Supplementary Tables]:

| Trait | Tolerant 4 accessions | Sensitive 4 accessions |
|---|---|---|
| Total seed weight/plant | **about −50%** | **about −71%** |
| Single seed weight | **−8.13% / −8.12%** (2023 / 2024) | **−34.16% / −34.12%** |
| Harvest index | −23.25% / −24.69% | −54.40% / −55.24% |
| Pollen viability | −55 to −59% | −64 to −72% |
| Pollen germination | −59 to −65% | −71 to −79% |
| Electrolyte leakage | +11.2 / +12.7% | +43.9 / +42.4% |

- **Filled pods/plant and seeds/plant:** the text says only that reductions were larger in sensitive accessions. Values are in Fig. 5 and the supplementary tables: [FIG-NOT-DIGITIZED].
- Biomass declined similarly in both groups (no % given).

### B3. Prasad, Boote, Allen & Thomas 2002 (starting paper)
- **Citation:** Prasad, P.V.V.; Boote, K.J.; Allen, L.H. Jr; Thomas, J.M.G. (2002). Effects of elevated temperature and carbon dioxide on seed-set and yield of kidney bean (*Phaseolus vulgaris* L.). *Global Change Biology* 8(8):710–721.
- **DOI:** 10.1046/j.1365-2486.2002.00508.x
- **URLs:** abstract read via https://bvbvf1.bib-bvb.de/vufind/PrimoRecord/cdi_proquest_miscellaneous_17847684 and the SciSpace record. Wiley full text refused the fetch.
- **Licence:** subscription.
- **Cultivar:** red kidney bean 'Montcalm'.
- **Design:** controlled environment, day/night 28/18 to 40/30 °C, at 350 or 700 µmol mol⁻¹ CO₂. These are whole-season regimes, not short shocks.

**Temperature response [ABSTRACT]** (slopes only; the per-temperature table was not reachable):

| Trait | Response over 28/18 → 40/30 °C |
|---|---|
| Seed-set | **−6.5% per °C** (both CO₂ levels) |
| Seeds per pod | **−0.34 per °C** |
| Seed number | −2.3 (ambient) / −3.3 (elevated) seeds per °C above 28/18 |
| Seed yield | −1.1 (ambient) / −1.5 (elevated) g plant⁻¹ per °C above 28/18 |
| Seed size | no change up to 31/21; **−0.07 g per °C above 31/21** |
| Photosynthesis | −0.3 / −0.9 µmol m⁻² s⁻¹ per °C |
| Elevated CO₂ effect | photosynthesis about +50%, yield about +24%, no CO₂ × T interaction |
| Mechanism | lower pollen production and viability |

Implied magnitudes (derived by me from the slopes, not read from a table):
- Seed-set falls about 19 percentage points at 31/21 and about 39 points at 34/24. It reaches near zero at 40/30 if the slope is in absolute percentage points.
- Seeds/pod falls by 2 at 34/24.
- The authors' conclusion is that yield losses are likely above 34/24 °C even with CO₂ enrichment.
- Caution: 0.07 g/°C × 9 °C = 0.63 g, which is larger than a typical Montcalm seed (about 0.5 g). The unit may be per seed on a different basis. Check Fig. or Table in the full text before using it.

**Not found:** the tabulated per-regime values. Full text needed (Wiley, paywalled).

### B4. Rose et al. 2023 (starting paper; the "31/24 threshold" in the plan is not supported)
- **Citation:** Rose, T.; Lowe, C.; Miret, J.A.; Walpole, H.; Halsey, K.; Venter, E.; Urban, M.O.; Buendia, H.F.; Kurup, S.; O'Sullivan, D.; Beebe, S.; Heuer, S. (2023). High temperature tolerance in a novel, high-quality *Phaseolus vulgaris* breeding line is due to maintenance of pollen viability and successful germination on the stigma. *Plants* (MDPI) 2023. Final volume and DOI were not on the manuscript version read.
- **URL read:** https://repository.rothamsted.ac.uk/download/9edbfdc6f6b0c7950ba69f462744464cf59e98230f5f32e3771431ab8e123df0/10404638/MDPI_Article_Template_Rose_et_al_2023.pdf (accepted-manuscript template)
- **Licence:** CC BY 4.0.
- **Cultivars:** Calima (heat-sensitive Andean bush) and HTA4 (tolerant line).
- **Design:** 8-inch pots in a growth room.
- **Treatment:** control 26/19 °C vs HS **31/24 °C**, chronic from 7 days after germination to pod maturity, so not a flowering pulse.
- **Note on the plan:** **31/24 °C is simply the HS treatment. The paper does not identify it as a damage-onset threshold.** The Introduction cites more than 30 °C day or more than 20 °C night as damaging (Rainey & Griffiths).

**Results [TEXT/FIG values]:**

| Trait | Calima, control → HS | HTA4, control → HS |
|---|---|---|
| Total seed weight/plant | 12.5 → 5 g (**−60%**) | about 13 → 10.5 g (**about −19%**) |
| Pod number/plant | 13.5 → 21.4 (**+59%**; many short, empty pods) | 15.1 under HS, no change |
| Pollen viability | 98.4 → 24.9% (**−75%**) | 95.4 → 71.1% (−25%) |
| Vegetative dry weight | +35% | +13% |
| Plant height | +67% at day 20 | +20% at day 35 |
| Pod length | about 12 → about 4 cm | — |

- Single seed weight: [NOT FOUND] (shown qualitatively only).

### B5. Vargas et al. 2021 (starting paper; night temperature)
- **Citation:** Vargas, Y. et al. (2021) (full author list not verified from the fetched text). Physiological and genetic characterization of heat stress effects in a common bean RIL population. *PLoS ONE* 16(4): e0249859.
- **DOI:** 10.1371/journal.pone.0249859. PMC8084131.
- **URL read:** PMC full text via PubMed MCP (saved as downloads/resp/vargas2021.txt).
- **Licence:** CC BY (PLoS).
- **Material:** 107 RILs of IJR × AFR298 (ICA Quimbaya); Andean bush types.
- **Design:** field.
  - HS: Alvarado, Tolima, 460 m a.s.l., Jul–Oct 2016 and 2017.
  - NS: Palmira, 973 m a.s.l., 2017.
  - HS mean Tmin and Tmax were **3–7 °C above** NS for the whole season. Exact distributions are in Fig. 4 [FIG-NOT-DIGITIZED].
  - HS2016 had the highest night temperatures during the reproductive phase; HS2017 had the highest day maxima.

**Table 1 [TABLE]** (population means, NS2017 → HS2016 / HS2017):

| Trait | NS | HS2016 (hot nights) | HS2017 (hot days) |
|---|---|---|---|
| Yield (kg/ha) | 1512.5 | 954.2 (**−37%**) | 1112.4 (**−26%**) |
| Seed weight/plant (g) | 9.41 | 6.10 (−35%) | 7.42 (−21%) |
| Seeds/plant | 27.26 | 23.76 (−13%) | 21.81 (−20%) |
| Seeds/pod | 3.26 | 1.93 (**−41%**) | 2.38 (−27%) |
| Pods/plant | 8.22 | 12.67 (**+54%**) | 9.22 (+12%) |
| 100-seed weight (g) | 34.49 | 25.54 (**−26%**) | 33.64 (−2%) |
| Pod harvest index (%) | 70.0 | 59.15 (−15%) | 69.89 (0%) |
| Pollen viability (%) | 90.57 | 75.16 (−17%) | 74.95 (−17%) |
| Pods with well-formed seed (%) | 69.5 | 31.2 | 56.3 |
| Empty pods (%) | 0.84 | 8.79 | 6.65 |
| Days to flowering | 33.9 (NS2018) | 38.5 | 38.1 |

- Stated conclusion: high night temperatures had larger effects than high day temperatures. Single seed weight fell only in the hot-night trial.
- Background threshold cited in the paper: more than 20 °C night or more than 30 °C day reduces yield.

### B6. Silva et al. 2020 (added; chamber, continuous HT from pre-flowering)
- **Citation:** Silva, D.A.; Pinto-Maglio, C.A.F.; Oliveira, E.C.; Reis, R.L.M.; Carbonell, S.A.M.; Chiorato, A.F. (2020). Influence of high temperature on the reproductive biology of dry edible bean (*Phaseolus vulgaris* L.). *Scientia Agricola* 77(3): e20180233.
- **DOI:** 10.1590/1678-992X-2018-0233
- **URL read:** https://new.scielo.br/j/sa/a/hhMF7qwrTn5H5JPkdkphXDb/?lang=en
- **Licence:** CC BY 4.0.
- **Material:** 12 Brazilian cultivars (e.g. Pérola, IAC Imperador).
- **Design:** 5-L pots in a chamber. Control 25/20 °C; HT **37/26 °C** from pre-flowering (the abstract says V4) to maturity.

**Table 2 / text [TABLE/TEXT]** (mean of 12 cultivars):

| Trait | Control | HT | Relative change |
|---|---|---|---|
| Flowers/plant | 38.17 | 22.08 | −42% |
| Pods/plant | 12.50 | 9.28 | **−26%** |
| Seeds/plant | 55.69 | 34.81 | −37% |
| 100-seed weight (g) | 29.14 | 21.09 | **−28%** |
| Seed yield (g/plant) | 16.39 | 7.46 | **−54%** |
| Pod set | about 35% | about 47% | higher under HT, because fewer flowers formed |
| Pollen grains | 511 | 367 | −28% |

---

## C. 50% shade

### C1. Hanyu, Shoji & Ji 1996 (starting paper)
- **Citation:** Hanyu, H.; Shoji, K.; Ji, S.-B. (1996). Evaluation of light quality variation through supplement of far-red light and the difference in the effects on growth of a pole-type and a bush-type kidney bean, *Phaseolus vulgaris* L. *Environment Control in Biology* 34(4):267–275.
- **DOI:** 10.2525/ecb1963.34.267
- **URL read:** https://www.jstage.jst.go.jp/article/ecb1963/34/4/34_4_267/_article/-char/en. J-STAGE robots.txt blocks the PDF.
- **Licence:** J-STAGE "Free Access"; copyright JSABEES.
- **Cultivars:** Kentucky Wonder (pole) and **Celina (bush)**.
- **Treatment:** far-red (>700 nm) added to fluorescent light.
- **Results [ABSTRACT]:**
  - Stem length, stem DW and the stem fraction of total DW increased as Pfr/P fell, in both cultivars.
  - The stem-length increase was steep while Pfr/P stayed above **0.77** and slight below it.
  - Pfr/P is a rectangular hyperbola in R/FR.
  - **Pole: longer internodes plus more nodes. Bush (Celina): longer internodes only.**
- Numeric stem lengths and % changes: [NOT FOUND] (PDF blocked).

### C2. Cossu et al. 2021 (dwarf green bean, PV greenhouse with 50% PV cover, i.e. −45% PAR)
- **Citation:** Cossu, M.; Sirigu, A.; Deligios, P.A.; Farci, R.; Carboni, G.; Urracci, G.; Ledda, L. (2021). Yield response and physiological adaptation of green bean to photovoltaic greenhouses. *Frontiers in Plant Science* 12:655851.
- **DOI:** 10.3389/fpls.2021.655851
- **URL read:** https://www.frontiersin.org/articles/10.3389/fpls.2021.655851/full
- **Licence:** Frontiers, CC BY (standard; not stated in the fetch).
- **Cultivar:** dwarf (bush) green bean 'Valentino'.
- **Design:** greenhouse, Sardinia; three cycles in 2013.
- **Shading:** cumulated PAR was −45% (whole PV greenhouse vs conventional) and about −75% in rows under the panels.

| Trait (PV vs control) | Result | Tag |
|---|---|---|
| Fresh pod yield | **−31%** (1.36 vs 1.98 kg m⁻²; whole greenhouse, 45% less PAR). PV rows vs plastic rows −49%. | [TABLE] (Table 2) / [TEXT] |
| Pod number | **−35%** under PV | [TEXT] (Fig. 6B) |
| Leaf number | −25% | [TEXT] |
| Pod fresh weight | about −17% | [TABLE] (Table 1) |
| Total dry mass | **−52%** (end of cycle 1); −33% (end of cycle 3) | [TEXT] (Fig. 5A) |
| SLA | **up to +32%** (cycle 1); +15% (cycle 3) | [TEXT] (Fig. 5D) |
| LAR | up to +32% | Fig. 5E |
| Stem dry-mass share | +34% | Fig. 6A |
| LAI | −43% (cycle 1), −24% (cycle 2) | Fig. 5B |
| RUE | about +36% | Discussion |
| Stem or internode length | Not measured | [NOT FOUND] |

Yield fell 0.44% for each 1% less cumulated PAR.

### C3. Lopes et al. 1987 (greenhouse, 62% and 30% of sunlight)
- **Citation:** Lopes, N.F.; Oliva, M.A.; Gomes, M.M.S.; Souza, V.F.; Cardoso, M.J. (1987). Crescimento, morfologia, partição de assimilados e produção de matéria seca do feijão (*Phaseolus vulgaris* L.) submetido a três densidades do fluxo radiante e dois regimes hídricos. *Revista Ceres* 34(192):110–124.
- **URL read:** https://www.alice.cnptia.embrapa.br/alice/bitstream/doc/699439/1/Crescimento-morfologia-particao-de-assimilados...pdf (OCR copy)
- **Cultivar:** Negrito 897 (growth habit not stated).
- **Design:** pots in a greenhouse, whole cycle, at 100%, 62% and 30% of incident radiation, each at field capacity or −1.0 MPa.

**Results at field capacity [TEXT]:**

| Trait | 100% | 62% | 30% |
|---|---|---|---|
| Max plant height (cm) | 39.2 | 78.0 (**+99%**) | 101.7 (+159%) |
| Max root DW (g/pot) | 4.4 | 3.1 (−30%) | 2.2 (−50%) |
| Max pod DW (g/pot) | 6.0 | 7.6 (n.s.) | 6.5 (n.s.) |

- Leaf and pod number were not changed by shade.
- Shoot:root ratio increased with shading.
- Internode elongation increased (qualitative only).
- The OCR copy has inconsistencies (e.g. 78.0 vs 75.8 cm). Verify against the original.

### C4. Almeida et al. 1988 (field, 30/70/100% light)
- **Citation:** Almeida, A.A.F. de; Lopes, N.F.; Oliva, M.A.; Barros, R.S. (1988). Doses de nitrogênio e densidades de luz no crescimento do feijoeiro e na conversão da energia solar. *Pesquisa Agropecuária Brasileira* 23(8):849–859.
- **URL read:** https://apct.sede.embrapa.br/pab/en/article/download/13977/7942
- **Cultivar:** Negrito 897; field, Viçosa.
- **Results [TEXT]:**
  - SLA and LAR increased with shading (no shaded values given; max SLA in full sun 3.4 dm² g⁻¹).
  - Max LAI 0.96 / 1.06 / 1.11 at 100 / 70 / 30%.
  - The light levels are ambiguous: the text says 30% and 70% *reduction*.
  - Weak data, low LAI.

### C5. Souza et al. 2001 (snap bean, determinate)
- **Citation:** Souza, J.R.P.; Miglioranza, E.; Brandão, R.A.P.; Athanázio, J.C. (2001). Produção e textura de feijão-vagem cultivado sob diferentes níveis de sombreamento. *Horticultura Brasileira* 19(3):247–249.
- **DOI:** 10.1590/S0102-05362001000300020
- **URL read:** https://www.scielo.br/pdf/hb/v19n3/v19n3a20.pdf
- **Licence:** SciELO, CC BY (journal default; not stated in the fetch).
- **Cultivar:** UEL 1 (determinate).
- **Design:** field, Londrina; shade screens from 15 DAE at 100/70/50/30% light.
- **Table 1 [TABLE]:**
  - Pods/plant, square-root transformed: 1.72 (100%) vs 1.31 (50%), significant. Back-transformed (x² − 0.5) this is 2.46 vs 1.22, about **−50%**.
  - Pod weight/plant: 3.59 vs 3.42 g, not significant.
- The absolute values are very low and the sample basis is unclear. Low-quality target.

### C6. Ntamwira et al. 2026 (bush beans, screenhouses)
- **Citation:** Ntamwira, J.; Ocimati, W.; Kearsley, E.; Mirindi, T.C.; Shabani, G.; Blomme, G. (2026). Screenhouse-based shade-tolerance assessments of coffee and bean varieties, and cover crops, to optimize multi-strata intercropping in African smallholder systems. *Frontiers in Agronomy* 8:1775355.
- **DOI:** 10.3389/fagro.2026.1775355
- **URL read:** https://www.frontiersin.org/journals/agronomy/articles/10.3389/fagro.2026.1775355/full
- **Design:** 10 bush varieties. Nominal 50% screen measured at 44.7% PAR.
- **Results:**
  - Grain yield change vs open field ranged from +96.5% (HM21-7) to −56.1% (NABE20). My unweighted mean is about +9%, with very high variance [TABLE] (Table 2).
  - All bush beans were taller under shade, even at 70% light [FIG-NOT-DIGITIZED] (Fig. 1).
  - Pods and biomass are in Supplementary Tables S2–S3 (not read).

### C7. Worku et al. 2004 (irradiance by phase, chamber)
- **Citation:** Worku, W.; Skjelvåg, A.O.; Gislerød, H.R. (2004). Responses of common bean to photosynthetic irradiance levels during three phenological phases. *Agronomie* 24(5):267–274.
- **DOI:** 10.1051/agro:2004024
- **Material:** an indeterminate cultivar (not named in the abstract); 100, 250 or 400 µmol m⁻² s⁻¹ per phase.
- **Results [ABSTRACT]:**
  - Lower irradiance increased SLA, LAR and shoot:root ratio.
  - Pods/plant was most sensitive during flowering.
  - Seed weight responded during flowering and seed filling.
- Numbers: [NOT FOUND] (PDF fetch failed).

---

## D. Dense planting

### D1. Clavijo Michelangeli et al. 2019 (starting paper)
- **Citation:** Clavijo Michelangeli, J.A.; Ricaurte, J.; Sinclair, T.R.; Rao, I.M.; Beebe, S.E. (2019). Influence of plant density and growth habit of common bean on leaf area development and N accumulation. *Journal of Crop Improvement*, 1–14 (vol/issue not verified).
- **DOI:** 10.1080/15427528.2019.1644694. hdl:10568/102225.
- **URL read:** https://core.ac.uk/download/224976763.pdf (CGSpace accepted manuscript)
- **Licence:** CC BY-NC 4.0.
- **Genotypes:**
  - Type I (bush): CAL 96, ICA Quimbaya
  - Type II: DOR 364, SER 118
  - Type III: G 21212, Carioca
- **Densities:** 5, 10, 15, 20, 25, 35 plants m⁻².
- **Sites:** Palmira, 999 m a.s.l., Tmax mean 31.1 °C, sown Oct 2013; Darién, 1570 m a.s.l., Tmax mean 25.5 °C, sown May 2014. Field.

| Trait | Result | Tag |
|---|---|---|
| Leaf area per plant | Above 0.3 m² plant⁻¹ at 5 pl m⁻²; below 0.1 m² plant⁻¹ for all genotypes at 35 pl m⁻². Change from 5 to 35 pl m⁻²: **about −70% or more**. Types II and III exceed Type I at low density. | [FIG-NOT-DIGITIZED] (Fig. 1; text values) |
| LAI (leaf area per m²) | About 1.5 at low density, rising to about 2.5–2.8 at high density (**about +70 to +85%** from 5 to 35 pl m⁻²). **Plateau: "20 plants m⁻² or more was sufficient to result in equal leaf area and N accumulation."** Types II and III slightly above Type I. | [TEXT] (Fig. 2, Table 3) |
| Leaf N per leaf area (SLN) | About 2.0 g N m⁻² (Darién) and about 1.3 g N m⁻² (Palmira). **Stable across densities of 10 pl m⁻² or more**; elevated at 5 pl m⁻² at Darién. | [TEXT] (Fig. 4, Table 5) |
| Leaf N per ground area | Palmira 2.11 g N m⁻² below Darién. The 5 and 10 pl m⁻² densities were somewhat lower than higher densities. | [TEXT] (Fig. 5) |
| Leaf weight per ground area at high density (Types II/III) | About 60–70 g m⁻² (Palmira) and about 100–110 g m⁻² (Darién) | [TEXT] (Fig. 3) |
| Yield link | At Palmira, leaf N below about 3 g N m⁻² went with low yield. | Fig. 6 |

---

## E. Frost and freezing

| Source | Finding | Tag |
|---|---|---|
| **Meyer, D.W. & Badaruddin, M. (2001).** Frost tolerance of ten seedling legume species at four growth stages. *Crop Science* 41:1838. DOI 10.2135/cropsci2001.1838. | Pinto and navy bean seedlings 1–4 weeks old. Hardened seedlings were cooled at 1 °C/h in a programmable chamber to −2, −4, −6 or −8 °C. **LT50 = −3.25 to −3.5 °C for dry beans**, the least tolerant of the 10 species (soybean and pea −4.5 °C). One-week-old seedlings were slightly more tolerant near LT50. | [ABSTRACT] |
| **Balasubramanian, P.; Vandenberg, A.; Hucl, P.; Gusta, L.V. (2004).** Resistance of *Phaseolus* species to ice crystallization at subzero temperature. *Physiologia Plantarum* (2004). DOI 10.1111/j.0031-9317.2004.00257.x. PMID 15032842. | "Dry bean cultivars possess little or no freezing tolerance and are killed at the temperature of ice formation in their tissues." Cultivar CDC Nighthawk. Tertiary-gene-pool species had LT50 0.5–1 °C lower and supercooled more. Absolute LT50 values are in the full text. | [ABSTRACT] |
| **Gubbels, G.H. (1969).** Frost protection of crops by sprinkler irrigation. *Canadian Journal of Plant Science* (1969). DOI 10.4141/cjps69-122. | Excised bean plants were **protected from serious injury when plant temperature stayed above −0.8 °C**. | [ABSTRACT] |
| **Cary, J.W.** (USDA-ARS NWISRL report 488). Bean mutations from azide... https://eprints.nwisrl.ars.usda.gov/763/1/488.pdf | "Beans will tolerate some ice in their tissue provided the temperature does not fall below −1 °C." | [ABSTRACT] |
| **Cary, J.W. & Mayland, H.F. (1970).** Factors influencing freezing of supercooled water in tender plants. *Agronomy Journal* 62:715. DOI 10.2134/agronj1970.00021962006200060009X. | Freezing of well-watered bean seedlings spread rapidly and was **always lethal**. Plants supercool, and nucleation depends on dew or ice on the leaves. | [ABSTRACT] |
| **Holubowicz, R. & Dickson, M.H. (1989).** *Euphytica*. DOI 10.1007/BF00022408. | Exotherm (ice formation) was linked to injury and death. Some seedlings survived 2 h at −6 °C, through supercooling. | [ABSTRACT] |
| **Lindow, S.E.; Arny, D.C.; Upper, C.D. (1982).** *Plant Physiology* 70:1084. PMC1065830. | With ice-nucleation-active bacteria present, leaves nucleate at about −2 °C. Without them, nuclei are active only at much lower temperatures, so field kill temperature depends on nucleation. | [ABSTRACT] |

Interpretation for the CROPGRO values (−2.2 °C leaf damage, −5 °C death):
- Experimental data put bean tissue kill at the ice-nucleation point. This is about −0.8 to −1 °C plant temperature with external ice or dew, about −2 °C with INA bacteria, and LT50 about −3.3 °C for whole seedlings in a controlled freeze.
- −2.2 °C is a reasonable onset for leaf damage.
- −5 °C for plant death is more lenient than the seedling LT50 of −3.25 to −3.5 °C (Meyer & Badaruddin 2001).
- The measured points are tissue or seedling temperatures. CROPGRO uses air Tmin, and canopy temperature on radiative frost nights can be 1–3 °C below screen temperature.

---

## Gaps
1. Gomes 2000: the Table 1 and Table 2 numbers could not be extracted (garbled PDF text, SciELO server errors). Only text percentages are available.
2. Guerrero-Domínguez 2023: every result is figure-only. Needs digitizing.
3. Gross & Kigel 1994 and Prasad 2002: full texts are paywalled. There is no per-temperature table for Prasad; only slopes are available.
4. Sharda 2026: filled pods and seed number per group are in the supplementary tables (not read).
5. Shade: no study found gives SLA, internode length, total dry mass and pods together for a bush bean under ~50% shade cloth over the whole cycle. Cossu 2021 is the closest (dwarf bean, −45% PAR) but lacks stem length.
6. Hanyu 1996: numeric values are blocked (the J-STAGE PDF is disallowed by robots.txt).
7. Clavijo 2019: per-density values are figure-only. The text gives the approximate ends and the plateau statement.
