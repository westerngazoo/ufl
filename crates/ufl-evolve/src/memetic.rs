//! The memetic geometric engine (SPEC-0011M §2.3) — R-0011 Gate-1.
//!
//! The four lane pieces plugged into `ufl_search::run_memetic`: the NaN-safe
//! cost [`RotErr`], the tree-GA [`GeoProposer`] (SPEC-0011 §2.2), the
//! [`GeoFitness`] rotation-residual verifier (`√Σ coeff²` over all 16 blades —
//! **never** the metric-blind `Mv::norm()`, which zeros `e₀`-bearing blades),
//! and the Gate-1 τ/4-rotation task ([`gate1_fitness`]). The screen and refiner
//! live with the lane's types in `ufl-geo` ([`GradeScreen`]/[`GeoParamRefiner`]).
//!
//! [`GradeScreen`]: ufl_geo::GradeScreen
//! [`GeoParamRefiner`]: ufl_geo::GeoParamRefiner

use std::cmp::Ordering;
use std::f64::consts::TAU;

use ufl_geo::{eval, Env, GeoExpr, GeoLaneError, Mv};
use ufl_prng::SplitMix64;
use ufl_search::{Fitness, Proposer};

/// The rotation residual as a `Copy + Ord` **cost** (lower = better), total over
/// `f64`: non-finite residuals are the **maximal** cost (SPEC-0011 §2.3's
/// `Fit::WORST`, translated to the minimizing seam), so a NaN can never win a
/// sort or corrupt selection.
#[derive(Clone, Copy, Debug)]
pub struct RotErr(f64);

impl RotErr {
    /// The solve threshold: a mean residual at or below this is an exact
    /// rediscovery (the pilot's `1e-6` bar).
    pub const SOLVE_EPS: f64 = 1e-6;

    /// Build a cost from a raw residual; non-finite ⇒ maximal.
    pub fn new(residual: f64) -> Self {
        if residual.is_finite() {
            Self(residual)
        } else {
            Self(f64::INFINITY)
        }
    }

    /// The residual value (`INFINITY` for the non-finite class).
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl PartialEq for RotErr {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}
impl Eq for RotErr {}
impl PartialOrd for RotErr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RotErr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// `√Σ coeff²` over all 16 blade coefficients — the blade-complete magnitude.
/// **Not** `Mv::norm()`: the metric norm zeros `e₀`-bearing blades (the
/// SPEC-0010 trap), silently hiding error on the null generator.
pub fn magnitude(m: &Mv) -> f64 {
    m.coeffs
        .as_slice()
        .iter()
        .map(|c| c * c)
        .sum::<f64>()
        .sqrt()
}

/// The geometric verifier: cost = mean blade-complete residual of `eval(g, {v})`
/// against the target rotation over the test vectors (SPEC-0011M §2.3(iii)).
/// A structural failure (unbound var, bad leaf) is a typed [`GeoLaneError`];
/// a *badly-scoring but well-formed* genome is a high cost, never an `Err`.
pub struct GeoFitness {
    /// `(input, expected)` pairs: the bound `v` and the target `rot(v)`.
    cases: Vec<(Mv, Mv)>,
}

impl GeoFitness {
    /// A fitness over explicit `(v, rot(v))` cases.
    pub fn new(cases: Vec<(Mv, Mv)>) -> Self {
        Self { cases }
    }
}

impl Fitness<GeoExpr, RotErr> for GeoFitness {
    type Error = GeoLaneError;

    fn score(&self, genome: &GeoExpr) -> Result<RotErr, GeoLaneError> {
        let mut total = 0.0f64;
        for (input, expected) in &self.cases {
            let mut env = Env::new();
            env.bind("v", *input);
            let got = eval(genome, &env)?;
            let sample = magnitude(&(got - *expected));
            // NaN-safe readout (SPEC-0011 §2.3): a non-finite sample poisons the
            // genome's cost to maximal rather than entering the mean as NaN.
            if !sample.is_finite() {
                return Ok(RotErr::new(f64::INFINITY));
            }
            total += sample;
        }
        Ok(RotErr::new(total / self.cases.len() as f64))
    }

    fn solved(&self, score: &RotErr) -> bool {
        score.value() <= RotErr::SOLVE_EPS
    }
}

/// The Gate-1 task (SPEC-0011 §2.4): the τ/4 rotation in the `e₁e₂` plane,
/// `rotor = exp(e₁e₂ · (−τ/8))`, applied to the six probe vectors. The target is
/// a *general rotation of the input*, so a constant genome scores poorly and the
/// rotation structure is forced.
pub fn gate1_fitness() -> GeoFitness {
    let rotor = (Mv::basis(1) * Mv::basis(2) * Mv::scalar(-TAU / 8.0)).exp();
    let probes = [
        Mv::basis(1),
        Mv::basis(2),
        Mv::basis(1) + Mv::basis(2),
        Mv::basis(1) - Mv::basis(4),
        Mv::basis(2) + Mv::basis(4),
        Mv::basis(4),
    ];
    let cases = probes
        .into_iter()
        .map(|v| {
            let rotated = rotor.sandwich(&v);
            (v, rotated)
        })
        .collect();
    GeoFitness::new(cases)
}

/// The tree-GA over `GeoExpr` (SPEC-0011 §2.2): random depth-capped trees,
/// elitism + tournament selection, subtree crossover, subtree-replace mutation.
/// Answer-blind — it never sees the target or a score's meaning, only the
/// cost-sorted ranking the engine hands to `vary`.
pub struct GeoProposer {
    /// Population size (constant across generations).
    pub population: usize,
    /// Elites copied unchanged each generation.
    pub elitism: usize,
    /// Tournament size for parent selection.
    pub tournament: usize,
    /// Maximum tree depth at generation and mutation.
    pub max_depth: usize,
    /// Maximum node count per genome — the anti-bloat cap (SPEC-0011 §2.2; the
    /// pilot's load-bearing 60-node cap: unbounded crossover bloats trees until
    /// recursion overflows and eval hits garust's slow paths).
    pub max_nodes: usize,
}

impl GeoProposer {
    /// The pinned Gate-1 configuration (the pilot's robust budget shape).
    pub fn pinned(population: usize) -> Self {
        Self {
            population,
            elitism: 4,
            tournament: 5,
            max_depth: 4,
            max_nodes: 60,
        }
    }

    fn random_leaf(&self, rng: &mut SplitMix64) -> GeoExpr {
        match rng.below(3) {
            0 => GeoExpr::Param(rng.normal(0.0, 1.0).clamp(-3.0, 3.0)),
            1 => GeoExpr::Basis(rng.below(16) as u8),
            _ => GeoExpr::Var("v".to_string()),
        }
    }

    /// A random tree of depth ≤ `depth` — uniform over the internal forms (the
    /// SPEC-0011 §2.4 honesty guard: no `Sandwich` bias in the sampler).
    fn random_expr(&self, depth: usize, rng: &mut SplitMix64) -> GeoExpr {
        if depth == 0 || rng.below(4) == 0 {
            return self.random_leaf(rng);
        }
        match rng.below(8) {
            0 => GeoExpr::GradeLift(
                rng.below(5) as u8,
                Box::new(self.random_expr(depth - 1, rng)),
            ),
            1 => GeoExpr::GeoProduct(
                Box::new(self.random_expr(depth - 1, rng)),
                Box::new(self.random_expr(depth - 1, rng)),
            ),
            2 => GeoExpr::Wedge(
                Box::new(self.random_expr(depth - 1, rng)),
                Box::new(self.random_expr(depth - 1, rng)),
            ),
            3 => GeoExpr::Inner(
                Box::new(self.random_expr(depth - 1, rng)),
                Box::new(self.random_expr(depth - 1, rng)),
            ),
            4 => GeoExpr::Reverse(Box::new(self.random_expr(depth - 1, rng))),
            5 => GeoExpr::GradeProject(
                rng.below(5) as u8,
                Box::new(self.random_expr(depth - 1, rng)),
            ),
            6 => GeoExpr::Sandwich(
                Box::new(self.random_expr(depth - 1, rng)),
                Box::new(self.random_expr(depth - 1, rng)),
            ),
            _ => GeoExpr::Exp(Box::new(self.random_expr(depth - 1, rng))),
        }
    }

    fn tournament_pick<'a>(
        &self,
        ranked: &'a [(GeoExpr, RotErr)],
        rng: &mut SplitMix64,
    ) -> &'a GeoExpr {
        // `ranked` is cost-ascending, so the LOWEST drawn index wins the round.
        let mut best = ranked.len() - 1;
        for _ in 0..self.tournament {
            let i = rng.below(ranked.len() as u64) as usize;
            if i < best {
                best = i;
            }
        }
        &ranked[best].0
    }
}

/// The number of nodes in `e` (pre-order).
fn node_count(e: &GeoExpr) -> usize {
    match e {
        GeoExpr::Param(_) | GeoExpr::Basis(_) | GeoExpr::Var(_) => 1,
        GeoExpr::GradeLift(_, a)
        | GeoExpr::Reverse(a)
        | GeoExpr::GradeProject(_, a)
        | GeoExpr::Exp(a) => 1 + node_count(a),
        GeoExpr::GeoProduct(a, b)
        | GeoExpr::Wedge(a, b)
        | GeoExpr::Inner(a, b)
        | GeoExpr::Sandwich(a, b) => 1 + node_count(a) + node_count(b),
    }
}

/// The `idx`-th subtree of `e` in pre-order (0 = the whole tree).
fn nth_subtree(e: &GeoExpr, idx: usize) -> &GeoExpr {
    fn walk<'a>(e: &'a GeoExpr, idx: usize, seen: &mut usize) -> Option<&'a GeoExpr> {
        if *seen == idx {
            return Some(e);
        }
        *seen += 1;
        match e {
            GeoExpr::Param(_) | GeoExpr::Basis(_) | GeoExpr::Var(_) => None,
            GeoExpr::GradeLift(_, a)
            | GeoExpr::Reverse(a)
            | GeoExpr::GradeProject(_, a)
            | GeoExpr::Exp(a) => walk(a, idx, seen),
            GeoExpr::GeoProduct(a, b)
            | GeoExpr::Wedge(a, b)
            | GeoExpr::Inner(a, b)
            | GeoExpr::Sandwich(a, b) => walk(a, idx, seen).or_else(|| walk(b, idx, seen)),
        }
    }
    let mut seen = 0;
    // idx is always drawn `< node_count(e)`, so the walk finds it; the fallback
    // keeps this total rather than panicking.
    walk(e, idx, &mut seen).unwrap_or(e)
}

/// A copy of `e` with its `idx`-th pre-order subtree replaced by `new`.
fn replace_nth(e: &GeoExpr, idx: usize, new: &GeoExpr) -> GeoExpr {
    fn walk(e: &GeoExpr, idx: usize, seen: &mut usize, new: &GeoExpr) -> GeoExpr {
        if *seen == idx {
            *seen += 1;
            return new.clone();
        }
        *seen += 1;
        match e {
            GeoExpr::Param(_) | GeoExpr::Basis(_) | GeoExpr::Var(_) => e.clone(),
            GeoExpr::GradeLift(k, a) => GeoExpr::GradeLift(*k, Box::new(walk(a, idx, seen, new))),
            GeoExpr::Reverse(a) => GeoExpr::Reverse(Box::new(walk(a, idx, seen, new))),
            GeoExpr::GradeProject(k, a) => {
                GeoExpr::GradeProject(*k, Box::new(walk(a, idx, seen, new)))
            }
            GeoExpr::Exp(a) => GeoExpr::Exp(Box::new(walk(a, idx, seen, new))),
            GeoExpr::GeoProduct(a, b) => GeoExpr::GeoProduct(
                Box::new(walk(a, idx, seen, new)),
                Box::new(walk(b, idx, seen, new)),
            ),
            GeoExpr::Wedge(a, b) => GeoExpr::Wedge(
                Box::new(walk(a, idx, seen, new)),
                Box::new(walk(b, idx, seen, new)),
            ),
            GeoExpr::Inner(a, b) => GeoExpr::Inner(
                Box::new(walk(a, idx, seen, new)),
                Box::new(walk(b, idx, seen, new)),
            ),
            GeoExpr::Sandwich(a, b) => GeoExpr::Sandwich(
                Box::new(walk(a, idx, seen, new)),
                Box::new(walk(b, idx, seen, new)),
            ),
        }
    }
    let mut seen = 0;
    walk(e, idx, &mut seen, new)
}

impl Proposer<GeoExpr, RotErr> for GeoProposer {
    fn seed(&self, rng: &mut SplitMix64) -> Vec<GeoExpr> {
        (0..self.population)
            .map(|_| self.random_expr(self.max_depth, rng))
            .collect()
    }

    fn vary(&self, ranked: &[(GeoExpr, RotErr)], rng: &mut SplitMix64) -> Vec<GeoExpr> {
        let mut next: Vec<GeoExpr> = Vec::with_capacity(self.population);
        for (g, _) in ranked.iter().take(self.elitism) {
            next.push(g.clone());
        }
        while next.len() < self.population {
            let a = self.tournament_pick(ranked, rng);
            let b = self.tournament_pick(ranked, rng);
            // Subtree crossover: replace a random subtree of `a` with a random
            // subtree of `b`.
            let cut_a = rng.below(node_count(a) as u64) as usize;
            let cut_b = rng.below(node_count(b) as u64) as usize;
            let mut child = replace_nth(a, cut_a, nth_subtree(b, cut_b));
            // Point mutation: with probability 1/2, replace a random subtree
            // with a fresh random small tree.
            if rng.below(2) == 0 {
                let cut = rng.below(node_count(&child) as u64) as usize;
                let fresh = self.random_expr(2, rng);
                child = replace_nth(&child, cut, &fresh);
            }
            // The anti-bloat cap: an oversized child is replaced by a fresh
            // random tree rather than entering the population (pilot finding).
            if node_count(&child) > self.max_nodes {
                child = self.random_expr(self.max_depth, rng);
            }
            next.push(child);
        }
        next
    }
}
