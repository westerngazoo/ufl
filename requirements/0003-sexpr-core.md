# R-0003 — Homoiconic S-Expression Core

- **Status:** Draft
- **Milestone:** M1
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-05-28
- **Pillar / atom:** UFL's single abstract syntax — the homoiconic AST that all
  atoms (`eml`, and later `𝒢ₖ`, `∗`, `⟦P⟧`, `⊗`) are *forms* within
- **Depends on:** R-0001 (behavioral parity target — the `eml` semantics)
- **Supersedes (representation only):** the `Eml` enum of SPEC-0001; the typed
  `Multivector`/`GradeLift` ASTs of R-0002 (paused). The *behavioral*
  requirements of R-0001 survive and are re-verified here.
- **Realized by:** SPEC-0003 (pending)
- **QA:** `qa` agent run scoped to R-0003

## 1. Statement

UFL's single abstract syntax is a **homoiconic S-expression**. Every UFL program
*and* every UFL datum is an S-expression — **code is data**. There is one tree
type; the atoms (`eml` now; `𝒢ₖ`, `∗`, `⟦P⟧`, `⊗` later) are **forms** — symbols
in operator position — not bespoke Rust types.

```
;; UFL, as s-expressions
(eml 1 1)                          ; e
(eml x 1)                          ; exp(x)
(eml 1 (eml (eml 1 x) 1))          ; ln(x)
```

This replaces the per-layer *typed* ASTs — the `Eml` enum (R-0001) and the
`Multivector`/`GradeLift` enums (R-0002) — with one uniform tree. Evaluation
produces a **dynamically-typed runtime value**.

R-0003 delivers the core: the `Sexpr` type, a **reader** (text → `Sexpr`), the
dynamic runtime value, and an **evaluator** for the `eml` form — re-deriving
R-0001's numeric behaviour through the s-expression path. Geometric, predicate,
and substrate forms are later requirements.

## 2. Rationale

UFL's thesis is *everything is a uniform tree of one operator* — `eml` all the
way down, the continuous analogue of NAND. **Homoiconicity lifts that thesis to
the meta level**: the language itself is a uniform tree, and code is
indistinguishable from data. The grammar `S → 1 | var | eml(S, S)` already *is*
an S-expression grammar; making the AST an S-expression is the honest
realization of what UFL has been all along.

It also serves the part of UFL that most needs it. The substrate orchestrator
(atom `⊗`) compiles and rewrites an expression to a target substrate — CPU,
FPGA, analog. That is **tree rewriting**, exactly what a homoiconic S-expression
makes natural. `docs/the-shape-of-ufl.md` already frames "EML trees are the
universal *how*" that the orchestrator transforms; a uniform S-expression IR is
that "universal how" made concrete. One AST, one reader, one evaluator dispatch
— adding an atom becomes adding a *form*, never a new type plus a new dispatch
path.

## 3. Acceptance criteria

- **AC1 — Homoiconic representation.** There is exactly one syntax tree type,
  `Sexpr`: an *atom* (a number or a symbol) or a *list* of `Sexpr`. A UFL
  program is an `Sexpr` and is itself ordinary data — constructible,
  traversable, and comparable. Code and data share one representation.
- **AC2 — Reader.** Text S-expressions parse to `Sexpr`: `(eml 1 1)`,
  `(eml x 1)`, arbitrary nesting, insignificant whitespace, and line comments.
  Malformed input (unbalanced parentheses, empty application, stray tokens)
  yields a **typed parse error** — never a panic.
- **AC3 — `eml` form evaluation.** `(eml a b)` evaluates to
  `exp(eval a) − ln(eval b)` over the dynamic runtime value. The literal `1` is
  a number; a symbol in argument position resolves from an environment; an
  unbound symbol yields a **typed evaluation error**.
- **AC4 — Behavioural parity with R-0001.** Through the S-expression path, the
  identities hold within R-0001's tolerance, over inputs including negative
  real `x`:
  - `(eml 1 1) = e`
  - `(eml x 1) = exp(x)`
  - `(eml 1 (eml (eml 1 x) 1)) = ln(x)`

  The R-0001 / Q-AC4 branch convention carries over: derived `i`, `τ`, and
  `ln x` for `x < 0` are principal-correct.
- **AC5 — Extended reals.** `ln 0`, `exp(−∞)`, and signed-zero/infinity cases
  follow IEEE-754 and propagate as ordinary values — no trap, panic, or abort
  (R-0001 AC3, carried over).
- **AC6 — Dynamic value and error model.** Evaluation yields a runtime value;
  every misuse is a **typed error, never a panic** — an unbound symbol, a head
  symbol that names no known form, or a form applied with the wrong argument
  count. (Because the typed enums are dropped, these checks are *runtime*, not
  compile-time — the explicitly accepted tradeoff of §6.)

## 4. Constraints & non-goals

**Constraints**

- One `Sexpr` AST; no per-layer typed syntax trees.
- The runtime value is dynamically typed (extensible to multivectors and beyond
  in later requirements); R-0003 needs only the numeric (complex) case.
- `eml` semantics, the complex substrate, and the branch convention are
  inherited from R-0001 unchanged — only the *representation* changes.

**Non-goals** (separate, later requirements)

- Geometric, predicate, and substrate **forms** (`𝒢ₖ`, `∗`, `⟦P⟧`, `⊗`).
- **Macros / quasiquote / reader macros** — homoiconicity is established; the
  metaprogramming layer that exploits it is later.
- **R-0002's typed multivector implementation** — paused; GA will be
  re-expressed as S-expression forms in a later requirement.
- Optimization, substrate compilation, `let`/binding surface sugar.

## 5. Open questions

- **Numeric literals in the reader.** R-0001's *semantics* admit only `1` as a
  primitive constant (everything else is a derived `eml` tree). Does the
  *reader's tokenizer* nonetheless accept arbitrary numeric literals (`3`,
  `-1`, `2.5`) for test ergonomics and practicality, with the "only `1` is
  primitive" claim remaining a statement about the *language*, not the lexer?
  SPEC-0003 decides.
- **Runtime value shape.** A `Value` enum with a `Num(Complex<f64>)` variant
  now, designed to grow (multivectors, booleans) — versus keeping R-0003
  strictly complex. SPEC-0003 decides, mindful of later GA/predicate forms.
- **Crate placement.** Since R-0003 supersedes R-0001's representation, does it
  reshape `ufl-core` in place (replacing `eml.rs`/`eval.rs`) or land a new
  `ufl-sexpr` crate that `ufl-core` builds on? SPEC-0003 decides; the architect
  reviews the boundary.
- **`Eml` enum disposition.** Is the R-0001 `Eml` enum deleted outright, or kept
  transitionally behind the s-expr core during migration? SPEC-0003 decides.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | UFL adopts a **homoiconic S-expression** as its single AST (the "full LISP" direction); all atoms become forms within it. | Homoiconicity is the meta-level form of UFL's "everything is a uniform tree of one operator" thesis; the orchestrator's substrate-rewriting job is tree rewriting, which S-expressions make natural; one AST / reader / evaluator maximizes composability. Owner decision. |
| 2026-05-28 | **Drop the typed enums** (`Eml`, `Multivector`, `GradeLift`) in favour of one `Sexpr` AST + one dynamically-typed runtime value. **Accepted tradeoff, recorded:** this forfeits the compile-time structural guarantees that the typed enums gave — the `Eml` enum made an ill-formed grammar unrepresentable, and the three-lens review used `Multivector`/`GradeLift` precision to catch real bugs. In exchange UFL gains homoiconic uniformity and first-class tree rewriting. Errors that were compile-time (e.g. applying `eml` to a non-number) become **runtime** typed errors (AC6). | Owner chose the "full LISP, drop typed enums" option over the "s-expr IR + typed core" synthesis. Per CLAUDE.md §1 the choice is discussed and recorded here; the engineer flagged the loss of structural safety as the cost being paid. |
| 2026-05-28 | R-0003 **supersedes the representation** of SPEC-0001 (the `Eml` enum), but **not** R-0001's behavioural requirement. `eml = exp(x) − ln(y)`, the `e`/`exp`/`ln` identities, the complex substrate, and the Q-AC4 branch convention all carry over and are re-verified through the S-expression evaluator (AC4/AC5). | The pivot is representational, not semantic — the EML atom and its behaviour are unchanged; only the AST that holds them changes. R-0001's tests largely carry over, re-expressed against the s-expr API. |
| 2026-05-28 | **R-0002 (typed geometric algebra) is paused**, frozen on branch `R-0002-geometric-algebra` (tip `c92a38a`, TDD-red), recoverable. Its GA behaviour will be re-expressed as S-expression forms in a later requirement. | The pivot drops the typed approach R-0002 was built on; finishing it as typed enums would be throwaway work. Freezing (not deleting) preserves the design and the qa test plan for reuse. |

## Changelog

- 2026-05-28 — created (Draft).
