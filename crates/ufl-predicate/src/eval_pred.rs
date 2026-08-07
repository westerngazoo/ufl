//! The boolean evaluator and the single classifier (SPEC-0004 §2.2–§2.6).

use ufl_core::{Env, EvalError, Value};
use ufl_syntax::{LowerError, Sexpr};

/// A failure while evaluating a predicate (SPEC-0004 §2.6).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PredError {
    #[error("expected a boolean expression, found {found}")]
    ExpectedBool { found: String },
    #[error("expected a numeric expression, found {found}")]
    ExpectedNumber { found: String },
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

/// Which evaluation mode an `Sexpr` denotes (SPEC-0004 §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Numeric,
    Boolean,
}

fn is_pred_head(h: &str) -> bool {
    matches!(h, "and" | "or" | "not" | "=" | "pred")
}

/// The single classifier consulted by both evaluation modes. `true`/`false`
/// are boolean at the predicate/numeric boundary; every other symbol is a
/// numeric variable; a list is boolean iff its head is a predicate form.
fn classify(s: &Sexpr) -> Mode {
    match s {
        Sexpr::Sym(t) if t == "true" || t == "false" => Mode::Boolean,
        Sexpr::Sym(_) | Sexpr::Num(_) => Mode::Numeric,
        Sexpr::List(items) => match items.split_first() {
            Some((Sexpr::Sym(head), _)) if is_pred_head(head) => Mode::Boolean,
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

/// Evaluate a numeric operand of `=`. Boolean-shaped operands are a type error
/// caught before `lower` (SPEC-0004 §2.2).
fn eval_num(s: &Sexpr, env: &Env) -> Result<Value, PredError> {
    if matches!(classify(s), Mode::Boolean) {
        return Err(PredError::ExpectedNumber { found: describe(s) });
    }
    let eml = ufl_syntax::lower(s)?;
    Ok(ufl_core::eval(&eml, env)?)
}
