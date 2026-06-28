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
/// Recursive post-order walk per SPEC-0001 §2.5:
///
/// - `One` → the complex value `1 + 0i`.
/// - `Var(name)` → the binding from `env`, or `Err(UnboundVariable)`.
/// - `Node { exp_arg, log_arg }` → `eval` both children, then return
///   `exp(x) − ln_eml(y)`.
///
/// Infallible on numeric edge cases (`inf` / `nan` propagate as ordinary
/// `Value`s); the only failure mode is an unbound variable.
pub fn eval(expr: &Eml, env: &Env) -> Result<Value, EvalError> {
    match expr {
        Eml::One => Ok(Value::new(1.0, 0.0)),
        Eml::Var(name) => env
            .get(name)
            .ok_or_else(|| EvalError::UnboundVariable(name.clone())),
        Eml::Node { exp_arg, log_arg } => {
            let exp_val = eval(exp_arg, env)?;
            let log_val = eval(log_arg, env)?;
            Ok(exp_val.exp() - crate::log::ln_eml(log_val))
        }
    }
}
