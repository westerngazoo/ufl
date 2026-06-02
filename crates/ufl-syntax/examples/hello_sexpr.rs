//! Hello, s-expression — UFL is now a language you can *type*.
//!
//! The same literal strings the docs use (`(eml 1 1)`, `(eml x 1)`,
//! `(eml 1 (eml (eml 1 x) 1))`) are read, lowered into the typed `Eml` core,
//! and evaluated — reusing R-0001's verified evaluator. This is the moment the
//! docs' notation becomes runnable syntax.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-syntax --example hello_sexpr
//! ```

use ufl_core::{Env, Value};
use ufl_syntax::eval_str;

fn show(src: &str, env: &Env) {
    match eval_str(src, env) {
        Ok(v) if v.im.abs() < 1e-12 => println!("  {src:<34} = {:>12.6}", v.re),
        Ok(v) => println!("  {src:<34} = {:>12.6} + {:>12.6} i", v.re, v.im),
        Err(e) => println!("  {src:<34} ! {e}"),
    }
}

fn main() {
    println!("UFL as s-expressions — the docs' notation, now runnable.\n");

    // e = (eml 1 1) — no variables, an empty environment suffices.
    println!("e — Euler's number, two `1`s and one operator:");
    show("(eml 1 1)", &Env::new());

    // exp(x) = (eml x 1)
    println!("\nexp(x) = (eml x 1):");
    for x in [0.0_f64, 1.0, 2.5] {
        let mut env = Env::new();
        env.bind("x", Value::new(x, 0.0));
        show("(eml x 1)", &env);
        println!("    (x = {x})");
    }

    // ln(x) = (eml 1 (eml (eml 1 x) 1)) — works on negatives via the f64
    // self-correction inherited from R-0001.
    println!("\nln(x) = (eml 1 (eml (eml 1 x) 1)):");
    for x in [0.5_f64, 1.0, 2.5, -1.0] {
        let mut env = Env::new();
        env.bind("x", Value::new(x, 0.0));
        show("(eml 1 (eml (eml 1 x) 1))", &env);
        println!("    (x = {x})");
    }

    // Typed errors, never panics — surfaced at the earliest layer.
    println!("\nErrors are typed and layered (read → lower → eval):");
    show(")", &Env::new()); // ReadError
    show("(foo 1 1)", &Env::new()); // LowerError
    show("x", &Env::new()); // EvalError (unbound)

    println!(
        "\nEvery line read the literal text, lowered it to the typed `eml` core,\n\
         and reused R-0001's evaluator — homoiconic surface, verified numerics."
    );
}
