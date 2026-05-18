# SPEC-0001 — EML Operator Core

- **Status:** Draft
- **Realizes:** R-0001
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-05-18
- **Depends on:** none
- **Crate(s):** `ufl-core`

## 1. Motivation

SPEC-0001 realizes [R-0001](../requirements/0001-eml-operator-core.md): it gives
the EML operator `eml(x,y) = exp(x) − ln(y)` and the literal `1` a concrete Rust
representation plus a reference evaluator. This is the first executable artifact
of UFL — the numeric substrate every later layer (geometric algebra, predicates,
substrates) is built over.

The design goal is *correctness, legibly stated* — not speed. R-0001 scopes this
to a reference evaluator; stable and substrate-compiled evaluation are later
requirements.

## 2. Design

### 2.1 Crate

A new workspace crate **`ufl-core`** holds UFL's numeric (and later
geometric-algebra) core. SPEC-0001 populates it with the `eml` representation
and evaluator. It is registered in the workspace `members` list.

Dependencies: `num-complex` (complex arithmetic), `thiserror` (the error type).

### 2.2 The EML tree

An EML expression is a recursive enum mirroring the grammar
`S → 1 | <var> | eml(S, S)` exactly — by construction nothing else is
representable, satisfying AC1 structurally:

```rust
pub enum Eml {
    One,
    Var(String),
    Node { exp_arg: Box<Eml>, log_arg: Box<Eml> },
}
```

The node variant uses **named fields** because `eml` is asymmetric: `exp_arg`
flows through `exp`, `log_arg` flows through `ln`. Positional fields would leave
the asymmetry implicit. `Box` gives the recursion a heap indirection.

Ergonomic constructors (`Eml::one()`, `Eml::var(name)`, `Eml::node(a, b)`) hide
the `Box`. The type derives `Debug, Clone, PartialEq`.

### 2.3 Values are complex

```rust
pub type Value = num_complex::Complex<f64>;
```

Per R-0001's constraints the substrate is ℂ over IEEE-754 `f64`. We reuse
`num-complex` rather than hand-rolling: it is the standard crate, and `f64`
already carries the `inf` / `-inf` / signed-zero / `nan` behaviour AC3 requires.

### 2.4 The `eml` operation and the branch-controlled logarithm

The value of a node is

```
value(Node { exp_arg, log_arg }) = exp(value(exp_arg)) − ln_eml(value(log_arg))
```

where `exp` is the complex exponential (`Complex::exp`) and `ln_eml` is a
**single branch-controlled complex logarithm** used inside every node.

R-0001's AC4 / open question Q-AC4 require a documented branch convention. We
adopt **operator-level correction**: the branch is fixed once, inside `ln_eml`,
so every derived quantity (`i`, `π`, `ln x` for `x<0`) is principal-correct *by
construction* — no caller ever patches a sign. This is chosen over AllEle §4.1's
alternative of correcting the `i` sign downstream, which leaks the concern into
every consumer.

The baseline is the principal branch (`Complex::ln`, `Im ∈ (-π, π]`). AllEle
§4.1 shows the derived `ln z` identity routes through an `e^e/z` term, so for
real `z < 0` the principal branch leaves a `2πi` discrepancy. `ln_eml` therefore
carries a documented correction term. **The exact correction is this spec's open
question (§5)** — it must be fixed before status moves to `Accepted`.

### 2.5 The evaluator

```rust
pub fn eval(expr: &Eml, env: &Env) -> Result<Value, EvalError>
```

A recursive post-order walk:

- `One` → `Value::new(1.0, 0.0)`
- `Var(name)` → the binding from `env`, or `Err(EvalError::UnboundVariable)`
- `Node { .. }` → `eval` both children, then apply §2.4.

Evaluation is **infallible on numeric edge cases**: `Complex<f64>` arithmetic,
`exp`, and `ln` never panic — `inf`/`nan` propagate as ordinary `Value`s (AC3).
The *only* failure is a variable with no binding, so `EvalError` has exactly one
variant:

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EvalError {
    #[error("unbound variable: {0}")]
    UnboundVariable(String),
}
```

Variable bindings are held in a small `Env`:

```rust
pub struct Env { bindings: HashMap<String, Value> }

impl Env {
    pub fn new() -> Self;
    pub fn bind(&mut self, name: impl Into<String>, value: Value) -> &mut Self;
    pub fn get(&self, name: &str) -> Option<Value>;
}
```

### 2.6 Module layout

```
crates/ufl-core/
├── Cargo.toml
└── src/
    ├── lib.rs    — crate root; re-exports Eml, eval, Env, EvalError, Value
    ├── eml.rs    — the Eml type and constructors
    ├── eval.rs   — Env, EvalError, eval
    └── log.rs    — ln_eml, the branch-controlled logarithm (isolated:
                    it carries the spec's one delicate decision)
```

## 3. Code outline

Representative — refined with the owner before implementation.

```rust
// eml.rs
pub enum Eml {
    One,
    Var(String),
    Node { exp_arg: Box<Eml>, log_arg: Box<Eml> },
}

impl Eml {
    pub fn one() -> Eml { Eml::One }
    pub fn var(name: impl Into<String>) -> Eml { Eml::Var(name.into()) }
    pub fn node(exp_arg: Eml, log_arg: Eml) -> Eml {
        Eml::Node { exp_arg: Box::new(exp_arg), log_arg: Box::new(log_arg) }
    }
}

// log.rs
/// Branch-controlled complex logarithm used inside every `eml` node.
/// Convention: <documented once Q-AC4 in §5 is resolved>.
pub(crate) fn ln_eml(w: Value) -> Value { /* principal branch + correction */ }

// eval.rs
pub fn eval(expr: &Eml, env: &Env) -> Result<Value, EvalError> {
    match expr {
        Eml::One => Ok(Value::new(1.0, 0.0)),
        Eml::Var(name) => env.get(name)
            .ok_or_else(|| EvalError::UnboundVariable(name.clone())),
        Eml::Node { exp_arg, log_arg } => {
            let x = eval(exp_arg, env)?;
            let y = eval(log_arg, env)?;
            Ok(x.exp() - ln_eml(y))
        }
    }
}
```

## 4. Non-goals

- The formula → EML compiler (a later requirement).
- Numerically stable / optimized evaluation; substrate compilation.
- Deep-tree stack safety — the recursive evaluator is adequate for R-0001's
  shallow identity trees (max depth 3); iterative evaluation is deferred.
- Geometric algebra, predicates, surface-syntax parsing, simplification.

## 5. Open questions

- **Q-AC4 — exact `ln_eml` branch rule.** §2.4 commits to the operator-level
  approach; the precise correction term is unresolved. Resolution plan: a
  numeric experiment in design review — evaluate the `ln` identity tree for a
  sample of real `z < 0`, measure the discrepancy from the principal `ln z`,
  and choose the `ln_eml` correction that makes the derived `ln`, `i`, and `π`
  principal-correct. Status stays `Draft` until this is fixed and documented.
- **AC5 tolerance and sample.** Proposed: relative tolerance `1e-12`, checked
  over real inputs `{-3.0, -1.0, -0.5, 0.5, 1.0, 2.5}`. To be confirmed (and, if
  needed, tightened) with the `qa` agent against the depth-3 `ln` identity.

## 6. Acceptance criteria

Each row maps a SPEC-0001 deliverable to an R-0001 acceptance criterion; the
`qa` agent's tests for R-0001 verify them.

- [ ] **AC1** — the `Eml` enum admits exactly `S → 1 | <var> | eml(S,S)`;
  enforced structurally by the type, confirmed by a constructor doc test.
- [ ] **AC2** — `eval` returns a `Value` for any closed tree, and for any
  variable-bearing tree given a complete `Env`.
- [ ] **AC3** — trees evaluating `ln 0` / `exp(-∞)` and producing signed
  zeros/infinities yield `inf`/`nan` `Value`s with no panic.
- [ ] **AC4** — `ln_eml`'s convention is documented; derived `i`, `π`, and
  `ln x` for `x<0` carry the principal-branch sign.
- [ ] **AC5** — `e = eml(1,1)`, `exp(x) = eml(x,1)`, and
  `ln(x) = eml(1,eml(eml(1,x),1))` match reference values within tolerance over
  the §5 input sample, including negative `x`.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-18 | New crate `ufl-core` is UFL's numeric/GA core; SPEC-0001 populates its `eml` representation and evaluator. | One crate per bounded responsibility; the numeric substrate is the natural first crate. |
| 2026-05-18 | `Eml` is a recursive `Box`-ed enum with a named-field node variant (`exp_arg` / `log_arg`). | Mirrors the grammar exactly (AC1 holds structurally); named fields make the asymmetric operator self-documenting. |
| 2026-05-18 | Values are `Complex<f64>` via `num-complex`. | Standard crate, no hand-rolling; `f64` supplies the IEEE inf/nan behaviour AC3 needs. |
| 2026-05-18 | Operator-level branch correction (`ln_eml` inside every node), over AllEle's downstream `i`-sign patching. | Every derived quantity is principal-correct by construction; no caller patches signs. |
| 2026-05-18 | Evaluator is infallible on numeric edge cases; `EvalError` has the single variant `UnboundVariable`. | AC3 requires no traps on numeric edges; the only genuine failure is an unbound variable. |

## Changelog

- 2026-05-18 — created (Draft).
