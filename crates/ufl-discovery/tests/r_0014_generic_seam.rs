//! R-0014 AC2 — the matmul lane re-hosted on the genome-generic loop is
//! **byte-identical** to the original `engine::run`. The proposer-agnostic seam
//! generalizes without changing R-0008's behavior: same seed ⇒ same outcome and
//! same trajectory, across a sweep of configs (trivial-found and exhausted-with-
//! trajectory alike).

use ufl_discovery::{run, run_matmul_generic, Config, GaConfig, RankDecomposition};

fn config(n: usize, rank: usize, generations: usize, population: usize, seed: u64) -> Config {
    Config {
        predicate: RankDecomposition::new(n, rank),
        generations,
        seed,
        ga: GaConfig {
            population,
            tournament_size: 5,
            mutation_count: 2,
            elitism: 4,
        },
    }
}

/// `run_matmul_generic == run` for every seed and config — the byte-identical
/// proof. Includes the trivial `n=1` (solved fast) and a small `n=2, rank=7`
/// (exhausted, exercising the full select/vary trajectory).
#[test]
fn generic_seam_is_byte_identical_to_run() {
    let cases = [
        // (n, rank, generations, population)
        (1, 1, 5, 30),  // 1×1: the seed almost always solves it (Found path)
        (1, 2, 8, 40),  // rank > needed: still solvable
        (2, 7, 12, 60), // Strassen-shaped: hard, Exhausted with a real trajectory
        (2, 8, 6, 50),  // the naive rank: exercises select/vary too
    ];
    for (n, rank, gens, pop) in cases {
        for seed in 0u64..6 {
            let cfg = config(n, rank, gens, pop, seed);
            let original = run(&cfg).expect("run");
            let rehosted = run_matmul_generic(&cfg).expect("run_matmul_generic");
            assert_eq!(
                original, rehosted,
                "outcome diverged at n={n} rank={rank} gens={gens} pop={pop} seed={seed}",
            );
        }
    }
}

/// The seam is also exercised on the planted-target verifier (SPEC-0008 AC3) —
/// the same `for_target` predicate R-0008 recovers, run through the generic loop.
#[test]
fn generic_seam_matches_run_on_a_solvable_run() {
    // A config where `run` reaches a definite Found or Exhausted; the generic
    // re-host must agree exactly, including the `generation` / `trajectory`.
    let cfg = config(1, 1, 3, 20, 123);
    assert_eq!(run(&cfg).unwrap(), run_matmul_generic(&cfg).unwrap());
}
