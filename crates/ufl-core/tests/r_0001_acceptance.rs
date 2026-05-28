//! R-0001 — EML operator core: end-to-end acceptance tests.
//!
//! One section per acceptance criterion (AC1–AC6). Each test cites its AC id
//! in a `// ACk — …` comment so the architect (PR review) and the orchestrator
//! (status update) can map the suite to R-0001's acceptance criteria
//! mechanically.
//!
//! These tests are authored at loop step 3 (test plan) and are expected to
//! **fail (red)** until R-0001 step 5 (implementation) replaces the `todo!()`
//! stubs in `eval` and `ln_eml`. AC6's `sin(τ/2) ≠ 0` invariant (a runtime
//! tripwire) may legitimately pass already — it tests the floating-point
//! environment, not UFL code.
//!
//! See:
//! - `requirements/0001-eml-operator-core.md` — AC1–AC5
//! - `specs/0001-eml-operator-core.md` — AC1–AC6 (AC6 added at spec level)
//! - `experiments/q-ac4-branch.py` — the Q-AC4 resolution this codifies.

use ufl_core::{eval, Eml, Env, EvalError, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The AC5 input sample, fixed by SPEC-0001 §6 (AC5).
const AC5_SAMPLE: &[f64] = &[-3.0, -1.0, -0.5, 0.5, 1.0, 2.5];

/// The AC5 relative tolerance, fixed by SPEC-0001 §6 (AC5).
const AC5_TOL: f64 = 1e-14;

/// Relative-error closeness in ℂ: `|actual - expected| <= tol * max(|expected|, 1)`.
///
/// Mirrors the Python experiment's denominator (`max(|true|, 1.0)`), so the
/// tolerance is meaningful for both small and large reference magnitudes.
fn close(actual: Value, expected: Value, tol: f64) -> bool {
    let denom = expected.norm().max(1.0);
    (actual - expected).norm() <= tol * denom
}

/// Build the AC5 `ln(x)` identity tree: `ln(x) = eml(1, eml(eml(1, x), 1))`.
fn ln_tree(x_var: &str) -> Eml {
    Eml::node(
        Eml::one(),
        Eml::node(Eml::node(Eml::one(), Eml::var(x_var)), Eml::one()),
    )
}

/// Build the AC5 `exp(x)` identity tree: `exp(x) = eml(x, 1)`.
fn exp_tree(x_var: &str) -> Eml {
    Eml::node(Eml::var(x_var), Eml::one())
}

/// Build the AC5 `e` identity tree: `e = eml(1, 1)`.
fn e_tree() -> Eml {
    Eml::node(Eml::one(), Eml::one())
}

/// Closed-tree evaluation convenience: `eval` under an empty `Env`, unwrap the
/// `Result` so the test surfaces the value directly.
fn eval_closed(expr: &Eml) -> Value {
    eval(expr, &Env::new()).expect("closed tree should evaluate without binding lookup")
}

/// Evaluation with a single `x` binding.
fn eval_with_x(expr: &Eml, x: Value) -> Value {
    let mut env = Env::new();
    env.bind("x", x);
    eval(expr, &env).expect("tree should evaluate with `x` bound")
}

// ===========================================================================
// AC1 — Representation
//
// "An EML expression is representable as a binary tree whose leaves are the
// literal `1` or a named variable and whose every internal node is `eml`. The
// representation admits exactly the grammar S → 1 | <var> | eml(S, S) — no
// other node or leaf kind."
//
// AC1 holds structurally because `Eml` is a closed enum with exactly three
// variants. We additionally check the public constructors produce each
// variant verbatim — the only path from caller code to a tree.
// ===========================================================================

#[test]
fn ac1_constructors_produce_only_grammar_variants() {
    // AC1 — `Eml::one()` is the literal `1` leaf.
    assert!(matches!(Eml::one(), Eml::One));

    // AC1 — `Eml::var(name)` is a named-variable leaf.
    match Eml::var("x") {
        Eml::Var(name) => assert_eq!(name, "x"),
        other => panic!("Eml::var should produce Eml::Var, got {other:?}"),
    }

    // AC1 — `Eml::node(a, b)` is an `eml` internal node over two subtrees.
    match Eml::node(Eml::one(), Eml::var("y")) {
        Eml::Node { exp_arg, log_arg } => {
            assert_eq!(*exp_arg, Eml::One);
            assert_eq!(*log_arg, Eml::Var("y".into()));
        }
        other => panic!("Eml::node should produce Eml::Node, got {other:?}"),
    }
}

#[test]
fn ac1_grammar_is_exhaustive_three_variants() {
    // AC1 — exhaustively matching `Eml` requires exactly the three grammar
    // variants. If a fourth variant is ever added, this match stops compiling
    // and the AC1 structural guarantee must be re-examined.
    fn classify(e: &Eml) -> &'static str {
        match e {
            Eml::One => "one",
            Eml::Var(_) => "var",
            Eml::Node { .. } => "node",
        }
    }
    assert_eq!(classify(&Eml::one()), "one");
    assert_eq!(classify(&Eml::var("x")), "var");
    assert_eq!(classify(&Eml::node(Eml::one(), Eml::one())), "node");
}

// ===========================================================================
// AC2 — Reference evaluation
//
// "A closed (variable-free) EML tree evaluates to a single complex value.
// A tree containing variables evaluates to a complex value given a binding
// for every variable it mentions."
// ===========================================================================

#[test]
fn ac2_closed_tree_evaluates_to_a_value() {
    // AC2 — the trivially closed tree `1` evaluates to `(1, 0)`.
    let v = eval_closed(&Eml::one());
    assert_eq!(v, Value::new(1.0, 0.0));

    // AC2 — a non-trivial closed tree evaluates without error.
    //  `eml(1, 1)` = `exp(1) − ln(1)` = `e − 0` = `e`.
    let v = eval_closed(&e_tree());
    assert!(close(v, Value::new(std::f64::consts::E, 0.0), AC5_TOL));
}

#[test]
fn ac2_tree_with_variable_evaluates_with_binding() {
    // AC2 — `eml(x, 1)` evaluates given a binding for `x`.
    //  With x = 2 + 0i this is `exp(2) − ln(1)` = `e^2`.
    let v = eval_with_x(&exp_tree("x"), Value::new(2.0, 0.0));
    assert!(close(
        v,
        Value::new(std::f64::consts::E.powi(2), 0.0),
        AC5_TOL
    ));
}

#[test]
fn ac2_unbound_variable_is_an_evaluation_error() {
    // AC2 — every variable mentioned must have a binding. Missing binding is
    // the single legitimate evaluation failure (SPEC-0001 §2.5).
    let err = eval(&Eml::var("missing"), &Env::new())
        .expect_err("unbound variable must produce an error");
    assert_eq!(err, EvalError::UnboundVariable("missing".into()));
}

#[test]
fn ac2_unbound_variable_inside_a_node_is_detected() {
    // AC2 — the error surfaces from inside a node, not only from a top-level
    // leaf. `eml(1, x)` with no `x` binding must error.
    let expr = Eml::node(Eml::one(), Eml::var("x"));
    let err = eval(&expr, &Env::new()).expect_err("unbound var in node should error");
    assert_eq!(err, EvalError::UnboundVariable("x".into()));
}

// ===========================================================================
// AC3 — Extended reals
//
// "Evaluation of `ln 0`, `exp(−∞)`, and expressions producing signed zeros or
// infinities follows IEEE-754 semantics and never traps, panics, or aborts;
// such values propagate as ordinary results."
// ===========================================================================

#[test]
fn ac3_ln_of_zero_propagates_without_panic() {
    // AC3 — `eml(1, 0)` = `exp(1) − ln(0)`. `ln(0+0i)` in `num-complex` is
    // `(-∞, 0)`, so the result has `Re = +∞`. No panic.
    let mut env = Env::new();
    env.bind("zero", Value::new(0.0, 0.0));
    let expr = Eml::node(Eml::one(), Eml::var("zero"));
    let v = eval(&expr, &env).expect("ln(0) must not error");
    assert!(
        v.re.is_infinite() && v.re.is_sign_positive(),
        "expected +inf real part, got {v:?}"
    );
}

#[test]
fn ac3_exp_of_neg_infinity_propagates_without_panic() {
    // AC3 — `eml(-∞, 1)` = `exp(-∞) − ln(1)` = `0 − 0` = `0`. No panic.
    let mut env = Env::new();
    env.bind("neg_inf", Value::new(f64::NEG_INFINITY, 0.0));
    let expr = Eml::node(Eml::var("neg_inf"), Eml::one());
    let v = eval(&expr, &env).expect("exp(-inf) must not error");
    assert!(v.re.is_finite(), "real part should be finite, got {v:?}");
    assert_eq!(v.re, 0.0);
}

#[test]
fn ac3_nan_propagates_as_ordinary_value() {
    // AC3 — NaN inputs propagate, they do not trap.
    let mut env = Env::new();
    env.bind("nanv", Value::new(f64::NAN, 0.0));
    let expr = Eml::node(Eml::var("nanv"), Eml::one());
    let v = eval(&expr, &env).expect("NaN input must not error");
    assert!(v.re.is_nan(), "expected NaN real part, got {v:?}");
}

// ===========================================================================
// AC4 — Branch convention
//
// "EML's `ln` uses one documented branch cut, chosen so that the derived
// quantities `i` and `τ`, and `ln x` for real `x < 0`, carry the sign of the
// standard principal branch."
//
// SPEC-0001 §2.4 fixes `ln_eml = Complex::ln` (principal branch) and relies
// on the f64 self-correction documented in `experiments/q-ac4-branch.py`.
//
// We construct the trees in pure EML where possible. `i = exp(ln(-1) / 2)`
// is not a pure-EML tree on its own (division by `2` is itself a derived
// EML construction), so per the scoping note in the task description we
// compute the derived `i` and `τ` via `eval`'d intermediates and document
// the choice here. The substantive checks remain: every value below is
// produced by `eval` over an EML tree built from the public API.
// ===========================================================================

#[test]
fn ac4_derived_ln_negative_real_uses_principal_branch() {
    // AC4 — for real `x < 0`, the derived `ln(x)` tree must carry the
    // principal-branch sign (`Im = +τ/2` on the positive cut for `x = -1`).
    let ln_x = ln_tree("x");
    let v = eval_with_x(&ln_x, Value::new(-1.0, 0.0));
    let expected = Value::new(0.0, std::f64::consts::PI); // ln(-1) = +iπ = +i τ/2
    assert!(
        close(v, expected, AC5_TOL),
        "ln(-1) should be principal (Im = +π), got {v:?}"
    );
    assert!(
        v.im > 0.0,
        "principal ln(-1) must have positive imaginary part"
    );
}

#[test]
fn ac4_derived_i_has_positive_imaginary_unit() {
    // AC4 — the derived `i` (from `exp(ln(-1) / 2)`) carries the principal
    // sign: `i ≈ (0, +1)`, not `(0, -1)`. The Python experiment's spot-check
    // returns `(6.12e-17, +1.000)`.
    //
    // Construction note: we evaluate the pure-EML `ln(-1)` tree, halve the
    // result in host arithmetic, and apply `eml(.., 1)` for `exp`. The `/2`
    // is the only step not expressible as a single binary EML node at this
    // stage of UFL — flagged explicitly per the QA scoping note.
    let ln_minus_one = eval_with_x(&ln_tree("x"), Value::new(-1.0, 0.0));
    let derived_i = eval_with_x(&exp_tree("x"), ln_minus_one / Value::new(2.0, 0.0));

    assert!(
        close(derived_i, Value::new(0.0, 1.0), 1e-14),
        "derived i should be ≈ (0, +1), got {derived_i:?}"
    );
    assert!(
        derived_i.im > 0.0,
        "derived i must have positive imaginary unit (principal branch)"
    );
}

#[test]
fn ac4_derived_tau_is_positive() {
    // AC4 — the derived `τ` (as `2 * Im(ln(-1))`) must be positive: principal
    // `ln(-1) = +iπ`, so `τ = 2π = 2 * Im(ln(-1)) > 0`. A wrong branch would
    // make it negative.
    let mut env = Env::new();
    env.bind("x", Value::new(-1.0, 0.0));
    let ln_minus_one = eval(&ln_tree("x"), &env).expect("ln(-1) tree must evaluate");
    let derived_tau = 2.0 * ln_minus_one.im;
    let expected_tau = std::f64::consts::TAU;
    assert!(
        (derived_tau - expected_tau).abs() <= AC5_TOL * expected_tau,
        "derived τ should be ≈ 2π, got {derived_tau}"
    );
    assert!(
        derived_tau > 0.0,
        "derived τ must be positive — wrong branch would flip the sign"
    );
}

// ===========================================================================
// AC5 — Known identities
//
// "Each of the following evaluates to within a documented tolerance of an
// independently computed reference value, over a sample of inputs that
// includes negative real x:
//   - e   = eml(1, 1)
//   - exp(x) = eml(x, 1)
//   - ln(x)  = eml(1, eml(eml(1, x), 1))"
//
// Tolerance: relative 1e-14 (SPEC-0001 §6 AC5).
// Sample: { -3.0, -1.0, -0.5, 0.5, 1.0, 2.5 } (SPEC-0001 §6 AC5).
// ===========================================================================

#[test]
fn ac5_e_identity_matches_reference() {
    // AC5 — `e = eml(1, 1)` matches `std::f64::consts::E` within 1e-14.
    let v = eval_closed(&e_tree());
    let expected = Value::new(std::f64::consts::E, 0.0);
    assert!(
        close(v, expected, AC5_TOL),
        "eml(1,1) should be e, got {v:?}"
    );
}

#[test]
fn ac5_exp_identity_matches_reference_over_sample() {
    // AC5 — `exp(x) = eml(x, 1)` over the §6 sample, including negative x.
    let expr = exp_tree("x");
    for &xr in AC5_SAMPLE {
        let x = Value::new(xr, 0.0);
        let actual = eval_with_x(&expr, x);
        let expected = x.exp();
        assert!(
            close(actual, expected, AC5_TOL),
            "exp identity failed at x={xr}: actual={actual:?}, expected={expected:?}"
        );
    }
}

#[test]
fn ac5_ln_identity_matches_reference_over_sample() {
    // AC5 — `ln(x) = eml(1, eml(eml(1, x), 1))` over the §6 sample, including
    // the negative-real entries where the AllEle §4.1 textbook discrepancy
    // would predict a τi jump; the f64 self-correction (SPEC-0001 §2.4)
    // keeps the result within 1e-14 (Python baseline ≤ 1 ulp ≈ 1.11e-16).
    let expr = ln_tree("x");
    for &xr in AC5_SAMPLE {
        let x = Value::new(xr, 0.0);
        let actual = eval_with_x(&expr, x);
        let expected = x.ln();
        assert!(
            close(actual, expected, AC5_TOL),
            "ln identity failed at x={xr}: actual={actual:?}, expected={expected:?}"
        );
    }
}

// ===========================================================================
// AC6 — `sin(τ/2) ≠ 0` invariant (acceptance level)
//
// "A unit test asserts that the runtime's `f64::sin(std::f64::consts::PI)`
// is non-zero. Its purpose is to fail loudly if a future arithmetic backend
// makes `sin(τ/2)` exactly zero — in that case the AC4 self-correction
// silently breaks and Q-AC4 must be re-opened."
//
// Also asserted inline in `crates/ufl-core/src/log.rs` so the tripwire
// travels with the single function that depends on it.
// ===========================================================================

#[test]
fn ac6_sin_tau_over_two_is_non_zero_in_f64() {
    // AC6 — the floating-point self-correction relied on by §2.4.
    let s = std::f64::consts::PI.sin(); // τ/2 == π
    assert_ne!(
        s, 0.0,
        "sin(τ/2) is exactly zero — Q-AC4 must be re-opened, see SPEC-0001 §2.4"
    );
    // The Python experiment observed ≈ -1.22e-16; require strictly sub-ulp,
    // not zero. (Magnitude check is informational — the AC is the `!= 0`.)
    assert!(
        s.abs() < 1e-10,
        "sin(τ/2) should be a small non-zero residue, got {s}"
    );
}
