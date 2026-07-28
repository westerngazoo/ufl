//! R-0018 / SPEC-0018 — the rectangular (square-embedded) flip-graph and the
//! certified **beyond-Strassen** ⟨2,2,3⟩ rank-11 reduction.
//!
//! The fast tests certify the **banked artifact** (the discovered scheme, as a
//! literal) so the result is committed code, not a claim; the `#[ignore]` gate
//! re-derives it from naive at the pinned seed (SPEC-0018 §3, §4).

use ufl_discovery::{naive_embedded, reduce_matmul_rect, target_rect, RankDecomposition};
use ufl_predicate::Predicate;
use ufl_prng::SplitMix64;
use ufl_tensor::{reconstruct, target, Scheme, Triple};

/// The certified rank-11 ⟨2,2,3⟩ scheme this engine discovered (seed 3, 10⁶
/// flips) — banked as a literal so its certification is a committed regression.
/// Slots are the `d = max(2·2, 2·3, 2·3) = 6` square embedding; `u`'s dims 4–5
/// are structurally zero (⟨2,2,3⟩'s `u` truly has length `m·n = 4`).
const BANKED_223_RANK11: [([i8; 6], [i8; 6], [i8; 6]); 11] = [
    ([1, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0], [1, -1, 0, 0, 0, 0]),
    ([0, 1, 0, 1, 0, 0], [1, 1, 0, 1, 1, 0], [0, 1, 0, 0, 0, 0]),
    ([1, 0, 0, 0, 0, 0], [0, 0, 1, 0, 0, 0], [0, 0, 1, 0, 0, 0]),
    ([-1, 1, 0, 0, 0, 0], [0, 0, 0, 1, 0, 0], [1, 0, 0, -1, 0, 0]),
    ([0, 0, 0, 1, 0, 0], [0, 0, 0, 0, 0, 1], [0, 0, 0, 0, 0, 1]),
    ([0, 1, 0, 0, 0, 0], [0, 0, 0, 0, 0, 1], [0, 0, 1, 0, 0, 0]),
    ([1, -1, 1, -1, 0, 0], [1, 1, 0, 0, 0, 0], [0, 0, 0, 1, 0, 0]),
    ([0, 0, 1, 0, 0, 0], [0, 1, 0, 0, 0, 0], [0, 0, 0, -1, 1, 0]),
    ([0, 0, 1, 0, 0, 0], [0, 0, 1, 0, 0, 0], [0, 0, 0, 0, 0, 1]),
    ([-1, 1, 0, 1, 0, 0], [1, 1, 0, 1, 0, 0], [0, -1, 0, 1, 0, 0]),
    ([0, 0, 0, 1, 0, 0], [0, 0, 0, 0, 1, 0], [0, -1, 0, 0, 1, 0]),
];

fn banked_scheme() -> Scheme {
    let mut s = Scheme::new();
    for (u, v, w) in BANKED_223_RANK11 {
        let t = Triple::new(u.to_vec(), v.to_vec(), w.to_vec()).expect("banked slots are ternary");
        s.push(t).expect("banked slots share length 6");
    }
    s
}

/// AC1 — the embedded naive is exact, and ⟨2,2,2⟩ is unchanged (the regression
/// guard on the square path SPEC-0013 certified).
#[test]
fn embedded_naive_is_exact_and_square_is_unchanged() {
    // Square: the embedding of ⟨n,n,n⟩ reproduces the committed square target.
    let sq = naive_embedded(2, 2, 2);
    assert_eq!(sq.rank(), 8, "naive ⟨2,2,2⟩ is rank 8");
    assert_eq!(
        sq.dim(),
        4,
        "d = max(4,4,4) = 4 — the square case is untouched"
    );
    let sq_scheme = sq.to_scheme().expect("naive is ternary");
    assert_eq!(
        reconstruct(&sq_scheme),
        target(2),
        "the embedded ⟨2,2,2⟩ IS the committed square target"
    );

    // Rectangular: rank m·n·p, ternary, and exact against its own target.
    for (m, n, p, rank, d) in [(2, 2, 3, 12, 6), (2, 3, 3, 18, 9), (1, 2, 3, 6, 6)] {
        let s = naive_embedded(m, n, p);
        assert_eq!(s.rank(), rank, "naive ⟨{m},{n},{p}⟩ rank");
        assert_eq!(s.dim(), d, "d = max(m·n, n·p, m·p)");
        assert!(s.is_ternary(), "the naive scheme is 0/1");
        let scheme = s.to_scheme().expect("ternary");
        assert_eq!(
            reconstruct(&scheme),
            target_rect(m, n, p),
            "naive ⟨{m},{n},{p}⟩ reconstructs to its embedded target"
        );
    }
}

/// The embedding invariant (SPEC-0018 §1.2): padded dimensions are a zero
/// subspace — no move can write into them, so the walk stays in the embedded
/// problem and the frontier equals a true-rectangular one.
#[test]
fn padded_dimensions_stay_structurally_zero_under_the_walk() {
    // ⟨2,2,3⟩: u truly has length m·n = 4, so dims 4 and 5 must stay zero.
    let scheme = reduce_matmul_rect(2, 2, 3, 12, 7, 2_000).expect("rank ≤ 12 is immediate");
    for t in scheme.triples() {
        assert_eq!(t.u()[4], 0, "padded u dim 4 must stay structurally zero");
        assert_eq!(t.u()[5], 0, "padded u dim 5 must stay structurally zero");
    }
}

/// AC2 (the artifact) — the discovered rank-11 ⟨2,2,3⟩ scheme certifies through
/// the exact verifier, and the verifier is the sole judge: a one-coefficient
/// corruption is rejected, and so is the wrong rank.
#[test]
fn the_banked_rank11_scheme_certifies_beyond_strassen() {
    let scheme = banked_scheme();
    assert_eq!(
        scheme.rank(),
        11,
        "beyond the naive 12 — Hopcroft–Kerr optimal"
    );

    let t223 = target_rect(2, 2, 3);
    assert_eq!(
        RankDecomposition::for_target(t223.clone(), 11).discharge(&scheme),
        Ok(true),
        "the certified beyond-Strassen ⟨2,2,3⟩ reduction"
    );
    // The rank claim is part of the judgement: rank 11 ≠ 12.
    assert_eq!(
        RankDecomposition::for_target(t223.clone(), 12).discharge(&scheme),
        Ok(false),
        "discharge certifies at the claimed rank only"
    );

    // Corruption: flip one w coefficient — the tensor no longer reconstructs.
    let mut corrupt = Scheme::new();
    for (i, t) in scheme.triples().iter().enumerate() {
        let mut w = t.w().to_vec();
        if i == 0 {
            w[0] = if w[0] == 0 { 1 } else { 0 };
        }
        corrupt
            .push(Triple::new(t.u().to_vec(), t.v().to_vec(), w).expect("ternary"))
            .expect("equal length");
    }
    assert_eq!(
        RankDecomposition::for_target(t223, 11).discharge(&corrupt),
        Ok(false),
        "the verifier is the sole judge — a corrupted scheme never certifies"
    );
}

/// The independent check: the banked scheme really multiplies 2×2 by 2×3, over
/// exact `i64`, for random matrices — trusting only its coefficients.
#[test]
fn the_banked_scheme_multiplies_matrices_exactly() {
    let mut rng = SplitMix64::new(0xB3_23);
    let scheme = banked_scheme();
    for _ in 0..20_000 {
        // A is 2×2 (flat u-index i·2+j), B is 2×3 (flat v-index j·3+k).
        let a: Vec<i64> = (0..4).map(|_| rng.below(9) as i64 - 4).collect();
        let b: Vec<i64> = (0..6).map(|_| rng.below(9) as i64 - 4).collect();

        // The scheme's products, then its output combination.
        let mut c = [0i64; 6];
        for t in scheme.triples() {
            let ua: i64 = t
                .u()
                .iter()
                .take(4)
                .zip(&a)
                .map(|(&x, &y)| x as i64 * y)
                .sum();
            let vb: i64 = t.v().iter().zip(&b).map(|(&x, &y)| x as i64 * y).sum();
            let prod = ua * vb;
            for (r, &wr) in t.w().iter().enumerate() {
                c[r] += wr as i64 * prod;
            }
        }

        // The textbook product C[i][k] = Σ_j A[i][j]·B[j][k], flat index i·3+k.
        for i in 0..2 {
            for k in 0..3 {
                let want: i64 = (0..2).map(|j| a[i * 2 + j] * b[j * 3 + k]).sum();
                assert_eq!(c[i * 3 + k], want, "C[{i}][{k}] from 11 products");
            }
        }
    }
}

/// AC2 (the search) — re-derive the reduction from naive at the pinned seed.
/// Release-only: ~10⁶ flips (SPEC-0018 §2.2, ~1.5 s release; far slower in debug
/// where the per-flip `debug_assert` tensor invariant is live).
#[test]
#[ignore = "release e2e: cargo test -p ufl-discovery --release --test r_0018_rect -- --ignored"]
fn the_gate_rederives_a_certified_rank11_from_naive() {
    let scheme = reduce_matmul_rect(2, 2, 3, 11, 3, 1_000_000).expect("seed 3 crosses the plateau");
    assert_eq!(scheme.rank(), 11);
    assert_eq!(
        RankDecomposition::for_target(target_rect(2, 2, 3), 11).discharge(&scheme),
        Ok(true),
        "the re-derived scheme certifies — beyond ⟨2,2,2⟩"
    );
}
