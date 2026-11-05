# R3 plant hydraulics: parameter research (bean, Phaseolus vulgaris)

Researched 2026-11-05 for spec `docs/research/realism-plan/r3-plant-hydraulics.md` (PR #107, refs/pull/107/head).
Tags: [F] = number read in the source (abstract, publisher page, or full-text extraction); [K, to verify] = recalled or
seen only second-hand; [I] = my inference or unit conversion (shown).

Unit conversions used throughout [I]:
- 1 bar = 0.1 MPa. 1 cm s⁻¹ bar⁻¹ = 1e-2 m s⁻¹ / 0.1 MPa = 1e-1 m s⁻¹ MPa⁻¹. So 1e-6 cm s⁻¹ bar⁻¹ = 1e-7 m s⁻¹ MPa⁻¹.
- Axial conductivity in kg m s⁻¹ MPa⁻¹ ÷ ρ_w (1000 kg m⁻³) = m⁴ s⁻¹ MPa⁻¹.
- CPlantBox units (head in cm): kr[d⁻¹] = Lp[m s⁻¹ MPa⁻¹] × 9810 × 86400 × 1e-6 = Lp × 847.6;
  kx[cm³ d⁻¹] = K_x[m⁴ s⁻¹ MPa⁻¹] × 9810 × 86400 = K_x × 8.476e8 (both factors read from
  `src/functional/root_conductivities.py`, `convert_radial` / `convert_axial`) [F].
- 1 mol H₂O = 1.8e-5 m³; so K in m³ s⁻¹ MPa⁻¹ ÷ 1.8e-5 = mol s⁻¹ MPa⁻¹.
- Hagen–Poiseuille per vessel: K = π r⁴ / (8 η), η(20 °C) ≈ 1.0e-3 Pa s = 1.0e-9 MPa s.

---

## Takeaway

### Filled-in §4.4 table

| Name | Recommended value | Unit | Evidence | Tag |
|---|---|---|---|---|
| van Genuchten `n` per texture | loam 1.56, sand 2.68, clay 1.09: **confirmed** as the Carsel & Parrish (1988) class means (α: loam 0.036, sand 0.145, clay 0.008 cm⁻¹; θr .078/.045/.068; θs .43/.43/.38) | - | Table as reproduced (attributed to Leij, Alves & van Genuchten 1996, which tabulates Carsel & Parrish) | [F] values; [K] attribution to C&P directly |
| **but** anchoring caveat | Anchoring on the sim's FC/WP with these n: loam OK (α = 0.0098 cm⁻¹, θr = 0.099); **clay fails** (θr = −0.24, unphysical); **sand degenerates** (θr = 0.0498 ≈ WP, ψ → −∞ just below WP). Fix: clay n ≥ 1.20 (n = 1.20 → θr = 0.050, α = 0.0085 cm⁻¹); sand anchor FC at −10 kPa (α = 0.0196 cm⁻¹, θr still ≈ WP) or lower n | - | my solve, §7 below | [I] |
| `root_radial_conductivity` Lp_r, young/fine roots | **2e-7** (range 0.8e-7 to 6e-7) | m s⁻¹ MPa⁻¹ | Whole bean root systems, pressure chamber: Lp 0.8–6.1e-6 cm s⁻¹ bar⁻¹ = 0.8–6.1e-7 m s⁻¹ MPa⁻¹ (Fiscus & Markhart 1979); CPlantBox default kr 1.73e-4 d⁻¹ = 2.0e-7 | [F] + [I] conversion |
| Lp_r, single young roots (probe) | 1.8–8.4e-8 | m s⁻¹ MPa⁻¹ | P. coccineus, root pressure probe (Steudle & Brinckmann 1989) | [F] |
| Lp_r, older / suberized / secondary-thickened | **~5e-8** (≈0.25–0.4 × young) | m s⁻¹ MPa⁻¹ | Analogy: pine brown vs white roots 7.55e-7 vs 1.95e-6 cm s⁻¹ bar⁻¹ (ratio 0.39) (Sands, Fiscus & Reid 1982); bean-specific age series not found | [F] pine, [I] for bean |
| `root_axial_conductance` K_x, basal root (Phaseolus) | **~1e-10** (wild P. vulgaris 8.9e-11; wild P. acutifolius 1.0e-10; domesticated taxa higher; 800-fold range across Phaseolus) | m⁴ s⁻¹ MPa⁻¹ | Massas et al. 2022: 8.9e-8 kg m MPa⁻¹ s⁻¹ (Hagen–Poiseuille from vessels, 7-week basal roots) ÷ 1000 | [F] + [I] |
| K_x, taproot by position | apex ~2e-11; mid ~4e-10; base ~2e-9 (effective, 0.4 × ideal) | m⁴ s⁻¹ MPa⁻¹ | Barrios-Gómez et al. 2012 vessel data (23 DAS): apex 2.6 vessels × 29 µm, mid 5.5 × 52 µm, base 11.1 × 66 µm; HP ideal 4.5e-11 / 9.9e-10 / 5.2e-9; bean vessels conduct 0.4 × Poiseuille (Sands et al. 1982) | [F] anatomy, [I] K |
| K_x, laterals | ~5e-13 (young tip) to ~2e-11 | m⁴ s⁻¹ MPa⁻¹ | CPlantBox default lateral table kx1 4.07e-4 → 1.36e-2 cm³ d⁻¹ (maize-type, not bean) | [F] file, [I] conversion; bean value [K] |
| Metaxylem vessel diameter | 29 µm (1 cm from tip) → 52 µm (mid) → 66 µm (below hypocotyl), cultivar means 46–54 µm; drought-tolerant cv. smaller | µm | Barrios-Gómez et al. 2012, main root, 5 cultivars | [F] |
| Strock et al. 2021 per-class values | **not obtained** (paywalled; preprint and abstract carry no numbers) | - | Gap | - |
| `stem_xylem_conductivity` K_s; or set stem `k_h` directly | Stem k_h ≈ **2.9e-5 kg m s⁻¹ MPa⁻¹** (= 2.9e-8 m⁴ s⁻¹ MPa⁻¹) at ψ ≈ −0.8 MPa; LSC 6.2e-5 kg m⁻¹ s⁻¹ MPa⁻¹ | as stated | Holste, Jerke & Matzner 2006 (control plants) | [F] |
| K_s, `xylem_area_fraction` | K_s on xylem area ~5–15 kg m⁻¹ s⁻¹ MPa⁻¹, xylem fraction ~0.1–0.25 of stem cross-section | kg m⁻¹ s⁻¹ MPa⁻¹, - | No bean K_s number found; recommend calibrating `K_s · A_x` to match Holste k_h at the measured stem size | [K, to verify] |
| Stem P50 | −1.11 ± 0.04 (well-watered, cv. Provider); drought/shade lower by 0.15–0.25 | MPa | Haverroth et al. 2025; Holste et al. 2006 | [F] |
| `leaf_hydraulic_conductance` K_leaf | **~9** (well-watered, EFM, back-calculated); keep spec band 5–15 | mmol m⁻² s⁻¹ MPa⁻¹ | Haverroth et al. 2025: K_leaf fell to 0.52 after 72 h waterlogging = "94% decline" → control ≈ 0.52/0.06 = 8.7 | [F] + [I] |
| K_leaf vulnerability | P12 = −0.72, P50 = −1.00 ± 0.06 MPa | MPa | Haverroth et al. 2025 | [F] |
| Tuzet `s_f`, `ψ_f` | **ψ_f ≈ −1.0 MPa; s_f ≈ 4 MPa⁻¹** (range 3–6). Generic defaults in plantecophys: s_f 3.2 / ψ_f −1.9 (fsig_tuzet) or 3 / −2 (wrappers) are too tolerant for bean | MPa⁻¹, MPa | Bean stomata start closing at −0.4 to −0.6 MPa leaf ψ in stressed field plots (Pazzetti et al. 1992) and "about −12 bar" = −1.2 MPa (Cary & Wright 1971); K_leaf P50 −1.0; TLP ≈ −0.8 to −1.1 | [F] anchors, [I] fit |
| `turgor_loss_point` ψ_tlp (well-watered) | **−0.9** (range −0.6 to −1.3) | MPa | Zlatev & Stoyanov 2005 controls −0.60 to −1.31 (6 values, median −0.91); Haverroth −0.83; Kim & Lee-Stadelmann 1984 ~−1.2 | [F] |
| ψ_tlp (drought-acclimated) | −1.0 to −2.3 (osmotic adjustment 0.3–0.8 MPa) | MPa | Zlatev & Stoyanov 2005 stressed | [F] |
| π₀ (osmotic potential at full turgor) | −0.54 to −1.26 (pot, young); −1.12 ± 0.03 (field, irrigated, 8 MAGIC parents) | MPa | Zlatev & Stoyanov 2005; Buckley et al. 2024 (π_ft = 1.12 MPa) | [F] |
| ε (bulk modulus) | 5.5–13 | MPa | Zlatev & Stoyanov 2005 | [F] |
| ψ_0 (turgor-expansion reference) | −0.2 to −0.3 (keep) | MPa | Indirect: well-watered predawn −0.1 to −0.3 MPa (Parsons & Howe 1984) | [I] |

### §4.5 bands: confirmed or corrected

- **Predawn within 0.1 MPa of root-zone soil ψ: correct it to "within 0.2 MPa", or leave it out.** Well-watered bean
  controls were −0.1 to −0.3 MPa before lights-on with soil at field capacity (Parsons & Howe 1984) [F], so a
  0.1–0.3 MPa offset is normal. Bean also transpires at night, and inverted soil-to-root gradients have been seen in dry
  soil (Taerum 1973; Galván Tovar et al. 2006) [F]. The model has no night transpiration and no capacitance, so it will
  meet the 0.1 MPa test trivially. Treat this as a sanity check, not as validation [I].
- **Midday leaf ψ −0.4 to −0.8 MPa on a sunny, well-watered day: widen it to −0.4 to −1.1 MPa.** The current band is
  right for pots and growth chambers. In the field, irrigated beans reach about −1.0 MPa around 13:00 (Guimarães et al.
  2006, Carioca −0.97, RAB 96 −1.07) [F, but see Findings §5 for ambiguity]. Irrigated field crops, beans included,
  "seldom rose above −8 bar" in the afternoon (Cary & Wright 1971) [F]. My sanity estimate also lands at about −1.1 MPa
  for a young plant (Inferences §1).
- **Dry-down: midday ψ below ψ_tlp and turgor 0: confirmed reachable.** Bean leaf ψ reaches −1.5 to −2.4 MPa in
  pot dry-downs (Zlatev & Stoyanov 2005; Trujillo et al. 2013; predawn −1.5 MPa at permanent wilting, Galván Tovar
  et al. 2006) [F].
- **Turgor back above 0.5 within 6 sim-hours of watering: confirmed for moderate stress, with a caveat for severe
  stress.** After a 7-day stress, leaf RWC recovered in 2 h, leaf water in 3 h and stomatal diffusion resistance in 6 h
  (Kim & Lee-Stadelmann 1984) [F]. After severe stress (leaf ψ −1.6 to −2.5 MPa), leaf water status recovered only
  partly in 1 day and gs not at all; both recovered fully by 8 days (Trujillo et al. 2013) [F]. The hydraulic model will
  bring turgor back within hours either way. A gs recovery lag (ABA, embolism) is out of scope unless added
  deliberately [I].
- **Isohydric label: drop it.** In a field study of 12 common-bean genotypes, 6 were isohydric and 6 anisohydric, and
  3 of 4 tepary genotypes were anisohydric (Buckley et al. 2024) [F]. Bean should not be parameterized as strictly
  isohydric. The Tuzet factor with ψ_f ≈ −1.0 MPa gives intermediate behaviour [I].

---

## Findings by item

### 1. Roots: Lp_r, K_x, metaxylem, whole-system conductance
- **Fiscus & Markhart 1979** (Phaseolus, solution culture, pressure chamber): root-system Lp 0.8–6.1e-6 cm s⁻¹ bar⁻¹,
  non-monotone with plant size. Total root-system conductance L_R rose continuously with plant size. Soybean
  (Glycine) Lp 2.2–5.6e-6 cm s⁻¹ bar⁻¹ [F]. Converted: 0.8–6.1e-7 m s⁻¹ MPa⁻¹ (soybean 2.2–5.6e-7) [I].
  Fiscus EL, Markhart AH (1979) Plant Physiol 64:770–773. doi:10.1104/pp.64.5.770 [K for DOI string].
  https://scispace.com/papers/relationships-between-root-system-water-transport-properties-rqvytnobd0
- **Steudle & Brinckmann 1989** (P. coccineus, young roots, root pressure probe): Lp_r 1.8–8.4e-8 m s⁻¹ MPa⁻¹,
  similar for osmotic and hydrostatic gradients. Cortex-cell Lp 0.3–4.7e-6. Except within 20 mm of the apex, the
  limiting resistance was radial, not xylem [F]. Bot Acta 102:85–95 [K for page/DOI].
  https://scispace.com/papers/the-osmometer-model-of-the-root-water-and-solute-relations-1wsq0nklaz
- **Sands, Fiscus & Reid 1982** (Aust J Plant Physiol 9:559; doi:10.1071/PP9820559 [K]): bean vessels conduct
  0.4 × ideal Poiseuille at 25 °C. Bean axial conductance changes with temperature only through viscosity. Axial
  resistance was negligible in whole-root-system measurements. Pine white vs brown (suberized) roots: 1.95e-6 vs
  7.55e-7 cm s⁻¹ bar⁻¹ [F, via SciSpace full-text extraction].
- **Massas et al. 2022** (Crop Sci 62:2347–2365, doi:10.1002/csc2.20838; co-author Strock): 7-week greenhouse,
  basal roots only. Theoretical axial conductance (modified HP) varies about 800-fold across Phaseolus taxa. Wild
  P. acutifolius 1.0e-7 and wild P. vulgaris 8.9e-8 kg m MPa⁻¹ s⁻¹ were the highest wild values, and domesticated taxa
  had more conductance than wild ones. Metaxylem vessel number ranged from 4 to more than 140 per basal-root section;
  cross-section area 0.2–3.7 mm² [F]. https://cgspace.cgiar.org/server/api/core/bitstreams/d7051066-6455-4a3c-ba97-9372f9d2a77f/content
- **Barrios-Gómez et al. 2012** (Rev Mex Cienc Agríc 3(4):655–669; http://www.redalyc.org/articulo.oa?id=263123209003):
  main root at 23 DAS. Vessel diameter 29 µm (1 cm above tip, 2.6 vessels), 52 µm (middle, 5.5), 66 µm (1 cm below
  nodal region, 11.1). Root diameter about 840–940 µm. Cultivar means: vessel diameter 46–54 µm, vessel number 5–8;
  the drought-tolerant cultivar had smaller vessels. Hypocotyl diameter 2.2–2.8 mm, hypocotyl vascular cylinder
  0.7–1.0 mm, 12–15 xylem bundles [F].
- **Strock et al. 2021** (Plant Cell Environ 44:49–67, doi:10.1111/pce.13875; PubMed 32839986): abstract only. It
  reports intraspecific variation in metaxylem and axial conductance in both species. High-conductance genotypes
  used more water. In shallow-rooted P. vulgaris, low axial conductance conserved water; in deep-rooted P. acutifolius,
  high conductance exploited deep water [F]. Per-class numbers not obtained (Wiley blocked; Authorea preprint
  10.22541/au.158879193.37073815 abstract only).
- **CPlantBox defaults** (raw.githubusercontent.com/Plant-Root-Soil-Interactions-Modelling/CPlantBox/master/src/functional/root_conductivities.py) [F]:
  - Constants: kr 1.73e-4 d⁻¹ = 2.04e-7 m s⁻¹ MPa⁻¹; kx 4.32e-2 cm³ d⁻¹ = 5.1e-11 m⁴ s⁻¹ MPa⁻¹ [I].
  - Age tables: tap kr0 1.14e-3 → 5.17e-4 d⁻¹ (1.3e-6 → 6.1e-7 m s⁻¹ MPa⁻¹). Laterals kr1 4.11e-3 → 1.58e-3 d⁻¹
    (4.8e-6 → 1.9e-6). Tap kx0 6.74e-2 → 0.357 cm³ d⁻¹ (8.0e-11 → 4.2e-10 m⁴ s⁻¹ MPa⁻¹). Lateral kx1
    4.07e-4 → 1.36e-2 cm³ d⁻¹ (4.8e-13 → 1.6e-11) [I].
  - The age-table kr values are 10–20× higher than the bean pressure-chamber Lp, and the file's own comments mislabel
    units. Use the constants, not the tables, as a cross-check [I].
- **Aroca / Ruiz-Lozano bean aquaporin studies** (Benabdellah et al. 2009 Plant Mol Biol, doi:10.1007/s11103-009-9497-7;
  Calvo-Polanco et al. 2014 PLOS ONE): root L is reported per root dry mass (exudation and pressure chamber), and only
  relative changes appear in the accessible text. No absolute values extracted [F, qualitative].

### 2. Stem
- **Holste, Jerke & Matzner 2006** (Plant Cell Environ 29:836–843, doi:10.1111/j.1365-3040.2005.01454.x): at
  ψ ≈ −0.8 MPa, control stem k_h = 2.87e-5 kg m s⁻¹ MPa⁻¹ (porous-soil treatment 1.63e-5) and LSC = 6.18e-5
  kg m⁻¹ s⁻¹ MPa⁻¹. Drought, shade and porous soil shifted P50 0.15–0.25 MPa more negative [F, via SciSpace
  extraction]. Implied leaf area per stem = 2.87e-5 / 6.18e-5 ≈ 0.46 m² [I]. No K_s number was extracted.
- **Haverroth et al. 2025** (Plant Physiol 197(1):kiae336, doi:10.1093/plphys/kiae336): stem embolism P50 −1.11 ± 0.04,
  P12 −0.96 MPa [F].
- **Matzner et al. 2014** (J Exp Bot; light acclimation in two bean cultivars): reduced light lowered wood density and
  raised P50; numbers not extracted [F, qualitative].

### 3. Leaf and whole-plant conductance
- **Haverroth et al. 2025**: cv. Provider, 4-L pots, growth chamber at 26/22 °C, K_leaf by evaporative flux. K_leaf
  0.52 "after 72 h waterlogging; 94% decline" (the text gives units as mmol m⁻² s⁻¹, evidently missing MPa⁻¹).
  K_leaf P12 −0.72, P50 −1.00 ± 0.06 MPa; leaf embolism P50 −1.40; ψ_tlp −0.83 MPa [F]. Back-calculated control
  K_leaf ≈ 8.7 mmol m⁻² s⁻¹ MPa⁻¹ [I].
- **Buckley et al. 2024** (Plant Cell Environ, doi:10.1111/pce.15106; PDF https://buckleylab.ucdavis.edu/wp-content/uploads/sites/511/2025/04/buckley-et-al-2024-bean-physiology.pdf):
  whole-plant K = gs·Δw/(ψ_soil − ψ_leaf) in mmol m⁻² s⁻¹ MPa⁻¹. The authors give no absolute values and caution
  against comparing them. Irrigated gs reached more than 600 mmol m⁻² s⁻¹ (M4.66) [F].
- **Tsuda & Tyree 2000** (J Exp Bot 51:823, doi:10.1093/jexbot/51.345.823): kidney bean among six crops, HPFM and EF
  agree 1:1, and K varies diurnally in all organs. Bean numbers not accessible [F, abstract].
- **Mencuccini & Comstock 1999** (Aust J Plant Physiol 26:115, doi:10.1071/PP98137): 12 cultivars; leaf-specific
  conductances scale with transpiration. No numbers on the abstract page [F, qualitative].
- Sack & Scoffoni compilations (Scoffoni et al. 2017 J Exp Bot, 310 species): no bean entry confirmed [gap].

### 4. Turgor loss point, π₀, ε
- **Zlatev & Stoyanov 2005** (J Cent Eur Agric 6(1):5–14; https://hrcak.srce.hr/file/25931). Pots, growth chamber,
  14-day-old plants, 10-day withholding to soil ψ −0.9 MPa; columns are primary leaf control / stressed, then first
  trifoliate control / stressed [F]:

  | Cultivar | π₀ | ψ_tlp | ε |
  |---|---|---|---|
  | Plovdiv 10 | −0.65 / −1.28; −0.94 / −1.58 | −0.79 / −1.82; −1.02 / −2.10 | 7.4 / 6.6; 8.5 / 7.5 |
  | Dobrudjanski ran | −0.54 / −0.83; −0.64 / −1.06 | −0.60 / −0.98; −0.80 / −1.24 | 5.5 / 12.3; 8.1 / 10.5 |
  | Prelom | −0.92 / −1.71; −1.26 / −2.01 | −1.10 / −2.05; −1.31 / −2.29 | 6.9 / 9.7; 11.8 / 13.0 |

  Leaf ψ at the end of stress: controls −0.3 to −0.6, stressed −1.5 to −2.4 MPa.
- **Kim & Lee-Stadelmann 1984** (J Exp Bot 35:841, doi:10.1093/jxb/35.6.841): no osmotic adjustment. π at full
  saturation about −0.76 MPa; turgor loss at about −1.2 MPa leaf ψ and RWC 71%; the primary leaf is more elastic than
  the trifoliate [F, SciSpace full-text extraction; the −1.2 value's exact definition should be checked].
- **Parsons & Howe 1984** (Physiol Plant 60:197, doi:10.1111/j.1399-3054.1984.tb04564.x): prestressing lowers
  π by about 0.2 MPa in all genotypes. Tepary π is 0.15–0.25 MPa below P. vulgaris (Pinto, WHR). Tepary has 0.05–0.25
  MPa more turgor at RWC 65–80% and a lower ε [F].
- **Buckley et al. 2024**: field π_ft 1.12 ± 0.03 MPa in irrigated plants, +0.07 ± 0.03 MPa under drought (marginal
  adjustment) [F].
- **Bartlett et al. 2012** (Methods Ecol Evol 3:880–888) regression ψ_tlp = 0.832·π₀ − 0.631 [K, to verify]. Applied
  to π₀ = −1.12 it gives −1.56 MPa for field bean [I]. That is more negative than the pot data; field-grown leaves may
  have a lower TLP. Whether P. vulgaris is in Bartlett's database was not confirmed.

### 5. Leaf ψ diurnal course and stomatal closure
- **Parsons & Howe 1984**: before lights-on, controls −0.1 to −0.3 MPa; prestressed −1.07 to −1.21 MPa [F].
- **Guimarães, Stone & Brunini 2006** (Rev Bras Eng Agríc Ambient 10(1):70–75 [K for pages]): diurnal minimum about
  13:00 of −0.97 MPa (Carioca) and −1.07 MPa (RAB 96); 18:00 values −0.3 (irrigated) and −0.37/−0.48 MPa (deficit)
  [F, SciSpace extraction; it is ambiguous whether the minima are for irrigated or deficit plots].
- **Cary & Wright 1971** (Agron J 63:691 [K]): southern Idaho, irrigated field crops including bean. Water potentials
  "seldom rose above −8 bar" and were never seen above −5 bar. Bean never fell below about −20 bar even in very dry
  soil. Bean stomata begin closing at about −12 bar [F, SciSpace extraction].
- **Pazzetti et al. 1992** (field, cv. Carioca, four irrigation regimes): critical leaf ψ for stomatal closure about
  −0.6 MPa. In the driest regime, the first critical point was about −0.4 MPa at 08:00–09:00 [F, SciSpace extraction].
- **Omae et al. 2007** (Plant Prod Sci; snap bean, 11:00–12:00): unirrigated plants only about 0.15 MPa below
  irrigated ones at 57 DAS [F]. Osmotic −1.0 → −0.87, leaf ψ −0.92 → −0.65 MPa [F, extraction; direction looks
  garbled, treat with caution].
- **Galván Tovar et al. 2006** (pots, sandy loam): predawn equilibrium between 04:30 and 06:30. Under withholding,
  predawn ψ fell from −0.5 to −1.1 MPa as ET declined, and permanent wilting began at about −1.5 MPa [F].
- **Santos et al. 2009** (Biol Plant): after 10 days of withholding, ψ was −0.35 to −0.77 MPa across 5 genotypes [F].
- **Isohydric or anisohydric (Buckley et al. 2024)**: 6/12 P. vulgaris genotypes were isohydric (SEA 5, L88-63, Flor de
  Mayo Eugenia, M4.66, M7.87, M3.94) and 6 anisohydric (Victor, Pinto San Rafael, SER 118, Matterhorn, UCD 9634,
  M6.77). gs fell 55 ± 5% in isohydric, 48 ± 3% in anisohydric and 38 ± 7% in tepary genotypes. The drop in midday
  ψ_leaf ranged from +0.01 to −0.54 MPa, and the drop in ψ_soil reached −0.32 MPa [F].
- **Mencuccini, Mambelli & Comstock 2000** (Plant Cell Environ): pressurizing roots reopened bean stomata strongly in
  the morning and only weakly in the afternoon, a time-of-day effect not explained by ABA [F]. Tuzet will not
  capture this [I].

### 6. Recovery after rewatering
- **Kim & Lee-Stadelmann 1984**: RWC back to control within 2 h, leaf water within 3 h, stomatal diffusion
  resistance within 6 h; leaf growth recovery "considerably delayed" [F].
- **Trujillo, Rivas & Castrillo 2013** (J Plant Interact): rewatered at predawn ψ −1.6 to −1.7 MPa (8 days dry) or −2.4
  to −2.55 MPa (11 days). After 1 day, leaf water status had recovered partly and gs not at all; after 8 days both had
  recovered fully [F].
- Pepper analogue (Ismail & Davies): leaf ψ recovered at once, gs only after 48 h [F; not bean].

### 7. van Genuchten
- The texture table [F] (https://apps.dasnr.okstate.edu/SSL/soilphysics.okstate.edu/software/water/conductivity.html,
  attributed there to Leij, Alves & van Genuchten 1996; these are the Carsel & Parrish 1988 class means, Water Resour
  Res 24:755–769, doi:10.1029/WR024i005p00755 [K]):

  | Texture | θr | θs | α (cm⁻¹) | n | Ks (cm h⁻¹) |
  |---|---|---|---|---|---|
  | Sand | 0.045 | 0.43 | 0.145 | 2.68 | 29.7 |
  | Loam | 0.078 | 0.43 | 0.036 | 1.56 | 1.04 |
  | Clay | 0.068 | 0.38 | 0.008 | 1.09 | 0.20 |

  The spec's n values (1.56, 2.68, 1.09) are confirmed.
- With the C&P curves, θ(−33 kPa) is 0.164 for loam, 0.046 for sand and 0.347 for clay, against the sim's FC of 0.27,
  0.15 and 0.40. This confirms the spec's reason for anchoring [I].
- Anchoring results (θs, FC, WP from `crates/ps-soil/src/lib.rs`: loam .45/.27/.12, sand .40/.15/.05, clay
  .50/.40/.22) [I]:
  - **Loam, n 1.56**: α = 0.00976 cm⁻¹, θr = 0.0987. Fine.
  - **Sand, n 2.68**: α = 0.00593 cm⁻¹, θr = 0.0498. θr equals WP to 3 digits, so ψ(θ) is near-singular at WP. Anchoring
    sand FC at −10 kPa, the usual sand convention, gives α = 0.0196 but θr is still 0.0500. The steep n puts WP on the
    residual plateau. Option: n ≈ 1.8–2.0 for sand, or put a ψ floor in `f_dry`.
  - **Clay, n 1.09**: θr = −0.244, which is unphysical. The minimum n giving θr ≥ 0 is about 1.17. n = 1.20 gives
    α = 0.00848, θr = 0.050; n = 1.25 gives α = 0.00745, θr = 0.097.
  - **R3a's done-when (anchors to 1e-9, monotone) is still met mathematically for clay**, since θr < 0 keeps the curve
    monotone. But θr < 0 breaks `f_dry` ("falls to 0 near residual"). Add a test that θr ≥ 0.

### 8. Tuzet and Couvreur forms
- **Tuzet**: f_ψ = (1 + e^{s_f ψ_f}) / (1 + e^{s_f (ψ_f − ψ_leaf)}), as coded in plantecophys `fsig_tuzet` (defaults
  s_f 3.2, ψ_f −1.9; wrappers 3 and −2; "ψ_f = leaf water potential at which gs is 50% of max") [F]
  (https://rdrr.io/cran/plantecophys/src/R/tuzet.R). ClimaLand.jl uses the same form with min(1, …) and pressures in
  Pa [F]. The spec's form matches.
  - Tuzet, Perrier & Leuning 2003, Plant Cell Environ 26:1097–1116, doi:10.1046/j.1365-3040.2003.01035.x [K]. That the
    original paper itself used 3.2 / −1.9 is [K, to verify]; I saw those numbers only as code defaults.
  - Note: the s_f quoted in plantecophys is effectively MPa⁻¹ when ψ is in MPa, although its documentation lists it
    as "(-)" [I].
- **Couvreur, Vanderborght & Javaux 2012** (HESS 16:2957–2971, doi:10.5194/hess-16-2957-2012) [F]: three macroscopic
  parameters, the standard sink fraction SSF (the spec's SUF), K_rs, and a separate compensatory conductance K_comp. It
  also gives an effective soil ψ (the spec's ψ_seq).
  - The spec writes the compensation term with K_rs in place of K_comp. In Couvreur's framework K_comp = K_rs holds
    when axial resistance is negligible and is otherwise a separate parameter [K, to verify].
  - Recommend keeping `K_comp` as its own value (default K_rs). It is the natural tuning knob for the fidelity report's
    hard cases [I].

---

## Inferences
1. **Order-of-magnitude closure.** A young bean has about 50 m of root, radius 0.25 mm, so surface ≈ 0.079 m². With
   Lp 2e-7, K_rs ≈ 1.6e-8 m³ s⁻¹ MPa⁻¹ ≈ 0.87 mmol s⁻¹ MPa⁻¹. On 0.1 m² of leaf that is 8.7 mmol m⁻² s⁻¹ MPa⁻¹,
   about equal to K_leaf. Whole plant ≈ 4.4 mmol m⁻² s⁻¹ MPa⁻¹. At E = 5 mmol m⁻² s⁻¹, ψ_leaf ≈ −1.1 MPa from field
   capacity. With more root length per leaf area (flowering plants) it is −0.6 to −0.8. So the parameter set is
   self-consistent with field midday values, and the midday band should reach −1.1 MPa.
2. **Stem resistance is negligible when not embolized.** With Holste's k_h (2.9e-8 m⁴ s⁻¹ MPa⁻¹), a 0.3 m stem and
   0.46 m² of leaf, stem conductance is about 1e4 mmol m⁻² s⁻¹ MPa⁻¹, three orders above K_leaf. R1a's pipe-model
   coupling (stem thickening raises conductance) will not change ψ_leaf measurably. Its value is in a later embolism
   or vulnerability model. Do not spend tuning effort on K_s.
3. **The root axial conductance trait (Strock) matters only for young or distal segments.** Effective K_x near the
   apex (~2e-11 m⁴ s⁻¹ MPa⁻¹) against radial uptake from a 1 cm segment (2π·2.5e-4·0.01·2e-7 ≈ 3e-12 m³ s⁻¹ MPa⁻¹ per
   cm) gives an axial/radial length scale of only a few cm at the tip and metres at the base. For the M3 heritable
   trait, scale K_x per class by a genotype multiplier (Massas: up to about 800× across the genus, much less within
   P. vulgaris), not by absolute value.
4. **Tuzet fit.** Closure should start (f ≈ 0.9) near −0.6 MPa and reach about 0.1 near −1.4 MPa. A logistic 10–90%
   width of about 0.8 MPa means s_f = 2·ln 9 / 0.8 ≈ 5.5 MPa⁻¹, with ψ_f ≈ −1.0. The spec's s_f 3 (width 1.5 MPa)
   is acceptable, but closure starts already around −0.3 MPa, which may lower well-watered midday gs. Calibrate
   against the WP27 leaf-temperature checks as the spec says. Recommended default: s_f 4, ψ_f −1.0 (~).
5. **Turgor rule.** The spec's `(ψ − ψ_tlp)/(ψ_0 − ψ_tlp)` is linear in ψ. Real turgor follows P = ψ − π(ψ) with π
   from the PV curve. With π₀ ≈ −0.9 and ε ≈ 8 MPa, the linear form is within about 10% of turgor/turgor₀ over the
   range. Acceptable [I].

## Gaps
- Strock et al. 2021 per-root-class metaxylem diameters and K_x for bean (paywalled; worth a library fetch). Also
  Strock et al. 2019/2022 secondary-growth papers for K_x after secondary thickening.
- Bean-specific Lp_r by root age or suberization. Only whole-system values (Fiscus & Markhart) and a P. coccineus probe
  range exist; the age decline is by analogy.
- Bean stem K_s and xylem area fraction (Holste gives k_h and LSC only; stem diameter not extracted).
- An absolute K_leaf for bean. The value used is back-calculated from Haverroth's percent decline; the Fig. 1 controls
  would confirm it. Whole-plant K absolute values (Tsuda & Tyree 2000 tables).
- Absolute irrigated field midday ψ by genotype (Buckley Fig. 2 values not in the text).
- Original Tuzet 2003 parameter values. Whether Bartlett's 2012 database includes P. vulgaris, and the exact
  regression coefficients.
- Root-system conductance per unit root length or per plant from the Aroca and Ruiz-Lozano studies (values are per
  root dry mass; not extracted).
