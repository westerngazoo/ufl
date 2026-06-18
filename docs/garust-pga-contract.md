# Capability contract — garust PGA kernel for UFL

**From:** UFL (Gustavo Delgadillo / westerngazoo) — the `ufl-ga` binding (R-0009).
**To:** the garust team.
**Status:** ✅ **Confirmed by garust** — pin tag `v0.1.0` (`main` @ `292bce5`). See §7.
**Verified against:** garust `5a31c44` (the `Cl(3,0,1)` PGA kernel — every item
below was located in the source at this commit; see the cited paths).

## 1. Why this exists

UFL is building a geometric-neuroevolution layer (UFL requirements R-0009 →
R-0011): real-valued **Projective Geometric Algebra** as the substrate for
evolving geometric programs (rotors, motors, grade-typed expressions) toward
targets such as inverse kinematics. UFL will depend on **garust** as its GA
kernel rather than rolling its own.

This document is the **contract**: the precise garust surface UFL's `ufl-ga`
crate will consume, so the garust team can (a) confirm each item is **supported,
semver-stable public API**, (b) fill any gap, and (c) tag a version UFL can pin.
**No new GA implementation is requested** — every capability already exists at
`5a31c44`. The ask is a *stability commitment* on this surface.

## 2. The required surface

UFL uses **real `f64`** PGA (the `Pga3` alias). Not `Complex` — UFL's analytic
core is separate; the geometric kernel is real, which is what rotors/motors and
the kinematics target need.

| # | Capability UFL needs | garust API (verified @ `5a31c44`) | Status |
|---|----------------------|-----------------------------------|--------|
| C1 | `Cl(3,0,1)` PGA multivector over `f64`, 16 blades | `pub type Pga3 = Multivector<Pga3Sig, f64>` (`garust-core/src/lib.rs:149`) | ✅ present |
| C2 | Geometric product | `impl Mul for Multivector<A, T: Ring>` (`garust-core/src/multivector.rs:208`) | ✅ |
| C3 | Outer (wedge) and inner products | `Multivector::wedge` / `::inner` (`garust-core/src/products.rs:55,86`) | ✅ |
| C4 | Grade projection ⟨·⟩ₖ | `Multivector::grade(k)` (`garust-core/src/products.rs:36`) | ✅ |
| C5 | Reverse, grade-involution, conjugate | `reverse` / `grade_involution` / `conjugate` (`garust-core/src/involutions.rs:37,49,61`) | ✅ |
| C6 | Versor sandwich `R x R̃` | `Multivector::sandwich(&self, x)` (`garust-core/src/transform.rs:92`) | ✅ |
| C7 | Exponential (rotor/motor from a bivector) | `Multivector::exp` (`garust-core/src/transform.rs:170`) | ✅ |
| C8 | Norm, norm², normalize | `norm` / `norm_squared` / `normalized` (`garust-core/src/involutions.rs:79,91,102`) | ✅ |
| C9 | Rigid-body **Motor** (rotor + translator) + composition + application | `Motor::{identity, translator, rotor, rotation_about}`, `impl Mul for Motor`, `Motor::apply(&Pga) -> Pga` (`garust-geo/src/motor.rs:63,173,189,208,240,85`) | ✅ |
| C10 | Typed PGA geometry + incidence | `pga::{Point, Line, Plane}` with `join` / `meet` (`garust-geo/src/pga.rs:146,163,172,196`) | ✅ |
| C11 | Real scalar with ordered ops (for norms / grade-typing) | `trait Real: Scalar<Magnitude=Self> + PartialOrd`, `f64: Real` (`garust-core/src/scalar.rs:96`) | ✅ |

## 3. Required semantics / invariants

These must hold for the pinned version (they do at `5a31c44`; please confirm
they are intended guarantees, not incidental):

1. **Signature `Cl(3,0,1)`** — basis `e1,e2,e3` square to `+1`; the ideal/null
   `e0` squares to `0` (the degenerate generator that makes translations and
   points-at-infinity native). 16 basis blades.
2. **Sandwich correctness** — for a unit rotor `R = exp(−θ/2·B)` (unit bivector
   `B`), `R.sandwich(v)` rotates a grade-1 `v` by angle `θ` in the `B`-plane;
   `Motor::apply` realizes the corresponding rigid-body motion on a `pga::Point`.
3. **Grade preservation** — `grade(k)` and the products behave per the standard
   GA grade algebra (UFL will build a grade-type system on top, R-0010).
4. **Determinism / purity** — the operations are pure functions of their inputs
   (no global state); `f64` results are reproducible on a platform.

## 4. Stability ask (the actual request)

1. **Treat C1–C11 as supported public API** under semver — UFL will pin a
   garust version and depend on exactly these items; breaking changes to them
   should be a major-version bump.
2. **Tag/publish a version UFL can pin** (a released `x.y.z` or a stable commit).
   UFL prefers a crates.io release if one is planned; a pinned git rev is fine
   otherwise. Please tell us the version string to depend on.
3. **No-std / features** — UFL only needs the default `std` build with `f64`.
   The optional `serde` / `simd` / `derive` features are not required by UFL.

## 5. Open items for the garust team

1. Confirm the version/rev UFL should pin (§4.2).
2. Confirm C1–C11 are intended **stable public API** (not internal), and the
   §3 semantics are guaranteed.
3. Flag any of C1–C11 you'd prefer UFL *not* depend on (e.g. an API you intend
   to reshape), so we can design `ufl-ga` around the stable subset.
4. **Optional, not blocking:** a documented "apply a motor to a batch of points"
   path already exists (`apply_each`); UFL may later want it — no action now.

## 6. What UFL builds on top (FYI, not a request)

For context, so you can see how the surface is used:

- **R-0009 (`ufl-ga`)** — a thin UFL-facing wrapper over C1–C11, validated by
  geometric-correctness tests (a rotor sandwich rotates a point by a known
  angle; a motor performs a known rigid-body motion).
- **R-0010** — geometric s-expression *forms* + a decidable grade-type system
  over C2–C5.
- **R-0011** — neuroevolution over geometric ASTs built from those forms; the
  fitness/accept step is verified exactly (UFL's predicate discharge), with the
  garust kernel doing the geometry.

## 7. garust team response — Confirmed

**Confirmed for garust `v0.1.0`** (git tag `v0.1.0`, `main` @ `292bce5`).

1. **Version to pin (§4.2, §5.1).** garust is not yet on crates.io; pin the git
   tag:
   ```toml
   [dependencies]
   garust = { git = "https://github.com/westerngazoo/garust", tag = "v0.1.0" }
   ```
   UFL needs only the default build (`std` + `f64`) — no extra features. A
   crates.io release will reuse the `0.1.0` version string when published (the
   ordered publish steps live in garust's `RELEASING.md`).

2. **C1–C11 are supported public API under semver (§4.1, §5.2).** A conformance
   test — `crates/garust-geo/tests/pga_contract.rs` — exercises every capability
   and asserts the §3 invariants on every CI run, so the surface is
   machine-guarded, not merely documented. A change that breaks it is a
   breaking change and warrants a major-version bump.

3. **§3 semantics are guaranteed** (intended, not incidental): the `Cl(3,0,1)`
   signature (`e1,e2,e3 → +1`, the ideal `e0 → 0`, 16 blades), sandwich / `exp`
   rotor- and motor-correctness, the standard grade algebra, and
   purity/determinism — each asserted by the conformance test.

4. **Nothing on C1–C11 is slated for reshaping (§5.3)** — depend on all of it.
   Additive changes since the `5a31c44` audit, none of which alter the pinned
   surface's signatures or semantics:
   - `Default` for `Motor` / `Conformal` (= identity);
   - `#[must_use]` on the typed value types (`Motor`, `Conformal`, and the
     `pga` / `cga` objects) — a lint only: ignoring a produced value now warns;
   - `Motor::to_matrix()` — new and additive: a column-major homogeneous 4×4
     for matrix-speed bulk transforms (not part of the contract, available if
     UFL wants the throughput bridge).

5. **Batch apply (§5.4)** — `Motor::apply_each` / `Conformal::apply_each` (and,
   behind the optional `simd` feature, `apply_each_simd`) are present whenever
   UFL wants the point-cloud path.

---

*Prepared from a direct source audit of garust at `5a31c44`; confirmed and
pinned at `v0.1.0` (`292bce5`). Everything UFL needs was already implemented —
this contract asked for confirmation and a stability commitment, now granted
and CI-enforced.*
