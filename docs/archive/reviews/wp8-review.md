# WP8 review: herb architecture and roots (PR #27, head fc3c2a9)

Reviewed on the PR branch. Files: crates/ps-arch (lib.rs, shoot.rs, roots.rs, geometry.rs, params.rs, tests, tests/ninety_days.rs), the `shed` refactor in ps-physio, species schema additions, decision 0013, docs/models/herb-architecture.md.

## Checks (all green on the branch)
- fmt, clippy pedantic, cargo test --workspace (164 tests; the 90-day runs for eight seeds take about 13 s), check_deps (11 crates), check_determinism.

## Done-when (plan section 7, WP8)
Met. A 90-day run gives counts inside the section 3.16 ranges for world seeds 1 to 8:

| Quantity | Seeds 1 to 8 | Plan 3.16 |
| --- | --- | --- |
| Main-stem nodes | 7 | 6 to 8 |
| Branches | 3 to 5 | 3 to 6 |
| Pods | 11 to 17 | 10 to 20 |
| Seeds per pod | 3.5 to 3.9 | 3 to 6 |
| Living dry mass at day 90 | 19.5 to 27 g | 20 to 40 g |
| Height | 39 cm | 30 to 50 cm |
| Maturity | day 72 to 79 | 65 to 93 days |
| Flower abortion | 53 to 65% | 45 to 80% (research review) |

Bit-identical replay for the same seed, roots stay in the pot, a dry seed does not germinate. The research checklist items are all present: radicle plus two basal whorls plus hypocotyl-borne roots (19 axes), specific root length from diameter and tissue density per root class, phyllotaxis as a parameter (180 degrees), pod set calibrated into the abortion range, seed fill starting halfway through pod expansion, and `leaf_capacity` fed the recorded thermal age at full expansion.

## The two places you asked about

### Sink feedback on leaf capacity (decision 0013 item 7)
Acceptable as an M0 safety valve, with three conditions.

1. **It masks a sink deficit; say so and measure it.** The symptom (30 g C reserve against 17 g C structure) means the plant has nowhere to put carbon: no stem thickening, roots at 4% of dry mass (bean is nearer 8 to 15% at flowering), fixed leaf count, pods capped by flowers. The plan's priority table has "secondary thickening 0.4" and "reserve refill 0.2" for exactly this surplus, and neither exists yet. The feedback makes the biomass targets pass by throttling the source rather than growing the sink, which will show in the lenses as midday assimilation near zero on a sunny day. Please add a daily diagnostic: gross assimilation lost to the feedback as a fraction of potential, in `DayReport` and the CSV. If it is more than about 20% over the season on the standard scenario, WP12 should add stem thickening and more root allocation before touching the thresholds.
2. **Do not let the feedback scale Rd.** `capacity` multiplies Vcmax, Jmax and Rd together in `kinetics`, which is right for aging but wrong for end-product inhibition: a leaf full of sugar does not respire less, if anything more. With the feedback near zero the leaf costs nothing to keep, so it can never starve and shade senescence loses its carbon basis. Fix: apply the feedback to gross assimilation only, `A_net' = f (A_net + Rd) - Rd`, which needs `Rd` in `Rates` (cheap) and leaves the table as it is.
3. **Thresholds.** Onset 0.10 g C per g DM is about 22% non-structural carbohydrate in leaf dry mass; bean leaves reach 10 to 20% at dusk, so onset is plausible and "full" at 0.30 is generous. Both are "~" and in the species file, good. Record in the gas-exchange card that this is provisional (decision 0013) and outside section 3.8's text.

### Primordia at 2% of potential mass (item 2)
Justified, and still needed with the implicit transport: a near-zero-mass organ equilibrates to its parent's concentration each hour, so its reserve and therefore its growth are proportional to its own mass, roughly 6% per hour at a 0.1 g C per g DM parent concentration, five days from a microgram to a milligram. The 2% jump start removes that lag; the cap at half the parent's reserve is sensible; moving reserve into structure leaves the stock unchanged, so the ledger is fine. Two small points: the primordium is built without growth respiration (construction cost 1.25 is skipped), about 0.4% of the organ's eventual carbon, so either charge it or note it; and the primordium is not booked through `record_demand`, which is right (it is not demand).

## Other findings (no bugs that break the done-when)

1. **Low: leaf lifespan is measured from birth, not from full expansion.** The schema defines `leaf_lifespan` as "after full expansion" and `leaf_capacity` uses it that way, but `senescence` sheds a leaf when `thermal_age >= leaf_lifespan`. With 150 Cd of expansion that is a quarter early and inconsistent with the capacity decline, which would still be at 1.0 at shedding time... no, it would be partway. Use `expanded + leaf_lifespan` when `expanded` is known.
2. **Low: the 3.16 biomass target should be compared at peak, not at day 90.** `dry_mass` counts living structure; at maturity all leaves have dropped, so 19.5 to 27 g is shoot minus leaves. The plan's 20 to 40 g is total plant mass, which in the literature includes leaves at or before maturity. Report peak living mass (or living plus shed) alongside, so WP11 compares like with like. Fruit at 85% of living mass is the same artefact; harvest index on total biomass will be nearer 0.55, which is right for bean.
3. **Low: roots.** 4 to 5% of dry mass is low (noted in known limits). Besides allocation, check that lateral cohorts are not under-counted in mass: a cohort's potential is `max_length x multiplicity / SRL` with the lateral type's max length, so cohorts near the pot wall hold mass for length they cannot have (the test accepts 5% outside). Fine for M0.
4. **Low: every seed gives exactly 7 nodes and 39 cm.** Expected with zero allele variance and a fixed internode profile, but the height is also insensitive to carbon because internode potential length is fixed and only mass varies. The research review's CROPGRO height profile is a shape, not a cap; worth a note that height variation arrives with genetics (M3), not with environment, unless internode length is made to respond to unmet demand like leaf area does.
5. **Nits.** `water_demand` builds the 68-solve class table at night too; a dark-hour shortcut (one solve for Rd) would save about a third of the plant's hourly cost once there are 30 plants. `flowering()` clones `HerbParams` daily; harmless. The `Meta.light_index` default of 1.0 makes a bud under a brand-new leaf see full light on its first day; it is overwritten after the first rebuild.

## Plan consistency
- Step order (daily 1 to 6, hourly 3 to 12) follows section 3.2. Water stress acts in two places only. Germination follows 3.10 (seed cell FTSW and soil temperature for consecutive hours; the plant's thermal time runs on soil temperature until emergence, a nice touch). Shade senescence uses the daily light index as decision 0013 defines; bud activation and pod set follow 3.4 and 3.11 with the CROPGRO temperature response from the research review.
- Section 3.8 gains the sink feedback and section 3.10 the four-organ seed; both need a line in the plan with a pointer to decision 0013 when the next docs PR goes in.

Verdict: merge; fold the Rd fix (feedback on gross assimilation), the lifespan-from-expansion fix and the two diagnostics into the WP9 PR.
