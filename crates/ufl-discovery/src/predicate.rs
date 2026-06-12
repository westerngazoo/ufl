//! The tensor instance of the `Predicate` trait (SPEC-0007 §2.3).

use ufl_predicate::Predicate;
use ufl_tensor::{error, reconstruct, target, Scheme, SchemeError, Tensor};

/// `P_{n,R}`: a scheme satisfies it iff it is a valid **rank-R decomposition**
/// of the matmul tensor `T_n` — `reconstruct(scheme) == T_n` AND
/// `rank(scheme) == R`. (Named for both conjuncts: the rank bound is the
/// discovery prize.)
///
/// Relation to `ufl_tensor::is_valid` (SPEC-0007 §2.3): on dim-consistent
/// schemes, `discharge == Ok(is_valid(scheme, n, rank))`; on dim-mismatched
/// schemes `discharge` is `Err(DimMismatch)` where `is_valid` collapses to
/// `false`.
///
/// ```
/// use ufl_discovery::RankDecomposition;
/// use ufl_predicate::Predicate;
/// use ufl_tensor::{Scheme, Triple};
///
/// // n = 1: 1×1 matmul is one multiplication — the trivial exact scheme.
/// let mut scheme = Scheme::new();
/// scheme
///     .push(Triple::new(vec![1], vec![1], vec![1]).unwrap())
///     .unwrap();
/// let p = RankDecomposition::new(1, 1);
/// assert_eq!(p.discharge(&scheme), Ok(true));
/// ```
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
    /// `Err(DimMismatch)` — independent of the rank field (SPEC-0007 §2.3; the
    /// review's blocking finding was a short-circuit that flipped the error
    /// contract on the rank conjunct).
    fn discharge(&self, scheme: &Scheme) -> Result<bool, SchemeError> {
        let recon = reconstruct(scheme);
        match error(&recon, &self.target) {
            None => Err(SchemeError::DimMismatch {
                n: self.n,
                expected: self.target.dim(),
                got: recon.dim(),
            }),
            Some(e) => Ok(e == 0 && scheme.rank() == self.rank),
        }
    }
}
