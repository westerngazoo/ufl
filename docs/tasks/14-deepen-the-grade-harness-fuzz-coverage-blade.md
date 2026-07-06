# T14 · Deepen the grade harness: fuzz coverage, blade support, contracts

- **Priority:** P2
- **Depends on:** none
- **Tags:** soundness, harness, crate:ufl-geo, crate:ufl-tensor, R-0010

## Context
The harness the evolutive loop trusts must itself be falsifiable about coverage. Today the CI soundness gate (crates/ufl-geo/tests/r_0010_soundness.rs) runs 3,200 shallow trees (8 seeds × 400, depth ≤ 4) against two singleton-grade vars — about 1% of the ~280k-tree throwaway audit that found the last two real bugs — and cannot tell whether the versor-Sandwich branch or Exp's arms were ever exercised. Separately, grade-level inference is blade-blind and passes identically-zero programs: Wedge(Basis(1),Basis(1)) infers {2} but realizes 0; Inner(Basis(8),Basis(8)) (e0 degenerate) infers {0} but realizes 0 — garust's output_grades is metric-blind by contract (SPEC-0010 §2.3), so a family of dead candidates survives the filter and bloats populations neutrally.

## Work (small spec addendum to SPEC-0010/SPEC-0011; test-first)
1. **Harden the fuzz gate** (r_0010_soundness.rs): branch-hit counters asserted nonzero per hand-rule arm (versor vs fallback Sandwich, Exp's three arms, GradeProject-empty); undeclared-⊤ and multi-grade Var bindings; document the env↔ctx precondition (realized(env[v]) ⊆ ctx[v], without which the contract is vacuous) in expr.rs/grade.rs and add `Env::consistent_with(&GradeCtx)`; an #[ignore]-tagged nightly heavy fuzz (~100k+ trees, deeper).
2. **Support refines Grade**: `support(e, ctx) -> BladeSet(u16)` in crates/ufl-geo alongside grade — exact per-blade-pair rules for GeoProduct/Wedge/Inner including the e0 degeneracy (a blade product is 0 iff the blades share e0; a wedge is 0 on overlapping blades), unions over sets; Sandwich/Exp fall back to blades-of-grade. Prune on empty support; the soundness tower extends to realized ⊆ support ⊆ blades(grade) under the same fuzz harness.
3. **Small contract fixes**: (a) `is_versor` (grade.rs:59-66) claims "never says true for a non-versor" but returns true for Basis(8)=e0 (null, non-invertible) — guard `*i != 8` or reword; soundness unaffected, the doc claim is false as written. (b) `GradeSet` has no subset method — grade.rs:77 and the soundness test both hand-roll it; add a helper (or upstream to garust). (c) `Tensor::zeros/target` panic via `expect("tensor capacity overflow")` on adversarial dims (tensor.rs:14-17,53, merged #45) — §6 demands a typed error or a spec-recorded justification.

## Acceptance gate (falsifiable)
- Branch counters fail the test if any rule arm goes unhit.
- Mutation spot-check: re-introducing the GradeLift bug (dropping the .grade(0) projection at eval.rs:42) turns the fuzz red.
- Extended fuzz green: realized ⊆ support AND support ⊆ blades(grade) over the same random-tree generator.
- A measured pruning delta reported honestly: percent of random candidates rejected by support but not by grade — a documented negative (negligible delta) is a valid outcome.

## Must NOT claim
That support analysis models the full degenerate-metric semantics — it is a tighter over-approximation, still an over-approximation; the decision-log entry of 2026-06-18 (metric absorbed by over-approximation) stays in force.

## Files/crates
crates/ufl-geo/src/{grade.rs,eval.rs,expr.rs}, crates/ufl-geo/tests/r_0010_soundness.rs, crates/ufl-tensor/src/tensor.rs, specs/0010/0011 addenda.
