# WP32 research: weed competition and intercropping with common bean (Phaseolus vulgaris)

Tags: [F] = read this session (abstract, publisher page or full text fetched; says which); [K, to verify] = from memory, not re-read; [I] = my inference.
Builds on (read first, not repeated): `docs/archive/research/notes-m2-m3/competition_validation.md` §3–4 (CPWC from Woolley, Burnside, Beiermann, Odero, Ghamari; Soltani 2018 loss survey; milpa LER) and `pole_bean_grass.md` Q4 (wheat parameters).
Tools: WebSearch + WebFetch (cambridge.org abstracts, scielo, udesc, core.ac.uk, wur edepot). PMC, HAL, BioOne and doaj were bot-blocked this session.

---

## Takeaway

1. **Yield-loss shape is solid; bean-specific I and A values are scattered and wide.** The Cousens rectangular hyperbola fits every bean data set found. Read values: Palmer amaranth I = 209 %/(plant m⁻¹ row), A = 95% (Miranda 2022); redroot pigweed in snap bean I = 10–32 %/(plant m⁻¹ row), A = 47–63% (Aguyoh & Masiunas 2003); wild poinsettia I = 2.4 %/(plant m⁻²), max loss 48% (5.5 and 60% when the weed emerged 12 d earlier; Machado 2015); Alexander grass i = 4.6–9.1 %/(plant m⁻²), a = 91–155% (Galon 2022). Hairy nightshade two-point data (13% at 2, 77% at 100 plants m⁻¹ row; Blackshaw 1991) back-fit to I ≈ 7.7, A ≈ 86 [I]. So: **large C4 broadleaves (Amaranthus) are 1–2 orders of magnitude more damaging per plant than grasses or small weeds; A for a single tall weed species is ~50–95%.**
2. **Time of emergence dominates density.** Weeds emerging at bean V3 (2nd trifoliolate) caused roughly half the loss of those emerging with the crop (common ragweed: 10–22% vs 4–9% at 1.5 plants m⁻¹ row; Chikoye 1995). Redroot pigweed sown at first trifoliolate: 39–42% vs 48–58% loss at 8 plants m⁻¹ row (Aguyoh 2003). A weed emerging 12 d *before* bean raised I by 2.3× (Machado 2015). The authors of Chikoye 1995 state emergence time "may be more important than weed density".
3. **CPWC (5% loss) ≈ V1–V2 (≈15–20 DAE) to R1–R3 (≈40–55 DAE)** in North America and Iran; it shrinks at 10% loss (Odero 2018 snap bean: 1.7–6.7 wk at 5%, 2.2–5.2 wk at 10%; Ghamari 2012: 20–68 vs 23–55 DAE). Brazil reports earlier, shorter windows (4–18 DAE at 5%; Borchartt 2011) or V4–R6 (Kozlowski 2002, secondhand). Season-long uncontrolled loss: 31–94% (N. America mean 71%; Soltani 2018), 83–99% at very high weed biomass (Beiermann 2022), 36–60% in lighter Brazilian/Iranian infestations.
4. **Mechanism: light first, N second, water situational.** Evidence: the loss is carried by pods per plant/m² and LAI, not plant density or seed size, except under severe competition (Chikoye 1995; Woolley 1993; Miranda 2022: pods −69%, seed mass −18% at 2 Palmer plants m⁻¹ row). The CPWC starts when weed biomass starts its rapid rise (Woolley 1993). N fertiliser *increases* weed damage to bean (Bannayan 2012; Saberali 2016), and N assimilation + chlorophyll explain 73–87% of the variance in loss (Saberali 2016). A sim with shared light, N and water should reproduce this without special casing if the weed out-grows bean in height.
5. **Intercropping.** Bean–maize: LER 1.09–1.99 depending on site and design (milpa ~1.34; Malawi 1.59–1.77; semi-arid Kenya only 0.87–1.09 when bean failed to nodulate); maize is the dominant partner (one maize plant ≈ 0.7–3.4 bean plants; Pilbeam 1994). Global meta-analysis: mean LER 1.22, +0.21 per unit temporal niche differentiation, theoretical max ≈1.39 for relay (Yu 2015). **No common bean–wheat/oat/barley LER was found**; faba bean–cereal proxies: 1.03–1.09 (faba–naked oat), up to 1.29 and 1.65 (faba–wheat), up to 1.8 (faba–barley alternate rows).

### Recommended WP32 challenge targets [I]
Use bush bean ~25–30 plants m⁻² on a 1–3 m plot, weed = redroot pigweed (draft below). Tolerances wide (±10–15 percentage points) because field ranges are wide.

| Challenge | Setup | Target (bean seed yield vs weed-free) | Basis |
|---|---|---|---|
| "Let it go" | pigweed 4–8 plants m⁻², emerging with bean, never weeded | **−45 to −80%** | Aguyoh 2003 (48–58%), Miranda 2022 (77% at ~2.6 m⁻²), Soltani 2018 mean 71% |
| "One weed" | 1 pigweed m⁻², with bean | **−10 to −35%** | Aguyoh I 10–32 per plant m⁻¹ row; Chikoye ragweed 10–22% at 1.5 m⁻¹ row |
| "Late weeds" | same density, weed emerges at bean V3 (~14–21 DAE) | **≤ half the loss of the with-crop case** (−5 to −30%) | Chikoye 1995; Aguyoh 2003 |
| "Critical period" | weed free from emergence until X; or weedy until X | 95% of weed-free yield needs weeding by **~V2 (15–20 DAE)** and weed-free until **~R1 (35–45 DAE)**; weeding only after R1 recovers little | Woolley 1993; Beiermann 2022; Ghamari 2012 |
| "Fertiliser trap" | high N vs no N, weedy | relative loss **larger** at high N | Bannayan 2012 (8–9% → 30–33%, semi-erect) |
| "Dense crop" | bean 40 vs 20 plants m⁻² under same weeds | loss smaller at higher density | Tejada 2025; Galon 2022 |
| "Grass weed" | C4 grass (foxtail-like) 10–20 m⁻² | **−10 to −30%** | Wilson 1993 millet 12–31% at 10 m⁻²; Galon 2022 |
| "Milpa" | pole bean + maize | total LER **1.1–1.6**, maize partial LER ≥0.85, bean partial 0.3–0.8 | competition_validation §4; Pilbeam 1994; Nkhata 2021 |
| "Bean + wheat" | row/mixed | total LER **1.0–1.3**, flag only as plausibility (no Phaseolus data) | Yu 2015; faba-cereal proxies |

Weedy-plot biomass check: weed shoot dry mass at bean maturity **~400–2,500 g m⁻²** (Beiermann 2022: 487 g m⁻² from 224 small weeds m⁻²; 2,433 g m⁻² from 40 plants m⁻² dominated by Palmer amaranth and lambsquarters). Bean shoot biomass 677 vs 1,239 g m⁻² weedy vs weed-free (Ghamari 2013, −45%).

### Draft weed species: redroot pigweed (*Amaranthus retroflexus*), "pigweed"
See the parameter table in "Weed species parameters". Headline: C4 annual dicot, SLA 13.5 m² kg⁻¹ (constant through season), RUE 1.74 g MJ⁻¹ IPAR, germination min ~5–8 °C / opt 25–35 °C, shoot elongation Tb 14.4 °C to opt 27.9 °C, height 0.9–2 m, taproot, k ~0.8, seeds 1 mm, 10⁴–10⁶ seeds per plant.

---

## Cousens table (bean, yield loss vs weed density)

Model (Cousens 1985) [F via prior notes]: YL = I·d / (1 + I·d/A); I = % loss per weed at d→0, A = asymptotic max % loss.
Relative-time form (Cousens et al. 1987, *Weed Sci.* 35: 720–725, doi:10.1017/S0043174500060872) [F that it is a rectangular hyperbola in D and T with three coefficients and predicts no yield gain from late weeds; the equation is only an image on the page]. Form [K, to verify]: **YL = I·d / (e^(c·T) + I·d/A)**, T = weed emergence minus crop emergence (days or °C·d), c > 0.

Density units matter: many US papers use plants per m of row. Bean rows are 0.56–0.76 m, so 1 plant m⁻¹ row ≈ 1.3–1.8 plants m⁻² [I].

| Weed | Bean type, site | Weed emergence | I (% per plant) | A (%) | Other loss points | Source |
|---|---|---|---|---|---|---|
| Palmer amaranth (C4) | dry bean, Scottsbluff NE 2020–21; 210,000 plants ha⁻¹, 56 cm rows | 5–11 d after bean | **208.9 (SE 52) per plant m⁻¹ row** (≈158 per plant m⁻² by the paper's conversion [I]) | **94.6 (SE 11)** | 77% at 2 m⁻¹ row (2.64 m⁻²); 5% loss at 0.02 m⁻¹ row; yields 5,090/3,670 weed-free → 596/1,140 kg ha⁻¹ | [F full text] Miranda JWA, Jhala AJ, Bradshaw J, Lawrence NC (2022). *Weed Technol.* 35: 995–1006, doi:10.1017/wet.2021.101 |
| Redroot pigweed (C4) | snap bean, Illinois 1998–99 | with crop (early) vs at 1st trifoliolate (late) | **10–32 per plant m⁻¹ row** (higher when early) | **47–63** (higher when early) | 8 m⁻¹ row: 58/42% (1998), 48/39% (1999) early/late | [F abstract] Aguyoh JN, Masiunas JB (2003). *Weed Sci.* 51: 202–207, doi:10.1614/0043-1745(2003)051[0202:IORPAR]2.0.CO;2 |
| Common ragweed | white bean, Ontario 1990–92 | VE vs V3 | I "more consistent" across sites but large SE (no value in abstract) | varied by site/timing | 1.5 m⁻¹ row: **10–22% (VE), 4–9% (V3)**; LAI, biomass, pods m⁻² most hit; seeds/pod, seed weight unaffected | [F abstract] Chikoye D, Weise SF, Swanton CJ (1995). *Weed Sci.* 43: 375–380, doi:10.1017/S0043174500081352 |
| Wild poinsettia (*Euphorbia heterophylla*) | common bean, Paraná, Brazil | with crop vs 12 d before | **2.4 (with) / 5.5 (12 d before) per plant m⁻²** | max loss **48 / 60** | densities 0–142 m⁻²; pods/plant main component; economic threshold <3.3 m⁻² | [F abstract] Machado AB, Trezzi MM, Vidal RA et al. (2015). *Planta Daninha* 33: 41–48, doi:10.1590/S0100-83582015000100005 |
| Alexander grass (*Urochloa plantaginea*, C4) | common bean IPR Uirapuru, 140–260k plants ha⁻¹, RS Brazil | with crop | **i = 5.01, 5.62, 9.11, 4.55** (bean 140, 180, 220, 260k ha⁻¹) | **a = 107, 155, 91, 123** (a > 100 = extrapolation artefact per authors) | 80 plants m⁻²: 84, 89, 81, 92% loss | [F full text, Table 2] Galon L, Biachessi F et al. (2022). *Rev. Ciênc. Agrovet.* 21: 263–273, doi:10.5965/223811712132022263 |
| Hairy nightshade (*Solanum sarrachoides*) | dry bean, Alberta | with crop | **≈7.7 per plant m⁻¹ row [I, two-point back-fit]** | **≈86 [I]** | 13% at 2 m⁻¹ row, 77% at 100 m⁻¹ row; loss from 3 wk interference; up to 9 wk weed-free needed | [F abstract] Blackshaw RE (1991). *Weed Sci.* 39: 48–53, doi:10.1017/S0043174500057854 |
| Wild proso millet (*Panicum miliaceum*, C4 grass) | dry bean, Nebraska irrigated | with crop | hyperbola fitted (values not in abstract) | — | **12–31% at 10 m⁻²**; 4 wk weed-free sufficient; removal at 6 WAP: 41% (1990), 11% (1991) | [F abstract] Wilson RG (1993). *Weed Sci.* 41: 607–610, doi:10.1017/S0043174500076396 |
| Green foxtail (*Setaria viridis*, C4) and common sunflower | pinto bean, Wyoming 1994–95 | with crop | — | — | foxtail: no loss 1994, loss only at top density 1995; economic thresholds 1.6–2.9 foxtail and 0.12–0.2 sunflower per m row; tolerable duration 4.5 wk foxtail, 3.2 wk sunflower | [F abstract] Mesbah AO, Miller SD, Koetz PJ (1995) *Weed Technol.* (Cambridge page gives 2017 online date) [K, to verify vol. 9 pp.] |
| Beggarticks (*Bidens* sp.) | 6 bean cultivars, RS Brazil, 12 plants m⁻² | with crop | i, a only in figures (all a < 100) | — | **3.7–13.4% loss per 100 g m⁻² weed dry mass** depending on cultivar | [F full text page] Galon L et al. (2016). *Planta Daninha* 34: 411–422, doi:10.1590/S0100-83582016340300002 |
| Redroot pigweed | bean, Iran, N rates | with crop | — | — | loss 8–9% (0 N) → 30–33% (high N), semi-erect; 13–17% → 33–40%, erect | [F prior notes] Bannayan M et al. (2012). *Field Crops Res.* |
| Mixed weeds (biomass basis) | dry bean | — | **0.208 kg bean loss per kg weed DM** (1,000 kg weed DM → 208 kg bean) | — | — | [F secondhand, cited in Miranda 2022] Wilson RG et al. (1980) |

Barnyardgrass + black nightshade vs Red Kidney bean (replacement series, California): bean germinated earlier and suppressed both weeds; intra-specific bean effect > either weed (Fennimore, Mitich & Radosevich 1984, *Weed Sci.* 32: 336–342, doi:10.1017/S0043174500059099) [F abstract]. No I/A. Useful as a "crop first wins" check.

Not found with I/A for bean: *Chenopodium album*, *Echinochloa crus-galli*, *Xanthium*, *Amaranthus hybridus*. Lambsquarters in maize (Iran): relative emergence time affected loss more than density [F abstract, journals.areeo.ac.ir article_122779].

---

## CPWC table (dry and snap bean)

| Study | Crop, site | Loss level | Start | End | Season-long loss | Tag |
|---|---|---|---|---|---|---|
| Woolley BL, Michaels TE, Hall MR, Swanton CJ (1993). *Weed Sci.* 41: 180–184, doi:10.1017/S0043174500076037 | white bean, Ontario 1986–87 | 3% | 2nd trifoliolate (≈V3) | first flower (R1) | — | [F abstract]; start = start of rapid weed biomass rise |
| Burnside OC et al. (1998). *Weed Sci.* | dry bean, Minnesota | — | 3 WAP | 5–6 WAP | — | [F prior notes] |
| Beiermann CW, Miranda JWA, Creech CF, Knezevic SZ, Jhala AJ, Harveson R, Lawrence NC (2022). *Weed Technol.* 36: 168–176, doi:10.1017/wet.2021.99 | dry bean, Scottsbluff NE | 5% | critical timing of removal: **16 DAE / 170 GDD₁₀ / V1** (2018, no PRE); **39 DAE / 476 GDD / R1** (2020, no PRE); **47 DAE / R2** with PRE herbicide | — | **83% (2018), 99% (2020)** no PRE; 28% / 0% with PRE | [F full text]. 4-par logistic on GDD: d = 82.4%, e = 359 GDD (2018); d = 96.6%, e = 531 GDD (2020) |
| Odero DC et al. (2018). *HortScience* | snap bean, Florida | 2.5 / 5 / 10% | 1.2 / 1.7 / 2.2 WAE | 8.4 / 6.7 / 5.2 WAE | — | [F prior notes] |
| Ghamari H et al. (2012) | dry bean, Iran (C. album, A. retroflexus) | 5 / 10% | 20 / 23 DAE | 68 / 55 DAE | 60% | [F prior notes] |
| Borchartt L, Jakelaitis A et al. (2011). *Rev. Ciênc. Agron.* 42: 725–734 | common bean Carioca Precoce, no-till, Rondônia BR | 5% | **4 DAE** | **18 DAE** | 35.8% (Digitaria + volunteer maize) | [F full text PDF, redalyc] |
| Kozlowski LA et al. (2002). *Planta Daninha* 20 | common bean, no-till, Paraná BR | — | **V4** | **R6** | — | [F secondhand via Borchartt 2011]; DAE values not read |
| Blackshaw 1991 | dry bean, hairy nightshade | — | <3 WAE | up to 9 WAE | — | [F abstract] |
| Wilson 1993 | dry bean, wild proso millet | — | — | 4 WAP | — | [F abstract] |
| Mesbah 1995 | pinto, foxtail / sunflower | economic | 4.5 / 3.2 / 2.5 wk tolerable (foxtail / sunflower / both) | — | — | [F abstract] |
| Tejada 2025 | arid Peru, 2 bean densities | — | denser crop: CPWC 17 d shorter, PBI 4 d longer | — | — | [F prior notes] |
| Soltani N et al. (2018). *Weed Technol.* | survey, N. America | — | — | — | **31–94%, mean 71.4%** | [F prior notes] |

Method: Knezevic SZ, Evans SP, Blankenship EE, Van Acker RC, Lindquist JL (2002). *Weed Sci.* 50: 773–786 [F full text, core.ac.uk/download/210032426.pdf]: logistic for increasing weedy duration, Y = [1/(exp(c(T−d)) + f) + (f−1)/f]·100; Gompertz for increasing weed-free duration, Y = a·exp(−b·exp(−kT)); T in GDD or DAE; acceptable loss normally 2–10%. No bean example in the paper.

Consensus [I]: in temperate dry bean, **weed by V1–V3 (~2–3 WAE, ~150–250 GDD₁₀) and keep clean until R1 (~5–6 WAE, ~450–550 GDD₁₀)**. Lower acceptable loss widens the window on both sides.

---

## Weed species parameters

### Redroot pigweed (*Amaranthus retroflexus*) — recommended generic weed

| Parameter | Value | Unit | Source / tag |
|---|---|---|---|
| Pathway | C4 | — | [F] Singh N, Rubione C, Sarangi D (2023/2025) GROW info sheet "Redroot & Smooth Pigweed", growiwm.org |
| RUE (intercepted PAR) | **1.74** | g DM MJ⁻¹ | [F abstract] Knezevic SZ, Horak MJ, Vanderlip RL (1999). *Weed Sci.* 47: 291–296, doi:10.1017/S0043174500091797 (unaffected by density) |
| SLA | **13.5** (135 cm² g⁻¹), constant through season | m² kg⁻¹ | [F abstract] Knezevic 1999 |
| Partitioning | stable within vegetative and reproductive phases; unaffected by density | — | [F abstract] Knezevic 1999 |
| Light extinction k | 0.8–0.9 (horizontal-leaved broadleaf weeds, generic) | — | [F] Debaeke P et al. (1997). *Weed Res.* 37: 325–341 (ALMANAC; generic range, not pigweed-specific) |
| Germination T min | 5 °C (41 °F) [GROW]; 8 °C [Coleman 2021]; none at 15/10 °C day/night [Guo 2003] | °C | [F] three sources disagree; use Tb ≈ 8–10 °C [I] |
| Germination T opt / max | 25–40 opt, 45 max; peak germination at 35/30 °C d/n, falls above | °C | [F] Coleman M et al. (2021) UNE vegetable IWM guide; Guo P, Al-Khatib K (2003). *Weed Sci.* 51: 869–875, doi:10.1614/P2002-127 |
| Seedbank emergence | 50% of flush at 380.7 hydrothermal °C·d (Tb 0 °C, ψb −0.4 MPa); main flush 350–450 | °C·d | [F] Taab A, Royo-Esnal A (2023). *Span. J. Agric. Res.* 21(2) e1001, doi:10.5424/sjar/2023212-19814 — from season start, not from sowing |
| Sown seed to emergence | ~60–100 °C·d (Tb 10 °C), i.e. 4–8 d in summer | °C·d | [K, to verify]; Amador-Ramirez 1999 saw first weed emergence at 80–120 TT units [F abstract, units' base not given] |
| Shoot elongation Tb / Topt | **14.4 / 27.9** | °C | [F abstract] Oryokot JOE, Murphy SD, Thomas AG, Swanton CJ (1997). *Weed Sci.* 45: 488–496, doi:10.1017/S0043174500088718 |
| Leaf appearance | linear in temperature; slowed by canopy shade | — | [F abstract] McLachlan SM, Swanton CJ, Weise SF, Tollenaar M (1993). *Weed Sci.* 41: 590–593 |
| Shade plasticity | under shade: less total DM, fewer branches, more DM to main stem and upper canopy | — | [F abstract] McLachlan SM, Tollenaar M, Swanton CJ, Weise SF (1993). *Weed Sci.* 41: 568–573, doi:10.1017/S0043174500076335 |
| Heat lethal | died after 8 d at 45/40 °C (Palmer 25 d) | — | [F abstract] Guo & Al-Khatib 2003 |
| Max height | 0.9–1.8 m (3–6 ft); up to 2 m | m | [F] GROW; Coleman 2021 |
| Rooting | taproot; depth not quantified in sources read; ~1–1.5 m [K, to verify, Weaver 1926 / Costea et al. 2004 *Can. J. Plant Sci.* 84: 631–668] | m | taproot [F]; depth [K] |
| Root competition | reduced neighbouring maize root density by 41–87% (lambsquarters 27–73%) | — | [F abstract] Britschgi D, Stamp P, Herrera JM (2013). *Weed Sci.* 61: 319–327, doi:10.1614/WS-D-12-00120.1 |
| Seed | ~1 mm; 230,000–500,000 per plant typical; >10⁶ uncompeted; 10,000–40,000 under crop competition | — | [F] GROW; Coleman 2021 |
| Biomass per plant (Palmer, in bean) | 760 g at 0.2–0.3 m⁻¹ row; 388 g at 2 m⁻¹ row | g | [F] Miranda 2022 (Palmer, not redroot; redroot is smaller) |
| Flowering | short-day promoted; flowers mid-late summer | — | [K, to verify] |
| N response | strong; weed damage to bean rises with N rate | — | [F] Bannayan 2012; Saberali SF et al. (2016). *J. Agric. Sci.* 154: 598–611, doi:10.1017/S0021859615000477 |
| Gas exchange | CER = power function of PPFD × quadratic in T | — | [F abstract only, no values] Nielsen DC, Anderson RL (1994). *Weed Technol.* 8: 265–269, doi:10.1017/S0890037X00038756 |

Draft species file fields [I] (values to be marked "~" where [K]):
```
name = "pigweed"; photosynthesis = "C4"
vcmax25 ~ 40–60 µmol m⁻² s⁻¹; vpmax25 ~ 100–150  (C4 von Caemmerer; [K])
g1_medlyn ~ 1.6–2.4 kPa^0.5 (C4, Lin et al. 2015) [K]
sla = 13.5 m² kg⁻¹ [F]; leaf_angle ~ planophile, k ≈ 0.8 [F generic]
emergence_tt = 80 °C·d base 10 [~]; t_base_dev = 10 °C [~]; t_base_elong = 14.4 °C [F]; t_opt = 28–35 °C [F]
max_height = 1.5 m [F range]; internode elongation strongly shade-plastic (R:FR) [I]
root = taproot, max_depth 1.2 m [~], elongation ~2–3 cm d⁻¹ [~]
seed_mass ≈ 0.4 mg [K]; seeds_per_g_dm ~ 300–500 [~]
leaf_N_area lower than C3 bean (C4 NUE) [K]
```

### Grass weed alternative (C4 annual: green foxtail / barnyardgrass) — mostly [K, to verify]
- Green foxtail (*Setaria viridis*): C4; height 0.2–0.6 m; weak competitor in bean (no loss in 1 of 2 years; Mesbah 1995 [F]). Fine fibrous roots ~0.5–1 m [K].
- Barnyardgrass (*Echinochloa crus-galli*): C4; height 0.8–1.5 m; Tb ~10–13 °C [K]; in bean replacement series, bean suppressed it when bean emerged first (Fennimore 1984 [F]).
- Alexander grass (*Urochloa plantaginea*): C4; i ≈ 5 %/plant m⁻² (Galon 2022 [F]).
- The WP23 spring wheat (C3, Tb 0 °C, phyllochron ~100 °C·d) can stand in as a "C3 grass weed" (e.g. volunteer cereal/wild oat) with no new code [I].

---

## LER table (common bean intercrops)

| System | Design | Total LER | Partial LERs / competition | Source / tag |
|---|---|---|---|---|
| Maize + bean (milpa) | review | **1.34** (range 1.16–2.78 in another review) | — | [F prior notes] Mota-Cruz 2025; Chiorese 2026 |
| Maize + bean relay, Egypt | additive 100:100; bean 166,667 ha⁻¹ | **1.64–1.99** (shorter co-growth = higher) | bean 0.72–0.98 | [F prior notes] El-Mehy 2023, doi:10.3389/fsufs.2023.1052392 |
| Maize + bean, Malawi | intra-row, 2 sites × 2 seasons | **1.59 (NUA45), 1.77 (SER83)** | maize yield unchanged vs sole | [F abstract] Nkhata W, Hussein S, Rowland C (2021). *Front. Sustain. Food Syst.* 5, doi:10.3389/fsufs.2021.741177 |
| Maize + bean, semi-arid Kenya | bivariate density factorial | **1.09 (long rains), 0.87 (short rains)**; bean failed to nodulate | 1 maize plant ≈ 0.7–3.4 bean plants | [F abstract] Pilbeam CJ, Okalebo JR, Simmonds LP, Gathua KW (1994). *J. Agric. Sci.* 123: 191–198, doi:10.1017/S0021859600068453 |
| Maize + bean, organic Mexico | — | — | per-plant bean yield = sole; maize 1.2× sole | [F prior notes] Ebel 2017 |
| **All intercrops (meta)** | 746 records | **1.22 ± 0.02** (median 1.17); 81% > 1 | +0.211 LER per unit temporal niche differentiation; C3/C4 +0.12 over C3/C3; max ≈1.39 | [F thesis full text] Yu Y (2016) PhD Wageningen, edepot.wur.nl/378297; journal: Yu Y, Stomph TJ, Makowski D, van der Werf W (2015). *Field Crops Res.* 184: 133–144 |
| Faba bean + spring wheat (proxy) | 75% bean + 50% wheat RD, UK organic | **1.65**; Bulson 1997: 1.29 (75:75) | bean unaffected by wheat; wheat fell with bean density | [F] Wolfe M et al. (2013) ORC Bulletin 112 (agricology.co.uk); Bulson HAJ, Snaydon RW, Stopes CE (1997). *J. Agric. Sci.* 128: 59–71 |
| Faba bean + barley (proxy) | alternate vs within-row | up to **1.8** (alternate rows only > 1) | barley more competitive | [F abstract] Martin MPLD, Snaydon RW (1982). *Exp. Agric.* 18: 139–148, doi:10.1017/S0014479700013612 |
| Faba bean + naked oat (proxy) | replacement 25–75% bean | **+3 to +9%** over sole (≈1.03–1.09) | CR > 1 for faba bean (bean dominant) | [F abstract] Klimek-Kopyra A et al. (2015). *Chilean J. Agric. Res.* 75(2), doi:10.4067/S0718-58392015000200005 |
| Barley + chickpea relay (proxy) | Iran | stress raised total LER 25% | CR max 0.68; barley more aggressive | [F abstract] agry.um.ac.ir article_37278 (authors not on page) |

Competitive ratio / aggressivity: no numeric values for a *Phaseolus* intercrop were read. In maize–bean, maize is consistently dominant (Pilbeam 1994; Rutjens 2025 FSPM bean LAI −80%). In cereal–grain-legume, the cereal is usually more aggressive (Martin & Snaydon 1982; barley–chickpea), but faba bean can dominate oat [F above].

---

## Inferences

- [I] **The sim does not need an empirical Cousens layer.** It should *emerge* loss from shared light/N/water. Use Cousens as a validation fit: run density series (0, 0.5, 1, 2, 4, 8, 16 weeds m⁻²) for "with crop" and "at V3" emergence, fit I and A, and check: A_with 50–95%, I_with ≥ 10 %/(plant m⁻²) for pigweed, A_V3 and I_V3 roughly half.
- [I] Pigweed must out-grow bean in height by ~R1. Bush bean canopy ~0.4–0.6 m [K]; pigweed 0.9–1.8 m. If the sim's pigweed stays below the bean (e.g. because Tb 14.4 °C elongation stalls in a cool spring), losses will be far below field values. The 14.4 °C base for elongation versus ~8–10 °C for bean development is what makes early-season bean competitive and late-spring/summer pigweed explosive.
- [I] C4 advantage: at high light and >25 °C, RUE 1.74 g MJ⁻¹ IPAR for pigweed vs bean ~1.2–1.5 [K, to verify]. A hot-summer challenge should be harder than a cool one.
- [I] Expected yield-component signature for scoring: pods per plant/m² fall first (−20 to −70%), seeds per pod and seed mass fall only under severe competition (≥ −15% at Palmer 2 m⁻¹ row). Plant stand unaffected.
- [I] N: because bean fixes N and pigweed is a luxury N consumer, adding fertiliser N should help the weed more than the bean. The sim reproduces Bannayan only if nodule fixation is down-regulated by soil nitrate and the weed's uptake capacity is high.
- [I] Density units: convert per-m-of-row values to per-m² with the study's row width (56 cm Miranda; 76 cm typical in older US bean studies [K]). On a 1–3 m garden plot, a single pigweed per m² is a realistic "one weed" challenge; Palmer-type I values imply that one plant near a 1 m² bean patch can take 30–60%.
- [I] Intercrop: Yu 2015 relay effect and El-Mehy's "shorter co-growth = higher LER" both say relay timing is the main lever; a "sow bean into standing wheat after heading" challenge should score LER > 1.1 if the bean escapes shade early.

---

## Gaps

- Full-text I and A tables not read for: Chikoye 1995 (ragweed), Wilson 1993 (millet), Aguyoh 2003 per-year values, Galon 2016 (figure-only), Kalsing 2013 (Urochloa), Malik et al. 1993 white bean–weed–cultivar study, Wall et al. 1993 wild mustard in navy bean (*Can. J. Plant Sci.* 73, doi:10.4141/cjps93-170, not fetched).
- No bean I/A for *Chenopodium album*, *Echinochloa crus-galli*, *Xanthium*, *A. hybridus*.
- Cousens 1987 equation form not seen (image); [K] form given.
- Kozlowski 2002 CPWC in DAE not read (only V4–R6 secondhand). Burnside 1998 and Ghamari 2012 rest on earlier notes.
- Pigweed: root depth, k, leaf photosynthesis parameters (Vcmax/Vpmax), leaf appearance slope, emergence thermal time from sowing are [K]. Costea et al. 2004 (*Can. J. Plant Sci.* 84: 631–668) and Knezevic 1999 full texts would close most of these.
- No *Phaseolus*–wheat/oat/barley LER, CR or aggressivity; only faba bean proxies. Blocked: BioOne (Blackshaw & Brandt 2008 N-response of weeds), HAL (Yu 2015 alt copy), PMC.
- Weed height and LAI trajectories vs bean in the same field (time series) not found; Saberali 2016 has LAI/LAD but values were paywalled.
