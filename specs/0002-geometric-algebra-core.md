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
each blade is a **3-bit mask** over basis vectors `{e₁=bit0, e₂=bit1, e₃=bit2}`,
and `popcount(mask)` is the grade. The storage order (the grade-then-lex table
above) and the mask order differ, so the map is a fixed permutation, pinned as
a **single named constant** that both the oracle and the table builder consume
(so the §2.4.1 tripwire covers the permutation, not just the table):

```rust
/// Storage index → basis-vector mask. The one place the blade encoding lives.
const MASK: [u8; 8] = [
    0b000, // 0: 1
    0b001, // 1: e₁
    0b010, // 2: e₂
    0b100, // 3: e₃
    0b011, // 4: e₁₂
    0b101, // 5: e₁₃
    0b110, // 6: e₂₃
    0b111, // 7: e₁₂₃
];
```

The mask encoding is **signature-agnostic**: the result-blade rule
`mask_i XOR mask_j` (§2.4.1) is dimension- and signature-independent. Only the
*sign* rule specializes the `(+,+,+)` signature — this is the seam where a
future G(p,q,r) requirement plugs in (recorded in
[`docs/conventions.md`](../docs/conventions.md)).

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
    /// Coefficient at a blade index in `0..8`. Indices are program-derived
    /// (from `MASK` / the §2.1 table), never external input; an out-of-range
    /// index is a genuine unreachable state, so this panics with a justifying
    /// message rather than returning `Result` (CLAUDE.md §6).
    pub fn coeff(&self, blade: usize) -> Value;
    /// The grade-`k` part (blades of grade `k` kept, others zeroed). `k` is in
    /// `0..=3`; a larger grade is likewise an unreachable state (panics).
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

The sign/result of `blade_i ∗ blade_j` is computed from the 3-bit masks via
**one unambiguous algorithm**:

- **Result blade** = `mask_i XOR mask_j` (shared basis vectors square to `+1`
  and cancel; the symmetric difference survives). This step is signature-
  independent.
- **Sign** = `(-1)^s`, computed as follows: form the concatenated basis-vector
  list `[i's vectors ascending][j's vectors ascending]`; sort the whole list
  into ascending order by **adjacent transpositions**, counting the total
  number of swaps `s`; *then* delete adjacent equal indices in the sorted list
  (each is an `eₖ² = +1`, contributing no sign). The sign is `(-1)^s`. The
  deletion happens *after* the swap count, never during it.

  This is the step that assumes the `(+,+,+)` signature: every squared pair
  contributes `+1`. Under a `(p,q,0)` signature a deleted `eₖ² = -1` pair would
  flip the sign, so this rule is G(3,0,0)-specific by design.

SPEC-0002 ships a **generation test** that recomputes all 64 entries by this
rule (consuming the same `MASK` constant) and asserts they equal the stored
`CAYLEY`. The table is the fast path; the rule is the oracle. If the two ever
disagree (a hand edit, a reorder, a changed `MASK`), the test fails loudly —
the **Oracle-Tripwire pattern** (see
[`docs/conventions.md`](../docs/conventions.md)), the same discipline as
SPEC-0001's AC6.

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

`|cᵢ|` is the complex modulus (`Value::norm`). This is total and real for *any*
multivector, including complex-coefficient ones — that totality is exactly why
it is chosen over the GA norm `√⟨M ∗ ~M⟩₀`, which is complex (and branch-cut-
prone) for general complex coefficients.

The two norms **coincide only on real-coefficient vectors**. For a *real*
grade-1 vector `v`, `v ∗ ~v = v ∗ v = Σ vᵢ²` (a real scalar) `= Σ|vᵢ|²`, so
`norm()` equals `√⟨v ∗ ~v⟩₀`. They do *not* agree for complex coefficients:
e.g. `v = (1+i)e₁` gives `⟨v ∗ ~v⟩₀ = 2i` (complex) while `norm() = √2`. AC5's
test vectors and rotor are therefore constrained to **real coefficients**
(§6 AC5), where the coefficient norm is the unambiguous physical magnitude.
The complex GA norm is deferred until a consumer needs it (R-0002 non-goal).

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
        // The reverse negates grades k with k(k-1)/2 odd — i.e. grades 2 and 3
        // in G(3,0,0). Stated directly as a per-blade constant (§2.1 order) so
        // the obvious reading is the correct reading; no arithmetic to defend.
        const NEGATE: [bool; 8] = [
            false, // 1     grade 0
            false, // e₁    grade 1
            false, // e₂    grade 1
            false, // e₃    grade 1
            true,  // e₁₂   grade 2
            true,  // e₁₃   grade 2
            true,  // e₂₃   grade 2
            true,  // e₁₂₃  grade 3
        ];
        let mut out = *self;
        for (c, &neg) in out.coeffs.iter_mut().zip(NEGATE.iter()) {
            if neg {
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
        // Pure ∑ᵢ∑ⱼ aᵢ bⱼ · CAYLEY[i][j] — no zero-coefficient short-circuit.
        // A `continue` on aᵢ == 0 would drop terms like 0 · ∞ = NaN, silently
        // diverging from the stated formula and from SPEC-0001's AC3 inf/NaN
        // propagation discipline. At reference scale the full 64-term loop is
        // the correct, simplest behaviour.
        let mut out = Multivector::zero();
        for i in 0..8 {
            let a = self.coeffs[i];
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

- **AC5/AC6 tolerance.** Proposed relative tolerance `1e-12` for the
  grade-zeroing, direction, and norm checks — looser than SPEC-0001's `1e-14`
  because the rotor sandwich is two geometric products deep (more rounding).
  To be confirmed with the `qa` agent against the actual computed residual;
  the three-lens review measured the rotor preservation at machine zero, so
  `1e-12` is expected to be comfortably generous.

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
- [ ] **AC5** — rotor sandwich `R ∗ v ∗ ~R` with the unit rotor
  `R = 𝒢₀(cos(τ/8)) + 𝒢₂([−sin(τ/8), 0, 0])` (a `+τ/4` rotation in the e₁∧e₂
  plane; the **`−sin`** sign per the rotor orientation in
  [`docs/conventions.md`](../docs/conventions.md)). All coefficients are real.
  The test asserts, to the §5 tolerance, **all three** of:
  1. **Direction & plane (mandatory, not "e.g.").** `e₁ → e₂` *and* `e₂ → −e₁`
     — two inputs pin both the rotation plane and its sign; a wrong-direction
     or identity rotor fails at least one.
  2. **Negative control.** `e₃ → e₃` (fixed) — distinguishes a genuine e₁₂
     rotor from identity and from a wrong-plane rotor.
  3. **Grade & norm.** each `v'` is grade-1 (non-grade-1 blades zero) and
     `norm(v') = norm(v)`.
- [ ] **AC6** — composition of the EML and GA atoms, asserted with a concrete
  number (not a type-level "they compose"). Build `v = 𝒢₁([eval(eml(1,1)), 0,
  0])` — `eml(1,1)` evaluates to `e ≈ 2.71828`, so `v = e·e₁`. Run it through
  the AC5 rotor: `R ∗ v ∗ ~R` must equal `e·e₂` to the §5 tolerance. This
  exercises the full EML-tree → `Value` → `lift` → `∗` data path with a
  falsifiable expected value.
- [ ] **Cayley tripwire** — the §2.4.1 generation test: the stored `CAYLEY`
  equals the rule-derived table for all 64 entries.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | Multivector is a dense `[Value; 8]` struct, `Copy`, in grade-then-lex blade order. | AC1 holds structurally; value semantics chosen for clarity. Copy cost (128 bytes) is irrelevant at reference scale and revisited only if GA enters a hot path (a future requirement); no performance claim is made here. |
| 2026-05-28 | `𝒢ₖ` is `Multivector::lift(GradeLift)` with an enum carrying per-grade fixed arrays. | R-0002 AC2 (component-vector lift); arity and the `k ≤ 3` bound are structural — the SPEC-0001 type-admits-exactly-valid-inputs discipline. |
| 2026-05-28 | `∗` is a static `8×8` Cayley table, with a rule-based oracle + generation test pinning it (consuming a single `MASK` constant). | At G(3,0,0) the table is 64 exact entries — small, fast, auditable. Rule-based on-the-fly multiplication is the general mechanism but over-engineered here; deriving the table by the rule and testing equality gets correctness *and* speed (the Oracle-Tripwire pattern). |
| 2026-05-28 | Reverse `~` and the coefficient norm are plain methods, not atoms. | They are structural support for AC5's rotor sandwich; UFL's atom set stays `𝒢ₖ` + `∗`. |
| 2026-05-28 | `ga` is a submodule of `ufl-core`, beside `eml`. | R-0002 targets `ufl-core`; both atoms share `Value`; no new crate warranted yet. |
| 2026-05-28 | Three-lens review (architect + hater + nice-guy) applied before acceptance. | Rotor sign **corrected** — the verified output of `cos(τ/8) + e₁₂ sin(τ/8)` is `e₁ → −e₂`; the spec now uses `−sin(τ/8)` so a `+τ/4` rotation sends `e₁ → e₂` per the recorded orientation convention. AC5 strengthened with `e₂ → −e₁`, an `e₃`-fixed negative control, and a mandatory direction check (was a trivially-passable "e.g."). AC6 given a concrete falsifiable value (`e·e₁ → e·e₂`). The `Mul` zero-skip removed (it broke SPEC-0001 AC3 inf/NaN propagation). Norm equivalence scoped to real coefficients. `MASK` pinned as one constant; `reverse` stated as a `NEGATE` table; blade order + rotor orientation + the Oracle-Tripwire pattern recorded in `docs/conventions.md`. |

## Changelog

- 2026-05-28 — created (Draft).
- 2026-05-28 — three-lens review applied: rotor sign corrected (`−sin`, `e₁→e₂`); AC5 strengthened (two-input direction check + `e₃` negative control); AC6 given a concrete expected value; `Mul` zero-skip removed; norm equivalence scoped to real coefficients; `MASK`/`NEGATE` constants pinned; sign-rule algorithm disambiguated; `coeff`/`grade` panic intent documented; blade order, rotor orientation, and the Oracle-Tripwire pattern recorded in `docs/conventions.md`.
