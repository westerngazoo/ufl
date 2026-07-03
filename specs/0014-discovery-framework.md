# SPEC-0014 — The shared discovery substrate

- **Status:** **Accepted** (2026-07-03) — three-lens complete: architect
  *approve* (blockers verified resolved against code; gates green), hater findings
  resolved (§6), nice-guy approved; the new-`ufl-search`-crate topology
  Gustavo-confirmed. Gustavo signs the transition (§1.2).
- **Realizes:** [R-0014](../requirements/0014-discovery-framework.md) (one search
  substrate, per-lane verifier instances)
- **Author:** main session
- **Created:** 2026-07-02 · **Revised:** 2026-07-03
- **Depends on:** SPEC-0008 (the matmul engine), SPEC-0011 (the geometric
  harness), SPEC-0013 (the flip-graph move invariant)
- **Crate(s):** **`ufl-search`** (NEW — the pure substrate), `ufl-discovery`
  (matmul lane, hardened in place first)

## 1. Motivation

R-0014 is merged as Draft with "Realized by: SPEC-0014 (pending)". The
genome-generic seam (`run_generic`, `Proposer`/`Fitness`) is on `main`, but it
carries three concrete defects and two structural debts:

1. **The error channel is not generic.** `Fitness::score` hardwires
   `Result<S, EngineError>` ([generic.rs:34](../crates/ufl-discovery/src/generic.rs))
   where `EngineError` is matmul-population-specific — the geometric lane's errors
   (`GeoError` from evaluation, [ufl-geo/src/eval.rs:10](../crates/ufl-geo/src/eval.rs);
   `GradeError` from the type system, [ufl-geo/src/grade.rs:45](../crates/ufl-geo/src/grade.rs))
   cannot flow through, blocking R-0014 AC2's geometric instance.
2. **Empty-population panic.** `run_generic` indexes `scored[0]`
   ([generic.rs:83](../crates/ufl-discovery/src/generic.rs)) unguarded — a custom
   `Proposer` returning an empty `Vec` panics in library code (§6 violation).
3. **No coherence seam.** SPEC-0011 AC2 requires grade-incoherent candidates
   rejected *before* scoring, but the loop offers only `Proposer`/`Fitness`.
4. **No eval accounting.** The engine counts *generations*, not *verifier calls*
   — but R-0015's meta-fitness *is* evals-to-target on held-out tasks
   (`theory/two-language-substrate.md`), so the meta-objective is unmeasurable.
5. **A crate-topology trap.** The traits must live somewhere pure; the naive
   choice (`ufl-evolve`) is already taken by a different concern (§2.1).

## 2. Design

### 2.1 Topology — a NEW pure crate `ufl-search` (corrected to the real tree)

**Correction (three-lens finding, §6):** `ufl-evolve` **already exists** on `main`
(merged PR #33) and holds the **fair-MLP Gate-2 baseline**
([ufl-evolve/src/baseline.rs](../crates/ufl-evolve/src/baseline.rs)), and R-0011
earmarks geometric *tasks* (deps `ufl-geo`/`ufl-ga`) for it. It cannot also be the
`ufl-prng`-only substrate. So the pure search seam gets its **own new crate**:

```
ufl-prng ← ufl-search  (pure: Proposer, Fitness, Screen, run_generic, Ledger)
                 ↑ ↑ ↑
   ufl-discovery ┘ │ └ ufl-geo        (lanes depend inward on the seam)
        ufl-evolve ┘                  (baseline + geo tasks) → ufl-search + ufl-geo
```

`ufl-search` depends on **`ufl-prng` only**. Lanes depend on it: `ufl-discovery`
(matmul) and `ufl-geo` (geometric) each `→ ufl-search`; `ufl-evolve` keeps the MLP
baseline and depends on `ufl-search + ufl-geo`. **The engine crate `ufl-discovery`
gains no `ufl-geo`/`ufl-ga` edge** (`cargo tree -p ufl-discovery` is the enforced
invariant). No merged code moves. Re-exports from `ufl-discovery` keep
`tests/r_0014_generic_seam.rs` imports unchanged. **The physical move to
`ufl-search` executes with T8**; this spec decides the topology and hardens the
traits **in place in `ufl-discovery` first**.

### 2.2 The generic error channel (real types)

`Fitness` gains an associated error type:

```rust
pub trait Fitness<G, S> {
    type Error;
    fn score(&self, genome: &G) -> Result<S, Self::Error>;
    fn solved(&self, score: &S) -> bool;
}
```

`run_generic` is generic over it. The matmul instance sets `type Error =
EngineError`. The **geometric** instance surfaces *two* error sources — evaluation
and grade-typing — so its error is a lane-local sum, not a single foreign type:

```rust
// in the geometric lane (ufl-geo), NOT in ufl-search:
pub enum GeoLaneError { Eval(GeoError), Grade(GradeError) }
```

The genericity witness in the Acceptance set is a **two-variant** toy enum (not a
one-error toy), proving the channel carries a real multi-source lane error.

### 2.3 The empty-population guard (no name collision)

`EngineError` **already has** an `EmptyPopulation` variant meaning "config
validation: population < 1" — a *different event* from "a `Proposer` returned an
empty `Vec` at runtime." To avoid confusing them, the runtime event is a new,
distinctly-named error:

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RunError<E> {
    #[error("proposer yielded an empty population at runtime")]
    ProposerYieldedEmpty,          // NOT EngineError::EmptyPopulation (config validation)
    #[error(transparent)]
    Lane(#[from] E),
}
```

The `scored[0]` index becomes `scored.first().ok_or(RunError::ProposerYieldedEmpty)?`.

**Byte-identical matmul boundary (the hard gate):** `run_matmul_generic` must keep
returning plain `EngineError`. It folds `RunError<EngineError>` back:
`Lane(e) → e`; `ProposerYieldedEmpty → unreachable`, because the matmul lane
**validates the config up front** (`config.validate()`) *and* its `Screen` is
always-admissible (§2.4), so `seed`/`vary`/screen can never empty the population.
The fold is therefore **provably dead code** for every validated matmul config —
the byte-identical trajectory is preserved (proven by the unchanged
`r_0014_generic_seam.rs` sweep). The fold is written as an explicit `match`, not an
`unwrap`, so an impossible state is a typed engine error, never a panic.

### 2.4 The answer-blind coherence screen

A `Screen` seam rejects structurally-incoherent candidates *before* `score` — the
harness-level rendering of SPEC-0011 AC2, audited in one place:

```rust
pub trait Screen<G> {
    /// Answer-blind admissibility. `false` drops the candidate before scoring.
    fn admissible(&self, genome: &G) -> bool;
}
```

**Answer-blindness is a construction property, not just a call-time one**
(three-lens finding, §6): a `Screen` instance **may depend only on the lane, never
on the task instance** — otherwise a lane could bake target-derived data into the
screen and leak the answer without "consulting the verifier" at call time. The
geometric screen (`grade`/`typecheck`) is admissible: grade coherence is a property
of the *algebra*, not the target, so it is constructed from the lane alone. Stated
as a contract each lane's screen construction must satisfy.

**Application point:** the loop filters **both** the seed population and each
`vary` output — it applies `admissible` to `pop` at the **top of each iteration,
before `score`** (so a screened *seed* genome, generation 0, also never reaches
`score()`). The default `NoScreen` admits everything (matmul, flip-graph). A
spy-`Fitness` test proves a screened genome never reaches `score()`.

### 2.5 The eval ledger (screen-aware, well-defined on both paths)

A plain `u64` verifier-call counter — one field, no framework — counting
**`Fitness::score` calls** (post-screen; a screened-out genome is *not* an eval):

- Incremented in `run_generic` by `scored.len()` per generation (the number of
  admissible genomes actually scored) — no interior mutability needed.
- Reported in `Outcome`/`GenericOutcome`; the flip-graph result reports its own
  `moves_tried` analogue.
- **Invariants, scoped:** on the **Exhausted** path *with the default `NoScreen`*,
  `evals == population × (generations + 1)`. Under a non-trivial screen, `evals ==
  Σ_gen (admissible count at gen)` — strictly the count of `score` calls. On the
  **Found** path, `evals == (Σ over completed gens) + (admissible count of the
  solving generation)` — the loop returns mid-generation, so the count is the
  partial sum, stated separately.

**Meta-fitness definition (for R-0015, resolving the cross-form comparability
finding):** R-0015 compares move-forms by **post-screen `score` calls to solve**,
right-censored at a fixed budget `B` — a form that never solves a held-out task
scores `B` (not undefined), with `best_rank_reached` as the documented tie-break.
Because R-0015's actual lane is the **flip-graph with the default `NoScreen`**
(§2.4), screen-drop-rate differences do **not** enter its comparison; the
screen-aware clause exists for the geometric lane, which is *not* on R-0015's
critical path. The GA `evals` and flip-graph `moves_tried` are reconciled as "the
lane's unit of verifier-work"; SPEC-0015 pins the single statistic.

### 2.6 The harness contract (three explicit pieces)

| Piece | matmul | geometric | 𝔽₂ |
|-------|--------|-----------|-----|
| **Move invariant** (preserved by construction) | tensor preservation (SPEC-0013 §2.4 `debug_assert` promoted to a tested property) | — | — |
| **Coherence screen** (answer-blind, §2.4) | `NoScreen` | `grade`/`typecheck` | well-formed polynomial |
| **Soundness fuzz** (`realized ⊆ inferred`, real kernel) | rank ≤ claimed | R-0010's grade fuzz | — |

**Design rule (the SPEC-0013 §2.1 lesson):** *types constrain candidates,
invariants constrain moves — never prune intermediates with the answer-space type.*
(Promoted to `docs/conventions.md` as a named convention — see the nice-guy
amplification, tracked in T14.)

### 2.7 Closure-typing for R-0015 — a *constructive* rule (not a ∀-over-outputs)

**Correction (three-lens finding, §6):** "an operator is admissible iff *every*
genome it can emit is admissible" is an infinite quantifier — undecidable by
enumeration for a parameterized move-form, and vacuous for the always-`NoScreen`
matmul lane R-0015 uses. Replace it with a **finite, structural** rule:

> An operator-form is *closure-admissible* iff (a) each **primitive** it composes
> is closed under the lane's screen — for the R-0013 flip-graph primitives this is
> discharged by the same *by-construction tensor-preservation* proof (SPEC-0013);
> and (b) each **DSL combinator** (`seq`, `choose`, bounded `perturb`) **preserves
> closure**. Closure is then a structural induction over the form, not a runtime
> test over an infinite output set.

For the matmul lane the primitives are tensor-preserving and there is no screen, so
closure is immediate; for a screened lane it is a finite check per combinator.

## 3. Code outline

```rust
// ufl-search (topology decided here; physical crate created in T8)
pub trait Proposer<G, S> { /* seed, vary — unchanged */ }
pub trait Fitness<G, S> { type Error; fn score(&self, g: &G) -> Result<S, Self::Error>; fn solved(&self, s: &S) -> bool; }
pub trait Screen<G> { fn admissible(&self, g: &G) -> bool; }
pub struct NoScreen;                     // admits everything; the matmul/flip-graph screen
impl<G> Screen<G> for NoScreen { fn admissible(&self, _: &G) -> bool { true } }

pub struct Ledger { pub evals: u64 }     // post-screen score() calls

// DECIDED (was Open Question 2): the wrapper with `#[from] E`.
pub enum RunError<E> { ProposerYieldedEmpty, Lane(E) }

// `run_generic` gains `screen`; existing calls pass `&NoScreen` (byte-identical).
pub fn run_generic<G, S, P, F, C>(
    proposer: &P, fitness: &F, screen: &C, generations: usize, seed: u64,
) -> Result<(GenericOutcome<G, S>, Ledger), RunError<F::Error>>
where G: Clone, S: Ord + Copy, P: Proposer<G, S>, F: Fitness<G, S>, C: Screen<G> {
    // pop = seed → filter(screen.admissible) → score (count into Ledger) → sort
    //     → first().ok_or(ProposerYieldedEmpty)? → solved? → vary → filter → …
    // score's `?` converts F::Error via RunError::Lane (#[from]); the collect type
    // is Result<_, RunError<F::Error>>.
    unimplemented!()
}
```

The matmul re-host passes `&NoScreen`, so `tests/r_0014_generic_seam.rs`'s
calls change by one argument and stay byte-identical in behavior (the required
gate). The signature break is confined to that one added `&NoScreen` at each call.

## 4. Non-goals

- **No physical `ufl-search` crate before T8** — topology on paper here; hardening
  lands in place in `ufl-discovery` first.
- **No new search algorithm** — the flip-graph (R-0013) and GA (R-0008) are
  behavior-unchanged.
- **No operator-evolution** — the closure rule (§2.7) is *stated* for R-0015.
- **No change to the byte-identical matmul re-host** — one `&NoScreen` arg aside.

## 5. Open questions

1. Whether `Ledger` and the flip-graph `moves_tried` unify into one struct or stay
   two lane-named counters — settled when T1's result type and SPEC-0015 land.
2. The exact `RunError<EngineError> → EngineError` fold spelling at the matmul
   boundary (a `match`, provably dead `ProposerYieldedEmpty` arm) — confirm the
   re-host test still shows identical `Outcome` for all sweep configs.

## 6. Decision log — three-lens resolutions (2026-07-03)

| Finding (lens) | Resolution |
|---|---|
| Topology premise stale — `ufl-evolve` is merged with the MLP baseline (architect/hater [blocking]) | §2.1 rewritten: the seam gets a **new pure `ufl-search` crate**; `ufl-evolve` keeps the baseline. **Gustavo-confirmed, 2026-07-03.** No merged code moves. |
| `EvalError` does not exist in `ufl-geo` (architect/hater [blocking]) | §1/§2.2 corrected to `GeoError`/`GradeError`; geometric lane error is a two-variant `GeoLaneError`; the witness toy is two-variant. |
| `RunError::EmptyPopulation` collides with `EngineError::EmptyPopulation` (hater [blocking]) | Renamed `ProposerYieldedEmpty`; §2.3 specifies the matmul fold-back and proves it dead for validated configs. |
| `run_generic` signature break threatens byte-identity (architect [major] 11) | §2.3/§3: existing calls pass `&NoScreen`; behavior byte-identical; break confined to one arg. |
| `RunError<E>` wrapper left open (architect [major] 12) | Decided in §3 (wrapper + `#[from] E`); collect-type change shown. |
| Ledger invariant false under a screen; cross-form comparability (architect/hater [major] 8/10) | §2.5: ledger counts post-screen `score` calls; invariant scoped to `NoScreen`/Exhausted; Found-case stated; R-0015 meta-fitness right-censored, and its flip-graph lane uses `NoScreen` so screen-rates don't confound. |
| Closure-typing undecidable / vacuous (architect/hater [major] 9/15) | §2.7 replaced with a **constructive structural** rule (primitives closed + combinators preserve closure). |
| Screen answer-blindness only at call time (hater [major] 10) | §2.4: blindness extended to **construction** — screens depend on the lane, never the task instance. |
| Seed population not screened (architect [minor] 13) | §2.4: `admissible` filters at the top of each iteration, covering the seed. |
| Guard justification vs. screen (architect [minor] 14) | §2.3/§2.4: the guard fires when seed/vary **or the screen** empties the population; matmul's `NoScreen` preserves byte-identity. |

Gustavo holds final approval before Draft→Accepted (§1.2). The nice-guy
amplifications (promote §2.6 + `Screen` to `docs/conventions.md`; the
`raise`-inspection upside; the R-0016 ∥ R-0015 crate-disjoint parallelism) are
tracked in T14 and the task briefs — non-blocking.
