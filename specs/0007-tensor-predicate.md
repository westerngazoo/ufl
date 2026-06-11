# SPEC-0007 — Tensor-Equality Predicate (the Hehner-discharge bridge)

- **Status:** Draft
- **Realizes:** R-0007 (Option A — the typed discharge abstraction)
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-08
- **Depends on:** R-0006 (`ufl-tensor`), R-0004 (`ufl-predicate`)
- **Crate(s):** `ufl-predicate` (gains the trait + `State`); **`crates/ufl-discovery`**
  (new — the tensor instance + tests)

## 1. Motivation

SPEC-0007 realizes [R-0007](../requirements/0007-tensor-predicate.md) by
**Option A**: a typed *predicate-discharge* abstraction, so the matmul predicate
`P_n,R` and R-0004's scalar Hehner check are both **predicates** under one
concept — *a decidable property of a candidate state*. The discovery engine
(R-0008) discharges the tensor predicate through this abstraction.

**Honest scope of the C1 closure.** FINDINGS C1 has two halves: the predicate
layer could neither *express* nor *discharge* `P_n,R`. This spec closes the
**discharge-unification half** — `P_n,R` becomes a first-class predicate
discharged through the layer's contract. The *expression* half (writing tensor
predicates as s-expressions, Option B) remains deliberately deferred.

**The trait is load-bearing, not decorative.** Two design obligations make the
unification real rather than nominal (the three-lens review's core demand):

1. R-0004's **production path routes through the trait** — `check`/`check_str`
   become construct-candidate-then-`discharge`, so the existing scalar tests
   exercise the trait method.
2. The acceptance tests include a **generic consumer** — a test helper generic
   over `P: Predicate`, instantiated with both instances.

## 2. Design

### 2.1 The `Predicate` trait (in `ufl-predicate`)

```rust
/// A predicate is a *decidable property of a candidate state* — Hehner's
/// notion, generalized over the domain. Discharging it is the check
/// (decidable); searching for a satisfying candidate (R-0008) and selecting a
/// substrate (the future orchestrator) are separate concerns.
pub trait Predicate {
    type Candidate;
    type Error: std::error::Error;
    /// Decide whether `candidate` satisfies this predicate. Total within the
    /// domain's stated envelope: a malformed candidate *or an undischargeable
    /// predicate* (e.g. a non-boolean `Sexpr`, an unbound variable) is a typed
    /// `Error`, never a panic.
    fn discharge(&self, candidate: &Self::Candidate) -> Result<bool, Self::Error>;
}
```

Notes settled by review:

- `Candidate` is **sized** (no speculative `?Sized` — both instances are sized).
- `Error: std::error::Error` so a generic consumer can report failures.
- This is the **candidate seam** for the future orchestrator, not a settled
  interface: it is not dyn-compatible across heterogeneous candidate types as
  is, and the decision log licenses reshaping (dyn-compatibility, error
  unification, owned candidates) when the orchestrator — the first *runtime*
  generic consumer — arrives.

### 2.2 The scalar instance — `Sexpr` is the predicate; `State` is the candidate

The scalar Hehner predicate **is the s-expression itself** (homoiconic: code is
data, and the predicate is code):

```rust
/// The guarded pre/post state a scalar predicate is checked against. The ONLY
/// constructor applies SPEC-0004 §2.5's rules: pre vars bind by name, post vars
/// bind primed, and a binding name containing `'` is rejected (ReservedName).
/// The guard therefore lives inside the candidate — it cannot be bypassed.
pub struct State { env: Env }

impl State {
    pub fn new(pre: &[(&str, Value)], post: &[(&str, Value)]) -> Result<State, CheckError>;
}

impl Predicate for Sexpr {
    type Candidate = State;
    type Error = CheckError;
    /// Evaluate this s-expression as a boolean predicate over the state.
    fn discharge(&self, state: &State) -> Result<bool, CheckError> {
        Ok(eval_pred(self, &state.env)?)
    }
}
```

**`check`/`check_str` route through the trait** (behaviour unchanged, all
R-0004 tests stay green):

```rust
pub fn check(predicate: &Sexpr, pre: …, post: …) -> Result<bool, CheckError> {
    predicate.discharge(&State::new(pre, post)?)
}
pub fn check_str(src: &str, pre: …, post: …) -> Result<bool, CheckError> {
    read(src)?.discharge(&State::new(pre, post)?)
}
```

This resolves the review's two §2.2 findings at once: there is **no second,
unguarded entry point** (the ReservedName guard is inside `State`'s only
constructor, so the trait path and `check` are the same path), and **no wrapper
struct with hedged fields** (the `Sexpr` is the predicate — no clone, no
adapter). `eval_pred` remains the documented low-level raw-`Env` door it already
is under SPEC-0004.

**Implementation constraint (AC2's "unchanged" made literal):** `State::new`
**delegates to the existing `combined_env` verbatim** (becoming its only
caller) rather than absorbing it — the five existing unit tests that target
`combined_env` directly then remain untouched, and "all R-0004 tests pass
unchanged" holds at the test-file level, not just behaviourally.

### 2.3 The tensor instance (in `crates/ufl-discovery`)

```rust
/// P_{n,R}: a scheme satisfies it iff it is a valid rank-R decomposition of
/// the matmul tensor T_n — reconstruct(scheme) == T_n AND rank(scheme) == R.
/// (Named for both conjuncts: the rank bound is the discovery prize.)
pub struct RankDecomposition {
    n: usize,
    rank: usize,
    target: Tensor, // T_n, computed ONCE here — not per discharge
}

impl RankDecomposition {
    pub fn new(n: usize, rank: usize) -> Self {
        Self { n, rank, target: target(n) }
    }
}

impl Predicate for RankDecomposition {
    type Candidate = Scheme;
    type Error = SchemeError;
    /// Reconstruct unconditionally, dim-check against the cached target, then
    /// conjoin the rank bound. A dim/n mismatch is ALWAYS Err(DimMismatch) —
    /// independent of the rank field (the review's blocking finding: the
    /// error contract must not flip on an unrelated conjunct).
    fn discharge(&self, scheme: &Scheme) -> Result<bool, SchemeError> {
        let recon = reconstruct(scheme);
        match error(&recon, &self.target) {
            None => Err(SchemeError::DimMismatch {
                n: self.n,
                expected: self.target.dim(),
                got: recon.dim(),
            }),
            Some(e) => Ok(e == 0 && scheme.rank() == self.rank),
        }
    }
}
```

Composition over `ufl-tensor`'s public API only (`reconstruct`, `error`,
`target`, `Scheme`) — `ufl-tensor` is untouched. Per-discharge cost is exactly
**one** reconstruction buffer (the target is cached at construction; the review
measured the draft's per-call `target(n)` rebuild and it is designed out).

**Relation to `is_valid` (precise, not "equals"):** on dim-consistent schemes,
`discharge == Ok(is_valid(scheme, n, rank))`; on dim-mismatched schemes
`discharge == Err(DimMismatch)` where `is_valid` collapses to `false`.

### 2.4 Crate placement and the dependency seam

The new crate lives at **`crates/ufl-discovery`**, registered in the workspace
`members`. The existing root-level `ufl-discovery/` directory remains the
**research-artifact home** (FINDINGS.md, future writeups) — docs and crate are
deliberately distinct homes; this is stated to prevent a maintainer trap.

```
crates/ufl-discovery ──▶ crates/ufl-predicate ──▶ ufl-syntax ──▶ ufl-core
        └──────────────▶ crates/ufl-tensor (pure leaf, unchanged)
```

A trait-only `ufl-predicate-core` crate was considered and rejected as the
actual premature abstraction (a crate for six lines, with no consumer that must
avoid `ufl-syntax`).

The same PR **updates `crates/README.md`** — its decomposition table still
carries pre-pivot spec numbers and lacks `ufl-tensor` and `ufl-discovery`
(review finding; the table is `CLAUDE.md` §6's crate-boundary reference).

### 2.5 Test fixtures

The Strassen 7-triple keystone lives as a private fn in `ufl-tensor`'s
integration tests (unimportable). `crates/ufl-discovery`'s tests **duplicate the
7-triple literal** with a comment citing SPEC-0006 §2.6 — fixture duplication
is not code duplication; a shared `fixtures` feature is deferred until a third
consumer exists.

## 3. Code outline

§2.1–§2.3 are the outline (trait, `State` + `impl Predicate for Sexpr` +
routed `check`/`check_str`, `RankDecomposition`). The generic consumer in tests:

```rust
/// AC6's generic batch helper — the trait's first generic consumer.
fn discharge_all<P: Predicate>(p: &P, candidates: &[P::Candidate])
    -> Result<usize, P::Error> // count of satisfied candidates
{
    let mut satisfied = 0;
    for c in candidates {
        if p.discharge(c)? { satisfied += 1; }
    }
    Ok(satisfied)
}
```

## 4. Non-goals

- The GA search / engine (R-0008) — `crates/ufl-discovery` gains only the
  predicate here. (R-0008's design review may reshape the crate's internals;
  recorded as an assumption, not a commitment.) **Breadcrumb for R-0008:**
  `RankDecomposition` already holds the cached target, so the GA's *graded*
  fitness is the same computation one method short — a `residual(&self, scheme)
  -> Result<i64, SchemeError>` beside `discharge` would make the fitness
  function and the verifier provably the same computation. Deliberately not
  added here.
- Tensor values/forms in the s-expr language (Option B) — the *expression* half
  of C1, deferred.
- The orchestrator; buffer-reuse optimization of `discharge` (a measured R-0008
  concern if profiling demands it).

## 5. Open questions

*None blocking.* The trait-earned question is settled (kept, load-bearing — §1);
the discharge contract, candidate guard, crate path, and fixture provenance are
fixed.

## 6. Acceptance criteria

- [ ] **AC1** — `ufl-predicate` exposes `Predicate` (`Candidate` sized,
  `Error: std::error::Error`, `discharge -> Result<bool, Error>`); total within
  the SPEC-0006 §2.5 envelope (cited as the precondition for the tensor
  instance's no-panic claim); typed errors, no panic.
- [ ] **AC2** — `Sexpr` implements `Predicate` over `State`; `check`/`check_str`
  route through `State::new` + `discharge`; **all existing R-0004 tests pass
  unchanged**; the ReservedName guard holds on the trait path (a primed binding
  name → `CheckError::ReservedName`).
- [ ] **AC3** — on dim-consistent schemes, `RankDecomposition::discharge ==
  Ok(is_valid(scheme, n, rank))` (tested over exact, wrong-reconstruction, and
  wrong-rank samples); on dim-mismatched schemes it is `Err` where `is_valid`
  is `false`.
- [ ] **AC4 — Strassen keystone.** `RankDecomposition::new(2, 7)
  .discharge(strassen) == Ok(true)`; a **broken** scheme — *well-formed dim,
  wrong reconstruction* (Strassen with one sign flipped) — discharges
  `Ok(false)`. (Dim-malformed inputs are AC5's domain; AC4 and AC5 partition
  the inputs.)
- [ ] **AC5** — a dim/`n` mismatch (including an empty scheme vs `n ≥ 1`)
  discharges to `Err(SchemeError::DimMismatch)` **regardless of the rank
  field** — never a panic or a silent `false`. The `n = 0` vacuous case
  (`new(0,0).discharge(empty) == Ok(true)`) is pinned in a test with a comment
  that `n ≥ 1` is the real domain.
- [ ] **AC6** — structural frugality, no wall clock: `T_n` is computed **once
  per `RankDecomposition`** (in `new`), not per discharge; `discharge` allocates
  at most the one reconstruction buffer. Verified by construction (the field) +
  the **generic** `discharge_all` batch test running a large mutated-Strassen
  batch through `P: Predicate` and asserting outcome counts — no timing
  assertion (a wall-clock bound is a flaky-test factory and cannot fail under
  the regression it guards).

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-08 | Realize R-0007 via **Option A** (typed `Predicate` discharge trait), per the owner's delegated call. | The GA discharges millions of candidates → a typed call; two real instances; no heterogeneous-value expansion. |
| 2026-06-08 | **Trait kept and made load-bearing** (three-lens ruling): `check`/`check_str` route through `discharge`, and a generic test consumer exercises `P: Predicate`. Revision licensed (dyn-compatibility, error unification) when the orchestrator — the first runtime generic consumer — arrives. | Architect: thinning would reduce R-0007 to a rename and un-accept the requirement; CLAUDE.md §2 guards against *wrong* structure, not against naming the founding concept with two real instances. Hater: a trait nobody calls is a fig leaf — routing the production path through it is what makes the unification real. |
| 2026-06-08 | Scalar instance is **`impl Predicate for Sexpr`** with a guarded **`State`** candidate (the only constructor enforces the ReservedName/priming rules). | Homoiconic (the s-expression *is* the predicate; no wrapper, no clone); the guard lives inside the candidate so the trait path cannot bypass SPEC-0004 §2.5 — the review's unguarded-second-entry-point finding, designed out. |
| 2026-06-08 | `RankDecomposition::new` **caches `T_n`**; discharge reconstructs unconditionally, dim-checks, then conjoins rank. Renamed from `TensorEq` (the rank bound is the discovery prize; the old name named one conjunct). | Fixes the review's blocking finding (short-circuit made the error contract flip on the rank field, violating AC5) and the measured per-call `target(n)` rebuild (2 allocations → 1). |
| 2026-06-08 | Crate at `crates/ufl-discovery`; root `ufl-discovery/` stays the research-doc home. Strassen fixture duplicated in tests (cited), shared-fixture machinery deferred. | Placement was unstated (maintainer trap); fixture duplication ≠ code duplication. |
| 2026-06-08 | C1 claim scoped honestly: this closes the **discharge-unification half**; the expression half (Option B tensor s-exprs) remains open by design. | The hater's overstatement finding; FINDINGS.md annotated to match. |
| 2026-06-08 | Architect + nice-guy passes applied: R-0007 reconciled (rename, AC precision, AC6 restatement, decision-log rows); `State::new` delegates to `combined_env` verbatim; trait doc widened to undischargeable predicates; `crates/README.md` update folded into the PR; the R-0008 `residual()` breadcrumb recorded; a crate-level doctest (an `n = 1` discharge end-to-end) is part of the implementation per CLAUDE.md §6. | Architect REQUEST CHANGES was reconciliation-only (design verified sound against the real APIs, including error precedence of the routed path); nice-guy STRONG WORK with two thin spots (FINDINGS annotation, AC2 cheap path), both closed. |

## 8. Companion edits (this branch)

- `requirements/0007-tensor-predicate.md` — reconciled (the architect's gating
  finding).
- `ufl-discovery/FINDINGS.md` — C1 annotated with the half-closure.
- `crates/README.md` — decomposition table brought to post-pivot reality.
- `docs/conventions.md` — engineering-patterns register added (nice-guy
  opportunity): Invariant Tripwire, Guard Inside the Candidate, Structural
  Frugality over Wall-Clock, Fixture Duplication with an Un-deferral Trigger.

## Changelog

- 2026-06-08 — created (Draft).
- 2026-06-08 — hater review applied (NEEDS WORK; executed the draft body and
  proved the AC5 short-circuit violation + measured the per-call target rebuild
  → both designed out): trait made load-bearing (routed `check`, generic test
  consumer, `Error` bound, `?Sized` dropped, orchestrator-seam claim softened);
  scalar instance re-shaped to `impl Predicate for Sexpr` over a guarded
  `State`; `TensorEq` → `RankDecomposition` with cached target and order-fixed
  discharge; AC3/AC4/AC5 made partition-precise; AC6 restated structurally (no
  wall clock); crate path + fixture provenance pinned; C1 claim halved honestly.
- 2026-06-08 — architect (REQUEST CHANGES, reconciliation-only; design verified
  sound) + nice-guy (STRONG WORK) applied: R-0007 amended to match; §2.2
  delegate constraint; §2.1 doc widen; §2.4 crates-README fold-in; §4 R-0008
  breadcrumb; FINDINGS/conventions companion edits (§8).
