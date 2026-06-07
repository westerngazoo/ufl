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
it, and decides exact equality — all in integer arithmetic, with no dependency
on the `Complex<f64>` EML core (FINDINGS C3). Its keystone is the **Strassen
Phase-0 gate**: the canonical 7-term 2×2 scheme reconstructs `T_2` with error 0
(verified during drafting against the conventions below).

## 2. Design

### 2.1 Conventions (fixed here, cited by every later discovery requirement)

- `n` is the matrix dimension; `d = n²` is the tensor side / vector length.
- A matrix entry `M[i][j]` flattens **row-major** to index `i·n + j`
  (`i, j ∈ 0..n`).
- Matrix product: `C[i][k] = Σ_j A[i][j]·B[j][k]`.
- **Target tensor** `T_n`, shape `(d, d, d)`:
  `T_n[p, q, r] = 1` iff `∃ i,j,k ∈ 0..n` with `p = i·n+j`, `q = j·n+k`,
  `r = i·n+k`; else `0`.
- **Scheme reconstruction:** for `scheme = [(u_t, v_t, w_t)]`,
  `reconstruct[p, q, r] = Σ_t u_t[p]·v_t[q]·w_t[r]` (`i64`).
- A scheme is **valid at rank `R`** iff `reconstruct == T_n` (equivalently
  `error == 0`) and `scheme.len() == R`.

### 2.2 Crate

A new crate **`ufl-tensor`**, no dependency on `ufl-core` / `-syntax` /
`-predicate` — it is the pure exact-arithmetic layer those later wrap.
`#![forbid(unsafe_code)]`, registered in the workspace.

```
crates/ufl-tensor/
├── Cargo.toml          # only thiserror
└── src/
    ├── lib.rs          # re-exports
    ├── tensor.rs       # Tensor (dense i64), target(n), error
    ├── scheme.rs       # Triple, Scheme, validated construction
    └── reconstruct.rs  # reconstruct(scheme, n) -> Tensor
└── tests/
    └── r_0006_acceptance.rs   # incl. the Strassen + naive fixtures
```

### 2.3 The tensor

Dense row-major `i64` storage, length `d³`, index `(p,q,r) → (p·d + q)·d + r`.
Dense is exact and ample at the `n` in scope (n=2 → 64 entries, n=3 → 729).

```rust
/// A dense 3-index integer tensor of shape (dim, dim, dim), dim = n².
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tensor {
    dim: usize,
    data: Vec<i64>, // length dim³, index (p·dim + q)·dim + r
}

impl Tensor {
    pub fn zeros(dim: usize) -> Self;
    pub fn dim(&self) -> usize;
    pub fn get(&self, p: usize, q: usize, r: usize) -> i64;   // panics out of range (unreachable: indices are program-derived)
    fn add_at(&mut self, p: usize, q: usize, r: usize, v: i64);
}

/// The matmul target tensor T_n (§2.1).
pub fn target(n: usize) -> Tensor;

/// Σ over all entries of (a − b)²; 0 iff a == b. Requires equal dim.
pub fn error(a: &Tensor, b: &Tensor) -> i64;
```

### 2.4 The scheme genotype

```rust
/// One scalar multiplication: u, v, w each a length-d vector in {-1,0,+1}.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple {
    u: Vec<i8>,
    v: Vec<i8>,
    w: Vec<i8>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemeError {
    #[error("vector length {got} ≠ expected d = n² = {expected}")]
    Length { expected: usize, got: usize },
    #[error("coefficient {0} outside {{-1, 0, +1}}")]
    Coefficient(i8),
}

impl Triple {
    /// Validated constructor: each vector must have length `d` and entries in
    /// {-1,0,+1} (R-0006 AC2). Rejection is a typed `SchemeError`, never a panic.
    pub fn new(u: Vec<i8>, v: Vec<i8>, w: Vec<i8>, d: usize) -> Result<Self, SchemeError>;
}

/// A scheme is an ordered list of triples; its rank is its length.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scheme {
    triples: Vec<Triple>,
}

impl Scheme {
    pub fn new() -> Self;
    pub fn push(&mut self, t: Triple) -> &mut Self;
    pub fn rank(&self) -> usize;            // == triples.len()
    pub fn triples(&self) -> &[Triple];
}
```

### 2.5 Reconstruction and verification

```rust
/// reconstruct[p,q,r] = Σ_t u_t[p]·v_t[q]·w_t[r], exact i64 (§2.1).
pub fn reconstruct(scheme: &Scheme, n: usize) -> Tensor;

/// error(reconstruct(scheme,n), target(n)); 0 iff the scheme is exact.
pub fn scheme_error(scheme: &Scheme, n: usize) -> i64;

/// Valid at rank R: exact reconstruction AND exactly R triples.
pub fn is_valid(scheme: &Scheme, n: usize, rank: usize) -> bool {
    scheme.rank() == rank && scheme_error(scheme, n) == 0
}
```

Reconstruction skips no terms and uses `i64` throughout — a sum of `R`
products of three `{-1,0,+1}` factors is bounded well within `i64`, exactly.

### 2.6 Fixtures (in tests)

- **Strassen 7-term 2×2** — the keystone (AC5), verified during drafting to
  reconstruct `T_2` with error 0 under §2.1. The 7 triples (`u; v; w`, each
  length 4, row-major `a11 a12 a21 a22`):

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
  `v = e_{j·n+k}`, `w = e_{i·n+k}` (standard basis vectors). Verified for n=2
  (R=8) and n=3 (R=27).

## 3. Code outline

```rust
// tensor.rs
pub fn target(n: usize) -> Tensor {
    let d = n * n;
    let mut t = Tensor::zeros(d);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                t.add_at(i * n + j, j * n + k, i * n + k, 1);
            }
        }
    }
    t
}

pub fn error(a: &Tensor, b: &Tensor) -> i64 {
    a.data.iter().zip(&b.data).map(|(x, y)| { let d = x - y; d * d }).sum()
}

// reconstruct.rs
pub fn reconstruct(scheme: &Scheme, n: usize) -> Tensor {
    let d = n * n;
    let mut t = Tensor::zeros(d);
    for tr in scheme.triples() {
        for p in 0..d {
            if tr.u[p] == 0 { continue; }            // skip is value-neutral on integers (no NaN here)
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
```

(The zero-skip here is safe — unlike SPEC-0002's geometric product, integers
have no `0·∞ = NaN` hazard, so skipping zero factors is a pure speed win that
changes no result.)

## 4. Non-goals

- The **GA search** / discovery loop (R-0008).
- The **tensor-equality predicate** bridging into `ufl-predicate` (R-0007).
- Reuse of the EML/`Complex` core; `egg`; neural guidance; performance tuning;
  sparse storage; sizes beyond those tested.

## 5. Open questions

*None blocking.* Storage is dense `Vec<i64>` (§2.3); the Strassen fixture is
pinned and pre-verified (§2.6); conventions are fixed (§2.1).

## 6. Acceptance criteria

- [ ] **AC1** — `target(n)` builds `T_n` with the §2.1 entries; verified for
  `n=2` against the known `T_2` (and structurally for general `n`).
- [ ] **AC2** — `Triple::new` accepts only length-`d` vectors with entries in
  `{-1,0,+1}`; a wrong length → `SchemeError::Length`, a bad entry →
  `SchemeError::Coefficient`. No panic.
- [ ] **AC3** — `reconstruct(scheme, n)` computes `Σ_t u⊗v⊗w` exactly in `i64`,
  shape `(d,d,d)`.
- [ ] **AC4** — `error` / `scheme_error` are exact `i64`; `0` iff equal. No
  floating point in the path.
- [ ] **AC5 — Strassen gate.** The §2.6 Strassen 7-term scheme has
  `scheme_error(·, 2) == 0` and `is_valid(·, 2, 7)`. **Keystone.**
- [ ] **AC6 — naive baseline.** The naive `R=n³` scheme is exact for `n=2`
  (`is_valid(·, 2, 8)`) and `n=3` (`is_valid(·, 3, 27)`).

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-04 | New crate `ufl-tensor`, pure integer, no `ufl-core` dependency. | Exact `{-1,0,+1}` arithmetic; the `Complex<f64>` EML core is the wrong field (FINDINGS C3); foundational and uncontested. |
| 2026-06-04 | Dense row-major `Vec<i64>`, length `d³`. | Exact and ample for the `n` in scope; simplest correct representation. |
| 2026-06-04 | Conventions (§2.1) fixed here and cited by all later discovery requirements. | One source of truth for flattening / tensor entries / reconstruction, so R-0007/R-0008 and any third-party certificate agree. |
| 2026-06-04 | Strassen fixture transcribed and **pre-verified** (error 0) before speccing; zero-skip in `reconstruct` is a safe integer speed-up. | AC5 is the keystone — pinning a verified fixture removes transcription risk; integers have no NaN hazard so the skip is value-neutral (unlike SPEC-0002). |

## Changelog

- 2026-06-04 — created (Draft); Strassen + naive fixtures pre-verified.
