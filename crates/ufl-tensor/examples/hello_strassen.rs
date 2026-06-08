//! Hello, Strassen — the exact verifier the discovery engine will search against.
//!
//! Phase 0 of the discovery program (`ufl-discovery/FINDINGS.md`): build the
//! matmul target tensor `T_2`, and confirm the canonical 7-multiplication
//! Strassen scheme reconstructs it *exactly* (error 0) — while the naive R=8
//! scheme also reconstructs it, and a deliberately-broken scheme does not. This
//! is the "if this fails, nothing downstream is trustworthy" gate, made
//! tangible: the engine (R-0008) will search for schemes that drive this error
//! to 0 at ever-smaller rank R.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-tensor --example hello_strassen
//! ```

use ufl_tensor::{is_valid, scheme_error, Scheme, Triple};

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

/// The naive `R = n³` scheme: one triple per `(i, j, k)`.
fn naive(n: usize) -> Scheme {
    let d = n * n;
    let e = |idx: usize| {
        (0..d)
            .map(|x| if x == idx { 1 } else { 0 })
            .collect::<Vec<i8>>()
    };
    let mut s = Scheme::new();
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                s.push(Triple::new(e(i * n + j), e(j * n + k), e(i * n + k)).expect("valid"))
                    .expect("consistent");
            }
        }
    }
    s
}

fn main() {
    println!("UFL discovery — Phase 0: the exact matmul verifier (T_2)\n");

    let strassen = strassen();
    println!(
        "  Strassen  R={}  error = {:?}   valid@7: {}",
        strassen.rank(),
        scheme_error(&strassen, 2),
        is_valid(&strassen, 2, 7),
    );

    let naive = naive(2);
    println!(
        "  naive     R={}  error = {:?}   valid@8: {}",
        naive.rank(),
        scheme_error(&naive, 2),
        is_valid(&naive, 2, 8),
    );

    // A broken scheme: drop Strassen's last triple → no longer exact.
    let mut broken = strassen.triples().to_vec().into_iter();
    let mut six = Scheme::new();
    for t in broken.by_ref().take(6) {
        six.push(t).expect("consistent");
    }
    println!(
        "  broken    R={}  error = {:?}   valid@6: {}   (≠ 0 → rejected)",
        six.rank(),
        scheme_error(&six, 2),
        is_valid(&six, 2, 6),
    );

    println!(
        "\nA discovery is registered only when error == 0. The engine (R-0008)\n\
         will search for such schemes at ever-smaller R — Strassen's 7 is the\n\
         target to rediscover, then beat. Exact integer arithmetic; no float."
    );
}
