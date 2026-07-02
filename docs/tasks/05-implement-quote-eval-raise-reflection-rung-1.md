# T5 · Implement quote/eval/raise — reflection rung 1

- **Priority:** P0
- **Depends on:** T4, T6
- **Tags:** rung-1, spine, reflection, crate:ufl-syntax, crate:ufl-predicate

## Context
Every evaluator is one-way and terminal: `ufl_core::eval(&Eml, &Env) -> Result<Value>` with `Value = Complex<f64>` only (crates/ufl-core/src/eval.rs:14,62), `eval_pred → bool` (crates/ufl-predicate/src/eval_pred.rs:63). No value can hold syntax, so `(quote e)` has no landing type — THE reflection blocker theory/two-language-substrate.md names ("No reflection ⇒ no self-modification"). Meanwhile the carrier is ready: `Sexpr` is Clone+PartialEq data (crates/ufl-syntax/src/sexpr.rs:8-16) with a pinned read/print round-trip (sexpr.rs:36-39), and both dispatch seams are documented and clean (`lower_form` lower.rs:44-45; `eval_form` eval_pred.rs:73-74). Lowering is also one-way: `lower: &Sexpr → Eml` (lower.rs:30) has no inverse anywhere in crates/, so even the Rust-side code↔data square is unclosable.

## Work (implements the T4 reflection requirement; test-first per §4 step 3)
1. **Third evaluation mode "syntax"** beside numeric/boolean: extend `Mode` (eval_pred.rs:27-30) and `classify` (eval_pred.rs:39-48). No god-enum — same discipline as the existing numeric/boolean split (ufl-predicate/src/lib.rs:7-11).
2. **`(quote e)`** yields the child Sexpr unevaluated (a Syntax-mode value).
3. **`(eval q)`** bridges syntax→numeric by discharging through the EXISTING `ufl_syntax::lower` + `ufl_core::eval` (lib.rs:43-47) — never a second evaluator (the R-0003 AC4 reuse rule).
4. **`=` on two syntax operands** is `Sexpr::PartialEq` — structural, exact, decidable (consistent with R-0004 AC2); numeric `=` unchanged (eval_pred.rs:90-92).
5. **`raise: &Eml -> Sexpr`** in crates/ufl-syntax (One→Num(1.0), Var→Sym, Node→(eml a b)) — the missing inverse of lower; this is what a future macro/orchestrator layer needs to hand typed results back to the rewriting surface.
6. Write the owed theory note: "full binary trees of one operator ≅ S-expressions of one head" (universal-computability.md §7.3, promised since R-0003 §2).

## Acceptance gate (falsifiable)
- Property test over ufl-prng-generated Sexprs in the reader's image: `⟦(= (eval (quote E)) E)⟧` holds for every sampled bound E (eval∘quote = identity).
- Negative: `(quote (eml y 1))` with y unbound discharges WITHOUT UnboundVariable (quote does not evaluate); `(quote e)` reached by the numeric path still fails typed (today: LowerError::UnknownForm, lower.rs:59).
- Theorem-shaped round trips: `lower(raise(t)) == Ok(t)` for ALL random Eml t (total) and `raise(lower(s)) == s` for all lowerable s, fuzzed via ufl-prng including deep trees (red-first).
- `read(raise(e).to_string()) == Ok(raise(e))` — the printed quote round-trips per the sexpr.rs:36-39 invariant.
- cargo test/clippy/fmt green.
- KILL: a constructible AST that cannot round-trip forces either a documented domain restriction in the spec or a redesign — no silent partiality.

## Must NOT claim
Metacircularity or self-modification — quote without a hosted eval is plumbing, and the docs must say so. Value→Sexpr reification stays out of scope (non-total by construction: inf/nan results, derived complexes).

## Files/crates
crates/ufl-predicate/src/{eval_pred.rs,lib.rs,predicate.rs}, crates/ufl-syntax/src/{lower.rs,sexpr.rs,lib.rs}, theory/universal-computability.md, specs/0016-*.md.
