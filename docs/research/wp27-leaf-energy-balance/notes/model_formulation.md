# WP27 leaf energy balance: model formulation and fit to Plant-Sim

Tags: [F] = I read the source (code or paper text) in this session; [K, to verify] = from my
knowledge, not re-read; [I] = my inference or my own prototype result.
Prototype used for the numbers marked [I, proto]: `../prototype/proto.py`, `sweep3.py`
(pure Python, FvCB + Medlyn copied from `ps-physio/src/gas.rs` with the bean parameters,
coupled to the energy balance below; the reference solution uses nested bisection on the full
nonlinear balance).

---

## 1. Takeaway (recommended algorithm)

**Where it sits.** Inside hourly step 6 (gas exchange per light class, §3.2). No step is
reordered. Step 1 gains wind speed (and uses cloud fraction for sky longwave); step 3 already
supplies each leaf's sky view (`LightCoeffs.diffuse`); step 7 consumes the new transpiration
unchanged; step 8 may read leaf temperature for leaves; step 11 (thermal time) stays on air
temperature. Because equations, defaults and goldens change, this is a model minor-version bump
plus a decision record (CLAUDE.md), but not a step-order change.

**Two-level scheme.**
(a) Each class-table node (17 PAR × 4 capacity) solves the *coupled* gas exchange and energy
balance at a *reference* radiative and wind environment shared by all plants of a species in
the hour (so tables stay shareable by `class_table_key` + wind + leaf size). The node stores A,
gs, its leaf temperature T_node and the node's gv.
(b) Each leaf part (sunlit, shaded) then computes *its own* leaf temperature and transpiration
in closed form from its own absorbed shortwave, its own sky view (longwave), its own wind
(attenuated by leaf area above it), and the gs it read from the table (after the sink-feedback
scaling), with one Newton correction. A is taken from the table (evaluated at T_node).

### Equations (per unit one-sided leaf area; both faces exchange heat and longwave)

Symbols: Ta air temperature (°C), Tk = Ta + 273.15 K, P air pressure (kPa), ea air vapour
pressure (kPa) = es(Ta) − D_air, ε_l leaf emissivity, σ Stefan–Boltzmann, cp molar heat
capacity of air (J mol⁻¹ K⁻¹), λ molar latent heat (J mol⁻¹), s = des/dT (kPa K⁻¹).

1. Absorbed shortwave (W m⁻²):
   `S_abs = Q_abs / 4.57 × (1 + r_nir)` where Q_abs is absorbed PAR (µmol m⁻² s⁻¹,
   what `ps_light::absorbed_par` already returns), 4.57 µmol J⁻¹ is the repo's
   `UMOL_PER_J_PAR` [F], and `r_nir = (f_nir / f_par) × (a_nir / a_par)` is absorbed NIR per
   absorbed PAR energy. With the repo's PAR_FRACTION 0.5 [F], a_par 0.85 [F species file] and
   a_nir ≈ 0.20 (Bonan's ρ_nir = τ_nir = 0.40 [F]) → r_nir ≈ 0.24 in the open [I].
   Shaded parts inside the canopy see NIR-enriched light: per leaf use
   `τ_nir = τ_par^(√(1−σ_nir)/√(1−σ_par))` ≈ `τ_par^0.5` (σ = leaf scattering 0.8 NIR,
   0.2 PAR; Goudriaan 1977 / Spitters 1986 form [K, to verify]) to raise r_nir for shaded
   parts [I].
2. Absorbed longwave (W m⁻², both faces): upper face sees sky over its sky view `f_sky`
   (use `LightCoeffs.diffuse`, already the share of horizontal diffuse sky reaching the leaf
   [F]) and leaves/ground at Ta elsewhere; lower face sees Ta:
   `L_abs = ε_l σ Tk⁴ (1 + (1 − f_sky) + f_sky ε_sky)`.
   Sky emissivity: Brutsaert 1975 clear sky `ε_clear = 1.72 (ea/Tk)^(1/7)` (ea in kPa) with
   cloud correction `ε_sky = (1 − 0.84 c) ε_clear + 0.84 c` (Unsworth & Monteith 1975, as in
   Campbell & Norman 1998 eq. 10.12) [K, to verify; plantecophys uses the same Brutsaert form
   as 0.642 (ea[Pa]/Tk)^(1/7) [F]], c = `HourWeather.cloud_fraction`.
3. Isothermal net radiation: `Rni = S_abs + L_abs − 2 ε_l σ Tk⁴`
   = `S_abs − ε_l σ Tk⁴ f_sky (1 − ε_sky)`.
4. Conductances (mol m⁻² s⁻¹):
   - heat, per face: `g_bh = 0.135 √(u/d) + g_free` with d = 0.72 × leaf width
     (Campbell & Norman eq. 7.30 [K, to verify]; Leuning 1995 forced 0.003√(u/w) m s⁻¹ × molar
     density ≈ 0.124√(u/w) [F via plantecophys]); free convection
     `g_free = 0.5 D_H (1.6e8 |TL − Ta| d³)^(1/4) / d × P/(R Tk)`, D_H = 21.5e-6 m² s⁻¹
     (Leuning 1995 App. E, as coded in plantecophys [F]). Floor u at 0.1 m s⁻¹ [I].
   - radiative, per face: `g_r = 4 ε_l σ Tk³ / cp` (≈ 0.17 mol m⁻² s⁻¹ at 25 °C) [F
     plantecophys `Gradiation`].
   - water vapour, boundary layer: `g_bw = 1.075 g_bh` per face (Leuning 1995 [F via
     plantecophys]; Helios uses 1.08 [F]); for amphistomatous leaves (bean, wheat) two faces in
     parallel, each in series with half of gs; for a one-line version
     `g_v = gs g_bw,tot / (gs + g_bw,tot)` with `g_bw,tot = n_faces × g_bw` [I; Helios
     `moistureConductance` does the per-face split exactly [F]].
5. Closed form (linearised Penman–Monteith; equivalent to Campbell & Norman 1998 eq. 14.6
   [K, to verify] and to the plantecophys/Leuning isothermal form [F]):
   ```
   ΔT = (Rni − λ g_v D_air / P) / (2 cp (g_bh + g_r) + λ g_v s(Ta) / P)
   TL = Ta + ΔT
   ```
6. One Newton step on the full nonlinear residual (removes the linearisation error):
   ```
   F(TL)  = S_abs + L_abs − 2 ε_l σ TLk⁴ − 2 cp g_bh (TL − Ta) − λ g_v (es(TL) − ea)/P
   F'(TL) = −8 ε_l σ TLk³ − 2 cp g_bh − λ g_v s(TL)/P
   TL ← TL − F/F'
   ```
   [I, proto]: closed form alone errs by up to 0.3 K (gs → 0, +11 K leaf); one Newton step
   brings it to < 0.001 K. Bonan's `LeafTemperature.m` is exactly this Newton iteration run to
   1e-6 W m⁻² [F].
7. Transpiration (mol m⁻² s⁻¹): `E = g_v (es(TL) − ea) / P`; λE closes the balance.
8. Stomata see the **leaf-to-air** VPD: `D_leaf = es(TL) − ea` goes into `LeafEnv.vpd` (its
   doc already says "leaf-to-air" [F gas.rs]). Medlyn et al. 2011 define D at the leaf surface
   [K, to verify]; Bonan uses the leaf-surface VPD `es(TL) − e_s` with e_s from g_bw and gs [F];
   plantecophys PhotosynEB passes the *air* VPD to gs (a simplification) [F].

### Coupling at a node (gs depends on TL through kinetics and D_leaf)

```
fn node(par, cap, env_ref) -> (A, gs, TL, gv):
    g(T) := T_EB(gs(T)) − T            # T_EB = closed form + 1 Newton (steps 5–6)
    a = Ta;  ga = g(a)                 # pass 1 = GECROS first round
    b = a + ga; gb = g(b)              # pass 2 = second fixed-point round
    while ga*gb > 0 and evals < N:     # extend by clamped secant until bracketed
        s = clamp(secant(a,b), Ta−20, Ta+25); a,ga = b,gb; b = s; gb = g(b)
    while evals < N and |b − a| > 1e-3 K:   # Illinois (modified regula falsi)
        c = (a gb − b ga)/(gb − ga); gc = g(c)
        if gc*gb < 0 { a,ga = b,gb } else { ga *= 0.5 }
        b,gb = c,gc
    TL = endpoint with smaller |g|; final gas solve at TL → A, gs; gv from gs and g_bh(TL)
    N = 8 evaluations max
```
- PAR = 0 nodes skip the loop: the dark branch of `gas::solve` fixes gs = g0 [F], so TL is
  the closed form + Newton directly [I].
- Each evaluation is one `gas::solve_with` at T with new `TemperatureTerms` (the per-table
  temperature cache no longer applies across nodes, since each node has its own TL) [I].
- Accuracy over 2,304 cases (Ta 0–46 °C, RH 10–95%, clear/overcast, u 0.05–5 m s⁻¹, PAR
  0–2000, stress 0–1) against the nested-bisection reference [I, proto]:

  | scheme | max error | p99 | p95 |
  | --- | --- | --- | --- |
  | fixed-point, 2 passes | 4.0 K | 1.5 K | 0.39 K |
  | fixed-point, 3 passes (≈ WRF-GECROS's 4 rounds) | 3.2 K | 0.56 K | 0.058 K |
  | fixed-point, 4 passes | 2.5 K | 0.23 K | 0.009 K |
  | fixed-point, 6 passes | 1.5 K | 0.033 K | 0.003 K |
  | Illinois, 6 evaluations | 0.12 K | 0.012 K | 0.002 K |
  | Illinois, 8 evaluations | 0.031 K | 0.006 K | 0.002 K |

  Fixed-point fails in the heat, at low wind: above the Vcmax/Jmax optimum a warmer leaf has
  lower A → lower gs → warmer still (gain near 1); one case (42 °C, RH 40%, u 0.05) had three
  roots of g(T) [I, proto]. The bracketed Illinois scheme is robust there; reference
  implementations use bracketed root finders too (plantecophys `optimize` then `uniroot` on
  [Ta−15, Ta+15] [F]; Bonan secant + Brent, 40 iterations, tol 0.1 K [F]; Helios secant, 100
  iterations [F]).
- Deterministic: f64, libm `exp`/`pow`, fixed evaluation cap, a convergence exit that depends
  only on the inputs. A fixed count with no early exit (always 8) is also fine and gives
  constant cost.

### Per leaf part (after the table lookup in `water_demand_with`)

```
for each leaf part (sunlit f, shaded 1−f):
    r = table.lookup(par, cap)                      # A, gs, TL_node (interpolated)
    gs_eff = gs × (sink-feedback ratio)             # code already scales E this way [F]
    u_leaf = u_top × τ_diffuse^(0.5 k_w / k_d)      # wind attenuation, see §3
    g_bh, g_v from u_leaf, d, gs_eff;  Rni from own S_abs, f_sky
    TL = closed form (5) + 1 Newton (6)            # ~2 exp per part
    E  = g_v (es(TL) − ea)/P                        # replaces table E for water uptake
    A  = table A                                    # evaluated at TL_node
    store TL (area-weighted over parts) on the leaf for step 8 and the inspector/lens
```

---

## 2. Findings

### 2.1 Code context (Plant-Sim)
- `gas::solve` takes `LeafEnv { par, t_leaf, vpd, ca, patm }`; `vpd` is documented as
  leaf-to-air; E is computed inside as `gs × vpd / patm`, with no boundary layer [F].
- `ClassTable::build` computes `TemperatureTerms` once for all 68 nodes because every node has
  the same leaf temperature [F]; with per-node TL this cache becomes per evaluation.
- `Plant::class_table` sets `t_leaf: wx.t_air, vpd: wx.vpd` [F]; `ps-world` shares tables
  between plants with equal `class_table_key()` = (leaf params bits, stress bits) [F].
- `water_demand_with` reads the table for sunlit and shaded PAR, mixes by sunlit fraction, and
  scales E by the sink-feedback ratio of gross A after the lookup [F]. So the per-leaf E
  already departs from the node solution; a per-leaf energy step makes that consistent [I].
- `HourWeather` has no wind field; the CSV reader accepts an optional `wind_m_s` column but it
  is not stored [F]. `cloud_fraction` is documented as "display only, never used in physiology"
  [F radiation.rs]; using it for sky longwave changes that contract. Daily disaggregation gives
  the day's Kt-based cloud fraction to every hour, including night [F hourly.rs].
- es in ps-weather is Tetens/FAO-56 `0.6108 exp(17.27 T/(T+237.3))` [F]. The same function
  must be used for ea (from RH) and for es(TL), so that D_leaf = D_air exactly when TL = Ta [I].
- Temperatures elsewhere: maintenance respiration uses `wx.t_air` for shoot organs and soil
  temperature for roots and seeds [F lib.rs ~1043]; plant thermal time uses air after
  emergence [F]; frost kills on the day's minimum *air* temperature below the frost threshold
  [F shoot.rs:629]; heat abortion of flowers is stated on daytime air temperature (§3.11) [F].
- `LightCoeffs.diffuse` = diffuse-sky transmittance to the leaf (1 in the open) [F
  ps-graph] — usable as sky view factor for longwave, and as a proxy for leaf area above the
  leaf (wind attenuation, NIR enrichment) [I].

### 2.2 plantecophys `PhotosynEB` (Duursma 2015), read from `R/LeafEnergyBalance.R`, v1.4-6 [F]
- Outer: `optimize` minimises `(FindTleaf(gs(Tleaf)) − Tleaf)²` over
  `[max(1, Tair−15), Tair+15]`; inner `FindTleaf` = `uniroot` of `LeafEnergyBalance` on
  `[Tair−15, Tair+15]` with gs fixed. On failure Tleaf = Tair and `failed = TRUE`.
- `gsfun` calls `Photosyn(Tleaf = x, VPD = VPD, ...)` with the **air** VPD (not converted to
  leaf), and the final `Photosyn` call also uses the air VPD; only the output column `VPDleaf`
  is converted with `VPDairToLeaf`.
- `LeafEnergyBalance` constants: Boltz 5.67e-8; Emissivity 0.95; CPAIR 1010 J kg⁻¹ K⁻¹;
  H2OLV0 2.501e6 J kg⁻¹; H2OMW 18e-3; AIRMA 29e-3 kg mol⁻¹; UMOLPERJ 4.57; DHEAT 21.5e-6
  m² s⁻¹. Air density `P·1000/(287.058 Tk)`; `LHV = (2.501e6 − 2365 Tair) × 0.018` J mol⁻¹;
  `SLOPE = (esat(Tair+0.1) − esat(Tair))/0.1` Pa K⁻¹ (finite difference);
  `Gradiation = 4 σ Tk³ ε / (cp Ma)`; `Gbhforced = 0.003 √(u/w) × CMOLAR` (CMOLAR = P/(R Tk));
  `Gbhfree = 0.5 DHEAT GRASHOF^0.25 / w × CMOLAR`, `GRASHOF = 1.6e8 |TL − Ta| w³`;
  `Gbh = 2 (free + forced)`; `Gbhr = Gbh + 2 Gradiation`; `Gbw = StomatalRatio × 1.075 Gbh`;
  `gw = gs Gbw/(gs + Gbw)`; `Rsol = 2 PPFD / 4.57` (shortwave = 2 × PAR energy);
  `Rnetiso = LeafAbs Rsol − (1 − ε_a) σ Tk⁴` with `ε_a = 0.642 (ea[Pa]/Tk)^(1/7)`;
  `ET = (SLOPE Rnetiso + VPD·1000 Gbh cp Ma)/(SLOPE + γ Gbhr/gw) / LHV`;
  `H = (1 + Gradiation/Gbh)⁻¹ (Rnetiso − λET)`; `Tleaf2 = Tair + H/(cp ρ Gbh/CMOLAR)`;
  balance = `Tleaf − Tleaf2`. Defaults in `PhotosynEB`: Wind 2 m s⁻¹, Wleaf 0.02 m,
  StomatalRatio 1, LeafAbs 0.86 (but `LeafEnergyBalance`'s own default is 0.5).
- E reported (`ELEAF`) is replaced by the Penman–Monteith `ELEAFeb` with gs and gb in series.
- esat (plantecophys, Jones 1992 / Buck): `f × 611.21 exp(17.502 T/(240.97 + T))` Pa,
  `f = 1.0007 + 3.46e-8 × P[Pa]` [F `R/RHtoVPD.R`].

### 2.3 Other implementations
- **Bonan (2019) supplementary programs 12.2** [F, `gbonan/bonanmodeling/sp_12_02`]:
  `TleafFunc(T)`: boundary layer at T → photosynthesis/stomata at T → `LeafTemperature`
  (Newton on the full balance, ≤ 100 iterations, |f| < 1e-6 W m⁻²) → returns new T − T;
  outer `hybrid_root` = secant from T±1 K switching to Brent once bracketed, itmax 40, tol 0.1 K.
  Balance: `qa − 2 ε σ T⁴ − 2 cp (T − Ta) gbh − λ/P (es(T) − ea) gleaf`, `gleaf` = gs and gbv in
  series; `qa` = absorbed VIS + NIR + ε (L_sky + L_ground). λ = (2501.6 − 2.3773 T) J g⁻¹ ×
  M_w. es and des/dT from Flatau et al. 1992 polynomials. Leaf ε 0.98; ρ = τ = 0.10 (VIS) and
  0.40 (NIR); dleaf 0.05 m. Boundary layer from Nusselt/Sherwood numbers (laminar 0.66
  Pr^0.33 Re^0.5, turbulent 0.036 Re^0.8, × 1.5 field enhancement; free 0.54 (Pr Gr)^0.25
  summed). Medlyn uses surface VPD and surface CO2 (cs).
- **CLM5 / CTSM `CanopyFluxesMod.F90`** [F]: Newton update of `t_veg` within the stability
  iteration, photosynthesis re-called every iteration; step clamped to `delmax = 1 K`; at least
  `itmin = 2`, at most `itmax_canopy_fluxes` iterations (namelist default 40, "fast" 3);
  converged when Δt < `dtmin = 0.01 K` and Δ(latent) < `dlemin = 0.1 W m⁻²`. Leaf boundary
  resistance `rb = 1/(cv √(uaf/dleaf))`. Leaf maintenance respiration is the photosynthesis
  `lmrsun/lmrsha` (at leaf temperature); stem/coarse-root MR uses 2 m air temperature
  (`t_ref2m`) [F `CNMRespMod.F90`]. Crop phenology reads `t_ref2m` [F `CNPhenologyMod.F90`
  references; heat-unit accumulation from 2 m air K, to verify].
- **GECROS** (Yin & van Laar 2005), WRF port `phys/module_sf_gecros.F` [F]: `PPHTR` does a
  first round at Tair, then repeats {Ci from VPD at TL, photosynthesis at TL, stomatal
  resistance, Penman–Monteith `PTRAN`, `DIFLA`} — 4 rounds in the WRF port (the original
  book code is 2 rounds [K, to verify]). `DIFLA`: `ΔT = clamp((Rn − λE)(rbh + rt)/1200, ±25)`
  (1200 J m⁻³ K⁻¹ volumetric heat capacity, λ 2.4e6 J kg⁻¹). `PTRAN` uses the **chord slope**
  `(es(TL) − es(Ta))/(TL − Ta)` after round 1, which makes the linear PM exact at the current
  TL. Net longwave loss `σ TLk⁴ (0.56 − 0.079 √(10 ea)) (0.1 + 0.9 clear) × FRAC` (Brunt-type,
  clearness `(ATMTR − 0.25)/0.45`). Leaf boundary conductance `0.01 √(u/w)` m s⁻¹; canopy
  integral with wind extinction `KW = KL` (leaf-light extinction coefficient). Separate
  sunlit/shaded classes. `TUNIT` docstring: "DIF: daytime plant-air temperature differential"
  (input to thermal units) — original GECROS adds canopy-air ΔT to daytime temperature for
  development [F doc string; the WRF port passes soil-air DIFS and does not use it in the body].
  Water stress: `APHTR` recomputes TL from *actual* (supply-limited) transpiration.
- **PlantBiophysics.jl `Monteith`** (MAESPA-like; Vezy) [F, `src/processes/energy/Monteith.jl`
  at 9ee4279]: start TL = Ta − 0.2; loop ≤ `maxiter = 10`, exit when |ΔT| ≤ 0.01 K; each
  iteration runs photosynthesis, recomputes net longwave with `sky_fraction` (0–2), gb (free +
  forced), cs, apparent γ*, λE (Monteith & Unsworth 2013 corrected by Schymanski & Or 2017
  eq. 22), and `TL_new = Ta + (Rn − λE)/(ρ cp a_sh / r_bh)`. ε = 0.955, a_sh = 2, a_sv = 1.
- **Helios energybalance** [F, `plugins/energybalance/src/EnergyBalanceModel.cu/.cpp`]:
  per-primitive secant on the full nonlinear balance (`max_iter 100`, relative tol 1e-4);
  gs is an input (no coupling inside); `gH = 0.135 √(U/L) × Nsides` (Pohlhausen, mol m⁻² s⁻¹);
  `cp_air_mol = 29.25`, `lambda_mol = 44000 J mol⁻¹`, `moisture_conductance_ratio = 1.08`;
  per-face moisture conductance with stomatal sidedness.
- **tealeaves** (Muir 2019) [F, PMC6899345]: `R_abs = α_s(1 + r) S_sw + α_l σ(T_sky⁴ + T_air⁴)`,
  `T_sky = T_air − 20 S_sw/1000`, emission `2 σ α_l TL⁴`; Nu mixed convection
  `Nu^3.5 = Nu_forced^3.5 + Nu_free^3.5`; `uniroot`; defaults d 0.1 m, α_s 0.5, α_l 0.97,
  u 2 m s⁻¹, `h_vap = 56847.68 − 43.12514 T[K]` J mol⁻¹. Notes plantecophys uses the
  Leuning isothermal approximation.

### 2.4 What the prototype says about magnitudes [I, proto]
Bean (d = 0.72 × 0.08 m, amphistomatous, PAR_abs 1500, clear, Ta 35 °C, RH 30%):

| u (m s⁻¹) | stress 1 | stress 0.3 | stress 0 (g0) |
| --- | --- | --- | --- |
| 0.2 | +1.2 K | +7.2 K | +7.7 K |
| 0.5 | +0.6 K | +6.0 K | +6.4 K |
| 1.0 | +0.2 K | +5.0 K | +5.4 K |
| 3.0 | −0.05 K | +3.5 K | +3.8 K |

- Status quo vs energy balance at u 0.5 m s⁻¹, well watered: A 17.7 → 17.1 µmol m⁻² s⁻¹
  (−3%), E 8.1 → 6.8 mmol m⁻² s⁻¹ (−16%); most of the E drop is the boundary layer in series,
  not the temperature.
- Clear night, Ta 2 °C, RH 90%: open leaf −2.2 K (u 0.5) to −3.1 K (u 0.1); half sky view
  −1.1 to −1.6 K; interior 0 K. Radiation frost at air +2 °C becomes possible.
- Ta 30 °C, RH 50%, u 1: sky view 1 → 0 shifts TL by about +1.1 K (PAR 1500) to +1.4 K
  (PAR 50); r_nir 0.15 → 0.5 shifts TL by +1.7 K at PAR 1500 but < 0.4 K at PAR ≤ 300;
  A changes −4% to −6%, E +5% to +8% across these ranges.

---

## 3. Constants table

| Quantity | Value | Unit | Source |
| --- | --- | --- | --- |
| σ Stefan–Boltzmann | 5.670374419e-8 | W m⁻² K⁻⁴ | CODATA 2018 exact [K]; codes use 5.67e-8 [F] |
| ε_l leaf emissivity | 0.97 (0.95–0.98) | 1 | plantecophys 0.95, Bonan 0.98, PlantBiophysics 0.955, tealeaves 0.97 [F] |
| cp molar, moist air | 29.3 | J mol⁻¹ K⁻¹ | C&N 1998 [K]; plantecophys 1010 J kg⁻¹ K⁻¹ × 0.029 = 29.29 [F]; Helios 29.25 [F] |
| λ(T) | (2.501e6 − 2361 T[°C]) × 0.018015 | J mol⁻¹ (44.0 kJ at 25 °C) | plantecophys slope 2365, Bonan 2377 (J kg⁻¹ K⁻¹) [F] |
| M_w | 0.018015 | kg mol⁻¹ | [K]; plantecophys 0.018, Bonan 0.01802 [F] |
| γ = cp P / λ | 0.0675 at 101.3 kPa, 25 °C | kPa K⁻¹ | derived [I]; GECROS 0.067 [F] |
| es(T) | 0.6108 exp(17.27 T/(T + 237.3)) | kPa | repo (FAO-56 Tetens) [F]; keep one function for ea and es(TL) |
| s(T) | 4098.2 es(T)/(T + 237.3)² (= 17.27 × 237.3) | kPa K⁻¹ | analytic derivative of the above [I] |
| alt. es (Buck) | 0.61121 exp(17.502 T/(T + 240.97)) × f | kPa | plantecophys, f = 1.0007 + 3.46e-8 P[Pa] [F] |
| molar air density | P/(R Tk) ≈ 40.9 at 25 °C | mol m⁻³ | [F plantecophys CMOLAR] |
| R | 8.314 | J mol⁻¹ K⁻¹ | repo `R_GAS` [F] |
| forced g_bh per face | 0.135 √(u/d) | mol m⁻² s⁻¹ | C&N eq. 7.30 [K, to verify]; Helios same [F] |
| d | 0.72 × leaf width | m | C&N [K, to verify] |
| free-convection | 0.5 D_H (1.6e8 ΔT d³)^¼ / d × P/(R Tk), D_H 21.5e-6 m² s⁻¹ | mol m⁻² s⁻¹ | Leuning 1995 App. E [F plantecophys] |
| g_bw / g_bh | 1.075 (1.08) | 1 | Leuning 1995 [F plantecophys]; Helios 1.08 [F] |
| g_r per face | 4 ε σ Tk³ / cp | mol m⁻² s⁻¹ | [F plantecophys] |
| ε_sky clear | 1.72 (ea[kPa]/Tk)^(1/7) | 1 | Brutsaert 1975 [K]; = plantecophys 0.642 (ea[Pa]/Tk)^(1/7) [F] |
| alt. ε_sky clear | 9.2e-6 Tk² | 1 | Swinbank 1963 (C&N eq. 10.11) [K, to verify] |
| cloud correction | (1 − 0.84 c) ε_clear + 0.84 c | 1 | Unsworth & Monteith 1975; C&N eq. 10.12 [K, to verify] |
| PAR energy fraction | 0.5 | 1 | repo `PAR_FRACTION` [F] |
| µmol per J PAR | 4.57 | µmol J⁻¹ | repo `UMOL_PER_J_PAR` [F] |
| a_par / a_nir | 0.85 / 0.20 | 1 | species file [F] / Bonan ρ = τ = 0.4 NIR [F] |
| wind default when missing | 2 | m s⁻¹ at 2 m | FAO-56 recommended stand-in [K, to verify]; plantecophys default 2 [F] |
| wind floor | 0.1 | m s⁻¹ | [I]; free convection covers still air |
| Leaf width, bean leaflet | ~0.06–0.10 | m | "~" [K, to verify] |
| Leaf width, wheat blade | ~0.01–0.015 | m | "~" [K, to verify] |

---

## 4. Design options and recommendation

### 4.1 Where to solve: per node vs per leaf
| Option | Cost per plant-hour | Error | Notes |
| --- | --- | --- | --- |
| A. Per node, plant-level environment only | 68 nodes × ~4–8 gas solves (night: closed form only) | per-leaf TL ±1–1.5 K from sky view and NIR mix; A ±4–6%, E ±5–8% per leaf, mostly cancelling in the plant total (first-order linear in the deviations) [I, proto] | Table key gains wind, leaf size *and* plant sky view, so plants in a plot stop sharing tables unless quantised [I] |
| B. Per leaf, full coupled solve | leaves × parts × ~5 gas solves | reference | 300-leaf plant → ~3,000 gas solves per hour vs 68 today: too costly for the M2 speed budget (STATUS: 20 ms per sim-day target) [I] |
| C. **Node coupled at a reference environment + per-leaf closed-form energy step (recommended)** | Option A cost + ~2 `exp` per leaf part | TL and E per leaf exact to < 0.01 K for the gs it reads; A carries the node's temperature (error ≈ (∂A/∂T) × (TL − TL_node), ~1–5% per leaf, smaller in the total) [I] | Tables stay shareable (reference env = species, hour, wind); sink-feedback-scaled gs gets a consistent TL and E; night and PAR 0 need no coupling; per-leaf TL feeds lenses, frost and respiration |
| D. Table with a temperature axis (e.g. 4 TL nodes per PAR × capacity) and per-leaf root-find on the interpolated gs(T) | 4× today's table | good except near the high-temperature feedback | More code; still needs a per-leaf 1-D solve [I] |

Reference environment for C [I]: sky view f_ref = the plant's (or species-in-plot)
area-weighted mean of `LightCoeffs.diffuse`, quantised to 0.1 if tables must be shared, or a
fixed 0.5; wind = the hour's wind at the canopy top; r_nir from the hour's beam/diffuse mix
(beam: ~0.24; diffuse open sky slightly lower since diffuse is PAR-rich [K, to verify]).
Sunlit and shaded parts keep reading one table, as today.

### 4.2 Iteration scheme
- Node coupling: bracketed Illinois on g(T) = T_EB(gs(T)) − T, starting from the two
  GECROS-like fixed-point rounds, cap 8 evaluations, exit at |Δ| < 1e-3 K (or always 8). Max
  error 0.03 K over the sweep [I, proto].
- Inner energy balance given gs: closed form + exactly one Newton step (fixed count). Do not
  iterate the Newton step to a tolerance; one step is < 1e-3 K [I, proto].
- Reject: plain fixed-point with 2–4 rounds (GECROS style): fine for 95% of hours but errs 1.5–4 K
  in hot, still conditions where TL matters most [I, proto]. CLM's Newton with 1 K step clamp is
  fine too but needs ∂gs/∂T, which `gas::solve` does not expose [I].
- Debug assert (ledger step 13 style): per node, |F(TL)| with the final gs below e.g.
  1e-3 W m⁻² after the Newton step [I]. The water ledger is unaffected as long as the E used
  for demand is the E removed from soil.

### 4.3 What reads leaf temperature vs air temperature (recommendation)
| Consumer | Recommendation | Why |
| --- | --- | --- |
| Photosynthesis kinetics, Rd, gs (step 6) | Leaf T (node TL) | The point of WP27 |
| Transpiration E (steps 6–7) | Leaf T, leaf-to-air VPD, gs and g_bw in series | Energy-consistent; plantecophys, Bonan, CLM [F] |
| Leaf maintenance respiration (step 8) | Leaf T (hourly, per leaf) | CLM5 leaf MR at leaf temperature [F]; stems stay air, roots soil (as today) |
| Thermal time, phenology (step 11), cardinal temps | **Air** (unchanged) | Cardinal temperatures in the species files are calibrated on air temperature; CROPGRO, CERES, APSIM and CLM crop phenology use air (CLM `t_ref2m` [F], others [K, to verify]); GECROS adds the daytime canopy-air difference [F doc string] — offer as a later option |
| Leaf expansion temperature multiplier | Air in v1; leaf T for dicot leaves is defensible later | Monocot leaf elongation follows meristem (near-apex/soil) temperature, not blade temperature (Ben Haj Salah & Tardieu 1995 [K, to verify]); wheat apex temperature before jointing tracks near-surface soil (Jamieson et al. 1995 [K, to verify]; Vinocur & Ritchie 2001 for maize [K, to verify]) |
| Frost kill (daily) | Option: minimum leaf TL of exposed leaves | Radiation frost (−2 to −3 K on clear nights [I, proto]); changes frost validation, needs owner decision |
| Flower heat abortion | Air (unchanged) | Thresholds (≈ 32 °C) are from air-temperature studies [K, to verify] |

---

## 5. Inferences [I]
- Transpiration falls in most conditions (boundary layer in series; well-watered leaves at
  high VPD run near or below air temperature), and rises for hot, stressed leaves in still
  air. Expect daily water use to drop by roughly 5–20% at low wind, so FTSW stays higher longer
  and drought responses shift; re-check the drought and water-shortfall validation targets.
- Supply shortfall (step 7 logs it, no second penalty) means actual λE < computed λE, so the
  leaf would be warmer than computed. GECROS `APHTR` re-solves TL from actual transpiration [F];
  Plant-Sim can list it as a known limit, since the stress factor already throttles gs.
- The table interpolates TL linearly in PAR between nodes, as it does A and E; TL is smooth in
  PAR, so the existing node spacing should suffice (check with the table-vs-direct test).
- Including g_bc (boundary layer for CO2, g_bw/1.37 [K, to verify]) in the CO2 supply path would
  cut A by ~1–3% at low wind; it breaks the plantecophys `Photosyn` 1e-6 fixture, so keep the
  core solver Photosyn-compatible and treat g_bc as a separate decision.
- Wind inside the plant: `u_leaf ≈ u_top exp(−0.5 k_w L_above)`; with L_above from the leaf's
  diffuse transmittance `τ_d = exp(−k_d L_above)` this is `u_top τ_d^(0.5 k_w/k_d)`, ≈ `u_top √τ_d`
  for k_w ≈ k_d (GECROS sets k_w = k_l [F]). Wind at the canopy top from the 2 m value by a log
  profile, or simply u_2m for small garden plants.
- A validation fixture: rerun the `photosyn.csv` grid style against plantecophys
  `PhotosynEB` is *not* a like-for-like target, because PhotosynEB uses air VPD for gs, Leuning
  isothermal Rn, SW = 2 × PAR and its own gb; a closer reference is Bonan's sp_12_02 or a direct
  nested-bisection solve.

## 6. Gaps
- Campbell & Norman 1998 eqs. 7.30, 10.10–10.12, 14.6 and Leuning et al. 1995 Appendix E were not
  re-read from the books/papers (paywalled); equation forms are from knowledge and from the
  plantecophys code that cites Leuning.
- The original GECROS (book code) round count (2?) and its TUNIT use of DIF not verified; only the
  WRF port was read.
- MAESPA Fortran source not fetched (repository path unknown, GitHub API blocked); MAESPA
  behaviour is taken from PlantBiophysics.jl, which says it is "close to MAESPA".
- CROPGRO/CERES/APSIM thermal-time temperatures are from knowledge; DSSAT's ETPHOT hourly
  energy-balance option (Pickering et al. 1995) not checked.
- Literature range of leaf–air ΔT on a hot still day (the WP27 acceptance criterion) not gathered
  here; the prototype gives +0.2 to +1.2 K well watered and +5 to +8 K stomata-closed at
  Ta 35 °C, u 0.2–1 m s⁻¹, bean-sized leaflets.
- Night-time cloud fraction for hourly CSV weather (Kt undefined at night) needs a rule (carry
  the last daytime value).
- Bean and wheat leaf widths, NIR absorptance, and stomatal sidedness are "~" values to source.

## References
- Brutsaert W (1975) On a derivable formula for long-wave radiation from clear skies. Water
  Resour Res 11:742–744. https://doi.org/10.1029/WR011i005p00742
- Bonan G (2019) Climate Change and Terrestrial Ecosystem Modeling. Cambridge Univ Press.
  https://doi.org/10.1017/9781107339217 ; code https://github.com/gbonan/bonanmodeling (sp_12_02)
- Campbell GS, Norman JM (1998) An Introduction to Environmental Biophysics, 2nd ed. Springer.
  https://doi.org/10.1007/978-1-4612-1626-1
- Duursma RA (2015) Plantecophys. PLoS ONE 10:e0143346. https://doi.org/10.1371/journal.pone.0143346 ;
  code https://raw.githubusercontent.com/RemkoDuursma/plantecophys/master/R/LeafEnergyBalance.R
- Duursma RA, Medlyn BE (2012) MAESPA. Geosci Model Dev 5:919–940. https://doi.org/10.5194/gmd-5-919-2012
- Jones HG (2014) Plants and Microclimate, 3rd ed. Cambridge Univ Press. https://doi.org/10.1017/CBO9780511845727
- Lawrence DM et al. (2019) The Community Land Model version 5. JAMES 11:4245–4287.
  https://doi.org/10.1029/2018MS001583 ; code https://github.com/ESCOMP/CTSM (src/biogeophys/CanopyFluxesMod.F90)
- Leuning R, Kelliher FM, de Pury DGG, Schulze E-D (1995) Leaf nitrogen, photosynthesis,
  conductance and transpiration: scaling from leaves to canopies. Plant Cell Environ 18:1183–1200.
  https://doi.org/10.1111/j.1365-3040.1995.tb00628.x
- Medlyn BE et al. (2011) Reconciling the optimal and empirical approaches to modelling stomatal
  conductance. Glob Change Biol 17:2134–2144. https://doi.org/10.1111/j.1365-2486.2010.02375.x
- Monteith JL, Unsworth MH (2013) Principles of Environmental Physics, 4th ed. Academic Press.
- Muir CD (2019) tealeaves. AoB Plants 11:plz054. https://doi.org/10.1093/aobpla/plz054
- Schymanski SJ, Or D (2017) Leaf-scale experiments reveal an important omission in the
  Penman–Monteith equation. HESS 21:685–706. https://doi.org/10.5194/hess-21-685-2017
- Swinbank WC (1963) Long-wave radiation from clear skies. QJRMS 89:339–348. https://doi.org/10.1002/qj.49708938105
- Unsworth MH, Monteith JL (1975) Long-wave radiation at the ground I. QJRMS 101:13–24. https://doi.org/10.1002/qj.49710142703
- Vezy R et al. (2018) Measuring and modelling energy partitioning in canopies of varying
  complexity using MAESPA. Agric For Meteorol 253–254:203–217. https://doi.org/10.1016/j.agrformet.2018.02.005 ;
  PlantBiophysics.jl https://github.com/VEZY/PlantBiophysics.jl (src/processes/energy/Monteith.jl @ 9ee4279)
- Yin X, van Laar HH (2005) Crop Systems Dynamics (GECROS). Wageningen Academic.
  https://doi.org/10.3920/978-90-8686-539-0 ; WRF port https://github.com/wrf-model/WRF (phys/module_sf_gecros.F)
- Bailey BN (2019) Helios. Front Plant Sci 10:1185. https://doi.org/10.3389/fpls.2019.01185 ;
  code https://github.com/PlantSimulationLab/Helios (plugins/energybalance)
- Goudriaan J (1977) Crop Micrometeorology: a Simulation Study. Pudoc. [K, to verify]
- Spitters CJT (1986) Separating the diffuse and direct component of global radiation and its
  implications for modeling canopy photosynthesis II. Agric For Meteorol 38:231–242.
  https://doi.org/10.1016/0168-1923(86)90061-4 [K, to verify]
- Ben Haj Salah H, Tardieu F (1995) Temperature affects expansion rate of maize leaves without
  change in spatial distribution of cell length. Plant Physiol 109:861–870. https://doi.org/10.1104/pp.109.3.861 [K, to verify]
- Jamieson PD, Brooking IR, Porter JR, Wilson DR (1995) Prediction of leaf appearance in wheat:
  a question of temperature. Field Crops Res 41:35–44. [K, to verify]
- Vinocur MG, Ritchie JT (2001) Maize leaf development biases caused by air–apex temperature
  differences. Agron J 93:767–772. https://doi.org/10.2134/agronj2001.934767x [K, to verify]
- Allen RG et al. (1998) FAO Irrigation and Drainage Paper 56. [K, to verify]
