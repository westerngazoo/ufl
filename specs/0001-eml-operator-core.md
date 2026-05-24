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
**single complex logarithm** used inside every node, isolated in its own module
so the choice is visible and swappable.

R-0001's AC4 and Q-AC4 required a documented branch convention. **Resolution:**
`ln_eml(w) = Complex::ln(w)` — the standard principal branch, `Im ∈ (-τ/2, τ/2]`
(equivalently the conventional `(-π, π]`; UFL uses `τ` per
[`docs/conventions.md`](../docs/conventions.md)). **No correction term.**

This is not arbitrary; it is verified by experiment
([`experiments/q-ac4-branch.py`](../experiments/q-ac4-branch.py)). The textbook
`τi` discrepancy AllEle §4.1 predicts for the `ln(x)` identity on negative real
`x` does *not* appear in IEEE-754 `f64`. The chain's outer `ln` consumes a value
produced by `exp(e − iτ/2)`, and in `f64` that result's imaginary part is not
zero — `sin(-π) ≈ -1.22e-16` — so the value lands slightly *below* the negative
real axis. Principal `ln` of a point just below the cut returns `Im ≈ -τ/2`
(not `+τ/2`), which is exactly the value the chain needs to recover the true
principal `ln(x)`. The branch self-corrects via the floating-point
representation of `sin(τ/2)`. Over the §6 input sample the maximum observed
discrepancy is **≤ 1 ulp (≈ 1.11e-16)**.

The resolution is *specific to* IEEE-754 floating-point arithmetic with
`sin(τ/2) ≠ 0`. With arbitrary-precision arithmetic where `sin(τ/2)` is *exact*
zero, the self-correction vanishes and the textbook `τi` discrepancy returns; a
real correction term must then be re-derived. This dependency is made explicit
as **AC6** below — its tripwire (a unit test that fails if `sin(τ/2) == 0`)
forces Q-AC4 to be re-opened deliberately rather than silently broken by a
future arithmetic backend change.

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
/// Complex logarithm used inside every `eml` node.
///
/// For `Value = Complex<f64>` this is the standard principal branch — no
/// correction term. The textbook τi discrepancy on the negative real axis is
/// absorbed by the floating-point imprecision of `sin(τ/2)` (≈ -1.22e-16);
/// see SPEC-0001 §2.4 and `experiments/q-ac4-branch.py`.
///
/// Isolated here so that, if the value type ever changes to arithmetic where
/// `sin(τ/2) = 0` exactly, this function is the single point of change.
/// The AC6 invariant test (`sin(τ/2) != 0`) is the tripwire.
pub(crate) fn ln_eml(w: Value) -> Value { w.ln() }

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

*None.* Q-AC4 is resolved in §2.4 and recorded in the decision log; AC5
tolerance is fixed at `1e-14` (§6 AC5).

## 6. Acceptance criteria

Each row maps a SPEC-0001 deliverable to an R-0001 acceptance criterion; the
`qa` agent's tests for R-0001 verify them.

- [ ] **AC1** — the `Eml` enum admits exactly `S → 1 | <var> | eml(S,S)`;
  enforced structurally by the type, confirmed by a constructor doc test.
- [ ] **AC2** — `eval` returns a `Value` for any closed tree, and for any
  variable-bearing tree given a complete `Env`.
- [ ] **AC3** — trees evaluating `ln 0` / `exp(-∞)` and producing signed
  zeros/infinities yield `inf`/`nan` `Value`s with no panic.
- [ ] **AC4** — `ln_eml` is `Complex::ln` (the standard principal branch); the
  rationale, the f64 self-correction, and its limit are documented in §2.4.
  Derived `i`, `τ`, and `ln x` for `x<0` carry the principal-branch sign via
  that self-correction.
- [ ] **AC5** — `e = eml(1,1)`, `exp(x) = eml(x,1)`, and
  `ln(x) = eml(1,eml(eml(1,x),1))` match reference values within a relative
  tolerance of `1e-14` over the input sample
  `{-3.0, -1.0, -0.5, 0.5, 1.0, 2.5}`, including negative `x`. (The
  `experiments/q-ac4-branch.py` baseline shows ≤ 1 ulp ≈ 1.11e-16 discrepancy,
  so `1e-14` is two orders of magnitude generous.)
- [ ] **AC6 — `sin(τ/2) ≠ 0` invariant.** A unit test asserts that the
  runtime's `f64::sin(std::f64::consts::PI)` is non-zero. Its purpose is to
  *fail loudly* if a future arithmetic backend (arbitrary-precision, symbolic,
  or an exotic `sin` implementation) makes `sin(τ/2)` exactly zero — in that
  case the AC4 self-correction silently breaks and Q-AC4 must be re-opened.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-18 | New crate `ufl-core` is UFL's numeric/GA core; SPEC-0001 populates its `eml` representation and evaluator. | One crate per bounded responsibility; the numeric substrate is the natural first crate. |
| 2026-05-18 | `Eml` is a recursive `Box`-ed enum with a named-field node variant (`exp_arg` / `log_arg`). | Mirrors the grammar exactly (AC1 holds structurally); named fields make the asymmetric operator self-documenting. |
| 2026-05-18 | Values are `Complex<f64>` via `num-complex`. | Standard crate, no hand-rolling; `f64` supplies the IEEE inf/nan behaviour AC3 needs. |
| 2026-05-18 | Operator-level branch correction (`ln_eml` inside every node), over AllEle's downstream `i`-sign patching. | Every derived quantity is principal-correct by construction; no caller patches signs. |
| 2026-05-18 | Evaluator is infallible on numeric edge cases; `EvalError` has the single variant `UnboundVariable`. | AC3 requires no traps on numeric edges; the only genuine failure is an unbound variable. |
| 2026-05-24 | **Q-AC4 resolved.** `ln_eml = Complex::ln` (principal branch), no correction term. | Verified by `experiments/q-ac4-branch.py` over `{-3, -1, -0.5, 0.5, 1, 2.5}`; max discrepancy ≈ 1.11e-16 (1 ulp). In `f64` the chain's outer `ln` consumes `exp(... ± iτ/2)` which lands ~1.22e-16 *off* the negative real axis (because `sin(τ/2) ≠ 0`), so principal `ln` returns the value the chain needs. The dependency is captured by **AC6**. |
| 2026-05-24 | AC5 tolerance fixed at relative `1e-14` over the §6 input sample. | Two orders of magnitude generous vs the observed 1-ulp discrepancy; survives expected drift in `num-complex`'s `exp` / `ln` rounding. |
| 2026-05-24 | New AC6 codifying the `sin(τ/2) ≠ 0` invariant as a tripwire test. | The AC4 resolution is contingent on this floating-point property; an explicit failing test is the only safe way to force Q-AC4 to be re-opened if the property ever stops holding. |

## Changelog

- 2026-05-18 — created (Draft).
- 2026-05-19 — `π` replaced with `τ` in §2.4, §5, and the AC4 mapping per [`docs/conventions.md`](../docs/conventions.md) (notational; the `(-τ/2, τ/2]` interval equals the conventional principal-branch `(-π, π]`).
- 2026-05-24 — Q-AC4 resolved (§2.4 rewritten with the resolution + f64 caveat); AC5 tolerance fixed at `1e-14`; AC6 added; §5 closed; experiment script saved as `experiments/q-ac4-branch.py`.
