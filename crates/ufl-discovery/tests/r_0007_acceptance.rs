//! R-0007 — Tensor-Equality Predicate (the Hehner-discharge bridge):
//! end-to-end acceptance tests.
//!
//! One section per acceptance criterion (AC1–AC6), in order. Each test cites
//! its AC id in a `// ACk — …` comment so the architect (PR review) and the
//! orchestrator (status update) can map the suite to R-0007's acceptance
//! criteria mechanically.
//!
//! Authored at loop step 3 (test plan). TDD-red status: the scalar side of
//! the trait (`impl Predicate for Sexpr`, `State`, the routed
//! `check`/`check_str`) is REAL, while `RankDecomposition::discharge`
//! (`ufl_discovery`) is `unimplemented!()`. Concretely:
//!
//!  - Every test that drives `RankDecomposition::discharge` — directly or
//!    through the generic consumer — panics now and is RED: the AC1 tensor
//!    pair, all of AC3, the AC4 keystone pair, all of AC5, and the AC6 batch.
//!  - The scalar-instance tests (AC1 scalar pair, all of AC2) and the fixture
//!    guards (AC4 shape, AC6 mutation coverage) touch only real code and are
//!    GREEN now. Each is noted inline.
//!
//! AC2's "all 34 existing R-0004 tests pass unchanged" clause is verified by
//! running the workspace suite (those tests live in `ufl-predicate`), not by
//! duplicating them here.
//!
//! See:
//!  - `requirements/0007-tensor-predicate.md` — AC1–AC6
//!  - `specs/0007-tensor-predicate.md` — §2.1 (the `Predicate` trait), §2.2
//!    (scalar instance + guarded `State`), §2.3 (`RankDecomposition`), §2.5
//!    (fixture duplication), §3 (the generic consumer), §6 (the AC list with
//!    exact expected behaviours)

use ufl_core::{eval, Env, EvalError, Value};
use ufl_discovery::RankDecomposition;
use ufl_predicate::{check_str, CheckError, PredError, Predicate, State};
use ufl_syntax::{lower, read};
use ufl_tensor::{is_valid, Scheme, SchemeError, Triple};

// ---------------------------------------------------------------------------
// Generic consumers (SPEC-0007 §3) — the trait's first generic call sites.
// AC1 requires these to compile and run against BOTH instances (scalar +
// tensor); the instantiations live in the AC1 and AC6 sections.
// ---------------------------------------------------------------------------

/// AC6's generic batch helper — discharge `p` over every candidate, counting
/// the satisfied ones; the first typed error aborts the batch (SPEC-0007 §3).
fn discharge_all<P: Predicate>(p: &P, candidates: &[P::Candidate]) -> Result<usize, P::Error> {
    let mut satisfied = 0;
    for c in candidates {
        if p.discharge(c)? {
            satisfied += 1;
        }
    }
    Ok(satisfied)
}

/// AC1's error-reporting probe: a generic consumer can format a discharge
/// failure precisely because of the trait's `Error: std::error::Error` bound
/// (`to_string` via the `Display` supertrait, `source` via the trait itself).
fn discharge_failure_message<P: Predicate>(p: &P, candidate: &P::Candidate) -> Option<String> {
    p.discharge(candidate).err().map(|e| {
        let _ = std::error::Error::source(&e);
        e.to_string()
    })
}

// ---------------------------------------------------------------------------
// Tensor fixtures.
// ---------------------------------------------------------------------------

/// The canonical 7-term Strassen 2×2 scheme — the SPEC-0006 §2.6 table,
/// duplicated here as a literal (row-major `a11 a12 a21 a22`). Fixture
/// duplication per SPEC-0007 §2.5: `ufl-tensor`'s copy is a private fn in its
/// integration tests (unimportable); shared-fixture machinery is deferred
/// until a third consumer exists.
const STRASSEN: [([i8; 4], [i8; 4], [i8; 4]); 7] = [
    ([1, 0, 0, 1], [1, 0, 0, 1], [1, 0, 0, 1]),
    ([0, 0, 1, 1], [1, 0, 0, 0], [0, 0, 1, -1]),
    ([1, 0, 0, 0], [0, 1, 0, -1], [0, 1, 0, 1]),
    ([0, 0, 0, 1], [-1, 0, 1, 0], [1, 0, 1, 0]),
    ([1, 1, 0, 0], [0, 0, 0, 1], [-1, 1, 0, 0]),
    ([-1, 0, 1, 0], [1, 1, 0, 0], [0, 0, 0, 1]),
    ([0, 1, 0, -1], [0, 0, 1, 1], [1, 0, 0, 0]),
];

/// Build a `Scheme` from a coefficient table. Panics only on a *test-author*
/// mistake (an invalid fixture), never as part of the behaviour under test.
fn scheme_from(table: &[([i8; 4], [i8; 4], [i8; 4])]) -> Scheme {
    let mut s = Scheme::new();
    for (u, v, w) in table {
        let t = Triple::new(u.to_vec(), v.to_vec(), w.to_vec())
            .expect("fixture triple must be valid (test-author invariant)");
        s.push(t)
            .expect("fixture triple length must match scheme (test-author invariant)");
    }
    s
}

/// The exact Strassen scheme (rank 7, dim 4).
fn strassen_scheme() -> Scheme {
    scheme_from(&STRASSEN)
}

/// Strassen with ONE SIGN FLIPPED — triple 1's `u[0]`: `1 → -1`. Well-formed
/// (dim 4, rank 7) but wrong reconstruction: the delta tensor is
/// `-2·e₀⊗v₁⊗w₁ ≠ 0`. This is AC4's `Ok(false)` side of the AC4/AC5 input
/// partition (dim-malformed inputs are AC5's domain).
fn broken_strassen_scheme() -> Scheme {
    let mut table = STRASSEN;
    table[0].0[0] = -1;
    scheme_from(&table)
}

/// The naive `R = n³` scheme (SPEC-0006 §2.6): one triple per `(i,j,k)` with
/// `u = e_{i·n+j}`, `v = e_{j·n+k}`, `w = e_{i·n+k}`. Reconstructs `T_n`
/// exactly at rank `n³` — AC3's right-tensor-wrong-rank probe.
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

/// A single `e₀⊗e₀⊗e₀` triple of length `d`: well-formed, rank 1, dim `d`.
/// At `d = 4` it is dim-consistent with `n = 2` but reconstructs a lone 1 at
/// `(0,0,0)` (not `T_2`); at `d = 9` it is the dim-mismatch probe.
fn unit_triple_scheme(d: usize) -> Scheme {
    let mut e = vec![0i8; d];
    e[0] = 1;
    let mut s = Scheme::new();
    let t = Triple::new(e.clone(), e.clone(), e).expect("unit fixture triple must be valid");
    s.push(t)
        .expect("first push always matches the empty scheme");
    s
}

/// Every single-entry mutation of the Strassen table: for each of the
/// `7 × 3 × 4 = 84` coefficient slots, both alternative values in
/// `{-1, 0, +1}` — 168 deterministic variants, each differing from the exact
/// scheme in exactly one entry (so each stays well-formed: dim 4, rank 7).
fn mutated_strassen_variants() -> Vec<Scheme> {
    let mut variants = Vec::new();
    for (t, (u, v, w)) in STRASSEN.iter().enumerate() {
        for (vec_sel, vector) in [u, v, w].into_iter().enumerate() {
            for (pos, &current) in vector.iter().enumerate() {
                for alt in [-1i8, 0, 1] {
                    if alt == current {
                        continue;
                    }
                    let mut table = STRASSEN;
                    let slot = match vec_sel {
                        0 => &mut table[t].0,
                        1 => &mut table[t].1,
                        _ => &mut table[t].2,
                    };
                    slot[pos] = alt;
                    variants.push(scheme_from(&table));
                }
            }
        }
    }
    variants
}

// ---------------------------------------------------------------------------
// Scalar fixtures (the R-0004 oracle pattern).
// ---------------------------------------------------------------------------

/// A real `Value` `q + 0i`.
fn real(q: f64) -> Value {
    Value::new(q, 0.0)
}

/// Evaluate a *numeric* eml form under `env` — the same path `=`'s operands
/// take internally (`read → lower → ufl_core::eval`). Binding `x'` to exactly
/// this result makes `(= x' (eml x 1))` an exact equality (the R-0004 AC5
/// oracle pattern; no literal-`e` mismatch).
fn eval_eml(src: &str, env: &Env) -> Value {
    let s = read(src).expect("eml oracle text should read");
    let e = lower(&s).expect("eml oracle text should lower");
    eval(&e, env).expect("eml oracle should evaluate")
}

// ===========================================================================
// AC1 — A predicate is a dischargeable property.
//
// "`ufl-predicate` exposes `Predicate` (`Candidate` sized, `Error:
// std::error::Error`, `discharge -> Result<bool, Error>`); total within the
// SPEC-0006 §2.5 envelope; typed errors, no panic." (R-0007 AC1; SPEC-0007
// §2.1, §6 AC1.) The generic consumers above compile against the trait alone;
// here they RUN against both instances. The no-panic half of the contract is
// exercised by AC5 (malformed candidates discharge to `Err`, and a panic
// would fail those tests).
// ===========================================================================

#[test]
fn ac1_generic_consumer_counts_scalar_batch() {
    // AC1 — [GREEN now] `discharge_all` instantiated with the SCALAR instance
    // (`P = Sexpr`, candidates = `State`s): `(= x 1)` is satisfied by exactly
    // the two states binding x = 1 (exact `=`, SPEC-0004 §2.4).
    let predicate = read("(= x 1)").expect("predicate text should read");
    let states = vec![
        State::new(&[("x", real(1.0))], &[]).expect("state should build"),
        State::new(&[("x", real(2.0))], &[]).expect("state should build"),
        State::new(&[("x", real(1.0))], &[]).expect("state should build"),
        State::new(&[("x", real(0.0))], &[]).expect("state should build"),
    ];
    assert_eq!(
        discharge_all(&predicate, &states),
        Ok(2),
        "AC1: the generic consumer must count exactly the two x = 1 states"
    );
}

#[test]
fn ac1_scalar_discharge_error_is_typed_and_formats() {
    // AC1 — [GREEN now] a malformed/undischargeable case on the scalar
    // instance is a TYPED error through the trait, never a panic: `(= y 1)`
    // with `y` unbound → `CheckError::Pred(Eval(UnboundVariable))`.
    let state = State::new(&[], &[]).expect("empty state should build");
    let unbound = read("(= y 1)").expect("predicate text should read");
    assert_eq!(
        unbound.discharge(&state),
        Err(CheckError::Pred(PredError::Eval(
            EvalError::UnboundVariable("y".to_string())
        ))),
        "AC1: an unbound variable must surface as the typed CheckError"
    );
    // And it is formattable through the generic `Error: std::error::Error`
    // bound (the bound is what lets a generic consumer report failures).
    let msg = discharge_failure_message(&unbound, &state)
        .expect("the unbound-variable discharge must fail");
    assert!(
        !msg.is_empty(),
        "AC1: the typed error must format to a non-empty message via to_string"
    );
}

#[test]
fn ac1_generic_consumer_counts_tensor_batch() {
    // AC1 — [RED: drives RankDecomposition::discharge, unimplemented!()]
    // `discharge_all` instantiated with the TENSOR instance: of [exact
    // Strassen, broken Strassen], exactly one candidate satisfies P_{{2,7}}.
    let p27 = RankDecomposition::new(2, 7);
    let candidates = vec![strassen_scheme(), broken_strassen_scheme()];
    assert_eq!(
        discharge_all(&p27, &candidates),
        Ok(1),
        "AC1: the generic consumer must count only the exact Strassen scheme"
    );
}

#[test]
fn ac1_tensor_discharge_error_is_typed_and_formats() {
    // AC1 — [RED: drives RankDecomposition::discharge, unimplemented!()] the
    // tensor instance's typed error formats through the same generic bound...
    let p27 = RankDecomposition::new(2, 7);
    let msg = discharge_failure_message(&p27, &unit_triple_scheme(9))
        .expect("a dim-9 candidate against n = 2 must fail discharge");
    assert!(
        !msg.is_empty(),
        "AC1: the typed SchemeError must format to a non-empty message"
    );
    // ...and the generic consumer propagates it as the batch's typed result.
    let batch = vec![strassen_scheme(), unit_triple_scheme(9)];
    assert_eq!(
        discharge_all(&p27, &batch),
        Err(SchemeError::DimMismatch {
            n: 2,
            expected: 4,
            got: 9
        }),
        "AC1: discharge_all must propagate the typed error, not mask it"
    );
}

// ===========================================================================
// AC2 — Scalar predicate is an instance.  [ALL GREEN now: the scalar side of
// the trait is real.]
//
// "`Sexpr` implements `Predicate` over `State`; `check`/`check_str` route
// through `State::new` + `discharge`; all existing R-0004 tests pass
// unchanged; the ReservedName guard holds on the trait path." (R-0007 AC2;
// SPEC-0007 §2.2, §6 AC2.) The "R-0004 tests unchanged" clause is verified by
// the workspace run (5 unit + 29 e2e in `ufl-predicate`).
// ===========================================================================

#[test]
fn ac2_trait_discharge_agrees_with_check_str_true_case() {
    // AC2 — [GREEN now] the trait path (`read(..).discharge(State::new(..))`)
    // and `check_str` agree on the SAME inputs, true case: post `x'` is bound
    // to the eml-computed oracle, so `(= x' (eml x 1))` holds exactly.
    let mut pre_env = Env::new();
    pre_env.bind("x", real(1.0));
    let expected_post = eval_eml("(eml x 1)", &pre_env);

    let pre = [("x", real(1.0))];
    let post = [("x", expected_post)];

    let via_trait = read("(= x' (eml x 1))")
        .expect("predicate text should read")
        .discharge(&State::new(&pre, &post).expect("state should build"));
    let via_check_str = check_str("(= x' (eml x 1))", &pre, &post);

    assert_eq!(
        via_trait,
        Ok(true),
        "AC2: the trait path must discharge true for the correct post-state"
    );
    assert_eq!(
        via_trait, via_check_str,
        "AC2: the trait path and check_str must agree (check_str routes through discharge)"
    );
}

#[test]
fn ac2_trait_discharge_agrees_with_check_str_false_case() {
    // AC2 — [GREEN now] same agreement on the false case: a deliberately
    // wrong post-state (oracle + 1) discharges `Ok(false)` on both paths.
    let mut pre_env = Env::new();
    pre_env.bind("x", real(1.0));
    let wrong_post = eval_eml("(eml x 1)", &pre_env) + real(1.0);

    let pre = [("x", real(1.0))];
    let post = [("x", wrong_post)];

    let via_trait = read("(= x' (eml x 1))")
        .expect("predicate text should read")
        .discharge(&State::new(&pre, &post).expect("state should build"));
    let via_check_str = check_str("(= x' (eml x 1))", &pre, &post);

    assert_eq!(
        via_trait,
        Ok(false),
        "AC2: the trait path must discharge false for a wrong post-state"
    );
    assert_eq!(
        via_trait, via_check_str,
        "AC2: the trait path and check_str must agree on the false case too"
    );
}

#[test]
fn ac2_reserved_name_guard_holds_on_trait_path() {
    // AC2 — [GREEN now] the ReservedName guard lives inside the candidate's
    // ONLY constructor (SPEC-0007 §2.2), so the trait path cannot bypass
    // SPEC-0004 §2.5: a primed binding name is rejected before any discharge.
    assert_eq!(
        State::new(&[("x'", real(1.0))], &[]).err(),
        Some(CheckError::ReservedName("x'".to_string())),
        "AC2: a primed PRE binding name must be ReservedName at State::new"
    );
    assert_eq!(
        State::new(&[], &[("x'", real(1.0))]).err(),
        Some(CheckError::ReservedName("x'".to_string())),
        "AC2: a primed POST binding name must be ReservedName at State::new"
    );
}

// ===========================================================================
// AC3 — The tensor predicate is exactly `is_valid` on dim-consistent schemes.
// [ALL RED: every test drives RankDecomposition::discharge, unimplemented!()]
//
// "On dim-consistent schemes, `RankDecomposition::discharge ==
// Ok(is_valid(scheme, n, rank))` (exact, wrong-reconstruction, and wrong-rank
// samples); on dim-mismatched schemes it is `Err` where `is_valid` is
// `false`." (R-0007 AC3; SPEC-0007 §2.3's precise relation, §6 AC3.)
// ===========================================================================

#[test]
fn ac3_agrees_with_is_valid_on_exact_strassen() {
    // AC3 — the exact sample: Strassen at (n=2, R=7) is valid, and discharge
    // must say exactly what is_valid says.
    let s = strassen_scheme();
    assert!(
        is_valid(&s, 2, 7),
        "fixture sanity: Strassen IS valid at rank 7 (R-0006)"
    );
    assert_eq!(
        RankDecomposition::new(2, 7).discharge(&s),
        Ok(is_valid(&s, 2, 7)),
        "AC3: discharge must equal Ok(is_valid) on the exact scheme"
    );
}

#[test]
fn ac3_agrees_with_is_valid_on_wrong_reconstruction() {
    // AC3 — the wrong-reconstruction sample: dim-consistent (dim 4 = n²) and
    // the rank field matches (1), but e₀⊗e₀⊗e₀ reconstructs a lone 1 — not
    // T_2 — so both sides say false.
    let s = unit_triple_scheme(4);
    assert!(
        !is_valid(&s, 2, 1),
        "fixture sanity: a wrong reconstruction is invalid (R-0006)"
    );
    let got = RankDecomposition::new(2, 1).discharge(&s);
    assert_eq!(
        got,
        Ok(false),
        "AC3: a dim-consistent wrong reconstruction must discharge Ok(false)"
    );
    assert_eq!(
        got,
        Ok(is_valid(&s, 2, 1)),
        "AC3: discharge must equal Ok(is_valid) on the wrong-reconstruction sample"
    );
}

#[test]
fn ac3_agrees_with_is_valid_on_wrong_rank() {
    // AC3 — the wrong-rank sample: the naive n=2 scheme (R = 8) reconstructs
    // T_2 EXACTLY, so at rank field 7 only the rank conjunct fails — right
    // tensor, wrong rank ⇒ Ok(false), matching is_valid.
    let s = naive_scheme(2);
    assert!(
        !is_valid(&s, 2, 7),
        "fixture sanity: naive-8 is not valid AT RANK 7 (R-0006)"
    );
    assert!(
        is_valid(&s, 2, 8),
        "fixture sanity: naive-8 IS valid at its true rank 8 (R-0006)"
    );
    let at_seven = RankDecomposition::new(2, 7).discharge(&s);
    assert_eq!(
        at_seven,
        Ok(false),
        "AC3: exact reconstruction at the wrong rank must discharge Ok(false)"
    );
    assert_eq!(
        at_seven,
        Ok(is_valid(&s, 2, 7)),
        "AC3: discharge must equal Ok(is_valid) on the wrong-rank sample"
    );
    // Contrast: the same scheme at its true rank discharges true.
    assert_eq!(
        RankDecomposition::new(2, 8).discharge(&s),
        Ok(true),
        "AC3: the same scheme at its true rank (8) must discharge Ok(true)"
    );
}

#[test]
fn ac3_dim_mismatch_is_err_where_is_valid_is_false() {
    // AC3 — the partition sentence: on a dim-mismatched scheme `is_valid`
    // collapses to `false` but discharge is `Err` (honest, not a silent
    // false). Rank field 1 = the scheme's actual rank, so the ONLY defect is
    // the dim. (The regardless-of-rank-field strengthening is AC5's.)
    let s = unit_triple_scheme(9);
    assert!(
        !is_valid(&s, 2, 1),
        "fixture sanity: dim-9 vs n=2 is invalid (R-0006 collapses it to false)"
    );
    assert!(
        RankDecomposition::new(2, 1).discharge(&s).is_err(),
        "AC3: a dim-mismatched scheme must discharge Err where is_valid is false"
    );
}

// ===========================================================================
// AC4 — Strassen through the predicate (the keystone).
//
// "`RankDecomposition::new(2, 7).discharge(strassen) == Ok(true)`; a broken
// scheme — well-formed dim, wrong reconstruction (one sign flipped) —
// discharges Ok(false). AC4 and AC5 partition the inputs." (R-0007 AC4;
// SPEC-0007 §6 AC4.)
// ===========================================================================

#[test]
fn ac4_strassen_fixture_shape() {
    // AC4 — [GREEN now: touches only real Scheme accessors] guards the
    // duplicated SPEC-0006 §2.6 literal: rank 7, dim 4, and the broken
    // variant is well-formed but genuinely different.
    let exact = strassen_scheme();
    assert_eq!(exact.rank(), 7, "AC4: the Strassen fixture has 7 triples");
    assert_eq!(exact.dim(), Some(4), "AC4: the Strassen fixture has dim 4");
    let broken = broken_strassen_scheme();
    assert_eq!(broken.rank(), 7, "AC4: the broken variant keeps rank 7");
    assert_eq!(broken.dim(), Some(4), "AC4: the broken variant keeps dim 4");
    assert_ne!(
        exact, broken,
        "AC4: the sign flip must actually change the scheme"
    );
}

#[test]
fn ac4_strassen_discharges_true() {
    // AC4 — [RED: drives RankDecomposition::discharge, unimplemented!()] the
    // keystone, now via the predicate layer: P_{{2,7}}(strassen) holds.
    assert_eq!(
        RankDecomposition::new(2, 7).discharge(&strassen_scheme()),
        Ok(true),
        "AC4 (keystone): Strassen must discharge Ok(true) through the predicate layer"
    );
}

#[test]
fn ac4_broken_strassen_discharges_false() {
    // AC4 — [RED: drives RankDecomposition::discharge, unimplemented!()] one
    // flipped sign: well-formed dim (4) and rank (7), wrong reconstruction ⇒
    // the predicate is decidedly false — Ok(false), never Err (AC5 owns the
    // malformed side of the partition).
    assert_eq!(
        RankDecomposition::new(2, 7).discharge(&broken_strassen_scheme()),
        Ok(false),
        "AC4: a well-formed scheme with a wrong reconstruction must be Ok(false)"
    );
}

// ===========================================================================
// AC5 — Honest discharge, not a wrapper that hides errors.
// [ALL RED: every test drives RankDecomposition::discharge, unimplemented!()]
//
// "A dim/n mismatch (including an empty scheme vs n ≥ 1) discharges to
// `Err(SchemeError::DimMismatch)` REGARDLESS OF THE RANK FIELD — never a
// panic or a silent false. The n = 0 vacuous case is pinned." (R-0007 AC5;
// SPEC-0007 §2.3, §6 AC5.) A panic would fail these tests, so they are also
// AC1's no-panic evidence on malformed candidates.
// ===========================================================================

#[test]
fn ac5_dim_mismatch_is_err_regardless_of_rank_field() {
    // AC5 — a dim-9 scheme (rank 1) against n = 2 must be the SAME typed
    // error at rank fields 7 and 3 (≠ scheme rank) AND 1 (= scheme rank):
    // the error contract must not flip on the unrelated rank conjunct (the
    // three-lens review's blocking short-circuit finding, SPEC-0007 §2.3).
    let s = unit_triple_scheme(9);
    let expected_err = SchemeError::DimMismatch {
        n: 2,
        expected: 4,
        got: 9,
    };
    for rank in [7usize, 3, 1] {
        assert_eq!(
            RankDecomposition::new(2, rank).discharge(&s),
            Err(expected_err.clone()),
            "AC5: dim-9 vs n=2 must be Err(DimMismatch) at rank field {rank}"
        );
    }
}

#[test]
fn ac5_empty_scheme_is_err_regardless_of_rank_field() {
    // AC5 — the empty scheme (dim None, treated as got 0) against n = 2 is a
    // DimMismatch at every rank field — never a panic or a silent false.
    let expected_err = SchemeError::DimMismatch {
        n: 2,
        expected: 4,
        got: 0,
    };
    for rank in [7usize, 3] {
        assert_eq!(
            RankDecomposition::new(2, rank).discharge(&Scheme::new()),
            Err(expected_err.clone()),
            "AC5: the empty scheme vs n=2 must be Err(DimMismatch) at rank field {rank}"
        );
    }
}

#[test]
fn ac5_n0_vacuous_empty_scheme_discharges_true() {
    // AC5 — the n = 0 vacuous case, pinned: target(0) is the empty dim-0
    // tensor, the empty scheme reconstructs to dim 0 with rank 0, so P_{{0,0}}
    // holds vacuously. `n ≥ 1` is the REAL domain (the SPEC-0006 §2.5
    // envelope); this pin makes any future n-guard a conscious, visible
    // change rather than a silent one.
    assert_eq!(
        RankDecomposition::new(0, 0).discharge(&Scheme::new()),
        Ok(true),
        "AC5: the n = 0 vacuous case is pinned as Ok(true)"
    );
}

// ===========================================================================
// AC6 — The discovery engine can discharge it (structural frugality).
//
// "T_n is computed once per RankDecomposition (in `new`), not per discharge;
// discharge allocates at most the one reconstruction buffer. Verified by
// construction (the cached field — code review + the `new` signature) plus
// the GENERIC discharge_all batch test asserting outcome counts. No timing
// assertion." (R-0007 AC6; SPEC-0007 §2.3, §3, §6 AC6.) Only behaviour is
// asserted here, deterministically — no wall clock by design.
// ===========================================================================

#[test]
fn ac6_mutation_fixture_covers_every_entry() {
    // AC6 — [GREEN now: touches only real fixture code] the mutation fixture
    // is exhaustive and well-formed: 7 triples × 3 vectors × 4 positions × 2
    // alternative values = 168 variants, every one rank 7 / dim 4 and ≠ the
    // exact scheme. No fixture vector is all-zero, which is the precondition
    // for the delta argument below (a single-entry change of u_t shifts the
    // reconstruction by (new−old)·e_p⊗v_t⊗w_t ≠ 0, and symmetrically for
    // v_t/w_t — so EVERY variant must discharge false).
    for (u, v, w) in &STRASSEN {
        for vector in [u, v, w] {
            assert!(
                vector.iter().any(|&c| c != 0),
                "AC6: no Strassen fixture vector is all-zero (delta-argument precondition)"
            );
        }
    }
    let variants = mutated_strassen_variants();
    assert_eq!(
        variants.len(),
        168,
        "AC6: single-entry mutation must cover 7·3·4·2 = 168 variants"
    );
    let exact = strassen_scheme();
    for variant in &variants {
        assert_eq!(variant.rank(), 7, "AC6: every variant keeps rank 7");
        assert_eq!(variant.dim(), Some(4), "AC6: every variant keeps dim 4");
        assert_ne!(
            *variant, exact,
            "AC6: every variant differs from the exact scheme"
        );
    }
}

#[test]
fn ac6_generic_batch_counts_only_exact_schemes() {
    // AC6 — [RED: drives RankDecomposition::discharge, unimplemented!()] ONE
    // predicate instance across a 200-candidate batch: T_2 is computed once
    // in `new` (the cached `target` field — structural frugality by
    // construction, SPEC-0007 §6) and `discharge` allocates only the
    // per-candidate reconstruction. All 168 mutants are well-formed (dim 4,
    // rank 7) with a provably wrong reconstruction (delta ≠ 0, see the
    // fixture-guard test), so the GENERIC consumer must count exactly the 32
    // exact copies — no Err ever trips the batch.
    let p27 = RankDecomposition::new(2, 7);
    let mut batch = mutated_strassen_variants();
    batch.extend(std::iter::repeat_with(strassen_scheme).take(32));
    assert_eq!(batch.len(), 200, "AC6: the batch is 200 candidates");
    assert_eq!(
        discharge_all(&p27, &batch),
        Ok(32),
        "AC6: only the exact Strassen copies satisfy P_{{2,7}}"
    );
}
