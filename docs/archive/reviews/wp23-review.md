# WP23 review: grass module and spring wheat (PR #67, decision 0022)

Reviewed at commit e3ccced on `claude/project-thread-ur2udi` against `origin/main` (model 0.6.0).
Reviewer: Fable review thread. Date: 2026-11-05.

## Verdict

**Approve, with one fix to fold in before the stand numbers are trusted.** The grass path is a
clean second architecture on the shared engine: crown nodes with sheath and blade, tiller sites
decided once on their own stream, the stop at jointing, delayed intercalary elongation, a
peduncle and ear, binomial grain set. The bean goldens match and the implementer says the bean
CSVs are byte-identical (I did not rebuild main this time; the mechanism is a separate code
path behind `is_grass()`, and the goldens agree). All checks are green and the done-when
test holds.

**The fix: every wheat plant's blades point the same way.** `grass_emerge` sets the first
azimuth to a constant (`FRAC_PI_2`, east), the phyllotaxis is 180°, and tillers inherit their
bud's azimuth. In `wheat_stand` the rows run east to west, so every blade of every plant lies
along its row: 200 plants make five leaf fences 27 cm long and about 3 cm wide, with the
inter-row bare. The light grid sees a strip with four to seven times the stand's leaf area
density and nothing between. At day 45 the stand's leaves have an area-weighted light index of
0.42 at a stand LAI of 1.2, where a mixed canopy would be near 0.75. I patched a keyed random
first azimuth per plant (one line, `Purpose::Tropism` on the hypocotyl id) and reran:

| `wheat_stand`, 200 per m² | as built | random azimuth |
| --- | --- | --- |
| tillers per plant | 2.0 | 3.7 |
| ears per plant | 3.0 | 4.7 |
| grains per plant | 33 | 50 |
| peak leaf area per plant, m² | 0.016 | 0.022 |
| total dry mass per plant, g | 3.4 | 4.9 |
| seed per plant, g | 0.88 | 1.26 |
| height, m | 0.40 | 0.42 |

Spaced plants move by a few percent (RNG). So the fence costs the stand about 30% of its mass
and yield and is why the stand looks half a crop (biomass 680 g m⁻² against 1200 to 1800,
LAI 2.7 against 4 to 6). It also means the R:FR threshold was tuned on an artifact: once the
azimuths are random, 4.7 ears per plant is 940 ears per m², above the 400 to 600 of a real
crop, and the tillering stop needs to bite earlier (finding 1). The height does not move,
which confirms the height miss is the transport problem (finding 2). Do the azimuth fix in this
PR or the next, with WALTer's geometry: plant azimuth random, 185° between blades, tillers at
40° from the parent axis, so a plant's blades fan over a season rather than stay in one plane.
Grass only, so the bean lock is untouched.

## What I checked

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| clippy, sim and CLI crates, `-D warnings` | clean (CI covers the Bevy crates) |
| `cargo test` on the sim and CLI crates | all pass, including `spaced_wheat_tillers_heads_and_fills_grain`, `a_dense_stand_tillers_less` and the grass unit tests |
| `scripts/check_deps.py`, `scripts/check_determinism.py` | pass |
| golden hashes (pots 90 d, plot 30 d) | match |
| `ps-cli validate` | 81 checks: 65 pass, 16 Info misses, 0 failing, 0 known; every M0 gate passes |
| diagnostic runs | the two wheat scenarios with a random first azimuth per plant (table above) |

## The three choices

### 1. The tillering stop at 0.6 on the proxy, not WALTer's 0.35

Right to not use 0.35 on this signal: the proxy `1.2 exp(−0.5 LAI above)` is a vertical-column
LAI dressed as a ratio, and it is too dark for a lone plant (its own leaves sit in the column
above its crown) and too bright in a stand (no far-red from the sides). The honest calibration
is Evers 2006's other statement of the same result: tillering stops when the stand's green
area index reaches 0.5 to 0.6. In the run as built the stand's last site fills around day 35
to 40, at a stand LAI of 0.4 to 0.8, which matches Evers. So the acceptance test for this
threshold is "stand LAI on the day the last tiller site fills, 0.4 to 0.7", not a measured
R:FR. Put that in the card and the target note, and re-tune the number after the azimuth fix
(it will need to rise, since the random-azimuth stand reaches 4.7 ears per plant). Two further
points:

- A spaced plant's own column makes the proxy read 0.7 to 1.2 at its crown; with the
  threshold at 0.6 that is safe, but at 0.9 (the proxy value of LAI 0.55) a spaced plant
  would stop tillering from its own shade. That is the proxy's flaw, not the threshold's.
- The real fix is the follow-up the decision already lists: derive the signal from the
  diffuse (sky-view) coefficient the ray tracer already computes per leaf, which sees
  neighbours sideways, and map it to R:FR with Evers' interception relation. Then 0.35 can
  come back as a measured number.

### 2. Wheat phloem conductance at 1500

Acceptable as "~", and the physical argument (a hollow culm with many bundles against a solid
cylinder of the same mass) is better than the pole bean's. But this is now the third species
on a conductance the bush bean set at 500 and the two new ones need 3× of; the stand's
height miss (0.40 m against 0.7 to 1.0, unchanged by the azimuth fix) is the same symptom:
`feedback_loss` is 0.01 to 0.08 in the stand through jointing, so the stand is source-limited,
and the elongating internodes miss their carbon with no catch-up (decision 0009). In real wheat
the growing stem is the dominant sink at jointing, taking half the assimilate, and tillers die
to feed it. The follow-up in 0022 ("stem elongation as a priority sink at jointing") is the
right one and it is the WP7 priority question again, now with three species asking. Before M2
exit the transport rule should be revisited once rather than patched per species.

### 3. Targets: 3 to 10 tillers spaced, 1 to 5 in the stand

The stand band is right: Darwinkel's density series gives 1.5 to 2.5 ears per plant at 200 to
400 per m², and 2.0 tillers (3 ears) sits in it. The spaced band's lower edge is too low. A fed
spring wheat alone in a 40 cm pot carries 5 primaries (T1 to T5 from leaves 1 to 5, since
jointing comes near leaf 7) and several secondaries, so 6 to 12 shoots is typical and 3 is a
stressed plant. The sim's 4.25 means almost no secondaries form: with `site_filling` 0.69 on
five primary sites the primaries alone expect 3.5. Suggest 4 to 12 for the spaced band and a
note that the sim sits at its low edge because secondary sites rarely fill (carbon factor at
the tiller's crown node). The research table's 15 to 40 is winter wheat and ryegrass, agreed.

## Other findings

4. **Phenology is early.** Sown 10 April at 45 N, the plants head at day 60 and mature at day
   91. Real spring wheat there heads in late June and matures in early to mid August, 110 to
   130 days. The species sums (flowering 900, maturity 1500 °C·d base 0 from germination) are
   at the short end; APSIM-Wheat spring types run about 1700 to 1900 °C·d sowing to maturity.
   Mark both "~ to verify" against a named cultivar and add a `maturity_day` Info target
   (100 to 130 d) so the gap shows in the report.
5. **Grains per ear 11 to 13** (field 30 to 40). Grain set is `Binomial(50, 0.85 · heat ·
   carbon_ratio(ear))` and the ear's reserve concentration against a bean reference
   (`pod_set_reserve_reference`) gives about 0.3. Fischer's rule is the documented alternative
   and removes the borrowed parameter: grains per ear = spike dry mass at anthesis × 70 to 100
   grains per g. With 0.45 g of chaff that is 30 to 45 grains. Already a follow-up in 0022;
   the parameter source makes it the next one.
6. **Stand yield and biomass need targets.** Only the height Info target catches the stand's
   undersize. Add Info targets for peak stand LAI (4 to 6), above-ground biomass at maturity
   (1000 to 1800 g m⁻²) and grain yield (300 to 600 g m⁻²), computed per m² from the plot
   area, so the azimuth and priority fixes can be measured.
7. **The crown sits 1 cm above the soil** (the hypocotyl slot carries coleoptile and subcrown
   internode to +1 cm). A wheat crown is 1 to 2 cm below the surface; it matters for frost
   (the card says −8 °C kills leaves, but the crown survives colder) and for where crown roots
   start. Low priority; note it in the card.
8. **All crown roots come as one cohort at 300 °C·d.** Listed as a limit; one set per tiller is
   the follow-up. Root mass is 2% of the plant at day 45 (0.023 g of 1.1 g), the same
   root:shoot problem the bean has (decision 0018), and worth one line in the card.
9. **`default_grass()`** has `tiller_stop_rfr` 0.35, `tiller_delay` 3, `survive_leaves` 3
   against the file's 0.6, 2, 2. It is only reached when a non-grass calls `gp()`, which
   should not happen; make `gp()` return the species values or panic in debug rather than
   carry a second parameter set.
10. **Jointing kills tillers once** (`self.jointed` latch) and suppresses all dormant sites.
    WALTer regresses tillers over 35 to 50 °C·d intervals until anthesis, youngest first,
    which gives the smooth tiller-number curve the papers show. The one-shot cull is fine for
    the done-when; the interval version is small and would help the stand's ear count land.
11. **Blade light points** carry a normal per strip, but the light model still uses the species
    G of 0.5 for all leaves. Wheat blades at 40° drooping to about 5° are closer to erectophile
    (G about 0.4 to 0.45 for a high sun). Another reason to resolve the open G question
    (decision 0018) by reading G from the drawn normals.

## For the record

- Model 0.7.0; species schema 4 (`grass` group, optional, required for `growth_form: Grass`).
  Bean goldens unchanged except through new serialized fields.
- Validation at WP23: 81 checks, 65 pass, 16 Info misses. New: `wheat_tillers_spaced` 4.25
  (pass), `wheat_tillers_stand` 2.0 (pass), `wheat_height_spaced` 0.76 m (pass),
  `wheat_height_stand` 0.40 m (miss), `wheat_density_tillers` −52% (pass).
- Diagnostic: random first azimuth per plant raises the stand's mass and seed by about 44% and
  its ears to 4.7 per plant; height unchanged at 0.42 m.
- Owner's card on the grass choice (spring wheat) is still open per decision 0022; the
  reviewer agrees with spring wheat.

## Follow-up: fixes verified at 4ffdd4d (PR #67)

Random first-blade azimuth per plant (Tropism stream), 185° phyllotaxis, tillers at 40° from
the parent, `default_grass` removed from production, `tiller_stop_rfr` 1.1 on the proxy,
maturity 1700 °C·d (day 104), new Info targets for maturity day, stand ears, peak stand LAI,
stand tops and stand grain. Verified here: fmt, clippy and tests clean, pot goldens match,
validation 86 checks, 67 pass, 19 Info misses, 0 failing.

| `wheat_stand`, 200 per m² | at review | after fixes |
| --- | --- | --- |
| tillers per plant | 2.0 | 1.9 |
| ears per plant (per m²) | 3.0 (600) | 2.9 (570) |
| height, m | 0.40 | 0.56 |
| peak stand LAI | 2.7 | 4.3 |
| tops, g m⁻² | 680 | 730 |
| grain, g m⁻² | 180 | 230 |
| maturity, day | 91 | 104 |

Open after the fixes, all logged as follow-ups: spaced plant 3.75 tillers against 4 to 12
(the proxy's own-column flaw, threshold 1.1 now stops a lone plant's secondaries); tillering
stops at stand LAI about 1.0 against Evers 0.5 to 0.6 (same flaw, other side); stand height,
tops and grain misses stay with the transport/priority follow-up. Approved for merge.
