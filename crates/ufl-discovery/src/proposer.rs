//! The blind genetic proposer (SPEC-0008 §2.4).
//!
//! One implementation behind the proposer seam; R-0011 adds the geometric /
//! agentic proposers. Seeds and varies a population of [`Genome`]s; the engine
//! scores them via the verifier (Verifier-Held Transparency).

use crate::genome::Genome;
use crate::prng::SplitMix64;

/// GA hyper-parameters. `elitism ≥ 1` is required (AC6) and validated by
/// `Config::validate`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaConfig {
    pub population: usize,
    pub tournament_size: usize,
    pub mutation_count: usize,
    pub elitism: usize,
}

impl GaConfig {
    /// The single pinned configuration that recovered the planted target 8/10
    /// in the de-risk (`ufl-discovery/papers-review.md` §4b). `generations`
    /// (1500) lives on [`Config`](crate::Config).
    pub const fn pinned() -> Self {
        Self {
            population: 300,
            tournament_size: 5,
            mutation_count: 2,
            elitism: 4,
        }
    }
}

/// The blind proposer: uniform-random seed + tournament/crossover/mutation.
// Fields + seed/vary are consumed by `engine::run` in R-0008 step 5; unused
// during the TDD-red scaffold.
#[allow(dead_code)]
pub struct GaProposer {
    d: usize,
    rank: usize,
    cfg: GaConfig,
}

impl GaProposer {
    /// Build for `d = n²`-length vectors and `rank` triples. Config validity is
    /// the engine's `Config::validate` responsibility.
    pub fn new(d: usize, rank: usize, cfg: GaConfig) -> Self {
        Self { d, rank, cfg }
    }

    /// Initial population — each entry uniform `{-1, 0, +1}`.
    #[allow(dead_code)]
    pub(crate) fn seed(&self, rng: &mut SplitMix64) -> Vec<Genome> {
        (0..self.cfg.population)
            .map(|_| self.random_genome(rng))
            .collect()
    }

    #[allow(dead_code)]
    fn random_genome(&self, rng: &mut SplitMix64) -> Genome {
        let vec = |rng: &mut SplitMix64| (0..self.d).map(|_| rng.ternary()).collect::<Vec<i8>>();
        let triples = (0..self.rank)
            .map(|_| [vec(rng), vec(rng), vec(rng)])
            .collect();
        Genome { triples }
    }

    /// Next generation from scored parents (elitism + tournament-selected
    /// uniform triple-crossover + point mutation). **R-0008 step 5 — the
    /// TDD-red search target.**
    #[allow(dead_code)]
    pub(crate) fn vary(&self, _scored: &[(Genome, i64)], _rng: &mut SplitMix64) -> Vec<Genome> {
        unimplemented!("R-0008 step 5 — GA variation, see SPEC-0008 §2.4")
    }
}
