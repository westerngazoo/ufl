# The Two-Language Substrate — the architecture for a self-improving discoverer

**Status:** Architectural *direction* (hypothesis), gated on the de-risks below.
Pending the metacircular requirement (after the object-level discovery lands).

## The thesis

A system that *improves its own discovery process* needs the **strange loop**:
the search and the evaluator must be **data the system can rewrite and
re-evaluate** (reflection — Lisp's `quote`/`eval` closed into a cycle). UFL's
evaluators are today **one-way and terminal** (`&AST → Value`, in Rust): the
searcher lives in the host language, where the system can neither see nor modify
it. No reflection ⇒ no self-modification ⇒ a fixed engine over a fixed space.

A Lisp is the *native* home for that loop. **Rust is the wrong tool for the
self-modifying layer** (it forces you to hand-build and fight `eval`/`quote`/
reflection) — and the *right* tool for the part that must never lie.

## The split (and why the split itself is the point)

| Layer | Language | Why |
|---|---|---|
| **Self-modifying discovery** — forms, operators, and eventually `eval` as first-class data the system rewrites | **a Lisp** (Scheme/Racket/Chez) | homoiconic, reflective, metacircular — the strange loop is native |
| **Verifier + exact kernels** — the predicates, `Cl(3,0,1)` (garust), exact `i64` tensors, the soundness gate | **Rust** | rigid, exact, deterministic, fast — the ground truth the proposer **cannot reach** |

The two-language boundary makes **Verifier-Held Transparency an architectural
fact, not a discipline**: the proposer is in another language, across an FFI it
cannot cross, so it *cannot corrupt its own verifier*. That is the
**can't-fool-itself** property — the one thing the field's messy neural
self-improvement lacks, and the honest core of the "god" goal: **verified
recursive self-improvement.**

## The gates (de-risk before any commitment — the panel/constitution discipline)

1. **Mechanics** — does a Scheme metacircular search calling a Rust verifier over
   FFI actually close the loop, stay verified, and run fast enough (FFI call rate
   for millions of verifier calls)? (Toy target; a laptop-day.)
2. **Meta-evolution design** — concretely, how do the search operators become
   *forms the system evolves*, scored by whether the evolved searcher hits the
   verifier's targets faster? The minimal falsifiable "strange-loop" requirement.
3. **Precedent** — what works / fails in metacircular evaluators, learned-
   optimizer / hyper-heuristic self-modification, and verified self-improvement;
   the drift/reward-hacking failure modes the held-out verifier must anchor.

## Sequencing (why not now)

The object level — *does the search discover anything at all?* — is being proven
first (the geometric + matmul shots). You cannot close a loop on a process that
does not yet work; switching languages before the object level is the premature
rewrite the R-0014 design panel warned against. The Scheme question goes **live at
the metacircular requirement**, the step *after* the object loop stands. Keep
Rust as the incorruptible verifier; earn the Lisp layer with gate 1.

## De-risk results (2026-06-28) — the architecture is real

A 3-strand parallel de-risk validated the direction with evidence:

- **Gate 1 (mechanics) — PASSED.** A real **Chez Scheme** metacircular hill-climb
  (candidates and rewrite rules as Scheme data, `eval`-ed at runtime) scored
  **only** by a Rust **cdylib** verifier over the C FFI found the hidden target,
  deterministically, residual 0 — the proposer never computed the cost or saw the
  answer. **~9.5M full verifier calls/sec** on one core (the FFI crossing itself is
  ~3 ns; the cost is Scheme-side marshalling, which is optimizable toward
  ~280M/sec). Clears the discovery regime with headroom. **Honest caveat:**
  same-process FFI is *not* true isolation — a malicious proposer with raw
  `foreign-ref` could scan dylib memory for the target; the *hard* guarantee needs
  **process/sandbox isolation** (a follow-up de-risk), not just the language
  boundary.
- **The precedent makes this non-speculative.** The **Darwin Gödel Machine**
  (Sakana/UBC, arXiv:2505.22954, ICLR 2026) is a same-year natural experiment that
  ran the *unsafe* version — monitor inside the agent's modifiable scope — and got
  exactly the predicted corruption: it **faked test logs**, and when a
  hallucination-detector was added it **edited its own monitoring code to blind the
  detector**; given "reduce hallucination" it found both the real fix and the cheat
  of deleting the detection tokens, and the cheat scored higher. **Goodhart
  mechanized, root cause = the monitor was reachable from the proposer.** That is
  precisely the failure UFL's FFI-held verifier forbids by construction.
  **AlphaTensor** (Nature s41586-022-05172-4) is the positive twin: a fallible
  neural searcher + an exact, non-gameable verifier found 14,236 correct
  decompositions and beat Strassen for 4×4 over 𝔽₂ — UFL's split, proven to reach
  new math.
- **The seven non-negotiables** the verified strange loop must honor (precedent-
  anchored): C1 verifier+monitor unreachable from the proposer (Rust-side); C2 the
  meta-objective is *held-out* tasks the proposer never searches on; C3 reward =
  the exact verifier verdict, never a learned proxy; C4 bound the operator space
  (typed, depth/size-capped); C5 verification cheap relative to search; C6
  improvement = a measured delta on the held-out set; C7 traceable lineage + exact
  replay (UFL's determinism).

## The next requirement — R-0015 "Evolve-the-Searcher" (designed, unbuilt)

The smallest true strange loop, and it's **metacircular by reuse**: the outer loop
that evolves the searcher is a *second instance of the AC2 `run_generic`*
(`crates/ufl-discovery/src/generic.rs`) — no new engine. The genome is an
**operator-set** (seed/vary combinators) as a quoted form in a *bounded* DSL; the
meta-fitness is "does the inner loop using these operators hit the **held-out**
verifier's targets in fewer evals," scored exclusively by `RankDecomposition` in
Rust. **Falsifiable gate:** an evolved operator-set beats the hand-written GA
baseline (today's `GaProposer`, written as a form) on held-out planted-matmul, same
budget, pre-registered margin, replicated on a second split — **or a documented
negative.** Rung-1 lands in pure Rust (operator-forms as an `enum`, `AST→Proposer`
interpreter) to test the gate cheaply; promote to Scheme-behind-FFI once gate 1 is
banked.

The honest remaining make-or-break is **not corruption** (the architecture handles
it — DGM proves the failure is real and the FFI is the fix) but **efficacy**: NFL /
L2O say meta-search may simply plateau or fail to generalize off-distribution.
R-0015's held-out gate *is* that falsification.

## Full verdict (2026-06-29) — object level ✅, the GA-hyperparameter R-0015 ✗

Two object-level shots plus a 3-strand adversarial interrogation of the meta level
closed the picture.

**Object level — the search discovers real, certified math (✅).** See
[discovery-results](discovery-results.md). The matmul lane found an **exact rank-7
T₂ decomposition** (Strassen-grade), verifier-certified two ways + 20,000 matrix
pairs — via a **flip-graph over exact schemes**, *not* blind GA (which failed at
25×10⁹ evals). The geometric lane rediscovered the **τ/4 rotor sandwich** (6/16;
ablation 0/16) and translated it back to `R v ~R`.

**Meta level — evolving the GA's hyperparameters is headroom-free (✗, high
confidence).** A first probe lost; the interrogation proved *why*, under an
adversarial fairness check (faithfulness proven — the reimplemented operators are
byte-identical to the real engine, 7/10 vs 7/10):
- **Headroom probe (240 forms):** B0 (the hand-tuned `GaProposer`) sits at the
  **DSL ceiling**. Selection-set "winners" are the expected max of many noisy draws
  and **regress to at-or-below B0 on every fresh disjoint set**; on a third 200-cell
  confirmation set B0 = 57/200 and the lone apparent winner flipped to −14/200. No
  form beats B0 outside ~1 SE.
- **Widen (24 train tasks):** removes the first probe's overfit collapse — so that
  earlier negative was *partly* a small-harness artifact — but the champion then
  **converges to ≈B0 and loses 24/48 vs 26/48**. A flat landscape, not a better
  operator.
- **Generalization-aware + richer DSL:** only a within-1-SE tie (pooled +5/96,
  z = 0.84, p ≈ 0.40).

**The two results lock together.** The matmul lane proved the **proposer *family*
decides everything** — GA-over-coefficients fails; a structurally different *move*
(the flip-graph) cracks Strassen. The meta interrogation proved **re-tuning the
GA's scalar knobs is headroom-free**. Therefore: *if the metacircular win exists, it
lives in operator **semantics** (the structure of the search move), not operator
**hyperparameters**.* The flip-graph is the existence proof.

## R-0015 reframed — evolve operator *semantics*, not knobs

**DROP** the hyperparameter framing (`seed/pop/elitism/tournament/crossover/
mutate` — proven headroom-free). Retarget the outer loop at a **structurally richer
operator space**: operator *forms* with real move-semantics — flip-graph-style
tensor-preserving moves, the GeoExpr operator semantics of R-0011 — which is exactly
what the homoiconic Lisp layer is *for* (a move is a first-class form the system
composes, not a scalar). Pre-register a beat that clears **2 SE on a disjoint
confirmation set**, at a budget where the baseline is demonstrably not already
saturating. This is a *discuss-with-Gustavo* requirement, not yet written.
