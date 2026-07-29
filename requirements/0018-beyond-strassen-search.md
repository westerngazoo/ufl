# R-0018 — Smarter search for a certified beyond-⟨2,2,2⟩ matmul reduction

- **Status:** **Accepted** (ACs confirmed by Gustavo 2026-07-24; de-risk banked
  §3b). Realized by SPEC-0018.
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

## 3b. De-risk findings (2026-07-24, measured before spec — the discipline)

A throwaway rectangular pilot measured the gate *before* speccing an engine:

- **⟨2,2,3⟩ naive-start random-flip (eager-reduce), 300k flips, 12 seeds → 0/12.**
  Best rank stays **12** (= naive); zero reduction. Same as ⟨3,3,3⟩ (rank 27, 0
  progress at 1M).
- **⟨2,2,3⟩ greedy steepest-descent-on-rank, 4000 steps, 20 seeds → 0/20**, and —
  the diagnostic — **no single flip from naive ever lowers the rank**. The 12→11
  reduction is therefore **behind a plateau**: it needs a long *rank-preserving*
  flip walk before a reduction flip appears. Greedy can't climb a plateau; random
  doesn't wander far enough.
- **Diagnosis / what the real build needs.** This matches the Kauers–Moosbauer
  method exactly: they cross these plateaus with **10⁸–10⁹ flips** and a
  **fixed-rank walk** (accept rank-equal flips; do *not* eager-reduce; a reduction
  is a rare event along the plateau). Our pilot used ~10⁵ flips with eager reduce —
  **~1000× too few and the wrong walk structure.** So AC3 rung (a) [naive random /
  greedy] is **measured-refuted**; the real attempt is the fixed-rank plateau walk
  at a ≥10⁸-flip budget (which is laptop-feasible at ~µs/flip ≈ minutes for one
  target — the open question is whether the *plateau is crossable at all* at that
  scale, not whether the compute exists).

**Consequence for the spec:** ~~SPEC-0018's search is the fixed-rank plateau walk~~
— **SUPERSEDED by §3c.**

## 3c. Correction + the WON gate (2026-07-24, measured — §3b's conclusion was wrong)

§3b's *diagnosis* (the 12→11 reduction is behind a plateau) was right; its
*conclusion* was **borrowed, not measured** — the "10⁸–10⁹ flips, fixed-rank walk"
figure is Kauers–Moosbauer's **⟨3,3,3⟩-record** scale, imported to ⟨2,2,3⟩ without
justification. That is the exact anti-pattern this project exists to prevent, and
the hater caught it. **Measured (hater + reproduced independently, seed 3):**

- The **existing, unmodified eager-reduce `reduce_matmul_with` loop** crosses ⟨2,2,3⟩
  12→11 and returns a **certified rank-11 ternary scheme** (`for_target(…,11)
  .discharge = Ok(true)`; 50k random bilinear re-derivations pass) — at **10⁶ flips**,
  ~100× *below* the borrowed claim.
- A **fixed-rank walk got 0/32** — measured *worse*, not better. AC3's rung (a) is
  **vindicated**, not refuted; §3b's "the real method is fixed-rank" is withdrawn.
- The plateau is **two-level**: (1) *reach* rank 11 (~6–12% of seeds at 10⁶; hard
  seeds cross by 10⁸), then (2) *land on a ternary state* — the rank-11 plateau is
  **~99.999% non-ternary (≈1:135,000)**, so the loop's `is_ternary()` early-stop is
  the load-bearing **terminal filter** that catches a rare ternary moment. Budget
  matters because more rank-11 samples = more ternary draws, not because of a deeper
  plateau.
- A **ternary rank-11 ⟨2,2,3⟩ demonstrably exists** (a Strassen-7 ⊕ matvec-4 block
  scheme certifies), so the gate is well-posed — the only question was reachability,
  now answered **yes**.

**AC2 is MET.** The corrected method is the eager-reduce loop over `naive_embedded`
with the `is_ternary` terminal filter; the gate is a **pinned seed** (seed 3 @ 10⁶,
**0.74 s measured** — the `r_0013` pattern), with a seed-block robustness run at 10⁸ optional. The
lesson (banked): *even a diagnosis-backed conclusion is a hypothesis until the target
itself is measured — a borrowed constant is not a measurement.*

**What AC2 does NOT establish (PR #77 architect finding, verified).** The scheme the
walk found **is** that same Strassen-7 ⊕ matvec-4 block scheme — products 3/5/6/9
compute column 3 trivially and no product mixes the blocks. So AC2 demonstrates
**reaching a certified optimal-rank scheme on a new rectangular target from a naive
start**, *not* discovering a non-obvious (indecomposable) algorithm. Recorded in
`theory/discovery-results.md`; finding an indecomposable reduction is the honest next
question, and the natural AC for a follow-up requirement.

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
