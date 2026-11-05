# Soil nitrogen, plant N and biological N fixation for bush bean (WP24)

Tags: [F] = read this session (source file, code or abstract); [K, to verify] = prior knowledge, not re-read this session; [I] = my inference or arithmetic.
Sources read in full this session: CROPGRO-Drybean species file BNGRO048.SPE (local copy), DSSAT CROPGRO Fortran source `NFIX.for`, `CROPGRO.for` and `NUPTAK.for` (GitHub, develop branch), OpenSimRoot bean `carioca.xml` and `nitrate.xml` (GitLab), APSIM Next Gen `Models/Resources/Nutrient.json` (GitHub; only partly readable, see Gaps). Paper evidence comes from abstracts (Consensus). Abstracts are marked [F]. A number that would need the full paper is tagged [K].

Source URLs used throughout:
- BNGRO048.SPE: local `/mnt/project-files/research/source-files/dssat-bngro048/BNGRO048.SPE` (DSSAT CSM 4.8, CROPGRO-Drybean)
- NFIX.for: https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Plant/CROPGRO/NFIX.for
- CROPGRO.for: https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Plant/CROPGRO/CROPGRO.for
- NUPTAK.for: https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Plant/CROPGRO/NUPTAK.for
- OpenSimRoot carioca.xml: https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml
- OpenSimRoot nitrate.xml: https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/nitrate.xml
- APSIM Nutrient.json: https://raw.githubusercontent.com/APSIMInitiative/ApsimX/master/Models/Resources/Nutrient.json

## Soil N: structure of established models and a minimal model for the game

### Takeaway
APSIM and CERES-type models all have the same layout: per layer, fresh organic matter (FOM) in 2-3 fractions, a microbial (BIOM) pool, a humus (HUM) pool, an inert pool, and mineral NH4 and NO3. Each organic pool decomposes first-order, with temperature and water factors. N is released or immobilised from the C:N difference, and NO3 moves with water. For the game, a three-pool model per cell is enough: litter/FOM, active humus, and mineral N split into NH4 and NO3, with first-order nitrification. Leaching of NO3 should use the cell's drainage flux. Denitrification can be left out at first, or kept as a waterlogging-only sink.

### Cited Findings
- APSIM Next Gen Nutrient pools [F]: Inert, FOMLignin, FOMCellulose, FOMCarbohydrate, Microbial, Humic. Initial FOM carbon is split 0.1 lignin, 0.7 cellulose and 0.2 carbohydrate, with FOM C = 0.4 × FOM mass. FOM decomposition goes 0.9 to Microbial and 0.1 to Humic. Microbial decomposition goes 0.6 to Microbial and 0.4 to Humic, and Humic decomposes into Microbial. Microbial C:N is 8, and Humic and Inert C:N equal the soil C:N (`Soil.Organic.CNR`). CO2 efficiency is 0.4, so 60% of decomposed C is respired. The microbial potential decomposition rate is 0.0081 /day (an alternative branch uses 0.0324 /day). — [APSIM Nutrient.json](https://raw.githubusercontent.com/APSIMInitiative/ApsimX/master/Models/Resources/Nutrient.json)
- APSIM water factor for mineralisation [F]: piecewise-linear in relative water content, X = 0 (LL15), 0.5, 1 (DUL), 2 (SAT); Y = 0.05 (or 0), 1, 1, 0.5. Nitrification loses 0.002 of the nitrified N as N2O. Denitrification exists as a process with NLoss 1.0. — [APSIM Nutrient.json](https://raw.githubusercontent.com/APSIMInitiative/ApsimX/master/Models/Resources/Nutrient.json)
- Conversion between ppm and kg/ha in APSIM [F]: kg/ha = ppm × BD (g/cm³) × thickness (mm) / 100. — [APSIM Nutrient.json](https://raw.githubusercontent.com/APSIMInitiative/ApsimX/master/Models/Resources/Nutrient.json)
- APSIM Classic SoilN default rates [K, to verify]:
  - rd_hum about 1.5e-4 /day and rd_biom 0.0081 /day (the latter matches the [F] value above).
  - FOM fractions decompose at rd_carb 0.2, rd_cell 0.05 and rd_lign 0.0095 /day.
  - Temperature factor (T/opt)² with an optimum of about 32-35 °C.
  - Nitrification is Michaelis-Menten: potential about 40 ppm NH4-N /day, half-saturation about 90 ppm.
  - Denitrification coefficient about 6e-4 × active C, and it acts only above DUL.
  - Source: APSIM SoilN documentation (Probert, Dimes, Keating, Dalal & Strong 1998, "APSIM's water and nitrogen modules and simulation of the dynamics of water and nitrogen in fallow systems", Agricultural Systems 56:1-28).
- CERES/DSSAT soil N (Godwin & Jones 1991; Godwin & Singh 1998) [K, to verify]: the same FOM (carbohydrate/cellulose/lignin) plus humus structure. NH4 is adsorbed and does not move. NO3 leaches with the downward water flux, as `fraction leached = flux / (SW·thickness + flux)` per layer. Urea hydrolysis takes days, nitrification about a week. CENTURY (Parton et al. 1987) uses active, slow and passive SOM pools with turnover of about 1.5 yr, 25 yr and 1000 yr. SWAT uses an active/stable humic split and an anion-exclusion fraction of about 0.5 for nitrate.
- CROPGRO uptake reads NO3 and NH4 per layer in ppm, and leaves a floor of 0.25 ppm NO3-N and 0.5 ppm NH4-N that roots cannot remove [F]. — [NUPTAK.for](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Plant/CROPGRO/NUPTAK.for)
- Typical garden and arable soil values [K, to verify]:
  - Topsoil organic C 1.5-5%, C:N 10-12, total N 0.1-0.3%.
  - In the 0-20 cm layer at BD 1.2-1.3 (about 2,500 t soil/ha), that is about 2,500-7,500 kg N/ha.
  - Net mineralisation is 1-3% of organic N per year, about 50-150 kg N/ha/yr, or about 0.5-1.5 kg N/ha/day in warm, moist summer conditions.
  - Mineral N in unfertilised topsoil is about 5-30 mg N/kg. 1 mg N/kg over 20 cm at BD 1.3 is 2.6 kg N/ha.

### Inferences
- [I] Recommended minimal model, per soil cell:
  - Pools (5 state variables):
    - `org_n_active` (humus N) with fixed C:N 10-12 (one parameter);
    - `litter_c` and `litter_n` (fresh residue: senesced roots, nodules, fallen leaves, compost), first-order with C:N-dependent net N release;
    - `nh4`;
    - `no3`.
    - Leave out explicit BIOM and use the APSIM CO2 efficiency of 0.4 implicitly. When litter C:N exceeds about 25, net immobilisation comes out of the C:N balance and draws from mineral N.
  - Rates at the 30 °C reference: humus 1.5e-4 /day [K]; litter 0.02-0.05 /day as an effective lumped FOM rate [I, between the rd_cell and rd_carb values].
  - Rate modifiers: temperature `f_T = (T/30)²` clipped to [0, 1.2] (or Q10 = 2 around 25 °C), multiplied by the APSIM water factor above [F].
  - Nitrification: first-order NH4 → NO3 at about 0.1-0.2 /day at 20 °C, scaled by the same f_T and f_W [K, to verify; this gives the "about a week" turnover]. This keeps NH4 and NO3 separate. That matters because only NO3 leaches, and nitrate rather than ammonium is the main nodulation inhibitor.
  - Fertilizer forms map onto the pools: urea → NH4 after about 2-day first-order hydrolysis; ammonium sulfate → NH4; calcium/potassium nitrate → NO3; compost/manure → litter with C:N 10-20; blood meal → litter with C:N about 4 (fast).
  - Leaching: NO3 moves with each cell-to-cell water flux using the CERES mixing rule `moved = no3 × q/(θ·V + q)` [K]. A mobile fraction around 0.5-1.0 (SWAT-like anion exclusion) is optional. NH4 does not move.
  - Denitrification: leave it out of the first WP24 cut, or add `k_dn = 0.0006 × C_active × f_T` only while θ > field capacity (APSIM-like) [K]. In pots this matters only for overwatered pots. Its value in the game is a lesson: waterlogging loses nitrogen.
- [I] Hourly step: use k_hour = 1 − (1 − k_day)^(1/24), or k_day/24 since all rates are ≤ 0.2 /day. Both are deterministic. Conservation check: org N + litter N + NH4 + NO3 + plant N + leached + denitrified + fixed-input = constant.
- [I] Pot soil (2 cm cells, potting mix) differs from field soil. Commercial potting mixes are often low in organic N but charged with slow-release fertilizer. A "sterile sand + N-free nutrient solution" substrate is needed for the WP24 acceptance test.

### Gaps
- I could not read the APSIM Humic and FOM rate constants or the temperature factor from Nutrient.json; the file nests them in a form my scan did not resolve. The APSIM Classic values above are [K]. Verify them against the APSIM SoilN science docs or the `SoilNitrogen` Classic source.
- No garden-soil survey numbers for mineral N or mineralisation were fetched this session. The typical values above are [K].

## Root N uptake: Michaelis-Menten parameters and whether diffusion matters at 2-5 cm

### Takeaway
SimRoot/OpenSimRoot gives bean nitrate kinetics per unit root surface (Barber book values): Imax 1.9 µmol/cm²/day, Km 16.1 µM, Cmin 1 µM. CROPGRO uses an empirical per-root-length capacity of 0.006 mg N/cm/day for each of NO3 and NH4, saturating in soil ppm. The two land at the same per-length uptake of about 2-6 µg N per cm root per day. In any fertile cell the solution concentration is far above Km, so pure MM saturates. Supply to the root surface by diffusion and mass flow is what really limits uptake. At 2-5 cm cells, use a CROPGRO-like saturating function of bulk soil N, with root length density as the driver, instead of solution-concentration MM.

### Cited Findings
- OpenSimRoot bean (cv. Carioca) nitrate [F]:
  - Imax = 1.9 µmol/cm²/day (per root surface), Km = 16.1e-3 µmol/mL (16.1 µM), Cmin = 1e-3 µmol/mL (1 µM).
  - The same values are used for primary root, basal whorls and hypocotyl-born roots.
  - The file's header comment reads "barberbook for nitrate: Imax=1.9, Km=16.1e-3, E=0.5", i.e. Barber 1995, Soil Nutrient Bioavailability.
  - Source: [OpenSimRoot nitrate.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/nitrate.xml)
- OpenSimRoot uptake carbon cost [F]: 1.392e-5 g C/µmol nitrate, i.e. about 1 g C/g N, citing Bouma, Broekhuysen & Veen 1996 (0.39-1.16 mol O2/mol nitrate). — [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- CROPGRO NUPTAK [F]:
  - Per layer: `RFAC = RLV × sqrt(SMDFR) × DLAYR × 100`, then `RNO3U = RFAC × FNO3 × RTNO3` (kg N/ha/day).
  - `FNO3 = 1 − exp(−0.08 × NO3ppm)`, set to 0 if below 0.04; NH4 uses the same form.
  - `SMDFR = (SW−LL)/(DUL−LL)`, floored at 0.1, and reduced above DUL toward SAT.
  - Total uptake = min(demand, Σ capacity), scaled proportionally and limited by the residual floors.
  - Comments in the code: "RTNO3 + RTNH4 = Nitrogen uptake / root length (mg N/cm)".
  - Source: [NUPTAK.for](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Plant/CROPGRO/NUPTAK.for)
- CROPGRO-Drybean RTNO3 = 0.006 and RTNH4 = 0.006 mg N/cm root/day [F]. — BNGRO048.SPE line 76.
- Other root-N parameters [K, to verify]:
  - Itoh & Barber (1983) and the Barber-Cushman model add a diffusion coefficient (NO3 De about 1e-6 to 1e-5 cm²/s in moist soil) and buffer power (about 1 for nitrate, about 5-10 for NH4).
  - CPlantBox and OpenSimRoot solve 3D diffusion-uptake (OpenSimRoot via SWMS/"Barber-Cushman" per segment).
  - APSIM legume models use supply = kNO3 × NO3 × RLV-based exploration, with kNO3 about 0.02 /day per unit root length density.

### Inferences
- [I] Unit reconciliation:
  - Bean fine root diameter is about 0.3 mm, so surface ≈ 0.094 cm²/cm root. Imax 1.9 µmol/cm²/day then gives 0.18 µmol N/cm/day ≈ 2.5 µg N/cm/day.
  - CROPGRO RTNO3 = 6 µg N/cm/day.
  - These agree within a factor of about 2.5, so use about 3-6 µg N/cm/day as the maximum per-length capacity for each form.
- [I] Concentrations:
  - 10 mg N/kg soil at BD 1.3 and θ 0.25 is about 52 mg N/L solution ≈ 3.7 mM, more than 200 × Km.
  - Solution MM is therefore always saturated except in depleted cells. Uptake in real soil is limited by transport to the root, which at 2-5 cm resolution happens inside the cell (depletion zones are mm-scale).
  - So: `uptake_cell = min(demand_share, L_root × Imax_len × f(N_cell) × f(θ))`, with `f(N) = 1 − exp(−k × N_ppm)` (CROPGRO, k = 0.08/ppm, half-saturation about 8.7 ppm) or `N/(N + K_soil)` with K_soil about 5-10 ppm. Cap it by `N_cell − residual` (0.25-0.5 ppm).
  - No explicit diffusion solver is needed. The saturating soil-ppm function is the coarse-grid equivalent.
- [I] Split demand among cells by capacity, as CROPGRO does (proportional scaling NUF). That keeps everything deterministic and conservative.

### Gaps
- I did not fetch the Barber (1995) or Itoh & Barber (1983) primary tables. I found no bean-specific NH4 kinetics.

## Plant N: critical dilution curve, organ N concentrations, leaf N to Vcmax, remobilisation and emergent senescence

### Takeaway
A field-fitted critical N dilution curve exists for common bean: Nc (%) = 4.72 W^−0.32, with W in t/ha shoot DM. That is close to the legume-generic Lemaire curve of about 4.8 W^−0.33. Organ N ranges are best taken from CROPGRO-Drybean (protein × 0.16) and OpenSimRoot bean (optimal and minimal µmol/g). Bottom-up senescence emerges if seed N demand is met first from mobile leaf protein (CROPGRO "N mining"). Leaves losing protein below their final level then senesce, lowest and oldest first.

### Cited Findings
- Common bean critical N dilution curve [F]: Nc (%) = 4.72 W^−0.32 (generic across 3 cultivars, Bayesian fit, two field experiments at 0, 60 and 120 kg N/ha, Botucatu, Brazil). The integrated NNI explained relative seed yield (R² 0.74), and NNI between R5 and R7 predicted yield. — [Soratto et al. 2025, Field Crops Research](https://consensus.app/papers/details/e2921cae57b951a4b863cf42f315bca8/?utm_source=claude_desktop)
- A global dataset of CNDC for 19 crop species (4,454 observations) exists for cross-checking. — [Ciampitti et al. 2022, Scientific Data](https://consensus.app/papers/details/fad7d9fa109c57fba1f86b52cba7fe34/?utm_source=claude_desktop) [F abstract]
- Generic C3 curve Nc = 5.7 W^−0.5 (Greenwood et al. 1990); alfalfa/legume curve 4.8 W^−0.33 (Lemaire et al. 1985) [K, to verify].
- CROPGRO-Drybean tissue protein fractions [F, BNGRO048.SPE lines 23-25]. N% = protein × 0.16 [I arithmetic]:

  | Organ | Initial (new growth, N rich) | Growth | Final (after mining) |
  | --- | --- | --- | --- |
  | Leaf | PROLFI 0.24 → 3.84% N | PROLFG 0.16 → 2.56% N | PROLFF 0.07 → 1.12% N |
  | Stem | PROSTI 0.15 → 2.40% N | 0.10 → 1.60% N | 0.05 → 0.80% N |
  | Root | PRORTI 0.10 → 1.60% N | 0.08 → 1.28% N | 0.06 → 0.96% N |
  | Shell | PROSHI 0.22 → 3.52% N | 0.22 → 3.52% N | 0.08 → 1.28% N |
  | Seed | SDPROS 0.235 → 3.76% N | SDPROG 0.235 | |
  | Nodule | PRONOD 0.30 → 4.80% N | | |

- CROPGRO N mobilisation [F, BNGRO048.SPE line 36]: NMOBMX 0.12 (maximum fraction of mobile protein mobilised per day), NVSMOB 0.36 (relative mobilisation rate in the vegetative phase), NRCVR 0.15. CMOBMX 0.030 for carbohydrate. NSTFAC 0.70, XPODF "SD".
- OpenSimRoot bean N concentrations [F] (µmol N/g DW; × 14 µg/µmol gives %N [I]):
  - Leaf optimal 2600 (3.64%), minimal 1300 (1.82%).
  - Stem optimal 1300 (1.82%), minimal 700 (0.98%).
  - Roots and hypocotyl optimal 1200 (1.68%), minimal 600 (0.84%).
  - Seed N reserve 714 µmol per seed (about 10 mg N; "should be enough for the first 7 days or so"). Bean seed is 3-5% N (Mosse 1990).
  - Bean whole-plant N about 3.7% in the three-sisters study.
  - Source: [nitrate.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/nitrate.xml)
- CROPGRO photosynthesis-N [F, BNGRO048.SPE line 12]: SLWREF 0.0034, SLWSLO 0.0012, NSLOPE 0.35, LNREF 3.14, PGREF 1.00. Canopy PG scales with leaf N relative to LNREF (3.14% leaf N, a protein of about 0.196) with slope NSLOPE, and with SLW via the XPGSLW/YPGSLW table [K, to verify the exact equation in PHOTO.for].
- Vcmax-Na [K, to verify]:
  - Most C3 data fit Vcmax25 = a + b·Na, with b ≈ 30-60 µmol CO2 per g N per s (often written as about 20-40 µmol m⁻² s⁻¹ per g N m⁻², depending on dataset) and a non-photosynthetic N intercept of about 0.3-0.5 g N/m². Kattge et al. 2009 (Global Change Biology 15:976) gives PFT-specific "nitrogen use efficiency of Vcmax" for C3 crops. Walker et al. 2014 (Ecology and Evolution 4:3218) gives a global log-log fit, Vcmax = exp(3.946 + 0.921·ln Na − 0.121·ln SLA + 0.282·ln Na·ln SLA) [K; verify coefficients].
  - Bean leaf Na at full N is about 1.5-2.5 g N/m² [K].
  - Jmax ≈ 1.6-2 × Vcmax.
- Self-destruct hypothesis [K, to verify]: Sinclair & de Wit 1975 (Science 189:565) showed that high-protein seeds of legumes (bean, soybean) need N faster than uptake plus fixation can supply during seed fill. They mine leaf N, which shortens the seed-fill period. CROPGRO implements this as "N mining" (NMOBMX) from leaf, stem, root and shell protein down to PRO*F. Leaf mass loss is coupled to mobilised N (leaf abscission when leaf protein reaches PROLFF).

### Inferences
- [I] Organ N state per organ: `n_mass`, with bounds `[n_min, n_opt]` × DM from the CROPGRO table (initial = opt for new tissue, final = min).
  - Daily or hourly demand = growth × N_opt + refill toward N_opt.
  - Supply order: soil uptake → fixation (if nodulated) → mobilisation from vegetative organs (pool = n_mass − n_min × DM, at rate ≤ NMOBMX/day, higher after R5).
  - Seeds get priority after R5, as in CROPGRO (NDMREP served before NDMVEG).
- [I] Emergent bottom-up senescence: when a leaf's N/area falls below a threshold (for example Na < 0.6-0.8 g N/m², or N% < PROLFF × 0.16 = 1.12%), the leaf senesces and drops, and its residual N goes to litter. Mining should draw first from the leaves with the lowest light (lowest and oldest), as real phloem remobilisation is driven by shading and age [K]. Then senescence starts at the bottom without being scripted. Lower leaves also have lower Na because Vcmax/Na acclimates to light, so Na-proportional mining reinforces it.
- [I] Vcmax: use Vcmax25 = b × (Na − Nb) with b about 40 µmol CO2 per g N per s (that is, µmol m⁻² s⁻¹ per g N m⁻²) and Nb 0.3 g N/m². Validate so that well-fertilised bean (Na about 2 g/m²) gives Vcmax25 about 70-100 µmol m⁻² s⁻¹. All three are [K, to verify] against Kattge 2009 or bean gas-exchange data. Bean Vcmax25 is often reported around 60-110 [K].
- [I] Acceptance check for the dilution curve: plants on non-limiting N should track 4.72 W^−0.32 within about ±15%. Plants on N-free substrate without nodules should fall to NNI < 0.5.

### Gaps
- Kattge 2009 and Walker 2014 were not fetched. The Vcmax-Na slope must be verified before it goes into data/ with a source.
- I did not read CROPGRO's PHOTO.for or SENES.for, so the exact LNREF/NSLOPE formula and the coupling of N mining to leaf abscission are [K].

## Nodulation and fixation in bean: timing, rates, cost, Ndfa, inhibition, stresses and model implementations

### Takeaway
CROPGRO makes fixation demand-driven. Carbon goes to nodules only for the part of the N demand not met by soil uptake and mining. That reproduces nitrate suppression of fixation without a separate inhibition function. CROPGRO-Drybean carries explicit parameters: specific activity, nodule growth rate, initial mass, death rate, and temperature, drought, flooding and age curves. Bean is the weakest grain-legume fixer: meta-analytic Ndfa is 0.57 against 0.84 for other legumes. The real fixation cost is about 6-7 g C per g N (CROPGRO's 2.83 g CH2O per g protein equals 7.1 g C/g N). The 2.8 g C/g N figure quoted in OpenSimRoot looks like a unit misreading.

### Cited Findings
- CROPGRO-Drybean fixation parameters [F, BNGRO048.SPE lines 40-46, 52]:
  - SNACTM 0.050 g N per g nodule per day (maximum specific nodule activity).
  - NODRGM 0.170 /day (maximum relative nodule growth).
  - DWNODI 0.014 g nodule per plant (initial).
  - TTFIX 0.0 (V-stage lag before fixation starts).
  - NDTHMX 0.04 /day (maximum nodule death rate).
  - CNODCR 0.05 (C requirement for nodule respiration, g C/g nodule/d per the code comment).
  - FRCNOD 0.05 (fraction of root-allocated C reserved for nodules before R2).
  - Temperature curves, LIN (soil temperature, °C):
    - Nodule growth FNNGT 6/21/35/44: zero at 6 °C, rising to 1 at 21 °C, flat to 35 °C, zero at 44 °C.
    - Fixation FNFXT 4/19/35/44.
  - Drought FNFXD 0/0.85/1/10 vs TURFAC: fixation full while turgor factor ≥ 0.85, falling linearly to 0 at 0.
  - Flooding FNFXW −0.02/0.001/1/2 vs air-filled porosity (SAT − SW): fixation stops when the soil is saturated.
  - Age FNFXA 0/0.1/1/0 INL vs DXR57.
- CROPGRO NFIX equations [F] — [NFIX.for](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Plant/CROPGRO/NFIX.for):
  - Temperature and flooding factors are depth-weighted over the top 50 cm (DNOD = 50 cm).
  - `SWFACT = curve(TURFAC)`, plus an 8-day running mean of SWFACT (SWMEM8): drought has memory.
  - `CLEFT = CTONOD − CNODMN`.
  - Potential fixation `PNFIXN = min(CLEFT × 0.16 / RFIXN, DWNOD × SNACT) × TNFIX`, with `SNACT = SNACTM × EFNFIX` (strain efficiency).
  - Actual fixation `NFIXN = PNFIXN × min(SWFACT, SWMEM8, FLDACT)`. Fixation is 0 once DAS ≥ NR7 (physiological maturity).
  - C used `CUSFIX = NFIXN/0.16 × RFIXN`. Leftover C plus CNODMN grows nodules at up to `NODRGM × EFNFIX × EFINOC` (EFINOC = inoculation or rhizobium density factor).
  - Nodule death `RNDTH = NDTHMX × max(1−FLDACT, 1−SWFACT, 1−CNFACT)`, where CNFACT = (C to nodules per nodule mass)/CNODCR. Dead nodules go to soil as SENNOD by layer.
- CROPGRO C allocation to nodules [F] — [CROPGRO.for](https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/Plant/CROPGRO/CROPGRO.for):
  - `CTONODR = max(0, (NDMREP − NAVL) × RFIXN/0.16)`: C to fix the reproductive N demand not covered by NAVL, the N available from uptake plus mining.
  - `CNDFX = (RFIXN/0.16) × (NDMVEG − NAVLV)`. CTONOD for vegetative demand comes from a partition formula, plus `CNODMN = CAVVEG × FRRT × FRCNOD` before R2.
  - NFIX runs only if ISWSYM = 'Y' and VSTAGE > TTFIX.
  - Soil N supply is used first, so high soil nitrate leads to little C sent to nodules, which leads to little fixation and slow nodule growth.
- CROPGRO costs [F, SPE line 19]: RFIXN 2.830 g CH2O per g protein fixed; RNO3C 2.556 and RNH4C 2.556 g CH2O per g protein for nitrate and ammonium assimilation; RPRO 0.360.
- OpenSimRoot BNF cost [F]: 3.95e-5 g C/µmol N = 39.5 g C/mol N = 2.82 g C/g N. The comment says "cropgro uses 7.06 g glucose/g N fixed … halfway [between] values reported for soybean by LaRue and Patterson (1981) and Ryle et al. (1979)". A second comment cites Minchin & Witty 2005: "costs 3–5 kg C per kg N fixed (7–12 mol CO2 per mol N2) including construction and maintenance of nodules". OpenSimRoot sets relativeRelianceOnBNF = 0.30 for Carioca bean. — [carioca.xml](https://gitlab.com/rootmodels/OpenSimRoot/-/raw/master/OpenSimRoot/InputFiles/plantParameters/Bean/Carioca/carioca.xml)
- Bean Ndfa:
  - Meta-analysis of 83 studies and 889 points: Ndfa slope 0.84 across legumes except common bean at 0.57, with the largest gap between potential and mean fixation. — [Palmero et al. 2022, Eur J Agron](https://consensus.app/papers/details/b29e560b555153669b9bb2c4a8db2898/?utm_source=claude_desktop) [F abstract]
  - 63 genotypes in Eswatini: Ndfa 19.6-88.4%, 2.7-66.8 kg N fixed/ha. 40 of 63 took more N from soil than from fixation. — [Gunununu et al. 2026, Front Agron](https://consensus.app/papers/details/5e0732a8155f5531b9688595ad137338/?utm_source=claude_desktop) [F abstract]
  - Heirloom panel: five genotypes above 60% Ndfa; Middle American fixed more than Andean. — [Wilker et al. 2019](https://consensus.app/papers/details/e80cc206aadd5a1b97a0bb3ccae619a4/?utm_source=claude_desktop) [F abstract]
  - Andean lines nodulated more, but Middle American lines fixed more under good moisture. — [Farid & Navabi 2015](https://consensus.app/papers/details/fb178bf2ff405156bc39d444ff8a6bfb/?utm_source=claude_desktop) [F abstract]
- Fertilizer N reduces SNF in bean, to differing degrees across 22 genotypes on N-poor soils over 3 years. Rhizobial inoculation had only inconsistent effects (resident rhizobia). — [Reinprecht et al. 2020, Front Plant Sci](https://consensus.app/papers/details/c2b2184654ea5383a6c50b6b1871c82a/?utm_source=claude_desktop) [F abstract]
- Drought "severely limits" bean SNF; Ndfa fell significantly under drought (36 lines, CIAT Palmira). — [Polanía et al. 2016, Eur J Agron](https://consensus.app/papers/details/b24dd6cf98f85feca11879c8606932fa/?utm_source=claude_desktop) [F abstract]
- Micronutrients matter: biochar raised bean Ndfa from 50% to 72%, mainly through greater B and Mo availability. Soil N uptake by fixing beans fell as fixation rose. — [Rondón et al. 2007, Biol Fertil Soils](https://consensus.app/papers/details/034e160a90d65fe99bdb92e3307a86ef/?utm_source=claude_desktop) [F abstract]
- Timing [K, to verify]:
  - Bean nodules are visible about 7-10 days after emergence, and acetylene-reduction activity is detectable by about 14-21 DAE.
  - Activity peaks around flowering to early pod set (R1-R3, about 35-50 DAP) and drops sharply during pod fill. Bean nodules senesce earlier than soybean nodules, which contributes to low Ndfa.
  - Nodule dry mass at flowering is about 50-300 mg per plant in pots and fields (Graham 1981; Hardarson et al. 1993; Piha & Munns 1987).
- Inhibition by nitrate [K, to verify]: in nutrient solution, nodulation is sharply reduced above about 2-5 mM NO3⁻ and largely absent at 10-15 mM. Established nodules lose nitrogenase activity within days at 5-10 mM (Streeter 1988 review; Streeter & Wong 1988). In soil, starter rates up to about 20-30 kg N/ha usually do not suppress nodulation, but 60-120 kg N/ha usually reduce Ndfa strongly.
- Rhizobia [K, to verify]: bean is promiscuous. Effective symbionts include Rhizobium tropici (acid- and heat-tolerant, CIAT 899 is a common inoculant strain), R. etli (dominant in Mesoamerican soils), R. leguminosarum bv. phaseoli and R. gallicum. Low P limits nodulation strongly: nodules need high P, and P deficiency cuts nodule mass and Ndfa (Vadez & Drevon; Kouas et al.).
- Other model implementations [K, to verify]:
  - APSIM legume crops (soybean, mungbean, chickpea) use an "arbitrator" with a Fixation supply = nodule-demand-limited potential fixation rate (g N per g root or nodule per day) × water and temperature factors. Fixation is drawn after soil uptake, so it is also demand-driven. Fixation has a respiratory cost (about 6 g CH2O per g N fixed in the older Plant module).
  - STICS uses a potential fixation rate per plant from growth rate and a nodule-activity factor, inhibited by soil NO3 above a threshold (about 30-70 kg N/ha in the nodulated zone; Duc et al.) and by water, temperature and anoxia.
  - OpenSimRoot covers N demand not met by uptake through BNF up to relativeRelianceOnBNF, at a C cost per µmol.

### Inferences
- [I] Cost reconciliation:
  - RFIXN 2.83 g CH2O per g protein ÷ 0.16 g N/g protein = 17.7 g CH2O per g N = 7.07 g C per g N (CH2O is 40% C).
  - Nitrate assimilation in CROPGRO: 2.556/0.16 = 16.0 g CH2O/g N = 6.4 g C/g N. So fixation is only about 10% more expensive than nitrate in CROPGRO.
  - OpenSimRoot's "7.06 g glucose/g N = 2.8 g C/g N" divides 2.83 by 0.4 instead of 0.16 (or reads RFIXN as per g N), which under-states CROPGRO's cost by a factor of 2.5.
  - Literature totals: Minchin & Witty 3-5 g C/g N (incl. nodule growth and maintenance). The [K] classic soybean figures are 6-7 g C/g N (Vance & Heichel 1991 range 4-17 g C/g N).
  - Recommended: about 6 g C per g N fixed as the fixation respiration, plus explicit nodule growth (4.8% N, ~0.45 C fraction) and nodule maintenance respiration.
  - The design-plan figure of "10-15% of assimilate" should be an emergent check, not a parameter. At peak, a bean fixing 1.5 mg N/plant/day at 6 g C/g N spends 9 mg C/day, about 10-15% of a 70-90 mg C/day flowering plant [I; magnitude only].
- [I] Recommended model:
  - Nodules are a root-attached organ pool per plant, optionally placed in the cells holding the upper 0-15 cm of roots, with mass M_nod.
  - Fixation demand = max(0, N_demand − N_uptake − N_mined).
  - Potential fixation = min(SNA × M_nod × f_T(soil T) × f_age, C_to_nodules × 0.16/RFIXN-equivalent).
  - Actual fixation = potential × min(f_drought, f_drought_8day_mean, f_flood).
  - Nodule growth gets FRCNOD (0.05) of root C before R2, plus any surplus C, at up to 0.17 /day relative.
  - Nodules die at up to 0.04 /day under C shortage, drought or flooding, with dead nodule N going to the litter pool. This gives the "beans leave nitrogen in the soil" effect.
  - Also add a direct nitrate inhibition multiplier on nodule growth, for example f_NO3 = 1 for NO3 ≤ 5 ppm, falling linearly to 0.1 at 30 ppm [I, calibrate to about 2-10 mM in solution]. Without it, demand-driven allocation alone suppresses fixation but still lets nodules form in rich soil, while real bean nodulation is visibly reduced.
- [I] SNACTM check: 0.05 g N/g nodule/day × 0.15 g nodule = 7.5 mg N/plant/day maximum. CROPGRO's C-limit and stress factors keep realised rates lower. A realistic bean peak is about 1-3 mg N/plant/day (60-120 kg N/ha over a season at 20-30 plants/m², about 2-4 mg N/plant/day at peak) [K/I]. Use 0.02-0.05 and calibrate so that season Ndfa in a moderate-N garden soil lands at about 30-60%.
- [I] Genotype loci for M3: SNA (strain/host efficiency, EFNFIX), nodulation ability (EFINOC-like), and a non-nodulating allele (nod⁻, like R99 or BAT 477NN), which gives the breeding game a real trait.
- [I] Acceptance test design for "Nodulated beans grow in N-free soil and unnodulated ones do not":
  - Substrate: sterile sand or vermiculite with no organic N and NO3 = NH4 = 0, watered with an N-free nutrient solution (Broughton & Dilworth 1971 type [K]).
  - Seed N about 10 mg (OpenSimRoot, 714 µmol [F]). Without nodules, total plant N ≤ seed N, so biomass ≤ 10 mg N / ~1.5% mean N ≈ 0.7 g DW, plus chlorosis (Na falling) and no pods or very few [I].
  - With nodules (inoculated), after about 3 weeks fixation should carry the plant to several grams DW and set pods.
  - Pass if nodulated/unnodulated shoot DM at 45 DAE is > 3 and the unnodulated plant's N stays below seed N plus 1%.
  - Real-world analogues: the non-nodulating isoline BAT 477NN against nodulating BAT 477 ([Rondón et al. 2007](https://consensus.app/papers/details/034e160a90d65fe99bdb92e3307a86ef/?utm_source=claude_desktop)), and the non-nodulating mutant R99, used as the non-fixing reference ([Farid & Navabi 2015](https://consensus.app/papers/details/fb178bf2ff405156bc39d444ff8a6bfb/?utm_source=claude_desktop); [Reinprecht et al. 2020](https://consensus.app/papers/details/c2b2184654ea5383a6c50b6b1871c82a/?utm_source=claude_desktop)) [F].

### Gaps
- I did not retrieve a primary bean time course of nodule mass or acetylene reduction (for example Piha & Munns 1987 or Hungria et al.), so the timing and nodule mass figures are [K].
- I did not retrieve primary nitrate-threshold numbers (Streeter 1988) for bean. Thresholds are [K].
- I did not fetch the APSIM soybean/mungbean and STICS fixation docs. Their descriptions are [K].
- No primary source was read for the bean-specific P × nodulation relationship.

## Fertilizer: garden rates, forms and expected bean response

### Takeaway
Beans need little or no N fertilizer in a reasonably fertile garden. Field trials use 0, 60 and 120 kg N/ha, and N raises yield on N-poor soils while cutting fixation. The fertilize action should offer forms that map onto NH4, NO3, urea and organic pools, at garden-scale doses.

### Cited Findings
- Common bean field trials used 0, 60 and 120 kg N/ha. Seed yield responded to N rate (p < 0.01), with no cultivar × N interaction. — [Soratto et al. 2025](https://consensus.app/papers/details/e2921cae57b951a4b863cf42f315bca8/?utm_source=claude_desktop) [F abstract]
- N fertilizer is commonly used for high bean yields and "in general inhibits nitrogen fixation". N reduced SNF to different degrees across genotypes. — [Reinprecht et al. 2020](https://consensus.app/papers/details/c2b2184654ea5383a6c50b6b1871c82a/?utm_source=claude_desktop) [F abstract]
- Faba bean analogue: BNF and yield peaked at 90 kg N/ha and declined above it. The total of fertilizer N plus soil residual N should stay below about 200 kg N/ha. — [Pu et al. 2025, Crop Science](https://consensus.app/papers/details/3b1b7154e6a259bfa3164cbc1df00209/?utm_source=claude_desktop) [F abstract]
- Garden extension guidance [K, to verify]:
  - Vegetable gardens get about 1-2 lb N per 1,000 ft² (50-100 kg N/ha) per season. Beans are usually advised to get little or no N, or a small starter of about 0.25-0.5 lb N/1,000 ft² (12-25 kg N/ha).
  - Common forms: urea 46-0-0, ammonium sulfate 21-0-0, calcium nitrate 15.5-0-0, balanced 10-10-10, blood meal about 12-0-0, compost about 1-2% N with C:N 10-20.
  - Pot doses: about 100-200 mg N/L of liquid feed weekly.

### Inferences
- [I] Fertilize action: the player picks a product (form plus N content) and a dose in g per pot or g/m². The sim adds N to the surface cells: NH4, NO3 or urea → NH4, or the litter pool for organics. The game can show:
  - fast greening on poor soil;
  - a reduced nodule count and Ndfa at high doses;
  - leaching loss after heavy watering of nitrate forms;
  - little benefit for nodulated beans in fertile soil.
- [I] Expected response magnitude to aim for: on N-poor soil, unfertilised nodulated bean gives about 60-80% of fertilised yield. On fertile garden soil, the response to N is under 10-15% [K/I; calibrate against Soratto 2025 full text].

### Gaps
- Garden extension rates were not fetched this session ([K]).
- I have no primary dose-response curve for bush snap bean (as opposed to dry bean).

Note: the Consensus searches used for paper abstracts carried this notice: "2 credits left this month; resets to 30 on December 1st. Upgrade to Consensus Pro to get 500 searches per month and include more data like study design and key takeaways for every result.: https://consensus.app/pricing/?utm_source=claude_desktop"
