# R-0005 — Value Conditional (`if`)

- **Status:** Draft
- **Milestone:** M3
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-04
- **Pillar / atom:** Control — a numeric expression selected by a predicate.
  The bool→value bridge (the complement of `=`'s value→bool).
- **Depends on:** R-0004 (predicates — the condition) and R-0001/R-0003 (the
  numeric operands lower to `Eml` and evaluate via the reused core).
- **Realized by:** SPEC-0005 (pending)
- **QA:** `qa` agent run scoped to R-0005

## 1. Statement

UFL gains the **value conditional** `(if b a c)`: a *numeric* expression whose
value is the numeric value of `a` when the predicate `b` holds, and of `c`
otherwise.

```
(if (= flag 1) (eml x 1) z)      -- "e^x when flag = 1, else z"
```

`b` is a boolean expression (R-0004 predicate); `a` and `c` are numeric (`eml`)
expressions. `if` is the **bool→value bridge** — the complement of `=`, which
is the value→bool bridge. It is the first form whose evaluation *mixes both
modes*: it asks a predicate, then yields a number.

### Why this, and not predicate-branching

Predicate-branching `(if b S T)` over *predicates* `S`, `T` is already
derivable from R-0004 — `(or (and b S) (and (not b) T))`, verified. So a
predicate-level `if` would be mere sugar. The genuinely new construct is
selecting a **value** by a predicate, which `and`/`or`/`not` (boolean-valued)
cannot express. That is R-0005.

## 2. Rationale

`if` is the usable meaning of "branching" — choosing *which value to compute*
based on a test. It is a real computation building block and the natural next
step after the predicate checker: the checker can *ask* questions (`=`,
`and`/`or`/`not`); `if` lets an expression's *value* depend on the answer.

Architecturally it has a consequence worth stating in the requirement: `(if …)`
has no `Eml` representation (the numeric core has no conditional node), so a
numeric expression containing `if` cannot be lowered wholesale to `Eml`. The
conditional therefore lives one layer up (with the predicates), and the numeric
evaluator that handles it delegates pure-`eml` subtrees to the **reused**
`lower` + `eval` — the verified numerics stay untouched.

## 3. Acceptance criteria

- **AC1 — Selection.** `(if b a c)` evaluates `b` as a predicate; if `b` holds
  the result is the numeric value of `a`, otherwise of `c`. The *unselected*
  branch's value is not required (lazy: only the taken branch is evaluated).
- **AC2 — Mixed-mode typing.** `b` must be a boolean expression (a numeric `b`
  is a typed error); `a` and `c` must be numeric expressions (a boolean `a`/`c`
  is a typed error). `if` has arity exactly 3. No coercion, no panic.
- **AC3 — Nesting.** `if` may nest in its own branches:
  `(if b1 (if b2 a1 a2) c)` evaluates correctly. (`if` inside an `eml` node is
  **not** supported — `eml` subtrees are pure and lower wholesale — and yields
  a typed error.)
- **AC4 — As a `=` operand.** `(= x' (if b a c))` checks correctly: the
  conditional's selected value is compared (exactly, per R-0004) to the other
  operand.
- **AC5 — Worked example.** `⟦ x' = (if (= flag 1) (eml x 1) z) ⟧` checks
  `true` exactly when: `flag = 1` and `x'` equals the eml-computed `e^x`; or
  `flag ≠ 1` and `x'` equals `z`. Demonstrated over both branches, true and
  false post-states.
- **AC6 — Layered typed errors.** Lazy evaluation: an error in the *unselected*
  branch is not surfaced. An error in the selected branch, a non-boolean
  condition, a non-numeric branch, wrong arity, or `if` inside `eml` — each is a
  typed error at the earliest detecting layer, never a panic. Composes with
  R-0004's `PredError` / `CheckError`.

## 4. Constraints & non-goals

**Constraints**

- `b` is evaluated by the R-0004 predicate evaluator; `a`/`c` by the numeric
  evaluator (which reuses `lower` + `ufl_core::eval` for pure-`eml` subtrees).
- **Lazy:** only the selected branch is evaluated (AC1/AC6).
- `if` is a numeric form; it cannot appear inside an `eml` node (AC3).

**Non-goals** (later)

- Sequencing (`;`) and recursion/fixpoints — the remaining control constructs.
- Ordering comparisons, numeric literals other than `1`, the EML compiler.
- The substrate orchestrator (selecting *where* to run).
- A boolean-valued `if` (predicate-branching) — already derivable, not added.

## 5. Open questions

- **Crate / entry point.** `if` needs both the predicate evaluator and the
  numeric evaluator, so it lives in `ufl-predicate`. Does R-0005 also expose a
  public "evaluate a (conditional) numeric expression to a `Value`" entry
  (`eval_value` / `eval_value_str`), or is `if` reached only as a `=` operand
  for now? SPEC-0005 decides.
- **`if` inside `eml` diagnostic.** `(eml (if …) 1)` currently fails as
  `LowerError::UnknownForm("if")`. Confirm that is the intended typed error, or
  whether a clearer "`if` not allowed in a numeric `eml` subtree" message is
  warranted. SPEC-0005 decides.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-04 | R-0005 is the **value conditional** `(if b a c)`, not predicate-branching. | Predicate-branching is derivable from R-0004's `and`/`or`/`not` (verified); selecting a *value* by a predicate is genuinely new and is the usable meaning of branching. (Owner decision.) |
| 2026-06-04 | `if` is a numeric form evaluated one layer up (in `ufl-predicate`), delegating pure-`eml` subtrees to the reused `lower` + `eval`; it cannot appear inside an `eml` node. | `(if …)` has no `Eml` node, so a numeric tree with `if` can't lower wholesale; keeping `eml` subtrees pure preserves verbatim reuse of the verified evaluator. |
| 2026-06-04 | Evaluation is **lazy** — only the selected branch is evaluated. | Standard conditional semantics; lets a predicate guard an otherwise-erroring branch (e.g. an unbound variable in the untaken arm). |

## Changelog

- 2026-06-04 — created (Draft).
