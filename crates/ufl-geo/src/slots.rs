//! Typed param-slots — the continuous-parameter view of a [`GeoExpr`]
//! (SPEC-0011M §2.2).
//!
//! [`params_mut`] enumerates mutable borrows of every `Param` leaf in a fixed
//! pre-order; [`params`] is the read-only snapshot in the **same** order (it is
//! implemented *through the same traversal*, so the two cannot drift). The slots
//! are grade-`{0}` **by construction** — `grade(Param(_)) = {0}` and no
//! `grade`/`typecheck`/`is_versor` rule reads a `Param`'s value — so writing any
//! `f64` through a slot preserves `typecheck(..).is_ok()` and the inferred
//! `Ok(GradeSet)` (the *scoped* invariant; the `Err` payload may embed the value
//! and is deliberately out of scope — SPEC-0011M §2.2 caveat).
//!
//! This is the first concrete **typed quotation site**: an enumerated set of
//! typed holes a search can write through without disturbing the surrounding
//! structure's type — the shape R-0015's operator DSL reuses.

use crate::expr::GeoExpr;

/// Mutable borrows of every `Param` leaf of `e`, in pre-order.
pub fn params_mut(e: &mut GeoExpr) -> Vec<&mut f64> {
    let mut out = Vec::new();
    collect(e, &mut out);
    out
}

/// The current `Param` values of `e`, in the same pre-order as [`params_mut`]
/// (snapshot/restore for a hill-climb step). Implemented via the same traversal
/// on a clone, so the orders provably coincide.
pub fn params(e: &GeoExpr) -> Vec<f64> {
    let mut clone = e.clone();
    params_mut(&mut clone).into_iter().map(|s| *s).collect()
}

/// The single traversal both views share: a lifetime-threaded pre-order walk.
/// Disjoint leaf borrows do not alias, so simultaneous `&mut` are sound.
fn collect<'a>(e: &'a mut GeoExpr, out: &mut Vec<&'a mut f64>) {
    match e {
        GeoExpr::Param(s) => out.push(s),
        GeoExpr::Basis(_) | GeoExpr::Var(_) => {}
        GeoExpr::GradeLift(_, a)
        | GeoExpr::Reverse(a)
        | GeoExpr::GradeProject(_, a)
        | GeoExpr::Exp(a) => collect(a, out),
        GeoExpr::GeoProduct(a, b)
        | GeoExpr::Wedge(a, b)
        | GeoExpr::Inner(a, b)
        | GeoExpr::Sandwich(a, b) => {
            collect(a, out);
            collect(b, out);
        }
    }
}
