# R-0020 — `grade` and `typecheck` visit each node once

- **Status:** **Draft** — acceptance criteria await Gustavo's sign-off (CLAUDE.md §4 step 1).
- **Milestone:** M5 (geometric neuroevolution).
- **Severed from:** [R-0019](0019-geo-depth-contract.md) AC8 / §7. R-0019 is
  recommended for shelving; this is the one piece of it that stands on its own.
- **Tracks:** [#83](https://github.com/westerngazoo/ufl/issues/83) (in part).

## 1. The defect

`grade`'s `Sandwich` arm (`crates/ufl-geo/src/grade.rs:120-131`) calls
`is_versor(r)`, which for `Exp(b)` computes `grade(b)`; on the non-versor branch
it then computes `grade(r) = grade(Exp(b))`, which recomputes `grade(b)`. Two
walks of the same subtree per level, so on nested rotor sandwiches `grade` is
**2^depth**. `typecheck` (`grade.rs:151-182`) compounds it by calling `grade(e)`
afresh at *every* node instead of using its children's results — **O(n²)** over
an already-exponential function.

Measured in release on `bₖ₊₁ = Sandwich(Exp(bₖ), Basis(1))` (R-0019 §1.2):

| depth | nodes | `grade` | `typecheck` | `eval` |
|---|---|---|---|---|
| 14 | 43 | 816 µs | 2.44 ms | 39 µs |
| 18 | **55** | 13.0 ms | **71.2 ms** | 62 µs |
| 20 | 61 | 67.6 ms | 179.5 ms | 75 µs |

A 55-node genome fits inside today's `max_nodes: 60`, and on it the screen costs
~1,150× the evaluation it exists to be cheaper than.

## 2. What it costs today — measured, and smaller than I first claimed

R-0019 §7 originally said this defect "very likely explains most" of the cap
sweep's 7.6×/12.8× cost. **That was wrong.** A timed run of the real lane
(`GradeScreen` and `GeoFitness` wrapped with timers, pinned Gate-1 config,
release) shows where wall-clock actually goes:

| cap | seed | total | screen (`typecheck`) | fitness (`eval`) | worst `typecheck` |
|---|---|---|---|---|---|
| 60 | 0 | 4.6 s | 0.9 s (20%) | 2.9 s (63%) | 117 µs |
| 60 | 6 | 2.8 s | 0.4 s (15%) | 1.8 s (63%) | 172 µs |
| 150 | 0 | 19.0 s | 3.6 s (19%) | 13.4 s (71%) | 8.9 ms |
| 150 | 6 | 149.4 s | 39.1 s (26%) | 102.7 s (69%) | 3.7 ms |

Three things this settles:

- **The screen is 15–26% of wall-clock at every cap; `eval` is 63–71%.** Fixing
  this defect cannot make the lane more than ~25% faster at cap 150, and only a
  few percent at cap 60.
- **The 145 s outlier is a call-count story, not a per-call one.** Seed 6 at cap
  150 made **2.6M** screen calls versus 500K for seed 0 — bigger trees carry more
  `Param` slots, and `GeoParamRefiner` emits 22 × slots neighbors per elite per
  step. That scaling is the real cost driver at larger caps, and it is **not**
  this requirement.
- **The exponential regime is nonetheless reached.** Worst-case `typecheck` grows
  **50×** from cap 60 to cap 150 (117 µs → 8.9 ms). It averages out today because
  the GA rarely produces the triggering shape.

## 3. Why fix it anyway

Not for speed — §2 rules that out as the headline. For two reasons that survive
§2:

1. **It is exponential on exactly the solution class the lane exists to find.**
   The triggering shape is a nested versor sandwich — `Sandwich(Exp(…), …)` inside
   `Sandwich(Exp(…), …)` — which is what a **motor chain** is. A 6-DoF rigid-body
   map is 61 nodes of precisely this shape (nice-guy, SPEC-0019 review). The GA
   does not produce it at random, so the defect is latent; but if the search ever
   converges *toward* the answers R-0011 wants, the screen becomes the bottleneck
   at the worst possible moment. An algorithm that is 2^depth on the target class
   is a landmine, whatever its average cost.
2. **The fix is small and local.** One post-order pass that returns
   `(GradeSet, is_versor)` per node, with `grade`/`is_versor` as thin wrappers,
   and a `typecheck` that threads its children's `GradeSet`s up instead of
   recomputing them. No public signature changes, no depth work, no arena. On the
   order of thirty lines in one file.

## 4. What this is *not*

- **Not the depth contract.** R-0019 is shelved; this needs no explicit-stack
  rewrite and stays recursive.
- **Not the refiner's neighbor scaling.** §2 shows that is the real cost at large
  caps. It is a search-efficiency question for the R-0011 lane, tracked
  separately.
- **Not a semantics change.** The versor predicate stays exactly as conservative
  as today; only how many times a subtree is *visited* changes, never the answer.

## 5. Proposed acceptance criteria — **for Gustavo's sign-off**

- **AC1 (single visit, asserted by mechanism).** `grade` visits each node at most
  a small constant *c* times, proven by a **node-visit counter**, not a timing
  bound (`docs/conventions.md` — *Structural Frugality over Wall-Clock*). The
  test fails loudly on any future rule that re-walks a child, and is stable on
  shared CI.
- **AC2 (`typecheck` is O(n)).** `typecheck` no longer calls `grade` per node; it
  computes grades once, post-order, and reports `Incoherent` from the same pass.
  Same counter, same bound.
- **AC3 (semantic equivalence, proven).** `grade`, `is_versor`, and `typecheck`
  are differentially fuzzed against verbatim transcriptions of the pre-change
  functions over random `GeoExpr`s: identical `GradeSet`s, identical `bool`,
  identical `Result` **and error precedence** (`BadBlade`/`BadGrade` before
  descent, `Incoherent` post-order, children left-to-right). `is_versor` is
  `pub(crate)`, so its fuzz lives in an in-`src` `#[cfg(test)]` module — an
  integration test cannot observe it.
- **AC4 (the table re-measured).** §1's 55-node nest: `typecheck` reported
  **unconditionally** before and after. Expected to fall from 71 ms to well under
  1 ms; the test asserts the number was recorded, not that it hit a target.
- **AC5 (no public surface change).** `grade`, `typecheck`, `GradeError`, and
  `GradeSet` keep their signatures. `GradeError::Incoherent` keeps its payload
  (SPEC-0019 §2.7's reasoning stands).

## 6. Open question

Whether *c* = 1 is achievable, or whether `Sandwich` legitimately needs its rotor
visited twice (once for the grade, once for the structural versor test). A single
pass returning `(GradeSet, bool)` should make it 1 — the versor test for
`GeoProduct(a, b)` is `versor(a) && versor(b)`, which the tuple carries up without
a re-walk. The spec should state the bound it actually achieves and the counter
should assert that bound, not a hoped-for one.
