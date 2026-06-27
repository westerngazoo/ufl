//! UFL — the shared deterministic PRNG (R-0011 / SPEC-0011 §2.1).
//!
//! A seeded [`SplitMix64`] generator with the samplers the evolver needs:
//! uniform `u64`/`f64`, an **unbiased** bounded integer ([`below`](SplitMix64::below)),
//! and a Gaussian ([`normal`](SplitMix64::normal)). It is the *one* source of the
//! determinism-critical generator: `ufl-discovery` (R-0008) and `ufl-evolve`
//! (R-0011) both build on it, so a seeded run is reproducible across the project
//! and the float→ℝ maps live in one tested place (the architect's three-lens note).
//!
//! The core stream ([`next_u64`](SplitMix64::next_u64)) is the standard SplitMix64
//! (Vigna) — bit-identical to the generator R-0008 shipped, so re-pointing
//! `ufl-discovery` here preserves its exact stream.

#![forbid(unsafe_code)]

/// A seeded SplitMix64 generator. Same seed ⇒ same stream (the determinism
/// contract). `Clone` so a sub-search can fork a reproducible sub-stream.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Construct from a seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64-bit output (standard SplitMix64). Same seed ⇒ same stream.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform integer in `0..n`, **without modulo bias** (Lemire's
    /// multiply–shift with rejection). `below(0) == 0` and `below(1) == 0`.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let mut m = (self.next_u64() as u128) * (n as u128);
        if (m as u64) < n {
            // Reject the low `n.wrapping_neg() % n` outputs to remove the bias.
            let threshold = n.wrapping_neg() % n;
            while (m as u64) < threshold {
                m = (self.next_u64() as u128) * (n as u128);
            }
        }
        (m >> 64) as u64
    }

    /// A uniform `f64` in `[0, 1)` (the top 53 bits scaled — never returns `1.0`).
    pub fn f64_unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// A Gaussian sample with mean `mu` and standard deviation `sigma`
    /// (Box–Muller; `τ` is UFL's circle constant — `docs/conventions.md`).
    pub fn normal(&mut self, mu: f64, sigma: f64) -> f64 {
        let mut u1 = self.f64_unit();
        if u1 < f64::MIN_POSITIVE {
            u1 = f64::MIN_POSITIVE; // guard ln(0); `f64_unit` can return 0.0
        }
        let u2 = self.f64_unit();
        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = core::f64::consts::TAU * u2;
        mu + sigma * (radius * theta.cos())
    }
}

#[cfg(test)]
mod tests {
    use super::SplitMix64;

    /// Determinism (the core contract): the same seed yields the same stream.
    #[test]
    fn same_seed_same_stream() {
        let mut a = SplitMix64::new(0xC0FFEE);
        let mut b = SplitMix64::new(0xC0FFEE);
        for _ in 0..256 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        // A different seed diverges.
        let mut c = SplitMix64::new(0xC0FFEF);
        assert_ne!(a.next_u64(), c.next_u64());
    }

    /// Regression lock — the stream is the canonical SplitMix64 (Vigna). The
    /// first value for seed 0 is the published `0xE220A8397B1DCDAF`; pinning it
    /// guarantees re-pointing `ufl-discovery` here cannot silently shift its
    /// seeded R-0008 results.
    #[test]
    fn canonical_stream_seed_zero() {
        let mut r = SplitMix64::new(0);
        assert_eq!(
            [r.next_u64(), r.next_u64(), r.next_u64()],
            [
                0xE220_A839_7B1D_CDAF,
                0x6E78_9E6A_A1B9_65F4,
                0x06C4_5D18_8009_454F
            ],
        );
    }

    /// `f64_unit` lands in `[0, 1)` and is mean-centered near 0.5.
    #[test]
    fn f64_unit_range_and_mean() {
        let mut r = SplitMix64::new(1);
        let n = 200_000;
        let mut sum = 0.0;
        for _ in 0..n {
            let x = r.f64_unit();
            assert!((0.0..1.0).contains(&x), "f64_unit out of [0,1): {x}");
            sum += x;
        }
        let mean = sum / n as f64;
        assert!((mean - 0.5).abs() < 0.01, "f64_unit mean {mean} not ≈ 0.5");
    }

    /// `normal` has the requested mean and standard deviation.
    #[test]
    fn normal_mean_and_std() {
        let mut r = SplitMix64::new(7);
        let (mu, sigma) = (2.0, 3.0);
        let n = 400_000;
        let (mut sum, mut sumsq) = (0.0, 0.0);
        for _ in 0..n {
            let x = r.normal(mu, sigma);
            sum += x;
            sumsq += x * x;
        }
        let mean = sum / n as f64;
        let var = sumsq / n as f64 - mean * mean;
        assert!((mean - mu).abs() < 0.05, "normal mean {mean} not ≈ {mu}");
        assert!(
            (var.sqrt() - sigma).abs() < 0.05,
            "normal std {} not ≈ {sigma}",
            var.sqrt()
        );
    }

    /// `below(n)` stays in range, is unbiased, and handles the edge cases.
    #[test]
    fn below_range_unbiased_edges() {
        let mut r = SplitMix64::new(42);
        assert_eq!(r.below(0), 0, "below(0) must be 0");
        assert_eq!(r.below(1), 0, "below(1) must be 0");

        let n = 7u64;
        let trials = 70_000;
        let mut buckets = [0u64; 7];
        for _ in 0..trials {
            let v = r.below(n);
            assert!(v < n, "below({n}) returned {v} ≥ {n}");
            buckets[v as usize] += 1;
        }
        let expected = trials / n;
        for (k, &count) in buckets.iter().enumerate() {
            let dev = (count as i64 - expected as i64).unsigned_abs();
            assert!(
                dev < expected / 8,
                "bucket {k} count {count} far from {expected} (unbiased?)"
            );
        }
    }
}
