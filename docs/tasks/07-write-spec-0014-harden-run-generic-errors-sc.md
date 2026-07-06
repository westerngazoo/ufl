# T7 · Write SPEC-0014; harden run_generic: errors, screen, eval ledger

- **Priority:** P0
- **Depends on:** none
- **Tags:** spec, rung-0, spine, crate:ufl-discovery, R-0014

## Context
R-0014 is Draft on main with "Realized by: SPEC-0014 (pending)" — no SPEC-0014 exists, yet the seam it governs has three concrete defects and two structural debts:
1. **Error channel not generic**: `Fitness::score` hardwires `Result<S, EngineError>` (crates/ufl-discovery/src/generic.rs:35) where `EngineError` (engine.rs:18-31) is matmul-population-specific — ufl-geo's `EvalError` cannot flow through, blocking R-0014 AC2's second half.
2. **Empty-population panic**: `run_generic` indexes `scored[0]` (generic.rs:83) with no guard — a custom Proposer whose `seed`/`vary` returns an empty Vec panics in library code (§6 violation; `run_matmul_generic` validates, raw `run_generic` does not).
3. **No coherence seam**: SPEC-0011 AC2 requires grade-incoherent candidates rejected BEFORE scoring, but the loop offers only Proposer/Fitness — pruning stays a per-proposer convention, unauditable, contradicting the verifier-held discipline at the harness level.
4. **No eval accounting**: the engine counts generations, not verifier calls — but R-0015's meta-fitness IS "fewer evals to hit held-out targets" (theory/two-language-substrate.md); the meta-objective is literally unmeasurable today.
5. **Crate-topology trap**: generic.rs's own header promises "SPEC-0014 relocates the traits to the shared ufl-evolve substrate", but SPEC-0011 §3 also puts geometric TASKS (deps: ufl-geo/ufl-ga) in ufl-evolve — if traits and tasks share a crate, ufl-discovery gains a transitive ufl-geo edge.

## Work
**Spec (first):** write specs/0014-*.md covering: (a) topology — ufl-evolve as PURE substrate (Proposer/Fitness/Refiner/Screen/run_generic/eval-ledger, deps: ufl-prng only); lanes stay in their own crates (matmul in ufl-discovery depending on ufl-evolve; geometric tasks NOT in the engine crate); re-exports keep tests unchanged; relocation EXECUTES with T8, after #33 merges. (b) the harness contract as three explicit pieces: a by-construction move invariant, an answer-blind coherence screen, a realized-⊆-inferred fuzz template — instantiated geo = grade/typecheck, tensor = tensor-preservation (promote SPEC-0013 §2.4's debug_assert to a tested property), F2 = well-formed polynomials. Record the SPEC-0013 §2.1 lesson as a design rule: types constrain candidates, invariants constrain moves — never prune intermediates with the answer-space type. (c) the closure-typing rule for R-0015: an operator is typed by its production closure (every emitted genome typechecks) — SPEC-0011 AC2 lifted one level.
**Code (in-place in ufl-discovery, three-lens on the spec first):**
1. Give `Fitness` an associated `type Error` (or generic E) instead of hardwired `EngineError`.
2. Guard the empty population: return a typed `EmptyPopulation` error, never index-panic.
3. Add the answer-blind screen hook (e.g. `fn admissible(&self, g: &G) -> bool` on Proposer, or a `Screen` trait) defaulting always-true.
4. Thread a u64 verifier-call counter through `engine::run`, `run_generic`, and (with T1) the flip-graph result: evals reported in Found/Exhausted/GenericOutcome, moves-tried in the flip result. Plain field, no framework.

## Acceptance gate (falsifiable)
- tests/r_0014_generic_seam.rs byte-identical sweep stays green (same seeds, same trajectories).
- Unit test: empty-seed proposer ⇒ `Err(EmptyPopulation)`, no panic.
- Compile-tested instance whose `Fitness::Error` is a non-EngineError toy enum proves the channel is lane-generic.
- Spy-Fitness test: a screened-out genome never reaches `score()`.
- Ledger test: `evals == population × (generations_elapsed + 1)` for a pinned exhausted GA run.
- SPEC-0014 passes the three-lens review; `cargo tree -p ufl-discovery` shows no ufl-geo/ufl-ga edge (invariant preserved now, enforced after T8's relocation).

## Must NOT claim
Do not physically relocate crates before #33 merges (T2) — topology is decided on paper here, executed in T8.

## Files/crates
specs/0014-*.md (new), crates/ufl-discovery/src/{generic.rs,engine.rs}, crates/ufl-discovery/tests/r_0014_generic_seam.rs.
