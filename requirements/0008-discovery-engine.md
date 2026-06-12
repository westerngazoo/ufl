# R-0008 — Discovery Engine (GA search for matmul decompositions)

- **Status:** Draft
- **Milestone:** M5 — Discovery
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-08
- **Pillar / atom:** The search half of the discovery program — PRD Phases 0–1.
  R-0006 built the exact verifier; R-0007 made it the Hehner discharge; R-0008
  is the **searcher** whose accept step is that discharge.
- **Depends on:** R-0007 (`RankDecomposition` — the accept step; its cached
  target is also the graded-fitness seam) and R-0006 (`Scheme`/`Triple`).
- **Realized by:** SPEC-0008 (pending)
- **QA:** `qa` agent run scoped to R-0008

## 1. Statement

UFL gains a **genetic search engine** over decomposition schemes: populations of
`Scheme`s (rank-`R` lists of `{-1,0,+1}` triples), evolved by mutation and
recombination, **graded by the exact integer residual** against `T_n` and
**accepted by discharging `RankDecomposition { n, R }`** — the R-0007 predicate.
A *discovery* is registered only when the discharge returns `Ok(true)`, and it
ships as a **certificate**: the scheme itself, which any third party re-verifies
by re-discharging it.

The prize (PRD Phase 1): starting from random schemes, **find an exact rank-7
decomposition of `T_2` without being given Strassen**. Any valid 7-term scheme
counts — a non-Strassen one is more interesting.

## 2. Rationale

This is the experiment the whole M5 pivot exists for. AlphaTensor showed
search + exact verification rediscovers (and beats) human matmul algorithms;
UFL's claim is that its substrate carries the same loop with a *general*
verifier — the predicate discharge — rather than a bespoke one. Phases 0–1 prove
the mechanism (the PRD's explicit goal: "rediscovering Strassen is the win that
proves the mechanism; records are stretch"). The search must be transparent
(GA, no neural guidance) so failures are diagnosable — the PRD's
landscape-vs-operators falsification question is part of the deliverable.

## 3. Acceptance criteria

- **AC1 — Deterministic, seeded search.** The engine is driven by a seeded PRNG:
  the same seed + configuration produces the identical run (trajectory and
  outcome). All statistical acceptance below is over a **pre-registered seed
  set (seeds 0..=9, not curated)** — deterministic for qa, falsifiable by
  construction.
- **AC2 — Fitness is the verifier's own arithmetic.** The graded fitness of a
  candidate is the exact integer residual `‖reconstruct(scheme) − T_n‖²`
  obtained from the *same cached-target computation* as the discharge (the
  `residual()` extension licensed by SPEC-0007 §4); a candidate is *accepted*
  iff `RankDecomposition::discharge == Ok(true)`. Fitness and verifier are
  provably the same computation — no parallel ad-hoc check.
- **AC3 — Phase-0 gate (machinery runs).** For `n = 2`, rank 8: from seeded
  random initialization, the engine finds an exact scheme within a fixed
  generation budget for **≥ 9 of the 10 pre-registered seeds**. (Bounded by
  generations, not wall clock — the Structural-Frugality convention; the PRD's
  "<10 s" intent is honoured by the budget being small.)
- **AC4 — Phase-1 rediscovery (the prize).** For `n = 2`, rank 7: an exact
  scheme is found within a fixed generation budget for **≥ 3 of the 10
  pre-registered seeds** — with Strassen's scheme **nowhere in the engine's
  code path** (the fixture exists only in tests, as the expected-output oracle
  it has always been). The found scheme need not be Strassen's.
- **AC5 — Certificates.** Every registered discovery emits the scheme; the test
  re-discharges it through a **freshly constructed** `RankDecomposition` and
  gets `Ok(true)` — the "here is the scheme, check it" contract, never
  "trust me".
- **AC6 — Falsification diagnostics.** A run that exhausts its budget reports
  its best-residual trajectory (per-generation best), so a Phase-1 failure is
  diagnosable as (a) landscape navigability vs (b) operator design — the PRD's
  evolvability question. The Phase-1 outcome (success, or the diagnosed failure
  mode) is written up in `ufl-discovery/` either way.

## 4. Constraints & non-goals

**Constraints**

- Scope is `n = 2`, ranks 7–8 (PRD Phases 0–1). The genotype is fixed-rank.
- Search operators are the PRD's transparent set — entry mutation, triple-level
  crossover/recombination — sized by the spec; **no neural guidance**.
- Randomness is seeded and owned by the engine (reproducibility is AC1).

**Non-goals** (later requirements)

- **Phase 2 generalization** (3×3 / `R ≤ 23`, 4×4) — same machinery, separate
  requirement once Phase 1 lands.
- `egg` / equality-saturation quotienting; parallelism (`rayon`); learned
  policy/value guidance; record-beating as a gate.
- Option B (tensor predicates as s-exprs) — unchanged from R-0007.

## 5. Open questions (SPEC-0008 decides)

- **PRNG dependency.** A tiny in-crate deterministic generator (e.g.
  xorshift/splitmix64 — ~10 lines) vs the `rand` crate (UFL's first heavyweight
  external dep). Lean: in-crate, keeping the dependency discipline and making
  AC1's determinism trivially auditable.
- **`residual()` placement.** On `RankDecomposition` (the SPEC-0007-licensed
  extension) vs a separate fitness fn in the engine. Lean: on the predicate —
  that is what makes AC2's "same computation" claim structural.
- **CI shape.** AC3/AC4 runs must be budgeted for CI (fast seeded smoke tests
  gate the merge; the full 10-seed Phase-1 ladder may be an `--ignored`/example
  run with its results recorded). qa decides the harness split.
- **Operator minimality.** Start with mutation + crossover only and add the
  PRD's third timescale (merge/split rewrites) if the Phase-1 budget demands
  it, or spec all three upfront? Lean: minimal first, with AC6's diagnostics
  deciding.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-08 | R-0008 is PRD **Phases 0–1 only** (rank-8 sanity gate + rank-7 rediscovery at n=2); Phase 2+ is a later requirement. | The PRD's own framing: rediscovery proves the mechanism; keep the slice falsifiable and small. |
| 2026-06-08 | Acceptance statistics are over a **pre-registered seed set** (0..=9). | Deterministic for qa (no flaky tests), and immune to seed-cherry-picking — the falsifiable-experiment discipline. |
| 2026-06-08 | The accept step is the **R-0007 discharge**; graded fitness is the same cached-target residual. | The thesis: the engine rides UFL's verifier. A parallel fitness path would reopen the gap R-0007 closed. |

## Changelog

- 2026-06-08 — created (Draft).
