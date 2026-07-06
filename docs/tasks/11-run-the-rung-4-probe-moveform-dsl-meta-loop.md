# T11 · Run the Rung-4 probe: MoveForm DSL, meta-loop, 2-SE gate

- **Priority:** P1
- **Depends on:** T1, T4, T7, T9
- **Tags:** rung-4, spine, R-0015, crate:ufl-discovery, crate:ufl-evolve

## Context
The make-or-break falsification point of the whole program. The two locked results (proposer FAMILY decides everything — flip-graph 0.16s vs blind GA dead at 25e9 evals; knob-tuning is headroom-free) say the only live metacircular bet is operator SEMANTICS — and there is zero positive evidence for it. theory/two-language-substrate.md is explicit: a human-designed flip-graph beating a GA is NOT evidence a meta-search can evolve such a move (the non-sequitur caveat), and this cheap pure-Rust falsification must pay BEFORE any Lisp substrate is built. Everything above this rung, including Rung-5, is conditional on this gate.

## Work (implements accepted requirements/0015 from T4; SPEC-0015 first, three-lens)
1. **SPEC-0015 must first resolve the modal mismatch** — `Proposer<G,S>` is population-shaped (seed/vary) but the flip-graph is a graph WALK (requirements/0013: "a distinct search modality… does not reuse run_generic"). Design the common move seam spanning BOTH families; this open question is currently written nowhere.
2. **MoveForm DSL**: a depth/size-capped `enum MoveForm` (C4 bounded, typed) over the T1 flip primitives and GA steps — e.g. Seq, Flip{selector}, Reduce, Perturb{k}, TournamentVary{…} — expressive enough to write BOTH known families: GaProposer-style coefficient moves AND flip-graph-style tensor-preserving rewrites.
3. **Interpreter**: `compile(&MoveForm) -> impl Proposer` — the interpreter stays Rust-side, verifier-held, answer-blind.
4. **Meta-loop**: a SECOND `run_generic` instance whose genome is MoveForm and whose fitness is evals-to-target (T7's ledger) on TRAIN planted instances, measured through T9's held-out harness. C1-C7 enforced: the proposer never sees held-out targets; reward = exact verifier verdict only.
5. **Parallelize scoring as the first implementation step** (not before): rayon par_iter on the pure per-genome scoring in engine::score (engine.rs:88-101) and run_generic's map — order-preserving collect keeps determinism since scoring draws no RNG. The meta-loop multiplies cost (population-of-forms × inner runs × planted instances); sequential scoring makes the 2-SE budget wall-clock-infeasible on a laptop.
6. Genome carrier is the Rust enum per the substrate doc's Rung-1 ("operator-forms as an enum, AST→Proposer interpreter", pure Rust). The quoted-Sexpr carrier (bridging to T5's reflection rung) is an explicitly recorded follow-on taken only AFTER this gate pays.

## Acceptance gate (falsifiable, pre-registered in requirements/0015 — verbatim)
- The evolved operator-set beats the hand-written baseline (today's flip driver and GaProposer, EACH written as a MoveForm) by ≥2 SE on a DISJOINT held-out confirmation set, replicated on a second split, at a budget where the baseline is demonstrably not saturating — OR a documented negative lands in theory/. Both are valid deliverables; the only failure is a silent middle (moved goalposts, post-hoc winners).
- Determinism: same seed ⇒ byte-identical trajectory through run_generic; the full r_0014 sweep byte-identical with RAYON_NUM_THREADS=1 vs 8; pop-300 scoring shows ≥4x wall-clock speedup on 8 cores.
- A documented negative formally kills Rung-5 and redirects effort to object-level scaling (R-0013 AC3's T₃ attempt).

## Must NOT claim
That the flip-graph's human-designed success predicts meta-search efficacy; that a positive here proves general self-improvement (it proves one operator family is evolvable on one task distribution).

## Files/crates
specs/0015-*.md (new), crates/ufl-discovery (or the ufl-evolve substrate): moveform.rs + interpreter + meta-loop test, crates/ufl-discovery/src/engine.rs (rayon), requirements/0015 (gate source), theory/ (outcome).
