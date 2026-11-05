# WP27 research notes: measured leaf/canopy minus air temperature (bean, spring wheat)

Tags: [F] = read in the source text (abstract counts if the number is in it); [F2] = read, but in a secondary source citing the original; [K, to verify] = from my own knowledge, not re-read; [I] = my inference or arithmetic.
Sign convention throughout: dT = Tleaf (or Tcanopy) − Tair. CTD = Tair − Tcanopy = −dT.

## Takeaway

- In bean, field CWSI lower baselines (well watered, sunlit, clear sky, around midday, wind about 2–3 m/s) give dT = a + b·VPD with a ≈ +2.3 to +3.5 °C and b ≈ −1.9 to −2.7 °C/kPa. At VPD 3 kPa that is dT ≈ −3.5 to −4.6 °C; at 4 kPa it is −5.4 to −7.3 °C [F, I]. The direct hand-held IR survey at CIAT (Deva et al. 2020) found a mean of −5.2 °C (range −13 to +2.1) in well-watered bean, with 27 % of air readings above 35 °C [F].
- The upper (non-transpiring) bean baselines are +2.4 °C (Erdem et al. 2006) and +3.0 to +4.0 °C (Silveira et al. 2024) [F]. Stressed field bean reached dT up to +5 °C (Kalaidjieva et al. 2017) [F]. Across crops, up to +8 °C at 800–1000 W m⁻² was measured (Jensen et al. 1990) [F], and a near-worst-case field model (water stress, low wind) gives +6 °C (Nelson & Bugbee 2015) [F].
- Low wind widens both tails: transpiring leaves cool less per unit of VPD, and closed leaves heat more. The published baselines were measured at moderate wind, so in still air (u ≤ 1 m/s) they overstate cooling [I].
- **Recommended test range, "hot still day, well-watered bean"** (Tair 33–36 °C, VPD 3–4 kPa, full sun with horizontal or sun-facing leaf, u ≈ 0.5–1 m/s, gs not limiting): **dT in [−7, +1] °C, with the expected centre at about −2 to −4 °C**. At minimum, assert dT < 0 when VPD ≥ 2.5 kPa and gs ≥ 0.3 mol m⁻² s⁻¹ [I].
- **Recommended test range, "stressed bean, stomata closed"** (same weather, gs ≈ 0.01–0.05 mol m⁻² s⁻¹, horizontal leaf): **dT in [+1, +8] °C, with the expected centre at about +3 to +5 °C**. The stressed-minus-watered gap on the same day should be about 3–8 °C (Lobo 2004 used 1–5 °C canopy differences as irrigation triggers; Jensen 1990 measured up to 6 °C) [F, I].
- Night, clear and calm: dT ≈ −0.5 to −3 °C (soybean −1.0 to −1.5 °C mean, tobacco down to −3.0 °C) [F]. A useful test is that dT < 0 and |dT| < 4 °C at 02:00 under clear sky.
- Spring wheat check: irrigated hot-environment CTD at midday is typically 4–8 °C (dT −4 to −8). Examples: Sudan maximum CTD 4.7–7.2 °C at Tair 37 °C; Chinese winter wheat at flowering, irrigated 7.8–10.3 °C and dryland 5.6–8.5 °C [F]. This is consistent with the bean range.

## Findings table

| # | Source | Species | Conditions (Tair, VPD/RH, radiation, wind, water, time) | dT = Tleaf − Tair | Method | Tag |
|---|---|---|---|---|---|---|
| 1 | Deva et al. 2020 | P. vulgaris (Calima, SAB 686, SEF 60) | CIAT Cali, Colombia, 965 m. H1 ambient: mean Ta 26.6 °C, RH 58 %, PAR 811. H2: median Ta > 37 °C, lower RH. Well watered. 08:00–16:00 | Mean −5.2 (SD 1.9); min −13.0; max +2.1. Heat-tolerant genotypes cooled 0.1–2 °C more (2.8 °C more at canopy top). 27 % of Ta readings > 35 °C but only 4.8 % of Tleaf readings. LTD correlates with VPD (Spearman −0.26 to −0.46) | MultispeQ v1 contactless IR on single leaves (air sensor ±0.5 °C) | [F] |
| 2 | Erdem, Şehirali, Erdem & Kenar 2006 | P. vulgaris, drip irrigated | Tekirdağ, Turkey, July 2003. Clear sky, 11:00–14:00. Monthly mean Ta about 25 °C; VPD axis 0–3 kPa | Lower baseline −2.6955·VPD + 3.5309 (R² 0.81, SEE 0.49 °C). Upper limit +2.4 °C | Raytek ST8 IRT, 3° FOV, ε 0.95, 20–30° oblique on sunlit plants | [F] |
| 3 | Silveira et al. 2024 | P. vulgaris IAC1850, grain filling | Campinas, Brazil. Tmax 30.7 °C (phase mean), Rn 329 W m⁻² (10–14 h), wind 2.9 m/s at 10 m (data with u > 6 m/s removed), VPD > 2 kPa used | Lower −1.92·VPD + 2.26 (R² 0.78). Upper 0.23·VPD + 3.01. Combined-method upper +3.96 °C. Intercept fell to −1.09 by 17 h. Figure span −12 to +6 °C | Apogee SI-411 IR radiometers, continuous | [F] |
| 4 | Idso 1982 (cited in #2) | P. vulgaris, sunlit | Arizona, clear sky | Lower baseline −2.35·VPD + 2.91 | Hand-held IRT | [F2] (bean coefficients read via Erdem 2006). Original paper also has a shaded bean baseline, which shaded/cloudy baselines put about 3.8 °C lower at VPD 3.6 kPa [F] |
| 5 | Kalaidjieva, Kuneva & Matev 2017 | Green bean (P. vulgaris) | Plovdiv, Bulgaria, 2010–12, field. Irrigation at 60–90 % FC pre-irrigation moisture plus a rainfed treatment | Stressed / no-rain: up to +5 °C. Optimally irrigated (80 % FC): negative, down to about −4.5 °C | Hand-held IRT | [F] |
| 6 | Lobo et al. 2004 | P. vulgaris cv. Carioca | Sete Lagoas, Brazil. Midday PAR about 1305; readings from 12 h | Canopy minus well-watered control (not air): triggers 1–5 °C; season means 0.6–2.8 °C | AG-42 IRT, 4° FOV, ε 0.99 | [F] |
| 7 | Bergamaschi et al. 1989 | P. vulgaris | Piracicaba, Brazil. 18-day drought, days of high evaporative demand | Stressed canopy warmer than air; irrigated cooler than air (abstract gives no numbers) | Remote IR sensor | [F] (qualitative) |
| 8 | Pastenes, Pimentel & Lillo 2005 (J Exp Bot 56:425) | P. vulgaris, field | Santiago, Chile, summer. Water-stressed vs watered; free vs restrained leaflets | Water-stressed free-moving leaves stayed **below ambient despite closed stomata**; restrained leaves heated up (numbers not retrieved) | Field | [F] (qualitative) |
| 9 | Pastenes et al. 2004 (J Plant Physiol 161:1315) | 4 bean cultivars, field | Watered vs stressed; midday maxima | Paraheliotropism "helps maintain leaf temperatures far below the ambient" | Field | [F] (qualitative) |
| 10 | Raeini-Sarjaz & Chalavi 2008 | P. vulgaris var. Provider, potted, well watered | Greenhouse, Ta 33–42 °C | Leaf stayed below air throughout; Tleaf was a function of Ta (r 0.92) | Thermocouple plus LI-6200 | [F] (sign only) |
| 11 | Coulson, Stigter, Akunda & Floor-Drees 1988 | P. vulgaris (3 cvs) plus tepary | Field, Kenya, 06–18 h | dT depends strongly on instrument pair (porometer sensor vs IRT; screen vs local air). Caution for test design | Delta-T porometer, Heimann KT15 IRT | [F] (method) |
| 12 | Nelson & Bugbee 2015 | Generic leaf (model + measurements) | Field sun, greenhouse, HPS, LED | No water stress: "typically within 2 °C of air". Near-worst case of water stress and low wind (modelled): field +6, LED +8, greenhouse +10, HPS +12 °C | Energy-balance model | [F] |
| 13 | Jensen, Svendsen, Jensen & Mogensen 1990 | Wheat, barley, rape, ryegrass | Denmark, humid temperate. Global radiation 800–1000 vs 100–200 W m⁻² | Up to +8 °C at high radiation; near 0 or negative at low radiation even under severe stress. Stressed minus irrigated up to 6 °C. Fluctuates 6 °C within minutes with clouds | IRT 8–14 µm | [F] |
| 14 | Mohammed, Tsubo et al. 2023 | Spring wheat Imam, Bohaine, irrigated | Gezira, Sudan, dry season. Max Ta 37.2–37.3 °C, Max Tc 34.5–36.1 °C. 2 m wind 3.2 (day) / 1.7 (night) m/s | CTD (Ta − Ts) maxima 4.7–7.2 °C (dT −4.7 to −7.2). Night surface temperature below air | SI-411 radiometer, 45°, continuous | [F] |
| 15 | Lv Shu-zuo et al. (Chinese, year n/a) | 20 winter wheat cvs | Flowering, 13:30–14:00 | CTD irrigated mean 8.81 (7.8–10.3); dryland mean 7.09 (5.6–8.5) | IRT | [F] (abstract) |
| 16 | Amani, Fischer & Reynolds 1996 | Spring wheat, 23–24 cvs, irrigated | Hot low-latitude Mexico (Tlaltizapán) | CTD vs yield r = 0.84 and 0.89. Best window 12:00–16:00. CTD values not in abstract | Hand-held IRT | [F] (no values) |
| 17 | Fischer et al. 1998 | Spring wheat, 8 cvs 1962–88, irrigated | NW Mexico (Obregón) | Newer cultivars' canopies 0.6 °C cooler | IRT | [F] |
| 18 | Reynolds et al. 1994 | Spring wheat, 16 genotypes | Hot irrigated (Mexico, Egypt, India, Sudan, Brazil). CTD 12:00–16:00 | Positive CTD–yield correlation (no values in abstract) | IRT | [F] |
| 19 | Steinmeyer, Lukac, Reynolds & Jones 2013 | Wheat ears | Growth cabinets, 28/18 °C vs 22/14 °C | Ear temperature depression 0.1–0.6 °C, up to about 1.5–2.0 °C under heat | FLIR T335 | [F] |
| 20 | Schwintzer 1971 | Soybean | Calm clear nights near freezing | Night dT: horizontal leaves −1.5, vertical −1.0 (5-night means) | Thermocouples | [F] |
| 21 | Cole 1967 | Tobacco, field | Rhodesia. Day Ta 25–28 °C | Night dT between +0.2 and −3.0 °C (depends on cloud, humidity, leaf cover). Daytime extremes printed as "+18 / −40" are OCR-garbled, likely +1.8 / −4.0 | Resistance thermometers | [F] (night) |
| 22 | Monteith & Szeicz 1962 | Tall crops, grass, soil | Cloudless summer days, UK | Daily max radiative temperature of tall crops close to max air temperature; short grass coldest at night | Radiometer | [F] |
| 23 | Ludlow & Björkman 1984 | Siratro (legume, paraheliotropic) | Restrained horizontal vs free leaves, droughted | Restrained droughted leaves suffered heat and photoinhibition. Damage above 42 °C, death at 48 °C | Field and lab | [F] |
| 24 | Zhu et al. 2015 (Photosynthetica 53:231) | Sophora alopecuroides (non-bean analogue) | Midday Ta 35–39.5 °C, PAR 2200–2500 | Free (paraheliotropic) leaves 36.80 °C vs restrained 37.73 °C: **0.9 °C cooler**. Forenoon (sun-tracking) free leaves 1.9 °C warmer | Leaf thermometry | [F] |
| 25 | Fu & Ehleringer 1989 (Plant Physiol 91:1162, doi 10.1104/pp.91.3.1162) | P. vulgaris Blue Lake Bush, well watered | Outdoor and lab | Leaflets turn more oblique as Ta rises, keeping lamina temperature near the photosynthetic optimum. Restrained vs free numbers not retrieved | Lab/outdoor | [F] (abstract, qualitative) |
| 26 | Berg & Hsiao 1986 (Crop Sci 26:980) | Small red kidney bean seedlings | Field, summer | Light avoidance at midday even when well watered, stronger in dry soil (no temperatures in abstract) | Inclinometer | [F] (qualitative) |
| 27 | Berg & Heuchelin 1990 | Soybean | Restrained horizontal vs free | Paraheliotropism gave lower leaf temperature with no gs difference (numbers not retrieved) | Field | [F] (qualitative) |
| 28 | Schakel & Hall 1979 | Cowpea, droughted | Field | Manually moving vertical leaflets to horizontal "caused substantial changes in leaflet temperatures" | Field | [F] (qualitative) |

### Heat thresholds (once leaf temperature exists)

| Source | Finding | Tag |
|---|---|---|
| Prasad, Boote, Allen & Thomas 2002, Glob Change Biol 8:710, doi 10.1046/j.1365-2486.2002.00508.x | Kidney bean cv. Montcalm, 28/18 to 40/30 °C. Seed set −6.5 % per °C above 28/18. Photosynthesis falls above 28/18. Seed size falls above 31/21 (−0.07 g °C⁻¹). Yield losses expected above 34/24 °C. Cause: pollen production and viability | [F] |
| Porch & Jahn 2001, Plant Cell Environ 24:723 | 32/27 °C day/night before anthesis gives abnormal pollen and anther development | [F] |
| Monterroso & Wien 1990, J Am Soc Hort Sci 115:631 | 2 days at 35 °C (10 h/day) causes flower and pod abscission. Sensitive window about 8 days before anthesis | [F] (treatment), [K, to verify] for the window |
| Suzuki et al. 2001, J Am Soc Hort Sci 126:571 | Green bean pollen stainability drops when mean air temperature exceeds 28 °C during 8–11 days before flowering | [F] |
| Gaur et al. 2014 (grain legumes review) | Heat-sensitive common bean loses yield when night temperature exceeds 20 °C | [F] (search-engine summary only) |
| Pastenes & Horton 1999, Photosynth Res 62:197 | F0-rise threshold 42–43 °C in two bean varieties | [F] |
| Smillie & Gibbons 1981, Carlsberg Res Commun 46:395 | At 1 °C/min heating, bean Fv becomes undetectable at 45.2 °C | [F] |
| Levykina & Karavaev 2016 | 15 min heating above 38 °C lowers Fv/Fm in bean leaves | [F] |
| Pastenes & Horton 1996 (Plant Physiol 112:1253) | Thylakoid reorganisation at 30–35 °C limits CO₂ assimilation | [F] |

## CWSI baselines table (dT = a + b·VPD, VPD in kPa, °C)

| Crop | Source | a (°C) | b (°C/kPa) | Upper limit (°C) | Conditions | dT at 3 kPa [I] | Tag |
|---|---|---|---|---|---|---|---|
| Bean | Idso 1982 (via Erdem 2006) | +2.91 | −2.35 | — | Arizona, sunlit, clear sky | −4.1 | [F2] |
| Bean | Erdem et al. 2006, Turk J Agric For 30:195 | +3.53 | −2.70 | +2.4 | Turkey, 11–14 h, clear | −4.6 | [F] |
| Bean | Silveira et al. 2024, ISPRS Arch XLVIII-3:501, doi 10.5194/isprs-archives-XLVIII-3-2024-501-2024 | +2.26 | −1.92 | 0.23·VPD + 3.01 (≈ +3.7 at 3 kPa); combined method +3.96 | Brazil, 10–14 h, u 2.9 m/s | −3.5 | [F] |
| Bean, combined method | Silveira et al. 2024 | dT = c·Rn + d·VPD; lower c = 0.0069 °C per W m⁻², d = −1.92; upper c = 0.012, d = 0 | | | ra 15.1 s/m, rc 56.4 s/m | — | [F] |
| Durum wheat, pre-heading | Idso 1982 | +3.38 | −3.25 | — | Arizona | −6.4 | [K, to verify] |
| Durum wheat, post-heading | Idso 1982 | +2.88 | −2.11 | — | Arizona | −3.5 | [K, to verify] (the original states the post-heading slope is less steep [F]) |
| Winter wheat | Alderfasi & Nielsen 2001, Agric Water Manage 47:69 | +0.41 | −1.35 | — | Colorado | −3.6 | [F2] (via Mohamed et al. 2024) |
| Wheat | Bijanzadeh & Emam 2012 | +1.89 | −1.00 | — | Iran, flowering to maturity | −1.1 | [F2] |
| Wheat genotypes | Mohamed et al. 2024, Agronomy 14:1814, doi 10.3390/agronomy14081814 | −3.5 to +0.07 | −2.1 to −0.37 | — | Egypt, arid | — | [F] (ranges; coefficient labels in the paper are ambiguous) |
| Rainfed wheat | Feiziasl et al. 2014 (via Feiziasl 2022) | — | — | ≤ +6 | Iran | — | [F2] (garbled text) |
| Barley (dryland) | Feiziasl 2022, J Water Soil 36:197 | +9.15 | −2.47 | +5.9 (5.5–6.9) | Maragheh, Iran | — | [F] (intercept looks implausible; OCR) |
| Several crops | Jensen et al. 1990 | poor fits in humid climate | | up to +8 | Denmark | — | [F] |
| Corn (context) | Payero et al. 2005 review | | | +3, +5, +4.6, > +10 across studies | | — | [F] |

## Inferences

1. [I] All three bean lower baselines agree within about 1 °C at VPD 3 kPa (−3.5 to −4.6 °C). They were all measured at midday in clear sky with typical field wind of 2–3 m/s. A Penman–Monteith-type leaf energy balance with bean-like gs (0.4–0.8 mol m⁻² s⁻¹) and ra ≈ 15–30 s/m should reproduce this. Silveira's fitted ra = 15 s/m and rc = 56 s/m give a usable calibration point for the WP27 golden test.
2. [I] Hot, still day: VPD at Ta 35 °C and RH 30 % is about 3.9 kPa. Baselines at moderate wind would give −5.4 to −7.4 °C. With u = 0.5 m/s the leaf boundary-layer conductance falls by a factor of about 2–3 (gb ∝ √(u/d)), so radiative heating gains weight and depression should shrink, plausibly to −1 to −4 °C. For a single horizontal, unshaded leaf near 1000 W m⁻², even a slightly positive dT is physically possible. Hence the asymmetric range [−7, +1].
3. [I] The upper baselines (+2.4 to +4 °C) come from canopies with residual transpiration and moderate wind. A single horizontal leaf with gs ≈ 0 in still air under full sun should exceed them. The Nelson & Bugbee field worst case (+6) and Jensen's maximum (+8) bound it, so the stressed range is [+1, +8].
4. [I] Real stressed bean leaves are often cooler than a horizontal model leaf because paraheliotropism cuts absorbed shortwave (Pastenes 2004, 2005 report stressed leaves below air). If WP27 has no leaf-angle response, the test should specify a horizontal leaf and say that field stressed bean can read near or below air for that reason. The one non-bean free-vs-restrained number (Sophora, 0.9 °C at midday) suggests a modest effect at Ta about 37 °C. Expect a 1–4 °C effect for strongly stressed bean, by analogy, unverified.
5. [I] Measurement caveat for any validation dataset: Coulson et al. (1988) and Deva et al. (2020, MultispeQ single-leaf IR, air sensor ±0.5 °C) show that the instrument pairing changes dT by around 1 °C or more. The Deva minimum of −13 °C is likely an outlier or artefact for the test range, so do not use it as a bound.
6. [I] Night: with clear sky and low wind, the model should give dT ≈ −1 to −3 °C. Under overcast skies or wind, it should approach 0. A positive night dT above about +0.5 °C under clear sky would indicate a bug.
7. [I] Heat thresholds to wire up later: about 28–32 °C day / 20 °C night for reproductive damage (use air or canopy temperature during the window about 10 days before anthesis), about 35 °C for abscission, and 42–45 °C leaf for irreversible PSII damage.

## Gaps

- No numeric free-vs-restrained leaflet temperatures for bean were retrieved. Pastenes 2005 (OUP PDF blocked) and Fu & Ehleringer 1989 (PMC captcha) would have them. Worth one manual look.
- The CIAT drought CTD (Polania, Rao, Beebe) papers give CTD–yield correlations but no absolute values in the material read. The Beebe et al. 2010 protocol (CIAT, core.ac.uk 132648057) describes the method only (Telatemp AG-42D, 50 cm, 45°).
- The Idso 1982 original Table I is OCR-garbled. The bean sunlit coefficients come from Erdem's citation, and the durum wheat numbers are from memory [K, to verify].
- No spring-wheat CTD values were read from the CIMMYT core papers (Amani 1996; Reynolds 1994) because the abstracts have no numbers. From memory, the commonly quoted irrigated Obregón/Tlaltizapán CTD range is about 3–10 °C [K, to verify].
- No dataset gives bean dT at controlled low wind. Bean leaf dT versus wind remains a gap, and the still-air range above is model-based reasoning [I].
- Night-time values are not bean-specific (soybean, tobacco).

## Full citations

1. Deva, C.R., Urban, M.O., Challinor, A.J., Falloon, P., Svitákova, L. (2020). Enhanced leaf cooling is a pathway to heat tolerance in common bean. Frontiers in Plant Science 11:19. doi:10.3389/fpls.2020.00019
2. Erdem, Y., Şehirali, S., Erdem, T., Kenar, D. (2006). Determination of crop water stress index for irrigation scheduling of bean (Phaseolus vulgaris L.). Turkish Journal of Agriculture and Forestry 30:195–202. https://aj.tubitak.gov.tr/agriculture/issues/tar-06-30-3/tar-30-3-4-0508-9.pdf
3. Silveira, J.M.C., Góes, J.A., Prado, D.F.S., Pires, R.C.M., Chiorato, A.F. (2024). Baselines for calculating crop water stress index in bean cultivation. Int. Arch. Photogramm. Remote Sens. Spatial Inf. Sci. XLVIII-3-2024:501. doi:10.5194/isprs-archives-XLVIII-3-2024-501-2024
4. Idso, S.B. (1982). Non-water-stressed baselines: a key to measuring and interpreting plant water stress. Agricultural Meteorology 27:59–70. doi:10.1016/0002-1571(82)90020-6 [DOI K, to verify]
5. Kalaidjieva, R., Kuneva, V., Matev, A. (2017). Using an infrared thermometer for establishment of water stress in green beans irrigated by different pre-irrigation soil moisture. https://scispace.com/papers/using-on-infrared-thermometer-for-establishment-of-water-3wldviycxi
6. Lobo, F.A., Oliva, M.A., Resende, M., Lopes, N.F., Maestri, M. (2004). Infrared thermometry to schedule irrigation of common bean. Pesquisa Agropecuária Brasileira 39(2):113–121. doi:10.1590/S0100-204X2004000200003 [DOI K, to verify]
7. Bergamaschi, H. et al. (1989). Deficiência hídrica em feijoeiro. IV. Modificações micrometeorológicas. Pesquisa Agropecuária Brasileira. https://scispace.com/papers/deficiencia-hidrica-em-feijoeiro-4oom9l93tz
8. Pastenes, C., Pimentel, P., Lillo, J. (2005). Leaf movements and photoinhibition in relation to water stress in field-grown beans. Journal of Experimental Botany 56(411):425–433. doi:10.1093/jxb/eri061
9. Pastenes, C., Porter, V., Baginsky, C., Horton, P., González, J. (2004). Paraheliotropism can protect water-stressed bean (Phaseolus vulgaris L.) plants against photoinhibition. Journal of Plant Physiology 161(12):1315–1323. doi:10.1016/j.jplph.2003.09.002
10. Raeini-Sarjaz, M., Chalavi, V. (2008). Pulvinus activity, leaf movement and leaf water-use efficiency of bush bean (Phaseolus vulgaris L.) in a hot environment. International Journal of Biometeorology 52:815–822. doi:10.1007/s00484-008-0175-2
11. Coulson, C.L., Stigter, C.J., Akunda, E., Floor-Drees, E. (1988). Instrument-based distortions of leaf/air temperature differences and interpretations of bean drought stress resistance. Tropical Agriculture (Trinidad) 65.
12. Nelson, J.A., Bugbee, B. (2015). Analysis of environmental effects on leaf temperature under sunlight, high pressure sodium and light emitting diodes. PLoS ONE 10:e0138930. doi:10.1371/journal.pone.0138930
13. Jensen, H.E., Svendsen, H., Jensen, S.E., Mogensen, V.O. (1990). Canopy-air temperature of crops grown under different irrigation regimes in a temperate humid climate. Irrigation Science 11. doi:10.1007/BF00189456
14. Mohammed, A.A.A., Tsubo, M., Ma, S., Kurosaki, Y., Ibaraki, Y., Tahir, I.S.A., Gorafi, Y.S.A., Idris, A.A.M., Tsujimoto, H. (2023). Micrometeorological comparison of canopy temperature between two wheat cultivars grown under irrigation in a hot environment in Sudan. Agronomy 13(12):3032. doi:10.3390/agronomy13123032
15. Lv Shu-zuo, Xie Hui-min, Zhang Jie. Study on the relation of CTD and drought resistance / water-saving in different winter wheats. https://scispace.com/papers/study-on-the-relation-of-ctd-and-drought-resistance-water-xj6bcjknshva
16. Amani, I., Fischer, R.A., Reynolds, M.P. (1996). Canopy temperature depression association with yield of irrigated spring wheat cultivars in a hot climate. Journal of Agronomy and Crop Science 176:119–129. doi:10.1111/j.1439-037X.1996.tb00454.x
17. Fischer, R.A., Rees, D., Sayre, K.D., Lu, Z.-M., Condon, A.G., Larqué Saavedra, A. (1998). Wheat yield progress associated with higher stomatal conductance and photosynthetic rate, and cooler canopies. Crop Science 38:1467–1475. doi:10.2135/cropsci1998.0011183X003800060011x
18. Reynolds, M.P., Balota, M., Delgado, M.I.B., Amani, I., Fischer, R.A. (1994). Physiological and morphological traits associated with spring wheat yield under hot, irrigated conditions. Australian Journal of Plant Physiology 21(6):717–730. doi:10.1071/PP9940717
19. Steinmeyer, F.T., Lukac, M., Reynolds, M.P., Jones, H.E. (2013). Quantifying the relationship between temperature regulation in the ear and floret development stage in wheat under heat and drought stress. Functional Plant Biology 40(7):700–707. doi:10.1071/FP12362
20. Schwintzer, C.R. (1971). Energy budgets and temperatures of nyctinastic leaves on freezing nights. Plant Physiology 48:203–207. https://scispace.com/papers/energy-budgets-and-temperatures-of-nyctinastic-leaves-on-3fa9upwln1
21. Cole, J.S. (1967). Some effects of the environment on the temperature of tobacco leaves in field plots in Rhodesia. Journal of Experimental Botany 18(2):254. doi:10.1093/jxb/18.2.254
22. Monteith, J.L., Szeicz, G. (1962). Radiative temperature in the heat balance of natural surfaces. Quarterly Journal of the Royal Meteorological Society 88:496–507.
23. Ludlow, M.M., Björkman, O. (1984). Paraheliotropic leaf movement in Siratro as a protective mechanism against drought-induced damage to primary photosynthetic reactions. Planta 161:505–518. doi:10.1007/BF00407082
24. Zhu, C.G., Chen, Y.N., Li, W.H., Chen, X.L., He, G.Z. (2015). Heliotropic leaf movement of Sophora alopecuroides L. Photosynthetica 53(2):231–240. doi:10.1007/s11099-015-0089-2
25. Fu, Q.A., Ehleringer, J.R. (1989). Heliotropic leaf movements in common beans controlled by air temperature. Plant Physiology 91:1162–1167. doi:10.1104/pp.91.3.1162
26. Berg, V.S., Hsiao, T.C. (1986). Solar tracking: light avoidance induced by water stress in leaves of kidney bean seedlings in the field. Crop Science 26:980–986.
27. Berg, V.S., Heuchelin, S. (1990). Leaf orientation of soybean seedlings. I. Effect of water potential and photosynthetic photon flux density on paraheliotropism. Crop Science 30:631–638.
28. Schakel, K.A., Hall, A.E. (1979). Reversible leaflet movements in relation to drought adaptation of cowpeas. Australian Journal of Plant Physiology 6:265–276.
29. Prasad, P.V.V., Boote, K.J., Allen, L.H., Thomas, J.M.G. (2002). Effects of elevated temperature and carbon dioxide on seed-set and yield of kidney bean. Global Change Biology 8:710–721. doi:10.1046/j.1365-2486.2002.00508.x
30. Porch, T.G., Jahn, M. (2001). Effects of high-temperature stress on microsporogenesis in heat-sensitive and heat-tolerant genotypes of Phaseolus vulgaris. Plant, Cell & Environment 24:723–731.
31. Monterroso, V.A., Wien, H.C. (1990). Flower and pod abscission due to heat stress in beans. J. Amer. Soc. Hort. Sci. 115:631–634.
32. Suzuki, K., Tsukaguchi, T., Takeda, H., Egawa, Y. (2001). Decrease of pollen stainability of green bean at high temperatures and relationship to heat tolerance. J. Amer. Soc. Hort. Sci. 126:571–574.
33. Pastenes, C., Horton, P. (1999). Resistance of photosynthesis to high temperature in two bean varieties. Photosynthesis Research 62:197–203.
34. Smillie, R.M., Gibbons, G.C. (1981). Heat tolerance and heat hardening in crop plants measured by chlorophyll fluorescence. Carlsberg Research Communications 46:395–403.
35. Levykina, I.P., Karavaev, V.A. (2016). Fluorescence induction changes in bean leaves after heat treatment. Moscow University Physics Bulletin.
36. Pastenes, C., Horton, P. (1996). Effect of high temperature on photosynthesis in beans (II. CO2 assimilation and metabolite contents). Plant Physiology 112:1253–1260.
37. Mohamed, N.E. et al. (2024). Utilizing infrared thermometry to assess the crop water stress index of wheat genotypes in arid regions under varying irrigation regimes. Agronomy 14(8):1814. doi:10.3390/agronomy14081814
38. Alderfasi, A.A., Nielsen, D.C. (2001). Use of crop water stress index for monitoring water status and scheduling irrigation in wheat. Agricultural Water Management 47:69–75.
39. Payero, J.O., Neale, C.M.U., Wright, J.L. (2005). Non-water-stressed baselines for calculating crop water stress index (CWSI) for alfalfa and tall fescue grass. Transactions of the ASAE 48(2):653–661.
40. Beebe, S.E., Rao, I.M., Blair, M.W., Acosta-Gallegos, J.A. (2010). Phenotyping common beans for adaptation to drought. In: Ribaut & Monneveux (eds), Drought Phenotyping in Crops, GCP, pp. 311–334 (protocol: https://core.ac.uk/download/132648057.pdf).
41. Gaur, P.M. et al. (2014). High temperature tolerance in grain legumes. https://scispace.com/papers/high-temperature-tolerance-in-grain-legumes-1p0llvitd3
