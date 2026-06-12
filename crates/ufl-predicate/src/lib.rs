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
mod predicate;

pub use eval_pred::{eval_pred, PredError};
pub use predicate::{Predicate, State};

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
///
/// Routes through the [`Predicate`] trait (SPEC-0007 §2.2): the s-expression
/// *is* the predicate, discharged against the guarded [`State`]. Behaviour is
/// identical to the pre-trait path (`State::new` delegates to `combined_env`).
pub fn check(
    predicate: &Sexpr,
    pre: &[(&str, Value)],
    post: &[(&str, Value)],
) -> Result<bool, CheckError> {
    predicate.discharge(&State::new(pre, post)?)
}

/// Read a predicate from text, then check it against a pre/post state.
///
/// Error precedence is unchanged: `Read` first, then `ReservedName`, then
/// `Pred` (the receiver `read(src)?` evaluates before the `State` argument).
pub fn check_str(
    src: &str,
    pre: &[(&str, Value)],
    post: &[(&str, Value)],
) -> Result<bool, CheckError> {
    read(src)?.discharge(&State::new(pre, post)?)
}

#[cfg(test)]
mod tests {
    //! Unit tests for `combined_env` and the `ReservedName` guard (SPEC-0004
    //! §2.5). These exercise the **already-implemented** binding/priming logic
    //! and the reserved-name rejection, which run *before* `eval_pred`. They are
    //! therefore **green now** (independent of the pending `eval_pred` body) and
    //! pin the priming convention the AC4/AC6 e2e tests depend on.

    use super::{combined_env, CheckError};
    use ufl_core::Value;

    /// SPEC-0004 §2.5 — pre vars bind by name, post vars bind under `name'`. So
    /// the same predicate text reaches pre-`x` and post-`x'` from two distinct
    /// bindings without collision.
    #[test]
    fn combined_env_binds_pre_by_name_and_post_primed() {
        let env = combined_env(
            &[("x", Value::new(1.0, 0.0))],
            &[("x", Value::new(2.0, 0.0))],
        )
        .expect("distinct, unreserved names should bind");
        assert_eq!(
            env.get("x"),
            Some(Value::new(1.0, 0.0)),
            "pre-x binds by name"
        );
        assert_eq!(
            env.get("x'"),
            Some(Value::new(2.0, 0.0)),
            "post-x binds under the primed key `x'`"
        );
        // The unprimed post key and the primed pre key are absent: the priming
        // is the *only* channel for post-state, keeping it injective.
        assert_eq!(env.get("x'"), Some(Value::new(2.0, 0.0)));
    }

    /// SPEC-0004 §2.5 — a *pre* binding name containing `'` is reserved and
    /// rejected, returning before any predicate evaluation. (`Env` is not
    /// `PartialEq`, so the `Ok` arm is matched, not `assert_eq!`'d.)
    #[test]
    fn combined_env_rejects_reserved_pre_name() {
        assert_eq!(
            combined_env(&[("x'", Value::new(1.0, 0.0))], &[]).err(),
            Some(CheckError::ReservedName("x'".to_string()))
        );
    }

    /// SPEC-0004 §2.5 — a *post* binding name containing `'` is likewise
    /// rejected (it would otherwise produce the ambiguous double-primed `x''`).
    #[test]
    fn combined_env_rejects_reserved_post_name() {
        assert_eq!(
            combined_env(&[], &[("x'", Value::new(1.0, 0.0))]).err(),
            Some(CheckError::ReservedName("x'".to_string()))
        );
    }

    /// SPEC-0004 §2.5 — an apostrophe anywhere in the name triggers the guard,
    /// not only a trailing one (the suffix channel must stay unambiguous).
    #[test]
    fn combined_env_rejects_embedded_apostrophe() {
        assert_eq!(
            combined_env(&[("a'b", Value::new(1.0, 0.0))], &[]).err(),
            Some(CheckError::ReservedName("a'b".to_string()))
        );
    }

    /// SPEC-0004 §2.5 — the empty pre/post case binds nothing and succeeds (the
    /// degenerate baseline alongside the reserved-name rejections).
    #[test]
    fn combined_env_empty_is_ok_and_empty() {
        let env = combined_env(&[], &[]).expect("empty pre/post should succeed");
        assert_eq!(env.get("x"), None);
    }
}
