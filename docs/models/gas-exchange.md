# Gas exchange

> Model cards are shown to players in the app's side panel (design plan §5.7), so write
> the Purpose and Known limits for a curious player, and the rest for a developer.

## Purpose
How a leaf trades water for carbon. Light drives the leaf's chemistry, and CO2 has to
get in through the stomata, the pores on the leaf. Those same pores let water out. The
leaf opens them wider when it is fixing carbon fast, and closes them when the air is dry
or the soil runs short of water.

## Equations
Per unit leaf area, crate `ps-physio::gas` (Farquhar, von Caemmerer and Berry 1980;
Medlyn et al. 2011), solved once per light class (`ps-physio::classes`).

- Temperatures: `Tk = T + 273.15`, `R = 8.314 J mol⁻¹ K⁻¹`, reference 298.15 K.
- Rubisco kinetics (Bernacchi et al. 2001), with `f(E) = exp(E (Tk − 298.15)/(298.15 R Tk))`:
  `Kc = 404.9 f(79430)`, `Ko = 278.4 f(36380)` (mmol mol⁻¹), `Km = Kc (1 + O/Ko)` with
  `O = 210 · P/100`, `Γ* = 42.75 f(37830) · P/100` (µmol mol⁻¹).
- Capacities (peaked Arrhenius, Medlyn et al. 2002):
  `V(T) = V25 · exp((Tk − 298.15) Ea / (R Tk 298.15)) · (1 + exp((298.15 ΔS − Ed)/(298.15 R))) / (1 + exp((Tk ΔS − Ed)/(Tk R)))`
  for Vcmax and Jmax, times leaf capacity `c` (aging, below).
- Day respiration: `Rd = c · Rd25 · Q10^((T − 25)/10)`.
- Electron transport: `J = (αQ + Jmax − √((αQ + Jmax)² − 4θαQ Jmax)) / (2θ)`, `Q` absorbed
  PAR, α electrons per absorbed photon, θ curvature.
- Demand: `Ac = Vcmax (Ci − Γ*)/(Ci + Km) − Rd`; `Aj = (J/4)(Ci − Γ*)/(Ci + 2Γ*) − Rd`.
- Medlyn conductance to CO2: `gc = g0/1.57 + s · (1 + g1/√max(D, 0.5)) A / Ca`, where `s`
  is the water stress factor (0 to 1) from the soil (§3.2 hourly step 5).
- Supply: `A = gc (Ca − Ci)`. For each limitation, the residual
  `F(Ci) = A(Ci)(1 − m(Ca − Ci)) − (g0/1.57)(Ca − Ci)`, with `m = s(1 + g1/√D)/Ca`, is
  solved by safeguarded Newton on `[Ca − 1/m, Ca]` (bisection whenever a Newton step
  leaves the bracket or does not halve the residual; tolerance 1e-6 µmol mol⁻¹, at most
  50 iterations). `A = min(Ac, Aj)`; the limitation is reported.
- Below light compensation (`A ≤ 0` at `Ci = Ca`): `Ci = Ca`, `A = min(Ac, Aj)` there,
  `gs = g0`.
- Water: `gs = 1.57 gc` (mol m⁻² s⁻¹ water vapor), `E = gs D / P`.
- Leaf capacity: `c = 1` until the leaf is fully expanded, then linear to the floor (0.4)
  at the end of its lifespan, which counts from full expansion (§3.8 leaf aging).
- Sink feedback (decision 0013, provisional; amended after the WP8 review): with
  `f(C) = clamp(1 − (C − C_on)/(C_full − C_on), 0, 1)`, where `C = R / W` is the
  leaf's own reserve concentration (g C per g DM), net assimilation becomes
  `A' = f · max(0, A + Rd) − Rd`. Only gross assimilation is inhibited, so a leaf full
  of sugar still pays its day respiration and can starve. Transpiration scales by the
  same factor as gross assimilation (Medlyn conductance follows assimilation). The
  daily report carries the share of potential gross assimilation lost to it
  (`feedback_loss`); above about 0.2 the plant lacks sinks, and WP12 adds sinks
  before changing the thresholds.
- Light classes: each hour a table of 17 PAR nodes (0, then 16 log-spaced from 1 to
  2000 µmol m⁻² s⁻¹) × 4 capacity nodes (1.0, 0.8, 0.6, 0.4) is solved (68 solves per
  plant-hour). A leaf part reads it linearly in PAR between neighboring nodes and
  linearly in capacity. Sunlit and shaded parts read the same table at their own PAR.
- Carbon: `g C per hour = A · area · 3600 · 12e-6`.

## Parameters
| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| vcmax25 | 89.3 | µmol m⁻² s⁻¹ | Pimentel 2011 (heritable) |
| jmax25 | 159.8 | µmol m⁻² s⁻¹ | Pimentel 2011 (heritable) |
| rd25 | 0.88 | µmol m⁻² s⁻¹ | Pimentel 2011 (heritable) |
| medlyn_g1 | 4.0 | kPa^0.5 | "~" herbaceous crop default (heritable) |
| medlyn_g0 | 0.01 | mol m⁻² s⁻¹ | "~" design plan §11.4 |
| quantum_yield α | 0.3 | mol mol⁻¹ | design plan §11.4; per absorbed photon (plantecophys' 0.24 is per incident photon, about 0.3 × 0.8 absorptance) |
| electron_transport_curvature θ | 0.7 | 1 | design plan §11.4 |
| rd_q10 | 1.92 | 1 | "~" plantecophys default |
| vcmax_activation, _deactivation, _entropy | 58550, 200000, 629.26 | J mol⁻¹, J mol⁻¹, J mol⁻¹ K⁻¹ | "~" Medlyn 2002 form, values for bean to validate |
| jmax_activation, _deactivation, _entropy | 29680, 200000, 631.88 | same | "~" Medlyn 2002 form, values for bean to validate |
| Kc25, Ec | 404.9, 79430 | µmol mol⁻¹, J mol⁻¹ | Bernacchi et al. 2001 |
| Ko25, Eo | 278.4, 36380 | mmol mol⁻¹, J mol⁻¹ | Bernacchi et al. 2001 |
| Γ*25, EΓ | 42.75, 37830 | µmol mol⁻¹, J mol⁻¹ | Bernacchi et al. 2001 |
| O2 | 210 | mmol mol⁻¹ | atmosphere |
| H2O/CO2 diffusivity ratio | 1.57 | 1 | plantecophys (Medlyn 2011 writes 1.6) |
| VPD floor | 0.5 | kPa | plantecophys `vpdmin` |
| leaf_capacity_floor | 0.4 | 1 | "~" design plan §3.8 |
| reserve_feedback_onset C_on | 0.10 | g C / g DM | "~" about 18% non-structural carbohydrate in total dry mass (decision 0013) |
| reserve_feedback_full C_full | 0.30 | g C / g DM | "~" about 40% non-structural carbohydrate (decision 0013) |

## Inputs and outputs
| Direction | Quantity | Unit | From / to |
| --- | --- | --- | --- |
| in | absorbed PAR, sunlit and shaded | µmol m⁻² s⁻¹ | ps-light `absorbed_par` |
| in | air temperature (leaf temperature in M0), VPD, CO2, pressure | °C, kPa, ppm, kPa | ps-weather |
| in | water stress factor | 1 | ps-soil |
| out | net assimilation, conductance, transpiration, Ci, limitation | µmol m⁻² s⁻¹, mol m⁻² s⁻¹, mol m⁻² s⁻¹, ppm | leaf income (allocation), water uptake (soil) |

## Encoded or emergent
Encoded: the biochemistry, the stomatal rule, the stress multiplier on the stomatal
slope. Emergent: midday depression on hot dry days, CO2 fertilization, the water cost of
carbon (water-use efficiency) and how it shifts with humidity, the slower carbon gain of
old leaves.

## Known limits
- Leaf temperature equals air temperature (no energy balance until M2).
- No mesophyll conductance, no triose-phosphate limit, no boundary layer.
- Water stress acts on the stomatal slope only, not on photosynthetic capacity.
- The light-class table costs up to 3.0% in A between nodes (measured at 333 µmol m⁻² s⁻¹
  and capacity 0.7).
- Bean temperature responses are "~" values in the Medlyn 2002 form, not bean
  measurements (WP12).

## Validation
- `ps-physio/tests/plantecophys.rs::gas_exchange_matches_plantecophys`: 1,008 cases
  (PAR 50–2000 × T 10–40 °C × VPD 0.5–4 kPa × CO2 280, 420, 800 ppm), target 2% on A,
  gs and E. Measured worst: A 1.18%, gs 1.11%, E 1.11%. The difference is
  plantecophys' smooth minimum of Ac and Aj (θ = 0.9999) near co-limitation.
- Fixture: `crates/ps-physio/tests/fixtures/photosyn.csv` from
  `validation/fixtures/photosyn_reference.py`; CI job `plantecophys` reruns the grid with
  plantecophys 1.4-6 in R and requires agreement to 1e-6 relative.
- Unit tests in `ps_physio::tests`: `bean_at_reference_conditions` (supply equals demand
  to 1e-6), `darkness_gives_respiration_and_g0`, `full_stress_closes_stomata_to_g0`,
  `aging_scales_capacity`, `class_table_matches_direct_solves`,
  `leaf_capacity_declines_after_expansion`.

## References
- Bernacchi, C. J. et al. (2001). Improved temperature response functions for models of
  Rubisco-limited photosynthesis. Plant, Cell and Environment 24.
- Duursma, R. A. (2015). Plantecophys: an R package for analysing and modelling leaf gas
  exchange data. PLoS ONE 10.
- Farquhar, G. D., von Caemmerer, S. and Berry, J. A. (1980). A biochemical model of
  photosynthetic CO2 assimilation in leaves of C3 species. Planta 149.
- Medlyn, B. E. et al. (2002). Temperature response of parameters of a biochemically based
  model of photosynthesis. II. Plant, Cell and Environment 25.
- Medlyn, B. E. et al. (2011). Reconciling the optimal and empirical approaches to
  modelling stomatal conductance. Global Change Biology 17.
