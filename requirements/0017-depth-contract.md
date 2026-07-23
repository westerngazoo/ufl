# R-0017 — One iterative depth contract (bounded code↔data recursion)

- **Status:** **Draft** (2026-07-16 — owner + Claude, discuss phase). Policy
  decided with Gustavo: **iterative everywhere, no cap**. Acceptance criteria below.
- **Milestone:** the self-eval staircase, **Rung 1 substrate** — the soundness
  contract `quote`/`eval` (R-0016) depend on; see `theory/two-language-substrate.md`.
- **Depends on:** R-0003 (Sexpr core), R-0001 (Eml core). Retro-hardens R-0016
  (reflection): R-0016 shipped against a codec that is symmetric only for shallow
  ASTs; this makes it symmetric and panic-free for the deep, machine-generated ASTs
  the reflection loop makes normal.
- **Supersedes:** the recursion-depth handling in PR #40 (which added a shared
  `MAX_DEPTH = 128` cap consulted by `read`/`lower`/`Display` while `eval`/`eval_pred`
  stayed uncapped) — this **removes** that cap. PR #38's heap-bounded `eval` was
  never merged (`eval` is still recursive on `main`); its correct-in-spirit shape is
  reimplemented here. Both PRs are closed pointing here.

## Context

Recursion-depth policy on the exact surface the reflection loop owns is a set of
uncoordinated accidents (verified against `main`, 2026-07-16):

- **`read`** (`read.rs:111`) and **`lower`** (`lower.rs:37`) are recursive and
  **capped** at the shared `ufl_core::get_max_depth()` (default 128, PR #40),
  returning `RecursionDepthExceeded`.
- **`Display::fmt_internal`** (`sexpr.rs:52`) is recursive and capped **returning
  `fmt::Error`** — a *latent panic*: `Sexpr::to_string()` on a deeper-than-cap tree
  panics (the `ToString` contract turns a `fmt::Error` into a panic).
- **`eval`** (`eval.rs`) and **`eval_pred`** (`eval_pred.rs:95,99,116,124`) are
  recursive and **uncapped** — a genuine **stack overflow** on deep trees.
- The **`Drop`** of `Eml` (`eml.rs:42`) and `Sexpr` (`sexpr.rs:35`) are **already
  iterative** (PR #40) — so those are a regression guard, not new work.

The asymmetry is the bug: the system happily *builds* 1000-deep `Sexpr`s (#35's own
test does), but `read` **rejects** anything deeper than 128 while `Display` panics
and `eval` overflows — so `read(print(x))` breaks the **round-trip invariant**
(`sexpr.rs` — the reader/printer codec) that R-0016's `quote`/`eval` are built on.
Once programs are machine-generated (the SPEC-0011 pilot already produced an 11 MB
runaway render), deep ASTs re-read as input are the *normal* case, and a stack
overflow or `to_string` panic anywhere is a **panic in library code** — forbidden by
CLAUDE.md §6.

## Requirement

Exactly **one** recursion-depth policy across the whole code↔data surface:
**iterative implementations everywhere, with no depth cap and no magic constant.**
**Every** recursive tree-walk on the `Sexpr`/`Eml` code↔data surface walks with an
explicit heap work-stack — so an AST of any depth the machine can allocate is read,
lowered, evaluated, printed, re-read, cloned, compared, raised, and dropped
**without stack overflow and without a panic in library code**. The class to close
(scope expanded 2026-07-23 after the three-lens proved the 5-walk scope insufficient
— see below):

- `read`, `Display` (`Sexpr`), `lower`, `eval`, `eval_pred` — recursive today.
- **`Sexpr::Clone`/`PartialEq` and `Eml::Clone`/`PartialEq`** — *derived, hence
  recursive*; **`eq?` compares `Sexpr`s with `==` and `eval_syntax` `.clone()`s
  quote children** (`eval_pred.rs:110,182`), so `(eq? (quote DEEP))` and
  `(eval (quote DEEP))` — the reflection forms this requirement exists to harden —
  currently **abort in library code**. Hand-written iterative `Clone`/`PartialEq`
  replace the derives.
- **`raise: &Eml → Sexpr`** (`lower.rs:86`) — recursive, uncapped; the `Eml → Sexpr`
  leg of the same round-trip codec.
- `Eml`/`Sexpr` `Drop` — **already iterative** (#40); regression-guarded.

The reader/printer codec becomes symmetric at *every* depth (`read` accepts exactly
what `Display` emits), and no deep code↔data operation aborts in library code.

## Load-bearing scoping decisions (stated here so the spec cannot drift)

1. **Iterative, not capped** (decided 2026-07-16). No shared depth budget, no
   `MAX_DEPTH` constant anywhere. PR #40's 128-cap on `read` is **removed**, not
   relocated — a cap, even a symmetric one, is an arbitrary limit that the
   reflection loop's machine-generated deep ASTs would eventually break. This
   completes the direction #38 (heap-bounded eval) and #40 (iterative `Sexpr` Drop)
   already started, coherently, instead of mixing an iterative `Drop` with a capped
   `read`.
2. **Reimplement `eval`/`eval_pred`/`read`/`lower`/`Display` iteratively; keep the
   iterative `Drop`s.** The `Eml`/`Sexpr` `Drop`s are already iterative (regression-
   guarded, not rewritten). The new iterative `eval` must reproduce #38's
   heap-bounded shape **without** any bare `values.pop().unwrap()` (§6: no `unwrap`
   in lib code), **preserve the `eml(exp_arg, log_arg)` post-order convention**
   (SPEC-0001 §2.4 — exp child before log child, `exp(x) − ln_eml(y)`), and make the
   doc comments truthful (`eval.rs` must not still say "Recursive post-order walk").
3. **No new typed error is required by the no-cap policy** — depth is no longer a
   failure mode. Existing parse/lower errors (`ReadError`, `LowerError`) are
   unchanged; this requirement removes a failure mode, it does not add one.
4. **This is the substrate contract, not reflection.** It delivers the symmetric,
   panic-free codec `quote`/`eval` stand on; it does **not** itself deliver any new
   reflection capability. No overclaim.

## Acceptance criteria

- **AC1 (round-trip at depth).** A **depth-10⁵** `Sexpr` (built iteratively)
  survives the codec: `display(read(display(s))) == display(s)` — a **`String`
  comparison**, since tree `==` is itself recursive (a `10⁵`-deep derived
  `PartialEq` aborts; that recursion is *in scope* per §Clone/PartialEq above, but
  AC1 must not depend on it to test the codec). An `(eml x (eml x …))` depth-10⁵
  spine survives `read → lower → eval` returning a `Value` with no overflow/panic.
- **AC2 (symmetric codec).** For a sweep of depths spanning past the old 128 cap, a
  fuzz test proves `read` **accepts** exactly what `Display` **emits** — the codec
  is symmetric at every depth (no depth at which print-emits but read-rejects).
- **AC3 (bounded Drop — regression guard).** A **10⁵-deep** `Eml` **and** a 10⁵-deep
  `Sexpr` each drop without stack overflow. (Both `Drop`s are already iterative on
  `main`; this pins them so a future refactor cannot silently re-recurse.)
- **AC4 (no panics in lib code).** The **callsite-anchored** grep
  `grep -rnE '\.unwrap\(|\.expect\(|panic!\(' crates/{ufl-core,ufl-syntax,ufl-predicate}/src`
  (test modules excluded) shows **zero** hits — anchored so it does not false-match
  `unexpected`/`expected` (R-0017's original loose pattern was unsatisfiable). Every
  remaining `unreachable!` carries a justifying message per §6. **AC4b:**
  `(eq? (quote DEEP))` and `(eval (quote DEEP))` at depth 10⁵ complete without a
  library-code abort (the reflection-path guarantee the scope expansion delivers).
- **AC5 (doc truthfulness + supersession).** No doc comment describes an iterative
  function as "recursive"; PRs #38 and #40 are **closed with supersession notes**
  pointing to this requirement; the one-policy decision (and that there is no
  constant) is recorded in the decision log.

## Non-goals

- Any new reflection form (that is R-0016, shipped).
- A configurable/pluggable depth budget (the policy is *no* budget).
- Reifying `Value → Sexpr` (out of scope; unrelated to the depth contract).
