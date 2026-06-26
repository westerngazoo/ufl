//! The deterministic generator (now shared via `ufl-prng`) + the matmul-genome
//! sampling helpers R-0008 uses (SPEC-0008 §2.1).
//!
//! The core [`SplitMix64`] stream lives in `ufl-prng` (one source of the
//! determinism-critical generator). The `below`/`ternary` helpers stay here,
//! unchanged — a fast `% n` whose tiny bias is negligible for search, kept
//! bit-identical so R-0008's seeded results do not shift. (`ufl-evolve` uses
//! `ufl-prng`'s *unbiased* `below` instead.)

pub use ufl_prng::SplitMix64;

/// Matmul-genome sampling over the shared generator: a bounded index and a
/// uniform ternary coefficient. Named distinctly from `ufl-prng`'s inherent
/// `below` to avoid shadowing it (and to keep the legacy `% n` behaviour).
pub(crate) trait MatmulSampling {
    /// A value in `0..n` (legacy `% n`; bias negligible for search, kept for
    /// stream parity with the pre-`ufl-prng` R-0008 results).
    fn below_usize(&mut self, n: usize) -> usize;

    /// A uniform ternary coefficient in `{-1, 0, +1}`.
    fn ternary(&mut self) -> i8;
}

impl MatmulSampling for SplitMix64 {
    fn below_usize(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    fn ternary(&mut self) -> i8 {
        [-1, 0, 1][self.below_usize(3)]
    }
}
