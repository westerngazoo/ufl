# R-0019 — Extend the depth contract to the geometric surface

- **Status:** **Draft** — acceptance criteria await Gustavo's sign-off (CLAUDE.md §4 step 1).
- **Milestone:** M5 (geometric neuroevolution).
- **Tracks:** [#83](https://github.com/westerngazoo/ufl/issues/83).
- **Precedent:** [R-0017](0017-depth-contract.md) / [SPEC-0017](../specs/0017-depth-contract.md)
  — the same class, already discharged on the `Sexpr`/`Eml` code↔data surface.
- **Convention it applies:** `docs/conventions.md` — *Explicit-Stack Tree Walk*,
  *Bounded-Stack Regression Arena*.

## 1. Context — measured, not inferred

`GeoExpr` is a `Box`-recursive enum walked by **twelve** recursive functions
across two crates. R-0017 fixed seven such walks on `Sexpr`/`Eml`; none of that
work reached here, and unlike those two types `GeoExpr` has **no iterative
`Drop` at all** — it relies on compiler-generated drop glue.

Measured abort depths (a `Reverse(Reverse(…Param))` spine, debug build, default
8 MB main-thread stack, binary-searched to the exact node):

| walk | crate | survives | aborts at |
|------|-------|----------|-----------|
| **`eval`** | `ufl-geo` | 214 | **215** |
| `typecheck` | `ufl-geo` | 2,798 | 2,799 |
| `params` / `params_mut` (`collect`) | `ufl-geo` | 4,112 | 4,113 |
| `Clone` (derived) | `ufl-geo` | 4,112 | 4,113 |
| `grade` | `ufl-geo` | 4,699 | 4,700 |
| `PartialEq` (derived) | `ufl-geo` | 4,700 | 4,701 |
| `Debug` (derived) | `ufl-geo` | 5,059 | 5,060 |
| `Drop` (glue) | `ufl-geo` | 18,800 | 18,801 |

`render` (`node`/`factor`), and `ufl-evolve`'s `node_count`, `nth_subtree`, and
`replace_nth` are recursive by inspection; their thresholds are not yet measured
(see §5 Q3).

`eval` is an order of magnitude worse than every other walk because each frame
carries an `Mv` (16 × `f64`) and the binary nodes hold **two** evaluated children
simultaneously.

## 2. The real problem: one knob serving two purposes

There is **no abort in production today** — `GeoProposer::pinned` caps genomes at
`max_nodes: 60`, and since `depth ≤ node count`, depth never exceeds 60 against
`eval`'s limit of 215. The hazard is latent, with 3.5× headroom.

`max_depth: 4` bounds **generation and mutation** only. Crossover splices a
subtree from one parent into another and can deepen the result past 4, so
**post-crossover depth is bounded only indirectly, by `max_nodes`**. That makes a
single knob serve two unrelated purposes:

1. **anti-bloat / eval cost** — its intended job, and its doc comment says so;
2. **depth safety** — an accident of `depth ≤ nodes`, holding only because
   `60 < 215`.

The consequences are the reason to act:

- **`max_nodes` cannot be raised past ~215** without `eval` aborting, whatever
  the search wants. A stack limit is silently capping the hypothesis space.
- The bound is **loose in the wrong direction**: a 60-node *bushy* tree has depth
  ~6, so the cap restricts breadth far more than depth safety requires.
- The cap is itself **enforced by a recursive walk** (`node_count`), so the
  guard shares the failure mode of the thing it guards.

## 3. What this requirement is *not*

- **Not a tuning change.** `max_nodes` stays at 60 here. Decoupling first keeps
  the diff reviewable and keeps any later re-tune a clean experiment.
- **Not a claim about Gate-1.** Whether a larger `max_nodes` improves the
  current 6/16 is a **separate experiment** in the R-0011 lane. This requirement
  makes that experiment *possible*; it predicts nothing about its outcome.
  (I implied otherwise in #83 and have corrected it there.)
- **Not a new geometric form**, and no public API change beyond what iterative
  rewrites require (the trait signatures are unchanged).

## 4. Proposed acceptance criteria — **for Gustavo's sign-off**

- **AC1 (the geometric walks are iterative).** `eval`, `grade`, `typecheck`,
  `params`/`params_mut`, and `render` use explicit heap work-stacks. No depth
  cap and no magic constant is introduced.
- **AC2 (the class closure).** `GeoExpr`'s derived `Clone` and `PartialEq` are
  replaced by hand-written iterative impls, and `GeoExpr` gains an **iterative
  `Drop`** (it has none today). Same observable behaviour; no signature change.
- **AC3 (the search-side walks).** `ufl-evolve`'s `node_count`, `nth_subtree`,
  and `replace_nth` are iterative — the anti-bloat guard must not share the
  failure mode of what it guards.
- **AC4 (semantic equivalence, proven).** Every rewritten walk is differentially
  fuzzed against a transcription of the pre-change implementation over randomly
  generated `GeoExpr`s: identical results **and identical error precedence**.
  `render` compares **byte-identically**.
- **AC5 (depth, in a bounded-stack arena).** Every walk in AC1–AC3 completes at
  depth **10⁵** inside a subprocess arena pinned to the `dev` profile, per
  `docs/conventions.md`. The arena must be shown to **fail** when a recursion is
  reintroduced.
- **AC6 (the decoupling is real).** A test asserts `max_nodes` can be set to a
  value that would previously have aborted `eval` (e.g. 5,000) and the lane still
  runs to completion — the property that makes the R-0011 experiment possible.
- **AC7 (no library-code abort).** `ufl-geo` and `ufl-evolve` adopt
  `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used,
  clippy::panic))]`, closing two of the seven crates in
  [#82](https://github.com/westerngazoo/ufl/issues/82).

## 5. Open questions for the three-lens (and for Gustavo first)

1. **Is `Debug` in or out?** R-0017 deferred it ([#81](https://github.com/westerngazoo/ufl/issues/81))
   after I measured that it is never invoked from library code. The same is
   likely true here — but `render` exists precisely to print these trees, so the
   caller-facing risk is higher. *My recommendation: measure reachability first,
   then decide; do not assume R-0017's answer transfers.*
2. **Is depth 10⁵ the right target** for a surface whose production trees are
   ≤ 60 nodes? R-0017's answer was "no cap, so pick a depth far past anything
   real". *My recommendation: keep 10⁵ for consistency — it costs 0.2 s.*
3. **Should the unmeasured walks be measured before the spec**, per
   *Measured Before Specified*? *My recommendation: yes — `render` and the three
   `ufl-evolve` walks, so the spec cites numbers rather than inference. This is
   the step I skipped when I first wrote #83, and it produced a wrong claim.*
4. **Is `Mv`-per-frame worth attacking separately?** `eval`'s 215 is driven by
   frame size, not just recursion. An iterative `eval` moves those `Mv`s to the
   heap — worth confirming that is a win and not a throughput regression on the
   hot path the search calls on every candidate.
