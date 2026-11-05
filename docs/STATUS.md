# Status

Read this first at the start of every session (CLAUDE.md).

## Milestone
M0: headless core. Backlog: docs/design-plan.md §7; one GitHub issue per WP (#1 = WP0 ... #13 = WP12).

| WP | Issue | State |
| --- | --- | --- |
| 0 Repo scaffold | #1 | done in this PR |
| 1 Units and RNG | #2 | done in this PR |
| 2 Weather and sun | #3 | done (in PR #14) |
| 3 Organ graph | #4 | done (in PR #14) |
| 4 Species schema | #5 | next |
| 5 Soil and water | #6 | not started |
| 6 Light | #7 | not started |
| 7 Physiology | #8 | not started |
| 8 Herb architecture and roots | #9 | not started |
| 9 World | #10 | not started |
| 10 Export and CLI | #11 | not started |
| 11 Validation harness | #12 | not started |
| 12 Tuning and lock | #13 | not started |

## Last validation report
None yet (WP11).

## Open questions for the owner
- Set `main` as the repository's default branch in GitHub settings (the agent cannot change repo settings).
- WP13 (Bevy smoke test on the GTX 1070) should run around sessions 3 or 4 of M0.

## Notes for the next session
- Local checks: see CLAUDE.md "Session start".
- `cargo nextest` runs in CI; locally `cargo test --workspace` is equivalent.
