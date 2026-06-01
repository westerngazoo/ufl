//! UFL — numeric and (later) geometric-algebra core.
//!
//! This crate provides the EML expression tree and a reference evaluator,
//! realising [R-0001](../../../requirements/0001-eml-operator-core.md) per
//! [SPEC-0001](../../../specs/0001-eml-operator-core.md).
//!
//! See the spec for the design; this lib's public surface is `Eml` plus
//! `eval` over an `Env` returning a complex `Value`.

#![forbid(unsafe_code)]

pub mod eml;
pub mod eval;
pub mod ga;
mod log;

pub use eml::Eml;
pub use eval::{eval, Env, EvalError, Value};
pub use ga::{GradeLift, Multivector};
