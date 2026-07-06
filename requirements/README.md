# Requirements

A **requirement** states *what* UFL must do — a capability or property, from the
problem perspective, independent of implementation. Requirements are the WHAT;
[`specs/`](../specs/) are the HOW.

Every requirement is decided **together** (Gustavo + Claude) before a spec is
written, and every requirement is owned by a `qa` agent run that verifies it.

## Process

1. **Discuss.** Gustavo and Claude agree the capability and its acceptance
   criteria. See [`CLAUDE.md`](../CLAUDE.md) §4.
2. **Record.** Create a file from [`TEMPLATE.md`](TEMPLATE.md), numbered
   `R-NNNN` (next free 4-digit id): `NNNN-short-name.md`.
3. **Accept.** When acceptance criteria are unambiguous, status → `Accepted`.
   Only then may a spec realize it.
4. **Realize.** One or more `SPEC-NNNN` in `specs/` implement the requirement.
5. **Verify.** The `qa` agent, scoped to this `R-NNNN`, confirms every
   acceptance criterion. Status → `Met`.

## Status values

`Draft` → `Accepted` → `Met` · (or `Superseded`)

## Relationship to specs

A requirement links forward to the spec(s) that realize it; a spec links back to
the requirement(s) it satisfies. The mapping is maintained in
[`ROADMAP.md`](../ROADMAP.md).

## Index

| Req | Title | Milestone | Status |
|-----|-------|-----------|--------|
| [R-0001](0001-eml-operator-core.md) | EML Operator Core | M1 | Met (the typed core R-0003 lowers into) |
| [R-0002](0002-geometric-algebra-core.md) | Geometric Algebra Core over G(3,0,0) — *via garust* | M1 | **Superseded by [R-0009](0009-pga-kernel-binding.md)** (wrong signature — no ideal generator) |
| [R-0003](0003-sexpr-core.md) | Homoiconic S-Expression Core | M2 | Met (merged, PR #11) |
| [R-0004](0004-predicate-layer.md) | Predicate Layer (`⟦P⟧`) — the Checker | M3 | Met (merged, PR #14) |
| R-0005 | Value Conditional (`if`) | M3 | **Shelved** (language-build thread paused for the discovery pivot; on branch `R-0005-value-conditional`) |
| [R-0006](0006-integer-tensor-core.md) | Exact Integer-Tensor Core (`ufl-tensor`) | M5 — Discovery | Met (merged, PR #16) |
| [R-0007](0007-tensor-predicate.md) | Tensor-equality predicate (the Hehner-discharge bridge) | M5 — Discovery | Met (merged, PR #18) |
| [R-0008](0008-discovery-engine.md) | Discovery engine — loop validation + blind-proposer falsification | M5 — Discovery | Met (merged, PR #19) |
| [R-0009](0009-pga-kernel-binding.md) | `Cl(3,0,1)` PGA kernel binding (`ufl-ga` → garust v0.1.0) | M5 — Discovery | Met (merged, PR #21; 15 tests) |
| [R-0010](0010-geometric-forms-grade-types.md) | Geometric forms + the grade-type system (`ufl-geo`) | M5 — Discovery | Met (merged, PR #23; 18 tests — 14 AC + 4 soundness) |
| [R-0011](0011-geometric-neuroevolution.md) | Geometric neuroevolution — evolve the `GeoExpr` genotype (sandwich-rediscovery gate + the equivariant-OOD-generalization headline) | M5 — Discovery (**headline**) | **SPEC-0011 Accepted** (three-lens closed; §2.6 + §2.8 de-risks positive — the morph→discover→translate loop closed); impl in #26/#27/#28 + the `ufl-evolve` engine |
| [R-0012](0012-f2-boolean-deduction.md) | Boolean deduction via equality saturation over 𝔽₂ (the egg↔Gröbner bridge; discrete-logic lane) | M5 — Discovery (reasoning) | Draft (falsifiable spike; new `egg` + SAT engine, **not** a reuse) |
| [R-0014](0014-discovery-framework.md) | The shared discovery framework — one search/rewrite substrate, three verifier instances (the lanes keep their atoms) | M5 — Discovery (**unifying**) | Draft (design-panel scoped: unify the AST + search harness, **not** the atoms; eml-substrate over-claim refuted) |
