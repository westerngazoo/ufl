//! The grade-lift atom `𝒢ₖ` (SPEC-0002 §2.3).

use crate::eval::Value;

use super::Multivector;

/// Input to the grade-lift atom `𝒢ₖ`. Each variant carries exactly the number
/// of components of its grade — the binomial row `C(3,k) = 1, 3, 3, 1`. Grades
/// outside `{0,1,2,3}` are unrepresentable (R-0002 AC2), so arity and the
/// `k ≤ 3` bound are structural — the SPEC-0001 "type admits exactly valid
/// inputs" discipline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradeLift {
    /// `𝒢₀` — grade 0: the `1` blade.
    Scalar(Value),
    /// `𝒢₁` — grade 1: `[e₁, e₂, e₃]`.
    Vector([Value; 3]),
    /// `𝒢₂` — grade 2: `[e₁₂, e₁₃, e₂₃]`.
    Bivector([Value; 3]),
    /// `𝒢₃` — grade 3: the `e₁₂₃` pseudoscalar.
    Trivector(Value),
}

impl Multivector {
    /// The grade-lift atom `𝒢ₖ`: place a grade's components on its blades,
    /// zeroing every other blade (SPEC-0002 §2.3).
    pub fn lift(input: GradeLift) -> Self {
        let mut m = Multivector::zero();
        match input {
            GradeLift::Scalar(s) => m.coeffs[0] = s,
            GradeLift::Vector(v) => m.coeffs[1..4].copy_from_slice(&v),
            GradeLift::Bivector(b) => m.coeffs[4..7].copy_from_slice(&b),
            GradeLift::Trivector(t) => m.coeffs[7] = t,
        }
        m
    }
}

#[cfg(test)]
mod tests {
    //! Structural unit tests for the grade-lift atom `𝒢ₖ` (AC2). `lift` is
    //! implemented, so these are green at TDD-red — they pin where each grade's
    //! components land and that all other blades are zeroed. The `k ≤ 3` bound
    //! is enforced by the closed `GradeLift` enum (a grade-4 lift does not
    //! compile), so AC2's "not representable" clause is structural, not a
    //! runtime test — see the module-level note in `tests/r_0002_acceptance.rs`.

    use super::*;

    fn v(re: f64) -> Value {
        Value::new(re, 0.0)
    }

    /// Assert exactly `blade` carries `expected`, every other blade is zero.
    fn assert_only(m: &Multivector, blade: usize, expected: Value) {
        for b in 0..8 {
            let want = if b == blade { expected } else { v(0.0) };
            assert_eq!(m.coeff(b), want, "blade {b}");
        }
    }

    // AC2 — 𝒢₀ places its single component on the `1` blade (index 0) only.
    #[test]
    fn ac2_scalar_lift_lands_on_blade_zero() {
        let m = Multivector::lift(GradeLift::Scalar(v(7.0)));
        assert_only(&m, 0, v(7.0));
    }

    // AC2 — 𝒢₁ places its three components on e₁,e₂,e₃ (indices 1,2,3) in
    // blade order, zeroing all others.
    #[test]
    fn ac2_vector_lift_lands_on_grade_one_blades() {
        let m = Multivector::lift(GradeLift::Vector([v(1.0), v(2.0), v(3.0)]));
        assert_eq!(m.coeff(0), v(0.0));
        assert_eq!(m.coeff(1), v(1.0), "e₁");
        assert_eq!(m.coeff(2), v(2.0), "e₂");
        assert_eq!(m.coeff(3), v(3.0), "e₃");
        for b in 4..8 {
            assert_eq!(m.coeff(b), v(0.0), "blade {b} must be zero");
        }
    }

    // AC2 — 𝒢₂ places its three components on e₁₂,e₁₃,e₂₃ (indices 4,5,6) in
    // blade order, zeroing all others.
    #[test]
    fn ac2_bivector_lift_lands_on_grade_two_blades() {
        let m = Multivector::lift(GradeLift::Bivector([v(4.0), v(5.0), v(6.0)]));
        for b in 0..4 {
            assert_eq!(m.coeff(b), v(0.0), "blade {b} must be zero");
        }
        assert_eq!(m.coeff(4), v(4.0), "e₁₂");
        assert_eq!(m.coeff(5), v(5.0), "e₁₃");
        assert_eq!(m.coeff(6), v(6.0), "e₂₃");
        assert_eq!(m.coeff(7), v(0.0));
    }

    // AC2 — 𝒢₃ places its single component on the e₁₂₃ pseudoscalar (index 7).
    #[test]
    fn ac2_trivector_lift_lands_on_blade_seven() {
        let m = Multivector::lift(GradeLift::Trivector(v(9.0)));
        assert_only(&m, 7, v(9.0));
    }

    // AC2 — each lift result is pure: its grade-k projection recovers it, and
    // every other grade projects to zero. This pins "zeroing every other
    // blade" at the grade level.
    #[test]
    fn ac2_lift_is_pure_grade() {
        let cases = [
            (0u8, Multivector::lift(GradeLift::Scalar(v(7.0)))),
            (
                1,
                Multivector::lift(GradeLift::Vector([v(1.0), v(2.0), v(3.0)])),
            ),
            (
                2,
                Multivector::lift(GradeLift::Bivector([v(4.0), v(5.0), v(6.0)])),
            ),
            (3, Multivector::lift(GradeLift::Trivector(v(9.0)))),
        ];
        for (k, m) in cases {
            assert_eq!(m.grade(k), m, "𝒢ₖ result must equal its own grade-{k} part");
            for other in 0..=3u8 {
                if other != k {
                    assert_eq!(
                        m.grade(other),
                        Multivector::zero(),
                        "grade {other} of a 𝒢{k} lift must be zero"
                    );
                }
            }
        }
    }
}
