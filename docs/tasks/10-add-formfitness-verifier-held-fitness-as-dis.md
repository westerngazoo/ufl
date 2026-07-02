# T10 · Add FormFitness: verifier-held fitness as discharged UFL forms

- **Priority:** P1
- **Depends on:** T7
- **Tags:** rung-3, spine, reflection, crate:ufl-discovery, crate:ufl-predicate

## Context
This is the rung the repo is secretly closest to: the s-expression already IS a dischargeable Hehner predicate — `impl Predicate for Sexpr` (crates/ufl-predicate/src/predicate.rs:54-61, "the s-expression is the predicate") over the form language {pred, =, and, or, not, true, false}. Converting the C3 constraint (reward = exact verifier verdict, never a proxy — theory/two-language-substrate.md) from prose into a typed seam means the Rung-4 meta-loop inherits it for free, and it is the first place a UFL form participates in the search loop at all.

## Work (small spec section under SPEC-0014/SPEC-0015; test-first)
1. A `FormFitness` implementing `Fitness<G,S>` (crates/ufl-discovery/src/generic.rs:32, post-T7 with the generic error channel) whose verdict is produced by discharging a `Sexpr` predicate through the existing `impl Predicate for Sexpr`, with verifier-computed quantities (e.g. the RankDecomposition residual) bound into `State` as variables — e.g. the form `(= residual 0)`.
2. Pin C1/C3 in the spec: the fitness FORM is held on the verifier side; the proposer can never supply or rewrite it. UFL-expressed fitness is a transparency window, not an authority.

## Acceptance gate (falsifiable)
- Byte-identical verdicts: FormFitness reproduces the hand-coded MatmulFitness outcome on the full tests/r_0014_generic_seam.rs sweep (same seeds, same trajectories) — the AC2 byte-identity discipline reused.
- KILL: if the {pred,=,and,or,not} form language cannot express the acceptance property without ad-hoc new forms, record exactly which forms are missing as the next predicate-layer requirement rather than widening silently.

## Must NOT claim
"The language scores itself" in any autonomous sense — the verdict is still the Rust discharge (C3), and the form is verifier-held, never evolved.

## Files/crates
crates/ufl-discovery/src/ (FormFitness), crates/ufl-predicate/src/predicate.rs (consumer), crates/ufl-discovery/tests/ (parity test), specs/0014-*.md or specs/0015-*.md section.
