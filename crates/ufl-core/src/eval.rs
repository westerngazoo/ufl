//! Reference evaluator for EML expressions.
//!
//! See SPEC-0001 §2.5 and §3.

use std::collections::HashMap;

use num_complex::Complex;

use crate::eml::Eml;

/// The value of a UFL EML expression — complex over IEEE-754 `f64`.
///
/// See SPEC-0001 §2.3.
pub type Value = Complex<f64>;

/// Variable bindings consumed by [`eval`].
#[derive(Debug, Clone, Default)]
pub struct Env {
    bindings: HashMap<String, Value>,
}

impl Env {
    /// An empty environment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a variable name to a value.
    pub fn bind(&mut self, name: impl Into<String>, value: Value) -> &mut Self {
        self.bindings.insert(name.into(), value);
        self
    }

    /// Look up a variable's value.
    pub fn get(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).copied()
    }
}

/// Errors returned by [`eval`].
///
/// Evaluation is infallible on numeric edge cases (per SPEC-0001 §2.5 / AC3) —
/// `inf` / `nan` propagate as ordinary `Value`s. The only genuine failure is a
/// variable referenced in the expression but not present in the [`Env`].
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EvalError {
    #[error("unbound variable: {0}")]
    UnboundVariable(String),
}

/// Evaluate an EML expression under the given environment.
///
/// Recursive post-order walk per SPEC-0001 §2.5. R-0001 implementation is
/// pending — this function panics until step 5 of the requirement loop lands.
pub fn eval(_expr: &Eml, _env: &Env) -> Result<Value, EvalError> {
    todo!("R-0001 implementation — see SPEC-0001 §2.5")
}
