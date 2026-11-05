# Numbers for specs R1, R2 and R3

Written 2026-11-05 by the research thread. It checks the [K, to verify] values and
validation bands in [r1-carbon-sinks.md](../r1-carbon-sinks.md),
[r2-root-system.md](../r2-root-system.md) and
[r3-plant-hydraulics.md](../r3-plant-hydraulics.md). Tags: **[F]** source read (an
abstract counts when it gives the number; numbers read off figures say so), **[K, to
verify]** from knowledge, **[I]** inference or arithmetic.

Notes with full citations:
- R1: [r1_sinks.md](r1_sinks.md)
- R2: [r2_roots.md](r2_roots.md)
- R3: [r3_hydraulics.md](r3_hydraulics.md)

Soil retention check script: [vg_fit.py](vg_fit.py).

## Changes the specs need (most important first)

1. **R3a's anchoring rule fails for clay and sand, and loam has a related problem.** Fitting
   the literature n to the sim's own field capacity and wilting point (`ps-soil`
   `Hydraulics`) gives [I, `vg_fit.py`]:

   | Texture | Result | Problem |
   | --- | --- | --- |
   | Loam | θ_r 0.099 | Above the loam air-dry content (0.05), and evaporation can dry the soil that far |
   | Sand | θ_r 0.050 | Equal to the wilting point, so ψ goes to −∞ just below it |
   | Clay | θ_r −0.24 | Impossible |

   **Fix: set θ_r = 0 and solve n and α** from the two anchors (θ(−33 kPa) = FC,
   θ(−1500 kPa) = WP). That gives:

   | Texture | n | α (cm⁻¹) | ψ at air-dry |
   | --- | --- | --- | --- |
   | Loam | 1.215 | 0.0305 | −88 MPa |
   | Sand | 1.289 | 0.088 | −36 MPa |
   | Clay | 1.165 | 0.0094 | −178 MPa |

   These are finite, monotone and close to the order of real air-dry soil. The n values
   differ from Carsel & Parrish's texture means (loam 1.56, sand 2.68, clay 1.09, which are
   confirmed [F]) because the sim's FC and WP are not theirs. Add a test that
   θ_r ≤ air-dry and that ψ is finite down to air-dry.
2. **R1a §3.3: OpenSimRoot's secondary-growth scaling is keyed on distance from the root
   base (cm), not on age.** Checked in `growthrates.xml` [F]:

   | Root | Factor by distance from the root base |
   | --- | --- |
   | Tap root | 4 from 0 to 20 cm, falling to 2 at 40 cm and beyond |
   | Basal roots | 0.7 to 20 cm, 0.4 from 40 cm |
   | Hypocotyl-borne roots | 0.6 at the base to 0.2 at 50 cm |
   | Laterals | 0.1 (fast laterals 0.3 to 0.2) |
   | Hypocotyl | 7, on a 0.4 cm base diameter |

   The rate table (0 to 0.003 cm/day by segment age, day 29) is a **diameter** increment:
   the file says it "adds 0.1 cm after 43 days", and the table integrates to 0.101 cm [I].
   A tap root near the collar then reaches about 0.9 + 4 × 1.0 ≈ 4.9 mm by day 43, which
   matches field data. Taper comes from the distance key. Already sent to Main Game.
3. **R2 `fine_root_share`: 0.1 to 0.4, not 0.5 to 0.8, when "fine" means second-order
   laterals.**
   - Miller et al. 2003 found second-order branching "negligible even at 43 DAP" in the
     field [F].
   - Soybean second- and third-order roots hold 39% and 34% of root length (Jin 2013) [F].
   - 0.5 to 0.8 holds only for a diameter definition. Define the metric by root order.
4. **R3 §4.5 midday leaf ψ: −0.4 to −1.1 MPa, not −0.4 to −0.8.** Irrigated field bean
   reaches about −1.0 MPa around 13:00 [F].
   **Predawn: within 0.2 MPa of soil ψ, not 0.1.** Well-watered controls read −0.1 to
   −0.3 MPa, and bean transpires at night [F].
5. **R2b `θ_stop` 0.05 is too harsh.** At about −1.6 MPa, maize and soybean primary roots
   keep full elongation in the apical 3 to 4 mm (Sharp 2004; Yamaguchi 2010) [F]. Use
   `θ_stop` ≈ 0 or a floor of about 0.3 on the factor. **Xerobranching:** keep the
   repression (Orman-Ligeza 2018 [F]) but drop the 7-day rewet window, which has no
   support (the barley effect looked irreversible).
6. **R1 `exudation_fraction`: 0.3 to 0.5, not 0.67.**
   - Legume exudates are about 2.7% of assimilated ¹⁴C (Warembourg 2003) [F].
   - Rhizodeposition is about 5 to 10% of assimilated C (Nguyen 2003) [F].
   - Bean root respiration is about 20% of daytime assimilation (Nielsen 2001) [F].
   - Together: exudation ≈ 0.1 to 0.5 of root respiration [I].
7. **R1 wheat `storage_capacity`: 0.25 to 0.30, not 0.35.** Peak WSC is 25 to 38% of dry
   mass 10 to 14 days after anthesis, under 5% at maturity (Davidson & Chevalier 1992)
   [F]. Genotype means at anthesis are 11 to 21% (Ruuska 2006) [F].

## R1 bands and parameters

| Item | Spec | Recommended | Basis |
| --- | --- | --- | --- |
| `stem_share_peak` | 0.35 to 0.55 | **0.35 to 0.60** | Stem + petiole 0.59 to 0.66 at R6 (UFGA8501) [F]; 0.43 to 0.48 in pots (Lopes 1983) [F]; petioles move about 0.07 to 0.10 [I] |
| `stem_decline_fill` | −10 to −35% | keep | UFGA8501 −33 to −36% including petioles [F]; stem sugars fall from about 7% to 1.6% DM (Peña-Valdivia 1991) [F]; stem starch −42% in 7 days of fill (Waters 1980) [F] |
| `basal_stem_diameter` | 5 to 10 mm | **4.5 to 10 mm** | Hypocotyl at maturity 4.3 to 5.4 mm in dense plots (Anjos 2018) [F]; basal stem 7.6 to 8.0 mm (Rodríguez Rodríguez 2026) [F]; a lone pot plant sits higher [I] |
| `tap_root_diameter` | 3 to 8 mm | **2 to 6 mm, Info** | Burridge 2016 measured it 2 cm below the hypocotyl at 35 to 45 DAP; their caption reads "up to 3 mm" [F]; OpenSimRoot gives about 5 mm by day 43 [I] |
| `h_L` | 3e-4 | keep (1.5 to 3e-4) | Basal stem area ÷ plant leaf area from the diameters above and UFGA8501 LAI [I] |
| `g_max` | 0.1 mm/day | keep as a cap | Bean needs 0.02 to 0.05 mm/day on average [I]; sunflower about 0.15 (Radu & Gurău 2023) [F] |
| bean `storage_capacity` | 0.25 | keep (0.15 to 0.30) | Greenhouse stem starch up to 30 to 68% (Waters 1980) [F], a ceiling |
| `θ_max` (phototropism) | 10°/day | 10° is conservative; 20 to 30° is defensible | Seedling bending 25° in about 2 h (sunflower, pea) [F]; no bean stem kinetics found |

## R2 bands and parameters

| Item | Spec | Recommended | Basis |
| --- | --- | --- | --- |
| `root_length_day21` | 20 to 120 m | **20 to 150 m, aim 40 to 90** | Miller 2003, 20 L sand pots: basal + adventitious 78 to 228 m (high P), 40 to 90 m (low P) [F]; Polania 2009 soil tubes, 32 days: 35 to 62 m [F, figure] |
| `root_length_flowering` | to set | **60 to 180 m** | Trindade & Araújo 2014, 4 kg pots: 93 to 158 m at pod setting, 105 to 180 m in early fill [F]; Araújo 2004, 45 days: 75 to 116 m [F]. Field excavation recovers only 9 to 21 m, so don't use it |
| `fine_root_share` | 0.5 to 0.8 | **0.1 to 0.4** (by order) | see change 3 |
| `split_pot_foraging` | ≥ 1.5 | keep; expect 2 to 4 | Wet-side concentration in other species; tomato root mass +19 to 55% under partial root drying [F]; no bean data |
| `nitrate_patch_proliferation` | ≥ 2 | **≥ 1.5**, expect 1.5 to 4 | Arabidopsis about 2× lateral elongation (Zhang 1999); 1.8 vs 0.8 cm mean lateral length (Linkohr 2002) [F]; Drew 1975 "several-fold" is [K]; no legume data |
| Lateral timing | in the first week | **first tap-root laterals 3 to 5 days after germination** | 154 cm of laterals at 7 days in pouches (Vieira 2008); lateral zones at 4 days (Arthikala 2018) [F] |
| Hydropatterning | keep | keep | About 80% of laterals start on the wet side (Bao 2014) [F, secondary] |
| Fine-lateral tissue density 0.094 g/cm³, about 1,350 m/g | keep, flagged | keep | Whole-plant SRL is 75 to 100 m/g in the field and 161 to 202 m/g in pots [F], which fits [I]. Miller's own diameters imply about 0.03 g/cm³, unresolved |

The day-21 gap is mostly **first-order laterals** on basal and adventitious axes, not fine
roots [I]. So R2a should tune lateral density, length and timing first.

## R3 parameters (filled-in §4.4)

| Name | Value | Unit | Source | Tag |
| --- | --- | --- | --- | --- |
| `Lp_r`, young and fine roots | 2e-7 (range 0.8 to 6e-7) | m s⁻¹ MPa⁻¹ | Fiscus & Markhart 1979, whole bean root systems; the CPlantBox default kr converts to 2.0e-7 | [F], [I] conversion |
| `Lp_r`, single young roots | 1.8 to 8.4e-8 | m s⁻¹ MPa⁻¹ | Steudle & Brinckmann 1989, *P. coccineus* | [F] |
| `Lp_r`, thickened | about 5e-8 | m s⁻¹ MPa⁻¹ | Pine analogy (suberized ÷ young ≈ 0.39) | [I] |
| `K_x`, basal root | about 1e-10 (wild *P. vulgaris* 8.9e-11; 800-fold range across *Phaseolus*) | m⁴ s⁻¹ MPa⁻¹ | Massas 2022 | [F], [I] units |
| `K_x`, tap root by position | apex 2e-11, middle 4e-10, base 2e-9 | m⁴ s⁻¹ MPa⁻¹ | Barrios-Gómez 2012 metaxylem 29 → 52 → 66 µm, Hagen–Poiseuille × 0.4 | [F] anatomy, [I] values |
| Stem conductance | k_h 2.9e-5 kg m s⁻¹ MPa⁻¹; leaf-specific 6.2e-5 | – | Holste 2006 | [F] |
| Stem `K_s`, xylem area fraction | none found; calibrate `K_s · A_x` to Holste's k_h | – | – | gap |
| `K_leaf` | about 8.7 (keep the 5 to 15 band) | mmol m⁻² s⁻¹ MPa⁻¹ | back-calculated from Haverroth 2025 | [I] |
| `K_leaf` vulnerability | P12 −0.72, P50 −1.00 | MPa | Haverroth 2025 | [F] |
| `ψ_tlp`, well watered | −0.9 (−0.6 to −1.3) | MPa | Zlatev & Stoyanov 2005; Haverroth −0.83 | [F] |
| `ψ_tlp`, drought-acclimated | −1.0 to −2.3 | MPa | Zlatev & Stoyanov 2005 | [F] |
| Osmotic potential at full turgor | −0.54 to −1.26 in pots; −1.12 in irrigated field | MPa | Zlatev & Stoyanov 2005; Buckley 2024 | [F] |
| Tuzet `s_f`, `ψ_f` | 4 (3 to 6), −1.0 | MPa⁻¹, MPa | Fit to bean closure starting at −0.4 to −0.6 MPa and about −1.2 MPa, plus K_leaf P50 and ψ_tlp | [F] anchors, [I] fit |
| van Genuchten | see change 1 | – | – | [I] |

Notes:
- **Tuzet form.** It matches the plantecophys code [F]. Its generic defaults (`s_f` 3.2,
  `ψ_f` −1.9) are too drought-tolerant for bean.
- **Couvreur 2012.** The form is confirmed [F]. It has its own compensation conductance
  `K_comp`, while the spec uses `K_rs` there. Keep `K_comp` as a parameter defaulting to
  `K_rs`: it is the natural knob for the fidelity report's hard cases.
- **Stem resistance** is about three orders of magnitude below leaf resistance, so R1a's
  thickening barely moves leaf ψ [I]. Don't spend tuning on `K_s`.
- **Root axial conductance** only limits flow near tips and in distal segments [I]. For
  M3, use a per-class genotype multiplier.
- **"Bean is isohydric" is not supported.** Of 12 common-bean genotypes in the field, 6
  were isohydric and 6 anisohydric (Buckley 2024) [F]. `s_f` and `ψ_f` can become heritable.
- **Recovery after rewatering.** For moderate stress, turgor above 0.5 within 6 h is
  confirmed: water content recovers in 2 to 3 h and stomata in 6 h (Kim & Lee-Stadelmann
  1984) [F]. After severe stress (−1.6 to −2.5 MPa), stomata need up to 8 days (Trujillo
  2013) [F]. The spec has no stomatal lag. Say so in the card, or consider an R6 item.

## Gaps

- **Roots:** no single bean root-length time course (Lynch & van Beem 1993 is the source
  to fetch). No bean field root length density profile (Sponchiado 1989 is paywalled). No
  bean data on nitrate patches, xerobranching or hydropatterning.
- **Hydraulics:** Strock 2021 per-class metaxylem and K_x (paywalled). Bean Lp_r by root
  age. Stem K_s and xylem area fraction. A directly measured bean K_leaf.
- **Stems:** petiole share of bean leaf mass (soybean about 20 to 30% is [K]). Hypocotyl
  diameter over the season. Field stem starch over time.
