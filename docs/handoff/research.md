# Research workstream: handoff

Written 2026-11-05 by the research thread ("Research plant simulation science"). On the new
account the research files live in the repo under `docs/archive/research/` (the handoff thread
copied them from `/mnt/project-files/research/`). Paths below are relative to that folder.

## Index of reports

| File | One line |
| --- | --- |
| `plant-simulation-science-review.md` | Round 1 (published as "Plant Simulation Science Review"). Plan v3 against FSPMs (GreenLab, L-PEACH, Helios, OpenAlea), crop models (CROPGRO-Drybean, WOFOST, APSIM), Farquhar/Medlyn, light, roots (OpenSimRoot), bean data sets and procedural visuals. 34 ranked changes by WP, a bush-bean parameter table, validation data sets, WP8 architecture parameters. Corrected in place on 2026-11-05: nodule fixation costs about 7.1 g C/g N, not 2.8. |
| `m2-m3-bean-science.md` | Round 2 (published as "M2 and M3 Bean Science"). Pole bean twining, spring wheat, soil N and bean fixation (WP24), bean genetics for M3 breeding (named loci, heritabilities, linkage, selfing, hand crosses, mutation), plot competition and density data. 39 ranked changes, 17 tests with pass criteria. Its six owner decisions were approved and recorded in decision 0023. |
| `notes/` | Round 1 notes per topic: fspm_carbon_allocation, photosynthesis_light, crop_models_bean, roots, bean_data, procedural_visuals_games. |
| `notes-m2-m3/` | Round 2 notes per topic: pole_bean_grass, nitrogen_fixation, bean_genetics, competition_validation. |
| `source-files/dssat-bngro048/` | DSSAT CROPGRO-Drybean v4.8 BNGRO048.SPE/.CUL/.ECO, the main source of bean parameters. DSSAT-CSM is BSD-3-Clause. |
| `source-files/opensimroot-bean/` (not copied; see source-files/README.md) | OpenSimRoot "Bean-Carioca-SimRoot4" root parameter XML. **GPLv3: use the values, but don't vendor these files into the code or data/.** If licence hygiene matters, leave them out of the repo and refetch from gitlab.com/rootmodels/OpenSimRoot (InputFiles/plantParameters/Bean/Carioca/). |

Every claim in the reports and notes carries a tag:
- **[F]**: the source was read in a research session.
- **[K, to verify]**: from model knowledge and not re-read; check it before putting the number in `data/`.
- **[I]**: inference or arithmetic.

## How the workstream runs (reusable instructions)

1. **Scope from the plan and the code.** Read `docs/design-plan.md` (the relevant §3 and §7 rows), `docs/STATUS.md`, the species RON in `data/species/` and recent decisions. Compare research against the values as merged, not only the plan text.
2. **Fan out by topic.** The owner explicitly asked for deep research. Four to six parallel researchers, one per topic, each writing notes in a fixed format: takeaway, cited findings with tags, inferences, gaps. Then one writer synthesizes. The writer leads with a ranked change table: plan section, current value, recommended value, source, confidence, grouped by WP. Separate tables follow for parameters, validation data and tests. Owner decisions are called out, each with a recommendation.
3. **Publish and file.** Publish an HTML artifact for the owner, and put markdown in the research folder (now `docs/archive/research/` in the repo).
4. **Turn findings into decisions.**
   - Send the implementation thread a compact message keyed to WPs: values, units, source, and "fix now" vs "later". It adopts items through decision records (for example 0011 for round-1 physiology, 0023 for the round-2 owner decisions) and cites "research review item N" in the species file `source` fields.
   - Send the review thread the checks to apply when reviewing WP PRs.
   - Anything that changes the plan's design goes to the owner as a short list with recommendations. Only the owner's own words count as approval.
5. **Send early.** When a finding affects the WP being built right now, message it before the full report is done. Round 1 did this for WP7 and WP8 and saved a rework cycle.

### Sources: what works and what's blocked from cloud containers (as of 2026-11-05)
- **Work:**
  - PyPI (`pip download`). The DSSAT bean files came from the `DSSATTools` 3.0.2 wheel and the WOFOST crop files from the `pyswap` 0.3.9 wheel.
  - GitLab raw (OpenSimRoot).
  - raw.githubusercontent.com (worked in round 2; blocked for some repos in round 1).
  - algorithmicbotany.org PDFs, rdrr.io package docs, core.ac.uk open PDFs, Oregon State extension PDFs, maxapress, k-state.edu.
  - The Consensus, SciSpace and PubMed MCP search tools, for abstracts.
- **Blocked or flaky:**
  - The GitHub API and directory listings, so file names had to be guessed. dssat-csm-data experiment files (*.BNX/*.BNT) were never listed.
  - The CRAN tarball, jsdelivr, Project Gutenberg via curl.
  - PubMed Central and PubMed HTML pages (captcha), the HAL archive (bot block).
  - Consensus rate limits: the account had 2 searches left on 2026-11-05 and resets 2026-12-01.
- **Licences:**
  - DSSAT-CSM: BSD-3.
  - OpenSimRoot: GPLv3, values only.
  - WOFOST faba bean file: EUPL.
  - Rutjens et al. 2025 climbing-bean model: CC BY 4.0 paper, code at github.com/pmxrr3/maizebeanpolyculture_2023, code licence unchecked.

## Findings not yet acted on (ranked, checked against main on 2026-11-05)

1. **Vcmax/Jmax basis.** `vcmax25` 89.3 and Jmax 160 (Pimentel 2011) are chloroplast-based (gm 0.47). Without gm, Rubisco-limited A runs about 20% high. Either add gm with the Bernacchi 2002 constants, or use Ci-based values (Vcmax about 70 to 75, Jmax about 150). Parked since WP12 (STATUS). Round 1, item 5.
2. **Development temperatures.** `t_base` 8 and `t_opt` 28 are still a single peak. CROPGRO, APSIM and field fits support a trapezoid of 7/25/33/42 °C, with every thermal-time target restated on base 7. This changes the phenology tuning, so it needs a decision record and a version bump. Item 7.
3. **Construction cost.** It is still a single 1.25 g C/g C. The recommendation is per organ: leaf 1.36, stem 1.31, root 1.24, seed 1.40 (CROPGRO composition). Item 9.
4. **Medlyn g1.** Still 4.0. CLM5 uses about 5.8 for C3 crops ([K, to verify]; no bean measurement found). Item 10.
5. **Plot density response is weak.** Measured −25% against −70% (STATUS). Targets:
   - LAI and leaf N per ground area plateau at 20 plants/m² or more, while leaf N per leaf area is unchanged (Clavijo 2019).
   - Plant-mass Gini rises with density only when light competition is on.
   - Per plant, pods, branches and leaf area fall strongly with density, while seeds per pod and seed mass barely change.

   Round 2 has the tests.
6. **Pot-size effect (Poorter 2012).** Biomass rises about 43% per doubling of pot volume, and plants above 1 to 2 g per litre are pot-limited. A field-sized bean (21.8 g) in the M1 pot is about 3.5 g/L. Only the herb-architecture card mentions it. Add it to the validation notes and consider a pot-volume sweep test.
7. **Paraheliotropism.** Bean leaflets turn edge-on to the sun at midday, more so in heat and drought (Fu & Ehleringer 1991, Pastenes 2004). Decision 0025 has no sun tracking yet. It would be an hourly per-leaf incidence multiplier on direct PAR, and it is visible in the viewer.
8. **Smaller items:**
   - Check clay field capacity and wilting point against Saxton & Rawls 2006. The current 0.40/0.22 may really be about 0.42/0.30.
   - Structural carbon resorbed at death should be 5 to 10%, not 30% ([I]).
   - Severe drought should lower Vcmax as well as g1 (Zhou 2013).
   - Bean's photoperiod sensitivity (PPSEN 0 to 0.07 per hour, CSDL 12.17 h) could become heritable loci in M3.
   - Remove `OrganKind::Tendril` at the next version bump (decision 0023).

## Research the next phase needs

- **WP26/27 (M2 lenses, leaf energy balance):**
  - Bean leaf–air temperature differences on hot, still days. Transpiring, edge-on leaves run below air temperature; stressed leaves run above it. This is the WP27 done-when range.
  - Leaf boundary-layer conductance for trifoliate leaflets.
  - Bean leaf optical properties (PAR absorptance about 0.85; reflectance and transmittance not found).
- **M3 breeding (WP28–32):** round 2 covers the basics. Still missing:
  - cM distances for *fin*–*Ppd* (Pv01) and *P*–seed size (Pv07).
  - Primary sources for the chromosome positions of the seed-colour genes C, V, T, J, G, B and Asp (all [K]).
  - The seed-colour epistasis table (Bassett's P/C/J/G/B/V/Rk/T/Z/Bip), worked out as a rule table with example phenotypes and photos.
  - Narrow-sense or realized heritabilities from selection experiments.
  - Variation in Vcmax and root angle among genotypes.
  - Emasculation timing and days from cross to mature seed.
  - Numbers for inbreeding depression and heterosis.
- **Challenges (WP32):**
  - Bean–weed Cousens I and A parameters. The critical period for weed control runs from about V1/16 days to R1/39 days, and uncontrolled weeds cost 31 to 94% of yield.
  - Intercrop land equivalent ratios beyond milpa (about 1.34): bean with grass.
- **Validation data still missing:**
  - An open per-organ weekly time series for bush or snap bean. The best lead is the DSSAT Drybean experiment files in DSSAT/dssat-csm-data, which need GitHub listing access.
  - Per-plant masses from a density trial.
  - Digitized figures from Clavijo 2019 and Pinzón-Sandoval 2024.
- **M4 tree:** not researched yet. Round 1 has the Palubicki shadow-propagation and Borchert-Honda λ notes only.
