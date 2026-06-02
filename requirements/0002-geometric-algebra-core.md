# R-0002 — Geometric Algebra Core over G(3,0,0)

- **Status:** Accepted (capability); **realization pivoted to garust** 2026-06-02
- **Milestone:** M1
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-05-28 · **Revised:** 2026-06-02 (garust realization)
- **Pillar / atom:** Spatial layer — atoms `𝒢ₖ` (grade-lift) and `∗` (geometric product) (proposal Pillar 2)
- **Depends on:** R-0001 (provides the complex scalar `Value = Complex<f64>`)
- **Realized by:** SPEC-0002 (to be (re)written by the GA agent flow — garust-based)
- **Built by:** a separate agent flow (not the main UFL session). This document
  is the contract that flow builds against.
- **QA:** `qa` agent run scoped to R-0002

## 1. Statement

UFL gains a spatial layer: **multivectors over the geometric algebra
G(3,0,0)** with two atoms — `𝒢ₖ` (grade-lift: promote scalar components to a
grade-*k* multivector) and `∗` (geometric product). Multivector coefficients
are the complex `Value` that EML expressions evaluate to (R-0001), so
EML scalars sit *below* the geometric layer.

**Realization: via [`garust`](https://github.com/westerngazoo/garust).** UFL
does not hand-roll its GA. garust is a from-scratch Rust GA library, generic
over the Clifford signature `Cl(P,Q,R)` *and* the scalar type, providing
G(3,0,0) (`Vga3`), the geometric product, wedge/inner, grade projection,
reverse, `norm_squared`, the sandwich product, and a closed-form `exp` (the
bivector→rotor bridge). UFL's GA core is a thin layer over
`garust::Multivector<Complex<f64>, 3, 0, 0, 8>`.

## 2. Rationale

The proposal's Pillar 2 makes the geometric product "the universal composition
operator." R-0001 delivered the numeric substrate; R-0002 delivers the spatial
one that consumes it. Reusing garust — rather than the hand-rolled
`Multivector` of the (now-superseded) first attempt — avoids duplicating a
substantial, already-built, tested GA library, and gives UFL rotors, `exp`,
PGA, and CGA essentially for free as the GA story grows.

## 3. Acceptance criteria

- **AC1 — Multivector representation.** A G(3,0,0) multivector is exactly 8
  complex coefficients (garust's `Multivector<Complex<f64>, 3,0,0,8>`), forming
  a linear space (component-wise `+`, `−`, scalar scaling).
- **AC2 — Grade-lift `𝒢ₖ`.** A component-vector lift: grade 0 and 3 take one
  component, grades 1 and 2 take three each (`C(3,k) = 1,3,3,1`), placed on
  that grade's blades with all others zero; `k ∉ {0,1,2,3}` not representable.
- **AC3 — Geometric product axioms.** `eᵢ ∗ eᵢ = 1`; `eᵢ ∗ eⱼ = −eⱼ ∗ eᵢ`
  (i≠j); the scalar `1` is the two-sided identity. (garust provides this;
  R-0002's job is to verify it holds for the **complex-coefficient**
  instantiation.)
- **AC4 — Inner/outer.** `e₁ ∗ e₂` is the grade-2 (outer) part; `e₁ ∗ e₁₂ = e₂`
  is grade-lowering (inner); a grade-1 × grade-1 product splits into grade-0
  (dot) and grade-2 (outer).
- **AC5 — Rotor sandwich preserves grade and norm.** With a real unit rotor for
  a `+τ/4` rotation in the e₁∧e₂ plane, `R ∗ v ∗ ~R` sends `e₁ → e₂`,
  `e₂ → −e₁`, `e₃` fixed, grade-1 preserved, norm preserved. (The rotor sign /
  orientation convention and the verification oracle from the superseded
  attempt — [`experiments/r0002-rotor.py`](../experiments/r0002-rotor.py) on
  the frozen branch — carry over.)
- **AC6 — EML-scalar composition.** A multivector whose coefficients are
  produced by evaluating EML trees (R-0001 `Value`) participates correctly in
  `𝒢ₖ` and `∗`. End-to-end, mixing both atoms — the load-bearing reason GA must
  accept complex coefficients.

## 4. Constraints & non-goals

**Constraints**

- Algebra fixed at **G(3,0,0)** for R-0002.
- Coefficients are `Complex<f64>` (R-0001's `Value`) — *not* real-only.
- Realized via garust, not hand-rolled.

**Prerequisite (owned by the GA agent flow): split garust's `Scalar` trait.**
garust's current `Scalar` (scalar.rs) requires `PartialOrd` and
`abs(self) -> Self`, which `Complex<f64>` cannot satisfy (complex is unordered;
its modulus is real-valued, not `Self`). The geometric product needs only the
ring/field operations (`+ − × / ZERO ONE`). The fix — matching garust's own
"deliberate split" philosophy (it already separates `Real` out of `Scalar`):

- Introduce a **`Ring`/`Field` super-trait** with just the arithmetic +
  `ZERO`/`ONE` that the geometric product, wedge, inner, reverse, and grade
  projection require.
- Make `Scalar: Ring + PartialOrd` plus `abs` the *ordered* refinement, used
  only by tolerance/`cleaned`/`max`/versor-inverse paths.
- `Complex<f64>` implements `Ring` (and supplies a real-valued magnitude via a
  separate method/assoc type, not `abs(self) -> Self`).

This is a garust-repo change; UFL's R-0002 depends on it.

**Non-goals**

- Generalized `Cl(p,q,r)` for UFL (garust supports it; UFL fixes G(3,0,0) here).
- The **GA s-expr forms** (`𝒢ₖ`/`∗` as forms lowering into the garust
  multivector) — a separate later UFL requirement.
- Predicates, neural layer, differentiability of grade projection.

## 5. Open questions

- **Norm under complex coefficients.** Define UFL's multivector norm as the
  coefficient norm `√Σ|cᵢ|²` (always real; agrees with the GA norm on real
  grade-1 vectors), since garust's `norm_squared` returns `Self` and is
  Real-oriented. The GA flow's SPEC fixes this.
- **Blade order reconciliation.** garust stores in **bitmask** order
  (`coeffs[0]` scalar, index = generator bitmask); the superseded attempt used
  grade-then-lex storage with a `MASK` permutation. The GA SPEC adopts garust's
  bitmask order as canonical and records it in `docs/conventions.md`.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | First GA is G(3,0,0), dense 8-coefficient, complex coefficients reusing R-0001's `Value`. | 3D Euclidean is the proposal's worked example; complex coefficients make "EML scalars below GA" literal. |
| 2026-06-02 | **Realization pivoted from hand-rolled to garust.** | garust is an existing, tested, from-scratch GA lib generic over signature + scalar; reusing it avoids duplicating substantial work and yields rotors/`exp`/PGA/CGA for free. (Owner decision after assessing `~/projects/garust`.) |
| 2026-06-02 | **Prerequisite: split garust's `Scalar`** into a `Ring`/`Field` super-trait (geometric product) + an ordered `Scalar` refinement, so `Complex<f64>` is an admissible coefficient. Owned by the GA agent flow. | garust's `Scalar` bundles `PartialOrd` + `abs(self)->Self`, which complex can't satisfy; the product needs only ring ops. Matches garust's own `Real`-out-of-`Scalar` split philosophy. |
| 2026-06-02 | The hand-rolled attempt (frozen branch `R-0002-geometric-algebra`, tip `c92a38a`) is **superseded prior art** — its blade conventions, rotor-sign fix, and `r0002-rotor.py` oracle carry over as reference. | The three-lens review already hardened those; no reason to re-derive. |

## Changelog

- 2026-05-28 — created; accepted; hand-rolled SPEC-0002 drafted (three-lens
  reviewed), then paused at the homoiconic-s-expr pivot.
- 2026-06-02 — realization pivoted to garust (Scalar-split prerequisite);
  rewritten as the contract for the GA agent flow. Hand-rolled approach
  superseded.
