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
//! hardened traits to a new pure `ufl-search` substrate; T8 executes the move,
//! this hardens them in place in `ufl-discovery` first.)
//!
//! The seam is hardened per SPEC-0014 §2.2–§2.5: a lane-generic error channel
//! ([`Fitness::Error`]/[`RunError`]), an empty-population guard
//! ([`RunError::ProposerYieldedEmpty`]), an answer-blind coherence
//! [`Screen`], and a post-screen eval [`Ledger`].

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
///
/// The cost channel is **lane-generic**: each lane names its own [`Error`]
/// (SPEC-0014 §2.2), so a multi-source lane error (e.g. the geometric lane's
/// evaluation-or-grade sum) flows through without leaking a foreign type into
/// the substrate. The matmul instance sets `Error = EngineError`.
///
/// [`Error`]: Fitness::Error
pub trait Fitness<G, S> {
    /// The lane's structural-failure type surfaced by [`score`](Fitness::score).
    type Error;
    /// The genome's cost (lower = better). `Err` propagates a structural failure.
    fn score(&self, genome: &G) -> Result<S, Self::Error>;
    /// Is this cost an exact solution? (Matmul: residual `== 0`.)
    fn solved(&self, score: &S) -> bool;
}

/// A failure running the generic loop: either a lane [`Fitness::Error`] surfaced
/// by scoring, or the runtime event that a [`Proposer`] (or the [`Screen`])
/// yielded an **empty** population — a *different event* from the config-time
/// [`EngineError::EmptyPopulation`] validation (SPEC-0014 §2.3).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RunError<E> {
    /// The population was empty when a best genome was needed — `seed`/`vary` or
    /// the [`Screen`] admitted nothing. Guards the `first()` that would panic.
    #[error("proposer yielded an empty population at runtime")]
    ProposerYieldedEmpty,
    /// A lane error from [`Fitness::score`], carried transparently.
    #[error(transparent)]
    Lane(#[from] E),
}

/// Answer-blind admissibility: rejects structurally-incoherent candidates
/// **before** [`Fitness::score`] sees them (SPEC-0014 §2.4) — the harness-level
/// rendering of SPEC-0011 AC2, audited in one place.
///
/// **Answer-blindness is a construction contract, not merely a call-time one:** a
/// `Screen` instance may depend only on the *lane*, never on the *task instance*.
/// Otherwise a lane could bake target-derived data into the screen and leak the
/// answer without consulting the verifier. Grade coherence, for example, is a
/// property of the algebra, not the target, so it is constructible from the lane
/// alone.
pub trait Screen<G> {
    /// `false` drops the candidate before scoring. Must not consult the target.
    fn admissible(&self, genome: &G) -> bool;
}

/// The default screen: admits every candidate. The matmul and flip-graph lanes
/// use it, so their trajectories are byte-identical to the unscreened loop.
pub struct NoScreen;

impl<G> Screen<G> for NoScreen {
    fn admissible(&self, _genome: &G) -> bool {
        true
    }
}

/// The eval ledger — a plain count of **post-screen** [`Fitness::score`] calls
/// (SPEC-0014 §2.5); a screened-out genome is *not* an eval. This is R-0015's
/// meta-fitness unit ("verifier-work to target"). On the Exhausted path under the
/// default [`NoScreen`], `evals == population × (generations + 1)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ledger {
    /// The number of `Fitness::score` calls actually made.
    pub evals: u64,
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

/// A completed generic run: the [`GenericOutcome`] paired with its eval
/// [`Ledger`], or a [`RunError`] over the lane's error `E`.
pub type RunResult<G, S, E> = Result<(GenericOutcome<G, S>, Ledger), RunError<E>>;

/// The genome-generic seeded search — the [`engine::run`](crate::engine::run)
/// loop, lifted off `Genome`/`i64` and hardened (SPEC-0014 §2.2–§2.5).
/// Deterministic in `seed`; the proposer and [`Screen`] are answer-blind.
///
/// The `screen` filters **both** the seed population and each `vary` output at
/// the **top of every iteration, before `score`** (so a screened seed genome at
/// generation 0 also never reaches scoring). Returns the outcome alongside a
/// [`Ledger`] of post-screen `score` calls. With the default [`NoScreen`] the
/// filter is a pass-through, so the trajectory and `SplitMix64` draw order are
/// byte-identical to the unscreened loop.
pub fn run_generic<G, S, P, F, C>(
    proposer: &P,
    fitness: &F,
    screen: &C,
    generations: usize,
    seed: u64,
) -> RunResult<G, S, F::Error>
where
    G: Clone,
    S: Ord + Copy,
    P: Proposer<G, S>,
    F: Fitness<G, S>,
    C: Screen<G>,
{
    let mut rng = SplitMix64::new(seed);
    let mut pop = proposer.seed(&mut rng);
    let mut trajectory = Vec::with_capacity(generations + 1);
    let mut evals: u64 = 0;

    for generation in 0..=generations {
        // Drop structurally-incoherent candidates before scoring (answer-blind);
        // then score, then stable-sort ascending by cost (equal-cost genomes keep
        // insertion order — the lower-index tie-break of R-0008 AC1). Only
        // admissible, actually-scored genomes count into the ledger.
        let mut scored = pop
            .into_iter()
            .filter(|g| screen.admissible(g))
            .map(|g| {
                let s = fitness.score(&g)?;
                Ok::<_, RunError<F::Error>>((g, s))
            })
            .collect::<Result<Vec<(G, S)>, RunError<F::Error>>>()?;
        evals += scored.len() as u64;
        scored.sort_by_key(|(_, s)| *s);
        // Guard the empty population (`seed`/`vary` or the screen admitted
        // nothing) — a typed runtime error, never an index panic (§2.3).
        let best_score = scored.first().ok_or(RunError::ProposerYieldedEmpty)?.1;

        if fitness.solved(&best_score) {
            return Ok((
                GenericOutcome::Found {
                    genome: scored.swap_remove(0).0,
                    generation,
                },
                Ledger { evals },
            ));
        }

        trajectory.push(best_score);

        if generation == generations {
            return Ok((
                GenericOutcome::Exhausted {
                    best: scored.swap_remove(0).0,
                    best_score,
                    trajectory,
                },
                Ledger { evals },
            ));
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
/// (SPEC-0014 §2.3). The [`Ledger`] is discarded here: the matmul [`Outcome`]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

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
        type Error = EngineError;
        fn score(&self, g: &i64) -> Result<u64, EngineError> {
            Ok((g - self.target).unsigned_abs())
        }
        fn solved(&self, s: &u64) -> bool {
            *s == 0
        }
    }

    /// The same `run_generic` loop, instantiated over `i64`, finds the target —
    /// genome-agnostic by construction. Passes `&NoScreen` (byte-identical seam).
    #[test]
    fn run_generic_is_genome_agnostic() {
        let prop = ToyProposer {
            start: 50,
            population: 4,
        };
        let fit = ToyFitness { target: 42 };
        let (outcome, _ledger) = run_generic(&prop, &fit, &NoScreen, 20, 0).expect("toy run");
        match outcome {
            GenericOutcome::Found { genome, generation } => {
                assert_eq!(genome, 42, "hill-climb reaches the target");
                assert_eq!(generation, 8, "50 → 42 is 8 steps of −1");
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    // ── (b) the error channel is lane-generic — a non-`EngineError` toy enum ──

    /// A two-variant lane error with **no** relation to `EngineError` — the
    /// genericity witness (SPEC-0014 §2.2): the channel must carry a real
    /// multi-source lane error, not a single foreign type.
    #[derive(Debug, PartialEq, Eq)]
    enum ToyLaneError {
        Evaluation,
        Typing,
    }

    /// A `Fitness` whose `Error` is `ToyLaneError` — compiles iff `run_generic`
    /// is generic over `Fitness::Error` (not hardwired to `EngineError`).
    struct ToyLaneFitness {
        target: i64,
    }

    impl Fitness<i64, u64> for ToyLaneFitness {
        type Error = ToyLaneError;
        fn score(&self, g: &i64) -> Result<u64, ToyLaneError> {
            // Reject a sentinel to prove the `Err` path type-checks end to end.
            if *g == i64::MIN {
                return Err(ToyLaneError::Evaluation);
            }
            Ok((g - self.target).unsigned_abs())
        }
        fn solved(&self, s: &u64) -> bool {
            *s == 0
        }
    }

    /// The loop runs with a `Fitness::Error` that is a two-variant enum unrelated
    /// to `EngineError`, and its `RunError::Lane(ToyLaneError)` type-checks — the
    /// channel is lane-generic (SPEC-0014 §2.2). Referencing both variants keeps
    /// the enum from being dead.
    #[test]
    fn error_channel_is_lane_generic() {
        let prop = ToyProposer {
            start: 50,
            population: 4,
        };
        let fit = ToyLaneFitness { target: 42 };
        let (outcome, _ledger) = run_generic(&prop, &fit, &NoScreen, 20, 0).expect("toy lane run");
        assert!(matches!(outcome, GenericOutcome::Found { genome: 42, .. }));

        // The two-variant lane error round-trips through `RunError::Lane`.
        let lane_err: RunError<ToyLaneError> = RunError::Lane(ToyLaneError::Typing);
        assert_eq!(lane_err, RunError::Lane(ToyLaneError::Typing));
        assert_ne!(
            RunError::Lane(ToyLaneError::Evaluation),
            RunError::Lane(ToyLaneError::Typing)
        );
    }

    // ── (a) the empty-population guard — no panic ──

    /// A proposer whose `seed` yields an empty population — the runtime event
    /// `run_matmul_generic` validates away, but raw `run_generic` must guard
    /// (SPEC-0014 §2.3).
    struct EmptySeedProposer;

    impl Proposer<i64, u64> for EmptySeedProposer {
        fn seed(&self, _rng: &mut SplitMix64) -> Vec<i64> {
            Vec::new()
        }
        fn vary(&self, _ranked: &[(i64, u64)], _rng: &mut SplitMix64) -> Vec<i64> {
            Vec::new()
        }
    }

    /// An empty-seed proposer yields `Err(RunError::ProposerYieldedEmpty)` — a
    /// typed error, never an index panic (SPEC-0014 §2.3, CLAUDE.md §6).
    #[test]
    fn empty_seed_proposer_is_a_typed_error_not_a_panic() {
        let prop = EmptySeedProposer;
        let fit = ToyFitness { target: 42 };
        let result = run_generic(&prop, &fit, &NoScreen, 5, 0);
        assert_eq!(result, Err(RunError::ProposerYieldedEmpty));
    }

    // ── (c) the screen fires before `score` — a spy `Fitness` ──

    /// A `Fitness` that records every genome it scores — proves a screened-out
    /// genome never reaches `score()` (SPEC-0014 §2.4).
    struct SpyFitness {
        target: i64,
        seen: RefCell<Vec<i64>>,
    }

    impl Fitness<i64, u64> for SpyFitness {
        type Error = EngineError;
        fn score(&self, g: &i64) -> Result<u64, EngineError> {
            self.seen.borrow_mut().push(*g);
            Ok((g - self.target).unsigned_abs())
        }
        fn solved(&self, s: &u64) -> bool {
            *s == 0
        }
    }

    /// A `Screen` rejecting the exact genome `13` — answer-blind (it depends only
    /// on the constant, not on any task target).
    struct RejectThirteen;

    impl Screen<i64> for RejectThirteen {
        fn admissible(&self, g: &i64) -> bool {
            *g != 13
        }
    }

    /// A proposer that seeds `[13, 7]` — one screened-out, one admissible — and
    /// keeps re-emitting them, so `13` is offered at generation 0 and beyond.
    struct SeedThirteenAndSeven;

    impl Proposer<i64, u64> for SeedThirteenAndSeven {
        fn seed(&self, _rng: &mut SplitMix64) -> Vec<i64> {
            vec![13, 7]
        }
        fn vary(&self, _ranked: &[(i64, u64)], _rng: &mut SplitMix64) -> Vec<i64> {
            vec![13, 7]
        }
    }

    /// The screened-out seed genome `13` is never handed to `score()`, while the
    /// admissible `7` is — the screen fires at the top of the iteration, covering
    /// generation 0 (SPEC-0014 §2.4).
    #[test]
    fn screened_out_genome_never_reaches_score() {
        let prop = SeedThirteenAndSeven;
        let fit = SpyFitness {
            target: 0,
            seen: RefCell::new(Vec::new()),
        };
        let _ = run_generic(&prop, &fit, &RejectThirteen, 2, 0).expect("screened run");
        let seen = fit.seen.into_inner();
        assert!(
            !seen.contains(&13),
            "screened genome 13 must never be scored"
        );
        assert!(seen.contains(&7), "admissible genome 7 must be scored");
    }

    // ── (d) the eval ledger — evals == population × (generations + 1) ──

    /// A fixed-population proposer that never solves (target unreachable), so the
    /// run always Exhausts and every generation scores the full population.
    struct FixedPopExhaust {
        population: usize,
    }

    impl Proposer<i64, u64> for FixedPopExhaust {
        fn seed(&self, _rng: &mut SplitMix64) -> Vec<i64> {
            vec![0; self.population]
        }
        fn vary(&self, ranked: &[(i64, u64)], _rng: &mut SplitMix64) -> Vec<i64> {
            // Constant population size, never reaching the target.
            vec![ranked[0].0; ranked.len()]
        }
    }

    /// On the Exhausted path under `NoScreen`, the ledger counts exactly
    /// `population × (generations + 1)` post-screen `score` calls (SPEC-0014 §2.5).
    #[test]
    fn ledger_counts_population_times_generations_plus_one() {
        let population = 5usize;
        let generations = 4usize;
        let prop = FixedPopExhaust { population };
        // target 1 is unreachable from the constant genome 0 ⇒ Exhausted.
        let fit = ToyFitness { target: 1 };
        let (outcome, ledger) =
            run_generic(&prop, &fit, &NoScreen, generations, 0).expect("exhaust run");
        assert!(
            matches!(outcome, GenericOutcome::Exhausted { .. }),
            "the run must exhaust its budget"
        );
        assert_eq!(
            ledger.evals,
            (population * (generations + 1)) as u64,
            "evals == population × (generations + 1) on the NoScreen/Exhausted path"
        );
    }
}
