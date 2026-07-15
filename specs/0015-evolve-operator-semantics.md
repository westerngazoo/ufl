# SPEC-0015 — Evolve operator semantics (the Rung-4 probe)

- **Realizes:** [R-0015](../requirements/0015-evolve-operator-semantics.md)
  (Accepted) — AC1 (the move-form DSL), AC2 (the pre-registered three-set gate),
  AC2b (the headroom window), AC3 (the kill-criterion), C1–C7.
- **Status:** **Draft** — owed before code (R-0015 §Status). Three-lens pending.
- **Milestone:** the self-eval staircase, **Rung 4** — the *decision node*. A
  positive earns Rung-5 (the Lisp substrate); a documented negative kills it
  permanently and redirects to object-level scaling + the reflection line.
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

   THE SPLIT (all disjoint by seed-block construction — §5):
     DEV  ── the meta-fitness signal (forms are optimized against this)
     K1, K2, K3 ── the three confirmation sets (the GATE; never seen in search)
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
`NODES_MAX = 24`, `STALL_MAX = 4096`, `KICK_MAX = 32`. A form exceeding a cap is
**not representable** — the `FormProposer` (§4) never emits one, and a
`fn well_formed(&MoveForm) -> bool` (depth + node count ≤ caps) is asserted at
every meta-loop entry so an out-of-cap form is a test failure, not a silent run.

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
reproduces `reduce_matmul_with(.., FlipConfig::pinned())` **byte-identically**
(the T-faithful test, §7) — so the meta-search competes against the *real*
baseline on the *same* harness, not a re-implementation.

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

The interpreter maintains `(s: IntScheme, best: IntScheme, stall: usize, rng)` —
the same state `reduce_matmul_with` maintains — and executes the form as a
step-policy for ≤ `budget` steps:

- `FlipReduce` / `FlipRaw` / `FlipBiased(v)` — one step: pick a frontier pair
  (uniform, or biased to `v`), apply `flip_at`, then `reduce` (or not, for
  `FlipRaw`). A step that draws no applicable flip is a no-op step (still counts
  against the budget — the determinism contract).
- `Kick(k)` — `s = perturb(best, k, rng)`.
- `Walk { stall, body, on_stall }` — run `body` as the per-step policy; on
  `stall` steps without a strict `best.rank()` drop, run `on_stall` once, reset
  the counter; loop.
- `Seq { n, first, second }` — `first` as the step-policy for `n` steps, then
  `second` for the remainder.
- `Choose { p, a, b }` — each step, one `rng.below(256)`; `< p` ⇒ `a`, else `b`.

After every step: if `s.rank() ≤ target_rank ∧ s.is_ternary()`, record
`moves_to_solve` and the candidate and stop. Track `best` by `rank()`.

**Determinism contract (C7, mirrors SPEC-0013 §2.4):** the interpreter draws
`rng` only inside the primitives (`flip_at`'s frontier pick, `perturb`, and the
`Choose` branch draw); the meta-harness pins the per-`(form, task)` seed, so every
accepted form's inner trajectory replays exactly.

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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MetaCost {
    sum_moves: u64,      // primary: total censored moves-to-solve
    sum_best_rank: u64,  // tie-break: Σ best_rank_reached (only when censored)
}
// Ord: by (sum_moves, sum_best_rank) lexicographically.
```

- **`FormProposer: Proposer<MoveForm, MetaCost>`** — `seed` samples random
  well-formed forms within the caps (always including `b0()` in the initial
  population, so the search can never do *worse* than the baseline by losing it);
  `vary` mutates the cost-ranked parents: perturb a numeric knob (`stall`/`n`/`p`/
  `k`) by a bounded step, swap a combinator, or grow/shrink a subtree — **always
  re-clamped to the caps** (never emits an out-of-cap form). Answer-blind: it
  operates on `MoveForm` structure and `rng` only.
- **`MetaFitness: Fitness<MoveForm, MetaCost>`** — `score(form)` runs
  `interpret(form, ..)` on **every task in the DEV set** at budget `B`, and for
  each *claimed* solve **certifies the candidate through
  `RankDecomposition::discharge` (C3 — the reward is the exact verifier verdict,
  never the internal heuristic)**; a solve the verifier rejects is treated as
  censored (impossible by §3.2, guarded defensively). Returns the aggregate
  `MetaCost`. `type Error` = the tensor/scheme error channel. `solved` returns
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

### 5.1 The task family — ⟨2,2,2⟩→rank-7 at a tight budget, seed-parameterized

**A task = a `(task_seed)`** naming one deterministic inner search of `T_2` toward
`target_rank = 7` at the pinned step budget `B`. All tasks share the *same* tensor
and target rank; they differ only in the inner search seed. **A move-form is a
search *policy*, not a per-task answer** — it cannot memorize a specific tensor —
so a single tensor with a seed-parameterized family is a sound, and far cheaper,
substitute for planted `for_target` instances (see §9 deviation 1).

### 5.2 The headroom window (AC2b — a precondition, demonstrated by a pre-run)

⟨2,2,2⟩ at *full* budget is too easy (B0 → 100% → no headroom); at an
impossibly-tight budget it is too hard (B0 → 0% → no signal). The knob that opens
the window is **`B`** (the per-search step budget). The metric is
moves-to-solve *right-censored at `B`* (SPEC-0014 §2.5), so the window is the `B`
at which B0's **censoring rate** (fraction of DEV seeds that hit `B` without
solving) sits in **[0.2, 0.8]** — a regime where both "solve more of the hard
seeds" and "solve the easy seeds faster" are live improvements.

> **PRE-RUN (mandatory, before the probe is armed):** sweep `B ∈ {200, 400, 800,
> 1600, 3200}` over the DEV seed block; measure B0's censoring rate at each; pin
> `B` at the smallest value whose rate ∈ [0.2, 0.8]. **If no `B` yields a rate in
> the window, that is itself a documented result** — the ⟨2,2,2⟩ family has no
> headroom for this move set, and the probe re-scopes to a harder family (§9
> deviation 1's planted-instance path) before arming. The pre-run's table is
> recorded in the PR.

### 5.3 The gate (AC2 — three disjoint sets, ≥2 SE each, correction = conjunction)

After the meta-search returns its best form `f*` (optimized against DEV only), the
gate runs **`f*` and `b0()` on each of the three confirmation sets** `K1,K2,K3`:

- **Paired statistic per set `Ki`:** for each task `t ∈ Ki`, the paired delta
  `d_t = moves_{B0}(t) − moves_{f*}(t)` (censored moves; positive ⇒ `f*` better).
  `D̄_i = mean_t d_t`, `SE_i = sd(d_t)/√|Ki|`. Paired (same tasks, same task
  seeds for both arms) → the SE cancels task-difficulty variance.
- **Decision rule (the multiple-comparisons correction *is* the conjunction):**
  the gate **PASSES** iff `D̄_i ≥ 2·SE_i` for **all three** `i ∈ {1,2,3}`
  *simultaneously*. Under independence each set's false-positive rate is ≈0.023, so
  the joint FPR is ≈0.023³ ≈ 1.2×10⁻⁵ — **stricter** than a Bonferroni-α/3 split of
  a single 0.05 test, and it is exactly the protocol that caught the earlier
  headroom-probe overfit (survived two splits, flipped to −14/200 on the third;
  R-0015 AC2). A beat on one or two sets but not all three is a **documented
  negative**, not a pass.

Everything in §5 — the family, `B`, `K` (tasks per set), the seed blocks, the
margin, the correction rule — is **frozen in the pre-registration table (§6)
before the meta-search runs** (C6). The gate code asserts `f*` was never scored on
any `Ki` seed (C2).

---

## 6. Pre-registration table (frozen before the run — proposed values)

| Parameter | Symbol | Proposed value | Rationale |
|---|---|---|---|
| Tensor / target rank | ⟨2,2,2⟩, `target_rank` | `T_2`, 7 | the committed, tested lane (SPEC-0013); rank-7 is Strassen |
| Per-search step budget | `B` | **pinned by the §5.2 pre-run** (candidate 800) | the headroom knob; frozen at the window |
| Tasks per set | `K` | 200 | SE ∝ 1/√200 ≈ 0.07·sd — resolves a ≥2 SE beat at realistic effect sizes |
| DEV (meta-signal) seeds | — | task-seed block `[0, 200)`, run-seed block `[10⁶, 10⁶+200)` | the only set the meta-search sees |
| Confirm set K1 | — | task `[1000,1200)`, run `[2·10⁶, +200)` | disjoint by construction |
| Confirm set K2 | — | task `[2000,2200)`, run `[3·10⁶, +200)` | disjoint by construction |
| Confirm set K3 | — | task `[3000,3200)`, run `[4·10⁶, +200)` | disjoint by construction |
| Meta-population | — | 40 forms | the outer `run_generic` population |
| Meta-generations | — | 40 | outer budget; `b0()` seeded into gen 0 |
| Meta-seed | — | 20260715 | the one outer seed (C7 replay) |
| Margin | — | ≥2 SE **on each** of K1,K2,K3 | AC2 |
| Correction | — | conjunction of all three (joint FPR ≈1.2×10⁻⁵) | AC2 multiple-comparisons |
| DSL caps | — | DEPTH 5 / NODES 24 / STALL 4096 / KICK 32 | C4 bound |

*(Values are the spec's proposal; the three-lens + Gustavo freeze them at
acceptance. The `B` cell is filled by the §5.2 pre-run and recorded before arming.)*

---

## 7. Tests (TDD — written first, red)

`crates/ufl-discovery/tests/r_0015_rung4.rs` (fast lane) + the `#[ignore]` probe:

1. **`b0_form_replays_reduce_matmul_byte_identically`** (AC1, C7): `interpret(b0(),
   2, 7, seed, B)` yields the *same* `moves_to_solve`, `best_rank`, and certified
   scheme as `reduce_matmul_with(2, 7, seed, B, FlipConfig::pinned())`, over a seed
   block — the DSL contains the real baseline, not a copy (reuses SPEC-0013 §2.6.5
   replay discipline).
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
   `D̄ = 1.6`, `SE = sd/√5`; assert the harness's `(D̄, SE)` equal the hand values
   (guards the statistics code, the thing most likely to be subtly wrong).
7. **`well_formed_rejects_out_of_cap_forms`** (C4): a depth-6 / 25-node form is
   `well_formed == false`; the `FormProposer` never emits one (fuzz its `vary`).
8. **`the_probe`** (`#[ignore]`, release) — the experiment, with the runbook in a
   doc-comment: the §5.2 pre-run → freeze `B` → meta-search on DEV → the §5.3 gate
   on K1,K2,K3 → print the per-set `(D̄_i, SE_i, pass?)` table and the PASS/NEGATIVE
   verdict + `f*`'s rendered form. **The committed assertion is not "PASS"** — a
   negative is a valid, expected result (AC3); the test asserts only that the gate
   *ran on all three disjoint sets* and recorded a verdict.

---

## 8. The seven non-negotiables, discharged (C1–C7)

| # | Guarantee | How this spec secures it |
|---|---|---|
| C1 | verifier unreachable from proposer | `interpret`/`FormProposer` hold no `RankDecomposition` (test 3); only `MetaFitness` does (§4) |
| C2 | held-out scoring | DEV drives the search; K1/K2/K3 are disjoint by seed-block construction (test 5); the gate asserts no `Ki` seed was searched |
| C3 | reward = exact verdict | `MetaFitness` certifies every claimed solve through `RankDecomposition::discharge` (test 4) |
| C4 | bounded operator space | typed grammar + `DEPTH/NODES/STALL/KICK` caps; `well_formed` asserted (test 7) |
| C5 | verification cheap vs search | one `discharge` per *solve* (rare), O(d³) reconstruct; the inner walk is thousands of O(frontier) steps |
| C6 | improvement = measured held-out delta | the gate (§5.3) is computed *outside* the loop on frozen confirmation sets; `MetaFitness::solved` never fires |
| C7 | traceable lineage + replay | every seed pinned (§6); `b0()` replays byte-identically (test 1); `MoveForm: PartialEq` + the meta-seed reproduce any accepted form |

---

## 9. Deviations from R-0015's sketch (flagged for the three-lens)

1. **The task family is one tensor at a tight budget, not planted `for_target`
   instances — the biggest decision.** R-0015/AC2b gestures at a "task family";
   the committed flip-graph (`reduce_matmul_with`) is **⟨n⟩-specific** — it starts
   from `naive(n)` and targets `target_int(n)`, and there is *no* naive
   decomposition of an arbitrary planted tensor in the repo. Building a general
   `for_target` flip-graph (a naive decomposition of an arbitrary tensor + a
   general frontier) is a substantial new module with its own correctness burden.
   This spec instead opens the headroom window with the **budget knob `B`** on the
   existing, tested ⟨2,2,2⟩→7 lane, parameterizing tasks by seed. **Risk the
   three-lens must weigh:** a single tensor may offer *no* move-semantic headroom
   over B0 (B0 may already be near-optimal on ⟨2,2,2⟩, exactly as the
   hyperparameters were headroom-free) — in which case the probe returns a
   *negative that is about this family, not about the meta-search idea in general*.
   The §5.2 pre-run's "no window ⇒ re-scope to planted instances" clause is the
   escape hatch, but it means a negative here is **weaker evidence** than a
   negative on a richer family. This is the load-bearing judgement call.
2. **Metric = moves-to-solve (SPEC-0014 §2.5), with AC2b's "success rate ∈ (0,1)"
   realized as "B0 censoring rate ∈ [0.2, 0.8]."** The continuous moves-to-solve
   statistic has smaller SE than a Bernoulli success rate and matches §2.5; the
   censoring-rate window is the saturation guard AC2b actually wants. Flagged in
   case the three-lens reads AC2b as mandating a binary success-rate metric.
3. **The "train" set is elided.** R-0015 C2 says "train/holdout"; a MoveForm is
   task-agnostic (a policy, not a per-task fit), so there is no set the inner
   search *fits* to — DEV is the meta-signal, K1/K2/K3 the gate. If the three-lens
   judges a form *can* overfit to DEV (e.g. by exploiting a DEV-seed idiosyncrasy),
   the fix is to enlarge DEV / re-draw it per meta-generation; called out so they
   can push.
4. **The meta-loop's `solved` never fires.** `run_generic` is built to stop on a
   solution; the meta-loop has none, so it always `Exhausted`s at
   `meta_generations` and returns the best form. This is a benign use of the seam
   (a pure minimizer), but it means the meta-loop always spends its full budget —
   flagged as an intended, not accidental, cost.

---

## 10. Open questions for the three-lens

1. **Is ⟨2,2,2⟩→7 the right first family, or must the probe build planted
   `for_target` instances up front** to avoid a family-specific negative (§9.1)?
   The cost/evidence trade is the central call.
2. **The DSL's expressive ceiling.** Is `{FlipReduce, FlipRaw, FlipBiased, Kick,
   Walk, Seq, Choose}` rich enough to *contain* a move meaningfully better than
   B0, yet bounded enough for C4/C5? Should `FlipBiased` carry a
   rank-descent-greedy variant (pick the frontier pair that most reduces rank) —
   more headroom, but more inner cost per step (C5)?
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

- **PASS** (`f*` beats `b0()` by ≥2 SE on all three sets): earns *exactly* the
  statement "an evolved move-form beat the hand-written one on held-out matmul
  targets" (R-0015 non-goal: **no** "recursive self-improvement" claim from one
  positive). It unlocks Rung-5 (the Lisp substrate) as *earned*, and R-0013 AC3's
  T₃ record attempt as the object-level payoff.
- **NEGATIVE** (any set fails, or no headroom window exists): **kills Rung-5
  permanently** (no Lisp substrate) and redirects to object-level scaling (T₃) +
  the reflection line (R-0016), which stands either way. Recorded honestly in
  `theory/discovery-results.md` with the per-set table.

The probe is built to make *either* result trustworthy. That is the whole point.
