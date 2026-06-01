# R-0002 — Geometric Algebra Core over G(3,0,0)

- **Status:** Accepted (2026-05-28 — open questions resolved together)
- **Milestone:** M1
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-05-28
- **Pillar / atom:** Spatial layer — atoms `𝒢ₖ` (grade-lift) and `∗` (geometric product) (proposal Pillar 2)
- **Depends on:** R-0001 (EML operator core — provides the complex scalar `Value`)
- **Realized by:** [SPEC-0002](../specs/0002-geometric-algebra-core.md)
- **QA:** `qa` agent run scoped to R-0002

## 1. Statement

UFL gains a spatial layer: **multivectors over the geometric algebra
G(3,0,0)** (3-dimensional Euclidean space), built on top of the EML scalar
substrate. Two atoms are introduced:

- **`𝒢ₖ` — grade-lift.** Promote a scalar to a grade-*k* multivector, assigning
  geometric meaning (`k ∈ {0,1,2,3}`) to a numeric value.
- **`∗` — geometric product.** The universal composition operator on
  multivectors — encoding inner (contraction) and outer (extension) products
  simultaneously.

Multivector coefficients are the same complex `Value` that EML expressions
evaluate to, so a multivector's components may be computed by EML trees: EML
scalars sit *below* the geometric layer.

The canonical *rotor sandwich* `R ∗ v ∗ ~R` (rotation of a vector by a rotor)
is a derived construction, not a new atom; it is realized by composing `∗` with
the Clifford reverse `~`.

## 2. Rationale

The proposal's Pillar 2 ([`docs/ufl-first-draft.md`](../docs/ufl-first-draft.md)
§2) makes the geometric product "the universal composition operator" — it
generalizes the dot product, the cross product, and complex multiplication at
once, and rotors describe transformations. R-0001 delivered the numeric
substrate; R-0002 delivers the spatial substrate that consumes it. Together
they are the irreducible core (milestone M1) every later layer builds on:
predicates (R-0004) constrain multivector-valued state, and the neural layer
(proposal §4) expresses a layer as a grade-filtered geometric product.

G(3,0,0) is chosen as the first algebra because it is the proposal's worked
example (the rotation sketch in §3) and because 3D Euclidean space is the
smallest algebra rich enough to exhibit every grade phenomenon UFL needs:
scalars, vectors, bivectors (oriented planes / rotors), and the pseudoscalar.

## 3. Acceptance criteria

- **AC1 — Multivector representation.** A G(3,0,0) multivector is representable
  as exactly **8 complex coefficients**, one per basis blade of the algebra, in
  grade-then-lexicographic order `{1, e₁, e₂, e₃, e₁₂, e₁₃, e₂₃, e₁₂₃}`. No
  other component count is representable. Multivectors form a linear space over
  the complex `Value`: component-wise addition and `Value`-scaling are defined
  (needed to assemble, e.g., a rotor from a scalar part and a bivector part).
- **AC2 — Grade-lift `𝒢ₖ`.** Grade-lift takes the *components of a single
  grade* and places them on that grade's blades, zeroing every other blade.
  Grades 0 and 3 take one component each (the `1` and `e₁₂₃` blades); grades 1
  and 2 take three components each (`e₁,e₂,e₃` and `e₁₂,e₁₃,e₂₃`, in blade
  order). Lifting a grade outside `{0,1,2,3}` is **not representable** —
  structurally precluded, not runtime-rejected.
- **AC3 — Geometric product axioms.** `∗` satisfies the orthonormal-basis
  axioms of G(3,0,0): `eᵢ ∗ eᵢ = 1` for each `i ∈ {1,2,3}`, and
  `eᵢ ∗ eⱼ = − eⱼ ∗ eᵢ` for `i ≠ j`. The scalar `1` blade is the product
  identity: `1 ∗ M = M ∗ 1 = M`.
- **AC4 — Inner and outer behaviour.** The product encodes both: for orthogonal
  vectors `e₁ ∗ e₂ = e₁₂` (a pure grade-2 / outer result), and the contraction
  `e₁ ∗ e₁₂ = e₂` (a grade-lowering / inner result). A general product of two
  grade-1 vectors yields a grade-0 part equal to their dot product and a
  grade-2 part equal to their outer product.
- **AC5 — Rotor sandwich preserves grade and norm.** With the unit rotor
  `R = 𝒢₀(cos(τ/8)) + 𝒢₂([−sin(τ/8), 0, 0])` (a `+τ/4` rotation in the e₁∧e₂
  plane — the bivector component on `e₁₂`, sign per the rotor orientation in
  [`docs/conventions.md`](../docs/conventions.md)) and a real grade-1 vector
  `v`, the result `v' = R ∗ v ∗ ~R` is grade-1 (non-grade-1 blades zero to
  tolerance) and `|v'| = |v|` under the coefficient norm `|M| = √Σᵢ|cᵢ|²`. The
  rotation direction is pinned in SPEC-0002 (`e₁ → e₂`, `e₂ → −e₁`, `e₃` fixed).
  The reverse `~R` negates the grade-2 (and grade-3) part of `R`.
- **AC6 — EML-scalar composition.** A multivector whose coefficients are
  obtained by evaluating EML trees (R-0001) participates correctly in `𝒢ₖ` and
  `∗` — i.e. the geometric layer consumes EML `Value`s with no separate scalar
  type. Demonstrated by an end-to-end case mixing both atoms.

## 4. Constraints & non-goals

**Constraints**

- The algebra is fixed at **G(3,0,0)** — signature `(+,+,+)`, 3 basis vectors,
  8 blades. Other signatures are out of scope for R-0002.
- Coefficients are the R-0001 `Value` (`Complex<f64>`). No new scalar type.
- Representation is **dense** (all 8 coefficients stored). This is a reference
  implementation, mirroring R-0001's correctness-first stance.

**Non-goals** (each a separate, later requirement)

- Generalized G(p,q,r) / arbitrary-dimension algebras.
- Sparse or SIMD-optimized multivector storage; performance tuning.
- Surface syntax for multivectors (parser is R-0005).
- The grade-filtered neural layer (proposal §4) — depends on this but is later.
- Differentiability of the grade-projection operator (proposal Q4).
- The exponential map `exp(𝒢₂(·)) → rotor` as a *general* operator — R-0002
  requires only that a rotor built from `𝒢₀`/`𝒢₂` components rotates correctly
  (AC5); a general multivector exponential is deferred.

## 5. Open questions

The three design questions raised at drafting are now resolved (see the
decision log). What remains is a SPEC-0002 implementation detail, not a
requirement-level unknown:

- **AC5 tolerance.** The exact relative tolerance for the grade/norm
  preservation test is fixed in SPEC-0002 with the `qa` agent, informed by the
  rotor's tree depth and the complex `exp`/`ln` rounding inherited from R-0001.

The blade-order and sign convention is to be recorded in
[`docs/conventions.md`](../docs/conventions.md) when SPEC-0002 is accepted.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | First GA is **G(3,0,0)**, dense 8-coefficient representation, complex coefficients reusing R-0001's `Value`. | 3D Euclidean is the proposal's worked example and the smallest algebra exhibiting all grades; dense + reference-first matches R-0001's stance; reusing `Value` makes "EML scalars below GA" literal. |
| 2026-05-28 | Rotor sandwich `R ∗ v ∗ ~R` is a *derived* construction (AC5), not a new atom. | Keeps the atom set at `𝒢ₖ` + `∗` (+ reverse `~` as a structural operation); rotation is a theorem, not a primitive — consistent with the EML stance of deriving rather than baking in. |
| 2026-05-28 | General multivector `exp` deferred; R-0002 only needs a rotor assembled from `𝒢₀`/`𝒢₂` components to rotate correctly. | Bounds scope to the two atoms; the exponential map is a substantial sub-topic better handled once its consumers (neural layer) exist. |
| 2026-05-28 | Blade order is grade-then-lexicographic: `1, e₁, e₂, e₃, e₁₂, e₁₃, e₂₃, e₁₂₃`. Recorded in `docs/conventions.md` when SPEC-0002 is accepted. | One fixed order every later layer relies on; grade-then-lex is conventional and reads naturally. |
| 2026-05-28 | `𝒢ₖ` is a **component-vector** lift (each grade's lift takes exactly that grade's blade-count of components), not a single-scalar broadcast. | Refines the proposal's loose "lift a scalar" wording, as EML refined Pillar 1 — a single scalar cannot specify a general grade-1 or grade-2 element. Intended SPEC-0002 encoding: one `grade_lift` atom parameterized by an enum whose variants carry fixed-size component arrays, making arity and the `k ≤ 3` bound structural (the R-0001 "type admits exactly valid inputs" discipline). |
| 2026-05-28 | The multivector norm is the coefficient norm `|M| = √Σᵢ|cᵢ|²` over the 8 complex coefficients. | Always real and total (no complex-√ branch issue); coincides with the GA norm `√⟨M∗~M⟩₀` on the real grade-1 vectors AC5 tests, so the rotor result is unambiguous. The complex GA norm is deferred until a consumer needs it. |

## Changelog

- 2026-05-28 — created (Draft).
- 2026-05-28 — resolved the three drafting open questions (blade order, component-vector grade-lift, coefficient norm); AC1/AC2/AC5 reworded accordingly; decision log extended.
