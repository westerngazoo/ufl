//! UFL — the typed geometric layer (Pillar 2: the geometric *forms* + the
//! grade-type system).
//!
//! Realizes [R-0010](../../../requirements/0010-geometric-forms-grade-types.md)
//! per [SPEC-0010](../../../specs/0010-geometric-forms-grade-types.md): the
//! [`GeoExpr`] AST (the genotype R-0011 evolves), [`eval`] onto the `ufl-ga`
//! `Cl(3,0,1)` kernel, and a **decidable grade-type system** ([`grade`] /
//! [`typecheck`]).
//!
//! The grade algebra is **not hand-rolled** — it delegates to garust's
//! `Op::output_grades` (re-exported via `ufl-ga`) for the catalog forms, riding
//! garust's [`GradeSet`]; only `Sandwich`/`Exp`/`GradeLift` get hand rules. A
//! facade over the kernel's type machinery, the same way `ufl-ga` is a facade
//! over the kernel.

#![forbid(unsafe_code)]

mod eval;
mod expr;
mod grade;
mod lane;
mod render;
mod slots;

pub use eval::{eval, GeoError};
pub use expr::{Env, GeoExpr};
pub use grade::{grade, typecheck, GradeCtx, GradeError};
pub use lane::{GeoLaneError, GeoParamRefiner, GradeScreen};
pub use render::render;
pub use slots::{params, params_mut};
pub use ufl_ga::{GradeSet, Mv};
