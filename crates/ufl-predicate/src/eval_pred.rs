//! The boolean evaluator and the single classifier (SPEC-0004 §2.2–§2.6).

use ufl_core::{Env, EvalError, Value};
use ufl_syntax::{LowerError, Sexpr};

/// A failure while evaluating a predicate (SPEC-0004 §2.6, SPEC-0016 §2.2).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PredError {
    #[error("expected a boolean expression, found {found}")]
    ExpectedBool { found: String },
    #[error("expected a numeric expression, found {found}")]
    ExpectedNumber { found: String },
    #[error("expected a syntax expression (a `quote` form), found {found}")]
    ExpectedSyntax { found: String },
    #[error("`{form}` expects {expected} arguments, got {got}")]
    Arity {
        form: String,
        expected: usize,
        got: usize,
    },
    #[error(transparent)]
    Lower(#[from] LowerError),
    #[error(transparent)]
    Eval(#[from] EvalError),
}

/// Which evaluation mode an `Sexpr` denotes (SPEC-0004 §2.2, SPEC-0016 §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Numeric,
    Boolean,
    Syntax,
}

fn is_pred_head(h: &str) -> bool {
    matches!(h, "and" | "or" | "not" | "=" | "eq?" | "pred")
}

/// The only syntax-producing head in Rung 1 (SPEC-0016 §2.1). `eval` is *not*
/// here: `(eval q)` denotes a `Value`, so it is a numeric head, dispatched in
/// [`eval_num`] (§2.3), not classified as `Syntax`.
fn is_syntax_head(h: &str) -> bool {
    h == "quote"
}

/// The single classifier consulted by every evaluation mode. `true`/`false`
/// are boolean at the predicate/numeric boundary; every other symbol is a
/// numeric variable; a list is boolean iff its head is a predicate form, syntax
/// iff its head is a syntax form, else numeric.
fn classify(s: &Sexpr) -> Mode {
    match s {
        Sexpr::Sym(t) if t == "true" || t == "false" => Mode::Boolean,
        Sexpr::Sym(_) | Sexpr::Num(_) => Mode::Numeric,
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(head), _)) if is_pred_head(head) => Mode::Boolean,
            Some((Sexpr::Sym(head), _)) if is_syntax_head(head) => Mode::Syntax,
            _ => Mode::Numeric,
        },
    }
}

/// A short human description of an `Sexpr` for error payloads.
fn describe(s: &Sexpr) -> String {
    match s {
        Sexpr::Num(n) => format!("number {n}"),
        Sexpr::Sym(t) => format!("symbol `{t}`"),
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(head), _)) => format!("form `{head}`"),
            _ => "non-form list".to_string(),
        },
    }
}

/// Evaluate a predicate `Sexpr` to a boolean under `env` (SPEC-0004 §3).
pub fn eval_pred(s: &Sexpr, env: &Env) -> Result<bool, PredError> {
    match s {
        Sexpr::Sym(t) if t == "true" => Ok(true),
        Sexpr::Sym(t) if t == "false" => Ok(false),
        Sexpr::List(items) => eval_form(items, env),
        _ => Err(PredError::ExpectedBool { found: describe(s) }),
    }
}

/// Dispatch a list form in boolean position on its head symbol. This `match` is
/// the documented seam where the deferred control forms (`;`, choice, fixpoint)
/// plug in — one arm at a time (SPEC-0004 §2.3).
fn eval_form(items: &[Sexpr], env: &Env) -> Result<bool, PredError> {
    let Some((Sexpr::Sym(head), args)) = items.split_first() else {
        return Err(PredError::ExpectedBool {
            found: "non-form list".to_string(),
        });
    };
    match head.as_str() {
        "pred" => match args {
            [e] => eval_pred(e, env),
            _ => Err(arity("pred", 1, args.len())),
        },
        "not" => match args {
            [p] => Ok(!eval_pred(p, env)?),
            _ => Err(arity("not", 1, args.len())),
        },
        "=" => match args {
            [a, b] => Ok(eval_num(a, env)? == eval_num(b, env)?),
            _ => Err(arity("=", 2, args.len())),
        },
        // Structural equality on *syntax* (SPEC-0016 §2.4): compares the denoted
        // `Sexpr`s (the `quote` children) by the exact, decidable
        // `Sexpr::PartialEq`. Distinct from numeric `=` — no mixed-mode case.
        "eq?" => match args {
            [a, b] => Ok(eval_syntax(a, env)? == eval_syntax(b, env)?),
            _ => Err(arity("eq?", 2, args.len())),
        },
        // Lazy short-circuit: an unreached operand's error is not surfaced.
        "and" => {
            for p in args {
                if !eval_pred(p, env)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        "or" => {
            for p in args {
                if eval_pred(p, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        other => Err(PredError::ExpectedBool {
            found: format!("form `{other}`"),
        }),
    }
}

fn arity(form: &str, expected: usize, got: usize) -> PredError {
    PredError::Arity {
        form: form.to_string(),
        expected,
        got,
    }
}

/// Evaluate a numeric operand. Boolean-shaped operands are a type error caught
/// before `lower` (SPEC-0004 §2.2). A `(eval q)` form is dispatched here (it
/// denotes a `Value`), discharging `q` through the *one* pipeline — `eval_syntax`
/// then the same `lower` + `ufl_core::eval` every numeric expression uses
/// (SPEC-0016 §2.3); no second evaluator. Every other list falls through
/// unchanged, so all existing numeric results are preserved.
fn eval_num(s: &Sexpr, env: &Env) -> Result<Value, PredError> {
    if matches!(classify(s), Mode::Boolean) {
        return Err(PredError::ExpectedNumber { found: describe(s) });
    }
    // Numeric form-dispatch on head, before `lower`. One arm today: `eval`.
    if let Sexpr::List(items) = s {
        if let Some((Sexpr::Sym(head), args)) = items.split_first() {
            if head == "eval" {
                return match args {
                    [q] => {
                        let quoted = eval_syntax(q, env)?; // q must be a `quote`
                        let eml = ufl_syntax::lower(&quoted)?; // the SAME lowering
                        Ok(ufl_core::eval(&eml, env)?) // the SAME evaluator
                    }
                    _ => Err(arity("eval", 1, args.len())),
                };
            }
        }
    }
    let eml = ufl_syntax::lower(s)?;
    Ok(ufl_core::eval(&eml, env)?)
}

/// Evaluate an `Sexpr` in *syntax* position to the `Sexpr` it denotes
/// (SPEC-0016 §2.2). The only syntax-producing form is `(quote e)`, which yields
/// its child **unevaluated** — `e` is not lowered, so a non-numeric child (a
/// bare symbol, a nested `quote`, a future form) is legal *as data*. Anything
/// that is not a `quote` is [`PredError::ExpectedSyntax`] (the payload is a
/// `describe`d `String`, never a private `Mode`).
fn eval_syntax(s: &Sexpr, _env: &Env) -> Result<Sexpr, PredError> {
    match s {
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(h), [e])) if h == "quote" => Ok(e.clone()),
            Some((Sexpr::Sym(h), args)) if h == "quote" => Err(arity("quote", 1, args.len())),
            _ => Err(PredError::ExpectedSyntax { found: describe(s) }),
        },
        _ => Err(PredError::ExpectedSyntax { found: describe(s) }),
    }
}
