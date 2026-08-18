# R-0019 — Extend the depth contract to the geometric surface

- **Status:** **Draft — recommended for SHELVING.** The cap probe (§2.1) removed the stated justification; §7 recommends severing AC8 into its own requirement.
- **Milestone:** M5 (geometric neuroevolution).
- **Tracks:** [#83](https://github.com/westerngazoo/ufl/issues/83).
- **Precedent:** [R-0017](0017-depth-contract.md) / [SPEC-0017](../specs/0017-depth-contract.md)
  — the same class, already discharged on the `Sexpr`/`Eml` surface.
- **Convention it applies:** `docs/conventions.md` — *Explicit-Stack Tree Walk*,
  *Bounded-Stack Regression Arena*.

## 1. Context — measured, and re-measured

`GeoExpr` is a `Box`-recursive enum walked by **fourteen** recursive functions
across two crates. R-0017 fixed seven such walks on `Sexpr`/`Eml`; none of that
work reached here, and unlike those two types `GeoExpr` has **no iterative
`Drop` at all` — it relies on compiler-generated drop glue.

### 1.1 Abort depths — and the methodology correction

The first draft of this table labelled its numbers "default 8 MB main-thread
stack". **That was wrong.** They were measured through `cargo test`, and libtest
runs each `#[test]` on a *spawned* thread whose default stack is **2 MiB**. The
architect caught it; I verified it directly with a standalone binary:

| `eval`, debug | aborts at |
|---|---|
| libtest spawned thread (2 MiB) | **215** |
| `fn main()` (8 MB) | survives 800, aborts by 860 |

A **4× difference driven by the caller, not the code.** Two consequences:

- The table below is valid **only** for a 2 MiB thread. It is a property of the
  calling context, not a property of `GeoExpr`.
- **The portable invariant is bytes-per-frame, not depth.** `eval` costs
  ≈ 9,754 B/frame in debug and ≈ 1,560 B/frame in release (architect, measured).
  A depth survives no change of machine, profile, or thread size; a frame cost
  does. Future measurements here should report both.

| walk | crate | aborts at (2 MiB) |
|------|-------|-------------------|
| **`eval`** | `ufl-geo` | **215** |
| `render` (`node`/`factor`) | `ufl-geo` | 2,480 |
| `typecheck` | `ufl-geo` | 2,799 |
| `replace_nth` | `ufl-evolve` | 3,656 |
| `collect` (`params`/`params_mut`) | `ufl-geo` | 4,113 |
| `Clone` (derived) | `ufl-geo` | 4,113 |
| `grade` | `ufl-geo` | 4,700 |
| `is_versor` | `ufl-geo` | 4,701 † |
| `PartialEq` (derived) | `ufl-geo` | 4,701 |
| `Debug` (derived) | `ufl-geo` | 5,060 |
| `node_count` | `ufl-evolve` | 16,458 |
| `Drop` (glue) | `ufl-geo` | 18,801 |
| `nth_subtree` | `ufl-evolve` | 26,332 |
| `random_expr` (generator) | `ufl-evolve` | not measured ‡ |

† `is_versor` is mutually recursive with `grade`. It does **not** worsen the
bound — its recursion is bounded by the *rotor* subtree, not the spine.

‡ `random_expr` (`memetic.rs:182`) is a recursive **generator** bounded by the
`pub` field `max_depth`, and it is the fallback the anti-bloat cap invokes
(`memetic.rs:341`). Safe at `max_depth: 4`; in scope because AC6 raises the other
knob while leaving this one recursive (architect finding 9).

**A mechanism claim in the first draft was also wrong.** It said `eval`'s 215
comes from binary nodes holding two `Mv`s at once. The architect measured a
`Reverse` spine — *zero* binary nodes — aborting at exactly 215 too. The real
cause is ~9.7 KB of opt-level-0 frame across the 11-arm match; the `Mv` payload
(`size_of::<Mv>() = 128`) is ~1.3% of it.

### 1.2 `grade` is exponential — a complexity bug, not a depth bug

`grade`'s `Sandwich` arm (`grade.rs:120-131`) calls `is_versor(r)`, which for
`Exp(b)` computes `grade(b)`; on the non-versor branch it then computes
`grade(r) = grade(Exp(b))`, recomputing `grade(b)`. **Two walks of the same
subtree per level ⇒ 2^depth.** Measured in **release** on
`bₖ₊₁ = Sandwich(Exp(bₖ), Basis(1))`:

| depth | nodes | `grade` | `typecheck` | `eval` |
|---|---|---|---|---|
| 14 | 43 | 816 µs | 2.44 ms | 39 µs |
| 18 | **55** | 13.0 ms | **71.2 ms** | 62 µs |
| 20 | 61 | 67.6 ms | 179.5 ms | 75 µs |

At 55 nodes — **inside today's `max_nodes: 60`** — `GradeScreen::admissible`
costs 71 ms while `eval` on the same tree costs 62 µs. A screen whose purpose is
to be cheaper than evaluation is ~1,150× more expensive than what it replaces.

`typecheck` compounds it: it calls `grade(e)` at *every* node (`grade.rs:176`),
so it is **O(n²)** over an already-exponential `grade` (architect finding 11).

**This is not a depth problem and an iterative rewrite does not fix it.** An
explicit-stack machine that still re-walks is still 2^depth. It is recorded here
because it is the constraint that actually binds, and because AC6 would activate
it (§2).

### 1.3 What the real search actually produces

The decisive question — is any of this live? Measured with the **actual pinned
Gate-1 proposer**, 60 generations × population 400, release:

| | measured |
|---|---|
| max depth reached | **32** |
| max nodes | 60 (the cap) |
| worst `typecheck` over 24,000 genomes | **22.9 µs** |

Two findings, pulling in opposite directions:

- **Crossover really does deepen trees far past the generation cap** — depth 32
  against `max_depth: 4`, an 8× overrun. §2's structural claim is confirmed.
- **Neither hazard is live.** Real depth 32 sits ~7× below `eval`'s 2 MiB
  ceiling (215) and ~27× below its main-thread ceiling. The exponential regime
  needs a specifically rotor-nested `Sandwich` shape that the GA does not
  produce: the worst real genome costs 23 µs, not 71 ms.

The first draft's "3.5× headroom" figure was wrong — it assumed depth = nodes =
60. Real headroom is ~7× (test thread) or ~27× (main thread).

## 2. The case for building this — stated honestly

**Nothing is broken today.** §1.3 measured it. This requirement is not a bug fix,
and framing it as one would be motivated reasoning.

What is true is narrower and conditional:

`max_depth: 4` bounds **generation and mutation** only. Crossover splices a
subtree from one parent into another and deepens the result — measured to 32,
8× the cap. So **post-crossover depth is bounded only indirectly, by
`max_nodes`**, and one knob serves two unrelated purposes:

1. **anti-bloat / eval cost** — its intended job;
2. **depth safety** — an accident of `depth ≤ nodes`.

The value is therefore **conditional on wanting to raise `max_nodes`**. If we
never raise it, this requirement buys latent safety and nothing else. If we do
raise it — and AC6's 5,000 is the value the R-0011 experiment wants — then at the
measured depth/nodes ratio of 0.53 a 5,000-node genome implies depth **≈ 2,666**,
which:

- exceeds `eval`'s ceiling on **both** thread sizes (215 / 860), and
- lands squarely in §1.2's exponential regime, where `grade` alone would not
  terminate.

So the honest statement is: **the depth contract is a prerequisite for the
`max_nodes` experiment, and it is not sufficient — §1.2 must be fixed too.** Both
are latent today.

### 2.1 The conditional was tested — and it does not hold

§2's case rests on *wanting* to raise `max_nodes`. That is testable without
building any of R-0019: `eval` survives to depth 215 on a test thread and real
genomes sit at depth 32, so the cap can go to ~150 today, unchanged.

Pre-registered sweep (`crates/ufl-evolve/tests/r_0019_cap_probe.rs`), release,
`max_nodes` the only varying knob:

| cap | wins/16 | per-seed | wall-clock |
|-----|---------|----------|------------|
| **60 (control)** | **6/16** | `...#...###..#.#.` | 34 s |
| 100 | 4/16 | `...#..#.#..#....` | 258 s (7.6×) |
| 150 | 4/16 | `.#......#...#..#` | 436 s (12.8×) |

The control reproduces the pilot's 6/16 exactly.

**What this supports.** No evidence that the cap is binding on search quality.
Raising it is **expensive**: 7.6× and 12.8× wall-clock, one seed alone taking
145 s.

**What it does not support.** It is *not* evidence that raising the cap hurts.
6 vs 4 is **1.03 SD** on `Binomial(16, 6/16)`, and `P(X ≤ 4 | p = 6/16) = 0.223`
— 4/16 is an ordinary draw from the control. Resolving 0.375 vs 0.50 at 80%
power would take ~247 seeds per arm. The win-rate comparison *is* unconfounded by
the cost, since `GENS` is fixed at 400 regardless of speed.

**Consequence for this requirement.** §2's justification was explicitly
conditional, and the condition now has no empirical support: we have no measured
reason to want a larger `max_nodes`, and a measured reason not to. R-0019 as
scoped is **not justified** — see §7.

## 3. What this requirement is *not*

- **Not a bug fix.** §1.3 shows no live abort and no live blowup.
- **Not a tuning change.** `max_nodes` stays 60. Decoupling first keeps the diff
  reviewable and any later re-tune a clean experiment.
- **Not a claim about Gate-1.** Whether a larger `max_nodes` improves the current
  6/16 is a separate R-0011 experiment. This makes it *possible*; it predicts
  nothing. (I implied otherwise in #83 and corrected it there.)
- **Not a fix for §1.2 by itself** — see AC8.

## 4. Acceptance criteria — **need re-confirmation after the §2 rewrite**

- **AC1 (the geometric walks are iterative).** `eval`, `grade`, `is_versor`,
  `typecheck`, `params`/`params_mut`, and `render` use explicit heap work-stacks.
  No depth cap and no magic constant.
- **AC2 (the class closure).** `GeoExpr`'s derived `Clone`, `PartialEq`, and
  `Debug` become hand-written iterative impls, and `GeoExpr` gains an **iterative
  `Drop`**. `Debug` is in scope — unlike R-0017 — because §5's containment chain
  puts it on a reachable path.
- **AC2b (`GradeError` and its containment chain are depth-safe).** The chain is
  `GeoExpr → GradeError → GeoLaneError → RunError<E>`, four types across three
  crates, all deriving `Debug`/`Clone`/`PartialEq`.
- **AC3 (the search-side walks).** `node_count`, `nth_subtree`, `replace_nth`
  iterative; `random_expr`'s recursion explicitly addressed or explicitly
  deferred with a stated bound.
- **AC4 (semantic equivalence, proven).** Every rewritten walk differentially
  fuzzed against a verbatim transcription of the pre-change implementation:
  identical values **and** identical error precedence. `render` and `Debug`
  compare byte-identically. `is_versor` is `pub(crate)`, so its fuzz lives in an
  in-`src` `#[cfg(test)]` module — an integration test cannot observe it.
- **AC5 (depth, in a bounded-stack arena).** Every walk completes at depth 10⁵ in
  a subprocess arena pinned to the `dev` profile **and to an explicit stack
  size** — without pinning, sensitivity varies 4× with the calling thread (§1.1).
  The arena must be shown to fail when a recursion is reintroduced.
- **AC6 (the decoupling is real).** The lane runs to completion with
  `max_nodes = 5,000`. Asserts *completion*, not a fitness change. **Depends on
  AC8** — §1.2 makes this non-terminating otherwise.
- **AC7 (no library-code abort).** `ufl-geo` and `ufl-evolve` adopt the
  `not(test)` deny of `unwrap`/`expect`/`panic`. Measured cost: three
  `write!(…).unwrap()` in `ufl-geo`, zero in `ufl-evolve`.
- **AC8 (new — single-visit `grade`).** `grade` visits each node a bounded number
  of times, asserted by a **node-visit counter**, not a timing bound
  (`docs/conventions.md` — *Structural Frugality over Wall-Clock*). This closes
  §1.2 and is what makes AC6 reachable.

## 5. The containment chain — why `Debug` is in scope

R-0017 deferred `Debug` ([#81](https://github.com/westerngazoo/ufl/issues/81))
because it is never invoked from library code. `{:?}` is likewise absent from
`ufl-geo`/`ufl-evolve` library code — **but that is not the whole audit surface.**

```
GeoExpr → GradeError::Incoherent(GeoExpr)        grade.rs:49
        → GeoLaneError::Grade(#[from] GradeError) lane.rs:23
        → RunError<E>::Lane(E)                    ufl-search/src/lib.rs:56
```

Four types, three crates, each deriving `Debug`/`Clone`/`PartialEq`. And
`r_0011m_gate1.rs:92`'s `.expect(…)` formats that chain via `Debug` — a derived
recursive walk over a `GeoExpr`, invoked from a committed test, four types away
from the type anyone would think to audit.

This generalizes R-0017's lesson. That one was *a derived trait on the recursive
type is a walk no grep for `fn` finds*. This one is stronger: **the audit set is
the transitive closure of types containing `T` by value, plus every site that
materializes `T` into one of them** — here `grade.rs:178`'s `e.clone()`, which
puts a full tree walk on a `pub fn`'s *failure* path. Worth promoting to
`docs/conventions.md` once this lands.

`Display` is safe: `grade.rs:48`'s `#[error]` string has no `{0}`.

## 6. How the numbers were produced

Two of §1's measurement passes were wrong before they were right, and both
failures are more useful than the numbers.

**The false pass.** The `ufl-evolve` walks first reported surviving 200,000 —
because the test-name filter matched nothing and `libtest` exits 0 having run
zero tests, so every probe "passed" without executing. The corrected harness
asserts `1 passed` before believing a result. This is exactly the false-pass
`docs/conventions.md` — *Bounded-Stack Regression Arena* — exists to prevent,
walked into in an ad-hoc harness three days after writing the convention.

**The mislabelled stack.** The whole table was attributed to an 8 MB main-thread
stack when it was measured on libtest's 2 MiB spawned thread — a 4× error in the
quantity the requirement's argument rests on, caught by the architect and
verified with a standalone binary (§1.1).

Both point the same way: **a measurement is not a fact until its harness has been
checked as carefully as its result.** Recorded so whoever builds this
requirement's arena is warned twice over.

## 7. Recommendation after the cap probe (2026-08-18)

**Do not build R-0019 as scoped.** §2.1 removed its stated justification. What
remains is latent safety for a hazard §1.3 measured as unreached — real, but not
worth fourteen rewrites across two crates and four PRs.

Two pieces are worth keeping, and they are severable:

1. **AC8 (§1.2's exponential `grade`) should become its own requirement.** It is
   a *complexity* bug, not a depth bug, with a live measured cost: `typecheck` at
   71 ms on a 55-node genome that fits **inside today's cap**, ~1,150× the
   `eval` it screens for, on a path that runs for every candidate. It needs no
   depth contract, no arena, and no iterative rewrite — only that `grade(r)` be
   computed once per `Sandwich`. It very likely also explains most of §2.1's
   7.6×/12.8× cost, which means fixing it first would make any future cap
   experiment both cheaper **and** fairer.
2. **The measurement itself stays** — `r_0019_cap_probe.rs` is committed with its
   verdict, so the next person to propose raising `max_nodes` starts from data
   rather than from the same intuition I had.

The depth work proper (AC1–AC5, AC7) should be **shelved**, not closed: if the
exponential fix lands and a properly powered sweep (~247 seeds/arm) then shows
the cap *is* binding, the justification returns and this document is ready.
