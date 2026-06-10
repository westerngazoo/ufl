# SPEC-0007 — Tensor-Equality Predicate (the Hehner-discharge bridge)

- **Status:** Draft
- **Realizes:** R-0007 (Option A — the typed discharge abstraction)
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-08
- **Depends on:** R-0006 (`ufl-tensor`), R-0004 (`ufl-predicate`)
- **Crate(s):** `ufl-predicate` (gains the trait); `ufl-discovery` (new — the
  tensor instance + tests)

## 1. Motivation

SPEC-0007 realizes [R-0007](../requirements/0007-tensor-predicate.md) by
**Option A**: a small, typed *predicate-discharge* abstraction, so the matmul
predicate `P_n,R` and R-0004's scalar Hehner check are both **predicates** under
one concept — *a decidable property of a candidate state*. The discovery engine
(R-0008) discharges the tensor predicate through this abstraction, so the
verifier **is** the Hehner discharge (closing FINDINGS C1) rather than a bespoke
call to `ufl_tensor::is_valid`.

## 2. Design

### 2.1 The `Predicate` trait (in `ufl-predicate`)

```rust
/// A predicate is a *decidable property of a candidate state* — Hehner's
/// notion, generalized over the domain. Discharging it is the check (decidable);
/// searching for a satisfying candidate (R-0008) and selecting a substrate (the
/// future orchestrator) are separate.
pub trait Predicate {
    type Candidate: ?Sized;
    type Error;
    /// Decide whether `candidate` satisfies this predicate. Total: a malformed
    /// candidate is a typed `Error`, never a panic.
    fn discharge(&self, candidate: &Self::Candidate) -> Result<bool, Self::Error>;
}
```

The trait is deliberately tiny. Its job is to **name the shared concept** and
give a future generic consumer (the orchestrator: "discharge any predicate to
decide if a substrate satisfies it") one interface. Today's consumers are the
two instances below; the trait is the seam the orchestrator plugs into.

> **Honesty note (for the review).** Right now only the tensor instance is
> discharged in a loop (R-0008); the scalar instance implements the trait but is
> still used via its own `check`. The trait's *generic* consumer is future
> (the orchestrator). The two instances are real; whether that justifies the
> trait now — versus deferring it until the orchestrator exists and shipping
> R-0007 as just the tensor predicate framed as a Hehner discharge — is the
> explicit question for the architect/hater. SPEC commits to the trait but will
> thin to "tensor predicate + documented concept, trait deferred" if the review
> finds it unearned.

### 2.2 The scalar instance (R-0004 adapted)

R-0004's checker is wrapped as a `Predicate` without changing its behaviour —
the existing `check`/`eval_pred` and all R-0004 tests stay:

```rust
/// A scalar Hehner predicate: an Sexpr over a pre/post state.
pub struct ScalarPredicate { src: Sexpr }   // or holds the parsed predicate

impl Predicate for ScalarPredicate {
    type Candidate = Env;        // the combined pre/post environment
    type Error = PredError;
    fn discharge(&self, env: &Env) -> Result<bool, PredError> { eval_pred(&self.src, env) }
}
```

### 2.3 The tensor instance (in the new `ufl-discovery` crate)

To keep `ufl-tensor` a **pure leaf** (no dependency on `ufl-predicate`/syntax/
core) and `ufl-predicate` free of any tensor dependency, the instance lives in a
*bridge* crate that depends on both. That crate is **`ufl-discovery`** — created
here with just the predicate bridge; R-0008 grows the GA search into it.

```rust
// ufl-discovery: depends on ufl-predicate (trait) + ufl-tensor (domain)
use ufl_predicate::Predicate;
use ufl_tensor::{Scheme, SchemeError, scheme_error};

/// P_{n,R}: a scheme satisfies it iff it reconstructs T_n exactly at rank R.
pub struct TensorEq { pub n: usize, pub rank: usize }

impl Predicate for TensorEq {
    type Candidate = Scheme;
    type Error = SchemeError;
    /// reconstruct(scheme) == T_n  ∧  rank(scheme) == R.
    fn discharge(&self, scheme: &Scheme) -> Result<bool, SchemeError> {
        Ok(scheme.rank() == self.rank && scheme_error(scheme, self.n)? == 0)
    }
}
```

`discharge` is exactly `ufl_tensor::is_valid`'s decision, now expressed as the
predicate's discharge — and it surfaces a dim/`n` mismatch as the typed
`SchemeError::DimMismatch` (AC5), never a panic or a silent `false`.

### 2.4 Dependency direction

```
ufl-discovery ──▶ ufl-predicate ──▶ ufl-syntax ──▶ ufl-core
      └─────────▶ ufl-tensor (pure leaf, unchanged)
```

`ufl-tensor` stays a pure leaf; `ufl-predicate` gains only the trait (no tensor
dep); the bridge (`ufl-discovery`) is the single place the two domains meet.
Inward-pointing, no cycles. The architect confirms this is the right seam (vs. a
trait-only `ufl-predicate-core` crate, the alternative).

## 3. Code outline

The `Predicate` trait (§2.1), `ScalarPredicate` adapter (§2.2), and the
`ufl-discovery` crate with `TensorEq` (§2.3). `ufl-discovery/src/lib.rs`:

```rust
#![forbid(unsafe_code)]
pub use predicate::TensorEq;
mod predicate;
```

## 4. Non-goals

- The GA search / engine (R-0008) — `ufl-discovery` gains only the predicate
  here.
- Tensor values/forms in the s-expr language (Option B) — deferred; tensor
  predicates are not s-expr-written here.
- The orchestrator (the generic `Predicate` consumer) — future.

## 5. Open questions

- **Is the trait earned now (§2.1 honesty note)?** Architect/hater decide; thin
  to "tensor predicate only, trait deferred" if not.
- **Crate seam** — `ufl-discovery` as the bridge (chosen) vs. a trait-only
  `ufl-predicate-core`. Architect confirms.

## 6. Acceptance criteria

- [ ] **AC1** — `ufl-predicate` exposes `Predicate` with `discharge(&Candidate)
  -> Result<bool, Error>`; total, typed error, no panic.
- [ ] **AC2** — `ScalarPredicate` implements `Predicate` and agrees with R-0004's
  `check`/`eval_pred` (existing R-0004 tests unchanged and green).
- [ ] **AC3** — `TensorEq { n, rank }.discharge(scheme)` is `true` iff
  `reconstruct(scheme) == T_n` and `rank(scheme) == rank` (equals R-0006
  `is_valid`).
- [ ] **AC4 — Strassen through the predicate (keystone).**
  `TensorEq { n: 2, rank: 7 }.discharge(strassen) == Ok(true)`; on a broken
  scheme, `Ok(false)`.
- [ ] **AC5** — a dim/`n` mismatch (or empty scheme vs `n ≥ 1`) discharges to
  `Err(SchemeError::DimMismatch)`, never a panic or silent `false`.
- [ ] **AC6** — discharging `TensorEq` over many candidates in a tight loop is
  allocation-frugal enough for a million-candidate search (a loop test runs a
  large batch within a generous time bound, no per-call surprise).

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-08 | Realize R-0007 via **Option A** (typed `Predicate` discharge trait), per the owner's delegated call. | The GA discharges millions of candidates → a typed call, not s-expr eval; two real instances; no heterogeneous-value expansion. |
| 2026-06-08 | `Predicate` trait in `ufl-predicate`; tensor instance in a new **`ufl-discovery`** bridge crate; `ufl-tensor` stays a pure leaf. | Keeps deps inward and `ufl-tensor` dependency-free; the bridge is the one place predicate × tensor meet; `ufl-discovery` is then grown by R-0008. |
| 2026-06-08 | Recorded an explicit **"is the trait earned"** open question for the three-lens. | No premature abstraction (CLAUDE.md §2): the trait's generic consumer is the future orchestrator; the review decides keep-vs-thin. |

## Changelog

- 2026-06-08 — created (Draft).
