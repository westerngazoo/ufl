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

    /// Add `v` at `(p, q, r)`. Internal — every caller (`target`,
    /// `reconstruct`) derives its indices from loops bounded by the tensor's
    /// own `dim`, so an out-of-bounds index here is a caller **bug**, never
    /// data. Debug builds fail loudly (`debug_assert!`): this accessor sits
    /// inside the exact verifier, and a silently skipped write is the one
    /// failure mode the verifier-held discipline forbids (R-0013). Release
    /// builds stay total and skip the write — no panic path in library code
    /// (CLAUDE.md §6); the debug gate is what every test run enforces.
    pub(crate) fn add_at(&mut self, p: usize, q: usize, r: usize, v: i64) {
        let in_bounds = p < self.dim && q < self.dim && r < self.dim;
        debug_assert!(
            in_bounds,
            "add_at index ({p}, {q}, {r}) out of bounds for dim {}: caller bug",
            self.dim
        );
        if !in_bounds {
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

    /// An out-of-bounds `add_at` is a caller bug and must fail loudly in
    /// debug builds — the verifier-integrity precondition (R-0013): a silent
    /// no-op inside `reconstruct` is silently-wrong verification.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "caller bug")]
    fn add_at_out_of_bounds_fails_loud_in_debug() {
        let mut t = Tensor::zeros(2);
        t.add_at(2, 0, 0, 1);
    }

    /// In release builds `add_at` stays total: the write is skipped, never a
    /// panic (CLAUDE.md §6 — no panic path in library code).
    #[test]
    #[cfg(not(debug_assertions))]
    fn add_at_out_of_bounds_skips_in_release() {
        let mut t = Tensor::zeros(2);
        t.add_at(2, 0, 0, 1);
        t.add_at(0, 2, 0, 1);
        t.add_at(0, 0, 2, 1);
        assert_eq!(t, Tensor::zeros(2));
    }
}
