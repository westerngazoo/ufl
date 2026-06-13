//! Deterministic, seeded `SplitMix64` (SPEC-0008 §2.1).
//!
//! In-crate (no `rand` dependency) so AC1 determinism is trivially auditable.

/// A seeded splitmix64 generator.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Construct from a seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next 64-bit output. Same seed ⇒ same stream (AC1).
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `0..n` (modulo bias negligible for search; deterministic).
    pub(crate) fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// A uniform ternary coefficient in `{-1, 0, +1}`.
    pub(crate) fn ternary(&mut self) -> i8 {
        [-1, 0, 1][self.below(3)]
    }
}
