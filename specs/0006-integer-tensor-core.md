# SPEC-0006 — Exact Integer-Tensor Core (`ufl-tensor`)

- **Status:** Draft
- **Realizes:** R-0006
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-04
- **Depends on:** none (pure integer arithmetic — *not* `ufl-core`)
- **Crate(s):** `ufl-tensor` (new)

## 1. Motivation

SPEC-0006 realizes [R-0006](../requirements/0006-integer-tensor-core.md): the
exact integer-tensor core the matmul-decomposition discovery engine verifies
against. It builds the target tensor `T_n`, represents a scheme, reconstructs
it, and decides exact equality — all in integer arithmetic, no `Complex<f64>`
EML dependency (FINDINGS C3). Its keystone is the **Strassen Phase-0 gate**: the
canonical 7-term 2×2 scheme reconstructs `T_2` with error 0 (verified during
drafting and independently by the three-lens review).

## 2. Design

### 2.1 Conventions (fixed here; cited by every later discovery requirement)

- `n` is the matrix dimension; `d = n²` is the tensor side / vector length.
- A matrix entry `M[i][j]` flattens **row-major** to `i·n + j` (`i,j ∈ 0..n`).
- Matrix product: `C[i][k] = Σ_j A[i][j]·B[j][k]`.
- **Target tensor** `T_n`, shape `(d, d, d)`:
  `T_n[p,q,r] = 1` iff `∃ i,j,k ∈ 0..n` with `p = i·n+j`, `q = j·n+k`,
  `r = i·n+k`; else `0`.
  The map `(i,j,k) ↦ (i·n+j, j·n+k, i·n+k)` is **injective** — it is recoverable
  by `i = p÷n`, `j = q÷n`, `k = q mod n` (and `r = i·n+k` is then forced) — so
  no two `(i,j,k)` collide and **every entry of `T_n` is exactly 0 or 1**. This
  is what makes `error == 0` mean exact equality.
- **Scheme reconstruction:** for `scheme = [(u_t, v_t, w_t)]`,
  `reconstruct[p,q,r] = Σ_t u_t[p]·v_t[q]·w_t[r]` (`i64`).
- A scheme is **valid at rank `R`** iff `reconstruct == T_n` (`error == 0`) and
  `scheme.len() == R`.

### 2.2 Crate

New crate **`ufl-tensor`**, no dependency on `ufl-core`/`-syntax`/`-predicate`
(the pure exact-arithmetic layer those later wrap). `#![forbid(unsafe_code)]`,
registered in the workspace; sole dependency `thiserror`.

```
crates/ufl-tensor/
├── Cargo.toml
└── src/
    ├── lib.rs          # re-exports
    ├── tensor.rs       # Tensor (dense i64), target(n), error
    ├── scheme.rs       # Triple, Scheme, SchemeError — validated, length-consistent
    └── reconstruct.rs  # reconstruct, scheme_error, is_valid
└── tests/
    └── r_0006_acceptance.rs   # incl. the Strassen + naive fixtures
```

### 2.3 The tensor — total accessor (no panic)

Dense row-major `i64` storage, length `d³`, index `(p,q,r) → (p·d + q)·d + r`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tensor { dim: usize, data: Vec<i64> } // data.len() == dim³

impl Tensor {
    pub fn zeros(dim: usize) -> Self;
    pub fn dim(&self) -> usize;
    /// Total accessor — `None` if any index ≥ dim (matches `Env::get`'s
    /// Option convention; no panic, CLAUDE.md §6).
    pub fn get(&self, p: usize, q: usize, r: usize) -> Option<i64>;
    /// Internal, in-range by construction (callers loop `0..dim`).
    fn add_at(&mut self, p: usize, q: usize, r: usize, v: i64);
}

/// The matmul target tensor T_n (§2.1).
pub fn target(n: usize) -> Tensor;

/// Σ over entries of (a − b)²; `None` if the dims differ (total, no panic);
/// `Some(0)` iff equal. See §2.5 for the i64 envelope.
pub fn error(a: &Tensor, b: &Tensor) -> Option<i64>;
```

There is **no public panicking accessor** — `get` and `error` are total,
returning `Option`, matching `ufl_core::Env::get`. Internal indexing in
`target`/`reconstruct` loops over `0..dim` and is in-range by construction.

### 2.4 The scheme genotype — one consistent dim, validated

The `d`/`n` desync hole is closed by construction: **a `Triple` validates its
own internal consistency, and a `Scheme` enforces that all triples share one
length.** There is no free `d` parameter to desynchronize.

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemeError {
    #[error("u/v/w lengths differ: {u}, {v}, {w}")]
    Ragged { u: usize, v: usize, w: usize },
    #[error("empty vector: a triple's vectors must be non-empty")]
    Empty,
    #[error("coefficient {0} outside {{-1, 0, +1}}")]
    Coefficient(i8),
    #[error("triple length {got} ≠ scheme length {expected}")]
    Mismatch { expected: usize, got: usize },
    #[error("scheme dim {got} ≠ n² = {expected} for n = {n}")]
    DimMismatch { n: usize, expected: usize, got: usize },
}

/// One scalar multiplication: u, v, w equal-length {-1,0,+1} vectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple { u: Vec<i8>, v: Vec<i8>, w: Vec<i8> }

impl Triple {
    /// Validated: u/v/w equal, non-empty length, all entries in {-1,0,+1}.
    /// The triple's `len()` is that shared length (= d for a real scheme).
    pub fn new(u: Vec<i8>, v: Vec<i8>, w: Vec<i8>) -> Result<Self, SchemeError>;
    pub fn len(&self) -> usize;            // u.len() == v.len() == w.len()
    pub fn is_empty(&self) -> bool;        // false (lengths are non-empty)
}

/// An ordered list of triples; invariant: all triples share one length.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scheme { triples: Vec<Triple> }

impl Scheme {
    pub fn new() -> Self;
    /// Append a triple; rejects one whose length differs from the scheme's
    /// existing length (Mismatch). Keeps the scheme length-consistent.
    pub fn push(&mut self, t: Triple) -> Result<&mut Self, SchemeError>;
    pub fn rank(&self) -> usize;           // triples.len()
    pub fn dim(&self) -> Option<usize>;    // shared triple length; None if empty
    pub fn triples(&self) -> &[Triple];
}
```

> **Scope note.** The mutable builder (`push`) and `Clone`/`PartialEq` exceed
> Phase 0's literal needs (build a fixed scheme, reconstruct, compare) — they
> are the affordances R-0008's GA loop will use. Nothing Phase 0 needs is
> missing; nothing generic-tensor-lib (contraction, transpose) is present.

### 2.5 Reconstruction, verification, and the i64 envelope

```rust
/// reconstruct[p,q,r] = Σ_t u_t[p]·v_t[q]·w_t[r] (§2.1). The tensor dim is the
/// scheme's own dim (no `n` parameter — the scheme defines its size, so a
/// dim/n desync is impossible). An empty scheme reconstructs to `zeros(0)`.
pub fn reconstruct(scheme: &Scheme) -> Tensor;

/// Build target(n), check the scheme's dim is n² (else DimMismatch), and
/// return the exact integer error. Total — never panics.
pub fn scheme_error(scheme: &Scheme, n: usize) -> Result<i64, SchemeError>;

/// Valid at rank R: exactly R triples AND exact reconstruction.
pub fn is_valid(scheme: &Scheme, n: usize, rank: usize) -> bool {
    scheme.rank() == rank && matches!(scheme_error(scheme, n), Ok(0))
}
```

**The i64 envelope (stated, not assumed).** The binding overflow surface is
`error`'s squaring: `error ≤ n⁶·(R+1)²`. This is below `i64::MAX ≈ 9.2·10¹⁸`
for, e.g., `n ≤ 100` and `R ≤ 10⁸` — astronomically beyond any
Strassen-class search (`R=7 → error ≤ 4096`). **Within this envelope the
arithmetic is exact; outside it, results are undefined (i64 wraps).** The
discovery work lives deep inside the envelope; `reconstruct` entries (bounded by
`R`) overflow even later, so `error` is the relevant bound.

The zero-skip in `reconstruct` (skip a factor of `0`) omits only provably-`+0`
(additive-identity) terms, so the result is **bit-identical** with or without
it — integers have no absorbing-then-NaN element, unlike SPEC-0002's geometric
product where the analogous skip was a real bug.

### 2.6 Fixtures (in tests)

- **Strassen 7-term 2×2** — the keystone (AC5). Triples (`u; v; w`, length 4,
  row-major `a11 a12 a21 a22`), pre-verified (error 0) under §2.1:

  | r | u | v | w |
  |---|---|---|---|
  | 1 | `[1,0,0,1]` | `[1,0,0,1]` | `[1,0,0,1]` |
  | 2 | `[0,0,1,1]` | `[1,0,0,0]` | `[0,0,1,-1]` |
  | 3 | `[1,0,0,0]` | `[0,1,0,-1]` | `[0,1,0,1]` |
  | 4 | `[0,0,0,1]` | `[-1,0,1,0]` | `[1,0,1,0]` |
  | 5 | `[1,1,0,0]` | `[0,0,0,1]` | `[-1,1,0,0]` |
  | 6 | `[-1,0,1,0]` | `[1,1,0,0]` | `[0,0,0,1]` |
  | 7 | `[0,1,0,-1]` | `[0,0,1,1]` | `[1,0,0,0]` |

- **Naive `R = n³`** — one triple per `(i,j,k)`: `u = e_{i·n+j}`,
  `v = e_{j·n+k}`, `w = e_{i·n+k}`. Verified n=2 (R=8), n=3 (R=27).

## 3. Code outline

```rust
// tensor.rs
pub fn target(n: usize) -> Tensor {
    let d = n * n;
    let mut t = Tensor::zeros(d);
    for i in 0..n { for j in 0..n { for k in 0..n {
        t.add_at(i * n + j, j * n + k, i * n + k, 1);
    }}}
    t
}
pub fn error(a: &Tensor, b: &Tensor) -> Option<i64> {
    if a.dim != b.dim { return None; }
    Some(a.data.iter().zip(&b.data).map(|(x, y)| { let d = x - y; d * d }).sum())
}

// scheme.rs
impl Scheme {
    pub fn push(&mut self, t: Triple) -> Result<&mut Self, SchemeError> {
        if let Some(d) = self.dim() {
            if t.len() != d { return Err(SchemeError::Mismatch { expected: d, got: t.len() }); }
        }
        self.triples.push(t);
        Ok(self)
    }
    pub fn dim(&self) -> Option<usize> { self.triples.first().map(Triple::len) }
}

// reconstruct.rs
pub fn reconstruct(scheme: &Scheme) -> Tensor {
    let d = scheme.dim().unwrap_or(0);
    let mut t = Tensor::zeros(d);
    for tr in scheme.triples() {
        for p in 0..d {
            if tr.u[p] == 0 { continue; }
            for q in 0..d {
                if tr.v[q] == 0 { continue; }
                for r in 0..d {
                    let prod = tr.u[p] as i64 * tr.v[q] as i64 * tr.w[r] as i64;
                    if prod != 0 { t.add_at(p, q, r, prod); }
                }
            }
        }
    }
    t
}
pub fn scheme_error(scheme: &Scheme, n: usize) -> Result<i64, SchemeError> {
    let expected = n * n;
    match scheme.dim() {
        Some(d) if d != expected =>
            Err(SchemeError::DimMismatch { n, expected, got: d }),
        _ => Ok(error(&reconstruct(scheme), &target(n))
                 .expect("dims equal by the check above")),
    }
}
```

(The lone `expect` is in a genuinely-unreachable branch — dims are equal by the
preceding check — with a justifying message, per CLAUDE.md §6. An empty scheme
with `n` such that `n² == 0` is impossible since `n ≥ 1`; an empty scheme with
`n ≥ 1` has `dim() == None`, falls to the `_` arm, reconstructs `zeros(0)` vs
`target(n)` of dim `n²` → `error` returns `None` → the `expect` *would* fire.
**Fix in the outline:** the `_` arm must also reject an empty scheme against
`n ≥ 1` as `DimMismatch { got: 0 }`. The implementation handles
`scheme.dim()` `None` by treating it as dim 0.)

## 4. Non-goals

- The GA search / discovery loop (R-0008); the tensor-equality predicate bridge
  (R-0007); EML/`Complex` reuse; `egg`; neural guidance; sparse storage;
  performance tuning; sizes beyond those tested.

## 5. Open questions

*None blocking.* The desync hole is closed by the §2.4 redesign; accessors are
total (§2.3); the i64 envelope is stated (§2.5); the Strassen fixture is pinned
and pre-verified (§2.6).

## 6. Acceptance criteria

- [ ] **AC1** — `target(n)` builds `T_n` per §2.1; verified for `n=2` against
  the known `T_2`; entries are 0/1 (the injectivity argument, §2.1).
- [ ] **AC2** — `Triple::new` accepts only equal-length non-empty `{-1,0,+1}`
  vectors (`Ragged`/`Empty`/`Coefficient` else); `Scheme::push` rejects a
  length-mismatched triple (`Mismatch`). No panic.
- [ ] **AC3** — `reconstruct(scheme)` computes `Σ_t u⊗v⊗w` exactly in `i64`,
  dim = the scheme's dim.
- [ ] **AC4** — `error` / `scheme_error` are exact `i64`, total (no panic): a
  dim/`n` mismatch is `DimMismatch` (incl. an empty scheme vs `n ≥ 1`); `0` iff
  equal; no floating point.
- [ ] **AC5 — Strassen gate (keystone).** The §2.6 Strassen scheme has
  `scheme_error(·, 2) == Ok(0)` and `is_valid(·, 2, 7)`.
- [ ] **AC6 — naive baseline.** Exact for n=2 (`is_valid(·, 2, 8)`) and n=3
  (`is_valid(·, 3, 27)`).

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-04 | New crate `ufl-tensor`, pure integer, no `ufl-core` dependency; dense row-major `Vec<i64>`. | Exact `{-1,0,+1}` work; wrong field in EML core (FINDINGS C3); simplest correct representation. |
| 2026-06-04 | Conventions (§2.1) fixed here incl. the **injectivity argument** (entries are 0/1). | One source of truth all later discovery requirements cite; the 0/1 fact makes `error==0` mean exact equality. |
| 2026-06-04 | Strassen fixture pre-verified (error 0) before speccing; zero-skip is a value-neutral integer speed-up. | AC5 keystone; no transcription risk; integers have no NaN hazard (unlike SPEC-0002). |
| 2026-06-04 | **Three-lens review applied.** (1) Closed the `d`/`n` desync panic: dropped the free `d` param; a `Triple` self-validates, a `Scheme` enforces one shared length, and `reconstruct` derives dim from the scheme — `scheme_error` validates dim vs `n` and returns `DimMismatch`, never panics. (2) `Tensor::get`/`error` are total `Option`-returning accessors (matching `Env::get`), no public panic. (3) Stated the i64 envelope (`error ≤ n⁶·(R+1)²`) instead of an unbounded "exactly safe" claim. (4) Added the injectivity argument. | architect REQUEST CHANGES + hater NEEDS WORK converged on these (both re-verified the keystone); the desync was a guaranteed panic on a plausible caller error in the verifier everything downstream trusts. |

## Changelog

- 2026-06-04 — created (Draft); Strassen + naive fixtures pre-verified.
- 2026-06-04 — three-lens review applied: desync hole closed (no free `d`,
  scheme owns its dim), total `Option` accessors, i64 envelope stated,
  injectivity argument added. Open questions closed.
