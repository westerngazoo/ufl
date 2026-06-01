//! The geometric product atom `∗` — Cayley-table driven (SPEC-0002 §2.4).
//!
//! The product is the static `CAYLEY` table (the fast path) pinned by a
//! rule-based oracle (the [`oracle`] module) and a generation test — the
//! Oracle-Tripwire pattern (`docs/conventions.md`). The table and oracle both
//! consume the single [`super::MASK`] constant.

use super::Multivector;

/// `(sign, result blade index)` for the product of two basis blades. In
/// G(3,0,0) every blade product is a single signed blade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BladeProduct {
    pub sign: i8,
    pub blade: usize,
}

/// `CAYLEY[i][j] = blade_i ∗ blade_j`, in the §2.1 storage index order.
///
/// R-0002 implementation pending (loop step 5): the 64 entries are produced by
/// the [`oracle`] rule and pinned by the generation test. Until then this
/// panics so the geometric product is genuinely unimplemented (TDD red).
pub(crate) fn cayley(_i: usize, _j: usize) -> BladeProduct {
    unimplemented!("R-0002 implementation — CAYLEY table, see SPEC-0002 §2.4")
}

impl std::ops::Mul for Multivector {
    type Output = Multivector;

    /// Geometric product `∑ᵢ ∑ⱼ aᵢ bⱼ · CAYLEY[i][j]` (SPEC-0002 §2.4). No
    /// zero-coefficient short-circuit — `0 · ∞ = NaN` must propagate per
    /// SPEC-0001 AC3.
    fn mul(self, rhs: Multivector) -> Multivector {
        let mut out = Multivector::zero();
        for i in 0..8 {
            let a = self.coeff(i);
            for j in 0..8 {
                let BladeProduct { sign, blade } = cayley(i, j);
                let term = a * rhs.coeff(j);
                out.coeffs[blade] += if sign >= 0 { term } else { -term };
            }
        }
        out
    }
}

/// The rule-based oracle that derives the Cayley table from [`super::MASK`],
/// and the generation test that pins the table to it (SPEC-0002 §2.4.1).
///
/// R-0002 implementation pending (loop step 5).
pub(crate) mod oracle {
    use super::BladeProduct;

    /// Derive `blade_i ∗ blade_j` from the 3-bit masks: result = `mask_i XOR
    /// mask_j`; sign = `(-1)^s` from sorting the concatenated basis-vector list
    /// by adjacent transpositions, then cancelling adjacent equal indices
    /// (each `eₖ² = +1`). See SPEC-0002 §2.4.1.
    // Implemented and exercised by the generation test in R-0002 step 5.
    #[allow(dead_code)]
    pub(crate) fn derive(_i: usize, _j: usize) -> BladeProduct {
        unimplemented!("R-0002 implementation — Cayley oracle, see SPEC-0002 §2.4.1")
    }
}
