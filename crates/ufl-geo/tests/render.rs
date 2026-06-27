//! #27 — the `GeoExpr` → GA-notation printer (the "translate-back" leg).

use std::f64::consts::TAU;

use ufl_geo::{render, GeoExpr};

/// The keystone form renders as a `let`-bound rotor sandwich `R v ~R`.
#[test]
fn keystone_renders_as_let_bound_sandwich() {
    // Sandwich(Exp(GeoProduct(Param(−τ/8), e₁₂)), v) — the discovered rotation.
    let form = GeoExpr::Sandwich(
        Box::new(GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
            Box::new(GeoExpr::Param(-TAU / 8.0)),
            Box::new(GeoExpr::Basis(3)),
        )))),
        Box::new(GeoExpr::Var("v".into())),
    );
    let s = render(&form);
    assert!(
        s.contains("let R = exp("),
        "expected a let-bound rotor, got:\n{s}"
    );
    assert!(s.contains("e₁₂"), "expected the e₁₂ plane, got:\n{s}");
    assert!(
        s.contains("R v ~R"),
        "expected the sandwich body `R v ~R`, got:\n{s}"
    );
}

/// Blade names are correct across grades (garust convention: e₀ is the null
/// generator; subscripts in ascending generator order).
#[test]
fn blade_names_across_grades() {
    let cases = [
        (0u8, "1"),
        (1, "e₁"),
        (2, "e₂"),
        (4, "e₃"),
        (8, "e₀"),
        (3, "e₁₂"),
        (9, "e₀₁"),
        (7, "e₁₂₃"),
        (15, "e₀₁₂₃"),
    ];
    for (i, want) in cases {
        assert_eq!(render(&GeoExpr::Basis(i)), want, "blade {i}");
    }
}

/// A deeply-nested versor sandwich renders in BOUNDED length (the let-binding
/// fix) — a naive `R v ~R` expansion would be exponential.
#[test]
fn nested_sandwich_stays_bounded() {
    let mut e = GeoExpr::Var("v".into());
    for _ in 0..12 {
        // Each level nests the previous expression in the VERSOR position — the
        // exact shape that blows up without `let`-binding the rotor.
        e = GeoExpr::Sandwich(Box::new(e), Box::new(GeoExpr::Var("v".into())));
    }
    let s = render(&e);
    assert!(
        s.len() < 4000,
        "12-deep nested sandwich must stay bounded (no exponential blow-up), got {} chars",
        s.len(),
    );
}

/// Product chains are parenthesised so they are unambiguous: a wedge inside a
/// geometric product is wrapped.
#[test]
fn product_chains_are_parenthesised() {
    let form = GeoExpr::GeoProduct(
        Box::new(GeoExpr::Wedge(
            Box::new(GeoExpr::Basis(1)),
            Box::new(GeoExpr::Basis(2)),
        )),
        Box::new(GeoExpr::Basis(3)),
    );
    let s = render(&form);
    assert!(
        s.contains("(e₁∧e₂)"),
        "wedge inside a product must be parenthesised, got: {s}"
    );
}
