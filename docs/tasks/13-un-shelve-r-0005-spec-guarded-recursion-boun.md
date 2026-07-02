# T13 · Un-shelve R-0005; spec guarded recursion; bounded in-UFL eval

- **Priority:** P2
- **Depends on:** T5, T12
- **Tags:** rung-2, reflection, R-0005, crate:ufl-predicate, crate:ufl-syntax

## Context
Rung 2 — an eval written IN UFL — is blocked by the admitted control non-goals: R-0004 §4 defers sequencing/choice/fixpoints (requirements/0004-predicate-layer.md:98-107) and the value conditional is shelved on branch `R-0005-value-conditional` (ROADMAP M4). Of these, subterm-decreasing structural recursion is the ONE that unlocks a metacircular eval — eval IS a fold over syntax — while staying total and verifier-checkable; choice is already expressible via and/or/not (R-0004 AC1) and general fixpoints are undecidable. universal-computability.md §6 is honest that without control forms, "the language evaluates itself" is unwritable except by bounded unrolling.

## Work (requirement + spec first; three-lens; builds on T5's quote and T12's probes)
1. Recover branch `R-0005-value-conditional` (`(if b a c)` in numeric position) and land it under its requirement.
2. Spec bounded structural recursion: a fold over QUOTED syntax whose recursive calls are restricted to proper subterms, checked syntactically at lowering time (decidable), with a fuel budget returning a typed `RecursionLimit` error as belt-and-braces — never a panic; mirrors C4's "typed, depth/size-capped" in theory/two-language-substrate.md. Plugs into the documented eval_form seam (crates/ufl-predicate/src/eval_pred.rs:73-74).
3. Write the in-language evaluator for the closed Eml fragment, differential-tested against `ufl_core::eval`, which remains the SOLE authority — never a second trusted evaluator. (If the requirement discussion prefers the bounded-unrolling route over recursion forms, that is a valid spec outcome — decide with Gustavo, record in the decision log.)

## Acceptance gate (falsifiable)
- Metacircular parity: the in-language evaluator's verdict matches `ufl_core::eval` on ≥10^4 ufl-prng-seeded (program, env) pairs on the reader's image, at a PRE-REGISTERED tolerance (bit-exact if achievable; else a documented ULP bound).
- A rejected-program test: a non-structural recursion fails the lowering-time check with a typed error.
- KILL: agreement unreachable without per-case special-casing — record the leak, park Rung-2 behind the control-layer requirement.

## Must NOT claim
Self-hosting, bootstrap, or control universality — a bounded/guarded eval is not a full metacircular evaluator, and the docs must state which it is. The Rust eval stays the oracle.

## Files/crates
branch R-0005-value-conditional, crates/ufl-predicate/src/eval_pred.rs (eval_form seam), crates/ufl-syntax/src/lower.rs (structural check), requirements/0005 + new recursion requirement/spec, theory/universal-computability.md (§6 update).
