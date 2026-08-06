//! UFL — homoiconic S-expression surface and intermediate representation.
//!
//! Realizes [R-0003](../../../requirements/0003-sexpr-core.md) per
//! [SPEC-0003](../../../specs/0003-sexpr-core.md). The pipeline is
//!
//! ```text
//! text ──read──▶ Sexpr ──lower──▶ Eml ──eval──▶ Value
//! ```
//!
//! [`Sexpr`] is the one homoiconic syntax tree (code is data). [`read`] parses
//! text into it; [`lower`] translates a well-formed `eml` form into
//! `ufl_core`'s typed [`Eml`](ufl_core::Eml); [`eval_str`] runs the whole path
//! by reusing `ufl_core::eval` verbatim — so the verified numerics (the branch
//! convention, the `sin(τ/2)` self-correction) are inherited, never
//! re-implemented. [`raise`] is the inverse of [`lower`], closing the code↔data
//! square on the reader's canonical image (SPEC-0016 §2.5).

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod lower;
mod read;
mod sexpr;

pub use lower::{lower, raise, LowerError};
pub use read::{is_reader_canonical_sym, read, ReadError};
pub use sexpr::Sexpr;

use ufl_core::{Env, EvalError, Value};

/// Any failure in the `text → Sexpr → Eml → Value` pipeline, surfaced at the
/// earliest layer that can detect it (SPEC-0003 §2.5 / AC6).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UflError {
    #[error(transparent)]
    Read(#[from] ReadError),
    #[error(transparent)]
    Lower(#[from] LowerError),
    #[error(transparent)]
    Eval(#[from] EvalError),
}

/// Read, lower, and evaluate a UFL s-expression. The `env` supplies any free
/// variables (e.g. `x` in `(eml x 1)`), exactly as `ufl_core::eval` requires.
pub fn eval_str(src: &str, env: &Env) -> Result<Value, UflError> {
    let sexpr = read(src)?;
    let eml = lower(&sexpr)?;
    Ok(ufl_core::eval(&eml, env)?)
}
