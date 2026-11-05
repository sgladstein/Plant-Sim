# WP27 research notes: leaf boundary-layer conductance and the wind leaves see

Tags: [F] = I read the source (code or abstract). [K, to verify] = from my own knowledge, not re-read here. [I] = my inference or arithmetic.
Date: 2026-11-05.

---

## Takeaway (recommended for Plant-Sim)

**Forced convection, per leaf side (heat), mol m⁻² s⁻¹:**

    g_bH,forced,side = E · 0.135 · sqrt(u_leaf / d)          (u in m/s, d in m)
    E = 1.4 (field enhancement for turbulence, Campbell & Norman 1998)
    d = 0.72 · w  (w = maximum leaf width across the wind)

- The coefficient 0.135 is Pohlhausen's laminar flat-plate result, Nu = 0.664 Re^½ Pr^⅓, at about 20 °C and 101.3 kPa [F, Helios code comment; I, re-derived: 0.664·D_H·Pr^⅓/√ν = 3.28 mm s⁻¹ ≈ 0.136 mol m⁻² s⁻¹ per side at c = 41.6 mol m⁻³].
- To scale with temperature and pressure, use the m s⁻¹ form times c_air = P/(R·T): `g = E · 0.00328 · sqrt(u/d) · P/(R T)`. The coefficient's temperature dependence through D_H and ν is weak. This is how plantecophys and MAESTRA (CMOLAR) do it [F; I].

**Free convection, per side (heat):** `g_bH,free,side = 0.05 · (|T_leaf − T_air| / d)^¼` mol m⁻² s⁻¹ [K, to verify; Campbell & Norman 1998 eq. 7.33]. Equivalently, plantecophys uses `0.5 · D_H · Gr^¼ / w · c_air` with `Gr = 1.6e8 · |ΔT| · w³` [F].

**Mixed convection:** sum free and forced for each side, as plantecophys/Leuning 1995 do [F]. The cubic rule `Nu³ = Nu_forced³ + Nu_free³` (Helios InclinedPlate, after Chen et al. 1986) is the more correct option [F]. For a hand-solved hourly model, the sum is fine; it overestimates by at most ~20–40 % where Gr/Re² ≈ 1 [I].

**Both sides (heat):** `g_bH = 2 · g_bH,side` [F: plantecophys `Gbh <- 2*(Gbhfree+Gbhforced)`; Helios multiplies by Nsides]. Radiative conductance adds in parallel for the isothermal-net-radiation form.

**Vapour per side:** `g_bV,side = 1.08 · g_bH,side`.
- Helios uses 1.08 [F]. Leuning 1995 and plantecophys use 1.075 [F]. Campbell & Norman's 0.147/0.135 = 1.089 [K, to verify].
- The theory value is (D_V/D_H)^⅔ = (24.2/21.5)^0.67 ≈ 1.08 [I].
- For CO₂: g_bC = g_bV / 1.37 [K, to verify; standard ratio].

**Amphistomatous leaf:** the two faces act in parallel, and each face is stomata in series with that face's boundary layer [F, Helios `moistureConductance`]:

    g_tV = 1/(1/g_s,ab + 1/g_bV,side) + 1/(1/g_s,ad + 1/g_bV,side)
    g_s,ab = g_s · f_ab,   g_s,ad = g_s · (1 − f_ab)

- **Bean:** abaxial stomata are ~7× as numerous as adaxial [F, Kanemasu & Tanner 1969 abstract], so f_ab ≈ 7/8 ≈ 0.875 [I].
- **Wheat:** amphistomatous with roughly equal densities, so f_ab ≈ 0.5 [K, to verify; adaxial is often slightly higher in wheat].
- Do **not** use plantecophys's `Gbw = StomatalRatio·1.075·Gbh` shortcut with a series gs. It treats g_s as one conductance in series with a lumped boundary layer and gets the face partition wrong (see Inferences).

**Example values** (E = 1.4, per side, heat, mol m⁻² s⁻¹) [I]:

| leaf | w (m) | d (m) | u = 0.1 | 0.2 | 0.5 | 1.0 | 2.0 m/s | free, ΔT = 5 K |
|---|---|---|---|---|---|---|---|---|
| bean terminal leaflet | 0.08 | 0.058 | 0.25 | 0.35 | 0.56 | 0.79 | 1.11 | 0.15 |
| small bean leaflet | 0.05 | 0.036 | 0.32 | 0.45 | 0.70 | 1.00 | 1.41 | 0.17 |
| wheat blade | 0.015 | 0.011 | 0.58 | 0.81 | 1.29 | 1.82 | 2.57 | 0.23 |

At u ≤ 0.2 m s⁻¹ on a bean leaflet with ΔT ≈ 5 K, Gr/Re² is 0.26–1.0. That is the mixed regime, so free convection must be included [I].

**Wind at the leaf:**

- Get u at canopy top from the reference height with a log profile: d₀ = 0.67·h (Helios) or 0.77·h (DSSAT), z₀m = 0.1·h (Helios) or 0.13·h (DSSAT) [F].
- Inside the canopy: `u(z) = u_h · exp(a·(z/h − 1))`, with the leaf's height z [F, Cionco form in Berkeley notes and Helios].
- Attenuation coefficient a:
  - Wheat: a = 2.5 [F, Cionco 1972 via Baldocchi notes].
  - Bean: no direct value found. Use LAI-dependent `a = 4.39 − 3.97·exp(−0.258·LAI)` (McNaughton & van den Hurk 1995) [F, via Bigleaf.jl docs], which gives 1.3 at LAI 1, 2.0 at LAI 2, 2.6 at LAI 3.
  - Helios uses a = 0.5·LAI [F]. DSSAT uses a = 3 [F].
  - For a 1–3 m plot with isolated plants, the low-LAI end applies (a ≈ 1–2) [I].
- Floor u_leaf at **0.1 m s⁻¹**. This is the DSSAT minimum (Kimball changed it from 1.0 in 2014 because 1.0 was too high) [F]. CLM also floors um at 0.1 inside its iteration [F]. Free convection covers the rest.
- For a design default, the floor could also be 0.2 m s⁻¹ to stand in for gusts and convective "still" days [I, ~].

**Hourly wind from daily mean:** use the DSSAT HWIND scheme [F]:

    W_night = 0.3 · W_max
    W_max   = W_avg / (0.3 + 0.7 · DAYL/48)
    daytime:   W(t) = W_max·0.3 + 0.7·W_max/2 · (1 + sin(2π (t − t_rise)/(t_set − t_rise) + 1.5π))
    nighttime: W(t) = W_night

- The peak falls at solar noon, and the max/min ratio is 3.33.
- It conserves the daily mean exactly: the integral is W_max·(0.3·24 + 0.7·DAYL/2), divided by 24 [I].
- If no wind is given at all, use FAO-56's 2 m s⁻¹ at 2 m [F].

---

## Findings

### 1. Forced/free convection, enhancement, d, combination, vapour/heat ratio, amphistomaty

**Campbell & Norman (1998), *An Introduction to Environmental Biophysics*, 2nd ed., Springer, ch. 7 and 14.** DOI 10.1007/978-1-4612-1626-1. [K, to verify; book not re-read]
- Forced convection, heat: g_Ha = 1.4 × 0.135 √(u/d). Vapour: g_va = 1.4 × 0.147 √(u/d), in mol m⁻² s⁻¹.
- Free convection: g_Ha = 0.05 [(T_s − T_a)/d]^¼; vapour coefficient 0.055.
- d = 0.72 × leaf width for broad leaves. For narrow grass blades and needles, d ≈ width (0.72 for broad leaves).
- The 1.4 is an "outdoor turbulence" factor, the low end of the 1–2.5 range.

**Pohlhausen coefficient check.** The 0.135 per side is Pohlhausen's Nu = 0.664 Re^½ Pr^⅓ at 20 °C, 101.3 kPa [F, Helios BLConductanceModel.cpp comment]. My re-derivation gives 3.28 mm s⁻¹ × 41.6 mol m⁻³ = 0.136 [I].

**Jones (2014), *Plants and Microclimate*, 3rd ed., CUP.** [K, to verify] Gives g_aH ≈ 6.62 √(u/d) mm s⁻¹. This is ≈ 2 × 3.3 mm s⁻¹, so it is the two-sided value [I]. Jones also discusses the field factor of 1–2.5.

**Nobel (2009), *Physicochemical and Environmental Plant Physiology*.** [K, to verify] Boundary-layer thickness δ_bl (mm) = 4.0 √(l/v), where l is the mean length in the wind direction (m) and v is wind speed (m s⁻¹). Then g_bV = D_wv/δ_bl.

**Schuepp (1993), "Tansley Review No. 59: Leaf boundary layers," *New Phytologist* 125: 477–507.** DOI 10.1111/j.1469-8137.1993.tb03898.x.
- [F, abstract only] The review covers free and forced convection on leaves and models.
- [K, to verify] Field and real-leaf enhancement over the laminar flat plate is typically 1.0–2.5. Gr/Re² is the free/forced criterion: free convection is negligible below about 0.1 and dominant above about 10.

**Measured enhancement factors:**
- Parlange, Waggoner & Heichel (1971), *Plant Physiol.* 48: 437–442 [F, abstract]: "turbulence of a realistic wind decreases the resistance by a constant factor of about 2.5", flapping or not.
- Chamberlain (1974), "Mass transfer to bean leaves," *Boundary-Layer Meteorol.* 6: 477–486, DOI 10.1007/BF02137680 [F, abstract]: for leaves in a (bean) canopy, Sh was about 25 % above Pohlhausen. That is an enhancement of about 1.25 for bean in situ.
- Murphy & Knoerr (1977), *BLM* 11: 223–241 [F, abstract]: laminar theory is reasonable if multiplied by "a constant multiplier greater than one".
- Hunt, Impens & Lemon (1968), *Plant Physiol.* 43: 522 [F, abstract]: sunflower leaf boundary-layer resistance in the field was "significantly smaller than predicted".
- Dixon & Grace (1983), *Plant Cell Environ.* 6: 665 [F, abstract]: free convection is underpredicted for Gr < 10⁵. That covers most leaves in nature.

**Leuning et al. (1995), *Plant Cell Environ.* 18: 1183–1200** (two-leaf canopy model, DOI 10.1111/j.1365-3040.1995.tb00628.x). [K, to verify; equations confirmed only via plantecophys code that cites "Leuning 1995"]
- Forced convection: g_bH,forced = 0.003 √(u/w) m s⁻¹ per side.
- Free convection: g_bH,free = 0.5 D_H Gr^¼ / w, with D_H = 21.5e-6 m² s⁻¹.
- Combination: g_bH = 2 (free + forced). Vapour: g_bW = 1.075 g_bH.
- Note that w is the leaf width, not 0.72·w, and there is no 1.4 factor. 0.003 × 41.6 = 0.125 mol m⁻² s⁻¹ per side, close to the bare Pohlhausen value [I].

**Helios, BLConductanceModel InclinedPlate (Chen et al. 1986 mixed convection)** [F]:
- Pr = 0.7, ν = 1.568e-5 m² s⁻¹. Forced Nu = 2·F1·Re^½ with F1 = 0.339·Pr^⅓·(1 + (0.0468/Pr)^⅔)^−¼ (Churchill & Ozoe 1973).
- Mixed convection: Nu = Nu_f·(1 ± X³)^⅓, where X = free/forced. The rule is "assisting" for vertical-ish plates.
- For near-horizontal plates (inclination ≥ 75°), the upper face of a heated leaf gets the unstable value and the lower face the stable one. Free-convection Nu is 0.54 Gr^¼ (unstable) and 0.27 Gr^¼ (stable) (McAdams 1954). Both faces are scaled by molar density 41.4 mol m⁻³.

**Vapour vs heat ratio.**
- Helios uses `moisture_conductance_ratio = 1.08` [F]. Leuning and plantecophys use 1.075 [F].
- Lewis-number basis: (D_V/D_H)^⅔ = (24.2e-6/21.5e-6)^0.67 = 1.083 for laminar flow [I]. For free convection the exponent is about ¼ to ⅓, giving ≈ 1.03–1.04 [I].
- Bigleaf.jl generalises this as Gb_x = Gb_h/(Sc_x/Pr)^0.67 [F].

**One-sided vs two-sided [F, Helios energy balance code].**
- Heat: g_H = 0.135√(U/L)·Nsides.
- Vapour per face: g_bv,face = 1.08·g_H/Nsides.
- Total vapour: g_M = Σ over faces of g_bv,face·g_s,face/(g_bv,face + g_s,face), where g_s,front = g_s·stomatal_sidedness and g_s,back = g_s·(1 − stomatal_sidedness).

**Bean stomatal ratio.**
- Kanemasu & Tanner (1969), "Stomatal diffusion resistance of snap beans. I," *Plant Physiol.* 44: 1547–1552, DOI 10.1104/pp.44.11.1547 [F, abstract]: abaxial stomata are "about 7 times more numerous than on the adaxial surface".
- The same group's paper II found that adaxial stomata need more light to open [F, abstract].
- Stenglein et al. (2004), *Aust. J. Bot.* 52: 73–80, DOI 10.1071/BT02083 [F, abstract]: the adaxial surface has a "lower density of bigger stomates".
- da Silva et al. (1988), *Pesq. Agropec. Bras.* 23 [F, abstract]: abaxial density is higher than adaxial.
- [K, to verify] Typical counts are ~200–300 mm⁻² abaxial and ~30–60 mm⁻² adaxial, which gives a range of about 4–8.

### 2. Bean leaflets and wheat blades

**Bean leaflet dimensions** [K, to verify]: the terminal leaflet of a mature trifoliate is typically 8–14 cm long and 5–11 cm wide; the lateral leaflets are slightly smaller. Several leaf-area allometry papers measure the central leaflet's length and width but did not give means in the abstracts:
- Toebe et al. 2012, *Semina Ciênc. Agrár.*
- Suárez et al. 2022, *Agronomy* 12, 1504 leaves, 40 genotypes.
- Queiroga et al. 2003, *Hortic. Bras.*: leaf area AF = 0.1026·L^1.6871, with L = central leaflet width.

**Wheat blade** [K, to verify]: flag leaf width is about 1.2–2.2 cm and length 15–30 cm; lower leaves are about 0.8–1.5 cm wide. Spring wheat high-yield ideotype flag leaf L:W ≈ 12:1 (Yang, *J. Triticeae Crops*, abstract) [F].
- For grass blades the wind mostly crosses the blade, so d is about the width. Campbell & Norman's 0.72 factor is for broad leaves, but it is often applied to blades too [K, to verify].
- Haseba & Ito (1976), *J. Agric. Meteorol.*: a rice-leaf-shaped plate has a transfer coefficient 1.05× that of a rectangle [F, abstract].

**Measured gb, bean:**
- Chamberlain 1974 (above): about 1.25× Pohlhausen in the canopy [F].
- Thom (1968), *QJRMS* 94: 44–55, used an artificial bean leaf in a wind tunnel [K, to verify]. That paper is the basis of the r_V/r_H ≈ 0.93 (gV/gH ≈ 1.08) convention [K, to verify].

**Measured gb, wheat:** no specific measurement found in this pass (gap).

**Leaf flutter.**
- Parlange et al. 1971: flapping did not change the 2.5× factor [F].
- Haseba et al. (1986, 1987), *J. Agric. Meteorol.*: transfer coefficients for fluttering model leaves were "larger than those estimated from the laminar" theory [F, abstract].
- Schuepp (1972), *BLM*, fluttering realistic leaf models, Re 2e3–4e4 [F, abstract].
- In Plant-Sim, fold flutter into E; do not model it separately [I].

### 3. Wind in and above small canopies

**Exponential profile** u(z) = u_h exp[a(z/h − 1)] [F, Baldocchi ESPM 129 lecture notes, "Lecture 20 Wind and Turbulence part 1", nature.berkeley.edu/biometlab]. Attenuation coefficients from Cionco 1972b [F]:

| canopy | a |
|---|---|
| immature corn | 2.8 |
| oats | 2.8 |
| wheat | 2.5 |
| corn | 2.0 |
| sunflower | 1.3 |

No bean value was given. Soybean (Baldocchi et al. 1983) is listed without a number.

**LAI-dependent a** (McNaughton & van den Hurk 1995, *BLM* 74: 261–288) [F, via Bigleaf.jl docs]: a = 4.39 − 3.97 e^(−0.258 LAI). This gives a = 1.32 at LAI 1, 2.02 at LAI 2, 2.56 at LAI 3 and 2.98 at LAI 4 [I].

**Helios** [F]:
- Cionco profile with a = 0.5·LAI. Leaf height fraction is clamped to [0, 1].
- Canopy airspace: d = 0.67 h, z₀m = 0.1 h, z₀h = 0.01 h, with Businger–Dyer stability. Requires LAI ≥ 0.5.

**DSSAT ETPHR RESBLR** [F]:
- d = 0.77 h; z₀m = max(0.03, 0.13 h), ×4 for partial row cover; z₀h = z₀m/5.
- Wind at canopy top: u_h = max(u_ref·ln((h − d)/z₀m)/ln((z_ref − d)/z₀m), 0.1).
- The canopy-integrated leaf boundary-layer resistance follows Choudhury & Monteith (1988), with a (ETAW) = 3 and per-leaf conductance coefficient 0.01 m s^−½:

      r_b = a·√(w/u_h) / (2·0.01·LAI·(1 − e^(−a/2)))

  It is split into sunlit and shaded parts. The default leaf width is LWIDTH = 0.02 m (set in ETPHOT).

**CLM5** [F, CanopyFluxesMod.F90]:
- r_b = 1/(c_v·√(u_af/d_leaf)), where u_af = um·√(1/(ram1·um)), which equals u* [I].
- c_v = 0.01 m s^−½ is a parameter file value [K, to verify; the code only reads `cv`]. d_leaf is a PFT parameter (0.04 m for all PFTs [K, to verify]).
- The same r_b is used for heat and vapour, and per one-sided LAI.
- Wind floors: `ur = max(wind_min, |u|)` (wind_min from the params file; I believe 1.0 m s⁻¹ [K, to verify]), and um = max(ur, 0.1) plus a convective velocity w_c in unstable conditions [F].

**Log profile from 2 m to the canopy top:** u(h) = u(z)·ln((h − d)/z₀)/ln((z − d)/z₀). For h = 0.5 m (bush bean), d = 0.335, z₀ = 0.065: u(h)/u(2 m) = ln(0.165/0.065)/ln(1.665/0.065) = 0.93/3.24 ≈ 0.29 [I]. For a 1 m wheat canopy: ln(0.33/0.13)/ln(1.33/0.13) = 0.93/2.33 ≈ 0.40 [I].
- These ratios are smaller than many expect. For **isolated plants in a small 1–3 m plot surrounded by short ground**, the relevant roughness is the surroundings', not the plot's. Use u(h) ≈ u(2 m)·ln(h/z₀s)/ln(2/z₀s) with z₀s ≈ 0.01–0.03 m: for h = 0.5 that is about 0.6–0.75 of u2 [I].
- FAO-56 eq. 47 converts other heights to 2 m: u₂ = u_z · 4.87/ln(67.8 z − 5.42) [F, via worked example 0.75 at z = 10 m; formula K].

**Minimum wind:**
- DSSAT 0.1 m s⁻¹ (Kimball 2014: "1.0 m/s is too high a wind speed to be the minimum") [F].
- CLM um ≥ 0.1 [F].
- FAO-56 recommends u₂ ≥ 0.5 m s⁻¹ in the ETo equation "to account for buoyancy-driven exchange when air is calm" [F].
- Why have a floor at all [I]: anemometers stall at about 0.2–0.5 m s⁻¹; calm-hour air is still not still at the leaf (gusts, convective plumes, the leaf's own free convection); and laminar formulas with u → 0 send g_b → 0, which blows up leaf temperature. Free convection is what really sets the floor. With the summed free term in place, a 0.1 m s⁻¹ wind floor is enough.

### 4. Hourly wind from daily mean

**DSSAT HMET.for HWIND** (N.B. Pickering, 2000) [F]:
- The code comment says the night wind is 10 % of the maximum, but the code sets WNIGHT = 0.3.
- Daily input conversion: WINDAV = WINDSP/86.4 × (REFHT/WINDHT)^0.2, from km/d to m/s, with a power-law height correction of exponent 0.2.
- W_max = W_avg/(0.3 + 0.7·DAYL/48). Daytime is a sine bump from 0.3·W_max at sunrise to W_max at midday; nighttime is constant at 0.3·W_max.
- Max:min = 3.33. Example: DAYL 14 h and mean 2 m s⁻¹ give W_max 3.97 and night 1.19 m s⁻¹ [I].

**Guo (2013), arXiv:1305.3367** [F]:
- With daily mean and max: W_n = W_ave + (1/π)·W_max·cos((n − H_max)π/12).
- With mean only: W_n = W_ave + ½·W_ave·cos((n − H_max)π/12), which gives max:min = 3. Negative values are clipped to 0.
- Validated at Lubbock, TX (Nash–Sutcliffe 0.92 and 0.81 for wind power density).
- Published as Guo, Chang & Wang (2016), CCIS 569, Springer. The R package fruclimadapt's `hourly_windspeed` implements it, shifting the max to daytime [F].

**FAO-56** (Allen et al. 1998, Irrigation & Drainage Paper 56, ch. 3) [F]:
- "Where no wind data are available within the region, a value of 2 m/s can be used as a temporary estimate" (the mean of about 2000 stations).
- Classes: light ≤ 1, light-to-moderate 1–3, moderate-to-strong 3–5, strong ≥ 5 m s⁻¹.
- The FAO-56 chapter 3 page has no day/night table.
- The U_day/U_night ratio (default 2, choices 1–4) comes from FAO-24 (Doorenbos & Pruitt 1977) Penman adjustment factor c [K, to verify].

**WGEN/CLIGEN** [K, to verify]: WGEN generates no wind. CLIGEN generates daily wind speed and direction from monthly statistics, but has no hourly diurnal shape. WEPP's wind-erosion component (WEPS) uses an hourly cosine with a peak around 15 h and a max/min ratio input (default ≈ 2.0–3.0).

**Recommendation** [I]: use DSSAT's form, which is daylength-aware and conserves the mean. Make WNIGHT a parameter, 0.3, source DSSAT HWIND, "~". Optionally add deterministic hour-to-hour noise via ps-rng later.

### 5. How established models do it

**plantecophys** (Duursma 2015, *PLoS ONE* 10: e0143346, DOI 10.1371/journal.pone.0143346), R/LeafEnergyBalance.R [F, via rdrr.io source]:

LeafEnergyBalance defaults: Tleaf = 21.5, Tair = 20, gs = 0.15, PPFD = 1500, VPD = 2, Patm = 101, Wind = 2, Wleaf = 0.02, StomatalRatio = 1 ("2 for amphistomatous"), LeafAbs = 0.5.

Constants:

    Boltz = 5.67e-8; Emissivity = 0.95; DHEAT = 21.5e-6; CPAIR = 1010; AIRMA = 29e-3
    CMOLAR = Patm·1000/(8.314·Tk(Tair))
    Gradiation = 4·Boltz·Tk(Tair)³·Emissivity/(CPAIR·AIRMA)

Boundary layer:

    Gbhforced = 0.003·sqrt(Wind/Wleaf)·CMOLAR
    GRASHOF = 1.6e8·|Tleaf − Tair|·Wleaf³
    Gbhfree = 0.5·DHEAT·GRASHOF^0.25/Wleaf·CMOLAR
    Gbh = 2·(Gbhfree + Gbhforced)
    Gbw = StomatalRatio·1.075·Gbh   # Leuning 1995
    gw = gs·Gbw/(gs + Gbw)

Radiation and energy balance:

    Rsol = 2·PPFD/UMOLPERJ
    Rnet = LeafAbs·Rsol − Emissivity·Boltz·Tk(Tleaf)⁴
    Rnetiso = LeafAbs·Rsol − (1 − ema)·Boltz·Tk(Tair)⁴,  ema = 0.642·(ea/Tk(Tair))^(1/7)
    LHV = (H2OLV0 − 2.365e3·Tair)·H2OMW
    ET = (1/LHV)·(SLOPE·Rnetiso + 1000·VPD·Gbh·CPAIR·AIRMA)/(SLOPE + GAMMA·Gbhr/gw)
    H = −CPAIR·AIRDENS·(Gbh/CMOLAR)·(Tair − Tleaf)
    balance = Tleaf − Tleaf2   (root found with uniroot on [Tair − 15, Tair + 15])

Other functions:
- `FindTleaf(gs, Tair, ...)`.
- `PhotosynEB(Tair = 25, VPD = 1.5, Wind = 2, Wleaf = 0.02, StomatalRatio = 1, LeafAbs = 0.86, RH = NULL, ...)`. The 0.86 is a PAR absorptance; LeafEnergyBalance's own default is 0.5, for shortwave.

**CLM5 / CTSM** (github.com/ESCOMP/CTSM, src/biogeophys/CanopyFluxesMod.F90) [F]: see section 3. A bulk r_b = 1/(c_v √(u*/d_leaf)) with c_v = 0.01 m s^−½ [K]. There is no free-convection term and no enhancement factor beyond c_v.

**DSSAT CROPGRO ETPHOT/ETPHR** (github.com/DSSAT/dssat-csm-os, SPAM/ETPHR.for RESBLR, Weather/HMET.for) [F]:
- Uses hourly wind from HWIND, a wind floor of 0.1, and stability-corrected r_a (Choudhury et al. 1986).
- Canopy leaf boundary-layer resistance follows Choudhury & Monteith 1988 with a = 3 and w = LWIDTH = 0.02 m.
- Apparent bug: `RBSH = FRSHV * RBSL` overwrites the shaded-leaf resistance with the sunlit one [F; I that it is unintended].

**Helios** (github.com/PlantSimulationLab/Helios, plugins/energybalance and boundarylayerconductance) [F]:
- Default wind is 1 m/s.
- Heat: g_H = 0.135√(U/L)·Nsides, where L defaults to √(object area). This is not 0.72·width, and there is no 1.4 factor.
- Vapour per face: 1.08·g_H/Nsides. Faces combine in parallel, each with stomata in series with its boundary layer, using `stomatal_sidedness`.
- Constants: cp_air_mol = 29.25 J mol⁻¹ K⁻¹; λ = 44000 J mol⁻¹.
- Optional models: Pohlhausen, InclinedPlate (Chen 1986 mixed convection), Sphere (Nu = 2 + 0.6 Re^½ Pr^⅓), and Ground (Kustas & Norman 1999: (0.004 + 0.012 U)·41.56 mol m⁻² s⁻¹).
- Within-canopy wind is Cionco with a = 0.5·LAI.

**APSIM** [K, to verify]:
- APSIM Classic and NG use Penman–Monteith or Priestley–Taylor at canopy scale. They have no leaf energy balance and no leaf boundary layer in the standard wheat module.
- The MicroClimate module uses canopy aerodynamic conductance from a log profile with d and z₀ from height, plus a fixed canopy boundary-layer term.

---

## Parameter table

| name | value | unit | source | tag |
|---|---|---|---|---|
| forced coeff., heat, per side (laminar) | 0.135 | mol m⁻² s⁻¹ (m s⁻¹)^−½ m^½ | Campbell & Norman 1998; Helios | F (Helios) / K (C&N) |
| same in m/s form | 0.0033 (C&N/Pohlhausen); 0.003 (Leuning 1995) | m s^−½ | derivation; plantecophys | I; F |
| field enhancement E | 1.4 (range 1.0–2.5; bean canopy ≈ 1.25) | – | C&N 1998; Chamberlain 1974; Parlange 1971 | K; F; F |
| characteristic dimension d | 0.72 × max width | m | C&N 1998 | K, to verify |
| free coeff., heat, per side | 0.05 | mol m⁻² s⁻¹ K^−¼ m^¼ | C&N 1998 | K, to verify |
| free (plantecophys) | Nu = 0.5 Gr^¼, Gr = 1.6e8·ΔT·w³ | – | plantecophys/Leuning 1995 | F |
| D_H | 21.5e-6 | m² s⁻¹ | plantecophys | F |
| vapour/heat gb ratio | 1.08 (1.075) | – | Helios; Leuning 1995 | F |
| CO₂/vapour gb ratio | 1/1.37 | – | standard | K |
| bean abaxial:adaxial stomata | ≈ 7 (range ~4–8) | – | Kanemasu & Tanner 1969 | F (abstract) |
| bean f_ab (abaxial fraction of g_s) | 0.875 | – | derived | I |
| wheat f_ab | 0.5 | – | ~ | K, to verify |
| bean terminal leaflet width | 0.05–0.11 (default 0.08) | m | ~ | K, to verify |
| bean terminal leaflet length | 0.08–0.14 | m | ~ | K, to verify |
| wheat blade width (flag) | 0.012–0.022 (default 0.015) | m | ~ | K, to verify |
| wheat flag L:W | ≈ 12 | – | Yang (spring wheat) | F (abstract) |
| canopy attenuation a, wheat | 2.5 | – | Cionco 1972 | F (secondary) |
| a(LAI) | 4.39 − 3.97 e^(−0.258 LAI) | – | McNaughton & van den Hurk 1995 | F (secondary) |
| a, Helios | 0.5·LAI | – | Helios | F |
| a, DSSAT (ETAW) | 3.0 | – | DSSAT ETPHR | F |
| displacement d₀ | 0.67 h (Helios) / 0.77 h (DSSAT) | m | code | F |
| roughness z₀m | 0.1 h / 0.13 h | m | code | F |
| leaf wind floor | 0.1 | m s⁻¹ | DSSAT (Kimball 2014); CLM um | F |
| FAO-56 ETo wind floor | 0.5 | m s⁻¹ | FAO-56 | F |
| default wind when missing | 2.0 at 2 m | m s⁻¹ | FAO-56 | F |
| night/max wind ratio WNIGHT | 0.3 | – | DSSAT HWIND | F |
| plantecophys Wind default | 2 | m s⁻¹ | plantecophys | F |
| plantecophys Wleaf default | 0.02 | m | plantecophys | F |
| plantecophys StomatalRatio | 1 (2 = amphistomatous) | – | plantecophys | F |
| plantecophys LeafAbs | 0.86 (PhotosynEB); 0.5 (LeafEnergyBalance) | – | plantecophys | F |
| plantecophys emissivity | 0.95 | – | plantecophys | F |
| CLM c_v | 0.01 | m s^−½ | CLM5 params | K, to verify |
| CLM d_leaf | 0.04 | m | CLM5 params | K, to verify |
| DSSAT LWIDTH | 0.02 | m | DSSAT ETPHOT | F |
| Helios default wind | 1.0 | m s⁻¹ | Helios | F |

---

## Inferences

1. **Units and sidedness are the main trap.**
   - Helios and C&N's 0.135 are per side. plantecophys's 0.003·CMOLAR is per side, then ×2. Jones's 6.62 mm s⁻¹ is two-sided. CLM's c_v form is per one-sided leaf area (heat uses LAI+SAI, not 2·LAI).
   - A per-side constant gives the safest code: heat = sum over faces; vapour = parallel sum over faces of (stomata in series with boundary layer).
2. **plantecophys's `Gbw = StomatalRatio·1.075·Gbh` is not a per-face model.**
   - Gbh is already two-sided, so for a hypostomatous leaf (StomatalRatio = 1) Gbw counts both faces' boundary layers in series with gs. That is about 2× too high.
   - For StomatalRatio = 2 it is 4× the one-face value. The intent matches MAESTRA's convention but is dimensionally loose.
   - The error matters little when gs ≪ gb, which is the usual case.
   - Recommendation: do not copy it. Use the Helios-style per-face formula.
3. **Bean free convection.**
   - For bean (d ≈ 0.06 m), free convection matters when u_leaf ≲ 0.2 m s⁻¹, which is common inside a bean canopy on light-wind days.
   - Wheat's narrow blades are almost always forced-dominated.
4. **Small-plot wind.**
   - A 1–3 m plot cannot develop its own internal boundary layer (fetch ≪ 100·h). The above-canopy wind is set by the surroundings' roughness.
   - Exponential attenuation within the plants still applies, but with low a (≈ 1–2) at the plot edges.
   - A single, simple choice: u_h = u₂·ln(h/z₀s)/ln(2/z₀s) with z₀s = 0.02 m, then a = a(LAI_local).
5. **Mean-conserving hourly wind.** DSSAT HWIND conserves the daily mean exactly. A cosine with amplitude ½·mean (Guo eq. 2) also conserves the mean but needs clipping only if the amplitude exceeds the mean. Either is deterministic and needs no RNG.
6. **E = 1.4 vs 1.25.** C&N's E = 1.4 and Chamberlain's bean-canopy 1.25 are consistent within uncertainty. Use 1.4 with source C&N 1998 and note Chamberlain 1974 as support.
7. **Determinism (sim rules).** All of the above needs only sqrt, exp, ln, sin and pow(¼). These must come from libm per CLAUDE.md; plain arithmetic suffices otherwise.

---

## Gaps

- Did not read Campbell & Norman, Jones, Nobel or Schuepp full texts. Their exact equations, the 0.72 factor, the free-convection coefficient 0.05 and the stated E ranges are tagged [K, to verify].
- No direct measurement of g_b on wheat blades found. No bean-specific canopy attenuation a found (soybean Baldocchi et al. 1983 is the nearest; the value was not extracted).
- Bean stomatal counts (absolute mm⁻² for ad and ab) not extracted. Only the "~7×" ratio, from the Kanemasu & Tanner abstract.
- Bean leaflet and wheat blade dimensions are from memory. Suárez et al. 2022 (*Agronomy*, 40 genotypes) would give measured central leaflet length and width distributions.
- Leuning et al. 1995's exact equations were confirmed only through plantecophys code comments, not the paper.
- CLM5 parameter values (cv, dleaf, wind_min) not confirmed from the params netCDF.
- WEPS/CLIGEN hourly wind shape not verified. The FAO-24 U_day/U_night table is not verified.
- PMC (e.g. PMC12621100, a 2025 review "Beyond the boundary: a new road to improve photosynthesis via wind") was captcha-blocked. It would be a useful recent synthesis.
- GitHub API and CRAN tarballs were blocked by the proxy. Only raw.githubusercontent.com and rdrr.io (via WebFetch) worked; plantecophys's GitHub raw path for PhotosynEB.R returned 404 (the file is R/LeafEnergyBalance.R).

## Source URLs

- plantecophys source: https://rdrr.io/cran/plantecophys/src/R/LeafEnergyBalance.R
- CTSM: https://raw.githubusercontent.com/ESCOMP/CTSM/master/src/biogeophys/CanopyFluxesMod.F90
- Helios: https://raw.githubusercontent.com/PlantSimulationLab/Helios/master/plugins/energybalance/src/EnergyBalanceModel.cpp ; .../plugins/boundarylayerconductance/src/BoundaryLayerConductanceModel.cpp ; .../plugins/energybalance/include/EnergyBalanceModel.h
- DSSAT: https://raw.githubusercontent.com/DSSAT/dssat-csm-os/develop/SPAM/ETPHR.for ; .../SPAM/ETPHOT.for ; .../Weather/HMET.for
- Baldocchi lecture: https://nature.berkeley.edu/biometlab/espm129/notes/Lecture_20_Wind_and_Turbulence_part_1_Canopy_Air_space_Notes_2014.pdf
- Bigleaf.jl: https://docs.juliahub.com/General/Bigleaf/stable/boundary_layer_conductance/
- FAO-56 ch. 3: https://www.fao.org/4/x0490e/x0490e07.htm
- Guo 2013: https://arxiv.org/pdf/1305.3367
- Kanemasu & Tanner 1969: https://scispace.com/papers/stomatal-diffusion-resistance-of-snap-beans-i-influence-of-4121mseeft
- Chamberlain 1974: https://scispace.com/papers/mass-transfer-to-bean-leaves-53o8yuf3ks
- Parlange et al. 1971: https://scispace.com/papers/boundary-layer-resistance-and-temperature-distribution-on-18kv9ynmul
- Schuepp 1993: https://scispace.com/papers/tansley-review-no-59-leaf-boundary-layers-wmozgn6kr9
- Dixon & Grace 1983: https://scispace.com/papers/natural-convection-from-leaves-at-realistic-grashof-numbers-2bnxwa34kv
- Stenglein et al. 2004: https://www.publish.csiro.au/BT/BT02083
