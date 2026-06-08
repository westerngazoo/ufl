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
    pub(crate) fn add_at(&mut self, p: usize, q: usize, r: usize, v: i64) {
        self.data[(p * self.dim + q) * self.dim + r] += v;
    }
}

/// The matmul target tensor `T_n` (SPEC-0006 §2.1): `T_n[i·n+j, j·n+k, i·n+k] =
/// 1` for all `i,j,k ∈ 0..n`, else 0. The map is injective, so every entry is
/// 0 or 1.
pub fn target(n: usize) -> Tensor {
    let d = n * n;
    let mut t = Tensor::zeros(d);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                t.add_at(i * n + j, j * n + k, i * n + k, 1);
            }
        }
    }
    t
}

/// `Σ (a − b)²` over all entries; `None` if the dims differ (total, no panic);
/// `Some(0)` iff equal. Exact within the §2.5 i64 envelope.
pub fn error(a: &Tensor, b: &Tensor) -> Option<i64> {
    if a.dim != b.dim {
        return None;
    }
    Some(
        a.data
            .iter()
            .zip(&b.data)
            .map(|(x, y)| {
                let diff = x - y;
                diff * diff
            })
            .sum(),
    )
}
