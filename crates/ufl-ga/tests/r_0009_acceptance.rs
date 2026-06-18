//! R-0009 acceptance suite — `Cl(3,0,1)` PGA kernel binding (`ufl-ga`).
//!
//! Derived from [SPEC-0009 §6](../../../specs/0009-pga-kernel-binding.md) — one
//! or more tests per acceptance criterion, each citing its `ACn` id. The tests
//! are the requirement's weight (the crate itself is a thin facade): they
//! validate garust's `Cl(3,0,1)` kernel *through UFL's own surface*
//! (`ufl_ga::{Mv, basis::*, Motor, Point}`).
//!
//! # TDD status (loop step 3, RED)
//!
//! Every assertion routes through a `ufl_ga::basis::*` constructor, and those
//! constructors are `unimplemented!()` until R-0009 step 5. So **every test in
//! this file panics today** — the failing-test-first state CLAUDE.md §4/§5
//! requires. Step 5 implements the six pinned blade masks (SPEC-0009 §2.2);
//! these tests turn GREEN with no edit.
//!
//! # AC1 is *not* here
//!
//! "`ufl-ga` does not depend on `ufl-core`" (AC1) is a build-graph fact, proved
//! by `cargo tree -p ufl-ga -i ufl-core` exiting **non-zero** (the package is
//! absent), per SPEC-0009 §6 AC1 — a CI/command gate, not a runtime unit test.
//! It is deliberately omitted as a `#[test]`.
//!
//! # Conventions
//!
//! - `τ = std::f64::consts::TAU` (UFL's circle constant — `docs/conventions.md`).
//! - **ε = 1e-10** (SPEC-0009 §2.5). Floating compares use [`approx_mv`] /
//!   [`approx_xyz`], which follow the spec's *`cleaned`-then-compare* pattern.
//! - **Bit-exact** facts (AC3, signature invariants) use `==` with no tolerance.

use std::f64::consts::TAU;

use ufl_ga::{basis, Motor, Mv, Point};

/// SPEC-0009 §2.5 — the floating tolerance for AC4–AC6.
const EPS: f64 = 1e-10;

/// Floating multivector equality, the way SPEC-0009 §2.5 mandates: subtract and
/// `cleaned(ε)`. Any blade whose coefficients differ by `≥ ε` survives the
/// clean and trips the `==` against `zero`; garust's `~1e-16` symmetry dust is
/// scrubbed. (Spec: "compare per coefficient within ε" — done here in one shot
/// via the difference.)
fn approx_mv(got: &Mv, want: &Mv) {
    let residual = (*got - *want).cleaned(EPS);
    assert_eq!(
        residual,
        Mv::zero(),
        "multivectors differ by more than ε={EPS}: got {got:?}, want {want:?}",
    );
}

/// Euclidean-coordinate compare, each component within ε (SPEC-0009 §6 AC5).
fn approx_xyz(got: (f64, f64, f64), want: (f64, f64, f64)) {
    let (x, y, z) = got;
    let (a, b, c) = want;
    assert!(
        (x - a).abs() < EPS && (y - b).abs() < EPS && (z - c).abs() < EPS,
        "coords differ by more than ε={EPS}: got ({x}, {y}, {z}), want ({a}, {b}, {c})",
    );
}

// ===========================================================================
// AC2 — Construction & grades.
// `basis::{scalar, e1, e2, e3, e0, pseudoscalar}` construct without panic;
// `grade(k)` projects; the five grades (0..=4) are structurally
// distinguishable.
// ===========================================================================

/// AC2 — every named constructor builds a value (no panic at the facade).
#[test]
fn ac2_basis_constructors_build() {
    let _ = basis::scalar(1.0);
    let _ = basis::e1();
    let _ = basis::e2();
    let _ = basis::e3();
    let _ = basis::e0();
    let _ = basis::pseudoscalar();
}

/// AC2 — `grade(k)` is a projector: a pure grade-1 element is unchanged by
/// `grade(1)` and annihilated by every other grade.
#[test]
fn ac2_grade_projection_selects_the_right_grade() {
    let e1 = basis::e1();
    assert_eq!(e1.grade(1), e1, "grade(1) must fix a pure grade-1 element");
    assert_eq!(e1.grade(0), Mv::zero(), "a vector has no scalar part");
    assert_eq!(e1.grade(2), Mv::zero(), "a vector has no bivector part");
    assert_eq!(e1.grade(3), Mv::zero(), "a vector has no trivector part");
    assert_eq!(e1.grade(4), Mv::zero(), "a vector has no grade-4 part");
}

/// AC2 — the five grades are structurally distinguishable: a scalar lives in
/// grade 0, a vector in grade 1, a bivector in grade 2, a trivector in grade 3,
/// and the pseudoscalar in grade 4 — each pure in *its* grade.
#[test]
fn ac2_the_five_grades_are_distinguishable() {
    let scalar = basis::scalar(1.0);
    let vector = basis::e1();
    let bivector = basis::e1() * basis::e2(); // e12, grade 2
    let trivector = basis::e1() * basis::e2() * basis::e3(); // e123, grade 3
    let pseudo = basis::pseudoscalar(); // grade 4

    assert_eq!(scalar.grade(0), scalar, "scalar is pure grade 0");
    assert_eq!(vector.grade(1), vector, "e1 is pure grade 1");
    assert_eq!(bivector.grade(2), bivector, "e12 is pure grade 2");
    assert_eq!(trivector.grade(3), trivector, "e123 is pure grade 3");
    assert_eq!(pseudo.grade(4), pseudo, "pseudoscalar is pure grade 4");

    // Distinguishable: no two of them share a grade (cross-grade parts vanish).
    assert_eq!(scalar.grade(1), Mv::zero());
    assert_eq!(vector.grade(2), Mv::zero());
    assert_eq!(bivector.grade(3), Mv::zero());
    assert_eq!(trivector.grade(4), Mv::zero());
    assert_eq!(pseudo.grade(0), Mv::zero());
}

// ===========================================================================
// AC3 — The three products (signature facts), BIT-EXACT.
// e1·e1 = 1, e1∧e2 = e12, e2∧e1 = −e12, an orthogonal inner product = 0, and
// the PGA null property e0·e0 = 0 — all `==`, no tolerance. These pin the
// basis masks of SPEC-0009 §2.2.
// ===========================================================================

/// AC3 — geometric product `e1 * e1 = 1` (bit-exact; `e1² = +1`).
#[test]
fn ac3_e1_squared_is_scalar_one_exact() {
    assert_eq!(
        basis::e1() * basis::e1(),
        basis::scalar(1.0),
        "e1*e1 must be exactly the scalar 1",
    );
}

/// AC3 — the wedge is antisymmetric and bit-exact: `e1∧e2 = e12`,
/// `e2∧e1 = −e12`.
#[test]
fn ac3_wedge_is_antisymmetric_exact() {
    let e12 = basis::e1() * basis::e2(); // for orthogonal e1,e2 the geometric
                                         // product is the wedge (grade 2)
    assert_eq!(basis::e1().wedge(&basis::e2()), e12, "e1∧e2 must equal e12");
    assert_eq!(
        basis::e2().wedge(&basis::e1()),
        Mv::zero() - e12,
        "e2∧e1 must equal −e12",
    );
}

/// AC3 — orthogonal vectors have inner product exactly 0 (`e1·e2 = 0`).
#[test]
fn ac3_orthogonal_inner_product_is_zero_exact() {
    assert_eq!(
        basis::e1().inner(&basis::e2()),
        Mv::zero(),
        "orthogonal e1·e2 must be exactly 0",
    );
}

/// AC3 — **the PGA null property**: `e0·e0 = 0`, bit-exact. The degenerate
/// metric forces the coefficient to *exactly* zero — a signature invariant, not
/// a near-zero. This is the fact that distinguishes `Cl(3,0,1)` from `G(3,0,0)`
/// and is a hard `==` tripwire (SPEC-0009 §2.5).
#[test]
fn ac3_e0_null_property_is_exact_zero() {
    assert_eq!(
        basis::e0().inner(&basis::e0()),
        basis::scalar(0.0),
        "e0·e0 must be exactly the scalar 0 — the PGA degenerate metric",
    );
}

// ===========================================================================
// AC4 — Versor sandwich (the keystone).
// A unit rotor about e12 by τ/4 sandwiches e1 → e2 within ε (after cleaned),
// PLUS the convention-equivalence between the raw-exp versor and Motor::rotor.
// ===========================================================================

/// AC4 — the keystone: `((e1*e2)*(−τ/8)).exp().sandwich(&e1) ≈ e2`. A τ/4
/// rotation in the e12 plane sends e1 to e2 (observed error ~2e-16, eight
/// orders under ε). The geometric analogue of R-0006's Strassen keystone.
#[test]
fn ac4_rotor_sandwich_sends_e1_to_e2() {
    let plane = basis::e1() * basis::e2(); // unit bivector, plane² = −1
    let rotor = (plane * (-TAU / 8.0)).exp(); // exp(−½·(τ/4)·e12)
    let rotated = rotor.sandwich(&basis::e1());
    approx_mv(&rotated, &basis::e2());
}

/// AC4 — **convention-equivalence**: the hand-built versor `exp(−τ/8·e12)`
/// equals `Motor::rotor(τ/4, e12)`'s versor within ε, so AC4's raw `exp` and
/// AC5's `Motor::rotor` cannot silently disagree on the half-angle/sign
/// convention (SPEC-0009 §6 AC4).
#[test]
fn ac4_handbuilt_versor_matches_motor_rotor() {
    let plane = basis::e1() * basis::e2();
    let handbuilt = (plane * (-TAU / 8.0)).exp();
    let motor_versor = Motor::rotor(TAU / 4.0, basis::e1() * basis::e2()).versor();
    approx_mv(&handbuilt, &motor_versor);
}

// ===========================================================================
// AC5 — Motor on a point (rigid-body motion).
// Via Point::transform: a native translator moves a point by the exact offset;
// a rotor rotates it; a composed M2*M1 applies M1 then M2 — all within ε.
// ===========================================================================

/// AC5 — a pure **translator** moves a point by the exact offset; translations
/// are native via the null generator `e0` (no `G(3,0,0)` can do this).
#[test]
fn ac5_translator_moves_point_by_offset() {
    let moved = Point::new(0.0, 0.0, 0.0)
        .transform(&Motor::translator(1.0, 2.0, 3.0))
        .to_euclidean();
    approx_xyz(moved, (1.0, 2.0, 3.0));
}

/// AC5 — a **rotor** about the e12 plane by τ/4 rotates `(1,0,0)` to `(0,1,0)`.
#[test]
fn ac5_rotor_rotates_point() {
    let rotor = Motor::rotor(TAU / 4.0, basis::e1() * basis::e2());
    let moved = Point::new(1.0, 0.0, 0.0).transform(&rotor).to_euclidean();
    approx_xyz(moved, (0.0, 1.0, 0.0));
}

/// AC5 — **composition order**: `M2 * M1` applies `M1` first, then `M2`. Verify
/// with a known translator + rotor whose two orders give *different* results,
/// so the test actually pins the order (not a commuting pair).
///
/// `M1` = translate `(1,0,0)`; `M2` = rotor τ/4 in e12. Starting at the origin:
/// - `M2 * M1`: translate origin → `(1,0,0)`, then rotate → `(0,1,0)`.
/// - `M1 * M2`: rotate origin → `(0,0,0)`, then translate → `(1,0,0)`.
///
/// Distinct ⇒ the assertion is load-bearing on the order.
#[test]
fn ac5_composition_applies_m1_then_m2() {
    let m1 = Motor::translator(1.0, 0.0, 0.0);
    let m2 = Motor::rotor(TAU / 4.0, basis::e1() * basis::e2());

    let m2_then = (m2 * m1).transform_origin();
    approx_xyz(m2_then, (0.0, 1.0, 0.0));

    let m1_then = (m1 * m2).transform_origin();
    approx_xyz(m1_then, (1.0, 0.0, 0.0));
}

/// Local helper: apply a motor to the origin point and read its coordinates —
/// keeps the AC5 composition test readable.
trait TransformOrigin {
    fn transform_origin(self) -> (f64, f64, f64);
}

impl TransformOrigin for Motor<f64> {
    fn transform_origin(self) -> (f64, f64, f64) {
        Point::new(0.0, 0.0, 0.0).transform(&self).to_euclidean()
    }
}

// ===========================================================================
// AC6 — Reverse / norm / totality.
// A normalized rotor has norm ≈ 1; for a unit rotor R, R * R.reverse() ≈ 1
// (scalar); every basis::* constructor returns without panic.
// ===========================================================================

/// AC6 — a normalized rotor has norm ≈ 1.
#[test]
fn ac6_normalized_rotor_has_unit_norm() {
    let rotor = (basis::e1() * basis::e2() * (-TAU / 8.0)).exp();
    let unit = rotor.normalized();
    assert!(
        (unit.norm() - 1.0).abs() < EPS,
        "normalized rotor norm must be ≈ 1, got {}",
        unit.norm(),
    );
}

/// AC6 — for a unit rotor `R`, `reverse` is its inverse: `R * R̃ ≈ 1` (scalar).
#[test]
fn ac6_unit_rotor_reverse_is_its_inverse() {
    let rotor = (basis::e1() * basis::e2() * (-TAU / 8.0))
        .exp()
        .normalized();
    let product = rotor * rotor.reverse();
    approx_mv(&product, &basis::scalar(1.0));
}

/// AC6 — **totality**: every `basis::*` constructor returns without panic (the
/// masks are compile-time-valid `< 16`). The bare calls below complete only if
/// none of the six constructors panics.
#[test]
fn ac6_all_basis_constructors_are_total() {
    let constructed = [
        basis::scalar(2.5),
        basis::e1(),
        basis::e2(),
        basis::e3(),
        basis::e0(),
        basis::pseudoscalar(),
    ];
    assert_eq!(
        constructed.len(),
        6,
        "all six constructors returned a value"
    );
}
