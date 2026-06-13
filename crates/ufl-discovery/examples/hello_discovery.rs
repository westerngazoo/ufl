//! Hello, discovery — the engine finds its own certificate, the verifier checks it.
//!
//! Verifier-Held Transparency made tangible: a blind genetic search *proposes*
//! candidates; only the exact `RankDecomposition` discharge *accepts* one. The
//! engine rediscovers a decomposition of a planted target it was never handed,
//! and the found scheme re-discharges `Ok(true)` through a freshly constructed
//! predicate — "here is the scheme, check it", never "trust me".
//!
//! The same engine on the 2×2 matmul tensor (rank 7) plateaus — the honest
//! falsification that motivates R-0011's stronger proposer (the search is the
//! wall, not the verifier).
//!
//! Run with:
//!
//! ```text
//! cargo run -p ufl-discovery --example hello_discovery
//! ```

use ufl_discovery::{run, Config, GaConfig, Outcome, RankDecomposition};
use ufl_predicate::Predicate;
use ufl_tensor::{reconstruct, Scheme, Triple};

/// The planted target's 5 triples (papers-review §4b) — true rank ≤ 4, searched
/// at rank 5. The engine never sees these; it only sees the reconstructed target.
const PLANTED: [([i8; 4], [i8; 4], [i8; 4]); 5] = [
    ([0, 0, 1, 0], [1, 1, 0, 0], [1, 1, -1, 0]),
    ([-1, 1, 0, 0], [-1, -1, 1, 1], [1, -1, 0, -1]),
    ([0, 0, -1, 0], [-1, -1, 0, 1], [-1, 0, -1, 1]),
    ([0, 0, 1, 1], [-1, 0, 1, 1], [0, 0, 0, 1]),
    ([0, 0, 0, 0], [1, 0, 1, -1], [-1, 0, 1, 0]),
];

fn planted_scheme() -> Scheme {
    let mut s = Scheme::new();
    for (u, v, w) in PLANTED {
        s.push(Triple::new(u.to_vec(), v.to_vec(), w.to_vec()).expect("valid"))
            .expect("consistent");
    }
    s
}

fn main() {
    let config = |predicate, seed| Config {
        predicate,
        generations: 1500,
        seed,
        ga: GaConfig::pinned(),
    };

    // ── Discovery: rediscover a decomposition of the planted target ──────────
    println!("UFL discovery — the engine proposes, the verifier accepts.\n");
    let target = reconstruct(&planted_scheme());
    let predicate = || RankDecomposition::for_target(target.clone(), 5);
    match run(&config(predicate(), 3)).expect("runs") {
        Outcome::Found { scheme, generation } => {
            println!("  planted target (rank ≤ 4)  → Found at generation {generation}");
            // The certificate, re-checked by a fresh predicate — never "trust me".
            println!(
                "  certificate re-discharges    → {:?}",
                predicate().discharge(&scheme)
            );
        }
        Outcome::Exhausted { best_residual, .. } => {
            println!("  planted target  → Exhausted (residual {best_residual})");
        }
    }

    // ── Falsification: the same engine cannot rediscover Strassen ────────────
    println!("\nThe same blind engine on 2×2 matmul, rank 7 (the AlphaTensor prize):");
    if let Outcome::Exhausted {
        best_residual,
        trajectory,
        ..
    } = run(&config(RankDecomposition::new(2, 7), 0)).expect("runs")
    {
        println!(
            "  seed 0  → Exhausted: residual {} → {} (descends, then the landscape walls it)",
            trajectory.first().copied().unwrap_or(0),
            best_residual,
        );
    }

    println!(
        "\nThe proposer is blind and weak; the verifier is exact. Transparency\n\
         lives in the verifier — so R-0011 can swap in a stronger proposer\n\
         (memetic / agentic) with the accept step unchanged."
    );
}
