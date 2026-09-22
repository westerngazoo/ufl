//! Emit `t1,t2,x_witness,y_witness,x_f64ref,y_f64ref` as CSV for one angle band,
//! so `experiments/0022-exact-fk-reference.py` can score both against an
//! 80-digit reference (SPEC-0022 §4.1's boundary table).
//!
//! ```text
//! cargo run -p ufl-evolve --release --example boundary_samples \
//!     | python3 experiments/0022-exact-fk-reference.py
//! ```
use ufl_evolve::baseline::ArmFk;
use ufl_evolve::witness::{fk_witness, read_xy, sample};

/// The bands SPEC-0022 §4.1 tabulates, and the grid it uses.
const BANDS: [(f64, f64); 6] = [
    (-2.0, 2.0),
    (2.0, 3.0),
    (-8.0, 8.0),
    (-1e2, 1e2),
    (-1e3, 1e3),
    (-1e9, 1e9),
];
const N: usize = 60;

fn main() {
    let arm = ArmFk { l1: 1.0, l2: 0.7 };
    let expr = fk_witness(arm.l1, arm.l2);
    println!("# l1={} l2={} n={N}", arm.l1, arm.l2);
    println!("band_lo,band_hi,t1,t2,xw,yw,xr,yr");
    for (lo, hi) in BANDS {
        for i in 0..N {
            for j in 0..N {
                let t1 = lo + (hi - lo) * i as f64 / (N - 1) as f64;
                let t2 = lo + (hi - lo) * j as f64 / (N - 1) as f64;
                let Ok(mv) = sample(&expr, t1, t2) else {
                    eprintln!("eval failed at ({t1}, {t2})");
                    continue;
                };
                let (xw, yw) = read_xy(&mv);
                let (xr, yr) = arm.forward(t1, t2);
                println!("{lo:e},{hi:e},{t1:e},{t2:e},{xw:e},{yw:e},{xr:e},{yr:e}");
            }
        }
    }
}
