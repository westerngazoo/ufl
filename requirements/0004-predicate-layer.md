# R-0004 — Predicate Layer (`⟦P⟧`) — the Checker

- **Status:** Draft
- **Milestone:** M3
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-02
- **Pillar / atom:** Predicative logic — atom `⟦P⟧` (proposal Pillar 3). Resolves
  proposal open question Q2 (predicate expressiveness) — first slice.
- **Depends on:** R-0003 (the s-expression core — predicates are forms) and
  R-0001 (numeric operands are eml-`Value`s).
- **Realized by:** SPEC-0004 (pending)
- **QA:** `qa` agent run scoped to R-0004

## 1. Statement

UFL gains the **predicate atom `⟦P⟧`** as an s-expression form. A predicate is a
**boolean-valued expression over pre-state and post-state**, following Hehner's
predicative programming. UFL can **check** whether a given (pre-state,
post-state) pair satisfies a predicate.

This first slice is a **checker, not a solver**: evaluating a predicate against
a *concrete* pre/post pair is decidable; *finding* a satisfying post-state (the
substrate orchestrator's job) is undecidable in general and is a later
requirement. It is the first piece of UFL's **control-universality** layer —
see [`theory/universal-computability.md`](../theory/universal-computability.md),
Route B.

New forms (all s-expressions, composing with the existing `eml` form):

```
true   false                       -- boolean literals
(= a b)                            -- exact equality of two numeric (eml) values → bool
(and p q ...)  (or p q ...)  (not p)  -- logical connectives over bools
(pred E)                           -- wrap a boolean expression E as a predicate ⟦E⟧
```

A predicate mentions **pre-state** variables (`x`) and **post-state** variables
(written primed, `x'`). Checking binds the pre-state and post-state and
evaluates the predicate to `true` or `false`.

```
⟦ x' = (eml x 1) ⟧                  -- "the post-state x' is e^x"
```

## 2. Rationale

The proposal's Pillar 3 makes a program a *predicate* over pre/post state, so a
gate, a function, and a neural layer become the same kind of object — a
constraint. R-0001 gave UFL values; R-0003 gave it a homoiconic surface; R-0004
gives it the first **logic** — the ability to *state and check* constraints.

This is where the "all computable" claim begins to be discharged. `eml` is
value-universal but has no control; predicates supply sequencing, branching, and
recursion (later). The checker is the irreducible foundation: before you can
*solve* a constraint or *sequence* computations, you must be able to *evaluate
whether a constraint holds*. R-0004 delivers exactly that.

## 3. Acceptance criteria

- **AC1 — Booleans enter the language.** UFL has boolean values, distinct from
  numeric (eml) values. `true` and `false` are expressible and evaluate to the
  two boolean values. Applying a numeric operation to a boolean, or a logical
  operation to a number, is a **typed error**, never a panic or a silent
  coercion.
- **AC2 — Exact equality.** `(= a b)` evaluates its two operands as numeric
  (eml) values and returns a boolean: `true` iff they are **exactly equal** (no
  tolerance). Equality is decidable and deterministic.
- **AC3 — Logical connectives.** `(and p q ...)`, `(or p q ...)`, `(not p)`
  evaluate boolean operands with the standard truth tables. (Short-circuit
  behaviour, arity, and the empty cases are a spec detail.)
- **AC4 — Pre/post-state and checking.** A predicate may mention pre-state
  variables and post-state variables (the latter primed, e.g. `x'`). Given a
  pre-state binding and a post-state binding, the predicate **checks** to
  `true` or `false`. A predicate variable with no binding is a typed error.
- **AC5 — Predicates express eml semantics (worked example).** The predicate
  `⟦ x' = (eml x 1) ⟧` checks to `true` for any pre/post pair where `x'` equals
  the eml-evaluated `e^x`, and `false` otherwise. Demonstrated over several
  `x`, including a deliberately-wrong post-state that must check `false`.
- **AC6 — Layered typed errors.** Every failure is a typed error surfaced at
  the earliest layer that can detect it — read, lower, or the new
  predicate-evaluation layer (type mismatch, unbound state variable) — never a
  panic. Composes with R-0003's `ReadError` / `LowerError` / `EvalError`.

## 4. Constraints & non-goals

**Constraints**

- A **checker only**: `check(predicate, pre_state, post_state) -> bool`.
- **Exact equality**, no tolerance (AC2). The floating/complex `==` semantics
  (IEEE: `NaN ≠ NaN`, `±0` equal — vs literal bit equality) is fixed in
  SPEC-0004 and reviewed.
- Numeric operands are R-0001 eml-`Value`s (`Complex<f64>`), evaluated by the
  reused `ufl_core::eval`.

**Non-goals** (later requirements)

- **Sequential composition** (`S ; T` = conjunction over an existential
  intermediate state) and **parallel** (∃).
- **Recursion / fixpoints** — the rest of control universality.
- The **substrate orchestrator** (`⊗`) using predicates to *select* a substrate
  (solving, not checking).
- **Approximate / tolerance equality** — derivable later; exact only here.
- **Quantifiers** (`∀`, `∃`) over state.
- Ordering comparisons (`<`, `≤`) — `=` only in this slice unless trivial.

## 5. Open questions

- **The value model.** Does R-0004 introduce a heterogeneous runtime value enum
  (`Num(Complex) | Bool`), or keep two typed evaluation modes (numeric →
  `Complex` via the reused `eval`; boolean → Rust `bool`) bridged by `=`?
  SPEC-0004 decides; the synthesis favours staying typed where possible (a
  bool-returning predicate evaluator that calls the numeric evaluator for
  comparison operands, no god-enum). A three-lens topic.
- **Primed post-state syntax.** Is `x'` a distinct symbol (the reader already
  admits `'` in symbols), with the checker resolving unprimed → pre-env and
  primed → post-env? Confirm in SPEC-0004.
- **Exact `==` semantics.** IEEE `PartialEq` on `Complex<f64>` vs literal
  bit-equality (they differ on `±0` and `NaN`). SPEC-0004 picks one with
  rationale.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-02 | R-0004 is a **checker only** — pred + booleans + `=` + and/or/not. Sequencing, recursion, and the orchestrator are deferred. | Checking a concrete pre/post pair is decidable and is the irreducible foundation; solving/sequencing build on it. Keeps the first slice testable. (Owner decision.) |
| 2026-06-02 | Predicate equality is **exact**, not tolerance-based. | For a checker, both sides of a useful equality are computed the same way and match exactly; tolerance only matters when asserting symbolic identities against literals, which is not what a checker does. Keeps `=` decidable and the language free of a tolerance parameter; approximate equality is a derivable later form. (Owner decision; the precise IEEE-vs-bit `==` semantics is a SPEC-0004 open question.) |
| 2026-06-02 | Booleans are introduced as a value kind distinct from numeric `Value`; cross-type operations are typed errors (no coercion). | Keeps the type discipline of the synthesis — illegal states (number where bool expected) are caught, not silently coerced. |

## Changelog

- 2026-06-02 — created (Draft).
