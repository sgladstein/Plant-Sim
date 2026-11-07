# 0045: Dry soil limits roots, and the water network has a simplified level (R3c)

- **Date:** 2026-11-06
- **Status:** accepted (the realism plan's R3c, owner-approved queue; the owner's
  level-of-detail rule of 2026-11-06: full detail when zoomed on a few plants at slow
  speed, simplified for big plots and fast-forward, outcomes similar)

## Context
R3b (decision 0044, PR #141) solves each plant's water network every hour. Code review
of it found that the root–soil link kept its full conductance down to the wilting
point (`f_dry` = 1 there), while the soil around a root conducts less than the root
itself below about −0.3 MPa in loam: roots in dry soil took and leaked water as if it
were wet. It asked for a soil conductance in series as R3c's first change. The network
also cost about 50% more time per pot-day, over the ~4 ms budget, and the realism
plan's R3c (`docs/research/realism-plan/r3-plant-hydraulics.md` §6 and §7,
`docs/research/realism-plan/fidelity-levels.md`, which wins where they differ) adds a
simplified level for plots and fast play, switching by logged actions, and a report
that holds the two levels' outcomes together.

## Decision
1. **Soil conductance in series with each root link** (plant-hydraulics card
   equation 1). Per metre of root, `k_soil = 2π K(θ) / (ρ g ln(r_b / r))` (Gardner
   1960: steady radial flow from half the distance between roots, `r_b = 1/√(π RLD)`
   with this plant's root length density in the cell, at least `2r`), and the link is
   `L · k_root k_soil / (k_root + k_soil)`. `K(θ)` is Mualem's conductivity on the R3a
   retention curve (`Retention::psi_and_conductivity`, which shares a power with the
   potential) times Carsel and Parrish's (1988) saturated conductivity per texture
   (loam 24.96, sand 712.8, clay 4.8 cm d⁻¹). It replaces R3b's `f_dry`. Using each
   plant's own root density keeps plants independent within the hour, as the joint
   uptake requires.
2. **Two levels per plant** (`ps_arch::HydraulicsLevel`, in the plant's state and so
   the hash). Detailed is R3b's hourly solve. Simplified (Couvreur et al. 2012) solves
   the same network once a day with every root cell at one potential and the
   transpiration shared by leaf area; it keeps `K_rs`, each root cell's standard
   uptake fraction and each organ's drop per unit transpiration. Each hour it takes
   `U_c = E·SUF_c + K_comp·SUF_c·(ψ_c − ψ_seq)` from each cell, `ψ_seq = Σ SUF ψ_c`,
   `K_comp = c·K_rs` with `c` = 1 (species key `hydraulic_compensation`, schema 18),
   and sets every organ at `ψ_seq − D_i E` (a leaf less its own `E_i / (K_leaf A)`).
   The summary is built every day at both levels, so a switch converts nothing.
3. **Which level.** The scenario's `fidelity`: `Auto` (default) is detailed when it
   sows at most 8 seeds at the start (`MAX_DETAILED`, a guess for Performance),
   simplified above; `Detailed`, `Simplified`, or `Locked(level)`, which also refuses
   `SetFidelity`. A challenge's `fidelity: Locked(level)` locks its scenario (off by
   default; none is locked). Plants sown later take the scenario's level. At `Auto`
   every validation pot and response experiment (at most 8 pots) runs detailed;
   plot_30_beans, m2/plot_low_n, both density plots and the wheat stand run
   simplified.
4. **Switching.** `ActionKind::SetFidelity { plants, level }` (an empty list for all)
   changes plants' level from the next hour. The viewer will write it as zoom and speed
   change (its rule is the viewer's, not sim state); a replay of the log is exact.
   `ps_game` counts it as no player action, so it never breaks a challenge rule.
5. **Outputs.** The daily CSV gains `root_conductance_kg_h_mpa` (`K_rs`) and
   `uptake_depth_m` (the uptake fractions' mean depth); the lens gains the plant's
   level and its uptake share per layer.
6. **The fidelity report** (`ps-cli fidelity`, a CI job): single_pot,
   response/drought_from_flowering, m2/split_pot, plot_30_beans, m2/plot_low_n and
   m2/wheat_stand on four seeds at both levels, plus `fidelity_switch` (single_pot
   switching to simplified on day 30 and back on day 60) against detailed. Sixteen
   outcomes each (dry mass total and by organ, seed, pods, height, water use and
   efficiency, root mass above and below 20 cm, N uptake and plant N, flowering and
   maturity days, lowest leaf potential); each passes when the gap is at most the
   larger of 5% and half the detailed seed range. The same job runs
   `validate --fidelity simplified`.
7. **The bias check** reads the mean signed seed-yield gap over the scenarios, and
   fails when it is beyond ±2% **and** beyond twice its standard error from the
   seeds' paired gaps. The spec's plain ±2% can't be read on four seeds: over twelve
   seeds the pot's paired seed gap averages +0.1% with a seed-to-seed spread of 17%
   (the drought pots −1.3% and 17%), because a level switch reshuffles pod set, so a
   four-seed bias has a standard error of about 3%.
8. New goldens `single_pot_simplified` and `fidelity_switch`; plot_30_beans' golden now
   runs simplified (its `Auto` level).

### Tried and not kept
- **The spec's plain ±2% bias gate:** passes today (+1.7%) by chance; see 7.

## Consequences
- Model 0.26.0, species schema 18; every golden hash moves.
- The fidelity report: 111 of 111 outcomes pass; seed-yield bias +1.73% (single_pot
  +3.1%, drought +5.9%, plot_30_beans +1.2%, plot_low_n −1.6%, wheat stand −0.1%).
  The largest gaps are a pot's pods (+7%, allowed 23%) and, in the switch run, its
  seed (−10.5%, allowed 30%) and root mass below 20 cm (+24%, allowed 69%), all inside
  the seeds' own spread. Water use agrees within 2.2% everywhere.
- Soil conductance alone (four seeds, before the fidelity levels, every plant
  detailed): 71 of 96 checks pass, as at 0.25.0, every M0 gate passes, no verdict
  flips. Seed per plant 7.49 → 8.09 g, mean seed 0.187 → 0.201 g, harvest index 0.422
  → 0.455, pods 11.5 → 10.75; drought pods −76% → −70%, drought root:shoot +31% →
  +26%, drought shoot mass −67% → −62% (still a miss, target −26%), drought seed size
  +24% → +20%; heat seeds −31% → −34%; split_pot_foraging 1.248 → 1.254,
  nitrate_patch_proliferation +34% → +37%.
- Validation at `Auto` (four seeds): 70 of 96 checks pass, every M0 gate passes. The one
  verdict that moves from the soil-conductance run is density_leaf_area_per_plant, now
  −34.97% against a band from −35% (the plots run simplified at `Auto`).
- Validation with every plant simplified (`validate --fidelity simplified`): 69 of 96,
  every M0 gate passes; pole_height_day60 also falls to 1.495 m against 1.5 m. Both
  moves sit at a band edge and inside the seeds' spread.
- Cost on this machine (release, per pot-day): single_pot detailed 7.3 to 7.6 ms
  against 6.5 to 7.2 ms at 0.25.0 (the soil term's conductivity), simplified 5.4 to
  6.4 ms; plot_30_beans, now simplified, 15.8 to 16.1 ms against 16.4 to 16.8 ms.
  Callgrind: the simplified pot runs 13% fewer instructions, and its water step is
  about 12% of the total, almost all retention-curve powers (the hourly simplified
  step reads only each cell's potential, skipping the conductivity). The pot is still
  over the ~4 ms budget; that is the Performance thread's next target.
- Challenges, swept on world seeds 7 to 14: no medal moves. Thirsty summer's bloom
  schedules average 6.4 to 8.5 pods (5 / 7 / 9), the fixture's (every 4 days from
  day 40) 8.4 and 8 on seed 7, spread daily 2.6, none 2.1; the heat wave sown on days 28
  and 32 averages 10.8 and 11.1 (8 / 10 / 12), sown by day 16 at most 6.5; hungry soil
  split doses average 7.0 to 7.5 g (5.5 / 7 / 8.5), all on day 45 3.4, none 0.7.
- R3b's done-whens still hold: in the split pot the wet half gives 0.928 kg and the dry
  half 0.143 kg (87% wet), and 0.337 of 3.259 kg taken up goes back into dry soil at
  night (10%, was 18%: the dry soil's own conductance now limits it); predawn leaf
  potential −0.037 to −0.075 MPa against soil −0.033, daily low −0.68 to −0.82; the
  wilting pot reaches −2.50 MPa at zero turgor and recovers to full turgor.
- Open: the simplified level's summary lags a day (roots grown and cells dried since
  midnight are not seen until the next day); `K_comp` = `K_rs` is untuned (the report
  passes without tuning); the viewer's zoom and speed rule (ps-app) is the viewer
  thread's; nitrogen keeps one level (R7). Soil water still moves between cells by the
  bucket rules, not by the new conductivity.
