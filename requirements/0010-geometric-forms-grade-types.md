# R-0010 — Geometric Forms + the Grade-Type System (`ufl-geo`)

- **Status:** Accepted (2026-06-18 — owner)
- **Milestone:** M5 — Discovery → Geometric Neuroevolution
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-18
- **Pillar / atom:** Pillar 2 — the **geometric *forms*** (`𝒢ₖ`, `∗`, …) and the
  **dimensional (grade) type system**. The typed geometric expression layer that
  lowers onto the R-0009 `ufl-ga` kernel — and the **genotype R-0011 evolves**.
- **Depends on:** R-0009 (`ufl-ga` — the `Cl(3,0,1)` PGA kernel `Mv`), R-0003
  (`ufl-syntax` — the homoiconic s-expression surface).
- **Realized by:** SPEC-0010 (pending)
- **QA:** `qa` agent run scoped to R-0010

## 1. Statement

UFL gains a **typed geometric expression layer** — a new `ufl-geo` crate with
three coupled pieces:

1. **A geometric AST (`GeoExpr`)** — the geometric *forms* as a tree:
   grade-lift `𝒢ₖ`, the geometric product `∗`, wedge `∧`, inner `·`, reverse
   `~`, grade projection `⟨·⟩ₖ`, the versor **sandwich** `R x R̃`, and `exp`
   (rotor from a bivector), over leaves (basis elements, evolvable scalar
   parameters, input variables). This is the homoiconic geometric program — and
   the **genotype R-0011 mutates and recombines** (the structure the
   sandwich-discovery de-risk evolved).
2. **Evaluation** — `eval(GeoExpr, env) → ufl_ga::Mv`: lower a form tree onto
   the R-0009 kernel and compute. The geometric analogue of R-0001's `eval`.
3. **A decidable grade-type system** — `grade(GeoExpr) → GradeSet`: infer which
   grades a form's result can carry, by the standard GA grade algebra (wedge
   *adds* grades, inner *subtracts*, the geometric product *mixes*, reverse
   *preserves*, **the versor sandwich preserves grade**, grade-lift *produces*
   grade `k`). This is the "dimensional type system" the Program-Hypergraph
   work (Haynes) proves decidable — the type that lets the system *know a vector
   from a point*, and the source of R-0011's grade-parsimony fitness term.

## 2. Rationale

R-0009 gave UFL a correct geometric *kernel*; R-0010 gives it a geometric
*language* — forms that compose into programs, and a type system over them. This
is the layer the neuroevolution (R-0011) operates on:

- The **`GeoExpr` AST is the genotype.** The de-risk experiment showed blind
  evolution *discovers* geometric structure (the rotor sandwich, 6/6) when the
  forms are compositional and the fitness dense — but only if there *is* a form
  AST to evolve. R-0010 builds it.
- The **grade-type system is what makes evolved programs trustworthy and
  parsimonious.** Grade inference (a) rejects or down-weights grade-incoherent
  programs (a rotation must map a vector to a vector), and (b) feeds R-0011's
  `−γ·grade-entropy` term (parsimony rewards low-grade solutions). It is also
  UFL's answer to the Haynes "decidable-by-construction" thesis, realized over
  the geometric forms rather than asserted.

This mirrors the M1/M2 pattern: `ufl-core` is the typed `eml` kernel, `ufl-syntax`
the s-expr forms over it; `ufl-ga` is the typed PGA kernel, `ufl-geo` the
geometric forms + types over it.

## 3. Acceptance criteria

- **AC1 — The geometric AST.** `ufl-geo` exposes `GeoExpr` — a `Clone`-able,
  inspectable tree of the geometric forms (grade-lift, geo-product, wedge, inner,
  reverse, grade-project, sandwich, exp; leaves: basis element, scalar
  parameter, variable). The form set is the one R-0011's genotype evolves.
- **AC2 — Evaluation onto the kernel.** `eval(GeoExpr, env) → Mv` lowers each
  form onto its `ufl_ga` operation and computes; a form tree's value equals the
  hand-written `ufl_ga` composition (e.g. the `sandwich` form on a rotor + a
  vector equals `ufl_ga`'s sandwich, within `ε`).
- **AC3 — Grade inference (decidable, total).** `grade(GeoExpr) → GradeSet`
  computes the correct result grades for every form: grade-lift `k` → `{k}`;
  wedge adds; inner subtracts (`|gₐ−g_b|`); geo-product mixes
  (`{|gₐ−g_b|, …, gₐ+g_b}` step 2, capped at the algebra dimension); reverse
  preserves; grade-project `k` → `{k}`; sandwich preserves the operand's grade.
  Total (no panic) and decidable (finite grade sets over `0..4`).
- **AC4 — The grade-preservation keystone.** For a rotor `R` and a grade-1
  vector `v`, `grade(Sandwich(R, v)) == {1}` — *a rotated vector is still a
  vector*. Tied to the R-0009 keystone: the same sandwich that sends `e₁ → e₂`
  is grade-typed vector → vector. This is the dimensional-type invariant the
  literature touts, made checkable.
- **AC5 — Homoiconic surface.** `GeoExpr` is the **code-as-data form
  representation** (a `Clone`-able, inspectable, constructor-built tree — what
  `Eml` is before R-0003 added a reader). *The textual `Sexpr → GeoExpr` reader
  is **deferred** to a future requirement (a human-authoring consumer) — resolved
  by SPEC-0010 §2.6 + the three-lens (R-0011 evolves the `GeoExpr` AST directly,
  so it consumes the AST, not text). See the decision log.*
- **AC6 — Grade-coherence check.** A grade-incoherent program is detectable:
  `grade` flags (or a `typecheck` returns an error for) a form that cannot carry
  a sensible grade (e.g. grade-project to an absent grade) — the decidable
  "type error" R-0011 uses to prune or down-weight candidates.

## 4. Constraints & non-goals

**Constraints**
- The kernel is **R-0009's `ufl-ga` `Cl(3,0,1)` PGA over real `f64`**; grade sets
  range over `0..4` (16-blade algebra). Eval correctness is to `ε` (the R-0009
  tolerance); grade inference is exact/structural.
- `ufl-geo` depends only on `ufl-ga` (not `ufl-core`; the textual reader that
  would pull in `ufl-syntax` is deferred — see AC5 / the decision log).

**Non-goals** (later requirements)
- **The evolution itself** — the genotype operators (mutation/crossover over
  `GeoExpr`), the fitness (accuracy − parsimony − grade-entropy), the search —
  are **R-0011**. R-0010 builds the *thing evolved* and the *type that scores
  it*, not the evolver.
- **Motors / translations as forms** (the typed `Point`/`Motor` path) — the
  R-0010 form set is over `Mv` (rotations/products/grades); rigid-body motor
  forms are a later add when R-0011's IK target needs them.
- A full reject-on-every-mismatch type *checker* beyond grade coherence (AC6) —
  the grade *inference* is the deliverable; a richer checker can follow.

## 5. Open questions (SPEC-0010 decides)

- **Grade-typing depth.** Grade inference as a *computed `GradeSet` property*
  (recommended — feeds typing + fitness, decidable, tractable) vs. a full
  reject-on-mismatch *type checker*. Lean: the inference + a coherence check
  (AC6), not a full checker.
- **The s-expr surface now vs deferred (AC5).** Homoiconicity is the thesis, so
  a minimal reader for the geometric forms is attractive — but R-0011 evolves the
  `GeoExpr` AST directly (not via text), so the textual surface may be premature
  until a human wants to *write* geometric programs. Decide with the three-lens.
- **The form set / leaf representation for R-0011.** Scalar parameters as
  evolvable leaves (the de-risk's `Q` rotor was a parameter); confirm the leaf +
  form set is exactly what R-0011's mutation/crossover needs (and no more).
- **Crate vs. extension.** A new `ufl-geo` crate (recommended, parallel to
  `ufl-predicate`) vs. extending `ufl-syntax`/`ufl-ga`.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-18 | R-0010 = the **typed geometric expression layer** (forms + eval + grade inference), the genotype R-0011 evolves — *not* the evolution itself. | Keeps R-0010 a coherent, testable unit (a typed core), and separates "the thing evolved + its type" (R-0010) from "the evolver" (R-0011), per the proposer-agnostic discipline. |
| 2026-06-18 | The **grade-type system** is built here, as decidable grade *inference* over `0..4`, with the **sandwich grade-preservation** keystone (AC4). | This is UFL's realization of the Haynes "decidable dimensional type" — and the source of R-0011's grade-parsimony fitness. The sandwich-preserves-grade invariant is the smallest convincing proof, tied to the R-0009 keystone. |
| 2026-06-18 | Form set scoped to the **`Mv`-over-rotations core** (grade-lift, products, reverse, project, sandwich, exp); motors/`Point` deferred. | The de-risk validated structure-evolution on exactly this core; rigid-body motor forms add surface R-0011's first gate (rediscover `R x R̃`) doesn't need. |
| 2026-06-18 | **AC5 amended: the textual `Sexpr → GeoExpr` reader is deferred**; AC5 delivers `GeoExpr` as the homoiconic AST, not a textual reader. The §4 dependency is narrowed to **`ufl-ga`-only** accordingly. | Three-lens (SPEC-0010 §2.6): R-0011 evolves the AST directly (no text consumer yet); a reader now is premature (CLAUDE.md §2), and mirrors `Eml` (R-0001) preceding its reader (R-0003). Recorded here so the spec is not silently redefining an accepted AC (the SPEC-0007/0009 discipline). The reader returns when a human-authoring use case appears, re-adding the `ufl-syntax` (dev-)dependency. |

## Changelog

- 2026-06-18 — created (Draft).
