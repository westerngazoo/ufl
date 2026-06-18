# R-0009 — `Cl(3,0,1)` PGA Kernel Binding (`ufl-ga`)

- **Status:** Accepted (2026-06-12 — owner)
- **Milestone:** M5 — Discovery → Geometric Neuroevolution
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-12
- **Pillar / atom:** Pillar 2 — the **Geometric Algebra spatial layer**. The
  typed GA kernel UFL's geometric s-expression *forms* (R-0010) lower into, just
  as `ufl-core`'s `Eml` is the kernel the `eml` form lowers into.
- **Depends on:** **garust `v0.1.0`** (the confirmed capability contract —
  [`docs/garust-pga-contract.md`](../docs/garust-pga-contract.md), C1–C11, CI-guarded
  by garust's `pga_contract.rs`). No other UFL crate.
- **Supersedes:** [R-0002](0002-geometric-algebra-core.md) (G(3,0,0) VGA) — see §6.
- **Realized by:** SPEC-0009 (pending)
- **QA:** `qa` agent run scoped to R-0009

## 1. Statement

UFL gains a **geometric-algebra substrate**: a new `ufl-ga` crate that binds
garust's `Cl(3,0,1)` **Projective Geometric Algebra** kernel (real `f64`,
`Pga3`) and exposes a focused, UFL-facing surface of the operations the
geometric forms (R-0010) and neuroevolution (R-0011) build on — multivector
construction and grades, the geometric / outer / inner products, grade
projection, reverse, the versor **sandwich**, `exp` (rotors), and the
rigid-body **Motor** (rotor + translator). The binding is validated for
**geometric correctness** against known transforms.

`ufl-ga` is a **substrate leaf** — parallel to `ufl-tensor`, depending only on
garust. It does **not** depend on `ufl-core`: the analytic EML core
(`Complex<f64>`) and this spatial PGA kernel (real `f64`) are *separate
substrates* serving different roles; the log–GA bridge connects them only at the
theory level, not in code at this layer.

## 2. Rationale

The geometric-neuroevolution arc (M5 Phase 1) needs a GA kernel for its genotype
(geometric ASTs) and fitness (geometric correctness). garust provides a
complete, tested `Cl(3,0,1)` PGA kernel, now pinned at a guaranteed surface
(`v0.1.0`). R-0009 is the **binding**: a thin UFL-facing wrapper that

1. **insulates UFL** from garust's full API and any future churn (UFL's forms
   and evolution target the stable wrapper, not garust internals directly);
2. **validates geometric correctness** end-to-end through that wrapper, so
   R-0010/R-0011 build on a surface UFL has itself tested; and
3. **commits the signature**: `Cl(3,0,1)` (not the G(3,0,0) of R-0002) — the
   ideal/null generator `e₀² = 0` is what makes translations and rigid-body
   *motors* native, which the eventual inverse-kinematics target requires.

This mirrors the M1 pattern: `ufl-core` is the typed `eml` kernel; `ufl-ga` is
the typed PGA kernel — both are substrate the s-expression forms lower into.

## 3. Acceptance criteria

*(The kernel is real `f64`, so geometric-correctness assertions hold to a stated
floating tolerance `ε` — UFL's first GA-layer floating gate, consistent with the
`Complex<f64>` EML core. SPEC-0009 fixes `ε`.)*

- **AC1 — The crate & the pinned dependency.** `crates/ufl-ga` exists, depends on
  garust pinned at `v0.1.0` (git tag), and exposes a focused UFL-facing PGA
  surface (a multivector value type + the op set of AC2–AC6). It builds; it does
  **not** depend on `ufl-core`.
- **AC2 — Construction & grades.** A multivector can be constructed; grade
  projection ⟨·⟩ₖ selects grade `k`; scalar / vector (grade 1) / bivector (grade
  2) / trivector / pseudoscalar are representable and distinguishable.
- **AC3 — The three products.** Geometric product, outer (wedge), and inner
  compute correctly on known inputs, including the signature's defining facts:
  `e₁·e₁ = 1`, `e₁∧e₂ = e₁₂`, orthogonal vectors' inner product `= 0`, and the
  **null property `e₀·e₀ = 0`** (the PGA degeneracy that distinguishes it from
  G(3,0,0)).
- **AC4 — Versor sandwich (the keystone).** A unit rotor `R = exp(−θ/2·B)` for a
  unit bivector `B`, applied by sandwich to a grade-1 vector, rotates it by angle
  `θ` in the `B`-plane — verified on a known rotation (e.g. a `τ/4` rotation in
  the `e₁₂` plane sends `e₁ → e₂` within `ε`). The geometric analogue of R-0006's
  Strassen keystone.
- **AC5 — Motor (rigid-body motion).** A `Motor` performs the correct rigid-body
  motion on a PGA point: a pure **translator** moves a point by the exact offset
  (translations are native via `e₀`); a **rotor** rotates it; a **composed**
  motor does both, and motor composition (`M₂ ∘ M₁`) applies in the right order —
  all within `ε`.
- **AC6 — Reverse / norm / numerical hygiene.** `reverse`, `norm`,
  `normalized` behave correctly: a normalized rotor has norm 1; for a unit
  versor, `reverse` is its inverse (`R · R̃ = 1` within `ε`); the wrapper returns
  values (no panics) on its supported domain.

## 4. Constraints & non-goals

**Constraints**
- **Real `f64` PGA only** (`Pga3`); no `Complex`, no CGA (`Cl(4,1)`), no
  `garust-physics`.
- garust pinned at the **`v0.1.0` git tag**; default build (`std` + `f64`), no
  extra garust features.
- The exposed surface is a **subset of the contract C1–C11** — exactly what
  AC2–AC6 (and R-0010's form taxonomy) need.

**Non-goals** (later requirements)
- Geometric **s-expression forms** + the decidable **grade-type system** —
  **R-0010**.
- **Neuroevolution** over geometric ASTs (the genotype, the operators, the
  fitness) — **R-0011**; the relocated Strassen prize and the *evolutionary*
  rediscovery of `R x R̃` live there.
- CGA (spheres/circles), batch/SIMD apply, the homogeneous-matrix bridge
  (`Motor::to_matrix`) — available in garust, pulled in only when a later
  requirement needs them.

## 5. Open questions (SPEC-0009 decides)

- **Binding shape.** A focused UFL-facing wrapper (recommended — insulation +
  a stable target for R-0010) vs. a thin re-export of garust types. The wrapper's
  exact surface (which methods, UFL-facing names vs. garust names) is the spec's
  call.
- **Tolerance `ε`.** The floating tolerance for AC3–AC6 (e.g. `1e-10`), and
  whether any assertion can be made exact (e.g. `e₀·e₀` may be bit-exact `0`).
- **How much of C9/C10 to surface now.** `Motor` + a PGA `Point` are needed for
  AC4/AC5; `Line`/`Plane`/join/meet (C10) may be deferred or thin-passed until
  R-0010/R-0011 need incidence.
- **Where the grade-lift `𝒢ₖ` lives.** Construction primitives belong here; the
  `𝒢ₖ` *form* is R-0010 — confirm the split.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-12 | **R-0009 supersedes R-0002** (G(3,0,0) VGA → `Cl(3,0,1)` PGA). | R-0002's signature has no ideal generator, so it cannot express translations / motors — required by the kinematics target. R-0002's hand-rolled prior art and conventions (rotor oracle) carry over as reference. |
| 2026-06-12 | **Real `f64`**, a substrate *separate* from the `Complex<f64>` EML core; `ufl-ga` does not depend on `ufl-core`. | Rotors/motors/kinematics are real spatial geometry (matching CliffordNet/GATr and the roadmap); the EML core's complex field serves the analytic/elementary-function role. Two substrates, bridged only in theory. |
| 2026-06-12 | garust pinned at **`v0.1.0`**; UFL depends on the confirmed contract surface (C1–C11), not garust internals. | The contract is green and CI-guarded by garust's `pga_contract.rs`; pinning a tag fixes the dependency and any churn is a garust major bump. |
| 2026-06-12 | Acceptance is **geometric correctness to a tolerance `ε`** (not exact), keystone = the rotor sandwich (AC4). | The kernel is floating `f64`; UFL's GA layer is therefore a tolerance gate (like the `Complex` EML core), with the rotor sandwich as the smallest convincing correctness proof. |
| 2026-06-12 | **"Insulate" (§2.1, §5) reframed** to *a curated, pinned import surface* — **not** a type that re-wraps every garust operation. | Three-lens (SPEC-0009): a transparent alias gives zero API insulation; insulation against garust churn is held by the **version pin** (a `v0.1.0` tag → a locked commit rev), not a wrapping newtype. The newtype that would insulate is the deferred escalation (a second backend / a UFL invariant). Recorded here, not silently reinterpreted by the spec (CLAUDE.md §1). |

## Changelog

- 2026-06-12 — created (Draft).
