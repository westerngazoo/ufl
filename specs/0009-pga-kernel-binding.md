# SPEC-0009 — `Cl(3,0,1)` PGA Kernel Binding (`ufl-ga`)

- **Status:** Draft
- **Realizes:** R-0009
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-12
- **Depends on:** garust `v0.1.0` (the confirmed contract C1–C11)
- **Crate:** `crates/ufl-ga` (new — a substrate leaf; depends only on garust)

## 1. Motivation

SPEC-0009 realizes [R-0009](../requirements/0009-pga-kernel-binding.md): bind
garust's `Cl(3,0,1)` PGA kernel into UFL as `ufl-ga`, a **thin facade** that
gives the geometric forms (R-0010) and neuroevolution (R-0011) a curated,
UFL-named, UFL-validated surface — without re-implementing geometric algebra.

The binding is deliberately thin (CLAUDE.md §2 — no premature abstraction):
garust is pinned at a contract-guaranteed `v0.1.0`, so the churn the requirement
worried about is held by the *pin*, not by a heavy wrapper. The facade's earned
value is three concrete things, none of which is reimplementation:

1. **UFL-named basis constructors** that hide garust's blade-bitmask convention
   (`Pga3::basis(0b…)`) behind meaningful names (`e0/e1/e2/e3`, `scalar`,
   `pseudoscalar`) — the construction primitives R-0010's `𝒢ₖ` form needs;
2. a **single curated import surface** (`ufl_ga::…`) R-0010/R-0011 target instead
   of reaching into garust directly; and
3. **UFL-owned geometric-correctness tests** — UFL validates the kernel through
   its own surface, so the substrate it builds on is proven here.

## 2. Design

### 2.1 The crate & the pinned dependency

`crates/ufl-ga/Cargo.toml`:

```toml
[package]
name = "ufl-ga"
# … workspace edition/license/authors/repo …
[dependencies]
garust = { git = "https://github.com/westerngazoo/garust", tag = "v0.1.0" }
```

`#![forbid(unsafe_code)]`. Depends only on garust — **not `ufl-core`** (the
real-`f64` PGA substrate is separate from the `Complex` EML core; AC1).

### 2.2 The multivector surface — alias + named basis constructors

```rust
/// UFL's geometric value: a Cl(3,0,1) PGA multivector over f64. A transparent
/// alias for garust's kernel type — its inherent methods (wedge, inner, grade,
/// reverse, sandwich, norm, …) and the geometric product `*` are available
/// directly, so R-0010 targets `ufl_ga::Mv` without importing garust.
pub type Mv = garust::Pga3;

/// UFL-named basis constructors — hide garust's blade-bitmask convention.
/// The exact masks are pinned by the AC3 identities (e1·e1 = 1, e0·e0 = 0).
pub mod basis {
    use super::Mv;
    pub fn scalar(s: f64) -> Mv { Mv::scalar(s) }
    pub fn e0() -> Mv { /* the ideal/null generator, e0² = 0 */ }
    pub fn e1() -> Mv { /* … via Mv::basis(<mask>) */ }
    pub fn e2() -> Mv { /* … */ }
    pub fn e3() -> Mv { /* … */ }
    pub fn pseudoscalar() -> Mv { /* the grade-4 element */ }
}
```

The **operations come free via the alias** (garust's inherent methods on
`Multivector`): geometric product `a * b`, `a.wedge(&b)`, `a.inner(&b)`,
`a.grade(k)`, `a.reverse()`, `a.sandwich(&x)`, `a.norm()`, `a.normalized()`.
SPEC-0009 adds **no wrapper functions** over these — re-naming them would be the
decorative wrapping §1 rejects. R-0010 documents which it lifts into forms.

### 2.3 Versors — rotor, sandwich, Motor, Point

Curated re-exports for the rigid-body surface (the only types UFL re-exports by
name, because R-0010/R-0011 build motions and act on points):

```rust
pub use garust::Motor;        // rotor / translator / identity / rotation_about / Mul / apply
pub use garust::pga::Point;   // Point::new(x, y, z) — the typed target of Motor::apply
```

A rotor is `garust::Motor::rotor(radians, plane)` = `exp(−½·radians·plane)` for a
unit bivector `plane` (e.g. `e1()*e2()` for the `e₁₂` plane). The raw algebraic
sandwich is `Mv::sandwich` (a rotor `R` acting `R x R̃`).

`Line`/`Plane`/`join`/`meet` (contract C10) are **not** surfaced here — incidence
arrives when R-0010/R-0011 need it (no premature surface).

### 2.4 Binding shape — thin facade, newtype deferred (the §5 resolution)

The facade is an **alias + named constructors + curated re-exports + tests** —
*not* a newtype that re-wraps every operation. A newtype (`struct Mv(Pga3)`)
would insulate UFL from a garust API change, but: (a) garust is pinned and
contract-CI-guarded, so the churn risk is already controlled; and (b) a newtype
costs a delegating method for every operation — exactly the boilerplate-wrapper
CLAUDE.md §2 warns against, with no consumer today that needs the opacity. The
newtype is the **deferred escalation**: introduce it only if UFL gains a second
GA backend or a UFL-specific invariant the raw kernel can't enforce. (The same
"earned by a second instance" discipline as SPEC-0007's trait and SPEC-0008's
proposer seam.)

### 2.5 Tolerance & exactness

Assertions are to `ε = 1e-10` (unit-scale `f64` geometry — rotations of
unit elements, norms, point coordinates). Some facts are **bit-exact** and
asserted as such: `e0().inner(&e0()) == scalar(0.0)` (the degenerate metric →
coefficient exactly 0), and the defining blade products (`e1·e1 = 1`,
`e1∧e2 = e12`) where garust's integer-coefficient construction makes them exact.
SPEC fixes `ε = 1e-10`; the spec notes which assertions are exact vs toleranced.

## 3. Code outline

`crates/ufl-ga/src/lib.rs`: the `Mv` alias, the `basis` module (§2.2), the
`Motor`/`Point` re-exports (§2.3). `examples/hello_ga.rs`: build a rotor, sandwich
`e1` to `e2`, move a `Point` with a motor — the geometric `hello_*`. The bulk of
the requirement's weight is the **acceptance test suite** (§6), since the code is
a facade.

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
| Binding shape | **Thin facade** (alias + named constructors + curated re-exports + tests); newtype deferred (§2.4). |
| Tolerance `ε` | **`1e-10`**; `e0·e0` and the defining blade products asserted **bit-exact** (§2.5). |
| C9/C10 surface | **`Motor` + `Point`** re-exported (AC4/AC5); `Line`/`Plane`/incidence deferred. |
| Grade-lift `𝒢ₖ` | Construction primitives (`basis::*`, `grade(k)`) live **here**; the `𝒢ₖ` *form* is **R-0010**. |

## 6. Acceptance criteria

- [ ] **AC1 — Crate & pinned dependency.** `crates/ufl-ga` builds, depends on
  garust pinned at `v0.1.0` (git tag), exposes `Mv` + `basis::*` + `Motor` +
  `Point`, and **does not depend on `ufl-core`** (a `cargo tree -p ufl-ga` test
  asserts the absence).
- [ ] **AC2 — Construction & grades.** `basis::{scalar, e0..e3, pseudoscalar}`
  construct; `grade(k)` projects; the five grades (0..4) are distinguishable
  (a grade-1 has zero grade-2 part, etc.).
- [ ] **AC3 — The three products.** Geometric product, `wedge`, `inner` satisfy
  the defining facts: `e1·e1 = 1`, `e1∧e2 = e12` (and `e2∧e1 = −e12`), orthogonal
  inner product `= 0`, and **`e0·e0 = 0`** (the PGA null property) — the
  bit-exact ones asserted exactly.
- [ ] **AC4 — Versor sandwich (keystone).** A unit rotor about `e₁₂` by `τ/4`
  sandwiches `e1` to `e2` within `ε`: `((e1*e2)*(−τ/8)).exp().sandwich(e1) ≈ e2`
  (or the `Motor::rotor(τ/4, e1*e2)` versor) — confirming garust's rotor
  convention and the sandwich. The geometric analogue of R-0006's Strassen
  keystone.
- [ ] **AC5 — Motor (rigid-body motion).** On a `Point`: a `translator(dx,dy,dz)`
  moves `Point::new(0,0,0)` to `(dx,dy,dz)` within `ε` (native via `e0`); a
  `rotor(τ/4, e₁₂)` rotates `Point::new(1,0,0)` to `≈(0,1,0)`; a composed motor
  `M₂ * M₁` applies `M₁` then `M₂` (order verified) — all within `ε`.
- [ ] **AC6 — Reverse / norm.** A normalized rotor has `norm = 1` within `ε`;
  for a unit rotor `R`, `R * R.reverse() ≈ 1` (scalar 1) within `ε`; the facade
  returns values on its supported domain (no panics).

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-12 | **Thin facade** (alias + named constructors + curated re-exports + tests), no newtype. | garust is pinned + contract-CI-guarded, so churn is held by the pin; a delegating newtype is the boilerplate-wrapper §2 warns against. Newtype deferred until a second backend / UFL invariant earns it. |
| 2026-06-12 | `Mv = garust::Pga3` transparent alias; ops via garust's inherent methods (no re-naming wrappers). | Re-naming `wedge`/`inner`/… would be decorative; the alias gives R-0010 a UFL import surface for free. |
| 2026-06-12 | UFL-named `basis::{e0..e3, scalar, pseudoscalar}` over `Pga3::basis(mask)`. | The one genuine construction value-add — hides garust's bitmask convention behind UFL-meaningful names; the masks are pinned by AC3's identities. |
| 2026-06-12 | `ε = 1e-10`; `e0·e0` + defining blade products bit-exact. | f64 geometry is a tolerance gate (like the Complex EML core), but the degenerate metric and integer-coefficient blade products are exact and asserted so. |
| 2026-06-12 | Surface `Motor` + `Point` only; defer `Line`/`Plane`/incidence. | AC4/AC5 need motions + points; incidence has no consumer until R-0010/R-0011. |

## Changelog

- 2026-06-12 — created (Draft).
