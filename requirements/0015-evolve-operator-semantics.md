# R-0015 — Evolve operator semantics (the Rung-4 probe)

- **Status:** Draft (moves to Accepted via the three-lens review; no code before
  that — constitution §1)
- **Milestone:** the self-eval staircase, **Rung 4** (see the canonical ladder in
  `theory/two-language-substrate.md`)
- **Depends on:** R-0013 Gate-0 (the flip-graph move primitives), R-0014 AC1/AC2
  (the genome-generic loop), the held-out statistics harness (task 09)

## Context

The 2026-06-29 interrogation settled it at high confidence: **evolving the GA's
scalar hyperparameters is headroom-free** — the hand-tuned baseline sits at the
DSL ceiling, and every apparent winner regressed to at-or-below it on fresh
disjoint sets. The matmul lane settled the complement: **the proposer *family*
decides everything** (blind coefficient-GA dead at 25×10⁹ evals; the flip-graph's
structurally different *move* certified rank-7 in 0.16 s). Together: *if* the
metacircular win exists, it lives in **operator semantics** — the structure of the
search move — not in operator hyperparameters. This requirement is that bet, made
falsifiable.

> **The non-sequitur caveat (verbatim, load-bearing):** that a *human-designed*
> flip-graph beats a GA is **not** evidence a *meta-search* can evolve such a
> move. R-0015 has zero positive efficacy evidence — only the hyperparameter
> negative. This probe exists to produce that evidence or the documented negative.

## Requirement

An outer search loop — a **second instance of the R-0014 `run_generic`**, no new
engine — whose genome is a **move-form**: a term in a *bounded, typed* DSL whose
primitives are the committed R-0013 flip-graph moves (`shared_factor_pairs`,
`flip_at`, `reduce`, `perturb`) and whose interpreter turns a form into a
`Proposer`. The meta-fitness is *how well the inner search using that move-form
hits the exact verifier's targets*, measured **only** on held-out tasks.

## Acceptance criteria

- **AC1 (the move-form DSL):** a closed, depth/size-capped grammar over the
  R-0013 primitives (sequence, choice, parameterized kick/walk composites — the
  spec fixes the grammar). The hand-written Gate-0 driver policy is expressible
  as one point `B0` in the DSL, so the comparison is within-harness fair.
- **AC2 (the pre-registered gate, verbatim from the substrate doc):** the evolved
  move-form beats the hand-written baseline by **≥ 2 SE on a disjoint
  confirmation set, replicated on a second split, at a budget where the baseline
  is demonstrably not saturating**. Margins, budgets, seed policy, and both
  splits are pre-registered in SPEC-0015 **before** the meta-search runs.
- **AC3 (the kill-criterion):** a documented negative **kills Rung-5 permanently**
  (no Lisp substrate) and redirects effort to object-level scaling (R-0013 AC3's
  `T₃` record attempt) plus the reflection line (R-0016), which stands on its own
  merits either way. Either outcome closes the gate; both are results.

## The seven non-negotiables (C1–C7, transcribed from the substrate doc)

1. **C1** — the verifier and any monitor are **unreachable from the proposer**
   (Rust-side; the meta-loop optimizes a number it cannot compute or corrupt).
2. **C2** — the meta-objective is **held-out tasks the proposer never searches
   on** (train/holdout split; scoring exclusively on holdout).
3. **C3** — reward = the **exact verifier verdict**, never a learned proxy.
4. **C4** — the operator space is **bounded** (typed grammar, depth/size caps).
5. **C5** — verification stays **cheap relative to search**.
6. **C6** — improvement = a **measured delta on the held-out set**, nothing else.
7. **C7** — **traceable lineage + exact replay** (deterministic seeds; every
   accepted form reproducible).

## Non-goals

- **No Lisp/Scheme substrate in this requirement** — Rung-5 is *earned* only by
  this probe's positive (the substrate doc's non-sequitur discipline).
- **No hyperparameter re-tuning** — that space is measured dead; the DSL is over
  move *structure*.
- **No self-grading** — nothing in the loop scores itself (C1/C3).
- **No claim of "recursive self-improvement"** from a single positive: AC2's
  pass earns exactly the statement "an evolved move-form beat the hand-written
  one on held-out tasks," no more.
