# R-0002 — Geometric Algebra Core over G(3,0,0)

- **Status:** Draft
- **Milestone:** M1
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-05-28
- **Pillar / atom:** Spatial layer — atoms `𝒢ₖ` (grade-lift) and `∗` (geometric product) (proposal Pillar 2)
- **Depends on:** R-0001 (EML operator core — provides the complex scalar `Value`)
- **Realized by:** SPEC-0002 (pending)
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
  as exactly **8 complex coefficients**, one per basis blade of the algebra:
  `{1, e₁, e₂, e₃, e₁₂, e₁₃, e₂₃, e₁₂₃}`. No other component count is
  representable.
- **AC2 — Grade-lift `𝒢ₖ`.** For `k ∈ {0,1,2,3}`, `𝒢ₖ(s)` produces a
  multivector whose blades of grade *k* carry the scalar `s` (in the documented
  component order) and whose every other blade is zero. `k > 3` is rejected.
- **AC3 — Geometric product axioms.** `∗` satisfies the orthonormal-basis
  axioms of G(3,0,0): `eᵢ ∗ eᵢ = 1` for each `i ∈ {1,2,3}`, and
  `eᵢ ∗ eⱼ = − eⱼ ∗ eᵢ` for `i ≠ j`. The scalar `1` blade is the product
  identity: `1 ∗ M = M ∗ 1 = M`.
- **AC4 — Inner and outer behaviour.** The product encodes both: for orthogonal
  vectors `e₁ ∗ e₂ = e₁₂` (a pure grade-2 / outer result), and the contraction
  `e₁ ∗ e₁₂ = e₂` (a grade-lowering / inner result). A general product of two
  grade-1 vectors yields a grade-0 part equal to their dot product and a
  grade-2 part equal to their outer product.
- **AC5 — Rotor sandwich preserves grade and norm.** With
  `R = 𝒢₀(cos(τ/8)) + 𝒢₂(sin(τ/8))·e₁₂` (a τ/4 rotation in the e₁∧e₂ plane)
  and a grade-1 vector `v`, the result `v' = R ∗ v ∗ ~R` is grade-1 (its
  non-grade-1 blades are zero to tolerance) and `|v'| = |v|`, each to a
  relative tolerance fixed in SPEC-0002. The reverse `~R` negates the grade-2
  part of `R`.
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

- **Component order & sign convention.** The exact ordering of the 8 blades and
  the orientation signs (e.g. `e₁₃` vs `e₃₁`) must be fixed in SPEC-0002 and
  documented in [`docs/conventions.md`](../docs/conventions.md), so every later
  layer agrees. Proposed: blades in grade-then-lexicographic order
  `1, e₁, e₂, e₃, e₁₂, e₁₃, e₂₃, e₁₂₃`.
- **`𝒢ₖ` for k with multiple blades.** Grade 1 has three blades (`e₁,e₂,e₃`),
  grade 2 has three (`e₁₂,e₁₃,e₂₃`). Does `𝒢ₖ(s)` place `s` in *every* blade of
  grade k, in the *first*, or take a component vector? SPEC-0002 must decide;
  the AC2 phrasing ("blades of grade k carry s") currently implies all — to be
  confirmed, as a component-wise lift may be cleaner.
- **Norm definition under complex coefficients.** `|v|` for AC5 must be defined
  for complex coefficients (e.g. `√⟨M ∗ ~M⟩₀` vs a Euclidean coefficient norm).
  SPEC-0002 fixes the definition; the rotor test uses real-coefficient rotors
  so the two agree, but the general definition must be stated.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-28 | First GA is **G(3,0,0)**, dense 8-coefficient representation, complex coefficients reusing R-0001's `Value`. | 3D Euclidean is the proposal's worked example and the smallest algebra exhibiting all grades; dense + reference-first matches R-0001's stance; reusing `Value` makes "EML scalars below GA" literal. |
| 2026-05-28 | Rotor sandwich `R ∗ v ∗ ~R` is a *derived* construction (AC5), not a new atom. | Keeps the atom set at `𝒢ₖ` + `∗` (+ reverse `~` as a structural operation); rotation is a theorem, not a primitive — consistent with the EML stance of deriving rather than baking in. |
| 2026-05-28 | General multivector `exp` deferred; R-0002 only needs a rotor assembled from `𝒢₀`/`𝒢₂` components to rotate correctly. | Bounds scope to the two atoms; the exponential map is a substantial sub-topic better handled once its consumers (neural layer) exist. |

## Changelog

- 2026-05-28 — created (Draft).
