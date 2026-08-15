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
///
/// **Iterative** (R-0017): a continuation machine over an explicit frame stack,
/// so the boolean spine nests to any depth without recursion. Semantics are
/// unchanged — in particular `and`/`or` keep their **lazy short-circuit**, so an
/// unreached operand's error is still never surfaced.
///
/// Only `and`/`or`/`not`/`pred` deepen the spine. `=` and `eq?` delegate to
/// `eval_num`/`eval_syntax`, which do **not** re-enter here, so they are leaves
/// that compute one boolean and push it.
pub fn eval_pred(s: &Sexpr, env: &Env) -> Result<bool, PredError> {
    enum Frame<'a> {
        /// Launch a fresh predicate evaluation.
        Eval(&'a Sexpr),
        /// Resume: pop the operand's result; short-circuit on `false`, else continue.
        And(std::slice::Iter<'a, Sexpr>),
        /// Resume: pop the operand's result; short-circuit on `true`, else continue.
        Or(std::slice::Iter<'a, Sexpr>),
        /// Resume: pop and negate.
        Not,
    }

    let mut frames = vec![Frame::Eval(s)];
    let mut vals: Vec<bool> = Vec::new();

    while let Some(frame) = frames.pop() {
        match frame {
            Frame::Not => {
                let Some(v) = vals.pop() else {
                    unreachable!("not: its operand's Eval always pushes exactly one bool")
                };
                vals.push(!v);
            }
            Frame::And(mut rest) => {
                // Resume *by popping* the operand just evaluated (never peeking
                // — a peek would read a stale value).
                let Some(v) = vals.pop() else {
                    unreachable!("and: the launched operand always pushes exactly one bool")
                };
                if !v {
                    vals.push(false); // short-circuit: the remaining operands are never Eval'd
                } else {
                    match rest.next() {
                        Some(next) => {
                            frames.push(Frame::And(rest));
                            frames.push(Frame::Eval(next));
                        }
                        None => vals.push(true),
                    }
                }
            }
            Frame::Or(mut rest) => {
                let Some(v) = vals.pop() else {
                    unreachable!("or: the launched operand always pushes exactly one bool")
                };
                if v {
                    vals.push(true); // short-circuit
                } else {
                    match rest.next() {
                        Some(next) => {
                            frames.push(Frame::Or(rest));
                            frames.push(Frame::Eval(next));
                        }
                        None => vals.push(false),
                    }
                }
            }
            Frame::Eval(node) => match node {
                Sexpr::Sym(t) if t == "true" => vals.push(true),
                Sexpr::Sym(t) if t == "false" => vals.push(false),
                Sexpr::List(items) => {
                    let Some((Sexpr::Sym(head), args)) = items.split_first() else {
                        return Err(PredError::ExpectedBool {
                            found: "non-form list".to_string(),
                        });
                    };
                    // This `match` is the documented seam where the deferred
                    // control forms (`;`, choice, fixpoint) plug in — one arm at
                    // a time (SPEC-0004 §2.3).
                    match head.as_str() {
                        // `pred` is the identity on its operand's bool, so it is
                        // a **tail launch** — no resume frame. (Treating it as a
                        // leaf would mean calling `eval_pred` recursively, which
                        // is exactly the overflow this rewrite removes; and a
                        // recursive `pred` is invisible in release builds, where
                        // the compiler TCOs its tail call.)
                        "pred" => match args {
                            [e] => frames.push(Frame::Eval(e)),
                            _ => return Err(arity("pred", 1, args.len())),
                        },
                        "not" => match args {
                            [p] => {
                                frames.push(Frame::Not);
                                frames.push(Frame::Eval(p));
                            }
                            _ => return Err(arity("not", 1, args.len())),
                        },
                        // Leaves: neither re-enters the boolean spine.
                        "=" => match args {
                            [a, b] => vals.push(eval_num(a, env)? == eval_num(b, env)?),
                            _ => return Err(arity("=", 2, args.len())),
                        },
                        // Structural equality on *syntax* (SPEC-0016 §2.4).
                        "eq?" => match args {
                            [a, b] => vals.push(eval_syntax(a, env)? == eval_syntax(b, env)?),
                            _ => return Err(arity("eq?", 2, args.len())),
                        },
                        "and" => {
                            let mut it = args.iter();
                            match it.next() {
                                Some(first) => {
                                    // First entry launches operand 0 *without*
                                    // inspecting any value — there is no "last
                                    // boolean" yet.
                                    frames.push(Frame::And(it));
                                    frames.push(Frame::Eval(first));
                                }
                                None => vals.push(true), // `(and)` is vacuously true
                            }
                        }
                        "or" => {
                            let mut it = args.iter();
                            match it.next() {
                                Some(first) => {
                                    frames.push(Frame::Or(it));
                                    frames.push(Frame::Eval(first));
                                }
                                None => vals.push(false), // `(or)` is vacuously false
                            }
                        }
                        other => {
                            return Err(PredError::ExpectedBool {
                                found: format!("form `{other}`"),
                            })
                        }
                    }
                }
                _ => {
                    return Err(PredError::ExpectedBool {
                        found: describe(node),
                    })
                }
            },
        }
    }

    let Some(result) = vals.pop() else {
        unreachable!("eval_pred produced no value: the root Eval always pushes exactly one")
    };
    Ok(result)
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
