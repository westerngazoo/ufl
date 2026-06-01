//! Geometric algebra over G(3,0,0) — the spatial layer.
//!
//! Realizes [R-0002](../../../../requirements/0002-geometric-algebra-core.md)
//! per [SPEC-0002](../../../../specs/0002-geometric-algebra-core.md). A
//! [`Multivector`] is 8 complex coefficients in grade-then-lexicographic blade
//! order; the atoms are [`Multivector::lift`] (`𝒢ₖ`) and the geometric product
//! (`∗`, the [`std::ops::Mul`] impl in [`product`]).

mod lift;
mod product;

pub use lift::GradeLift;

use crate::eval::Value;

/// Storage index → basis-vector mask. The single source of the blade encoding;
/// the geometric-product table and its oracle both consume it (SPEC-0002 §2.1).
// Wired up by the Cayley oracle in R-0002 step 5; unused during TDD-red.
#[allow(dead_code)]
pub(crate) const MASK: [u8; 8] = [
    0b000, // 0: 1
    0b001, // 1: e₁
    0b010, // 2: e₂
    0b100, // 3: e₃
    0b011, // 4: e₁₂
    0b101, // 5: e₁₃
    0b110, // 6: e₂₃
    0b111, // 7: e₁₂₃
];

/// A multivector of G(3,0,0): 8 complex coefficients in grade-then-lexicographic
/// blade order `[1, e₁, e₂, e₃, e₁₂, e₁₃, e₂₃, e₁₂₃]` (SPEC-0002 §2.1, §2.2).
///
/// R-0002 AC1 holds structurally — the coefficient count is always 8.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multivector {
    coeffs: [Value; 8],
}

/// Grade of each blade by storage index (SPEC-0002 §2.1).
pub(crate) const GRADE: [u8; 8] = [0, 1, 1, 1, 2, 2, 2, 3];

impl Multivector {
    /// The additive identity — all coefficients zero.
    pub fn zero() -> Self {
        Self {
            coeffs: [Value::new(0.0, 0.0); 8],
        }
    }

    /// Construct directly from the 8 coefficients (blade order, §2.1).
    pub fn from_coeffs(coeffs: [Value; 8]) -> Self {
        Self { coeffs }
    }

    /// Coefficient at a blade index in `0..8`. Indices are program-derived
    /// (from [`MASK`] / the §2.1 table), never external input; an out-of-range
    /// index is a genuine unreachable state, so this panics rather than
    /// returning `Result` (CLAUDE.md §6).
    pub fn coeff(&self, blade: usize) -> Value {
        self.coeffs[blade]
    }

    /// The grade-`k` part: blades of grade `k` kept, all others zeroed. `k` is
    /// in `0..=3`; a larger grade is an unreachable state (the `GRADE` table
    /// simply never matches, yielding `zero()`).
    pub fn grade(&self, k: u8) -> Self {
        let mut out = Multivector::zero();
        for (i, &c) in self.coeffs.iter().enumerate() {
            if GRADE[i] == k {
                out.coeffs[i] = c;
            }
        }
        out
    }

    /// The Clifford reverse `~M`: negate grade-2 and grade-3 components
    /// (SPEC-0002 §2.5). Stated as a per-blade `NEGATE` table — the obvious
    /// reading is the correct reading.
    pub fn reverse(&self) -> Self {
        const NEGATE: [bool; 8] = [
            false, // 1     grade 0
            false, // e₁    grade 1
            false, // e₂    grade 1
            false, // e₃    grade 1
            true,  // e₁₂   grade 2
            true,  // e₁₃   grade 2
            true,  // e₂₃   grade 2
            true,  // e₁₂₃  grade 3
        ];
        let mut out = *self;
        for (c, &neg) in out.coeffs.iter_mut().zip(NEGATE.iter()) {
            if neg {
                *c = -*c;
            }
        }
        out
    }

    /// The coefficient norm `|M| = √(Σᵢ |cᵢ|²)` — always real and
    /// non-negative (SPEC-0002 §2.6).
    pub fn norm(&self) -> f64 {
        self.coeffs.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt()
    }
}

impl std::ops::Add for Multivector {
    type Output = Multivector;
    fn add(self, rhs: Multivector) -> Multivector {
        let mut out = Multivector::zero();
        for i in 0..8 {
            out.coeffs[i] = self.coeffs[i] + rhs.coeffs[i];
        }
        out
    }
}

impl std::ops::Sub for Multivector {
    type Output = Multivector;
    fn sub(self, rhs: Multivector) -> Multivector {
        let mut out = Multivector::zero();
        for i in 0..8 {
            out.coeffs[i] = self.coeffs[i] - rhs.coeffs[i];
        }
        out
    }
}

impl std::ops::Mul<Value> for Multivector {
    type Output = Multivector;
    fn mul(self, scalar: Value) -> Multivector {
        let mut out = Multivector::zero();
        for i in 0..8 {
            out.coeffs[i] = self.coeffs[i] * scalar;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    //! Structural unit tests that belong beside the type. These exercise only
    //! the implemented linear-space surface (`zero`/`from_coeffs`/`coeff`/
    //! `grade`/`reverse`/`norm`/`Add`/`Sub`/`Mul<Value>`) — they do **not**
    //! touch the geometric product `∗`, so they are green even at TDD-red.
    //! Product-dependent behaviour (AC3–AC6, the Cayley tripwire) lives in
    //! `tests/r_0002_acceptance.rs` and is red until R-0002 step 5.

    use super::*;

    fn v(re: f64) -> Value {
        Value::new(re, 0.0)
    }

    // AC1 — a G(3,0,0) multivector is exactly 8 coefficients in §2.1 blade
    // order; `from_coeffs` / `coeff` round-trip every index, pinning the count
    // and the storage layout structurally.
    #[test]
    fn ac1_eight_coeffs_round_trip_in_blade_order() {
        let coeffs = [
            v(0.0),
            v(1.0),
            v(2.0),
            v(3.0),
            v(4.0),
            v(5.0),
            v(6.0),
            v(7.0),
        ];
        let m = Multivector::from_coeffs(coeffs);
        for (blade, &c) in coeffs.iter().enumerate() {
            assert_eq!(m.coeff(blade), c, "coeff({blade}) must round-trip");
        }
    }

    // AC1 — `zero()` is the additive identity: every coefficient is 0.
    #[test]
    fn ac1_zero_is_all_zero_coeffs() {
        let z = Multivector::zero();
        for blade in 0..8 {
            assert_eq!(z.coeff(blade), v(0.0), "zero().coeff({blade}) must be 0");
        }
    }

    // AC1 — component-wise `Add` over the linear space.
    #[test]
    fn ac1_add_is_componentwise() {
        let a = Multivector::from_coeffs([v(1.0); 8]);
        let b = Multivector::from_coeffs([v(2.0); 8]);
        let sum = a + b;
        for blade in 0..8 {
            assert_eq!(sum.coeff(blade), v(3.0), "add must be component-wise");
        }
    }

    // AC1 — component-wise `Sub` over the linear space.
    #[test]
    fn ac1_sub_is_componentwise() {
        let a = Multivector::from_coeffs([v(5.0); 8]);
        let b = Multivector::from_coeffs([v(2.0); 8]);
        let diff = a - b;
        for blade in 0..8 {
            assert_eq!(diff.coeff(blade), v(3.0), "sub must be component-wise");
        }
    }

    // AC1 — `Mul<Value>` scales every coefficient (Value-scaling of the space).
    #[test]
    fn ac1_value_scaling_scales_every_coeff() {
        let a = Multivector::from_coeffs([v(2.0); 8]);
        let scaled = a * v(3.0);
        for blade in 0..8 {
            assert_eq!(scaled.coeff(blade), v(6.0), "scale must hit every coeff");
        }
    }

    // AC1 — a rotor is assembled by adding a scalar part and a bivector part,
    // the exact linear-space move AC1 must support for AC5's rotor. This uses
    // only `lift` + `Add`, never the product.
    #[test]
    fn ac1_rotor_assembles_from_scalar_and_bivector_parts() {
        let angle = std::f64::consts::TAU / 8.0;
        let scalar = Multivector::lift(GradeLift::Scalar(v(angle.cos())));
        let bivector = Multivector::lift(GradeLift::Bivector([v(-angle.sin()), v(0.0), v(0.0)]));
        let rotor = scalar + bivector;
        assert_eq!(rotor.coeff(0), v(angle.cos()), "scalar part on blade 0");
        assert_eq!(rotor.coeff(4), v(-angle.sin()), "−sin on the e₁₂ blade");
        for blade in [1, 2, 3, 5, 6, 7] {
            assert_eq!(rotor.coeff(blade), v(0.0), "blade {blade} must be zero");
        }
    }

    // AC1 (support) — `grade(k)` projects: keep grade-k blades, zero the rest.
    // Used by AC5's grade-1 check; verified here on a full multivector.
    #[test]
    fn ac1_grade_projection_keeps_only_target_grade() {
        let full = Multivector::from_coeffs([v(1.0); 8]);
        let g1 = full.grade(1);
        for (blade, &grade) in GRADE.iter().enumerate() {
            let expected = if grade == 1 { v(1.0) } else { v(0.0) };
            assert_eq!(g1.coeff(blade), expected, "grade(1) at blade {blade}");
        }
    }

    // AC5 (support) — the Clifford reverse negates grades 2 and 3 only; this is
    // the `~R` used in the rotor sandwich. Verified independently of `∗`.
    #[test]
    fn ac5_reverse_negates_grades_two_and_three() {
        let m = Multivector::from_coeffs([v(1.0); 8]);
        let r = m.reverse();
        for (blade, &grade) in GRADE.iter().enumerate() {
            let expected = if grade >= 2 { v(-1.0) } else { v(1.0) };
            assert_eq!(r.coeff(blade), expected, "reverse at blade {blade}");
        }
    }

    // AC5 (support) — the coefficient norm `|M| = √Σ|cᵢ|²`. A real grade-1
    // vector `3e₁ + 4e₂` has norm 5; verified independently of `∗`.
    #[test]
    fn ac5_coefficient_norm_of_real_vector() {
        let v3e1_4e2 = Multivector::lift(GradeLift::Vector([v(3.0), v(4.0), v(0.0)]));
        assert!((v3e1_4e2.norm() - 5.0).abs() <= 1e-12, "norm(3e₁+4e₂) == 5");
    }
}
