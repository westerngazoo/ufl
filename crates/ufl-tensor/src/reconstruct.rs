//! Reconstruction and exact verification (SPEC-0006 §2.5).

use crate::scheme::{Scheme, SchemeError};
use crate::tensor::{error, target, Tensor};

/// `reconstruct[p,q,r] = Σ_t u_t[p]·v_t[q]·w_t[r]` (SPEC-0006 §2.1). The tensor
/// dim is the scheme's own dim, so a dim/`n` desync is impossible. An empty
/// scheme reconstructs to `zeros(0)`.
pub fn reconstruct(scheme: &Scheme) -> Tensor {
    let d = scheme.dim().unwrap_or(0);
    let mut t = Tensor::zeros(d);
    for tr in scheme.triples() {
        let (u, v, w) = (tr.u(), tr.v(), tr.w());
        for (p, &up) in u.iter().enumerate() {
            if up == 0 {
                continue; // skipped terms are provably +0 (additive identity)
            }
            for (q, &vq) in v.iter().enumerate() {
                if vq == 0 {
                    continue;
                }
                for (r, &wr) in w.iter().enumerate() {
                    let prod = up as i64 * vq as i64 * wr as i64;
                    if prod != 0 {
                        t.add_at(p, q, r, prod);
                    }
                }
            }
        }
    }
    t
}

/// Build `target(n)`, check the scheme's dim is `n²` (else `DimMismatch` —
/// including an empty scheme against `n ≥ 1`), and return the exact integer
/// error. Total — never panics.
pub fn scheme_error(scheme: &Scheme, n: usize) -> Result<i64, SchemeError> {
    let expected = n * n;
    // An empty scheme has dim None; treat it as dim 0, which (for n ≥ 1) is a
    // DimMismatch rather than a spurious comparison against a 0-sized tensor.
    let got = scheme.dim().unwrap_or(0);
    if got != expected {
        return Err(SchemeError::DimMismatch { n, expected, got });
    }
    // dims are equal by the check above, so `error` returns `Some`.
    error(&reconstruct(scheme), &target(n)).ok_or(SchemeError::DimMismatch { n, expected, got })
}

/// Valid at rank `R`: exactly `R` triples AND exact reconstruction.
pub fn is_valid(scheme: &Scheme, n: usize, rank: usize) -> bool {
    scheme.rank() == rank && matches!(scheme_error(scheme, n), Ok(0))
}
