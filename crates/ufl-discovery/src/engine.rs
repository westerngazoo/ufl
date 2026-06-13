//! The discovery engine loop (SPEC-0008 §2.5).
//!
//! `run` reaches candidates only via `proposer.{seed,vary}` and accepts only via
//! `predicate.discharge` — the proposer-agnostic / verifier-exact seam
//! (*Verifier-Held Transparency*).

use ufl_tensor::{Scheme, SchemeError};

use crate::predicate::RankDecomposition;
use crate::proposer::{GaConfig, GaProposer};

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

/// Run the seeded genetic search. **R-0008 step 5 — the TDD-red target.**
pub fn run(config: &Config) -> Result<Outcome, EngineError> {
    config.validate()?;
    let _proposer = GaProposer::new(config.predicate.dim(), config.predicate.rank(), config.ga);
    unimplemented!("R-0008 step 5 — the engine loop, see SPEC-0008 §2.5")
}
