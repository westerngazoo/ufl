//! R-0003 — Homoiconic S-expression core: end-to-end acceptance tests.
//!
//! One section per acceptance criterion (AC1–AC6), in order. Each test cites
//! its AC id in a `// ACk — …` comment so the architect (PR review) and the
//! orchestrator (status update) can map the suite to R-0003's acceptance
//! criteria mechanically.
//!
//! Authored at loop step 3 (test plan). Tests are expected to **fail (red)**
//! until R-0003 step 5 replaces the `unimplemented!()` stubs in `read`
//! (`src/read.rs`) and `lower` (`src/lower.rs`). Concretely:
//!
//!  - Every test that calls `read`, `lower`, or `eval_str` panics now (the
//!    stubs are `unimplemented!()`) and so is RED. That is the whole suite
//!    below.
//!  - The purely structural assertions on `Sexpr` constructors + `Display`
//!    that do **not** touch the reader are green now; they live in
//!    `src/sexpr.rs::tests` (next to the type), not here.
//!
//! See:
//!  - `requirements/0003-sexpr-core.md` — AC1–AC6
//!  - `specs/0003-sexpr-core.md` — §2.3 (reader), §2.4 (lowering table),
//!    §2.5 (`eval_str` + `UflError`), §6 (the AC list with exact expected
//!    values).
//!  - `crates/ufl-core/tests/r_0001_acceptance.rs` — the parity oracle whose
//!    `close`/tolerance/tree conventions AC4 reuses.

use ufl_core::{Env, EvalError, Value};
use ufl_syntax::{eval_str, lower, read, LowerError, ReadError, Sexpr, UflError};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The AC4 relative tolerance, fixed by SPEC-0003 §6 (AC4) — the R-0001 value.
const AC4_TOL: f64 = 1e-14;

/// Relative-error closeness in ℂ: `|actual - expected| <= tol * max(|expected|, 1)`.
///
/// Identical to the R-0001 acceptance oracle
/// (`crates/ufl-core/tests/r_0001_acceptance.rs::close`), so AC4 parity is
/// measured the same way on both sides of the lowering boundary. No new deps.
fn close(actual: Value, expected: Value, tol: f64) -> bool {
    let denom = expected.norm().max(1.0);
    (actual - expected).norm() <= tol * denom
}

/// An `Env` binding `x` to the real value `q + 0i`, per SPEC-0003 §6 (AC4).
fn env_with_x(q: f64) -> Env {
    let mut env = Env::new();
    env.bind("x", Value::new(q, 0.0));
    env
}

// ===========================================================================
// AC1 — Homoiconic representation
//
// "There is one syntax tree type, `Sexpr` … A UFL program is an `Sexpr` and is
// itself ordinary data — constructible, traversable, and comparable. Code and
// data share one representation." (R-0003 AC1; SPEC-0003 §2.2.)
//
// The constructor / Clone / PartialEq / Display assertions live next to the
// type in `src/sexpr.rs::tests` (green now). Here we assert the AC1 *round-trip
// invariant*, which is a property of the reader: for every `s` the reader
// produces, `read(s.to_string()) == Ok(s)`. RED until `read` exists.
// ===========================================================================

#[test]
fn ac1_round_trip_atoms() {
    // AC1 — round-trip invariant over reader-produced atoms (SPEC-0003 §2.2).
    // `read` produces only finite `Num`s and delimiter-free, non-numeric
    // `Sym`s, whose `Display` re-reads to the same value.
    for src in ["1", "x", "eml"] {
        let s = read(src).expect("reader should accept a lone atom");
        assert_eq!(
            read(&s.to_string()),
            Ok(s.clone()),
            "round-trip failed for atom {src:?}"
        );
    }
}

#[test]
fn ac1_round_trip_canonical_eml_form() {
    // AC1 — round-trip the canonical `(eml 1 1)` form: read it, render it, read
    // the rendering, and require equality with the first parse.
    let s = read("(eml 1 1)").expect("reader should accept (eml 1 1)");
    assert_eq!(read(&s.to_string()), Ok(s));
}

#[test]
fn ac1_round_trip_nested_ln_form() {
    // AC1 — round-trip the nested `ln(x)` form `(eml 1 (eml (eml 1 x) 1))`.
    // This is the four-way string identity (docs/spec/example/test) made into
    // a property: structure survives text → Sexpr → text → Sexpr.
    let s = read("(eml 1 (eml (eml 1 x) 1))").expect("reader should accept the nested form");
    assert_eq!(read(&s.to_string()), Ok(s));
}

// ===========================================================================
// AC2 — Reader (text → Sexpr; typed ReadError; no panic)
//
// "Text S-expressions parse to `Sexpr`: `(eml 1 1)`, `(eml x 1)`, arbitrary
// nesting, insignificant whitespace, and line comments. Malformed input
// (unbalanced parentheses, empty application, stray tokens) yields a parse
// error reported via a typed `ReadError` enum — never a panic." (R-0003 AC2;
// SPEC-0003 §2.3.)
// ===========================================================================

#[test]
fn ac2_reads_canonical_form() {
    // AC2 — `(eml 1 1)` → `List([Sym("eml"), Num(1.0), Num(1.0)])`.
    assert_eq!(
        read("(eml 1 1)"),
        Ok(Sexpr::list([
            Sexpr::sym("eml"),
            Sexpr::num(1.0),
            Sexpr::num(1.0)
        ]))
    );
}

#[test]
fn ac2_reads_form_with_variable() {
    // AC2 — `(eml x 1)` → `List([Sym("eml"), Sym("x"), Num(1.0)])`.
    assert_eq!(
        read("(eml x 1)"),
        Ok(Sexpr::list([
            Sexpr::sym("eml"),
            Sexpr::sym("x"),
            Sexpr::num(1.0)
        ]))
    );
}

#[test]
fn ac2_reads_arbitrary_nesting() {
    // AC2 — arbitrary nesting `(eml 1 (eml (eml 1 x) 1))` parses to the exact
    // nested structure.
    let expected = Sexpr::list([
        Sexpr::sym("eml"),
        Sexpr::num(1.0),
        Sexpr::list([
            Sexpr::sym("eml"),
            Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0), Sexpr::sym("x")]),
            Sexpr::num(1.0),
        ]),
    ]);
    assert_eq!(read("(eml 1 (eml (eml 1 x) 1))"), Ok(expected));
}

#[test]
fn ac2_whitespace_is_insignificant() {
    // AC2 — extra spaces, tabs, and newlines do not change the parse:
    // `(  eml\t1\n 1 )` reads the same as the canonical `(eml 1 1)`.
    let canonical = read("(eml 1 1)").expect("canonical form should read");
    assert_eq!(read("(  eml\t1\n 1 )"), Ok(canonical));
}

#[test]
fn ac2_line_comments_are_ignored() {
    // AC2 — `;` begins a line comment to end of line (LISP convention); the
    // commented input parses to the same tree as the bare form.
    let src = "; leading comment\n(eml 1 1) ; trailing comment\n";
    let canonical = read("(eml 1 1)").expect("canonical form should read");
    assert_eq!(read(src), Ok(canonical));
}

#[test]
fn ac2_inf_and_nan_tokens_are_symbols() {
    // AC2 — non-finite numeric spellings (`inf`, `nan`, which `f64::from_str`
    // accepts) are classified as `Sym`, NOT `Num` (SPEC-0003 §2.3). This keeps
    // `Num` always finite, which is what makes the round-trip total.
    assert_eq!(read("inf"), Ok(Sexpr::sym("inf")));
    assert_eq!(read("nan"), Ok(Sexpr::sym("nan")));
    assert_eq!(read("infinity"), Ok(Sexpr::sym("infinity")));
}

#[test]
fn ac2_finite_numeric_spellings_are_numbers() {
    // AC2 — finite numeric tokens parse to `Num` (SPEC-0003 §2.3): `1`, `2.5`,
    // `-1`, `1e0`. (`1e0` == 1.0 exactly; `-1` is finite.)
    assert_eq!(read("1"), Ok(Sexpr::num(1.0)));
    assert_eq!(read("2.5"), Ok(Sexpr::num(2.5)));
    assert_eq!(read("-1"), Ok(Sexpr::num(-1.0)));
    assert_eq!(read("1e0"), Ok(Sexpr::num(1.0)));
}

#[test]
fn ac2_empty_list_reads_as_list() {
    // AC2 — `()` is valid *data* and reads as `List([])`. It is rejected later,
    // at lowering (AC3 / `LowerError::NotAForm`), not by the reader (§2.3).
    assert_eq!(read("()"), Ok(Sexpr::list([])));
}

#[test]
fn ac2_bare_close_paren_is_unexpected_close() {
    // AC2 — a stray `)` with no open list is `ReadError::UnexpectedClose`.
    assert_eq!(read(")"), Err(ReadError::UnexpectedClose));
}

#[test]
fn ac2_unclosed_list_is_unclosed_list() {
    // AC2 — an unclosed `(eml 1` at end of input is `ReadError::UnclosedList`.
    assert_eq!(read("(eml 1"), Err(ReadError::UnclosedList));
}

#[test]
fn ac2_empty_input_is_empty_input() {
    // AC2 — empty and whitespace/comment-only input is `ReadError::EmptyInput`
    // (there is no s-expression to read).
    assert_eq!(read(""), Err(ReadError::EmptyInput));
    assert_eq!(read("   \n\t  "), Err(ReadError::EmptyInput));
    assert_eq!(read("; only a comment\n"), Err(ReadError::EmptyInput));
}

#[test]
fn ac2_two_top_level_forms_are_trailing_tokens() {
    // AC2 — `read` returns exactly one top-level form; a second one is
    // `ReadError::TrailingTokens` (SPEC-0003 §2.3).
    assert_eq!(read("(eml 1 1) (eml 1 1)"), Err(ReadError::TrailingTokens));
}

#[test]
fn ac2_trailing_atom_after_form_is_trailing_tokens() {
    // AC2 — a trailing atom after a complete first form is also
    // `TrailingTokens` (the "stray tokens" case, manifesting per §2.3).
    assert_eq!(read("(eml 1 1) x"), Err(ReadError::TrailingTokens));
}

// ===========================================================================
// AC3 — Lowering the `eml` form (Sexpr → Eml; typed LowerError; before eval)
//
// "A well-formed `eml` form lowers to R-0001's `Eml` … Lowering validates
// structure at lowering time — an unknown head symbol, an `eml` with other
// than two arguments, or a non-form list yields a lowering error reported via
// a typed `LowerError` enum." (R-0003 AC3; SPEC-0003 §2.4 table.)
//
// `lower` returns the typed-core `Eml`, which derives `PartialEq`, so we
// assert against `Eml` values built from `ufl_core`'s own constructors.
// ===========================================================================

#[test]
fn ac3_num_one_lowers_to_eml_one() {
    // AC3 — `Num(1.0)` → `Eml::One` (the primitive is the *value* 1; 1.0 is
    // exactly representable, so the value-equality check is total).
    assert_eq!(lower(&Sexpr::num(1.0)), Ok(ufl_core::Eml::One));
}

#[test]
fn ac3_non_one_literal_is_unsupported() {
    // AC3 — `Num(n)` for `n != 1.0` → `LowerError::UnsupportedLiteral(n)`,
    // including `2.5` and the negative-zero edge case `-0.0` (which is `!= 1.0`
    // and must be reported, not silently accepted).
    assert_eq!(
        lower(&Sexpr::num(2.5)),
        Err(LowerError::UnsupportedLiteral(2.5))
    );
    assert_eq!(
        lower(&Sexpr::num(0.0)),
        Err(LowerError::UnsupportedLiteral(0.0))
    );
    assert_eq!(
        lower(&Sexpr::num(-0.0)),
        Err(LowerError::UnsupportedLiteral(-0.0))
    );
}

#[test]
fn ac3_symbol_lowers_to_var() {
    // AC3 — `Sym(s)` → `Eml::var(s)` (a variable, resolved at eval time).
    assert_eq!(lower(&Sexpr::sym("x")), Ok(ufl_core::Eml::var("x")));
}

#[test]
fn ac3_eml_form_lowers_to_node() {
    // AC3 — `List([Sym("eml"), a, b])` → `Eml::node(lower a, lower b)`.
    // `(eml 1 1)` → `Eml::node(One, One)`.
    let form = Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0), Sexpr::num(1.0)]);
    assert_eq!(
        lower(&form),
        Ok(ufl_core::Eml::node(ufl_core::Eml::One, ufl_core::Eml::One))
    );
}

#[test]
fn ac3_nested_eml_form_lowers_recursively() {
    // AC3 — the nested `(eml x 1)` lowers to `Eml::node(Var("x"), One)`,
    // confirming recursion through the children.
    let form = Sexpr::list([Sexpr::sym("eml"), Sexpr::sym("x"), Sexpr::num(1.0)]);
    assert_eq!(
        lower(&form),
        Ok(ufl_core::Eml::node(
            ufl_core::Eml::var("x"),
            ufl_core::Eml::One
        ))
    );
}

#[test]
fn ac3_eml_with_zero_args_is_arity_error() {
    // AC3 — `(eml)` → `LowerError::Arity { form: "eml", expected: 2, got: 0 }`.
    let form = Sexpr::list([Sexpr::sym("eml")]);
    assert_eq!(
        lower(&form),
        Err(LowerError::Arity {
            form: "eml".to_string(),
            expected: 2,
            got: 0,
        })
    );
}

#[test]
fn ac3_eml_with_one_arg_is_arity_error() {
    // AC3 — `(eml 1)` → arity error with `got: 1`.
    let form = Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0)]);
    assert_eq!(
        lower(&form),
        Err(LowerError::Arity {
            form: "eml".to_string(),
            expected: 2,
            got: 1,
        })
    );
}

#[test]
fn ac3_eml_with_three_args_is_arity_error() {
    // AC3 — `(eml 1 1 1)` → arity error with `got: 3`.
    let form = Sexpr::list([
        Sexpr::sym("eml"),
        Sexpr::num(1.0),
        Sexpr::num(1.0),
        Sexpr::num(1.0),
    ]);
    assert_eq!(
        lower(&form),
        Err(LowerError::Arity {
            form: "eml".to_string(),
            expected: 2,
            got: 3,
        })
    );
}

#[test]
fn ac3_unknown_head_is_unknown_form() {
    // AC3 — `(foo 1 1)` → `LowerError::UnknownForm("foo")` (only `eml` is known
    // in R-0003). The arity is *not* checked for an unknown head.
    let form = Sexpr::list([Sexpr::sym("foo"), Sexpr::num(1.0), Sexpr::num(1.0)]);
    assert_eq!(
        lower(&form),
        Err(LowerError::UnknownForm("foo".to_string()))
    );
}

#[test]
fn ac3_empty_list_is_not_a_form() {
    // AC3 — `()` is valid data but not a form → `LowerError::NotAForm`. This is
    // where R-0003 AC2's "empty application" is rejected (decision log).
    assert_eq!(lower(&Sexpr::list([])), Err(LowerError::NotAForm));
}

#[test]
fn ac3_non_symbol_head_is_not_a_form() {
    // AC3 — a list whose head is not a symbol is `LowerError::NotAForm`:
    // `(1 1)` (numeric head) and `((eml 1 1) 1)` (list head).
    assert_eq!(
        lower(&Sexpr::list([Sexpr::num(1.0), Sexpr::num(1.0)])),
        Err(LowerError::NotAForm)
    );
    let list_head = Sexpr::list([
        Sexpr::list([Sexpr::sym("eml"), Sexpr::num(1.0), Sexpr::num(1.0)]),
        Sexpr::num(1.0),
    ]);
    assert_eq!(lower(&list_head), Err(LowerError::NotAForm));
}

#[test]
fn ac3_unsupported_literal_inside_form_propagates() {
    // AC3 — lowering recurses, so a non-`1` literal *inside* a well-formed
    // `eml` form surfaces the child's `UnsupportedLiteral`: `(eml 2.5 1)`.
    let form = Sexpr::list([Sexpr::sym("eml"), Sexpr::num(2.5), Sexpr::num(1.0)]);
    assert_eq!(lower(&form), Err(LowerError::UnsupportedLiteral(2.5)));
}

// ===========================================================================
// AC4 — Behavioural parity with R-0001, through the typed core
//
// "Through the `text → Sexpr → Eml → eval` path the identities hold within
// R-0001's tolerance, over inputs including negative real x:
//   (eml 1 1) = e ; (eml x 1) = exp(x) ; (eml 1 (eml (eml 1 x) 1)) = ln(x)."
// (R-0003 AC4; SPEC-0003 §6 AC4 — tolerance relative 1e-14.) Driven entirely
// through `eval_str`, which reuses `ufl_core::eval` verbatim.
// ===========================================================================

#[test]
fn ac4_e_identity_through_eval_str() {
    // AC4 — `eval_str("(eml 1 1)") ≈ e`.
    let v = eval_str("(eml 1 1)", &Env::new()).expect("(eml 1 1) should evaluate");
    assert!(
        close(v, Value::new(std::f64::consts::E, 0.0), AC4_TOL),
        "(eml 1 1) should be e, got {v:?}"
    );
}

#[test]
fn ac4_exp_identity_through_eval_str() {
    // AC4 — `eval_str("(eml x 1)", env{x=q}) ≈ exp(q)` for q ∈ {0, 1, 2.5}.
    for q in [0.0, 1.0, 2.5] {
        let v = eval_str("(eml x 1)", &env_with_x(q)).expect("(eml x 1) should evaluate");
        let expected = Value::new(q, 0.0).exp();
        assert!(
            close(v, expected, AC4_TOL),
            "exp identity failed at x={q}: actual={v:?}, expected={expected:?}"
        );
    }
}

#[test]
fn ac4_ln_identity_through_eval_str_positive_reals() {
    // AC4 — `eval_str("(eml 1 (eml (eml 1 x) 1))", env{x=q}) ≈ ln(q)` for
    // positive q ∈ {0.5, 1, 2.5}.
    for q in [0.5, 1.0, 2.5] {
        let v =
            eval_str("(eml 1 (eml (eml 1 x) 1))", &env_with_x(q)).expect("ln form should evaluate");
        let expected = Value::new(q, 0.0).ln();
        assert!(
            close(v, expected, AC4_TOL),
            "ln identity failed at x={q}: actual={v:?}, expected={expected:?}"
        );
    }
}

#[test]
fn ac4_ln_identity_through_eval_str_negative_reals() {
    // AC4 — the ln form over negative real x, where the principal branch gives
    // a complex result. `ln(-1) ≈ 0 + (τ/2)i = 0 + πi` (SPEC-0003 §6 AC4 uses
    // `Value::new(0.0, std::f64::consts::PI)` as the reference). The branch
    // convention and `sin(τ/2)` self-correction are inherited from the reused
    // R-0001 evaluator.
    let v =
        eval_str("(eml 1 (eml (eml 1 x) 1))", &env_with_x(-1.0)).expect("ln(-1) should evaluate");
    let expected = Value::new(0.0, std::f64::consts::PI);
    assert!(
        close(v, expected, AC4_TOL),
        "ln(-1) should be principal (≈ 0 + πi), got {v:?}"
    );
    assert!(
        v.im > 0.0,
        "principal ln(-1) must have positive imaginary part, got {v:?}"
    );

    // And a second negative input, x = -3: `ln(-3) = ln(3) + πi`.
    let v =
        eval_str("(eml 1 (eml (eml 1 x) 1))", &env_with_x(-3.0)).expect("ln(-3) should evaluate");
    let expected = Value::new(3.0_f64.ln(), std::f64::consts::PI);
    assert!(
        close(v, expected, AC4_TOL),
        "ln(-3) should be ln(3) + πi, got {v:?}"
    );
}

// ===========================================================================
// AC5 — Extended reals (inherited from R-0001 via the reused evaluator)
//
// "`ln 0`, `exp(−∞)`, and signed-zero/infinity cases follow IEEE-754 and
// propagate as ordinary values — no trap, panic, or abort." (R-0003 AC5;
// SPEC-0003 §6 AC5.) Exercised through `eval_str` to prove the behaviour
// survives the read → lower → eval path.
// ===========================================================================

#[test]
fn ac5_ln_of_zero_propagates_without_panic() {
    // AC5 — drive ln(0) through the lowered `ln` tree with x = 0. `ln(0)` is
    // `-∞ + 0i` (num-complex), so the lowered form yields a real -∞ with no
    // panic. Asserts the value is infinite (the IEEE result), not a trap.
    let v = eval_str("(eml 1 (eml (eml 1 x) 1))", &env_with_x(0.0))
        .expect("ln(0) must not panic or error");
    assert!(
        v.re.is_infinite() && v.re.is_sign_negative(),
        "ln(0) should be -inf real part, got {v:?}"
    );
}

#[test]
fn ac5_exp_of_neg_infinity_propagates_without_panic() {
    // AC5 — `exp(-∞)` through `(eml x 1)` with x = -∞ is `exp(-∞) - ln(1)` =
    // `0 - 0` = `0`. Finite, no panic — the IEEE limit propagates as a value.
    let mut env = Env::new();
    env.bind("x", Value::new(f64::NEG_INFINITY, 0.0));
    let v = eval_str("(eml x 1)", &env).expect("exp(-inf) must not panic or error");
    assert!(
        v.re.is_finite(),
        "exp(-inf) real part should be finite, got {v:?}"
    );
    assert_eq!(v.re, 0.0);
}

#[test]
fn ac5_finite_form_is_finite() {
    // AC5 — the ordinary `(eml 1 1)` stays finite (the non-degenerate baseline
    // alongside the inf/nan cases): `e` is finite.
    let v = eval_str("(eml 1 1)", &Env::new()).expect("(eml 1 1) should evaluate");
    assert!(v.is_finite(), "(eml 1 1) should be finite, got {v:?}");
}

// ===========================================================================
// AC6 — Error model: typed, layered, no panics
//
// "Every failure surfaces as a typed error enum, never a panic, at the
// earliest layer that can detect it: `ReadError` (lexical/syntactic),
// `LowerError` (unknown form, wrong arity, non-form) at the lowering boundary,
// and `EvalError::UnboundVariable` at evaluation." (R-0003 AC6; SPEC-0003
// §2.5 / §6 AC6.) Verified through `eval_str`, whose `UflError` unifies the
// three layers.
// ===========================================================================

#[test]
fn ac6_read_layer_error_is_uflerror_read() {
    // AC6 — a lexical/syntactic failure surfaces at the *read* layer:
    // `eval_str(")")` → `UflError::Read(ReadError::UnexpectedClose)`.
    assert_eq!(
        eval_str(")", &Env::new()),
        Err(UflError::Read(ReadError::UnexpectedClose))
    );
}

#[test]
fn ac6_lower_layer_error_is_uflerror_lower() {
    // AC6 — a grammar failure surfaces at the *lowering* layer:
    // `eval_str("(foo 1 1)")` → `UflError::Lower(LowerError::UnknownForm(..))`.
    assert_eq!(
        eval_str("(foo 1 1)", &Env::new()),
        Err(UflError::Lower(LowerError::UnknownForm("foo".to_string())))
    );
}

#[test]
fn ac6_eval_layer_error_is_uflerror_eval() {
    // AC6 — an unbound variable surfaces at the *eval* layer (the genuinely
    // dynamic frontier): `eval_str("x")` with an empty env →
    // `UflError::Eval(EvalError::UnboundVariable("x"))`.
    assert_eq!(
        eval_str("x", &Env::new()),
        Err(UflError::Eval(EvalError::UnboundVariable("x".to_string())))
    );
}

#[test]
fn ac6_lowering_error_precedes_eval_for_empty_application() {
    // AC6 — the earliest detecting layer wins: `()` is a *lowering* error
    // (`NotAForm`), reported before evaluation is ever reached.
    assert_eq!(
        eval_str("()", &Env::new()),
        Err(UflError::Lower(LowerError::NotAForm))
    );
}

#[test]
fn ac6_read_error_precedes_lowering_for_unclosed_list() {
    // AC6 — read errors precede lowering: an unclosed `(eml 1` never reaches
    // the lowering boundary → `UflError::Read(ReadError::UnclosedList)`.
    assert_eq!(
        eval_str("(eml 1", &Env::new()),
        Err(UflError::Read(ReadError::UnclosedList))
    );
}

#[test]
fn ac2_deeply_nested_list_is_recursion_depth_exceeded() {
    let opens = "(".repeat(129);
    let closes = ")".repeat(129);
    let src = format!("{}1{}", opens, closes);
    assert_eq!(read(&src), Err(ReadError::RecursionDepthExceeded));

    let opens_ok = "(".repeat(128);
    let closes_ok = ")".repeat(128);
    let src_ok = format!("{}1{}", opens_ok, closes_ok);
    assert!(read(&src_ok).is_ok());
}
