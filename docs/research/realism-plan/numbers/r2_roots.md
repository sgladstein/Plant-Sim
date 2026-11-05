# R2 root system: research notes for spec r2-root-system.md (PR #107)

Written 2026-11-05. Tags: [F] I read the source (an abstract counts when the number is in it; "fig" means the number was read from a figure or a garbled table and needs checking against the PDF); [K, to verify] from memory or a secondary mention, not re-read; [I] my inference or arithmetic. Spec section numbers refer to docs/research/realism-plan/r2-root-system.md at refs/pull/107/head.

## Takeaway

Recommended bands for the §5 targets:

| Target id | Spec band | Recommended | Basis |
| --- | --- | --- | --- |
| `root_length_day21` (single_pot, 21 days after sowing) | 20 to 120 m | **20 to 150 m**, aim for 40 to 90 m on seed means | Miller 2003 sand, 21 DAS, high P: basal + adventitious = 77 to 228 m (tap excluded); low P 36 to 90 m [F]. Polania 2009, 7.5 cm × 50 cm tubes, 32 DAS, irrigated: about 35 to 62 m [F, fig]. Guimarães 2002 tubes: about 30 to 50 m/plant in the top 20 cm at 15 DAE, about 100 to 140 m at 30 DAE [I, from their fitted curves]. Hand-measured outdoor tubes give much less: 7 to 9 m at 23 DAS (Barrios-Gómez 2012) [F]. Upper edge raised because Miller's best genotype is 228 m. Today's 1 m at day 20 is 1 to 2 orders of magnitude low. |
| `root_length_flowering` (single_pot at R1) | data gap | **60 to 180 m** (pot); do not use field excavation numbers as a target | Trindade & Araújo 2014, 4 kg pots: 93 to 158 m at pod setting (30 to 39 DAE), 105 to 180 m at early pod fill (41 to 50 DAE) [F]. Araújo et al. 2004, 3 kg pots, pod setting 45 DAE: about 75 to 116 m [F, garbled table]. Field excavation at 43 DAP recovers only 9 to 21 m (Miller 2003) [F], because fine roots are lost. R1 comes before pod setting, so the lower half of the Trindade range applies [I]. |
| `fine_root_share` (second-order laterals' share of length at R1) | 0.5 to 0.8 | **0.1 to 0.4** if "fine" means second-order laterals; 0.5 to 0.8 only if it means diameter < about 0.3 to 0.5 mm (first-order laterals count) | Miller 2003: in the field, "secondary branching was negligible even at 43 DAP" [F]. OpenSimRoot bean geometry gives about 10 to 30% [I]. In soybean, 2nd- and 3rd-order roots hold 39.4% and 34.2% of total length (Jin 2013) [F]. If order 1 is the tap root, the 3rd order equals our second-order laterals [I]. Pulses have much less length in the 0 to 0.2 mm class than wheat (Liu 2010) [F]. **Recommendation: define the metric by order and use 0.1 to 0.4.** |
| `split_pot_foraging` (wet:dry length at day 45) | ≥ 1.5 [I] | **≥ 1.5, expected 2 to 4** [I] | No bean wet:dry length ratio was found. Qualitatively, roots concentrate in the wet half: Sitka spruce (Coutts 1982) [F], Brassica napus (Wang 2005) [F]. Under alternating PRD, tomato root biomass rose 19 to 55% (Mingo 2004) [F]. Arithmetic: if the dry half stops at about 25 m by day 20 while the wet half grows from 25 to 100 m, the ratio is about 4 [I]. |
| `nitrate_patch_proliferation` (patch:control lateral + fine length) | ≥ 2 | **1.5 to 4, pass at ≥ 1.5** | Arabidopsis: a local NO3⁻ patch gives about 2× lateral elongation rate and no change in lateral number (Zhang et al. 1999) [F]. Arabidopsis in a soil patch: mean lateral length 1.8 vs 0.8 cm, and 50% of lateral length in the patch vs < 10% in the control area (Linkohr 2002) [F]. Barley (Drew 1975; Drew & Saker 1975): localized N promotes initiation and extension of 1st- and 2nd-order laterals [F, qualitative]. The commonly cited size is "several-fold" [K, to verify]. No legume or bean nitrate-patch number was found. Leguminous trees showed lower root plasticity (mentioned in passing by Yang 2021) [F, qualitative]. A nodulated bean probably responds less than barley [K, to verify]. |

Parameters in §3 and §4 of the spec, checked:

| Spec value | Verdict |
| --- | --- |
| Fine-lateral diameter 0.1 mm, tissue density 0.094 g/cm³, so about 1,350 m/g | Keep, but flag the density. Miller 2003 field SRL and diameters imply a whole-root tissue density of only about 0.03 g/cm³ [I]. Whole-system SRL is 75 to 100 m/g in the field (Miller) and 161 to 202 m/g in pots (Trindade) [F]. With 0.094 g/cm³, whole-system SRL falls in 80 to 200 m/g when length is mostly 0.2 to 0.3 mm laterals [I]. Consistent enough. |
| "Laterals on the tap root within days of germination" [K] | Confirmed in direction: at 7 days in growth pouches, primary-root laterals total up to 154 cm per seedling (Jalo MG-65; Vieira et al. 2008) [F]. Lateral-root zones were sampled from 4-day-old seedlings at 28 °C (Arthikala & Quinto 2018) [F]. **Target: first tap-root lateral cohort 3 to 5 days after germination, basal laterals 2 to 4 days after basal emergence** [I]. |
| Dry-soil slowing: θ_stop 0.05, θ_full 0.3 | Partly replace. At −1.6 MPa (about wilting point) the maize and soybean primary roots keep full elongation in the apical 3 to 4 mm, and only the basal growth zone is inhibited. Shoots stop at much higher water potentials (Sharp et al. 2004; Yamaguchi et al. 2010) [F]. The whole-root rate at −1.6 MPa is roughly 1/3 to 1/2 of control in maize seedlings (Sharp et al. 1988) [K, to verify]. **Set θ_stop near 0, or give f_w a floor of about 0.3 at θ_rel 0.05.** This is seedling data with seed reserves, so keep "~". |
| Xerobranching: no cohort below θ_rel 0.1; forms later if rewetted within 7 days | Keep the repression. Drop or shorten the recovery window. Lateral formation is "almost completely repressed" in air-filled soil gaps (micro-CT, maize and barley), and ABA abolishes primordia (barley and maize control about 10 laterals, ABA-treated 0 to 3) [F]. In barley, auxin after ABA did not restore lateral formation; the authors read this as irreversible loss of pericycle competence [F]. Shown only in cereals and Arabidopsis. |
| Hydropatterning rule | Keep. In the Arabidopsis agar assay about 80% of laterals start on the contact side; patterning toward soil is strong in wheat, barley, maize and Arabidopsis, with species differences (Bao et al. 2014, via a UCLouvain document) [F, secondary]. |
| Hydrotropism weight 0.05 | No data gathered; stays "~". |

## Root length time course (per plant)

| System | Cultivar | Age | Total root length | Notes / method | Source |
| --- | --- | --- | --- | --- | --- |
| 20 L pots, silica sand:vermiculite 20:80 + alumina-P, high P (100 µM) | G2333 / G19839 / G19833 / G4017 / DOR364 | 21 DAS (6 d germination + 2 wk) | Basal + adventitious = 148.6 / 228.0 / 201.4 / 77.8 / 84.7 m (tap root excluded) | Delta-T SCAN on neutral-red-stained roots | Miller et al. 2003 Table 6 [F] |
| Same, low P (1 µM) | same | 21 DAS | 62.8 / 90.0 / 56.9 / 39.7 / 39.6 m | | Miller 2003 [F] |
| Field, Typic Paleudult, raised beds, Costa Rica | 4 genotypes, medium P | 43 DAP | 15.3 to 20.8 m (intact); low P 9.4 to 16.7 m | Excavated whole plants; recovery incomplete [I] | Miller 2003 Table 2 [F] |
| Transparent tubes 7.5 cm × 50 cm, 2.4 kg Andisol:sand 2:1, irrigated at 80% FC | 8 lines (BAT 477, DOR 364, SEA 5, ...) | 32 DAS | about 35 to 62 m irrigated; 28 to 44 m under terminal drought | Read from figure | Polania et al. 2009 [F, fig] |
| PVC tubes 10.5 cm × 50 cm, sandy loam, outdoor, Montecillo (Tmin 7.7 °C) | 5 type-III cultivars | 23 DAS | 6.9 to 9.2 m (main root 0.28 to 0.39 m); root DW 142 to 199 mg | Units ambiguous (m vs cm); method not described | Barrios-Gómez et al. 2012 Table 4 [F] |
| PVC columns 25 cm × 40 cm, Latossolo, 3 plants per column, bulk density 1.0 to 1.6 | Pérola, BAT 477 | 15, 30, 45, 60, 72 DAE | RLD time courses as exponential-quadratic fits; see the inference below | Newman line-intersect method | Guimarães, Stone & Moreira 2002 [F for design; values I] |
| 4 kg pots, P 20 or 80 mg/kg | 24 genotypes, habits I to IV | Pod setting (30 to 39 DAE) | Means by habit 93 to 158 m; overall mean 102 (low P), 129 (high P) | Image analysis | Trindade & Araújo 2014 Table 3 [F] |
| Same | same | Early pod fill (41 to 50 DAE) | 105 to 180 m; mean 138 (low P), 163 (high P) | | Trindade & Araújo 2014 [F] |
| 3 kg pots, sandy clay loam, 3 mg/kg P | ICA Pijao, Carioca, Ouro Negro, Puebla 152 | Pod setting, 45 DAE | about 75 to 116 m | Table garbled in extraction; check the PDF | Araújo et al. 2004 [F, fig] |
| Growth pouches, no P | 19 genotypes | 7 d | Primary-root laterals up to 154 cm (Jalo MG-65) | | Vieira, Carneiro & Lynch 2008 [F] |
| Pots, soil (Oxisol), Cali | 4 habits | 3-day harvests | Time course exists; values not obtained | | Lynch & van Beem 1993 [K] |

**Guimarães 2002 inference [I].** The extracted equations read Y = a·exp(bX − cX²) (the exponent "²" was lost in extraction; the fitted curves only make sense with it). For the three steepest curves (equations 1, 3 and 5; probably the loose 0 to 20 cm layer), RLD is 1.0 to 1.5 cm/cm³ at 15 DAE, 3.0 to 4.3 at 30, 6.6 to 8.8 at 45, 9.5 to 12.6 at 60, and 8.9 to 13 at 72 DAE. The deep-layer or compacted curves are 0.1 to 3 cm/cm³. The 0 to 20 cm layer is 9,817 cm³ with 3 plants, so 1 cm/cm³ ≈ 33 m per plant. That gives about 33 to 47 m/plant at 15 DAE and 100 to 140 m at 30 DAE, in the top layer alone. Two caveats: the mapping of equations to panels is uncertain, and the line-intersect method counts all fine roots.

**Field context.**
- Rooting depth: drought-tolerant lines reached 1.3 m at Palmira, sensitive lines 0.8 m; at Quilichao (acid soil) all lines stayed above 0.8 m (Sponchiado et al. 1989) [F].
- In lysimeters, about 5% of pulse roots were below 60 cm. For all crops, 44% of root volume was in the top 20 cm and 70% in the top 40 cm. Root volume peaked at late flowering, then declined (Gan et al. 2011, chickpea, pea and lentil, Saskatchewan) [F].
- I found no measured field root length density profile (cm/cm³ by depth) for bean.

## Fine root and SRL table

| Quantity | Value | Conditions | Source |
| --- | --- | --- | --- |
| SRL, whole system | 75 to 100 m/g (by genotype and P) | Field, 43 DAP | Miller 2003 Table 9 [F] |
| SRL by class (including their laterals) | Adventitious 117, basal 89, tap 64 m/g | Field, 43 DAP | Miller 2003 text [F] |
| Mean diameter by class | Intact 0.73, adventitious 0.65, basal 0.68, tap 0.86 mm | Field, 43 DAP | Miller 2003 Table 1 [F] |
| SRL, basal + laterals | 161 to 202 m/g (mean 169 to 180) | 4 kg pots, pod setting to early pod fill | Trindade & Araújo 2014 [F] |
| SRL | about 103 to 107 m/g; samples 7.4 m per 60 mg ≈ 123 m/g | 3 kg pots, 45 DAE | Araújo 2004 [F, garbled] |
| Tap root share of root mass | 11 to 21% | 3 kg pots, 45 DAE | Araújo 2004 [F] |
| Primary root share of root system | about 10% (up to 50% when other classes are removed); hypocotyl-borne 25/30/38% at 2/5/8 cm seeding depth | Solution and solid media | Rubio & Lynch 2007 [F] |
| SRL | about 45 m/g | Hand-measured, 23 DAS | Barrios-Gómez 2012 [I] |
| Second-order branching | "negligible even at 43 DAP" | Field | Miller 2003 [F] |
| Soybean length share by order | 2nd order 39.4%, 3rd order 34.2% | Pot, seedlings, 3 P levels | Jin et al. 2013 [F]. Mapping to our orders [I] |
| Pulses vs wheat by diameter class | Pulses have much less length in 0 to 0.2 mm and more in 0.4 to 2.0 mm than wheat. Oilseeds have about 60% of length in 0 to 0.2 mm and 30% in 0.2 to 0.4 mm | Field lysimeters, Swift Current | Liu, Gan & Bueckert 2010 [F] |
| "Extra fine" share (oilseeds and pulses pooled) | about 85% | Lysimeters | Liu 2009 MSc thesis [F]; threshold cut off in the abstract |
| Fine root proportion (< 0.5 mm) | Correlates with total length (r = 0.58) and mean diameter (r = −0.95); mean diameter 0.50 to 0.76 mm (mean 0.63) under drought, 16 lines | 120 cm × 7.5 cm cylinders, 45 d after transplanting | Polania et al. 2017 [F]; FRP values only in a figure |
| SRL from OpenSimRoot diameters at 0.094 g/cm³ | Tap 17, basal 29, hypocotyl-borne 33, 1st-order lateral 150, fast fine 600, fine 1,350 m/g | Model parameters | Archive roots note [I] |
| Tissue density implied by Miller | Basal: 1/(8,900 cm/g × π × 0.0339² cm²) ≈ 0.031 g/cm³; intact ≈ 0.028 | | [I]. Lower than OpenSimRoot's 0.094; Miller's scanned diameters may be overestimated |

## Plasticity table

| Response | Species | Size | Conditions | Source |
| --- | --- | --- | --- | --- |
| Local nitrate, lateral elongation | Arabidopsis | about 2× elongation rate in the NO3⁻ zone; lateral number unchanged | Agar, segmented plates | Zhang et al. 1999 PNAS 96:6529 [F] |
| Nutrient-rich soil patch | Arabidopsis | Mean lateral length 1.8 ± 0.22 vs 0.8 ± 0.19 cm; 50% of lateral length in the patch vs < 10% in the same area of the control; density unchanged | Soil, 23 d | Linkohr et al. 2002 [F] |
| Nitrate patch on agar | Arabidopsis | Density +1 lateral per cm in the patch; length increase not significant; length outside the patch strongly reduced | Agar, 18 d | Linkohr 2002 [F] |
| Localized N or P | Barley | Localized promotion of initiation and extension of 1st- and 2nd-order laterals; axes little affected; shoot growth only slightly lower | Sand culture, 21 d, 3 vertical compartments | Drew 1975 New Phytol 75:479 [F, qualitative]; "several-fold" [K, to verify] |
| 1.0 mM NO3⁻ on 4 cm of one axis, 0.01 mM elsewhere | Barley | Marked local lateral growth and ¹⁵N uptake per unit root; whole-plant RGR equal to controls after 14 d | Solution, 29 d | Drew & Saker 1975 JXB 26:79 [F] |
| P patch | Bean | Root length in the patch comparable for low-P and high-P plants; low-P plants put about half as much root biomass in the patch and invest more in fine terminal roots there | Split root, soil medium | Snapp, Koide & Lynch 1995 Plant Soil 177:211 [F] |
| Xerobranching | Maize, barley, Arabidopsis | Lateral formation "almost completely repressed" in a 2 cm air gap. ABA: barley/maize control 9.5 ± 1.7 laterals vs 0 to 3 arrested primordia; ABA rises 4× within 4 h of water deficit; auxin does not rescue after ABA | Soil micro-CT; aeroponics | Orman-Ligeza et al. 2018 Curr Biol 28:3165 [F] |
| Hydropatterning | Arabidopsis, maize, wheat, barley | about 80% of laterals on the contact side (Arabidopsis agar); strong patterning toward soil in all species tested | Agar and soil macropores | Bao et al. 2014 PNAS 111:9319 [F, via secondary] |
| Low ψ, primary root | Maize, soybean | At −1.6 MPa the apical 3 to 4 mm keeps the well-watered elongation rate; the growth zone shortens; shoot growth stops at much higher ψ | Vermiculite, seedlings | Sharp et al. 2004; Yamaguchi et al. 2010 [F]; whole-root about 1/3 to 1/2 of control [K] |
| Partial root-zone drying | Tomato | Root biomass +55% (4 PRD cycles, glasshouse), +19% (2 cycles, cabinet); extra growth in the re-watered compartment | Split root, alternating | Mingo et al. 2004 [F] |
| Fixed wet/dry halves | Sitka spruce; Brassica napus | Most root growth in the wet half; selective placement in wet parts, foraging less effective in the second experiment | Split pots | Coutts 1982 [F]; Wang et al. 2005 [F] |
| Bean partial root drying | P. vulgaris | Studied, but only stomatal and ABA endpoints obtained | Split pots | Puértolas et al. 2016 (xylem ABA model); Vega Ponce & Mejía 2017 [F] |
| Compensation after root removal | Bean | Removing a root class increases growth of the others enough to keep plant performance | Solution and solid media | Rubio & Lynch 2007 [F] |

## Inferences

1. **Lateral timing.** The tap root grows 2.36 cm/d and OpenSimRoot has a 10 cm unbranched apical zone, so the first tap laterals appear about 4 to 5 days after germination. This matches lateral zones at day 4 (Arthikala) and up to 1.5 m of laterals by day 7 (Vieira) [I]. If the sim waits for "parent segment completes" plus a long apical zone, it will lag by weeks: suspect #1 in spec §3.1 is likely right.
2. **The scale gap is mostly lateral length.** Miller's day-21 sand total of 80 to 230 m sits on basal and adventitious axes with only tens of metres of axis. So more than 90% of that length must be first-order laterals at 0.15 cm spacing [I]. Fine second-order laterals are not needed to reach the day-21 target; first-order lateral density and length are.
3. **Fine-root share should be small for bean.** With OpenSimRoot geometry (fine laterals 0.5 to 2 cm long every 0.5 cm, behind a 4 cm apical zone, on 5 to 6 cm laterals), a lateral carries only about 1 to 2 cm of branched length, so fine roots make up about 10 to 30% of total length [I]. Together with Miller's "negligible secondary branching", 0.5 to 0.8 is too high. The spec's +35% organ estimate is fine, but the uptake gain from fine roots will be modest; R2a's uptake re-fit should not count on them [I].
4. **Field excavation vs pots.** Field excavation at 43 DAP recovers 9 to 21 m, against 100 to 160 m in 3 to 4 kg pots at the same age. Use pot bands for single_pot. A plot target would need cores (RLD) rather than excavation [I].
5. **Pot size matters.** The Miller (20 L) and Trindade (4 kg) numbers come from pots at least as large as single_pot's 24 cm pot (about 6 to 10 L). A smaller pot with a root-bound mat could still reach about 100 m by R1, since Polania's 2.4 kg tubes gave 35 to 62 m at 32 DAS [I].
6. **Xerobranching recovery.** The cereal evidence points to irreversible loss for the segment that was dry when it would have branched. The spec's 7-day rewet window has no support; a window of 0 to 2 days, or none, fits better [I].
7. **Nitrate patch in a legume.** Plant N status sets the systemic part of the response; with nodules supplying N, the local signal still acts but the systemic demand signal is weaker. Use pass ≥ 1.5 and expect 2 to 3 under low N [I; K for the mechanism].

## Gaps

- No bean root-length time course at 7/14/21/28 days in one experiment. Lynch & van Beem 1993 (Crop Sci 33:1253, doi:10.2135/cropsci1993.0011183X003300060028x) harvested at 3-day intervals in Oxisol containers, but I could not get the numbers. It is the best single source to fetch.
- No measured field RLD profile (cm/cm³ by depth) for bean. Guimarães 2002 is tubes and figure-only. Sponchiado 1989 (Exp Agric 25:249, doi:10.1017/S0014479700016756) has profiles behind a paywall.
- Bean root length by WinRHIZO diameter class: Polania 2017's fine-root proportion values are in a figure only. Not obtained.
- Drew 1975 fold changes and Hodge 2004 (New Phytol 162:9–24) summary values not read. No legume nitrate-patch number found.
- No bean xerobranching or hydropatterning study; the evidence is cereals and Arabidopsis.
- Bean root elongation vs soil water potential: no bean-specific threshold found (only maize and soybean seedlings).
- Split-pot wet:dry root length ratio: no quantitative dicot number obtained (Coutts 1982 and Wang 2005 full texts would have them).
- The Miller tissue density inconsistency (about 0.03 vs 0.094 g/cm³) is unresolved.

## References

- Miller CR, Ochoa I, Nielsen KL, Beck D, Lynch JP (2003) Genetic variation for adventitious rooting in response to low phosphorus availability. Funct Plant Biol 30:973–985. doi:10.1071/FP03078. PDF: https://plantscience.psu.edu/research/labs/roots/publications/pdfs/fp03078-miller.pdf
- Trindade RS, Araújo AP (2014) Variability of root traits in common bean genotypes at different levels of phosphorus supply and ontogenetic stages. Rev Bras Ciênc Solo 38:1170–1180. https://www.redalyc.org/pdf/1802/180231726013.pdf
- Araújo AP, Fernandes AM, Kubota FY, Brasil FC, Teixeira MG (2004) Sample size for measurement of root traits on common bean by image analysis. Pesq Agropec Bras 39:313–318. doi:10.1590/S0100-204X2004000400003
- Polanía JA, Rao IM, Beebe S, García R (2009) Desarrollo y distribución de raíces bajo estrés por sequía en fríjol común en un sistema de tubos con suelo. Agron Colomb 27:25–32. https://distantreader.org/stacks/journals/agrocol/agrocol-11327.pdf
- Polania J, Poschenrieder C, Rao IM, Beebe S (2017) Root traits and their potential links to plant ideotypes to improve drought resistance in common bean. Theor Exp Plant Physiol 29:143–154. doi:10.1007/s40626-017-0090-1
- Barrios-Gómez EJ, López-Castañeda C, Kohashi-Shibata J (2012) Comparación de las estructuras morfológicas en raíz e hipocótilo en frijol. Rev Mex Cienc Agríc 3(4):655–669. https://www.redalyc.org/pdf/2631/263123209003.pdf
- Guimarães CM, Stone LF, Moreira JAA (2002) Compactação do solo na cultura do feijoeiro. II. Rev Bras Eng Agríc Ambient 6:213–218. https://www.alice.cnptia.embrapa.br/alice/bitstream/doc/210888/1/rbea-2002.pdf
- Sponchiado BN, White JW, Castillo JA, Jones PG (1989) Root growth of four common bean cultivars in relation to drought tolerance in environments with contrasting soil types. Exp Agric 25:249–257. doi:10.1017/S0014479700016756
- Vieira RF, Carneiro JES, Lynch JP (2008) Root traits of common bean genotypes used in breeding programs for disease resistance. Pesq Agropec Bras 43:707–712. https://www.scielo.br/j/pab/a/jDvJHkYx7k6HcysYn88qvtg/
- Arthikala MK, Quinto C (2018) RbohA coordinates lateral root emergence in common bean. Commun Integr Biol 11(2). doi:10.1080/19420889.2018.1467188
- Lynch JP, van Beem JJ (1993) Growth and architecture of seedling roots of common bean genotypes. Crop Sci 33:1253–1257. doi:10.2135/cropsci1993.0011183X003300060028x [numbers not obtained]
- Rubio G, Lynch JP (2007) Compensation among root classes in Phaseolus vulgaris L. Plant Soil 290:307–321. doi:10.1007/s11104-006-9163-7
- Jin J, Liu XB, Wang GH, et al. (2013) Effect of phosphorus application on hierarchical lateral root morphology and phosphorus acquisition in soybean. J Plant Nutr 36:1578–1589. https://neigae.cas.cn/sourcedb/zw/lw/201308/t20130826_3918093.html
- Liu L, Gan Y, Bueckert R, et al. (2010) Fine root distributions in oilseed and pulse crops. Crop Sci 50:222–226. doi:10.2135/cropsci2009.03.0156
- Gan Y, Liu L, Cutforth H, Wang X, Ford G (2011) Vertical distribution profiles and temporal growth patterns of roots in selected oilseeds, pulses and spring wheat. Crop Pasture Sci 62:457–466. doi:10.1071/CP10406
- Zhang H, Jennings A, Barlow PW, Forde BG (1999) Dual pathways for regulation of root branching by nitrate. PNAS 96:6529–6534. doi:10.1073/pnas.96.11.6529
- Linkohr BI, Williamson LC, Fitter AH, Leyser HMO (2002) Nitrate and phosphate availability and distribution have different effects on root system architecture of Arabidopsis. Plant J 29:751–760. doi:10.1046/j.1365-313X.2002.01251.x
- Drew MC (1975) Comparison of the effects of a localised supply of phosphate, nitrate, ammonium and potassium on the growth of the seminal root system, and the shoot, in barley. New Phytol 75:479–490. doi:10.1111/j.1469-8137.1975.tb01409.x [K for the DOI]
- Drew MC, Saker LR (1975) Nutrient supply and the growth of the seminal root system in barley II. J Exp Bot 26:79–90 [K for pages]
- Snapp SS, Koide RT, Lynch JP (1995) Exploitation of localized phosphorus-patches by common bean roots. Plant Soil 177:211–218. doi:10.1007/BF00010127
- Orman-Ligeza B, Morris EC, Parizot B, et al. (2018) The xerobranching response represses lateral root formation when roots are not in contact with water. Curr Biol 28:3165–3173. doi:10.1016/j.cub.2018.07.074
- Bao Y, Aggarwal P, Robbins NE, et al. (2014) Plant roots use a patterning mechanism to position lateral root branches toward available water. PNAS 111:9319–9324 [K for pages; content via a UCLouvain document]
- Sharp RE, Poroyko V, Hejlek LG, et al. (2004) Root growth maintenance during water deficits: physiology to functional genomics. J Exp Bot 55:2343–2351 [K for citation details; content F]
- Yamaguchi M, Valliyodan B, Zhang J, et al. (2010) Regulation of growth response to water stress in the soybean primary root I. Plant Cell Environ 33:223–243. doi:10.1111/j.1365-3040.2009.02073.x (via PubMed)
- Yamaguchi M, Sharp RE (2010) Complexity and coordination of root growth at low water potentials. Plant Cell Environ 33:590–603. doi:10.1111/j.1365-3040.2009.02064.x (via PubMed)
- Sharp RE, Silk WK, Hsiao TC (1988) Growth of the maize primary root at low water potentials I. Plant Physiol 87:50–57. doi:10.1104/pp.87.1.50 [K for numbers]
- Mingo DM, Theobald JC, Bacon MA, Davies WJ, Dodd IC (2004) Biomass allocation in tomato plants grown under partial rootzone drying. Funct Plant Biol 31:971–978 [K for pages]
- Wang L, de Kroon H, Smits AJM (2005) Partial root drying effects on biomass production in Brassica napus and the significance of root responses. Plant Soil. doi:10.1007/s11104-005-5085-z
- Coutts MP (1982) Growth of Sitka spruce seedlings with roots divided between soils of unequal matric potential. New Phytol 92:49–61 [K for pages]
- Yang Z et al. (2021) Species-specific responses of root morphology of three co-existing tree species to nutrient patches. Front Plant Sci 11:618222. doi:10.3389/fpls.2020.618222
- OpenSimRoot Bean/Carioca parameter files, via docs/archive/research/notes/roots.md
