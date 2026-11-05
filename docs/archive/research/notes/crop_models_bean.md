# Process-based crop models and common bean (Phaseolus vulgaris): parameters and checks against design plan v3

Scope: what CROPGRO-Drybean (DSSAT), APSIM, WOFOST (and SUCROS/LINTUL lineage) and the literature say about bean phenology, partitioning, respiration, water stress and soil water, compared with design-plan-v3 sections 3.6, 3.10, 3.11, 3.16 and 8.2 to 8.5.

**How the model files were obtained (important for WP11).** GitHub (raw.githubusercontent, api.github.com, jsdelivr) and CRAN are blocked from this session. PyPI is reachable, and two PyPI packages bundle the files:
- `DSSATTools` 3.0.2 wheel ships `DSSATTools/dssat-csm-os/Data/Genotype/BNGRO048.SPE`, `.CUL`, `.ECO` (the DSSAT v4.8 dry bean files). Local copy: `/tmp/claude-0/-home-claude-Plant-Sim/b9667701-e1b8-5d1a-aab2-5cb6e2180076/scratchpad/downloads/pypi/x/DSSATTools/dssat-csm-os/Data/Genotype/`. Fetch with `pip download --no-deps DSSATTools` ([PyPI DSSATTools](https://pypi.org/project/DSSATTools/)).
- `pyswap` 0.3.9 wheel ships a copy of ajwdewit/WOFOST_crop_parameters (`pyswap/libs/WOFOST_crop_parameters/fababean.yaml`, `soybean.yaml`, `mungbean.yaml`, plus cowpea, chickpea, pigeonpea, groundnut). Local copy: `/tmp/claude-0/-home-claude-Plant-Sim/b9667701-e1b8-5d1a-aab2-5cb6e2180076/scratchpad/downloads/pypi2/x/pyswap/libs/WOFOST_crop_parameters/` ([PyPI pyswap](https://pypi.org/project/pyswap/); upstream [WOFOST_crop_parameters](https://github.com/ajwdewit/WOFOST_crop_parameters)).
- `pcse` 6.0.13 wheel (WOFOST code): `pcse/crop/respiration.py`, `pcse/crop/evapotranspiration.py` ([PyPI pcse](https://pypi.org/project/pcse/)).
Below, "SPE line N" means line N of BNGRO048.SPE in that bundle. The same files are upstream at [DSSAT/dssat-csm-data](https://github.com/DSSAT/dssat-csm-data) (Genotype/) and inside [DSSAT/dssat-csm-os](https://github.com/DSSAT/dssat-csm-os) (Data/Genotype/).

---

## 1. CROPGRO-Drybean (DSSAT): species, ecotype and cultivar values; licence and access

### Takeaway
CROPGRO-Drybean's files give a full, citable bean parameter set: development cardinal temperatures 5/27/37/45 °C (vegetative) and 5/24/34/44 °C (early reproductive). These are well above the plan's 8/28/38, and they have an optimum plateau where the plan has a single peak. Temperate Type I cultivars carry PPSEN = 0, which means they ignore day length. Partitioning by V-stage, seed composition (23.5% protein, 3% lipid) and Penning de Vries construction costs are all available. DSSAT-CSM is open source under BSD-3-Clause.

### Cited Findings
**Licence and access**
- The DSSAT-CSM source repository github.com/DSSAT/dssat-csm-os is licensed BSD-3-Clause (licence file "license.txt") — [ecosyste.ms project record](https://ost.ecosyste.ms/projects/1068); repo: [DSSAT/dssat-csm-os](https://github.com/DSSAT/dssat-csm-os).
- The DSSATTools Python wrapper that redistributes the Data/Genotype files is itself GPL-3.0 (licence in its wheel `dssattools-3.0.2.dist-info/licenses/LICENSE`) — [PyPI DSSATTools](https://pypi.org/project/DSSATTools/).
- DSSAT crop list (CROPGRO covers dry bean, soybean, peanut, faba bean, etc.) — [DSSAT.net: crops simulated](https://dssat.net/plant-growth-modules-in-dssat-csm/).

**Phenology: cardinal temperatures (SPE lines 95–99, "TB TO1 TO2 TM", °C; four-point, linear "LIN" function with a plateau between TO1 and TO2)**
- Vegetative development: TB 5.0, TO1 27.0, TO2 37.0, TM 45.0 — BNGRO048.SPE line 97 ([dssat-csm-data](https://github.com/DSSAT/dssat-csm-data)).
- Early reproductive development: 5.0, 24.0, 34.0, 44.0 — SPE line 98.
- Late reproductive (seed fill): 6.0, 25.0, 37.0, 60.0 — SPE line 99.
- Phase table (SPE lines 103–115). Planting→emergence and emergence→end-juvenile use the vegetative curve with no photoperiod function ("NON"). End-juvenile→flower induction→first flower→first pod→first seed use the early-reproductive curve with photoperiod function "INL". First seed→maturity uses the late-reproductive curve, also "INL".
- Each phase in that table also carries a water-stress sensitivity: −0.30 in vegetative phases, −0.40 in early reproductive phases, +0.70 in late reproductive phases (SPE lines 103–112). (Inference from DSSAT conventions: negative values slow development under water deficit, positive values speed it, so drought during seed fill shortens the phase and hastens maturity.)

**Photoperiod (CUL lines 7–10, 44–78)**
- CSDL (critical short day length, below which development has no day-length effect) = 12.17 h for almost all cultivars. Porrillo Sintetico uses 13.50 h — BNGRO048.CUL lines 44–78.
- PPSEN (slope of the relative development response to photoperiod, 1/h, positive for short-day plants): 0.000 for the generic "Andean Habit 1" and "Meso Amer. Hab.1" determinate types and for temperate determinate cultivars Seafarer (navy, MESDET), Isabella and Redkloud (ANDDET). Small values for Manitou 0.015, Canadian Wonder 0.020, A 195 0.020, A 486 0.030, Jatu Rong 0.040. Indeterminate Andean habits 2–4 use 0.050. Porrillo 0.017. Maximum allowed 0.070 — CUL lines 44–78.
- Ecotype file: R1PPO (increase in day-length sensitivity after R1) = 0.000, OPTBI = 0.0, SLOBI = 0.000 for all bean ecotypes, so no cold-night slowing of progress to flowering is parameterised — BNGRO048.ECO lines 37–52.

**Phase durations (CUL/ECO; photothermal days, PTD = days at optimal temperature and non-limiting photoperiod)**
- Ecotype (ECO line 47, MESDET; line 49, ANDDET): PL-EM 2.4 (Meso) / 3.4 (Andean) thermal days; EM-V1 3.0; V1-JU 0.0; JU-R0 5.0 PTD; PM09 0.55; LNGSH (shell growth time per pod) 8.0 PTD; R7-R8 7.0 d; TRIFL (main-stem leaf appearance) 0.34 leaves per thermal day; FL-VS 0.0 for determinate types versus 11.0 for indeterminate — BNGRO048.ECO lines 46–52.
- Cultivar examples (EM-FL / FL-SH / FL-SD / SD-PM in PTD; WTPSD g; SFDUR PTD; SDPDV seeds per pod; PODUR PTD; SLAVR cm² g⁻¹; SIZLF cm²): CUL lines 47–78 ([dssat-csm-data](https://github.com/DSSAT/dssat-csm-data)).
  - Seafarer (Type I navy, MESDET): 28.0 / 4.0 / 11.0 / 19.0; 0.22; 17.0; 5.0; 11.0; 315; 133.
  - Andean Habit 1 generic: 22.6 / 3.0 / 12.0 / 18.4; 0.60; 15.0; 3.5; 10.0; 305; 133.
  - Meso Amer. Habit 1 generic: 27.0 / 3.0 / 9.0 / 20.0; 0.23; 16.0; 5.2; 5.0; 310; 133.
  - Redkloud (ANDDET): 23.5 / 3.0 / 9.8 / 19.7; 0.55; 18.0; 3.5; 11.0; 320; 150.
  - Allowed ranges ("MINIMA/MAXIMA"): EM-FL 20–35, FL-SH 2–5, FL-SD 6–13, SD-PM 14–29, WTPSD 0.22–0.66, SFDUR 11–22, SDPDV 3–5, PODUR 4–16, SLAVR 250–350, SIZLF 133–180. THRSH (maximum seed share of pod mass) 78%. SDPRO 0.235, SDLIP 0.030 — CUL lines 44–45.

**Photosynthesis temperature (SPE lines 7–10)**
- FNPGT (canopy gross photosynthesis vs temperature, LIN): 3.0, 21.5, 28.5, 40.0 °C — SPE line 7.
- XLMAXT/YLMAXT (relative leaf light-saturated rate vs temperature): 0 at 8 °C, 1.0 at 32 °C, 0.8 at 36 °C, 0 at 42 °C — SPE lines 8–9.
- FNPGL (effect of night minimum on next-day photosynthesis, QDR): −1, 16, 50, 60 °C. (Inference: full effect is reached only when Tmin ≥ 16 °C, a chilling-night penalty.) — SPE line 10.
- LFMAX 0.93–1.01 mg CO2 m⁻² s⁻¹ at 30 °C, 350 vpm, high light (cultivar file) — CUL lines 19, 47–78.
- PARMAX 38, PHTMAX 56, canopy KCAN 0.65 — SPE line 4. KEP (energy extinction for soil evaporation) 0.65 — SPE line 131.

**Respiration and construction costs (SPE lines 17–20; composition lines 22–33)**
- RES30C = 3.5E-04 (maintenance coefficient on plant mass at 30 °C, per hour) and R30C2 = 0.0040 (maintenance coefficient proportional to gross photosynthesis) — SPE line 18.
- Glucose costs (g CH2O per g product) — SPE lines 19–20:
  - RNO3C 2.556, RNH4C 2.556, RPRO 0.360, RFIXN 2.830
  - RCH2O 1.242, RLIP 3.106, RLIG 2.174, ROA 0.929, RMIN 0.05
  - PCH2O 1.13
- Organ composition, protein fractions (initial / growth / final), SPE lines 23–25:
  - leaf .240/.160/.070; stem .150/.100/.050; root .100/.080/.060; shell .220/.220/.080
  - seed SDPROS .235; nodule PRONOD .300
- Organ composition, other fractions, SPE lines 26–30:
  - carbohydrate: leaf .521, stem .664, root .703, shell .410, seed .640, nodule .480
  - lipid: leaf .025, stem .020, root .020, shell .020, nodule .050
  - lignin: leaf .070, stem .070, root .070, shell .280, seed .020
  - organic acids .04–.05; minerals: leaf .094, seed .035
- Seed lipid parameters LIPTB 7.168, LIPOPT 23.65 °C — SPE line 33.

**Partitioning (SPE lines 48–53)**
- Vegetative partitioning by V-stage XLEAF = 0, 1.8, 4.0, 6.5, 8.5, 9.5, 12, 30, with:
  - YLEAF = .46 .56 .60 .55 .45 .40 .30 .30
  - YSTEM = .15 .25 .30 .35 .45 .50 .59 .59
  - (root share = 1 − leaf − stem = .39 .19 .10 .10 .10 .10 .11 .11)
  - Source: SPE lines 49–51.
- WTFSD 0.55, PORPT 0.30, FRSTMF 0.59, FRLFF 0.30, ATOP 1.00, FRCNOD 0.05 (fraction of new root growth to nodules), FRLFMX 0.70 — SPE lines 52–53.
- XFRT (maximum fraction of daily growth to seed + shell) = 1.00 for all cultivars — CUL.

**Leaf area and SLA (SPE lines 56–62)**
- FINREF 250, SLAREF 300 cm² g⁻¹, SIZREF 133 cm². SLAMAX 900, SLAMIN 255, SLAPAR −0.035 (light effect), TURSLA 1.0 — SPE lines 56–57.
- Leaf area per plant vs V-stage (YVREF): 30, 60, 110, 280, 1000, 1800 cm² at V 0, 1.83, 2.8, 4.3, 6.8, 9.6 — SPE lines 59–60.
- SLA temperature effect: relative SLA 0.30 at ≤10 °C, rising to 1.0 at ≥22 °C — SPE lines 61–62.

**Senescence, freezing, roots (SPE lines 64–76)**
- FREEZ1 −2.22 °C (leaf freeze damage), FREEZ2 −5.00 °C (plant death). Light compensation ICMP 0.10, time constant TCMP 10 d — SPE lines 65–66.
- Initial root depth RTDEPI 25 cm(?), RFAC1 9000 (root length to mass, cm g⁻¹), RLDSM 0.1, RWUEP1 1.50, RWUMX 0.04 (cm³ water cm⁻¹ root d⁻¹) — SPE line 74.

**Reproductive temperature and water (SPE lines 78–90)**
- SETMAX 0.90, RFLWAB 0.10, XMPAGE 100 — SPE line 79.
- FNPDT (temperature effect on pod set, QDR): 7.0, 22.0, 25.0, 36.0 °C — SPE line 81.
- FNSDT (temperature effect on seed growth, QDR): 4.0, 23.0, 25.0, 38.0 °C — SPE line 82.
- XXFTEM/YXFTEM (partitioning to reproductive organs): 1.0 up to 30 °C, 0 at 38 °C — SPE lines 83–84.
- XSWFAC/YSWFAC (water effect on pod addition): factor 1 for SWFAC ≥ 0.5, falling only when SWFAC < 0.5 — SPE lines 85–86. XSWBAR/YSWBAR (topsoil water on pod addition) all 1.0, so inactive for bean — SPE lines 87–88.
- Pod detachment DETACH = N (off) — SPE line 93.

**Nitrogen fixation (SPE lines 40–46)**
- SNACTM 0.050, NODRGM 0.170, DWNODI 0.014, TTFIX 0.0, NDTHMX 0.04, CNODCR 0.05 — SPE line 41.
- Nodule growth temperature: 6, 21, 35, 44 °C. Fixation temperature: 4, 19, 35, 44 °C — SPE lines 42–43.
- Dry-soil effect on fixation 0 at relative soil water 0, 0.85 at 0.85 (LIN) — SPE line 44.

**Canopy height (SPE lines 120–128)**
- Internode length per node (YVSHT) 0.025–0.12 m, peaking at V8–V16. Height and width expand 4× in darkness (XHWPAR) — SPE lines 120–127.

**Calibration literature**
- Calibration and testing papers for CROPGRO-Dry bean exist (e.g. [calibration and test of the CROPGRO-Dry bean model, PDF](https://pdfs.semanticscholar.org/7ed5/76b7a0230ee19b98b53654eab27365960d0d.pdf)); not read in detail here.

### Inferences
- **Converting PTD to °C·day (inference).** One thermal day equals one day at the optimum plateau, i.e. (TO1 − TB) degree-days: 22 °C·d on the vegetative curve and 19 °C·d on the reproductive curves (base 5–6 °C). For Seafarer the phases are planting→emergence 2.4 TD, EM→FL 28 PTD (about 3 vegetative and 25 reproductive), FL→first seed 11 PTD and first seed→maturity 19 PTD. That gives roughly 2.4×22 + 3×22 + 25×19 + 11×19 + 19×19 ≈ 1,160 °C·d above a 5 °C base, about 930 °C·d above an 8 °C base at a 20 °C mean. In days at a 20 °C constant mean this is about 78 days, inside the plan's 65–93-day target and consistent with the plan's 800–1,200 °C·d range.
- **Emergence (inference).** PL-EM 2.4–3.4 TD × 22 °C·d gives 53–75 °C·d above 5 °C, close to the plan's 60–80 °C·d. The plan's value is consistent.
- **Plan divergence on cardinal temperatures.** The plan's single three-point curve, 8/28/38 °C, makes development fall linearly above 28 °C and halve at 33 °C. CROPGRO keeps the maximum development rate flat between 24–27 and 34–37 °C and reaches zero only at 44–45 °C. For a temperate summer this matters on hot days: the plan would slow development during heat waves, while CROPGRO would not. Recommendation: use a four-point (trapezoid) development function, about Tb 5–7, To1 24–27, To2 32–35, Tmax 42–45, and keep heat damage as a separate reproductive-abortion process. Ferreira et al. 1997 (§5) fit 6.8/24.0/34.1 with hourly data, which supports a lower optimum and a higher base than CROPGRO for emergence to flowering.
- **Photoperiod.** Temperate Type I bush beans (Seafarer, Isabella, Redkloud, Andean/Meso Habit 1) are day-neutral in CROPGRO (PPSEN = 0). The M0 temperate-summer run can ship with no photoperiod response, provided the species file exposes CSDL (default 12.17 h) and PPSEN (default 0, range 0–0.07 h⁻¹) as heritable loci for M3.
  - The response form needs checking in DSSAT source (PHENOL/CURV with "INL"). Our understanding is relative rate ≈ 1 − PPSEN×(daylength − CSDL), clipped to 0–1.
  - Under that form, a 15.5 h temperate midsummer day and PPSEN 0.05 would slow reproductive development by about 17%.
  - Not verified: whether CROPGRO's day length includes civil twilight. We believe it does (TWILEN), but this is from memory.
- **Partitioning.** The plan has no explicit organ-fraction table, by design: allocation emerges from demand and priority. CROPGRO's table can serve as a validation target. Leaf share of vegetative growth is about 0.55–0.60 at V2–V6; stem share rises to 0.59 late; roots get about 0.39 at emergence and about 0.10 from V4 on. Our allocation should produce roughly these ratios under well-watered conditions.
- **Seed composition and cost (inference from SPE lines 19–33, Penning de Vries-type costs).**
  - Seed: carbohydrate .64×1.242 + lipid .03×3.106 + lignin .02×2.174 + organic acid .04×0.929 + mineral .035×0.05 ≈ 0.97 g CH2O, plus protein .235×(2.556 to 2.83) ≈ 0.60–0.67. Total ≈ 1.57–1.64 g CH2O per g seed, i.e. conversion efficiency about 0.61–0.64.
  - Leaf: the same calculation gives ≈ 1.54 g CH2O per g.
  - In carbon terms, at 0.40 g C per g CH2O and 0.45 g C per g dry matter, that is about 1.37 (leaf) to 1.42 (seed) g substrate C per g structural C, or 27–30% growth respiration.
  - The plan's single construction cost of 1.25 (20% growth respiration) is at the low end and does not distinguish protein-rich seeds and leaves.
  - Lipid: CROPGRO uses 3.0% seed lipid (CUL SDLIP), not the 1–2% in the brief. Protein 23.5% matches the brief's 22–25%.
- **Maintenance respiration: possible unit error in the plan (inference).** Plan §3.8 says "0.015 g C per g N per day at 20 °C".
  - Converting the plan's value: leaf N ≈ 0.24 protein / 6.25 ≈ 3.8% (SPE line 23), so 0.015 g C per g N per day ≈ 0.00058 g C per g DW per day ≈ 0.0014 g CH2O per g DW per day.
  - CROPGRO: RES30C 3.5E-4 per hour × 24 = 0.0084 g CH2O per g per day at 30 °C, about 0.0042 at 20 °C with Q10 = 2. This is mass-based only; CROPGRO adds a further term proportional to gross photosynthesis (R30C2).
  - WOFOST: leaves 0.027–0.03 g CH2O per g per day at 25 °C, about 0.02 at 20 °C (§3).
  - So the plan's value is about 3× below CROPGRO and about 14× below WOFOST for leaves. Expressed per g N, WOFOST's leaf value is about 0.2–0.3 g C per g N per day.
  - Recommendation: re-derive the per-N coefficient (Ryan 1991-type values were not retrieved here; see Gaps) or switch to per-organ mass-based coefficients (WOFOST RML/RMS/RMR/RMO) with Q10 = 2 at 25 °C.
- **Leaf size.** CROPGRO SIZLF 133–180 cm² (maximum full trifoliate, three leaflets) versus the plan's 50–100 cm² mature trifoliate. The plan is low for a maximum, though the Kumar 2025 average of 0.12 m² over 20 leaves (≈60 cm²) suggests a typical rather than maximum size. Suggest: maximum potential 120–150 cm², with emergent shading and deficits lowering realised size.
- **SLA.** CROPGRO SLAVR 280–325 cm² g⁻¹ and SLAMIN 255, versus the plan's 200–300 (§8.2 Gebeyehu ≈ 240). CROPGRO new-leaf SLA is higher than the plan's average; the plan's range is fine as a whole-canopy average. CROPGRO cuts SLA to 30% below 10 °C (thicker leaves when cold), which the plan could emulate.
- **Pod and seed timing.** CROPGRO shell (pod wall) growth per cohort LNGSH = 8 PTD (≈10 days at 20 °C) and seed fill SFDUR 15–22 PTD (≈19–28 days at 20 °C). The plan's "pods expand ~20 days, then seeds fill ~20 days" doubles the pod phase. In CROPGRO, seed growth starts while shells are still growing (FL-SH 3–4.5 PTD; FL-SD 9–12 PTD). Suggest 8–12 days of pod-wall growth, overlapping seed fill of about 18–25 days.
- **Freeze.** The plan's −1 °C leaf death is warmer than CROPGRO's −2.22 °C leaf damage / −5 °C plant death. Either is defensible; −2 °C matches CROPGRO.
- **Pod-set temperature.** CROPGRO's pod-set factor peaks at 22–25 °C and is zero at 7 and 36 °C (quadratic). It is a continuous function of temperature rather than a hard 32 °C day threshold, and it also covers the cold side. Seed growth is zero at 38 °C.
- **Drought and pod set.** CROPGRO cuts pod addition only when SWFAC (supply/demand) < 0.5. That is broadly consistent with the plan's FTSW < 0.3 abortion trigger, since SWFAC falls below 1 only at low FTSW (inference).
- **Nodules.** CROPGRO RFIXN 2.830 vs RNO3C 2.556 g CH2O per g protein means fixed N costs about 11% more than nitrate N. Nodules get only 5% of new root growth (FRCNOD) and have their own temperature curves. The plan's "10–15% of assimilate" is not directly comparable; note the CROPGRO values for M2.

### Gaps
- The DSSAT Fortran source (RESPIR.for, PHENOL.for, CURV) was not readable (GitHub blocked), so the exact equations below are from recall and marked as inference:
  - photoperiod form "INL"
  - RES30C units (per g plant mass excluding seed oil and protein?)
  - TURFAC = TRWUP/(RWUEP1×EP)
- The licence of the separate data repository dssat-csm-data was not confirmed; dssat-csm-os (which includes Data/Genotype) is BSD-3.
- Units of RTDEPI line items were not verified.

---

## 2. APSIM: bean or legume models and the SoilWater cascading bucket

### Takeaway
APSIM's SoilWater (SoilWat) is a daily layered "tipping/cascading bucket". It drains SWCON × (SW − DUL) per day, uses Ritchie two-stage evaporation (U, CONA) and USDA curve-number runoff. Our hourly cell bucket is the same family and is accepted practice, but its rate constant should be a per-hour fraction derived from SWCON, not "all excess moves one cell per hour". APSIM's common bean parameterisation (legume framework) uses Tb 7.5 / Topt 30 / Tmax 40 °C, RUE 0.94 g MJ⁻¹, k 0.40 and specific root length 65 m g⁻¹.

### Cited Findings
- SoilWat drainage between DUL and saturation: Flux = SWCON × (SW − DUL), "the proportion of soil water above dul that drains in one day"; example SWCON 0.02 in a black-earth soil. Water above SAT cascades to the next layer (MWCON = 1 default) — [APSIM SoilWat documentation](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/).
- Unsaturated flow below DUL: diffusivity = diffus_const × exp(diffus_slope × θav), with defaults 88 / 35.4 (CERES) and 40 / 16 for cracking clays — [APSIM SoilWat](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/).
- Evaporation: Ritchie two-stage. Stage 1 Es = Eos until cumulative U mm (example U = 6 mm); stage 2 Es = CONA×√t (example CONA 3.5, presumably mm d⁻⁰·⁵) — [APSIM SoilWat](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/).
- Runoff: USDA-SCS curve number. CN2Bare example 75, reduced by CNRed (e.g. 20) at cover up to CNCov (about 80%) — [APSIM SoilWat](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/).
- SoilWat is called daily — [APSIM SoilWat](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/). The parameter list (SummerCona, SummerU, WinterCona, WinterU, DiffusConst, DiffusSlope, Salb, CN2Bare, CNRed, CNCov, SWCON …) is also in [apsimx R package soilwat_parms](https://search.r-project.org/CRAN/refmans/apsimx/html/soilwat_parms.html). A soil-parameter protocol is in [APSoil protocol PDF](https://www.apsim.info/wp-content/uploads/2019/10/Parameters-for-soil-water-Ver24.pdf) and [PMC8720820](https://pmc.ncbi.nlm.nih.gov/articles/PMC8720820/) (not read).
- APSIM-Legume framework (Robertson et al. 2002, Aust. J. Agric. Res. 53:429–446):
  - Daily step, cardinal-temperature thermal time, configurable photoperiod.
  - Leaf area from node appearance and leaves per node; biomass from RUE × intercepted radiation; harvest index increases daily.
  - Water stress reduces leaf expansion via a soil water deficit factor; transpiration efficiency is scaled by VPD; water extraction uses kl.
  - Parameterised species: chickpea, mungbean, peanut, lucerne (no common bean in that paper).
  - Source: [Robertson et al. 2002 PDF](https://era.dpi.qld.gov.au/id/eprint/10127/1/Simulation%20of%20growth%20and%20development%20of%20diverse%20legume%20species%20in%20APSIM.pdf).
- APSIM common bean (Amiri, Zakeri, Yousefi 2024, Technology in Agronomy 4: e017) — [Amiri et al. 2024](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015):
  - Base 7.5, optimum 30, maximum 40 °C.
  - RUE 0.94 g MJ⁻¹; extinction coefficient 0.40.
  - Node appearance 92–100 °C·d; 2 leaves per node.
  - frac_leaf_pre_flower 0.55–0.60; frac_leaf_grain_fill 0.30–0.35; frac_stem2pod 0.46–0.49; frac_pod2grain 0.28.
  - Critical N 0.045 g g⁻¹; specific root length 65,000 mm g⁻¹ (= 65 m g⁻¹); transpiration efficiency coefficient 0.0055 kPa.
  - Validation: grain yield nRMSE 12%, dry matter nRMSE 13%, soil water nRMSE 12–14%.
  - The paper does not state which APSIM version was used, and it does not report phase thermal times or photoperiod settings.

### Inferences
- **Hourly drainage (divergence).** Plan §3.6, "drainage moves excess down one cell per hour", is effectively SWCON = 1 per hour: every cell sheds all water above field capacity each hour. Field soils take about 1–3 days to drain to DUL, which is why APSIM and CERES use SWCON about 0.2–0.7 per day (example 0.02 for heavy clay). For pots with free drainage, fast drainage is realistic; for plots it is too fast.
  - Suggest: per-hour fraction k_h = 1 − (1 − SWCON)^(1/24), with SWCON by texture (sand high, clay low), and keep saturation-excess cascade.
  - This also gives the §8.1 "drainage time" fixture a closed form: an exponential approach to FC.
- **Evaporation (divergence).** The plan's "fraction of radiation reaching the soil surface, reduced as it dries" is close in spirit but lacks the two-stage form. Adopt Ritchie: potential soil evaporation = potential ET × exp(−KEP × LAI) (KEP 0.65 for bean, SPE line 131); stage 1 up to U ≈ 6 mm (DSSAT soil parameter SLU1, "stage 1 evaporation limit, mm", SOIL.CDE line 83); stage 2 CONA√t, with CONA ≈ 3.5 mm d⁻⁰·⁵. At an hourly step, apply the daily √t law to cumulative stage-2 time.
- **Runoff.** Curve number with CN2 ≈ 75 bare is the standard in both APSIM and DSSAT. The plan has none (infiltration from the top only), so ponding and runoff on clay are missing. This matters only for M2 plots.
- **Bucket validity.** Daily tipping-bucket models (CERES/DSSAT, APSIM SoilWat) are accepted practice. Hourly use needs rates rescaled and a check that lateral redistribution (10% of the gradient per hour) does not dominate vertical drainage in 2 cm cells.
- **Root length.** APSIM's 65 m g⁻¹ specific root length supports the plan's ~70 m g⁻¹.
- **APSIM partitioning.** frac_leaf_pre_flower 0.55–0.60 agrees with CROPGRO YLEAF 0.55–0.60 at V2–V6.

### Gaps
- APSIM Next Generation model list (PMF Soybean, Mungbean, Chickpea, FabaBean, Peanut and whether a CommonBean model exists) could not be confirmed: the apsimnextgeneration.netlify.app documentation returned errors. APSIM NG SoilWater default SWCON values by texture were not retrieved.
- Units of CONA were not stated on the page fetched.

---

## 3. WOFOST (and SUCROS lineage): respiration, conversion efficiencies, partitioning, water stress

### Takeaway
WOFOST has no Phaseolus file. The nearest are soybean and mungbean (warm-season) and faba bean (cool-season Vicia, whose phenology is not transferable). Legume maintenance respiration is RML 0.027–0.03, RMS 0.015, RMR 0.010, RMO 0.005–0.017 kg CH2O kg⁻¹ d⁻¹ at 25 °C with Q10 = 2. Conversion efficiencies are CVL 0.61–0.72, CVS 0.63–0.69, CVR 0.66–0.72, CVO 0.48 (soybean) to 0.72 (mungbean). WOFOST's water-stress threshold depends on evaporative demand rather than being a fixed FTSW.

### Cited Findings
(All from `pyswap/libs/WOFOST_crop_parameters/*.yaml`, mirroring [ajwdewit/WOFOST_crop_parameters](https://github.com/ajwdewit/WOFOST_crop_parameters); faba bean file metadata: source report [edepot.wur.nl/308997](https://edepot.wur.nl/308997), licence EUPL.)

**Soybean_901**
- TBASEM 7, TEFFMX 22, TSUMEM 70 °C·d; TSUM1 350; TSUM2 850.
- DTSMTB: 0 at ≤7 °C rising to 23 at 30 °C and 38 at 45 °C.
- IDSL 0, so no photoperiod in this set.
- SPAN 23 d; TBASE 7.
- CVL 0.72, CVO 0.48, CVR 0.72, CVS 0.69.
- Q10 2; RML 0.03, RMO 0.017, RMR 0.01, RMS 0.015.
- FLTB 0.7 until DVS 1.0, then 0.6 at DVS 1.15, 0.43 at 1.3, 0.15 at 1.5, 0 at 2.0.
- FRTB 0.65 at DVS 0 → 0.35 at 0.75 → 0.15 at 1.0 → 0 at 1.5.
- FOTB 0.15 at DVS 1.15 → 0.47 at 1.3 → 0.75 at 1.5 → 1.0 at 2.0.
- SLATB 0.0014–0.0025 ha kg⁻¹ (14–25 m² kg⁻¹); AMAXTB 29; EFFTB 0.40; KDIFTB 0.8.
- TMPFTB (AMAX vs daytime temperature): 0 at 0 °C, 0.3 at 10, 0.6 at 20, 0.8 at 25, 1.0 at 30–35 °C.
- DEPNR 5.0; RDMCR 120 cm; RRI 1.2 cm d⁻¹.

**Mungbean_VanHeemst_1988**
- TBASEM 10, TSUMEM 60, TSUM1 640, TSUM2 753.
- DTSMTB 0 at 10 °C to 18 at 28 °C, flat to 45 °C.
- CVL/CVO/CVR 0.72, CVS 0.69.
- RML 0.03, RMO 0.01, RMR 0.01, RMS 0.015.
- TMPFTB 0 at 8 °C, 1 from 20 to 35 °C, 0 at 40 °C.
- DEPNR 3.5.

**Faba_bean_801**
- TBASEM 0, TSUMEM 100, TSUM1 833, TSUM2 1351.
- CVL 0.608, CVO 0.591, CVR 0.659, CVS 0.631.
- RML 0.027, RMO 0.005, RMR 0.01, RMS 0.015, Q10 2.
- SLATB 0.0031–0.0042 ha kg⁻¹ (31–42 m² kg⁻¹, i.e. 310–420 cm² g⁻¹).
- AMAXTB 35; EFFTB 0.48; DEPNR 4.5.

**WOFOST code (pcse)**
- Maintenance respiration RMRES = RMR·WRT + RML·WLV + RMS·WST + RMO·WSO, scaled by RFSETB(DVS) and TEFF = Q10^((T − 25)/10). Reference temperature is 25 °C — `pcse/crop/respiration.py` lines 100–106 ([pcse](https://pypi.org/project/pcse/)).
- Assimilate requirement: CVF = 1/((FL/CVL + FS/CVS + FO/CVO)(1 − FR) + FR/CVR) — `pcse/crop/wofost72.py` line 219.
- Water stress: SWEAF = 1/(0.76 + 1.5·ET0[cm d⁻¹]) − (5 − DEPNR)·0.10, limited to 0.10–0.95 (Doorenbos & Kassam 1979; van Keulen & Wolf 1986). Critical moisture SMCR = (1 − SWEAF)(SMFC − SMW) + SMW. RFWS = (SM − SMW)/(SMCR − SMW), clipped 0–1, multiplies transpiration and hence assimilation — `pcse/crop/evapotranspiration.py` lines 15–37, 202–206.

### Inferences
- **WOFOST critical FTSW (inference, arithmetic from the code).** In FTSW terms the critical point is FTSWcrit = 1 − SWEAF. For a legume with DEPNR 4.5 at ET0 0.5 cm d⁻¹, SWEAF = 0.66 − 0.05 = 0.61, so FTSWcrit ≈ 0.39. At ET0 0.3 cm d⁻¹, FTSWcrit ≈ 0.22; at ET0 0.7 cm d⁻¹, ≈ 0.48. The plan's 0.4 therefore matches WOFOST on a high-demand summer day but is too strict on cool days. A demand-dependent threshold is the standard WOFOST and FAO-56 (p-fraction) practice.
- **Bean maintenance respiration.** For bean, WOFOST's legume values (leaf 0.03, stem 0.015, root 0.01, pods/seeds 0.01–0.017 g CH2O g⁻¹ d⁻¹ at 25 °C, Q10 2) are a defensible mass-based default, and far higher than the plan's per-N value (§1 Inferences).
- **Conversion efficiency for bean seeds.** For bean seeds (23.5% protein, 3% lipid), a CVO of about 0.62–0.64 (§1) lies between WOFOST mungbean (0.72) and soybean (0.48, oil-rich). Leaves CVL about 0.65–0.72.
- **Faba bean phenology.** Do not use it for Phaseolus phenology (Tb 0 °C, cool-season).

### Gaps
- No WOFOST Phaseolus vulgaris parameter set exists in the repository (files present: barley, cassava, chickpea, cotton, cowpea, fababean, groundnut, maize, millet, mungbean, pigeonpea, potato, rapeseed, rice, seed_onion, sorghum, soybean, sugarbeet, sugarcane, sunflower, sweetpotato, tobacco, wheat).
- SUCROS/LINTUL bean-specific parameters were not found. LINTUL uses RUE and FRTRL-style partitioning (`pcse/crop/lintul3.py`), with no bean set.
- AquaCrop bean parameters (p-exp, p-sto, base/upper temperature) were not retrieved. A candidate is the [FAO AquaCrop annexes](https://www.fao.org/docs/devbuildingforwardbetterlibraries/default-document-library/annexesaquacropv73.pdf) (not read).

---

## 4. FTSW thresholds for bean transpiration and leaf expansion

### Takeaway
A single 0.4 threshold for both conductance and expansion is the common simplification (pea: both 0.4). But reviews and most legume studies show expansion responds earlier (FTSW about 0.45–0.65) than transpiration (about 0.25–0.4). Bean-specific transpiration thresholds range from 0.17 to 0.36 and depend on cultivar and evaporative demand. Recommend separate parameters: expansion about 0.55 and conductance about 0.35, with genotype variance.

### Cited Findings
- Review across species: average PAW threshold is 0.56 for tissue expansion versus 0.40 for gas exchange, and 0.37 for stomatal conductance. Expansion thresholds are 0.66 in dicots, 0.50 in monocots, 0.72 in coarse and 0.43 in fine soils. Thresholds depend on evaporative demand, root distribution, texture and bulk density — [Sadras & Milroy 1996, Field Crops Res.](https://consensus.app/papers/details/560119c017565009bf2305e34afc99f2/?utm_source=claude_desktop)
- Common bean cultivars: FTSWc 0.36 ('Garapiá') and 0.23 ('Triunfo') in a low-demand season. In a high-demand off-season, 'Garapiá' FTSWc fell to 0.17 — [Pohlmann et al. 2022, Pesq. Agropec. Bras.](https://consensus.app/papers/details/11dae588fa535ebb8ddca34ccef94be9/?utm_source=claude_desktop)
- Mesoamerican common bean germplasm shows high variation in FTSWc (the breakpoint where normalised transpiration ratio declines), with QTL on Pv01 and Pv07 — [Cordoba-Novoa et al. 2024](https://consensus.app/papers/details/654eb84bfa13575e8b93a921941107d3/?utm_source=claude_desktop)
- Common bean: transpiration vs FTSW follows a linear-plateau, with similar thresholds under regulated deficit irrigation (RDI) and partial root drying (PRD) — [Wakrim et al. 2005](https://consensus.app/papers/details/218e638f2c0d5be0820d24dc1486edc9/?utm_source=claude_desktop) (threshold value not in abstract).
- Field pea: transpiration and leaf expansion are both unchanged until FTSW 0.4, then decline linearly to 0 — [Lecoeur & Sinclair 1996, Crop Sci.](https://consensus.app/papers/details/91e5b8bdc8d1541aa1a1057743336698/?utm_source=claude_desktop)
- Chickpea: leaf expansion threshold 0.48, transpiration 0.34 — [Soltani et al. 2000, Field Crops Res.](https://consensus.app/papers/details/7d9e3a9a14415d4db0087e83df989f22/?utm_source=claude_desktop)
- Soybean (six European cultivars): expansion FTSWt 0.44–0.93, transpiration 0.17–0.56, senescence 0.05–0.16 — [Kang et al. 2024, J. Agron. Crop Sci.](https://consensus.app/papers/details/65c89929d948573a9518dd523952d74f/?utm_source=claude_desktop)
- Cowpea: leaf expansion declines earlier in drying than transpiration in all eight genotypes, and recovers within about 1 d of rewatering — [Manandhar et al. 2017, Crop Sci.](https://consensus.app/papers/details/492a0a5595415a02968812e236f1f662/?utm_source=claude_desktop)
- Sunflower: expansion about 0.6, transpiration about 0.4 — [Casadebaig et al. 2008](https://consensus.app/papers/details/913adc344b3a5ba09a5bcea54cf92466/?utm_source=claude_desktop)
- A counter-example: soybean stomatal conductance threshold 0.64 versus leaf expansion rate 0.29, so the ordering is not universal — [Liu et al. 2003, Funct. Plant Biol.](https://consensus.app/papers/details/cf9cbeca1c6756fb95a953d3229baa71/?utm_source=claude_desktop)
- CROPGRO handles the expansion/transpiration difference without FTSW. SWFAC = root water supply / potential transpiration; TURFAC (expansion) uses RWUEP1 = 1.50 (SPE line 74), so expansion declines once supply is below 1.5× demand. (Equation form recalled from DSSAT documentation; parameter value from [BNGRO048.SPE](https://github.com/DSSAT/dssat-csm-data).)

### Inferences
- The plan applies one factor (threshold 0.4) to both conductance and expansion. Split it into two species parameters:
  - FTSW_expansion ≈ 0.5–0.6 (dicot average 0.66 in Sadras & Milroy; chickpea 0.48; soybean ≥0.44)
  - FTSW_conductance ≈ 0.3–0.4 (bean 0.17–0.36; pea 0.4; chickpea 0.34)
  - Both should be heritable, so genotypes can differ, as Pohlmann and Cordoba-Novoa show.
- The bean drought-abortion trigger of FTSW < 0.3 is plausible, as it sits below the transpiration threshold, but no bean study tying pod abortion to a specific FTSW was found.
- Because the coupled Medlyn model already lowers conductance through VPD, applying the FTSW factor to g1 (or to Vcmax and g1 together) rather than directly to transpiration avoids double-counting demand effects (inference).

### Gaps
- No bean-specific leaf expansion vs FTSW threshold was found. Sinclair & Ludlow 1986 and Devi/Sinclair Phaseolus papers were not retrieved directly.

---

## 5. Bean phenology: base temperature, thermal times, heat and cold thresholds, photoperiod by habit

### Takeaway
Published base temperatures for bean development cluster at 5–8 °C: CROPGRO 5, Ferreira 6.8, Dapaah 7, APSIM 7.5. The plan's 8 °C is at the top of the range; 10 °C is used mainly in simple GDD schemes. Heat damage to reproduction starts around 28–32 °C by day and about 20 °C or more by night: seed set falls about 6.5% per °C above 28/18 °C. Temperate determinate bush beans are generally day-neutral, while some (mainly tropical or Andean) lines abort buds under long days.

### Cited Findings
- Green bean cv. Alcade (Portugal, hourly data), emergence to first flower: Tb 6.8, To 24.0, Tc 34.1 °C (piecewise linear). SE of predicted first-flower date 2.73 d — [Ferreira et al. 1997, Sci. Hortic.](https://consensus.app/papers/details/63549f78a71755b591f0df173f5004b8/?utm_source=claude_desktop)
- Pinto bean (Canterbury, NZ):
  - Emergence to flowering rate is linear in mean temperature (r² 0.98), with no photoperiod relationship. All stages follow thermal time above a 7 °C base.
  - Thermal times: E–F 306, F–podding 79, podding–physiological maturity 392, sowing–harvest 974 °C·d.
  - Source: [Dapaah et al. 1999, NZ J. Crop Hortic. Sci.](https://consensus.app/papers/details/348fda4e3d1352bf9b62b11537c631dc/?utm_source=claude_desktop)
- APSIM common bean: Tb 7.5, To 30, Tmax 40 °C — [Amiri et al. 2024](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015).
- CROPGRO: 5/27/37/45 vegetative, 5/24/34/44 early reproductive, 6/25/37/60 late reproductive — [BNGRO048.SPE lines 97–99](https://github.com/DSSAT/dssat-csm-data).
- Heat at flowering:
  - 32/27 °C day/night for 1–5 d at different reproductive stages reduced pod and seed set through non-viable pollen, failed anther dehiscence, and reduced pollen-tube penetration and female function — [Gross & Kigel 1994, Field Crops Res.](https://consensus.app/papers/details/d0e154d8f0035b178b28813744fe424a/?utm_source=claude_desktop)
  - Kidney bean cv. Montcalm, 28/18 to 40/30 °C: seed set falls 6.5% per °C and seeds per pod by 0.34 per °C. Seed size falls 0.07 g per °C above 31/21 °C. Yield losses are likely above 34/24 °C; elevated CO2 does not offset them — [Prasad et al. 2002, Global Change Biol.](https://consensus.app/papers/details/9cefc6f976d15b199249d9bb8c963749/?utm_source=claude_desktop)
  - 32/22 vs 25/15 °C for 7 d at flowering–podding: total seed weight −50% (tolerant) to −71% (sensitive); single seed weight −8% to −34% — [Sharda et al. 2026, Front. Plant Sci.](https://consensus.app/papers/details/0f6080758ba55e87a0bba510b6043b75/?utm_source=claude_desktop)
  - Field heat stress: high night temperatures had larger effects than daytime maxima on an Andean RIL population — [Vargas et al. 2021, PLoS ONE](https://consensus.app/papers/details/5a8ff961203c5fa8bd8965754a310aa4/?utm_source=claude_desktop)
  - 31/24 °C: heat-sensitive Calima pollen viability fell to 24.9%, versus 98.4% in the control — [Rose et al. 2023, Plants](https://consensus.app/papers/details/34c183781c585a078bb46d36e5481fdf/?utm_source=claude_desktop)
  - 37/26 °C from V4 to maturity: seed yield 16.4 → 7.5 g per plant — [da Silva et al. 2019, Sci. Agric.](https://consensus.app/papers/details/6a8f22ce9c9256378838e7bc3ef4be16/?utm_source=claude_desktop)
- Cold: in vivo pollen germination is lower at 8 and 12 °C than at 18 °C. Low night temperatures (8–12 °C) inhibited ovule viability; 30/8 °C day/night gave the lowest yields — [Dickson & Boettger 1984, J. Am. Soc. Hortic. Sci.](https://consensus.app/papers/details/473d4575e93a534f86639cf4532d0bd7/?utm_source=claude_desktop)
- Cool temperatures slow seedling emergence; emergence and early growth may be under different genetic control — [De Ron et al. 2016, Front. Plant Sci.](https://consensus.app/papers/details/d513c633c8cf586aa4befbe4725391b2/?utm_source=claude_desktop)
- Photoperiod in determinate types: three South American determinate varieties responded to long days (15 h vs 11–13 h) with abscission of flower buds on the main axis and branches, which explains their poor performance in the British summer — [Ojehomon, Rathjen & Morgan 1968, J. Agric. Sci.](https://www.cambridge.org/core/journals/journal-of-agricultural-science/article/abs/effects-of-day-length-on-the-morphology-and-flowering-of-five-determinate-varieties-of-phaseolus-vulgaris-l/FFA59EAC9176A2D316DB894392C258F4)
- CROPGRO: temperate Type I cultivars have PPSEN 0; some Andean determinate and indeterminate lines 0.015–0.05 h⁻¹; CSDL 12.17 h — [BNGRO048.CUL](https://github.com/DSSAT/dssat-csm-data).

### Inferences
- **Base temperature.** Keep the plan's base at 8 °C only if the §8.2 thermal-time targets (Pinzón-Sandoval, ~800 °C·d) were computed with base 8. Otherwise use 6–7 °C, with optimum plateau 24–30 °C and upper threshold 34–40 °C (field fits) or 44 °C (CROPGRO). Thermal-time targets from different papers must be recomputed to one base before comparison.
  - Dapaah: 974 °C·d (Tb 7) from sowing to harvest.
  - CROPGRO Seafarer: ≈1,160 °C·d (Tb 5).
  - Pinzón-Sandoval: ~800 °C·d (base unknown here).
- **Heat abortion.** The plan's "daytime temperature above ~32 °C" rule should also include night temperature. Suggested shape:
  - abortion probability rising from Tmax ≈ 30 °C and Tnight ≈ 20 °C
  - about −6.5% seed set per °C of daily mean above ~23 °C (Prasad: 28/18 mean 23)
  - a cold branch for nights below about 10 °C (Dickson & Boettger)
  - CROPGRO's quadratic pod-set curve 7/22/25/36 °C (on daily temperature) is a ready-made alternative.
  - Also, the plan's single "32 °C" threshold can miss the 31/24 °C damage seen in Rose et al.
- **Photoperiod.** Default day-neutral for the bush bean preset. Include PPSEN and CSDL as heritable parameters (the default allele is 0). Long-day bud abscission (Ojehomon) could be a late-genetics feature.

### Gaps
- Rainey & Griffiths 2005 and Porch & Jahn 2001 (snap bean heat tolerance; night temperature thresholds) were not retrieved. The ">20 °C night" figure is supported here only qualitatively by Vargas 2021 and by Prasad 2002's treatments.
- White & Laing (CIAT) photoperiod work by growth habit was not retrieved.
- No literature value for the snap bean leaf frost-kill temperature was found, beyond CROPGRO's −2.22 °C and −5 °C.

---

## 6. Soil hydraulic parameters, bucket practice, evaporation, soil temperature

### Takeaway
The plan's field capacity and wilting point are in the published ranges for loam and sand. Its clay field capacity (0.40) is high relative to one compiled table (0.32), and its clay wilting point (0.22) may be low relative to Saxton & Rawls. Daily tipping-bucket models with Ritchie evaporation are standard (DSSAT, APSIM). An hourly step needs rescaled rate constants.

### Cited Findings
- Compiled table ("Saxton and Rawls (2006), Cornell, USDA-NIFA, Minnesota Stormwater Manual"), field capacity / wilting point in vol/vol — [Minnesota Stormwater Manual](https://stormwater.pca.state.mn.us/index.php/Soil_water_storage_properties):
  - sand 0.17 / 0.025–0.09
  - loamy sand 0.09 / 0.04
  - sandy loam 0.14 / 0.05
  - loam 0.25–0.32 / 0.09–0.15
  - silt loam 0.28 / 0.11
  - clay loam 0.32 / 0.15
  - clay 0.32 / 0.20
  - (Note: loamy sand FC 0.09 < sand FC 0.17 looks inconsistent in the source.)
- Saxton & Rawls 2006 pedotransfer code exists in USDA WEPS — [saxpar.for](https://infosys.ars.usda.gov/svn/code/weps1/branches/weps.src.wepp/src/lib_wepp/saxpar.for); interactive retention curves at [soilwater.github.io](https://soilwater.github.io/interactive-swrc) (not read).
- DSSAT soil file parameters include SLDR ("drainage rate, fraction day⁻¹") and SLU1 ("stage 1 evaporation limit, mm") — DSSAT `Data/SOIL.CDE` lines 58 and 83 (bundled in DSSATTools; [dssat-csm-os](https://github.com/DSSAT/dssat-csm-os)).
- APSIM: SWCON per day, Ritchie U and CONA, curve number — [APSIM SoilWat](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/).
- CROPGRO bean soil-evaporation extinction KEP 0.65; crop coefficient SKCBmax 1.10 (short reference) — SPE lines 131–134.

### Inferences
- **Texture values.**
  - Loam 0.27 / 0.12 sits mid-range of 0.25–0.32 / 0.09–0.15. OK.
  - Sand 0.15 / 0.05: OK.
  - Clay 0.40 / 0.22 versus the table's 0.32 / 0.20. Our recollection of the Saxton & Rawls 2006 Table 3 (not verified here) is clay ≈ 0.42 / 0.30, which would make the plan's clay plant-available water (0.18) too large; Saxton & Rawls would give ≈ 0.12.
  - Verify against the original Saxton & Rawls 2006, Soil Sci. Soc. Am. J. 70:1569, before finalising.
- **Soil temperature damping (physics inference, not from a source here).** Diurnal damping depth d = √(2κ/ω). With moist-soil thermal diffusivity κ ≈ 0.4–0.8 × 10⁻⁶ m² s⁻¹ and ω = 7.27 × 10⁻⁵ s⁻¹, d ≈ 0.10–0.15 m. Annual damping depth is about 19× larger (≈ 2–3 m). The diurnal amplitude falls to e⁻¹ at d with a phase lag of about 3.8 h per 0.1 m. DSSAT's STEMP (EPIC-derived) makes damping depth depend on bulk density and water content (recall; not verified).
- **Bucket at hourly step.** Acceptable if drainage uses a per-hour SWCON equivalent and evaporation uses Ritchie stage logic on cumulative time. Document in the model card, as the plan already does for its known limits.

### Gaps
- The Saxton & Rawls 2006 original table and Ritchie 1972 (Water Resour. Res. 8:1204) were not fetched. DSSAT STEMP damping-depth equations were not retrieved.

---

## Summary table: plan values vs model and literature values

| Plan item (v3) | Plan value | Model / literature value | Verdict |
| --- | --- | --- | --- |
| Development cardinal temperatures | 8/28/38 °C, single peak | CROPGRO 5/27/37/45 (veg), 5/24/34/44 (repro); Ferreira 6.8/24/34.1; APSIM 7.5/30/40; Dapaah Tb 7 | Use trapezoid; lower Tb to 5–7; raise Tmax to ≥40 |
| Emergence | 60–80 °C·d | CROPGRO 53–75 °C·d (Tb 5); WOFOST soybean 70 (Tb 7) | OK |
| Sowing to maturity | ~800–1,200 °C·d, 65–93 d | Seafarer ≈1,160 °C·d (Tb 5) ≈ 78 d at 20 °C; Dapaah 974 (Tb 7) | OK; state base temperature for each target |
| Photoperiod | none | Temperate Type I PPSEN 0; CSDL 12.17 h | OK for M0; add heritable PPSEN/CSDL |
| Water-stress threshold | one FTSW 0.4 for conductance and expansion | Expansion 0.45–0.66; transpiration bean 0.17–0.36, legumes 0.34–0.4; WOFOST demand-dependent 0.22–0.48 | Split into two; consider demand dependence |
| Drought pod abortion | FTSW < 0.3 | CROPGRO pod addition cut when SWFAC < 0.5 | Plausible |
| Heat pod abortion | Tday > 32 °C | Damage from 31/24 and 32/27; −6.5% seed set per °C above 28/18; night temperature matters; CROPGRO pod set 7/22/25/36 | Add night-temperature and cold branches |
| Pod growth / seed fill | 20 d / 20 d sequential | Shell 8 PTD (≈10 d); seed fill 15–22 PTD; overlapping | Shorten pod phase; overlap the two |
| Maintenance respiration | 0.015 g C per g N per day at 20 °C | WOFOST leaf 0.03 g CH2O per g DW per day at 25 °C (≈0.2–0.3 g C per g N); CROPGRO 0.0084 g per g per day at 30 °C | Plan likely 3–14× too low; recheck units |
| Growth respiration | 1.25 (20%) | Bean leaf ≈1.54, seed ≈1.6 g CH2O per g (≈27–30% in C terms); WOFOST CVL 0.61–0.72 | Raise and make organ-specific |
| Seed composition | not specified | Protein 23.5%, lipid 3.0%, carbohydrate 64% | Add to species file |
| SLA | 200–300 cm² g⁻¹ | SLAVR 280–325, SLAMIN 255; cold reduces SLA | OK as an average |
| Max trifoliate area | 50–100 cm² | SIZLF 133–180 cm² | Raise the maximum potential |
| Leaf frost | −1 °C | FREEZ1 −2.22, FREEZ2 −5 °C | Minor |
| Drainage | all excess one cell per hour | SWCON fraction per day (e.g. 0.02 to ~0.7) | Use a per-hour rate from SWCON |
| Soil evaporation | fraction of radiation at the soil | Ritchie two-stage (U ≈ 6 mm, CONA ≈ 3.5), KEP 0.65 | Adopt Ritchie |
| Clay FC / WP | 0.40 / 0.22 | 0.32 / 0.20 (MN compilation); Saxton & Rawls possibly 0.42 / 0.30 | Verify |
| Specific root length | ~70 m g⁻¹ | APSIM bean 65 m g⁻¹ | OK |
