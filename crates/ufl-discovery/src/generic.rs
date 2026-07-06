//! The matmul lane on the **genome-generic** search substrate (R-0014 AC2).
//!
//! The lane-agnostic seam — [`Proposer`]/[`Fitness`]/[`Screen`] and the hardened
//! [`run_generic`] loop — lives in the pure [`ufl_search`] crate (SPEC-0014 §2.1).
//! This module hosts the **matmul instances**: it implements those traits for the
//! `Genome`/`i64` lane and re-hosts [`engine::run`](crate::engine::run) on the
//! generic loop.
//!
//! [`run_matmul_generic`] is **byte-identical** to `engine::run` (proven in
//! `tests/r_0014_generic_seam.rs`) — the same seed yields the same trajectory and
//! outcome, because the operations and the `SplitMix64` draw order are mirrored
//! exactly, and the lane passes [`NoScreen`] so no candidate is ever dropped.

use ufl_search::{run_generic, Fitness, GenericOutcome, NoScreen, Proposer, RunError};

use crate::engine::{Config, EngineError, Outcome};
use crate::genome::{express, Genome};
use crate::predicate::RankDecomposition;
use crate::prng::SplitMix64;
use crate::proposer::GaProposer;

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
    type Error = EngineError;
    fn score(&self, genome: &Genome) -> Result<i64, EngineError> {
        Ok(self.predicate.residual(&express(genome)?)?)
    }
    fn solved(&self, score: &i64) -> bool {
        *score == 0
    }
}

/// Run the matmul lane on the generic loop. **Byte-identical to
/// [`engine::run`](crate::engine::run)** for every seed (the R-0014 AC2 proof).
///
/// The lane passes [`NoScreen`] (no candidate is ever dropped) and returns plain
/// [`EngineError`], folding the generic [`RunError`] back at the boundary
/// (SPEC-0014 §2.3). The `Ledger` is discarded here: the matmul [`Outcome`]
/// keeps its byte-identical shape; evals are reported by the generic loop's
/// [`GenericOutcome`] for the lanes that consume them.
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
    let (outcome, _ledger) = run_generic(
        &proposer,
        &fitness,
        &NoScreen,
        config.generations,
        config.seed,
    )
    .map_err(fold_matmul_run_error)?;
    Ok(match outcome {
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
    })
}

/// Fold the generic [`RunError<EngineError>`](RunError) back to the matmul lane's
/// plain [`EngineError`] (SPEC-0014 §2.3). Written as an explicit `match`, not an
/// `unwrap`: `Lane(e)` carries the real error through; `ProposerYieldedEmpty` is
/// **provably unreachable** for the matmul lane — `Config::validate` guarantees
/// `population ≥ 1`, `GaProposer::vary` preserves the population size, and the
/// lane's [`NoScreen`] drops nothing, so `seed`/`vary`/screen can never empty the
/// population. The impossible state maps to a typed engine error, never a panic.
fn fold_matmul_run_error(err: RunError<EngineError>) -> EngineError {
    match err {
        RunError::Lane(e) => e,
        // Unreachable for validated matmul configs (see doc); typed, not a panic.
        RunError::ProposerYieldedEmpty => EngineError::EmptyPopulation,
    }
}
