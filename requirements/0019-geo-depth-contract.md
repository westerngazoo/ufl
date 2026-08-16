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

Measured abort depths — **every** walk, none inferred. A `Reverse(Reverse(…))`
spine, debug build, default 8 MB main-thread stack, binary-searched to the exact
node:

| walk | crate | survives | aborts at |
|------|-------|----------|-----------|
| **`eval`** | `ufl-geo` | 214 | **215** |
| `render` (`node`/`factor`) | `ufl-geo` | 2,479 | 2,480 |
| `typecheck` | `ufl-geo` | 2,798 | 2,799 |
| `replace_nth` | `ufl-evolve` | 3,655 | 3,656 |
| `params` / `params_mut` (`collect`) | `ufl-geo` | 4,112 | 4,113 |
| `Clone` (derived) | `ufl-geo` | 4,112 | 4,113 |
| `grade` | `ufl-geo` | 4,699 | 4,700 |
| `PartialEq` (derived) | `ufl-geo` | 4,700 | 4,701 |
| `Debug` (derived) | `ufl-geo` | 5,059 | 5,060 |
| `node_count` | `ufl-evolve` | 16,457 | 16,458 |
| `Drop` (glue) | `ufl-geo` | 18,800 | 18,801 |
| `nth_subtree` | `ufl-evolve` | 26,331 | 26,332 |

`eval` is an order of magnitude worse than every other walk because each frame
carries an `Mv` (16 × `f64`) and the binary nodes hold **two** evaluated children
simultaneously.

### 1.1 `GradeError` carries a tree — the case R-0017 never met

`GradeError::Incoherent(GeoExpr)` (`grade.rs:49`) embeds a whole `GeoExpr`, and
`grade.rs:178` constructs it as `Err(GradeError::Incoherent(e.clone()))`. Three
consequences that have no analogue on the `Sexpr`/`Eml` surface, where **no**
error type held a tree:

- **Producing the error is itself a recursive walk** — the derived `Clone`, on
  the failure path of a `pub fn`. Measured: an incoherent deep tree aborts
  `typecheck` at **2,799**, i.e. `typecheck`'s own recursion binds first and the
  clone is not the limiting factor *today* — but it is a second abort site on the
  same path.
- `GradeError` **derives `Clone`, `PartialEq`, and `Debug`**, so all three are
  recursive walks over caller-held data, on a `pub` type.
- `GradeScreen` (`lane.rs`) calls `typecheck(g, &ctx).is_ok()` on **every
  candidate**, so this path is the search's hot loop, not a corner.

Its `#[error(…)]` string is a constant with no `{0}`, so `Display` does **not**
walk the tree. Only `Debug` does.

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
- **AC2 (the class closure).** `GeoExpr`'s derived `Clone`, `PartialEq`, **and
  `Debug`** are replaced by hand-written iterative impls, and `GeoExpr` gains an
  **iterative `Drop`** (it has none today). Same observable behaviour, no
  signature change, and `Debug` output stays **byte-identical to the derive**
  (it appears in error payloads). `Debug` is in scope here — unlike R-0017 —
  because §1.1's `GradeError` embeds a tree (see §5 Q1).
- **AC2b (`GradeError` is depth-safe).** `GradeError`'s derived `Clone`,
  `PartialEq`, and `Debug` no longer recurse over an embedded `GeoExpr`,
  whichever resolution §5 Q4 takes.
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

## 5. Open questions

**Q3 is answered:** every walk is now measured (§1) — `render` and the three
`ufl-evolve` walks included. The remaining questions are for Gustavo, then the
three-lens.

1. **Is `Debug` in or out?** R-0017 deferred it
   ([#81](https://github.com/westerngazoo/ufl/issues/81)) once I measured that it
   is never invoked from library code. **That answer does not transfer.** `{:?}`
   is likewise absent from `ufl-geo`/`ufl-evolve` library code — but §1.1's
   `GradeError::Incoherent` embeds a tree, so `Debug` on the *error* is a
   recursive walk over caller data reachable from any `unwrap`, log, or
   `assert_eq!` on a `Result`. *My recommendation: `Debug` is **in** for this
   requirement, and #81 should be reconsidered in the same light.*
2. **Is depth 10⁵ the right target** for a surface whose production trees are
   ≤ 60 nodes? R-0017's answer was "no cap, so pick a depth far past anything
   real". *My recommendation: keep 10⁵ for consistency — it costs ~0.2 s.*
3. **Is `Mv`-per-frame worth attacking separately?** `eval`'s 215 is driven by
   frame size, not recursion count alone. An iterative `eval` moves those `Mv`s
   to the heap — worth confirming that is a win and not a throughput regression
   on the hot path the search calls for every candidate. *My recommendation: make
   AC4's differential fuzz double as a throughput comparison, so the answer is
   measured rather than assumed.*
4. **Does `GradeError` keep its tree payload at all?** Carrying the offending
   subtree is genuinely useful for diagnostics, but it is what puts a recursive
   walk on a `pub` error type. Alternatives: keep it (and make the derives
   iterative), or replace it with a `render`ed `String` — cheaper and already
   human-readable, at the cost of losing structured access. *No recommendation —
   this is a design trade the three-lens should argue.*

## 6. A note on how §1 was produced

The `ufl-evolve` thresholds were measured **twice**. The first run reported all
three walks surviving 200,000 — because the test-name filter matched nothing and
`libtest` exits 0 when it runs zero tests, so every probe "passed" without
executing. The corrected harness asserts `1 passed` before believing a result.

That is precisely the false-pass `docs/conventions.md` — *Bounded-Stack
Regression Arena* — was written to prevent, walked into in an ad-hoc harness
three days after writing the convention. Recorded here because the same trap will
be waiting for whoever builds this requirement's arena.
