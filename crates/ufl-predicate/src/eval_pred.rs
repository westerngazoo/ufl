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
pub fn eval_pred(_s: &Sexpr, _env: &Env) -> Result<bool, PredError> {
    unimplemented!("R-0004 implementation — eval_pred, see SPEC-0004 §3")
}

/// Evaluate a numeric operand of `=`. Boolean-shaped operands are a type error
/// caught before `lower` (SPEC-0004 §2.2). Used by the `=` form.
#[allow(dead_code)]
fn eval_num(s: &Sexpr, env: &Env) -> Result<Value, PredError> {
    if matches!(classify(s), Mode::Boolean) {
        return Err(PredError::ExpectedNumber { found: describe(s) });
    }
    let eml = ufl_syntax::lower(s)?;
    Ok(ufl_core::eval(&eml, env)?)
}
