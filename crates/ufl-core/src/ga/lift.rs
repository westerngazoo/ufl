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
