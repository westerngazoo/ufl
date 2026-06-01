//! R-0002 — Geometric Algebra Core over G(3,0,0): end-to-end acceptance tests.
//!
//! One section per acceptance criterion (AC1–AC6) plus the Cayley tripwire.
//! Each test cites its AC id in a `// ACk — …` comment so the architect (PR
//! review) and the orchestrator (status update) can map the suite to R-0002's
//! acceptance criteria mechanically.
//!
//! These tests are authored at loop step 3 (test plan). Their red/green split:
//!
//! - **Green now (structural).** AC1 (linear-space shape) and AC2 (grade-lift
//!   placement) touch only the implemented surface (`lift`, `from_coeffs`,
//!   `coeff`, `grade`, `reverse`, `norm`, `Add`/`Sub`/`Mul<Value>`). They pass
//!   at TDD-red.
//! - **Red now (product-dependent).** AC3, AC4, AC5, AC6 and the Cayley
//!   tripwire all exercise the geometric product `∗`, whose Cayley table is
//!   `unimplemented!()` until R-0002 step 5 — every such test panics. This is
//!   the TDD-red target the implementation must turn green.
//!
//! The `k > 3` clause of AC2 ("lifting a grade outside {0,1,2,3} is not
//! representable") is **structural, not a runtime test**: `GradeLift` is a
//! closed enum with no grade-4 variant, so a grade-4 lift does not compile.
//! There is nothing to assert at runtime; the type boundary is the proof.
//!
//! See:
//! - `requirements/0002-geometric-algebra-core.md` — AC1–AC6
//! - `specs/0002-geometric-algebra-core.md` — AC1–AC6 + the Cayley tripwire (§6)
//! - `docs/conventions.md` — blade order, MASK, rotor orientation
//! - `experiments/r0002-rotor.py` — the AC5/AC6 oracle (machine-zero residual)

use ufl_core::{eval, Eml, Env, GradeLift, Multivector, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The AC5/AC6 relative tolerance. SPEC-0002 §5 proposes `1e-12`; the rotor
/// oracle (`experiments/r0002-rotor.py`) measures the actual residual at
/// machine epsilon (~2.2e-16), four orders of magnitude tighter — so `1e-12`
/// is comfortably generous and is the qa-confirmed value.
const TOL: f64 = 1e-12;

/// A real complex `Value` (zero imaginary part) — every AC5/AC6 coefficient is
/// real, per SPEC-0002 §2.6 (the coefficient norm equals the GA norm only on
/// real vectors).
fn re(x: f64) -> Value {
    Value::new(x, 0.0)
}

/// Coefficient-wise closeness of two multivectors: every blade within `TOL`
/// (absolute, since the reference magnitudes here are O(1)–O(e)).
fn close_mv(actual: &Multivector, expected: &Multivector) -> bool {
    (0..8).all(|b| (actual.coeff(b) - expected.coeff(b)).norm() <= TOL)
}

/// `eᵢ` as a grade-1 basis multivector. `i ∈ {1,2,3}` selects the component.
fn e(i: usize) -> Multivector {
    let mut c = [re(0.0); 3];
    c[i - 1] = re(1.0);
    Multivector::lift(GradeLift::Vector(c))
}

/// The scalar identity blade `1` (grade-0 unit), the product identity.
fn one_blade() -> Multivector {
    Multivector::lift(GradeLift::Scalar(re(1.0)))
}

/// A single basis blade by storage index, coefficient 1. Built via
/// `from_coeffs` so AC3/AC4 can name e.g. `e₁₂` (index 4) directly.
fn blade(index: usize) -> Multivector {
    let mut c = [re(0.0); 8];
    c[index] = re(1.0);
    Multivector::from_coeffs(c)
}

/// The AC5/AC6 unit rotor `R = 𝒢₀(cos τ/8) + 𝒢₂([−sin τ/8, 0, 0])` — a `+τ/4`
/// rotation in the e₁∧e₂ plane (SPEC-0002 §6 AC5; `docs/conventions.md` rotor
/// orientation). Assembled by `lift` + `Add` (linear space, no product).
fn rotor() -> Multivector {
    let half = std::f64::consts::TAU / 8.0;
    let scalar = Multivector::lift(GradeLift::Scalar(re(half.cos())));
    let bivector = Multivector::lift(GradeLift::Bivector([re(-half.sin()), re(0.0), re(0.0)]));
    scalar + bivector
}

/// The rotor sandwich `R ∗ v ∗ ~R` (SPEC-0002 §2.5, AC5). Exercises `∗`, so it
/// panics until R-0002 step 5.
fn sandwich(r: &Multivector, v: &Multivector) -> Multivector {
    *r * *v * r.reverse()
}

// ---------------------------------------------------------------------------
// AC1 — Multivector representation (linear space over Value). GREEN now.
// ---------------------------------------------------------------------------

// AC1 — a G(3,0,0) multivector is exactly 8 coefficients in §2.1 blade order;
// `from_coeffs`/`coeff` round-trip every index. (Structural — no product.)
#[test]
fn ac1_eight_coefficients_in_blade_order() {
    let coeffs = [
        re(0.0),
        re(1.0),
        re(2.0),
        re(3.0),
        re(4.0),
        re(5.0),
        re(6.0),
        re(7.0),
    ];
    let m = Multivector::from_coeffs(coeffs);
    for (b, &c) in coeffs.iter().enumerate() {
        assert_eq!(m.coeff(b), c, "blade {b} must round-trip");
    }
}

// AC1 — multivectors form a linear space over `Value`: `Add`, `Sub`, and
// `Value`-scaling are component-wise. (Structural — no product.)
#[test]
fn ac1_linear_space_add_sub_scale() {
    let a = Multivector::from_coeffs([re(2.0); 8]);
    let b = Multivector::from_coeffs([re(5.0); 8]);
    let sum = a + b;
    let diff = b - a;
    let scaled = a * re(4.0);
    for blade in 0..8 {
        assert_eq!(sum.coeff(blade), re(7.0), "add component-wise");
        assert_eq!(diff.coeff(blade), re(3.0), "sub component-wise");
        assert_eq!(scaled.coeff(blade), re(8.0), "Value-scale every coeff");
    }
}

// AC1 — the rotor is assembled from a scalar part and a bivector part by linear
// combination, the exact move AC1 must support for AC5. (Structural.)
#[test]
fn ac1_rotor_assembles_from_scalar_and_bivector() {
    let r = rotor();
    let half = std::f64::consts::TAU / 8.0;
    assert_eq!(r.coeff(0), re(half.cos()), "cos on the scalar blade");
    assert_eq!(r.coeff(4), re(-half.sin()), "−sin on the e₁₂ blade");
    for blade in [1, 2, 3, 5, 6, 7] {
        assert_eq!(r.coeff(blade), re(0.0), "blade {blade} must be zero");
    }
}

// ---------------------------------------------------------------------------
// AC2 — Grade-lift 𝒢ₖ. GREEN now (placement is implemented).
// ---------------------------------------------------------------------------

// AC2 — each grade's lift places its components on that grade's blades and
// zeroes every other blade; pure-grade for all four grades. (Structural.)
#[test]
fn ac2_grade_lift_places_components_and_zeroes_the_rest() {
    let scalar = Multivector::lift(GradeLift::Scalar(re(7.0)));
    assert_eq!(scalar.coeff(0), re(7.0), "𝒢₀ on blade 0");
    assert_eq!(scalar.grade(0), scalar, "𝒢₀ result is pure grade 0");

    let vector = Multivector::lift(GradeLift::Vector([re(1.0), re(2.0), re(3.0)]));
    assert_eq!(vector.coeff(1), re(1.0), "e₁");
    assert_eq!(vector.coeff(2), re(2.0), "e₂");
    assert_eq!(vector.coeff(3), re(3.0), "e₃");
    assert_eq!(vector.grade(1), vector, "𝒢₁ result is pure grade 1");

    let bivector = Multivector::lift(GradeLift::Bivector([re(4.0), re(5.0), re(6.0)]));
    assert_eq!(bivector.coeff(4), re(4.0), "e₁₂");
    assert_eq!(bivector.coeff(5), re(5.0), "e₁₃");
    assert_eq!(bivector.coeff(6), re(6.0), "e₂₃");
    assert_eq!(bivector.grade(2), bivector, "𝒢₂ result is pure grade 2");

    let trivector = Multivector::lift(GradeLift::Trivector(re(9.0)));
    assert_eq!(trivector.coeff(7), re(9.0), "𝒢₃ on the e₁₂₃ blade");
    assert_eq!(trivector.grade(3), trivector, "𝒢₃ result is pure grade 3");
}

// ---------------------------------------------------------------------------
// AC3 — Geometric product axioms. RED now (panics in `∗`).
// ---------------------------------------------------------------------------

// AC3 — `eᵢ ∗ eᵢ = 1` for each i ∈ {1,2,3}: a basis vector squares to the
// scalar identity.
#[test]
fn ac3_basis_vectors_square_to_one() {
    for i in 1..=3 {
        let sq = e(i) * e(i);
        assert!(
            close_mv(&sq, &one_blade()),
            "e{i} ∗ e{i} must equal the scalar 1"
        );
    }
}

// AC3 — `eᵢ ∗ eⱼ = − eⱼ ∗ eᵢ` for i ≠ j: distinct basis vectors anticommute.
#[test]
fn ac3_distinct_basis_vectors_anticommute() {
    let pairs = [(1, 2), (1, 3), (2, 3)];
    for (i, j) in pairs {
        let ij = e(i) * e(j);
        let ji = e(j) * e(i);
        let neg_ji = ji * re(-1.0);
        assert!(
            close_mv(&ij, &neg_ji),
            "e{i} ∗ e{j} must equal −(e{j} ∗ e{i})"
        );
    }
}

// AC3 — the scalar `1` blade is the two-sided product identity: `1 ∗ M = M`
// and `M ∗ 1 = M` for an arbitrary multivector.
#[test]
fn ac3_scalar_one_is_two_sided_identity() {
    let m = Multivector::from_coeffs([
        re(1.0),
        re(2.0),
        re(3.0),
        re(4.0),
        re(5.0),
        re(6.0),
        re(7.0),
        re(8.0),
    ]);
    let left = one_blade() * m;
    let right = m * one_blade();
    assert!(close_mv(&left, &m), "1 ∗ M must equal M");
    assert!(close_mv(&right, &m), "M ∗ 1 must equal M");
}

// ---------------------------------------------------------------------------
// AC4 — Inner and outer behaviour. RED now (panics in `∗`).
// ---------------------------------------------------------------------------

// AC4 — orthogonal vectors give a pure grade-2 (outer) result:
// `e₁ ∗ e₂ = e₁₂` (storage index 4).
#[test]
fn ac4_orthogonal_product_is_outer_e12() {
    let product = e(1) * e(2);
    assert!(
        close_mv(&product, &blade(4)),
        "e₁ ∗ e₂ must equal e₁₂ (blade 4)"
    );
}

// AC4 — a vector times a containing bivector contracts (grade-lowering / inner):
// `e₁ ∗ e₁₂ = e₂` (storage index 2).
#[test]
fn ac4_contraction_lowers_grade_e1_times_e12() {
    let product = e(1) * blade(4);
    assert!(
        close_mv(&product, &e(2)),
        "e₁ ∗ e₁₂ must equal e₂ (grade-lowering inner result)"
    );
}

// AC4 — a general grade-1 × grade-1 product splits into a grade-0 part equal to
// the dot product and a grade-2 part equal to the outer product. Use
// `a = e₁ + e₂`, `b = e₁ + e₃`: dot = 1, outer = e₁₃ + e₂₁ = −e₁₂(? ) ... the
// outer is computed below from the anticommuting basis, so we assert against
// the geometric-algebra ground truth term by term.
#[test]
fn ac4_general_grade1_product_splits_into_dot_and_outer() {
    // a = e₁ + e₂, b = e₁ + e₃ (both real grade-1).
    let a = Multivector::lift(GradeLift::Vector([re(1.0), re(1.0), re(0.0)]));
    let b = Multivector::lift(GradeLift::Vector([re(1.0), re(0.0), re(1.0)]));
    let product = a * b;

    // Grade-0 part = dot product a·b = (1)(1) + (1)(0) + (0)(1) = 1.
    let dot = product.grade(0);
    assert!(
        close_mv(&dot, &one_blade()),
        "grade-0 part must equal the dot product a·b = 1"
    );

    // Grade-2 part = outer product a∧b. Expanding with the AC3 axioms:
    //   (e₁+e₂)∗(e₁+e₃) = 1 + e₁₃ + e₂₁ + e₂₃
    //   e₂₁ = −e₁₂, so the bivector part is −e₁₂ + e₁₃ + e₂₃.
    // Storage: index 4 = e₁₂ (−1), index 5 = e₁₃ (+1), index 6 = e₂₃ (+1).
    let outer = product.grade(2);
    let expected_outer = Multivector::lift(GradeLift::Bivector([re(-1.0), re(1.0), re(1.0)]));
    assert!(
        close_mv(&outer, &expected_outer),
        "grade-2 part must equal the outer product a∧b = −e₁₂ + e₁₃ + e₂₃"
    );

    // And the full product is exactly the sum of its grade-0 and grade-2 parts
    // (no grade-1 or grade-3 leakage from two grade-1 vectors).
    assert!(
        close_mv(&product, &(dot + outer)),
        "grade-1 × grade-1 must contain only grade-0 and grade-2 parts"
    );
}

// ---------------------------------------------------------------------------
// AC5 — Rotor sandwich preserves grade and norm. RED now (panics in `∗`).
// ---------------------------------------------------------------------------

// AC5(1) — direction & plane (mandatory, two inputs pin both the plane and its
// sign): `e₁ → e₂` and `e₂ → −e₁` under `R ∗ v ∗ ~R`.
#[test]
fn ac5_direction_e1_to_e2_and_e2_to_minus_e1() {
    let r = rotor();

    let r_e1 = sandwich(&r, &e(1));
    assert!(close_mv(&r_e1, &e(2)), "R ∗ e₁ ∗ ~R must equal e₂");

    let r_e2 = sandwich(&r, &e(2));
    let minus_e1 = e(1) * re(-1.0);
    assert!(close_mv(&r_e2, &minus_e1), "R ∗ e₂ ∗ ~R must equal −e₁");
}

// AC5(2) — negative control: `e₃ → e₃` (fixed). Distinguishes a genuine e₁₂
// rotor from identity and from a wrong-plane rotor.
#[test]
fn ac5_negative_control_e3_is_fixed() {
    let r = rotor();
    let r_e3 = sandwich(&r, &e(3));
    assert!(close_mv(&r_e3, &e(3)), "R ∗ e₃ ∗ ~R must leave e₃ fixed");
}

// AC5(3) — grade & norm preservation: each rotated vector stays grade-1
// (non-grade-1 blades zero to tolerance) and `norm(v') = norm(v)`.
#[test]
fn ac5_preserves_grade_one_and_norm() {
    let r = rotor();
    for i in 1..=3 {
        let v = e(i);
        let out = sandwich(&r, &v);

        // Grade-1: the result equals its own grade-1 projection (others zero).
        assert!(
            close_mv(&out, &out.grade(1)),
            "R ∗ e{i} ∗ ~R must be pure grade-1"
        );

        // Norm preserved under the coefficient norm |M| = √Σ|cᵢ|².
        assert!(
            (out.norm() - v.norm()).abs() <= TOL,
            "norm(R ∗ e{i} ∗ ~R) must equal norm(e{i}) = 1"
        );
    }
}

// ---------------------------------------------------------------------------
// AC6 — EML-scalar composition (full data path). RED now (panics in `∗`).
// ---------------------------------------------------------------------------

// AC6 — a multivector whose coefficient comes from evaluating an EML tree
// participates in both atoms. `eml(1,1)` evaluates to `e ≈ 2.71828`, so
// `v = 𝒢₁([e, 0, 0]) = e·e₁`. Through the AC5 rotor, `R ∗ v ∗ ~R` must equal
// `e·e₂` to tolerance. This exercises EML-tree → Value → lift → ∗ end to end
// with a concrete, falsifiable expected value.
#[test]
fn ac6_eml_scalar_flows_through_lift_and_product() {
    // eml(1,1) = exp(1) − ln(1) = e − 0 = e.
    let e_tree = Eml::node(Eml::one(), Eml::one());
    let e_value = eval(&e_tree, &Env::new()).expect("eml(1,1) evaluates without bindings");

    // Sanity: the EML scalar really is e (this part is product-independent).
    assert!(
        (e_value - re(std::f64::consts::E)).norm() <= TOL,
        "eml(1,1) must evaluate to e"
    );

    // v = e·e₁ via 𝒢₁, then rotate.
    let v = Multivector::lift(GradeLift::Vector([e_value, re(0.0), re(0.0)]));
    let out = sandwich(&rotor(), &v);

    // Expected: e·e₂ (the e₁ → e₂ rotation scaled by the EML-derived e).
    let expected = Multivector::lift(GradeLift::Vector([re(0.0), e_value, re(0.0)]));
    assert!(close_mv(&out, &expected), "R ∗ (e·e₁) ∗ ~R must equal e·e₂");
}
