# R-0016 — Reflection rung 1: quote, eval, raise

- **Status:** **Accepted** (2026-07-03) — three-lens complete (SPEC-0016 carries
  the coupled review); AC3 amended to `eq?` (numeric `=` untouched),
  Gustavo-confirmed. SPEC-0016 is Accepted; ready for T5.
- **Milestone:** the self-eval staircase, **Rung 1** (see the canonical ladder in
  `theory/two-language-substrate.md`)
- **Depends on:** R-0003 (Sexpr core), R-0004 (predicate layer), the unified
  depth contract (task 06 — quote must not land on an asymmetric read/print codec)

## Context

Every UFL evaluator today is **one-way and terminal**: `text → Sexpr → Eml →
Value` with `Value = Complex<f64>` and `eval_pred → bool`. There is no value that
can *hold syntax*, no `quote`, and no inverse of `lower` — so UFL programs cannot
be UFL data, and nothing above Rung 1 of the staircase can exist. The carrier is
already in place: `Sexpr` is comparable, clonable data with a reader **and** a
printer whose round-trip invariant is documented and tested. What is missing is
the *landing type* and the two named forms.

## Requirement

UFL code becomes UFL **data**: a third typed evaluation mode ("syntax") beside
numeric and boolean, carrying an unevaluated `Sexpr`; a `(quote e)` form that
produces it; an `(eval q)` form that discharges it **through the existing
pipeline** (`lower` + `ufl_core::eval`) — never a second evaluator; structural
`=` on two syntax operands (the existing `Sexpr::PartialEq`); and the missing
Rust-side inverse `raise: &Eml → Sexpr` closing the code↔data square
(`raise ∘ lower = id` on the reader's image).

## Load-bearing scoping decisions (stated here so the spec cannot drift)

1. **Quote is a NAMED form `(quote …)`, never the apostrophe.** `'` is a legal
   symbol character the tokenizer passes through, and it is load-bearing for
   Hehner priming — `x'` is the post-state variable (SPEC-0004 §2.5). The
   classic reader-macro is foreclosed by design, not omitted.
2. **Scope is code-as-value ONLY. `Value → Sexpr` reification is explicitly
   out**, because it can never be total: only `1` is a literal; `inf`/`nan` are
   legitimate eval *results* outside the reader's image (R-0001 AC3). The rung
   is `(quote e)` (code into value-position), not values into code.
3. **No second evaluator.** `(eval q)` reuses `lower` + the verified `ufl_core`
   eval — the same reuse rule that governs the predicate layer (R-0003 AC4).
4. **No new control forms.** Quote/eval need zero recursion, branching, or
   state; those remain deferred (R-0004 non-goals, R-0005 shelved). An eval
   *written in UFL* is Rung 2 — a separate, future requirement.

## Acceptance criteria

- **AC1 (eval∘quote = id):** property test over generated `Sexpr`s in the
  reader's image: `⟦(= (eval (quote E)) E)⟧` holds for every sampled bound `E`.
- **AC2 (quote does not evaluate):** `(quote (eml y 1))` with `y` unbound
  discharges **without** `UnboundVariable`; and `(quote e)` reached in numeric
  position still fails typed (`UnknownForm` today → the new syntax mode, never a
  silent coercion).
- **AC3 (structural equality):** a **distinct form `(eq? a b)`** compares two
  quoted forms by `Sexpr::PartialEq` — exact and decidable. Numeric `=` is left
  **exactly** as SPEC-0004 defines it (numeric-only, untouched). *(Amended
  2026-07-03 after the three-lens review: overloading `=` silently changed numeric
  `=` and shipped a classifier that becomes unsound once Rung 2 adds syntax-typed
  bindings; a separate `eq?` closes both. Gustavo-confirmed.)*
- **AC4 (the square closes):** `raise ∘ lower = id` on the reader's image,
  property-tested; `raise` emits only reader-image `Sexpr`s for reader-image
  inputs.
- **AC5 (the gates):** `cargo test` / `clippy -D warnings` / `fmt --check` green
  (now machine-enforced by CI).

## Non-goals

- No `Value → Sexpr` reification (see scoping decision 2).
- No in-language eval (Rung 2), no pattern-matching forms on syntax beyond `=`
  (head/arg accessors are a spec decision *if* AC1–AC4 need them, else deferred).
- No apostrophe reader-macro, ever, under this design.
