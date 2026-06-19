//! Lowering `GeoExpr` onto the `ufl-ga` kernel (SPEC-0010 §2.2).

use ufl_ga::Mv;

use crate::expr::{lowest_blade, Env, GeoExpr};

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
/// `ufl_ga::Mv` operation (SPEC-0010 §2.2). **Total** — every garust call that
/// could panic (`Mv::basis(i)` for `i ≥ 16`) is guarded by a typed `GeoError`
/// first, so a malformed leaf returns `Err`, never a panic.
pub fn eval(e: &GeoExpr, env: &Env) -> Result<Mv, GeoError> {
    match e {
        GeoExpr::Param(s) => Ok(Mv::scalar(*s)),
        GeoExpr::Basis(i) => {
            if *i >= 16 {
                Err(GeoError::BadBlade(*i))
            } else {
                Ok(Mv::basis(*i as usize))
            }
        }
        GeoExpr::Var(name) => env.get(name).ok_or_else(|| GeoError::Unbound(name.clone())),
        GeoExpr::GradeLift(k, a) => {
            let blade = lowest_blade(*k).ok_or(GeoError::BadGrade(*k))?;
            Ok(eval(a, env)? * Mv::basis(blade as usize))
        }
        GeoExpr::GeoProduct(a, b) => Ok(eval(a, env)? * eval(b, env)?),
        GeoExpr::Wedge(a, b) => Ok(eval(a, env)?.wedge(&eval(b, env)?)),
        GeoExpr::Inner(a, b) => Ok(eval(a, env)?.inner(&eval(b, env)?)),
        GeoExpr::Reverse(a) => Ok(eval(a, env)?.reverse()),
        GeoExpr::GradeProject(k, a) => {
            if *k > 4 {
                Err(GeoError::BadGrade(*k))
            } else {
                Ok(eval(a, env)?.grade(*k as usize))
            }
        }
        GeoExpr::Sandwich(r, x) => Ok(eval(r, env)?.sandwich(&eval(x, env)?)),
        GeoExpr::Exp(a) => Ok(eval(a, env)?.exp()),
    }
}
