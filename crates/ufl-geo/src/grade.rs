//! The decidable grade-type system (SPEC-0010 §2.3–§2.5).
//!
//! `grade` is a **sound over-approximation** of the result grades — every grade
//! a value can carry is in the set; under the degenerate metric the set may be a
//! strict superset of the realized support. The catalog forms delegate to
//! garust's `Op::output_grades`; only `Sandwich`/`Exp`/`GradeLift` are hand-ruled.

use std::collections::HashMap;
use ufl_ga::GradeSet;

use crate::expr::GeoExpr;

/// The number of `Cl(3,0,1)` generators.
#[allow(dead_code)] // consumed by `grade` in R-0010 step 5
pub(crate) const N: usize = 4;

/// Grade context: input variables declared with their grade set
/// (⊤ = `full(4)` if undeclared).
#[derive(Clone, Debug, Default)]
pub struct GradeCtx {
    vars: HashMap<String, GradeSet>,
}

impl GradeCtx {
    /// An empty context (every `Var` is ⊤ until declared).
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a variable's grade set.
    pub fn declare(&mut self, name: impl Into<String>, grades: GradeSet) {
        self.vars.insert(name.into(), grades);
    }

    /// The declared grade of a variable, or ⊤ (`full(4)`) if undeclared.
    #[allow(dead_code)] // consumed by `grade` in R-0010 step 5
    pub(crate) fn get(&self, name: &str) -> GradeSet {
        self.vars
            .get(name)
            .copied()
            .unwrap_or_else(|| GradeSet::full(N))
    }
}

/// A grade-type failure (the decidable pruning signal R-0011 uses).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GradeError {
    /// A grade-incoherent form — its grade set is `∅` (e.g. `GradeProject(k, a)`
    /// with `k ∉ grade(a)`); it can only ever be zero.
    #[error("grade-incoherent form (empty grade set)")]
    Incoherent(GeoExpr),
    /// A `Basis(i)` with `i ≥ 16`.
    #[error("blade index {0} out of range")]
    BadBlade(u8),
    /// A `GradeLift`/`GradeProject` grade `> 4`.
    #[error("grade {0} out of range")]
    BadGrade(u8),
}

/// A conservative, sound static versor predicate — `true` only when `r` is
/// provably a versor (then `Sandwich(r, ·)` preserves grade). May say `false`
/// for some real versors (the grade rule then falls back to the safe bound).
/// **R-0010 step 5.**
#[allow(dead_code)] // consumed by `grade` in R-0010 step 5
pub(crate) fn is_versor(_r: &GeoExpr, _ctx: &GradeCtx) -> bool {
    unimplemented!("R-0010 step 5 — see SPEC-0010 §2.4")
}

/// Infer a sound over-approximation of a form's result grades.
/// **R-0010 step 5 — the TDD-red target.**
pub fn grade(_e: &GeoExpr, _ctx: &GradeCtx) -> GradeSet {
    unimplemented!("R-0010 step 5 — see SPEC-0010 §2.3")
}

/// Infer the grade set, or fail on a grade-incoherent / out-of-range form.
/// **R-0010 step 5 — the TDD-red target.**
pub fn typecheck(_e: &GeoExpr, _ctx: &GradeCtx) -> Result<GradeSet, GradeError> {
    unimplemented!("R-0010 step 5 — see SPEC-0010 §2.5")
}
