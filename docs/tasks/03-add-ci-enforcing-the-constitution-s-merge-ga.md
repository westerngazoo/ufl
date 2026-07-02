# T3 · Add CI enforcing the constitution's merge gate

- **Priority:** P0
- **Depends on:** none
- **Tags:** infra, ci, process

## Context
CLAUDE.md §5 makes `cargo test` + `cargo clippy` + `cargo fmt --check` a hard merge gate, but NO CI exists: `gh pr checks` reports "no checks reported" on every one of the 12 open branches. Consequences already observed: two otherwise-good PRs (#39, #35) sit failing `cargo fmt --check` undetected, and #38 wears a MERGEABLE/CLEAN badge while being a revert bomb that deletes five merged R-0014 artifacts. The fleet generates PRs faster than un-gated review absorbs them — the constitution's gate must be machine-held, exactly like the verifier discipline the language itself preaches.

## Work
Add `.github/workflows/ci.yml` running on every PR and on pushes to main:
1. `cargo test --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo fmt --check`

Make all three required checks on the protected `main` branch. Cache the cargo registry/target dir for speed. No product code changes.

## Acceptance gate (falsifiable)
- Re-pushing #39's branch as-is turns the PR red on the fmt job.
- A test PR deleting a tracked test file turns red on the test job.
- Branch protection on main lists the three jobs as required.

## Must NOT claim
CI replaces neither the architect review nor Gustavo's sign-off (constitution §3) — it mechanizes only the §5 gate.

## Files/crates
.github/workflows/ci.yml (new), GitHub branch-protection settings.
