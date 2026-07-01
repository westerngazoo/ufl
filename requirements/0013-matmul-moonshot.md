# R-0013 — Matmul discovery moonshot (flip-graph)

- **Status:** Draft (this branch realizes **Gate-0**, AC1/AC2)
- **Milestone:** discovery / object-level scaling
- **Depends on:** R-0006 (integer tensor core), R-0007 (tensor predicate), R-0008
  (discovery engine). *Independent of R-0014* — the flip-graph is a distinct search
  modality (a graph walk over exact schemes), not the population loop, so it does
  not reuse `run_generic`.

## Context

The discovery engine (R-0008) reaches candidates through an **answer-blind
proposer** and accepts only through the exact `RankDecomposition` verifier
(proposer-agnostic / verifier-exact). A blind GA over `{−1,0,+1}` coefficients
cannot find low-rank matmul decompositions — the coefficient landscape is densely
studded with deceptive error-1 traps (recorded in `theory/discovery-results.md`).
The **Kauers–Moosbauer flip-graph** — a walk over *exact* schemes via
tensor-preserving moves plus rank-reducing merges — is the proposer family that
can. Banking it turns a deleted de-risk pilot into reproducible, verifier-certified
code, and supplies the *second* proven proposer family the reframed R-0015
meta-search needs something non-trivial to evolve between.

## Requirement

A **flip-graph proposer** that, driven from the naive rank-`n³` scheme, reduces the
rank of the matmul tensor `T_n` via tensor-preserving moves — with **every accepted
result certified by the exact `RankDecomposition` verifier**, never by the proposer
itself.

## Acceptance criteria

- **AC1 (Gate-0 — this branch):** from the naive rank-8 scheme for `T₂`, the
  flip-graph reaches a rank-7 scheme over `{−1,0,+1}` that
  `RankDecomposition::new(2,7).discharge` certifies `Ok(true)` — **deterministically**
  (fixed seed), re-certified through a *freshly constructed* verifier and a
  **bilinear check on ≥ 20,000 random integer matrix pairs** (`C = A·B`).
  Reproduces the deleted pilot as a regression test.
  > **Honest scope:** rank-7 for ⟨2,2,2⟩ is Strassen's algorithm by **de Groote's
  > 1978 uniqueness theorem** — a certified *re-derivation*, **not** a new result.
  > The deliverable is the banked, reproducible **engine**, not the object.
- **AC2 (verifier-held):** the flip-graph **never certifies a scheme itself** —
  acceptance is solely `RankDecomposition::discharge`. A tensor-breaking move can
  only *fail to certify*, never yield a false positive. (Enforced by test: a
  corrupted scheme discharges `Ok(false)` / `Err`, never `Ok(true)`.)
- **AC3 (record attempt — DEFERRED, future spec):** the same engine aimed at `T₃` —
  reach rank ≤ 23 (parity with the known best), then probe < 23. **Falsifiable
  kill:** if it cannot re-reach 23 within a pre-registered budget, the
  object-scaling thesis is documented-dead and we fall back to hardening the
  geometric Gate-1.

## Non-goals

- **No new mathematical claim from AC1** — Strassen is known-optimal at rank 7.
- **No change to the `{−1,0,+1}` `Scheme` type.** The flip-graph uses an internal
  unrestricted-integer workspace; only a certifiable end state (back in `{−1,0,+1}`)
  becomes a `ufl_tensor::Scheme`.
- No coupling to R-0015 (the meta-loop) — this bank is a standalone object-level
  proposer.
