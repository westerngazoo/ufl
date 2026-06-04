//! UFL — the predicate layer (atom `⟦P⟧`), as a **checker**.
//!
//! Realizes [R-0004](../../../requirements/0004-predicate-layer.md) per
//! [SPEC-0004](../../../specs/0004-predicate-layer.md). Given a concrete
//! pre-state and post-state, does a predicate hold?
//!
//! The value model is **two typed evaluators, no god-enum** (SPEC-0004 §2.1):
//! numeric expressions are lowered (`ufl_syntax::lower`) and evaluated by the
//! *reused* `ufl_core::eval` → [`Value`]; boolean expressions are evaluated by
//! [`eval_pred`] → `bool`; `(= a b)` bridges. Booleans never enter the numeric
//! `Value`, and the verified numerics are inherited, never re-implemented.

#![forbid(unsafe_code)]

mod eval_pred;

pub use eval_pred::{eval_pred, PredError};

use ufl_core::{Env, Value};
use ufl_syntax::{read, ReadError, Sexpr};

/// A failure while checking a predicate from text (SPEC-0004 §2.6).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CheckError {
    #[error("reserved variable name `{0}`: state-variable names may not contain '\\''")]
    ReservedName(String),
    #[error(transparent)]
    Read(#[from] ReadError),
    #[error(transparent)]
    Pred(#[from] PredError),
}

/// Build the combined env: pre vars by name, post vars under `name'`.
/// A binding name containing `'` is reserved (SPEC-0004 §2.5).
fn combined_env(pre: &[(&str, Value)], post: &[(&str, Value)]) -> Result<Env, CheckError> {
    let mut env = Env::new();
    for (name, value) in pre {
        if name.contains('\'') {
            return Err(CheckError::ReservedName((*name).to_string()));
        }
        env.bind(*name, *value);
    }
    for (name, value) in post {
        if name.contains('\'') {
            return Err(CheckError::ReservedName((*name).to_string()));
        }
        env.bind(format!("{name}'"), *value);
    }
    Ok(env)
}

/// Check a predicate `Sexpr` against a pre-state and a post-state.
pub fn check(
    predicate: &Sexpr,
    pre: &[(&str, Value)],
    post: &[(&str, Value)],
) -> Result<bool, CheckError> {
    let env = combined_env(pre, post)?;
    Ok(eval_pred(predicate, &env)?)
}

/// Read a predicate from text, then check it against a pre/post state.
pub fn check_str(
    src: &str,
    pre: &[(&str, Value)],
    post: &[(&str, Value)],
) -> Result<bool, CheckError> {
    let predicate = read(src)?;
    let env = combined_env(pre, post)?;
    Ok(eval_pred(&predicate, &env)?)
}
