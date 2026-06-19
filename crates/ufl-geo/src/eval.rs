//! Lowering `GeoExpr` onto the `ufl-ga` kernel (SPEC-0010 §2.2).

use ufl_ga::Mv;

use crate::expr::{Env, GeoExpr};

/// A failure evaluating a `GeoExpr`. Total — the only failures are an unbound
/// variable or an out-of-range blade/grade leaf; `eval` never panics.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GeoError {
    /// A `Var` with no binding in the environment.
    #[error("unbound variable `{0}`")]
    Unbound(String),
    /// A `Basis(i)` with `i ≥ 16` (out of the 16-blade algebra).
    #[error("blade index {0} out of range (must be < 16)")]
    BadBlade(u8),
    /// A `GradeLift`/`GradeProject` grade `> 4`.
    #[error("grade {0} out of range (must be ≤ 4)")]
    BadGrade(u8),
}

/// Evaluate a `GeoExpr` to a multivector by lowering each form onto its
/// `ufl_ga::Mv` operation. **R-0010 step 5 — the TDD-red target.**
pub fn eval(_e: &GeoExpr, _env: &Env) -> Result<Mv, GeoError> {
    unimplemented!("R-0010 step 5 — see SPEC-0010 §2.2")
}
