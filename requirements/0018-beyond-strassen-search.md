# R-0018 — Smarter search for a certified beyond-⟨2,2,2⟩ matmul reduction

- **Status:** **Draft** — discuss phase (acceptance criteria proposed below, to be
  decided together per CLAUDE.md §1.2/§4.1).
- **Milestone:** M5 object-level discovery — the honest, laptop-scale "redo
  AlphaTensor from our perspective."
- **Depends on:** R-0013 / SPEC-0013 (the flip-graph — proven to certify Strassen
  for ⟨2,2,2⟩), R-0014 (the exact `RankDecomposition` verifier).

## 1. Why (the honest framing)

The 2026-07-24 measurement settled what our engine can and cannot do. It **works** —
naive-start flip search certifies Strassen rank-7 for ⟨2,2,2⟩ (2/8 seeds at 1M
flips, exact-verified). It **does not scale by brute force** — on ⟨3,3,3⟩ it reaches
rank **27 (= naive, zero reduction)** at 1M flips, every seed. The record papers
(Kauers–Moosbauer) reach ⟨3,3,3⟩ records with **billions** of flips, GPU clusters,
**start-from-known-schemes**, and symmetry — a compute/strategy gap, not a
correctness gap. Out-computing them is not the goal and is not possible here.

**What *is* achievable and genuinely novel from our perspective:** the
correctness-first method — every candidate tensor-exact **by construction**, checked
by an **exact verifier the proposer cannot reach** — demonstrated on a matmul
reduction **beyond the ⟨2,2,2⟩ special case**. Two things stand between us and that:
(a) the engine is **square-only** (`naive(n)`/`target_int(n)` assume ⟨n,n,n⟩;
`IntScheme` carries one `dim`), and (b) the search is **naive-start random-flip** —
the weakest strategy, not the maintain-and-perturb-good-schemes strategy the record
papers use.

## 2. Scope

1. **Rectangular ⟨m,n,p⟩ support** — generalize the flip-graph to per-slot
   dimensions (`d_u = m·n`, `d_v = n·p`, `d_w = m·p`). The flip primitives
   (`shared_factor_pairs`/`flip_at`/`reduce`/`perturb`) are already dim-agnostic
   (verified: only `reconstruct_int`/`target_int` read the single `dim`), so this is
   the tensor/reconstruct layer, not the moves.
2. **A smarter search strategy** — the record papers' shape: maintain the
   best-so-far scheme(s), perturb, local-search back, keep on improvement; optional
   start-from-a-known-scheme and ⟨m,n,p⟩ symmetry. The verifier stays sole
   acceptance authority (VHT preserved).

## 3. Proposed acceptance criteria (TO BE DECIDED TOGETHER)

- **AC1 — Rectangular correctness (the enabling infra).** For ⟨m,n,p⟩ ∈
  {⟨2,2,2⟩, ⟨2,2,3⟩, ⟨2,3,3⟩}, the naive rank-`mnp` scheme reconstructs to the exact
  ⟨m,n,p⟩ tensor, every flip preserves it, and `reduce` never raises rank — the
  SPEC-0013 invariants, re-proven rectangular. ⟨2,2,2⟩ stays byte-identical to today.

- **AC2 — The falsifiable gate: a certified beyond-Strassen reduction.** On
  **⟨2,2,3⟩** (naive 12; **known optimal 11**, Hopcroft–Kerr 1971), the search
  returns a **rank-11 ternary scheme certified by the exact verifier** within a
  laptop-minute — a *single* reduction, analogous to ⟨2,2,2⟩'s 8→7 which the engine
  already does. **This is the make-or-break.** *(Open for the spec: whether a
  {−1,0,1} rank-11 scheme exists — Hopcroft–Kerr coefficients are small integers; if
  no ternary rank-11 exists, the gate re-scopes to the smallest ternary-reachable
  reduction and says so.)*

- **AC3 — The escalation ladder is pre-registered, and every rung is a result.**
  (a) naive-start reaches rank-11 → done. (b) If not, start-from-a-known ⟨2,2,3⟩
  scheme + perturb-and-recover reaches it → the *strategy* is the contribution.
  (c) If **neither** reaches it within budget → a **documented negative**: the
  correctness-first flip-graph does not scale past the ⟨2,2,2⟩ special case at
  laptop scale even with smarter search — banked in `theory/discovery-results.md`,
  a real and publishable result. No silent middle.

- **AC4 — VHT + honesty preserved.** The proposer never holds the verifier; every
  claimed reduction is discharged through `RankDecomposition`; the budget, seeds,
  and strategy are pre-registered before the run (the pre-run discipline).

## 4. Non-goals

- **Beating the ⟨3,3,3⟩ record** — measured out of laptop reach; not attempted.
- No RL/LLM-guided mutation, no GPU, no approximate/numerical schemes (exact
  `{−1,0,1}` only). No new engine — the meta-loop is not involved (Rung-4 is closed).

## 5. What a result means

- **AC2 met:** the engine found a certified matmul algorithm *beyond* the textbook
  2×2 case — a genuine object-level discovery-direction win, the correctness-first
  method validated past its first special case.
- **AC3(c) negative:** the honest limit of the approach at our scale is now
  *measured and bounded*, not guessed — which is itself the paper-grade finding the
  method-demonstrator would cite.
