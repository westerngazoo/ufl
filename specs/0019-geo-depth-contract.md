# SPEC-0019 — The depth contract on the geometric surface

- **Realizes:** [R-0019](../requirements/0019-geo-depth-contract.md).
- **Status:** **Draft (rev 2)** — three-lens round 1 complete; the three blocking
  findings are folded below. Awaiting re-review.
- **Crates touched:** `ufl-geo`, `ufl-evolve`. No new crate, no new public type.
- **Precedent:** [SPEC-0017](0017-depth-contract.md). This spec applies that
  work's two conventions rather than re-deriving them.

## 1. The policy

Identical to R-0017: **iterative everywhere, no cap, no constant.** `max_nodes`
keeps its value of 60 (R-0019 §3 — decoupling, not tuning).

**Fourteen** recursive walks are in scope (R-0019 §1.1). Two things that table
does **not** say, and this spec is built around:

- **The depths are properties of a 2 MiB thread, not of the code.** `eval` aborts
  at 215 under libtest and survives 800 on an 8 MB main thread. The portable
  invariant is **bytes per frame** — `eval` ≈ 9,754 B/frame debug, ≈ 1,560 B/frame
  release.
- **`grade` is exponential** (R-0019 §1.2), and *an iterative rewrite does not fix
  that*. A stack machine that re-walks is still 2^depth. §2.2b addresses it
  separately, as AC8.

## 2. Design

The same idiom throughout — task stack plus result stack, post-order — applied
**per site**, never through a shared helper (*Explicit-Stack Tree Walk*). Each
site carries a **push-order comment** and a **differential order tripwire** (§4).

### 2.1 `eval` (`ufl-geo/src/eval.rs`)

```rust
enum Task<'a> { Visit(&'a GeoExpr), Apply(&'a GeoExpr) }
```

`Visit` on a leaf pushes an `Mv`; on an internal node it pushes `Apply(node)`
then the children **in reverse**, so the left child evaluates first. `Apply` pops
`(b, a)` in that order — stated in a comment, because `Wedge`/`Inner` are not
commutative and a transposition is silent.

Error order is unchanged: the left subtree fully drains before the right
subtree's tasks are reached.

**Correction to rev 1.** It claimed the 215 ceiling comes from binary nodes
holding two `Mv`s. Measured false — a `Reverse` spine with zero binary nodes
aborts at exactly 215 too. The cause is ~9.7 KB of opt-level-0 frame across the
11-arm match; `Mv` is ~1.3% of it. Consequences:

- **§5 Q2's throughput comparison must run in release**, where the recursive
  frame is ~6× cheaper than debug suggests and the iterative form's `Vec<Mv>`
  allocation is proportionally more visible.
- Size or reuse that `Vec` — `GeoFitness::score` calls `eval` **six times per
  candidate per generation** (`memetic.rs:96-108`).

### 2.2 `grade` + `is_versor` — **option 1**, not option 3

Rev 1 proposed one machine with two typed result stacks. **Changed to two
independent machines** on the architect's reasoning:

- An iterative `grade` calling an iterative `is_versor` adds exactly one call
  frame, not a spine. The depth contract is **fully discharged** by option 1;
  there is no safety argument for merging.
- *Explicit-Stack Tree Walk* says "per site, not via a shared helper". Merging is
  that shortcut, and §3 declares this spec "not a tuning change".
- Rev 1's safety claim was **overstated**: two typed stacks remove *variant*
  confusion, not *value* confusion. Two non-empty stacks holding the wrong
  entries yield a silent wrong `GradeSet` and no underflow.

Merging does **not** endanger the conservative predicate — both functions are
total and pure, so eager evaluation is observationally identical — but that was
never the reason to merge.

### 2.2b Single-visit `grade` (AC8) — the constraint that actually binds

R-0019 §1.2: `grade`'s `Sandwich` arm computes `grade(b)` inside `is_versor(r)`
and again inside `grade(r)` on the non-versor branch ⇒ 2^depth. Measured 71 ms
`typecheck` on a **55-node** genome, inside today's cap, against 62 µs for `eval`.

This is **not** solved by §2.2. It is a separate, explicitly measured change:

- Compute `grade(r)` **once** per `Sandwich` node and have both the versor test
  and the product-bound rule read that one result. Since `is_versor(Exp(b))` is
  `subset_of(grade(b), &[2])` (`grade.rs:68`), the versor test is a *predicate on
  a `GradeSet` the machine already has* — it does not need its own walk of `b`.
- **Assert the mechanism, not the clock** (*Structural Frugality over
  Wall-Clock*): a **node-visit counter** asserting `grade` visits each node at
  most *c* times. This fails loudly on any future rule that re-walks a child and
  is stable on shared CI. A timing bound is neither.
- §2.3's `typecheck` gets the same treatment: it calls `grade(e)` at every node
  (`grade.rs:176`), making it **O(n²)** over an already-exponential `grade`. The
  iterative machine already holds the children's `GradeSet`s on its result stack;
  threading them up makes it O(n) with identical values, since `grade` is pure.

**AC6 depends on this.** At `max_nodes = 5,000` a rotor-nested genome does not
terminate in `grade`, whatever `eval` does.

### 2.3 `typecheck` — the full error precedence

Rev 1 stated leaf validation precedes descent. That is necessary but
**incomplete**. The order the iterative form must reproduce:

1. own `BadBlade`/`BadGrade` — in `Visit`, before pushing children;
2. children, left to right;
3. own `Incoherent` — in `Apply`, **post-order** (`grade.rs:176-178`, after both
   children fully typecheck).

A machine that checks emptiness in `Visit` reports a parent's `Incoherent` before
a child's `BadBlade` — a silent reordering of a public error.

### 2.4 `render` — rev 1's frame machine was wrong

Three defects, all confirmed against `render.rs`:

**(a) The rotor name is emitted twice.** `render.rs:132` and `:136`, straddling
`factor(x)` at `:134`. The name is only known when `BindRotor` *executes*, but
the frame that emits it the second time is already on the stack. `Lit(&'static
str)` cannot carry it. A **third stack** is required — `Vec<String>` of pending
rotor names, pushed by `BindRotor`, popped by a new `EmitBoundName` frame. The
push/pop discipline is properly balanced under both nesting shapes, so LIFO is
correct.

**(b) `GradeProject` closes with a runtime `k`.** `⟩_{k}` is emitted *after* the
child (`render.rs:110`), so it needs `CloseProject(u8)` (or an `Owned(String)`
frame). `𝒢_{k}(` at `:114` is pre-order and can be written at visit time.

**(c) The atom-rotor branch needs no sink at all.** `render.rs:121-125` also
needs its text twice but does **not** bind. Since `is_atom` ⟺ leaf
(`render.rs:57-59`), compute the leaf text inline and push two `Owned(String)`
frames. This confines `OpenSink`/`BindRotor` to the non-atom case and simplifies
the machine materially.

```rust
enum Frame<'a> {
    Node(&'a GeoExpr),
    Factor(&'a GeoExpr),       // parenthesised unless atom or self-delimiting
    Lit(&'static str),
    Owned(String),             // (b), (c)
    CloseProject(u8),          // (b)
    OpenSink,                  // non-atom rotor only (c)
    BindRotor,                 // pop sink, ctx.fresh(), push to ctx.lets and to the name stack
    EmitBoundName,             // (a) — pops the name stack
}
```

**The ordering invariant in rev 1 was backwards.** Measured against the current
renderer:

```
Sandwich(r1, Sandwich(r2, v))  →  let R = exp(1 e₁₂) / let S = exp(2 e₁₂) / R (S v ~S) ~R
Sandwich(Sandwich(r2, v), v)   →  let R = exp(2 e₁₂) / let S = R v ~R     / S v ~S
```

Body-nesting binds **outer first**. Rev 1's "inner-before-outer" holds only for
rotor-nesting and is false for the commoner shape. The true invariant:

> `ctx.fresh()` fires in the order sandwich nodes finish rendering their **rotor**
> subtree — rotor-subtree-first, left to right — which a LIFO frame stack
> preserves by construction.

An implementer coding to rev 1's stated rule gets `R (S v ~S) ~R` wrong, and the
output would still be well-formed GA notation — so only T-render-nested-sandwich
catches it.

**Byte-identity is achievable.** `render` is total, infallible, and has no
re-entrancy or early return; its only order-dependence is call order, which the
frame stack reproduces exactly. `ctx.lets` must stay a **single flat vector
outside the sink stack** — the `let S = R v ~R` case shows a definition
referencing an earlier name, which only works with one shared `Ctx`.

`render` also holds the crate's only library `.unwrap()`s (`render.rs:110,114,150`).
Writing to a `String` is infallible, so these become `push_str` with a
pre-formatted number — no suppression, no `let _ =`.

### 2.5 `collect` (`ufl-geo/src/slots.rs`) — verified sound

A `Vec<&'a mut GeoExpr>` work-stack **compiles in safe Rust** (architect built
it): the scrutinee is moved into the `match`, so bindings are reborrows at `'a`
of disjoint fields. No `unsafe`, no reformulation.

Two additions rev 1 missed:

- **Push right, then left**, so left pops first and pre-order survives.
  `lane.rs:84`'s `params_mut(..).nth(i)` depends on this ordering.
- `params` (`slots.rs:29-32`) is **two** walks — `e.clone()` then `collect` — so
  it is only depth-safe once §2.6's `Clone` lands. Whether `params` keeps the
  clone (agreement by construction) or gets its own `&GeoExpr` loop (no clone,
  but T-slots-agreement then actually tests something) is a decision to record,
  not inherit.

### 2.6 `GeoExpr`'s trait impls

- **`Clone`** — two-stack post-order rebuild.
- **`PartialEq`** — lockstep pair-stack; `f64` keeps derive semantics
  (`NaN != NaN`, `-0.0 == 0.0`).
- **`Debug`** — see below.
- **`Drop`** — see below.

**`Debug`: byte-identity is a *choice*, and rev 1 over-justified it.** Rev 1 said
it "appears in error payloads and in existing test assertions". The assertion
claim is false: the only `Debug` assertion on a `GeoExpr` is
`r_0010_acceptance.rs:88`, which asserts merely `!format!("{form:?}").is_empty()`;
`r_0010_soundness.rs:71,142` use `{e:?}` only in failure messages. `{:#?}`
appears **zero** times repo-wide.

So the contract must be stated deliberately:

- **`{:?}` — in contract, byte-identical.** `Box<GeoExpr>` is transparent (no
  `Box(...)` wrapper); `, ` separators; no trailing comma.
- **`{:#?}` — out of contract.** Reproducing it means reimplementing
  `core::fmt::builders::PadAdapter`'s indentation by hand, because
  `f.debug_tuple(..).field(&child)` nests by *calling* `Debug::fmt` on the child —
  the very recursion being removed, so the builders are unusable. Given zero
  repo-wide uses, alternate mode falls back to a documented non-identical form
  rather than paying that cost. **This is a deliberate, recorded limitation.**
- **Leaves must delegate** to the primitives' `Debug`, never hand-format: `f64`
  prints `1e300`, `-0.0`, `NaN`, `inf`; `String` escapes `\n` but not emoji.

**`Drop`: two consequences rev 1 didn't record.**

- **`impl Drop for GeoExpr` makes the type permanently non-destructurable by
  value** (E0509). Nothing does that today in either crate, but it is an
  irreversible constraint on a `pub` enum that R-0015's operator work will build
  on. Accepted, and recorded as a cost.
- **The `Eml` idiom does not transplant.** `eml.rs:120-140` is allocation-free on
  re-entry only because its sentinel `Eml::One` is a *different variant* from its
  one internal variant. `GeoExpr` has **eight** internal variants: a shell
  `Sandwich(Basis(0), Basis(0))` still matches its arm on re-entry and pushes two
  sentinels — **one `Vec` allocation per internal node**, O(n) allocations to drop
  an n-node tree, on the path `vary` takes for the whole population every
  generation. Guard the take:

  ```rust
  if !matches!(**slot, GeoExpr::Basis(0)) {
      stack.push(mem::replace(&mut **slot, GeoExpr::Basis(0)));
  }
  ```

  so the shell pushes nothing and `Vec::new()` never allocates.

Once these four land, that block is ~200 lines of machinery on a 24-line enum; it
belongs in its own module (`expr/traits.rs`), keeping the data declaration
legible.

### 2.7 `GradeError` — **decision: keep the payload (option a)**

Rev 1 deferred this to the lens. Both lenses answered, and they disagreed; the
architect's cost model is decisive because rev 1's was wrong.

**Rev 1's claim that (b) makes the error "O(text) instead of O(tree)" is false.**
Construction is still O(tree), and `render` is strictly *more* expensive than
`Clone` — same nodes, plus a `log10`, a `format!`, and two `trim_end_matches` per
`Param` (`render.rs:163-176`), plus `ctx.lets` maintenance. **Option (b) makes the
hot path worse.**

The options as they finally stood:

- **(a) keep `Incoherent(GeoExpr)`** — §2.6 makes the derives safe for free; zero
  new surface; no breaking change.
- **(b) `Incoherent(String)`** — negative-measured benefit, breaking change.
- **(c) `Incoherent { at: usize, rendered: String }`** (nice-guy) — genuinely
  better than (b): the pre-order index composes with `replace_nth` for a future
  `GradeRepair` refiner. But it inherits (b)'s construction cost and its breaking
  change.

**Chosen: (a).** CLAUDE.md §6 wants typed, non-lossy errors; `grade.rs:43` calls
`GradeError` "the decidable pruning signal R-0011 uses", and a signal carrying the
subtree composes with the repair operator (c) is really arguing for — you can
always derive `render(&e)` from the tree, never the tree from the string. **(c)'s
insight is recorded as a follow-up**: if a repair refiner is built, add the
pre-order index *alongside* the tree rather than instead of it.

**But name the hot path.** `GradeScreen::admissible` (`lane.rs:47`) runs
`typecheck(..).is_ok()` on every candidate, and every rejected one clones a
subtree and drops it. Mitigating: `typecheck` recurses into children *before* its
own emptiness check, so it clones the **innermost** incoherent subtree, not the
root — small at 60 nodes, not small at 5,000. §4's throughput test must therefore
cover the **screen** path at `max_nodes = 5,000`, not just `eval`.

**The chain, named for auditability (AC2b):** `GeoExpr` → `GradeError`
(`grade.rs:49`) → `GeoLaneError` (`lane.rs:23`, `#[from]`) → `RunError<E>`
(`ufl-search/src/lib.rs:56`). All derive `Debug`/`Clone`/`PartialEq`; all are
transitively covered by §2.6 under option (a).

### 2.8 `ufl-evolve`

`node_count`, `nth_subtree`, `replace_nth` iterative by the same idiom.
`replace_nth` rebuilds, so it takes §2.6's `Clone` shape with a substitution at
the counted index; its pre-order index must match `nth_subtree`'s exactly, since
crossover pairs them.

`random_expr` (`memetic.rs:182`) is a recursive **generator** bounded by the `pub`
`max_depth` field. AC6 raises `max_nodes` to 5,000 while leaving this recursive at
`max_depth: 4` — safe, but stated explicitly rather than left implicit, since the
convention's own note warns that recursive generators overflow before the code
under test runs.

### 2.9 The lint (AC7)

Both crates adopt `#![cfg_attr(not(test), deny(clippy::unwrap_used,
clippy::expect_used, clippy::panic))]`. Measured: three `write!(…).unwrap()` in
`ufl-geo` (§2.4), zero in `ufl-evolve` — the three `.expect(` there are inside
`#[cfg(test)] mod tests` and therefore exempt.

## 3. Non-goals

- **No tuning.** `max_nodes` stays 60.
- **No grade-semantics change.** The versor predicate stays conservative;
  §2.2b changes only how many times a subtree is visited, never the answer.
- **No new geometric form**, no `unsafe`, no `{:#?}` fidelity (§2.6).

## 4. Tests (TDD — written first, red)

1. **T-differential (AC4)** — each walk fuzzed against a verbatim transcription of
   the pre-change function: identical values **and** error precedence. `render`
   and `Debug` (`{:?}` only) byte-identical.
2. **T-is-versor-direct** — `is_versor` is `pub(crate)`, so an integration test
   can observe it only through `grade(Sandwich(r, x))`, where the versor branch
   and the product-bound branch coincide whenever `grade(r) = {0}` — masking a
   wrong answer. Its fuzz must live in an in-`src` `#[cfg(test)]` module.
3. **T-order-tripwire** — the corpus must include non-commutative shapes
   (`Wedge`, `Inner`, `Sandwich`, `replace_nth` at index > 0). Underflow guards
   arity; only a differential catches a transposition. For `collect`, the seeded
   Gate-1 e2e is the real oracle — `params`/`params_mut` agree by construction
   (`slots.rs:31`) whatever the order, but a transposition changes which slot
   `lane.rs:84` perturbs and therefore the reported N/16.
4. **T-render-nested-sandwich** — both nesting shapes from §2.4, asserting
   `ctx.lets` order and rotor-name assignment. A flat corpus does not exercise
   these.
5. **T-arena (AC5)** — every walk at depth 10⁵ in a subprocess arena, pinned to
   the `dev` profile **and to an explicit stack size** (§1). Asserts the child's
   exit status **and** `1 passed`.
6. **T-arena-can-fail** — the arena shown to fail by name before it is trusted.
   A recorded PR step, not a committed test.
7. **T-grade-visit-count (AC8)** — a node-visit counter proving `grade` visits
   each node at most *c* times. The mechanism, not a clock.
8. **T-decoupling (AC6)** — the lane completes at `max_nodes = 5,000`. Asserts
   completion, not fitness. **Requires AC8.**
9. **T-throughput** — `eval` **in release** on production shapes (≤ 60 nodes,
   depth ≤ 32 measured), *and* the `GradeScreen` path at `max_nodes = 5,000`.
   Reported unconditionally.
10. **T-lint (AC7)** — probe each crate, confirm clippy fails, revert.

**Ordering constraint:** §2.6's iterative `Drop` must land **before or with** the
arena. A depth-10⁵ fixture still has to be torn down, and drop glue aborts at
~18,801 — every arena case aborts at teardown until `Drop` exists.

## 5. PR seam

Rev 1 proposed the crate boundary. **Wrong seam** — it puts 10 walks, both hard
ones, the arena, the lint, and the `GradeError` decision on one side and 3
mechanical ones on the other. Four PRs, ordered by the dependencies above:

1. **PR1 — foundation:** `Clone`/`PartialEq`/`Debug`/`Drop` + the arena + the
   lint. First: `Drop` unblocks every arena case, `Clone` unblocks `params`, and
   the E0509 fallout is isolated here.
2. **PR2 — value walks:** `eval`, `grade` (+ `is_versor`), **AC8's single-visit
   rule**, `typecheck`, `collect`, T-throughput.
3. **PR3 — `render` alone.** The only non-linear-output walk, with its own sink
   stack, name stack, and byte-identity corpus (§2.4).
4. **PR4 — `ufl-evolve`** + AC6's decoupling test, green only once both crates
   are iterative.

If four is too many, fold PR4 into PR2. **Do not fold PR3.**

## 6. Open questions for three-lens round 2

1. **Is AC8 (§2.2b) in this requirement or its own?** It is a *complexity* fix,
   not a depth fix, and R-0019 §1.2 shows it is the binding constraint. Argument
   for keeping it: AC6 is unreachable without it. Argument for splitting: it has
   nothing to do with the depth contract and drags a performance change into a
   safety requirement.
2. **Does `params` keep its clone** (§2.5)? Agreement-by-construction versus a
   test that actually tests something.
3. **Is `{:#?}` out of contract acceptable** (§2.6)? Zero repo-wide uses today,
   but it is a permanent, documented divergence from a derive on a `pub` type.
4. **Is R-0019 worth building at all**, given R-0019 §1.3 measured that nothing
   aborts and nothing blows up in the real search? The honest case is conditional
   on raising `max_nodes` — and if that experiment is not planned, this is latent
   safety and nothing more.
