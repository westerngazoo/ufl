# SPEC-0011M — The memetic engine (`ufl-evolve`) — R-0011 Gate-1 rediscovery

> **This is a companion draft that extends the Accepted [SPEC-0011 —
> Geometric Neuroevolution](0011-geometric-neuroevolution.md)** with the
> **memetic design that post-dated its acceptance**. SPEC-0011 (Accepted
> 2026-06-26) closed on the tree-GA harness + the §2.6/§2.8 pilots; the
> **elite `Param`-refinement step** that actually crossed the exact bar
> (`theory/discovery-results.md`: **6/16** with refinement, **0/16** without)
> and the **`Refiner` seam** it needs were folded into SPEC-0011 §2.2 as a
> bullet but never given a realizing design. This document supplies that
> design against the **real, current** crate tree — the hardened `ufl-search`
> seam (SPEC-0014) that did not exist when SPEC-0011 was accepted. It **does
> not restate** SPEC-0011's motivation, Gate-2 headline, MLP baseline, or
> decision log; it cross-references them.

- **Status:** **Draft**
- **Realizes:** **R-0011 Gate-1** (the geometric rediscovery — the validation
  gate, [R-0011 AC4](../requirements/0011-geometric-neuroevolution.md)); also
  discharges **R-0014 AC2**'s *geometric second instance* of the generic seam.
- **Author:** main session — drafted with Claude
- **Created:** 2026-07-03
- **Depends on:**
  - **SPEC-0011** (Accepted) — the harness, the `GeoExpr` genotype, the tree-GA
    operators, the NaN-safe readout policy, the §2.4 Gate-1 task, the §2.8 pilot
    evidence. **This document extends it; it does not duplicate it.**
  - **SPEC-0014** (Accepted) — the pure `ufl-search` substrate: `Proposer`,
    `Fitness{type Error}`, `Screen`, `NoScreen`, `Ledger`, `run_generic`, and
    the topology (§2.1) that puts the seam in `ufl-search`, the geo lane in
    `ufl-geo`, and the geo *tasks*/engine glue in `ufl-evolve`.
- **Crate(s):** `crates/ufl-evolve` (the memetic engine + tasks — already exists
  on `main` holding the fair-MLP baseline, SPEC-0014 §2.1), `crates/ufl-geo`
  (the new param-slots API + the `Screen`/`Refiner` instances), `crates/ufl-search`
  (the NEW `Refiner<G>` trait + the refinement pass on `run_generic`).

---

## 0. Reconciliation note — read this first (topology + seam divergences)

Two ground-truth documents disagree, because one post-dated the other. This
spec follows the **newer, Accepted** decision and flags every divergence so the
three-lens catches nothing by surprise.

1. **Topology.** `docs/tasks/08-…` (T8) predates SPEC-0014 and says "traits/engine
   move to `ufl-evolve` (deps: ufl-prng only)". **SPEC-0014 §2.1 (Accepted
   2026-07-03) overrode this**: `ufl-evolve` *already exists* on `main` and holds
   the fair-MLP Gate-2 baseline (`crates/ufl-evolve/src/baseline.rs`), so it
   cannot be the `ufl-prng`-only substrate. The pure seam lives in the **new
   `ufl-search` crate** (already a workspace member, deps `ufl-prng` only). **This
   spec places the `Refiner` trait in `ufl-search`** (beside `Proposer`/`Fitness`)
   and the memetic engine/tasks in `ufl-evolve` (deps `ufl-search` + `ufl-geo`).
   The T8 acceptance-gate *checks* (below) are transcribed verbatim and remain
   valid — only the "which crate holds the trait" sentence in T8's Work list is
   superseded.

2. **`Fitness` shape — SPEC-0011 sketch vs. the real seam.** SPEC-0011 §2.1
   sketched `trait Fitness { fn score(&self, &GeoExpr) -> Fit }` where `Fit: Ord`
   and the engine **maximizes** (higher better), with NaN mapped to `Fit::WORST`.
   The **real** `ufl-search` seam is `Fitness<G, S> { type Error; fn score(&self,
   &G) -> Result<S, Self::Error>; fn solved(&self, &S) -> bool }` and
   `run_generic` **minimizes** (`S: Ord + Copy`, cost-ascending sort). This spec
   realizes the memetic engine against the **real** seam: the geometric fitness is
   a **cost** (lower = better), the NaN-safe total order lives in the concrete
   `S`, and `Fit::WORST` becomes "the maximal cost". This is a faithful
   translation of SPEC-0011 §2.3's intent, not a new decision — but it *is* a
   shape change from SPEC-0011's snippet and is called out here.

3. **`ufl-geo → ufl-search` edge is new.** For the geometric `Screen` and
   `Refiner` instances to live in `ufl-geo` (SPEC-0014 §2.1's arrow), `ufl-geo`
   gains a `ufl-search` dependency it does not have today (`ufl-geo`'s only dep is
   `ufl-ga` + `thiserror`). This spec adds it. `ufl-search` stays `ufl-prng`-only,
   so no cycle. (Alternative considered in §9.)

---

## 1. Motivation

SPEC-0011's harness is accepted, but three facts leave R-0011 Gate-1 unshipped
and the whole "grade as the evolution constraint" premise **without a single
consumer**:

- **The only proposer that ever solved the geometric lane is a deleted pilot.**
  `theory/discovery-results.md` records a **memetic** GA — tree-structure search
  **plus local `Param` refinement on the elites** — rediscovering the τ/4 rotor
  sandwich on **6/16** seeds, with the ablation *without* refinement scoring
  **0/16**. The refinement step is load-bearing and currently exists nowhere in
  the repo. SPEC-0011 §2.2 names it in one bullet; it has no realizing design and
  no home in the seam.

- **`typecheck` has zero real consumers.** `git grep typecheck origin/main` hits
  only `crates/ufl-geo/examples/hello_geo.rs`. SPEC-0014 built the `Screen` seam
  so grade-incoherence can prune *before* scoring (SPEC-0011 AC2); wiring
  `ufl_geo::typecheck` as the geometric `Screen` instance turns AC2 from a
  convention into an architectural fact and makes R-0010's decidable grade signal
  earn its keep.

- **R-0014 AC2 has no geometric instance.** SPEC-0014 hardened `run_generic` and
  proved it genome-generic with a toy lane, but the *second real lane* (geometric
  fitness on the generic seam) is unbuilt; this spec is it.

The memetic upgrade has one hard constraint: **local refinement must *score*
candidates**, and if the *proposer* scores, the proposer sees the target —
breaking answer-blindness and **Verifier-Held Transparency** (only the verifier
`Fitness` may consult the target; SPEC-0014 §2.4). The design below resolves this
by making the **engine** hold the hill-climb and the **`Refiner`** merely propose
neighbor genomes — the same structural move SPEC-0014 used to keep the `Screen`
answer-blind.

---

## 2. Design

Three pieces: a new **`Refiner<G>` seam** in `ufl-search` (§2.1), a **typed
param-slots API** in `ufl-geo` (§2.2), and the **memetic engine + geometric
instances** in `ufl-evolve` (§2.3). All signatures below are grounded in the
real `crates/ufl-search/src/lib.rs`, `crates/ufl-geo/src/{expr,grade,eval,render}.rs`,
and `crates/ufl-prng/src/lib.rs`.

### 2.1 The `Refiner<G>` seam (new, in `ufl-search`)

A new trait beside `Proposer`/`Fitness`. The refiner is **answer-blind by
construction, exactly like `Screen`**: it takes an elite genome and an RNG and
returns *candidate neighbor genomes*. It never scores them — it cannot, because
it holds no `Fitness`. **The engine scores them.**

```rust
// crates/ufl-search/src/lib.rs — beside Proposer / Fitness / Screen.

/// Proposes *neighbor* genomes of an elite for the engine to hill-climb — the
/// memetic seam (SPEC-0011-memetic §2.1). The refiner is **answer-blind**: like
/// `Screen`, an instance may depend only on the *lane*, never on the *task
/// instance*, and it returns candidates **without scoring them**. Only the engine
/// scores (via `Fitness`), so Verifier-Held Transparency survives the memetic
/// upgrade — the refiner never touches the target.
pub trait Refiner<G> {
    /// The local neighborhood of `elite`. May be empty (no move available). The
    /// engine scores each neighbor and keeps improvements; the refiner only
    /// *proposes* moves.
    fn neighbors(&self, elite: &G, rng: &mut SplitMix64) -> Vec<G>;
}

/// The default refiner: proposes nothing. Lanes with no local move (matmul today)
/// use it, so `run_memetic` collapses to `run_generic`'s trajectory byte-for-byte.
pub struct NoRefine;

impl<G> Refiner<G> for NoRefine {
    fn neighbors(&self, _elite: &G, _rng: &mut SplitMix64) -> Vec<G> {
        Vec::new()
    }
}
```

**The engine holds the hill-climb.** A new `run_memetic` wraps the existing
`run_generic` loop with a per-generation refinement pass. Each generation, after
scoring and cost-sorting the population, the engine hill-climbs the **top-`k`
elites**: for each elite it asks the refiner for `neighbors`, **scores them under
its own `Fitness`** (counting each into the `Ledger`, screened first), and
replaces the elite with the best strictly-lower-cost neighbor found, iterating up
to a bounded number of steps. Refined elites re-enter the population before
`vary`. The refiner is invoked with the engine's `SplitMix64`, so the run stays
deterministic in `seed`.

```rust
/// The memetic budget: how hard the engine hill-climbs each elite. Zero elites or
/// zero steps ⇒ `run_memetic` is exactly `run_generic` (the `NoRefine` collapse).
#[derive(Clone, Copy, Debug)]
pub struct MemeticConfig {
    /// How many cost-lowest elites to refine each generation.
    pub elites: usize,
    /// Max hill-climb steps per elite (each step: propose neighbors, take the best
    /// strict improvement, or stop).
    pub steps: usize,
}

/// The genome-generic memetic search: `run_generic`'s loop plus a per-generation
/// elite-refinement pass driven by a `Refiner` and scored by the engine's
/// `Fitness`. The proposer AND the refiner are answer-blind; only `run_memetic`
/// holds `fitness`. With `NoRefine` (or `MemeticConfig{elites:0,..}`) the
/// trajectory and `SplitMix64` draw order are byte-identical to `run_generic`.
pub fn run_memetic<G, S, P, F, C, R>(
    proposer: &P,
    fitness: &F,
    screen: &C,
    refiner: &R,
    memetic: MemeticConfig,
    generations: usize,
    seed: u64,
) -> RunResult<G, S, F::Error>
where
    G: Clone,
    S: Ord + Copy,
    P: Proposer<G, S>,
    F: Fitness<G, S>,
    C: Screen<G>,
    R: Refiner<G>;
```

**Design rule — the refinement pass reuses the loop's existing invariants.**
Neighbors are `screen.admissible`-filtered before scoring (so a refiner that
proposes a grade-incoherent neighbor is filtered, never scored — the §2.4 screen
contract holds for refined candidates too), each scored neighbor is a `Ledger`
eval, and non-finite costs sort last via the same total `S` order (§2.3). The
hill-climb keeps a neighbor **iff its cost is strictly lower** than the current
elite's — so refinement is monotone and can never worsen an elite. This is why
the ablation is clean: disable the refiner (`NoRefine`) and the *only* thing that
changes is the elite-improvement pass; the tree-GA, screen, and ledger are
untouched.

> **Answer-blindness argument (mirrors SPEC-0014 §2.4).** `Refiner::neighbors`
> receives only `&G` and `&mut SplitMix64` — no target, no `Fitness`. A
> `Refiner` *instance* is constructed from lane data only (the geometric one
> below carries nothing task-specific — it perturbs grade-{0} slots). Therefore
> the proposer/refiner pair is answer-blind **by construction**, and the memetic
> upgrade does not relax Verifier-Held Transparency.

### 2.2 Typed param-slots (new, in `ufl-geo`) — the first typed quotation site

A typed view over the **grade-{0} `Param` leaves** of a `GeoExpr`, so elite
refinement is a **first-class, unit-tested operator over slots**, not ad-hoc tree
walking. Grounded in the real `GeoExpr` (`crates/ufl-geo/src/expr.rs`): `Param(f64)`
is the only leaf whose `grade` rule is `GradeSet::singleton(0)` unconditionally
(`grade.rs:87`).

```rust
// crates/ufl-geo/src/expr.rs (or a new slots.rs) — the slot view.

/// Mutable borrows of every `Param` leaf of a `GeoExpr`, in a fixed pre-order.
/// This is the concrete **typed quotation site** (the shape R-0015's operator DSL
/// reuses): the enumerated holes are `Param`s, which are grade-{0} *by
/// construction* (`grade(Param) = {0}`, grade.rs:87), so writing any `f64` through
/// a slot **cannot change `typecheck`'s verdict** (§ invariant below).
pub fn params_mut(e: &mut GeoExpr) -> Vec<&mut f64>;

/// The read-only companion: the current slot values, same pre-order as
/// `params_mut` (for snapshot/restore in a hill-climb step).
pub fn params(e: &GeoExpr) -> Vec<f64>;
```

**The typecheck-invariant (stated, tested — the load-bearing property):**

> For any `GeoExpr e`, `GradeCtx ctx`, and any reassignment of the `f64` values
> reachable through `params_mut(&mut e)`, `typecheck(&e, ctx)` returns the same
> `Result` (same `Ok(GradeSet)` or same `Err`) as before the reassignment.

*Why it holds by construction.* `typecheck`/`grade` (`grade.rs`) branch on the
tree's **structure** and on leaf *indices* (`Basis(i)`, `GradeLift(k, _)`,
`GradeProject(k, _)`), never on a `Param`'s **value**: `grade(Param(_)) =
singleton(0)` ignores the payload, and no other rule reads a `Param`'s `f64`.
`is_versor` likewise matches on `Exp`/`GeoProduct`/`Basis`/`Reverse` structure,
not on any `Param` value. Since `params_mut` yields borrows of **only** `Param`
payloads (never a `u8` index, never structure), refining through it is invisible
to `grade`/`typecheck`. The unit test (§4) asserts this against the real
`typecheck` over a spread of trees.

> **Honest scope note (this is the whole safety story).** The slot mechanism
> exposes **grade-{0} `Param` values only**. It **cannot** reach a `Basis`/`GradeLift`/
> `GradeProject` index or any structural node — those are the tree-GA's job
> (SPEC-0011 §2.2). So "refinement never changes typecheck's verdict" is not a
> runtime check that could regress; it is a **type-level fact** of what the slot
> API can address. Local search over slots is orthogonal to structure by
> construction — the same reason it is answer-blind (§2.1) and the reason the
> ablation isolates *refinement*, not the *screen*.

### 2.3 The memetic engine + geometric instances (new, in `ufl-evolve`)

`ufl-evolve` (deps `ufl-search` + `ufl-geo`) hosts four concrete pieces plugged
into `run_memetic`. It **reuses SPEC-0011's harness design** for the tree-GA
operators and the readout policy — this section specifies only what the memetic
realization adds or pins to real types.

**(i) The genome + cost.** Genome `G = GeoExpr`. Cost `S` is a `Copy + Ord`
**total order over rotation error** (lower = better), NaN-safe: a newtype wrapping
the error's `f64` bits into an ordered key with **non-finite ⇒ maximal cost**
(the real-seam translation of SPEC-0011 §2.3's `Fit::WORST`; the engine minimizes,
so "worst" is the top of the order). `solved(&s)` is `s <= ε_solve`.

**(ii) `GeoProposer: Proposer<GeoExpr, S>`** — the tree-GA over `GeoExpr` from
SPEC-0011 §2.2 (point mutation, subtree replacement/crossover, depth+size caps,
`Exp` `Param` bounding), answer-blind, using `ufl_prng::SplitMix64`'s
`f64_unit`/`normal`/`below` (all real, `ufl-prng/src/lib.rs`). **Unchanged in
kind** from SPEC-0011 — cross-referenced, not restated.

**(iii) `GeoFitness: Fitness<GeoExpr, S>`** — the NaN-safe geometric cost.

```rust
// crates/ufl-evolve/src/fitness.rs (sketch — real seam types).
pub struct GeoFitness { /* the test vectors V and the target rotation */ }

impl Fitness<GeoExpr, RotErr> for GeoFitness {
    type Error = GeoLaneError;                 // SPEC-0014 §2.2: Eval(GeoError) | Grade(GradeError)
    fn score(&self, g: &GeoExpr) -> Result<RotErr, GeoLaneError> { /* §2.3 policy */ }
    fn solved(&self, s: &RotErr) -> bool { *s <= RotErr::SOLVE_EPS }
}
```

- **Cost = the rotation residual**, `cost(g) = (1/|V|) Σ_v magnitude(eval(g,{v}) −
  rot(v))`, so a constant like `Basis(2)` (ignores `v`) scores poorly — the task
  forces the *rotation structure* (SPEC-0011 §2.4).
- **`magnitude` is `√Σ coeff²` over all 16 blade coefficients — NOT garust's
  `Mv::norm()`.** This is a load-bearing pilot finding (`discovery-results.md`;
  SPEC-0011 §2.8): `Mv = garust::Pga3` and its metric `norm()` is metric-blind —
  it **zeros `e₀`-bearing blades** (the SPEC-0010 trap), silently hiding error on
  the null generator. The fitness must sum squared coefficients over the full
  blade basis. **(Design point, §3/§9): the exact way to read all 16 coefficients
  off `garust::Pga3` — a direct accessor vs. summing `grade(k)` projections `k∈0..=4`
  — is confirmed against the pinned garust rev during the code-outline step; the
  *requirement* (magnitude over blades, not `norm()`) is fixed here.**
- **NaN-safe readout.** Per SPEC-0011 §2.3: a non-point / non-finite intermediate
  contributes the worst per-sample error and the genome's cost is maximal — never
  a raw `NaN` into the mean, never a raw-`NaN` sort (which corrupts the
  cost-ascending selection).
- **`GeoLaneError`** is the two-variant lane sum SPEC-0014 §2.2 specified —
  `Eval(GeoError) | Grade(GradeError)` — living in the geo lane, flowing through
  `RunError<GeoLaneError>`. (Structural failures are `Err`; a *badly-scoring but
  well-formed* genome is a high cost, not an `Err`.)

**(iv) The `Screen` = `ufl_geo::typecheck`.** The geometric `Screen` instance
rejects grade-incoherent candidates before scoring — the first real consumer of
the grade harness.

```rust
// crates/ufl-geo/src/… — the answer-blind grade screen.
pub struct GradeScreen { ctx: GradeCtx }     // ctx declares INPUT var grades only

impl Screen<GeoExpr> for GradeScreen {
    fn admissible(&self, g: &GeoExpr) -> bool {
        typecheck(g, &self.ctx).is_ok()       // ∅-grade / out-of-range ⇒ dropped pre-score
    }
}
```

> **Answer-blind construction (SPEC-0014 §2.4).** `GradeScreen`'s `GradeCtx`
> declares only the **input** variable's grade (e.g. `v : {1}`, public from the
> task signature) — never the *target/output* grade. Grade coherence is a
> property of the `Cl(3,0,1)` algebra + the input grades, not of the target
> rotation, so the screen depends on the lane, not the task instance (the
> SPEC-0011 §2.2 grade-seed-bias restriction, applied to the screen).

**(v) `GeoParamRefiner: Refiner<GeoExpr>`** — the memetic step, over slots only.

```rust
// crates/ufl-geo/src/… — perturbs grade-{0} Param slots; structure-blind.
pub struct GeoParamRefiner { sigma: f64 }     // step scale; carries NO task data

impl Refiner<GeoExpr> for GeoParamRefiner {
    fn neighbors(&self, elite: &GeoExpr, rng: &mut SplitMix64) -> Vec<GeoExpr> {
        // For each Param slot (and/or a joint jitter), clone the elite, perturb
        // that slot by `rng.normal(0.0, self.sigma)` via `params_mut`, and emit
        // the clone. Structure is never touched — only grade-{0} f64s move.
        // Returns [] when the elite has no Param slots.
        // (The ENGINE scores these and keeps strict improvements — §2.1.)
    }
}
```

Because it writes only through `params_mut` (§2.2), every neighbor it emits has
**the same `typecheck` verdict as the elite** — so it can never turn an
admissible elite into a screened-out one, and it can never change the genome's
grade. It is answer-blind (carries only `sigma`) and structure-blind (slots are
grade-{0} `Param`s). This is precisely the operator whose absence makes the
ablation score 0/16.

**(vi) The task + translate-back.** The Gate-1 task is SPEC-0011 §2.4's forced
general rotation: input `Var("v")`, target the `e₁→e₂` rotation applied to `v`,
global optimum `Sandwich(Exp(GeoProduct(Param(≈−τ/8), Basis(3))), Var("v"))` (the
`hello_geo` keystone form). Winners translate back via **`ufl_geo::render`**
(real, `render.rs`) — e.g. `let R = exp(−0.785 e₁₂) ; R v ~R`, the artifact
`discovery-results.md` records. The engine runs `run_memetic` at
**gens=400/pop=400** (the pilot's robust budget).

---

## 3. Code outline (representative — not committed code)

Skeleton only; the tree-GA operator bodies and the readout are SPEC-0011's,
cross-referenced.

```
crates/ufl-search/src/lib.rs
  + trait Refiner<G> { fn neighbors(&self, &G, &mut SplitMix64) -> Vec<G>; }
  + struct NoRefine; impl<G> Refiner<G> for NoRefine { … [] }
  + struct MemeticConfig { elites: usize, steps: usize }
  + fn run_memetic<G,S,P,F,C,R>(proposer, fitness, screen, refiner,
        memetic, generations, seed) -> RunResult<G,S,F::Error>
      // = run_generic's loop; after the cost-sorted population each gen:
      //   for elite in top `memetic.elites`:
      //     repeat up to `memetic.steps`:
      //       ns = refiner.neighbors(elite, &mut rng)
      //       ns = ns.filter(screen.admissible)            // §2.4 contract
      //       score each ns via fitness (count into Ledger) // engine scores, not refiner
      //       best = argmin cost(ns); if cost(best) < cost(elite) { elite = best } else break
      //   splice refined elites back before proposer.vary(...)

crates/ufl-geo/src/{expr.rs|slots.rs}
  + pub fn params_mut(&mut GeoExpr) -> Vec<&mut f64>   // grade-{0} slots, pre-order
  + pub fn params(&GeoExpr) -> Vec<f64>
crates/ufl-geo/src/… (needs a new `ufl-search` dep — §0.3)
  + enum GeoLaneError { Eval(GeoError), Grade(GradeError) }   // SPEC-0014 §2.2
  + struct GradeScreen { ctx: GradeCtx };  impl Screen<GeoExpr> (typecheck)
  + struct GeoParamRefiner { sigma: f64 }; impl Refiner<GeoExpr> (params_mut jitter)

crates/ufl-evolve/src/
  + RotErr        // Copy+Ord total order over the rotation residual; non-finite ⇒ max
  + proposer.rs   // GeoProposer: Proposer<GeoExpr, RotErr>  (SPEC-0011 §2.2 tree-GA)
  + fitness.rs    // GeoFitness:  Fitness<GeoExpr, RotErr>   (§2.3, √Σcoeff² magnitude)
  + tasks/sandwich.rs  // the Gate-1 rotation task + gens=400/pop=400 wiring + render
  + examples/evolve_sandwich.rs  // Gate-1 live demo + behavioral regression oracle

crates/ufl-discovery/src/…  (SPEC-0014 §2.1 re-exports, unchanged behavior)
  // re-export ufl_search::{Refiner, NoRefine, MemeticConfig, run_memetic} so no
  // downstream import path breaks; matmul stays run_generic + NoScreen + (implicitly) NoRefine.
```

---

## 4. Tests (TDD — red first)

Written and failing before the code that satisfies them.

- **T-slots-1 (unit, `ufl-geo`).** `params_mut` enumerates exactly the `Param`
  leaves of a mixed tree (`Sandwich(Exp(GeoProduct(Param, Basis(3))), Var)`, a
  `GradeLift(2, Param)`, nested products) in pre-order; count and order match
  `params`.
- **T-slots-2 (unit, `ufl-geo`) — the typecheck-invariant.** For a spread of
  trees, snapshot `typecheck(&e, ctx)`, then write arbitrary `f64`s (incl.
  `NaN`, `±inf`, `0.0`, large) through `params_mut`, and assert
  `typecheck(&e, ctx)` is **unchanged** (same `Ok`/`Err`). This is the
  committed proof of the §2.2 invariant.
- **T-refiner-blind (unit, `ufl-search`).** A spy `Fitness` proves
  `Refiner::neighbors` is never handed the target and never scores: only
  `run_memetic` calls `score`. A screened-out neighbor never reaches `score`
  (extends SPEC-0014's spy-fitness test to the refinement pass).
- **T-memetic-collapse (unit, `ufl-search`).** `run_memetic` with `NoRefine`
  (or `elites:0`) yields the **byte-identical** `GenericOutcome` + `Ledger` +
  `SplitMix64` draw order as `run_generic` for the toy lane — the ablation and
  the matmul re-host both rely on this.
- **T-monotone (unit, `ufl-search`).** A toy refiner + fitness where refinement
  can only lower cost: assert a refined elite's cost is `<=` its pre-refinement
  cost, every generation (refinement never worsens).
- **T-gate1-repro (e2e, `ufl-evolve`).** On the pinned seed set at
  gens=400/pop=400, the memetic engine rediscovers the rotor sandwich on
  **≥6/16** seeds; winners `render` to a `R v ~R`-family form. Deterministic.
- **T-ablation (e2e, `ufl-evolve`).** The **same** engine/seeds with the
  `Refiner` disabled (`NoRefine`) scores **0/16** — the committed regression that
  refinement is load-bearing (was folklore).
- **T-screen-fuzz (fuzz, `ufl-geo`/`ufl-evolve`) — SPEC-0011 AC2.** Every
  proposer-emitted (and refiner-emitted) genome either `typecheck`s or is counted
  as filtered by `GradeScreen`, **never scored** while incoherent. (Extends
  SPEC-0011 AC2 to the memetic path.)
- **T-magnitude (unit, `ufl-evolve`).** A genome whose error lives on an
  `e₀`-bearing blade scores a **nonzero** cost — guards the metric-blind-`norm()`
  regression (`discovery-results.md`).

---

## 5. Acceptance gate — transcribed VERBATIM from `docs/tasks/08-…` §Acceptance gate

> - Deterministic tests reproduce the pilot: rotor-sandwich rediscovery on ≥6/16
>   pinned seeds at gens=400/pop=400, AND the ablation harness with the Refiner
>   disabled scores 0/16 — the refinement step's load-bearing status becomes a
>   committed regression, not folklore.
> - SPEC-0011 AC2 fuzz green: every proposer-emitted genome typechecks or is
>   counted as filtered, never scored.
> - Unit test: refinement never changes typecheck's verdict (slots are grade-{0}).
> - `cargo tree -p ufl-discovery` shows no ufl-geo/ufl-ga edge; the r_0014
>   byte-identical sweep still green post-relocation.

**Added gate (architect suggestion — promote the topology invariant to CI).** The
`cargo tree -p ufl-discovery` "no `ufl-geo`/`ufl-ga` edge" property is promoted
from a manual check to a **CI gate**: a CI step runs `cargo tree -p ufl-discovery`
(or `cargo tree -e no-dev -p ufl-discovery`, invert-checked for `ufl-geo`/`ufl-ga`)
and **fails the build** if either appears. This makes the `ufl-search`/`ufl-evolve`
topology (SPEC-0014 §2.1 — the pure engine crate gains no geometric dependency) a
merge-blocking machine invariant, not a convention that can silently rot when a
future `use` is added.

---

## 6. Non-goals — transcribed VERBATIM from `docs/tasks/08-…` §Must NOT claim

> That the 6/16 result generalizes beyond the rotation task, or that grade pruning
> caused it (the ablation isolates refinement, not the screen).

Additionally (inherited from SPEC-0011 §4, restated so this companion is
self-contained): no Gate-2 headline / MLP-baseline work here (that stays in
SPEC-0011/`ufl-evolve`'s `baseline.rs`); no `Normalize`/`Log` forms / literal IK
(SPEC-0011 §2.7, deferred); no grade-*entropy* fitness term (SPEC-0011 §2.2,
dropped); no new `ufl-ga` kernel work; no `GeoExpr` textual *reader*; no new
search *algorithm* — `run_memetic` is `run_generic` plus a scored elite-refinement
pass, and with `NoRefine` it is behaviorally `run_generic`.

**Honest scope note (load-bearing).** The `Refiner` runs local search over
**grade-{0} `Param` slots ONLY**; it never mutates structure — that is the
tree-GA's job (SPEC-0011 §2.2). Therefore the **answer-blind** property (§2.1) and
the **typecheck-invariant** (§2.2) both hold **by construction**, not by a runtime
check that could regress. The ablation consequently isolates *refinement* (the
`NoRefine` collapse changes only the elite-improvement pass) and says nothing about
whether the *screen* helped — consistent with the "must NOT claim grade pruning
caused it" clause above.

---

## 7. Cross-references (what this does NOT restate)

| Concern | Owner document |
|---|---|
| Motivation, the reframed headline, param-count honesty | SPEC-0011 §1, §2.5 |
| Tree-GA operators (point/subtree/crossover, caps, `Exp` bounds), grade-seed bias | SPEC-0011 §2.2 |
| The NaN-safe Mv→pose readout guard, `Fit`/cost intent | SPEC-0011 §2.3 |
| Gate-1 task definition, honesty guards (uniform-arity control, seed-node dist.) | SPEC-0011 §2.4 |
| Gate-2 headline, fair-MLP OOD comparison | SPEC-0011 §2.5, `ufl-evolve/baseline.rs` |
| §2.6 expressibility de-risk, §2.8 pilot evidence | SPEC-0011 §2.6, §2.8 |
| Generic seam (`Proposer`/`Fitness{Error}`/`Screen`/`NoScreen`/`Ledger`/`run_generic`), topology, byte-identity gate, lane-error two-variant witness | SPEC-0014 §2.1–§2.5, §3 |
| The pilot numbers (6/16, ablation 0/16, translate-back artifacts) | `theory/discovery-results.md` |

---

## 8. Decision log (this companion)

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-07-03 | **`Refiner<G>` lives in `ufl-search`, not `ufl-evolve`.** | SPEC-0014 §2.1 (Accepted) put the pure seam in `ufl-search`; T8's "traits move to `ufl-evolve`" predates that and is superseded. The refiner is a substrate seam beside `Proposer`/`Fitness`. |
| 2026-07-03 | **The engine holds the hill-climb; the refiner only proposes neighbors and never scores.** | Answer-blindness: if the proposer/refiner scored, it would see the target, breaking Verifier-Held Transparency (SPEC-0014 §2.4). Same structural move as `Screen`. |
| 2026-07-03 | **Slots address grade-{0} `Param` payloads only; the typecheck-invariant is by construction.** | `grade`/`typecheck` never read a `Param` value (grade.rs:87); so refinement through slots cannot change the verdict — a type-level fact, not a runtime guard. First typed quotation site for R-0015. |
| 2026-07-03 | **Geometric fitness is a *cost* (lower=better) on the real `Fitness<G,S>` seam; NaN-safety lives in `S`.** | The real `run_generic` minimizes `S: Ord+Copy`; SPEC-0011's `Fit`-maximize snippet predates the SPEC-0014 seam. Faithful translation, flagged (§0.2). |
| 2026-07-03 | **`magnitude = √Σ coeff²` over all 16 blades, not `Mv::norm()`.** | garust's metric `norm()` zeros `e₀`-blades (SPEC-0010 trap; `discovery-results.md`), hiding null-blade error. Load-bearing pilot finding. |
| 2026-07-03 | **`ufl-geo` gains a `ufl-search` dep to host `GradeScreen`/`GeoParamRefiner`.** | SPEC-0014 §2.1's lane arrow (`ufl-geo → ufl-search`); `ufl-search` stays `ufl-prng`-only so no cycle. Alternative (instances in `ufl-evolve`) in §9. |
| 2026-07-03 | **`cargo tree -p ufl-discovery` no-geo/ga edge promoted to a CI gate.** | Architect suggestion: make the pure-engine topology a merge-blocking machine invariant, not a convention that rots on the next stray `use`. |

---

## 9. Open questions

1. **Where do `GradeScreen`/`GeoParamRefiner` live — `ufl-geo` or `ufl-evolve`?**
   §2.3 places them in `ufl-geo` (per SPEC-0014 §2.1's lane arrow), which adds a
   `ufl-geo → ufl-search` dep. The alternative — defining them in `ufl-evolve`
   (which already deps both) and keeping `ufl-geo` free of `ufl-search` — trades
   the extra edge for a slightly less "the lane owns its screen" story. **Needs
   Gustavo's call before the three-lens; T8/SPEC-0014 lean toward `ufl-geo`.**

2. **Exact garust coefficient read for `√Σ coeff²`.** Whether `garust::Pga3`
   exposes a direct 16-coefficient accessor or the magnitude is assembled by
   summing `grade(k)`-projected parts (`k∈0..=4`) — confirmed against the pinned
   garust rev at the code-outline step (§3). Does not change the spec's
   *requirement* (magnitude over blades, not `norm()`), only its spelling.

3. **`run_memetic` vs. `run_generic` + external pass.** §2.1 specifies a new
   `run_memetic`. An alternative keeps `run_generic` untouched and exposes a
   `refine_population` helper the caller interleaves — smaller substrate surface,
   but pushes the answer-blind hill-climb discipline onto every caller. §2.1's
   in-engine version is preferred (the discipline lives in one place); confirm at
   three-lens.

4. **Refinement neighborhood shape + `MemeticConfig` defaults.** Per-slot jitter
   vs. joint all-slots jitter; `elites`/`steps`/`sigma` values. The pilot used
   elite `Param`-refinement; exact defaults are set with qa against the ≥6/16 bar
   (they are a means to the gate, not the gate).

5. **Does refined-elite splicing perturb the `SplitMix64` draw order enough to
   shift the tree-GA trajectory vs. a hypothetical pre-memetic baseline?** The
   `NoRefine` collapse test (T-memetic-collapse) pins byte-identity when
   refinement is *off*; when *on*, the extra `rng` draws in `neighbors` are part
   of the (still deterministic) memetic run. Confirm the determinism test seeds
   the refiner from the engine `rng` (not a fresh one) so a fixed `seed`
   reproduces the whole memetic trajectory.

---

## 10. Changelog

- 2026-07-03 — created (Draft). Extends Accepted SPEC-0011 with the memetic
  design (the `Refiner` seam, the typed param-slots, the geometric engine
  instances) against the real `ufl-search` (SPEC-0014) seam. Reconciled the T8
  "traits in `ufl-evolve`" text with SPEC-0014's `ufl-search` topology (§0).
  Awaiting three-lens (architect / hater / nice-guy) and Gustavo's acceptance.
