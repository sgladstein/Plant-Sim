# Validation reference data (data/validation/)

The published numbers that the WP11 harness (`ps-cli validate`) checks a run against. This
file is the schema contract between the data and the harness; changing a field or a variable
name is a `schema` bump. Design plan §8 lists the targets; §8.3 gives the pass rules.

Every number carries a unit and a source key. Sources live once in `sources.ron`; every
other file refers to them by key. A value that we inferred rather than read (a unit
conversion, a figure read by eye) says so in `note` or `method`.

## Files

| File | Contents | Root type |
| --- | --- | --- |
| `sources.ron` | Every paper, data set or model file cited, with DOI, URL, licence, cultivar and conditions | `Sources` |
| `targets.ron` | Whole-plant range targets (§8.2) for the `single_pot` scenario | `ValidationTargets` |
| `responses.ron` | Response targets (§8.3) for the `response/` scenarios against `control` | `ValidationResponses` |
| `parameters.ron` | Published values for species-file parameters (Vcmax, SLA, cardinal temperatures ...) | `ParameterChecks` |
| `curves/<set>/*.ron` | Time series and model curves, one `Curve` per file, grouped by source (`dssat_*`, `pinzon2024`, `maldonado1993`, `cropgro`) | `Curve` |

## Types

Written as RON with externally tagged enums, so they map one to one onto serde types.

```ron
Sources(
  schema: 1,
  sources: {
    "kumar2025": Source(
      citation: "Kumar A. et al. (2025) ... Journal, vol(issue): pages.",
      doi: Some("10.xxxx/yyyy"),         // None when the item has no DOI
      url: "https://...",                // what we actually read
      licence: "CC BY 4.0",              // or "not stated"; values are cited as facts either way
      cultivar: "...",
      conditions: "field, Kharif season, ...",
    ),
  },
)

ValidationTargets(
  schema: 1,
  species: "bush_bean",
  targets: [
    Target(
      id: "height_at_maturity",         // unique within the file, snake_case
      scenario: "single_pot",           // data/scenarios/<name>.ron
      variable: "height_m",             // a daily.csv column or a derived variable (below)
      at: Stage(Mature),                // when to read it (below)
      expect: Range(min: 0.25, max: 0.45),
      unit: "m",                        // unit of expect, always the variable's own unit
      basis: None,                      // e.g. Some("mean-method degree-days, base 10 C")
      sources: ["kumar2025"],             // one or more keys into sources.ron
      gate: M0,                         // M0: fails the build once WP12 locks; Info: reported only; Later: scenario not written yet, skip
      note: "Range across varieties and sowing dates; plan quotes the mean 35 cm.",
    ),
  ],
)

ValidationResponses(
  schema: 1,
  species: "bush_bean",
  responses: [
    Response(
      id: "drought_pods",
      scenario: "response/drought_from_flowering",
      baseline: "response/control",
      variable: "pods",
      at: Final,
      expect: Cited(rel_change: -0.45), // see Expect below
      sources: ["gomes2000"],
      gate: M0,
      tag: Emergent,                    // Encoded | PartlyEncoded | Emergent (plan §8.3 "Tag")
      note: "...",
    ),
  ],
)

ParameterChecks(
  schema: 1,
  species: "bush_bean",
  checks: [
    ParamCheck(
      parameter: "vcmax25",             // heritable key, constants field, or "phenology.<field>" in data/species/bush_bean.ron
      expect: Range(min: 60.0, max: 120.0),
      unit: "umol/m2/s",
      basis: Some("Ci basis, 25 C"),
      sources: ["pimentel2011"],
      gate: Info,
      note: "...",
    ),
  ],
)

Curve(
  schema: 1,
  id: "pinzon2024_leaf_mass_season1",
  source: "pinzon2024",
  conditions: "greenhouse, season 1, ...",
  x: Axis(variable: "days_since_sowing", unit: "d"),
  y: Axis(variable: "leaf_mass_g", unit: "g/plant"),
  basis: None,
  method: "Table 2" ,                   // or "digitized from Fig. 3, about +/- 5% of full scale"
  gate: Info,
  points: [ Point(x: 14.0, y: 0.41, sd: None), Point(x: 28.0, y: 2.10, sd: Some(0.3)) ],
)
```

### `at`: when a value is read

| Variant | Meaning |
| --- | --- |
| `Final` | The last row of the run |
| `Day(n)` | The row with `day == n` (day equals days after sowing) |
| `Max` | The largest value over the run (peak leaf area, for example) |
| `Stage(s)` | The first row whose `stage` is `s`; `s` is the Rust `Stage` enum: `Sown`, `Germinated`, `Emerged`, `FirstTrifoliate`, `Flowering`, `PodSet`, `SeedFill`, `Mature` |

A scenario's plants are replicates: a value is the mean over the plants that have it, and at
least half must have it (a pot that never matures, or a plant with no pods for
`seeds_per_pod`, drops out; the report notes "mean of k of n plants"). An M0 check that
cannot be evaluated fails the build like a miss.

### `expect` for targets and parameters

| Variant | Passes when |
| --- | --- |
| `Range(min, max)` | `min <= value <= max` |
| `Approx(value, rel_tol)` | `abs(sim - value) <= rel_tol * abs(value)` |
| `LogisticR2(min_r2)` | A three-parameter logistic `K / (1 + exp(-r (day - t0)))` fitted to the variable's daily course (first plant, every day it is defined) has `R² >= min_r2` (plan §8.3 shape rule); `at` is ignored |

### `expect` for responses

`rel_change = scenario / baseline - 1`, both read with the same `at`.

| Variant | Passes when |
| --- | --- |
| `Decrease` / `Increase` | the sign of `rel_change` matches |
| `RelChange(min, max)` | `min <= rel_change <= max` |
| `Cited(rel_change: c)` | the sign matches and `abs(rel_change)` is within a factor of two of `abs(c)` (plan §8.3 pass rule) |
| `FallsMoreThan(v)` | `rel_change` of this variable is below the `rel_change` of variable `v` |

### Derived variables

Computed by the harness from daily.csv (and organs.csv where stated). Anything not listed is a
daily.csv column used as is.

Two daily.csv mass columns matter here. Both include roots and count structural dry matter only:
`dry_mass_g` is the living plant on that day; `total_dry_mass_g` is everything the plant has ever
built, including shed leaves, closed flowers and cotyledons. `fruit_mass_g` is pod walls plus
seeds. Targets use `total_dry_mass_g` for cumulative biomass and `dry_mass_g` where the paper
weighed standing plants; each target's note says which and why.

| Name | Definition | Unit |
| --- | --- | --- |
| `thermal_time_cd_bN` | Cumulative sum from day 0 of `max(0, (t_min_c + t_max_c)/2 - N)`, N in °C with at most one decimal (for example `thermal_time_cd_b10`, `thermal_time_cd_b4.4`) | °C·d |
| `sla_cm2_per_g` | `leaf_area_m2 * 1e4 / leaf_mass_g` | cm²/g |
| `days_since_emergence` | `days_since_sowing` minus its value on the first row with stage `Emerged` | d |
| `vegetative_shoot_mass_g` | `leaf_mass_g + stem_mass_g` | g |
| `shoot_mass_g` | `leaf_mass_g + stem_mass_g + fruit_mass_g` (living above-ground structure) | g |
| `root_shoot_ratio` | `root_mass_g / (leaf_mass_g + stem_mass_g + fruit_mass_g)` | g/g |
| `mean_seed_mass_g` | `seed_mass_g / seeds` | g |
| `harvest_index_tops` | `seed_mass_g / shoot_mass_g` (the DSSAT HIAD convention: seed over living above-ground mass) | g/g |
| `seeds_per_pod` | `seeds / pods` | - |
| `nar_g_m2_day` | Net assimilation rate over the trailing 7 days: `(total_dry_mass_g[d] - total_dry_mass_g[d-7]) / (7 * mean(leaf_area_m2[d-7..=d]))` | g/m²/d |
| `leaf_area_per_leaf_cm2` | `leaf_area_m2 * 1e4 / leaves` | cm² |
| `leaf_share_new_veg`, `stem_share_new_veg`, `root_share_new_veg` | Share of new vegetative structure over the trailing 3 days: for leaf, `Δleaf_mass_g / (Δleaf_mass_g + Δstem_mass_g + Δroot_mass_g)` with `Δ` = value on day d minus day d-3. Undefined (skipped) when the denominator is not positive | g/g |
| `mean_internode_cm` | Mean `length_m * 100` over live organs.csv rows with role `Stem` (end of run only, so `at: Final`) | cm |
| `day` | The sim day (days after sowing); the x of a curve whose axis is `days_since_sowing` or `days_after_sowing` | d |
| `day_of_<Stage>` or `day_of(<Stage>)` | The first day at or past the stage | d |
| `height_cm` | `height_m * 100` | cm |
| `leaf_area_cm2` | `leaf_area_m2 * 1e4` | cm² |
| `harvest_index` | `seed_mass_g / (total_dry_mass_g - root_mass_g)` | g/g |

A curve may name `scenario` (default `single_pot`). Its comparison reports NRMSE (RMSE over
the mean observed value) and shape NRMSE (each series divided by its own maximum first),
since field curves at stand density compare in shape and timing only.

The thermal-time variables let a target keep the base temperature its paper used: a target in
`thermal_time_cd_b10` is compared against a sum the harness builds on base 10 °C from the
run's own daily temperatures, regardless of the base the species file uses.

## Conventions

- Targets are ranges, not points, because cultivars differ (plan §8.2). Where a paper reports
  one cultivar mean, the range is built as stated in `note` (usually the spread across
  cultivars, seasons or sowing dates in that paper).
- `gate: M0` marks the targets the M0 exit list in plan §7 names. Everything else is `Info`
  until WP12 decides it is reliable enough to gate.
- A value read off a figure is marked in `method` ("digitized from Fig. 2") with its rough
  precision. A value read from a table says which table.
- Licences: we cite values as facts with attribution. No file from a source is copied.

## What is in the set (2026-11-05)

- **Range targets (32):** 6 gated for M0 (flowering day, maturity day, total dry mass, pod
  count, height, peak leaf area), the rest Info: stage thermal sums on each paper's own base
  (8.3, 10, 7 and 4.4 °C), root:shoot ratio at seed fill, main-stem nodes, seeds per pod, seed
  size, harvest index, SLA, NAR, leaves and branches.
- **Responses (23):** drought from flowering, heat at flowering and 50% shade against the
  control, with the plan §7 M0 exit checks gated; one M2 density check marked `Later`.
- **Parameter checks (14):** Vcmax, Jmax, Rd, SLA, leaf size, seed mass, seeds per pod,
  flowers per raceme, pod set, phyllochron, frost kill, base temperature and the two
  phenology sums.
- **Curves (61):** measured per-organ time series for five Type I dry beans and one snap bean
  from the DSSAT CROPGRO experiment files (leaf, stem, pod, seed and shoot mass, leaf area,
  main-stem leaves, and height for the snap bean), Pinzón-Sandoval 2024 root, shoot and pod
  logistic fits against thermal time, Maldonado 1993 leaf area per plant, and the CROPGRO
  reference leaf area and partitioning tables.

## Findings for WP12

- **Flowering is early in the species file.** Converted to the file's base of 8 °C, the
  published flowering sums run from about 310 (Dapaah 1999) through about 475 (CROPTIME,
  temperate snap beans) and 485 (Cavalcante 2020) to about 565 (Pinzón 2024), counted from
  germination. Each conversion adds (base - 8) x days to the stage and takes off about 30 °C·d
  for sowing to germination; the day counts are in `parameters.ron`. The default 320 sits at
  the bottom of that spread, and the review run confirms the sim fails the Cavalcante, Pinzón
  and CROPTIME flowering sums.
- **Pinzón 2024's base temperature is 8.3 °C**, so its ~800 °C·d is close to the species file's
  base. Cavalcante 2020 gives a Type I total of 826.8 °C·d above 10 °C in 75 days. It is Info,
  not a gate: converted to base 10 the other maturity sums sit below it (Pinzón about 680,
  Dapaah about 500-530), so the maturity gate is the day count.
- **Kumar 2025 could not be found** by title in SciSpace, PubMed or web search, so its height,
  leaf area, dry mass and yield numbers stay as unverified plan quotes. Each gated target that
  uses them has at least one verified second source.
- **Gebeyehu 2019 and Merga 2021 were not found either.** The plan's SLA and NAR numbers match
  Desalew 2021 and Mohammed 2018 from the same Ethiopian site, which replace them.
- **Rose 2023's 31/24 °C is a treatment, not a damage threshold**, so the plan's wording in the
  research review should change. Hot nights can raise pod number while the pods stay empty
  (Vargas 2021), so heat checks lean on seeds and seed mass more than on pods.
- **Pimentel 2011 does not state whether Vcmax is on a Ci or Cc basis.** Treat 89.3 as Ci-based
  until Table 1 can be read.
- **Field data are at stand density** (25-52 plants/m²), so per-plant masses from the DSSAT
  curves are smaller than a lone pot plant's; use them for shape and timing.

## Gaps still open

- A per-organ weekly series for a lone pot-grown Type I plant: Pinzón 2024 Figs 4-5 and Lépiz
  Ildefonso 2015 ('Peruano Mostaza', e-CUCBA) need someone to open the PDFs and digitize the
  figures; the publisher hosts are blocked from this environment.
- A single bush-bean study at about 50% shade cloth reporting internode length, SLA, dry mass
  and pods together. Cossu 2021 lacks stem length; Lopes 1987 lacks SLA.
- A measured percentage for leaf area under drought from flowering (Gomes 2000 has it only in
  a figure).
- Hanyu 1996 numbers (J-STAGE blocks the PDF), Gross & Kigel 1994 and Prasad 2002 full texts
  (paywalled), Sharda 2026 supplementary tables (filled pods and seed number).
