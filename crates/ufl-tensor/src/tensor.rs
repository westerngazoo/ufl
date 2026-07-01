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
        let capacity = dim
            .checked_mul(dim)
            .and_then(|x| x.checked_mul(dim))
            .expect("tensor capacity overflow");
        Self {
            dim,
            data: vec![0; capacity],
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
        if p >= self.dim || q >= self.dim || r >= self.dim {
            return;
        }
        self.data[(p * self.dim + q) * self.dim + r] += v;
    }
}

/// The matmul target tensor `T_n` (SPEC-0006 §2.1): `T_n[i·n+j, j·n+k, i·n+k] =
/// 1` for all `i,j,k ∈ 0..n`, else 0. The map is injective, so every entry is
/// 0 or 1.
pub fn target(n: usize) -> Tensor {
    let d = n.checked_mul(n).expect("target dimension overflow");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_at_out_of_bounds_does_not_panic() {
        let mut t = Tensor::zeros(2);

        // Out of bounds on p
        t.add_at(2, 0, 0, 1);

        // Out of bounds on q
        t.add_at(0, 2, 0, 1);

        // Out of bounds on r
        t.add_at(0, 0, 2, 1);

        // Make sure it remains zeros
        let expected = Tensor::zeros(2);
        assert_eq!(t, expected);
    }
}
