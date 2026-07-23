# SPEC-0017 — One iterative depth contract

- **Realizes:** [R-0017](../requirements/0017-depth-contract.md) AC1–AC5.
- **Status:** **Draft — revised 2026-07-23 after the three-lens.** Nice-guy *SOLID*,
  architect *approve-with-changes*, hater *needs-work*. The hater proved (by running
  the code) that the original 5-walk scope leaves `Sexpr`/`Eml`'s **derived recursive
  `Clone`/`PartialEq`** aborting `(eq?/eval (quote DEEP))` in library code — so
  **scope expanded (Gustavo) to close the whole class**: + iterative `Clone`/
  `PartialEq` (`Sexpr`+`Eml`) + iterative `raise`. All findings folded (§5 ledger).
  **Re-review pending.**
- **Crates:** `ufl-core` (`eval`, `Eml` `Clone`/`PartialEq`, delete `depth`),
  `ufl-syntax` (`read`, `lower`, `raise`, `Sexpr` `Display`/`Clone`/`PartialEq`),
  `ufl-predicate` (`eval_pred`). No new crate; no new public API (the derives become
  hand-written `impl`s — same signatures).
- **Depends on:** R-0001 (Eml), R-0003 (Sexpr). Retro-hardens R-0016.

## 2.1 Remove the cap infrastructure (the policy is *no cap*)

- **Delete** `crates/ufl-core/src/depth.rs`, the `pub mod depth;` (`lib.rs:12`),
  **and** the `pub use depth::{get_max_depth, set_max_depth}` re-export (`lib.rs:17`)
  — both lines, or it fails to compile (architect).
- **Remove all three** now-dead `RecursionDepthExceeded` variants:
  `ReadError` (`read.rs:24`), `LowerError` (`lower.rs:27`), **and
  `EvalError` (`eval.rs:50`)** — the last is already dead (the recursive `eval` never
  constructs it) and would trip the §2.9 no-cap grep if left (architect + hater).
- The three cap checks (`read.rs:111`, `lower.rs:37`, `sexpr.rs:52`) are deleted as
  part of the iterative rewrites below.

## 2.2 Iterative `eval` (`ufl-core`) — post-order, explicit two-stack

Replace the recursive `eval` (`eval.rs:64`) with an explicit **task stack** + a
**value stack**, preserving SPEC-0001 §2.4/§2.5 exactly (`exp_arg` before `log_arg`;
`Node → exp(x) − ln_eml(y)`; unbound var is the only error; `inf`/`nan` propagate):

```rust
enum Task<'a> { Eval(&'a Eml), Combine }

pub fn eval(expr: &Eml, env: &Env) -> Result<Value, EvalError> {
    let mut work = vec![Task::Eval(expr)];
    let mut vals: Vec<Value> = Vec::new();
    while let Some(t) = work.pop() {
        match t {
            Task::Eval(Eml::One) => vals.push(Value::new(1.0, 0.0)),
            Task::Eval(Eml::Var(n)) => {
                vals.push(env.get(n).ok_or_else(|| EvalError::UnboundVariable(n.clone()))?)
            }
            Task::Eval(Eml::Node { exp_arg, log_arg }) => {
                // LIFO: push Combine first, then log, then exp — so exp pops
                // first and is evaluated first (§2.4 convention), leaving the
                // value stack as [.., exp_val, log_val] when Combine runs.
                work.push(Task::Combine);
                work.push(Task::Eval(log_arg));
                work.push(Task::Eval(exp_arg));
            }
            Task::Combine => {
                // By construction every Combine is preceded by exactly two Eval
                // pushes, so two values are present — a genuinely unreachable
                // underflow, justified per §6 (never a bare unwrap).
                let (log_val, exp_val) = match (vals.pop(), vals.pop()) {
                    (Some(l), Some(e)) => (l, e),
                    _ => unreachable!("eval value-stack underflow: each Combine follows two Eval pushes"),
                };
                vals.push(exp_val.exp() - crate::log::ln_eml(log_val));
            }
        }
    }
    match vals.pop() {
        Some(v) => Ok(v),
        None => unreachable!("eval produced no value for a well-formed Eml"),
    }
}
```

The doc comment is rewritten — it must **not** say "Recursive post-order walk"
(R-0017 AC5). The `log.rs` SPEC-0001 §2.4 branch-convention comment is untouched.

## 2.3 Iterative `lower` (`ufl-syntax`) — post-order over the `eml` tree

`lower` recurses only through the binary `(eml a b)` form (`lower.rs`); every other
head is a leaf or an error. Mirror §2.2: a task stack over `&Sexpr` + a result stack
of `Eml`. When a `List` is first popped, **validate the head/arity eagerly** (so
`LowerError::{UnknownForm, Arity, …}` fire in the same order as today — an error
short-circuits the whole walk via `?`); for the valid `(eml a b)` case push a
`Combine` then the two children. Leaves: `Sexpr::Num(1.0) → One` (non-`1` numbers
keep their existing error), `Sexpr::Sym → Var`. `Combine` pops two `Eml`s
(justified-unreachable on underflow) and builds `Node { exp_arg, log_arg }` in the
correct order. The existing `LowerError` variants are unchanged except the deleted
depth one.

## 2.4 Iterative `read` (`ufl-syntax`) — an explicit partial-list stack

Replace `parse_expr` (`read.rs:110`) with the standard iterative reader: a stack of
**partial lists** (`Vec<Sexpr>`).

- `(` → push a fresh empty `Vec<Sexpr>`.
- atom → parse to `Sexpr::{Num,Sym}`; push onto the top partial list, or (empty
  stack, no result yet) record it as the top-level result.
- `)` → pop the top partial list, wrap as `Sexpr::List`, push onto the new top, or
  record as the result if the stack is now empty. An unmatched `)` with **no result
  yet** → `ReadError::UnexpectedClose` (the real variant name — not `Unexpected`).
- **The single-top-level-datum invariant (hater/architect — pinned by
  `r_0003_acceptance.rs:220,227`):** once a top-level result is recorded, **any**
  further token — atom, `(`, or `)` — is `ReadError::TrailingTokens`. The recursive
  reader enforced this via `pos != tokens.len()` (`read.rs:150`); the iterative
  reader must reproduce it, so `read("(a))")` and `read("a)")` stay `TrailingTokens`
  (not `UnexpectedClose`) and `read("(a) b")` stays `TrailingTokens`.
- End of tokens with a non-empty stack → `ReadError::UnclosedList` (the real variant
  name — not `Unbalanced`).

No `depth` parameter; no cap. The reader now accepts **any** depth `Display` can
emit (AC2), with **byte-for-byte the same error behaviour** as today (§3).

## 2.5 Iterative `Display` (`Sexpr::fmt_internal`) — **format-preserving** (footgun)

Replace the recursive `fmt_internal` (`sexpr.rs:51`) with an explicit stack, and —
load-bearing — **emit byte-identical output to today's**, because AC2's symmetric
codec test and the R-0003 round-trip tests pin the exact format (`(`, single-space
separators, `)`; `Num` via `{n}`, `Sym` via `{s}`). The cap-returning-`fmt::Error`
branch (`sexpr.rs:52-54`) is **deleted** — this removes the latent `to_string()`
panic.

Design: a work stack of `Frame` items:

```rust
enum Frame<'a> { Node(&'a Sexpr), Close, Space }
```
- `Node(Num|Sym)` → write it.
- `Node(List(items))` → write `"("`; push `Close`; then push the items **in reverse**
  with a `Space` between consecutive items (so they pop in order, space-separated,
  no leading/trailing space). `Close` → write `")"`.

A test asserts `Display` output is unchanged on the existing R-0003 corpus (not just
"parses back" — *byte-identical*), so the rewrite cannot silently change spacing.

## 2.6 Iterative `eval_pred` (`ufl-predicate`) — **short-circuit-preserving** (footgun)

`eval_pred` (`eval_pred.rs:75`) is the hard one: `and` (`:116`) short-circuits on the
first `false`, `or` (`:124`) on the first `true`, so a naïve eager post-order would
**change semantics** — it would evaluate (and surface errors from) operands the
recursive version skips. The iterative version **must** preserve the exact
short-circuit *and error* semantics.

**Resolved iterative, not deferred (architect + hater).** The §4 "leave it recursive"
escape is dead: the hater *measured* `(not)^d` overflowing at **3–4k frames (debug)**
— far below 10⁵ and within machine-generated reach — so there is no honest
"shallow-by-domain" bound, and the requirement lists `eval_pred` explicitly.

**The footgun is smaller than it looks (architect's call-graph trace).** Only
`and`/`or`/`not`/`pred` re-enter `eval_pred` (`eval_pred.rs:94,99,116,124`); the
**real heads are exactly `and|or|not|=|eq?|pred`** (`is_pred_head`, `:36` — there is
**no `imply`/`iff`**, the prior draft was wrong). `=` delegates to `eval_num`
(`:102` → `lower`+`ufl_core::eval`, both now iterative) and `eq?` to `eval_syntax`
(`:109` → unwrap `quote`, then structural `==`, now iterative per §2.7) — **neither
re-enters `eval_pred`**. So the continuation machine only linearizes the boolean
spine; `=`/`eq?` are **leaves** that compute one boolean and push it.

Design — an explicit stack over the boolean spine, one operand at a time:

```rust
enum Frame<'a> {
    Eval(&'a Sexpr),                    // launch a fresh predicate eval
    And(std::slice::Iter<'a, Sexpr>),   // resume: pop the last bool, short-circuit on false
    Or(std::slice::Iter<'a, Sexpr>),    // resume: pop the last bool, short-circuit on true
    Not,                                // resume: pop, push its negation
    // =, eq?, pred, true/false are LEAVES inside Eval — compute one bool, push it.
}
```

**The driver invariants (hater — the two ways and/or machines break):**
1. **Net +1 boolean per sub-eval.** Every `Eval(p)` pushes exactly one boolean onto
   the value stack; `And`/`Or`/`Not` *pop* their operand's result on resume (never
   peek — a peek reads a stale value).
2. **No "last boolean" on first entry.** `And`/`Or` are entered by *first launching
   operand 0* (`Eval(op0)` + re-push `And(iter@1)`) **without inspecting** any value;
   only on the *resume* pop do they decide. So `(and (or false false) BAD)` launches
   `(or false false)` → `false`, `And` pops `false` → short-circuits `Ok(false)`,
   `BAD` never `Eval`'d (its error never surfaces).

Non-predicate forms and syntax dispatch keep their exact `PredError` behaviour.

**The gate (§2.9 T-shortcircuit):** `(and false BAD)`→`Ok(false)`, `(or true BAD)`→
`Ok(true)`, and `(and (or false false) BAD)`→`Ok(false)` (`BAD` unevaluated) — all
identical before and after; plus a depth-10⁵ `(not)^d` / `(and (and …))` evaluates
without overflow (subprocess arena, §2.9).

## 2.7 Iterative `Clone` + `PartialEq` for `Sexpr` and `Eml` (the class closure)

The derived `Clone`/`PartialEq` on `Sexpr` (`sexpr.rs:7`) and `Eml` (`eml.rs:9`) are
**recursive and abort at ~60k depth** (hater, measured) — and they are on the
reflection path: `eq?` compares `Sexpr`s with `==` and `eval_syntax` `.clone()`s
quote children (`eval_pred.rs:110,182`), so `(eq? (quote DEEP))`/`(eval (quote DEEP))`
abort in **library code** today. Replace the two derives on each type with
hand-written `impl`s using the same explicit-stack idiom (same observable behaviour,
same signatures — no API change):

- **`Clone`** — a two-stack post-order rebuild (mirroring §2.2): a task stack over
  `&self` + a result stack of cloned nodes; a `Reassemble(kind, n)` task pops the `n`
  cloned children and rebuilds the parent. Leaves clone directly.
- **`PartialEq`** — a lockstep walk: a stack of `(&a, &b)` pairs; pop a pair —
  differing variants ⇒ `false`; leaves ⇒ compare the primitive; `List`/`Node` of
  equal arity ⇒ push the child pairs (unequal arity ⇒ `false`); empty ⇒ `true`.

`Debug` stays derived (recursive): it is used only by `assert_*!` **on failure**, so
it never runs on a deep tree that *passes* — the deep tests (§2.9) compare `String`s
and never `assert_eq!` two deep trees, so `Debug` is never invoked at depth (§4 Q4).

## 2.8 Iterative `raise` (`Eml → Sexpr`, `lower.rs:86`)

`raise` is the `Eml → Sexpr` inverse closing R-0016's code↔data square, and it is
recursive + uncapped today — so a deep `Eml → raise → Display → read` cycle overflows
*at `raise`* (nice-guy). Rewrite it as the same two-stack post-order (mirroring §2.2):
`One → Sexpr::num(1)`, `Var → Sexpr::sym`, `Node{exp,log}` → a `Reassemble` that pops
the two raised children and builds `(eml <exp> <log>)`. `raise ∘ lower = id` (on the
reader's image) is preserved by construction and re-tested at depth.

## 2.9 Tests (TDD — written first, red)

Deep cases run in a **subprocess** (a `#[test]` re-invoking a tiny helper bin, or the
harness's own re-exec) asserting a **clean exit status** — because a stack overflow is
an `abort()`, *not* a catchable panic: a child *thread* cannot `join()` it (the prior
draft's claim was wrong — architect + hater), and on the main thread it takes the whole
test binary down. Pass signal: "the subprocess exits 0 at depth 10⁵"; a recursive
regression exits non-zero (SIGABRT) → the assertion fails. Deep fixtures are **built
iteratively** (a recursive generator would itself overflow); the codec fixture
(`ufl-prng` arbitrary-head spine) is **distinct** from the `(eml x (eml x …))` spine
the `lower→eval` leg needs (only `eml`-spines lower — hater).

1. **T-roundtrip-1e5** (AC1): a depth-10⁵ `Sexpr` satisfies
   `display(read(display(s))) == display(s)` — a **`String`** compare, never tree
   `==` (which is the recursion under test). Subprocess exit 0.
2. **T-eml-eval-1e5** (AC1): a depth-10⁵ `(eml x (eml x …))` spine survives
   `read → lower → eval` returning a finite `Value`. Subprocess.
3. **T-reflection-deep** (AC4b — the scope-expansion gate): `(eq? (quote DEEP)
   (quote DEEP)) → Ok(true)` and `(eval (quote DEEP))` complete at depth 10⁵ with no
   library-code abort — the forms that abort today.
4. **T-codec-symmetric** (AC2): depths `{100,128,129,200,1000,10_000}` —
   `read(display(s))` succeeds and `display`-equals `s`; kills the old 128 asymmetry.
5. **T-display-byte-identical** (§2.5): `Display` unchanged vs a frozen string on the
   R-0003 corpus.
6. **T-clone-eq-deep** (§2.7): a depth-10⁵ `Sexpr` and `Eml` clone + structural-eq
   without overflow. Subprocess.
7. **T-drop-1e5** (AC3, regression): depth-10⁵ `Eml` and `Sexpr` drop. Subprocess.
8. **T-shortcircuit** (§2.6): `(and false BAD)`/`(or true BAD)`/`(and (or false
   false) BAD)` → `Ok(false)`/`Ok(true)`/`Ok(false)` with `BAD` unevaluated; a
   depth-10⁵ nested `and`/`not` evaluates. Subprocess.
9. **T-read-errors** (§2.4): `read("(a))")`/`read("a)")`/`read("(a) b")` →
   `TrailingTokens`; `read(")")` → `UnexpectedClose`; `read("(a")` → `UnclosedList`
   — unchanged from today.
10. **T-raise-roundtrip** (§2.8): `raise(lower(read(s))?)` re-`display`s to `s` on the
    R-0003 corpus; a depth-10⁵ `Eml` raises without overflow. Subprocess.
11. **DELETE** `r_0003_acceptance.rs:571` `ac2_deeply_nested_list_is_recursion_depth_
    exceeded` — it asserts the **inverse** of AC2 and references a removed variant
    (won't compile); T-codec-symmetric supersedes it (architect + hater).
12. **T-no-cap** (AC5): a grep test asserts no `get_max_depth`/`set_max_depth`/
    `MAX_DEPTH`/`RecursionDepthExceeded` token remains in `crates/*/src` **or
    `crates/*/tests`** (the stale test lives in `tests/`, invisible to a src-only
    grep — hater).
13. **T-no-panic** (AC4): the **callsite-anchored** `grep -rnE
    '\.unwrap\(|\.expect\(|panic!\('` over the three crates' `src` (test modules
    excluded) is zero — the loose `unwrap|expect|panic!` pattern false-matches
    `unexpected`/`expected` and is unsatisfiable (architect + hater).

## 3. Non-goals

No new reflection form (R-0016). No configurable budget. No `Value → Sexpr`. No
behavioural change to `read`/`lower`/`eval`/`eval_pred`/`Display`/`Clone`/`PartialEq`/
`raise` **other than** removing the depth failure modes (and for `eval_pred`,
preserving short-circuit + error semantics exactly). `Debug` stays recursive (§2.7).
`ufl-geo`'s recursive `eval`/`grade`/`Drop` over `Box<GeoExpr>` (nice-guy — the same
hazard, R-0011's lane, no iterative `Drop` at all) is **out of scope**: a separate
follow-up requirement, surfaced by naming the idiom in `conventions.md`.

## 4. Open questions for the three-lens (re-review)

1. **RESOLVED (2026-07-23):** `eval_pred` is implemented **iteratively** (§2.6) —
   the hater measured `(not)^d` overflowing at 3–4k, foreclosing any "shallow-by-
   domain" bound, and the requirement lists it explicitly.
2. **§2.5:** byte-identical `Display` pin — the codec `quote`/`eval` stand on needs
   it more than a future pretty-printer needs freedom; **proposed: pin now**, a
   pretty-printer is a separate spec. (Architect + hater concur.)
3. Removing the three `RecursionDepthExceeded` variants is a breaking change to three
   public error enums — **straight removal** (pre-1.0 research crate, zero external
   consumers, confirmed by grep — architect). Recorded in the decision log.
4. **New:** `Debug` stays derived/recursive (§2.7) — acceptable because it only runs
   in `assert_*!` failure formatting, never on a passing deep tree, and the deep
   tests never `assert_eq!` two deep trees. Confirm this scoping holds, or add
   iterative `Debug` too.

## 5. Three-lens resolutions (2026-07-23)

Nice-guy **SOLID**, architect **approve-with-changes**, hater **needs-work**. All
folded; the architecture (§2.2 `eval`, §2.5 `Display`, the `unreachable!`s) was
cleared by both. The load-bearing change: the **scope expanded** because the hater
proved the 5-walk scope aborts on the reflection path.

| Lens · finding | Resolution |
|---|---|
| **Hater 1 (BLOCKING)** — derived `Clone`/`PartialEq` recursive; `(eq?/eval (quote DEEP))` abort in lib code | Scope expanded: iterative `Clone`/`PartialEq` for `Sexpr`+`Eml` (§2.7); AC4b + T-reflection-deep |
| **Hater 2 / Architect I5 (BLOCKING)** — 8 MB-thread can't catch an overflow (abort ≠ panic) | §2.9 **subprocess** arena asserting clean exit; the false claim removed |
| **Hater 3 / Architect I5 (BLOCKING)** — stale `r_0003` cap test breaks build + encodes opposite policy; src-only grep misses it | §2.9 test 11 deletes it; test 12 greps `tests/` too |
| **Hater 4 / Architect I1** — `read` trailing-`)` → wrong error; variant names wrong | §2.4 `TrailingTokens` invariant + real names (`UnexpectedClose`/`UnclosedList`) |
| **Hater 5+6 / Architect I2** — `eval_pred` `Frame` phantom `imply`/`iff`; driver underspecified | §2.6 real heads (`and/or/not/=/eq?/pred`), start-sentinel + net-+1 driver invariants |
| **Hater 7 / Architect I2** — "leave recursive" unsafe (3–4k overflow) | §2.6 resolved iterative; §4 Q1 closed |
| **Hater 8 / Architect I4** — `EvalError::RecursionDepthExceeded` omitted; `pub mod depth` line | §2.1 both added to the removal list |
| **Hater 9** — the two 10⁵ fixtures conflate shapes | §2.9 distinct codec vs `eml`-spine fixtures (tests 1 vs 2) |
| **Architect I5** — `T-no-panic` grep unsatisfiable (`expect`⊂`unexpected`) | §2.9 test 13 callsite-anchored `\.unwrap\(|\.expect\(|panic!\(`; R-0017 AC4 too |
| **Architect** — AC5 non-code deliverables unlisted | §6 lists closing #38/#40 + the decision-log entry |
| **Nice-guy** — `raise` unguarded on the same round-trip | §2.8 iterative `raise` in scope |
| **Nice-guy** — promote the idiom + the arena to `conventions.md` | §6 deliverable (arena **corrected** to subprocess, not thread) |
| **Nice-guy** — spec under-claims the R-0016 payoff | §Context/AC4b: `(eval (quote DEEP))` becomes total |

## 6. Deliverables (AC5 + the promotions)

- Close **PR #38 and #40** with supersession notes pointing to R-0017.
- Decision-log entry: the one-policy/no-constant decision + the scope expansion.
- `docs/conventions.md`: **Explicit-Stack Tree Walk** (per-site, no shared helper;
  push-order comment + differential order-tripwire mandatory) and **Bounded-Stack
  Regression Arena** (the **subprocess** technique — corrected from the thread claim).

## 7. Changelog

- 2026-07-23 — revised after the three-lens: scope **expanded** to close the class
  (+ iterative `Clone`/`PartialEq`/`raise`); subprocess test arena; real `eval_pred`
  heads; `read` `TrailingTokens`; the `EvalError` variant + `pub mod` line; grep
  patterns; delete the stale `r_0003` cap test; §5 ledger. Re-review pending.
- 2026-07-16 — created (Draft).
