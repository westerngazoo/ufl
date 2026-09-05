# SPEC-0020 — One pass for `grade` and `typecheck`

- **Realizes:** [R-0020](../requirements/0020-single-visit-grade.md) (Accepted).
- **Status:** **Draft (rev 3)** — three-lens round 1 complete (architect REQUEST
  CHANGES, hater NEEDS WORK, nice-guy SOLID) and folded. **Round 2: both the
  architect and hater agents stalled twice**, so the main session ran their
  checks itself (§6, §8) — the spec's code compiled as printed, fuzzed 0/300,000
  against the old functions, and the ladder held. **One open decision remains for
  Gustavo (§7).**
- **Crate touched:** `ufl-geo`, one file — `src/grade.rs`. No public signature
  changes: `GradeSet`, `GradeCtx`, `GradeError`, `grade`, `typecheck` keep their
  types exactly. `is_versor` (`pub(crate)`) is **deleted** (§2.4).

## 1. The defect, precisely — and by mechanism, not only by clock

`grade.rs:120-131` — `Sandwich(r, x)` calls `is_versor(r)`, which for `Exp(b)`
computes `grade(b)`; the non-versor branch then computes `grade(r)`, whose `Exp`
arm computes `grade(b)` again. Two full walks of `r` per `Sandwich` whose rotor
is not a provable versor. When the rotor itself contains such a `Sandwich`, the
work doubles per level: **2^depth**.

`grade.rs:176` — `typecheck` calls `grade(e)` at every node of its own recursion,
so it re-walks each subtree once per ancestor: **O(n²)**, over the above.

Counted (architect, by an entry counter on the old code): on the rotor-nested
shape at k = 10 — **31 nodes** — old `grade` makes **5,116 entries**. At k = 22,
old `typecheck` makes 62.9M entries in 615 ms. That entry count, not a timing, is
what §4.3 asserts against.

## 2. Design

### 2.1 The invariant: `grade` and the versor predicate are one function with two outputs

Every arm of `grade` depends only on its children's grade sets, and every arm of
`is_versor` only on its children's versor flags — except the two that cross:
`is_versor(Exp(b))` reads `grade(b)`, and `grade(Sandwich(r, x))` reads
`is_versor(r)`. Two bottom-up analyses that read each other at a non-leaf arm are
**one** bottom-up function returning both, never two mutually recursive functions
(the convention this spec promotes, §5):

```rust
/// What one post-order visit learns about a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Analysis {
    /// A sound over-approximation of the result grades (SPEC-0010 §2.3).
    grade: GradeSet,
    /// `true` only when the sandwich rule `r ∗ x ∗ ~r` provably **preserves
    /// `x`'s grade set** (SPEC-0010 §2.4) — the property `Sandwich` needs, and
    /// the only thing this flag promises. Conservative: may be `false` for a
    /// real versor (the safe product bound then applies). Witnesses: `Exp` of a
    /// pure bivector, a single basis *vector*, a `GeoProduct` of witnesses,
    /// `Reverse` of a witness.
    ///
    /// Not "is a versor": `Basis(8) = e₀` is null and non-invertible, yet the
    /// flag is `true` and *sound* — `e₀ x e₀ = 0`, whose grade set ∅ is a subset
    /// of anything. The old doc comment claimed "never `true` for a non-versor";
    /// `docs/tasks/14` (#70) recorded that as false, and this spec fixes the
    /// claim rather than moving it.
    versor: bool,
}
```

**Reconciling with SPEC-0019 §2.2**, which declined to merge `grade` and
`is_versor` into one machine: that refusal was for *depth safety*, where merging
buys nothing and the "typed stacks" safety claim was overstated. This spec merges
for *complexity*, where the merge **is** the fix — every cross-dependency becomes
a field read and no arm calls a walk. Both decisions are correct; they answer
different questions.

### 2.2 `rule` — the per-node algebra, written once, owning its own arity

The rule at a node is a function of the node and its children's analyses. Rather
than collect the children into a buffer and pattern-match on arity (rev 1 —
which cost an allocation per node and a wildcard arm that lost exhaustiveness),
`rule` receives the recursion as a visitor and calls it exactly where each child
is named:

```rust
/// The grade/versor rule at one node. `kid` is the recursion — `rule` never
/// walks; it only says *which* children to visit, *in which order*, and *what*
/// to conclude. Total on every node: the totality defaults (`⊤` for an
/// out-of-range leaf, `∅` for a bad projection) are here, exactly as in today's
/// `grade` (SPEC-0010 §2.3); it is `check` that promotes them to errors.
fn rule<E>(
    e: &GeoExpr,
    ctx: &GradeCtx,
    kid: &mut impl FnMut(&GeoExpr) -> Result<Analysis, E>,
) -> Result<Analysis, E> {
    let out = |grade, versor| Analysis { grade, versor };
    Ok(match e {
        GeoExpr::Param(_) => out(GradeSet::singleton(0), false),
        GeoExpr::Basis(i) => out(
            if *i >= 16 { GradeSet::full(N) } else { GradeSet::singleton(i.count_ones() as usize) },
            *i < 16 && i.count_ones() == 1,
        ),
        GeoExpr::Var(name) => out(ctx.get(name), false),
        GeoExpr::GradeLift(k, a) => {
            kid(a)?; // visited so `check` can reject it; its analysis is not consulted (as today)
            out(if *k > 4 { GradeSet::full(N) } else { GradeSet::singleton(*k as usize) }, false)
        }
        GeoExpr::GeoProduct(a, b) => {
            let (a, b) = (kid(a)?, kid(b)?); // left, then right
            out(Op::Geometric.output_grades(&[a.grade, b.grade], N), a.versor && b.versor)
        }
        GeoExpr::Wedge(a, b) => {
            let (a, b) = (kid(a)?, kid(b)?);
            out(Op::Wedge.output_grades(&[a.grade, b.grade], N), false)
        }
        GeoExpr::Inner(a, b) => {
            let (a, b) = (kid(a)?, kid(b)?);
            out(Op::Inner.output_grades(&[a.grade, b.grade], N), false)
        }
        GeoExpr::Reverse(a) => {
            let a = kid(a)?;
            out(Op::Reverse.output_grades(&[a.grade], N), a.versor)
        }
        GeoExpr::GradeProject(k, a) => {
            let a = kid(a)?;
            // Guard the raw `u8` before garust: `singleton(k) = 1 << k` overflows
            // `u32` for `k ≥ 32`. Projecting onto a grade the algebra lacks is ∅.
            out(if *k > 4 { GradeSet::EMPTY } else { Op::GradeProject(*k).output_grades(&[a.grade], N) }, false)
        }
        GeoExpr::Sandwich(r, x) => {
            let (r, x) = (kid(r)?, kid(x)?);
            let grade = if r.versor {
                x.grade // a versor sandwich preserves the operand's grade
            } else {
                // The sound product bound: grades of `(r ∗ x) ∗ r` (`~r` carries
                // the same grades as `r`).
                let rx = Op::Geometric.output_grades(&[r.grade, x.grade], N);
                Op::Geometric.output_grades(&[rx, r.grade], N)
            };
            out(grade, false)
        }
        GeoExpr::Exp(a) => {
            let a = kid(a)?;
            let grade = if subset_of(a.grade, &[0]) {
                GradeSet::singleton(0)
            } else if subset_of(a.grade, &[0, 2]) {
                GradeSet::EMPTY.with(0).with(2).with(4) // exp of an even element is even
            } else {
                GradeSet::full(N)
            };
            out(grade, subset_of(a.grade, &[2]))
        }
    })
}
```

What this buys, each verified by a lens:

- **Exhaustive on `e`.** A twelfth `GeoExpr` variant is E0004 at compile time,
  as today (hater F4). No wildcard, no `unreachable!`, no `children` helper — the
  arm that names a child is the arm that visits it, so there is one arity source
  by construction.
- **Allocation-free.** No buffer of children's analyses (architect 2, hater F1).
- **Order is explicit at the binding site**: `(kid(a)?, kid(b)?)` is
  left-then-right, and `?` makes the first error win.
- **Every cross-dependency is a field read** — `Sandwich` reads `r.versor`,
  `Exp` sets `versor` from `a.grade`. No arm calls a walk. That is the whole fix.

One quirk is preserved deliberately: `Exp` of an ∅-graded operand is a vacuous
versor (`subset_of(∅, {2})` is `true`). Sound — ∅ means identically zero, and
`exp(0) = 1`.

### 2.2b The alternative: slice patterns over a fixed buffer

The rev-1 shape, corrected per hater F4 (match on `e` exhaustively; arity checked
**per arm** with `let … else`, so a twelfth variant is still E0004):

```rust
/// The children of a node, in left-to-right order — the single arity source.
fn children(e: &GeoExpr) -> [Option<&GeoExpr>; 2] {
    match e {
        GeoExpr::Param(_) | GeoExpr::Basis(_) | GeoExpr::Var(_) => [None, None],
        GeoExpr::GradeLift(_, a) | GeoExpr::Reverse(a) | GeoExpr::GradeProject(_, a) | GeoExpr::Exp(a) => [Some(a), None],
        GeoExpr::GeoProduct(a, b) | GeoExpr::Wedge(a, b) | GeoExpr::Inner(a, b) | GeoExpr::Sandwich(a, b) => [Some(a), Some(b)],
    }
}

fn rule(e: &GeoExpr, kids: &[Analysis], ctx: &GradeCtx) -> Analysis {
    let out = |grade, versor| Analysis { grade, versor };
    match e {
        GeoExpr::Param(_) => out(GradeSet::singleton(0), false),
        // … leaves as in §2.2 …
        GeoExpr::GeoProduct(..) => {
            let [a, b] = kids else { unreachable!("`children` gives GeoProduct two kids") };
            out(Op::Geometric.output_grades(&[a.grade, b.grade], N), a.versor && b.versor)
        }
        // … one arm per variant, each destructuring its own arity …
    }
}

fn analyse(e: &GeoExpr, ctx: &GradeCtx) -> Analysis {
    let mut buf = [Analysis::EMPTY; 2];
    let mut n = 0;
    for c in children(e).into_iter().flatten() { buf[n] = analyse(c, ctx); n += 1; }
    rule(e, &buf[..n], ctx)
}
// `check` identically, with the pre-order guards and post-order ∅ of §2.3.
```

Same semantics (0 mismatches on the same 300,000 trees), same single-visit
property, allocation-free, exhaustive on `e`. Costs one `unreachable!` per
non-leaf arm, justified under existing precedent (`eval.rs:101,109`,
`eml.rs:69,76`, `eval_pred.rs`) because `children` is the sole arity source.

### 2.2c The measured trade between the two forms

| | visitor (§2.2) | slice + buffer (§2.2b) |
|---|---|---|
| semantics vs old, 300K trees | 0 mismatches | 0 mismatches |
| `typecheck` ns/call, release, random ≤60 (architect) | **115 (0.50×)** | 121 (0.53×) |
| `analyse` depth ceiling, **release**, 1 MiB | 16,567 | 16,566 |
| `check` depth ceiling, **release**, 1 MiB | 9,466 | 9,466 |
| `analyse` depth ceiling, **debug**, 1 MiB | **958** (old: 2,362) | 2,203 |
| `check` depth ceiling, **debug**, 1 MiB | **400** (old: 1,406) | 1,377 |
| `unreachable!` in library code | none | one per non-leaf arm |
| exhaustive on `GeoExpr` | yes | yes |

In release the two are indistinguishable — LLVM inlines the closure. In
**debug**, `cargo test`'s profile, the visitor keeps `rule`'s frame *and* the
closure's on the stack during recursion — three frames per level instead of one
— and loses **59–72%** of depth against today's code, on a `pub` surface. The
slice form is at parity with today. The visitor's whole advantage is 6% of a
function that is 15–26% of lane wall-clock: ~1% end to end.

**Recommendation: the slice form (§2.2b).** Debug is the profile every test
runs in, depth on a `pub` function is a stated property (§2.6), and 1% release
throughput is inside the noise of the R-0020 §2 time split. The cost is the
`unreachable!`s, which are of the class CLAUDE.md §6 permits and this repo
already uses. This **reverses the architect's round-1 preference** on data it
did not have (it measured release only); it is put to Gustavo as §7 Q1, and
whichever form is chosen, the other's section is deleted at Acceptance.

### 2.3 The two drivers

```rust
/// The total pass: every node visited exactly once (§4.3 asserts it).
fn analyse(e: &GeoExpr, ctx: &GradeCtx) -> Analysis {
    let Ok(a) = rule(e, ctx, &mut |c| Ok::<_, Infallible>(analyse(c, ctx)));
    a
}

pub fn grade(e: &GeoExpr, ctx: &GradeCtx) -> GradeSet {
    analyse(e, ctx).grade
}

/// `typecheck` is `grade` with its totality defaults promoted to errors: the
/// same `rule` at every node; the heads where `grade` returns ⊤/∅ to stay total
/// are rejected *before* descent, and an ∅ that survives composition is rejected
/// *after* it — so the innermost incoherent subtree is the one reported.
fn check(e: &GeoExpr, ctx: &GradeCtx) -> Result<Analysis, GradeError> {
    match e {
        GeoExpr::Basis(i) if *i >= 16 => return Err(GradeError::BadBlade(*i)),
        GeoExpr::GradeLift(k, _) | GeoExpr::GradeProject(k, _) if *k > 4 => {
            return Err(GradeError::BadGrade(*k))
        }
        _ => {}
    }
    let a = rule(e, ctx, &mut |c| check(c, ctx))?;
    if a.grade.is_empty() {
        Err(GradeError::Incoherent(e.clone()))
    } else {
        Ok(a)
    }
}

pub fn typecheck(e: &GeoExpr, ctx: &GradeCtx) -> Result<GradeSet, GradeError> {
    check(e, ctx).map(|a| a.grade)
}
```

The irrefutable `let Ok(a) = …` over `Result<_, Infallible>` is stable Rust
(1.82+; CI is `stable`, local 1.95).

**Error precedence is preserved exactly** (SPEC-0019 §2.3): own
`BadBlade`/`BadGrade` before descent; children left to right with `?`; own
`Incoherent` after both. The architect traced all three §4.6 cases through both
implementations; the hater fuzzed 370,079 trees; zero divergence, including
*which* subtree `Incoherent` carries.

**The bound, stated** (R-0020 §6): *c* = **1**. `analyse` enters each node
exactly once. `check` enters exactly the nodes it reaches before the first
error — all of them on every `Ok` tree. §4.3 asserts **equality** with the node
count, not `≤`.

### 2.4 What is deleted

- The recursive `grade` (11 arms) and `typecheck`'s per-node `grade(e)` call.
- **`is_versor`, entirely.** Its only caller was `grade`'s `Sandwich` arm; under
  §2.2 that is `r.versor`, so a `pub(crate)` wrapper would be dead code and CI
  (`clippy --workspace --all-targets -- -D warnings`) turns red (architect,
  blocking). The predicate lives as `Analysis::versor`, its doc comment moves
  onto the field (§2.1), and the prose mention at `slots.rs:8` is updated. AC3's
  "fuzz the predicate directly" holds with `analyse(r, ctx).versor` as the
  subject, in the same in-`src` test module.

  Every reference the deletion touches (grepped): `slots.rs:8` (prose — update);
  `specs/0010 §2.4/§2.5 :148,:192` and `specs/0011 :283` (accepted specs — a
  dated note, not a rewrite); `docs/tasks/14 :13` and `docs/tasks/README :91`
  (historical task notes — leave, and close their `is_versor` doc-claim item as
  discharged by §2.1's reworded field doc); `specs/0019` and `requirements/0019`
  (shelved — leave).

Net: three walks become one algebra plus two ten-line drivers.

### 2.5 Cost — measured, and the reason rev 1 was wrong

Release, `typecheck` ns/call. Architect on 200K random trees ≤ 60 nodes (avg
17.6); hater on 20K trees from `GeoProposer::random_expr`'s exact distribution
and 20K crossover-bloated ≤ 60:

| form | architect (random ≤60) | hater (proposer) | hater (bloated ≤60) |
|---|---|---|---|
| old `typecheck` | 230 | 146 | 390 |
| rev 1 — `Vec` per node | 228 (0.99×) | **171–176 (+18–20%)** | **435–444 (+11–15%)** |
| `[Analysis; 2]` buffer | 121 (0.53×) | 100 (−31%) | 254 (−35%) |
| **§2.2 visitor** | **115 (0.50×)** | — | — |

On production shapes the old `typecheck` is O(n²) over an O(n) `grade` at small
n, so the algorithmic win is ~2× — and one heap allocation per node ate exactly
that. Rev 1 deferred the buffer "if the throughput test shows the allocation";
it does, and since the screen is 15–26% of lane wall-clock (R-0020 §2), rev 1 was
a ~3–5% *regression* sold as hygiene. The visitor form is the fastest measured
and is adopted. R-0020 §3's pricing stands — this is still hygiene — but the
~10% end-to-end is now a real, measured side effect rather than a claim.

### 2.6 Depth — stated, not hidden

Both drivers add a frame per level versus today (the closure). Two consequences
the hater measured on a 1 MiB thread:

- `grade` on a plain `Reverse` chain, **measured** (§2.2c): release — old 9,466,
  either new form ≈ 16,570 (**+75%**); debug — old 2,362, slice form 2,203
  (parity), visitor form **958**. `typecheck`: release parity at 9,466 for all
  three; debug — old 1,406, slice 1,377, visitor **400**. Rev 2's "expected
  between" was wrong in both directions at once; the numbers are now in the spec
  rather than deferred to a test.
- `analyse` descends into `GradeLift`'s child; today's `grade` does not. So
  `grade(GradeLift(1, Reverse^N(Param)))` survives N = 4,000,000 today and
  ~4,700 after. This bites only `pub fn grade` called directly on a deep
  `GradeLift` — no production caller exists (`lane.rs:47` and all of
  `ufl-evolve` use `typecheck`, which **already** descends there,
  `grade.rs:163`). Uniformity is kept, for the architect's reason: two arity
  sources, one per driver, is exactly what would make any future guard
  unjustifiable. Stated here so it is a choice, not a surprise.

Shape (C) at k = 1,000 (3,001 nodes) runs in 57–104 µs in release without
incident; the shelved depth contract (R-0019) is not made worse at any realistic
size.

## 3. Non-goals

- **No semantics change.** `rule` is today's arms transcribed; the versor
  predicate is exactly as conservative. §4.1's fuzz is the proof.
- **Not the depth contract.** Both drivers stay recursive.
- **Not the refiner's neighbor scaling** (R-0020 §2 — the real cost at large
  caps).
- **No `GradeError` change.** `Incoherent` keeps its payload (SPEC-0019 §2.7).

## 4. Tests (TDD — written first, red)

1. **T-differential (AC3)** — `grade` and `typecheck` against **verbatim
   transcriptions** of the pre-change functions kept in the test file
   (`old_grade`, `old_is_versor`, `old_typecheck`), over random `GeoExpr`s
   covering all 11 variants, `Basis` 0..=255, `k ∈ {0..=5, 31, 32, 255}`, vars
   declared `{1}`/`{0,2}`/`∅`/`{7}` and undeclared. Identical `GradeSet`;
   identical `Result` including which error and which subtree `Incoherent`
   carries. **Plus, pinned explicitly:** shape (C) at k ∈ 0..=16 — random fuzz
   reaches rotor-nest ≥ 5 at 2×10⁻⁴ and nest 7+ never (hater F6), so without this
   the differential never exercises what the change exists for. The old oracle
   costs ~30 ms for the whole series at k ≤ 16.
2. **T-versor-direct (AC3)** — `analyse(r, ctx).versor` against `old_is_versor`
   in an in-`src` `#[cfg(test)]` module. Through `grade(Sandwich(r, x))` alone a
   wrong answer is masked wherever `grade(r) = {0}`.
3. **T-visit-ladder (AC1, AC2)** — the mechanism. A module-private
   `#[cfg(test)] thread_local! { static VISITS: Cell<usize> }` ticked at entry of
   `analyse` and `check` — counts real entries, costs nothing in production,
   touches no struct or API, and is stable under `cargo test`'s parallel threads
   (a `static AtomicUsize` would not be). Asserts `VISITS == node_count` on
   **all three** shapes (A) motor chain, (B) operand-nested non-versor, (C)
   rotor-nested, at k ∈ {10, 20, 40, 64} **ascending, in one `#[test]`**. A
   regression fails at k = 10 in ~150 µs and never reaches k = 64; if k = 10
   passes, the walk is per-node linear and k = 64 (193 nodes, 6.4 µs) cannot
   hang. No clock, no subprocess. Rev 1's "T-feasibility" is subsumed: (C) alone
   had a hole (hater F2 — an operand re-walk passes (C) in 12 µs and is 2^k on
   (A)); the ladder over all three closes it. Also asserts on 1,000 random trees.
4. **T-versor-sound** (new, nice-guy) — for every random `r` with
   `analyse(r).versor == true`: `realized(eval(r) · eval(r).reverse()) ⊆ {0}`.
   Sound as a necessary condition in `Cl(3,0,1)` (`exp(B)·exp(B)~ = 1`;
   `eᵢeᵢ` is scalar, including `0` for `e₀`; products and reverses inherit). The
   predicate's own soundness is only indirectly covered today; with `versor` a
   field on every node this is one assertion long.
5. **T-typecheck-implies-eval** (new, nice-guy) — `check`'s three strict points
   are `eval`'s three guards at the same pre-order position (`eval.rs:29-51`), so
   `typecheck(e).is_ok() ⇒ eval(e, env)` fails only with `Unbound`. Nothing pins
   that implication today; one assertion over the existing fuzz generator.
6. **T-precedence** — four named cases: `Sandwich(Basis(20), GradeProject(9, x))`
   → `BadBlade(20)`; `GradeProject(9, Basis(20))` → `BadGrade(9)`;
   `GradeProject(2, GradeProject(3, Param))` → `Incoherent` carrying the **inner**
   projection; and (architect) `GeoProduct(Var z /* declared ∅ */, Basis(20))` →
   `Incoherent(Var "z")`, **not** `BadBlade(20)` — the left child's post-order
   error beats the right child's pre-order one, because children are fully
   processed left to right. The non-obvious consequence of the ordering; both
   implementations agree; nothing pinned it.
7. **T-remeasure (AC4)** — R-0020 §1's table (k = 14/18/20/22) re-run and printed
   **unconditionally**. The "before" comes from `old_typecheck` in the test file,
   since the old code is deleted. `#[ignore = "release e2e: …"]` gated like
   `r_0019_cap_probe.rs`, so the numbers are release numbers.
8. **T-throughput and T-depth** — `typecheck` on production shapes (the proposer
   distribution and ≤ 60 bloated), old vs new, release, same gating, reported
   unconditionally; and the §2.6 `Reverse`-chain ceiling for the visitor form,
   measured once and recorded. Expected −30–50% on throughput; a regression is a
   finding, not a failure.

## 5. Deliverables beyond code

- **`docs/conventions.md` — *Fused Synthesized Attributes*.** Two bottom-up
  analyses over one tree that read each other at any non-leaf arm are one
  function returning a product, never two mutually recursive functions. Review
  check: *does any arm of a walk call a different walk on a child?* Instance:
  this spec. Counter-instance, recorded for the right reason: SPEC-0019 §2.2,
  where merging for *safety* was correctly declined.
- **SPEC-0010 §2.5** gains a dated note: "they cannot disagree" was true by
  *call* (`typecheck` invoked `grade`); after SPEC-0020 it is true by
  *construction* (both are `rule`). The stronger form.
- `slots.rs:8` prose no longer names `is_versor`.

## 6. Round-1 questions, resolved

| rev 1 question | resolution | by |
|---|---|---|
| Q1 how to count visits | `#[cfg(test)] thread_local!` on entry; option (a) counted composition, not calls — a discarded re-walk left it green | all three |
| Q2 `GradeLift` descent | keep uniform; `check` already descends there; cost stated in §2.6 | architect, hater |
| Q3 60 s ceiling a clock? | not a violation, but unnecessary — subsumed by the ascending ladder | architect (hater: not flaky, 29 µs at k=64) |
| Q4 worth building? | yes, priced as hygiene; the visitor form makes it a measured −50% on the screen *and* a simplification of a soundness-critical file | all three |
| — `Vec` per node | measured to cancel the gain; visitor form adopted | architect, hater |
| — `match (e, kids)` wildcard | lost exhaustiveness; visitor form matches `e` | hater, architect |
| — `is_versor` wrapper | dead code, CI red; deleted | architect |
| **round 2** (agents stalled ×2 each) | main session ran the checks: spec code compiled **as printed** under `#![deny(warnings)]`; 0/300,000 vs old (both forms); (C) k≤16 pinned, 0 mismatches; ladder `visits == nodes` on (A)/(B)/(C) × k∈{10,20,40,64}, both drivers; all four §4.6 precedence cases agree, incl. `Incoherent(Var z)` over `BadBlade(20)`; depth bisected in both profiles; `is_versor` grep complete | main session |

## 7. Open — for Gustavo

1. **Which form: visitor (§2.2) or slice + buffer (§2.2b)?** §2.2c has the
   complete measurement. Recommendation: **slice**, on debug depth. The
   architect's round-1 preference was the visitor, on release throughput only.
   Both are correct; this is the one decision the data does not make alone
   because it weighs a `pub`-surface property in the CI profile against 1% of
   wall-clock in the production one.
2. **Accept on the main-session verification (§6), or re-attempt the architect
   agent?** Every round-1 finding is folded and every round-2 check the
   architect was asked to make has been run and passed — but by me, not by the
   agent CLAUDE.md §4 names. Stated plainly so the record shows who verified what.

## 8. What the main-session verification ran

`scratchpad/spec20` (not committed): §2.1–§2.3 transcribed *as printed*, plus
§2.2b, plus verbatim `old_grade`/`old_typecheck` via the real crate; a random
generator over all 11 variants, `Basis` 0..=255, `k ∈ {0..=5, 31, 255}`, vars
declared `{1}`/`{0,2}`/`∅` and undeclared; the three shapes at four rungs with a
`thread_local!` entry counter; the four precedence cases; and a 1 MiB-thread
depth bisect per form per profile. Corpus split: 106,331 `Ok` / 45,131
`Incoherent` / 86,488 `BadBlade` / 62,050 `BadGrade`.
