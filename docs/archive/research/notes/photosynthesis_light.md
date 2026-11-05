# Leaf photosynthesis, stomatal conductance, respiration and canopy light: parameters for common bean, checked against design plan v3 (§3.2, 3.5, 3.8, 3.16, 8.1)

Source-status legend. **[fetched]** means I opened the source this session and the claim comes from it. **[known; not opened]** means a standard literature value I am confident of but did not reopen this session (the URL is the DOI landing page). The writer or implementer should spot-check those before encoding them. Live fetches of the plantecophys source code and the CRAN tarball were refused by the egress proxy (HTTP 403 policy denial), so details of the plantecophys internals come from its documentation page [fetched] plus my knowledge of its code [known; not opened].

---

## 1. FvCB model: Kc, Ko, Gamma* and their temperature dependence; Vcmax/Jmax for bean; TPU, Jmax/Vcmax, theta, alpha

### Takeaway
Use the Bernacchi et al. (2001) Ci-based constants (Kc 404.9 µmol mol⁻¹, Ko 278.4 mmol mol⁻¹, Γ* 42.75 µmol mol⁻¹ at 25 °C, Arrhenius). plantecophys uses exactly these, so the fixture can match. **Divergence to fix:** the plan's Vcmax 89.3 / Jmax 159.8 (Pimentel 2011) were fitted together with a finite mesophyll conductance (gm 0.47). They are chloroplast (Cc) based. Our model has no gm, so we must either add gm or use Ci-based "apparent" values. Otherwise A will be about 20% too high in the Rubisco-limited range (inference below). A Jmax/Vcmax of 1.79 is normal.

### Cited findings
- **Core equations** (FvCB): Ac = Vcmax(C − Γ*)/(C + Kc(1 + O/Ko)); Aj = J(C − Γ*)/(4C + 8Γ*) (some codes use 4.5C + 10.5Γ*); J from the non-rectangular hyperbola θJ² − (αI + Jmax)J + αI·Jmax = 0, taking the smaller root; An = min(Ac, Aj, Ap) − Rd. — [Farquhar, von Caemmerer & Berry 1980, Planta 149:78](https://doi.org/10.1007/BF00386231) [known; not opened]. The A/Ci analysis method was established on *Phaseolus vulgaris* by [von Caemmerer & Farquhar 1981, Planta 153:376](https://doi.org/10.1007/BF00384257) [known; not opened].
- **Bernacchi 2001 (in vivo, transgenic tobacco, Ci based, gm assumed infinite):** Kc25 = 404.9 µmol mol⁻¹, Ea 79.43 kJ mol⁻¹; Ko25 = 278.4 mmol mol⁻¹, Ea 36.38 kJ mol⁻¹. The same Kc/Ko Ea values, converted to Pa (Kc25 39.97 Pa, Ko25 27,480 Pa), are used in rpmodel, which cites Bernacchi et al. 2001 *PCE* 24:253–259 — [rpmodel calc_kmm docs](https://search.r-project.org/CRAN/refmans/rpmodel/html/calc_kmm.html) [fetched]. Γ*25 = 42.75 µmol mol⁻¹, Ea 37.83 kJ mol⁻¹; Vcmax Ea 65.33 kJ mol⁻¹ — [Bernacchi et al. 2001 PCE](https://doi.org/10.1111/j.1365-3040.2001.00668.x) [known; not opened]. Form: k(T) = k25·exp(Ea(Tk − 298.15)/(298.15·R·Tk)).
- **Bernacchi 2002 (Cc based, needs gm):** Γ* 37.43 µmol mol⁻¹ (Ea 24.46 kJ), Kc 272.38 µmol mol⁻¹ (Ea 80.99), Ko 165.82 mmol mol⁻¹ (Ea 23.72). gm itself is temperature dependent and peaks near 35–37 °C. These constants must only be paired with Cc-based Vcmax. — [Bernacchi et al. 2002 Plant Physiol 130:1992](https://doi.org/10.1104/pp.008250) [known; not opened].
- **von Caemmerer 2000** (book, Cc based): Kc 260 µbar, Ko 179 mbar, Γ* ≈ 38.6 µbar at 25 °C [known; not opened; *Biochemical Models of Leaf Photosynthesis*, CSIRO Publishing].
- **Medlyn et al. 2002** fit a peaked Arrhenius for Vcmax and Jmax: f(T) = k25·exp(Ea(Tk − 298)/(298RTk))·[1 + exp((298ΔS − Hd)/(298R))]/[1 + exp((TkΔS − Hd)/(RTk))], with Hd = 200 kJ mol⁻¹. It uses Bernacchi-type Kc, Ko and Γ*. — [Medlyn et al. 2002 PCE 25:1167](https://doi.org/10.1046/j.1365-3040.2002.00891.x) [known; not opened].
- **plantecophys Photosyn defaults:** alpha = 0.24, theta = 0.85, TPU = 1000 (effectively off), gmeso = NULL (infinite), Rd0 = 0.92 with Q10 = 1.92 and TrefR = 25, Rdayfrac = 1, EaV = 82620.87, **EdVC = 0** (no high-temperature deactivation of Vcmax), delsC = 645.1013, EaJ = 39676.89, EdVJ = 2e5, delsJ = 641.3615, g1 = 4, g0 = 0, default gsmodel "BBOpti" (Medlyn 2011). Km and Γ* come from a built-in temperature function, and temperature responses "follow Medlyn et al. (2002)". — [plantecophys Photosyn docs](https://rdrr.io/cran/plantecophys/man/Photosyn.html) [fetched]. The built-in Γ*/Km use the Bernacchi 2001 constants above with Oi = 210 mmol mol⁻¹, and Aj uses J/4·(Ci − Γ*)/(Ci + 2Γ*) — [plantecophys source, GitHub](https://github.com/RemkoDuursma/plantecophys) [known; not opened, fetch blocked].
- **Mesophyll conductance matters for parameter values:** assuming infinite gm can underestimate Vcmax by up to 75% and Jmax by up to 60%, and inflates the Jmax/Vcmax ratio — [Sun et al. 2014 PCE](https://consensus.app/papers/details/32d24ccd148159708f67f27c03644359/?utm_source=claude_desktop) [fetched abstract]. A/Ci parameterisations overestimated daily A by 6–8% at high gm and underestimated it by up to 70% at low gm — [Niinemets et al. 2009 JXB](https://consensus.app/papers/details/38f945fc5230538fabe26e6ad59ea23d/?utm_source=claude_desktop) [fetched abstract].
- **Bean gm can vary with Ci:** in bean (cv. Dark Red Kidney) grown at high PPFD, gm at Ci 300 was about 40% of its value at Ci 100. In soybean it was about 85%. Measured at 25 °C, PPFD 1500. — [Bunce 2010 Photosynthetica 48:507](https://ps.ueb.cas.cz/pdfs/phs/2010/04/04.pdf) [fetched]. The paper reports no absolute Vcmax, Jmax or Rd values.
- **Temperature acclimation (Kattge & Knorr 2007, 36 species):** ΔS_V = 668.39 − 1.07·Tgrowth and ΔS_J = 659.70 − 0.75·Tgrowth (J mol⁻¹ K⁻¹); Ha_V = 71.51 and Ha_J = 49.88 kJ mol⁻¹; Hd = 200 kJ mol⁻¹; Jmax/Vcmax at 25 °C = 2.59 − 0.035·Tgrowth (°C). CLM5 uses this scheme. — [Kattge & Knorr 2007 PCE 30:1176](https://doi.org/10.1111/j.1365-3040.2007.01690.x) [known; not opened]; [CLM5 technical note](https://escomp.github.io/ctsm-docs/versions/master/html/tech_note/) [known; not opened].
- **Quantum yield:** the C3 quantum yield for CO2 is about 0.052 mol CO2 per mol absorbed photon at 30 °C and normal O2/CO2, and it falls with temperature through photorespiration — [Ehleringer & Björkman 1977 Plant Physiol 59:86](https://doi.org/10.1104/pp.59.1.86) [known; not opened]. Electron-transport α on an absorbed basis is about 0.3 mol e⁻ per mol photon. The plantecophys α of 0.24 is on an **incident** basis (≈ 0.3 × ~0.8 absorptance) [known; not opened; implied by the plantecophys docs, where PPFD is incident].
- **TPU:** plantecophys disables it by default (TPU = 1000) [fetched]. When fitted, TPU is roughly Vcmax/6 to Vcmax/8 [known; not opened]. Infinite-gm fitting overestimates TPU by up to 40% — [Sun et al. 2014](https://consensus.app/papers/details/32d24ccd148159708f67f27c03644359/?utm_source=claude_desktop) [fetched abstract].

### Inferences
- **Size of the gm divergence:** at A ≈ 25 µmol m⁻² s⁻¹ with gm 0.47 mol m⁻² s⁻¹ bar⁻¹, the drawdown Ci − Cc ≈ 53 µmol mol⁻¹. With Ci 280 and the Bernacchi Km ≈ 405·(1 + 210/278.4) ≈ 710 µmol mol⁻¹, the Rubisco factor (C − Γ*)/(C + Km) is 0.240 at Ci versus 0.196 at Cc. Using the Cc-based Vcmax 89 against Ci therefore overstates Ac by about 20%. **Options:** (a) add gm (one more conductance in series, so the solve stays a quadratic or cubic), with Bernacchi 2002 Cc constants; or (b) refit or convert to Ci-based apparent values (Vcmax roughly 65–75, inference) with Bernacchi 2001 constants. Option (b) keeps the plantecophys oracle simple. The plan's "Vcmax 89, measured on common bean" line in §3.16 should say which basis it uses. Wullschleger's herbaceous mean of 75 (§8.2) is Ci based, which is consistent with option (b).
- **Recommended M0 set:** Bernacchi 2001 Kc, Ko and Γ* with Arrhenius; peaked Arrhenius for Jmax (Ha ~50 kJ, Hd 200 kJ, ΔS ~650); peaked Arrhenius for Vcmax as well (Ha ~65–72 kJ, Hd 200 kJ, ΔS ~645–650). plantecophys's default EdVC = 0 has no Vcmax deactivation, so at 35–40 °C it will diverge from any peaked function. For the 40 °C fixture, either set EdVC = 2e5 in the oracle or match its default.
- Jmax/Vcmax = 160/89 = 1.79 sits inside Kattge-Knorr's 2.59 − 0.035·Tg (1.82 at Tg = 22 °C). That is plausible, but it is a Cc-based ratio; Ci-based ratios are higher.
- θ = 0.7–0.9 (use 0.85 to match plantecophys). α = 0.3 on absorbed PAR (or 0.24 if incident PAR is fed in).
- TPU: at 800 ppm and low temperature, TPU could limit real bean leaves, but the plantecophys fixture has it off. Include a TPU term, but disable it, or set it high, in the fixture run.

### Gaps
- No bean-specific Kc/Ko or Vcmax temperature-response parameters were found; tobacco-derived (Bernacchi) values are standard practice.
- I could not open Pimentel 2011 to confirm cultivar, growth temperature, and which Kc/Ko/Γ* set was used in the fit (a Consensus search did not surface it).
- von Caemmerer & Farquhar 1981 Phaseolus values come in older units; not retrieved.

---

## 2. Bean temperature acclimation, Vcmax and leaf N, leaf N per area

### Takeaway
Bean-specific Vcmax-N slopes and acclimation data were not found within budget. Use Kattge & Knorr (2007) acclimation and a generic C3-crop linear Vcmax(Narea) for M2, calibrated so that Narea ≈ 1.5–2.5 g m⁻² gives Vcmax25 ≈ 70–90.

### Cited findings
- At planting densities of 20 plants m⁻² and above, bean leaf area and leaf N per area plateau (the plan's own §8.3 source) — [Clavijo Michelangeli 2019, as cited in the plan](https://consensus.app/papers/details/11b9dd4ab6fd51aa9d2da96ca8314961/?utm_source=claude_desktop) [not opened].
- Acclimation equations: see §1, Kattge & Knorr 2007 [known; not opened].

### Inferences
- From SLA 200–300 cm² g⁻¹ (LMA 33–50 g m⁻²) and a typical bean leaf N of 4–5% of dry mass, Narea ≈ 1.3–2.5 g N m⁻². Then Vcmax 89 implies roughly 35–65 µmol CO2 (g N)⁻¹ s⁻¹, within the usual C3-crop range. This is an inference, not a measured bean slope.

### Gaps
- No bean-specific Vcmax-Narea regression; no bean growth-temperature acclimation study retrieved.

---

## 3. Medlyn g1 and g0; Ball-Berry alternatives; where drought acts; bean stomatal data

### Takeaway
The plan's g1 ≈ 4 kPa^0.5 is low for crops. Lin et al. 2015 rank C3 crops and grasses second-highest of all functional types, and CLM5 uses about 5.8 for crops. Use 5–6 as the default and keep it a tunable parameter. For drought, scaling only g1 (or gs) is fine for M0, but under severe stress Zhou et al. 2013 found that Vcmax must also drop.

### Cited findings
- **Medlyn 2011 model:** gs = g0 + 1.6(1 + g1/√D)·A/Ca, with D in kPa and g1 in kPa^0.5 — [Medlyn et al. 2011 GCB 17:2134](https://doi.org/10.1111/j.1365-2486.2010.02375.x) [known; not opened]. plantecophys "BBOpti" is this model, with defaults g1 = 4, g0 = 0, and the alternatives BBLeuning (Leuning 1995, D0 = 5) and BallBerry (Ball et al. 1987) — [plantecophys docs](https://rdrr.io/cran/plantecophys/man/Photosyn.html) [fetched].
- **Lin et al. 2015 global synthesis:** the largest g1 is in evergreen savannah trees, followed by C3 crops and grasses, then other angiosperm trees, then gymnosperms; C4 is lowest. g1 rises with growth temperature (p = 0.009) and moisture index (p = 0.004). For angiosperm trees, g1 = −3.97·WD + 6.53. — [Lin et al. 2015 Nature Climate Change](https://www.k-state.edu/ecophyslab/pdf's/Lin et al. 2015 NCC.pdf) [fetched; the per-PFT numbers were in a figure and supplementary table I could not read].
- **CLM5 Medlyn parameters** (taken from Lin et al.): g1 ≈ 5.79 for C3 crops, 5.25 for C3 grass, 1.62 for C4 grass, 2.35 for needleleaf trees; g0 = 100 µmol m⁻² s⁻¹ — [CLM5 tech note](https://escomp.github.io/ctsm-docs/versions/master/html/tech_note/) [known; not opened. Verify the exact values in the CLM5 PFT parameter file].
- **Where drought acts:** Zhou et al. compared drought responses across species and found that reducing g1 alone could not capture severe drought; apparent Vcmax also has to fall. Their recommended form uses two separate functions of predawn water potential, one on g1 and one on Vcmax — [Zhou et al. 2013 AFM 182–183:204](https://doi.org/10.1016/j.agrformet.2013.05.009) [known; not opened]. Models differ widely in where the β factor is applied (g1, gs, or Vcmax), which drives large inter-model spread — [De Kauwe et al. 2013 GCB 19:1759](https://doi.org/10.1111/gcb.12164) [known; not opened].
- **Bean stomata and leaf movement:** in bean, the leaf incidence angle correlated with gs (R² = 0.54) and with A (R² = 0.84); leaf temperature stayed below air temperature at air temperatures of 33–42 °C — [Raeini-Sarjaz 2008 Int J Biometeorol](https://consensus.app/papers/details/6bc5f519510c5a2cb5694022d12e4d24/?utm_source=claude_desktop) [fetched abstract].

### Inferences
- With D = 1.5 kPa, gs(g1 = 4) / gs(g1 = 5.8) ≈ 4.27/5.73 ≈ 0.75 at equal A. The plan's value would underestimate transpiration by about 25%. That affects soil drawdown timing and the FTSW stress onset.
- **Implementation detail to pin down in the plan:** "stress factor used in stomatal conductance" is ambiguous. β·g1 keeps the "1" term, so gs never goes fully to g0; β applied to the whole A-dependent term lets gs fall to g0. Pick one and document it. Zhou-style scaling of g1 and Vcmax is the better-supported choice for M2.
- g0: use 0 to match the plantecophys default in fixtures, or a small value (0.01 mol m⁻² s⁻¹) for night-time conductance in the game. If g0 > 0 in the sim, the fixture must set it too.

### Gaps
- No published Medlyn g1 fitted specifically for *Phaseolus vulgaris* was found.

---

## 4. Numerical solution; pitfalls; plantecophys as test oracle

### Takeaway
Coupled FvCB-Medlyn has a closed-form quadratic per limiting rate when g0 = 0 and gm is infinite (a cubic in other cases). plantecophys solves it analytically. A Newton solver will converge to the same root, but only if every convention matches. plantecophys is a reasonable oracle, but the 2% tolerance is only reachable with identical constants, light basis, Rd handling and min-smoothing.

### Cited findings
- plantecophys "finds the intersection between the supply of CO2 by diffusion and the demand for CO2 by photosynthesis" and returns Ac, Aj and the hyperbolic minimum — [plantecophys docs](https://rdrr.io/cran/plantecophys/man/Photosyn.html) [fetched]. The method is described in [Duursma 2015 PLoS ONE 10:e0143346](https://doi.org/10.1371/journal.pone.0143346) [known; not opened].
- Analytical coupled solutions: cubic for the Ball-Berry/Leuning family — [Baldocchi 1994 Tree Physiol 14:1069](https://doi.org/10.1093/treephys/14.7-8-9.1069); [Leuning 1995 PCE 18:339](https://doi.org/10.1111/j.1365-3040.1995.tb00370.x) [known; not opened].
- plantecophys converts the CO2 conductance to H2O with a factor of 1.57 for transpiration, while the Medlyn equation uses 1.6. Transpiration is E = 1.57·gs·VPD/Patm. [known; not opened; source code].

### Inferences (pitfalls)
- **Light basis:** the plan's step 3 produces *absorbed* PAR per leaf, while the plantecophys PPFD is *incident* (α = 0.24 includes absorptance). In the fixture, either feed PPFD = absorbed/0.85 or set α = 0.24/0.85.
- **Rd in the light:** plantecophys uses Rday = Rd (Rdayfrac = 1) with Q10 = 1.92 referenced to 25 °C. The plan uses Rd 0.9 at 25 °C. Match these values for the fixture, even if the game later uses light inhibition (Rday ≈ 0.6–0.7 Rd, a common modelling choice).
- **Min function:** plantecophys takes a near-hard hyperbolic minimum between Ac and Aj. A different collatz θ (e.g. 0.98) changes A by about 1–3% near co-limitation, which by itself could break the 2% tolerance.
- **Negative A at low light:** when An < 0, the Medlyn formula gives gs < g0. Clamp gs to ≥ g0 and solve A with fixed gs. Newton can also jump to the non-physical root of the quadratic, so bracket it with Ci ∈ (Γ*, Ca] or use the analytic roots directly.
- **Determinism:** the analytic route (a quadratic per limitation using a numerically stable root formula, q = −½(b + sign(b)√disc)) is cheaper, deterministic and branch-stable. Newton converges to tolerance, so bit patterns depend on the iteration count. Recommend switching the plan from "Newton iteration" to the analytic solution, with Newton kept as a fallback for gm/TPU variants.
- **A second oracle** would help guard against shared-convention errors, e.g. the R package `photosynthesis` (Stinziano et al.) or a hand-coded Python reference (no source fetched).

### Gaps
- I could not read the plantecophys source this session (proxy 403), so the exact coefficient algebra is unverified.

---

## 5. Leaf temperature = air temperature in M0

### Takeaway
For well-watered bean this is acceptable as a first approximation. Field and greenhouse bean leaves are often *below* air temperature at high temperature, helped by paraheliotropism. The assumption overestimates heat effects under hot, humid conditions and underestimates leaf heating under drought, when stomata close.

### Cited findings
- Water-stressed field beans kept leaf temperature below ambient despite stomatal closure, thanks to paraheliotropism. Leaves physically restrained from moving warmed up and showed photoinhibition — [Pastenes et al. 2004 JXB](https://consensus.app/papers/details/37e96cd579305d2fa0b9a637272da058/?utm_source=claude_desktop) [fetched abstract]. Paraheliotropism "helps maintain leaf temperatures far below the ambient" — [Pastenes et al. 2004 J Plant Physiol](https://consensus.app/papers/details/48e7fdee49c05d898559db371380b845/?utm_source=claude_desktop) [fetched abstract].
- At air temperatures of 33–42 °C, bush-bean leaf temperature stayed below air temperature and tracked it (r = 0.92) — [Raeini-Sarjaz 2008](https://consensus.app/papers/details/6bc5f519510c5a2cb5694022d12e4d24/?utm_source=claude_desktop) [fetched abstract].

### Inferences
- The typical well-watered sunlit leaf-to-air difference is about ±1–3 °C. Under drought without leaf movement it can be +3–6 °C (general energy-balance reasoning, not measured here). Through Vcmax/Jmax temperature sensitivity and leaf-to-air VPD, each 3 °C error changes A by about 5–15% and E by about 20% (inference).
- Because real bean leaves move, Tleaf = Tair together with no leaf movement partly cancels in heat, but not in drought. Document this in the model card.

### Gaps
- No quantitative bean Tleaf − Tair distribution (e.g. thermal imaging under well-watered versus drought conditions) was retrieved.

---

## 6. Respiration: maintenance coefficients, units, growth respiration, Q10, acclimation

### Takeaway
**The plan's maintenance coefficient "0.015 g C per g N per day at 20 °C" has a unit error and is roughly 15 times too low.** The N-based Ryan (1991) coefficient is about 0.22 g C (g N)⁻¹ d⁻¹ at 20 °C (CLM's br_mr = 2.525e-6 g C g N⁻¹ s⁻¹). The number 0.015 matches McCree's dry-mass-based coefficient (~0.015 g per g dry mass per day), so the plan probably mixed up the two bases. The plan's growth respiration (construction cost 1.25 on a carbon basis, i.e. 20% of allocated C) is plausible even for bean seeds.

### Cited findings
- CLM/ELM maintenance respiration: MR = N_tissue × br_mr × q10^((T − 20)/10), per tissue N, with a reference of 20 °C (293.15 K); units g C (g N)⁻¹ s⁻¹; after Ryan 1991 — [E3SM ELM tech guide, plant respiration](https://docs.e3sm.org/E3SM/pr-preview/pr-7443/ELM/tech-guide/plant_respiration) [fetched; the page gives the form and units but not the value].
- Ryan's coefficient is 0.0106 mol C (mol N)⁻¹ h⁻¹ at 20 °C, and the CLM value is br_mr = 2.525 × 10⁻⁶ g C (g N)⁻¹ s⁻¹, Q10 = 1.5 (CLM5) or 2.0 (CLM4) — [Ryan 1991 Ecol Appl 1:157](https://doi.org/10.2307/1941808); [CLM5 tech note](https://escomp.github.io/ctsm-docs/versions/master/html/tech_note/) [known; not opened].
- McCree's equation R = k·Pg + c·W: k ≈ 0.25 (growth) and c ≈ 0.015 d⁻¹ (maintenance, per unit dry weight, ~20 °C), from white clover — McCree 1970, in *Prediction and Measurement of Photosynthetic Productivity* (Pudoc); reviewed in [Amthor 2000 Ann Bot 86:1](https://doi.org/10.1006/anbo.2000.1175) [known; not opened].
- SUCROS/WOFOST per-dry-mass maintenance at 25 °C, Q10 = 2 (g CH2O g⁻¹ DM d⁻¹): leaves 0.03, stems 0.015, roots 0.01, storage organs 0.01. Construction-cost conversion efficiencies (g DM per g CH2O) are about 0.72 for leaves, 0.69 for stems and 0.72 for roots, with storage organs crop dependent: high-protein/lipid seeds are lower (soybean ~0.48), starchy seeds ~0.7 — Penning de Vries et al. 1989, *Simulation of ecophysiological processes of growth in several annual crops* (Pudoc); [Penning de Vries 1974 / Penning de Vries, Brunsting & van Laar 1974 J Theor Biol 45:339](https://doi.org/10.1016/0022-5193(74)90119-2) [known; not opened. Verify the WOFOST crop-file values].
- Glucose cost per gram of compound class (Penning de Vries 1974): carbohydrate ~1.21, protein ~1.6–1.8 (higher with nitrate as the N source), lipid ~3.0, lignin ~2.2, organic acids ~0.9, minerals ~0.1 g glucose g⁻¹ [known; not opened].
- Respiration temperature acclimation: Q10 falls as temperature rises, Q10 ≈ 3.09 − 0.043·T — [Atkin & Tjoelker 2003 TIPS 8:343](https://doi.org/10.1016/S1360-1385(03)00136-5). Global convergent curve: ln R = a + 0.1012·T − 0.0005·T² — [Heskel et al. 2016 PNAS 113:3832](https://doi.org/10.1073/pnas.1520282113) [known; not opened].
- The Q10 for Rd in plantecophys is 1.92 by default — [plantecophys docs](https://rdrr.io/cran/plantecophys/man/Photosyn.html) [fetched].

### Inferences
- **Unit check:** 0.0106 mol C/mol N/h × (12/14) × 24 = 0.218 g C (g N)⁻¹ d⁻¹ at 20 °C. Ryan and CLM agree: 2.525e-6 × 86,400 = 0.218. The plan's 0.015 is 14.5× lower.
- **Cross-check against leaf Rd:** Rd 0.9 µmol m⁻² s⁻¹ ≈ 0.93 g C m⁻² d⁻¹. With Narea ≈ 2 g m⁻², that is ≈ 0.47 g C (g N)⁻¹ d⁻¹ at 25 °C, versus Ryan at 25 °C (Q10 2) ≈ 0.31. Same order. The plan's 0.015 would make leaf maintenance about 30× smaller than its own Rd. Using SUCROS: 0.03 g CH2O/g DM/d × 0.4 g C/g CH2O ÷ 0.045 g N/g DM ≈ 0.27 g C/g N/d at 25 °C, again consistent with Ryan.
- **Double-counting risk:** step 6 (FvCB net A includes Rd) and step 8 (maintenance respiration per organ, presumably including leaves) both charge leaf respiration. Recommend: leaves pay Rd only (inside An, at night as well), and step 8 applies only to non-leaf organs. Alternatively, set Rd = 0 in FvCB and charge leaves via maintenance, but then the fixture comparison needs Rd handled the same way.
- **Recommended M0 values:** N-based Rm = 0.22 g C (g N)⁻¹ d⁻¹ at 20 °C, Q10 = 2 (or acclimated per Atkin & Tjoelker), for stems, roots, pods and seeds. Alternative on a dry-mass basis: stems 0.015, roots 0.01, pods/seeds 0.01 g CH2O g⁻¹ DM d⁻¹ at 25 °C.
- **Growth respiration for bean seeds:** a seed of roughly 24% protein, 60% carbohydrate, 1.5% lipid and 4% minerals costs about 1.35–1.45 g glucose g⁻¹ DM. On a carbon basis that is 1.4 × 0.40 / 0.45 ≈ 1.23–1.29 g C used per g C structure, so the plan's 1.25 holds for seeds as well. Leaves (high protein) are similar or slightly higher. If species files ever express construction cost in g glucose/g DM (1.2–1.5), the basis must be labeled. N2-fixation costs (~6–7 g C per g N, an inference from general legume literature) are separate and belong to the plan's M2 nodule cost.

### Gaps
- Bean-specific maintenance measurements were not retrieved; SUCROS/WOFOST *Phaseolus* (or fababean) crop-file values were not verified.

---

## 7. Canopy light: Beer/G-function, bean leaf angle and paraheliotropism, optics, two-leaf, sky models, direct/diffuse partitioning, ray tracers, R:FR

### Takeaway
The turbid-medium voxel plus Beer/G approach is standard (RATP). Three divergences need attention:
1. **Bean leaves are strongly paraheliotropic.** They face the sun in the morning and late afternoon and turn edge-on at midday as PFD, heat and drought rise. A static spherical G = 0.5 overestimates midday direct interception under stress.
2. **No scattering term.** RATP-type models include scattering.
3. **16 sky sectors with a standard overcast sky are coarse.** The usual choice is the 46-direction turtle sky, plus an anisotropic clear sky.

The R:FR proxy is in a plausible range but ignores the neighbour-reflected far-red signal.

### Cited findings
- **Paraheliotropism in *P. vulgaris*:** cos(incidence) decreased linearly with PFD, with a minimum PFD for movement of about 25 µmol m⁻² s⁻¹. Outdoors, leaflets faced the sun in the morning and late afternoon but avoided direct rays at midday. PFD alone can drive it, even with the pulvinus temperature held constant — [Fu & Ehleringer 1991 PCE](https://consensus.app/papers/details/20cffcd50dda59e899f727a7e5e53169/?utm_source=claude_desktop) [fetched abstract].
- Higher PFD and temperature raised leaf elevation; lower water potential increased paraheliotropism; *P. acutifolius* moves more than *P. vulgaris* (leaf angles measured at 24/27/30 °C and 500/750 µmol m⁻² s⁻¹) — [Yu & Berg 1994 Plant Physiol](https://consensus.app/papers/details/dd3913bd8bfd5d558ad8e65ab3e59eed/?utm_source=claude_desktop) [fetched abstract].
- The photosynthetic cost of the movements was negligible, while water-use efficiency rose substantially — [Bielenberg et al. 2003 Env Exp Bot](https://consensus.app/papers/details/bd6923266b4451ab900fc149f695f62c/?utm_source=claude_desktop) [fetched abstract]. In contrast, field work found that paraheliotropism lowers daytime CO2 assimilation in *well-watered* beans by reducing interception — [Pastenes et al. 2004 JXB](https://consensus.app/papers/details/37e96cd579305d2fa0b9a637272da058/?utm_source=claude_desktop) [fetched abstract]. Lateral-leaflet twisting tracked leaf (pulvinule) temperature more than water potential — [Sato 1993 Jpn J Crop Sci](https://consensus.app/papers/details/1fac3fcc57e459b480fc23af08196913/?utm_source=claude_desktop) [fetched abstract].
- **Direct/diffuse partitioning (Spitters et al. 1986)**, diffuse fraction of PAR from the broadband diffuse fraction kd and the solar elevation β: k_dPAR = [1 + 0.3(1 − kd²)]kd / [1 + (1 − kd²)cos²(90° − β)cos³β] — [pvlib diffuse_par_spitters](https://pvlib-python.readthedocs.io/en/v0.11.0/reference/generated/pvlib.irradiance.diffuse_par_spitters.html) [fetched]; original paper [Spitters, Toussaint & Goudriaan 1986 AFM 38:217](https://edepot.wur.nl/217601) [not opened]. Spitters also gives the hourly broadband kd versus the clearness index (piecewise: kd = 1 for kt < 0.22, then quadratic and linear segments down to a floor that depends on solar elevation) [known; not opened. Verify the coefficients in the paper]. Erbs et al. 1982 and Reindl et al. 1990 are alternative kt-based correlations, both available in pvlib [known; not opened].
- **Two-leaf canopy (de Pury & Farquhar 1997):** sunlit and shaded big leaves; kb = 0.5/sin β (spherical); kb′ = 0.46/sin β with scattering σ = 0.15 for PAR; kd′ ≈ 0.719; canopy PAR reflectance ρ ≈ 0.04 (diffuse) — [de Pury & Farquhar 1997 PCE 20:537](https://doi.org/10.1111/j.1365-3040.1997.00094.x) [known; not opened].
- **RATP:** voxel turbid medium, the turtle sky (46 directions) for diffuse, sunlit and shaded leaf area per voxel, scattering, and coupling to photosynthesis and transpiration — [Sinoquet et al. 2001 PCE 24:395](https://doi.org/10.1046/j.1365-3040.2001.00694.x) [known; not opened]. **CARIBU** (nested radiosity / projection, used for R:FR in FSPMs by Chelle and Evers) — [Chelle & Andrieu 1998 Ecol Model 111:75](https://doi.org/10.1016/S0304-3800(98)00100-8) [known; not opened].
- **Standard overcast sky:** radiance L(θ) ∝ (1 + 2cos θ)/3, with θ the zenith angle (Moon & Spencer; CIE). The uniform overcast sky (UOC) has isotropic radiance [known; not opened].

### Inferences
- **Paraheliotropism in the plan:** bean is the one M0 species whose leaf angles change hourly, which is visible in a game and changes energy balance and interception. Suggested M1 rule: each leaflet's normal rotates toward the sun (diaheliotropic) at PAR < ~300, and away from it in proportion to PAR above about 25–500 µmol m⁻² s⁻¹, enhanced by temperature > 27 °C and by the drought factor. That would give cos(i) falling linearly with PFD, as in Fu & Ehleringer. The exact slope must come from the paper's figure (gap). This conflicts with the plan's "daily geometry, hourly radiation" split, because per-leaf hourly coefficients assume a fixed orientation. The feasible option: compute the voxel optical depth daily (unchanged) and apply the leaf's own cos(incidence) as an hourly per-leaf multiplier on absorbed direct PAR. The movement changes the leaf's own interception far more than the shading it casts on others.
- **Optics:** typical green-leaf PAR absorptance ≈ 0.85 (reflectance ~0.08–0.10, transmittance ~0.05–0.07) is a widely used default; I found no bean-specific spectra. Without scattering, shaded leaves deep in the canopy are underlit by about 10–20% of their diffuse budget (inference from σ = 0.15). Add a single-order scattering correction, or document the limitation. The Monte Carlo validation (10% per leaf) will expose this if the Python ray tracer includes scattering and the sim does not, so both need the same assumption.
- **Sky discretisation:** 16 sectors in 2 elevation bands under-resolves elevation for diffuse light. Consider the turtle-46 directions, or 3–4 bands, at about 3× the ray cost. Also use a clear-sky (CIE clear) distribution when kd is small, since diffuse from clear skies is circumsolar-weighted.
- **Clumping:** a single plant in a voxel grid captures clumping between voxels explicitly; within a voxel, use Ω ≈ 1 (sub-voxel clumping is neglected, which is acceptable at 2–3 cm voxels).
- **R:FR proxy:** sunlight R:FR is about 1.1–1.2. Under dense canopies it falls to about 0.1–0.3 (well-established; Smith 1982, Holmes & Smith 1977, not opened). The plan's 1.2·exp(−0.5·LAI) gives 0.73 at LAI 1 and 0.27 at LAI 3, which is plausible. What it misses: (1) neighbour detection by horizontally reflected far-red *before* any shading (Ballaré et al. 1990, not opened), which drives early shade avoidance in dense sowing; (2) any directional dependence. The proxy also uses the LAI "above" the leaf, which in a voxel grid should be the optical-depth-weighted path toward the sky, not the vertical column only.

### Gaps
- No bean leaf-angle distribution data (e.g. a fitted ellipsoidal χ) and no quantitative cos(i)-versus-PFD slope were retrieved; the Fu & Ehleringer 1991 figures would give the slope.
- Bean leaf PAR reflectance and transmittance were not retrieved.
- The Helios radiation model (Bailey) was not researched within budget.

---

## 8. Per-light-class binning in other models

### Takeaway
Binning by PAR is defensible, but using a class's single representative value for every leaf in the class introduces large errors in the linear (low-light) part of the curve. Evaluate the solver at the class nodes and interpolate each leaf's rate linearly in PAR (or log PAR). That keeps the cost of 16 solves and removes most of the error.

### Cited findings
- Canopy models discretise by sunlit/shaded status (two-leaf: de Pury & Farquhar 1997, link above) and, in multilayer models, by layer and by sunlit leaf-angle class (e.g. 9 inclination classes for cos(i)) [known; not opened]. The plan's §3.15 already uses the two-leaf model at M4.

### Inferences
- **Size of the error:** 16 log-spaced classes from 1 to 2,000 µmol m⁻² s⁻¹ give a ratio of 2000^(1/15) ≈ 1.66 between neighbouring class edges. At low light, A ≈ α·PAR − Rd, so a leaf at a class edge credited with the class's geometric-centre rate is off by up to ±29% in gross A. Piecewise-linear interpolation between node solutions of a smooth concave A(PAR) curve has an error under 1–2% (inference). The cost is the same; interpolation weights are deterministic.
- Classes must share leaf temperature, VPD and CO2, which holds per plant in M0 (Tleaf = Tair). Once a leaf energy balance arrives in M2, classes need a temperature dimension, or the solve moves to per leaf. The plan's budget (23k solves per day) would allow per-leaf solving for one plant anyway.

### Gaps
- No published FSPM using exactly this PAR-binning optimisation was identified within budget.

---

### Summary of plan errors and divergences (inference, built from the findings above)
1. **§3.8 maintenance respiration: "0.015 g C per g N per day" is wrong** (unit or basis confusion with McCree's per-dry-mass c ≈ 0.015). Use about 0.22 g C (g N)⁻¹ d⁻¹ at 20 °C (Ryan 1991 / CLM), Q10 = 2, or per-dry-mass SUCROS values.
2. **§3.16 Vcmax 89 / Jmax 160 are Cc based (gm 0.47).** Add gm, or use Ci-based values (about 70) with the Bernacchi 2001 constants. Otherwise Ac is about 20% high.
3. **§3.16 g1 ≈ 4 is below crop values** (Lin 2015 / CLM5 ≈ 5.8). Expect about 25% lower transpiration than a crop default.
4. **Leaf respiration may be counted twice** (Rd in FvCB plus leaf maintenance in step 8).
5. **§8.1 fixture conventions must match plantecophys:** incident versus absorbed PAR (α = 0.24 is incident), EdVC = 0 (no Vcmax decline at 40 °C), TPU off, Rday = Rd with Q10 1.92, g0 = 0, the 1.57 vs 1.6 factor, and the hard vs smooth minimum.
6. **§3.8 Newton:** prefer the analytic quadratic for determinism and exact agreement with the oracle.
7. **§3.8 light-class binning:** interpolate between class nodes, or per-leaf error reaches about ±29% at low light.
8. **§3.5 static G = 0.5 ignores bean paraheliotropism,** which is well documented and strongly PFD-, temperature- and drought-dependent.
9. **§3.5 no scattering, coarse 16-sector standard-overcast sky.** Consider turtle-46 and a clear-sky distribution; the Monte Carlo validation must use matching assumptions.
10. **§3.8 Tleaf = Tair** is acceptable for M0 well-watered conditions. Real bean leaves run cooler than air in heat, while drought without leaf movement runs hotter.
11. **§3.5 R:FR proxy** values are plausible but leave out neighbour-reflected far-red.
12. **Construction cost 1.25** (carbon basis) is consistent with bean tissue and seed composition. Label the basis (g C per g C, not g glucose per g DM).
