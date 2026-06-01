# SPEC-0002 — Geometric Algebra Core over G(3,0,0)

- **Status:** Draft
- **Realizes:** R-0002
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-05-28
- **Depends on:** SPEC-0001 (provides `Value = Complex<f64>`)
- **Crate(s):** `ufl-core`

## 1. Motivation

SPEC-0002 realizes [R-0002](../requirements/0002-geometric-algebra-core.md): the
spatial layer of UFL. It gives the two atoms `𝒢ₖ` (grade-lift) and `∗`
(geometric product) a concrete Rust representation over the geometric algebra
**G(3,0,0)**, on top of the EML scalar substrate from SPEC-0001.

The multivector is the second universal object of UFL — where EML trees are the
universal *numeric* mechanism, multivectors are the universal *spatial* one.
Their coefficients are EML's `Value`, so EML scalars sit literally below the
geometric layer.

As with SPEC-0001 the goal is *correctness, legibly stated* — a dense reference
implementation, not an optimized one.

## 2. Design

### 2.1 The algebra and its blades

G(3,0,0) is the Clifford algebra of 3-dimensional Euclidean space, signature
`(+,+,+)`. It has `2³ = 8` basis blades. SPEC-0002 fixes their order once, in
**grade-then-lexicographic** order, and indexes them `0..8`:

| index | blade | grade |
|-------|-------|-------|
| 0 | `1`     | 0 |
| 1 | `e₁`    | 1 |
| 2 | `e₂`    | 1 |
| 3 | `e₃`    | 1 |
| 4 | `e₁₂`   | 2 |
| 5 | `e₁₃`   | 2 |
| 6 | `e₂₃`   | 2 |
| 7 | `e₁₂₃`  | 3 |

This order is recorded in [`docs/conventions.md`](../docs/conventions.md) when
this spec is accepted, so every later layer agrees.

A compact internal encoding makes the product table derivable and checkable:
each blade is a **3-bit mask** over basis vectors `{e₁=bit0, e₂=bit1, e₃=bit2}`.
Thus `1 = 0b000`, `e₁ = 0b001`, `e₂ = 0b010`, `e₁₂ = 0b011`, `e₃ = 0b100`,
`e₁₃ = 0b101`, `e₂₃ = 0b110`, `e₁₂₃ = 0b111`. The mask's `popcount` is the
grade. (Note: the *storage* order is the grade-then-lex table above; the
bitmask is an internal device for deriving the Cayley table in §2.4, mapped to
storage indices by a fixed permutation.)

### 2.2 The multivector type

```rust
/// A multivector of G(3,0,0): 8 complex coefficients in grade-then-lex
/// blade order (see §2.1). R-0002 AC1 holds structurally — the array is
/// always length 8.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multivector {
    /// Coefficients indexed by blade, in the §2.1 order.
    coeffs: [Value; 8],
}
```

`[Value; 8]` makes AC1 structural: no other component count is representable.
`Copy` is cheap and reasonable for 8 complex numbers (128 bytes) and keeps the
algebra value-semantic.

Multivectors form a **linear space** (AC1): component-wise `Add`, `Sub`, and
scalar multiplication by a `Value`.

```rust
impl Multivector {
    /// The additive identity (all coefficients zero).
    pub fn zero() -> Self;
    /// Coefficient at a blade index `0..8`.
    pub fn coeff(&self, blade: usize) -> Value;
    /// The grade-`k` part (blades of grade k kept, others zeroed).
    pub fn grade(&self, k: u8) -> Self;
}

impl std::ops::Add for Multivector { /* component-wise */ }
impl std::ops::Sub for Multivector { /* component-wise */ }
impl std::ops::Mul<Value> for Multivector { /* scale every coefficient */ }
```

### 2.3 Grade-lift `𝒢ₖ`

Per R-0002 AC2 (component-vector lift), grade-lift is **one atom parameterized
by an enum** whose variants carry the exact component arity of each grade. This
is the SPEC-0001 discipline — the type admits exactly the valid inputs, so
arity and the `k ≤ 3` bound are structural, not runtime-checked.

```rust
/// Input to the grade-lift atom `𝒢ₖ`. Each variant carries exactly the
/// number of components of its grade (1, 3, 3, 1 — the binomial row C(3,k)).
/// Grades outside {0,1,2,3} are unrepresentable.
pub enum GradeLift {
    Scalar(Value),          // 𝒢₀ — grade 0: blade `1`
    Vector([Value; 3]),     // 𝒢₁ — grade 1: e₁ e₂ e₃
    Bivector([Value; 3]),   // 𝒢₂ — grade 2: e₁₂ e₁₃ e₂₃
    Trivector(Value),       // 𝒢₃ — grade 3: e₁₂₃
}

impl Multivector {
    /// The grade-lift atom 𝒢ₖ: place a grade's components on its blades,
    /// zeroing all others.
    pub fn lift(input: GradeLift) -> Self;
}
```

### 2.4 The geometric product `∗` — Cayley table

The product is **table-driven**: a static `8 × 8` Cayley table maps a pair of
blade indices to a `(sign, result_blade)` pair. For G(3,0,0) every blade
product is a single signed blade (the algebra is non-degenerate and the basis
orthonormal), so the table is exact and total.

```rust
/// (sign, result blade index) for the product of two basis blades.
/// `sign ∈ {+1, -1}`; the result is always a single blade in G(3,0,0).
struct BladeProduct { sign: i8, blade: usize }

/// CAYLEY[i][j] = blade_i ∗ blade_j, in the §2.1 index order.
const CAYLEY: [[BladeProduct; 8]; 8] = /* 64 entries, see §2.4.1 */;

impl std::ops::Mul for Multivector {
    type Output = Multivector;
    /// Geometric product: ∑ᵢ ∑ⱼ aᵢ bⱼ · CAYLEY[i][j].
    fn mul(self, rhs: Multivector) -> Multivector { /* double loop over CAYLEY */ }
}
```

**Why a table, not on-the-fly blade multiplication.** At G(3,0,0) the table is
64 fixed entries — small, exact, and trivially auditable. Rule-based blade
multiplication (sort basis indices, count transposition swaps for the sign,
XOR masks for the result) is the *general* mechanism for arbitrary G(p,q,r),
but it is more logic to get right and buys nothing at this fixed scale. R-0002
is explicitly G(3,0,0)-only (non-goal: generalized algebras), so the table is
the reference-first choice. **§2.4.1 derives the table by the rule-based method
and a generation test pins it**, so the table is *verified*, not hand-transcribed.

#### 2.4.1 Table derivation and its tripwire

The sign/result of `blade_i ∗ blade_j` is computed from the 3-bit masks:

- **Result blade** = `mask_i XOR mask_j` (shared basis vectors square to `+1`
  and cancel; the symmetric difference survives).
- **Sign** = `(-1)^s` where `s` is the number of adjacent transpositions needed
  to sort the concatenated basis-vector list `[i's vectors][j's vectors]` into
  ascending order, with each squared pair removed as it meets (all squares are
  `+1` in signature `(+,+,+)`).

SPEC-0002 ships a **generation test** that recomputes all 64 entries by this
rule and asserts they equal the stored `CAYLEY`. The table is the fast path;
the rule is the oracle. If the two ever disagree (a hand edit, a reorder), the
test fails loudly — the same tripwire discipline as SPEC-0001's AC6.

### 2.5 The reverse `~`

The Clifford reverse reverses the order of basis vectors in each blade, which
flips the sign of grades `k` with `k(k−1)/2` odd — i.e. grades 2 and 3 in
G(3,0,0):

```rust
impl Multivector {
    /// The Clifford reverse `~M`: negate grade-2 and grade-3 components.
    pub fn reverse(&self) -> Self;
}
```

The reverse is the structural operation behind the rotor sandwich (AC5); it is
not a UFL atom.

### 2.6 The coefficient norm

Per R-0002 AC5 the norm is the **coefficient norm** — the multivector's
coefficients treated as a vector in ℂ⁸:

```rust
impl Multivector {
    /// |M| = √(Σᵢ |cᵢ|²), always real and non-negative.
    pub fn norm(&self) -> f64;
}
```

`|cᵢ|` is the complex modulus (`Value::norm`). This is total and real for any
multivector, and coincides with the GA norm `√⟨M ∗ ~M⟩₀` on the real grade-1
vectors AC5 tests (proof sketch: for a real vector `v`, `v ∗ ~v = v ∗ v` is the
scalar `Σ vᵢ²`, equal to `Σ|vᵢ|²`).

### 2.7 Module layout

```
crates/ufl-core/
└── src/
    ├── lib.rs    — adds `pub mod ga;` and re-exports
    └── ga/
        ├── mod.rs        — Multivector, linear-space ops, grade(), norm(), reverse()
        ├── lift.rs       — GradeLift enum + Multivector::lift
        └── product.rs    — BladeProduct, CAYLEY, the rule-based oracle, Mul impl
```

The geometric core lives in a `ga` submodule of the existing `ufl-core` crate,
sitting beside `eml` — both consume the shared `Value`. No new crate (R-0002
target is `ufl-core`).

## 3. Code outline

Representative — refined with the owner before implementation.

```rust
// ga/mod.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multivector {
    coeffs: [Value; 8],
}

impl Multivector {
    pub fn zero() -> Self { Self { coeffs: [Value::new(0.0, 0.0); 8] } }
    pub fn coeff(&self, blade: usize) -> Value { self.coeffs[blade] }

    pub fn norm(&self) -> f64 {
        self.coeffs.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt()
    }

    pub fn reverse(&self) -> Self {
        // grades by blade index (§2.1): [0,1,1,1,2,2,2,3]
        const GRADE: [u8; 8] = [0, 1, 1, 1, 2, 2, 2, 3];
        let mut out = *self;
        for (i, c) in out.coeffs.iter_mut().enumerate() {
            let k = GRADE[i];
            if (k * (k.wrapping_sub(1)) / 2) % 2 == 1 {
                *c = -*c;
            }
        }
        out
    }
}

// ga/lift.rs
pub enum GradeLift {
    Scalar(Value),
    Vector([Value; 3]),
    Bivector([Value; 3]),
    Trivector(Value),
}

impl Multivector {
    pub fn lift(input: GradeLift) -> Self {
        let mut m = Multivector::zero();
        match input {
            GradeLift::Scalar(s)     => m.coeffs[0] = s,
            GradeLift::Vector(v)     => m.coeffs[1..4].copy_from_slice(&v),
            GradeLift::Bivector(b)   => m.coeffs[4..7].copy_from_slice(&b),
            GradeLift::Trivector(t)  => m.coeffs[7] = t,
        }
        m
    }
}

// ga/product.rs  (Mul impl, double loop)
impl std::ops::Mul for Multivector {
    type Output = Multivector;
    fn mul(self, rhs: Multivector) -> Multivector {
        let mut out = Multivector::zero();
        for i in 0..8 {
            let a = self.coeffs[i];
            if a == Value::new(0.0, 0.0) { continue; }
            for j in 0..8 {
                let BladeProduct { sign, blade } = CAYLEY[i][j];
                let term = a * rhs.coeffs[j];
                out.coeffs[blade] += if sign >= 0 { term } else { -term };
            }
        }
        out
    }
}
```

## 4. Non-goals

- Generalized G(p,q,r) / arbitrary-dimension algebras (R-0002 is G(3,0,0)-only).
- Sparse / SIMD multivector storage; performance tuning.
- Surface syntax for multivectors (parser is R-0005).
- General multivector exponential `exp(B) → rotor`; R-0002 only requires a
  rotor assembled from `𝒢₀`/`𝒢₂` components to rotate correctly (AC5).
- Differentiability of grade projection (proposal Q4).

## 5. Open questions

- **AC5 tolerance.** Proposed relative tolerance `1e-12` for the grade-zeroing
  and norm-preservation checks, looser than SPEC-0001's `1e-14` because the
  rotor sandwich is two geometric products deep (more rounding). To be confirmed
  with the `qa` agent against the actual computed residual.

## 6. Acceptance criteria

Each row maps a SPEC-0002 deliverable to an R-0002 acceptance criterion; the
`qa` agent's tests verify them.

- [ ] **AC1** — `Multivector` is exactly `[Value; 8]`; `Add`/`Sub`/`Mul<Value>`
  give the linear-space structure. Structural + unit tests.
- [ ] **AC2** — `Multivector::lift(GradeLift)` places each grade's components on
  its blades and zeroes the rest; grade > 3 is unrepresentable (compile-time).
- [ ] **AC3** — `eᵢ ∗ eᵢ = 1`, `eᵢ ∗ eⱼ = − eⱼ ∗ eᵢ` (i ≠ j), `1 ∗ M = M`.
  Verified by unit tests over the basis and the generation test of §2.4.1.
- [ ] **AC4** — `e₁ ∗ e₂ = e₁₂`; `e₁ ∗ e₁₂ = e₂`; a general grade-1 × grade-1
  product splits into grade-0 (dot) and grade-2 (outer) parts.
- [ ] **AC5** — rotor sandwich `R ∗ v ∗ ~R` with `R = 𝒢₀(cos(τ/8)) +
  𝒢₂([sin(τ/8),0,0])` keeps `v` grade-1 and preserves `norm()` to the §5
  tolerance; a known input rotates to the expected output (e.g. `e₁ → e₂`).
- [ ] **AC6** — a `Multivector` whose coefficients are produced by `eval`-ing
  EML trees (SPEC-0001) participates correctly in `lift` and `∗`. End-to-end
  test mixing both atoms.
- [ ] **Cayley tripwire** — the §2.4.1 generation test: the stored `CAYLEY`
  equals the rule-derived table for all 64 entries.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | Multivector is a dense `[Value; 8]` struct, `Copy`, in grade-then-lex blade order. | AC1 holds structurally; 128 bytes is cheap to copy; value semantics match the algebra. |
| 2026-05-28 | `𝒢ₖ` is `Multivector::lift(GradeLift)` with an enum carrying per-grade fixed arrays. | R-0002 AC2 (component-vector lift); arity and the `k ≤ 3` bound are structural — the SPEC-0001 type-admits-exactly-valid-inputs discipline. |
| 2026-05-28 | `∗` is a static `8×8` Cayley table, with a rule-based oracle + generation test pinning it. | At G(3,0,0) the table is 64 exact entries — small, fast, auditable. Rule-based on-the-fly multiplication is the general mechanism but over-engineered here; deriving the table by the rule and testing equality gets correctness *and* speed. |
| 2026-05-28 | Reverse `~` and the coefficient norm are plain methods, not atoms. | They are structural support for AC5's rotor sandwich; UFL's atom set stays `𝒢ₖ` + `∗`. |
| 2026-05-28 | `ga` is a submodule of `ufl-core`, beside `eml`. | R-0002 targets `ufl-core`; both atoms share `Value`; no new crate warranted yet. |

## Changelog

- 2026-05-28 — created (Draft).
