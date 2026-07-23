# R-0017 — One iterative depth contract (bounded code↔data recursion)

- **Status:** **Draft** (2026-07-16 — owner + Claude, discuss phase). Policy
  decided with Gustavo: **iterative everywhere, no cap**. Acceptance criteria below.
- **Milestone:** the self-eval staircase, **Rung 1 substrate** — the soundness
  contract `quote`/`eval` (R-0016) depend on; see `theory/two-language-substrate.md`.
- **Depends on:** R-0003 (Sexpr core), R-0001 (Eml core). Retro-hardens R-0016
  (reflection): R-0016 shipped against a codec that is symmetric only for shallow
  ASTs; this makes it symmetric and panic-free for the deep, machine-generated ASTs
  the reflection loop makes normal.
- **Supersedes:** the recursion-depth handling in PR #38 (heap-bounded eval — the
  correct-in-spirit part is salvaged) and PR #40 (which capped only `read` at a
  hard-coded 128 while the rest stayed unbounded). Both are closed pointing here.

## Context

Recursion-depth policy on the exact surface the reflection loop owns is three
uncoordinated accidents:

- **`read`** is capped at a hard-coded `MAX_DEPTH = 128` (PR #40), while
- **`Sexpr` construction**, **`Display`** printing (`sexpr.rs`), **`lower`**
  (`lower.rs`), **`eval`**, and **`eval_pred`** are unbounded-recursive, and
- the **`Box`-recursive `Drop`** of `Eml`/`Sexpr` overflows the stack on deep trees
  (PR #40 fixed only `Sexpr`'s `Drop`; `Eml`'s is still recursive).

The asymmetry is the bug: the system happily *builds* 1000-deep `Sexpr`s (#35's own
test does), and `Display` happily *prints* them, but `read` **rejects** anything
deeper than 128 — so `read(print(x))` breaks the **round-trip invariant**
(`sexpr.rs` — the reader/printer codec) that R-0016's `quote`/`eval` are built on.
Once programs are machine-generated (the SPEC-0011 pilot already produced an 11 MB
runaway render), deep ASTs re-read as input are the *normal* case, and a stack
overflow anywhere is a **panic in library code** — forbidden by CLAUDE.md §6.

## Requirement

Exactly **one** recursion-depth policy across the whole code↔data surface:
**iterative implementations everywhere, with no depth cap and no magic constant.**
`read`, `Display`, `lower`, `eval`, `eval_pred`, and the `Drop` of `Eml` and
`Sexpr` each walk their trees with an explicit heap-allocated work-stack (or
equivalent), so an AST of any depth the machine can allocate is read, lowered,
evaluated, printed, re-read, and dropped **without stack overflow and without a
panic in library code**. The reader/printer codec becomes symmetric at *every*
depth: `read` accepts exactly what `Display` emits.

## Load-bearing scoping decisions (stated here so the spec cannot drift)

1. **Iterative, not capped** (decided 2026-07-16). No shared depth budget, no
   `MAX_DEPTH` constant anywhere. PR #40's 128-cap on `read` is **removed**, not
   relocated — a cap, even a symmetric one, is an arbitrary limit that the
   reflection loop's machine-generated deep ASTs would eventually break. This
   completes the direction #38 (heap-bounded eval) and #40 (iterative `Sexpr` Drop)
   already started, coherently, instead of mixing an iterative `Drop` with a capped
   `read`.
2. **Salvage the correct-in-spirit prior work.** Reuse #38's heap-bounded `eval`
   shape and #40's iterative `Sexpr` `Drop` — but **without** #38's three bare
   `values.pop().unwrap()`s (§6: no `unwrap` in lib code — use typed
   unreachable-justified handling or a `Result`), keeping the `log.rs` SPEC-0001
   §2.4 branch-convention comment and making the doc comments truthful (`eval.rs`
   must not still say "Recursive post-order walk").
3. **No new typed error is required by the no-cap policy** — depth is no longer a
   failure mode. Existing parse/lower errors (`ReadError`, `LowerError`) are
   unchanged; this requirement removes a failure mode, it does not add one.
4. **This is the substrate contract, not reflection.** It delivers the symmetric,
   panic-free codec `quote`/`eval` stand on; it does **not** itself deliver any new
   reflection capability. No overclaim.

## Acceptance criteria

- **AC1 (round-trip at depth).** A `ufl-prng`-generated **depth-10⁵** `Sexpr`
  survives `read → lower → eval → print → read` with an identical result and **no
  stack overflow and no panic**. (The reflection loop's deep-AST case, made a test.)
- **AC2 (symmetric codec).** For a sweep of depths spanning past the old 128 cap, a
  fuzz test proves `read` **accepts** exactly what `Display` **emits** — the codec
  is symmetric at every depth (no depth at which print-emits but read-rejects).
- **AC3 (bounded Drop).** A **10⁵-deep** `Eml` **and** a 10⁵-deep `Sexpr` each drop
  without stack overflow (the leak both PRs missed for `Eml`).
- **AC4 (no panics in lib code).** `grep -rE 'unwrap|expect|panic!' crates/ufl-core/src crates/ufl-syntax/src crates/ufl-predicate/src` shows **zero** hits in library code (test modules excluded), and every remaining `unreachable!` carries a justifying message per §6.
- **AC5 (doc truthfulness + supersession).** No doc comment describes an iterative
  function as "recursive"; PRs #38 and #40 are **closed with supersession notes**
  pointing to this requirement; the one-policy decision (and that there is no
  constant) is recorded in the decision log.

## Non-goals

- Any new reflection form (that is R-0016, shipped).
- A configurable/pluggable depth budget (the policy is *no* budget).
- Reifying `Value → Sexpr` (out of scope; unrelated to the depth contract).
