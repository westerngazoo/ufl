# R-0008 — Discovery Engine (GA search for matmul decompositions)

- **Status:** Accepted (2026-06-12 — engine-validation step of the neuroevolution program)
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

**Scope (re-scoped 2026-06-12):** R-0008 builds the engine and proves the
*loop + proposer-agnostic seam* on a **planted solvable target** (which a blind
proposer recovers, 8/10), and **runs + documents** the matmul experiment whose
plateau is the honest motivation for a stronger proposer. The Phase-1 prize —
finding an exact rank-7 decomposition of `T_2` without being given Strassen —
**relocates to R-0011**, where the proposer is upgraded (memetic / agentic). The
empirical de-risk ([`papers-review.md`](../ufl-discovery/papers-review.md) §4a)
showed blind discrete search cannot clear it; the proposer-agnostic seam makes
the upgrade a proposer swap with the verifier untouched.

## 1a. Role in the neuroevolution program (2026-06-12 reframe)

UFL's Phase-1 centre of gravity is **neuroevolution of `Cl(3,0,1)` geometric
ASTs** (see [[project-neuroevolution-direction]]; the follow-on chain R-0009 PGA
kernel → R-0010 geometric forms + grade inference → R-0011 neuroevolution). The
literature (CliffordNet, GATr, the Haynes Program-Hypergraph series) validates
GA-as-neural-primitive and grade-typed hypergraphs but is **gradient-trained, not
evolved** — neuroevolution is UFL's unproven edge.

R-0008 is the **engine-validation step**: prove the search +
predicate-discharge-fitness loop on a *known-answer* problem (Strassen) before
the genotype generalizes to geometric ASTs in R-0011.

**Forward seam — proposer-agnostic, verifier-exact** (a breadcrumb, not a
speculative abstraction; rationale in [`papers-review.md`](../ufl-discovery/papers-review.md)
§4). The viability analysis of the evidence base concluded that blind genetic
search likely will not scale past toy problems (the reason AlphaTensor used
learned guidance), so the engine must not hard-wire *blind GA* as the only
candidate source. SPEC-0008 names a boundary where:

- the **candidate source** (blind genetic operators here) is one implementation
  behind an interface — so R-0011 can add the geometric-AST genotype *and* an
  agentic proposer (the GA-VisAgent pattern) as new sources;
- **acceptance is always `Predicate::discharge`** — transparency lives in the
  *verifier*, not the proposer. A blind GA, an LLM agent, or anything may
  *propose*; only an exact discharge may *accept*.

SPEC-0008 only *builds* the blind-GA proposer (no premature abstraction) — it
just names the seam so the differentiator ("verified geometric-program discovery,
proposer-agnostic") survives into R-0011.

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

*(Re-scoped 2026-06-12 after the empirical de-risk —
[`papers-review.md`](../ufl-discovery/papers-review.md) §4a — proved blind GA
cannot rediscover Strassen (0/10) but **can** recover a planted solvable target
(8/10). R-0008 now validates the **loop + seam** on the solvable instance and
**documents the matmul falsification**; rediscovering Strassen relocates to
R-0011's stronger proposer.)*

- **AC1 — Deterministic, seeded search.** Same seed + configuration ⇒ identical
  run (trajectory and outcome). Statistical acceptance is over a
  **pre-registered seed set (0..=9, not curated)**.
- **AC2 — Fitness is the verifier's own arithmetic.** Graded fitness is the
  exact integer residual `‖reconstruct(scheme) − T_n‖²` from the *same cached
  computation* as the discharge (`residual()`, SPEC-0007 §4); acceptance is
  `discharge == Ok(true)`. Provably the same computation — no parallel check.
- **AC3 — Loop validation on a solvable known-answer instance.** For a
  **planted** target (the sum of `K = 5` fixed `{-1,0,+1}` triples), the engine
  finds an exact decomposition (residual 0) at search rank 5 for **≥ 6 of seeds
  0..=9** within the pre-registered budget. (Evidence-based: measured 8/10. This
  exercises the whole loop — propose → residual → discharge → certificate —
  on a problem a blind proposer provably solves.)
- **AC4 — Blind-proposer falsification, with a working-engine guard.** The
  engine is run on the matmul target `T_2` at ranks 7 and 8 over seeds 0..=9;
  outcomes and best-residual trajectories are recorded in a `ufl-discovery/`
  writeup. The honest result (the residual plateau — or, if blind GA surprises
  us, a discovery) is documented **either way**. To prove the engine *functions*
  (so "it plateaued" cannot launder a broken engine), the recorded rank-7
  trajectory must, for every seed, show an **initial strict decrease**
  (final-generation best `<` seed-population best) **and** terminate `> 0` — the
  descend-then-stall signature (papers-review §4b). A no-op engine fails. This is
  a falsifiable *experiment*, not a guaranteed negative; its diagnosed plateau is
  the empirical motivation for R-0011's stronger proposer. Strassen's scheme
  appears **only in tests**, never in the engine path.
- **AC5 — Certificates.** Every discovery emits its scheme, re-discharged
  through a **freshly constructed** `RankDecomposition` to `Ok(true)` — "here is
  the scheme, check it", never "trust me".
- **AC6 — Diagnostics.** An exhausted run reports its per-generation
  best-residual trajectory; with elitism ≥ 1 it is monotone non-increasing, and
  no genome ever truncates (so the trajectory reflects *landscape*, not engine
  bugs) — making the matmul plateau (AC4) a trustworthy diagnostic of
  landscape-vs-operators.

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
| 2026-06-12 | **Re-scoped to loop-validation + documented falsification** after the empirical de-risk; Strassen rediscovery → R-0011. ACs rewritten (AC3 planted target ≥6/10; AC4 documented matmul experiment). | Three independent blind methods fail exact 2×2 matmul (papers-review §4a, 0/10 rank-7), but blind GA recovers a planted target 8/10. The honest R-0008 validates what a blind proposer *can* + diagnoses the wall; forcing Strassen in would require a real solver/agent, collapsing R-0011 into R-0008. Owner-approved. |

## Changelog

- 2026-06-08 — created (Draft).
