//! The dense integer tensor, the target `T_n`, and exact error (SPEC-0006 §2.3).

/// A dense 3-index integer tensor of shape `(dim, dim, dim)`, `dim = n²`.
/// Row-major: index `(p, q, r) → (p·dim + q)·dim + r`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tensor {
    dim: usize,
    data: Vec<i64>, // length dim³
}

impl Tensor {
    /// An all-zero tensor of shape `(dim, dim, dim)`.
    pub fn zeros(dim: usize) -> Self {
        Self {
            dim,
            data: vec![0; dim * dim * dim],
        }
    }

    /// The side length (`d = n²`).
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Total accessor: `None` if any index is out of range (the `Option`
    /// convention of `ufl_core::Env::get`; no panic, CLAUDE.md §6).
    pub fn get(&self, p: usize, q: usize, r: usize) -> Option<i64> {
        if p >= self.dim || q >= self.dim || r >= self.dim {
            return None;
        }
        Some(self.data[(p * self.dim + q) * self.dim + r])
    }

    /// Add `v` at `(p, q, r)`. Internal — callers loop `0..dim`, so the index
    /// is in range by construction.
    // Wired up by `target`/`reconstruct` in R-0006 step 5; unused during TDD-red.
    #[allow(dead_code)]
    pub(crate) fn add_at(&mut self, p: usize, q: usize, r: usize, v: i64) {
        self.data[(p * self.dim + q) * self.dim + r] += v;
    }
}

/// The matmul target tensor `T_n` (SPEC-0006 §2.1).
pub fn target(_n: usize) -> Tensor {
    unimplemented!("R-0006 implementation — target, see SPEC-0006 §2.1/§3")
}

/// `Σ (a − b)²` over all entries; `None` if the dims differ (total, no panic);
/// `Some(0)` iff equal. Exact within the §2.5 i64 envelope.
pub fn error(_a: &Tensor, _b: &Tensor) -> Option<i64> {
    unimplemented!("R-0006 implementation — error, see SPEC-0006 §3")
}
