//! The blind genetic proposer (SPEC-0008 §2.4).
//!
//! One implementation behind the proposer seam; R-0011 adds the geometric /
//! agentic proposers. Seeds and varies a population of [`Genome`]s; the engine
//! scores them via the verifier (Verifier-Held Transparency).

use crate::genome::Genome;
use crate::prng::{MatmulSampling, SplitMix64};

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
    pub(crate) fn seed(&self, rng: &mut SplitMix64) -> Vec<Genome> {
        (0..self.cfg.population)
            .map(|_| self.random_genome(rng))
            .collect()
    }

    fn random_genome(&self, rng: &mut SplitMix64) -> Genome {
        let mut triples = Vec::with_capacity(self.rank);
        for _ in 0..self.rank {
            let mut u = Vec::with_capacity(self.d);
            for _ in 0..self.d {
                u.push(rng.ternary());
            }

            let mut v = Vec::with_capacity(self.d);
            for _ in 0..self.d {
                v.push(rng.ternary());
            }

            let mut w = Vec::with_capacity(self.d);
            for _ in 0..self.d {
                w.push(rng.ternary());
            }

            triples.push([u, v, w]);
        }
        Genome { triples }
    }

    /// Next generation from scored parents — **must be sorted ascending by
    /// residual** (`engine::run` guarantees this). Elitism carries the best
    /// `elitism` unchanged; the rest are tournament-selected parents joined by
    /// uniform triple-crossover and point-mutated (SPEC-0008 §2.4).
    pub(crate) fn vary(&self, scored: &[(Genome, i64)], rng: &mut SplitMix64) -> Vec<Genome> {
        let mut next: Vec<Genome> = scored
            .iter()
            .take(self.cfg.elitism)
            .map(|(g, _)| g.clone())
            .collect();
        while next.len() < self.cfg.population {
            let a = self.tournament(scored, rng);
            let b = self.tournament(scored, rng);
            let child = self.crossover(a, b, rng);
            next.push(self.mutate(child, rng));
        }
        next
    }

    /// Tournament selection. `scored` is sorted ascending by residual, so the
    /// best of the sampled candidates is the **lowest index** — which also
    /// gives the deterministic lower-index tie-break (AC1).
    fn tournament<'a>(&self, scored: &'a [(Genome, i64)], rng: &mut SplitMix64) -> &'a Genome {
        let mut best = rng.below_usize(scored.len());
        for _ in 1..self.cfg.tournament_size {
            best = best.min(rng.below_usize(scored.len()));
        }
        &scored[best].0
    }

    /// Uniform crossover over the `rank` triples (the scheme is an unordered sum).
    fn crossover(&self, a: &Genome, b: &Genome, rng: &mut SplitMix64) -> Genome {
        let mut triples = Vec::with_capacity(self.rank);
        for i in 0..self.rank {
            if rng.below_usize(2) == 0 {
                triples.push(a.triples[i].clone());
            } else {
                triples.push(b.triples[i].clone());
            }
        }
        Genome { triples }
    }

    /// Point mutation — set `mutation_count` random entries to a fresh ternary.
    fn mutate(&self, mut g: Genome, rng: &mut SplitMix64) -> Genome {
        for _ in 0..self.cfg.mutation_count {
            let t = rng.below_usize(self.rank);
            let vec = rng.below_usize(3);
            let idx = rng.below_usize(self.d);
            g.triples[t][vec][idx] = rng.ternary();
        }
        g
    }
}
