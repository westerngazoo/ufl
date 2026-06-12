//! The tensor instance of the `Predicate` trait (SPEC-0007 §2.3).

use ufl_predicate::Predicate;
use ufl_tensor::{target, Scheme, SchemeError, Tensor};

/// `P_{n,R}`: a scheme satisfies it iff it is a valid **rank-R decomposition**
/// of the matmul tensor `T_n` — `reconstruct(scheme) == T_n` AND
/// `rank(scheme) == R`. (Named for both conjuncts: the rank bound is the
/// discovery prize.)
// Fields are consumed by `discharge` in R-0007 step 5; unread during TDD-red.
#[allow(dead_code)]
pub struct RankDecomposition {
    n: usize,
    rank: usize,
    /// `T_n`, computed **once** here — never per discharge (SPEC-0007 AC6).
    target: Tensor,
}

impl RankDecomposition {
    /// Build the predicate for `n×n` matmul at rank `rank`, caching `T_n`.
    pub fn new(n: usize, rank: usize) -> Self {
        Self {
            n,
            rank,
            target: target(n),
        }
    }
}

impl Predicate for RankDecomposition {
    type Candidate = Scheme;
    type Error = SchemeError;

    /// Reconstruct unconditionally, dim-check against the cached target, then
    /// conjoin the rank bound. A dim/`n` mismatch is **always**
    /// `Err(DimMismatch)` — independent of the rank field (SPEC-0007 §2.3).
    fn discharge(&self, _scheme: &Scheme) -> Result<bool, SchemeError> {
        unimplemented!("R-0007 implementation — discharge, see SPEC-0007 §2.3")
    }
}
