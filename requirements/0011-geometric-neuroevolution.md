# R-0011 — Geometric Neuroevolution (evolve the `GeoExpr` genotype)

- **Status:** Draft (2026-06-21 — owner chose the full-headline scope; discuss phase)
- **Milestone:** M5 — Discovery → Geometric Neuroevolution (**the headline**)
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-21
- **Pillar / atom:** the **evolver** — search over geometric ASTs. Pillar 2
  (geometric forms) driven by the Pillar-3/discovery engine; the one thing the
  CliffordNet / Haynes / GATr family does *not* do (all gradient-trained).
- **Depends on:** **R-0010** (`ufl-geo` — the `GeoExpr` genotype + the grade-type
  system that feeds the grade-entropy fitness term), **R-0009** (`ufl-ga` — the
  `eval` target), **R-0008** (`ufl-discovery` — the proposer-agnostic /
  verifier-exact seam + *Verifier-Held Transparency*). **Possible added
  dependency:** an R-0010 form-set extension (motors/translations) if Gate 2's IK
  task needs them — see §5 Q1.
- **Realized by:** SPEC-0011 (pending — three-lens, then test plan)
- **QA:** `qa` agent run scoped to R-0011
- **Source:** the M5 headline (ROADMAP §M5) + the two de-risk experiments
  (sandwich-structure evolution **6/6**; the IK headline **~0.07%** params vs an
  MLP, exact, tractable search).

## 1. Statement

R-0011 builds the **evolver**: a search over the **`GeoExpr` AST** (R-0010's
genotype) that *discovers* geometric programs solving a target, through R-0008's
**proposer-agnostic / verifier-exact seam**. It is the payoff the whole M5 stack
was built for — `ufl-tensor`/`ufl-discovery` validated the engine, `ufl-ga` gave
the kernel, `ufl-geo` gave the typed genotype; R-0011 makes them *evolve*.

Three coupled pieces:

1. **The genotype + operators.** `GeoExpr` (R-0010) is the chromosome. R-0011 adds
   **mutation and crossover closed over the tree** — point mutation (swap a
   node/leaf), subtree replacement, `Param` perturbation, subtree crossover —
   each producing a valid `GeoExpr`, and **grade-aware**: candidates are filtered
   / down-weighted by R-0010's `typecheck`/`grade` (a grade-incoherent tree is
   pruned *without evaluating* — the decidable signal R-0010 built for exactly
   this).
2. **The fitness.** `fitness = accuracy − λₚ·parsimony − λ_g·grade-entropy`:
   task error (the verifier), minus a size penalty, minus a **grade-entropy** term
   computed from `grade()` (rewards low-grade, grade-coherent solutions). This
   keeps *Verifier-Held Transparency*: the proposer never sees the target; the
   verifier scores. It **generalizes the R-0008 seam from a boolean discharge to a
   real-valued fitness** (§5 Q2).
3. **The two gates.** A **validation gate** (rediscover the rotor sandwich) and the
   **headline** (an evolved geometric program beats an MLP on inverse kinematics
   with a large parameter reduction).

## 2. Rationale

The de-risk retired the two risks that matter, and they point the same way: the
**geometric domain is evolution-friendly** in a way matmul was not.

- **Sandwich-structure evolution: 6/6.** A tree-GA rediscovered the rotor sandwich
  `R x R̃` in 6 of 6 runs — versus blind-GA **Strassen 0/10** (R-0008's honest
  falsification). The geometric domain gives a *dense* fitness landscape and
  *compositional* structure; matmul gave a needle-in-a-haystack. This is why
  R-0011 is the geometric headline and *not* a second matmul attempt.
- **The IK headline: ~0.07%.** A small evolved geometric program (≈3 `Param`s)
  solved an inverse-kinematics task **exactly**, where an MLP needed ~4547
  parameters to approximate it — a ~100×+ parameter reduction, with a *tractable*
  search. Geometry encodes the rigid-body structure the MLP must learn from
  scratch.

R-0011 turns both into standing, in-codebase acceptance gates (§3), under the
R-0008 honesty discipline: a **fair** MLP baseline, disclosed seeds, and a
documented negative result if a gate fails (no cherry-picking).

## 3. Acceptance criteria

- **AC1 — The evolver over `GeoExpr`, on the seam.** An evolutionary loop reaches
  candidates only through a proposer interface (`seed`/`vary` over `GeoExpr`) and
  scores them only through the verifier/fitness — the R-0008 proposer-agnostic /
  verifier-exact seam, extended to a real-valued fitness. Deterministic: a seeded
  PRNG (`SplitMix64`) makes every run reproducible.
- **AC2 — Closed, grade-aware genetic operators.** Mutation (point / subtree /
  `Param`-perturb) and crossover (subtree swap) **always produce a valid
  `GeoExpr`**, and the evolver uses R-0010's `typecheck`/`grade` to **prune or
  down-weight grade-incoherent candidates without evaluating them** (the
  decidable pruning signal). Unit-tested: operators preserve well-formedness;
  an incoherent child is rejected by the grade filter.
- **AC3 — The fitness function.** `fitness = accuracy − λₚ·parsimony −
  λ_g·grade-entropy` is implemented and unit-tested: accuracy from the task
  verifier, parsimony from tree size, **grade-entropy from `grade()`** (an
  evolved program with a tighter grade set scores better). The weights are
  configurable.
- **AC4 — Gate 1: rediscover the rotor sandwich (validation).** From a seeded,
  *answer-blind* population, the evolver **rediscovers the sandwich structure
  `R x R̃`** that realizes the `e₁ → e₂` rotation within ε — converging on a
  `Sandwich(Exp(…), …)`-shaped `GeoExpr` — in **≥ a robust fraction of independent
  runs** (the de-risk's 6/6 is the reference; the exact threshold is set with the
  qa agent). Tied to the R-0009/R-0010 keystone.
- **AC5 — Gate 2: the IK headline (beat the MLP).** On a defined inverse-kinematics
  task, an **evolved `GeoExpr` solves it to within tolerance using a small
  parameter budget, beating an MLP baseline** — *fairly trained on the same task*
  — by a **large parameter margin** (target: the de-risked ~100×+ reduction /
  ~0.07%). The MLP baseline's parameter count and error are reported alongside the
  evolved program's, honestly.
- **AC6 — Honest reporting / falsification.** Results follow the R-0008 discipline:
  the MLP baseline is a real, fairly-trained comparison (not a strawman); seeds,
  run counts, and success rates are disclosed; and **if either gate fails to meet
  its bar, the negative result is documented** rather than hidden. A documented
  honest negative still *satisfies* AC6 (the gate verdict is the deliverable).

## 4. Constraints & non-goals

**Constraints**
- **Genotype = `GeoExpr`** (R-0010), evaluated on the `ufl-ga` `Cl(3,0,1)` kernel.
  Grade reasoning = R-0010's `grade`/`typecheck` (the soundness gate
  `realized(eval) ⊆ grade` is the invariant the grade-entropy term relies on).
- **Proposer = an enhanced tree-GA** (mutation/crossover/grade-aware selection) —
  the de-risk's validated approach.
- Verifier-Held Transparency is preserved: the proposer is answer-blind.

**Non-goals (later requirements)**
- **The agentic / memetic "GA-VisAgent" proposer** — a heavier proposer behind the
  same seam; **deferred** (a later requirement) unless the tree-GA stalls on Gate 2.
- **The Strassen / matmul attempt.** The relocated matmul prize stays a
  *documented standing challenge*; R-0011 does **not** re-open the matmul lane
  blind-GA falsified in R-0008. It belongs to the future agentic-proposer
  requirement, on the discrete/tensor genotype, not this geometric one.
- **New geometric-kernel work** beyond what Gate 2's IK task minimally needs (see
  §5 Q1).

## 5. Open questions (SPEC-0011 decides)

- **Q1 — The motor/translation question (the load-bearing one).** R-0010's
  `GeoExpr` is over rotors/products/grades/`sandwich`/`exp` and **deferred
  Motor/`Point`/translation forms** (R-0010 §4 / decision log). Gate 1 needs only
  the current forms; **Gate 2's inverse kinematics is rigid-body and likely needs
  translations/motors.** The spec must choose: **(a)** extend `GeoExpr` with
  Motor/translation forms (a small R-0010 form-set extension — adds a dependency),
  or **(b)** pick an IK formulation expressible with the current rotor/sandwich
  forms (e.g. an orientation/planar reduction). This decides R-0011's true blast
  radius.
- **Q2 — Crate + the seam generalization.** New `ufl-evolve` crate vs generalizing
  `ufl-discovery` (whose proposer is typed to the matmul `Genome` and whose accept
  step is a **boolean** `Predicate::discharge`; R-0011 needs a **generic genome +
  real-valued fitness**). Likely a new `ufl-evolve` crate (→ `ufl-geo`) that
  reuses the *seam pattern*, not the matmul types.
- **Q3 — The IK task definition.** DOF, planar vs 3D, exact vs tolerance, and the
  param budget the headline claims — pinned with the qa agent so AC5 is testable.
- **Q4 — The MLP baseline.** A pure-Rust hand-rolled MLP (fairly trained,
  reproducible) vs a recorded external reference; how "fairly trained" is defined
  so the comparison is honest (AC6).
- **Q5 — Fitness + operators.** `λₚ`, `λ_g`; the operator set and the
  grade-aware pruning policy (hard reject vs soft down-weight).

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-21 | **Scoped to the full geometric headline** — Gate 1 (rediscover `R x R̃`) **and** Gate 2 (evolved `GeoExpr` beats an MLP on IK with a large param reduction). | Owner's call (over gate-first and +Strassen). Both halves are de-risked (6/6; ~0.07%), and "the headline" is the point of M5 — gate-only would undersell what's already retired. |
| 2026-06-21 | **Proposer = enhanced tree-GA; the agentic GA-VisAgent proposer is deferred.** | The de-risk's 6/6 used a tree-GA; the agentic proposer is a heavier build to pull forward only if the tree-GA stalls on Gate 2 (a later requirement). |
| 2026-06-21 | **The Strassen / matmul attempt is NOT in R-0011 scope** (stays the documented relocated prize). | R-0008 falsified blind-GA matmul; the geometric domain is the evolution-friendly one (6/6 vs 0/10). Re-opening matmul belongs to the future agentic-proposer requirement, not the geometric headline. |
| 2026-06-21 | **The seam generalizes from boolean discharge (R-0008) to a real-valued fitness**; genotype = `GeoExpr`; grade-type system feeds grade-entropy + candidate pruning. | A discovery target with a soft, dense landscape (IK) needs a graded fitness, not a yes/no discharge — and the grade-type system R-0010 built is the parsimony/pruning signal it was designed to feed. |
| 2026-06-21 | **Flagged Q1 (motor forms) as the spec's load-bearing decision** — Gate 2 may require extending R-0010's form set. | IK is rigid-body; R-0010 deferred motors. The spec must either extend the genotype or pick a current-forms IK formulation — this sets R-0011's real scope. Recorded so it is decided, not discovered mid-build. |

## Changelog

- 2026-06-21 — created (Draft); scope set to the full geometric headline by the
  owner; the de-risk results, the two gates, and the open questions (esp. the
  motor-forms dependency + the seam generalization) recorded for SPEC-0011.
