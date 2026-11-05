# Spec R6: branches on branches, and a branch taking over after topping

Status: spec, written 2026-11-05 (owner's order: R5 soil, then this, then R7 nitrogen).
Nothing is built. Tags as in the review ([M], [F], [K, to verify], [I]).

## 1. Why

Today (`ps-arch/src/shoot.rs::bud_fate`) [M]:
- only buds on the main stem can grow, and only up to rank `main_nodes − 2`;
- every branch is a fixed 3-node axis (`branch_nodes`) with one terminal raceme and no
  buds of its own;
- branching stops at flowering, so topping a plant in flower releases nothing (WP25
  review);
- after a cut, buds above `main_nodes − 2` stay ineligible, and no branch ever takes over
  as the leader.

The result, logged at WP25: a topped plant ends with fewer branches than an untopped one,
which is the opposite of the garden experience. A bean branch carries axillary buds and
racemes at its nodes like the main stem does, and type I bush beans have secondary
branches [K, to verify for typical counts].

Game value: pruning and topping become a real strategy (bushier plants, more racemes),
pole beans topped at the top of the trellis bush out, and branch structure becomes a
visible, breedable trait.

## 2. Model

### 2.1 Buds on branches
- Every node of every axis gets an axillary bud, as main-stem nodes do now, up to axis
  order `max_branch_order` (2 for bean: branches and their branches, ~) [K, to verify].
- The fate rule is today's, read per axis: light index, the node's carbon factor, and
  apical dominance from **that axis's own apex** (the same `hold_nodes` and
  `dominance_nodes`), times an order factor `order_activation[k]` (1 for the main
  stem's buds, 0.5 for a branch's buds, ~) so second-order branches stay fewer.
- **Racemes on branches:** the upper buds of a branch become racemes at flowering, as
  on the main stem (today a branch has only its terminal raceme).
- **Branch length:** `branch_nodes` becomes the potential node count of a first-order
  branch (3 today), with second-order branches `branch_nodes_order2` (2, ~). Shade and
  carbon still shrink organs as today.

### 2.2 After a cut (topping and pruning)
- **All buds below a cut become eligible**, whatever their rank (WP25 review).
- **Leader takeover:** when the main apex is cut or dies before the main stem's node
  budget is used, the most vigorous active branch within `takeover_nodes` (2, ~) of the
  cut becomes the leader. Vigour is its apex's reserve concentration times its light
  index; ties go to the lower organ id, so the rule is deterministic. The new leader
  gets the remaining main-stem node budget and the main stem's internode and leaf
  profiles, and its new internodes bend upward under the main stem's gravitropism
  weight. Apical dominance and hold then act from the new leader's apex.
- **Flowering cut-off:** branching still slows at flowering, but a cut lifts it for
  `post_cut_window` days (10, ~), so a plant topped in flower can still break buds
  (WP25 review: "a pole bean topped at the trellis top should branch") [K, to verify].

### 2.3 Pole bean
Branches stay herb-like (no climbing) in R6; climbing branches remain a climbing-card
follow-up. A topped pole bean's takeover leader is a herb-like branch unless it is
within the capture sweep of the support.

### 2.4 Parameters

| Name | Value | Unit | Source |
| --- | --- | --- | --- |
| `max_branch_order` | 2 | - | ~ [K, to verify] |
| `order_activation` | 1, 0.5 | - | ~ |
| `branch_nodes`, `branch_nodes_order2` | 3, 2 | - | ~ (3 today) |
| `takeover_nodes` | 2 | nodes | ~ |
| `post_cut_window` | 10 | day | ~ |

All bud draws keep the per-bud `Purpose::BudFate` stream, so adding buds on branches
does not change the draws of main-stem buds.

## 3. Done when

- An untopped bush bean still has 3 to 6 branches (§3.16) on four-seed means, now with
  some second-order branches and racemes on branches.
- **Topping response:** a bush bean topped before flowering ends with at least as many
  branches and racemes as an untopped one, and one branch takes over as leader. Cline
  1991 is the source the herb card already cites for the bean decapitation response [F].
- A pole bean topped at the trellis top in flower grows branches within 10 days.
- Pod count and yield of untopped plants stay inside their bands after re-tuning.
- Determinism: the same plant twice is bit-identical; a cut replays bit for bit.

## 4. Cost and risks

- More leaves and racemes per plant: perhaps +10 to 20% shoot organs, and light scales
  with leaves (it is about two thirds of the daily cost), so +5 to 15% per plant-day [I].
  Carbon limits how many branches actually grow, so the rise is bounded. Performance
  should measure it.
- Branch count and LAI rise; the density follow-up (WP21 review: branches don't respond
  to density) may get easier, because buds on branches also read light.
- Validation checks on branches, nodes and pods move; re-tune on seed means.
- Viewer: nothing new (branches are axes the viewer already draws); the inspector shows
  axis order.
- Model cards: herb-architecture (bud fate per axis, takeover), climbing (topped vines).
- M3: `order_activation` and `branch_nodes` become breedable traits (bushier or sparser
  plants).

## 5. Tests

- A bud on a branch reads its own axis's dominance; a branch's top buds are held under
  its growing apex.
- After a cut, the takeover picks the documented branch, and its new internodes bend up.
- A cut in flower opens the window; without a cut nothing branches after flowering.
- Ninety-day: the §3 targets on four-seed means.
