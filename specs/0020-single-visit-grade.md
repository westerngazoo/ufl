# SPEC-0020 — One pass for `grade`, `is_versor`, and `typecheck`

- **Realizes:** [R-0020](../requirements/0020-single-visit-grade.md) (Accepted).
- **Status:** **Draft** — awaiting the three-lens (CLAUDE.md §4 step 2).
- **Crate touched:** `ufl-geo`, one file — `src/grade.rs`. No public signature
  changes; `GradeSet`, `GradeCtx`, `GradeError`, `grade`, `typecheck` keep their
  types exactly. `is_versor` stays `pub(crate)`.

## 1. The defect, precisely

`grade.rs:120-131` — `Sandwich(r, x)` calls `is_versor(r)`, which for `Exp(b)`
computes `grade(b)`; the non-versor branch then computes `grade(r)`, whose `Exp`
arm computes `grade(b)` again. Two full walks of `r` per `Sandwich` whose rotor
is not a provable versor. When the rotor *itself* contains such a `Sandwich`, the
work doubles per level: **2^depth**.

`grade.rs:176` — `typecheck` calls `grade(e)` at every node of its own recursion,
so it re-walks each subtree once per ancestor: **O(n²)**, over the above.

Measured (R-0020 §1–§3): 204 ms at ~70 nodes on the triggering shape; flat
microseconds on every other shape tried, including motor chains.

## 2. Design

### 2.1 The invariant: `grade` and `is_versor` are one function with two outputs

Every arm of `grade` depends only on its children's grade sets, and every arm of
`is_versor` depends only on its children's versor flags — except the two that
cross: `is_versor(Exp(b))` reads `grade(b)`, and `grade(Sandwich(r, x))` reads
`is_versor(r)`. So the pair is a single bottom-up function returning both:

```rust
/// What one post-order visit learns about a node: its sound grade
/// over-approximation and whether it is *provably* a versor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Analysis {
    grade: GradeSet,
    versor: bool,
}
```

The per-node rule is written **once**, over the children's analyses:

```rust
/// The grade/versor rule at one node, given its children's analyses in order.
/// Total: out-of-range leaves yield ⊤ (or ∅ for a bad projection), exactly as
/// `grade` does today (SPEC-0010 §2.3) — it is `check` that turns those into
/// errors.
fn rule(e: &GeoExpr, kids: &[Analysis], ctx: &GradeCtx) -> Analysis {
    match (e, kids) {
        (GeoExpr::Param(_), [])       => Analysis { grade: GradeSet::singleton(0), versor: false },
        (GeoExpr::Basis(i), [])       => Analysis {
            grade:  if *i >= 16 { GradeSet::full(N) } else { GradeSet::singleton(i.count_ones() as usize) },
            versor: *i < 16 && i.count_ones() == 1,
        },
        (GeoExpr::Var(name), [])      => Analysis { grade: ctx.get(name), versor: false },
        (GeoExpr::GradeLift(k, _), _) => Analysis {   // the child's analysis is not consulted (as today)
            grade: if *k > 4 { GradeSet::full(N) } else { GradeSet::singleton(*k as usize) },
            versor: false,
        },
        (GeoExpr::GeoProduct(..), [a, b]) => Analysis {
            grade:  Op::Geometric.output_grades(&[a.grade, b.grade], N),
            versor: a.versor && b.versor,
        },
        (GeoExpr::Wedge(..), [a, b])  => Analysis { grade: Op::Wedge.output_grades(&[a.grade, b.grade], N), versor: false },
        (GeoExpr::Inner(..), [a, b])  => Analysis { grade: Op::Inner.output_grades(&[a.grade, b.grade], N), versor: false },
        (GeoExpr::Reverse(_), [a])    => Analysis { grade: Op::Reverse.output_grades(&[a.grade], N), versor: a.versor },
        (GeoExpr::GradeProject(k, _), [a]) => Analysis {
            grade: if *k > 4 { GradeSet::EMPTY } else { Op::GradeProject(*k).output_grades(&[a.grade], N) },
            versor: false,
        },
        (GeoExpr::Sandwich(..), [r, x]) => Analysis {
            grade: if r.versor {
                x.grade                                   // a versor sandwich preserves grade
            } else {
                let rx = Op::Geometric.output_grades(&[r.grade, x.grade], N);
                Op::Geometric.output_grades(&[rx, r.grade], N)   // the sound product bound
            },
            versor: false,
        },
        (GeoExpr::Exp(_), [a]) => Analysis {
            grade: if subset_of(a.grade, &[0]) { GradeSet::singleton(0) }
                   else if subset_of(a.grade, &[0, 2]) { GradeSet::EMPTY.with(0).with(2).with(4) }
                   else { GradeSet::full(N) },
            versor: subset_of(a.grade, &[2]),
        },
        _ => unreachable!("`children` and `rule` disagree on a node's arity"),
    }
}
```

The slice patterns make arity a **compile-visible** property of each arm, and the
single `unreachable!` is justified: `children` (below) is the one source of arity,
so a mismatch is a programming error in this file, not a reachable state.

Every cross-dependency is now a field read: `Sandwich` reads `r.versor`; `Exp`
sets `versor` from `a.grade`. **No arm calls the walk.** That is the whole fix.

### 2.2 The total walk — `analyse`

```rust
/// The children of a node, in left-to-right order.
fn children(e: &GeoExpr) -> impl Iterator<Item = &GeoExpr> { /* 0, 1, or 2 */ }

/// One post-order pass. Each node is visited exactly once.
fn analyse(e: &GeoExpr, ctx: &GradeCtx) -> Analysis {
    let kids: Vec<Analysis> = children(e).map(|c| analyse(c, ctx)).collect();
    rule(e, &kids, ctx)
}

pub fn grade(e: &GeoExpr, ctx: &GradeCtx) -> GradeSet   { analyse(e, ctx).grade }
pub(crate) fn is_versor(r: &GeoExpr, ctx: &GradeCtx) -> bool { analyse(r, ctx).versor }
```

One behavioural nuance, stated so AC3's fuzz can pin it: today `grade` does
**not** descend into `GradeLift`'s child; `analyse` does (uniform `children`),
and `rule` ignores the result. Same value, one extra O(subtree) of work on a form
that is rare. Accepted for uniformity — `children` is then the single arity
source §2.1's `unreachable!` relies on.

`kids` is a two-element-max `Vec` per node. Fine at 60 nodes; if the throughput
test (§4.5) shows the allocation, `[Option<Analysis>; 2]` or `SmallVec` is a
local swap that changes nothing above.

### 2.3 The strict walk — `check`

`typecheck` differs from `grade` at exactly three points: an out-of-range leaf is
an error rather than ⊤; a bad projection is an error rather than ∅; and an empty
grade set at *any* node is `Incoherent`. Everything else is `rule`:

```rust
/// `typecheck`'s pass: validate pre-order, descend left-to-right, compose
/// post-order, reject ∅ post-order. Each node is visited exactly once.
fn check(e: &GeoExpr, ctx: &GradeCtx) -> Result<Analysis, GradeError> {
    match e {                                              // pre-order, before descent
        GeoExpr::Basis(i) if *i >= 16 => return Err(GradeError::BadBlade(*i)),
        GeoExpr::GradeLift(k, _) | GeoExpr::GradeProject(k, _) if *k > 4 => return Err(GradeError::BadGrade(*k)),
        _ => {}
    }
    let mut kids = Vec::with_capacity(2);
    for c in children(e) {
        kids.push(check(c, ctx)?);                          // left-to-right; first error wins
    }
    let a = rule(e, &kids, ctx);
    if a.grade.is_empty() {
        Err(GradeError::Incoherent(e.clone()))              // post-order, innermost first
    } else {
        Ok(a)
    }
}

pub fn typecheck(e: &GeoExpr, ctx: &GradeCtx) -> Result<GradeSet, GradeError> {
    check(e, ctx).map(|a| a.grade)
}
```

**Error precedence is preserved exactly** (SPEC-0019 §2.3's finding): own
`BadBlade`/`BadGrade` before descent; children left to right with `?`; own
`Incoherent` after both children — so the *innermost* incoherent subtree is the
one reported and cloned, as today.

### 2.4 What is deleted

The recursive `grade` (11 arms), the recursive `is_versor` (5 arms), and
`typecheck`'s per-node `grade(e)` call. Net: three walks become one `rule` plus
two ten-line drivers.

## 3. Non-goals

- **No semantics change.** `rule` is the existing arms transcribed; the versor
  predicate stays exactly as conservative. AC3's differential fuzz is the proof,
  not this sentence.
- **Not the depth contract.** Both drivers stay recursive. R-0019 is shelved.
- **Not the refiner's neighbor scaling** (R-0020 §2 — the real cost at large
  caps).
- **No `GradeError` change.** `Incoherent` keeps its payload (SPEC-0019 §2.7).

## 4. Tests (TDD — written first, red)

1. **T-differential (AC3)** — `grade` and `typecheck` fuzzed against **verbatim
   transcriptions** of the pre-change functions (kept in the test file as
   `old_grade`/`old_is_versor`/`old_typecheck`) over random `GeoExpr`s covering
   all 11 variants, out-of-range `Basis`/grades, unbound and declared `Var`s, and
   both `Sandwich` branches. Identical `GradeSet`; identical `Result` including
   **which** error and **which** subtree `Incoherent` carries.
2. **T-is-versor-direct (AC3)** — `is_versor` is `pub(crate)`, so it is fuzzed
   against `old_is_versor` in an in-`src` `#[cfg(test)]` module. Through
   `grade(Sandwich(r, x))` alone a wrong answer is masked wherever
   `grade(r) = {0}`.
3. **T-visit-count (AC1, AC2)** — the mechanism, not a clock. `Analysis` gains
   nothing for this; instead the in-`src` test wraps `analyse` and `check` in a
   counting shim over `children` **(open question §5.1 — how to count without
   touching the product type)**, and asserts `visits == node_count` on shape (C)
   at k ∈ {10, 20, 40} and on 1,000 random trees. For `check` the same holds on
   every tree that typechecks.
4. **T-feasibility** — shape (C) at **k = 64** completes under both `grade` and
   `typecheck`. At 2^64 visits the old code needs centuries; at O(n) this is
   microseconds. Not a timing bound — a hang detector. Runs in a subprocess with
   a 60 s ceiling so a regression fails the build instead of stalling it.
5. **T-remeasure (AC4)** — R-0020 §1's table (k = 14/18/20) re-run and printed
   **unconditionally**, before/after, with the assertion being only that the
   numbers were recorded. Expected: ~200 ms → ~10 µs at k = 22.
6. **T-precedence** — the three orderings §2.3 preserves, as named cases:
   `Sandwich(Basis(20), GradeProject(9, x))` → `BadBlade(20)` (pre-order, left
   first); `GradeProject(9, Basis(20))` → `BadGrade(9)` (own pre-order before
   child); `GradeProject(2, GradeProject(3, Param))` → `Incoherent` carrying the
   **inner** projection.
7. **T-throughput** — `typecheck` on production shapes (≤ 60 nodes, the shapes
   the R-0011 proposer emits), old vs new, release, reported unconditionally.
   Expected neutral-to-better; a regression here is a finding, not a failure.

## 5. Open questions for the three-lens

1. **How should T-visit-count count?** Options: (a) a `visits: usize` field on
   `Analysis` — honest and cheap, but a field only a test reads; (b) an in-`src`
   test that re-derives the walk with a counter and asserts it agrees with
   `analyse` on every tree — proves the *shape* of the walk, not the real
   function's visits; (c) a `#[cfg(test)]` counter threaded through `GradeCtx` —
   test-only branches in a lib struct. *Recommendation: (a), because the AC asks
   for the real function's visit count and (a) is the only option that measures
   it.* Architect, is a test-only-read field an acceptable cost here?
2. **`analyse` now descends into `GradeLift`'s child; today's `grade` does not
   (§2.2).** Same result, uniform arity. Is the uniformity worth the extra
   O(subtree) on a rare form, or should `children` special-case it and `rule`
   take `kids: &[Analysis]` of length 0 for `GradeLift`?
3. **Is a 60 s subprocess ceiling on T-feasibility a clock in the sense
   *Structural Frugality over Wall-Clock* forbids?** It is not a bound anyone
   tunes — the gap is µs versus centuries — but it is wall-clock. Hater: is there
   a shape that legitimately takes long enough to make this flaky?
4. **Should this be built at all, given R-0020 §3?** The honest case is hygiene
   on a `pub` function: small, local, correct. Not speed, not the target class.
   Nice-guy: is there an upside I have not seen? Hater: is sixty lines of change
   to a soundness-critical file worth a rare tail?
