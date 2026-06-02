# R-0003 — Homoiconic S-Expression Core

- **Status:** Accepted (2026-05-28 — synthesis, endorsed by the three-lens review)
- **Milestone:** M2
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-05-28
- **Pillar / atom:** UFL's single surface syntax & IR — the homoiconic tree that
  all atoms (`eml`, and later `𝒢ₖ`, `∗`, `⟦P⟧`, `⊗`) are *forms* within
- **Depends on:** R-0001 (the `Eml` typed core that the `eml` form lowers into)
- **Builds on (does not supersede):** R-0001's `Eml` enum and evaluator are
  **kept** as the lowering target. R-0002's typed `Multivector` will be the
  lowering target for the geometric forms (later requirement).
- **Realized by:** SPEC-0003 (pending)
- **QA:** `qa` agent run scoped to R-0003

## 1. Statement

UFL gains a **homoiconic S-expression** as its single surface syntax and
intermediate representation. Every UFL program *and* every UFL datum is an
S-expression — **code is data** — and the atoms (`eml` now; `𝒢ₖ`, `∗`, `⟦P⟧`,
`⊗` later) are **forms**: symbols in operator position.

```
;; UFL, as s-expressions
(eml 1 1)                          ; e
(eml x 1)                          ; exp(x)
(eml 1 (eml (eml 1 x) 1))          ; ln(x)
```

Crucially, the S-expression is a **front-end that lowers into the typed core**.
The pipeline is:

```
text ──reader──▶ Sexpr ──lower──▶ Eml ──eval──▶ Value
```

The reader is uniform; a **lowering pass** translates a well-formed `eml` form
into R-0001's `Eml` enum; R-0001's verified evaluator computes the result. The
homoiconic, code-as-data, tree-rewritable surface is delivered in full — *and*
the typed core's compile-time structural guarantees (R-0001 AC1) and its
delicate, verified numerics (the `sin(τ/2)` self-correction) are **reused, not
re-implemented**.

R-0003 delivers the core for the `eml` form: the `Sexpr` type, a **reader**
(text → `Sexpr`), and a **lowering pass** (`Sexpr → Eml`) feeding R-0001's
evaluator. Geometric, predicate, and substrate forms are later requirements;
each lowers into its own typed core (e.g. GA forms → R-0002's `Multivector`).

## 2. Rationale

UFL's thesis is *everything is a uniform tree of one operator* — `eml` all the
way down, the continuous analogue of NAND. **Homoiconicity lifts that thesis to
the meta level**: the language itself is a uniform tree, and code is
indistinguishable from data. The grammar `S → 1 | var | eml(S, S)` already *is*
an S-expression grammar — indeed full binary trees of one operator are
*isomorphic* to S-expressions of one head symbol (a clean formal fact worth a
`theory/` note). UFL is not bolting on homoiconicity; it is recognizing the
structure was a special case of a uniform tree all along. **UFL is the
continuous LISP.**

It also serves the part of UFL that most needs it. The substrate orchestrator
(atom `⊗`) compiles and rewrites an expression to a target substrate — CPU,
FPGA, analog. That is **tree rewriting**, which a homoiconic S-expression makes
native: the orchestrator becomes one `Sexpr → Sexpr` rewriter rather than a
separate compiler per typed layer. One AST, one reader, one dispatch — adding
an atom becomes adding a *form*.

**Why a lowering front-end, not a dynamic replacement (the three-lens review).**
The first draft of R-0003 went "full LISP" — dropping the typed enums for a
dynamically-typed runtime value. The three-lens review (architect, hater,
nice-guy) converged: the homoiconic *direction* is right and thesis-aligned,
but every benefit it unlocks is a property of the **S-expression AST**, not of
dropping the types. Going dynamic would forfeit the compile-time structural
guarantees that caught the SPEC-0002 rotor-sign bug *at design time*, push a
whole class of arity/type/grade errors to runtime (dangerous on FPGA/analog
substrates with no `Result`), conflict with `CLAUDE.md` §2/§6, and risk the
1-ulp `sin(τ/2)` self-correction during re-implementation — all for a
uniformity the lowering approach delivers for free. UFL therefore adopts the
**synthesis**: homoiconic S-expression surface/IR lowering into the typed core.

## 3. Acceptance criteria

- **AC1 — Homoiconic representation.** There is one syntax tree type, `Sexpr`:
  an *atom* (a number or a symbol) or a *list* of `Sexpr`. A UFL program is an
  `Sexpr` and is itself ordinary data — constructible, traversable, and
  comparable. Code and data share one representation.
- **AC2 — Reader.** Text S-expressions parse to `Sexpr`: `(eml 1 1)`,
  `(eml x 1)`, arbitrary nesting, insignificant whitespace, and line comments.
  Malformed input (unbalanced parentheses, empty application, stray tokens)
  yields a **parse error reported via a typed `ReadError` enum** — never a
  panic.
- **AC3 — Lowering the `eml` form.** A well-formed `eml` form lowers to R-0001's
  `Eml`: `1` → `Eml::One`, a symbol → `Eml::Var`, `(eml a b)` →
  `Eml::node(lower a, lower b)`. Lowering **validates structure at lowering
  time** — an unknown head symbol, an `eml` with other than two arguments, or a
  non-form list yields a **lowering error reported via a typed `LowerError`
  enum**. This check runs *before* evaluation (and, later, before substrate
  compilation), so structural validity of a known form is recovered by the
  type system at the lowering boundary, not deferred to runtime on a substrate
  that cannot report it.
- **AC4 — Behavioural parity with R-0001, through the typed core.** Lowering
  feeds R-0001's *existing, verified* evaluator — the numerics are reused, not
  re-implemented. Through the `text → Sexpr → Eml → eval` path the identities
  hold within R-0001's tolerance, over inputs including negative real `x`:
  - `(eml 1 1) = e`
  - `(eml x 1) = exp(x)`
  - `(eml 1 (eml (eml 1 x) 1)) = ln(x)`

  The R-0001 / Q-AC4 branch convention and its `sin(τ/2)` tripwire carry over
  unchanged (they live in the reused `Eml` evaluator).
- **AC5 — Extended reals.** Inherited from R-0001 via the reused evaluator:
  `ln 0`, `exp(−∞)`, and signed-zero/infinity cases follow IEEE-754 and
  propagate as ordinary values — no trap, panic, or abort.
- **AC6 — Error model: typed, layered, no panics.** Every failure surfaces as a
  typed error enum, never a panic, at the earliest layer that can detect it:
  `ReadError` (lexical/syntactic), `LowerError` (unknown form, wrong arity,
  non-form) at the lowering boundary, and R-0001's `EvalError::UnboundVariable`
  at evaluation. Errors that the *typed core* makes structurally impossible
  (an `eml` node with the wrong child count) remain impossible once lowered —
  the dynamic frontier is only the genuinely dynamic part (raw text, unbound
  symbols, unknown heads).

## 4. Constraints & non-goals

**Constraints**

- One `Sexpr` AST as the surface/IR; the typed cores (`Eml` now, `Multivector`
  later) are **retained** as lowering targets.
- R-0003's only runtime value is R-0001's `Value` (`Complex<f64>`), produced by
  the reused evaluator. A heterogeneous runtime value (multivectors, booleans)
  arrives with the forms that need it, in later requirements.
- `eml` semantics, the complex substrate, and the branch convention are
  R-0001's, unchanged.

**Non-goals** (separate, later requirements)

- Geometric, predicate, and substrate **forms** (`𝒢ₖ`, `∗`, `⟦P⟧`, `⊗`) and
  their lowering into their typed cores.
- **Macros / quasiquote / reader macros** — homoiconicity is established here;
  the metaprogramming layer that exploits it (and the orchestrator-as-rewriter)
  is later.
- Optimization / substrate compilation; an `Sexpr` pretty-printer is a small
  follow-on, not required here (though see §5).

## 5. Open questions

- **Numeric literals in the reader.** R-0001's *semantics* admit only `1` as a
  primitive constant. Does the *reader's tokenizer* nonetheless accept arbitrary
  numeric literals (`3`, `-1`, `2.5`) for test ergonomics, with "only `1` is
  primitive" remaining a statement about the language, not the lexer? SPEC-0003
  decides.
- **Crate placement.** Does the `Sexpr` + reader + lowering pass live in
  `ufl-core` beside `eml`, or in a new `ufl-syntax` crate that depends on
  `ufl-core`? The architect reviews the boundary; the front-end naturally
  depends on the core (inward), favouring a separate crate, but a submodule is
  acceptable for one form.
- **Dispatch seam.** Should the lowering dispatch (head symbol → form lowerer)
  be a *form table* rather than a hardcoded `match`, so the later
  orchestrator/macro layer can register rewrites against the same table? A
  design seam for SPEC-0003 to get right while cheap (nice-guy opportunity).
- **Runtime `Value` model.** When heterogeneous results arrive (GA forms), a
  uniform runtime `Value` (union of result types) is wanted — orthogonal to the
  typed IR. Isolate it in one module with an invariant tripwire when it lands
  (the SPEC-0001 `ln_eml` pattern). Out of scope for R-0003 (complex-only) but
  flagged so SPEC-0003 does not foreclose it.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | UFL adopts a **homoiconic S-expression** as its single surface syntax and IR; all atoms become forms. | Homoiconicity is the meta-level form of UFL's "everything is a uniform tree of one operator" thesis (full binary trees of one operator ≅ S-expressions); the orchestrator's substrate-rewriting job is tree rewriting, which S-expressions make native; one AST / reader / dispatch maximizes composability. Owner decision; all three review lenses endorsed the direction. |
| 2026-05-28 | **Synthesis, not full-dynamic:** the S-expression is a front-end that **lowers into the typed core** (`Eml` now, `Multivector` later); the typed enums are **kept**, not dropped. Supersedes the earlier (same-day) draft decision to drop the typed enums. | The three-lens review converged (architect REQUEST CHANGES → synthesis; hater NEEDS WORK; nice-guy's pros all hold under the synthesis): every LISP benefit is a property of the S-expr AST, not of dynamic typing. Lowering keeps R-0001 AC1's structural guarantees, the design-time bug-catching the rotor-sign fix relied on, `CLAUDE.md` §2/§6 compliance, and the 1-ulp `sin(τ/2)` self-correction (reused, not re-implemented) — at no cost to homoiconicity, tree-rewriting, or macros. Owner accepted the synthesis. |
| 2026-05-28 | R-0001's `Eml` enum and evaluator are **retained as the lowering target**; R-0003 *builds on* R-0001 rather than superseding it. Resolves the earlier draft's open "Eml disposition" question to "kept." | The typed core is the lowering target and the cross-check oracle for AC4 parity; deleting it would discard merged, qa-signed work and the verified numerics. |
| 2026-05-28 | **R-0002 (typed geometric algebra) is un-paused** — its `Multivector`/`GradeLift` become the lowering target for the future geometric forms, so the work is reused, not throwaway. | Under the synthesis the typed GA core is needed, not discarded; R-0002 can resume (finish the Cayley table → green) on its own track, and a later requirement adds the s-expr GA forms that lower into it. |

## Changelog

- 2026-05-28 — created (Draft, "full LISP / drop typed enums").
- 2026-05-28 — revised to the **synthesis** after the three-lens review:
  homoiconic S-expression front-end lowering into the retained typed core. AC3
  reframed as a lowering pass with lowering-time validation; AC4 parity now
  reuses R-0001's evaluator; AC6 reframed as a layered typed-error model; R-0002
  un-paused as the GA lowering target; decision log records the review outcome.
