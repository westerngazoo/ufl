//! Acceptance tests for R-0006 — Exact Integer-Tensor Core (`ufl-tensor`).
//!
//! One section per acceptance criterion (SPEC-0006 §6 / R-0006 §3). Every test
//! cites the AC id it verifies.
//!
//! TDD-red status (loop step 3): the computational functions `target`,
//! `error`, `reconstruct`, and `scheme_error` are `unimplemented!()`, so every
//! test that drives them PANICS (expected red). The AC2 validation paths touch
//! only the REAL `Triple::new` / `Scheme::push` and pass green now. Each module
//! below states which it is.

use ufl_tensor::{error, is_valid, reconstruct, scheme_error, target, Scheme, SchemeError, Triple};

// --------------------------------------------------------------------------
// Shared helpers (fixtures from SPEC-0006 §2.6).
// --------------------------------------------------------------------------

/// Build a single-triple `Scheme` from raw coefficient vectors.
/// Panics only on a *test-author* mistake (an invalid fixture), never as part
/// of the behaviour under test.
fn scheme_of(triples: &[(Vec<i8>, Vec<i8>, Vec<i8>)]) -> Scheme {
    let mut s = Scheme::new();
    for (u, v, w) in triples {
        let t = Triple::new(u.clone(), v.clone(), w.clone())
            .expect("fixture triple must be valid (test-author invariant)");
        s.push(t)
            .expect("fixture triple length must match scheme (test-author invariant)");
    }
    s
}

/// The canonical 7-term Strassen 2×2 scheme (SPEC-0006 §2.6 table), length-4
/// vectors in row-major `a11 a12 a21 a22`.
fn strassen_scheme() -> Scheme {
    scheme_of(&[
        (vec![1, 0, 0, 1], vec![1, 0, 0, 1], vec![1, 0, 0, 1]),
        (vec![0, 0, 1, 1], vec![1, 0, 0, 0], vec![0, 0, 1, -1]),
        (vec![1, 0, 0, 0], vec![0, 1, 0, -1], vec![0, 1, 0, 1]),
        (vec![0, 0, 0, 1], vec![-1, 0, 1, 0], vec![1, 0, 1, 0]),
        (vec![1, 1, 0, 0], vec![0, 0, 0, 1], vec![-1, 1, 0, 0]),
        (vec![-1, 0, 1, 0], vec![1, 1, 0, 0], vec![0, 0, 0, 1]),
        (vec![0, 1, 0, -1], vec![0, 0, 1, 1], vec![1, 0, 0, 0]),
    ])
}

/// The naive `R = n³` scheme (SPEC-0006 §2.6): one triple per `(i,j,k)` with
/// `u = e_{i·n+j}`, `v = e_{j·n+k}`, `w = e_{i·n+k}` (standard basis vectors of
/// length `d = n²`).
fn naive_scheme(n: usize) -> Scheme {
    let d = n * n;
    let unit = |idx: usize| {
        let mut e = vec![0i8; d];
        e[idx] = 1;
        e
    };
    let mut s = Scheme::new();
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let t = Triple::new(unit(i * n + j), unit(j * n + k), unit(i * n + k))
                    .expect("naive basis triple must be valid");
                s.push(t).expect("naive triple length must match scheme");
            }
        }
    }
    s
}

/// Count of nonzero (necessarily `1`) entries across the whole `(d,d,d)` grid.
fn count_nonzero(t: &ufl_tensor::Tensor) -> usize {
    let d = t.dim();
    let mut n = 0usize;
    for p in 0..d {
        for q in 0..d {
            for r in 0..d {
                if t.get(p, q, r).expect("in-range by 0..dim loop") != 0 {
                    n += 1;
                }
            }
        }
    }
    n
}

// --------------------------------------------------------------------------
// AC1 — Target tensor `target(n)`  [RED: drives `target`, unimplemented!()]
//
// SPEC-0006 §2.1: T_n[p,q,r] = 1 iff (p,q,r) = (i·n+j, j·n+k, i·n+k) for some
// i,j,k ∈ 0..n; else 0. For n=2 (d=4) the eight (i,j,k) give exactly:
//   (0,0,0) (0,1,1) (1,2,0) (1,3,1) (2,0,2) (2,1,3) (3,2,2) (3,3,3)
// — derived by hand below; all distinct (injectivity ⇒ entries are 0/1).
// --------------------------------------------------------------------------

/// The exact set of 1-entries of T_2, derived from the §2.1 definition.
const T2_ONES: [(usize, usize, usize); 8] = [
    (0, 0, 0), // i=0 j=0 k=0
    (0, 1, 1), // i=0 j=0 k=1
    (1, 2, 0), // i=0 j=1 k=0
    (1, 3, 1), // i=0 j=1 k=1
    (2, 0, 2), // i=1 j=0 k=0
    (2, 1, 3), // i=1 j=0 k=1
    (3, 2, 2), // i=1 j=1 k=0
    (3, 3, 3), // i=1 j=1 k=1
];

#[test]
fn ac1_target2_has_dim_four() {
    // AC1: shape (n², n², n²) ⇒ dim = 4 for n = 2.
    assert_eq!(target(2).dim(), 4, "AC1: target(2) must have dim n² = 4");
}

#[test]
fn ac1_target2_known_ones_are_one() {
    // AC1: every hand-derived (p,q,r) of T_2 is exactly 1.
    let t = target(2);
    for &(p, q, r) in &T2_ONES {
        assert_eq!(
            t.get(p, q, r),
            Some(1),
            "AC1: T_2[{p},{q},{r}] must be 1 (from the (i,j,k) map)"
        );
    }
}

#[test]
fn ac1_target2_has_exactly_eight_ones() {
    // AC1: the (i,j,k) ↦ (p,q,r) map is injective ⇒ exactly n³ = 8 one-entries.
    let t = target(2);
    assert_eq!(
        count_nonzero(&t),
        8,
        "AC1: T_2 must have exactly n³ = 8 nonzero entries (injectivity)"
    );
}

#[test]
fn ac1_target2_entries_are_zero_or_one() {
    // AC1 / §2.1 injectivity: every entry is 0 or 1 (this is what makes
    // error == 0 mean exact equality).
    let t = target(2);
    let ones: std::collections::HashSet<(usize, usize, usize)> = T2_ONES.iter().copied().collect();
    for p in 0..4 {
        for q in 0..4 {
            for r in 0..4 {
                let expected = if ones.contains(&(p, q, r)) { 1 } else { 0 };
                assert_eq!(
                    t.get(p, q, r),
                    Some(expected),
                    "AC1: T_2[{p},{q},{r}] must be exactly {expected}"
                );
            }
        }
    }
}

#[test]
fn ac1_target1_is_single_one() {
    // AC1: n ≥ 1 — for n = 1 (d = 1) the only tuple (0,0,0) gives T_1[0,0,0]=1.
    let t = target(1);
    assert_eq!(t.dim(), 1, "AC1: target(1) dim = 1");
    assert_eq!(t.get(0, 0, 0), Some(1), "AC1: T_1[0,0,0] = 1");
    assert_eq!(
        count_nonzero(&t),
        1,
        "AC1: T_1 has exactly 1³ = 1 one-entry"
    );
}

#[test]
fn ac1_target3_has_dim_nine_and_27_ones() {
    // AC1: verified beyond n=2 — n=3 (d=9) has exactly 3³ = 27 one-entries.
    let t = target(3);
    assert_eq!(t.dim(), 9, "AC1: target(3) dim = n² = 9");
    assert_eq!(
        count_nonzero(&t),
        27,
        "AC1: T_3 has exactly n³ = 27 one-entries"
    );
}

// --------------------------------------------------------------------------
// AC2 — Scheme genotype validation  [GREEN: touches only Triple::new /
// Scheme::push, which are REAL]
//
// SPEC-0006 §2.4: Triple::new accepts only equal-length, non-empty {-1,0,+1}
// vectors (Coefficient / Ragged / Empty else); Scheme::push rejects a
// length-mismatched triple (Mismatch). No panic.
// --------------------------------------------------------------------------

#[test]
fn ac2_triple_accepts_valid_coeffs() {
    // AC2: equal-length, non-empty, all in {-1,0,+1} ⇒ Ok.
    let t = Triple::new(vec![-1, 0, 1], vec![1, -1, 0], vec![0, 1, -1]);
    assert!(
        t.is_ok(),
        "AC2: a valid {{-1,0,+1}} triple must be accepted"
    );
    let t = t.unwrap();
    assert_eq!(t.len(), 3, "AC2: triple len is the shared vector length");
    assert!(!t.is_empty(), "AC2: a constructed triple is never empty");
}

#[test]
fn ac2_triple_rejects_coefficient_above_one() {
    // AC2: a coefficient outside {-1,0,+1} ⇒ Coefficient(c).
    let err = Triple::new(vec![2, 0, 0], vec![0, 0, 0], vec![0, 0, 0]).unwrap_err();
    assert_eq!(
        err,
        SchemeError::Coefficient(2),
        "AC2: coefficient 2 must be rejected as Coefficient(2)"
    );
}

#[test]
fn ac2_triple_rejects_coefficient_below_minus_one() {
    // AC2: -2 is also outside {-1,0,+1}. Boundary on the negative side.
    let err = Triple::new(vec![0, 0], vec![0, -2], vec![0, 0]).unwrap_err();
    assert_eq!(
        err,
        SchemeError::Coefficient(-2),
        "AC2: coefficient -2 must be rejected as Coefficient(-2)"
    );
}

#[test]
fn ac2_triple_rejects_ragged_lengths() {
    // AC2: u/v/w of differing lengths ⇒ Ragged{u,v,w}.
    let err = Triple::new(vec![1, 0, 0], vec![1, 0], vec![1]).unwrap_err();
    assert_eq!(
        err,
        SchemeError::Ragged { u: 3, v: 2, w: 1 },
        "AC2: ragged u/v/w lengths must be rejected as Ragged"
    );
}

#[test]
fn ac2_triple_rejects_empty() {
    // AC2: empty (but equal-length) vectors ⇒ Empty.
    let err = Triple::new(vec![], vec![], vec![]).unwrap_err();
    assert_eq!(
        err,
        SchemeError::Empty,
        "AC2: empty triple vectors must be rejected as Empty"
    );
}

#[test]
fn ac2_scheme_push_accepts_consistent_lengths() {
    // AC2: pushing triples that share the scheme's length keeps it consistent.
    let mut s = Scheme::new();
    assert_eq!(s.dim(), None, "AC2: an empty scheme has no dim");
    s.push(Triple::new(vec![1, 0], vec![0, 1], vec![1, 0]).unwrap())
        .expect("AC2: first push sets the scheme dim");
    s.push(Triple::new(vec![0, 1], vec![1, 0], vec![0, 1]).unwrap())
        .expect("AC2: a length-matching second push is accepted");
    assert_eq!(s.rank(), 2, "AC2: rank counts the triples");
    assert_eq!(
        s.dim(),
        Some(2),
        "AC2: scheme dim is the shared triple length"
    );
}

#[test]
fn ac2_scheme_push_rejects_length_mismatch() {
    // AC2: a triple whose length differs from the scheme's ⇒ Mismatch.
    let mut s = Scheme::new();
    s.push(Triple::new(vec![1, 0, 0, 1], vec![1, 0, 0, 1], vec![1, 0, 0, 1]).unwrap())
        .expect("AC2: first push sets dim = 4");
    let err = s
        .push(Triple::new(vec![1, 0], vec![0, 1], vec![1, 0]).unwrap())
        .unwrap_err();
    assert_eq!(
        err,
        SchemeError::Mismatch {
            expected: 4,
            got: 2
        },
        "AC2: a length-2 triple in a dim-4 scheme must be Mismatch"
    );
}

// --------------------------------------------------------------------------
// AC3 — Reconstruction  [RED: drives `reconstruct`, unimplemented!()]
//
// SPEC-0006 §2.1/§2.5: reconstruct[p,q,r] = Σ_t u_t[p]·v_t[q]·w_t[r], dim = the
// scheme's own dim. Cases below are hand-computable.
// --------------------------------------------------------------------------

#[test]
fn ac3_single_unit_triple_is_one_at_origin() {
    // AC3: u=[1,0], v=[1,0], w=[1,0] (dim 2) ⇒ a single 1 at (0,0,0), 0 else.
    let s = scheme_of(&[(vec![1, 0], vec![1, 0], vec![1, 0])]);
    let t = reconstruct(&s);
    assert_eq!(t.dim(), 2, "AC3: reconstruct dim = scheme dim = 2");
    assert_eq!(t.get(0, 0, 0), Some(1), "AC3: u⊗v⊗w has 1 at (0,0,0)");
    assert_eq!(count_nonzero(&t), 1, "AC3: exactly one nonzero entry");
}

#[test]
fn ac3_single_triple_places_one_at_indexed_slot() {
    // AC3: u=e1, v=e0, w=e1 (dim 2) ⇒ a single 1 at (1,0,1).
    let s = scheme_of(&[(vec![0, 1], vec![1, 0], vec![0, 1])]);
    let t = reconstruct(&s);
    assert_eq!(
        t.get(1, 0, 1),
        Some(1),
        "AC3: e1⊗e0⊗e1 places its 1 at (1,0,1)"
    );
    assert_eq!(count_nonzero(&t), 1, "AC3: exactly one nonzero entry");
}

#[test]
fn ac3_negative_coefficient_yields_minus_one() {
    // AC3: integer (signed) accumulation — w=-e0 gives entry -1.
    let s = scheme_of(&[(vec![1, 0], vec![1, 0], vec![-1, 0])]);
    let t = reconstruct(&s);
    assert_eq!(
        t.get(0, 0, 0),
        Some(-1),
        "AC3: a -1 coefficient yields a -1 entry (signed i64)"
    );
}

#[test]
fn ac3_sums_over_triples_with_integer_accumulation() {
    // AC3: Σ_t — two identical unit triples accumulate to 2 at (0,0,0).
    let s = scheme_of(&[
        (vec![1, 0], vec![1, 0], vec![1, 0]),
        (vec![1, 0], vec![1, 0], vec![1, 0]),
    ]);
    let t = reconstruct(&s);
    assert_eq!(
        t.get(0, 0, 0),
        Some(2),
        "AC3: two unit triples sum to 2 at (0,0,0)"
    );
    assert_eq!(count_nonzero(&t), 1, "AC3: still a single nonzero slot");
}

#[test]
fn ac3_dense_vectors_form_outer_product() {
    // AC3: u=[1,1], v=[1,0], w=[1,0] ⇒ 1 at (0,0,0) and (1,0,0).
    let s = scheme_of(&[(vec![1, 1], vec![1, 0], vec![1, 0])]);
    let t = reconstruct(&s);
    assert_eq!(t.get(0, 0, 0), Some(1), "AC3: outer product entry (0,0,0)");
    assert_eq!(t.get(1, 0, 0), Some(1), "AC3: outer product entry (1,0,0)");
    assert_eq!(count_nonzero(&t), 2, "AC3: exactly two nonzero entries");
}

// --------------------------------------------------------------------------
// AC4 — Exact error  [RED: drives `error` / `scheme_error`, unimplemented!()]
//
// SPEC-0006 §2.5: error = Σ(a−b)² as i64, total (None on dim mismatch);
// scheme_error builds target(n), checks dim == n² (DimMismatch else, including
// an empty scheme vs n ≥ 1), returns the exact i64. No floating point.
// --------------------------------------------------------------------------

#[test]
fn ac4_error_of_equal_tensors_is_zero() {
    // AC4: error(T_2, T_2) == Some(0).
    assert_eq!(
        error(&target(2), &target(2)),
        Some(0),
        "AC4: a tensor against itself has error 0"
    );
}

#[test]
fn ac4_error_of_different_tensors_is_positive() {
    // AC4: T_2 vs T_1 differ ⇒ a strictly positive error... but dims differ, so
    // this pair returns None (see the dedicated dim-mismatch test). Here we use
    // a SAME-dim positive case: zeros(4) vs target(2) has error = #ones = 8.
    let zeros = ufl_tensor::Tensor::zeros(4);
    assert_eq!(
        error(&zeros, &target(2)),
        Some(8),
        "AC4: zeros vs T_2 (eight 1-entries) has error Σ(0−1)² = 8"
    );
}

#[test]
fn ac4_error_is_none_on_dim_mismatch() {
    // AC4: error is total — different dims ⇒ None, no panic.
    assert_eq!(
        error(&target(2), &target(3)),
        None,
        "AC4: error of dim-4 vs dim-9 tensors is None"
    );
}

#[test]
fn ac4_scheme_error_rejects_wrong_dim() {
    // AC4: a non-empty scheme whose dim ≠ n² ⇒ DimMismatch. A dim-2 scheme
    // against n=2 (expected n² = 4) mismatches.
    let s = scheme_of(&[(vec![1, 0], vec![1, 0], vec![1, 0])]);
    let err = scheme_error(&s, 2).unwrap_err();
    assert_eq!(
        err,
        SchemeError::DimMismatch {
            n: 2,
            expected: 4,
            got: 2
        },
        "AC4: a dim-2 scheme vs n=2 (n²=4) is DimMismatch"
    );
}

#[test]
fn ac4_scheme_error_empty_scheme_is_dim_mismatch() {
    // AC4 / SPEC-0006 §3 corner: an EMPTY scheme (dim None ⇒ treated as 0)
    // against n=2 must be DimMismatch{got:0}, NOT a panic.
    let s = Scheme::new();
    let err = scheme_error(&s, 2).unwrap_err();
    assert_eq!(
        err,
        SchemeError::DimMismatch {
            n: 2,
            expected: 4,
            got: 0
        },
        "AC4: an empty scheme vs n=2 is DimMismatch{{got:0}} (no panic)"
    );
}

#[test]
fn ac4_scheme_error_naive_n2_is_zero() {
    // AC4: scheme_error of a correct scheme is exactly 0 (ties AC4 to AC6).
    assert_eq!(
        scheme_error(&naive_scheme(2), 2),
        Ok(0),
        "AC4: the naive n=2 scheme reconstructs T_2 with error 0"
    );
}

#[test]
fn ac4_scheme_error_wrong_scheme_is_positive() {
    // AC4: a dim-correct but WRONG scheme has strictly positive error. A single
    // unit triple (dim 4) reconstructs to one 1 at (0,0,0); T_2 has eight 1s,
    // one of which IS (0,0,0), so they differ in the other seven ⇒ error 7.
    let s = scheme_of(&[(vec![1, 0, 0, 0], vec![1, 0, 0, 0], vec![1, 0, 0, 0])]);
    assert_eq!(
        scheme_error(&s, 2),
        Ok(7),
        "AC4: a single e0⊗e0⊗e0 triple matches T_2 only at (0,0,0); error = 7"
    );
}

// --------------------------------------------------------------------------
// AC5 — Strassen gate (keystone)  [RED: drives the full target+reconstruct+
// error path, all unimplemented!()]
//
// SPEC-0006 §2.6/§6: the canonical 7-term Strassen 2×2 scheme reconstructs T_2
// with error 0 and is valid at rank 7. This is the keystone — its green is the
// confirmation that target + reconstruct + error are jointly correct.
// --------------------------------------------------------------------------

#[test]
fn ac5_strassen_reconstructs_t2_with_zero_error() {
    // AC5: the Phase-0 gate — scheme_error(strassen, 2) == Ok(0).
    assert_eq!(
        scheme_error(&strassen_scheme(), 2),
        Ok(0),
        "AC5 (keystone): the 7-term Strassen scheme must reconstruct T_2 exactly"
    );
}

#[test]
fn ac5_strassen_is_valid_at_rank_seven() {
    // AC5: valid at rank R = 7 (exactly 7 triples AND exact reconstruction).
    assert!(
        is_valid(&strassen_scheme(), 2, 7),
        "AC5 (keystone): Strassen is valid at rank 7"
    );
}

#[test]
fn ac5_strassen_has_seven_triples() {
    // AC5 support: the fixture itself is rank 7 (independent of the comp. path,
    // so this is GREEN — guards the fixture's shape).
    assert_eq!(
        strassen_scheme().rank(),
        7,
        "AC5: the Strassen fixture has exactly 7 triples"
    );
}

#[test]
fn ac5_strassen_not_valid_at_wrong_rank() {
    // AC5: is_valid is rank-sensitive — the correct scheme at the WRONG rank
    // (6) is invalid even though it reconstructs T_2.
    assert!(
        !is_valid(&strassen_scheme(), 2, 6),
        "AC5: a rank-7 scheme is not 'valid at rank 6'"
    );
}

// --------------------------------------------------------------------------
// AC6 — Naive baseline  [RED: drives reconstruct+target+error via is_valid]
//
// SPEC-0006 §2.6/§6: the naive R = n³ scheme reconstructs T_n exactly — n=2
// (R=8) and n=3 (R=27).
// --------------------------------------------------------------------------

#[test]
fn ac6_naive_n2_is_valid_at_rank_eight() {
    // AC6: naive n=2 scheme is valid at rank 8.
    assert!(
        is_valid(&naive_scheme(2), 2, 8),
        "AC6: the naive n=2 scheme is valid at rank n³ = 8"
    );
}

#[test]
fn ac6_naive_n3_is_valid_at_rank_27() {
    // AC6: naive n=3 scheme is valid at rank 27.
    assert!(
        is_valid(&naive_scheme(3), 3, 27),
        "AC6: the naive n=3 scheme is valid at rank n³ = 27"
    );
}

#[test]
fn ac6_naive_n2_has_eight_triples() {
    // AC6 support: the naive n=2 fixture is rank 8 (GREEN — guards the builder).
    assert_eq!(
        naive_scheme(2).rank(),
        8,
        "AC6: the naive n=2 builder yields n³ = 8 triples"
    );
}

#[test]
fn ac6_naive_n3_has_27_triples() {
    // AC6 support: the naive n=3 fixture is rank 27 (GREEN — guards the builder).
    assert_eq!(
        naive_scheme(3).rank(),
        27,
        "AC6: the naive n=3 builder yields n³ = 27 triples"
    );
}
