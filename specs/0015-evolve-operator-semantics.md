# SPEC-0015 — Evolve operator semantics (the Rung-4 probe)

- **Realizes:** [R-0015](../requirements/0015-evolve-operator-semantics.md)
  (Accepted) — AC1 (the move-form DSL), AC2 (the pre-registered three-set gate),
  AC2b (the headroom window), AC3 (the kill-criterion), C1–C7.
- **Status:** **Draft — revised 2026-07-15 after the three-lens.** The hater's
  mandatory pre-run *empirically refuted* the original ⟨2,2,2⟩→7 family (B0 solves
  0/200 at budget ≤ 25,600 — no headroom window); Gustavo chose the planted-instance
  path; the redundancy-scramble family's window was then **measured** (censoring
  0.79 at splits=2, ~13 ms/search). This revision folds all architect/hater/nice-guy
  findings (see §12). **Re-review (architect + hater) pending on this revision.**
- **Milestone:** the self-eval staircase, **Rung 4** — the *decision node*. A
  positive earns Rung-5 (the Lisp substrate); an *informative* negative kills it
  and redirects to object-level scaling + the reflection line (§11's decision tree
  — a non-informative "no window" re-scopes rather than kills).
- **Crate(s):** `crates/ufl-discovery` — the MoveForm DSL, its interpreter, the
  meta-loop harness, and the statistics live **with the matmul lane** (the
  primitives they compose are `ufl_discovery::flipgraph`). **No new engine:** the
  meta-loop is a second instance of `ufl_search::run_generic` (R-0015 requirement).
- **Depends on:** SPEC-0013 (the flip-graph move primitives), SPEC-0014 (the
  hardened `run_generic` seam + the §2.5 meta-fitness definition + the §2.7
  constructive closure rule), SPEC-0011M (the answer-blind-seam discipline).

---

## 0. What this probe is, in one paragraph

The 2026-06-29 interrogation proved evolving the GA's **hyperparameters** is
headroom-free; the matmul lane proved the proposer **family** decides everything.
R-0015 is the bet that the metacircular win, *if it exists*, lives in operator
**semantics** — the structure of the search *move*. This spec builds the
falsifiable test: an outer `run_generic` whose genome is a **move-form** (a term
in a bounded typed DSL over the committed flip-graph primitives), whose fitness is
*how few verifier-work units the inner flip-graph search using that form needs to
crack held-out matmul targets*, and a **pre-registered three-disjoint-set gate**
that an evolved form must clear by ≥2 SE on each to count as a real beat. The
**non-sequitur caveat is load-bearing** (R-0015): this probe has zero positive
efficacy evidence going in — it exists to manufacture that evidence or the
documented negative. Both outcomes close the gate; both are results (AC3).

---

## 1. The design at a glance (the three real pieces + the split)

```
                 ┌──────────────────────────────────────────────────────┐
   META-LOOP     │  run_generic< G = MoveForm, S = MetaCost >           │
   (2nd          │    proposer = FormProposer  (form mutations)         │
    run_generic) │    fitness  = MetaFitness   (runs inner searches on  │
                 │               the DEV set; holds RankDecomposition)  │
                 │    screen   = NoScreen      (closure is structural,  │
                 │               SPEC-0014 §2.7)                        │
                 └───────────────────────┬──────────────────────────────┘
                                         │ each MetaFitness::score(form):
                                         ▼
                 ┌──────────────────────────────────────────────────────┐
   INNER SEARCH  │  interpret(form): a flip-graph driver ≡ in power to   │
   (per form,    │  reduce_matmul_with — composes ONLY the public        │
    per task)    │  primitives (shared_factor_pairs / flip_at / reduce / │
                 │  perturb) + rng. NEVER touches RankDecomposition (C1).│
                 │  Returns moves-to-solve, right-censored at budget B.  │
                 └──────────────────────────────────────────────────────┘

   THE SPLIT (disjoint by BASE-TENSOR + seed-block construction — §5):
     DEV  ── the meta-fitness signal (forms are optimized against this)
     K1, K2, K3 ── three confirmation sets on DISTINCT base tensors (the GATE;
                   never seen in search) → tests structure-generalization
```

Nothing here scores itself; the verifier is unreachable from every proposer; the
reward is the exact verifier verdict; the operator space is bounded; verification
is cheap relative to search; improvement is a measured delta on held-out sets;
every accepted form replays deterministically. (C1–C7, checked in §8.)

---

## 2. The MoveForm DSL (AC1, C4)

A **closed, depth/size-capped** grammar. Every constructor composes the committed
`ufl_discovery::flipgraph` primitives; the interpreter (§3) is the only thing that
executes them.

```rust
// crates/ufl-discovery/src/moveform.rs
use crate::flipgraph::Variant;

/// A search-move policy — a term in the bounded DSL (SPEC-0015 §2). Every
/// reachable state under any form is tensor-exact by construction (§3 closure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveForm {
    /// The atomic step: draw one uniform flip from the frontier, then `reduce`.
    /// (This is exactly `reduce_matmul_with`'s inner step.)
    FlipReduce,
    /// Draw one uniform flip with **no** reduce (let coefficients grow toward
    /// the envelope — a distinct exploration move; still tensor-exact).
    FlipRaw,
    /// Bias the frontier draw toward a shared-slot `Variant` (U/V/W), falling
    /// back to a uniform draw when that variant is absent, then `reduce`.
    FlipBiased(Variant),
    /// Kick the **best-so-far checkpoint** by `k` un-reduced flips (`perturb`).
    /// `k` is bounded `1..=KICK_MAX`.
    Kick(u8),
    /// Run `body` until `stall` steps pass with no strict best-rank drop, then
    /// run `on_stall` once; repeat. `stall` bounded `1..=STALL_MAX`.
    Walk {
        stall: u16,
        body: Box<MoveForm>,
        on_stall: Box<MoveForm>,
    },
    /// `first` for `n` steps, then `second` (bounded `n`).
    Seq {
        n: u16,
        first: Box<MoveForm>,
        second: Box<MoveForm>,
    },
    /// With probability `p/256`, `a`, else `b` (one `rng.below(256)` draw).
    Choose { p: u8, a: Box<MoveForm>, b: Box<MoveForm> },
}
```

**Caps (C4 — pinned constants, `moveform.rs`):** `DEPTH_MAX = 5`,
`NODES_MAX = 24`, `KICK_MAX = 32`, and **`STALL_MAX` and `SEQ_MAX` both clamped to
the inner budget `B`** (hater 10: a `Walk.stall` or `Seq.n` larger than `B` can
never fire its `on_stall`/`second` branch, so any value `> B` is behaviourally
identical dead space that wastes meta-search — clamping to `B` makes every knob
value behaviourally distinct). `well_formed(&MoveForm) -> bool` (depth + node count
≤ caps **and** every `stall`/`n ≤ B`) is asserted at every meta-loop entry, so an
out-of-cap form is a test failure, not a silent run.

**B0 is one point in the DSL (AC1 — within-harness fairness):**

```rust
// The hand-written Gate-0 policy (FlipConfig::pinned() = {stall_window: 400,
// perturb_flips: 6}) expressed verbatim as a MoveForm:
MoveForm::Walk {
    stall: 400,
    body:     Box::new(MoveForm::FlipReduce),
    on_stall: Box::new(MoveForm::Kick(6)),
}
```

`B0` is a named constant `fn b0() -> MoveForm`. The interpreter running `b0()`
reproduces the committed **pinned flip-graph loop byte-identically** — verified
against an inline reference driver (§7 test 1), since `reduce_matmul_with` returns
only a `Scheme`, not a move count. So the meta-search competes against the *real*
baseline as a **strict specialization of its own search space**, not a
re-implementation that could drift (nice-guy: fairness by construction).

---

## 3. The interpreter (answer-blind; constructive closure)

```rust
// crates/ufl-discovery/src/moveform.rs
use crate::flipgraph::{naive, reduce, IntScheme};
use ufl_prng::SplitMix64;

/// The outcome of interpreting a form against one task, to a step budget `B`.
pub struct InnerRun {
    /// Steps until the first internally-solved state (rank ≤ target ∧ ternary),
    /// or `None` if the budget was exhausted first (right-censored — §2.5).
    pub moves_to_solve: Option<usize>,
    /// The best (lowest) rank reached — the meta-fitness tie-break.
    pub best_rank: usize,
    /// The candidate at the solving state, for the meta-fitness to CERTIFY
    /// through the real verifier (C3). `None` when censored.
    pub candidate: Option<IntScheme>,
}

/// Interpret `form` as a flip-graph driver on `T_n → target_rank`, to budget `B`.
/// **Answer-blind (C1):** composes only `shared_factor_pairs`/`flip_at`/`reduce`/
/// `perturb` + `rng` — it holds no `RankDecomposition` and no target rank beyond
/// the integer `target_rank` it is walking toward (public from the task, not the
/// answer). The internal solved-check is `rank ≤ target_rank ∧ is_ternary` — a
/// SOUND stopping heuristic because every state is tensor-exact by construction
/// (SPEC-0013), so it can never *false-positive* a solve; the meta-fitness still
/// re-certifies through the verifier (C3).
pub fn interpret(
    form: &MoveForm,
    n: usize,
    target_rank: usize,
    seed: u64,
    budget: usize,
) -> InnerRun { /* … §3.1 … */ }
```

### 3.1 Interpretation semantics (the driver)

The interpreter is an explicit **small-step machine** mirroring the committed
reference loop `reduce_matmul_with` (`flipgraph.rs`, replicated by the AC1 test
against `tests/r_0013_flipgraph.rs`'s driver). It maintains global state `(s:
IntScheme, best: IntScheme, moves: usize, rng)` plus **persistent per-node control
state** on the form tree (each `Walk` owns a `stall` counter, each `Seq` owns a
phase counter), and it does exactly this:

**Initialization** (mirrors `reduce_matmul_with`): `s = reduce(planted_start)`,
`best = s`, `moves = 0`; pre-loop solved-check — if `s.rank() ≤ target_rank ∧
s.is_ternary()`, return `moves_to_solve = Some(0)`.

**One budget unit** (repeated while `moves < budget`), which is `walk(form)` where
`walk` returns after executing **exactly one leaf primitive**:
- **Leaf `FlipReduce`/`FlipRaw`/`FlipBiased(v)`:** compute the frontier
  `pairs = shared_factor_pairs(&s)`. **If `pairs` is empty, the step draws *no*
  `rng`** (byte-identity clause — an empty-frontier step must not desync the
  stream) and is a no-op leaf. Else draw one index `rng.below(pairs.len())` (for
  `FlipBiased`, restrict to `v`-tagged pairs first, uniform fallback), `flip_at`,
  then `reduce` (skip `reduce` for `FlipRaw`). Exactly one `below` draw on a
  non-empty frontier — *the draw lives in the interpreter's own frontier pick, not
  in `flip_at`* (which is a pure function — hater 9).
- **Leaf `Kick(k)`:** `s = perturb(&best, k, &mut rng)` — this is a **tail action**
  (below), not a standalone budget unit.
- **`Walk { stall, body, on_stall }`:** `walk` recurses into `body` for one leaf.
  *Then* (this ordering is load-bearing — architect A): run the **solved-check and
  the `best`/`stall` update**; only *after* that, if `stall` has reached its bound,
  run `on_stall` **once as a tail** (a `perturb`/kick applied to `best`) and reset
  this node's `stall` — all within the *same* budget unit, exactly as the reference
  does flip → solved-check → stall-update → gated kick.
- **`Seq { n, first, second }`:** `walk` recurses into `first` while this node's
  phase `< n`, else into `second`; the phase counter **persists across budget
  units**. `Choose { p, a, b }`: one `rng.below(256)` per budget unit; `< p` ⇒ walk
  `a`, else `b`.

**After each budget unit** (post-leaf, and the reference's exact order): if `s.rank()
≤ target_rank ∧ s.is_ternary()`, record `moves_to_solve = Some(moves)` and the
candidate and stop; else update `best` (strict `rank()` drop resets every enclosing
`Walk.stall`), `moves += 1`. `b0()`'s single top-level `Walk` collapses this machine
to the reference loop line-for-line (the AC1 byte-identity test).

**Determinism contract (C7, mirrors SPEC-0013 §2.4):** every `rng` draw originates
in the interpreter's own frontier pick (`below(pairs.len())`, only on a non-empty
frontier), `perturb`'s internal draws, or the `Choose` branch draw — nothing else;
the meta-harness pins the per-`(form, task)` seed, so every accepted form replays
exactly.

### 3.2 Closure is a structural induction, not a runtime ∀ (SPEC-0014 §2.7)

> **Claim.** For every `MoveForm` within the caps, every state the interpreter
> visits reconstructs to `T_n` exactly.

**Proof (finite, by structure).** *(base)* `FlipReduce`/`FlipRaw`/`FlipBiased`/
`Kick` compose only `flip_at`, `reduce`, `perturb` — each **tensor-preserving by
construction** (SPEC-0013 §2.3, the sum-invariant proofs). `naive(n)` reconstructs
`T_n` by definition. *(step)* `Walk`/`Seq`/`Choose` only *select which* primitive
runs next — they introduce no new state transformation — so if their sub-forms
preserve the tensor, so do they. ∎ There is **no infinite quantifier over
outputs** (the SPEC-0014 §2.7 correction); closure is discharged once, here, and
re-checked by the T-closure fuzz (§7). Because the lane is `NoScreen` and every
form is closure-admissible, the meta-loop needs **no runtime screen**.

---

## 4. The meta-loop (a second `run_generic`, no new engine)

```rust
// crates/ufl-discovery/src/rung4.rs
use ufl_search::{run_generic, Fitness, NoScreen, Proposer};

/// The meta-cost: a MoveForm's aggregate verifier-work over a task set, as a
/// `Copy + Ord` total order (lower = better). `sum_moves` is Σ over the set of
/// per-task moves-to-solve right-censored at `B`; `unsolved` and `sum_best_rank`
/// are documented tie-breaks (SPEC-0014 §2.5).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetaCost {
    sum_moves: u64,      // primary: total censored moves-to-solve
    sum_best_rank: u64,  // tie-break: Σ best_rank_reached (only when censored)
}
// `run_generic` needs `S: Ord + Copy`; the derived lexicographic order over
// (sum_moves, sum_best_rank) — fields declared in that order — IS the intended
// order, so no hand impl (architect B).
```

- **`FormProposer: Proposer<MoveForm, MetaCost>`** — `seed` samples random
  well-formed forms within the caps, **always including `b0()`**. **`vary` is
  elitist (load-bearing — architect B / hater / nice-guy):** `run_generic` returns
  the **final-generation** best, not the best-ever, so `vary` **must carry the
  top-`e` cost-ranked forms forward unchanged** (like `GaProposer::vary`'s
  `take(elitism)`, `proposer.rs`) — otherwise a form better than `b0()` found
  mid-run can be *lost*, `f*` can regress below `b0()`, and — because case 3 of §11
  permanently kills Rung-5 — a **search bug could close the research line**.
  Elitism (`e ≥ 1`, with `b0()` never displaced from the elite slot) makes
  "final-gen-best = best-ever ≤ `b0()`" hold, so a negative genuinely means "no form
  beat `b0` on held-out", not "the elite drifted out". The rest of `vary` mutates
  ranked parents (perturb a knob, swap a combinator, grow/shrink a subtree) —
  **always re-clamped to the caps**. Answer-blind: `MoveForm` structure + `rng` only.
- **`MetaFitness: Fitness<MoveForm, MetaCost>`** — `score(form)` runs
  `interpret(form, ..)` on **every task in the DEV set** at budget `B`, and for
  each *claimed* solve **certifies the candidate through
  `RankDecomposition::for_target` at the reached rank (C3 — the reward is the exact
  verifier verdict, never the internal heuristic; §5.2)**; a solve the verifier
  rejects is treated as censored (guarded defensively). Returns the aggregate
  `MetaCost`. `type Error = FlipError` (it `#[from]`-absorbs `SchemeError`; inert in
  practice). `solved` returns
  **`false` always** — the meta-loop has no "win" state; it minimizes and returns
  the best form via `Exhausted` (the gate, §5, is evaluated *outside* the loop on
  the confirmation sets, per C6).
- **`screen = NoScreen`** — closure is structural (§3.2).

The meta-loop is literally `run_generic(&FormProposer, &MetaFitness, &NoScreen,
meta_generations, meta_seed)`. Its `Ledger.evals` counts `MetaFitness::score`
calls (= forms evaluated); the *inner* verifier-work is inside `MetaCost`.

**Why the verifier lives in `MetaFitness`, not the interpreter (C1 + C3
together):** the interpreter (the *proposer* of trajectories) never holds
`RankDecomposition`; the `MetaFitness` (the *scorer*) does, and certifies each
solve. So the verifier is unreachable from every proposer (C1) **and** the reward
is the exact verdict (C3) — the same split SPEC-0011M used for the memetic refiner.

---

## 5. The task family, the headroom window, and the gate (AC2, AC2b)

### 5.1 The task family — planted redundancy-scramble instances (structurally varied)

**A task = `(base, scramble_seed)`.** A **structured base scheme** — a flip-able,
exactly-reducible decomposition of a known tensor at rank `r` — is **scrambled** by
a fixed-depth sequence of redundancy-injecting moves (split one triple `(a,b,c)`
into `(a,b,c1),(a,b,c2)` with `c1+c2=c`; then `perturb` a few flips off the shared
axis), yielding a start scheme of rank `r + splits` whose **tensor is unchanged**.
The inner search's job: **reduce it back to rank `r`** — undo the planted
redundancy. A move-form is a *policy* for finding that reduction, and the target is
"remove what was added" — identical *in kind* across tasks — so a form **cannot
memorize** a specific tensor (the C1 answer-blindness the whole probe rests on).

**Why not ⟨2,2,2⟩→7 directly** (the refuted first design — §9.1, this was replaced
after the three-lens ran the pre-run): *measured*, B0 solves the rank-7 needle on
only **~1.5% of seeds at budget 300k and 0/200 at every budget ≤ 25,600**. The
difficulty is **bimodal in rank** (rank-8 trivial, rank-7 a needle, nothing
between), so **no budget interpolates to a headroom window**. Redundancy-scramble
replaces the needle with a *tunable, continuous* difficulty knob (the scramble
depth).

**Structure variety — the confirmation sets test structure-generalization, not
seed-generalization (hater finding 5).** Base schemes are **constructed** (a small
set of shared factor vectors → `r` triples reusing them → a flip-able rank-`r`
tensor), so distinct base seeds are **genuinely different tensors**, not
seed-resamplings of one. Each confirmation set `Ki` draws its tasks from a
**disjoint block of base tensors**, so clearing all three means the form
generalizes across *structure* — the property R-0015 actually asks about, and the
one a single tensor (the earlier probe) provably cannot test.

### 5.2 The headroom window (AC2b — MEASURED, not assumed)

The knob is the **scramble depth** (`splits`, `kicks`) at a fixed inner budget `B`.
Measured (B0 = `FlipConfig::pinned()` reducing a scrambled rank-7 `T_2` base back
to rank 7, 100 scramble seeds — the mechanism's existence proof):

| scramble | inner budget | B0 solve rate | censoring | per-search |
|---|---|---|---|---|
| splits=1, kicks=2 | 2,000 | 0.83 | 0.17 | — |
| **splits=2, kicks=2** | **4,000** | **0.21** | **0.79 ✓** | **~13 ms** |
| splits=2, kicks=3 | 5,000 | 0.21 | 0.79 ✓ | ~13 ms |
| splits≥3 | 8,000+ | 0.00 | 1.00 | — |

`splits=2` sits at censoring **0.79 ∈ [0.2, 0.8]** — a real window — and each search
costs **~13 ms** (vs the refuted lane's 0.2–1 s), so the full meta-loop
(pop×gens×|DEV| ≈ 328k searches) runs in **~70 min**, not the days the ⟨2,2,2⟩→7
budget implied. This discharges **hater finding 2 / C5** with a measured number.

> **PRE-RUN (mandatory, before arming):** for **each** base tensor, sweep
> (`splits`, `kicks`, `B`) and pin the point whose B0 **censoring rate ∈ [0.2,
> 0.8]** (moves-to-solve right-censored at `B`, SPEC-0014 §2.5). A base with **no**
> such point is **dropped** — this is *not* a project kill (§11). Arming requires a
> demonstrated window on **≥ 2 structurally-distinct bases**, so the three-set gate
> genuinely varies structure. The pre-run also measures the actual **paired
> B0-vs-B0′ delta correlation** (feeds §5.3). Its full table is recorded in the PR.
> *(The `T_2` row above is the measured existence proof; the multi-base sweep is the
> arming precondition.)*

**The `≤`/`==` rank fix (architect C / hater 8).** On a planted base of known rank
`r`, `r` is reachable by construction (the base *is* a rank-`r` decomposition), and
a form may find `rank < r`. The internal solved-check accepts `rank ≤ r ∧
is_ternary`, and the meta-fitness certifies via `RankDecomposition::for_target(
tensor, reached_rank)` **at the rank actually reached** — so a *better* (lower-rank)
solve is certified, never mis-scored as a failure. The old "impossible by §3.2"
justification was wrong (§3.2 proves tensor-preservation, not rank-exactness) and
is removed.

### 5.3 The gate (AC2 — three disjoint sets, ≥2 SE each, correction = conjunction)

After the meta-search returns its best form `f*` (optimized against DEV only), the
gate runs **`f*` and `b0()` on each of the three confirmation sets** `K1,K2,K3`:

- **Paired statistic per set `Ki`:** for each task `t ∈ Ki`, the paired delta
  `d_t = moves_{B0}(t) − moves_{f*}(t)` (censored moves; positive ⇒ `f*` better).
  `D̄_i = mean_t d_t`, `SE_i = sd(d_t)/√|Ki|` with **sample sd (n−1)** — pinned, it
  moves a 2-SE bar (hater 12). Pairing (same tasks both arms) reduces
  task-difficulty variance, but the arms **do not share a walk** (`f*` and `b0`
  diverge at the first frontier draw), so the cancellation is *partial* — the
  pre-run measures the actual `Cov(d_t)` and the §6 `K` is set from it, not from an
  assumed perfect pairing (hater 7).
- **Why three sets over one structure would be worthless, and why these aren't
  (the exchangeability premise — hater 5):** because the confirmation sets draw
  **different base tensors**, `K1/K2/K3` are three tests of *different* structure —
  so a form that exploits one tensor's flip-graph quirk fails the others. (Three
  seed-splits of *one* tensor, the refuted design, would be positively correlated
  and the joint-FPR argument would collapse.)
- **The null is CALIBRATED, not assumed (hater 6, the load-bearing statistics
  fix).** `moves_to_solve` is right-censored with a point mass at `B`
  (both-censored ⇒ `d_t = 0`), so the paired-delta distribution is a heavy-tailed
  discrete mixture — the normal-theory "`2·SE ⇒ FPR ≈ 0.023`" mapping does **not**
  hold. Before arming, run **`b0()` vs `b0′()`** (the same policy, a different
  run-seed block) through the *entire* three-set gate and **measure the actual
  "clears all three" rate under the null.** The pre-registered margin is set so the
  measured null pass-rate is ≤ 1×10⁻⁴; if 2-SE-on-each does not achieve that, widen
  the margin or switch to a sign/permutation test on the non-zero deltas. The
  "≈1.2×10⁻⁵ under independence" figure is dropped as an unproven normal-theory
  number.
- **Decision rule:** the gate **PASSES** iff `D̄_i ≥ margin_i` for **all three**
  `i` *simultaneously* (an intersection-union test — every set must independently
  clear the calibrated bar). This is exactly the protocol whose absence let the
  earlier probe's winner survive two splits and flip to −14/200 on the third
  (R-0015 AC2). A beat on one or two sets but not all three is a **documented
  negative**, not a pass.

Everything in §5 — the base tensors, `splits`/`kicks`/`B`, `K`, the seed blocks,
the calibrated margin — is **frozen in §6 before the meta-search runs** (C6). The
gate asserts `f*` was never scored on any `Ki` base or seed (C2).

---

## 6. Pre-registration table (frozen before the run — proposed values)

| Parameter | Symbol | Proposed value | Rationale |
|---|---|---|---|
| Task family | — | planted redundancy-scramble (§5.1) | tunable window (measured); structure-varied |
| Base tensors | `bases` | ≥ 6 constructed structured tensors (rank `r≈7`, dim 4), split 2 DEV / 4 confirm | genuinely different structures (hater 5) |
| Scramble depth | `splits`,`kicks` | **2, 2** (pinned by pre-run per base) | measured B0 censoring 0.79 ∈ [0.2,0.8] |
| Inner step budget | `B` | **4,000** (pinned by pre-run) | window knob; ~13 ms/search → meta ≈70 min |
| Tasks per set | `K` | 200 (per set; `K` re-derived from the pre-run's measured `Cov(d_t)`) | resolves a ≥2 SE beat at the measured effect size |
| DEV (meta-signal) | — | bases `{b0,b1}`, scramble-seed block `[0,200)` | the only set the meta-search sees |
| Confirm set K1 | — | base `b2`, scramble seeds `[1000,1200)` | disjoint base **and** seeds |
| Confirm set K2 | — | base `b3`, scramble seeds `[2000,2200)` | disjoint base **and** seeds |
| Confirm set K3 | — | bases `{b4,b5}`, scramble seeds `[3000,3200)` | disjoint base **and** seeds |
| Meta-population | — | 40 forms | the outer `run_generic` population |
| Meta-generations | — | 40 | outer budget; `b0()` **carried forward elitistically** (§4) |
| Meta-seed | — | 20260715 | the one outer seed (C7 replay) |
| Margin | `margin_i` | **calibrated** so measured null (`b0` vs `b0′`) "clears all three" ≤ 1e-4 (§5.3); ≥2 SE is the starting point | AC2 — empirical, not normal-theory |
| Correction | — | intersection-union: all three sets clear `margin_i` | AC2 multiple-comparisons |
| DSL caps | — | DEPTH 5 / NODES 24 / STALL `≤ B` / KICK 32 / SEQ `≤ B` | C4 bound (STALL/SEQ clamped to `B` — hater 10) |

*(Values are the spec's proposal; the three-lens + Gustavo freeze them at
acceptance. The `splits`/`kicks`/`B` and `margin` cells are **filled by the §5.2/§5.3
pre-run and recorded before arming** — the T_2 row in §5.2 is the measured evidence
the mechanism produces a window.)*

---

## 7. Tests (TDD — written first, red)

`crates/ufl-discovery/tests/r_0015_rung4.rs` (fast lane) + the `#[ignore]` probe:

1. **`b0_form_replays_the_reference_loop_byte_identically`** (AC1, C7): the oracle
   is an **inline reference loop** (the `tests/r_0013_flipgraph.rs` driver body),
   not `reduce_matmul_with` (which returns only a `Scheme`, not a move count —
   architect A minor). `interpret(b0(), planted_start, r, seed, B)` yields the
   *same* `moves_to_solve`, `best_rank`, and certified scheme as driving
   `shared_factor_pairs`/`flip_at`/`reduce`/`perturb` directly with the same
   `SplitMix64` — over a seed block, on both a solved and a censored task (reuses
   SPEC-0013 §2.6.5's replay discipline; this is what makes B0-in-the-space a strict
   specialization, not an audited re-implementation — nice-guy).
2. **`every_form_preserves_the_tensor`** (§3.2 closure): fuzz well-formed forms ×
   seeds; after **every** interpreter step, `reconstruct_int(s) == target_int(2)`.
   The runtime witness of the structural-induction proof.
3. **`interpreter_never_touches_the_verifier`** (C1): a compile-level guard —
   `moveform.rs` has no `use`/mention of `RankDecomposition`; asserted by a
   `grep`-style test over the module source (like the topology gate) + the type
   signature carrying no verifier.
4. **`meta_fitness_certifies_every_claimed_solve`** (C3): a spy `RankDecomposition`
   wrapper counts discharges; a run asserts every `moves_to_solve.is_some()` was
   discharged `Ok(true)` before counting as solved.
5. **`splits_are_disjoint_by_construction`** (C2): the four seed blocks (task and
   run) have empty pairwise intersection — a set-membership test over the frozen
   ranges.
6. **`se_matches_a_hand_computed_example`**: paired deltas `[3, −1, 4, 0, 2]` →
   `D̄ = 1.6`, **sample `sd` (n−1) = 1.9494…, `SE = sd/√5` (n−1 pinned in the spec
   and the test — hater 12)**; assert the harness's `(D̄, SE)` equal the hand values
   (guards the statistics code, the thing most likely to be subtly wrong).
7. **`well_formed_rejects_out_of_cap_forms`** (C4): a depth-6 / 25-node form, and a
   `Walk.stall > B` form, are `well_formed == false`; the `FormProposer` never emits
   one (fuzz its `vary`).
8. **`vary_is_elitist`** (architect B): after any `vary`, the top-`e` parents by
   `MetaCost` appear unchanged in the children, and `b0()` is never displaced from
   the elite set — the "never worse than baseline" guarantee, as a test not a hope.
9. **`the_probe`** (`#[ignore]`, release) — the experiment, with the runbook in a
   doc-comment: the §5.2 multi-base window pre-run (record the sweep) → the §5.3
   **null calibration** (`b0` vs `b0′` through the whole gate; record the measured
   "clears all three" rate; freeze `margin_i`) → freeze `splits`/`kicks`/`B` →
   meta-search on DEV → the gate on K1,K2,K3. It prints: the per-set `(D̄_i, SE_i,
   pass?)` table, the PASS/NEGATIVE verdict per §11's tree, `f*`'s rendered form,
   **`f*`'s mutation lineage from `b0()`** (parent-pointers through the meta-loop —
   makes a negative say "the search explored these forms; none generalized"), and
   **the paired `moves_{B0}` vs `moves_{f*}` per-task distributions** on each `Ki`
   (the money plot — already computed inside the gate; nice-guy 1). **The committed
   assertion is not "PASS"** — the test asserts only that the gate *ran on all three
   disjoint, structurally-distinct sets after a calibrated null* and recorded a
   verdict (assert-the-process, not the outcome — the un-p-hackable discipline).

---

## 8. The seven non-negotiables, discharged (C1–C7)

| # | Guarantee | How this spec secures it |
|---|---|---|
| C1 | verifier unreachable from proposer | `interpret`/`FormProposer` hold no `RankDecomposition` (test 3); only `MetaFitness` does (§4) |
| C2 | held-out scoring | DEV drives the search; K1/K2/K3 are disjoint by **base + seed-block** construction (test 5); the gate asserts no `Ki` base or seed was searched |
| C3 | reward = exact verdict | `MetaFitness` certifies every claimed solve through `RankDecomposition::for_target` at the reached rank (test 4) |
| C4 | bounded operator space | typed grammar + `DEPTH/NODES/KICK` caps and `STALL/SEQ ≤ B`; `well_formed` asserted (test 7) |
| C5 | verification cheap vs search | one `for_target` discharge per *solve*, O(d³) reconstruct at d=4 (trivial); the search dominates — **measured ~13 ms/search at B=4,000, so the full meta-loop ≈ 70 min** (hater 2: the tractability lives in the search cost, which the planted window keeps cheap — the refuted ⟨2,2,2⟩→7 lane was 0.2–1 s/search ⇒ days) |
| C6 | improvement = measured held-out delta | the gate (§5.3) is computed *outside* the loop on frozen confirmation sets; `MetaFitness::solved` never fires |
| C7 | traceable lineage + replay | every seed pinned (§6); `b0()` replays byte-identically (test 1); `MoveForm: PartialEq` + the meta-seed reproduce any accepted form |

---

## 9. Deviations from R-0015's sketch (flagged for the three-lens)

1. **The task family is planted redundancy-scramble instances — chosen *after* the
   three-lens empirically refuted the first design.** The original draft used one
   tensor (⟨2,2,2⟩→7) at a tight budget. The hater ran the mandatory pre-run against
   the committed primitives and **measured B0 solving 0/200 at every budget ≤ 25,600
   and ~1.5% at 300k** — the difficulty is bimodal in rank, so *no* budget opens a
   window, and one tensor tests only seed-generalization. Both were verified in the
   main session. §5.1's redundancy-scramble family fixes all three: a **measured**
   tunable window (splits knob, censoring 0.79 at splits=2), **tractable** (~13
   ms/search, ~70 min meta-loop), and **structurally varied** (constructed base
   tensors → the confirmation sets test structure-generalization). It reuses the
   *already-general* primitives (`shared_factor_pairs`/`flip_at`/`reduce`/`perturb`
   operate on any `IntScheme`); only a `from_triples` constructor + the
   scramble/plant helpers are new — a bounded addition, not the "substantial general
   `for_target` flip-graph" the original draft feared. **Residual risk the
   three-lens must still weigh:** the constructed base tensors must be genuinely
   flip-able and structurally diverse — the multi-base pre-run (arming precondition)
   demonstrates this or the base is dropped (§11 case 1).
2. **Metric = moves-to-solve (SPEC-0014 §2.5), with AC2b's "success rate ∈ (0,1)"
   realized as "B0 censoring rate ∈ [0.2, 0.8]."** The continuous moves-to-solve
   statistic has smaller SE than a Bernoulli success rate and matches §2.5; the
   censoring-rate window is the saturation guard AC2b actually wants. Flagged in
   case the three-lens reads AC2b as mandating a binary success-rate metric.
3. **DEV *is* the meta-loop's train set — the overfit is real, in the selection
   sense (architect note).** The *inner* search has no per-task fit (a MoveForm is a
   policy), but the *meta* loop **selects `f*` to minimize DEV cost**, so it overfits
   DEV exactly as any train-set selection does — which is *precisely* what the
   three-disjoint-structure gate exists to catch. R-0015 C2's "train/holdout" maps
   to DEV/{K1,K2,K3}. If the three-lens judges DEV too small to expose selection
   overfit before the gate, the fix is to enlarge DEV / re-draw it per
   meta-generation (at a determinism cost, C7); called out so they can push.
4. **The meta-loop's `solved` never fires.** `run_generic` is built to stop on a
   solution; the meta-loop has none, so it always `Exhausted`s at
   `meta_generations` and returns the best form. This is a benign use of the seam
   (a pure minimizer), but it means the meta-loop always spends its full budget —
   flagged as an intended, not accidental, cost.

---

## 10. Open questions for the three-lens

1. **RESOLVED (2026-07-15):** ⟨2,2,2⟩→7 was refuted by the pre-run; the family is
   now planted redundancy-scramble with a *measured* window (§5.1, §9.1). Remaining
   sub-question for re-review: are the constructed base tensors diverse enough that
   clearing three of them is real structure-generalization?
2. **The DSL's expressive ceiling — and the greedy move (nice-guy 2, promoted).**
   Add a **rank-descent-greedy `FlipBiased`** variant (pick the frontier pair that
   most reduces rank) as an opt-in constructor *before arming*: it is a
   structurally *different* policy, so it converts a possible weak negative ("no
   beat because the DSL ceiling *was* `b0`") into a **strong** negative ("no beat
   despite a genuinely richer move in the space") — and raises the chance a positive
   exists. Its C5 cost (one reconstruct-delta per frontier pair) is bounded and
   gated behind a per-step cost budget the `FormProposer` respects. Is
   `{FlipReduce, FlipRaw, FlipBiased(+greedy), Kick, Walk, Seq, Choose}` rich enough
   to *contain* a beat, yet bounded enough for C4/C5?
3. **`K = 200`, meta 40×40** — do the SEs resolve a plausible effect size, and is
   the meta-budget enough to explore a depth-5 form space without itself
   overfitting DEV? (Power-analysis territory; the pre-run informs it.)
4. **Does eliding the train set (§9.3) admit DEV overfit**, and is
   re-drawing DEV per meta-generation worth the determinism cost (C7)?
5. **`FlipRaw` and the envelope.** Un-reduced flips grow coefficients toward
   `ENVELOPE = 2¹⁶`; a form that spams `FlipRaw` may stall against the envelope. Is
   that a healthy pressure (the form learns to reduce) or a footgun that wastes
   budget (C5)?

---

## 11. What a result means (AC3 — both outcomes close the gate)

The consequence is a **decision tree keyed to how informative the outcome is** —
the architect + hater both blocked the earlier "any negative ⇒ permanent kill",
because a *baseline-only* measurement (no window) or a *weak* negative cannot carry
the strongest possible consequence (permanently closing the Lisp-substrate line):

1. **No window on ≥ 2 bases** (the §5.2 pre-run fails to find headroom anywhere):
   **the probe is not armed** — this is a *baseline-only measurement* that never
   evaluated a single evolved form. It is a **documented "not-yet-probeable"
   result**, recorded with the sweep table; it **re-scopes** the family (richer
   moves, other base constructions) — it does **not** kill Rung-5. A gate that
   never ran cannot decide the thesis.
2. **Window demonstrated, `f*` clears all three calibrated margins (PASS):** earns
   *exactly* "an evolved move-form beat the hand-written one on held-out,
   structurally-varied targets" (R-0015 non-goal: **no** "recursive
   self-improvement" claim from one positive). Unlocks Rung-5 as *earned*, and the
   R-0013 AC3 T₃ record attempt as the object-level payoff.
3. **Window demonstrated, `f*` beats `b0` on DEV but fails ≥ 1 confirmation set:**
   an **informative negative** — the search *found* a DEV-winner and it *did not*
   generalize across structure. This is the strong, decisive result the three-set
   gate exists to produce; it **kills Rung-5** (the evolved-move-semantics thesis
   is refuted on the substrate where it had its best shot) and redirects to
   object-level scaling (T₃) + the reflection line (R-0016), which stand either way.
4. **Window demonstrated, no form ever beats `b0` even on DEV:** a **bounded
   negative about this DSL/family** — the move space contained no beat. Recorded
   honestly; whether it re-scopes (a richer DSL — e.g. the greedy `FlipBiased`
   variant, §10 OQ2) or is treated as decisive is **a Gustavo decision at loop step
   7**, argued from the pre-run's headroom evidence — *not* an automatic permanent
   kill.

Every outcome is recorded in `theory/discovery-results.md` with the per-set table,
`f*`'s rendered form, and its mutation lineage from `b0()` (§7 test 9). The permanent
kill (case 3) fires **only** on an *informative* negative — a demonstrated window +
structure-varying confirmation sets + a DEV-winner that failed to generalize. The
probe is built to make *each* of these results trustworthy. That is the whole point.

---

## 12. Three-lens resolutions (2026-07-15)

The first three-lens ran on the original draft. **Nice-guy: STRONG WORK** (the
architecture — DSL, meta-loop-as-`run_generic`, the interpreter/verifier split,
B0-in-the-space, the constructive closure — all validated). **Architect: REQUEST
CHANGES. Hater: DO NOT SHIP** — it ran the pre-run and *refuted the family*. Every
finding is resolved below; the *architecture* survived intact, the *substrate* was
replaced.

| Lens · finding | Resolution |
|---|---|
| **Hater 1 (BLOCKING)** — no window on ⟨2,2,2⟩→7 (measured 0/200 ≤ 25.6k) | Family → planted redundancy-scramble; window **measured** (§5.1/§5.2) |
| **Hater 2 (BLOCKING)** — meta-loop intractable (days) | Planted window cheap: ~13 ms/search, ~70 min meta-loop (§5.2, C5) |
| **Hater 3+4 / Architect E (BLOCKING)** — kill-criterion self-contradiction; weak negative → permanent kill | §11 decision tree; "no window" ⇒ re-scope not kill; permanent kill only on an *informative* negative |
| **Hater 5** — one tensor = seed- not structure-generalization | Constructed structurally-distinct bases; each `Ki` a disjoint base block (§5.1/§5.3) |
| **Hater 6+7** — censoring breaks normal-theory FPR; pairing overstated | Null **empirically calibrated** (`b0` vs `b0′`); `K` from measured `Cov` (§5.3) |
| **Hater 8 / Architect C** — internal `≤` vs verifier `==` mis-scores a *better* solve | Certify via `for_target` at the *reached* rank; "impossible by §3.2" removed (§5.2) |
| **Hater 9** — `flip_at` draws no rng | §3.1: the draw is the interpreter's own `below(pairs.len())` |
| **Hater 10** — `Seq.n`/`Walk.stall` uncapped, dead above `B` | `STALL_MAX`/`SEQ_MAX` clamped to `B` (§2) |
| **Hater 12** — SE estimator (n vs n−1) unpinned | Sample sd (n−1) pinned in spec + test 6 |
| **Architect A (major)** — interpreter not a complete small-step machine | §3.1 rewritten as an explicit machine mirroring the reference loop |
| **Architect B (major) / nice-guy** — elitism unstated; `MetaCost` derive | `vary` elitism required + test 8 (§4); `MetaCost: Ord` derived |
| **Architect D** — loose FPR / exchangeability prose | §5.3 states the exchangeability premise; drops "stricter than Bonferroni" |
| **Architect G** — §5.1 one-seed vs §6 two-block scheme | §6 uses `(base, scramble_seed)`; base **and** seeds disjoint per set |
| **Nice-guy 1** — emit lineage + paired distributions | §7 test 9 emits `f*` lineage + per-task `moves` distributions |
| **Nice-guy 2** — greedy `FlipBiased` sharpens both outcomes | §10 OQ2 promotes it as an opt-in constructor before arming |
| **Nice-guy 3** — promote two patterns to `docs/conventions.md` | "Incumbent-in-the-Space" + "assert-the-process" — folded at implementation |

## 13. Changelog

- 2026-07-15 — revised after the three-lens: family → planted redundancy-scramble
  (measured window); §11 decision tree; §3.1 small-step machine; §4 elitism +
  `Ord`; §5.3 calibrated null; §2 cap clamps; §12 findings ledger. Re-review pending.
- 2026-07-14 — created (Draft).
