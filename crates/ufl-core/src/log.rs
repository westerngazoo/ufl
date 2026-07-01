//! The complex logarithm used inside every `eml` node.
//!
//! Per SPEC-0001 §2.4 (Q-AC4 resolution): for [`Value = Complex<f64>`] the
//! standard principal branch is correct as-is — no correction term. The
//! floating-point representation of `sin(τ/2)` (≈ -1.22e-16) self-corrects the
//! chain so that derived `i`, `τ`, and `ln x` for `x<0` are all
//! principal-correct (see `experiments/q-ac4-branch.py`).
//!
//! This function is isolated in its own module so that, if the value type ever
//! changes to arithmetic where `sin(τ/2)` is exact zero, this is the **single
//! point of change**. AC6 is the tripwire.

use crate::eval::Value;

/// The principal-branch complex logarithm — `Im ∈ (-τ/2, τ/2]` (equivalently
/// the conventional `(-π, π]`). Per SPEC-0001 §2.4 (Q-AC4 resolution) no
/// correction term is needed for `Value = Complex<f64>`: the floating-point
/// representation of `sin(τ/2)` self-corrects the chain. AC6 is the tripwire.
pub(crate) fn ln_eml(w: Value) -> Value {
    // Principal branch complex logarithm — relies on IEEE-754 self-correction
    // for `sin(τ/2)` as detailed in SPEC-0001 §2.4.
    w.ln()
}

#[cfg(test)]
mod tests {
    /// R-0001 AC6 (tripwire) — the AC4 self-correction depends on
    /// `sin(τ/2) ≠ 0` in the runtime's `f64`. If this ever becomes exactly
    /// zero (e.g. arbitrary-precision backend, exotic `sin`), Q-AC4 must be
    /// re-opened; see SPEC-0001 §2.4.
    ///
    /// Mirrored as an end-to-end test in
    /// `tests/r_0001_acceptance.rs::ac6_sin_tau_over_two_is_non_zero_in_f64`.
    #[test]
    fn ac6_sin_tau_over_two_is_non_zero() {
        let s = std::f64::consts::PI.sin();
        assert_ne!(s, 0.0, "sin(τ/2) is exactly zero — Q-AC4 must be re-opened");
    }
}
