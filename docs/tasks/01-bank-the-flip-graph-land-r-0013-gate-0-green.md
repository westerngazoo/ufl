# T1 · Bank the flip-graph: land R-0013 Gate-0 green

- **Priority:** P0
- **Depends on:** none
- **Tags:** rung-0, spine, crate:ufl-discovery, crate:ufl-tensor, R-0013

## Context
The flip-graph is the existence proof that move-structure decides everything (theory/discovery-results.md: blind GA dead at 25e9 evals, flip-graph certified rank-7 in 0.16s) — and it is currently "claims pending banked code." Branch `R-0013-flipgraph` (commit 0a10743) holds requirements/0013, specs/0013-matmul-moonshot.md, and the red Gate-0 test `crates/ufl-discovery/tests/r_0013_flipgraph.rs`. The test today fails with 7 COMPILE errors, not assertions: E0432 (missing `ufl_discovery::reduce_matmul`) plus six E0624 because `Triple::{u,v,w}` are `pub(crate)` (crates/ufl-tensor/src/scheme.rs:74-82). The branch base (2d58ab4) predates the R-0014 merge and the GradeLift/tensor soundness fixes.

## Work
1. **Rebase** `R-0013-flipgraph` onto current origin/main (3d5422f).
2. **Amend SPEC-0013 §2.1**: replace "keeps ufl-tensor unchanged" with "read-only accessor promotion, no invariant change"; widen `Triple::{u,v,w}` to `pub` (read-only slices, invariants stay constructor-enforced). A Scheme the language can construct but never read is not first-class data — this same accessor is what the reflection loop needs on discovered artifacts.
3. **Verifier integrity precondition**: certification discharges through `reconstruct()` → `add_at` (crates/ufl-tensor/src/reconstruct.rs:26), and merged #46 made `add_at` silently no-op on out-of-bounds (crates/ufl-tensor/src/tensor.rs:40-45) while its doc (line 38-39) still claims in-range-by-construction. Restore loud failure (debug_assert + documented unreachable justification, or a typed error path) and fix the doc — silent-wrong inside the exact verifier is the one failure mode the can't-fool-itself discipline forbids.
4. **Implement** `crates/ufl-discovery/src/flipgraph.rs` per SPEC-0013 §2.2-2.5: IntTriple/IntScheme i64 workspace, the three sum-preserving flip variants with doc-comment proofs, reduce-to-fixpoint, `reduce_matmul` returning `Scheme`, typed `FlipError`, no panics (CLAUDE.md §6).
5. **Pin the plateau policy** as a named, testable object: `FlipConfig { stall_window, perturb_flips, checkpoint: BestSoFar }` with a `pinned()` constructor (mirroring `GaConfig::pinned()`, proposer.rs:24). SPEC-0013 §2.4's "restart on stall of k steps" is underspecified and the killed pilot showed plateau-escape tuning is load-bearing; naive restart-from-naive is weak.
6. **Factor moves as data-ready primitives**: public, pure `flip_at(&IntScheme, pair, Variant) -> Option<IntScheme>`, `reduce(&IntScheme) -> IntScheme`, `perturb(&IntScheme, k, rng)`, plus a shared-factor pair enumerator; `reduce_matmul` is a thin driver over them. The Rung-4 MoveForm interpreter (T11) must compose these without rewriting the module.
7. Port PR #43's §6 decision-log table (moonshot-domain choice, "first probe inconclusive", honest-odds) into requirements/0013 before #43 closes (coordinate with T2). After merge, delete the matmul "claims pending banked code" rows in theory/discovery-results.md.

## Acceptance gate (falsifiable)
- `cargo test -p ufl-discovery --test r_0013_flipgraph` goes red→green: certified rank-7 at the pinned (SEED, BUDGET), deterministic, under a laptop-minute; re-certified through a fresh `RankDecomposition`; 20,000-pair bilinear check passes; the corrupted scheme never certifies (AC2).
- SPEC-0013 §2.6 unit tests: `flip_preserves_the_tensor`, `reduce_only_drops_rank`.
- A test reconstructs `reduce_matmul`'s exact pinned-seed trajectory by driving the public primitives directly (same SplitMix64 draws ⇒ identical certified scheme).
- Mutation test: deliberately corrupting one index computation in `reconstruct` makes the suite fail loudly (not silently-wrong).
- Suite green on 3 consecutive runs; fmt/clippy clean.
- KILL: if no (seed, budget) within a laptop-minute reaches rank 7, the pilot's claim was wrong — document and strike the discovery-results entry.

## Must NOT claim
Any new mathematics: this is a certified re-derivation of the known rank-7 (de Groote-unique) scheme, not a discovery. The requirement already says so; keep it.

## Files/crates
crates/ufl-discovery/src/flipgraph.rs (new), crates/ufl-discovery/tests/r_0013_flipgraph.rs, crates/ufl-tensor/src/{scheme.rs,tensor.rs,reconstruct.rs}, specs/0013-matmul-moonshot.md, requirements/0013-matmul-moonshot.md, theory/discovery-results.md.
