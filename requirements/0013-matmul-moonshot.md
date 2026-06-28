# R-0013 — The Matmul-Decomposition Moonshot (the relocated Strassen prize)

- **Status:** Draft (2026-06-28 — owner chose matmul as the moonshot domain)
- **Milestone:** M5 — Discovery (**the moonshot**)
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-28
- **Pillar / atom:** the discovery engine pointed at its hardest verifiable prize.
- **Depends on:** R-0006 (`ufl-tensor` — `Scheme`/`Triple` genome, `reconstruct`,
  `error`), R-0007 (`ufl-discovery::RankDecomposition` — the exact verifier),
  R-0008 (the proposer-agnostic / verifier-exact engine seam). **All merged.** The
  only genuinely new piece is a **stronger proposer**.
- **Realized by:** SPEC-0013 (pending)
- **QA:** `qa` agent run scoped to R-0013

## 1. Statement

**Discover a new, exactly-verified matrix-multiplication decomposition — a `Scheme`
of rank `R` below the known best for some size — by a *non-blind* proposer over the
R-0008 seam, and translate it back to readable form.** This is the **relocated
Strassen prize**: R-0008 proved blind GA *cannot* rediscover Strassen (0/10) but
the engine *can* recover a planted target (7/10), and **explicitly deferred the
Strassen rediscovery to a future requirement** — this one.

The verifier is unambiguous and already exists: `RankDecomposition::residual(scheme)
== 0 ∧ scheme.rank() == R` ⇔ an exact rank-`R` decomposition of `T_n`. The win is a
*number* (rank below known), not aesthetics — the crispest falsifiable "new math"
UFL can target.

## 2. Rationale

Everything aligns for matmul as the moonshot: an **exact verifier in hand**
(R-0007), an **unambiguous win** (rank `R` < known best), a **precedent**
(AlphaTensor found genuinely new decompositions by search), and it is the
**original UFL prize** R-0008 was built around and honestly falsified for blind GA.
The geometric lane (R-0011) *proved the machinery* (6/6 structure discovery, the
typed-AST + evolve + grade-harness); this points the same proposer-agnostic seam at
the hardest verifiable target. The **self-rewriting / Closure-Principle** ideas are
the *proposer-strength lever*, reached for if simpler proposers plateau.

## 3. Acceptance criteria

- **AC1 — A stronger (non-blind) proposer on the existing seam.** A proposer behind
  R-0008's `seed`/`vary` seam that is *not* the blind GA — and Verifier-Held
  Transparency preserved (the proposer never sees `T_n`; only the verifier scores).
- **AC2 — Gate 0 (the go/no-go): rediscover Strassen.** The proposer finds an
  **exact rank-7 decomposition of `T_2`** (the 84-coefficient `{-1,0,+1}` search
  space), verifier-certified (`residual == 0 ∧ rank == 7`), in ≥ a threshold of
  independent seeds within a stated budget — **beating blind GA's 0/10**. This is
  the load-bearing de-risk; if it fails, the moonshot is honestly dead and we say
  so. It **requires a fast-search harness** (§5).
- **AC3 — Gate 1 (the stretch): a new decomposition.** An exact decomposition at a
  size/field where the achieved rank **matches or beats the known best** — a real
  result — *or* a documented honest negative (the R-0008 ethos). A laptop-scale
  negative, honestly recorded, satisfies this AC.
- **AC4 — Honest reporting + a real harness.** The fast residual is validated
  against the canonical `RankDecomposition::residual` (every certified hit is
  re-checked by the real verifier); budgets, seeds, success rates, and the wall (if
  hit) are disclosed.

## 4. Constraints & non-goals

**Constraints**
- Reuse R-0006/0007/0008 — **do not rewrite the genome or verifier.** The coefficient
  set is `{-1,0,+1}` (R-0006); a wider field is a §5 question.

**Non-goals**
- **AlphaTensor-scale compute / deep-RL** — the moonshot is laptop-scale evolutionary
  / rewrite search; we document the wall rather than chase frontier compute.
- **The full agentic / self-rewriting proposer** — staged; reached for only if the
  simpler stronger proposers plateau on Gate 0.
- **Rewriting the verifier** — its exactness is the whole point.

## 5. Open questions (SPEC-0013 decides)

- **The proposer method (the research).** A first probe (basin-hopping coordinate
  descent over the *live* verifier) was **inconclusive — the verifier-per-eval
  harness was too slow** to complete a fair budget. So the spec needs **(a) a fast
  residual** (inline reconstruct over flat coefficients, validated against the real
  verifier), and **(b) a search method that actually works for rank-7 `T_2`** —
  candidates: discrete **ALS + rounding**, the **flip-graph** search (Kauers–
  Moosbauer), basin-hopping at scale, or an **agentic/LLM-guided** proposer. Blind
  GA and naïve coordinate descent are known/expected too weak.
- **Coefficient field** — `{-1,0,+1}` first; a wider/modular field (where AlphaTensor
  found new results) for Gate 1.
- **Gate-1 target sizes** — which `⟨n,m,p⟩` and rank bound constitutes a real new
  result.
- **Budget** — the compute/time box for Gate 0 (go/no-go) and Gate 1.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-28 | **Moonshot domain = matmul decompositions** (the Strassen prize). | Owner's call over geometric / 𝔽₂. Crispest falsifiable "new math": exact verifier in hand (R-0007), unambiguous win (rank), AlphaTensor precedent, the original UFL prize. |
| 2026-06-28 | **Reuse R-0006/0007/0008; the only new piece is a stronger proposer.** | The genome + exact verifier + proposer-agnostic seam are merged and green; R-0008 explicitly relocated the Strassen rediscovery to a future requirement (this one). |
| 2026-06-28 | **Gate 0 (rediscover Strassen) is the load-bearing de-risk / go/no-go.** | AlphaTensor's "recover the known result first" discipline. If a non-blind proposer can't find rank-7 `T_2`, the moonshot is dead — known cheaply, on a laptop. |
| 2026-06-28 | **First Gate-0 probe inconclusive → the spec must build a fast-search harness.** | The live-verifier-per-eval (Scheme rebuild + `reconstruct` per coordinate flip) timed out before a fair budget. A fast inline residual (certified against the real verifier) + a real method (ALS-rounding / flip-graph / agentic) is the actual Gate-0 work. |
| 2026-06-28 | **Honest about odds.** Rediscovering Strassen is *plausible* at laptop scale (a real result + the gate); a genuinely *new* useful decomposition is compute-hard, low base-rate — pursued and documented honestly. | Mirrors R-0008's honest falsification; the apparatus is right, the frontier is hard. |

## Changelog

- 2026-06-28 — created (Draft); moonshot domain set to matmul by the owner; Gate 0
  (Strassen rediscovery) framed as the go/no-go; first live-verifier probe
  inconclusive (harness too slow) → fast-search harness + the proposer method are
  the SPEC-0013 questions.
