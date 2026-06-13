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
    rank: usize,
    /// The target tensor, computed **once** here — never per discharge
    /// (SPEC-0007 AC6). `target(n)` for matmul; an arbitrary tensor for a
    /// planted instance (SPEC-0008 AC3).
    target: Tensor,
}

impl RankDecomposition {
    /// Build the predicate for `n×n` matmul at rank `rank`, caching `T_n`.
    pub fn new(n: usize, rank: usize) -> Self {
        Self {
            rank,
            target: target(n),
        }
    }

    /// Build the predicate for an **arbitrary target** at rank `rank` — the
    /// planted-instance verifier (SPEC-0008 AC3). Same residual/discharge
    /// contract; only the cached target differs.
    pub fn for_target(target: Tensor, rank: usize) -> Self {
        Self { rank, target }
    }

    /// The target dimension `d` (the length of each genome vector).
    pub fn dim(&self) -> usize {
        self.target.dim()
    }

    /// The rank `R` this predicate accepts.
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// The graded residual `‖reconstruct(scheme) − target‖²` against the cached
    /// target — the discovery engine's fitness (SPEC-0008 §2.2). A dim mismatch
    /// is `Err(DimMismatch)`, the same total contract as `discharge`.
    /// `discharge` is *defined in terms of* this, so fitness and the accept
    /// step are provably one computation (R-0008 AC2).
    pub fn residual(&self, scheme: &Scheme) -> Result<i64, SchemeError> {
        let recon = reconstruct(scheme);
        error(&recon, &self.target).ok_or(SchemeError::DimMismatch {
            n: self.target.dim().isqrt(), // logical matmul n for the matmul case
            expected: self.target.dim(),
            got: recon.dim(),
        })
    }
}

impl Predicate for RankDecomposition {
    type Candidate = Scheme;
    type Error = SchemeError;

    /// Derived from [`residual`](RankDecomposition::residual): exact iff the
    /// residual is 0 *and* the rank matches. A dim/`n` mismatch propagates as
    /// `Err(DimMismatch)` — always, independent of the rank field (SPEC-0007
    /// §2.3; the review's blocking finding was a short-circuit that flipped the
    /// error contract on the rank conjunct).
    fn discharge(&self, scheme: &Scheme) -> Result<bool, SchemeError> {
        Ok(self.residual(scheme)? == 0 && scheme.rank() == self.rank)
    }
}
