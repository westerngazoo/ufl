//! The **genome-generic** search seam (R-0014 AC2).
//!
//! [`run`](crate::engine::run) is the matmul-specific loop; this generalizes it
//! to *any* genome `G` and cost `S` behind a [`Proposer`]/[`Fitness`] pair —
//! still the proposer-agnostic / verifier-exact seam (the proposer is
//! answer-blind; only the `Fitness` sees the target). The loop is written and
//! tested **once**; lanes plug in as instances.
//!
//! [`run_matmul_generic`] re-hosts the existing matmul lane on this loop; it is
//! **byte-identical** to `engine::run` (proven in `tests/r_0014_generic_seam.rs`)
//! — the same seed yields the same trajectory and outcome, because the operations
//! and the `SplitMix64` draw order are mirrored exactly. (SPEC-0014 relocates the
//! traits to the shared `ufl-evolve` substrate; this lands the proof on `main`.)

use crate::engine::{Config, EngineError, Outcome};
use crate::genome::{express, Genome};
use crate::predicate::RankDecomposition;
use crate::prng::SplitMix64;
use crate::proposer::GaProposer;

/// Reaches candidates: an answer-blind seed and a variation step over the scored
/// (cost-sorted, ascending) population. Mirrors `GaProposer::{seed, vary}`.
pub trait Proposer<G, S> {
    /// The initial population.
    fn seed(&self, rng: &mut SplitMix64) -> Vec<G>;
    /// The next population from the cost-ascending-sorted parents.
    fn vary(&self, ranked: &[(G, S)], rng: &mut SplitMix64) -> Vec<G>;
}

/// Scores a genome as a **cost** (lower is better; the loop minimizes), and says
/// when a cost is a solution. The matmul instance uses the verifier residual.
pub trait Fitness<G, S> {
    /// The genome's cost (lower = better). `Err` propagates a structural failure.
    fn score(&self, genome: &G) -> Result<S, EngineError>;
    /// Is this cost an exact solution? (Matmul: residual `== 0`.)
    fn solved(&self, score: &S) -> bool;
}

/// The generic result. `Found::generation` counts `vary` applications (`0` ⇒ the
/// seed solved it); `Exhausted::trajectory` has `generations + 1` entries.
#[derive(Clone, Debug, PartialEq)]
pub enum GenericOutcome<G, S> {
    /// A solution genome and the generation it appeared at.
    Found { genome: G, generation: usize },
    /// Budget exhausted: the best genome, its cost, and the best-cost trajectory.
    Exhausted {
        best: G,
        best_score: S,
        trajectory: Vec<S>,
    },
}

/// The genome-generic seeded search — the [`engine::run`](crate::engine::run)
/// loop, lifted off `Genome`/`i64`. Deterministic in `seed`; the proposer is
/// answer-blind.
pub fn run_generic<G, S, P, F>(
    proposer: &P,
    fitness: &F,
    generations: usize,
    seed: u64,
) -> Result<GenericOutcome<G, S>, EngineError>
where
    G: Clone,
    S: Ord + Copy,
    P: Proposer<G, S>,
    F: Fitness<G, S>,
{
    let mut rng = SplitMix64::new(seed);
    let mut pop = proposer.seed(&mut rng);
    let mut trajectory = Vec::with_capacity(generations + 1);

    for generation in 0..=generations {
        // Score every genome, then stable-sort ascending by cost (equal-cost
        // genomes keep insertion order — the lower-index tie-break of R-0008 AC1).
        let mut scored = pop
            .into_iter()
            .map(|g| {
                let s = fitness.score(&g)?;
                Ok::<_, EngineError>((g, s))
            })
            .collect::<Result<Vec<(G, S)>, EngineError>>()?;
        scored.sort_by_key(|(_, s)| *s);
        let best_score = scored[0].1;

        if fitness.solved(&best_score) {
            return Ok(GenericOutcome::Found {
                genome: scored.swap_remove(0).0,
                generation,
            });
        }

        trajectory.push(best_score);

        if generation == generations {
            return Ok(GenericOutcome::Exhausted {
                best: scored.swap_remove(0).0,
                best_score,
                trajectory,
            });
        }

        pop = proposer.vary(&scored, &mut rng);
    }

    unreachable!("the loop returns Found or Exhausted at generation == generations")
}

// ── the matmul lane, re-hosted on the generic loop ──

/// The matmul proposer as a generic [`Proposer`] instance (delegates to the
/// existing `GaProposer`, so the draw order is unchanged).
struct MatmulProposer(GaProposer);

impl Proposer<Genome, i64> for MatmulProposer {
    fn seed(&self, rng: &mut SplitMix64) -> Vec<Genome> {
        self.0.seed(rng)
    }
    fn vary(&self, ranked: &[(Genome, i64)], rng: &mut SplitMix64) -> Vec<Genome> {
        self.0.vary(ranked, rng)
    }
}

/// The matmul verifier as a generic [`Fitness`] instance: cost = `residual`,
/// solved = `residual == 0`.
struct MatmulFitness<'a> {
    predicate: &'a RankDecomposition,
}

impl Fitness<Genome, i64> for MatmulFitness<'_> {
    fn score(&self, genome: &Genome) -> Result<i64, EngineError> {
        Ok(self.predicate.residual(&express(genome)?)?)
    }
    fn solved(&self, score: &i64) -> bool {
        *score == 0
    }
}

/// Run the matmul lane on the generic loop. **Byte-identical to
/// [`engine::run`](crate::engine::run)** for every seed (the R-0014 AC2 proof).
pub fn run_matmul_generic(config: &Config) -> Result<Outcome, EngineError> {
    config.validate()?;
    let proposer = MatmulProposer(GaProposer::new(
        config.predicate.dim(),
        config.predicate.rank(),
        config.ga,
    ));
    let fitness = MatmulFitness {
        predicate: &config.predicate,
    };
    Ok(
        match run_generic(&proposer, &fitness, config.generations, config.seed)? {
            GenericOutcome::Found { genome, generation } => Outcome::Found {
                scheme: express(&genome)?,
                generation,
            },
            GenericOutcome::Exhausted {
                best,
                best_score,
                trajectory,
            } => Outcome::Exhausted {
                best: express(&best)?,
                best_residual: best_score,
                trajectory,
            },
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // A toy lane over a DIFFERENT genome (`i64`) and cost (`u64`) — proves the
    // loop is genuinely generic, not matmul-shaped. A hill-climb toward a target.
    struct ToyProposer {
        start: i64,
        population: usize,
    }
    struct ToyFitness {
        target: i64,
    }

    impl Proposer<i64, u64> for ToyProposer {
        fn seed(&self, _rng: &mut SplitMix64) -> Vec<i64> {
            vec![self.start; self.population]
        }
        fn vary(&self, ranked: &[(i64, u64)], _rng: &mut SplitMix64) -> Vec<i64> {
            let best = ranked[0].0;
            // Neighbours of the best, padded to the population.
            let mut next = vec![best - 1, best, best + 1];
            next.resize(ranked.len(), best);
            next
        }
    }

    impl Fitness<i64, u64> for ToyFitness {
        fn score(&self, g: &i64) -> Result<u64, EngineError> {
            Ok((g - self.target).unsigned_abs())
        }
        fn solved(&self, s: &u64) -> bool {
            *s == 0
        }
    }

    /// The same `run_generic` loop, instantiated over `i64`, finds the target —
    /// genome-agnostic by construction.
    #[test]
    fn run_generic_is_genome_agnostic() {
        let prop = ToyProposer {
            start: 50,
            population: 4,
        };
        let fit = ToyFitness { target: 42 };
        let outcome = run_generic(&prop, &fit, 20, 0).expect("toy run");
        match outcome {
            GenericOutcome::Found { genome, generation } => {
                assert_eq!(genome, 42, "hill-climb reaches the target");
                assert_eq!(generation, 8, "50 → 42 is 8 steps of −1");
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }
}
