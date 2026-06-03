# SPEC-0004 — Predicate Layer (`⟦P⟧`) — the Checker

- **Status:** Draft
- **Realizes:** R-0004
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-02
- **Depends on:** SPEC-0003 (`Sexpr`, `lower`), SPEC-0001 (`eval`, `Env`, `Value`)
- **Crate(s):** `ufl-predicate` (new) — depends on `ufl-syntax` and `ufl-core`

## 1. Motivation

SPEC-0004 realizes [R-0004](../requirements/0004-predicate-layer.md): UFL's
predicate atom `⟦P⟧` as a **checker**. It adds booleans, the comparison `(= a b)`,
the connectives `(and …)` / `(or …)` / `(not …)`, and the predicate wrapper
`(pred E)`, plus a `check` that evaluates a predicate over a pre/post state to
`true` or `false`. It is the first piece of UFL's control layer
([`theory/universal-computability.md`](../theory/universal-computability.md),
Route B).

## 2. Design

### 2.1 The value model — two typed evaluators, no god-enum

The crux. UFL keeps **two typed evaluation modes**, bridged by `=`, rather than a
heterogeneous `Num | Bool` runtime value:

- **Numeric:** an `Sexpr` that denotes a number is lowered (`ufl_syntax::lower`)
  to `Eml` and evaluated by the **reused** `ufl_core::eval` → `Value`
  (`Complex<f64>`). *No numerics are added here* — the branch convention and the
  `sin(τ/2)` self-correction are inherited.
- **Boolean:** a new `eval_pred : &Sexpr × &Env → Result<bool, PredError>`
  evaluates an `Sexpr` that denotes a *predicate* (a boolean expression) → Rust
  `bool`.

`=` is the bridge: it evaluates its two operands *numerically* and returns a
`bool`. Booleans are therefore never values in the numeric substrate; they are
the result type of `eval_pred`. This honours the synthesis (stay typed; no god
enum; reuse the verified core) and keeps R-0004 from touching `ufl-core`'s
`Value`.

### 2.2 Crate

A new crate **`ufl-predicate`** depends on `ufl-syntax` (for `Sexpr`, `lower`,
`LowerError`) and `ufl-core` (for `eval`, `Env`, `Value`, `EvalError`).
Dependencies point inward: `ufl-predicate → ufl-syntax → ufl-core`. Predicate
logic is a bounded responsibility distinct from syntax.

```
crates/ufl-predicate/
├── Cargo.toml          # → ufl-syntax, ufl-core
└── src/
    ├── lib.rs          # #![forbid(unsafe_code)]; re-exports; check / check_str
    └── eval_pred.rs    # eval_pred, PredError, the form dispatch
└── examples/
    └── hello_pred.rs   # ⟦ x' = (eml x 1) ⟧ checked true/false
```

### 2.3 The predicate forms

Evaluated by `eval_pred` (boolean position):

| form | meaning | evaluation |
|------|---------|------------|
| `true` / `false` (symbols) | boolean literals | → `true` / `false` |
| `(= a b)` | exact equality | eval `a`, `b` *numerically*; `true` iff `a == b` |
| `(and p …)` | conjunction | each `pᵢ` boolean; `true` iff all `true` (short-circuit) |
| `(or p …)` | disjunction | each `pᵢ` boolean; `true` iff any `true` (short-circuit) |
| `(not p)` | negation | `p` boolean; logical not |
| `(pred E)` | the `⟦P⟧` atom — mark `E` as a predicate | transparent for the checker: evaluates `E` |

Anything else in boolean position — a number, a bare non-`true`/`false` symbol,
or a *numeric* form like `(eml …)` — is a `PredError::ExpectedBool`. Symmetrically
`=`'s operands must be numeric; a boolean form (`true`, `(and …)`) as an operand
of `=` is `PredError::ExpectedNumber`.

### 2.4 Exact equality

`(= a b)` is **exact**, no tolerance (R-0004 AC2). It is realized as IEEE value
equality on `Complex<f64>` (`a == b`, i.e. component `PartialEq`): so `+0.0`
equals `−0.0`, and `NaN ≠ NaN` — the standard "equal as numbers" semantics, the
least surprising reading of `=`. (Literal bit-equality — `to_bits` — was
considered; it would make `NaN == NaN` and `+0 ≠ −0`, both wrong for a numeric
`=`. The R-0004 "exact, no tolerance" decision is honoured; the IEEE reading is
the natural realization.) When both sides are computed the same way (the checker
use case), they are bit-identical and equal under either reading; the difference
only appears at `±0`/`NaN`, where IEEE is correct.

### 2.5 Pre/post-state as an env convention

A predicate mentions pre-state variables (`x`) and post-state variables (`x'`,
primed — the reader already admits `'` in symbols). **Checking does not change
the numeric evaluator:** `x` and `x'` are just two symbols. The checker binds
both into a single `Env` — `x` from the pre-state, `x'` from the post-state — so
the reused `ufl_core::eval` resolves them with no modification.

```rust
/// Check a predicate against a pre-state and a post-state.
/// Binds pre vars under their name, post vars under `name'`.
pub fn check(
    predicate: &Sexpr,
    pre: &[(&str, Value)],
    post: &[(&str, Value)],
) -> Result<bool, PredError>;

/// Convenience: read + check from text, given an already-built Env.
pub fn check_str(src: &str, env: &Env) -> Result<bool, CheckError>;
```

`check` builds the combined `Env` (post vars get a `'` suffix) and calls
`eval_pred`. `check_str` reads text first (so it composes `ReadError`).

### 2.6 The error model

`PredError` is typed; it composes the numeric-layer errors `=` can hit:

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
    Lower(#[from] LowerError),   // a numeric operand of `=` failed to lower
    #[error(transparent)]
    Eval(#[from] EvalError),     // a numeric operand had an unbound variable
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CheckError {
    #[error(transparent)] Read(#[from] ReadError),
    #[error(transparent)] Pred(#[from] PredError),
}
```

No panics. Every misuse — wrong kind in a position, bad arity, unbound state
variable — is a typed variant, surfaced at the earliest layer (R-0004 AC6).

## 3. Code outline

```rust
// eval_pred.rs
pub fn eval_pred(s: &Sexpr, env: &Env) -> Result<bool, PredError> {
    match s {
        Sexpr::Sym(t) if t == "true"  => Ok(true),
        Sexpr::Sym(t) if t == "false" => Ok(false),
        Sexpr::List(items) => eval_form(items, env),
        other => Err(PredError::ExpectedBool { found: describe(other) }),
    }
}

fn eval_form(items: &[Sexpr], env: &Env) -> Result<bool, PredError> {
    let Some((Sexpr::Sym(head), args)) = items.split_first() else {
        return Err(PredError::ExpectedBool { found: "non-form list".into() });
    };
    match head.as_str() {
        "pred" => match args { [e] => eval_pred(e, env), _ => arity("pred", 1, args) },
        "not"  => match args { [p] => Ok(!eval_pred(p, env)?), _ => arity("not", 1, args) },
        "="    => match args {
            [a, b] => Ok(eval_num(a, env)? == eval_num(b, env)?),
            _ => arity("=", 2, args),
        },
        "and"  => { for p in args { if !eval_pred(p, env)? { return Ok(false); } } Ok(true) }
        "or"   => { for p in args { if  eval_pred(p, env)? { return Ok(true);  } } Ok(false) }
        _ => Err(PredError::ExpectedBool { found: format!("form `{head}`") }),
    }
}

/// Evaluate a *numeric* operand: it must not be a boolean form.
fn eval_num(s: &Sexpr, env: &Env) -> Result<Value, PredError> {
    if is_boolean_form(s) {
        return Err(PredError::ExpectedNumber { found: describe(s) });
    }
    let eml = ufl_syntax::lower(s)?;          // LowerError → PredError::Lower
    Ok(ufl_core::eval(&eml, env)?)            // EvalError  → PredError::Eval
}
```

(`and`/`or` empty-arg cases — `(and)` = `true`, `(or)` = `false` — are the
standard identities; confirmed in tests.)

## 4. Non-goals

- Sequencing (`;`), parallel (`∃`), recursion/fixpoints, the orchestrator.
- Approximate/tolerance equality; ordering (`<`, `≤`); quantifiers.
- A heterogeneous runtime `Value` enum (explicitly avoided — §2.1).
- Booleans as first-class numeric operands (they are a separate mode).

## 5. Open questions

- **`(pred E)` transparency.** For the checker, `(pred E)` evaluates as `E`.
  Confirm that giving the atom a no-op-at-eval identity now (it gains meaning
  with sequencing later) is the right minimal move, vs omitting `pred` until it
  does something. Lean: keep it — it is the named atom R-0004 introduces.
- **`describe`/`found` strings.** The error payloads are human strings; confirm
  that is acceptable for `PredError` (vs a structured kind enum). Lean: strings
  are fine for a diagnostic; the *variant* carries the type information.

## 6. Acceptance criteria

- [ ] **AC1** — `eval_pred` yields `bool`; `true`/`false` evaluate; a number or a
  numeric form in boolean position is `PredError::ExpectedBool`; a boolean form
  as a `=` operand is `PredError::ExpectedNumber` — no coercion, no panic.
- [ ] **AC2** — `(= a b)` evaluates operands numerically (via the reused
  evaluator) and returns exact IEEE equality of the two `Value`s.
- [ ] **AC3** — `(and …)`, `(or …)`, `(not p)` give the standard truth tables,
  short-circuit, with `(and)`=`true` / `(or)`=`false`; wrong arity for `not` is
  `PredError::Arity`.
- [ ] **AC4** — `check(pred, pre, post)` binds pre vars by name and post vars
  primed, then evaluates; an unbound state variable surfaces as
  `PredError::Eval(UnboundVariable)`.
- [ ] **AC5** — `⟦ x' = (eml x 1) ⟧` (`(pred (= x' (eml x 1)))`) checks `true`
  when `post[x] = e^{pre[x]}` and `false` for a deliberately wrong post-state,
  over several `x`.
- [ ] **AC6** — every failure is a typed `PredError` / `CheckError` variant,
  never a panic, at the earliest detecting layer.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-02 | **Two typed evaluators, no god-enum.** Numeric → `Complex` via the reused `ufl_core::eval`; boolean → Rust `bool` via new `eval_pred`; `=` bridges. | The synthesis answer: stay typed, reuse the verified core, never pollute `Value`. Booleans are the result of predicate evaluation, not a numeric value kind. |
| 2026-06-02 | New crate `ufl-predicate` (→ `ufl-syntax`, `ufl-core`). | Predicate logic is a bounded responsibility; deps point inward. |
| 2026-06-02 | Pre/post-state is an **env convention** — post vars bound under `name'`; the numeric evaluator is unchanged. | Minimal: `x` and `x'` are just symbols; no change to `ufl-core`. |
| 2026-06-02 | Exact `=` realized as IEEE `Complex` equality (`==`), not literal bit-equality. | `=` means equal-as-numbers; IEEE gives the standard `NaN ≠ NaN`, `±0` equal. Honours R-0004 "exact, no tolerance"; bit-equality's `NaN == NaN` / `+0 ≠ −0` are wrong for `=`. |
| 2026-06-02 | `(pred E)` is transparent at check time (evaluates `E`). | It is the named `⟦P⟧` atom R-0004 introduces; it gains operational meaning with sequencing later, and marking predicates as first-class now costs nothing. |

## Changelog

- 2026-06-02 — created (Draft).
