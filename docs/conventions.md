# UFL Conventions

Notation and writing conventions UFL uses across its docs, requirements, specs,
code, and agent output. Append-only; new conventions are added with a short
rationale and a date.

This file is part of UFL's source-of-truth set (see [`CLAUDE.md`](../CLAUDE.md)
§8). Every session and every agent should honour what is recorded here.

## Notation

### Circle constant — τ (tau), not π (pi)

UFL uses **`τ = 2π`** as its circle constant. Wherever a UFL doc, requirement,
spec, test, or generated artifact talks about *the* circle constant — the angle
of a full turn, the Euler-formula constant, the imaginary-axis quarter-turn —
it writes `τ`. A full turn is `τ` radians; a half-turn is `τ/2`; a quarter-turn
(Euler's `i`) is `τ/4`.

`π` may still appear in two contexts only:

- inside *quoted* or *cited* material from external sources (e.g. the AllEle
  paper, the founding proposal);
- when explicitly contrasting with `τ`, in which case the bridge `π = τ/2` is
  the assumed identity.

**Rationale.** A full turn being one whole unit (`1 τ`) — rather than two
half-turns (`2π`) — makes angle algebra and most circle-related identities
read directly. UFL is foundationally re-deriving the elementary basis from a
single operator (`eml`); locking in the cleaner circle convention while we are
still the only callers costs nothing and is consistent with UFL's stance of
building from a clean base.

**Decided:** 2026-05-19.

## Engineering patterns

Named, reusable disciplines that recur across specs. Cite them by name instead
of rediscovering them.

### Invariant Tripwire

When a design's correctness is contingent on a property outside its own control
(a floating-point fact, a runtime behaviour), ship a unit test that asserts the
property directly, so a future change that breaks the assumption **fails
loudly** and re-opens the design question deliberately rather than silently.
Instances: SPEC-0001 AC6 (`sin(τ/2) ≠ 0` underpins the branch self-correction).

**Decided:** 2026-06-08 (practiced since 2026-05-24).

### Guard Inside the Candidate (invariant by construction)

When a value must satisfy an invariant to be used safely, put the guard in the
type's **only constructor** rather than at each use site — the invalid value
becomes unconstructible, and every code path is the guarded path. Instances:
`ufl-tensor`'s `Triple::new`/`Scheme::push` (length consistency — the `d`/`n`
desync is impossible), SPEC-0007's `State` (the priming/ReservedName rules live
in `State::new`, so the trait path cannot bypass them).

**Decided:** 2026-06-08.

### Explicit-Stack Tree Walk (no recursion on machine-shaped depth)

Any walk over a heap-recursive AST that a *generator* — not just a human — can
build deep (evaluate, lower, print, read, clone, compare, raise, drop) uses an
explicit heap work-stack, never call-stack recursion: depth is then bounded by
the heap, not the thread stack, and there is no cap to pick, tune, or breach.

Applied **per site, not via a shared helper**. The walks differ materially —
n-ary vs binary, eager-`?`-error vs lazy short-circuit, byte-identical emission —
and one signature over all of them is the wrong abstraction ("three similar lines
beat the wrong abstraction", CLAUDE.md §2).

Two companions are mandatory:
1. a comment stating the **child push order** wherever a reversal would silently
   transpose operands (LIFO means you push the *last* child first);
2. a **differential test** as the order tripwire — an underflow `unreachable!`
   guards *arity*, not *order*, so only comparing against a known-good result
   catches a swap.

Two traps this convention exists to remember, both found by review, both
invisible without it:
- **A tail call can hide a recursion in release builds.** `eval_pred`'s
  `(pred e)` looked like a leaf; implemented recursively it survived 100k nesting
  under `--release` because the compiler TCO'd it, and only overflowed in debug.
  Prefer a tail *launch* (push the operand, no resume frame).
- **A derived trait is a recursive walk.** `#[derive(Clone, PartialEq)]` on a
  boxed-recursive enum generates recursion that no grep for `fn` will find —
  `(eq? (quote DEEP))` aborted in library code for exactly this reason.

Instances: `Eml`/`Sexpr` iterative `Drop` (PR #40); R-0017's `eval`, `read`,
`Display`, `lower`, `raise`, `eval_pred`, and the hand-written `Clone`/`PartialEq`.

**Decided:** 2026-07-26.

### Bounded-Stack Regression Arena

A test for "this deep input does not overflow the stack" cannot be an ordinary
`#[test]`: a stack overflow is an `abort()`, **not** a catchable panic. A child
*thread* cannot `join()` it, and on the main thread it takes the whole test
binary down — so the regression reports as a runtime abort with its sibling tests
deleted, rather than as one named failing test.

Run the deep case in a **subprocess** — a re-exec of the test binary itself,
selected by an env var and a `--exact` filter — and assert the child's **exit
status**. Two companions are mandatory:

1. **Pin the arena to the `dev` (debug) profile.** `--release` TCOs a
   tail-recursive walk, so it false-passes at *any* depth. Measured on UFL's
   pre-R-0017 recursive `eval_pred`, a `(pred …)` spine overflows at 10⁵ in debug
   and returns `Ok` at 3·10⁶ in release. A release run must **decline** and say
   so, never report a green it cannot justify.
2. **Assert the child actually ran a test** (`stdout` contains `1 passed`).
   `libtest` exits 0 when its filter matches nothing, so exit status alone
   false-passes the moment the test name and the case string drift apart.

Deep fixtures are built **iteratively** — a recursive generator overflows before
the code under test runs. And deep values are compared with `assert!(a == b, …)`,
never `assert_eq!`: `Debug` is typically still a recursive derive, so formatting
the failure message would itself abort, reporting the exact symptom the test
exists to distinguish.

Prove the arena can fail before trusting it: reintroduce the recursion and watch
it fail *by name*.

Instances: R-0017's `r_0017_depth_contract.rs` in `ufl-syntax` and
`ufl-predicate` (11 cases at depth 10⁵).

**Decided:** 2026-07-26.

### Assert the Protocol, Not the Outcome

When a committed test *runs an experiment* whose scientific result is the
deliverable, the assertion checks that the experiment **ran as pre-registered**
(fixed budget, fixed seeds, verdict recorded) — never that it produced the hoped
result. A documented negative is then a first-class **green** outcome, and CI
cannot create pressure to keep tuning until the answer is the pleasant one.

Two companions make the negative *worth* something:

1. **Print the measurement unconditionally**, so the number lands in the PR
   whichever way it fell. A verdict nobody can read is not a result.
2. **Pair it with a reachability pre-check**, so a negative is *clean* (the thing
   was findable and we failed to find it) rather than *confounded* (it was never
   there). Without this, "not found" is unattributable.

Instances: SPEC-0018 §4 — the ⟨2,2,3⟩ gate asserts it ran at the pre-registered
budget and seeds, explicitly **not** that rank 11 was found, with §4's
`T-ternary-exists` as the reachability pre-check. SPEC-0014N §2.5 — the integer
probe asserts the pre-registered three-way verdict and prints the bit-exact count
and ulp bound unconditionally.

**Decided:** 2026-07-26.

### Fused Synthesized Attributes

Two bottom-up analyses over one tree that **read each other at any non-leaf
arm** are one function returning a product — never two mutually recursive
functions. Each cross-dependency becomes a field read; no arm calls a walk; every
node is visited once.

The review check is mechanical: *does any arm of a walk call a **different** walk
on a child?* If yes, the pair is exponential on some nesting of that arm — and the
fix is a `struct Analysis { a, b }` with one `rule` written over children's
analyses, not memoization.

The reason this is a convention and not a refactor: the exponential hides.
`grade`'s `Sandwich` arm called `is_versor(r)`, whose `Exp` arm called
`grade(b)`; the non-versor branch then called `grade(r)`, whose `Exp` arm called
`grade(b)` again. Two walks per level, 2^depth on rotor-nested inputs, 204 ms at
70 nodes — invisible on every shape the search actually produced (worst real
genome: 23 µs), and found only by counting entries: 5,116 for 31 nodes.

Assert it by **mechanism, not clock**: a `#[cfg(test)] thread_local!` counter
ticked on entry, asserting `visits == node_count` on the shapes that would
re-walk, at ascending depths in one test. A field summing children's counts does
**not** work — a discarded re-walk never enters the sum.

Counter-instance, recorded for the right reason: SPEC-0019 §2.2 declined to
merge the same two functions for *depth safety*, correctly — merging buys nothing
against stack depth. It is the *complexity* question this convention answers.

Instances: SPEC-0020 (`grade` + `is_versor` → `Analysis { grade, versor }`).

**Decided:** 2026-09-05.

### Structural Frugality over Wall-Clock

Performance acceptance criteria assert the **mechanism** (a cached field, a
bounded allocation count) — never a timing bound. A wall-clock test is flaky on
shared CI and cannot reliably fail under the regression it guards. Complement
of the Invariant Tripwire: assert the symptom when the mechanism is outside
your control; assert the mechanism when it is yours. Instance: SPEC-0007 AC6.

**Decided:** 2026-06-08.

### Fixture Duplication with an Un-deferral Trigger

Test fixtures (e.g. the Strassen 7-triple keystone) may be duplicated across
crates with a comment citing the source of truth — fixture duplication is not
code duplication. Shared-fixture machinery is deferred **until a third consumer
exists**; the deferral ships with its own un-deferral trigger, which is what
makes it a rule rather than a shrug. Instance: SPEC-0007 §2.5.

**Decided:** 2026-06-08.

### Verifier-Held Transparency

When a system must accept artifacts from sources of varying trustworthiness (a
blind search, an LLM agent, a human), put the transparency/correctness guarantee
in the **acceptance predicate**, not in the **generator**. The generator may be
opaque, learned, or random; only an exact, re-runnable check may admit a
candidate. This keeps the accept step auditable while leaving the proposer free
to become arbitrarily sophisticated — dissolving the false choice between
*transparency* and *power*. Instances: SPEC-0008's `run` (reaches candidates only
via `proposer.{seed,vary}`, accepts only via `RankDecomposition::discharge`);
SPEC-0007's `Predicate::discharge` (origin-agnostic). Companion to *Guard Inside
the Candidate* — which guards the boundary between an unvalidated genome and the
validated phenotype via a total `express`.

**Decided:** 2026-06-12 — forced by the R-0008 de-risk: blind GA could not
rediscover Strassen, but the architecture absorbed it because the verifier, not
the proposer, holds the guarantee.
