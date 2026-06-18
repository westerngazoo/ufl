# SPEC-0010 — Geometric Forms + the Grade-Type System (`ufl-geo`)

- **Status:** Draft
- **Realizes:** R-0010
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-18
- **Depends on:** R-0009 (`ufl-ga` — the `Cl(3,0,1)` PGA kernel `Mv`)
- **Crate:** `crates/ufl-geo` (new — a typed layer over `ufl-ga`)

## 1. Motivation

SPEC-0010 realizes [R-0010](../requirements/0010-geometric-forms-grade-types.md):
the typed geometric expression layer — `GeoExpr` (the forms / the **genotype
R-0011 evolves**), `eval` onto the `ufl-ga` kernel, and a **decidable
grade-type system**. It is to `ufl-ga` what `ufl-syntax`/`ufl-core` are to the
EML kernel: the forms and the type over a typed core.

## 2. Design

### 2.1 `GeoExpr` — the geometric AST (the genotype)

```rust
/// A geometric program: a tree of Cl(3,0,1) forms over the ufl-ga kernel.
/// `Clone` + inspectable — the genotype R-0011 mutates and recombines.
#[derive(Clone, Debug, PartialEq)]
pub enum GeoExpr {
    // ── leaves ──
    Param(f64),                 // an evolvable scalar parameter (R-0011 tunes these)
    Basis(u8),                  // a basis blade by index (e1=1, e2=2, e3=4, e0=8, e12=3, …)
    Var(String),                // an input multivector, bound at eval

    // ── forms ──
    GradeLift(u8, Box<GeoExpr>),            // 𝒢ₖ — scalar → the canonical unit grade-k blade × it
    GeoProduct(Box<GeoExpr>, Box<GeoExpr>), // ∗
    Wedge(Box<GeoExpr>, Box<GeoExpr>),      // ∧  (outer)
    Inner(Box<GeoExpr>, Box<GeoExpr>),      // ·  (inner)
    Reverse(Box<GeoExpr>),                  // ~
    GradeProject(u8, Box<GeoExpr>),         // ⟨·⟩ₖ
    Sandwich(Box<GeoExpr>, Box<GeoExpr>),   // R x R̃  (versor, operand)
    Exp(Box<GeoExpr>),                      // rotor from a bivector
}
```

The form set is **lean and R-0011-aligned** — exactly the operations the
sandwich-discovery de-risk evolved (`Sandwich`/`GeoProduct`/`Reverse` over a
parameter `Q` and an input), plus the grade machinery (`GradeLift`,
`GradeProject`, `Wedge`, `Inner`, `Exp`). `Param` leaves are the evolvable
parameters (the de-risk's rotor `Q`). Rigid-body motor forms are deferred
(R-0010 non-goal).

### 2.2 `eval` — lowering onto the kernel

```rust
pub fn eval(e: &GeoExpr, env: &Env) -> Result<Mv, GeoError> // Env: Var → Mv
```

Each form lowers onto its `ufl_ga::Mv` operation (so eval == the hand-written
kernel composition, the R-0010 AC2):

| Form | Lowers to |
|---|---|
| `Param(s)` | `Mv::scalar(s)` |
| `Basis(i)` | `Mv::basis(i)` |
| `Var(n)` | `env[n]` (or `Err(Unbound)`) |
| `GradeLift(k, e)` | `eval(e)` (scalar) × the canonical unit grade-`k` blade |
| `GeoProduct(a,b)` | `eval(a) * eval(b)` |
| `Wedge / Inner` | `a.wedge(&b)` / `a.inner(&b)` |
| `Reverse(a)` | `a.reverse()` |
| `GradeProject(k,a)` | `a.grade(k)` |
| `Sandwich(r,x)` | `eval(r).sandwich(&eval(x))` |
| `Exp(a)` | `eval(a).exp()` |

`eval` is total — the only failure is an unbound `Var` (a typed `GeoError`),
never a panic.

### 2.3 The grade-type system — `GradeSet` + inference

```rust
/// The decidable dimensional type: which grades (0..=4) a form's result can
/// carry. A 5-bit mask — finite, so inference is total and poly-time (Haynes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GradeSet(u8);          // bit k set ⟺ grade k possible

pub fn grade(e: &GeoExpr, ctx: &GradeCtx) -> GradeSet   // ctx: Var → GradeSet (⊤ if undeclared)
```

The inference rules are the **standard GA grade algebra**:

| Form | Grade rule |
|---|---|
| `Param` | `{0}` |
| `Basis(i)` | `{popcount(i)}` (the blade's grade) |
| `Var(n)` | `ctx[n]` (⊤ = `{0,1,2,3,4}` if undeclared) |
| `GradeLift(k, _)` | `{k}` |
| `Wedge(a,b)` | `{ gₐ+g_b ≤ 4 }` over the pairs (outer product **raises**) |
| `Inner(a,b)` | `{ \|gₐ−g_b\| }` (inner **lowers**) |
| `GeoProduct(a,b)` | `{ \|gₐ−g_b\|, \|gₐ−g_b\|+2, …, gₐ+g_b } ∩ 0..=4` (**mixes**) |
| `Reverse(a)` | `grade(a)` (**preserves**) |
| `GradeProject(k, a)` | `{k}` if `k ∈ grade(a)`, else `∅` (a coherence flag) |
| `Sandwich(r, x)` | `grade(x)` (a versor sandwich **preserves grade**) |
| `Exp(a)` | `{0,2}` if `grade(a) ⊆ {2}` (bivector → rotor); else ⊤ |

Finite sets over `0..=4`, so `grade` is **total and decidable**. This is UFL's
realization of the Haynes "decidable dimensional type."

### 2.4 The keystone — the sandwich preserves grade (AC4)

`grade(Sandwich(R, Basis(e1))) == {1}` — *a rotated vector is still a vector*.
Tied to the R-0009 keystone: the same sandwich form whose `eval` sends `e₁ → e₂`
(AC2) is grade-typed **vector → vector** (AC4). The dimensional-type invariant
the Program-Hypergraph literature touts, made checkable in one assertion.

### 2.5 Coherence — `typecheck` (AC6)

```rust
pub fn typecheck(e: &GeoExpr, ctx: &GradeCtx) -> Result<GradeSet, GradeError>
```

Returns the inferred `GradeSet`, or `Err` for a **grade-incoherent** form —
principally `GradeProject(k, a)` where `k ∉ grade(a)` (projecting onto an absent
grade yields `∅`, which can never be satisfied). This is the decidable "type
error" R-0011 uses to **prune or down-weight** candidates before they waste a
fitness evaluation. (Not a full reject-on-every-mismatch checker — the grade
*inference* + this coherence gate is the deliverable; §3 records the scope.)

### 2.6 The s-expr textual reader — deferred (the §5 resolution)

R-0010 builds `GeoExpr` as the **homoiconic form representation** (code-as-data:
an inspectable, `Clone`-able tree — exactly what R-0001's `Eml` is before R-0003
added a reader). A **textual `Sexpr → GeoExpr` reader is deferred**: R-0011
evolves the `GeoExpr` AST *directly* (mutation/crossover over the tree), so it
consumes the AST, not text; a textual surface has no consumer until a *human*
wants to hand-write geometric programs. Deferring keeps `ufl-geo` depending only
on `ufl-ga` (not `ufl-syntax`) and avoids a reader nothing yet calls (CLAUDE.md
§2). The reader is a thin later add (`ufl-syntax`'s `Sexpr` already exists; only
the geometric lowering is new). *(The three-lens is asked to confirm this defer
vs. building the reader now for the homoiconic payoff.)*

## 3. Code outline

`crates/ufl-geo/src/`: `expr.rs` (`GeoExpr`), `eval.rs` (`eval`, `GeoError`),
`grade.rs` (`GradeSet`, `grade`, `typecheck`, `GradeError`). `lib.rs` re-exports.
`examples/hello_geo.rs`: build the sandwich form `Sandwich(Exp(Param·Basis(e12)),
Basis(e1))`, `eval` it to `e2`, and `grade` it to `{1}` — the form layer's
`hello_*` (eval-correctness ∧ grade-preservation in one screen).

## 4. Non-goals

- **The evolution** (operators, fitness, search over `GeoExpr`) — R-0011.
- **Motors / `Point` / translations as forms** — later (the form set is over
  `Mv`).
- **The textual `Sexpr → GeoExpr` reader** — deferred (§2.6).
- **A full type *checker*** beyond grade coherence (AC6) — the inference is the
  deliverable.

## 5. Open questions — resolved

| R-0010 §5 question | Resolution |
|---|---|
| Grade-typing depth | **Inference (`GradeSet`) + a coherence `typecheck`** (§2.3/§2.5), not a full reject-on-mismatch checker. |
| s-expr surface now vs deferred | **Deferred** (§2.6) — `GeoExpr` is the homoiconic AST; the textual reader waits for a human-author consumer. |
| Form set / leaves for R-0011 | The lean set of §2.1 — products / reverse / project / sandwich / exp / grade-lift, with `Param`/`Basis`/`Var` leaves (the de-risk's shape). |
| Crate | **New `crates/ufl-geo`**, depending only on `ufl-ga`. |

## 6. Acceptance criteria

- [ ] **AC1 — The geometric AST.** `ufl-geo` exposes `GeoExpr` (the §2.1 forms),
  `Clone + Debug + PartialEq`, inspectable for R-0011's operators.
- [ ] **AC2 — Evaluation onto the kernel.** `eval(GeoExpr, env) → Mv` lowers
  each form onto its `ufl_ga` op; a form tree's value equals the hand-written
  `ufl_ga` composition (within `ε`) — incl. `eval(Sandwich(rotor, Basis(e1))) ≈
  e2` (the R-0009 keystone, through the form layer). `eval` is total (typed
  `Err` on unbound `Var`, no panic).
- [ ] **AC3 — Grade inference (decidable, total).** `grade` computes the correct
  `GradeSet` for every form per §2.3 (wedge adds, inner subtracts, geo-product
  mixes, reverse/sandwich preserve, grade-lift/project produce `{k}`); total,
  no panic, over `0..=4`.
- [ ] **AC4 — The grade-preservation keystone.** `grade(Sandwich(R, v)) == {1}`
  for a grade-1 `v` (and the same form `eval`s to a rotated vector). Vector →
  vector.
- [ ] **AC5 — Homoiconic AST (reader deferred).** `GeoExpr` is the code-as-data
  form representation (a `Clone`-able tree built via constructors); the textual
  `Sexpr → GeoExpr` reader is a documented non-goal here (§2.6), to be added when
  a human-authoring consumer appears.
- [ ] **AC6 — Grade coherence.** `typecheck` returns the inferred `GradeSet`, or
  a typed `GradeError` for an incoherent form (e.g. `GradeProject(k, a)` with
  `k ∉ grade(a)`) — the decidable signal R-0011 prunes on.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-18 | `GeoExpr` AST + `eval` onto `ufl-ga` + `grade`/`typecheck`; **new `ufl-geo` crate over `ufl-ga` only**. | The typed geometric layer mirrors `ufl-core`+`ufl-syntax` over the EML kernel; depending only on `ufl-ga` keeps it a clean leaf (the reader that would pull in `ufl-syntax` is deferred). |
| 2026-06-18 | Grade type = **decidable `GradeSet` inference** (5-bit mask over `0..=4`) + a coherence `typecheck`. | The Haynes decidable dimensional type, concretely: finite grade sets, total/poly-time. A full checker is deferred — inference + coherence is what R-0011 (parsimony + pruning) consumes. |
| 2026-06-18 | The **sandwich-preserves-grade** keystone (AC4), tied to the R-0009 `e₁→e₂` keystone. | The smallest convincing proof the type system is real, and the literature's headline invariant (the program *knows* it has a vector). |
| 2026-06-18 | The **textual s-expr reader is deferred** (§2.6); `GeoExpr` is the homoiconic AST. | R-0011 evolves the AST directly (no text consumer yet); building the reader now is premature (CLAUDE.md §2). Mirrors `Eml` (R-0001) preceding its reader (R-0003). |
| 2026-06-18 | Form set scoped to the **`Mv`-over-rotations core** with `Param`/`Basis`/`Var` leaves; motors/`Point` deferred. | Exactly the shape the structure-evolution de-risk validated (6/6); nothing R-0011's first gate (rediscover `R x R̃`) doesn't need. |

## Changelog

- 2026-06-18 — created (Draft).
