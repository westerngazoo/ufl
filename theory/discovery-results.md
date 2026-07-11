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

**Debt paid (2026-07-02, PR #55).** The flip-graph is committed as
`ufl_discovery::reduce_matmul` (SPEC-0013), with the certification, the
20,000-pair bilinear check, and the trajectory replay as regression tests —
the results above are banked, reproducible code, not claims.

## Geometric — rediscovery of the τ/4 rotor sandwich (BANKED, PR #73)

**Committed and regression-gated** (2026-07-04, SPEC-0011M / R-0011 Gate-1): the
memetic engine — the SPEC-0011 tree-GA on `run_memetic` with grade-`{0}`
param-slot refinement via a **±δ geometric ladder** (10⁻¹…10⁻¹¹) — rediscovers
the τ/4 rotation on **6/16 pinned seeds** at pop=400/gens=400 (architect-
reproduced), with the ablation (`NoRefine`, **identical `vary` stream** — the
ladder draws zero rng) at **2/16**: refinement triples the rediscovery rate, and
the contrast isolates refinement alone. Winners render through the real printer:

- seed 9 (verbatim): `(~((v exp(e₁₂ 0.785)) 1)) exp(e₁₂ 0.785)` — the rotor
  sandwich `R̃ v R`.
- seed 8 (verbatim): `e₃ exp(𝒢_2(0.785))` — an alternate route to the same
  rotation.

**Mechanism findings, measured in-repo (PR #73):** fixed-σ Gaussian refinement
scored **0/16** (a resolution floor above the 1e-6 bar) — the multi-scale ladder
is load-bearing, not refinement per se; and unbounded crossover stack-overflows
without the 60-node anti-bloat cap. The earlier deleted-pilot narrative
(6/16 vs 0/16 ablation, pure-GA 3/12) is retained as provenance only — the
citeable evidence is the committed e2e (`crates/ufl-evolve/tests/r_0011m_gate1.rs`).
qa ratifies the Gate-1 threshold at loop step 7 (R-0011 AC4).
