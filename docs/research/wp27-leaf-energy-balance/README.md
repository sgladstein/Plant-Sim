# WP27 leaf energy balance: research

Written 2026-11-05 by the research thread, for WP27 ("leaf energy balance: leaf temperature from
radiation, wind and transpiration"; done when the leaf-air temperature difference on a hot still
day falls within the literature range). Checked against `main` at model 0.11.0. Tags: **[F]**
the source was read, **[F2]** read in a paper that cites it, **[K, to verify]** from knowledge,
**[I]** inference or arithmetic. **[I, proto]** marks numbers from the Python prototype in
`prototype/`, which copies `gas.rs` (FvCB + Medlyn, bean parameters).

Notes, with findings and full citations:
- `notes/leaf_air_temperature.md`: about 28 measured sources, CWSI baselines and heat thresholds.
- `notes/boundary_layer_wind.md`: leaf boundary layer, canopy wind and hourly wind.
- `notes/leaf_optics.md`: PAR, NIR, emissivity and chlorophyll.
- `notes/model_formulation.md`: the algorithm, how it fits the class table, a survey of what
  other models do, and solver accuracy.

## 1. Headline: the energy balance exposes low stomatal conductance

Field bean baselines (sunlit canopy, well watered, midday, wind 2 to 3 m/s) agree on
**dT = Tleaf − Tair ≈ 2.3 to 3.5 − (1.9 to 2.7)·VPD** [F, F2: Idso 1982; Erdem 2006; Silveira
2024]. That is about −3.5 to −4.6 °C at VPD 3 kPa.

Running the prototype at the same conditions (Ta 30 °C, PAR 1500, wind 2.5 m/s, VPD 1 to 4 kPa)
and fitting a line through it [I, proto, `prototype/cwsi_sweep.py`]:

| medlyn_g1 | Fitted baseline | dT at VPD 3 | gs at 35 °C, VPD 3.9 |
| --- | --- | --- | --- |
| 4.0 (bean today) | +3.55 − 1.24·VPD | **0.0** | 0.21 mol m⁻² s⁻¹ |
| 5.79 (re-tune item 4) | +3.03 − 1.49·VPD | −1.3 | 0.34 |
| 8.0 | +2.65 − 1.70·VPD | −2.4 | 0.52 |
| Field | +2.3 to 3.5 − 1.9 to 2.7·VPD | −3.5 to −4.6 | (a fixed gs of 0.6 to 0.8 gives −3.8 to −4.8) |

The intercepts match, but the slope is too shallow. With today's g1 = 4, a well-watered sunlit
bean leaf on a hot, dry day sits at or above air temperature, which contradicts every field
baseline. The cause is low gs at high VPD. Medlyn's 1/√D closes stomata faster than field bean
does, and g1 = 4 starts low.

Partial explanations [I]:
- An infrared thermometer sees the whole sunlit canopy, including cooler, partly shaded leaves.
- Field leaves are better coupled to the air than one isolated leaf.

Neither explains 2 to 3 K.

Consequences:
1. Land re-tune item 4 (g1 = 5.79, `docs/research/retune/items-1-4.md`) **before or with**
   WP27. With g1 = 4 the WP27 acceptance test below fails.
2. After WP27, add a validation target: **midday gs of well-watered field bean at 0.4 to
   0.8 mol m⁻² s⁻¹**. Treat this as [K, to verify]: Pimentel 2011 reports gs max ≈ 0.4 in the
   greenhouse [F], and field values need a source. Keep the CWSI slope as a reported check, not
   a pass/fail check. If the slope stays far off with g1 = 5.79, the next thing to look at is
   the Medlyn VPD response at high D, not the energy balance.

## 2. Recommended algorithm (details in `notes/model_formulation.md` §1)

**Where it sits.** Inside hourly step 6. No step is reordered, but equations, defaults and
goldens change, so it needs a minor version bump and a decision record.
- Step 1 gains wind.
- Sky longwave reads `cloud_fraction`, so that field's "display only" doc changes.
- Step 3's `LightCoeffs.diffuse` serves as the sky view.
- Step 7 consumes the new transpiration.
- Thermal time stays on air temperature.

**Two levels.**
1. **Each class-table node** (17 PAR × 4 capacity) solves the coupled gas exchange and energy
   balance at a reference environment for the hour: air, reference wind and leaf width. This
   keeps tables shareable between plants once wind and leaf width are added to the key.
   - Iterate g(T) = T_EB(gs(T)) − T with a bracketed Illinois (regula falsi) method: start from
     two fixed-point rounds, cap at 8 evaluations, tolerance 1e-3 K.
   - Error is 0.03 K or less over 2,304 cases [I, proto].
   - Plain fixed-point iteration fails by 1.5 to 4 K in hot, still air. A warmer leaf loses A,
     closes its stomata and warms further, and one case had three roots.
   - Nodes with PAR = 0 skip the loop, since gs is fixed at g0 there.
2. **Each leaf part** (sunlit, shaded) then computes its own temperature and transpiration in
   closed form plus one Newton step (about 2 `exp` per part), from:
   - its own absorbed shortwave;
   - its own sky view;
   - its own wind, `u_top · τ_diffuse^(0.5 k_w/k_d)`;
   - the gs it read from the table, after the sink-feedback scaling.

   A stays the table value. Skipping this step leaves single leaves off by ±1 to 1.5 K, but
   the plant totals are nearly unchanged [I, proto].

**Equations.** Per one-sided leaf area; both faces exchange heat and longwave.
- `S_abs = Q_abs/4.57 · (1 + r_nir)`, with r_nir ≈ 0.25 in the open. Raise r_nir for shaded
  parts with τ_nir ≈ τ_par^0.5 [K].
- `Rni = S_abs − ε σ Tk⁴ · f_sky · (1 − ε_sky)`, with
  `ε_sky = (1 − 0.84c) · 1.72 (ea/Tk)^(1/7) + 0.84c` (Brutsaert with a cloud correction).
- `ΔT = (Rni − λ g_v D/P) / (2 cp (g_bh + g_r) + λ g_v s/P)`, then one Newton step on the full
  balance. That takes the error from up to 0.3 K to under 0.001 K.
- `E = g_v (es(TL) − ea)/P`. Stomata see the leaf-to-air VPD `es(TL) − ea`.
- Conductances (§3): g_bh per face, `g_r = 4εσTk³/cp`, `g_bw = 1.075 g_bh`. Stomata and the
  boundary layer act in series on each face, and the faces add in parallel.

**Reads leaf temperature:** FvCB kinetics, Rd, gs, transpiration and leaf maintenance
respiration (as CLM5 does).
**Stays on air temperature:** thermal time and phenology (all cardinal temperatures were
calibrated on air), leaf expansion, and flower heat abortion.

**Transpiration falls**, because the boundary layer is now in series with the stomata. At
35 °C, RH 30% and 0.5 m/s it drops by about 16% [I, proto]. Re-check the drought validation
targets after the change.

## 3. Parameters

| Name | Bean | Wheat | Unit | Source | Tag |
| --- | --- | --- | --- | --- | --- |
| `leaf_width` (leaflet or blade) | 0.08 (range 0.05 to 0.11) | 0.015 (0.012 to 0.022) | m | `notes/boundary_layer_wind.md` | ~ [K] |
| characteristic dimension d | 0.72 × width | 0.72 × width | – | Campbell & Norman 1998 | [K, to verify] |
| forced convection, heat, per face | E · 0.135 √(u/d) | same | mol m⁻² s⁻¹ | flat plate; E = 1.25 for bean leaves in a canopy (Chamberlain 1974), 1.4 in Campbell & Norman | [F] / [K] |
| free convection, per face | 0.5 D_H (1.6e8 ΔT d³)^¼ / d · P/(R Tk), D_H 21.5e-6 m² s⁻¹ | same | mol m⁻² s⁻¹ | Leuning 1995, as coded in plantecophys | [F] |
| free + forced | sum | sum | – | plantecophys, Leuning 1995 | [F] |
| vapour/heat boundary-layer ratio | 1.075 | 1.075 | – | Leuning 1995 (Helios uses 1.08) | [F] |
| abaxial share of stomatal conductance | 0.875 (abaxial:adaxial ≈ 7:1) | 0.5 | – | Kanemasu & Tanner 1969 abstract; wheat [K] | [F] / [K] |
| PAR absorptance | 0.85 (keep) | 0.85 | – | Björkman & Demmig 1987: 0.838 over 37 C3 species | [F2] |
| NIR absorptance | 0.20 (0.15 to 0.22) | 0.20 | – | PROSPECT-D runs; Goudriaan σ_NIR 0.8 | [I] / [K] |
| leaf longwave emissivity | 0.97 (0.95 to 0.99) | 0.97 | – | Chen 2015 review (bean 0.957, Fuchs & Tanner 1966); tealeaves default 0.97 | [F2] |
| chlorophyll to PAR absorptance (optional, for N yellowing) | Chl/(Chl + 76), Chl in µmol m⁻², floor 0.5 | same | – | Evans 1993 | [K, to verify] |
| canopy wind attenuation a | 4.39 − 3.97 e^(−0.258·LAI) (1.3 to 2.6) | 2.5 | – | McNaughton & van den Hurk 1995; Cionco 1972 | [F] |
| wind floor at the leaf | 0.1 | 0.1 | m s⁻¹ | DSSAT (lowered from 1.0 in 2014), CLM | [F] |
| wind default when missing | 2 m s⁻¹ at 2 m | – | – | FAO-56 | [F] |
| hourly wind from the daily mean | DSSAT HWIND: night 0.3·W_max, W_max = W_avg/(0.3 + 0.7·DAYL/48), daytime sine peaking at noon | – | – | DSSAT source | [F] |

PAR share of solar energy: the code uses 0.5, while ASTM G173 gives 0.43 global [I]. That
changes the light model, not just WP27, so leave it.

**Weather needs:**
- Add `wind: MetersPerSecond` to `HourWeather`.
- The CSV reader already parses `wind_m_s` but drops it.
- Presets need a daily mean wind (2 m s⁻¹ unless the preset's source gives one), spread over
  the day with HWIND, which conserves the daily mean exactly and is deterministic.
- Plot-scale wind at canopy top: a log profile from 2 m with the surroundings' z₀ ≈ 0.02 m
  gives u_h ≈ 0.6 to 0.75 · u₂ for a 0.5 m canopy [I].

## 4. Acceptance tests (WP27 "done when")

All tests use a sunlit, horizontal bean leaflet, clear sky and PAR 1500, unless stated.
"Expected" values are prototype numbers with g1 = 5.79.

| # | Case | Conditions | Pass range for dT | Expected |
| --- | --- | --- | --- | --- |
| 1 | Hot still day, well watered | Ta 35 °C, RH 30%, u 0.5 m s⁻¹ at the leaf | **−7 to +1** | about −1.5 |
| 2 | Same, sign check | VPD ≥ 3 kPa, u ≥ 1 m s⁻¹, stress factor 1 | **< 0** | −0.7 to −1.7 (with g1 = 4 it fails at +0.2 to +0.9) |
| 3 | Same, stomata closed | stress factor 0 | **+1 to +8**, and **≥ 3 K above** case 1 | +5 to +8 |
| 4 | Clear, calm night | Ta 15 °C, u 0.5 m s⁻¹, sky view 1 | **−3 to −0.5** | −2 to −3 |
| 5 | Wheat, irrigated, hot | Ta 33 to 37 °C, low RH, midday | **< 0** (field canopies run 4 to 8 below air) | – |
| 6 | Conservation | any | transpiration taken from the soil equals the plant total, at 1e-9 | – |

Measured basis (`notes/leaf_air_temperature.md`):
- Bean lower baselines: −3.5 to −4.6 at VPD 3. CIAT single leaves: mean −5.2 °C.
- Bean upper (non-transpiring) baselines: +2.4 to +4.0. Unirrigated field bean: up to +5.
- Several crops at 800 to 1000 W m⁻²: up to +8.
- Night leaves: −1 to −3 (soybean, tobacco).
- Irrigated wheat in Sudan at 37 °C: 4.7 to 7.2 below air.

No study measured bean at controlled low wind, so the still-air end of case 1 rests on
reasoning [I].

Leaf movement: stressed bean leaflets turn edge-on to the sun (paraheliotropism) and stay
cooler. Fix the leaf horizontal in these tests until research item 7 (sun tracking) is built.

## 5. Owner decision

**Should leaf frost read leaf temperature?** On clear, calm nights, exposed leaves run 2 to 3 K
below air [I, proto; F, night data]. If frost reads leaf temperature, a +2 °C air night can
frost-kill top leaves (radiation frost), while leaves under cover survive.

**Recommendation:** yes, for leaves only (`leaf_frost_kill` −2.2 °C stays). It is real, it shows
in the viewer, and it suits a frost challenge. Flowers and pods stay on air temperature until
there is a reason to change them.

Implementation calls with recommendations, not owner decisions:
- No boundary-layer term in the CO2 supply for now. It would cut A by 1 to 3% and break the
  plantecophys fixture.
- Wind and leaf width go in the class-table key.
- Show a leaf temperature lens (WP26) once WP27 lands.

## 6. Heat thresholds for later (bean, on leaf or air temperature)

- Seed set falls 6.5% per °C above 28/18 °C day/night (Prasad 2002) [F].
- Pollen develops abnormally at 32/27 °C (Porch & Jahn 2001) [F].
- Two days at 35 °C cause flower and pod abscission (Monterroso & Wien 1990) [F].
- PSII damage starts at a leaf temperature of 42 to 43 °C (Pastenes & Horton 1999) [F].

The RON's `pod_set_t_*` (22/25/36) already covers the reproductive side on air temperature.

## 7. Gaps

- Campbell & Norman, Jones, Leuning 1995 and Schuepp were not opened (paywalled), so their
  equation forms are [K, to verify]. The code-based forms (plantecophys, CLM5, DSSAT, Helios)
  were read.
- There is no bean leaf temperature measured at controlled low wind, and no bean
  free-versus-restrained leaflet numbers (Fu & Ehleringer 1989 and Pastenes 2005 were
  blocked).
- No measured bean spectrum: EcoSIS, LOPEX and Zenodo are blocked from the container. Optics
  come from PROSPECT-D runs and the literature.
- Field gs for well-watered bean at midday has no source yet; it is the key number behind §1.
- Bean canopy wind attenuation and leaflet width are not bean-specific measurements.
- DSSAT `ETPHR.for` has `RBSH = FRSHV * RBSL`, which looks like a bug that overwrites the
  shaded-leaf resistance [F; I that it is unintended]. Don't copy it.
