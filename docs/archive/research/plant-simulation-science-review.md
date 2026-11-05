# Rebuild the bean's respiration, roots and phenology

The established plant-modelling literature mostly supports the design plan v3 architecture. It favours carbon as the authority, local reserves, beta expansion curves in thermal time, a turbid-medium voxel light model and a cascading soil bucket. Where the plan goes wrong, the problem is usually a parameter, and in a few places a missing organ class. Four findings matter most. First, the plan's **maintenance respiration of 0.015 g C per g N per day was about 15× too low**, apparently a mix-up with McCree's per-dry-mass coefficient. PR #24 has already changed it to **0.218 g C per g N per day at 20 °C**. Second, the plan's **Vcmax 89 and Jmax 160 for bean were fitted with a finite mesophyll conductance**, so pairing them with an infinite-gm Farquhar model overstates Rubisco-limited assimilation by about 20%. Third, the bean root system is dominated by **basal and hypocotyl-borne roots, and the plan models neither**. OpenSimRoot ships a complete, commented bean root parameter set we can use instead of guesses. Fourth, the **single-peak 8/28/38 °C development curve is out of line with every bean model checked**: CROPGRO, APSIM and the field fits all use a lower base of 5 to 7.5 °C and a flat optimum plateau. The best open bean parameter source is the DSSAT CROPGRO-Drybean species, ecotype and cultivar files, which are BSD-3 licensed and already downloaded. No open, per-organ weekly time series for bush bean was found. The DSSAT Drybean experiment files are the most likely source, but GitHub access was blocked this session, so they could not be listed.

**Citation tags used throughout.** **[F]** means the source was read this session (the page, file or abstract was fetched). **[K, to verify]** means a standard reference cited from prior knowledge whose text was not re-read this session; specific numbers with this tag should be checked before they are locked. **[I]** means the research team's inference or arithmetic, not a source claim. File references such as "SPE line 97" refer to `BNGRO048.SPE` in the DSSAT v4.8 bundle at `/mnt/project-files/research/source-files/dssat-bngro048/` (OpenSimRoot bean XML in `/mnt/project-files/research/source-files/opensimroot-bean/`; research notes in `/mnt/project-files/research/notes/`), upstream at [DSSAT/dssat-csm-os](https://github.com/DSSAT/dssat-csm-os).

## Prioritized changes, grouped by the work they touch

The lists below compare the plan text with the code as merged, which includes PR #24 (decision 0011), the species file `data/species/bush_bean.ron`, and decisions 0007 to 0009. Items PR #24 already adopted are marked **done** so that reviewers do not reopen them.

### Fix now: M0 code already written (WP7 physiology, WP5 soil, WP6 light)

| # | Plan § | Current value | Recommended value or method | Source | Confidence |
| --- | --- | --- | --- | --- | --- |
| 1 | 3.8, 3.16 | Maintenance 0.015 g C (g N)⁻¹ d⁻¹ at 20 °C | **Done in PR #24:** 0.218 g C (g N)⁻¹ d⁻¹ at 20 °C, Q10 2. Still open: (a) give storage organs a multiplier of about 0.35, because per-N maintenance overcharges seed storage protein about 3× relative to WOFOST RMO; (b) set organ N defaults from CROPGRO protein fractions ÷ 6.25 (leaf 3.8%, stem 2.4%, root 1.6%, shell 3.5%, seed 3.8%) | Ryan 1991 / CLM br_mr [K, to verify]; WOFOST RMO, CROPGRO SPE lines 23–25 [F]; multiplier [I] | High (value); medium (multiplier) |
| 2 | 3.8, 11.4 | Leaf Rd and step-8 maintenance could both charge leaves | **Done** (decision 0009 item 7, PR #24): leaves pay Rd only | [Photosynthesis notes, I] | High |
| 3 | 3.9 step 3, 11.4 | One explicit Jacobi pass, conductance capped at 0.5·min(W/degree) | **Done in PR #24:** implicit (backward-Euler) O(n) tree solve. Still open: keep a "common-pool" switch (infinite conductance) for calibration (item 22) | L-PEACH fold/unfold [F](https://www.algorithmicbotany.org/papers/lpeach.nphyto2005.pdf); stability analysis [I] | High |
| 4 | 3.9 steps 4–5 | Priority multiplied demand, so organs were capped below potential; unmet demand was booked every hour | **Done:** priority scales the share of available reserve an organ may spend (decision 0009 item 8), and unmet demand is booked only when spent < demand (PR #24). A Michaelis–Menten draw, demand·C/(Km+C), is the literature-standard alternative if tuning struggles | Minchin et al. 1993 [K, to verify](https://doi.org/10.1093/jxb/44.5.947); L-PEACH [F](https://www.algorithmicbotany.org/papers/lpeach.nphyto2005.pdf) | High |
| 5 | 3.16, 8.2 | Vcmax25 89.3, Jmax25 159.8 (Pimentel 2011, chloroplast (Cc) basis, gm 0.47) used with infinite gm | **Parked for WP12 (coordinator).** Either use Ci-basis apparent values (about Vcmax 75 and Jmax 150, the Wullschleger herbaceous means already in §8.2), or add gm 0.47 mol m⁻² s⁻¹ bar⁻¹ together with the Bernacchi 2002 Cc-basis constants. Do not mix the two bases | Sun et al. 2014 [F abstract](https://consensus.app/papers/details/32d24ccd148159708f67f27c03644359/?utm_source=claude_desktop); Bernacchi 2001/2002 [K, to verify](https://doi.org/10.1104/pp.008250); ~20% Ac overstatement [I] | Medium-high |
| 6 | 8.1, 11.4 | Species file labels Vcmax 58550/200000/629.26 and Jmax 29680/200000/631.88 as "plantecophys 1.4-6 defaults" | **Check before the R CI job runs.** The `Photosyn()` documentation fetched this session lists EaV 82620.87, EdVC 0 (no Vcmax deactivation), delsC 645.1013, EaJ 39676.89, EdVJ 2e5, delsJ 641.3615, theta 0.85, alpha 0.24 on *incident* PAR, g0 0. The fixture must pass our theta 0.7, alpha 0.3 on absorbed PAR (or 0.3/0.85 incident), g0 0.01 and EdVC explicitly. Otherwise "within 2%" compares two different models | [plantecophys Photosyn docs, F](https://rdrr.io/cran/plantecophys/man/Photosyn.html) | High that a discrepancy exists; which side is wrong is unverified |
| 7 | 3.11, 3.16, 6.1 | Development 8/28/38 °C, single peak; emergence 70, flowering 350, maturity 850 °C·d (base 8) | **Four-point trapezoid, Tb 7, To1 25, To2 33, Tmax 42 °C.** Recompute every target on base 7: emergence about 55, flowering about 370 °C·d from sowing, dry-seed maturity about 970. Keep heat damage as a separate reproductive process | CROPGRO SPE lines 97–99 [F](https://github.com/DSSAT/dssat-csm-data); Ferreira 1997 [F](https://consensus.app/papers/details/63549f78a71755b591f0df173f5004b8/?utm_source=claude_desktop); Dapaah 1999 [F](https://consensus.app/papers/details/348fda4e3d1352bf9b62b11537c631dc/?utm_source=claude_desktop); APSIM bean [F](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015); synthesis [I] | Medium |
| 8 | 3.2 step 5, 3.6 | One FTSW threshold, 0.4, for both conductance and expansion | **Two parameters: expansion 0.55, conductance 0.35**, both heritable | Sadras & Milroy 1996 [F](https://consensus.app/papers/details/560119c017565009bf2305e34afc99f2/?utm_source=claude_desktop); Pohlmann 2022 bean 0.17–0.36 [F](https://consensus.app/papers/details/11dae588fa535ebb8ddca34ccef94be9/?utm_source=claude_desktop); Soltani 2000 [F](https://consensus.app/papers/details/7d9e3a9a14415d4db0087e83df989f22/?utm_source=claude_desktop) | Medium |
| 9 | 3.8, 6.1 | One construction cost, 1.25 g C per g C (20%) | **Organ-specific costs (g C used per g C built): leaf 1.36, stem 1.31, root 1.24, seed 1.40**, from CROPGRO composition with nitrate-N protein costs. 1.25 is right only for low-protein tissue or ammonium-N protein costs | CROPGRO SPE lines 19–33 [F]; Penning de Vries 1974 [K, to verify](https://doi.org/10.1016/0022-5193(74)90119-2); arithmetic [I] | Medium |
| 10 | 3.16 | Medlyn g1 4 kPa^0.5 | **5.8** (CLM5 value for C3 crops, from Lin et al. 2015). Keep it heritable; at D = 1.5 kPa, g1 = 4 gives about 25% less transpiration | [Lin et al. 2015, F](https://www.k-state.edu/ecophyslab/pdf's/Lin%20et%20al.%202015%20NCC.pdf) (ranking only); CLM5 value [K, to verify](https://escomp.github.io/ctsm-docs/versions/master/html/tech_note/) | Medium-low |
| 11 | 3.6, 3.7 | Uptake coefficient 1e-5 m² h⁻¹ (RLD 1 cm cm⁻³ takes 10% of available water per hour) | Define a maximum per unit root length, about 0.03 cm³ cm⁻¹ d⁻¹ (CERES), and saturate in RLD: U ∝ RLD/(RLD + 0.5). The current value gives about 12× the CERES maximum at full available water | CERES/Ritchie [K, to verify]; CROPGRO RWUMX 0.04 cm³ cm⁻¹ d⁻¹, SPE line 74 [F]; comparison [I] | Medium-low |
| 12 | 3.5 | No scattering; 2-band CIE overcast sky | Keep for M0, but make sure the Python Monte Carlo fixture has the same no-scattering assumption, and write it into the model card. A single-order scattering correction adds about 10–20% to diffuse light deep in the canopy | RATP (Sinoquet 2001) [K, to verify](https://doi.org/10.1046/j.1365-3040.2001.00694.x); de Pury & Farquhar σ 0.15 [K, to verify](https://doi.org/10.1111/j.1365-3040.1997.00094.x); magnitude [I] | Medium |
| 13 | 3.6 | Clay FC/WP 0.40/0.22 | Verify against Saxton & Rawls 2006. One compilation gives 0.32/0.20; recall of S&R gives about 0.42/0.30, which would cut plant-available water from 0.18 to about 0.12 | [Minnesota Stormwater Manual compilation, F](https://stormwater.pca.state.mn.us/index.php/Soil_water_storage_properties); S&R table [K, to verify] | Low |

### WP8 architecture and roots (next)

| # | Plan § | Current value | Recommended value or method | Source | Confidence |
| --- | --- | --- | --- | --- | --- |
| 14 | 3.7, 6.1 | Root types: tap plus 1st- and 2nd-order laterals | **Add basal roots** (2 whorls × 4, appearing at 100 h and 150 h, insertion 90°, gravitropism graded by whorl) and **hypocotyl-borne roots** (up to 10, from day 10, near-horizontal, 1 cm d⁻¹, about 27 cm). Values in the WP8 section below | [OpenSimRoot Bean/Carioca, F](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml) | High |
| 15 | 3.7, 6.1 | Tap root 2.0 cm d⁻¹, max 60 cm, spacing 1.0 cm, apical zone 3 cm | **Tap root 2.36 cm d⁻¹ with no cap (≥100 cm in the field); lateral spacing 0.1 cm on the tap and 0.15 cm on basal roots; apical unbranched zone 10 cm (8 cm on whorl 1)** | [growthrates.xml, F](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml) | High |
| 16 | 3.7, 3.16 | One SRL, 70 m g⁻¹, labelled "fine roots" | **Carbon per cm = 0.094 g cm⁻³ × π(d/2)² × 0.45 per class**, with diameters 0.9 / 0.68 / 0.64 / 0.3 / 0.15 / 0.1 mm. Keep 65–90 m g⁻¹ only as a whole-system check | OpenSimRoot [F]; APSIM 65 m g⁻¹ [F](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015); CROPGRO RFAC1 9000 cm g⁻¹ [F] | High |
| 17 | 3.4, 3.16 | Phyllotaxis "alternate, ~137.5°, botany" | **Make the angle a species parameter and test 180° (distichous) as the default candidate.** No bean source supports 137.5°, which is the angle of *spiral* phyllotaxis. Settle it with Debouck & Hidalgo 1986 or photographs | [Kuhlemeier 2017, F](https://consensus.app/papers/details/955be0798265544e8fd44d203afd2b84/?utm_source=claude_desktop); bean divergence angle not found | Medium |
| 18 | 3.11, 6.1 | Pods expand 280 °C·d, then seeds fill 280 °C·d, in sequence | **Pod wall about 145 °C·d (8 PTD); seed fill about 270–400 °C·d (15–22 PTD), starting about 3–4.5 PTD after the pod begins, so the two overlap** | CROPGRO LNGSH, SFDUR, FL-SH, FL-SD [F](https://github.com/DSSAT/dssat-csm-data); PTD conversion [I] | Medium |
| 19 | 3.11 | Heat abortion when daytime >32 °C; drought abortion at FTSW <0.3 | **Continuous pod-set factor:** quadratic 7/22/25/36 °C on daily temperature (CROPGRO), or −6.5% seed set per °C of daily mean above 23 °C (Prasad), plus a night term (>20 °C) and a cold branch (nights <10 °C). Keep FTSW 0.3 | CROPGRO SPE line 81 [F]; [Prasad 2002, F](https://consensus.app/papers/details/9cefc6f976d15b199249d9bb8c963749/?utm_source=claude_desktop); [Rose 2023, F](https://consensus.app/papers/details/34c183781c585a078bb46d36e5481fdf/?utm_source=claude_desktop); [Dickson & Boettger 1984, F](https://consensus.app/papers/details/473d4575e93a534f86639cf4532d0bd7/?utm_source=claude_desktop) | Medium-high |
| 20 | 3.11 | Pod-set baseline implicit | **Calibrate so 45–80% of flowers and young pods abort** under normal conditions, i.e. per-flower set of about 0.2–0.55, with early, proximal flowers favoured | [Binnie & Clifford 1981, F](https://www.cambridge.org/core/journals/journal-of-agricultural-science/article/flower-and-pod-production-in-phaseolus-vulgaris/69D11A53ED2B8241E73FBCDA5BB4B8DC) | High |
| 21 | 3.16, 6.1 | Leaf area potential 75 cm² (bounds 30–150) | **Maximum trifoliate 133 cm² (bounds to 180)** at the best nodes. The realised plant mean of about 60 cm² should emerge | CROPGRO SIZLF [F](https://github.com/DSSAT/dssat-csm-data); Kumar 2025 (plan §8.2) | Medium-high |
| 22 | 3.9, daily step 2 | One 0.05 W floor and a 3-day window | **Graded thresholds:** low carbon first slows initiation and bud break (about 0.05 g C g⁻¹ DW), then aborts flowers (0.1 g C g⁻¹ DW for 2 d), then kills shoots (0.025 g C g⁻¹ DW for 5 d). Consider normalising by the plant-level supply/demand ratio (Q/D) | [Helios CarbohydrateParameters, F](https://plantsimulationlab.github.io/Helios/struct_carbohydrate_parameters.html) (tree-tuned defaults) | Medium |
| 23 | 3.3 | Parent pointers and axis order | Add an MTG-style edge type (successor or branch) and an axis id; keep dead organs as tombstones so ids stay stable | MTG [K](https://doi.org/10.1006/jtbi.1997.0561); [Helios docs, F](https://plantsimulationlab.github.io/PyHelios/PlantArchitectureDoc.html) | High (low cost) |
| 24 | 3.7 | No root thickening; no exudation | Add tap and basal secondary thickening (radial rate up to 0.003 cm d⁻¹ × class factor, 4 for the tap root) and a lumped rhizodeposition sink of about 5–15% of root carbon | OpenSimRoot growthrates.xml, exudates.xml [F]; magnitude [I] | Medium (thickening); low (exudation) |
| 25 | 3.4 | Shade: internode ×1.5, SLA ×1.3 | Keep. Bush bean responds through internode length only, not node number; 1.5–2× is plausible under canopy shade | [Hanyu 1996, F](https://consensus.app/papers/details/d735d28f599753f6a8ab762e4a239ef4/?utm_source=claude_desktop); [Nakata 1966, F](https://consensus.app/papers/details/f90f4423a6315d32a31123307f124770/?utm_source=claude_desktop) | High |
| 26 | 3.9 step 6 | 30% of structural C resorbed at death | About 5–10% of structural C, plus the whole remaining reserve, returned to the parent | [I]; no source value found | Low |

### WP11 validation harness and WP12 tuning

| # | Topic | Recommendation | Source | Confidence |
| --- | --- | --- | --- | --- |
| 27 | Calibration method | Calibrate sink strengths and expansion curves first in common-pool mode against organ masses, GreenLab-style, using several harvest dates. Then turn finite transport back on and tune only conductance and the starvation thresholds | [Yan et al. 2004, F](https://consensus.app/papers/details/b29026a2d78b595288c3c0d065ff83dd/?utm_source=claude_desktop); [Guo et al. 2006, F](https://consensus.app/papers/details/e4f7f8f8497f50d79a4622048baf6769/?utm_source=claude_desktop) | Medium-high |
| 28 | Partitioning target | Well-watered leaf share of vegetative growth 0.55–0.60 at V2–V6; roots about 0.39 at emergence, then about 0.10 from V4 (CROPGRO) or 0.20 from day 5 (OpenSimRoot) | CROPGRO SPE lines 49–51 [F]; APSIM [F](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015); OpenSimRoot [F] | Medium |
| 29 | Thermal-time targets | State the base temperature on every target. Pinzón-Sandoval's 800 °C·d has an unknown base; resolve it or demote it | [I] | High |
| 30 | Root checks | Tap root 33 cm in 14 d; basal plus hypocotyl-borne length 120–170 m at 21 d in sand culture (an upper bound); whole-system SRL 65–300 m g⁻¹ | OpenSimRoot comments citing Lynch & van Beem 1993 and Miller 2003 [F] | Medium |
| 31 | Plant-level leaf area | YVREF: 30, 60, 110, 280, 1000, 1800 cm² per plant at V 0, 1.83, 2.8, 4.3, 6.8, 9.6 | SPE lines 59–60 [F] | Medium |
| 32 | Benchmark accuracy | Mature crop models reach about 12–13% nRMSE on bean yield and dry mass. Do not expect better | [Amiri et al. 2024, F](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015) | High |
| 33 | Inspector | Expose the plant Q/D ratio; it predicts fruit set and SLA in GreenLab tomato | [Louarn 2007, F](https://consensus.app/papers/details/2f3fdfa5fa4d59e794f1425139a6863a/?utm_source=claude_desktop) | Medium |
| 34 | Frost | Leaf damage −2.2 °C, plant death −5 °C (CROPGRO) in place of −1 °C | SPE line 65 [F] | Medium |

### Later milestones

At M1 and M2, **paraheliotropism** is the most visible missing behaviour. Bean leaflets face the sun in the morning and evening and turn edge-on at midday as light, heat and drought rise; cos(incidence) falls linearly with PFD above about 25 µmol m⁻² s⁻¹ ([Fu & Ehleringer 1991, F](https://consensus.app/papers/details/20cffcd50dda59e899f727a7e5e53169/?utm_source=claude_desktop); [Pastenes 2004, F](https://consensus.app/papers/details/37e96cd579305d2fa0b9a637272da058/?utm_source=claude_desktop)). It can go in as an hourly per-leaf multiplier on direct PAR without rebuilding the voxel grid [I].

Plots (WP21) need four soil and light upgrades:
- Drainage as a per-hour fraction k_h = 1 − (1 − SWCON)^(1/24), with SWCON set by texture ([APSIM SoilWat, F](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/)).
- Ritchie two-stage evaporation: U ≈ 6 mm, CONA ≈ 3.5, KEP 0.65.
- Curve-number runoff (CN2 bare about 75).
- A finer sky, such as turtle-46 directions or 3–4 elevation bands, with a clear-sky distribution.

The red:far-red proxy should also gain neighbour-reflected far-red [K, to verify].

Nitrogen (WP24) can draw on CROPGRO's nodule parameters:
- FRCNOD 0.05 of new root growth goes to nodules.
- Fixation costs RFIXN 2.83 against RNO3C 2.556 g CH2O per g protein.
- Nodule temperature curves, SPE lines 40–46.

Price nodule cost from N fixed: about 7 g C per g N for the fixation process (CROPGRO RFIXN 2.83 g CH2O per g *protein* ÷ 0.16 g N per g protein; corrected 5 Nov 2026, an earlier version of this report said 2.8 g C per g N, which misread the protein basis; see the M2 and M3 follow-up report), or 3–5 g C per g N including nodule construction and maintenance ([OpenSimRoot carioca.xml comments citing Minchin & Witty 2005, F](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)). This is better than a flat 10–15% of assimilate.

Two more physiology items belong to M2. Severe drought should lower Vcmax as well as g1 ([Zhou 2013, K, to verify](https://doi.org/10.1016/j.agrformet.2013.05.009)). Temperature acclimation can follow Kattge & Knorr 2007 [K, to verify]. At M3, add photoperiod sensitivity PPSEN (default 0, range 0–0.07 h⁻¹) and CSDL 12.17 h as heritable loci; temperate Type I cultivars are day-neutral in CROPGRO [F].

On the rendering side:
- Crysis-style vertex-colour wind ([GPU Gems 3 ch. 16, F](https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch16.html)).
- Generalized-cylinder stems with parallel-transport frames.
- "GPU growth skinning", where per-internode buffers drive vertex positions.
- Zero-size births and render-only death ghosts, following the dL-system time-lapse principle ([Prusinkiewicz et al. 1993, F](https://algorithmicbotany.org/papers/animdev.sig93.html)).
- For the M4 tree, Palubicki's shadow propagation together with the Borchert-Honda λ, which supports the "no markers" decision ([Palubicki 2009, F](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf)).
- A green-pod harvest endpoint. Snap 'Provider' reaches harvest at about 934 °C·d on a 4.4 °C base ([OSU CROPTIME, F](https://instar.biossys.oregonstate.edu/CROPTIME/bean_varieties_models.pdf)).

## Four conflicting notes, reconciled

### Maintenance respiration: per nitrogen and per dry mass agree once units are fixed

The notes describe two bases. The **nitrogen basis** (Ryan 1991, used in CLM/ELM) charges 0.0106 mol C per mol N per hour at 20 °C, which is **0.218 g C (g N)⁻¹ d⁻¹**; CLM's br_mr of 2.525 × 10⁻⁶ g C (g N)⁻¹ s⁻¹ is the same number [K, to verify]([Ryan 1991](https://doi.org/10.2307/1941808)). The form and units are confirmed in the [E3SM ELM guide, F](https://docs.e3sm.org/E3SM/pr-preview/pr-7443/ELM/tech-guide/plant_respiration). The **dry-mass basis** (SUCROS/WOFOST) charges, per gram of dry matter per day at 25 °C with Q10 2:
- leaves 0.027–0.03 g CH2O
- stems 0.015
- roots 0.010
- storage organs 0.005–0.017

These come from the legume crop files and the `pcse` respiration code ([WOFOST_crop_parameters via pyswap, F](https://pypi.org/project/pyswap/); [pcse, F](https://pypi.org/project/pcse/)). CROPGRO-Drybean uses a third form: 3.5 × 10⁻⁴ h⁻¹ on whole-plant mass at 30 °C, plus a term proportional to gross photosynthesis (R30C2 0.004, SPE line 18) [F].

Converting puts all three in the same range [I]. WOFOST's leaf value is 0.03 × 0.40 g C per g CH2O = 0.012 g C g⁻¹ DM d⁻¹ at 25 °C. At a leaf N of 4.5% that is 0.27 g C (g N)⁻¹ d⁻¹, or about 0.19 at 20 °C. Stems and roots come out at about 0.25 per g N. The plan's leaf Rd of 0.9 µmol m⁻² s⁻¹ at about 2 g N m⁻² gives 0.47 at 25 °C, the same order. The plan's 0.015 is **14.5× below** Ryan and matches McCree's per-dry-mass c ≈ 0.015 d⁻¹ instead ([Amthor 2000 review, K, to verify](https://doi.org/10.1006/anbo.2000.1175)), which is almost certainly the origin of the slip. CROPGRO's mass term looks lower (about 0.0017 g C g⁻¹ d⁻¹ at 20 °C), but it spreads over low-N stems and seeds and has a separate photosynthesis-linked term.

**Recommendation: keep the nitrogen basis that PR #24 adopted** (0.218 at 20 °C, Q10 2). It is the one that links to the plan's planned trade-off ("higher Vcmax costs nitrogen and maintenance") and to M2 nitrogen. The per-N basis disagrees with WOFOST in one place: seeds. Ryan applied to a seed at 3.8% N gives about 0.008 g C g⁻¹ d⁻¹ at 20 °C, while WOFOST RMO gives about 0.003. A storage-organ multiplier of about 0.35 on seeds-in-pod closes that gap [I]. Q10 2 matches WOFOST and the plan; CLM5 uses 1.5 [K, to verify]. OpenSimRoot's bean roots respire 0.04 g g⁻¹ d⁻¹, or 0.09 for segments under 2 days old ([respiration.xml, F](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/respiration.xml)). That is roughly 10× the per-N maintenance of a 1.6% N root, but it is total respiration (growth, ion uptake and maintenance) measured on young segments, and the file's authors note an unresolved gap between segment-level and whole-system measurements. Do not use it as a maintenance coefficient.

### Growth respiration: 20% and 27–30% differ in how protein is costed

The photosynthesis note keeps 1.25 (20% of allocated carbon respired). The crop-model note derives 27–30% from CROPGRO's own composition and glucose costs (SPE lines 19–33). The difference comes from two assumptions [I]:
- **Protein cost.** CROPGRO charges 2.556 g glucose per g protein, which is the cost when the nitrogen comes from nitrate. The photosynthesis note used the ammonium-based Penning de Vries cost of about 1.6–1.8 ([Penning de Vries 1974, K, to verify](https://doi.org/10.1016/0022-5193(74)90119-2)).
- **Seed lipid.** CROPGRO's seed lipid is 3%, not 1.5%.

Recomputing with CROPGRO's tables gives the following, with the carbon-basis cost = g glucose × 0.40 / 0.45 [I]:

| Organ | g glucose per g dry matter | Cost (g C used per g C built) | Respired share |
| --- | --- | --- | --- |
| Leaf (protein 0.24) | 1.53 | 1.36 | 27% |
| Stem (protein 0.15) | 1.47 | 1.31 | 23% |
| Root (protein 0.10) | 1.39 | 1.24 | 19% |
| Seed (protein 0.235, lipid 0.03) | 1.57 | 1.40 | 28% |

WOFOST's generic CVL of 0.72 corresponds to 1.24, which is why 1.25 looks "standard". **Recommendation:** in M0 the plant gets its N from nitrate or fixation (RFIXN 2.83 costs even more), so use the organ-specific values above. Stay on the carbon basis that §6.1 already labels. When M2 adds nitrogen, move the N-reduction cost out of construction and into explicit uptake and fixation costs.

### Specific root length: 65 m g⁻¹ and 17 to 1,350 m g⁻¹ measure different things

APSIM's bean SRL of 65 m g⁻¹ ([Amiri 2024, F](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015)) and CROPGRO's RFAC1 of 9,000 cm g⁻¹ = 90 m g⁻¹ (SPE line 74, [F]) are **whole-root-system conversion factors** in models that have no root classes. OpenSimRoot gives per-class diameters and one tissue density, 0.094 g cm⁻³ ([growthrates.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml), [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml), [F]). SRL = 1/(ρπr²) then gives about 17 m g⁻¹ for the 0.9 mm tap root, 29–33 for basal and hypocotyl-borne roots, 150 for 0.3 mm laterals, and 600–1,350 for fine laterals [I]. The two are consistent: a length-weighted mix that thickens with age averages tens to low hundreds of m g⁻¹.

The plan is wrong only in calling 70 m g⁻¹ the SRL of "fine roots". **Recommendation:** since WP8 builds root classes, cost every segment from its own diameter and density. The code already carries `root_density` 1.0 × 10⁵ g m⁻³, close to 0.094 g cm⁻³. Use 65–90 m g⁻¹ only as a whole-system plausibility check.

### Cardinal temperatures: the plan's single peak is the outlier

| Source | Tb | Optimum | Tmax | Shape |
| --- | --- | --- | --- | --- |
| Plan §3.11 | 8 | 28 | 38 | Single peak |
| CROPGRO vegetative ([SPE line 97, F](https://github.com/DSSAT/dssat-csm-data)) | 5 | 27–37 | 45 | Plateau |
| CROPGRO early reproductive (line 98) | 5 | 24–34 | 44 | Plateau |
| CROPGRO seed fill (line 99) | 6 | 25–37 | 60 | Plateau |
| Ferreira 1997, green bean, hourly fit ([F](https://consensus.app/papers/details/63549f78a71755b591f0df173f5004b8/?utm_source=claude_desktop)) | 6.8 | 24.0 | 34.1 | Piecewise linear |
| Dapaah 1999, pinto, NZ ([F](https://consensus.app/papers/details/348fda4e3d1352bf9b62b11537c631dc/?utm_source=claude_desktop)) | 7 | linear in observed range | not reported | Linear |
| APSIM bean ([F](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015)) | 7.5 | 30 | 40 | Three-point |
| WOFOST soybean (warm legume proxy, [F](https://pypi.org/project/pyswap/)) | 7 | to 30 | 45 | Table |
| OSU CROPTIME snap bean ([F](https://instar.biossys.oregonstate.edu/CROPTIME/bean_varieties_models.pdf)) | 4.4 | n/a | 32.2 cut-off | Single sine |

Field fits put Tb at **6.8–7.5 °C**. CROPGRO's 5 °C sits inside a framework whose "thermal day" normalisation compensates for it. Sources disagree above about 30 °C: Ferreira's rate reaches zero at 34 °C, while CROPGRO keeps the full rate to 34–37 °C. Temperate summers rarely have daily means that high, so the choice there matters mainly for heat-wave scenarios. Under the plan's curve, development would halve at 33 °C, which no bean model supports.

**Recommendation [I]:** use a trapezoid of 7/25/33/42 °C with the §11.4 normalisation f(plateau) = To1 − Tb. Recompute emergence, flowering and maturity on base 7. Dapaah's 306 °C·d from emergence to flowering and 974 from sowing to harvest are already on base 7. Keep the 5/25–30/40 growth multiplier and the heat effects on pod set separate, as CROPGRO does.

## WP8 architecture parameters for bush bean

Published bean models give **main-stem leaf appearance, maximum leaf size, a per-node internode profile and root-class timing** in citable form. None gives **which nodes branch, branch phyllochron, or leaf size by node**. Those three need placeholders tagged [I] and tuned in WP12 against Kumar 2025 (5.5 branches, 20 leaves, 0.12 m² per plant). One correction to the notes: the `bean_data` note read ecotype column 9 as FL-VS = 7 and RHGHT = 0. Re-reading `BNGRO048.ECO` row MESDET shows **R7-R8 = 7.0, FL-VS = 0.00, RHGHT = 1.0** (ANDDET is the same). The `crop_models` note was right: in Type I, the main stem makes **no new leaves after first flower**.

### Leaf initiation (phyllochron)

| Axis | Value | Unit | Source |
| --- | --- | --- | --- |
| Main stem | TRIFL 0.34 leaves per thermal day (calibration range 0.32–0.36), for all bean ecotypes | leaves per thermal day | ECO column TRIFL [F](https://github.com/DSSAT/dssat-csm-os) |
| Main stem, converted | 1/0.34 × (27 − 5) ≈ **65 °C·d per node above 5 °C** (61–69). About **53 °C·d per node on the recommended base 7, plateau from 25 °C** | °C·d per node | [I] |
| Emergence to first trifoliate (V1) | EM-V1 3.0 thermal days ≈ 54 °C·d (base 7) | °C·d | ECO [F]; conversion [I] |
| Main stem stops | FL-VS 0 for Type I: the last main-stem leaf appears at first flower (11 PTD for indeterminate types) | PTD | ECO [F] |
| Leaf expansion after first flower (whole plant) | FL-LF 10 (Andean Habit 1), 16 (Meso Habit 1), 18 (Seafarer) PTD from R1 to end of leaf expansion. Since the main stem has stopped, this is branch leaf production | PTD | CUL [F] |
| Branches | **No source.** Placeholder: same thermal phyllochron as the main stem from bud activation (factor 1.0, tunable 1.0–1.3). Branches stop at their own terminal raceme or at FL-LF | °C·d per node | [I] |
| Snap bean check | 'Provider' first trifoliate at 608 °F·d (base 40 °F) ≈ 338 °C·d (base 4.4); first flower about 608 °C·d (base 4.4) | °C·d | [OSU CROPTIME, F](https://instar.biossys.oregonstate.edu/CROPTIME/bean_varieties_models.pdf) |

The main-stem count follows from these [I]. EM-FL is 22.6–28 PTD for Type I. With TRIFL 0.34, and early phases running at the vegetative rate, that gives roughly 6–9 trifoliate nodes before the terminal raceme, which matches the plan's 6–8.

### Leaf area by node

| Quantity | Value | Unit | Source |
| --- | --- | --- | --- |
| Maximum full trifoliate (three leaflets) | SIZLF 133 (generic Type I, Seafarer); range 133–180; 150 in Redkloud and Porrillo | cm² | CUL [F] |
| Reference trifoliate size | SIZREF 133 | cm² | SPE line 56 [F] |
| Leaflet dimensions | 4–16 cm long, 2.5–11 cm wide; the terminal leaflet has the longer stalk | cm | [Santanna & Burnham, U. Michigan, F](https://climbers.lsa.umich.edu/wp-content/uploads/2013/06/PhasvulgFABAFINAL.pdf) |
| Plant leaf area against main-stem V-stage | 30, 60, 110, 280, 1000, 1800 at V 0, 1.83, 2.8, 4.3, 6.8, 9.6. These are plant totals including branches, so use them as a check, not a per-node table | cm² per plant | SPE lines 59–60 [F] |
| SLA, new leaves | SLAVR 305–315 (Type I); range 250–350; SLAMIN 255; SLA ×0.30 at ≤10 °C rising to 1.0 at ≥22 °C | cm² g⁻¹ | CUL; SPE lines 56–62 [F] |
| Per-node profile | **No source.** Placeholder: unifoliate pair about 15–25 cm² each; trifoliates rising from about 50% of SIZLF at node 1 to 100% at nodes 3–5, then falling to about 70% at the last node. Branch leaves at about 80% of the main-stem value for the same rank | cm² | [I], constrained by YVREF increments of about 110 cm² per node between V2.8 and V4.3 |

### Internode length by node

CROPGRO's **YVSHT** table gives internode length per new node, by main-stem V-stage (SPE lines 120–122) [F]:

| V-stage | 0.00 | 2.67 | 4.25 | 6.75 | 8.00 | 9.58 | 10.0 | 16.0 | 20.0 | 40.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Internode length (m) | 0.0442 | 0.0260 | 0.0437 | 0.100 | 0.100 | 0.110 | 0.120 | 0.120 | 0.090 | 0.025 |
| Canopy width per node, YVSWH (m) | 0.025 | 0.025 | 0.075 | 0.075 | 0.070 | 0.060 | 0.048 | 0 | 0 | 0 |

The table is shared by all habits and scaled per ecotype (RHGHT 1.0 for Type I). Its modifiers: expansion falls to 0.55 below 15 °C and is full at ≥20 °C; **height and width expand up to 4× in darkness and 1.5× at a PAR of 7.5** (XHWPAR/YHWPAR), CROPGRO's own shade-elongation response (SPE lines 124–127) [F]. For a 6–8 node Type I plant, the table predicts a sum of roughly 45–55 cm, above the 30–50 cm plan target and Kumar's 35 cm [I]. **Placeholder [I]:** use the YVSHT shape (short epicotyl about 4 cm, shortest internodes at nodes 2–3 at about 2.6 cm, rising to about 10 cm by node 6–7), scaled by about 0.7 so a 7-node main stem reaches about 35 cm. The plan's species default of 4.5 cm potential is about right as a mean but loses the shape. Branch internodes have no source; a placeholder of 0.6–0.8 × main stem at the same rank is reasonable.

### Branching pattern of a Type I bush bean

The CIAT definitions give the frame. **Type I** has a reproductive terminal bud, an erect or prostrate stem, no climbing guide, and pods spread along the stem ([UPRM Bean Improvement Center, F](https://www.bic.uprm.edu/wp-content/uploads/2018/04/GrowthHabit.pdf)). Inflorescences are axillary racemes of **3–8 flowers**, rarely longer than the subtending petiole ([U. Michigan, F](https://climbers.lsa.umich.edu/wp-content/uploads/2013/06/PhasvulgFABAFINAL.pdf)). The plan's §8.2 target is **5.5 branches per plant** (Kumar 2025). No note supplies which nodes branch or how many second-order branches form; CROPGRO is a canopy model and does not represent branches. Helios ships a `bean` architecture model ([PyHelios docs, F](https://plantsimulationlab.github.io/PyHelios/PlantArchitectureDoc.html)), but its parameters were not read, so it is the first place to look in WP8.

**Placeholder [I]:**
- Branches come from the cotyledon and unifoliate axils and the lowest 2–4 trifoliate nodes.
- These are first-order branches. The upper 2–3 main-stem axils and the terminal bud make racemes.
- Each branch makes 2–5 trifoliates before its own terminal raceme.
- Second-order branches are rare (0–1 per plant) and appear only in wide spacing.

Bud activation should emerge from the plan's light × reserve × apical-dominance rule. The axil assignment and the 5.5-branch target are the WP12 tuning anchors.

### Root initiation timing

| Root class | Initiation | Spacing and count | Growth | Source |
| --- | --- | --- | --- | --- |
| Hypocotyl | Grows 3 cm d⁻¹ over days 1–2, ending by day 5; diameter 4 mm | 1 | Ends by day 5 | [growthrates.xml, F](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/growthrates.xml) |
| Tap (primary) | At germination | 1 | 2.357 cm d⁻¹ constant (33 cm in 14 d); diameter 0.9 mm; no maximum | growthrates.xml [F] |
| Basal whorl 1 | 4.167 d (100 h) | 4 roots, insertion 90°, gravitropism 0.002 | 0.1 cm d⁻¹ rising to 4 cm d⁻¹ from day 3 to 10, 2.35 cm d⁻¹ at days 25–35, stopping by day 40; diameter 0.68 mm; about 105–110 cm long | [carioca.xml, F](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml); [angles.xml, F](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/angles.xml); length [I] |
| Basal whorl 2 | 6.25 d (150 h), 0.5 cm further along the hypocotyl | 4 roots, gravitropism 0.001. The original SimRoot file had 8 and 6 per whorl; field plants have 4–16 basal roots | As whorl 1 | carioca.xml, angles.xml [F] |
| Hypocotyl-borne (adventitious) | From day 10 | Every 0.4 cm, at most 10, below ground only; insertion 85°, no gravitropism | 1 cm d⁻¹, 0.8 at day 25, 0 by day 35 (about 27 cm); diameter 0.64 mm | carioca.xml, growthrates.xml [F] |
| 1st-order laterals on the tap | Once the parent tip is more than 10 cm past the site, about 4 d after tap emergence [I] | Every 0.1 cm (10 per cm) plus "fast" laterals every 1–5 cm; insertion 75° | 0.768 cm d⁻¹ for 6 d (normal, about 6 cm) or to day 15 (fast, ×1.6, 20–25 cm); diameter 0.3 mm. Walk 2006: tap laterals grow 288 h | growthrates.xml [F] |
| 1st-order laterals on basal and hypocotyl-borne roots | Apical zone 8 cm (whorl 1) or 10 cm | 0.15 cm on basal roots (about 7 per cm), 0.4 cm on hypocotyl-borne | 0.768 cm d⁻¹ to day 4, ×0.8, about 4–5 cm. Walk 2006: basal laterals grow 180 h | carioca.xml, growthrates.xml [F] |
| 2nd-order (fine) laterals | Apical zone 4 cm | Every 0.5 cm (0.55 cm on fast laterals) | 0.2 cm d⁻¹ for 3 d, about 0.5 cm, diameter 0.1 mm; fast fine laterals about 2 cm at 0.15 mm. Walk 2006: 72 h | carioca.xml, growthrates.xml [F] |

The OpenSimRoot times are plant age from the start of the simulation. CPlantBox's soybean set uses firstB 3 d, delayB 7 d, maxB 15 for comparison ([Glycine_max_Moraes2020.xml, F](https://raw.githubusercontent.com/Plant-Root-Soil-Interactions-Modelling/CPlantBox/master/modelparameter/structural/rootsystem/Glycine_max_Moraes2020.xml)). No Phaseolus file exists in CPlantBox, which closes §8.5's open item. Neither model includes root death or a bean temperature response for elongation; reuse the soil-temperature growth multiplier [I]. In the default 20 cm pot the tap root reaches the bottom about 8–9 days after germination, and basal roots reach the wall within 3–5 days of their own growth. The pot geometry will therefore shape the M1 root picture, and the model card should say so [I].

## Consolidated bush-bean parameter table

| Parameter | Plan or code value | Recommended | Units | Source (tag) |
| --- | --- | --- | --- | --- |
| Maintenance, non-leaf | 0.015 → 0.218 (PR #24) | 0.218 at 20 °C, Q10 2; seeds ×0.35 | g C (g N)⁻¹ d⁻¹ | Ryan/CLM [K]; WOFOST [F]; multiplier [I] |
| Organ N concentration | not set | leaf 3.8, stem 2.4, root 1.6, shell 3.5, seed 3.8 | % of DM | CROPGRO SPE lines 23–25 [F] ÷ 6.25 [I] |
| Construction cost | 1.25 | leaf 1.36, stem 1.31, root 1.24, seed 1.40 | g C per g C | CROPGRO composition [F]; arithmetic [I] |
| Seed composition | not set | protein 23.5, lipid 3.0, carbohydrate 64 | % | CUL SDPRO/SDLIP, SPE [F] |
| Development Tb/To1/To2/Tmax | 8/28/–/38 | 7/25/33/42 | °C | CROPGRO, Ferreira, Dapaah, APSIM [F]; synthesis [I] |
| Emergence | 70 (base 8) | about 55 (base 7); CROPGRO 2.4–3.4 TD | °C·d | ECO PL-EM [F]; [I] |
| Sowing to flowering | 350 (base 8) | about 370 (base 7) | °C·d | Dapaah [F] |
| Sowing to dry maturity | 850 (base 8) | about 970 (base 7); Seafarer about 1,160 (base 5) | °C·d | Dapaah [F]; CUL [F], [I] |
| Snap harvest ('Provider') | none | 934 (base 4.4); 60.5 d | °C·d | OSU CROPTIME [F] |
| Phyllochron, main stem | 50–70 | 65 (base 5) ≈ 53 (base 7) | °C·d per node | TRIFL [F]; [I] |
| Photoperiod | none | PPSEN 0 (range 0–0.07 h⁻¹), CSDL 12.17 h | — | CUL [F] |
| Vcmax25 / Jmax25 | 89.3 / 159.8 (Cc basis) | Ci basis about 75 / 150, or add gm 0.47 (WP12) | µmol m⁻² s⁻¹ | Pimentel, Wullschleger (plan §8.2); Sun 2014 [F] |
| Rd25, Q10 | 0.88, 1.92 | keep | µmol m⁻² s⁻¹ | Pimentel; plantecophys [F] |
| Kc, Ko, Γ* at 25 °C (Ci basis) | Bernacchi 2001 | 404.9 µmol mol⁻¹, 278.4 mmol mol⁻¹, 42.75 µmol mol⁻¹ | — | Bernacchi 2001 [K]; rpmodel [F](https://search.r-project.org/CRAN/refmans/rpmodel/html/calc_kmm.html) |
| θ, α | 0.7, 0.3 (absorbed) | keep; pass them to the fixture explicitly | — | plantecophys defaults 0.85, 0.24 incident [F] |
| Medlyn g1 | 4.0 | 5.8 | kPa^0.5 | CLM5/Lin 2015 [K] |
| FTSW, conductance / expansion | 0.4 / 0.4 | 0.35 / 0.55 | — | Sadras & Milroy, Pohlmann [F] |
| FTSW, pod abortion | 0.3 | keep | — | CROPGRO SWFAC <0.5 [F]; [I] |
| Pod-set temperature | Tday >32 °C | quadratic 7/22/25/36 °C; −6.5% per °C above 23 °C daily mean | °C | CROPGRO SPE line 81, Prasad [F] |
| Per-flower pod set, baseline | implicit | 0.2–0.55 | — | Binnie & Clifford [F]; [I] |
| Flowers per raceme | 6 | 3–8 | — | U. Michigan compilation [F] |
| Pod wall growth / seed fill | 280 / 280 in sequence | about 145 / 270–400, overlapping | °C·d | LNGSH 8, SFDUR 15–22 PTD [F]; [I] |
| Seed mass | 0.3 (0.15–0.8) | 0.22–0.25 Mesoamerican, 0.55–0.60 Andean | g | CUL WTPSD [F]; Lei 2020 [F](https://www.frontiersin.org/journals/genetics/articles/10.3389/fgene.2020.00698/full) |
| Seeds per pod | 5 potential, 3.7 realised | 3–5.5 dry; up to 8 ovules in snap types | — | CUL SDPDV [F] |
| Maximum trifoliate area | 75 (30–150) | 133 (133–180) | cm² | CUL SIZLF [F] |
| SLA | 250 (150–400) | 305–315 for new leaves; ×0.3 at ≤10 °C | cm² g⁻¹ | CUL, SPE [F] |
| Internode per node | 4.5 mean | YVSHT profile 2.6–12 cm, scaled about 0.7 for Type I | cm | SPE lines 120–122 [F]; [I] |
| Leaf frost | −1 | −2.2 (leaf), −5 (plant) | °C | SPE line 65 [F] |
| Shade internode multiplier | 1.5 | 1.5–2 cap; no node-number change | — | Hanyu, Nakata [F] |
| Phyllotaxis | 137.5° | species parameter; test 180° | ° | unverified [I] |
| Tap root elongation, max | 2.0, 60 cm | 2.357, no cap | cm d⁻¹, cm | OpenSimRoot [F] |
| Basal roots | none | 2 whorls × 4 at 100 h and 150 h; 4 cm d⁻¹ peak | — | OpenSimRoot [F] |
| Hypocotyl-borne roots | none | ≤10 from day 10; 1 cm d⁻¹ | — | OpenSimRoot [F] |
| Lateral spacing / apical zone | 1.0 / 3 cm | 0.1–0.15 / 8–10 cm | cm | OpenSimRoot [F] |
| Root tissue density | 0.1 | 0.094 | g cm⁻³ | OpenSimRoot [F] |
| Whole-system SRL | 70 (used for cost) | check only: 65–90 | m g⁻¹ | APSIM, CROPGRO RFAC1 [F] |
| Root water uptake maximum | 0.1 h⁻¹ per (cm cm⁻³) | about 0.03–0.04 | cm³ water per cm root per day | CERES [K]; CROPGRO RWUMX [F] |
| Seed reserve release | 70% mobilizable | 40% d⁻¹ release over 25 d (OpenSimRoot) | — | carioca.xml [F] |
| Starvation thresholds | 0.05 W, 3 d | graded: 0.05 slow, 0.1 abort flowers (2 d), 0.025 kill shoot (5 d) | g C g⁻¹ DW | Helios [F] (tree defaults) |
| Soil evaporation | 0.5 × radiation / λ | Ritchie U 6 mm, CONA 3.5, KEP 0.65 (M2) | — | APSIM [F]; SPE line 131 [F] |
| Drainage | all excess, one cell per hour | k_h = 1 − (1 − SWCON)^(1/24) (M2) | h⁻¹ | APSIM [F]; [I] |
| Nodule cost (M2) | 10–15% of assimilate | about 2.8–5 per g N fixed; FRCNOD 0.05 | g C per g N | CROPGRO, Minchin & Witty via OpenSimRoot [F] |

## Validation data sets

| Name | Variables | Conditions | Access / URL | Licence status |
| --- | --- | --- | --- | --- |
| DSSAT CROPGRO-Drybean BNGRO048 SPE/CUL/ECO | Phenology, phyllochron, SIZLF, SLA, partitioning by V-stage, internode per node, seed traits, cardinal temperatures | Generic Type I–IV and calibrated cultivars (Seafarer, Redkloud, Carioca …) | [dssat-csm-os](https://github.com/DSSAT/dssat-csm-os) Data/Genotype; local copy as above; [DSSATTools on PyPI](https://pypi.org/project/DSSATTools/) | dssat-csm-os BSD-3 [F]([ecosyste.ms](https://ost.ecosyste.ms/projects/1068)); DSSATTools wrapper GPL-3; dssat-csm-data repo licence unconfirmed |
| DSSAT Drybean experiment files (*.BNX/BNT/BNA) | LAI, organ mass and V-stage time series | Multi-site field trials | [dssat-csm-data](https://github.com/DSSAT/dssat-csm-data) (not listed; GitHub blocked) | Unconfirmed; **the best candidate for the missing per-organ time series** |
| OpenSimRoot Bean/Carioca | Root class rates, diameters, angles, branching, respiration, exudation, hairs, seed reserve | Carioca (Mesoamerican); hydroponics, sand and field comments | [GitLab](https://gitlab.com/rootmodels/OpenSimRoot/-/tree/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca) | Repository GPLv3: use values as facts; do not copy files verbatim |
| OSU CROPTIME snap bean | Degree-days to first trifoliate, first flower, harvest | 'Provider', '5630', 'Sahara'; Willamette Valley 2013–15 | [PDF](https://instar.biossys.oregonstate.edu/CROPTIME/bean_varieties_models.pdf) | Not stated |
| CIAT Keller et al. genomic prediction | Days to flowering and maturity, 100-seed weight, yield, weather, SNPs | 481 Andean lines; 12 trials; irrigated, drought, low P | [doi:10.7910/DVN/XCD67U](https://doi.org/10.7910/DVN/XCD67U) | Not confirmed (CIAT Dataverse is usually CC0 or CC-BY, unverified) |
| Dapaah et al. 1999 | Thermal times E–F, F–pod, pod–maturity (base 7) | Pinto, Canterbury NZ | [paper](https://consensus.app/papers/details/348fda4e3d1352bf9b62b11537c631dc/?utm_source=claude_desktop) | Published paper (cite values) |
| Ferreira et al. 1997 | Cardinal temperatures, first-flower date | Green bean 'Alcade', Portugal, hourly | [paper](https://consensus.app/papers/details/63549f78a71755b591f0df173f5004b8/?utm_source=claude_desktop) | Paper |
| Prasad et al. 2002 | Seed set, seeds per pod, seed size against temperature | Kidney 'Montcalm', 28/18 to 40/30 °C, ±CO2 | [paper](https://consensus.app/papers/details/9cefc6f976d15b199249d9bb8c963749/?utm_source=claude_desktop) | Paper |
| Binnie & Clifford 1981 | Flower and pod abscission 45–80% | Seven cultivars | [paper](https://www.cambridge.org/core/journals/journal-of-agricultural-science/article/flower-and-pod-production-in-phaseolus-vulgaris/69D11A53ED2B8241E73FBCDA5BB4B8DC) | Paper |
| Hanyu et al. 1996; Nakata 1966 | Internode response to far-red; node number unchanged in bush type | Bush 'Celina' against pole; pinto seedlings | [Hanyu](https://consensus.app/papers/details/d735d28f599753f6a8ab762e4a239ef4/?utm_source=claude_desktop), [Nakata](https://consensus.app/papers/details/f90f4423a6315d32a31123307f124770/?utm_source=claude_desktop) | Papers (the WP12 shade number) |
| Pohlmann 2022; Cordoba-Novoa 2024 | Bean FTSW transpiration thresholds and their genetic variation | Two cultivars, two seasons; Mesoamerican panel | [Pohlmann](https://consensus.app/papers/details/11dae588fa535ebb8ddca34ccef94be9/?utm_source=claude_desktop), [Cordoba-Novoa](https://consensus.app/papers/details/654eb84bfa13575e8b93a921941107d3/?utm_source=claude_desktop) | Papers |
| Thorpe, Lacointe & Minchin 2011 | ¹¹C phloem transport in pruned dwarf bean | 2-source, 3-sink manipulation | [paper](https://connectsci.au/fp/article/38/2/127/54249/Modelling-phloem-transport-within-a-pruned-dwarf) | Paper (qualitative transport check) |
| Amiri et al. 2024 (APSIM bean) | Yield, dry mass, soil water nRMSE 12–14% | Iran field | [paper](https://www.maxapress.com/article/doi/10.48130/tia-0024-0015) | Open-access paper |
| Fu & Ehleringer 1991 | cos(incidence) against PFD | P. vulgaris, outdoor | [paper](https://consensus.app/papers/details/20cffcd50dda59e899f727a7e5e53169/?utm_source=claude_desktop) | Paper; the slope must be digitized |
| Burridge 2016 / Strock 2019 | Basal whorl number, basal and hypocotyl root number, angles | Field shovelomics; 577 genotypes | [Burridge](https://consensus.app/papers/details/ea8c70e7d0555e3485ab7c5b68ed3446/?utm_source=claude_desktop), [Strock](https://consensus.app/papers/details/dd110c5b35865542aeacbde81af76c6e/?utm_source=claude_desktop) | Papers |
| WOFOST crop parameters | Respiration, conversion efficiency, partitioning (soybean, mungbean, faba bean; no Phaseolus) | — | [ajwdewit/WOFOST_crop_parameters](https://github.com/ajwdewit/WOFOST_crop_parameters) | EUPL [F] |
| Helios plant library and carbohydrate model | Bean, cowpea and soybean architecture; carbon thresholds | — | [Helios](https://www.github.com/PlantSimulationLab/Helios) | LGPL-2.1+: read for values; do not copy code |
| Plan's existing set | Pinzón-Sandoval 2024, Kumar 2025, Cavalcante 2020, Sharda 2026 and others | §8.2–8.3 | Plan §8 | Papers; digitizing still needed |

## Where the plan departs from established practice, and which departures are justified

The plan's most important departure is **per-organ carbon reserves**, and the literature supports it as long as it is calibrated with care. GreenLab shares one common biomass pool among organs in proportion to sink strength × a beta-law age function. Its parameters can be identified from destructive organ-mass harvests. However, it cannot let a shaded branch starve on its own ([Yan et al. 2004, F](https://consensus.app/papers/details/b29026a2d78b595288c3c0d065ff83dd/?utm_source=claude_desktop)). L-PEACH and PiafMunch make carbon local but solve transport implicitly, by Newton iteration or a stiff ODE solver. PiafMunch shows sink priority *emerging* from Münch flow, and has been tested on a pruned dwarf bean ([Lacointe & Minchin 2008, F](https://publish.csiro.au/FP/FP08085); [Thorpe et al. 2011, F](https://connectsci.au/fp/article/38/2/127/54249/Modelling-phloem-transport-within-a-pruned-dwarf)). Helios is the closest analogue to our plan: an open research framework that includes a bean library and a threshold-based carbohydrate model, with pruning driven by starvation ([PyHelios, F](https://plantsimulationlab.github.io/PyHelios/PlantArchitectureDoc.html)).

The plan sits between these approaches. PR #24's implicit tree solve removes its main numerical weakness. Before that change, the stability cap meant carbon advanced one edge per hour, so a leaf-to-root path of 10–20 edges lagged by 10–20 hours [I]. What is left is a calibration risk: organ-mass data constrain conductances and thresholds only weakly. The answer is the common-pool calibration mode (item 27), not a retreat to three pools.

Other departures are not justified:
- **Development temperatures:** the single peak with a high base (reconciled above).
- **Missing root classes:** basal and hypocotyl-borne roots are the defining bean root classes, and their growth angle is the classic heritable trait separating phosphorus foraging from deep water access ([Strock 2019, F](https://consensus.app/papers/details/dd110c5b35865542aeacbde81af76c6e/?utm_source=claude_desktop); [Burridge 2016, F](https://consensus.app/papers/details/ea8c70e7d0555e3485ab7c5b68ed3446/?utm_source=claude_desktop)). That makes them natural loci for the M3 breeding game.
- **One FTSW threshold:** expansion responds earlier than gas exchange in most dicots. The review averages are 0.56 against 0.40, and the expansion figure for dicots alone is 0.66 ([Sadras & Milroy 1996, F](https://consensus.app/papers/details/560119c017565009bf2305e34afc99f2/?utm_source=claude_desktop)).
- **The 137.5° phyllotaxis** labelled as botany.
- **A hard 32 °C daytime heat threshold:** night temperature drives bean heat damage at least as strongly as daytime maxima ([Vargas 2021, F](https://consensus.app/papers/details/5a8ff961203c5fa8bd8965754a310aa4/?utm_source=claude_desktop)), and damage begins at 31/24 °C ([Rose 2023, F](https://consensus.app/papers/details/34c183781c585a078bb46d36e5481fdf/?utm_source=claude_desktop)).

Several departures are defensible simplifications, provided the model card records them:
- An hourly cascading bucket in place of Richards' equation. APSIM and DSSAT run the same family daily ([APSIM SoilWat, F](https://www.apsim.info/documentation/model-documentation/soil-modules-documentation/soilwat/)).
- Leaf temperature equal to air temperature in M0. Bean leaves actually stay below air at 33–42 °C ([Raeini-Sarjaz 2008, F](https://consensus.app/papers/details/6bc5f519510c5a2cb5694022d12e4d24/?utm_source=claude_desktop)).
- A static spherical G = 0.5.
- No scattering.
- The red:far-red proxy 1.2·exp(−0.5·LAI), which gives plausible values of 0.73 at LAI 1 and 0.27 at LAI 3 [I].

The code has already resolved several plan defects:
- Light-class interpolation instead of snapping to a class (decision 0009 item 6). Snapping would cause up to ±29% error at low light [I].
- The priority cap and unmet-demand bookkeeping (items 2–4 above).

On the graphics side, avoiding space-colonization markers is supported directly by Palubicki 2009. Shadow propagation is about 3× faster than markers and handles shedding better ([PDF, F](https://www.algorithmicbotany.org/papers/selforg.sig2009.small.pdf)).

## Conclusion

The research changes where the risk in this project sits. Allocation, which the plan's risk register treated as the main danger, now has an established numerical method and a calibration path. The remaining realism risk is mostly **unit discipline and convention matching**: one parameter was off by 15× through a basis mix-up, Vcmax was taken from one fitting convention and used in another, and the plantecophys oracle can only validate a model whose constants and light basis match it exactly. The cheapest protection is to make the species-file loader record the *basis* of each value as well as its unit (Ci or Cc, absorbed or incident PAR, nitrate or ammonium protein cost, the base temperature of every °C·d target). CI could then reject a parameter that arrives with no stated basis.

The second implication concerns WP8. Bean roots are not a generic tap-root system. The basal whorls and hypocotyl-borne roots are where bean-specific behaviour and heritable variation live, and OpenSimRoot already supplies their parameters. Building them into WP8 now costs less than retrofitting them before M3, when root angle becomes a breeding trait. The largest data gap left is the per-organ time series. Getting GitHub access to `DSSAT/dssat-csm-data` to list the Drybean experiment files is the most valuable single step towards a real WP12 tuning target.
