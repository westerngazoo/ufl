//! The discovery engine loop (SPEC-0008 §2.5).
//!
//! `run` reaches candidates only via `proposer.{seed,vary}` and accepts only via
//! `predicate.discharge` — the proposer-agnostic / verifier-exact seam
//! (*Verifier-Held Transparency*).

use ufl_tensor::{Scheme, SchemeError};

use crate::genome::{express, Genome};
use crate::predicate::RankDecomposition;
use crate::prng::SplitMix64;
use crate::proposer::{GaConfig, GaProposer};
use ufl_predicate::Predicate;

/// A failure running the engine. The `Scheme` variant carries the *impossible*
/// dim error from `express`/`residual` — propagated, never hidden.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EngineError {
    #[error("population must be ≥ 1")]
    EmptyPopulation,
    #[error("generations must be ≥ 1")]
    ZeroGenerations,
    #[error("elitism must be ≥ 1")]
    ZeroElitism,
    #[error("tournament_size must be ≥ 1")]
    ZeroTournament,
    #[error("elitism ({elitism}) may not exceed population ({population})")]
    ElitismExceedsPopulation { elitism: usize, population: usize },
    #[error(transparent)]
    Scheme(#[from] SchemeError),
}

/// A discovery run: the predicate to satisfy, the budget, the seed, the GA config.
pub struct Config {
    pub predicate: RankDecomposition,
    pub generations: usize,
    pub seed: u64,
    pub ga: GaConfig,
}

impl Config {
    /// Reject degenerate configurations so the engine's invariants hold by
    /// construction (no `min().unwrap_or` sentinel ever fires).
    pub fn validate(&self) -> Result<(), EngineError> {
        if self.ga.population == 0 {
            return Err(EngineError::EmptyPopulation);
        }
        if self.generations == 0 {
            return Err(EngineError::ZeroGenerations);
        }
        if self.ga.elitism == 0 {
            return Err(EngineError::ZeroElitism);
        }
        if self.ga.tournament_size == 0 {
            return Err(EngineError::ZeroTournament);
        }
        if self.ga.elitism > self.ga.population {
            return Err(EngineError::ElitismExceedsPopulation {
                elitism: self.ga.elitism,
                population: self.ga.population,
            });
        }
        Ok(())
    }
}

/// The result of a run.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// A discovery — the certificate scheme, and the number of `vary`
    /// applications that preceded acceptance (`0` ⇒ the seed population solved
    /// it).
    Found { scheme: Scheme, generation: usize },
    /// Budget exhausted — the best phenotype, its residual, and the
    /// per-generation best-residual trajectory (`trajectory[0]` is the seed
    /// population; `len == generations + 1`).
    Exhausted {
        best: Scheme,
        best_residual: i64,
        trajectory: Vec<i64>,
    },
}

/// Express every genome and grade by the verifier's `residual`, returning the
/// population sorted ascending by residual (a stable sort, so equal-residual
/// genomes keep insertion order — the lower-index tie-break of AC1). The `?`
/// propagates the *impossible* dim error rather than hiding it.
fn score(
    predicate: &RankDecomposition,
    pop: Vec<Genome>,
) -> Result<Vec<(Genome, i64)>, EngineError> {
    let mut scored = pop
        .into_iter()
        .map(|g| {
            let scheme = express(&g)?;
            Ok((g, predicate.residual(&scheme)?))
        })
        .collect::<Result<Vec<_>, SchemeError>>()?;
    scored.sort_by_key(|(_, residual)| *residual);
    Ok(scored)
}

/// Run the seeded genetic search (SPEC-0008 §2.5). `run` reaches candidates only
/// via `proposer.{seed,vary}` and accepts only via `predicate.discharge` — the
/// proposer-agnostic / verifier-exact seam.
///
/// The `gen` index counts `vary` applications: `gen == 0` is the seed
/// population, so `Found { generation: k }` means `k` generations of variation
/// preceded acceptance, and an `Exhausted` trajectory has `generations + 1`
/// entries (index 0 = the seed population).
pub fn run(config: &Config) -> Result<Outcome, EngineError> {
    config.validate()?;
    let proposer = GaProposer::new(config.predicate.dim(), config.predicate.rank(), config.ga);
    let mut rng = SplitMix64::new(config.seed);
    let mut pop = proposer.seed(&mut rng);
    let mut trajectory = Vec::with_capacity(config.generations + 1);

    for gen in 0..=config.generations {
        let scored = score(&config.predicate, pop)?;
        let best_residual = scored[0].1;

        if best_residual == 0 {
            let scheme = express(&scored[0].0)?;
            // residual 0 ∧ fixed rank R ⇒ discharge Ok(true), by construction.
            debug_assert_eq!(config.predicate.discharge(&scheme), Ok(true));
            return Ok(Outcome::Found {
                scheme,
                generation: gen,
            });
        }

        trajectory.push(best_residual);

        if gen == config.generations {
            return Ok(Outcome::Exhausted {
                best: express(&scored[0].0)?,
                best_residual,
                trajectory,
            });
        }
        pop = proposer.vary(&scored, &mut rng);
    }

    unreachable!("the loop returns Found or Exhausted at gen == generations")
}
