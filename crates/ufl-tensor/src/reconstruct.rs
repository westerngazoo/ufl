//! Reconstruction and exact verification (SPEC-0006 §2.5).

use crate::scheme::{Scheme, SchemeError};
use crate::tensor::Tensor;

/// `reconstruct[p,q,r] = Σ_t u_t[p]·v_t[q]·w_t[r]` (SPEC-0006 §2.1). The tensor
/// dim is the scheme's own dim, so a dim/`n` desync is impossible. An empty
/// scheme reconstructs to `zeros(0)`.
pub fn reconstruct(_scheme: &Scheme) -> Tensor {
    unimplemented!("R-0006 implementation — reconstruct, see SPEC-0006 §2.5/§3")
}

/// Build `target(n)`, check the scheme's dim is `n²` (else `DimMismatch` —
/// including an empty scheme against `n ≥ 1`), and return the exact integer
/// error. Total — never panics.
pub fn scheme_error(_scheme: &Scheme, _n: usize) -> Result<i64, SchemeError> {
    unimplemented!("R-0006 implementation — scheme_error, see SPEC-0006 §2.5/§3")
}

/// Valid at rank `R`: exactly `R` triples AND exact reconstruction.
pub fn is_valid(scheme: &Scheme, n: usize, rank: usize) -> bool {
    scheme.rank() == rank && matches!(scheme_error(scheme, n), Ok(0))
}
