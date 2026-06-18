# SPEC-0009 — `Cl(3,0,1)` PGA Kernel Binding (`ufl-ga`)

- **Status:** Draft (three-lens applied; all findings verified against garust `v0.1.0`)
- **Realizes:** R-0009
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-12
- **Depends on:** garust `v0.1.0` (the confirmed contract C1–C11 —
  [`docs/garust-pga-contract.md`](../docs/garust-pga-contract.md))
- **Crate:** `crates/ufl-ga` (new — a substrate leaf; depends only on garust)

## 1. Motivation

SPEC-0009 realizes [R-0009](../requirements/0009-pga-kernel-binding.md): bind
garust's `Cl(3,0,1)` PGA kernel into UFL as `ufl-ga`, a **thin facade** that
gives the geometric forms (R-0010) and neuroevolution (R-0011) a curated,
UFL-named, UFL-validated surface — without re-implementing geometric algebra.

**What this facade does and does not buy (stated plainly, per the three-lens).**
The earned value is exactly three things:

1. **UFL-named basis constructors** that hide garust's blade-index convention
   (the null generator is `Pga3::basis(8)`, not an obvious "e-zero") behind
   meaningful names (`e0/e1/e2/e3`, `scalar`, `pseudoscalar`);
2. a **single curated import path** (`ufl_ga::…`) R-0010/R-0011 target; and
3. **UFL-owned geometric-correctness tests** — UFL validates the kernel through
   its own surface.

**Insulation is *not* provided at this layer.** `Mv` is a transparent alias
(§2.2), so a garust breaking change would reach R-0010 identically whether it
imports `ufl_ga::Mv` or `garust::Pga3`. Insulation against garust churn comes
from the **version pin** (a `v0.1.0` tag → a locked commit rev), not from a
wrapping type. A delegating newtype that *would* insulate is the deferred
escalation (§2.4). The earlier "insulate UFL from the API" framing in R-0009 §2.1
is reframed accordingly in R-0009's decision log — the curated, *pinned* import
surface, not a re-wrapping type.

## 2. Design

### 2.1 The crate, the pinned dependency, and reproducibility

`crates/ufl-ga/Cargo.toml`:

```toml
[package]
name = "ufl-ga"
# … workspace edition/license/authors/repo …
[dependencies]
garust = { git = "https://github.com/westerngazoo/garust", tag = "v0.1.0" }
```

`#![forbid(unsafe_code)]`. Depends only on garust — **not `ufl-core`** (AC1).

**Reproducibility.** A `git`+`tag` dependency resolves the tag to an immutable
commit rev recorded in `Cargo.lock`. The PR therefore **commits the updated
`Cargo.lock`**, and the `ufl-ga` merge gate runs **`cargo test --locked`** (and
`build --locked`), so the pin is the rev — a re-pointed tag surfaces as a
lockfile diff, never a silent build change.

### 2.2 The multivector surface — alias + named basis constructors (masks pinned)

```rust
/// UFL's geometric value: a Cl(3,0,1) PGA multivector over f64. A transparent
/// alias for garust's kernel type — its inherent methods and the geometric
/// product `*` are available directly, so R-0010 targets `ufl_ga::Mv` without
/// importing garust.
pub type Mv = garust::Pga3;

/// UFL-named basis constructors. The blade indices are **pinned to garust's
/// Cl(3,0,1) convention** (verified in garust's signature: the degenerate
/// generator is last). Each is total — every index is < DIM (16), so
/// `Mv::basis` cannot panic.
pub mod basis {
    use super::Mv;
    pub fn scalar(s: f64) -> Mv { Mv::scalar(s) }      // grade 0
    pub fn e1() -> Mv { Mv::basis(1) }                 // e1²=+1
    pub fn e2() -> Mv { Mv::basis(2) }                 // e2²=+1
    pub fn e3() -> Mv { Mv::basis(4) }                 // e3²=+1
    pub fn e0() -> Mv { Mv::basis(8) }                 // the ideal/null generator, e0²=0
    pub fn pseudoscalar() -> Mv { Mv::pseudoscalar() } // = basis(15), grade 4
}
```

The masks (`e1=1, e2=2, e3=4, e0=8, pseudoscalar=15`) are written here, not left
to the implementer — AC3 alone pins only `e1`/`e0`, so the literals + their
proving assertions (AC3) sit together as self-verifying definitions.

**Operations come free via the alias** (garust's inherent methods on
`Multivector`, callable on `Mv` with no trait import): geometric product
`a * b`, `a.wedge(&b)`, `a.inner(&b)`, `a.grade(k)`, `a.reverse()`,
`a.sandwich(&x)` (note: **argument by reference**), `a.norm()`,
`a.normalized()`. SPEC-0009 adds **no wrapper functions** over these.

**A note on PGA grades.** In `Cl(3,0,1)`, grade-1 elements are *planes*, grade-2
*lines*, grade-3 *points* (garust's `pga::Point` is the grade-3 trivector). AC2
tests grade *structure* (the five grades are distinguishable); AC4 is the
*algebraic* rotor action on a grade-1 basis element; AC5 is the *geometric*
rigid-body motion on a grade-3 `Point`. They are distinct claims.

### 2.3 Versors — rotor, sandwich, Motor, Point

Curated re-exports for the rigid-body surface (the only types re-exported by
name, because R-0010/R-0011 build motions and act on points):

```rust
pub use garust::Motor;        // rotor / translator / identity / rotation_about / Mul
pub use garust::pga::Point;   // Point::new(x, y, z); point.transform(&motor); point.to_euclidean()
```

- A rotor is `Motor::rotor(radians, plane)` = `exp(−½·radians·plane)` for a unit
  bivector `plane` (e.g. `e1() * e2()` for `e₁₂`, with `(e1*e2)² = −1`).
- The **raw algebraic sandwich** is `Mv::sandwich(&self, x: &Self)` — a rotor `R`
  acting `R x R̃` (AC4).
- A motion is applied to a typed point with **`Point::transform(&self, motor:
  &Motor)`** (not `Motor::apply` on a `Point` — `Motor::apply` takes a raw
  `&Pga`); coordinates come back via `point.to_euclidean()` (AC5).
- Motor composition: `M₂ * M₁` applies `M₁` then `M₂`.

`Line`/`Plane`/`join`/`meet` (contract C10) are **not** surfaced here — incidence
arrives when R-0010/R-0011 need it (a cheap re-export addition; `Point` already
pulls in `garust::pga`).

### 2.4 Binding shape — thin facade, newtype deferred

An **alias + named constructors + curated re-exports + tests** — *not* a newtype
re-wrapping every op. A newtype (`struct Mv(Pga3)`) would insulate against a
garust API change, but: (a) garust is pinned + contract-CI-guarded, so churn is
held by the pin; and (b) a newtype costs a delegating method for every operation
— the boilerplate-wrapper CLAUDE.md §2 warns against, with no consumer needing
the opacity today. The newtype is the **deferred escalation**: introduce it only
if UFL gains a second GA backend or a UFL-specific invariant the raw kernel
can't enforce — the same "earned by a second instance" discipline as SPEC-0007's
trait and SPEC-0008's proposer seam.

### 2.5 Tolerance, exactness, and f64 dust

Floating assertions use **`ε = 1e-10`** (unit-scale `f64` geometry; the observed
error on the AC4 keystone is ~`2e-16`, eight orders under ε). garust's
`sandwich`/`transform` leak ~`1e-16` dust into symmetry-zero blades, so tests
**`.cleaned(1e-10)`** before an exact-blade comparison, or compare per
coefficient within ε — the spec mandates `cleaned`-then-compare so qa writes it
the way the kernel intends.

**Bit-exact** facts (asserted with `==`, no tolerance): `e0().inner(&e0()) ==
scalar(0.0)` (the degenerate metric forces the coefficient to exactly 0 — a
*signature* invariant, not a near-zero), and the defining blade products
`e1·e1 = 1`, `e1∧e2 = e12`, `e2∧e1 = −e12` (garust's integer-coefficient blades
make them exact). An `e0·e0` drifting to `1e-12` would be a signature bug, so it
is a hard `==` tripwire, never a soft pass.

## 3. Code outline

`crates/ufl-ga/src/lib.rs`: the `Mv` alias, the `basis` module (§2.2), the
`Motor`/`Point` re-exports (§2.3). `examples/hello_ga.rs` (a compiling
doc-example, CLAUDE.md §6): build a rotor, `sandwich(&e1)` to `e2`, then
`Point::new(1,0,0).transform(&rotor)` to `(0,1,0)` — the *spatial* `hello_*`,
completing the family; it shows the τ/4 quarter-turn (the same angle as the
EML core's Euler `i`) and the **native translator** no `G(3,0,0)` can. The
requirement's weight is the **acceptance test suite** (§6) — the code is a facade.

## 4. Non-goals

- Geometric s-expr **forms** + the **grade-type system** — R-0010.
- **Neuroevolution** (genotype/operators/fitness; the Strassen prize; evolving
  `R x R̃`) — R-0011.
- CGA, batch/SIMD apply, `Motor::to_matrix`, incidence (`Line`/`Plane`/join/meet)
  — pulled in when a later requirement needs them.
- A newtype `Mv` — deferred (§2.4).

## 5. Open questions — resolved

| R-0009 §5 question | Resolution |
|---|---|
| Binding shape | **Thin facade** (alias + named constructors + curated re-exports + tests); newtype deferred (§2.4). Insulation is the *pin*, not a wrapping type (§1). |
| Tolerance `ε` | **`1e-10`** with `cleaned(1e-10)`; `e0·e0` + defining blade products **bit-exact** (§2.5). |
| C9/C10 surface | **`Motor` + `Point`** (apply via `Point::transform`); incidence deferred. |
| Grade-lift `𝒢ₖ` | Construction primitives (`basis::*`, `grade(k)`) live **here**; the `𝒢ₖ` *form* is **R-0010**. |

## 6. Acceptance criteria

- [ ] **AC1 — Crate & pinned dependency.** `crates/ufl-ga` builds, depends on
  garust pinned at `v0.1.0` (git tag, locked rev), exposes `Mv` + `basis::*` +
  `Motor` + `Point`, and **does not depend on `ufl-core`** — asserted by
  `cargo tree -p ufl-ga -i ufl-core` exiting non-zero (package absent), not a
  stdout substring grep. The gate runs `--locked`.
- [ ] **AC2 — Construction & grades.** `basis::{scalar, e1, e2, e3, e0,
  pseudoscalar}` construct without panic; `grade(k)` projects; the five grades
  (0..4) are structurally distinguishable (a grade-1 element has zero grade-2
  part, etc.).
- [ ] **AC3 — The three products (signature facts).** `e1·e1 = 1`,
  `e1∧e2 = e12` and `e2∧e1 = −e12`, an orthogonal inner product `= 0`, and
  **`e0·e0 = 0`** (the PGA null property) — all asserted **bit-exact**. These
  pin the basis masks of §2.2.
- [ ] **AC4 — Versor sandwich (keystone).** A unit rotor about `e₁₂` by `τ/4`
  sandwiches the grade-1 basis element `e1` to `e2` within `ε` (after
  `cleaned`): `((e1()*e2())*(−τ/8)).exp().sandwich(&e1()) ≈ e2()` — the
  *algebraic* rotor check (observed error ~`2e-16`). Confirms garust's rotor
  half-angle/sign convention and the sandwich. The geometric analogue of
  R-0006's Strassen keystone.
- [ ] **AC5 — Motor on a point (rigid-body motion).** Via `Point::transform`:
  `Point::new(0,0,0).transform(&translator(1,2,3)).to_euclidean() ≈ (1,2,3)`
  (translation native via `e₀`); `Point::new(1,0,0).transform(&rotor(τ/4, e₁₂))
  .to_euclidean() ≈ (0,1,0)`; a composed `M₂ * M₁` applies `M₁` then `M₂`
  (order verified) — all within `ε`.
- [ ] **AC6 — Reverse / norm / totality.** A normalized rotor has `norm = 1`
  within `ε`; for a unit rotor `R`, `R * R.reverse() ≈ 1` (scalar) within `ε`;
  every `basis::*` constructor returns without panic (totality — the masks are
  compile-time-valid `< 16`).

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-12 | **Thin facade** (alias + named constructors + curated re-exports + tests), no newtype. | garust is pinned + contract-CI-guarded; a delegating newtype is the boilerplate-wrapper §2 warns against. Newtype deferred until a second backend / UFL invariant earns it. |
| 2026-06-12 | **Insulation is the pin, not the type.** `Mv` is a transparent alias and provides no API insulation; the spec states so and R-0009's "insulate" language is reframed (R-0009 decision log) to "a curated, pinned import surface." | A transparent alias gives zero insulation (hater finding); claiming both is incoherent. The pin (locked rev) is what holds churn. |
| 2026-06-12 | **Basis masks pinned in the spec**: `e1=1, e2=2, e3=4, e0=8, pseudoscalar=15`. | garust's `Cl(3,0,1)` puts the degenerate generator last (`e0 = basis(8)`, confirmed by garust's own `translator`). AC3 pins only `e1`/`e0`; the `e2`/`e3` masks must be written, not guessed. |
| 2026-06-12 | `ε = 1e-10` with `cleaned(1e-10)`; `e0·e0` + defining blade products bit-exact. | f64 geometry is a tolerance gate, but the degenerate metric and integer-coefficient blades are exact; garust leaks ~1e-16 dust that `cleaned` removes before comparison. |
| 2026-06-12 | Point motion uses **`Point::transform(&motor)`**, not `Motor::apply` on a `Point`; `sandwich` takes `&self, &Self`. | The real garust signatures (`Motor::apply` takes `&Pga`; typed points use `transform`); the AC snippets must compile. |
| 2026-06-12 | Commit `Cargo.lock`; `ufl-ga` gate runs `--locked`. | A git-tag dep is reproducible only if the locked rev is committed and `--locked` enforced — UFL's first git dependency. |
| 2026-06-12 | Surface `Motor` + `Point` only; defer `Line`/`Plane`/incidence. | AC4/AC5 need motions + points; incidence has no consumer until R-0010/R-0011 (and `Point` already pulls in `garust::pga`, so adding it later is a re-export). |

## Changelog

- 2026-06-12 — created (Draft).
- 2026-06-12 — three-lens applied (hater NEEDS WORK + architect REQUEST CHANGES,
  both verified the math against garust v0.1.0; nice-guy STRONG WORK): the
  insulation contradiction resolved (insulation = the pin, not the alias; R-0009
  reframed); basis masks pinned explicitly (`e0=8` etc.); API signatures
  corrected (`sandwich(&e1)`, `Point::transform`, `Motor::apply` takes `&Pga`);
  `--locked` + committed `Cargo.lock` mandated; `cleaned(1e-10)` for f64 dust;
  AC1 `cargo tree -i` assertion; AC2/AC4/AC5 distinguish algebraic-vs-geometric
  and PGA grade meanings; observed keystone error (~2e-16) cited.
