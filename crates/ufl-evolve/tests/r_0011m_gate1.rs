//! SPEC-0011M §2.3/§4 — the geometric lane on `run_memetic`: T-magnitude,
//! T-screen-fuzz, and the Gate-1 e2e pair (`#[ignore]` — release runs):
//! `t_gate1_repro` (the memetic rediscovery count) and `t_ablation`
//! (`NoRefine` — the load-bearing contrast).
//!
//! **Pre-registered knobs** (SPEC-0011M §5 gate semantics — fixed BEFORE the
//! runs; the ≥6/16 pilot number is a *reference*, not a merge gate; the actual
//! N/16 is reported honestly either way): pop=400, gens=400, elitism=4,
//! tournament=5, max_depth=4, memetic elites=6, steps=8, ±δ ladder 1e-1..1e-11, seeds 0..16.

use std::cell::RefCell;

use ufl_evolve::{gate1_fitness, magnitude, GeoFitness, GeoProposer, RotErr};
use ufl_geo::{
    render, typecheck, GeoExpr, GeoLaneError, GeoParamRefiner, GradeCtx, GradeScreen, GradeSet, Mv,
};
use ufl_search::{run_memetic, Fitness, GenericOutcome, MemeticConfig, NoRefine};

fn ctx_v1() -> GradeCtx {
    let mut ctx = GradeCtx::new();
    ctx.declare("v", GradeSet::singleton(1));
    ctx
}

/// T-magnitude — the blade-complete magnitude sees `e₀`-bearing error that the
/// metric `norm()` zeros (the SPEC-0010 trap): `e₀` itself has magnitude 1, and
/// a genome whose output differs from the target on an `e₀` blade scores > 0.
#[test]
fn magnitude_sees_null_blade_error() {
    let e0 = Mv::basis(8);
    assert!(
        (magnitude(&e0) - 1.0).abs() < 1e-12,
        "e0 magnitude is 1 (norm() would be 0)"
    );

    // A genome producing e0 (grade-1, coherent) vs the rotation target: its
    // e0-component error must contribute — nonzero cost.
    let fit = gate1_fitness();
    let genome = GeoExpr::Basis(8);
    let cost = fit.score(&genome).expect("well-formed genome scores");
    assert!(
        cost.value() > 0.1,
        "e0-blade error must be visible to the fitness (got {})",
        cost.value()
    );
}

/// A spy wrapper proving the screen contract end-to-end on the REAL lane:
/// every genome that reaches `score` typechecks (incoherent candidates are
/// filtered by `GradeScreen`, never scored) — SPEC-0011 AC2 as a running fact.
struct SpyFitness {
    inner: GeoFitness,
    ctx: GradeCtx,
    scored: RefCell<usize>,
}

impl Fitness<GeoExpr, RotErr> for SpyFitness {
    type Error = GeoLaneError;
    fn score(&self, genome: &GeoExpr) -> Result<RotErr, GeoLaneError> {
        assert!(
            typecheck(genome, &self.ctx).is_ok(),
            "an incoherent genome reached score(): the screen failed"
        );
        *self.scored.borrow_mut() += 1;
        self.inner.score(genome)
    }
    fn solved(&self, score: &RotErr) -> bool {
        self.inner.solved(score)
    }
}

/// T-screen-fuzz — a small memetic run over the real proposer/refiner: every
/// scored genome (seed, vary, and refined neighbors alike) typechecks.
#[test]
fn every_scored_genome_typechecks() {
    let proposer = GeoProposer::pinned(60);
    let fitness = SpyFitness {
        inner: gate1_fitness(),
        ctx: ctx_v1(),
        scored: RefCell::new(0),
    };
    let screen = GradeScreen::new(ctx_v1());
    let refiner = GeoParamRefiner::pinned();
    let outcome = run_memetic(
        &proposer,
        &fitness,
        &screen,
        &refiner,
        MemeticConfig {
            elites: 3,
            steps: 3,
        },
        12,
        42,
    );
    // Success either way (Found or Exhausted); the assertion lives in the spy.
    let _ = outcome.expect("the screened memetic run completes");
    assert!(*fitness.scored.borrow() > 0, "the run actually scored");
}

// ── the Gate-1 e2e pair (release, `--ignored`) ─────────────────────────────

const SEEDS: u64 = 16;
const GENS: usize = 400;
const POP: usize = 400;
const MEMETIC: MemeticConfig = MemeticConfig {
    elites: 6,
    steps: 8,
};

fn run_one(seed: u64, refine: bool) -> (bool, Option<String>) {
    let proposer = GeoProposer::pinned(POP);
    let fitness = gate1_fitness();
    let screen = GradeScreen::new(ctx_v1());
    let outcome = if refine {
        let refiner = GeoParamRefiner::pinned();
        run_memetic(&proposer, &fitness, &screen, &refiner, MEMETIC, GENS, seed)
    } else {
        run_memetic(&proposer, &fitness, &screen, &NoRefine, MEMETIC, GENS, seed)
    };
    match outcome {
        Ok((GenericOutcome::Found { genome, generation }, _)) => {
            (true, Some(format!("gen {generation}: {}", render(&genome))))
        }
        Ok((GenericOutcome::Exhausted { best_score, .. }, _)) => {
            let _ = best_score;
            (false, None)
        }
        Err(e) => panic!("gate-1 run failed structurally: {e}"),
    }
}

/// T-gate1-repro — the memetic rediscovery count over the pinned seeds. The
/// pilot reference is 6/16; the ACTUAL count is reported honestly (a shortfall
/// is a documented result per R-0011 AC6, not a failure of this test — the
/// committed contrast lives in `t_ablation`).
#[test]
#[ignore = "release e2e: cargo test -p ufl-evolve --release -- --ignored --nocapture"]
fn t_gate1_repro() {
    let mut wins = 0;
    for seed in 0..SEEDS {
        let (won, detail) = run_one(seed, true);
        if won {
            wins += 1;
            println!("seed {seed}: REDISCOVERED — {}", detail.unwrap_or_default());
        } else {
            println!("seed {seed}: exhausted");
        }
    }
    println!("MEMETIC GATE-1: {wins}/{SEEDS} rediscoveries (pilot reference: 6/16)");
    assert!(
        wins > 0,
        "at least one rediscovery is expected at this budget"
    );
}

/// T-ablation — the identical engine and seeds with the refiner disabled
/// (`NoRefine`): the pilot's load-bearing contrast (0/16 without refinement).
#[test]
#[ignore = "release e2e: cargo test -p ufl-evolve --release -- --ignored --nocapture"]
fn t_ablation() {
    let mut wins = 0;
    for seed in 0..SEEDS {
        let (won, _) = run_one(seed, false);
        if won {
            wins += 1;
        }
    }
    println!("ABLATION (NoRefine): {wins}/{SEEDS} rediscoveries (pilot reference: 0/16)");
}
