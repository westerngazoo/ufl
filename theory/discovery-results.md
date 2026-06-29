# Discovery results — verifier-certified

Concrete artifacts the discovery engine produced (de-risk pilots, 2026-06-29).
Not yet R-loop deliverables (no qa sign-off) — recorded because they are
verifier-certified and load-bearing for R-0013 (matmul Gate-0) and R-0011
(geometric Gate-1). See [two-language-substrate](two-language-substrate.md) for
how they fit the architecture.

## Matmul — an exact rank-7 decomposition of T₂ (Strassen-grade)

Beats the naive rank 8. Found by a **Kauers–Moosbauer flip-graph over exact
schemes** (start from the naive rank-8; apply tensor-preserving split/flip +
rank-reducing merge moves, so every state is exact by construction; reduce to
rank 7) — **0.16 s, reproducible 3/3**. Certified by the real verifier two ways:
`RankDecomposition::new(2,7).residual(&scheme) == 0`, `discharge == Ok(true)`
(exact reconstruction *and* rank == 7), and the bilinear form checked on **20,000
random integer matrix pairs**.

Index map `0=(1,1) 1=(1,2) 2=(2,1) 3=(2,2)`. The 7 products:

```
m1 = (−a12 + a22)(−b22)
m2 = (−a21 + a22)(b11 + b12)
m3 = (−a11 + a12)(b21)
m4 = (−a11 + a22)(b11 + b12 + b21)
m5 = ( a22)(b11 + b12 + b21 + b22)
m6 = (−a11)(b11 + b21)
m7 = ( a11 − a21)(−b12)

c11 = m3 − m6
c12 = m1 − m4 + m5 + m6
c21 = −m2 + m4 − m6 − m7
c22 = −m4 + m5 + m6 + m7
```

**Honest, and the point:** blind GA / L2-coefficient basin-hopping did **not**
find this — 25×10⁹ evals across 10 threads, trapped forever at residual 1 (the
coefficient landscape is densely studded with deceptive error-1 traps — the wrong
substrate). The flip-graph over exact schemes did, instantly. *The proposer is the
result.*

**Not a new theorem — a system demonstration.** By **de Groote's 1978 uniqueness
theorem**, *every* rank-7 algorithm for ⟨2,2,2⟩ is Strassen's up to the problem's
symmetry group, so this scheme **cannot be novel** — it is a known-optimal result
*re-derived* by the engine. The asset is the *engine + exact verifier*, not the
object. A genuinely new result comes only from pointing the same engine at a tensor
whose optimal rank is **open** (e.g. ⟨3,3,3⟩).

**Reproducibility debt (honest).** The flip-graph proposer, the blind-GA arm, and
the 20,000-pair check were **throwaway de-risk harnesses that have been deleted** —
they are **not yet committed**. Per CLAUDE.md (test-first, trust nothing) the
results above are **claims pending banked code**. The load-bearing next step is to
commit the flip-graph as a real `Proposer` behind the R-0008 seam, with the
certification as a regression test.

## Geometric — rediscovery of the τ/4 rotor sandwich

A **memetic GA** (tree-structure search + local `Param` refinement) over `GeoExpr`
rediscovered the τ/4 rotation: **6/16 seeds** (pure-GA §2.8 baseline 3/12; ablation
without param-refinement **0/16** — refinement is the load-bearing step). Winners
translate back through the real ufl-geo printer:

- seed 0: `Sandwich(Exp(GeoProduct(Param(−0.7854), Basis(3))), Var("v"))`
  → `let R = exp(−0.785 e₁₂) ; R v ~R`  (−0.785 ≈ −τ/8 — the textbook rotor).
- seed 4: the same rotation via the grade-lift route, `exp(𝒢₂(−0.785))`.

**Honest:** robust at gens=400 / pop=400, only marginal at a low budget; a
self-certified pilot (cargo test green), not yet qa-reviewed. The morph → discover
→ translate-back loop closed on the real kernel (eval/typecheck/render).
