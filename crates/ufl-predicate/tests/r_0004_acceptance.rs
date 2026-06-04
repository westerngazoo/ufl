//! R-0004 — Predicate layer (the checker): end-to-end acceptance tests.
//!
//! One section per acceptance criterion (AC1–AC6), in order. Each test cites
//! its AC id in a `// ACk — …` comment so the architect (PR review) and the
//! orchestrator (status update) can map the suite to R-0004's acceptance
//! criteria mechanically.
//!
//! Authored at loop step 3 (test plan). Tests are expected to **fail (red)**
//! until R-0004 step 5 replaces the `unimplemented!()` body of
//! [`ufl_predicate::eval_pred`] (`src/eval_pred.rs`). Concretely:
//!
//!  - Every test that reaches `eval_pred` — i.e. anything through `check`,
//!    `check_str`, or `eval_pred` itself that actually evaluates a predicate —
//!    panics now (the body is `unimplemented!()`) and so is RED. That is the
//!    bulk of the suite below.
//!  - The `CheckError::ReservedName` path returns from `combined_env` *before*
//!    `eval_pred` is called, so those tests (AC4/AC6 reserved-name) are GREEN
//!    now. They are noted inline.
//!
//! See:
//!  - `requirements/0004-predicate-layer.md` — AC1–AC6
//!  - `specs/0004-predicate-layer.md` — §2.2 (classify), §2.3 (forms),
//!    §2.4 (exact-`=` contracts), §2.5 (pre/post priming + `ReservedName`),
//!    §2.6 (error model), §3 (code outline), §6 (the AC list with exact
//!    expected behaviours).

use ufl_core::{eval, Env, EvalError, Value};
use ufl_predicate::{check, check_str, eval_pred, CheckError, PredError};
use ufl_syntax::{lower, read, LowerError};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read a predicate `Sexpr` from text, panicking on a read error. Used to build
/// the predicate trees the checker consumes (the read layer is already verified
/// by R-0003 and is not the unit under test here).
fn p(src: &str) -> ufl_syntax::Sexpr {
    read(src).expect("test predicate text should read")
}

/// A real `Value` `q + 0i`.
fn real(q: f64) -> Value {
    Value::new(q, 0.0)
}

/// Evaluate a *numeric* eml form (`(eml x 1)` etc.) under `env`, the same path
/// `=`'s operands take internally: `read → lower → ufl_core::eval`. This is the
/// oracle for AC5's expected post-state — binding `x'` to exactly this result
/// makes `(= x' (eml x 1))` an exact equality (no literal-`e` mismatch).
fn eval_eml(src: &str, env: &Env) -> Value {
    let s = read(src).expect("eml oracle text should read");
    let e = lower(&s).expect("eml oracle text should lower");
    eval(&e, env).expect("eml oracle should evaluate")
}

// ===========================================================================
// AC1 — Booleans enter the language; the type boundary, with no coercion.
//
// "UFL has boolean values, distinct from numeric (eml) values. `true`/`false`
// evaluate to the two boolean values. Applying a numeric operation to a
// boolean, or a logical operation to a number, is a typed error, never a panic
// or a silent coercion." (R-0004 AC1; SPEC-0004 §2.2, §6 AC1.)
//
// The load-bearing guard is the *single classifier*: a numeric shape in boolean
// position → `ExpectedBool`; a boolean shape as a `=` operand → `ExpectedNumber`
// — and crucially the latter is caught at the boundary, NOT as a downstream
// `Eval(UnboundVariable("true"))`.
// ===========================================================================

#[test]
fn ac1_boolean_literals_evaluate() {
    // AC1 — `true`/`false` evaluate to the two boolean values (SPEC-0004 §2.3).
    assert_eq!(eval_pred(&p("true"), &Env::new()), Ok(true));
    assert_eq!(eval_pred(&p("false"), &Env::new()), Ok(false));
}

#[test]
fn ac1_number_as_equality_operand_is_not_coerced_but_a_boolean_is() {
    // AC1 — a *boolean* shape (`true`) where `=` expects a number is a typed
    // `ExpectedNumber`, caught at the classifier boundary BEFORE `lower` —
    // so the diagnostic is NOT a downstream `Eval(UnboundVariable("true"))`
    // (the precise leak the three-lens review closed; SPEC-0004 §2.2).
    let got = check_str("(= true 1)", &[], &[]);
    match got {
        Err(CheckError::Pred(PredError::ExpectedNumber { .. })) => {}
        other => panic!("`(= true 1)` should be ExpectedNumber, got {other:?}"),
    }
    // Explicitly assert it is *not* the unbound-variable leak.
    assert_ne!(
        got,
        Err(CheckError::Pred(PredError::Eval(
            EvalError::UnboundVariable("true".to_string())
        ))),
        "`(= true 1)` must be caught at the boundary, not leak as UnboundVariable"
    );
}

#[test]
fn ac1_compound_boolean_as_equality_operand_is_expected_number() {
    // AC1 — `(and true)` is a boolean *form* in `=`-operand (numeric) position →
    // `ExpectedNumber` (SPEC-0004 §6 AC1: `(= (and true) 1)`).
    match check_str("(= (and true) 1)", &[], &[]) {
        Err(CheckError::Pred(PredError::ExpectedNumber { .. })) => {}
        other => panic!("`(= (and true) 1)` should be ExpectedNumber, got {other:?}"),
    }
}

#[test]
fn ac1_numeric_form_in_boolean_position_is_expected_bool() {
    // AC1 — a numeric `eml` form where a boolean is expected → `ExpectedBool`.
    // `(not (eml 1 1))`: `not` needs a boolean operand, `(eml 1 1)` is numeric
    // (SPEC-0004 §6 AC1).
    match check_str("(not (eml 1 1))", &[], &[]) {
        Err(CheckError::Pred(PredError::ExpectedBool { .. })) => {}
        other => panic!("`(not (eml 1 1))` should be ExpectedBool, got {other:?}"),
    }
}

#[test]
fn ac1_numeric_form_in_connective_position_is_expected_bool() {
    // AC1 — `(and (eml 1 1) true)`: a numeric form as a connective operand →
    // `ExpectedBool` (SPEC-0004 §6 AC1).
    match check_str("(and (eml 1 1) true)", &[], &[]) {
        Err(CheckError::Pred(PredError::ExpectedBool { .. })) => {}
        other => panic!("`(and (eml 1 1) true)` should be ExpectedBool, got {other:?}"),
    }
}

#[test]
fn ac1_bare_number_in_boolean_position_is_expected_bool() {
    // AC1 — a bare number checked as a predicate is `ExpectedBool` (a `Num`
    // classifies `Numeric`; SPEC-0004 §2.2 + §3's catch-all arm).
    match eval_pred(&p("1"), &Env::new()) {
        Err(PredError::ExpectedBool { .. }) => {}
        other => panic!("a bare number as a predicate should be ExpectedBool, got {other:?}"),
    }
}

// ===========================================================================
// AC2 — Exact equality.
//
// "`(= a b)` evaluates its two operands as numeric (eml) values and returns a
// boolean: `true` iff they are exactly equal (no tolerance). Equality is
// decidable and deterministic." (R-0004 AC2; SPEC-0004 §2.4, §6 AC2.)
//
// Contracts tested (SPEC-0004 §2.4): operand-lowerability (`(= 2 2)` does not
// lower); IEEE non-reflexivity on NaN.
// ===========================================================================

#[test]
fn ac2_equal_literals_are_true() {
    // AC2 — `(= 1 1)` is `true` (both operands lower to the value 1, IEEE-equal).
    assert_eq!(eval_pred(&p("(= 1 1)"), &Env::new()), Ok(true));
}

#[test]
fn ac2_equality_against_a_bound_variable() {
    // AC2 — `(= x 1)` is `true` when `x = 1`, `false` when `x = 2`. Equality is
    // decidable and deterministic over the bound value.
    let mut env_one = Env::new();
    env_one.bind("x", real(1.0));
    assert_eq!(eval_pred(&p("(= x 1)"), &env_one), Ok(true));

    let mut env_two = Env::new();
    env_two.bind("x", real(2.0));
    assert_eq!(eval_pred(&p("(= x 1)"), &env_two), Ok(false));
}

#[test]
fn ac2_non_one_literal_operand_does_not_lower() {
    // AC2 (contract 1, SPEC-0004 §2.4) — only `1` is a numeric literal; `(= 2 2)`
    // surfaces the operand's `LowerError::UnsupportedLiteral(2.0)` via
    // `PredError::Lower`. `=` operands are therefore `1`, a variable, or an `eml`
    // form (AC5 respects this).
    assert_eq!(
        check_str("(= 2 2)", &[], &[]),
        Err(CheckError::Pred(PredError::Lower(
            LowerError::UnsupportedLiteral(2.0)
        )))
    );
}

#[test]
fn ac2_equality_is_non_reflexive_on_nan() {
    // AC2 (contract 2, SPEC-0004 §2.4) — `=` is IEEE value equality, so it is
    // non-reflexive on `NaN`: `(= W W)` is `false` when `W` evaluates to NaN.
    //
    // `W = (eml (eml 1 z) (eml 1 z))` with `z = 0` evaluates to NaN:
    //   inner `(eml 1 z)` = exp(1) − ln(0) = e − (−∞) = +∞ (real part);
    //   then exp(+∞) − ln(+∞) = +∞ − (+∞) = NaN.
    // (Verified against `ufl_core::eval`: re = NaN.) A predicate over a
    // NaN-valued state is thus unsatisfiable by `=` — IEEE-correct, and a
    // documented, tested contract.
    let mut env = Env::new();
    env.bind("z", real(0.0));

    // Sanity: the operand really does evaluate to NaN through the numeric path,
    // so the assertion below is a NaN non-reflexivity test, not an accident.
    let w = eval_eml("(eml (eml 1 z) (eml 1 z))", &env);
    assert!(
        w.re.is_nan(),
        "AC2 NaN fixture must evaluate to NaN (re), got {w:?}"
    );

    assert_eq!(
        eval_pred(
            &p("(= (eml (eml 1 z) (eml 1 z)) (eml (eml 1 z) (eml 1 z)))"),
            &env
        ),
        Ok(false),
        "(= W W) with W = NaN must be false (IEEE non-reflexivity)"
    );
}

// ===========================================================================
// AC3 — Logical connectives.
//
// "`(and p q …)`, `(or p q …)`, `(not p)` evaluate boolean operands with the
// standard truth tables." (R-0004 AC3; SPEC-0004 §2.3, §6 AC3.)
//
// Spec details: `(and)` = `true`, `(or)` = `false`; `(not)`/`(not a b)` →
// `Arity`; lazy short-circuit intentionally suppresses errors in *unreached*
// operands, and surfaces them in *reached* ones.
// ===========================================================================

#[test]
fn ac3_and_truth_table() {
    // AC3 — `and` truth table on the binary cases.
    assert_eq!(eval_pred(&p("(and true true)"), &Env::new()), Ok(true));
    assert_eq!(eval_pred(&p("(and true false)"), &Env::new()), Ok(false));
    assert_eq!(eval_pred(&p("(and false true)"), &Env::new()), Ok(false));
    assert_eq!(eval_pred(&p("(and false false)"), &Env::new()), Ok(false));
}

#[test]
fn ac3_or_truth_table() {
    // AC3 — `or` truth table on the binary cases.
    assert_eq!(eval_pred(&p("(or true true)"), &Env::new()), Ok(true));
    assert_eq!(eval_pred(&p("(or true false)"), &Env::new()), Ok(true));
    assert_eq!(eval_pred(&p("(or false true)"), &Env::new()), Ok(true));
    assert_eq!(eval_pred(&p("(or false false)"), &Env::new()), Ok(false));
}

#[test]
fn ac3_not_truth_table() {
    // AC3 — `not` truth table.
    assert_eq!(eval_pred(&p("(not true)"), &Env::new()), Ok(false));
    assert_eq!(eval_pred(&p("(not false)"), &Env::new()), Ok(true));
}

#[test]
fn ac3_empty_connectives_are_identities() {
    // AC3 — the empty-connective identities are normative (SPEC-0004 §2.3):
    // `(and)` = `true` (empty conjunction), `(or)` = `false` (empty disjunction).
    assert_eq!(eval_pred(&p("(and)"), &Env::new()), Ok(true));
    assert_eq!(eval_pred(&p("(or)"), &Env::new()), Ok(false));
}

#[test]
fn ac3_not_arity_errors() {
    // AC3 — `not` is arity 1: `(not)` and `(not true false)` are
    // `PredError::Arity { form: "not", .. }` (SPEC-0004 §2.3 / §6 AC3).
    match eval_pred(&p("(not)"), &Env::new()) {
        Err(PredError::Arity { form, .. }) if form == "not" => {}
        other => panic!("`(not)` should be Arity{{form:\"not\"}}, got {other:?}"),
    }
    match eval_pred(&p("(not true false)"), &Env::new()) {
        Err(PredError::Arity { form, .. }) if form == "not" => {}
        other => panic!("`(not true false)` should be Arity{{form:\"not\"}}, got {other:?}"),
    }
}

#[test]
fn ac3_and_short_circuit_suppresses_unreached_error() {
    // AC3 — lazy short-circuit: `(and false X)` is `false` even though `X`
    // (`(= zz 1)` with `zz` unbound) would error if reached. The verdict may
    // depend on operand order when an unreached operand contains an error — by
    // design (SPEC-0004 §2.3 decision log).
    assert_eq!(
        eval_pred(&p("(and false (= zz 1))"), &Env::new()),
        Ok(false),
        "(and false <erroring>) must short-circuit to false, not error"
    );
}

#[test]
fn ac3_or_short_circuit_suppresses_unreached_error() {
    // AC3 — `(or true X)` is `true` even though `X` (`(= zz 1)`, `zz` unbound)
    // would error if reached (SPEC-0004 §2.3).
    assert_eq!(
        eval_pred(&p("(or true (= zz 1))"), &Env::new()),
        Ok(true),
        "(or true <erroring>) must short-circuit to true, not error"
    );
}

#[test]
fn ac3_short_circuit_does_not_suppress_a_reached_error() {
    // AC3 — the converse: when the erroring operand IS reached, the error
    // surfaces. `(and true (= zz 1))` with `zz` unbound reaches the second
    // operand → `Eval(UnboundVariable("zz"))`. AC6's "earliest layer" governs
    // where a *produced* error surfaces, not that unreached operands run.
    assert_eq!(
        eval_pred(&p("(and true (= zz 1))"), &Env::new()),
        Err(PredError::Eval(EvalError::UnboundVariable(
            "zz".to_string()
        ))),
        "(and true <erroring>) reaches the second operand and must surface its error"
    );
}

// ===========================================================================
// AC4 — Pre/post-state and checking.
//
// "A predicate may mention pre-state variables and post-state variables (the
// latter primed, e.g. `x'`). Given a pre-state binding and a post-state
// binding, the predicate checks to `true` or `false`. A predicate variable with
// no binding is a typed error." (R-0004 AC4; SPEC-0004 §2.5, §6 AC4.)
// ===========================================================================

#[test]
fn ac4_pre_and_post_vars_resolve_through_priming() {
    // AC4 — `check` binds pre vars by name and post vars under `name'`. The
    // predicate `(= x' (eml x 1))` references pre-`x` and post-`x'`; with
    // `pre[x] = 1` and `post[x] = e` (= the eml-computed exp(1)), it checks
    // `true`. The post binding is the *eval result* of `(eml x 1)`, so the
    // equality is exact (SPEC-0004 §2.5; AC5 generalises this).
    let mut pre_env = Env::new();
    pre_env.bind("x", real(1.0));
    let post = eval_eml("(eml x 1)", &pre_env);
    assert_eq!(
        check_str("(= x' (eml x 1))", &[("x", real(1.0))], &[("x", post)]),
        Ok(true)
    );
}

#[test]
fn ac4_check_accepts_a_prebuilt_sexpr_tree() {
    // AC4 — `check` is the tree entry point (vs `check_str`'s text path), sharing
    // the same pre/post priming. A predicate `Sexpr` built directly (not read
    // from text) checks identically: `(= x' (eml x 1))` with the correct post.
    let predicate = p("(= x' (eml x 1))");
    let mut pre_env = Env::new();
    pre_env.bind("x", real(1.0));
    let post = eval_eml("(eml x 1)", &pre_env);
    assert_eq!(
        check(&predicate, &[("x", real(1.0))], &[("x", post)]),
        Ok(true)
    );
}

#[test]
fn ac4_unbound_state_variable_is_typed_eval_error() {
    // AC4 — a predicate variable with no binding is a typed error:
    // `(= y 1)` with `y` unbound in both pre and post →
    // `Pred(Eval(UnboundVariable("y")))` (SPEC-0004 §6 AC4). Caught at the
    // numeric eval layer, never a panic.
    assert_eq!(
        check_str("(= y 1)", &[], &[]),
        Err(CheckError::Pred(PredError::Eval(
            EvalError::UnboundVariable("y".to_string())
        )))
    );
}

#[test]
fn ac4_reserved_binding_name_is_rejected() {
    // AC4 — the priming convention is injective: a pre/post binding *name*
    // containing `'` is reserved → `CheckError::ReservedName` (SPEC-0004 §2.5).
    //
    // NOTE: this returns from `combined_env` BEFORE `eval_pred` is reached, so
    // this test is GREEN now (the only acceptance test that does not depend on
    // the pending `eval_pred`). It pins the reserved-name guard immediately.
    assert_eq!(
        check_str("(= x 1)", &[("x'", real(1.0))], &[]),
        Err(CheckError::ReservedName("x'".to_string()))
    );
    // The same applies to a post binding whose name contains `'`.
    assert_eq!(
        check_str("(= x 1)", &[], &[("x'", real(1.0))]),
        Err(CheckError::ReservedName("x'".to_string()))
    );
}

// ===========================================================================
// AC5 — Predicates express eml semantics (the worked example / headline).
//
// "The predicate `⟦ x' = (eml x 1) ⟧` checks to `true` for any pre/post pair
// where `x'` equals the eml-evaluated `e^x`, and `false` otherwise. Demonstrated
// over several `x`, including a deliberately-wrong post-state that must check
// `false`." (R-0004 AC5; SPEC-0004 §6 AC5.)
//
// The predicate is wrapped as `(pred …)` — the named `⟦P⟧` atom, transparent at
// check time (SPEC-0004 §7). The expected post is computed via `eval_eml`, so
// the equality is exact (verified bit-identical to `exp(x)`).
// ===========================================================================

#[test]
fn ac5_predicate_checks_true_for_correct_post_state() {
    // AC5 — for each real x ∈ {0.0, 1.0, 2.5}, with post[x] = eml-computed
    // exp(x), `(pred (= x' (eml x 1)))` checks `true`.
    for x in [0.0_f64, 1.0, 2.5] {
        let mut pre_env = Env::new();
        pre_env.bind("x", real(x));
        let expected_post = eval_eml("(eml x 1)", &pre_env);
        assert_eq!(
            check_str(
                "(pred (= x' (eml x 1)))",
                &[("x", real(x))],
                &[("x", expected_post)],
            ),
            Ok(true),
            "predicate should hold at x={x} for the correct post-state"
        );
    }
}

#[test]
fn ac5_predicate_checks_false_for_wrong_post_state() {
    // AC5 — a deliberately-wrong post-state (`expected + 1`) must check `false`,
    // over the same x's. This is the half of AC5 that proves the checker
    // actually discriminates (not a tautology).
    for x in [0.0_f64, 1.0, 2.5] {
        let mut pre_env = Env::new();
        pre_env.bind("x", real(x));
        let expected_post = eval_eml("(eml x 1)", &pre_env);
        let wrong_post = expected_post + real(1.0);
        assert_eq!(
            check_str(
                "(pred (= x' (eml x 1)))",
                &[("x", real(x))],
                &[("x", wrong_post)],
            ),
            Ok(false),
            "predicate should fail at x={x} for a deliberately wrong post-state"
        );
    }
}

// ===========================================================================
// AC6 — Layered typed errors, no panic.
//
// "Every failure is a typed error surfaced at the earliest layer that can
// detect it — read, lower, or the new predicate-evaluation layer (type
// mismatch, unbound state variable) — never a panic. Composes with R-0003's
// `ReadError` / `LowerError` / `EvalError`." (R-0004 AC6; SPEC-0004 §2.6,
// §6 AC6.)
//
// One assertion per detecting layer, all through `check_str` (whose
// `CheckError` unifies the read layer and the predicate layer).
// ===========================================================================

#[test]
fn ac6_read_layer_error_is_checkerror_read() {
    // AC6 — a lexical/syntactic failure surfaces at the *read* layer:
    // `check_str(")")` → `CheckError::Read(ReadError::UnexpectedClose)`.
    assert_eq!(
        check_str(")", &[], &[]),
        Err(CheckError::Read(ufl_syntax::ReadError::UnexpectedClose))
    );
}

#[test]
fn ac6_type_mismatch_is_checkerror_pred_expected_number() {
    // AC6 — a boolean as a `=` operand is a predicate-layer type error:
    // `(= true 1)` → `CheckError::Pred(PredError::ExpectedNumber{..})` (the
    // boundary, not a downstream unbound-variable — see AC1).
    match check_str("(= true 1)", &[], &[]) {
        Err(CheckError::Pred(PredError::ExpectedNumber { .. })) => {}
        other => panic!("`(= true 1)` should be Pred(ExpectedNumber), got {other:?}"),
    }
}

#[test]
fn ac6_unbound_state_var_is_checkerror_pred_eval() {
    // AC6 — an unbound state variable surfaces at the predicate-evaluation
    // (numeric eval) layer: `(= y 1)` with `y` unbound →
    // `CheckError::Pred(PredError::Eval(UnboundVariable("y")))`.
    assert_eq!(
        check_str("(= y 1)", &[], &[]),
        Err(CheckError::Pred(PredError::Eval(
            EvalError::UnboundVariable("y".to_string())
        )))
    );
}

#[test]
fn ac6_reserved_name_is_checkerror_reserved_name() {
    // AC6 — a reserved binding name surfaces as `CheckError::ReservedName`,
    // before any evaluation (SPEC-0004 §2.5). GREEN now (returns from
    // `combined_env`).
    assert_eq!(
        check_str("(= x 1)", &[("x'", real(1.0))], &[]),
        Err(CheckError::ReservedName("x'".to_string()))
    );
}

#[test]
fn ac6_no_failure_path_panics() {
    // AC6 — the umbrella guarantee: every failure path returns a typed `Err`,
    // never a panic. Each call here is a *distinct detecting layer*; that they
    // all return `Err(..)` (rather than unwinding) is the no-panic contract.
    //
    // NOTE: while `eval_pred` is `unimplemented!()`, the predicate-layer rows
    // (ExpectedNumber / Eval) panic — this test is RED until step 5, at which
    // point all four must be `Err`. The read-layer and reserved-name rows are
    // already typed `Err`s.
    let cases: [Result<bool, CheckError>; 4] = [
        check_str(")", &[], &[]),                        // read layer
        check_str("(= true 1)", &[], &[]),               // predicate type layer
        check_str("(= y 1)", &[], &[]),                  // numeric eval layer
        check_str("(= x 1)", &[("x'", real(1.0))], &[]), // reserved-name guard
    ];
    for (i, case) in cases.iter().enumerate() {
        assert!(
            case.is_err(),
            "failure case #{i} must be a typed Err (no panic, no Ok), got {case:?}"
        );
    }
}
