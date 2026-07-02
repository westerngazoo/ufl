# T9 · Commit the C2/C6 held-out split + SE harness as real code

- **Priority:** P1
- **Depends on:** T7
- **Tags:** rung-0, spine, statistics, crate:ufl-discovery

## Context
C2 (meta-objective = held-out tasks) and C6 (improvement = measured delta on the held-out set) are two of the seven non-negotiables in theory/two-language-substrate.md — and they have ZERO committed machinery: the meta-interrogation harness that produced the hyperparameter-negative (the B0=57/200-style protocol, substrate doc lines ~118-133) was throwaway and deleted. Without it, the R-0015 pre-registered 2-SE gate cannot be run reproducibly, every future meta-claim re-improvises its statistics, and "can't-fool-itself" remains a promise instead of a mechanism. The planted-instance generator already exists on main: `RankDecomposition::for_target` (crates/ufl-discovery/src/predicate.rs:49).

## Work (small requirement/spec addendum under R-0014/R-0015 scope)
1. A module (in ufl-discovery, or the relocated ufl-evolve substrate post-T8) providing seeded, disjoint train / held-out / confirmation task splits over planted-matmul targets built with `for_target`.
2. The SE and replication arithmetic the meta-interrogation used, as tested functions: mean/SE over per-task evals-to-target (T7's ledger is the measured quantity), the ≥2-SE comparison, and the second-split replication check.
3. A committed regression that encodes the documented negative's SHAPE: apparent selection-set winners regress to ≤ baseline on a disjoint confirmation set (the expected-max-of-noisy-draws effect) — so the harness itself demonstrably catches the failure mode it exists to catch.

## Acceptance gate (falsifiable)
- The regression test reproduces the documented negative's shape on fresh seeded splits.
- The SE computation is asserted against a hand-checked fixture.
- The splits are provably disjoint (a test asserts empty intersection of target sets across train/held-out/confirmation).
- The Rung-4 probe (T11) uses this module verbatim — no fork.

## Must NOT claim
Any meta-result. This is the measuring instrument only; a harness proving the negative's shape is not evidence for or against operator-semantics evolution.

## Files/crates
crates/ufl-discovery/src/ (new module, e.g. heldout.rs) or the ufl-evolve substrate, crates/ufl-discovery/src/predicate.rs (consumer), theory/two-language-substrate.md (cross-ref).
