# SPEC-0017 — One iterative depth contract

- **Realizes:** [R-0017](../requirements/0017-depth-contract.md) AC1–AC5.
- **Status:** **Draft** — three-lens pending (architect + hater; the iterative
  rewrites carry real footguns — §2.6 short-circuit, §2.5 format-preservation).
- **Crates:** `ufl-core` (`eval`, delete `depth`), `ufl-syntax` (`read`, `lower`,
  `Sexpr` `Display`), `ufl-predicate` (`eval_pred`). No new crate; no new public API.
- **Depends on:** R-0001 (Eml), R-0003 (Sexpr). Retro-hardens R-0016.

## 2.1 Remove the cap infrastructure (the policy is *no cap*)

- **Delete** `crates/ufl-core/src/depth.rs` and the `pub use depth::{get_max_depth,
  set_max_depth}` re-export (`lib.rs:17`). Nothing outside `depth.rs` may reference a
  depth budget after this spec (a grep gate in tests).
- **Remove** the now-unreachable error variants `ReadError::RecursionDepthExceeded`
  (`read.rs:24`) and `LowerError::RecursionDepthExceeded` (`lower.rs:27`). Depth is
  no longer a failure mode; no variant replaces them.
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
- atom → parse to `Sexpr::{Num,Sym}` and push onto the top partial list (or, at
  top level with an empty stack, it is the sole result).
- `)` → pop the top partial list, wrap as `Sexpr::List`, and push it onto the new
  top (or record it as the result if the stack is now empty). An unmatched `)` →
  the existing `ReadError::Unexpected`/`Unbalanced` variant (unchanged).
- End of tokens with a non-empty stack → the existing unbalanced-open error.

No `depth` parameter; no cap. The reader now accepts **any** depth `Display` can
emit (AC2).

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

Design: an explicit **continuation stack** of frames, evaluated one operand at a time:

```rust
enum Frame<'a> {
    Eval(&'a Sexpr),                     // evaluate this predicate next
    And(std::slice::Iter<'a, Sexpr>),    // remaining conjuncts; stop on first false
    Or(std::slice::Iter<'a, Sexpr>),     // remaining disjuncts; stop on first true
    Not,                                 // negate the top boolean
    // … one frame per existing pred form (imply, iff, comparison, atom) …
}
```
The driver pops a frame, and for `And`/`Or` inspects the **last produced boolean**
before advancing the iterator — reproducing the recursive short-circuit precisely.
Non-predicate forms and syntax dispatch keep their existing `PredError` behaviour.

**This section's test is the gate** (§2.7 T-shortcircuit): `(and false BAD)` and
`(or true BAD)` — where `BAD` is a form that would `Err` if evaluated — return
`Ok(false)`/`Ok(true)` **both before and after** the rewrite; and a deeply
left-nested `(and (and (and … )))` past 10⁵ evaluates without overflow.

*Open question for the three-lens (§4):* is a full continuation machine warranted,
or is `eval_pred`'s nesting shallow-by-domain (predicates over Hehner states rarely
nest 10⁵ deep) enough to scope AC1/AC3 to `read`/`lower`/`eval`/`Display` and leave
`eval_pred` recursive-but-documented? The requirement lists `eval_pred`; the spec
implements it, but flags the cost/benefit.

## 2.7 Tests (TDD — written first, red)

- **T-roundtrip-1e5** (AC1): a `ufl-prng`-generated **depth-10⁵** `Sexpr` (a
  left-nested spine, built iteratively) survives `read(display(s))  == s`, and
  `lower → eval` on its `eml`-shaped analogue returns the same `Value` — no overflow,
  no panic. Run in a **child thread with a default (8 MB) stack** so a regression
  *fails the test* rather than aborting the process.
- **T-codec-symmetric** (AC2): for depths `{100, 128, 129, 200, 1000, 10_000}`,
  `read(display(s))` succeeds and equals `s` — no depth at which `Display` emits but
  `read` rejects. (Directly kills the old 128 asymmetry.)
- **T-display-byte-identical** (§2.5): `Display` output is unchanged vs a frozen
  string on the R-0003 corpus.
- **T-drop-1e5** (AC3, regression): a 10⁵-deep `Eml` and `Sexpr` drop without
  overflow (guards the existing iterative `Drop`s).
- **T-shortcircuit** (§2.6): `(and false BAD)`/`(or true BAD)` return `Ok(false)`/
  `Ok(true)`; a 10⁵-deep nested `and` evaluates without overflow.
- **T-no-cap** (AC5): a grep-style test asserts no `get_max_depth`/`set_max_depth`/
  `MAX_DEPTH`/`RecursionDepthExceeded` token remains in `crates/*/src`.
- **T-no-panic** (AC4): the `grep -rE 'unwrap|expect|panic!'` gate over the three
  crates' `src` (test modules excluded) is zero; every `unreachable!` has a message.

## 3. Non-goals

No new reflection form (R-0016). No configurable budget. No `Value → Sexpr`. No
behavioural change to `eval`/`eval_pred`/`read`/`lower`/`Display` **other than**
removing the depth failure modes and (for `eval_pred`) preserving semantics exactly.

## 4. Open questions for the three-lens

1. **§2.6 scope:** full continuation machine for `eval_pred`, or leave it recursive
   with a documented domain-bound (predicate nesting is shallow) and scope the
   depth ACs to the four code↔data surfaces? (The hater should weigh the footgun
   vs the reflection loop's actual `eval_pred` depth exposure.)
2. **§2.5:** is a byte-identical `Display` assertion the right pin, or does it
   over-constrain a future pretty-printer? (Proposed: pin it now; a pretty-printer
   is a separate spec.)
3. Removing the `RecursionDepthExceeded` variants is a breaking change to two public
   error enums — acceptable (no external consumers), or deprecate-then-remove?
