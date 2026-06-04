# SPEC-0004 — Predicate Layer (`⟦P⟧`) — the Checker

- **Status:** Accepted (2026-06-02 — three-lens applied; owner accepted, exact equality confirmed)
- **Realizes:** R-0004
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-02
- **Depends on:** SPEC-0003 (`Sexpr`, `lower`), SPEC-0001 (`eval`, `Env`, `Value`)
- **Crate(s):** `ufl-predicate` (new) — depends on `ufl-syntax` and `ufl-core`

## 1. Motivation

SPEC-0004 realizes [R-0004](../requirements/0004-predicate-layer.md): UFL's
predicate atom `⟦P⟧` as a **checker**. It adds the **boolean and comparison
substrate** — booleans, `(= a b)`, `(and …)` / `(or …)` / `(not …)`, and the
predicate wrapper `(pred E)` — plus a `check` that evaluates a predicate over a
pre/post state to `true` or `false`.

This is **not** yet control universality. Following the honest ledger in
[`theory/universal-computability.md`](../theory/universal-computability.md) §6,
the control *constructions* — branching `(b ∧ S) ∨ (¬b ∧ T)`, sequencing,
recursion — are deferred. SPEC-0004 delivers the boolean substrate those
constructions are *expressed in*, and the `=` bridge from `eml`-valued
conditions to booleans. It is the first executable artifact on the
control-layer frontier; it does not cross it.

## 2. Design

### 2.1 The value model — two typed evaluators, no god-enum

UFL keeps **two typed evaluation modes**, bridged by `=`, rather than a
heterogeneous `Num | Bool` runtime value:

- **Numeric:** an `Sexpr` that denotes a number is lowered (`ufl_syntax::lower`)
  to `Eml` and evaluated by the **reused** `ufl_core::eval` → `Value`
  (`Complex<f64>`). *No numerics are added here* — the branch convention and the
  `sin(τ/2)` self-correction are inherited transitively.
- **Boolean:** `eval_pred : &Sexpr × &Env → Result<bool, PredError>` evaluates an
  `Sexpr` that denotes a predicate → Rust `bool`.

`=` bridges: it evaluates its operands *numerically* and returns a `bool`.
Booleans are the result type of `eval_pred`, never a numeric value. This honours
the synthesis (stay typed; reuse the verified core; never touch `ufl-core`'s
`Value`).

### 2.2 The single classifier

Both evaluators agree on what an `Sexpr` *is* through **one** function — there is
no second, drifting classification. This is the load-bearing guard for AC1.

```rust
enum Mode { Numeric, Boolean }

/// Which evaluation mode an Sexpr denotes, by its outermost shape.
fn classify(s: &Sexpr) -> Mode {
    match s {
        Sexpr::Sym(t) if t == "true" || t == "false" => Mode::Boolean,
        Sexpr::Sym(_) | Sexpr::Num(_) => Mode::Numeric,      // a variable, or a number
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(h), _)) if is_pred_head(h) => Mode::Boolean,
            _ => Mode::Numeric,                              // (eml …) or a non-form list
        },
    }
}

fn is_pred_head(h: &str) -> bool {
    matches!(h, "and" | "or" | "not" | "=" | "pred")
}
```

- `eval_pred(s)` requires `classify(s) == Boolean`; a `Numeric` shape in boolean
  position is `PredError::ExpectedBool`.
- `eval_num(s)` (the `=`-operand path) requires `classify(s) == Numeric`; a
  `Boolean` shape in numeric position is `PredError::ExpectedNumber` — caught
  *before* `lower`, so `(= true 1)` is a clean type error, **not** a downstream
  `UnboundVariable("true")`.

**The `true`/`false` rule, stated once.** `true` and `false` are boolean
literals **at the predicate/numeric boundary** — i.e. wherever `classify` is
consulted (a direct operand of `=`/`and`/`or`/`not`/`pred`, or a checked
predicate). They are *not* reserved globally: inside a numeric subtree, e.g.
`(eml true 1)`, the symbol `true` is an ordinary variable (numeric context), and
`(= (eml true 1) 1)` yields `PredError::Eval(UnboundVariable("true"))` unless
`true` is bound. Global reservation is deferred (it would couple `ufl-syntax` to
boolean concerns); the boundary rule suffices for the checker and is tested.

### 2.3 The predicate forms

| form | meaning | evaluation |
|------|---------|------------|
| `true` / `false` | boolean literals | → `true` / `false` |
| `(= a b)` | exact equality | `classify`-check both `Numeric`; eval each numerically; `true` iff `a == b` |
| `(and p …)` | conjunction | each `pᵢ` boolean; **lazy** short-circuit; `(and)` = `true` |
| `(or p …)` | disjunction | each `pᵢ` boolean; **lazy** short-circuit; `(or)` = `false` |
| `(not p)` | negation | `p` boolean; logical not; arity 1 |
| `(pred E)` | the `⟦P⟧` atom | arity 1; transparent at check time — evaluates `E` |

**Empty-connective identities are normative:** `(and)` = `true`, `(or)` = `false`
(the empty conjunction/disjunction). **Short-circuit is lazy and intentionally
suppresses errors in unreached operands:** `(and false X)` = `false` even if `X`
would error; `(or true X)` = `true` even if `X` would error. (Standard
boolean-short-circuit semantics; verdict may depend on operand order when an
unreached operand contains an error — this is by design, and tested.) AC6's
"earliest layer" governs *where a produced error surfaces*, not that unreached
operands must be evaluated.

### 2.4 Exact equality and its contract

`(= a b)` is **exact**, no tolerance (R-0004 AC2), realized as IEEE value
equality on `Complex<f64>` (`a == b`, component `PartialEq`): `+0.0 == −0.0`,
`NaN ≠ NaN` — the standard "equal as numbers" reading. (Literal bit-equality was
rejected: it would give `NaN == NaN` and `+0 ≠ −0`, both wrong for `=`.)

Three consequences are **documented contracts**, not surprises (each tested):

1. **`=` operands must be numerically lowerable.** Per R-0001, only `1` is a
   numeric *literal*; `(= 2 2)` does **not** lower (`2` → `LowerError::Unsupported‐
   Literal`). `=` operands are therefore `1`, a variable, or an `eml` form.
   (AC5's `(= x' (eml x 1))` respects this.)
2. **`=` is non-reflexive on `NaN`.** If an operand evaluates to `NaN` (reachable
   only via R-0001's extended-reals edges, e.g. `ln 0 = −∞` paths), `(= a a)` is
   `false`. So a predicate over a `NaN`-valued state is unsatisfiable by `=` —
   IEEE-correct, and a tested contract.
3. **`=` compares the full complex value (re *and* im).** A *clean real*
   post-state will not equal an `eml` result that carries a `sin(τ/2)`
   self-correction imaginary residue (~1e-16 i) — e.g. results through the
   `ln`-of-negatives path. (`(eml x 1) = exp(x)` is clean real, so AC5 is safe;
   the residue bites only self-correction comparisons.) The remedy —
   tolerance / real-projection equality — is deferred (R-0004 §4 non-goal); the
   exact contract is honoured and its limit is documented.

### 2.5 Pre/post-state, the priming convention, and its boundary

A predicate mentions pre-state variables (`x`) and post-state variables (`x'`,
primed — the reader admits `'` in symbols, so `x'` reads as one `Sym` with no
change to `ufl-core`). Checking binds both into one `Env` — `x` from the
pre-state, `x'` from the post-state — so the reused evaluator resolves them
unmodified.

**The priming convention is made injective.** Post vars are bound under
`name'`. To keep this unambiguous, **`check` rejects any pre/post binding name
that contains `'`** (it is reserved for the priming suffix) with a typed error.
So a pre-var cannot be named `x'`, and post-`x` (→ key `x'`) cannot collide with
a user pre-var. The *predicate text* still uses `x'` freely to reference
post-state; only the binding *names* passed to `check` are constrained.

```rust
/// Low-level: evaluate a predicate under an already-built env.
pub fn eval_pred(predicate: &Sexpr, env: &Env) -> Result<bool, PredError>;

/// Check a predicate against a pre-state and a post-state. Binds pre vars by
/// name and post vars under `name'`; rejects names containing `'`.
pub fn check(
    predicate: &Sexpr,
    pre: &[(&str, Value)],
    post: &[(&str, Value)],
) -> Result<bool, CheckError>;

/// Read + check from text, with the same pre/post priming as `check`.
pub fn check_str(
    src: &str,
    pre: &[(&str, Value)],
    post: &[(&str, Value)],
) -> Result<bool, CheckError>;
```

`check` and `check_str` share one state-binding convention (pre/post slices,
auto-primed). `eval_pred` is the low-level door for callers who build the `Env`
themselves.

### 2.6 The error model

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PredError {
    #[error("expected a boolean expression, found {found}")]
    ExpectedBool { found: String },
    #[error("expected a numeric expression, found {found}")]
    ExpectedNumber { found: String },
    #[error("`{form}` expects {expected} arguments, got {got}")]
    Arity { form: String, expected: usize, got: usize },
    #[error(transparent)]
    Lower(#[from] LowerError),   // a numeric `=` operand failed to lower
    #[error(transparent)]
    Eval(#[from] EvalError),     // a numeric operand had an unbound variable
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CheckError {
    #[error("reserved variable name `{0}`: state-variable names may not contain '\\''")]
    ReservedName(String),
    #[error(transparent)] Read(#[from] ReadError),
    #[error(transparent)] Pred(#[from] PredError),
}
```

No panics; every misuse is a typed variant surfaced at the earliest layer that
can detect it (R-0004 AC6).

## 3. Code outline

```rust
// eval_pred.rs
pub fn eval_pred(s: &Sexpr, env: &Env) -> Result<bool, PredError> {
    match s {
        Sexpr::Sym(t) if t == "true"  => Ok(true),
        Sexpr::Sym(t) if t == "false" => Ok(false),
        Sexpr::List(items) => eval_form(items, env),
        _ => Err(PredError::ExpectedBool { found: describe(s) }),
    }
}

fn eval_form(items: &[Sexpr], env: &Env) -> Result<bool, PredError> {
    let Some((Sexpr::Sym(head), args)) = items.split_first() else {
        return Err(PredError::ExpectedBool { found: "non-form list".into() });
    };
    match head.as_str() {
        "pred" => one_arg("pred", args).and_then(|e| eval_pred(e, env)),
        "not"  => one_arg("not", args).and_then(|p| Ok(!eval_pred(p, env)?)),
        "="    => {
            let [a, b] = two_args("=", args)?;
            Ok(eval_num(a, env)? == eval_num(b, env)?)
        }
        "and"  => { for p in args { if !eval_pred(p, env)? { return Ok(false); } } Ok(true) }
        "or"   => { for p in args { if  eval_pred(p, env)? { return Ok(true);  } } Ok(false) }
        // is_pred_head agrees with this match; any other head is non-boolean.
        other  => Err(PredError::ExpectedBool { found: format!("form `{other}`") }),
    }
}

/// Evaluate a numeric operand. A boolean-shaped operand is a type error,
/// caught before `lower` so the diagnostic is `ExpectedNumber`, not a
/// downstream unbound-variable error.
fn eval_num(s: &Sexpr, env: &Env) -> Result<Value, PredError> {
    if matches!(classify(s), Mode::Boolean) {
        return Err(PredError::ExpectedNumber { found: describe(s) });
    }
    let eml = ufl_syntax::lower(s)?;   // LowerError → PredError::Lower
    Ok(ufl_core::eval(&eml, env)?)     // EvalError  → PredError::Eval
}
```

Helpers `describe(&Sexpr) -> String`, `one_arg`/`two_args` (arity → `Arity`), and
`classify`/`is_pred_head` (§2.2) are implementation-local and defined in the PR.

## 4. Non-goals

- Sequencing (`;`), parallel (`∃`), recursion/fixpoints, the orchestrator.
- Approximate / tolerance / real-projection equality (the §2.4-(3) remedy).
- Ordering (`<`, `≤`); quantifiers.
- A heterogeneous runtime `Value` enum (explicitly avoided — §2.1).
- Globally reserving `true`/`false` (only the boundary rule, §2.2).

## 5. Open questions

*None blocking.* The classifier (§2.2), the equality contracts (§2.4), the
priming boundary (§2.5), and the short-circuit semantics (§2.3) are fixed. The
`(pred E)` transparency is now a decision (§7), not an open question.

## 6. Acceptance criteria

- [ ] **AC1** — `eval_pred` yields `bool`. `true`/`false` evaluate. A numeric
  shape in boolean position → `ExpectedBool`; a boolean shape as a `=` operand →
  `ExpectedNumber` (tested for `(= true 1)`, `(= (and true) 1)`, `(not (eml 1 1))`,
  `(and (eml 1 1) true)`) — caught at the boundary, never a downstream
  unbound-variable error, never a panic.
- [ ] **AC2** — `(= a b)` returns exact IEEE equality of the two numerically-
  evaluated `Value`s. Contracts tested: `(= 2 2)` → `Lower(UnsupportedLiteral)`;
  a `NaN`-valued `(= a a)` → `false` (non-reflexive); a clean-real vs residue
  pair → `false`.
- [ ] **AC3** — `(and …)`/`(or …)`/`(not p)` truth tables; `(and)` = `true`,
  `(or)` = `false`; `(not)` / `(not a b)` → `Arity`; lazy short-circuit
  suppresses an unreached erroring operand (`(and false <unbound>)` → `false`).
- [ ] **AC4** — `check(pred, pre, post)` binds pre by name, post primed; an
  unbound state variable → `Pred(Eval(UnboundVariable))`; a binding name
  containing `'` → `CheckError::ReservedName`.
- [ ] **AC5** — `⟦ x' = (eml x 1) ⟧` checks `true` when `post[x] = e^{pre[x]}`
  and `false` for a deliberately wrong post-state, over several real `x`.
- [ ] **AC6** — every failure is a typed `PredError` / `CheckError` variant,
  never a panic, at the earliest detecting layer.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-02 | **Two typed evaluators, no god-enum.** Numeric → `Complex` via reused `ufl_core::eval`; boolean → `bool` via `eval_pred`; `=` bridges. | The synthesis answer; reuse the verified core, never pollute `Value`. |
| 2026-06-02 | New crate `ufl-predicate` (→ `ufl-syntax`, `ufl-core`). | Bounded responsibility; deps inward. |
| 2026-06-02 | **One `classify(&Sexpr) → Mode`** consulted by both `eval_pred` and `eval_num`; `true`/`false` are boolean at the boundary, variables inside numeric subtrees. | Collapses the boolean/numeric type guard into a single auditable function (no drift); fixes the `(= true 1)` → `UnboundVariable` leak the three-lens review found. |
| 2026-06-02 | Exact `=` is IEEE `Complex` `==`; non-reflexive on `NaN`; compares re+im. Three contracts documented + tested (§2.4). | `=` means equal-as-numbers; the NaN/residue limits are IEEE-correct and narrow (AC5's `exp` path is clean); tolerance equality is the deferred remedy. |
| 2026-06-02 | Lazy short-circuit `and`/`or` **intentionally suppress errors in unreached operands**; `(and)`=`true`, `(or)`=`false`. | Standard boolean semantics; AC6 governs where produced errors surface, not evaluation of unreached operands. |
| 2026-06-02 | `check` **rejects pre/post binding names containing `'`** (`CheckError::ReservedName`); `check`/`check_str` share the pre/post priming; `eval_pred` is the raw-env door. | Makes the priming injective (no `x'`-collision); one ergonomic convention across the text and tree entry points. |
| 2026-06-02 | `(pred E)` kept, transparent at check time. | It is the named `⟦P⟧` atom and the **marked boundary** a future solver pattern-matches to find proof obligations; present-now/meaningful-later costs nothing and avoids a syntax break. |

## Changelog

- 2026-06-02 — created (Draft).
- 2026-06-02 — three-lens review applied (architect REQUEST CHANGES, hater
  NEEDS WORK, nice-guy STRONG WORK): defined the single `classify` (fixing the
  `true`/`false` type-confusion + undefined `is_boolean_form`); documented the
  three exact-`=` contracts (operand-lowerability, NaN non-reflexivity, complex
  residue); made the priming injective (`'` reserved, `ReservedName`); unified
  `check`/`check_str`; pinned short-circuit + empty-connective semantics;
  promoted `(pred E)` to a decision; reworded §1 to "boolean substrate", not
  "control layer" (matching the honest ledger). Open questions closed.
