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
