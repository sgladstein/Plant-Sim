# WP22 review: pole bean, twining and lodging (PR #63, decision 0021)

Reviewed at commit 4b1aa7c on `claude/project-thread-ur2udi` against `origin/main` (model 0.5.0).
Reviewer: Fable review thread. Date: 2026-11-05.

## Verdict

**Approve.** Twining instead of tendrils is the right call and it is implemented as the research
describes: a circumnutation sweep with Darwin's thickness limit, a right-handed helix at a fixed
inclination, an indeterminate main stem that stops a set thermal time after first flower (CROPGRO
FL-VS), lodging at a fixed free length, and a wandering heap. The bush bean is untouched: I
verified `daily.csv` and `organs.csv` of both pot scenarios byte-identical against a `main`
build. All checks are green and the done-when test holds.

The two questions the implementer asked have clear answers from the run data, and both point at
mechanisms rather than parameters:

1. **The height miss is not a carbon shortage.** The staked vine's whole main stem weighs 1.6 g
   at the end, out of 21 g of plant, while `feedback_loss` sits at 0.7 from flowering on: the
   plant is discarding 70% of its potential assimilation for lack of sinks at the same time as its
   internodes stop at 70% of their potential length. Internodes are starved by local transport
   (each organ spends from its own reserve, decision 0009/0011), not by the plant's budget. Fix
   the transport or the priority for the apical zone of an indeterminate stem; do not lengthen
   the potentials or delay the flower. If the potentials were realized, 13 nodes at day 60 would
   give a 1.73 m stem and a 1.50 m height, which meets the research target exactly.
2. **The unstaked penalty is a stand effect, and the sim measures it on isolated pots.** The
   −30 to −70% band (IITA, Reckling) comes from field rows, where fallen vines pile onto their
   neighbors and the mat shades itself and stays humid. An isolated vine sprawling on bare ground
   in full sun really does intercept more light than one wound around a stake, so the 9 to 21%
   gain before rot is not a bug. Measure the response on the row (8 plants 15 cm apart, with and
   without strings) and the light-only penalty should appear. Keep pod rot, named as the
   stand-in for soil-contact disease it is.

## What I checked

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| clippy, sim and CLI crates, `-D warnings` | clean (CI covers the Bevy crates) |
| `cargo test` on the sim and CLI crates | all pass, including `staked_pole_beans_climb_and_unstaked_ones_lodge`, `bush_beans_ignore_supports` and the vine unit tests |
| `scripts/check_deps.py`, `scripts/check_determinism.py` | pass |
| golden hashes (pots 90 d, plot 30 d) | match |
| `ps-cli validate` | 76 checks: 60 pass, 16 Info misses, 0 failing, 0 known; every M0 gate passes |
| bush bean unchanged: `daily.csv` and `organs.csv` of `single_pot` and `two_pot_sun_shade` from a `main` build against this branch | byte-identical |

## Findings

### 1. Height: realized internodes against potentials

Plant 0 of `m2/pole_trellis`, main-stem internodes by birth order (cm), racemes removed:

```
7.0  6.1  7.8  14.1  11.2  16.0  20.8  12.2  12.9  14.3  16.0  15.6  16.4  15.3   sum 1.98 m
```

Potentials run 6 cm at node 2 to 19 cm from node 10. Nodes 8 to 11 (made around flowering,
days 45 to 55) sit at 12 to 14 cm against 17 to 19; the later ones recover to 16. The season
trace shows why:

| Day | nodes | height m | stem g | fruit g | total g | feedback_loss |
| --- | --- | --- | --- | --- | --- | --- |
| 45 | 9 | 0.90 | 1.85 | 0.00 | 6.7 | 0.73 |
| 50 | 11 | 1.04 | 2.10 | 0.01 | 7.7 | 0.80 |
| 60 | 13 | 1.28 | 2.43 | 1.76 | 10.5 | 0.70 |
| 75 | 14 | 1.63 | 2.78 | 6.62 | 16.3 | 0.69 |
| 89 | 14 | 1.71 | 2.86 | 10.63 | 21.0 | 0.67 |

The stem gains 0.3 g between day 50 and 60 while the plant throws away two thirds of its
potential assimilation. A 19 cm internode at 120 cm/g is 0.16 g of dry matter. Nothing about the
budget stops it; the organ cannot pull the carbon in the hours it is expanding, and the shortfall
is lost without catch-up (decision 0009). This is the WP7 open question in `docs/STATUS.md`
("priority is the share of an organ's own available reserve it may spend per hour; taken
literally §3.9 would cap every organ below its potential") showing up as a visible symptom. Two
fixes, either of which is a model change with a version bump:

- **Apical priority for an indeterminate stem.** While a vine axis is twining and below its
  support's top, its youngest two internodes (the revolving zone) draw first, as apical dominance
  does in a climber. A `climbing.apical_priority` parameter, "~".
- **Resolve the WP7 reading.** If the hourly cap is meant to be a share of the plant's reserve
  reaching the organ rather than the organ's own, the whole plant would realize more of its
  potential and the bush bean would need re-tuning (decision 0018). That is bigger and should be
  the owner's call; the apical rule is local and leaves the bush bean alone.

Also: **the helix costs 13% of height**, since 60° inclination gives `sin 60 = 0.87` of stem
length as rise. On a 1.5 cm stake 60° is a turn every 10 cm, which is right. On a 5 mm string
the same angle makes three turns per internode, which beans do not do; they keep the pitch and
steepen. Make the inclination follow the support: `η = atan(pitch / (2π R))` with a species
pitch of about 10 cm, clamped to Rutjens' 35 to 75°. The row scenario (strings) would gain about
10% height from this alone.

**Recommendation to the owner on the target:** take the research numbers (1.5 m by day 60, 2 m by
day 75 at a 20 °C mean) rather than the plan's "2 m by day 60", and expect the sim to meet the
first once realization is fixed. The 2 m by day 75 is about one more node and a steeper helix on
strings.

### 2. Unstaked penalty

Unstaked and staked vines are the same plant until day 60 (same leaf area, same mass within 2%,
same node count), which is the right null result for an isolated plant in full sun. From day 60
the only difference is pod rot: pods fall from 8.8 to 5.8, seed from 5.5 to 3.4 g.

- The literature band is for stands. Add an unstaked row scenario beside `pole_row_trellis` and
  move `pole_unstaked_seed_yield` to that pair. The fallen vines of eight plants 15 cm apart will
  lie across each other (2 m of stem each in a 15 cm heap radius), and the shared light grid will
  show the mat's self-shading. If the light-only penalty is still near zero there, the heap
  geometry is too flat: real lodged vines climb each other and make a 20 to 40 cm mound with many
  leaves face-down or vertical, and the leaf model has every leaf as a horizontal disc.
- Keep pod rot. Soil contact rot and pests are the mechanism IITA names, and 0.02 per day set to
  the band is honest as long as the card says it is the stand-in for disease. Add the same rule
  for leaves on the soil (a leaf whose blade center is below the lodge height plus its own
  radius senesces at a daily chance), which is the other soil-contact loss and gives a fallen
  vine a cost before pods exist.
- Self-twining (vines wrapping their own and neighbors' stems) is on the follow-up list and is
  the real physical mechanism for the mound. It also matters for the row with strings, where a
  vine that misses its string catches its neighbor's.

### 3. Smaller points

- `find_supports` catches a support only when the tip's height is inside the support's span, and
  a stake 5 cm from the seed is caught at emergence. A seed 7 cm or more from any support never
  catches one (sweep 6 cm) and the plant lodges. That is the model, but say in the card what
  distance a player must plant within; a trellis action (WP25) should snap to it.
- `Climb::Topped` grows on forever at 0.4 rad below level in the last azimuth, so a vine that
  tops a short stake marches off sideways. Real vines hang back down the support or over the
  far side. Let a topped axis fall onto its own support (lodged onto a cylinder) or cap it.
- The pod-rot draw uses `Purpose::PodSet` with the pod's id and the current hour. Pod set draws
  with raceme and flower ids, so no collision today; a `Purpose::Rot` would keep it that way.
- `pod_rot_rate` is clamped to 0..1 in `ClimbingParams::new` but not rejected by
  `Species::validate`; a typo of 2.0 silently becomes 1.0. Reject out-of-range in validate.
- Species schema 3 makes every species carry a `climbing` group the bush bean never reads.
  `Option` would be cleaner for the grass (WP23) and the tree.
- `heap_radius` default 0.4 m in `ClimbingParams::default()` against 0.15 in the species file,
  `lodge_height` 0.1 against 0.02, `inclination` 55° against 60. Defaults are only reached by
  tests, but make them the species values so a reader is not misled.
- The branches of a pole bean stay herb-like (3 nodes, terminal raceme). Type IV beans branch
  little and the branches do climb; fine as a known limit, listed.
- The card says supports do not shade. A 5 mm string is nothing; a 2 m, 1.5 cm stake 5 cm from
  the stem is 300 cm² of opaque area in the grid's column, about 1% of the plant's leaf area.
  Fine to ignore; the number belongs in the card.
- Southern hemisphere handedness: beans twine the same way everywhere (it is genetic, not
  Coriolis); the card's "to verify" can be closed as "same handedness, Darwin".

## For the record

- Model 0.6.0; species schema 3 (`climbing` group, `growth_form: Vine`); scenario schema 1 with
  `supports` default empty. Bush bean byte-identical (verified).
- Validation at WP22: 76 checks, 60 pass, 16 Info misses. New: `pole_height_day60` (miss,
  1.28 m), `pole_height_final` (pass, 1.71 m), `pole_unstaked_seed_yield` (pass, −53%).
- Owner choice open (decision 0021): the height target. Reviewer's recommendation above.
