# SPEC-0010 — Geometric Forms + the Grade-Type System (`ufl-geo`)

- **Status:** Draft (three-lens applied; grade algebra delegated to garust)
- **Realizes:** R-0010
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-18
- **Depends on:** R-0009 (`ufl-ga` — the `Cl(3,0,1)` PGA kernel `Mv`, extended to
  re-export `garust::{GradeSet, Op}`)
- **Crate:** `crates/ufl-geo` (new — a typed layer over `ufl-ga`)

## 1. Motivation

SPEC-0010 realizes [R-0010](../requirements/0010-geometric-forms-grade-types.md):
the typed geometric expression layer — `GeoExpr` (the forms / the **genotype
R-0011 evolves**), `eval` onto the `ufl-ga` kernel, and a **decidable
grade-type system**. It is to `ufl-ga` what `ufl-syntax`/`ufl-core` are to the
EML kernel.

**Key design decision (three-lens):** the grade algebra is **not hand-rolled** —
garust already ships `GradeSet` and `Op::output_grades`, which encode the grade
rules *correctly* for the real `Cl(3,0,1)` algebra (the Hestenes inner-product
convention where scalars contribute nothing, the degenerate-`e₀` contractions
that vanish, the pseudoscalar grade wrap-down). The first draft hand-wrote these
rules and got three of them wrong against the actual kernel. So `ufl-geo`'s grade
inference **delegates to `Op::output_grades`** for the catalog forms and rides
garust's `GradeSet`; only the three forms garust has no `Op` for — `Sandwich`,
`Exp`, `GradeLift` — get hand-written rules, and those are **sound
over-approximations** (§2.3). This keeps `ufl-geo` riding garust the same way
`ufl-ga` does — a facade, not a fork.

## 2. Design

### 2.1 `GeoExpr` — the geometric AST (the genotype)

```rust
/// A geometric program: a tree of Cl(3,0,1) forms over the ufl-ga kernel.
/// `Clone` + inspectable — the genotype R-0011 mutates and recombines.
#[derive(Clone, Debug, PartialEq)]
pub enum GeoExpr {
    // ── leaves ──
    Param(f64),                 // an evolvable scalar parameter (R-0011 tunes these)
    Basis(u8),                  // a basis blade by index 0..15 (e1=1, e2=2, e3=4, e0=8, e12=3, …)
    Var(String),                // an input multivector, bound at eval

    // ── forms ──
    GradeLift(u8, Box<GeoExpr>),            // 𝒢ₖ — scalar × the lowest-index grade-k blade
    GeoProduct(Box<GeoExpr>, Box<GeoExpr>), // ∗
    Wedge(Box<GeoExpr>, Box<GeoExpr>),      // ∧  (outer)
    Inner(Box<GeoExpr>, Box<GeoExpr>),      // ·  (Hestenes inner)
    Reverse(Box<GeoExpr>),                  // ~
    GradeProject(u8, Box<GeoExpr>),         // ⟨·⟩ₖ
    Sandwich(Box<GeoExpr>, Box<GeoExpr>),   // r x ~r
    Exp(Box<GeoExpr>),                      // exp (rotor/motor from a bivector)
}
```

The form set is **lean and R-0011-aligned** — the operations the
sandwich-discovery de-risk's *findings* motivate (`ufl-discovery/FINDINGS.md` /
`papers-review.md` §4b: a `Sandwich`/`GeoProduct`/`Reverse` structure over a
parameter and an input — note that de-risk's *genome* was a matmul vector-of-
triples, so `GeoExpr` is genuinely greenfield), plus the grade machinery. `Param`
leaves are the evolvable parameters. Motor/`Point` forms are deferred.

`Basis`/`GradeProject`/`GradeLift` carry a raw `u8`; out-of-range values
(`Basis ≥ 16`, grade `> 4`) are **not constructor-rejected but eval/typecheck
rejected** as typed errors (§2.2/§2.5) — `GeoExpr` stays a plain data enum
(simplest for R-0011's operators), and totality is enforced at the boundary.

### 2.2 `eval` — lowering onto the kernel (total)

```rust
pub struct Env { /* Var → Mv, mirrors ufl_core::Env */ }
pub enum GeoError { Unbound(String), BadBlade(u8), BadGrade(u8) }

pub fn eval(e: &GeoExpr, env: &Env) -> Result<Mv, GeoError>
```

Each form lowers onto its `ufl_ga::Mv` operation:

| Form | Lowers to | Error |
|---|---|---|
| `Param(s)` | `Mv::scalar(s)` | — |
| `Basis(i)` | `Mv::basis(i)` | `BadBlade(i)` if `i ≥ 16` (garust `basis` panics ≥ DIM) |
| `Var(n)` | `env[n]` | `Unbound(n)` |
| `GradeLift(k, e)` | `eval(e)` (scalar) × `lowest_blade(k)` | `BadGrade(k)` if `k > 4` |
| `GeoProduct(a,b)` | `eval(a) * eval(b)` | — |
| `Wedge / Inner` | `a.wedge(&b)` / `a.inner(&b)` | — |
| `Reverse(a)` | `a.reverse()` | — |
| `GradeProject(k,a)` | `a.grade(k)` | `BadGrade(k)` if `k > 4` |
| `Sandwich(r,x)` | `eval(r).sandwich(&eval(x))` | — |
| `Exp(a)` | `eval(a).exp()` | — |

`eval` is **total — never a panic**: every garust call that could panic
(`Mv::basis(i)` for `i ≥ 16`) is guarded by a typed `GeoError` first. `lowest_blade(k)`
is the **lowest-index grade-`k` blade** (k=0→`1`, 1→`e1`, 2→`e12`, 3→`e123`,
4→pseudoscalar) — pinned, so `GradeLift`'s value is unambiguous and never lands
on a null blade.

### 2.3 The grade-type system — `GradeSet` + inference (sound over-approximation)

```rust
pub use ufl_ga::GradeSet;     // = garust::GradeSet (a grade bitmask; has len()/iter() for R-0011)

pub struct GradeCtx { /* Var → GradeSet, ⊤ = full(4) if undeclared */ }
pub fn grade(e: &GeoExpr, ctx: &GradeCtx) -> GradeSet
```

**`grade` is a *sound over-approximation*: every grade the value can actually
carry is in the returned set; under the degenerate metric the set may be strictly
larger than the realized support** (a contraction through `e₀` can vanish). This
is the only honest contract for a degenerate algebra — and it is exactly what a
type needs: a sound upper bound never raises a false "incoherent."

Rules — the catalog forms **delegate to `Op::output_grades`** (garust's correct
inference; `n = 4` generators):

| Form | `grade` |
|---|---|
| `Param` | `GradeSet::singleton(0)` |
| `Basis(i)` | `GradeSet::singleton(i.count_ones())` (`⊤` if `i ≥ 16`) |
| `Var(n)` | `ctx[n]` (`full(4)` if undeclared) |
| `GradeLift(k, _)` | `singleton(k)` (`⊤` if `k > 4`) |
| `Wedge / Inner / GeoProduct` | `Op::{Wedge,Inner,Geometric}.output_grades(&[grade(a), grade(b)], 4)` |
| `Reverse(a)` | `Op::Reverse.output_grades(&[grade(a)], 4)` (preserves) |
| `GradeProject(k, a)` | `Op::GradeProject(k).output_grades(&[grade(a)], 4)` (`= {k}∩grade(a)`, possibly `∅`) |
| **`Sandwich(r, x)`** | **`grade(x)` iff `r` is a *statically-known versor*** (§2.4); else the sound product bound `Geometric.output_grades(&[Geometric.output_grades(&[grade(r), grade(x)], 4), grade(r)], 4)` |
| **`Exp(a)`** | `{0}` if `grade(a) ⊆ {0}`; the even subalgebra `{0,2,4}` if `grade(a) ⊆ {0,2}` (exp of an even element is even — covers rotors *and* motors); else `⊤` |

Delegating to `Op::output_grades` makes the Hestenes inner (scalar operands
contribute nothing), the degenerate-`e₀` vanishing, and the pseudoscalar
wrap-down **correct by reuse** — the three bugs the first draft shipped are gone
because the math isn't re-derived. `grade` is total and decidable (finite sets).

### 2.4 The versor predicate + the keystone (AC4)

`Sandwich(r, x)` preserves grade **only when `r` is a versor** (a product of
vectors / an `exp` of a bivector). A conservative, *sound* static predicate:

```rust
fn is_versor(r: &GeoExpr) -> bool   // may say false for some real versors (then the safe bound applies)
```

— `true` for `Exp(b)` where `grade(b) ⊆ {2}` (a rotor/motor), and for
`GeoProduct` of versors; `false` otherwise. Sound: if it says versor, `r` is one,
so `grade(Sandwich(r,x)) = grade(x)` holds; if it says false, the grade rule
falls back to the general product bound (a superset — still sound).

**Keystone (AC4):** `grade(Sandwich(R, Basis(e1))) == {1}` for `R = Exp(GeoProduct(
Param, Basis(e12)))` — `R` is a statically-known versor (`Exp` of a `{2}`
bivector), so the sandwich preserves the grade-1 operand. *A rotated vector is
still a vector.* The same form `eval`s to `e2` (AC2) — the type says "vector →
vector," the eval says "specifically `e₁ → e₂`." Tied to the R-0009 keystone.

### 2.5 Coherence — `typecheck` (AC6)

```rust
pub enum GradeError { Incoherent(GeoExpr), BadBlade(u8), BadGrade(u8) }
pub fn typecheck(e: &GeoExpr, ctx: &GradeCtx) -> Result<GradeSet, GradeError>
```

Returns the inferred `GradeSet`, or `Err` for a **grade-incoherent** sub-form —
principally `GradeProject(k, a)` where `k ∉ grade(a)` (the projection's grade set
is `∅`, unsatisfiable) — and the out-of-range `BadBlade`/`BadGrade` leaves.
`typecheck` and `grade` share one source of truth (the `∅` is a *value* `grade`
produces; `typecheck` reads it), so they cannot disagree. This is the decidable
signal R-0011 prunes on **without evaluating** the candidate.

### 2.6 The s-expr textual reader — deferred (the §5 resolution)

`GeoExpr` is the **homoiconic form representation** (code-as-data: an inspectable
`Clone`-able tree — what `Eml` is before R-0003 added a reader). A **textual
`Sexpr → GeoExpr` reader is deferred**: R-0011 evolves the `GeoExpr` AST directly
(it consumes the AST, not text), so a textual surface has no consumer until a
*human* wants to hand-write geometric programs. Deferring keeps `ufl-geo`
depending only on `ufl-ga` (the reader would pull in `ufl-syntax`), and avoids a
reader nothing calls (CLAUDE.md §2). **R-0010's AC5 pre-authorized this defer**
("or deferred if the three-lens finds it premature"); it is recorded as an
amendment in R-0010's decision log (not a silent drop).

## 3. Code outline

`crates/ufl-geo/src/`: `expr.rs` (`GeoExpr`, `Env`, `lowest_blade`), `eval.rs`
(`eval`, `GeoError`), `grade.rs` (`grade`, `typecheck`, `GradeCtx`, `GradeError`,
the `is_versor` predicate). `lib.rs` re-exports + `pub use ufl_ga::GradeSet`.
A small **additive extension to `ufl-ga`**: `pub use garust::{GradeSet, Op}`
(the facade gains the grade alphabet — no behaviour change, R-0009 tests
unaffected). `examples/hello_geo.rs`: build `Sandwich(Exp(Param·Basis(e12)),
Basis(e1))`, `eval` it to `e2`, `grade` it to `{1}` — the typed superset of
`hello_ga` (the kernel example shows the sandwich; this shows it *with the grade
proof*).

## 4. Non-goals

- **The evolution** (operators, fitness, search over `GeoExpr`) — R-0011.
- **Motors / `Point` / translations as forms** — later.
- **The textual `Sexpr → GeoExpr` reader** — deferred (§2.6).
- **A full type *checker*** beyond grade coherence — the inference is the
  deliverable.
- **A hand-rolled grade algebra** — delegated to garust's `Op::output_grades`.

## 5. Open questions — resolved

| R-0010 §5 question | Resolution |
|---|---|
| Grade-typing depth | **Inference (`GradeSet`, a sound over-approximation) + a coherence `typecheck`** (§2.3/§2.5), delegating catalog forms to `Op::output_grades`. |
| s-expr surface now vs deferred | **Deferred** (§2.6) — pre-authorized by R-0010 AC5; recorded in R-0010's decision log. |
| Form set / leaves for R-0011 | The lean set of §2.1; `Param`/`Basis`/`Var` leaves; out-of-range leaves are typed errors, not panics. |
| Crate | **New `crates/ufl-geo`, depending only on `ufl-ga`** (which re-exports `garust::{GradeSet, Op}`). |

## 6. Acceptance criteria

- [ ] **AC1 — The geometric AST.** `ufl-geo` exposes `GeoExpr` (§2.1),
  `Clone + Debug + PartialEq`, inspectable for R-0011's operators.
- [ ] **AC2 — Evaluation onto the kernel, total.** `eval(GeoExpr, env) → Result<Mv,
  GeoError>` lowers each form onto its `ufl_ga` op; a form tree's value equals the
  hand-written `ufl_ga` composition within `ε` — incl. `eval(Sandwich(rotor,
  Basis(e1)))` whose grade-1 projection equals itself and is `≈ e2`. **Total:**
  an unbound `Var`, a `Basis(i ≥ 16)`, or a grade `> 4` returns a typed `GeoError`
  — **never a panic** (a test constructs each and asserts `Err`).
- [ ] **AC3 — Grade inference (sound, total, delegated).** `grade` returns a
  **sound over-approximation** of the result grades (every realizable grade is in
  the set; the degenerate metric may make it larger), computed by delegating the
  catalog forms to `Op::output_grades` and applying the §2.3 rules for
  `Sandwich`/`Exp`/`GradeLift`. Total, no panic, over `0..=4`.
- [ ] **AC4 — The grade-preservation keystone.** For `R = Exp(GeoProduct(Param,
  Basis(e12)))` (a statically-known versor) and a grade-1 `v`,
  `grade(Sandwich(R, v)) == GradeSet::singleton(1)` — vector → vector — and the
  same form `eval`s to a rotated vector (`eval(...).grade(1)` equals the value
  within `ε`). A *non*-versor `r` yields the sound product bound (a superset),
  asserted distinct from `{1}`.
- [ ] **AC5 — Homoiconic AST (reader deferred).** `GeoExpr` is the code-as-data
  form representation; the textual `Sexpr → GeoExpr` reader is a documented
  non-goal here (§2.6), per R-0010 AC5's pre-authorization + decision-log
  amendment.
- [ ] **AC6 — Grade coherence.** `typecheck` returns the inferred `GradeSet`, or a
  typed `GradeError` for an incoherent form (`GradeProject(k, a)` with
  `k ∉ grade(a)` → `∅`; out-of-range leaves) — the decidable pruning signal.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-18 | `GeoExpr` AST + `eval` onto `ufl-ga` + `grade`/`typecheck`; **new `ufl-geo` crate over `ufl-ga` only**. | Mirrors `ufl-core`+`ufl-syntax` over the EML kernel; `ufl-ga`-only keeps it a clean leaf (the reader that would pull `ufl-syntax` is deferred). |
| 2026-06-18 | **Grade algebra delegated to garust's `Op::output_grades` + `GradeSet`** (re-exported via `ufl-ga`); only `Sandwich`/`Exp`/`GradeLift` get hand rules. | The three-lens proved a hand-rolled grade algebra is unsound against the real degenerate `Cl(3,0,1)` (Hestenes inner, vanishing `e₀` contractions, pseudoscalar wrap-down). garust already encodes it correctly and tests it. Reuse, don't fork — the same discipline as `ufl-ga` being a facade. |
| 2026-06-18 | `grade` is a **sound over-approximation**, not exact. | A degenerate metric makes the realized support strictly smaller than the structural grade set; a *sound upper bound* is what a type needs (never a false "incoherent") and what `Op::output_grades` provides. AC3 reworded accordingly. |
| 2026-06-18 | **`Sandwich` grade rule is versor-conditioned** (§2.4): preserve grade only when `r` is a statically-known versor (`Exp` of a bivector / product of versors), else the sound product bound. | `r x ~r` preserves grade *iff `r` is a versor*; `Sandwich` takes an arbitrary `r` and R-0011 mutates it, so a non-versor `r` is the common case. The keystone (AC4) uses the versor case and stays crisp; everything else stays sound. |
| 2026-06-18 | **`eval` total** — out-of-range `Basis`/grade are typed `GeoError`s, not panics; `GradeLift`'s blade pinned to lowest-index. | `Mv::basis(≥16)` panics; R-0011 generates raw `u8` leaves, so the boundary must guard (the R-0009 totality discipline). Lowest-index `GradeLift` is unambiguous and dodges the null blade. |
| 2026-06-18 | **Textual s-expr reader deferred** (§2.6); R-0010 AC5 amended + decision-logged. | R-0011 evolves the AST directly (no text consumer); building the reader now is premature. Pre-authorized by R-0010 AC5. |

## 8. Companion edits (this branch)

- `requirements/0010-…md` — AC5 amended (reader-defer made explicit) + a
  decision-log row; the dependency line narrowed to `ufl-ga`-only.
- `crates/ufl-ga` — `pub use garust::{GradeSet, Op}` (additive; R-0009 tests
  unaffected).

## Changelog

- 2026-06-18 — created (Draft).
- 2026-06-18 — three-lens applied (hater NEEDS WORK + architect REQUEST CHANGES,
  both ran the grade rules against garust `292bce5`; nice-guy STRONG WORK):
  grade algebra **delegated to garust `Op::output_grades` + `GradeSet`** (fixing
  the unsound inner / geo-product / sandwich rules); `grade` reframed as a sound
  over-approximation (AC3); `Sandwich` versor-conditioned (AC4 keystone kept
  crisp + sound); `eval` made total (typed errors for out-of-range `Basis`/grade,
  no panic); `GradeLift` blade pinned to lowest-index; `Exp` rule widened to the
  even subalgebra; `Env`/`GradeCtx` specified; the de-risk "evolved this" wording
  softened to "findings motivate"; R-0010 AC5 amendment recorded.
