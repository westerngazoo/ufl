//! R-0013 Gate-0 (SPEC-0013): the flip-graph reaches a verifier-**certified**
//! rank-7 decomposition of `T₂` from the naive rank-8 start — the banked,
//! reproducible form of the deleted pilot. The exact `RankDecomposition` is the
//! sole judge (AC2); an independent 20,000-pair bilinear check re-derives
//! `C = A·B` without trusting the verifier.
//!
//! Reachability note: rank-7 for ⟨2,2,2⟩ is Strassen by de Groote's uniqueness
//! theorem — a certified re-derivation, not a new result. The deliverable is the
//! reproducible engine.

use ufl_discovery::{reduce_matmul, RankDecomposition};
use ufl_predicate::Predicate;
use ufl_tensor::{Scheme, Triple};

// A seed/budget pinned to a deterministic success (the impl may repin to whatever
// (seed, budget) reaches rank-7 within a laptop-second — that is legitimate).
const SEED: u64 = 0xC0FFEE;
const BUDGET: usize = 2_000_000;

/// AC1 — a certified rank-7 for `T₂`, three independent ways, deterministically.
#[test]
fn flip_graph_reaches_certified_rank7_t2() {
    let scheme = reduce_matmul(2, 7, SEED, BUDGET).expect("flip-graph reaches rank-7");
    assert_eq!(scheme.rank(), 7, "the prize is rank 7 (< the naive 8)");

    // (1) the exact verifier is the judge: reconstruct == T₂ AND rank == 7.
    assert_eq!(
        RankDecomposition::new(2, 7).discharge(&scheme),
        Ok(true),
        "the exact verifier certifies an exact rank-7 decomposition",
    );
    // (2) re-certify through a FRESHLY constructed verifier (no shared state).
    assert_eq!(RankDecomposition::new(2, 7).discharge(&scheme), Ok(true));
    // (3) independent bilinear re-derivation — 20,000 seeded random integer pairs,
    // trusting nothing but the scheme's own coefficients.
    let mut rng = 0x1234_5678_9abc_def0u64;
    for _ in 0..20_000 {
        let a = rand_matrix(&mut rng);
        let b = rand_matrix(&mut rng);
        assert_eq!(
            apply_scheme(&scheme, &a, &b),
            matmul(&a, &b),
            "the scheme computes 2×2 matmul on A={a:?} B={b:?}",
        );
    }
}

/// AC2 — the verifier is the sole judge: a tensor-breaking corruption can only
/// *fail* to certify, never yield a false `Ok(true)`.
#[test]
fn verifier_is_the_sole_judge() {
    let good = reduce_matmul(2, 7, SEED, BUDGET).expect("flip-graph reaches rank-7");
    let bad = corrupt_first_nonzero_w(&good);
    assert_ne!(
        RankDecomposition::new(2, 7).discharge(&bad),
        Ok(true),
        "a scheme with one flipped w-coefficient is never falsely certified",
    );
}

// ── helpers ────────────────────────────────────────────────────────────────

/// True 2×2 matmul, row-major flattened (`A[i][j]` at `i·2+j`).
fn matmul(a: &[i64; 4], b: &[i64; 4]) -> [i64; 4] {
    let m = |i: usize, k: usize| (0..2).map(|j| a[i * 2 + j] * b[j * 2 + k]).sum();
    [m(0, 0), m(0, 1), m(1, 0), m(1, 1)]
}

/// Evaluate the decomposition as its bilinear algorithm:
/// `m_t = ⟨u_t, ā⟩ · ⟨v_t, b̄⟩`, then `C[r] = Σ_t w_t[r] · m_t`.
fn apply_scheme(s: &Scheme, a: &[i64; 4], b: &[i64; 4]) -> [i64; 4] {
    let mut c = [0i64; 4];
    for t in s.triples() {
        let ua: i64 = t.u().iter().enumerate().map(|(p, &up)| up as i64 * a[p]).sum();
        let vb: i64 = t.v().iter().enumerate().map(|(q, &vq)| vq as i64 * b[q]).sum();
        let m = ua * vb;
        for (r, &wr) in t.w().iter().enumerate() {
            c[r] += wr as i64 * m;
        }
    }
    c
}

/// Flip the sign of the first nonzero `w` coefficient in the scheme — stays in
/// `{−1,0,+1}` and provably changes `reconstruct`, so the verifier must reject it.
fn corrupt_first_nonzero_w(s: &Scheme) -> Scheme {
    let mut out = Scheme::new();
    let mut done = false;
    for t in s.triples() {
        let (u, v, mut w) = (t.u().to_vec(), t.v().to_vec(), t.w().to_vec());
        if !done {
            if let Some(x) = w.iter_mut().find(|x| **x != 0) {
                *x = -*x;
                done = true;
            }
        }
        out.push(Triple::new(u, v, w).expect("still ternary"))
            .expect("length-consistent");
    }
    out
}

/// Random 2×2 integer matrix, entries in `[-3, 3]`, via inline splitmix64.
fn rand_matrix(rng: &mut u64) -> [i64; 4] {
    let mut m = [0i64; 4];
    for x in &mut m {
        *x = (next(rng) % 7) as i64 - 3;
    }
    m
}

fn next(rng: &mut u64) -> u64 {
    *rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *rng;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
