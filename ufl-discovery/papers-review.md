# PAPERS REVIEW — the geometric-neuroevolution evidence base

*Companion to [FINDINGS.md](FINDINGS.md). Reviews the five papers + the
CliffordNet reference code that set the 2026-06-12 direction
([[project-neuroevolution-direction]]), **adjusted to UFL's architecture**, and
stress-tests the approach's viability before SPEC-0008. Same discipline as
FINDINGS: the real code/result wins over the abstract's marketing.*

All five arXiv papers were verified to exist and say what is claimed; the
CliffordNet repo (`github.com/ParaMind2025/CAN`) was read at source level. Two
claims in the original framing did **not** survive contact with the artifacts —
flagged below.

---

## 1. What each paper actually gives UFL

| Paper | What it is (verified) | What it gives UFL | Honest caveat |
|---|---|---|---|
| **CliffordNet / CAN** (2601.06793) | A vision backbone whose "geometric product" is a **learned channel-shift approximation** (`wedge = z1*C_shift − C*z1_shift`), *not* a real Clifford algebra — no Cayley table, no signature, no grades (real PyTorch path; core is a compiled `clifford_thrust` CUDA wheel). 8× param cut on CIFAR-100. | A **substrate target**, not a semantics: the cheap GPU layer a *real* UFL geometric AST could compile down to. Fits the compilation thesis perfectly. | **Does NOT validate "real GA is efficient."** Its 8× comes from a cheap approximation. We may not cite its numbers as evidence for a faithful `Cl(3,0,1)` kernel. |
| **Decidable By Construction** (2603.25414, Haynes) | Design-time verification; model properties as constraints over finitely-generated abelian groups ℤⁿ, **decidable in poly time, unique principal type**; a **Program Hypergraph** inferring Clifford grade + a dimensional type system. | The external proof that UFL's own bet — *grade inference as a decidable type system, verification before training* — is sound and publishable. This is the closest external work to UFL. | It's the **adjacent competitor**. UFL must differentiate (see §4). |
| **Adaptive Domain Models** (2603.18104, Haynes) | The training half of the same series: Program Hypergraph + grade-preservation as a type-level invariant + posit arithmetic; PHG certificates; "warm rotation". | Confirms grade-preservation-through-computation is a *type-level* property — informs UFL's R-0010 grade inference. | **Gradient-trained**, not evolutionary. The whole Haynes program is. |
| **Symmetry in the Wild / AB-GATr** (2605.18816) | E(3)-equivariant GA Transformer for CFD; the GATr lineage it extends *is* PGA-based. | Validates **PGA for spatial/physics equivariance** — reinforces `Cl(3,0,1)` over `G(3,0,0)`. | **Two framing corrections:** the abstract never says "PGA"; and the result is *cautionary* — equivariance can **degrade** performance on symmetry-breaking data. Not "symmetries for free". |
| **GA-VisAgent** (2605.01299) | A dual-agent (Planner→Worker) pipeline that decomposes a GA formula into ordered typed subtasks and materializes them as validated GAALOPScript over **CGA `Cl(4,1)`**. 90% vs GPT-4o's 20% via constrained function-calling. | The **agentic AST-generation template** (see §3) — and a ready-made **form taxonomy** for R-0010. | **Codegen-from-known-formula, not discovery.** It translates; it doesn't search. CGA, not PGA. |

**Through-line:** the literature validates GA-as-neural-primitive, PGA for space,
and grade-typed-hypergraph verification — but **every one of them is
gradient-trained or codegen; none discovers programs by evolution.** That gap is
UFL's lane, and it is also why viability must be argued, not assumed (§4).

## 2. CliffordNet code reality (the "real code wins" finding)

The paper's `uv = u·v + u∧v` is, in the visible implementation, two cheap
shifted-channel ops with learnable weights and **no algebra underneath**. For
UFL this is clarifying, not discouraging: it means a faithful `Cl(3,0,1)` kernel
(R-0009, via garust) is doing something CliffordNet only *names*, and the
CliffordNet layer is a legitimate **compile target** for an evolved geometric
AST — real GA meaning at the top, cheap channel-shift kernel at the bottom.
UFL's efficiency claim therefore must rest on **program discovery** (a closed-form
geometric expression has ~0 learned parameters), not on "real-GA-is-cheap".

## 3. The GA-VisAgent pattern, mapped onto UFL

GA-VisAgent's transferable architecture, in UFL terms:

| GA-VisAgent | UFL equivalent | Requirement |
|---|---|---|
| Planner decomposes a formula into ordered typed subtasks | An **agent proposer** that emits a structured geometric **AST** from a goal | R-0011 |
| Five subtask categories (object-creation / algebraic / transformation / element / numerical) | The **grammar of geometric s-expr forms** — a ready-made taxonomy | R-0010 |
| Worker materializes each node via **constrained function-calls** | Typed form constructors (no malformed nodes — *Guard Inside the Candidate*) | R-0010 |
| **Validate Agent** bounces non-compliant code to regenerate | The **predicate discharge** — but on *semantics*, not just syntax | R-0007 → R-0011 |
| Subtask output format (id, GA type, vars, …) | AST node serialization for mutation/crossover | R-0011 |

The taxonomy is a gift; the agentic Planner is the **scalable proposer** UFL will
need beyond toy problems (§4). What GA-VisAgent does *not* give us is the
discovery itself — it translates known formulas; UFL searches for unknown ones.

## 4. Viability — thinking hard before the spec

The direction is **viable for Phase 1 with the risk correctly concentrated and
cheaply falsifiable**, *provided one architectural adjustment* (§5). The honest
analysis:

**The central risk is the search, not the check.** Genetic programming over
expression trees (Koza, 30 years) works for symbolic regression and small
synthesis but is well-known to fail on **deceptive, low-evolvability
landscapes** — where the answer's near-neighbours have terrible fitness so there
is no gradient to climb. This is exactly why **AlphaTensor used deep-RL guidance,
not blind GA**, for matmul decomposition. UFL's verification layer (predicate
discharge + grade typing) is its *de-risked strength* (R-0004/R-0007, 153 tests);
the discovery layer is where the project can die.

**The 2×2 canary (R-0008) is the cheap falsification.** 2×2 rank-7 is the
*friendly* case — small enough that local/evolutionary search has found
Strassen-equivalent schemes in the literature (SAT, numerical, GP). So blind
seeded GA *probably* clears it, and the graded residual gives a real gradient.
But the AC4 hedge ("≥3 of 10 seeds") is honest: if it clears **zero** seeds in
budget, that is the signal that blind GA is too weak — caught for one
requirement's cost, before any geometry is built. **Do not soften that gate.**

**The geometric target is friendlier than matmul, not harder.** The Phase-1
geometric gate — rediscover the sandwich `R x R̃` — is a 3-node tree with a
*dense* fitness signal (pose error is continuous, unlike matmul's sparse
exact-zero). If evolution cannot find a 3-node geometric tree under dense
fitness, GP is fundamentally broken and we learn it cheaply. Inverse kinematics
(the eventual target) is *geometrically natural* (a product of motors) and has a
smooth fitness landscape — the strongest part of the bet, because GA is genuinely
the right language for rigid-body motion (the reason motors/GATr exist).

**The "no neural guidance" purity is the wrong invariant.** Blind GA likely will
not scale past toy problems; the scalable proposer is the GA-VisAgent-style
**agent** — which is "neural guidance through the back door," contradicting the
roadmap's transparency goal. The resolution dissolves the tension:

> **Transparency belongs to the *verifier*, not the *proposer*.** UFL does not
> need to know *where* a candidate came from; it needs the *acceptance* to be an
> exact, transparent predicate discharge. A blind GA, an LLM agent, or a coin
> flip may propose — only an exact discharge may *accept*. This is already UFL's
> architecture (R-0007 discharges any candidate regardless of origin).

This makes the differentiator **verified discovery of geometric programs —
proposer-agnostic, verifier-exact** — which *subsumes* neuroevolution as the
Phase-1 proposer and is far more defensible than "pure neuroevolution" (which the
literature suggests won't scale). It is an *enhancement* of the chosen direction,
not a retreat from it.

**Red flags, stated bluntly (with mitigations):**

1. Blind GA clears *zero* seeds on 2×2 Strassen → engine thesis fails at the
   canary. *Mit: R-0008 AC4 hedge + AC6 diagnostics; if it fails, the agent
   proposer is promoted from R-0011 to rescue R-0008.*
2. Geometric-AST landscape too deceptive even with dense fitness → R-0011 can't
   find a 3-node sandwich. *Mit: that would be a fundamental GP failure, learned
   for one requirement's cost.*
3. Scope/time — four requirements + garust + grade inference is heavy.
   *Mit: falsifiable gates; kill-early-if-canary-fails.*
4. Efficiency claim doesn't materialize on a real task. *Mit: pick IK, where the
   closed-form geometric solution is provably tiny and exact — a known-high
   ceiling.*

### 4a. Empirical de-risk (2026-06-12) — the canary fired early

Before accepting SPEC-0008, the proposed operators were run against the real
2×2 problem (throwaway scripts, not product code). **Three independent blind
methods all failed to reach exact residual 0**, on the *friendly* 2×2 case:

| Method (seeds 0-9 unless noted) | Rank 8 (AC3 wants ≥9/10) | Rank 7 (AC4 wants ≥3/10) |
|---|---|---|
| Population GA — the spec's operators (tournament + uniform triple-crossover + point mutation + elitism) | **1/10** | **0/10** |
| (1+λ)-ES + plateau-restarts + annealing acceptance + variable mutation | 0/10 | 0/10 |
| Memetic — random-restart + greedy coordinate descent (600 restarts, 4 seeds) | 0/4 | 0/4 |

**Signature:** every method drives the residual ~60 → 1–2 quickly (finds the
*neighbourhood*) then **plateaus at a small nonzero floor** — the final exact fix
requires a *coordinated multi-entry* change that single-step blind moves cannot
make, and the population can't synthesise once converged. This is the deceptive
low-evolvability landscape §4 named — the exact wall AlphaTensor used learned
guidance to avoid — reproduced on our own problem in ~70 s per run.

**Verdict:** this is **not** a budget/operator-tuning gap (the trajectories are
flat at the floor, not slowly descending); blind discrete `{-1,0,+1}` search does
not solve exact 2×2 matmul decomposition. The literature agrees (Smirnov;
Heule-Kauers-Seidl used SAT; continuous relaxation + rounding finds Strassen
readily — none is naive GP). **SPEC-0008's AC3/AC4 are unachievable by a blind
proposer**, caught for ~5 minutes of experiment instead of a full implementation
cycle. The proposer-agnostic seam is vindicated: the weakness is isolated to the
proposer; the verifier is untouched.

**This does not kill the program — it relocates the prize.** Rediscovering
Strassen belongs to a *stronger* proposer (R-0011's memetic/agentic source).
R-0008 should be re-scoped to what a blind proposer *can* validate (the loop
mechanics on a solvable known-answer instance) plus the documented blind-GA
falsification as its honest headline result — which is precisely what AC6 was
written to capture.

### 4b. Empirical de-risk — what a blind proposer CAN validate (the planted target)

The §4a falsification forced the question: what solvable known-answer instance
can R-0008 validate the *loop* on? A **planted target** — a tensor built as the
sum of a fixed set of `{-1,0,+1}` triples, so a low-rank decomposition provably
exists. The engine must rediscover *a* decomposition blind (it is never handed
the planted triples).

**The fixed instance** (`d = 4`, the R-0008 test fixture — pin these literals so
it is RNG-independent and reproducible):

```
triple 1:  u=[ 0, 0, 1, 0]  v=[ 1, 1, 0, 0]  w=[ 1, 1,-1, 0]
triple 2:  u=[-1, 1, 0, 0]  v=[-1,-1, 1, 1]  w=[ 1,-1, 0,-1]
triple 3:  u=[ 0, 0,-1, 0]  v=[-1,-1, 0, 1]  w=[-1, 0,-1, 1]
triple 4:  u=[ 0, 0, 1, 1]  v=[-1, 0, 1, 1]  w=[ 0, 0, 0, 1]
triple 5:  u=[ 0, 0, 0, 0]  v=[ 1, 0, 1,-1]  w=[-1, 0, 1, 0]   (u = 0 ⇒ degenerate)
```

Target `T_planted` = Σ uᵢ⊗vᵢ⊗wᵢ has 39 non-zero entries. **True rank ≤ 4** (triple
5 is degenerate); searching at **rank 5** gives deliberate slack — appropriate for
a *solvable loop-validation gate* (we want it reliably findable, not minimal).

**Result — pinned config** (population 300, generations 1500, tournament 5,
mutation 2, elitism 4): the blind GA recovers an exact (residual 0) rank-5
decomposition for **8 of seeds 0..=9** (gens 55–1294 to first hit; the two misses
plateau at residual 2–3). This is the evidence base for AC3's ≥6/10 gate (set
below the measurement with margin). *Caveat:* this is the Python de-risk's pass
rate under Python's RNG; the Rust engine uses `SplitMix64`, so its exact pass
count is independently verified at QA — the instance's *solvability* is what
transfers, and ≥6/10 is the honest gate.

**The matmul descent signature (the AC4 checkable property).** On the unsolvable
`T_2` rank-7 case, the same engine on all 10 seeds drives the best residual from
~52–69 (seed population) down to 2–5 (final) — **strict decrease, terminating
above 0, every seed.** That is the falsifiable proof the *engine functions* and
the *landscape* is the wall: a no-op or broken engine would show no decrease.
AC4 must assert exactly this (initial strict decrease ∧ terminates > 0 for all
seeds), so "documented either way" cannot launder a broken engine.

## 5. Architectural implications for the spec chain

1. **The R-0008 forward seam becomes proposer-agnostic.** SPEC-0008 must name a
   boundary where the *candidate source* (blind genetic operators now) is one
   implementation behind an interface, and the *acceptance* is always
   `Predicate::discharge`. R-0011 then adds the geometric genotype **and** the
   agent proposer as new sources behind the same seam, the verifier unchanged.
   (Still a breadcrumb, not a built abstraction — SPEC-0008 builds only the GA
   proposer; it just names the seam.)
2. **R-0010 adopts the GA-VisAgent five-category taxonomy** as its form grammar.
3. **`Cl(3,0,1)` PGA for Phase 1** (points/lines/planes/motors — the kinematics
   target); **CGA `Cl(4,1)`** (spheres/circles, GA-VisAgent's space) is a Phase-2
   option, not a Phase-1 commitment.
4. **The efficiency thesis is reframed** to program-parsimony (closed-form
   geometric program vs learned weights), not real-GA-is-cheap — so the benchmark
   must be a task with a known compact geometric solution (IK).

## 6. Recommendation

Proceed to **SPEC-0008** with the **proposer-agnostic / verifier-exact** seam as
its one forward-looking design note. The viability risk is real but
*concentrated in the well-studied GP component, cheaply falsifiable at the 2×2
canary, and de-risked by UFL's existing exact-verifier strength* — which is the
right shape for a research bet. The differentiator is sharpened from
"neuroevolution" to "verified geometric-program discovery," which the evidence
base supports and the competitors (gradient-trained, every one) do not occupy.

## Changelog

- 2026-06-12 — created. Reviews 2601.06793, 2603.25414, 2603.18104, 2605.18816,
  2605.01299 + the CliffordNet/CAN reference code; sets the proposer-agnostic
  viability conclusion ahead of SPEC-0008.
