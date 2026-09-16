# D-0003 — `ufl-tensor`'s overflow guards stay panics (mission abort, not error)

- **Date:** 2026-09-16
- **Decided by:** Gustavo
- **Resolves:** [#88](https://github.com/westerngazoo/ufl/issues/88), raised by
  [#87](https://github.com/westerngazoo/ufl/pull/87) (the [#82](https://github.com/westerngazoo/ufl/issues/82) lint sweep)
- **Touches:** `crates/ufl-tensor/src/tensor.rs:24,85`; `crates/ufl-tensor/src/lib.rs`

## The decision

`Tensor::zeros` and `target` keep their `expect`s. `ufl-tensor` adopts the
workspace no-panic lint with a **targeted, reasoned `#[allow]`** at exactly those
two sites — so the gate now covers **10 of 10 crates** and the two exceptions are
visible in the code rather than absent from the gate.

## Why — a mission abort is not a recoverable error

Gustavo's reasoning, recorded because it generalizes: a capacity overflow is not
a condition a caller can handle. It is an **abort mission** — the useful response
is not a `Result` threaded through 26 call sites, but a loud stop plus enough
recorded state (the trajectory so far, the last moves, a status) to resume later
with a different strategy or a different starting point.

Measurement supports it, and sharpens "almost unreachable":

| guard | fires at | note |
|---|---|---|
| `target`'s `target dimension overflow` | `n > 4_294_967_296` | `n²` wraps `usize` |
| `zeros`' `tensor capacity overflow` | **`n > 1_625`** | `n⁶` wraps `usize` |

`Config::validate` (`ufl-discovery/src/engine.rs:44-64`) bounds population,
generations, elitism and tournament — **not `n`** — so the site is reachable by
argument. But a dim-1625² tensor needs **~128 EiB** of `i64` before the multiply
can wrap, so the allocator aborts first by an astronomical margin.

The decisive point is what the guard prevents. Without the `checked_mul`,
`dim × dim × dim` wraps, `vec![0; wrapped]` under-allocates, and `add_at` indexes
past the end — **silent corruption of the tensor the whole verifier trusts**. The
`expect` converts a silent wrap into a named abort. That is strictly better than
a `Result` nobody can act on, and far better than the wrap.

## Options rejected

- **(a) Return `Result`.** Breaking change across 26 call sites; invalidates both
  `#[should_panic]` tests in `tests/security.rs`; and hands every caller an error
  it cannot do anything with but propagate. Rejected.
- **(c) Amend CLAUDE.md §6.** Not taken *yet* — see below.

## The open constitutional question

CLAUDE.md §6 reads: *"Panics are for genuinely unreachable states only, with a
justifying message."* These states are **reachable** — `tests/security.rs`
reaches both on purpose. So as recorded, this is a **knowing, documented
exception to §6**, not compliance with it.

If §6 should instead *permit* this class, the amendment would read roughly:

> Panics are for genuinely unreachable states, **or for a mission abort that no
> caller can handle** — a condition where continuing would corrupt data the
> verifier trusts. Either way: a justifying message, a `# Panics` doc section,
> and a `#[should_panic]` test pinning the message.

`ufl-tensor`'s two sites satisfy all three of those conditions today. **Amending
the constitution is Gustavo's call and has not been made** — until it is, the
`#[allow]`s stand as exceptions and this entry is the record of why.

## Follow-up the rationale implies

Gustavo's "logged with the stack trace of last moves, or a status we can continue
later with a different strategy" describes a capability the codebase does not
have: on abort, persist enough of a run's state to resume or re-strategise.
Tracked separately — it is a feature, not part of this decision.
