//! The decidable grade-type system (SPEC-0010 §2.3–§2.5).
//!
//! `grade` is a **sound over-approximation** of the result grades — every grade
//! a value can carry is in the set; under the degenerate metric the set may be a
//! strict superset of the realized support. The catalog forms delegate to
//! garust's `Op::output_grades`; only `Sandwich`/`Exp`/`GradeLift` are hand-ruled.

use std::collections::HashMap;
use ufl_ga::{GradeSet, Op};

use crate::expr::GeoExpr;

/// The number of `Cl(3,0,1)` generators (grades range over `0..=4`).
const N: usize = 4;

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

/// A conservative, sound static versor predicate (SPEC-0010 §2.4) — `true` only
/// when `r` is *provably* a versor, so `Sandwich(r, ·)` preserves grade. It may
/// say `false` for a real versor (the grade rule then falls back to the safe
/// product bound — still sound); it never says `true` for a non-versor.
///
/// Versor witnesses: `Exp(b)` of a pure bivector (`grade(b) ⊆ {2}` — a rotor or
/// motor), a single basis *vector* (`Basis(i)`, one set bit), a `GeoProduct` of
/// versors, and `Reverse` of a versor.
pub(crate) fn is_versor(r: &GeoExpr, ctx: &GradeCtx) -> bool {
    match r {
        GeoExpr::Exp(b) => subset_of(grade(b, ctx), &[2]),
        GeoExpr::GeoProduct(a, b) => is_versor(a, ctx) && is_versor(b, ctx),
        GeoExpr::Basis(i) => *i < 16 && i.count_ones() == 1,
        GeoExpr::Reverse(a) => is_versor(a, ctx),
        _ => false,
    }
}

/// Is every grade in `g` one of `allowed`? (`g ⊆ allowed`.)
fn subset_of(g: GradeSet, allowed: &[usize]) -> bool {
    g.iter().all(|k| allowed.contains(&k))
}

/// Infer a **sound over-approximation** of a form's result grades (SPEC-0010
/// §2.3). The catalog forms delegate to garust's `Op::output_grades` (the
/// correct structural signature); only `Sandwich`/`Exp`/`GradeLift` are
/// hand-ruled. Total and decidable — out-of-range leaves return `⊤ = full(4)`
/// (it is `typecheck` that turns those into errors).
pub fn grade(e: &GeoExpr, ctx: &GradeCtx) -> GradeSet {
    match e {
        GeoExpr::Param(_) => GradeSet::singleton(0),
        GeoExpr::Basis(i) => {
            if *i >= 16 {
                GradeSet::full(N)
            } else {
                GradeSet::singleton(i.count_ones() as usize)
            }
        }
        GeoExpr::Var(name) => ctx.get(name),
        GeoExpr::GradeLift(k, _) => {
            if *k > 4 {
                GradeSet::full(N)
            } else {
                GradeSet::singleton(*k as usize)
            }
        }
        GeoExpr::GeoProduct(a, b) => {
            Op::Geometric.output_grades(&[grade(a, ctx), grade(b, ctx)], N)
        }
        GeoExpr::Wedge(a, b) => Op::Wedge.output_grades(&[grade(a, ctx), grade(b, ctx)], N),
        GeoExpr::Inner(a, b) => Op::Inner.output_grades(&[grade(a, ctx), grade(b, ctx)], N),
        GeoExpr::Reverse(a) => Op::Reverse.output_grades(&[grade(a, ctx)], N),
        GeoExpr::GradeProject(k, a) => {
            // Guard the raw `u8` before garust (its `singleton(k) = 1 << k`
            // overflows `u32` for `k ≥ 32`): projecting onto a grade the algebra
            // lacks (`k > 4`) is the empty set. Keeps `grade` total (SPEC-0010 §2.3).
            if *k > 4 {
                GradeSet::EMPTY
            } else {
                Op::GradeProject(*k).output_grades(&[grade(a, ctx)], N)
            }
        }
        GeoExpr::Sandwich(r, x) => {
            if is_versor(r, ctx) {
                // A versor sandwich preserves the operand's grade.
                grade(x, ctx)
            } else {
                // The sound product bound: grades of `(r ∗ x) ∗ r` (reverse
                // preserves grade, so `~r` carries the same grades as `r`).
                let rg = grade(r, ctx);
                let rx = Op::Geometric.output_grades(&[rg, grade(x, ctx)], N);
                Op::Geometric.output_grades(&[rx, rg], N)
            }
        }
        GeoExpr::Exp(a) => {
            let g = grade(a, ctx);
            if subset_of(g, &[0]) {
                GradeSet::singleton(0)
            } else if subset_of(g, &[0, 2]) {
                // exp of an even element is even — covers rotors and motors.
                GradeSet::EMPTY.with(0).with(2).with(4)
            } else {
                GradeSet::full(N)
            }
        }
    }
}

/// Infer the grade set, or fail on a grade-incoherent / out-of-range form
/// (SPEC-0010 §2.5). Recursively checks every sub-form: an out-of-range
/// `Basis`/grade leaf is a `BadBlade`/`BadGrade`, and a sub-form whose grade set
/// is `∅` (e.g. `GradeProject(k, a)` with `k ∉ grade(a)`) is `Incoherent` — it
/// can only ever be zero. The decidable pruning signal R-0011 reads.
pub fn typecheck(e: &GeoExpr, ctx: &GradeCtx) -> Result<GradeSet, GradeError> {
    match e {
        GeoExpr::Param(_) | GeoExpr::Var(_) => {}
        GeoExpr::Basis(i) => {
            if *i >= 16 {
                return Err(GradeError::BadBlade(*i));
            }
        }
        GeoExpr::GradeLift(k, a) | GeoExpr::GradeProject(k, a) => {
            if *k > 4 {
                return Err(GradeError::BadGrade(*k));
            }
            typecheck(a, ctx)?;
        }
        GeoExpr::Reverse(a) | GeoExpr::Exp(a) => {
            typecheck(a, ctx)?;
        }
        GeoExpr::GeoProduct(a, b)
        | GeoExpr::Wedge(a, b)
        | GeoExpr::Inner(a, b)
        | GeoExpr::Sandwich(a, b) => {
            typecheck(a, ctx)?;
            typecheck(b, ctx)?;
        }
    }
    let g = grade(e, ctx);
    if g.is_empty() {
        Err(GradeError::Incoherent(e.clone()))
    } else {
        Ok(g)
    }
}
