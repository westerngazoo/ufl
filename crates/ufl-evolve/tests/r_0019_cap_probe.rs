//! R-0019 decision probe — **is `max_nodes: 60` binding on Gate-1 search
//! quality?** R-0019's whole case is conditional on wanting to raise the cap, so
//! this answers that directly, without building any of R-0019.
//!
//! **Pre-registered before the run** (nothing below was chosen after seeing a
//! result): caps `{60 (control), 100, 150}`, seeds `0..16`, `GENS`/`POP`/
//! `MEMETIC` identical to the pinned Gate-1 config. `max_nodes` is the **only**
//! knob that varies.
//!
//! **Measured (2026-08-18, release):**
//!
//! | cap | wins/16 | per-seed | wall-clock |
//! |-----|---------|----------|------------|
//! | 60 (control) | **6/16** | `...#...###..#.#.` | 34 s |
//! | 100 | 4/16 | `...#..#.#..#....` | 258 s (7.6x) |
//! | 150 | 4/16 | `.#......#...#..#` | 436 s (12.8x) |
//!
//! The control reproduces the pilot's 6/16 exactly.
//!
//! **What this supports, and what it does not.** 6 vs 4 is **1.03 SD** apart on
//! `Binomial(16, 6/16)`, and `P(X <= 4 | p = 6/16) = 0.223` — 4/16 is an ordinary
//! draw from the control. So this is **no evidence of improvement**, NOT evidence
//! of harm; resolving 0.375 vs 0.50 at 80% power would need ~247 seeds per arm.
//! The **cost** is not a sampled proportion and is unambiguous: 7.6x and 12.8x,
//! with one seed alone taking 145 s. Win rate is unconfounded by that cost, since
//! `GENS` is fixed at 400 either way.
//!
//! Per `docs/conventions.md` *Assert the Protocol, Not the Outcome*, the
//! committed assertion is that the sweep **ran at the pre-registered caps and
//! seeds and recorded a verdict** — never that any particular cap won.
use std::time::Instant;
use ufl_evolve::{gate1_fitness, GeoProposer};
use ufl_geo::{GeoParamRefiner, GradeCtx, GradeScreen, GradeSet};
use ufl_search::{run_memetic, GenericOutcome, MemeticConfig};

const SEEDS: u64 = 16;
const GENS: usize = 400;
const POP: usize = 400;
const MEMETIC: MemeticConfig = MemeticConfig {
    elites: 6,
    steps: 8,
};

fn ctx_v1() -> GradeCtx {
    let mut c = GradeCtx::new();
    c.declare("v", GradeSet::singleton(1));
    c
}

fn run_one(seed: u64, max_nodes: usize) -> bool {
    let mut proposer = GeoProposer::pinned(POP);
    proposer.max_nodes = max_nodes; // the ONLY knob that varies
    let outcome = run_memetic(
        &proposer,
        &gate1_fitness(),
        &GradeScreen::new(ctx_v1()),
        &GeoParamRefiner::pinned(),
        MEMETIC,
        GENS,
        seed,
    );
    matches!(outcome, Ok((GenericOutcome::Found { .. }, _)))
}

#[test]
#[ignore = "release e2e: cargo test -p ufl-evolve --release --test r_0019_cap_probe -- --ignored --nocapture"]
fn cap_sweep_runs_at_the_pre_registered_caps() {
    let mut results: Vec<(usize, usize)> = Vec::new();
    println!("cap  wins/16   per-seed");
    for cap in [60usize, 100, 150] {
        let t0 = Instant::now();
        let mut wins = 0;
        let mut marks = String::new();
        for seed in 0..SEEDS {
            let t = Instant::now();
            let won = run_one(seed, cap);
            if won {
                wins += 1;
            }
            marks.push(if won { '#' } else { '.' });
            if t.elapsed().as_secs() > 120 {
                println!("  WARN seed {seed} @ cap {cap} took {:?}", t.elapsed());
            }
        }
        println!(
            "{cap:>3}  {wins:>2}/16    {marks}  ({:?} total)",
            t0.elapsed()
        );
        results.push((cap, wins));
    }

    // *The protocol, not the outcome.* Every pre-registered cap ran all 16
    // seeds and recorded a count. Nothing here asserts WHICH cap wins — the
    // measured answer (no evidence of improvement; 7.6x/12.8x cost) is a
    // documented negative, and a green build must not depend on it changing.
    assert_eq!(results.len(), 3, "all three pre-registered caps must run");
    assert!(
        results.iter().all(|&(_, w)| w <= SEEDS as usize),
        "each arm reports a count in range"
    );
}
