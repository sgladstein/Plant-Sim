# WP27 research notes: leaf optical properties (bean, wheat)

Tags: [F] = I read the source (or its abstract / a page quoting it, as stated); [K, to verify] = my own knowledge, not re-read this session; [I] = my inference or arithmetic (incl. PROSPECT-D runs below).

Working files:
- Script: `../prototype/prospect_bands.py`
- Output: `../prototype/prospect_results.txt`
- Tools: PyPI `prosail` (PROSPECT-D, Féret et al. 2017) and `pvlib` 0.16.1 (ASTM G173-03 reference spectra, bundled). EcoSIS, Zenodo and core.ac.uk could not be reached through the egress proxy (HTTP 403), so I **could not download measured LOPEX93/ANGERS/EcoSIS bean spectra**. The band averages below come from PROSPECT-D simulations with plausible bean and wheat inputs, not from measured bean spectra.

---

## 1. Takeaway (recommended parameters)

| Quantity | Bean (P. vulgaris) | Wheat | Basis |
|---|---|---|---|
| PAR absorptance, green leaf (400–700 nm) | **0.85** (range 0.82–0.90) | **0.85** (range 0.83–0.90) | Björkman & Demmig 1987: 0.838 ± 0.006 over 37 C3 species [F via BioNumbers]; PROSPECT-D gives 0.86–0.91 for bean/wheat inputs [I] |
| PAR reflectance / transmittance | ρ ≈ 0.07–0.09, τ ≈ 0.05–0.08 (σ_PAR = ρ+τ ≈ 0.15) | same | PROSPECT-D [I]; pyTSEB defaults ρ 0.07, τ 0.08 [F]; Goudriaan σ_PAR = 0.2 [K, to verify] |
| NIR absorptance (700–2500 nm, solar-weighted) | **0.20** (range 0.15–0.22) | **0.20** | PROSPECT-D: 0.15–0.20 [I]; Goudriaan σ_NIR = 0.8 → 0.2 [K, to verify]; Campbell & Norman NIR ≈ 0.2 [K, to verify] |
| NIR reflectance / transmittance | ρ ≈ 0.40, τ ≈ 0.42 | ρ ≈ 0.38, τ ≈ 0.43 | PROSPECT-D [I]; pyTSEB defaults ρ 0.32, τ 0.33 [F] |
| Broadband shortwave absorptance (whole solar spectrum) | **0.50–0.52** | **0.50–0.52** | PROSPECT-D 400–2500 nm: 0.47–0.53, plus UV and >2500 nm tails → ≈0.52 [I]; tealeaves default 0.50 [F]; Campbell & Norman ≈ 0.5 [K, to verify] |
| Solar energy fraction in PAR | 0.43 (global AM1.5), 0.42 (direct) | — | ASTM G173-03 integration [I] |
| Longwave emissivity | **0.97** (range 0.95–0.99) | **0.97** (range 0.96–0.99) | Bean 0.957 (Fuchs & Tanner 1966, cited in Chen 2015) [F, secondary]; leaf range 0.957–0.987 across species (Chen 2015 review) [F]; tealeaves 0.97 [F]; pyTSEB 0.98 [F] |
| Chlorophyll → PAR absorptance | **A_PAR = Chl / (Chl + 76)**, Chl a+b in µmol m⁻² (Evans 1993) | same | [K, to verify] for the constant 76; PROSPECT-D supports the shape above ~150 µmol m⁻² but sits higher at low Chl [I] |
| Typical green-leaf Chl a+b | ~400–550 µmol m⁻² (≈ 35–50 µg cm⁻²) | ~450–600 µmol m⁻² (flag leaf) | [K, to verify] |

**How to implement, in brief [I]:**
- `a_PAR(Chl) = Chl/(Chl+76)`, with Chl in µmol m⁻² derived from leaf N (yellowing). At 430 µmol m⁻² this gives the current 0.85, so behaviour at full green does not change.
- Add a floor of about 0.5 for yellow leaves that still hold carotenoids and brown pigments. The Evans form tends to 0 at Chl = 0, but PROSPECT and senescence measurements keep A_PAR at about 0.6–0.7 at Chl ≈ 90 µmol m⁻².
- `a_NIR = 0.20`, constant with Chl. In PROSPECT it changes only with leaf water: from 0.15 at Cw = 0.008 cm to 0.21 at Cw = 0.025 cm.
- Shortwave absorbed = `f_PAR·a_PAR + (1−f_PAR)·a_NIR`, with f_PAR ≈ 0.45–0.50. Use the sim's own split if it already has one. The ASTM AM1.5 global spectrum gives 0.43 for 400–700 nm, and 0.48 if the UV is lumped with PAR.
- Longwave ε = 0.97 for both species.

---

## 2. Findings

### 2.1 PAR absorptance, reflectance, transmittance
- Björkman & Demmig (1987) measured leaf absorptance across 37 C3 species and got a mean of **0.838 ± 0.006** [F; read on BioNumbers ID 104654, which quotes the paper]. Citation: Björkman O., Demmig B. (1987) Photon yield of O2 evolution and chlorophyll fluorescence characteristics at 77 K among vascular plants of diverse origins. *Planta* 170:489–504. https://doi.org/10.1007/BF00402983. BioNumbers: https://bionumbers.hms.harvard.edu/bionumber.aspx?id=104654. I could not check whether *Phaseolus* is among the 37 species. My recollection is that bean is in the dataset [K, to verify].
- A 2026 paper states that PAR absorptance "is commonly assumed as 85%" [F, abstract]. Xie, Yuan, Kang et al. (2026) Warming and drought effects on leaf light absorptance in three subtropical tree species. *BMC Plant Biology*. https://scispace.com/papers/warming-and-drought-effects-on-leaf-light-absorptance-in-96d6uwf1duuf. The abstract also says drought lowers absorptance by changing chlorophyll content and leaf structure (tree species, not bean).
- **Bean specifically:**
  - Madeira et al. (2000) worked on three green-bean cultivars [F, abstract]. They fitted linear SPAD–total Chl relations. Young greening leaves had low PAR transmittance and reflectance, and senescence raised PAR transmittance. Madeira A.C., Mendonça A., Ferreira M.E., Taborda M.L. (2000) Relationship between spectroradiometric and chlorophyll measurements in green beans. *Commun. Soil Sci. Plant Anal.* 31:631–643. https://doi.org/10.1080/00103620009370465.
  - Miller, Steven & Demetriades-Shah (1992) derived diffuse transmittance and front and back reflectance spectra of French bean leaves from layered-leaf measurements [F, abstract]. *Int. J. Remote Sens.* 13:3273–3286. https://doi.org/10.1080/01431169208904119 [DOI K, to verify]. Their spectra would give bean ρ and τ directly if the paper can be obtained.
  - Buschmann & Nagel (1993) measured in vivo reflectance of *Phaseolus vulgaris* leaves from yellow to green during greening, 400–800 nm [F, abstract]. *Int. J. Remote Sens.* 14:711–722. https://doi.org/10.1080/01431169308904370 [DOI K, to verify].
- PROSPECT-D values, weighted by G173 global irradiance, from my own runs [I]:

| Case (N, Cab µg cm⁻², Cw cm, Cm g cm⁻²) | ρ_PAR | τ_PAR | **A_PAR** | ρ_NIR | τ_NIR | **A_NIR** | **A_SW 400–2500** |
|---|---|---|---|---|---|---|---|
| Bean typical (1.6, 45, 0.016, 0.0035) | 0.070 | 0.041 | 0.889 | 0.403 | 0.417 | 0.180 | 0.502 |
| Bean thin/shade (1.4, 35, 0.012, 0.0025) | 0.075 | 0.066 | 0.860 | 0.385 | 0.465 | 0.149 | 0.472 |
| Bean dark (1.8, 60, 0.018, 0.0045) | 0.063 | 0.024 | 0.913 | 0.420 | 0.378 | 0.202 | 0.525 |
| Wheat flag (1.5, 50, 0.016, 0.0050) | 0.062 | 0.038 | 0.900 | 0.381 | 0.425 | 0.195 | 0.515 |
| Wheat low N (1.5, 30, 0.014, 0.0045) | 0.086 | 0.069 | 0.845 | 0.391 | 0.436 | 0.173 | 0.478 |
| Senescent yellow (1.6, 8, Cbrown 0.3) | 0.152 | 0.131 | 0.717 | 0.412 | 0.427 | 0.161 | 0.414 |

  - PAR weighting changes little: photon-weighted, energy-weighted and flat-spectrum A_PAR agree within 0.01 for green leaves and within 0.02 for yellow ones [I].
  - Shortwave absorptance weighted by the direct beam is about 0.013 lower than for global irradiance, because the direct beam is NIR-richer [I].
  - PROSPECT A_PAR tends to exceed integrating-sphere means of about 0.84. Possible reasons are that the inputs are guesses, and that measured absorptance includes sphere and edge losses. For that reason I recommend keeping 0.85, not 0.89, as the green-leaf value [I].

### 2.2 NIR and broadband shortwave
- Solar energy split in ASTM G173-03 global tilt (AM1.5, 1000 W m⁻²) [I, my integration of pvlib's bundled table]:

  | Band | Fraction |
  |---|---|
  | <400 nm | 0.046 |
  | 400–700 nm | 0.430 |
  | 700–2500 nm | 0.516 |
  | >2500 nm | 0.008 |

  The PAR fraction of the direct beam is 0.416. The common modelling value of 0.45–0.50 for the PAR share of global radiation reflects real skies with diffuse light and lower air mass [K, to verify]. If the sim already uses a PAR fraction, keep it for consistency.
- In PROSPECT, NIR absorptance for a green leaf is 0.15–0.20. It is set mainly by water (Cw) and dry matter. Chlorophyll and the structure parameter N barely change it: across N from 1.2 to 2.5, A_NIR stays 0.178–0.180 while the ρ/τ split moves from 0.34/0.49 to 0.51/0.31 [I].
- Full-spectrum shortwave absorptance for typical bean is 0.430·0.889 + 0.516·0.180 + 0.046·~0.92 (UV) + 0.008·~0.95 = **0.525** [I]. The UV and >2500 nm absorptances are [K, to verify].
- R package tealeaves (Muir 2019) defaults [F, source file `R/make_parameters.R` on GitHub cdmuir/tealeaves]: abs_s = **0.50** (0.3–4 µm) and abs_l = **0.97** (4–80 µm). Muir C.D. (2019) tealeaves: an R package for modelling leaf temperature using energy budgets. *AoB Plants* 11:plz054. https://doi.org/10.1093/aobpla/plz054 [K, to verify DOI].
- pyTSEB defaults [F, `Config_LocalImage.txt` on GitHub hectornieto/pyTSEB]: rho_vis 0.07, tau_vis 0.08, rho_nir 0.32, tau_nir 0.33, emis_C 0.98.
  - These imply A_NIR = 0.35, which is higher than PROSPECT's 0.18 and the canonical 0.2.
  - pyTSEB defines NIR as 700–2500 nm. Its values are probably tuned for canopy-scale use with dry or senescent material [I].
- Bookdown text *Physical Processes in Ecosystems* §11.4.1 [F]: "absorptivity to shortwave radiation of about 0.6, but it may vary from 0.4 to 0.8"; longwave emissivity 0.96. This is a secondary source and is higher than the other estimates. https://bookdown.org/huckley/Physical_Processes_In_Ecosystems/11-4-complete-energy-budget.html
- Campbell & Norman (1998), *An Introduction to Environmental Biophysics*, 2nd ed., Springer [K, to verify]:
  - Leaf absorptivity: PAR ≈ 0.8–0.85, NIR ≈ 0.2, total solar ≈ 0.5.
  - Longwave emissivity 0.94–0.99, 0.96–0.98 typical.
  - Canopy model scattering coefficients (Ch. 15): 1−a, giving σ_PAR ≈ 0.15–0.2 and σ_NIR ≈ 0.8.
- Goudriaan (1977) / Goudriaan & van Laar (1994) use σ_PAR = 0.2 (SUCROS/WOFOST/LINTUL) and σ_NIR = 0.8 [K, to verify].

### 2.3 Longwave emissivity
- Chen (2015) measured 0.978–0.985 for two orchids and *Pachira* [F]. Its literature review (Table 3) gives a leaf range of 0.957–0.987 [F]:
  - **bean 0.957** (Fuchs & Tanner 1966)
  - tobacco 0.971
  - alfalfa 0.987, olive 0.976, holm oak 0.985 (Rubio et al.)
  - horticultural crops 0.973–0.985 (López et al.)
  - 0.98 suggested as a general value for vegetables

  Chen C. (2015) Determining the leaf emissivity of three crops by infrared thermometry. *Sensors* 15:11387–11401. https://doi.org/10.3390/s150511387. Primary source for bean: Fuchs M., Tanner C.B. (1966) Infrared thermometry of vegetation. *Agron. J.* 58:597–601 [citation details K, to verify]. That measurement used the 8–14 µm band of an infrared thermometer.
- Inoue (1986) estimated canopy emissivity at about 1.0 for wheat, maize, soybean and sunflower leaf blades [F, abstract]. *Jpn. J. Crop Sci.* https://scispace.com/papers/remote-monitoring-of-function-and-state-of-crop-community-i-2h9mvy1c2t
- Wang, Horiguchi & Machimura (1994) recommend 0.97–0.98 for leaf infrared thermometry [F, abstract]. *J. Agric. Meteorol.* https://scispace.com/papers/emissivities-in-the-window-region-for-plant-canopies-and-its-2br8ygx5c1
- Sutherland (1986) measured broadband 8–14 µm emissivity of 0.97–0.99 for grass and leaves, with near-greybody spectra [F, abstract]. *J. Atmos. Oceanic Technol.* 3:199–202. https://scispace.com/papers/broadband-and-spectral-emissivities-2-18-mm-of-some-natural-14ev8ucp0b
- Idso, Jackson & Ehrler (1969) found that emittance differs among species. I recall their range as about 0.938–0.985 [K, to verify]. *Ecology* 50:899–902. https://doi.org/10.2307/1933705 [F for the existence of the paper and its qualitative finding; numbers K].
- Olioso et al. (2007) found that dry or senescent barley canopies can have emissivity below 0.98 [F, abstract]. This matters for late-season wheat. *IEEE GRSL* 4:112–116. https://scispace.com/papers/evidence-of-low-land-surface-thermal-infrared-emissivity-in-fktkyy484x
- Inference [I]: use ε = 0.97 for both crops. Changing ε by ±0.02 alters emitted longwave by about 2%, roughly ±9 W m⁻² at 25 °C, which is about ±0.3 K in leaf temperature. This is minor compared with absorptance uncertainty.

### 2.4 Datasets
- **LOPEX93** (JRC Ispra; Hosgood et al. 1995, EUR 16095 EN): about 70 samples of about 50 species, 400–2500 nm, ρ and τ with an integrating sphere [K, to verify]. I could not confirm that *Phaseolus* is included, because EcoSIS and the JRC PDF were unreachable.
- **ANGERS 2003** (Féret et al. 2008): 276 leaves of 43 species [K, to verify]. I did not confirm bean.
- **EcoSIS** (ecosis.org): blocked here (403).
- **Noda et al. (2014)** report ρ and τ spectra for 22 Japanese species [F, abstract]. These are probably wild species, not bean. *Ecol. Res.* https://scispace.com/papers/reflectance-and-transmittance-spectra-of-leaves-and-shoots-3xhj8xqcnv
- **Woolley (1971)** measured ρ and τ from 0.45 to 2.7 µm for faces and backs of leaves, at several angles, for several species [F, abstract]. I believe the species included soybean and maize [K, to verify]. *Plant Physiol.* 47:656–662. https://doi.org/10.1104/pp.47.5.656. The abstract does not give band averages.
- **PROSPECT N for bean:** I found no published inversion. Thin dicot leaves usually invert to N ≈ 1.4–1.8 and monocot (wheat) leaves to N ≈ 1.3–1.6 [K, to verify]. A_PAR, A_NIR and A_SW are insensitive to N (see 2.2) [I], so this gap matters little for the energy balance.

### 2.5 Adaxial/abaxial, pubescence, water stress, paraheliotropism
- **Adaxial vs abaxial:** In dicots, the abaxial (spongy mesophyll) side usually reflects more visible light than the adaxial side. Absorptance is nearly the same from both sides, typically within 0.01–0.03 [K, to verify]. Bean pulvinar movements can expose the abaxial side to the sun. The energy-balance effect is small, so ignore it [I].
- **Pubescence:** Bean leaves carry short hooked trichomes at low density. These are not dense enough to raise reflectance materially, unlike *Encelia* [K, to verify]. Wheat leaves are glabrous to sparsely hairy with a wax bloom. Glaucous wheat lines have higher visible reflectance, about +0.02–0.05 [K, to verify].
- **Water stress:** Optics change through leaf water (Cw). In PROSPECT, A_NIR falls from 0.18 to 0.15 when Cw drops from 0.016 to 0.008 cm, and A_SW falls by 0.02 [I]. This is negligible next to the geometric effect of wilting or leaf-angle change. Paraheliotropism alters incidence angle, not the optical properties, so it belongs in the geometry module [I].

### 2.6 Chlorophyll → absorptance
- Evans (1993) / Evans & Poorter (2001): **A = Chl/(Chl + 76)**, with Chl a+b in µmol m⁻² [K, to verify; I could not open either paper]. Evans J.R. (1993) Photosynthetic acclimation and nitrogen partitioning within a lucerne canopy. I. Canopy characteristics. *Aust. J. Plant Physiol.* 20:55–67. https://doi.org/10.1071/PP9930055 (existence and topic [F]; the equation is not in the abstract). Evans J.R., Poorter H. (2001) *Plant Cell Environ.* 24:755–767. https://doi.org/10.1046/j.1365-3040.2001.00724.x
- Comparison with PROSPECT-D (bean-like leaf, Cab swept from 2 to 80 µg cm⁻²) [I]:

| Chl µmol m⁻² | 56 | 111 | 223 | 334 | 446 | 557 | 669 | 892 |
|---|---|---|---|---|---|---|---|---|
| PROSPECT A_PAR (photon) | 0.50 | 0.63 | 0.77 | 0.84 | 0.87 | 0.90 | 0.91 | 0.93 |
| Evans Chl/(Chl+76) | 0.42 | 0.60 | 0.75 | 0.82 | 0.85 | 0.88 | 0.90 | 0.92 |

  - Best fits to PROSPECT: A = 0.968·Chl/(Chl+51), RMSE 0.029; A = Chl/(Chl+59), RMSE 0.033.
  - The two curves agree within 0.03 above about 200 µmol m⁻². Below that, PROSPECT stays higher because carotenoids still absorb [I].
- **Unit conversion:** 1 µg cm⁻² Chl a+b ≈ 11.2 µmol m⁻², using a mean molar mass of about 897 g mol⁻¹ at a:b = 3 [I].
- **SPAD:**
  - Markwell et al. (1995), soybean and maize: Chl (µmol m⁻²) = 10^(M^0.265), where M is the SPAD reading [K, to verify]. *Photosynth. Res.* 46:467–472. https://doi.org/10.1007/BF00032301
  - Uddling et al. (2007) found non-linear SPAD–Chl relations for wheat, birch and potato, with wheat similar to birch on an area basis [F, abstract]. *Photosynth. Res.* 91:37–46. https://doi.org/10.1007/s11120-006-9077-5
  - Earl & Tollenaar (1997), maize: leaf PAR absorptance against SPAD gave r² = 0.98 [F, abstract]. *Crop Sci.* 37:436–440. https://doi.org/10.2135/cropsci1997.0011183X003700020022x [DOI K, to verify]
- **Typical bean values:** well-fertilized field bean typically reads SPAD 38–48 [K, to verify]. By Markwell, SPAD 40 corresponds to about 10^(40^0.265) ≈ 10^2.66 ≈ 460 µmol m⁻² ≈ 41 µg cm⁻², and the Evans form gives A_PAR ≈ 0.86 [I]. Brazilian bean N-sufficiency studies, such as Silveira & Gonzaga (2017, *Pesq. Agropec. Trop.*) and Sant'Ana et al. (2010), use SPAD-502 for N management [F, abstracts]. I did not read their SPAD thresholds.

---

## 3. Parameter table (data/ format: name, value, unit, source, tag)

| name | value | unit | source | tag |
|---|---|---|---|---|
| leaf_par_absorptance_green (bean, wheat) | 0.85 | 1 | Björkman & Demmig 1987 Planta 170:489 (mean 0.838 ± 0.006, 37 C3 spp.) | F (secondary, BioNumbers) |
| leaf_par_reflectance | 0.08 | 1 | PROSPECT-D sim (this work); pyTSEB default 0.07 | I / F |
| leaf_par_transmittance | 0.07 | 1 | PROSPECT-D sim; pyTSEB default 0.08 | I / F |
| leaf_nir_absorptance | 0.20 | 1 | Goudriaan σ_NIR 0.8; PROSPECT-D 0.15–0.20 | K, to verify / I |
| leaf_nir_reflectance | 0.40 | 1 | PROSPECT-D sim (N 1.5–1.6) | I |
| leaf_nir_transmittance | 0.40 | 1 | PROSPECT-D sim | I |
| leaf_shortwave_absorptance (if one band used) | 0.50 | 1 | tealeaves default (Muir 2019); Campbell & Norman 1998; PROSPECT-D 0.50–0.52 | F / K / I |
| par_fraction_of_solar | 0.45 (keep sim value if set) | 1 | ASTM G173 gives 0.43 global, 0.42 direct | I (from F data) |
| leaf_lw_emissivity | 0.97 | 1 | tealeaves 0.97; Chen 2015 review 0.957–0.987; bean 0.957 (Fuchs & Tanner 1966) | F |
| chl_absorptance_half_sat (Evans) | 76 | µmol m⁻² | Evans 1993 Aust J Plant Physiol 20:55 | K, to verify |
| leaf_par_absorptance_floor (yellow leaf) | 0.5 | 1 | "~": PROSPECT-D gives 0.70 at 89 µmol m⁻² with brown pigment | I, "~" |
| leaf_chl_ab_green_bean | 450 | µmol m⁻² | "~": typical SPAD 40 via Markwell 1995 | K / I, "~" |
| leaf_chl_ab_green_wheat_flag | 500 | µmol m⁻² | "~" | K, "~" |

---

## 4. Inferences
- A two-band model (PAR, NIR) is worth adding over a single broadband value because chlorophyll loss changes only the PAR term. A full-green bean leaf with A_PAR 0.85 and A_NIR 0.20 absorbs 0.45·0.85 + 0.55·0.20 = 0.49 of shortwave. A yellow leaf at A_PAR 0.6 absorbs 0.38. That is a 22% drop in absorbed shortwave, which noticeably lowers leaf temperature [I].
- NIR absorptance is mostly a water and dry-matter effect, so it can be held constant. A refinement could tie it to leaf water content per area: A_NIR ≈ 0.10 + 4.5·Cw[cm], fitted to PROSPECT over Cw 0.008–0.025 cm: 0.146 → 0.208 [I, rough linear fit].
- The direct beam is NIR-richer than the global spectrum (PAR fraction 0.416 vs 0.430). A sim that splits direct and diffuse could use different PAR fractions, but the effect is ≤0.015 on A_SW [I].
- Emissivity uncertainty, ±0.02, is second-order for leaf temperature compared with absorptance uncertainty, ±0.05 on A_SW [I].

## 5. Gaps
1. **No measured bean spectrum was obtained.** EcoSIS, LOPEX93 and Zenodo were blocked by the proxy, and PMC is captcha-gated. Next steps: get Miller et al. 1992 (French bean ρ_ad, ρ_ab, τ) or the LOPEX93 species list, and recompute band averages from them with `prospect_bands.py`'s `bands()` function.
2. The constant 76 µmol m⁻² in Evans 1993 / Evans & Poorter 2001 is from memory. Check the paper text, including whether the equation was fitted to lucerne only.
3. Björkman & Demmig 1987: confirm whether *Phaseolus vulgaris* is in the 37 species and read its individual value.
4. Bean Chl per area and SPAD calibration: I found no bean-specific calibration that I could read. Madeira et al. 2000 has one, three cultivars, behind a paywall.
5. Emissivity of wheat leaves specifically: only canopy-level values were found, about 0.98–1.0 (Inoue 1986). Some studies report lower values for senescent cereals (Olioso 2007).
6. The Campbell & Norman table values and the Goudriaan σ values are from memory [K]. Verify against the book (C&N 1998 Table 11.4 / Ch. 15).
7. No PROSPECT N inversion was found for bean. The energy balance is insensitive to N, so this is low priority.
