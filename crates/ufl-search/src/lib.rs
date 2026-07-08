//! The **genome-generic** search substrate (R-0014 AC2) — the pure, lane-agnostic
//! seam (SPEC-0014 §2.1).
//!
//! [`run_generic`] is the seeded search loop lifted off any concrete genome: it
//! searches *any* genome `G` and cost `S` behind a [`Proposer`]/[`Fitness`] pair —
//! the proposer-agnostic / verifier-exact seam (the proposer is answer-blind; only
//! the `Fitness` sees the target). The loop is written and tested **once** here;
//! lanes (matmul in `ufl-discovery`, geometric in `ufl-geo`) plug in as instances.
//!
//! This crate depends on **`ufl-prng` only** — it carries no matmul, tensor, or
//! geometric dependency (SPEC-0014 §2.1). The determinism-critical draws all come
//! from [`ufl_prng::SplitMix64`], so a seeded run is reproducible across lanes.
//!
//! The seam is hardened per SPEC-0014 §2.2–§2.5: a lane-generic error channel
//! ([`Fitness::Error`]/[`RunError`]), an empty-population guard
//! ([`RunError::ProposerYieldedEmpty`]), an answer-blind coherence
//! [`Screen`], and a post-screen eval [`Ledger`].

#![forbid(unsafe_code)]

use core::fmt;

use ufl_prng::SplitMix64;

/// Reaches candidates: an answer-blind seed and a variation step over the scored
/// (cost-sorted, ascending) population. Mirrors a lane's `seed`/`vary`.
pub trait Proposer<G, S> {
    /// The initial population.
    fn seed(&self, rng: &mut SplitMix64) -> Vec<G>;
    /// The next population from the cost-ascending-sorted parents.
    fn vary(&self, ranked: &[(G, S)], rng: &mut SplitMix64) -> Vec<G>;
}

/// Scores a genome as a **cost** (lower is better; the loop minimizes), and says
/// when a cost is a solution. A lane's instance uses its verifier residual.
///
/// The cost channel is **lane-generic**: each lane names its own [`Error`]
/// (SPEC-0014 §2.2), so a multi-source lane error (e.g. the geometric lane's
/// evaluation-or-grade sum) flows through without leaking a foreign type into
/// the substrate.
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
/// yielded an **empty** population — a *different event* from a lane's config-time
/// empty-population validation (SPEC-0014 §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError<E> {
    /// The population was empty when a best genome was needed — `seed`/`vary` or
    /// the [`Screen`] admitted nothing. Guards the `first()` that would panic.
    ProposerYieldedEmpty,
    /// A lane error from [`Fitness::score`], carried transparently.
    Lane(E),
}

impl<E> From<E> for RunError<E> {
    fn from(err: E) -> Self {
        RunError::Lane(err)
    }
}

impl<E: fmt::Display> fmt::Display for RunError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::ProposerYieldedEmpty => {
                f.write_str("proposer yielded an empty population at runtime")
            }
            // Transparent: delegate to the wrapped lane error's own display.
            RunError::Lane(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RunError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RunError::ProposerYieldedEmpty => None,
            RunError::Lane(err) => Some(err),
        }
    }
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

/// Proposes *neighbor* genomes of an elite for the engine to hill-climb — the
/// memetic seam (SPEC-0011M §2.1). Answer-blind **by signature**, exactly like
/// [`Screen`]: `neighbors` receives only `&G` and the rng — **no target, no
/// [`Fitness`], and no cost `S`** — so the refiner cannot condition on fitness and
/// cannot see the answer. It returns candidates *without scoring them*; only the
/// engine ([`run_memetic`]) scores, so Verifier-Held Transparency survives the
/// memetic upgrade. An instance may depend only on the *lane*, never on the *task*.
pub trait Refiner<G> {
    /// The local neighborhood of `elite`. May be empty (no move available). The
    /// engine screens then scores each neighbor and keeps the best strict
    /// improvement; the refiner only *proposes* moves. **All rng the memetic pass
    /// draws originates here** — the engine's hill-climb bookkeeping draws none.
    fn neighbors(&self, elite: &G, rng: &mut SplitMix64) -> Vec<G>;
}

/// The default refiner: proposes nothing and **draws no rng**. Lanes with no local
/// move use it, so [`run_memetic`] collapses to [`run_generic`]'s trajectory
/// byte-for-byte (the ablation harness).
pub struct NoRefine;

impl<G> Refiner<G> for NoRefine {
    fn neighbors(&self, _elite: &G, _rng: &mut SplitMix64) -> Vec<G> {
        Vec::new()
    }
}

/// The memetic budget (SPEC-0011M §2.1): how hard the engine hill-climbs each
/// generation's elites. `elites == 0` **or** `steps == 0` skips the refinement
/// pass entirely, so [`run_memetic`] is byte-identical to [`run_generic`] (the
/// same collapse [`NoRefine`] gives with any budget).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemeticConfig {
    /// How many cost-lowest elites to refine each generation.
    pub elites: usize,
    /// Max hill-climb steps per elite (each step: propose, take the best strict
    /// improvement, or stop).
    pub steps: usize,
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

/// The genome-generic seeded search — the hardened search loop (SPEC-0014
/// §2.2–§2.5), lifted off any concrete genome/cost. Deterministic in `seed`; the
/// proposer and [`Screen`] are answer-blind.
///
/// The `screen` filters **both** the seed population and each `vary` output at
/// the **top of every iteration, before `score`** (so a screened seed genome at
/// generation 0 also never reaches scoring). Returns the outcome alongside a
/// [`Ledger`] of post-screen `score` calls. With the default [`NoScreen`] the
/// filter is a pass-through, so the trajectory and `SplitMix64` draw order are
/// byte-identical to the unscreened loop (the matmul lane's re-host proof).
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

/// The genome-generic **memetic** search — [`run_generic`]'s loop plus a
/// per-generation elite-refinement pass driven by a [`Refiner`] and scored by the
/// engine's [`Fitness`] (SPEC-0011M §2.1). The proposer *and* the refiner are
/// answer-blind; only this function holds `fitness`.
///
/// **The collapse (byte-identity — SPEC-0011M §2.1, the load-bearing property).**
/// The refinement pass is skipped *in full* when `memetic.elites == 0` or
/// `memetic.steps == 0`. When it does run, the engine's hill-climb bookkeeping
/// pulls **zero** `rng` draws — every draw originates inside
/// [`Refiner::neighbors`] — and [`NoRefine`] yields `[]` (no draw, no score, no
/// mutation). So with `NoRefine` (or a zero budget) the [`GenericOutcome`],
/// [`Ledger`], and `SplitMix64` draw order are **identical** to `run_generic`,
/// *including on a `Found` run* (no population mutation occurs, so the returned
/// best cannot drift). This is the ablation / collapse-proof path; production
/// lanes with no local move keep calling `run_generic` directly.
#[allow(clippy::too_many_arguments)]
pub fn run_memetic<G, S, P, F, C, R>(
    proposer: &P,
    fitness: &F,
    screen: &C,
    refiner: &R,
    memetic: MemeticConfig,
    generations: usize,
    seed: u64,
) -> RunResult<G, S, F::Error>
where
    G: Clone,
    S: Ord + Copy,
    P: Proposer<G, S>,
    F: Fitness<G, S>,
    C: Screen<G>,
    R: Refiner<G>,
{
    let mut rng = SplitMix64::new(seed);
    let mut pop = proposer.seed(&mut rng);
    let mut trajectory = Vec::with_capacity(generations + 1);
    let mut evals: u64 = 0;

    for generation in 0..=generations {
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

        // Memetic refinement — skipped in full unless BOTH knobs are positive, so
        // the `NoRefine`/zero-budget collapse draws no `rng`, scores nothing, and
        // mutates nothing (byte-identity to `run_generic`, incl. on `Found`).
        if memetic.elites > 0 && memetic.steps > 0 {
            let k = memetic.elites.min(scored.len());
            let mut refined_any = false;
            for elite in scored.iter_mut().take(k) {
                for _ in 0..memetic.steps {
                    // The refiner proposes neighbors (drawing its own `rng`); the
                    // ENGINE screens then scores them and keeps the best strict
                    // improvement. `NoRefine` yields `[]` here → no draw, no score.
                    let neighbors = refiner.neighbors(&elite.0, &mut rng);
                    let mut best: Option<(G, S)> = None;
                    for g in neighbors.into_iter().filter(|g| screen.admissible(g)) {
                        let s = fitness.score(&g)?;
                        evals += 1;
                        if s < elite.1 {
                            match &best {
                                None => best = Some((g, s)),
                                Some((_, bs)) if s < *bs => best = Some((g, s)),
                                _ => {}
                            }
                        }
                    }
                    match best {
                        // Keep the strict improvement and try another step.
                        Some(improved) => {
                            *elite = improved;
                            refined_any = true;
                        }
                        // No strictly-lower neighbor — this elite is locally done.
                        None => break,
                    }
                }
            }
            // Re-rank only if refinement actually moved an elite (a no-op re-sort
            // would still be stable, but skipping it keeps the collapse obvious).
            if refined_any {
                scored.sort_by_key(|(_, s)| *s);
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // A lane-local toy error standing in for a real lane's structural-failure
    // type — the pure substrate names no concrete lane error, so the toy lanes
    // carry their own.
    #[derive(Debug, PartialEq, Eq)]
    struct ToyError;

    // A toy lane over a genome (`i64`) and cost (`u64`) — proves the loop is
    // genuinely generic, not matmul-shaped. A hill-climb toward a target.
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
        type Error = ToyError;
        fn score(&self, g: &i64) -> Result<u64, ToyError> {
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

    // ── (b) the error channel is lane-generic — a two-variant toy enum ──

    /// A two-variant lane error — the genericity witness (SPEC-0014 §2.2): the
    /// channel must carry a real multi-source lane error, not a single foreign
    /// type.
    #[derive(Debug, PartialEq, Eq)]
    enum ToyLaneError {
        Evaluation,
        Typing,
    }

    /// A `Fitness` whose `Error` is `ToyLaneError` — compiles iff `run_generic`
    /// is generic over `Fitness::Error` (not hardwired to one lane's error).
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

    /// The loop runs with a `Fitness::Error` that is a two-variant enum, and its
    /// `RunError::Lane(ToyLaneError)` type-checks — the channel is lane-generic
    /// (SPEC-0014 §2.2). Referencing both variants keeps the enum from being dead.
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

    /// A proposer whose `seed` yields an empty population — the runtime event a
    /// lane validates away, but raw `run_generic` must guard (SPEC-0014 §2.3).
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
        type Error = ToyError;
        fn score(&self, g: &i64) -> Result<u64, ToyError> {
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

    // ── (e) the memetic layer: Refiner seam + run_memetic (SPEC-0011M §2.1) ──

    /// The best cost of an outcome (a solved run is cost 0).
    fn best_cost(outcome: &GenericOutcome<i64, u64>) -> u64 {
        match outcome {
            GenericOutcome::Found { .. } => 0,
            GenericOutcome::Exhausted { best_score, .. } => *best_score,
        }
    }

    /// **The collapse (byte-identity).** `run_memetic` with `NoRefine` — under a
    /// positive budget (pass entered, yields `[]`) AND with `elites:0` (pass
    /// skipped) — reproduces `run_generic`'s exact `GenericOutcome` + `Ledger` +
    /// draw order, on BOTH a `Found` run and an `Exhausted` run (the terminal
    /// generation, where refinement would otherwise drift the returned best).
    #[test]
    fn no_refine_collapses_to_run_generic_byte_identical() {
        let prop = ToyProposer {
            start: 50,
            population: 4,
        };

        // A run that reaches Found (50 → 42 in 8 generations).
        let fit_found = ToyFitness { target: 42 };
        let base_found = run_generic(&prop, &fit_found, &NoScreen, 20, 0);
        let mem_found_budget = run_memetic(
            &prop,
            &fit_found,
            &NoScreen,
            &NoRefine,
            MemeticConfig {
                elites: 2,
                steps: 3,
            },
            20,
            0,
        );
        let mem_found_zero = run_memetic(
            &prop,
            &fit_found,
            &NoScreen,
            &NoRefine,
            MemeticConfig {
                elites: 0,
                steps: 3,
            },
            20,
            0,
        );
        assert_eq!(
            mem_found_budget, base_found,
            "NoRefine (elites>0) == run_generic on a Found run"
        );
        assert_eq!(
            mem_found_zero, base_found,
            "elites:0 == run_generic on a Found run"
        );

        // A run that exhausts its budget (−100 is unreachable in 5 generations).
        let fit_exh = ToyFitness { target: -100 };
        let base_exh = run_generic(&prop, &fit_exh, &NoScreen, 5, 7);
        let mem_exh = run_memetic(
            &prop,
            &fit_exh,
            &NoScreen,
            &NoRefine,
            MemeticConfig {
                elites: 2,
                steps: 3,
            },
            5,
            7,
        );
        assert_eq!(
            mem_exh, base_exh,
            "NoRefine == run_generic on an Exhausted run"
        );
    }

    /// A screened-out **refiner-proposed** neighbor never reaches `score()` — the
    /// §2.4 screen contract holds for the refinement pass too, while an admissible
    /// neighbor is scored (proving the refiner path actually ran).
    #[test]
    fn refiner_neighbors_are_screened_before_score() {
        struct DownOrThirteen; // proposes elite−1 (admissible) and 13 (screened)
        impl Refiner<i64> for DownOrThirteen {
            fn neighbors(&self, elite: &i64, _rng: &mut SplitMix64) -> Vec<i64> {
                vec![*elite - 1, 13]
            }
        }
        let prop = ToyProposer {
            start: 50,
            population: 3,
        };
        let fit = SpyFitness {
            target: 42,
            seen: RefCell::new(Vec::new()),
        };
        let _ = run_memetic(
            &prop,
            &fit,
            &RejectThirteen,
            &DownOrThirteen,
            MemeticConfig {
                elites: 1,
                steps: 2,
            },
            3,
            0,
        )
        .expect("memetic run");
        let seen = fit.seen.into_inner();
        assert!(
            !seen.contains(&13),
            "screened refiner-proposed neighbor 13 must never be scored"
        );
        assert!(
            seen.contains(&49),
            "the admissible refiner neighbor 49 must be scored — the refiner path ran"
        );
    }

    /// Refinement never worsens the run: the engine keeps only strict
    /// improvements, so `run_memetic`'s best cost is `<=` the `NoRefine` baseline's
    /// at the same budget — even when the refiner also proposes worse neighbors.
    #[test]
    fn refinement_never_worsens_the_best() {
        struct BetterAndWorse; // one strict improvement, one regression
        impl Refiner<i64> for BetterAndWorse {
            fn neighbors(&self, elite: &i64, _rng: &mut SplitMix64) -> Vec<i64> {
                vec![*elite - 1, *elite + 5]
            }
        }
        let prop = ToyProposer {
            start: 50,
            population: 4,
        };
        let fit = ToyFitness { target: 42 };
        let cfg = MemeticConfig {
            elites: 4,
            steps: 2,
        };
        let (refined, _) =
            run_memetic(&prop, &fit, &NoScreen, &BetterAndWorse, cfg, 3, 0).expect("refined run");
        let (baseline, _) =
            run_memetic(&prop, &fit, &NoScreen, &NoRefine, cfg, 3, 0).expect("baseline run");
        assert!(
            best_cost(&refined) <= best_cost(&baseline),
            "refinement never worsens: refined best {} must be <= baseline best {}",
            best_cost(&refined),
            best_cost(&baseline),
        );
    }
}
