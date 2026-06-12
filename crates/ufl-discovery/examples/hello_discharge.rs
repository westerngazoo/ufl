//! Hello, discharge — the verifier IS the Hehner discharge.
//!
//! One trait, two domains (SPEC-0007): the same `Predicate::discharge` that
//! checks a scalar Hehner predicate (`⟦ x' = (eml x 1) ⟧` — the s-expression
//! itself is the predicate) also discharges the matmul-decomposition predicate
//! `P_{2,7}` on Strassen's scheme. The discovery engine (R-0008) will search
//! for schemes that make this discharge true at ever-smaller rank.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-discovery --example hello_discharge
//! ```

use ufl_core::Value;
use ufl_discovery::RankDecomposition;
use ufl_predicate::{Predicate, State};
use ufl_syntax::read;
use ufl_tensor::{Scheme, Triple};

/// The canonical 7-term Strassen 2×2 scheme (SPEC-0006 §2.6).
fn strassen() -> Scheme {
    let rows: [([i8; 4], [i8; 4], [i8; 4]); 7] = [
        ([1, 0, 0, 1], [1, 0, 0, 1], [1, 0, 0, 1]),
        ([0, 0, 1, 1], [1, 0, 0, 0], [0, 0, 1, -1]),
        ([1, 0, 0, 0], [0, 1, 0, -1], [0, 1, 0, 1]),
        ([0, 0, 0, 1], [-1, 0, 1, 0], [1, 0, 1, 0]),
        ([1, 1, 0, 0], [0, 0, 0, 1], [-1, 1, 0, 0]),
        ([-1, 0, 1, 0], [1, 1, 0, 0], [0, 0, 0, 1]),
        ([0, 1, 0, -1], [0, 0, 1, 1], [1, 0, 0, 0]),
    ];
    let mut s = Scheme::new();
    for (u, v, w) in rows {
        s.push(Triple::new(u.to_vec(), v.to_vec(), w.to_vec()).expect("valid triple"))
            .expect("consistent length");
    }
    s
}

fn main() {
    println!("UFL — one discharge contract, two domains.\n");

    // ── Scalar: the s-expression IS the predicate ────────────────────────
    let pred = read("(pred (= x' (eml x 1)))").expect("reads");
    let e = Value::new(std::f64::consts::E, 0.0);
    let good = State::new(&[("x", Value::new(1.0, 0.0))], &[("x", e)]).expect("state");
    let bad = State::new(
        &[("x", Value::new(1.0, 0.0))],
        &[("x", Value::new(3.0, 0.0))],
    )
    .expect("state");
    println!(
        "  ⟦ x' = (eml x 1) ⟧  x=1, x'=e   → {:?}",
        pred.discharge(&good)
    );
    println!(
        "  ⟦ x' = (eml x 1) ⟧  x=1, x'=3   → {:?}",
        pred.discharge(&bad)
    );

    // ── Tensor: P_{2,7} on Strassen, through the SAME trait ─────────────
    let p27 = RankDecomposition::new(2, 7);
    let strassen = strassen();
    println!(
        "\n  P_{{2,7}}(strassen)             → {:?}",
        p27.discharge(&strassen)
    );

    // A broken scheme: drop the last triple (rank 6, inexact).
    let mut six = Scheme::new();
    for t in strassen.triples().iter().take(6).cloned() {
        six.push(t).expect("consistent");
    }
    println!(
        "  P_{{2,7}}(first 6 triples)      → {:?}",
        p27.discharge(&six)
    );
    println!(
        "  P_{{2,6}}(first 6 triples)      → {:?}",
        RankDecomposition::new(2, 6).discharge(&six)
    );

    println!(
        "\nSame trait, same discharge: a gate, a function, a decomposition —\n\
         all predicates. The GA (R-0008) searches for candidates that make\n\
         P_{{n,R}} discharge true at ever-smaller R."
    );
}
