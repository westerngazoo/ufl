//! Hello, geometric algebra — the spatial substrate, in 20 lines.
//!
//! UFL's `Cl(3,0,1)` PGA kernel (a thin facade over garust). A **rotor**
//! sandwiches `e1` to `e2` — a τ/4 quarter-turn, the *same* angle as the EML
//! core's Euler `i` (one constant, two geometries). A **motor** then performs a
//! rigid-body motion on a point — including a **native translation**, which the
//! ideal generator `e₀` makes possible and no `G(3,0,0)` can show.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-ga --example hello_ga
//! ```

use std::f64::consts::TAU; // τ — UFL's circle constant
use ufl_ga::basis::{e1, e2};
use ufl_ga::{Motor, Point};

fn main() {
    println!("UFL — the Cl(3,0,1) PGA spatial substrate.\n");

    // ── A rotor: a τ/4 turn in the e₁₂ plane, applied by the sandwich R x R̃ ──
    let plane = e1() * e2(); // the unit bivector e₁₂
    let rotor = (plane * (-TAU / 8.0)).exp(); // exp(−½·τ/4·e₁₂)
    let rotated = rotor.sandwich(&e1()).cleaned(1e-10);
    println!("  rotor(τ/4, e₁₂).sandwich(e1)   →  {}", rotated); // e2

    // ── A motor: rotate a point, then a NATIVE translation (via e₀) ──────────
    let p = Point::new(1.0, 0.0, 0.0);
    let turned = p.transform(&Motor::<f64>::rotor(TAU / 4.0, e1() * e2()));
    let (x, y, z) = turned.to_euclidean();
    println!("  rotor·(1,0,0)                  →  ({x:.3}, {y:.3}, {z:.3})"); // (0,1,0)

    let moved = Point::new(0.0, 0.0, 0.0).transform(&Motor::<f64>::translator(2.0, 3.0, 4.0));
    let (x, y, z) = moved.to_euclidean();
    println!("  translator(2,3,4)·origin       →  ({x:.3}, {y:.3}, {z:.3})"); // (2,3,4)

    println!(
        "\nOne operator family — rotors and motors — over a real f64 multivector.\n\
         The translator is native (no VGA can do it); the τ/4 turn is the same\n\
         quarter-turn as the EML core's `i`. R-0010 lowers s-expr forms onto this."
    );
}
