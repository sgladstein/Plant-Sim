# Spec R1: carbon sinks (thickening, storage, seed-driven N) and phototropism

Status: spec, written 2026-11-05 after the owner chose R1, R2 and R3 for full specs
(review: [README.md](README.md)). Nothing is built. Accepted packages get WP numbers and
issues when the implementation thread picks them up; until then they are R1a, R1b and
R1c.

Tags as in the review: **[M]** measured in a run at model 0.13.0, **[F]** read in a
source, **[K, to verify]** from knowledge, to be checked by the Science research thread
before the value is used, **[I]** inference. Every new parameter in data/ needs a value,
unit and source; "~" marks a guess to calibrate.

## 1. Why

From flowering on the bush bean discards about 70% of its potential gross
assimilation (pole bean 79%, wheat 57%), because no sink is left [M]. Stems stop
gaining mass at day 60, roots never widen, and nothing stores starch. Three field and
pot facts show what is missing:

| Fact | Field or pot | Sim (single_pot, model 0.13.0) |
| --- | --- | --- |
| Stem (+ petiole) share of stem + leaf at peak (R5 to R6) | 0.59 Seafarer, 0.66 Isabella, irrigated field, UFGA8501 day 56 [F, docs/archive/research/notes/dssat_bean_timeseries.md] | 0.25 (stem only, leaf includes petiole) [M] |
| Stem mass from peak to maturity | falls 36% (Seafarer) and 33% (Isabella), part of it petioles lost with leaves [F, same file] | flat at 1.92 g [M] |
| Root:shoot at seed fill | 0.18 to 0.21 in a pot (Pinzón 2024), 0.08 to 0.30 band [F, data/validation/targets.ron] | 0.06 [M] |
| Stem and root diameter | basal stem widens through the season; tap root several mm with taper [K, to verify] | every stem 4.6 mm, hypocotyl 2.4 mm, tap root 0.68 mm, never changing [M] |

R1 adds the sinks a real bean has (secondary thickening of stems and roots, and a starch
store that refills and is drawn down in seed fill), and lets seed fill strip nitrogen
from leaves whatever the soil holds. Phototropism (R1c) rides along because it is cheap
and makes the WP20 lean real.

## 2. Packages and order

| Package | Contains | Size | Depends on |
| --- | --- | --- | --- |
| R1a Thickening | Primary and secondary mass split, stem pipe-model thickening, root secondary growth, taper, phloem conductance rule | M | WP27 merged (so the tuning pass after R1 runs once) |
| R1b Storage and seed-driven N | Starch store in stems and roots, release in fill, root exudation, seed-fill N stripping before soil uptake, reporting of storage in dry mass | M | R1a |
| R1c Phototropism | Daily bend of new internodes toward the light | S | none (may go first) |

One WP per PR (CLAUDE.md). R1a and R1b each bump the model minor version and regenerate
the golden hashes; R1c too. After R1b, re-tune the "~" parameters on four-seed means
(decision 0026), because the carbon budget moves a lot.

## 3. R1a: thickening

### 3.1 State
- New per-organ column `secondary: Vec<GramsDM>` in the organ arena (`ps-graph`),
  zero at birth, for `Internode` and `Root` organs. It is part of `structural` (so every
  existing sum, ledger and hash still sees one structural mass): `W = W_p + W_s`.
- Allometry (`ps-graph/src/allometry.rs`, `SpecificSize::Length`): length from primary
  mass only, radius from the total:
  `L = SL · (W − W_s)`, `r = sqrt(W / (ρ π L))`.
  With `W_s = 0` this is today's rule exactly. Once an organ's expansion curve is
  complete its primary mass stops growing, so its length is frozen and all later mass
  widens it.
- Daily per internode: `A_max,i`, the running maximum of the leaf area carried above it
  (pipes stay after leaves die: the "disused pipe" of the pipe model). New column
  `supported_area: Vec<M2>`, or kept in ps-arch state; it must be in the snapshot and
  the hash.

### 3.2 Stems: pipe model (Shinozaki et al. 1964 [K, to verify])
Daily, after initiation (daily step 3), one pass from the tips down gives for each living
internode `i` the leaf area above it now, `ΣA_i`, and the fruit dry mass above it,
`ΣF_i`. Then `A_max,i ← max(A_max,i, ΣA_i)` and the target cross-section

`X*_i = h_L · A_max,i + h_F · ΣF_i`  (m²)

The second term is the mechanical demand of a pod load; set `h_F = 0` if calibration
does not need it.

Hourly, in the growth step (§3.2 step 12), each internode whose expansion is complete
has a thickening demand

`ΔW_s,pot = min(X*_i − π r_i², g_max · 2π r_i · Δt) · L_i · ρ_stem · f_T · s_w`  (g DM, at least 0)

- `g_max` caps the radial rate (cambial activity); `f_T` is the organ's temperature
  growth multiplier (§3.11); `s_w` the water stress factor (radial growth is as
  drought-sensitive as expansion [K, to verify]).
- Carbon demand `ΔW_s,pot · c_f · cc_stem`, with the stem construction cost from the
  re-tune (1.29) once that lands.
- It is a sink at priority 0.4, the value §3.9 already lists for secondary thickening,
  through the existing rule `realized = min(demand, priority × max(0, R − 0.05 W))`.
- No thickening after plant maturity, in dead organs, or on a plant flagged as a grass.

The hypocotyl is an internode for this rule and carries the whole shoot, so it becomes
the thickest stem, which is what a bean looks like.

### 3.3 Roots: OpenSimRoot secondary growth
OpenSimRoot's bean set gives a radial growth rate rising to 0.003 cm/day by about day
29, scaled per root class: 4 for the tap root (2 after day 40), 0.7 for basal roots (0.4
later), 0.6 to 0.2 for hypocotyl-borne roots, 0.1 to 0.3 for laterals [F, archive roots
note §1, from growthrates.xml].

Hourly, per root segment and cohort:

`ΔW_s,pot = 2π r L · (dr/dt)_type(age) · Δt · ρ_root · f_T,soil · s_w`

- `(dr/dt)_type(age)` is the species table times the root type's factor. Key the age on
  the **segment's own age**, so older (basal) segments are wider and taper emerges. If
  OpenSimRoot keys it on plant age, record the deviation in the decision.
- For a lateral cohort, `L` already includes the multiplicity, so the formula holds per
  root.
- Priority 0.4, root construction cost, soil-layer temperature as today.
- The rate is a potential: carbon decides how much is realized.

Strock et al. 2017 show the trade-off this creates: bean lines with less root
secondary growth have 27% (greenhouse) to 32% (field) more root length and much more
shoot mass under low phosphorus [F]. So `root_secondary_factor` is a good heritable
trait for M3.

### 3.4 Phloem conductance
Today `k = K · π r² / l`. If `r` includes secondary growth, transport gets much easier
as stems widen. The pole bean's height already depends on a raised `K`, and its final
height sits near the top of its band (STATUS). Rule for R1a: `k` uses the **primary
radius**, `r_p = sqrt(W_p / (ρ π L))`, so transport is unchanged. A later package may
add a secondary-phloem term if review finds transport too weak in big plants. Record
this in the decision.

### 3.5 Parameters (species files)

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| `pipe_area_per_leaf_area` (`h_L`) | calibrate, start 3e-4 | m² m⁻² | ~ calibrated to the basal stem diameter target (§6) |
| `pipe_area_per_fruit_mass` (`h_F`) | 0 | m² g⁻¹ | ~ only if calibration needs it |
| `stem_radial_rate_max` (`g_max`) | 0.1 | mm day⁻¹ | ~ [K, to verify, herb cambium rates] |
| `root_radial_rate` table | 0 to 0.003 by day 29 | cm day⁻¹ | OpenSimRoot bean growthrates.xml [F] |
| per root type `secondary_factor` (early, late, switch day) | tap 4 / 2 / 40; basal 0.7 / 0.4; hypocotyl-borne 0.6 / 0.2; lateral 0.2 | - | OpenSimRoot bean [F] |
| `secondary_growth` flag | true for bean, false for wheat | - | botany: grasses have no cambium [K] |

Bean, pole bean and wheat files all get the keys (wheat with the flag off), so the
species schema bumps.

### 3.6 Done when (R1a)
- The hypocotyl and basal internode widen through the season, the stem tapers from base
  to tip, and the tap root tapers from the collar, in `organs.csv` of single_pot.
- Basal stem diameter at R6 inside the band of §6, and stem mass keeps rising through
  pod fill.
- Root mass at seed fill at least 1.5× today's on four-seed means (0.58 g at day 60
  today).
- Wheat is unchanged except through the shared code (bit-identical if the flag is off).
- Every M0 gate passes; ledgers close at 1e-9; the pole height checks still pass.

## 4. R1b: storage, exudation and seed-driven nitrogen

### 4.1 Starch store
New per-organ column `storage: Vec<GramsC>` for internodes and root axis segments (not
lateral cohorts; fine roots store little). Storage is carbon that does not diffuse and
does not count in the concentration `C = R / W` used by transport, the sink feedback
and growth.

Hourly, inside the growth step (no reorder):

1. **Release first**, so seeds can draw on it through transport:
   if `C_i < C_off`, `release = min(k_r · S_i · Δt, (C_off − C_i) · W_i)`, added to `R_i`.
2. **Refill** as a sink at priority 0.2 (the §3.9 "reserve refill" value):
   demand `= k_s · (S_max,i − S_i) · Δt`, with `S_max,i = s_max · W_i · c_f` and only
   when `C_i > C_on`.

`C_on > C_off`, so the store does not flicker. Wheat gets the store too: stem water-soluble
carbohydrates are its main reserve for grain fill [K, to verify].

At death, storage is treated as reserve (litter, or debt to the parent, as today).
Starch counts as dry matter: the daily and organ CSVs gain `storage_g` columns, and the
organ-class masses read by validation become structural plus storage dry matter
(`S / c_f`), as field dry mass includes non-structural carbohydrate. Keep the structural
masses as separate columns so old numbers stay comparable.

### 4.2 Root exudation
Each root organ pays `e × its maintenance respiration` per hour as exudate, from its
reserve, booked as a new `exudation` line in the carbon ledger (carbon leaving the
plant to the soil). `e = 0.67` (OpenSimRoot bean: exudation about two thirds of root
respiration) [F, archive roots note]. Paid like maintenance, so it never takes a reserve
below what maintenance may.

### 4.3 Seed-driven nitrogen stripping
Today mining (nitrogen card step 15) runs only "if still short" after soil uptake, so a
fertilized pot keeps all its leaves until they drop at maturity [M]. In seed fill real
beans move N from leaves to seeds whatever the soil holds (the "self-destruction" of
Sinclair and de Wit 1975) [K, to verify].

Change in daily step 0, from the SeedFill stage on: the seeds' N demand is served
**first by mining**, at the existing CROPGRO rate (`mobilization_rate` 0.12 d⁻¹ of mobile
N, low-light leaves first, nitrogen card step 15) [F]. Soil uptake then covers the rest
of the plant's demand. Before seed fill nothing changes. Effects that follow from
existing rules: leaf N falls, leaf capacity falls (step 20), leaves drop from the base
at the existing threshold (step 19), and the source shrinks during fill, which also eases
the sink limitation from the source side.

This reorders sub-steps inside daily step 0, not the §3.2 step list. State it in the
decision and in the nitrogen card.

### 4.4 Parameters

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| `storage_capacity` (`s_max`) | 0.25 bean stem and root, 0.35 wheat stem | g NSC g⁻¹ DM | ~ calibrated to the stem decline of §6; [K, to verify] for wheat |
| `storage_on`, `storage_off` (`C_on`, `C_off`) | 0.12, 0.06 | g C g⁻¹ DM | ~ between the growth floor 0.05 and the feedback onset 0.10 |
| `storage_refill_rate`, `storage_release_rate` (`k_s`, `k_r`) | 0.05, 0.02 | h⁻¹ | ~ |
| `exudation_fraction` (`e`) | 0.67 | - | OpenSimRoot bean exudates.xml [F] |
| `seed_fill_mining_first` | true | - | CROPGRO order "seeds served first after R5" [F, archive m2-m3 science] |

### 4.5 Done when (R1b)
- Stem dry mass (structural + storage) peaks near R6 and falls 10 to 35% to maturity
  (§6), on four-seed means.
- In fertilized single_pot, leaves drop from the base during seed fill
  (`seed_fill_drops_leaves_from_the_bottom_up` passes there too).
- Mean `feedback_loss` over seed fill at most 0.35 in single_pot (today about 0.7). This
  is a diagnostic, Info gate.
- `root_shoot_seed_fill` passes (0.08 to 0.30).
- Carbon ledger closes at 1e-9 with the storage and exudation terms.
- Every M0 gate passes after the re-tune.

## 5. R1c: phototropism

§3.4 already specifies it: "phototropism bends new internodes toward the
light-weighted direction from the daily coefficients ... small daily rotations, bounded
per day".

- Daily, after the light rebuild (daily step 6), each growing shoot apex gets a light
  vector `v = Σ_h I_dir,h · T_h · ŝ_h + I_dif · Σ_s w_s · T_s · d̂_s`, from extra rays
  cast from the apex position (24 sun directions, 16 sky sectors). That is 40 rays per
  apex, a few apices per plant: negligible next to the leaf rays.
- In daily step 3, a new internode's direction is the parent direction rotated toward
  `v̂` by `min(θ_max, k_p · angle(u, v̂))`, plus today's gravitropism and noise.
- Only new internodes bend; grown ones don't move (as today's rule for orientation).
- Parameters: `phototropism_weight` `k_p` 0.3 (~), `phototropism_max_per_day` `θ_max`
  10° (~) [K, to verify against bean seedling bending data].
- Done when: a single_pot plant next to a wall obstacle leans away from it (the tip's
  horizontal offset points away from the wall), and its leaves get more light than with
  phototropism off.
- Viewer: once the sim leans the stems, the WP20 drawn lean should shrink or switch off
  so the plant does not lean twice. Tell the viewer thread in the PR.

## 6. Validation targets to add (Science research to confirm the [K] values)

| Id (proposed) | Scenario | Variable | Band | Source |
| --- | --- | --- | --- | --- |
| `stem_share_peak` | single_pot | stem / (stem + leaf) dry mass at R6 | 0.35 to 0.55 (sim leaf includes petiole, field stem includes petiole) | UFGA8501 Seafarer and Isabella (0.59, 0.66 with petioles) [F]; band [I] |
| `stem_decline_fill` | single_pot | stem dry mass, R6 peak to maturity | −10 to −35% | UFGA8501 (−36%, −33%, including petioles) [F]; band [I] |
| `basal_stem_diameter` | single_pot | hypocotyl diameter at R6 | 5 to 10 mm | [K, to verify] |
| `tap_root_diameter` | single_pot | tap root diameter 5 cm below the collar at R6 | 3 to 8 mm | [K, to verify; shovelomics measure it, Burridge et al. 2016] |
| `feedback_loss_fill` | single_pot | mean daily feedback loss over seed fill | at most 0.35 | [I], diagnostic |
| existing `root_shoot_seed_fill` | single_pot | root:shoot at seed fill | 0.08 to 0.30 | Pinzón 2024, Acosta-Díaz 2007 [F] |

Data asks for Science research: the petiole share of bean leaf dry mass (to compare
stem shares fairly), bean hypocotyl and tap root diameters over the season, bean stem
NSC concentration through fill, wheat stem WSC peak and decline, and a herbaceous
cambial rate for `g_max`.

## 7. Cost

- R1a: one daily tip-to-base pass (O(organs)) and one more demand term per stem and root
  organ per hour inside the existing loop. Expected under +3% on single_pot and
  plot_30_beans [I]. Measure with `ps-cli bench` before and after.
- R1b: two more terms per stem and root organ per hour; same order.
- R1c: 40 rays per growing apex per day: well under 1% of the light rebuild.
- No new soil or light grids, no step-order change in §3.2. Same cost at every scale,
  so no simplified level is needed (review §4.1).

## 8. Interactions and risks

- **Re-tune.** R1 moves the carbon budget. The re-tune (model 0.15.0) stays (its items
  are physiology), but its seed-mean calibration needs a pass after R1b.
- **Pole bean height.** Held by the primary-radius conductance rule (§3.4). Check
  `pole_height_day60`, `pole_height_day75` and `pole_height_final` on seed means.
- **Shade response.** Shaded plants have less leaf area, so less pipe target; their
  stems stay thinner. The shade internode M0 gate compares lengths, not mass, so it
  should hold; check it.
- **Bigger plants.** More sinks absorb sugar that is discarded today, so total mass may
  rise toward the top of 20 to 40 g. Watch `total_dry_mass` and the logistic shape.
- **Heat and drought seed checks** may move toward their bands, because stored starch
  now matters for seed filling. Re-triage R6's shoot items after R1b.
- **Viewer.** Radii per organ are already in the snapshot, so taper shows up without
  viewer work. The sim lean (R1c) needs the viewer's drawn lean reduced.
- **Model cards to update:** organ-growth (allometry split), carbon-allocation (sinks,
  storage, exudation, conductance rule), herb-architecture (thickening, phototropism,
  and fix the stale "leaves don't senesce from the base" line), nitrogen (mining order),
  grass (storage, no thickening), respiration (exudation).

## 9. Tests (next to the code)

- `ps-graph`: with `W_s = 0` the allometry is bit-identical to today's; adding `W_s`
  keeps length and widens the radius; structural mass never shrinks.
- `ps-arch`: the pipe target never falls when a leaf dies; the hypocotyl gets the
  largest target; a grass gets no thickening.
- `ps-physio`: storage refill and release conserve carbon at 1e-12 per hour; the store
  never exceeds `S_max` or goes negative; release only below `C_off`.
- Nitrogen: in a fertilized plant, mining runs in seed fill and leaves drop bottom-up.
- Phototropism: a single apex with light from +x leans toward +x and never more than
  `θ_max` per day.
- Ninety-day: the §6 targets on four-seed means through `ps-cli validate`.
