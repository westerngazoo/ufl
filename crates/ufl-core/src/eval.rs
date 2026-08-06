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
/// **Iterative** post-order walk over an explicit heap work-stack (R-0017): a
/// tree of any depth the machine can allocate evaluates without a stack
/// overflow. Semantics are unchanged from SPEC-0001 §2.5:
///
/// - `One` → the complex value `1 + 0i`.
/// - `Var(name)` → the binding from `env`, or `Err(UnboundVariable)`.
/// - `Node { exp_arg, log_arg }` → evaluate both children, then return
///   `exp(x) − ln_eml(y)`.
///
/// Infallible on numeric edge cases (`inf` / `nan` propagate as ordinary
/// `Value`s); the only failure mode is an unbound variable.
pub fn eval(expr: &Eml, env: &Env) -> Result<Value, EvalError> {
    // The two-stack post-order machine. `Task::Combine` marks the point where a
    // node's two child values are ready to fold.
    enum Task<'a> {
        Eval(&'a Eml),
        Combine,
    }

    let mut tasks = vec![Task::Eval(expr)];
    let mut vals: Vec<Value> = Vec::new();

    while let Some(task) = tasks.pop() {
        match task {
            Task::Eval(e) => match e {
                Eml::One => vals.push(Value::new(1.0, 0.0)),
                Eml::Var(name) => {
                    let v = env
                        .get(name)
                        .ok_or_else(|| EvalError::UnboundVariable(name.clone()))?;
                    vals.push(v);
                }
                Eml::Node { exp_arg, log_arg } => {
                    // LIFO: push Combine, then log, then exp — so `exp` is
                    // evaluated first and `log` second, preserving SPEC-0001
                    // §2.4's convention *and* the order in which an unbound
                    // variable surfaces.
                    tasks.push(Task::Combine);
                    tasks.push(Task::Eval(log_arg));
                    tasks.push(Task::Eval(exp_arg));
                }
            },
            Task::Combine => {
                // Each `Eml` subtree nets exactly one value (leaf: +1; Node:
                // two Evals then a Combine that pops 2 and pushes 1), and LIFO
                // guarantees both children completed before their Combine — so
                // two values are always present here.
                let (Some(log_val), Some(exp_val)) = (vals.pop(), vals.pop()) else {
                    unreachable!("eval value-stack underflow: each Combine follows two Eval pushes")
                };
                vals.push(exp_val.exp() - crate::log::ln_eml(log_val));
            }
        }
    }

    let Some(result) = vals.pop() else {
        unreachable!("eval produced no value: the root task always pushes exactly one")
    };
    Ok(result)
}
