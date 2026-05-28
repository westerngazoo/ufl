//! Hello, EML — a tiny demo of what UFL looks like today.
//!
//! Builds the EML trees for `e`, `exp(x)`, and `ln(x)`, evaluates them under
//! the reference evaluator, and prints the results. After R-0001 the language
//! is exactly this: one atom (`eml`), one literal (`1`), variables, and a
//! complex-valued reference evaluator. No surface syntax yet — trees are
//! constructed directly via the `Eml` API.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-core --example hello_eml
//! ```

use ufl_core::{eval, Eml, Env, Value};

fn show(label: &str, v: Value) {
    // Trim near-zero imaginary parts for readability; otherwise show full ℂ.
    if v.im.abs() < 1e-12 {
        println!("  {label:<32} = {:>12.6}", v.re);
    } else {
        println!("  {label:<32} = {:>12.6} + {:>12.6} i", v.re, v.im);
    }
}

fn main() {
    println!("UFL today — one atom (`eml`), one literal (`1`), variables.\n");

    // ── e = eml(1, 1) ────────────────────────────────────────────────────
    println!("e — Euler's number, built from two `1`s and one operator:");
    let e_tree = Eml::node(Eml::one(), Eml::one());
    show("eml(1, 1)", eval(&e_tree, &Env::new()).expect("e"));

    // ── exp(x) = eml(x, 1) ───────────────────────────────────────────────
    println!("\nexp(x) — a one-node tree:");
    let exp_x = Eml::node(Eml::var("x"), Eml::one());
    for x_real in [0.0_f64, 1.0, 2.5] {
        let mut env = Env::new();
        env.bind("x", Value::new(x_real, 0.0));
        show(
            &format!("eml(x, 1)  with x = {x_real:>4}"),
            eval(&exp_x, &env).expect("exp"),
        );
    }

    // ── ln(x) = eml(1, eml(eml(1, x), 1)) ────────────────────────────────
    println!("\nln(x) — a three-node tree (works on negative reals via the f64 self-correction):");
    let ln_x = Eml::node(
        Eml::one(),
        Eml::node(Eml::node(Eml::one(), Eml::var("x")), Eml::one()),
    );
    for x_real in [0.5_f64, 1.0, 2.5, -1.0, -3.0] {
        let mut env = Env::new();
        env.bind("x", Value::new(x_real, 0.0));
        show(
            &format!("eml(1, eml(eml(1, x), 1))  x = {x_real:>4}"),
            eval(&ln_x, &env).expect("ln"),
        );
    }

    // ── Euler's identity, derived ────────────────────────────────────────
    println!("\nDerived Euler's identity — ln(-1) = iτ/2, built from `eml` and `1`:");
    let mut env = Env::new();
    env.bind("x", Value::new(-1.0, 0.0));
    show(
        "eml(1, eml(eml(1,x), 1))  x = -1",
        eval(&ln_x, &env).expect("ln(-1)"),
    );

    println!(
        "\nEverything above is one atom and one literal, composed into trees.\n\
         No `+`, no `×`, no `exp`, no `ln` primitive — just `eml` and `1`."
    );
}
