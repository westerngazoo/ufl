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
