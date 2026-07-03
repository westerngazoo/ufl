//! R-0016 — Reflection rung 1 (`quote` / `eval` / `eq?`): acceptance tests.
//!
//! One section per acceptance criterion, in SPEC-0016 §2.6 order. Each test
//! cites its AC id so the architect (PR review) and the orchestrator (status
//! update) can map the suite to R-0016's acceptance criteria mechanically.
//!
//! Authored at loop step 3 (test plan), **before** the implementation — so the
//! suite is RED until `ufl-predicate` gains `Mode::Syntax`, `eval_syntax`, the
//! numeric `eval` form-dispatch, the `eq?` boolean head, and
//! `PredError::ExpectedSyntax`.
//!
//! See:
//!  - `requirements/0016-reflection-quote-eval-raise.md` — AC1–AC5
//!  - `specs/0016-reflection-quote-eval-raise.md` — §2.1 (the Syntax mode),
//!    §2.2 (`quote`), §2.3 (`eval` through the one pipeline), §2.4 (`eq?`;
//!    numeric `=` untouched), §2.6 (the AC list with exact expected behaviours).

use ufl_core::Env;
use ufl_predicate::{eval_pred, PredError};
use ufl_prng::SplitMix64;
use ufl_syntax::{read, LowerError, Sexpr};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read a predicate `Sexpr` from text, panicking on a read error. The read
/// layer is verified by R-0003 and is not the unit under test here.
fn p(src: &str) -> Sexpr {
    read(src).expect("test predicate text should read")
}

/// Generate a lowerable, error-free `E` drawn from `{1, var(bound), eml(E, E)}`
/// (SPEC-0016 §2.3 — the domain on which `eval ∘ quote = id`). Every `var` it
/// emits is bound in [`ac1_env`], so `eval` never errors. `depth` bounds the
/// recursion (AC1 needs only that the domain is exercised, not unboundedly).
fn gen_dom_eval(rng: &mut SplitMix64, depth: u32) -> Sexpr {
    // At depth 0, a leaf: `1` or a bound variable. Otherwise, sometimes recurse.
    let leaf = depth == 0 || rng.below(2) == 0;
    if leaf {
        match rng.below(3) {
            0 => Sexpr::num(1.0),
            1 => Sexpr::sym("x"),
            _ => Sexpr::sym("y"),
        }
    } else {
        Sexpr::list([
            Sexpr::sym("eml"),
            gen_dom_eval(rng, depth - 1),
            gen_dom_eval(rng, depth - 1),
        ])
    }
}

/// The environment in which every variable [`gen_dom_eval`] emits (`x`, `y`) is
/// bound — so a generated `E` is always in `dom(eval)` (SPEC-0016 §2.3).
fn ac1_env() -> Env {
    let mut env = Env::new();
    env.bind("x", ufl_core::Value::new(1.0, 0.0));
    env.bind("y", ufl_core::Value::new(2.0, 0.0));
    env
}

/// Wrap a denoted form `e` as the predicate `(= (eval (quote e)) e)` — AC1's
/// exact shape: does `(eval (quote E))` equal `E`?
fn eval_quote_eq_self(e: &Sexpr) -> Sexpr {
    Sexpr::list([
        Sexpr::sym("="),
        Sexpr::list([
            Sexpr::sym("eval"),
            Sexpr::list([Sexpr::sym("quote"), e.clone()]),
        ]),
        e.clone(),
    ])
}

/// The self-equality `(= E E)` — the oracle for observing `eval ∘ quote = id`
/// through `=`: whatever verdict `=` gives `E` against itself (`Ok(true)`, or
/// `Ok(false)` when `E`'s value is `NaN`), it must give `(eval (quote E))`
/// against `E`. The reflection layer must be *indistinguishable from identity*
/// under `=`.
fn self_eq(e: &Sexpr) -> Sexpr {
    Sexpr::list([Sexpr::sym("="), e.clone(), e.clone()])
}

// ===========================================================================
// AC1 — eval ∘ quote = id on dom(eval).
//
// "property test over generated `Sexpr`s in the reader's image:
// `⟦(= (eval (quote E)) E)⟧` holds for every sampled bound `E`." (R-0016 AC1;
// SPEC-0016 §2.3, §2.6 test 1.) The generator is restricted to lowerable,
// error-free `E` (`{1, var(bound), eml(E, E)}`), so the identity is exercised
// on exactly its domain — not over-claimed as total.
//
// `(eval (quote E))` and a bare `E` take the *byte-identical* pipeline (§2.3),
// so `eval ∘ quote` is indistinguishable from identity. Observed through `=`
// that means `⟦(= (eval (quote E)) E)⟧ == ⟦(= E E)⟧` for EVERY E in dom(eval):
// `Ok(true)` on finite values, and equally `Ok(false)` where E's value is `NaN`
// (`=` is IEEE value equality, non-reflexive on `NaN` — SPEC-0004 §2.4 / AC2, a
// partiality the reflection layer inherits honestly, §2.3). Where `(= E E)` is
// `Ok(true)`, AC1's exact form `⟦(= (eval (quote E)) E)⟧` is `Ok(true)` too.
// ===========================================================================

#[test]
fn eval_quote_is_identity_on_dom_eval() {
    let mut rng = SplitMix64::new(0x5EED_0016);
    let env = ac1_env();
    let mut saw_true = false;
    for _ in 0..4_000 {
        let e = gen_dom_eval(&mut rng, 4);

        // Indistinguishable-from-identity under `=`, over the FULL domain:
        // `(eval (quote E))` vs `E` agrees with `E` vs `E`, NaN cases included.
        let via_eval = eval_pred(&eval_quote_eq_self(&e), &env);
        let via_self = eval_pred(&self_eq(&e), &env);
        assert_eq!(
            via_eval, via_self,
            "`(= (eval (quote E)) E)` must match `(= E E)` (eval ∘ quote ≡ id under =); E = {e}"
        );

        // AC1's exact form on the sub-domain where `=` sees reflexivity.
        if via_self == Ok(true) {
            saw_true = true;
            assert_eq!(
                eval_pred(&eval_quote_eq_self(&e), &env),
                Ok(true),
                "`(= (eval (quote E)) E)` must hold where E's value is not NaN; E = {e}"
            );
        }
    }
    assert!(
        saw_true,
        "the generator must exercise the non-NaN (Ok(true)) case"
    );
}

#[test]
fn eval_quote_is_identity_on_the_canonical_forms() {
    // AC1 — the three shapes spelled out, so a generator regression is caught
    // by named cases too: `1`, a bound `var`, and an `eml` form. All finite, so
    // the `⟦=⟧` form is `Ok(true)`.
    let env = ac1_env();
    for e in [
        Sexpr::num(1.0),
        Sexpr::sym("x"),
        Sexpr::list([Sexpr::sym("eml"), Sexpr::sym("x"), Sexpr::num(1.0)]),
    ] {
        assert_eq!(
            eval_pred(&eval_quote_eq_self(&e), &env),
            Ok(true),
            "`(= (eval (quote E)) E)` must hold for E = {e}"
        );
    }
}

// ===========================================================================
// AC2 — quote does not evaluate.
//
// "`(quote (eml y 1))` with `y` unbound discharges WITHOUT `UnboundVariable`
// (quote does not evaluate); `(quote e)` in numeric position fails typed
// (`UnknownForm("quote")`)." (R-0016 AC2; SPEC-0016 §2.2, §2.6 test 2.)
//
// The "does not evaluate" half is observed through the only public form that
// invokes the syntax evaluator on a `quote` and surfaces its denoted `Sexpr`:
// `eq?`. `(eq? (quote (eml y 1)) (quote (eml y 1)))` with `y` unbound is
// `Ok(true)` — had `quote` evaluated its child, `y`'s absence would have
// leaked as `UnboundVariable`.
// ===========================================================================

#[test]
fn quote_does_not_evaluate() {
    // The child `(eml y 1)` is never lowered/evaluated, so `y` unbound is fine:
    // both operands denote the same `Sexpr`, and `eq?` returns `true` — no
    // `UnboundVariable`.
    let got = eval_pred(&p("(eq? (quote (eml y 1)) (quote (eml y 1)))"), &Env::new());
    assert_eq!(
        got,
        Ok(true),
        "quote must not evaluate its child: an unbound `y` under quote must not error"
    );
    assert_ne!(
        got,
        Err(PredError::Eval(ufl_core::EvalError::UnboundVariable(
            "y".to_string()
        ))),
        "`(quote (eml y 1))` must not leak `UnboundVariable(\"y\")`"
    );

    // The numeric-position half: `(quote e)` reached by the numeric path (as a
    // `=` operand) is not a numeric form, so `lower` rejects the head `quote` as
    // `UnknownForm("quote")` — a typed failure, never a silent coercion.
    assert_eq!(
        eval_pred(&p("(= (quote e) 1)"), &Env::new()),
        Err(PredError::Lower(LowerError::UnknownForm(
            "quote".to_string()
        ))),
        "`(quote e)` in numeric position must fail typed as UnknownForm(\"quote\")"
    );
}

// ===========================================================================
// AC3 — structural equality on syntax is `eq?` (numeric `=` untouched).
//
// "`(eq? (quote (eml 1 1)) (quote (eml 1 1)))` is `Ok(true)`; differing forms
// `Ok(false)`; a non-syntax operand is `ExpectedSyntax`. Numeric `=` unaffected:
// a pinned SPEC-0004 `=` suite still passes verbatim." (R-0016 AC3; SPEC-0016
// §2.4, §2.6 test 3.)
// ===========================================================================

#[test]
fn eq_on_syntax_is_structural() {
    // Equal denoted forms → true (exact, decidable `Sexpr::PartialEq`).
    assert_eq!(
        eval_pred(&p("(eq? (quote (eml 1 1)) (quote (eml 1 1)))"), &Env::new()),
        Ok(true)
    );
    // Differing denoted forms → false.
    assert_eq!(
        eval_pred(&p("(eq? (quote (eml 1 1)) (quote (eml 1 x)))"), &Env::new()),
        Ok(false)
    );
    // Deeper structural difference is still decidable.
    assert_eq!(
        eval_pred(
            &p("(eq? (quote (eml 1 (eml 1 x))) (quote (eml 1 x)))"),
            &Env::new()
        ),
        Ok(false)
    );

    // A non-syntax operand (a bare `1`, not a `(quote …)`) is `ExpectedSyntax`.
    match eval_pred(&p("(eq? (quote (eml 1 1)) 1)"), &Env::new()) {
        Err(PredError::ExpectedSyntax { .. }) => {}
        other => panic!("`eq?` with a non-syntax operand should be ExpectedSyntax, got {other:?}"),
    }
    // Symmetric: a non-syntax first operand is likewise `ExpectedSyntax`.
    match eval_pred(&p("(eq? x (quote (eml 1 1)))"), &Env::new()) {
        Err(PredError::ExpectedSyntax { .. }) => {}
        other => {
            panic!("`eq?` with a non-syntax first operand should be ExpectedSyntax, got {other:?}")
        }
    }

    // `eq?` is arity 2.
    match eval_pred(&p("(eq? (quote 1))"), &Env::new()) {
        Err(PredError::Arity { form, .. }) if form == "eq?" => {}
        other => panic!("`(eq? …)` at wrong arity should be Arity{{form:\"eq?\"}}, got {other:?}"),
    }
}

#[test]
fn numeric_equality_is_unchanged_by_eq() {
    // AC3 — a pinned SPEC-0004 numeric `=` case still passes verbatim: `=` keeps
    // delegating to `eval_num`, unchanged. `(= 1 1)` is `true`; a boolean
    // operand is still `ExpectedNumber` (not touched by the `eq?` addition).
    assert_eq!(eval_pred(&p("(= 1 1)"), &Env::new()), Ok(true));
    match eval_pred(&p("(= true 1)"), &Env::new()) {
        Err(PredError::ExpectedNumber { .. }) => {}
        other => {
            panic!("`(= true 1)` must remain ExpectedNumber (numeric `=` untouched), got {other:?}")
        }
    }
    // And a bound-variable equality, exactly as SPEC-0004 AC2 pins it.
    let mut env = Env::new();
    env.bind("x", ufl_core::Value::new(1.0, 0.0));
    assert_eq!(eval_pred(&p("(= x 1)"), &env), Ok(true));
}

// ===========================================================================
// N1 — `(eval q)` is gated by the ONE `lower`.
//
// "`(eval (quote (eml 1 1)))` discharges; `(eval (quote (eval (quote 1))))`
// fails typed `UnknownForm("eval")` (the quoted body isn't lowerable) — assert
// the typed error, NOT a value." (SPEC-0016 §2.6 test 4, nit N1.)
//
// This pins that `(eval q)` reuses the *one* `lower` (which gates the quoted
// body to the lowerable class `{1, var, eml}`), never a second evaluator.
// ===========================================================================

#[test]
fn eval_body_is_gated_by_the_one_lower() {
    // A lowerable quoted body discharges through `lower` + `ufl_core::eval`.
    // `(= (eval (quote (eml 1 1))) (eml 1 1))`: both sides are the same numeric
    // value, so the equality is `true` — the `eval` path produced a value.
    assert_eq!(
        eval_pred(&p("(= (eval (quote (eml 1 1))) (eml 1 1))"), &Env::new()),
        Ok(true),
        "`(eval (quote (eml 1 1)))` must discharge to the numeric value of (eml 1 1)"
    );

    // The nested case: the outer `eval`'s quoted body is `(eval (quote 1))`,
    // whose head `eval` is unknown to `lower` (which knows only `eml`). So it is
    // `UnknownForm("eval")` — a typed error, never a silent value.
    assert_eq!(
        eval_pred(&p("(= (eval (quote (eval (quote 1)))) 1)"), &Env::new()),
        Err(PredError::Lower(LowerError::UnknownForm(
            "eval".to_string()
        ))),
        "the quoted body `(eval (quote 1))` is not lowerable → typed UnknownForm(\"eval\")"
    );
}

// ===========================================================================
// eval arity — `(eval q)` is arity 1 (SPEC-0016 §2.3).
// ===========================================================================

#[test]
fn eval_is_arity_one() {
    // `(eval)` and `(eval a b)` in numeric position are `Arity{form:"eval"}`.
    match eval_pred(&p("(= (eval) 1)"), &Env::new()) {
        Err(PredError::Arity { form, .. }) if form == "eval" => {}
        other => panic!("`(eval)` should be Arity{{form:\"eval\"}}, got {other:?}"),
    }
    match eval_pred(&p("(= (eval (quote 1) (quote 1)) 1)"), &Env::new()) {
        Err(PredError::Arity { form, .. }) if form == "eval" => {}
        other => panic!("`(eval a b)` should be Arity{{form:\"eval\"}}, got {other:?}"),
    }
}

// ===========================================================================
// eval of a non-quote operand — the operand of `eval` must be syntax.
// ===========================================================================

#[test]
fn eval_of_non_syntax_operand_is_expected_syntax() {
    // `(eval 1)`: the operand `1` is not a `(quote …)`, so `eval_syntax` rejects
    // it as `ExpectedSyntax` — the operand of `eval` denotes syntax (a quote).
    match eval_pred(&p("(= (eval 1) 1)"), &Env::new()) {
        Err(PredError::ExpectedSyntax { .. }) => {}
        other => panic!("`(eval 1)` should be ExpectedSyntax, got {other:?}"),
    }
}
