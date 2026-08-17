# SPEC-0019 — The depth contract on the geometric surface

- **Realizes:** [R-0019](../requirements/0019-geo-depth-contract.md).
- **Status:** **Draft** — awaiting the three-lens (CLAUDE.md §4 step 2).
- **Crates touched:** `ufl-geo`, `ufl-evolve`. No new crate, no new public type.
- **Precedent:** [SPEC-0017](0017-depth-contract.md) discharged this exact class on
  `Sexpr`/`Eml`. This spec applies the two conventions that work produced
  (*Explicit-Stack Tree Walk*, *Bounded-Stack Regression Arena*) and does **not**
  re-derive them.

## 1. The policy, restated

Identical to R-0017: **iterative everywhere, no cap, no constant.** Depth is
bounded by the heap. Nothing here introduces a `MAX_DEPTH`, and `max_nodes`
keeps its current value of 60 (R-0019 §3 — decoupling, not tuning).

**Thirteen** recursive walks are in scope, all measured (R-0019 §1, plus §2.2
below for `is_versor`). Ordered by how soon they die:

| # | walk | crate | aborts at |
|---|------|-------|-----------|
| 1 | `eval` | `ufl-geo` | **215** |
| 2 | `render` (`node`/`factor`) | `ufl-geo` | 2,480 |
| 3 | `typecheck` | `ufl-geo` | 2,799 |
| 4 | `replace_nth` | `ufl-evolve` | 3,656 |
| 5 | `collect` (`params`/`params_mut`) | `ufl-geo` | 4,113 |
| 6 | `Clone` (derived) | `ufl-geo` | 4,113 |
| 7 | `grade` | `ufl-geo` | 4,700 |
| 8 | `is_versor` | `ufl-geo` | 4,701 † |
| 9 | `PartialEq` (derived) | `ufl-geo` | 4,701 |
| 10 | `Debug` (derived) | `ufl-geo` | 5,060 |
| 11 | `node_count` | `ufl-evolve` | 16,458 |
| 12 | `Drop` (glue) | `ufl-geo` | 18,801 |
| 13 | `nth_subtree` | `ufl-evolve` | 26,332 |

† `is_versor` is mutually recursive with `grade` (it calls `grade` on a rotor's
bivector). Measured on a `Sandwich` spine it does **not** worsen the bound —
4,701 vs `grade`'s 4,700 — because its recursion is bounded by the *rotor*
subtree, not by the spine. It is in scope because it is a recursive walk on a
tree, not because it is currently the binding constraint.

## 2. Design

Every rewrite is the same idiom — a task stack plus a result stack, post-order —
applied **per site**, never through a shared helper (the walks return different
types and have different error disciplines; *Explicit-Stack Tree Walk* forbids
the shared-helper shortcut for exactly this reason). Each site carries a
**push-order comment** wherever a reversal would transpose operands, and each is
covered by a **differential order tripwire** (§4).

### 2.1 `eval` (`ufl-geo/src/eval.rs`) — the one that matters

```rust
enum Task<'a> { Visit(&'a GeoExpr), Apply(&'a GeoExpr) }
```

A `Visit` on a leaf pushes an `Mv`; on an internal node it pushes
`Apply(node)` then its children **in reverse** so the left child evaluates first.
`Apply` pops the operands and applies the node's operator. Binary nodes pop
`(b, a)` in that order — the comment must say so, since `Wedge`/`Inner` are not
commutative and a transposition is silent.

Error discipline is unchanged: `BadBlade`/`BadGrade`/`Unbound` surface at the
same node and in the same left-to-right order, because a `Visit` of the left
subtree fully drains before the right subtree's tasks are reached.

**Why this walk is the point of the requirement:** its 215-deep ceiling comes
from frame *size* — each frame holds an `Mv` (16 × `f64`) and binary nodes hold
two at once. Moving those to a heap `Vec<Mv>` is what lifts the ceiling by orders
of magnitude, and it is also the throughput risk (§5 Q2).

### 2.2 `grade` + `is_versor` (`ufl-geo/src/grade.rs`) — two results, one machine

`grade` returns a `GradeSet`; `is_versor` returns a `bool`; `grade`'s `Sandwich`
arm consults `is_versor` and branches on it. Three options, and the spec picks
the third:

1. two independent machines — `is_versor` re-walks subtrees `grade` has already
   walked (today's behaviour, preserved but wasteful);
2. one machine, one `enum Out { Grade(GradeSet), Versor(bool) }` result stack —
   type-unsafe in the sense that a mis-sequenced pop yields the wrong variant and
   an `unreachable!`;
3. **one machine with two typed result stacks** (`Vec<GradeSet>` and
   `Vec<bool>`) and a task enum spanning both — a mis-sequence is then a stack
   underflow at a named site, not a variant confusion.

Option 3 keeps the same **conservative** versor predicate: `is_versor` may still
answer `false` for a genuine versor, and the grade rule still falls back to the
sound product bound. This spec changes **no** grade semantics; AC4's differential
fuzz is what proves it.

### 2.3 `typecheck` (`ufl-geo/src/grade.rs`) — eager error, unchanged order

Same machine shape, `Result`-valued. The existing implementation validates a node
**before** descending, so `BadBlade`/`BadGrade` precede any child's
`Incoherent`. The iterative form must preserve that by validating in the `Visit`
arm, before pushing children — the same discipline SPEC-0017 §2.3 used for
`lower`.

### 2.4 `render` (`ufl-geo/src/render.rs`) — the hard one

`render` is the only walk with a **non-linear output discipline**. Its `Sandwich`
arm renders the rotor into a *separate* buffer, appends `(name, def)` to
`ctx.lets`, and emits only the name. So the machine cannot append to one output
string: it needs a **stack of sinks**.

```rust
enum Frame<'a> {
    Node(&'a GeoExpr),          // render into the current sink
    Factor(&'a GeoExpr),        // as Node, but parenthesised unless atom/self-delimiting
    Lit(&'static str),          // a literal separator, emitted in order
    OpenSink,                   // push a fresh String — the rotor's `def`
    BindRotor,                  // pop the sink, ctx.fresh(), push to ctx.lets, emit the name
}
```

Three observable properties must survive **byte-identically**, and each is a
tripwire in §4:

1. **the output bytes** — separators, parens, `⟨⟩_k`, `𝒢_k(…)`;
2. **the `ctx.next` allocation order** — which rotor gets `R` vs `S`. This is
   determined by the order `BindRotor` frames *execute*, which is not the order
   they are *pushed*; a nested sandwich must still bind inner-before-outer
   exactly as the recursive version does;
3. **the `ctx.lets` order** — emitted as a `let` prelude in dependency order.

Because (2) and (3) are order-sensitive in a way the other twelve walks are not,
`render` gets the strictest test: byte-equality against the pre-change function
on a large random corpus **including nested `Sandwich`es**.

`render` also holds the crate's only library-code `.unwrap()`s (three
`write!(…).unwrap()` at `render.rs:110,114,150`). Writing to a `String` is
infallible, so these become `push_str` with a pre-formatted number — no
suppression, no `let _ =`. This is what makes AC7 cheap here.

### 2.5 `collect` (`ufl-geo/src/slots.rs`) — the `&mut` walk

`params_mut` returns `Vec<&'a mut f64>` — simultaneous mutable borrows of
disjoint leaves. The iterative form keeps that soundness argument intact: a
`Vec<&'a mut GeoExpr>` work-stack, where matching on `&mut` **moves** the borrow
into disjoint child borrows, so no two stack entries alias. No `unsafe`.

Pre-order must be preserved exactly — `params` and `params_mut` are documented to
agree index-for-index, and R-0011's refiner indexes into the result.

### 2.6 `GeoExpr`'s trait impls — the class closure

Hand-written iterative `Clone`, `PartialEq`, and `Debug`, plus an iterative
`Drop` (there is none today; the type relies on drop glue). `Debug` is in scope
here although SPEC-0017 deferred it, because §2.7's error embeds a tree — see
R-0019 §5 Q1.

- **`Clone`** — two-stack post-order rebuild, mirroring §2.1.
- **`PartialEq`** — a lockstep `(&a, &b)` pair-stack; differing variants ⇒
  `false`; leaves compare primitives; equal-arity children push pairs.
  `f64` comparison keeps derive semantics exactly (`NaN != NaN`, `-0.0 == 0.0`).
- **`Debug`** — output **byte-identical to the derive**, because it appears in
  error payloads and in existing test assertions. This is the only impl here
  whose contract is "reproduce what the compiler generated", so it is pinned by a
  frozen-corpus differential test rather than by inspection.
- **`Drop`** — the `Sexpr`/`Eml` idiom: take the children into an explicit stack
  and drop them iteratively.

### 2.7 `GradeError` — **the open design trade (R-0019 §5 Q4)**

`GradeError::Incoherent(GeoExpr)` embeds a tree and is built by cloning it
(`grade.rs:178`), so producing the error is itself a recursive walk on a `pub`
fn's failure path, and `GradeError`'s derived `Clone`/`PartialEq`/`Debug` are
recursive walks over caller data.

**This spec deliberately does not choose.** Both options are viable and the
trade is about the crate's diagnostic contract, not about depth:

- **(a) Keep the payload.** `GeoExpr`'s iterative impls (§2.6) make the derives
  on `GradeError` safe automatically — no further work. Callers keep structured
  access to the offending subtree. Cost: a `pub` error stays as expensive to
  clone as the tree it holds, and every future field of this kind repeats the
  question.
- **(b) Replace it with `Incoherent(String)`** holding `render(e)`. Cheap,
  already human-readable, and the error becomes `O(text)` instead of `O(tree)`.
  Cost: a **breaking change** to a `pub` enum, and callers lose structured
  access — `r_0010_acceptance.rs:416` matches `Incoherent(_)` and would still
  compile, but any future caller wanting the subtree could not have it.

The three-lens is asked to argue this (§5 Q1). Whichever is chosen, **AC2b is
satisfied**: under (a) by §2.6, under (b) by construction.

### 2.8 `ufl-evolve` — the search-side walks

`node_count`, `nth_subtree`, and `replace_nth` become iterative by the same
idiom. `replace_nth` is the only one that *rebuilds*, so it takes the §2.6
`Clone` shape with a substitution at the counted index; its pre-order index must
match `nth_subtree`'s exactly, since crossover pairs them.

This is what stops the anti-bloat guard from sharing the failure mode of the
thing it guards (R-0019 §2).

### 2.9 The lint (AC7)

`ufl-geo` and `ufl-evolve` adopt
`#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]`.
Measured cost: **three** `write!(…).unwrap()` in `ufl-geo` (§2.4) and **zero** in
`ufl-evolve`. Two of [#82](https://github.com/westerngazoo/ufl/issues/82)'s seven
crates close here.

## 3. Non-goals

- **No tuning.** `max_nodes` stays 60. Whether raising it helps Gate-1 is a
  separate R-0011 experiment this merely makes runnable.
- **No grade-semantics change.** The versor predicate stays conservative.
- **No new geometric form**, no signature changes, no `unsafe`.

## 4. Tests (TDD — written first, red)

1. **T-differential (AC4)** — for each of the thirteen walks, a fuzz over
   randomly generated `GeoExpr`s (bounded depth, all variants, including
   out-of-range `Basis`/grade indices and unbound `Var`s) comparing the iterative
   result against a **verbatim transcription of the pre-change function**:
   identical values **and identical error precedence**. `render` and `Debug`
   compare **byte-identically**.
2. **T-order-tripwire** — the differential corpus must include non-commutative
   shapes (`Wedge`, `Inner`, `Sandwich`, `replace_nth` at index > 0). A stack
   underflow guards arity; only a differential catches a transposition.
3. **T-render-nested-sandwich** — nested `Sandwich`es specifically, asserting
   `ctx.lets` order and rotor-name assignment are unchanged (§2.4's properties 2
   and 3, which a flat corpus would not exercise).
4. **T-arena (AC5)** — every walk at depth **10⁵** in a subprocess arena pinned
   to the `dev` profile, per *Bounded-Stack Regression Arena*. The arena asserts
   the child's exit status **and** that it reports `1 passed` — R-0019 §6 records
   a false-pass caused by omitting exactly that check.
5. **T-arena-can-fail** — the arena is shown to fail by name when a recursion is
   reintroduced, before it is trusted. Not a committed test; a recorded step in
   the PR, per the convention.
6. **T-decoupling (AC6)** — the lane runs to completion with `max_nodes` set to
   5,000 (a value that aborts `eval` today at 215). Asserts *completion*, not a
   fitness improvement — the protocol, not the outcome.
7. **T-throughput (AC4, §5 Q2)** — `eval` on the shapes the search actually
   produces (≤ 60 nodes, depth ≤ 4–6), iterative vs recursive, reported
   **unconditionally** in the PR. See §5 Q2 for what the number gates.
8. **T-slots-agreement** — `params` and `params_mut` still agree index-for-index
   and in pre-order (§2.5).
9. **T-lint (AC7)** — probe each crate with a `panic!`/`.unwrap()` and confirm
   `cargo clippy --workspace --all-targets -- -D warnings` fails; revert.

## 5. Open questions for the three-lens

1. **`GradeError`'s payload (§2.7) — (a) keep or (b) `String`?** The spec is
   deliberately undecided. Architect: which is the better public contract?
   Hater: what breaks in the wild under each? Nice-guy: does either unlock
   something later — e.g. does a structured payload help the R-0015 operator work?
2. **What does T-throughput gate?** `eval` is called for every candidate every
   generation, so a regression on ≤ 60-node trees is a real cost paid for a
   ceiling nothing currently touches. *Proposal: if iterative `eval` is more than
   10% slower on production-shaped trees, keep the iterative form (correctness)
   but record the number and open a follow-up — do not silently accept it, and do
   not tune the shape until it is measured.* Is 10% the right line, and is
   "record and continue" the right response?
3. **Is `is_versor` worth including now?** It is a recursive walk, but it is not
   the binding constraint on any measured path (§1 †). Including it is cheap and
   closes the class; excluding it keeps the diff smaller. *Recommendation:
   include — a partial closure of a class is how R-0017's first scope was wrong.*
4. **Thirteen walks in one PR — is that too big to review?** SPEC-0017's seven
   were already at the limit of a reviewable diff. *Proposal: split into two PRs
   along the crate boundary — `ufl-geo` (§2.1–§2.7) then `ufl-evolve` (§2.8) —
   with the arena landing in the first. Is that the right seam, or should
   `render` (§2.4, by far the most intricate) be its own?*
