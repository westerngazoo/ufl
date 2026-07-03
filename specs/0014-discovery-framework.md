# SPEC-0014 — The shared discovery substrate

- **Status:** Draft (pending R-0014 acceptance + the three-lens review; no code
  before Accepted — constitution §4.2)
- **Realizes:** [R-0014](../requirements/0014-discovery-framework.md) (one search
  substrate, per-lane verifier instances)
- **Author:** main session
- **Created:** 2026-07-02
- **Depends on:** SPEC-0008 (the matmul engine), SPEC-0011 (the geometric
  harness), SPEC-0013 (the flip-graph move invariant)
- **Crate(s):** `ufl-evolve` (the substrate), `ufl-discovery` (matmul lane)

## 1. Motivation

R-0014 is merged as Draft with "Realized by: SPEC-0014 (pending)". The
genome-generic seam (`run_generic`, `Proposer`/`Fitness`) is on `main`, but it
carries three concrete defects and two structural debts that block the rungs
above it:

1. **The error channel is not generic.** `Fitness::score` hardwires
   `Result<S, EngineError>` ([generic.rs:35](../crates/ufl-discovery/src/generic.rs))
   where `EngineError` is matmul-population-specific — ufl-geo's `EvalError`
   cannot flow through, blocking R-0014 AC2's geometric second instance.
2. **Empty-population panic.** `run_generic` indexes `scored[0]`
   ([generic.rs:83](../crates/ufl-discovery/src/generic.rs)) unguarded — a custom
   `Proposer` returning an empty `Vec` panics in library code (§6 violation;
   `run_matmul_generic` validates, raw `run_generic` does not).
3. **No coherence seam.** SPEC-0011 AC2 requires grade-incoherent candidates
   rejected *before* scoring, but the loop offers only `Proposer`/`Fitness` — so
   answer-blind pruning stays a per-proposer convention, unauditable, contra the
   verifier-held discipline at the harness level.
4. **No eval accounting.** The engine counts *generations*, not *verifier calls*
   — but R-0015's meta-fitness *is* "fewer evals to hit held-out targets"
   (`theory/two-language-substrate.md`), so the meta-objective is literally
   unmeasurable today.
5. **A crate-topology trap.** `generic.rs`'s header promises SPEC-0014 relocates
   the traits to `ufl-evolve`, but SPEC-0011 §3 also puts geometric *tasks*
   (deps `ufl-geo`/`ufl-ga`) in `ufl-evolve` — if traits and tasks share one
   crate, `ufl-discovery` gains a transitive `ufl-geo` edge.

## 2. Design

### 2.1 Topology — `ufl-evolve` is a *pure* substrate

`ufl-evolve` holds **only** the search mechanism and its contracts —
`Proposer`, `Fitness`, `Refiner`, `Screen`, `run_generic`, the eval ledger —
with **`ufl-prng` as its only dependency**. Lanes stay in their own crates and
depend *inward* on `ufl-evolve`:

```
ufl-prng ← ufl-evolve ← ufl-discovery         (matmul lane: Genome, RankDecomposition, flip-graph)
                      ← ufl-geo   (+ ufl-ga)   (geometric lane: GeoExpr, grade harness)
```

The engine crate (`ufl-discovery`) **must not** gain a `ufl-geo`/`ufl-ga` edge
(`cargo tree -p ufl-discovery` is the enforced invariant). Geometric *tasks*
live with the geometric *types*, never in the substrate. Re-exports from
`ufl-discovery` keep the existing `tests/r_0014_generic_seam.rs` imports
unchanged. **The physical relocation EXECUTES with T8** (after PR #33 lands
`ufl-evolve`); this spec decides the topology on paper, and hardens the traits
in place in `ufl-discovery` first.

### 2.2 The generic error channel

`Fitness` gains an associated error type; the loop is generic over it:

```rust
pub trait Fitness<G, S> {
    type Error;
    fn score(&self, genome: &G) -> Result<S, Self::Error>;
    fn solved(&self, score: &S) -> bool;
}
```

`run_generic` returns `Result<GenericOutcome<G, S>, F::Error>`. The matmul
instance sets `type Error = EngineError`; a geometric instance sets
`type Error = EvalError` — the channel is lane-generic, proven by a
compile-tested toy enum instance (§Acceptance).

### 2.3 The empty-population guard

The `scored[0]` index is replaced by a checked head; an empty population after
`seed` or `vary` is a typed error, never a panic:

```rust
let (best_genome, best_score) = scored.first().ok_or(RunError::EmptyPopulation)?;
```

`RunError<E>` wraps `EmptyPopulation` plus the lane `E`. (The matmul re-host
keeps validating up front, so its byte-identical behavior is unchanged: a
validated config never hits the guard.)

### 2.4 The answer-blind coherence screen

A `Screen` seam lets a lane reject structurally-incoherent candidates *before*
they reach `Fitness::score` — the harness-level rendering of SPEC-0011 AC2,
audited in one place rather than per-proposer:

```rust
pub trait Screen<G> {
    /// Answer-blind admissibility: `false` drops the candidate before scoring.
    /// MUST NOT consult the target/verifier (that would leak the answer).
    fn admissible(&self, genome: &G) -> bool;
}
```

Default `admissible = true` (the matmul lane keeps every candidate). The
geometric lane screens on `grade`/`typecheck`; the flip-graph lane needs no
screen (its moves are exact by construction). The loop applies `admissible`
between `vary` and `score`; a screened genome **never** reaches `score()`
(spy-Fitness test, §Acceptance).

### 2.5 The eval ledger

A plain `u64` verifier-call counter threads through `engine::run`,
`run_generic`, and the flip-graph result — no framework, one field:

- `Outcome::Found`/`Exhausted` and `GenericOutcome::*` report `evals` (the
  number of `Fitness::score` / discharge calls).
- The flip-graph result reports `moves_tried` (its analogue of an eval).
- Invariant for a pinned exhausted GA run: `evals == population ×
  (generations_elapsed + 1)`.

This makes R-0015's meta-fitness ("evals-to-target on held-out") measurable.

### 2.6 The harness contract (three explicit pieces)

R-0014's "one search substrate, three verifier instances" is made precise as a
*contract* each lane instantiates, not one shared enum:

| Piece | What it is | matmul | geometric | 𝔽₂ |
|-------|-----------|--------|-----------|-----|
| **Move invariant** | a property preserved *by construction* by every move | tensor preservation (SPEC-0013 §2.4 `debug_assert` **promoted to a tested property**) | — (tree edits are unconstrained) | — |
| **Coherence screen** | answer-blind `admissible` (§2.4) | always-true | `grade`/`typecheck` | well-formed polynomial |
| **Soundness fuzz** | `realized ⊆ inferred`, fuzzed on the real kernel | rank ≤ claimed | R-0010's grade fuzz | — |

**Design rule (the SPEC-0013 §2.1 lesson, recorded):** *types constrain
candidates, invariants constrain moves — never prune intermediates with the
answer-space type.* The flip-graph could not live in the `{−1,0,+1}` `Scheme`
type precisely because a move's *intermediate* leaves the candidate type; the
invariant (`reconstruct_int == target`) is what holds it, not the type. Lanes
must not reject in-flight states with the final-answer type.

### 2.7 The closure-typing rule for R-0015

When R-0015 evolves *operators* (move-forms), an operator is typed by its
**production closure**: it is admissible iff *every* genome it can emit is
admissible under the lane's screen (§2.4). This is SPEC-0011 AC2 lifted one
level — the answer-blind screen applied to the operator's output set, not a
single candidate. (Stated here so R-0015's spec inherits it; not built here.)

## 3. Code outline

```rust
// ufl-evolve (topology decided here; relocation executes in T8)
pub trait Proposer<G, S> {
    fn seed(&self, rng: &mut SplitMix64) -> Vec<G>;
    fn vary(&self, ranked: &[(G, S)], rng: &mut SplitMix64) -> Vec<G>;
}
pub trait Fitness<G, S> {
    type Error;
    fn score(&self, genome: &G) -> Result<S, Self::Error>;
    fn solved(&self, score: &S) -> bool;
}
pub trait Screen<G> { fn admissible(&self, genome: &G) -> bool; }

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RunError<E> {
    #[error("proposer produced an empty population")]
    EmptyPopulation,
    #[error(transparent)]
    Lane(#[from] E),  // the lane's own error, generically
}

pub struct Ledger { pub evals: u64 }  // threaded, reported in the outcome

pub fn run_generic<G, S, P, F, C>(
    proposer: &P, fitness: &F, screen: &C, generations: usize, seed: u64,
) -> Result<(GenericOutcome<G, S>, Ledger), RunError<F::Error>>
where G: Clone, S: Ord + Copy, P: Proposer<G, S>, F: Fitness<G, S>, C: Screen<G> {
    // seed → [screen] → score → sort → solved? → vary … guarding scored.first()
    // and counting each score() call into Ledger.evals
    unimplemented!()
}
```

## 4. Non-goals

- **No physical crate relocation before PR #33 merges** — topology is decided on
  paper here, executed in T8. The hardening (§2.2–2.5) lands *in place* in
  `ufl-discovery` first.
- **No new search algorithm** — this hardens the existing seam; the flip-graph
  (R-0013) and GA (R-0008) are unchanged in behavior.
- **No operator-evolution** — the closure-typing rule (§2.7) is *stated* for
  R-0015 to inherit, not implemented.
- **No change to the byte-identical matmul re-host** — same seeds, same
  trajectories (the regression gate).

## 5. Open questions

1. `Screen` as a separate trait vs. a defaulted `Proposer::admissible` method —
   the trait keeps the seam auditable and lets one screen serve many proposers;
   confirm in review.
2. `RunError<E>` vs. requiring `E: From<EmptyPopulation>` — the wrapper avoids
   burdening every lane error; confirm ergonomics against the geometric instance.
3. Whether the flip-graph's `moves_tried` and the GA's `evals` should be one
   `Ledger` type or two — they count different things; provisionally one struct
   with lane-named fields, settled when T1's result type is in hand.
