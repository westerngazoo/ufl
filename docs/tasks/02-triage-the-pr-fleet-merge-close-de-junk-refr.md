# T2 · Triage the PR fleet: merge, close, de-junk, refresh ROADMAP

- **Priority:** P0
- **Depends on:** none
- **Tags:** hygiene, process, rung-0

## Context
12 open PRs against origin/main @ 3d5422f (#43,#42,#40,#39,#38,#37,#36,#35,#34,#33,#32,#30), all reporting "no checks reported" (no CI). The queue hides one revert bomb and blocks the second search lane. All verdicts below were locally merge-tested onto real main.

## Work — the consolidated verdict table
**MERGE (verified green on current main):**
- **#33** (crates/ufl-evolve: fair-MLP Gate-2 anti-strawman baseline per SPEC-0011 §2.5, closes #28) — fmt/clippy/tests verified: 5 passed + 1 ignored in 1.4s. Merge FIRST; it makes #32 formally conflict. Unblocks T8.
- **#37** (GaProposer::vary allocation, hot path) — RNG-draw-order and behavior identical; full-workspace green.

**FMT-FIX then MERGE (one `cargo fmt` commit each — both currently fail `cargo fmt --check`):**
- **#39** (EngineError::Scheme propagation test) — fmt diffs at r_0008_acceptance.rs:549,556.
- **#35** (Sexpr edge-case tests: NaN/±0/unicode/1000-deep/10000-wide) — fmt diff at sexpr.rs:167. Note: its 1000-deep construction is standing evidence against #40's 128 cap.

**CLOSE with one-line supersession notes:**
- **#38** — REVERT BOMB: GitHub says MERGEABLE/CLEAN but the head tree is a stale pre-R-0014 snapshot; merging DELETES crates/ufl-discovery/src/generic.rs, tests/r_0014_generic_seam.rs, requirements/0014-discovery-framework.md, theory/two-language-substrate.md, theory/discovery-results.md, crates/ufl-tensor/tests/security.rs and reverts #45/#46/#47/#49 (verified via `gh pr diff 38`). Its own fix also has 3 bare `unwrap()`s in lib code (§6) and leaves recursive Drop unbounded. Salvage the idea under T6.
- **#40** — right instinct (typed ReadError::RecursionDepthExceeded) but hard-coded MAX_DEPTH=128 with no decision-log entry, meta-babble in the test body, and it caps only `read` while print/lower/eval/Drop stay unbounded — an asymmetric codec. Superseded by T6's one depth contract.
- **#43** — older basin-hopping R-0013 draft, conflicts with the R-0013-flipgraph branch; two drafts of one R-number violates register discipline. BEFORE closing, port its §6 decision-log table into requirements/0013 (coordinate with T1).
- **#42** — superseded by merged #51 (main already has exp_val/log_val, crates/ufl-core/src/eval.rs:72-73).
- **#36** — zero-diff (+0/−0) no-op.
- **#32** — duplicate of #33 for issue #28 (same crate, inferior layout); mutually exclusive with #33.
- **#30** — superseded by merged #31 (crates/ufl-geo/src/render.rs on main).
- **#34** — cold-path microbenchmark (the reader is not in the evolution loop), no requirement behind it: §2 "no premature optimization".

**Repo hygiene:**
- Delete the committed junk at repo root: `test_size_hint` (a checked-in 100755 binary, ~3.9 MB) and `test_size_hint.rs`; add a .gitignore entry.
- Prune merged/stale remote branches (R-0011-render-*, add-ufl-evolve-mlp-baseline-*, perf-optimize-crossover-*, optimize-random-genome-*, fix-tensor-arithmetic-overflow-*, security-fix-tensor-add-at-*, perf/render-string-alloc-*, jules-*).
- **ROADMAP.md refresh** (constitution §4 step 8, orchestrator-owed): "Current focus" (line ~133-135) still says "R-0010 … is next" and "Six crates, 168 tests green" — R-0010/R-0011 are Done/in-flight, there are eight crates, the actual focus is R-0013/R-0014; R-0013 has no row at all.

## Acceptance gate (falsifiable)
Open-PR count drops from 12 to ≤2 (only live requirement branches); every closed PR carries a supersession/reason note; `git ls-tree origin/main` shows no test_size_hint*; post-merge main passes `cargo test --workspace`, `clippy -D warnings`, `fmt --check`; ROADMAP names R-0013/R-0014 as current focus with correct crate/test counts.

## Must NOT claim
Nothing in this batch is feature work; #38's and #40's underlying concerns are NOT resolved by closing them — T6 owns the real fix.

## Files/crates
GitHub PRs #30-#43, repo root, ROADMAP.md, requirements/0013-matmul-moonshot.md (#43 port).
