//! R-0014 **AC3** / SPEC-0014N — the literal `eml`-NAND tree run through
//! `ufl-core`, plus the integer-regime probe.
//!
//! Closes the one **"Owed"** row in `theory/universal-computability.md` §6: the
//! NAND embedding had been verified *semantically* (`experiments/nand-embedding.py`)
//! but the literal tree had never been evaluated by the real complex evaluator.
//!
//! **Claim discipline (§3):** combinational/functional universality only. Not
//! control universality, not self-hosting, and **not** "poly-size circuit ⇒
//! poly-size tree" (false — a tree is a *formula*; see `size_laws`).

mod nand;

use nand::*;
use ufl_core::{eval, Eml, Env, Value};

fn ev(t: &Eml) -> Value {
    eval(t, &Env::new()).expect("the builders are closed — no unbound variables")
}

/// T-primitives — every §1 encoding against its closed form, **and the measured
/// domain cliffs as boundary cases** (the cliffs are real: these are not
/// unconditional identities).
#[test]
fn primitives_and_their_domain_cliffs() {
    let e = std::f64::consts::E;
    assert_eq!(ev(&e_t()).re, e, "e = eml(1,1)");
    assert_eq!(ev(&zero_t()).re, 0.0, "0 = e − e, exactly");
    assert_eq!(ev(&exp_t(Eml::one())).re, e, "exp(1)");
    assert_eq!(ev(&ln_t(Eml::one())).re, 0.0, "ln 1 = 0 exactly");
    assert_eq!(ev(&ln_t(e_t())).re, 1.0, "ln e = 1 exactly");
    assert!(
        ev(&ln_t(zero_t())).re.is_infinite(),
        "ln 0 = −∞ (R-0001 AC3)"
    );
    assert_eq!(ev(&sub_t(Eml::one(), Eml::one())).re, 0.0, "1 − 1");
    assert_eq!(ev(&neg_t(Eml::one())).re, -1.0, "−1");
    assert_eq!(ev(&add_t(Eml::one(), Eml::one())).re, 2.0, "1 + 1");
    assert_eq!(ev(&mul_t(Eml::one(), Eml::one())).re, 1.0, "1 × 1");
    assert_eq!(ev(&mul_t(zero_t(), Eml::one())).re, 0.0, "0 × 1");

    // The cliffs need a free operand, so they bind `k` (the honest way to inject
    // a constant — not pretending it is `eml`-constructed).
    let k = || Eml::var("k");

    // Cliff 1 — `sub`/`neg`/`add`: the inner `exp(b)` overflows past ln(f64::MAX).
    let big_ok = ev_with(&sub_t(Eml::one(), k()), &[("k", 709.78)]);
    let big_no = ev_with(&sub_t(Eml::one(), k()), &[("k", 709.79)]);
    assert!(big_ok.re.is_finite(), "b = 709.78 is inside the domain");
    assert!(
        !big_no.re.is_finite() || big_no.im.is_nan(),
        "b = 709.79 overflows the inner exp"
    );

    // Cliff 2 — `ln`: the inner `e^e/y` overflows below e^e/f64::MAX ≈ 8.4298e-308.
    // (An earlier draft said 1.512e-308 — that was the *old* ln encoding's
    // intermediate, and is itself subnormal. Re-measured; BOTH sides asserted.)
    let ln_ok = ev_with(&ln_t(k()), &[("k", 8.43e-308)]);
    let ln_no = ev_with(&ln_t(k()), &[("k", 8.42e-308)]);
    assert!(
        ln_ok.re.is_finite(),
        "y = 8.43e-308 is inside the ln domain"
    );
    assert!(
        !ln_no.re.is_finite() || ln_no.im.is_nan(),
        "y = 8.42e-308 holes — the cliff sits ABOVE f64::MIN_POSITIVE"
    );
}

fn ev_with(t: &Eml, bindings: &[(&str, f64)]) -> Value {
    let mut env = Env::new();
    for (k, v) in bindings {
        env.bind(*k, Value::new(*v, 0.0));
    }
    eval(t, &env).expect("bound")
}

/// **T-truth-table (the AC3 gate)** — the four rows, **bit-exact**: `Re` equals
/// `{1,1,1,0}` exactly and `Im` is exactly `0.0`. Not ε-hedged: a nonzero `Im`
/// would mean the branch cut (R-0001 AC4) leaked into a real-valued Boolean
/// computation, which is a finding, not a rounding detail.
#[test]
fn the_nand_truth_table_is_bit_exact() {
    for (a, b, want) in [
        (false, false, 1.0),
        (false, true, 1.0),
        (true, false, 1.0),
        (true, true, 0.0),
    ] {
        let v = ev(&nand_t(bit(a), bit(b)));
        assert_eq!(v.re, want, "NAND({a},{b}).re");
        assert_eq!(v.im, 0.0, "NAND({a},{b}).im must be exactly 0");
        assert!(!v.im.is_sign_negative(), "no −0.0 in the imaginary part");
    }
}

/// **T-zero-edge** — `ln 0 = −∞` with no trap, and the property that actually
/// carries the result (§2.2): **no intermediate is `NaN`** *on the Boolean
/// domain*. The result `1` arrives by **cancellation of complex infinities**
/// (measured: 14 infinite intermediates), not by "absorption".
///
/// The scope matters — off `{0,1}` the no-NaN claim is false and asymmetric:
/// `nand_t(1e-310, 0)` is `(NaN, NaN)` while `nand_t(0, 1e-310)` is `1.0`.
#[test]
fn the_zero_path_is_nan_free_on_the_boolean_domain() {
    let env = Env::new();
    for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
        let tree = nand_t(bit(a), bit(b));
        assert!(eval(&tree, &env).is_ok(), "eval is total here");
        let mut vals = Vec::new();
        subtree_values(&tree, &env, &mut vals);
        let nans = vals
            .iter()
            .filter(|v| v.re.is_nan() || v.im.is_nan())
            .count();
        let infs = vals
            .iter()
            .filter(|v| !v.re.is_finite() || !v.im.is_finite())
            .count();
        assert_eq!(
            nans, 0,
            "NAND({a},{b}): no NaN intermediate — the load-bearing property"
        );
        if !a && !b {
            assert!(infs > 0, "the 0-path really does route through infinities");
        }
    }
}

/// **T-depth-exact (§1.2 — the strongest result)**. `{0,1}` is a closed exact
/// orbit, so exactness survives composition: `NOT^k` is bit-exact at every `k`,
/// and XOR (3 layers, 4 NANDs) is bit-exact on all rows. Plus the
/// self-correction witness: from `true = 0.99`, `NOT^8` returns bit-exact `1.0`.
#[test]
fn exactness_survives_composition_depth() {
    // NOT^k on both rails, k = 1..=6 (k=7,8 are large but debug-fine; kept to 6
    // here so the fast suite stays brisk — the induction is depth-independent).
    for start in [false, true] {
        let mut t = bit(start);
        let mut want = start;
        for k in 1..=6 {
            t = not_t(t);
            want = !want;
            let v = ev(&t);
            let expect = if want { 1.0 } else { 0.0 };
            assert_eq!(v.re, expect, "NOT^{k}({start}).re bit-exact");
            assert_eq!(v.im, 0.0, "NOT^{k}({start}).im exactly 0");
        }
    }

    // XOR = NAND(NAND(a, NAND(a,b)), NAND(b, NAND(a,b))) — a genuinely
    // multi-layer, non-chain circuit shape.
    for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
        let n = |x: Eml, y: Eml| nand_t(x, y);
        let ab = n(bit(a), bit(b));
        let xor = n(n(bit(a), ab.clone()), n(bit(b), ab));
        let v = ev(&xor);
        assert_eq!(v.re, if a ^ b { 1.0 } else { 0.0 }, "XOR({a},{b})");
        assert_eq!(v.im, 0.0, "XOR keeps Im exactly 0");
    }
}

/// The self-correction witness (§1.2): `{0,1}` is a **superattracting 2-cycle**
/// of `x ↦ 1 − x²`, so a perturbed rail is pulled back. From `true = 0.99`,
/// eight NOTs return **bit-exact** `1.0`.
#[test]
fn the_encoding_self_corrects_a_perturbed_rail() {
    let mut t = Eml::var("x");
    for _ in 0..8 {
        t = not_t(t);
    }
    let v = ev_with(&t, &[("x", 0.99)]);
    assert_eq!(v.re, 1.0, "NOT^8(0.99) returns bit-exact 1.0");
}

/// **T-functional-completeness** — `NOT`/`AND`/`OR` derived from the *same* NAND
/// combinator each reproduce their truth tables. Bounded evidence for the §3
/// composition claim: **combinational only** — this is not control universality.
#[test]
fn nand_is_functionally_complete_for_combinational_logic() {
    for a in [false, true] {
        let v = ev(&not_t(bit(a)));
        assert_eq!(v.re, if !a { 1.0 } else { 0.0 }, "NOT({a})");
    }
    for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
        assert_eq!(
            ev(&and_t(bit(a), bit(b))).re,
            if a && b { 1.0 } else { 0.0 },
            "AND"
        );
        assert_eq!(
            ev(&or_t(bit(a), bit(b))).re,
            if a || b { 1.0 } else { 0.0 },
            "OR"
        );
    }
}

/// §3's size laws, measured — the basis for the **formula-not-circuit** non-goal.
/// A `G`-gate NAND formula is `50·G + 1` nodes; chained NOT doubles because
/// `NOT a = NAND(a,a)` duplicates its argument (a property of the *NAND
/// presentation*, not of `Eml`).
#[test]
fn size_laws() {
    assert_eq!(
        nodes(&nand_t(Eml::one(), Eml::one())),
        51,
        "NAND = 50·1 + 1"
    );
    assert_eq!(nodes(&and_t(Eml::one(), Eml::one())), 151, "AND = 50·3 + 1");
    assert_eq!(nodes(&or_t(Eml::one(), Eml::one())), 151, "OR = 50·3 + 1");
    assert_eq!(
        depth(&nand_t(Eml::one(), Eml::one())),
        13,
        "eml-depth = 10·g + 3"
    );

    // Chained NOT: 50·2^g − 49, exactly.
    let mut t = Eml::one();
    for g in 1..=6usize {
        t = not_t(t);
        assert_eq!(nodes(&t), 50 * (1 << g) - 49, "NOT^{g} nodes");
        assert_eq!(depth(&t), 10 * g + 3, "NOT^{g} depth");
    }
}

/// **T-integer-probe (§2.4)** — the honest half. Does `eml` over `Complex<f64>`
/// carry the **exact** integer regime the matmul verifier requires?
///
/// Pre-registered three-way verdict (exact / **ulp-bounded** / leak), reported
/// with the **bit-exact count** as the headline — never a bare pass/fail, and
/// never an absolute ε (which is the wrong metric on a multiplicative substrate,
/// and is undefined where the expected value is 0).
#[test]
fn the_integer_regime_probe() {
    // The verifier's real shape: c = Σ_t u_t·v_t·w_t over {−1,0,1}, 7 terms.
    let coeffs = [-1i64, 0, 1];
    let mut total = 0usize;
    let mut bit_exact = 0usize;
    let mut worst_nonzero_ulp = 0i64;
    let mut worst_zero_abs = 0.0f64;
    let mut leaks = 0usize;
    let mut round_recovers = 0usize;

    let mut seed = 0xA53Cu64; // pinned — an unspecified draw is not reproducible
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        coeffs[(seed % 3) as usize]
    };

    for _ in 0..200 {
        let terms: Vec<(i64, i64, i64)> = (0..7).map(|_| (next(), next(), next())).collect();
        let want: i64 = terms.iter().map(|(u, v, w)| u * v * w).sum();

        // Build Σ_t (u·v)·w as one eml tree, binding the coefficients.
        let mut bindings: Vec<(String, f64)> = Vec::new();
        let mut sum: Option<Eml> = None;
        for (i, (u, v, w)) in terms.iter().enumerate() {
            let (ku, kv, kw) = (format!("u{i}"), format!("v{i}"), format!("w{i}"));
            bindings.push((ku.clone(), *u as f64));
            bindings.push((kv.clone(), *v as f64));
            bindings.push((kw.clone(), *w as f64));
            let prod = mul_t(mul_t(Eml::var(ku), Eml::var(kv)), Eml::var(kw));
            sum = Some(match sum {
                None => prod,
                Some(acc) => add_t(acc, prod),
            });
        }
        let mut env = Env::new();
        for (k, v) in &bindings {
            env.bind(k.clone(), Value::new(*v, 0.0));
        }
        let got = match eval(&sum.expect("7 terms"), &env) {
            Ok(v) => v,
            Err(_) => {
                leaks += 1;
                continue;
            }
        };
        total += 1;

        if got.re.is_nan() || got.im.is_nan() || !got.re.is_finite() {
            leaks += 1;
            continue;
        }
        if got.re == want as f64 && got.im == 0.0 {
            bit_exact += 1;
        }
        if got.re.round() as i64 == want {
            round_recovers += 1;
        }
        let err = (got.re - want as f64).abs();
        if want == 0 {
            worst_zero_abs = worst_zero_abs.max(err.max(got.im.abs()));
        } else {
            let ulp = (want as f64).abs() * f64::EPSILON;
            let in_ulps = (err / ulp).ceil() as i64;
            worst_nonzero_ulp = worst_nonzero_ulp.max(in_ulps);
        }
    }

    // The result is the deliverable — printed unconditionally ("no silent middle").
    println!("--- R-0014 AC3 integer-regime probe (7-term verifier shape, 200 draws) ---");
    println!("  bit-exact           : {bit_exact}/{total}");
    println!("  round() recovers i64: {round_recovers}/{total}");
    println!("  worst nonzero-expect: {worst_nonzero_ulp} ulp");
    println!("  worst zero-expect   : {worst_zero_abs:.3e} absolute");
    println!("  leaks (NaN/inf)     : {leaks}");

    assert_eq!(leaks, 0, "no NaN/inf leak in the {{−1,0,1}} regime");
    assert!(total > 0, "the probe actually ran");
    // The measured verdict: ulp-bounded, NOT exact. Both halves are asserted so a
    // future drift toward *either* extreme is a loud failure.
    assert!(
        bit_exact < total,
        "if this ever becomes exact, the ledger row must be rewritten — eml would \
         then carry the discrete regime"
    );
    assert_eq!(
        round_recovers, total,
        "the leak is REVERSIBLE: round() recovers the exact i64 in every case"
    );
}

/// The silently-wrong-and-finite case — the worst failure mode, named rather
/// than left to be discovered: `eml`'s product is not bit-exact on small
/// integers, so a naive `as i64` **truncates**.
#[test]
fn the_integer_leak_is_silently_wrong_under_truncation() {
    let t = mul_t(Eml::var("a"), Eml::one());
    let v = ev_with(&t, &[("a", 3.0)]);
    assert_ne!(v.re, 3.0, "3 × 1 is not bit-exact");
    assert_eq!(
        v.re as i64, 2,
        "and `as i64` truncates it to 2 — the silent failure"
    );
    assert_eq!(v.re.round() as i64, 3, "round() is the correct readout");

    // Not bit-commutative at the ±0.0 boundary, either.
    let neg_zero = ev_with(
        &mul_t(Eml::var("a"), Eml::var("b")),
        &[("a", -1.0), ("b", 0.0)],
    );
    let pos_zero = ev_with(
        &mul_t(Eml::var("a"), Eml::var("b")),
        &[("a", 0.0), ("b", -1.0)],
    );
    assert_eq!(neg_zero.re, 0.0);
    assert_eq!(pos_zero.re, 0.0);
    assert!(
        neg_zero.re.is_sign_negative() != pos_zero.re.is_sign_negative(),
        "mul_t(−1,0) = −0.0 but mul_t(0,−1) = +0.0 — not bit-commutative"
    );
}
