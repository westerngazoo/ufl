//! Hello, geometric forms — the typed geometric layer, with the grade proof.
//!
//! The R-0009 `hello_ga` example sandwiches `e1` to `e2` at the *kernel* level.
//! This shows the **same** sandwich as a [`GeoExpr`] form — built from
//! constructors (the homoiconic AST R-0011 evolves), then read two ways:
//! **evaluated** (`e1 → e2`) and **grade-typed** (vector → vector, `{1}`). The
//! eval says "specifically `e₁ → e₂`"; the type proves "a rotated vector is
//! still a vector" — *without evaluating*, the decidable signal R-0011 prunes on.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-geo --example hello_geo
//! ```

use std::f64::consts::TAU; // τ — UFL's circle constant
use ufl_geo::{eval, grade, typecheck, Env, GeoExpr, GradeCtx, GradeSet};

fn main() {
    println!("UFL — the typed geometric forms (GeoExpr) over the Cl(3,0,1) kernel.\n");

    // The keystone form: Sandwich(R, e1) with R = exp(−τ/8 · e₁₂), a versor.
    // e₁₂ is blade index 3; the Param carries the half-angle the rotor needs.
    let rotor = GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
        Box::new(GeoExpr::Param(-TAU / 8.0)),
        Box::new(GeoExpr::Basis(3)), // e₁₂, grade 2
    )));
    let form = GeoExpr::Sandwich(Box::new(rotor), Box::new(GeoExpr::Basis(1))); // onto e1

    // ── Eval: lower the form onto the kernel and compute (e1 → e2) ──────────
    let value = eval(&form, &Env::new()).expect("the keystone form evaluates");
    println!(
        "  eval(Sandwich(Exp(−τ/8·e₁₂), e1))  →  {}",
        value.grade(1).cleaned(1e-10) // the vector part: e2
    );

    // ── Grade-type: infer the result grade WITHOUT evaluating (vector → vector) ─
    let g = grade(&form, &GradeCtx::new());
    println!("  grade(...)                         →  {g:?}"); // {1}
    assert_eq!(
        g,
        GradeSet::singleton(1),
        "a rotated vector is still a vector"
    );

    // ── Typecheck: the form is grade-coherent; its type is its grade ────────
    match typecheck(&form, &GradeCtx::new()) {
        Ok(t) => println!("  typecheck(...)                     →  Ok({t:?})"),
        Err(e) => println!("  typecheck(...)                     →  Err({e})"),
    }

    println!(
        "\nOne homoiconic form, two readings: the eval lands on e₂; the grade type\n\
         proves vector → vector ({{1}}) WITHOUT evaluating. This GeoExpr AST is the\n\
         genotype R-0011 mutates — and grade is the type that scores it."
    );
}
