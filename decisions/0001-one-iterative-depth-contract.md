# D-0001 — One iterative depth contract (no cap, no constant)

- **Date:** 2026-07-26
- **Decided by:** Gustavo + Claude (R-0017 discuss phase, 2026-07-16; scope
  expanded 2026-07-23 after the three-lens)
- **Realized by:** [SPEC-0017](../specs/0017-depth-contract.md), PR #80

## The decision

Recursion-depth policy across the whole `Sexpr`/`Eml` code↔data surface is
**iterative everywhere, with no depth cap and no magic constant**. PR #40's
shared `MAX_DEPTH = 128` is **removed, not relocated**.

## Why not a (symmetric) cap

A cap is an arbitrary limit, and the reflection loop's *machine-generated* ASTs
eventually breach any number we pick — at which point the failure is either a
rejected valid program or a panic. The asymmetry PR #40 left behind made this
concrete: the system happily *built* 1000-deep trees while `read` rejected
anything past 128 and `Display` turned its cap into a **panic** through the
`ToString` contract. Removing the failure mode beats tuning it.

## What the three-lens changed

The originally-scoped five walks (`read`/`Display`/`lower`/`eval`/`eval_pred`)
were **insufficient**, proven by measurement rather than argument:

- `Sexpr`/`Eml` derived `Clone`/`PartialEq` are *recursive walks that no grep
  finds*, and the reflection path leans on them (`eq?` compares quoted trees with
  `==`; `eval_syntax` clones them) — so `(eq? (quote DEEP))` aborted in **library
  code**, the very thing R-0017 exists to prevent. Scope expanded to hand-written
  iterative impls.
- `raise` is the `Eml → Sexpr` leg of the same codec and was equally unguarded.

Two implementation traps were also found by review and are recorded in
`docs/conventions.md` (*Explicit-Stack Tree Walk*): a recursive `(pred e)` is
**invisible in release** because the compiler TCOs its tail call; and AC4 must be
a **clippy lint**, not a grep, because a grep cannot exclude inline
`#[cfg(test)]` modules.

## Consequences

- `ufl_core::depth` (and `get_max_depth`/`set_max_depth`) deleted.
- All three `RecursionDepthExceeded` variants deleted — depth is no longer a
  failure mode. This is a breaking change to three public error enums, accepted:
  pre-1.0 research crates, zero external consumers (verified by grep).
- The R-0003 acceptance test asserting the *inverse* of AC2 (129-deep must fail)
  is deleted; `r_0017_depth_contract.rs` supersedes it.
- **Adversarial-input posture changes shape, not severity.** Removing the cap
  moves `read`'s worst case from *stack overflow* (an `abort()`) to *heap
  proportional to input*: `"("×n` allocates n empty `Vec`s and then returns
  `UnclosedList`. Bounded and linear in the input we already hold in memory, and
  it fails as a typed `Err` rather than killing the process — but it is the
  substantive consequence of deleting PR #40's mitigation, recorded here so it
  need not be re-derived.
- **Superseded PRs:** #38 (heap-bounded `eval`, never merged — its correct-in-
  spirit shape is reimplemented here without the bare `unwrap`s) and #40 (the
  128-cap; its iterative `Drop`s are kept and regression-guarded).
