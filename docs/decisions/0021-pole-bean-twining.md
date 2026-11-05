# 0021: Pole bean: indeterminate stem, twining on supports, lodging (WP22)

- **Date:** 2026-11-05
- **Status:** accepted (the height target is the owner's open choice)

## Context
WP22 (design plan §7, M2) adds the vine module and the pole bean: the habit locus,
indeterminate growth, a support search with attachment, and trellis objects. It is done
when "a pole bean fills a 2 m trellis by day 60; without a trellis it flops and yields
less". The plan (§3.4) describes a tendril that searches a cone around the tip.

The M2 research report (research/m2-m3-bean-science.md, decisions 1 and 2) found that
common bean has no tendrils. It is a stem twiner: the shoot tip circumnutates every 80
to 130 minutes and wraps any support it touches, anticlockwise seen from above (Darwin
1875; Millet 1988; Hou 1997). It also showed that 2 m by day 60 needs 15 cm internodes
at a 20 °C mean, the top of the published range, and recommended at least 1.5 m by day
60 and 2 m by day 75. The owner has not chosen yet; a decision card is open.

## Decision
1. **Twining, not tendrils.** `OrganKind::Tendril` stays in the enum, unused, until a
   schema break removes it (ps-view still names it). An internode gains an optional
   coil (`Meta::twine`: support axis, radius, inclination).
2. **Habit.** `habit ≥ 0.5` makes the plant a vine (`HerbParams::indeterminate`). The
   main stem has a vegetative apex: no terminal raceme. It makes nodes until
   `vegetative_after_flowering` thermal time after first flower (CROPGRO FL-VS, 13
   photothermal days × 20 °C·d) or `max_main_nodes` (30). At flowering, buds above
   rank `nodes_before_terminal − 2` become racemes as for the bush bean, and every node
   made after flowering gets a raceme in its axil. Main-stem internodes rise from the
   first to the last fraction over `internode_ramp_nodes` and stay level after it.
   The bush bean's behavior is unchanged: its daily rows and organ tables are identical
   byte for byte; only the state hash moves, because the plant serializes new fields.
3. **Supports.** A scenario lists vertical supports per soil (offset, height, diameter;
   `Scenario::supports`). Each plant keeps those of its soil in its own frame. They do
   not shade (a 1.5 cm stake is a small fraction of a light cell).
4. **Capture.** Daily, before new nodes, a searching vine axis looks for the nearest
   support whose surface is within the sweep radius (6 cm) of its tip, at a height the
   support spans. It catches it with probability 1 up to 5 cm across, falling linearly
   to 0 at 20 cm (Darwin: fails on 23 cm). A day holds a dozen or more sweeps, so the
   catch is immediate.
5. **Coil.** Internodes made while twining lie on a helix around the support axis at
   radius `R = support radius + stem radius`, turning anticlockwise seen from above. Its
   angle above horizontal follows the support: `atan(pitch / 2πR)` with a 10 cm pitch,
   kept within 35 to 75° (Rutjens 2025), so a 5 mm string gives 75° and a 1.5 cm stake
   about 61°. Layout places the tip of
   a coiled internode on the helix at its current length; the coil is fixed once made.
   When the tip passes the support's top, the axis lets go and new internodes hang
   down beside it (1.2 rad below level).
6. **Lodging.** An unsupported vine axis holds up `self_support_length` (40 cm) of new
   stem. Past that it falls: every internode of the axis turns toward one random
   direction and dips to the lodge height (2 cm), and for the main stem every branch
   falls with it. This is the one place the sim moves grown organs, since lodging is a
   mechanical failure. New internodes then wander along the ground, turning back when
   they leave a 15 cm heap around the collar, and new branches start fallen.
7. **Pod rot.** A vine's pod that hangs onto the soil (its foot lower than its length)
   rots with a daily chance of 0.02 (its own stream, `Rot`) and is shed with its seeds.
   It stands in for the disease of pods and leaves on damp soil. Staking keeps pods off
   wet soil (IITA PROPAS; Reckling 2011). Only vines run this rule: a bush bean's pods
   hang clear, and the M0 lock stays untouched.
8. **Pole bean file.** data/species/pole_bean.ron is the bush bean file with `habit` 1,
   CROPGRO's later pole-type flowering (EM-FL 34 against 26: 575 °C·d) and maturity
   (1150 °C·d), and a thinner, longer climbing stem: 120 cm/g specific stem length (a
   2.6 mm stem), internodes 6 cm at node 2 rising to 19 cm from node 10, and three times
   the bush bean's phloem conductance (1500 g/m/h, see 10).
10. **Transport in a vine** (review of WP22). The edge conductance `k π r² / ℓ` of a
   2.6 mm, 14 cm twining internode is about 8 times smaller than that of a bush bean
   internode, so at the bush bean's 500 g/m/h the leaves filled up (70% of potential
   assimilation lost to the sink feedback) while the upper internodes reached 70% of
   their potential length. Climbing stems conduct more per cross-section than
   self-supporting ones (Ewers 1985; Ewers and Fisher 1991), so the pole bean's
   conductance is 1500 (~). The review's other suggestion, a higher spending priority
   for the two youngest internodes, was tried and dropped: their own reserves were
   empty, so a larger share of them changed nothing (1.26 to 1.29 m at day 60).
9. **Schemas and version.** Species schema 3 adds the `climbing` group (every species
   carries it; the bush bean's values are unused). Model 0.5.0 → 0.6.0, golden hashes
   regenerated.

## Consequences
- In m2/pole_trellis (4 pots, a 2 m stake 1.5 cm across 5 cm from each seed, sown 10
  June at 45 N) the vines catch their stakes at emergence and climb to 1.55 m at day 60,
  1.99 m at day 75 and 2.08 m at the end (Kentucky Wonder 1.5 to 2.1 m), which meets
  the research recommendation of 1.5 m by day 60 and 2 m by day 75. The main stem makes
  13 nodes by day 60 and 14 in all. Before the review's transport fix (point 10) they
  reached 1.28, 1.63 and 1.71 m. The row on 5 mm strings (m2/pole_row_trellis) reaches
  1.68 m at day 60 and 2.24 m at the end: its steeper coil climbs faster, and the
  internode that passes the string's top overtops it. Both height targets are Info and
  pass; the owner's choice of target is still open.
- Without strings, the same row (m2/pole_row_no_trellis) falls at about node 5, stays
  under 25 cm, and yields 34% less seed (band −30 to −70%; the isolated pots lose 44%).
  The response is measured on the row because the field trials are rows. With
  `pod_rot_rate` set to 0 the fallen row yields 1% more than the staked one (7.08
  against 7.00 g a plant), so on the row the whole −34% is the encoded pod rot and the
  fallen canopy costs nothing. Before the transport fix, too, fallen vines yielded 9 to 21% more than staked ones,
  because their leaves spread over more ground (area-weighted light index 0.68 against
  0.54 in the row, 0.79 against 0.70 in the pots, measured at day 75). A leaf-on-soil
  rot rule was considered and left out: a fallen vine's leaves sit 4 to 6 cm up on their
  petioles, so it would rarely fire.
- The plant stays sink-limited after flowering (about 75% of potential assimilation
  lost to the sink feedback), as the bush bean does.
- The viewer draws a coiled internode as a straight segment from its base to its tip
  on the helix. Drawing the helix needs `Meta::twine` and `Twine::point`.
- Follow-ups: climbing branches (branches stay herb-like with 3 nodes and a terminal
  raceme); supports as obstacles in the light grid; a build-a-trellis action (WP25);
  a heritable climbing-ability trait next to *fin* (research decision on habit);
  self-twining of lodged vines around each other.
