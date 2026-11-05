# WP4 review: species schema and the bush bean file (PR #15, merged as d63a8a9)

Reviewed on main after the merge. Files: crates/ps-species (lib.rs, schema.rs, tests.rs), data/species/bush_bean.ron, docs/models/species-and-genome.md, docs/decisions/0006.

## Checks (all green on main)
- cargo fmt --check, cargo clippy --workspace --all-targets -D warnings (pedantic), cargo test --workspace (89 tests, 14 in ps-species), scripts/check_deps.py (5 crates), scripts/check_determinism.py.

## Done-when (plan section 7, WP4)
- "Bean file loads": yes (`bean_file_loads`, 61 parameters, three root types).
- "A wrong unit or missing source fails with a clear error": yes. Messages name the path, for example `constants.carbon_fraction: unit "g DM / g C" is wrong, expected "g C / g DM"`. An absent `source` field, a misspelled key, an unknown heritable key, a locus naming a non-heritable trait, a default outside its bounds and a wrong schema version are all rejected with tests.

## What is good
- The `param_group!` macro makes the schema the single source of truth: field, unit string and doc string in one place, with `SPECS` for tooling and `deny_unknown_fields` so typos fail.
- Every parameter in the bean file has a source, mostly "~" with the plan section it comes from, so WP12 knows what to tune.
- Decision 0006 records the dominance formula and the choice to keep phenology non-heritable in M0, as the plan asks for anything the plan left open.
- Genotype sampling draws two normals per locus even when allele_sd is 0, so switching variance on in M3 does not shift any other RNG stream. Identity mapping is tested for several entities and seeds.
- Round trip through RON serialization is tested, so the file format is stable.

## Findings (no bugs; suggestions in priority order)

1. **Add cross-field sanity checks to `validate`.** A unit typo is caught, but `t_opt: 82.0` or `flowering > maturity` loads silently. Cheap checks that give the same "clear error" promise: `t_base < t_opt < t_max`; `emergence < flowering < maturity`; `growth_t_min < growth_t_opt_low <= growth_t_opt_high < growth_t_max`; `heat_abortion_t > growth_t_opt_high`; `gravitropism` and `hydrotropism` finite (they are raw f64 and skip `check_param`); `appearance` colors match `#rrggbb`. Could ride along with WP5 or any later PR.

2. **Give consumers a typed, indexed view at load time.** `Phenotype` is a `BTreeMap<String, f64>` and every constant is a `Param { value: f64 }`. If WP7 and WP8 call `phenotype.get("vcmax25")` per leaf per hour and spread `.value` f64 through the sim, the ps-units rule ("raw f64 only inside a function body") erodes and the string lookups cost time in the hottest loop. Suggestion: a `Trait` enum (one variant per HERITABLE entry, with its index) and `Phenotype(Vec<f64>)` indexed by it, plus a `Species::typed()` or per-group accessor that returns ps-units newtypes (`Celsius`, `UmolPerM2S`, `Cm2`, `DegreeDays`) built once when the plant is created. ps-units is already a dependency of ps-species but is not used.

3. **Decide the rounding rule for discrete traits.** `habit`, `nodes_before_terminal`, `seeds_per_pod` and `flowers_per_raceme` are continuous in the phenotype. With zero variance this is harmless, but in M3 a habit of 0.37 has no meaning. Record now (model card) that habit is `>= 0.5` and counts are `round()` (half away from zero, deterministic), so the M2 pole bean file and M3 breeding agree.

4. **Parameter values to expect WP12 to move.** `specific_stem_length` 40 cm/g makes a 35 cm main stem weigh about 0.9 g; bush bean stems at maturity are several grams. Something like 10 to 20 cm/g is more likely. `stem_density` and `specific_stem_length` together fix internode length and diameter; WP8 should take length from specific stem length and diameter from density, not derive both from one. All are marked "~", so this is a note, not a defect.

5. **Nits.** The `validate` doc says "first problem found, in file order" but the heritable map is checked alphabetically. The model card's parameter table lists only the genome parameters and points to the file for the rest, which is fine, but a generated table from `provenance()` would keep it from drifting (a small CI script could diff it).

## Plan consistency
- Units, defaults and bounds match plan sections 3.11, 3.12, 3.16 and 6.1. Tap root insertion angle is 0 instead of the plan sketch's 70 (correct: a tap root has no parent). Phenology cardinal temperatures (8/28/38) and expansion temperatures (5/25-30/40) are two separate responses in the plan and are both present.
- Nothing in WP4 changes an equation or step order, so no model version bump is needed. Species schema is versioned separately (schema: 1), as the plan asks.
