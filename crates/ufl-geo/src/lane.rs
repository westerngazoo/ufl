//! The geometric lane's search-seam instances (SPEC-0011M §2.3, §9.1):
//! the lane error, the answer-blind grade [`Screen`], and the slot-refining
//! [`Refiner`]. The lane owns these — they are pure functions of the
//! `Cl(3,0,1)` algebra and `GeoExpr` structure, carrying **no task data**.

use ufl_prng::SplitMix64;
use ufl_search::{Refiner, Screen};

use crate::eval::GeoError;
use crate::expr::GeoExpr;
use crate::grade::{typecheck, GradeCtx, GradeError};
use crate::slots::{params, params_mut};

/// The geometric lane's structural-failure sum (SPEC-0014 §2.2): an evaluation
/// failure or a grade-typing failure, kept distinct so the substrate's generic
/// error channel carries a real multi-source lane error.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GeoLaneError {
    /// The kernel evaluation failed (unbound variable, bad blade/grade leaf).
    #[error(transparent)]
    Eval(#[from] GeoError),
    /// The grade type system rejected the expression.
    #[error(transparent)]
    Grade(#[from] GradeError),
}

/// The answer-blind coherence screen: admits a candidate iff it grade-typechecks
/// (SPEC-0011 AC2 as an architectural fact — the harness's first real consumer).
///
/// **Answer-blind construction (SPEC-0014 §2.4):** the [`GradeCtx`] declares only
/// the *input* variables' grades (public from the task signature, e.g. `v : {1}`)
/// — never anything target-derived. Grade coherence is a property of the algebra
/// plus the input grades, not of the target.
pub struct GradeScreen {
    ctx: GradeCtx,
}

impl GradeScreen {
    /// A screen over the given input-grade context.
    pub fn new(ctx: GradeCtx) -> Self {
        Self { ctx }
    }
}

impl Screen<GeoExpr> for GradeScreen {
    fn admissible(&self, genome: &GeoExpr) -> bool {
        typecheck(genome, &self.ctx).is_ok()
    }
}

/// The memetic step: perturbs grade-`{0}` `Param` slots and **nothing else** —
/// structure-blind by construction (it writes only through [`params_mut`]), so
/// every neighbor has the same `typecheck` verdict as its elite, and answer-blind
/// by signature. The neighborhood is a **±δ geometric ladder** per slot
/// (δ = 10⁻¹ … 10⁻¹¹) — the pilot's load-bearing finding: a fixed-scale jitter
/// plateaus above the 10⁻⁶ solve bar; the multi-scale ladder lets the engine's
/// hill-climb line-search a slot down to exactness. Draws **no rng**, so a
/// refined run shares the ablation run's exact `vary` stream — the contrast
/// isolates refinement alone (SPEC-0011M §2.3(v)).
pub struct GeoParamRefiner {
    ladder: Vec<f64>,
}

impl GeoParamRefiner {
    /// The pinned ±δ ladder: `10⁻¹ … 10⁻¹¹` (the pilot's fine line-search).
    pub fn pinned() -> Self {
        Self {
            ladder: (1..=11).map(|k| 10f64.powi(-k)).collect(),
        }
    }
}

impl Refiner<GeoExpr> for GeoParamRefiner {
    fn neighbors(&self, elite: &GeoExpr, _rng: &mut SplitMix64) -> Vec<GeoExpr> {
        let slot_count = params(elite).len();
        if slot_count == 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(slot_count * self.ladder.len() * 2);
        for i in 0..slot_count {
            for &delta in &self.ladder {
                for sign in [delta, -delta] {
                    let mut n = elite.clone();
                    if let Some(slot) = params_mut(&mut n).into_iter().nth(i) {
                        *slot += sign;
                    }
                    out.push(n);
                }
            }
        }
        out
    }
}
