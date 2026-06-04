//! Hello, predicate — UFL checks a constraint over pre/post state.
//!
//! The first executable artifact on the control-layer frontier
//! (`theory/universal-computability.md`, Route B): the predicate
//! `⟦ x' = (eml x 1) ⟧` — "the post-state x' is e^x" — checked against concrete
//! pre/post pairs. Predicates *check* (decidable); the orchestrator will
//! *solve* (undecidable) — a later requirement.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-predicate --example hello_pred
//! ```

use ufl_core::{eval, Env, Value};
use ufl_predicate::check_str;
use ufl_syntax::{lower, read};

/// Evaluate `(eml x 1)` = e^x for a real x, the way the predicate's RHS does.
fn eml_x_1(x: f64) -> Value {
    let tree = lower(&read("(eml x 1)").expect("reads")).expect("lowers");
    let mut env = Env::new();
    env.bind("x", Value::new(x, 0.0));
    eval(&tree, &env).expect("evaluates")
}

fn main() {
    println!("UFL predicate checker — ⟦ x' = (eml x 1) ⟧  (\"post-state x' is e^x\")\n");

    let predicate = "(pred (= x' (eml x 1)))";

    for x in [0.0_f64, 1.0, 2.5] {
        let correct = eml_x_1(x); // the true post-state e^x
        let wrong = correct + Value::new(1.0, 0.0); // a deliberately wrong post

        let ok = check_str(predicate, &[("x", Value::new(x, 0.0))], &[("x", correct)]);
        let bad = check_str(predicate, &[("x", Value::new(x, 0.0))], &[("x", wrong)]);

        println!("  x = {x:>3}   post = e^x   → check = {:?}", ok.unwrap());
        println!("  x = {x:>3}   post = e^x+1 → check = {:?}", bad.unwrap());
    }

    println!("\nTyped, layered errors — never a panic:");
    for src in ["(= true 1)", "(and false (= zz 1))", ")"] {
        println!("  {src:<22} → {:?}", check_str(src, &[], &[]));
    }

    println!(
        "\nBooleans, equality, connectives — the substrate the control\n\
         constructions (branching, sequencing, recursion) are later expressed in."
    );
}
