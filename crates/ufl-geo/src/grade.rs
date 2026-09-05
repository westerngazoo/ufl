//! The decidable grade-type system (SPEC-0010 §2.3–§2.5).
//!
//! `grade` is a **sound over-approximation** of the result grades — every grade
//! a value can carry is in the set; under the degenerate metric the set may be a
//! strict superset of the realized support. The catalog forms delegate to
//! garust's `Op::output_grades`; only `Sandwich`/`Exp`/`GradeLift` are hand-ruled.

use std::collections::HashMap;
use ufl_ga::{GradeSet, Op};

use crate::expr::GeoExpr;

/// The number of `Cl(3,0,1)` generators (grades range over `0..=4`).
const N: usize = 4;

/// Grade context: input variables declared with their grade set
/// (⊤ = `full(4)` if undeclared).
#[derive(Clone, Debug, Default)]
pub struct GradeCtx {
    vars: HashMap<String, GradeSet>,
}

impl GradeCtx {
    /// An empty context (every `Var` is ⊤ until declared).
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a variable's grade set.
    pub fn declare(&mut self, name: impl Into<String>, grades: GradeSet) {
        self.vars.insert(name.into(), grades);
    }

    /// The declared grade of a variable, or ⊤ (`full(4)`) if undeclared.
    pub(crate) fn get(&self, name: &str) -> GradeSet {
        self.vars
            .get(name)
            .copied()
            .unwrap_or_else(|| GradeSet::full(N))
    }
}

/// A grade-type failure (the decidable pruning signal R-0011 uses).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GradeError {
    /// A grade-incoherent form — its grade set is `∅` (e.g. `GradeProject(k, a)`
    /// with `k ∉ grade(a)`); it can only ever be zero.
    #[error("grade-incoherent form (empty grade set)")]
    Incoherent(GeoExpr),
    /// A `Basis(i)` with `i ≥ 16`.
    #[error("blade index {0} out of range")]
    BadBlade(u8),
    /// A `GradeLift`/`GradeProject` grade `> 4`.
    #[error("grade {0} out of range")]
    BadGrade(u8),
}

/// A conservative, sound static versor predicate (SPEC-0010 §2.4) — `true` only
/// when `r` is *provably* a versor, so `Sandwich(r, ·)` preserves grade. It may
/// say `false` for a real versor (the grade rule then falls back to the safe
/// product bound — still sound); it never says `true` for a non-versor.
///
/// Versor witnesses: `Exp(b)` of a pure bivector (`grade(b) ⊆ {2}` — a rotor or
/// motor), a single basis *vector* (`Basis(i)`, one set bit), a `GeoProduct` of
/// versors, and `Reverse` of a versor.
pub(crate) fn is_versor(r: &GeoExpr, ctx: &GradeCtx) -> bool {
    match r {
        GeoExpr::Exp(b) => subset_of(grade(b, ctx), &[2]),
        GeoExpr::GeoProduct(a, b) => is_versor(a, ctx) && is_versor(b, ctx),
        GeoExpr::Basis(i) => *i < 16 && i.count_ones() == 1,
        GeoExpr::Reverse(a) => is_versor(a, ctx),
        _ => false,
    }
}

/// Is every grade in `g` one of `allowed`? (`g ⊆ allowed`.)
fn subset_of(g: GradeSet, allowed: &[usize]) -> bool {
    g.iter().all(|k| allowed.contains(&k))
}

/// Infer a **sound over-approximation** of a form's result grades (SPEC-0010
/// §2.3). The catalog forms delegate to garust's `Op::output_grades` (the
/// correct structural signature); only `Sandwich`/`Exp`/`GradeLift` are
/// hand-ruled. Total and decidable — out-of-range leaves return `⊤ = full(4)`
/// (it is `typecheck` that turns those into errors).
pub fn grade(e: &GeoExpr, ctx: &GradeCtx) -> GradeSet {
    match e {
        GeoExpr::Param(_) => GradeSet::singleton(0),
        GeoExpr::Basis(i) => {
            if *i >= 16 {
                GradeSet::full(N)
            } else {
                GradeSet::singleton(i.count_ones() as usize)
            }
        }
        GeoExpr::Var(name) => ctx.get(name),
        GeoExpr::GradeLift(k, _) => {
            if *k > 4 {
                GradeSet::full(N)
            } else {
                GradeSet::singleton(*k as usize)
            }
        }
        GeoExpr::GeoProduct(a, b) => {
            Op::Geometric.output_grades(&[grade(a, ctx), grade(b, ctx)], N)
        }
        GeoExpr::Wedge(a, b) => Op::Wedge.output_grades(&[grade(a, ctx), grade(b, ctx)], N),
        GeoExpr::Inner(a, b) => Op::Inner.output_grades(&[grade(a, ctx), grade(b, ctx)], N),
        GeoExpr::Reverse(a) => Op::Reverse.output_grades(&[grade(a, ctx)], N),
        GeoExpr::GradeProject(k, a) => {
            // Guard the raw `u8` before garust (its `singleton(k) = 1 << k`
            // overflows `u32` for `k ≥ 32`): projecting onto a grade the algebra
            // lacks (`k > 4`) is the empty set. Keeps `grade` total (SPEC-0010 §2.3).
            if *k > 4 {
                GradeSet::EMPTY
            } else {
                Op::GradeProject(*k).output_grades(&[grade(a, ctx)], N)
            }
        }
        GeoExpr::Sandwich(r, x) => {
            if is_versor(r, ctx) {
                // A versor sandwich preserves the operand's grade.
                grade(x, ctx)
            } else {
                // The sound product bound: grades of `(r ∗ x) ∗ r` (reverse
                // preserves grade, so `~r` carries the same grades as `r`).
                let rg = grade(r, ctx);
                let rx = Op::Geometric.output_grades(&[rg, grade(x, ctx)], N);
                Op::Geometric.output_grades(&[rx, rg], N)
            }
        }
        GeoExpr::Exp(a) => {
            let g = grade(a, ctx);
            if subset_of(g, &[0]) {
                GradeSet::singleton(0)
            } else if subset_of(g, &[0, 2]) {
                // exp of an even element is even — covers rotors and motors.
                GradeSet::EMPTY.with(0).with(2).with(4)
            } else {
                GradeSet::full(N)
            }
        }
    }
}

/// Infer the grade set, or fail on a grade-incoherent / out-of-range form
/// (SPEC-0010 §2.5). Recursively checks every sub-form: an out-of-range
/// `Basis`/grade leaf is a `BadBlade`/`BadGrade`, and a sub-form whose grade set
/// is `∅` (e.g. `GradeProject(k, a)` with `k ∉ grade(a)`) is `Incoherent` — it
/// can only ever be zero. The decidable pruning signal R-0011 reads.
pub fn typecheck(e: &GeoExpr, ctx: &GradeCtx) -> Result<GradeSet, GradeError> {
    match e {
        GeoExpr::Param(_) | GeoExpr::Var(_) => {}
        GeoExpr::Basis(i) => {
            if *i >= 16 {
                return Err(GradeError::BadBlade(*i));
            }
        }
        GeoExpr::GradeLift(k, a) | GeoExpr::GradeProject(k, a) => {
            if *k > 4 {
                return Err(GradeError::BadGrade(*k));
            }
            typecheck(a, ctx)?;
        }
        GeoExpr::Reverse(a) | GeoExpr::Exp(a) => {
            typecheck(a, ctx)?;
        }
        GeoExpr::GeoProduct(a, b)
        | GeoExpr::Wedge(a, b)
        | GeoExpr::Inner(a, b)
        | GeoExpr::Sandwich(a, b) => {
            typecheck(a, ctx)?;
            typecheck(b, ctx)?;
        }
    }
    let g = grade(e, ctx);
    if g.is_empty() {
        Err(GradeError::Incoherent(e.clone()))
    } else {
        Ok(g)
    }
}

#[cfg(test)]
mod tests {
    //! SPEC-0020 §4.2–§4.4 — the tests that need private items (`analyse`,
    //! `check`, `VISITS`), so they live inside the module.
    //!
    //! # RED (loop step 3)
    //!
    //! Nothing here compiles until R-0020 is implemented. It assumes, per
    //! SPEC-0020 §2.1–§2.3 and §4.3, that the parent module provides
    //!
    //! ```text
    //! struct Analysis { grade: GradeSet, versor: bool }
    //! fn analyse(e: &GeoExpr, ctx: &GradeCtx) -> Analysis
    //! fn check(e: &GeoExpr, ctx: &GradeCtx) -> Result<Analysis, GradeError>
    //! #[cfg(test)]
    //! thread_local! { static VISITS: Cell<usize> = const { Cell::new(0) }; }
    //! ```
    //!
    //! with `VISITS.with(|c| c.set(c.get() + 1))` as the first statement of
    //! both `analyse` and `check`, each under `#[cfg(test)]`. The counter is
    //! `thread_local!` so it is stable under `cargo test`'s parallel threads.
    //!
    //! # The oracle
    //!
    //! `old_grade` / `old_is_versor` / `old_subset_of` are **verbatim**
    //! transcriptions of the pre-change `grade.rs:66-144` (commit `a9cbee9`),
    //! kept here so the differential survives the deletion of the code it
    //! checks against. They read `GradeCtx::get` directly, so the oracle never
    //! touches the code under test.

    use std::cell::Cell;

    use ufl_ga::{GradeSet, Mv, Op};
    use ufl_prng::SplitMix64;

    use super::{analyse, check, GradeCtx, N, VISITS};
    use crate::{eval, Env, GeoExpr};

    // ── the pre-change oracle (verbatim, `a9cbee9`) ────────────────────────

    /// Verbatim pre-change `is_versor` (`grade.rs:66-74`).
    fn old_is_versor(r: &GeoExpr, ctx: &GradeCtx) -> bool {
        match r {
            GeoExpr::Exp(b) => old_subset_of(old_grade(b, ctx), &[2]),
            GeoExpr::GeoProduct(a, b) => old_is_versor(a, ctx) && old_is_versor(b, ctx),
            GeoExpr::Basis(i) => *i < 16 && i.count_ones() == 1,
            GeoExpr::Reverse(a) => old_is_versor(a, ctx),
            _ => false,
        }
    }

    /// Verbatim pre-change `subset_of` (`grade.rs:77-79`).
    fn old_subset_of(g: GradeSet, allowed: &[usize]) -> bool {
        g.iter().all(|k| allowed.contains(&k))
    }

    /// Verbatim pre-change `grade` (`grade.rs:86-144`).
    fn old_grade(e: &GeoExpr, ctx: &GradeCtx) -> GradeSet {
        match e {
            GeoExpr::Param(_) => GradeSet::singleton(0),
            GeoExpr::Basis(i) => {
                if *i >= 16 {
                    GradeSet::full(N)
                } else {
                    GradeSet::singleton(i.count_ones() as usize)
                }
            }
            GeoExpr::Var(name) => ctx.get(name),
            GeoExpr::GradeLift(k, _) => {
                if *k > 4 {
                    GradeSet::full(N)
                } else {
                    GradeSet::singleton(*k as usize)
                }
            }
            GeoExpr::GeoProduct(a, b) => {
                Op::Geometric.output_grades(&[old_grade(a, ctx), old_grade(b, ctx)], N)
            }
            GeoExpr::Wedge(a, b) => {
                Op::Wedge.output_grades(&[old_grade(a, ctx), old_grade(b, ctx)], N)
            }
            GeoExpr::Inner(a, b) => {
                Op::Inner.output_grades(&[old_grade(a, ctx), old_grade(b, ctx)], N)
            }
            GeoExpr::Reverse(a) => Op::Reverse.output_grades(&[old_grade(a, ctx)], N),
            GeoExpr::GradeProject(k, a) => {
                if *k > 4 {
                    GradeSet::EMPTY
                } else {
                    Op::GradeProject(*k).output_grades(&[old_grade(a, ctx)], N)
                }
            }
            GeoExpr::Sandwich(r, x) => {
                if old_is_versor(r, ctx) {
                    old_grade(x, ctx)
                } else {
                    let rg = old_grade(r, ctx);
                    let rx = Op::Geometric.output_grades(&[rg, old_grade(x, ctx)], N);
                    Op::Geometric.output_grades(&[rx, rg], N)
                }
            }
            GeoExpr::Exp(a) => {
                let g = old_grade(a, ctx);
                if old_subset_of(g, &[0]) {
                    GradeSet::singleton(0)
                } else if old_subset_of(g, &[0, 2]) {
                    GradeSet::EMPTY.with(0).with(2).with(4)
                } else {
                    GradeSet::full(N)
                }
            }
        }
    }

    // ── fixtures (mirror SPEC-0020 §8's harness; duplicated in
    //    `tests/r_0020_acceptance.rs`, which cannot see this module) ───────

    /// `v: {1}`, `w: {0, 2}`, `z: ∅`; `u` undeclared (⊤).
    fn ctx_all() -> GradeCtx {
        let mut ctx = GradeCtx::new();
        ctx.declare("v", GradeSet::singleton(1));
        ctx.declare("w", GradeSet::EMPTY.with(0).with(2));
        ctx.declare("z", GradeSet::EMPTY);
        ctx
    }

    /// Bindings consistent with [`ctx_all`]: `v` a vector, `w` even, `z` zero,
    /// `u` (⊤) a mixed-grade element with an `e₀` part.
    fn env_all() -> Env {
        let mut env = Env::new();
        env.bind("v", Mv::basis(1));
        env.bind("w", Mv::scalar(1.0) + Mv::basis(3) * 0.5);
        env.bind("z", Mv::zero());
        env.bind(
            "u",
            Mv::scalar(1.0) + Mv::basis(1) + Mv::basis(3) + Mv::basis(7) + Mv::basis(8),
        );
        env
    }

    fn bx(e: GeoExpr) -> Box<GeoExpr> {
        Box::new(e)
    }

    /// `Exp(Param(0.5) ∗ e12)` — a provable versor (grade `{2}` operand).
    fn rotor() -> GeoExpr {
        GeoExpr::Exp(bx(GeoExpr::GeoProduct(
            bx(GeoExpr::Param(0.5)),
            bx(GeoExpr::Basis(3)),
        )))
    }

    /// `Exp(e12 ∗ e12)` — a `{0, 2, 4}` operand, so *not* a provable versor.
    fn non_versor_rotor() -> GeoExpr {
        GeoExpr::Exp(bx(GeoExpr::GeoProduct(
            bx(GeoExpr::Basis(3)),
            bx(GeoExpr::Basis(3)),
        )))
    }

    /// (A) the motor chain — `Sandwich(rotor, ·)` nested `k` deep in the
    /// *operand*, over `Var v` (R-0020 §3). `5k + 1` nodes.
    fn shape_a(k: usize) -> GeoExpr {
        let mut e = GeoExpr::Var("v".into());
        for _ in 0..k {
            e = GeoExpr::Sandwich(bx(rotor()), bx(e));
        }
        e
    }

    /// (B) as (A) with a non-versor rotor — the product bound at every level.
    fn shape_b(k: usize) -> GeoExpr {
        let mut e = GeoExpr::Var("v".into());
        for _ in 0..k {
            e = GeoExpr::Sandwich(bx(non_versor_rotor()), bx(e));
        }
        e
    }

    /// (C) rotor-nested — `bₖ₊₁ = Sandwich(Exp(bₖ), Basis(1))`, `b₀ = Basis(1)`
    /// (R-0020 §1): the shape the old `grade` was 2^k on. `3k + 1` nodes.
    fn shape_c(k: usize) -> GeoExpr {
        let mut b = GeoExpr::Basis(1);
        for _ in 0..k {
            b = GeoExpr::Sandwich(bx(GeoExpr::Exp(bx(b))), bx(GeoExpr::Basis(1)));
        }
        b
    }

    /// A random `GeoExpr` over all 11 variants: `Basis` 0..=255 (half the leaf
    /// draws in range), `k ∈ {0..=5, 31, 32, 255}`, `Var`s over `v`/`w`/`z`/`u`,
    /// `|Param| ≤ 1` so `exp` stays finite on small trees.
    fn rand_tree(rng: &mut SplitMix64, depth: usize) -> GeoExpr {
        const KS: [u8; 9] = [0, 1, 2, 3, 4, 5, 31, 32, 255];
        const PARAMS: [f64; 4] = [-1.0, -0.5, 0.5, 1.0];
        const VARS: [&str; 4] = ["v", "w", "z", "u"];
        if depth == 0 || rng.below(4) == 0 {
            return match rng.below(4) {
                0 => GeoExpr::Param(PARAMS[rng.below(4) as usize]),
                1 => GeoExpr::Basis(rng.below(256) as u8),
                2 => GeoExpr::Var(VARS[rng.below(4) as usize].into()),
                _ => GeoExpr::Basis(rng.below(16) as u8),
            };
        }
        let k = KS[rng.below(9) as usize];
        let kind = rng.below(8);
        let a = bx(rand_tree(rng, depth - 1));
        match kind {
            0 => GeoExpr::GradeLift(k, a),
            1 => GeoExpr::GeoProduct(a, bx(rand_tree(rng, depth - 1))),
            2 => GeoExpr::Wedge(a, bx(rand_tree(rng, depth - 1))),
            3 => GeoExpr::Inner(a, bx(rand_tree(rng, depth - 1))),
            4 => GeoExpr::Reverse(a),
            5 => GeoExpr::GradeProject(k, a),
            6 => GeoExpr::Sandwich(a, bx(rand_tree(rng, depth - 1))),
            _ => GeoExpr::Exp(a),
        }
    }

    /// The children of a node, left to right (test-side; independent of the
    /// product's arity source).
    fn children_of(e: &GeoExpr) -> Vec<&GeoExpr> {
        match e {
            GeoExpr::Param(_) | GeoExpr::Basis(_) | GeoExpr::Var(_) => vec![],
            GeoExpr::GradeLift(_, a)
            | GeoExpr::Reverse(a)
            | GeoExpr::GradeProject(_, a)
            | GeoExpr::Exp(a) => vec![a],
            GeoExpr::GeoProduct(a, b)
            | GeoExpr::Wedge(a, b)
            | GeoExpr::Inner(a, b)
            | GeoExpr::Sandwich(a, b) => vec![a, b],
        }
    }

    /// Every subtree of `e`, pre-order (including `e`).
    fn subtrees(e: &GeoExpr) -> Vec<&GeoExpr> {
        let mut out = vec![e];
        for c in children_of(e) {
            out.extend(subtrees(c));
        }
        out
    }

    fn node_count(e: &GeoExpr) -> usize {
        subtrees(e).len()
    }

    /// Reset `VISITS`, run `f`, and report how many entries it made.
    fn visits_during(f: impl FnOnce()) -> usize {
        VISITS.with(|c| c.set(0));
        f();
        VISITS.with(Cell::get)
    }

    /// The entries `check` makes on `e` under SPEC-0020 §2.3's precedence —
    /// own `BadBlade`/`BadGrade` before descent, children left to right
    /// stopping at the first error, own `Incoherent` after — i.e. exactly the
    /// nodes reached before the first error, and every node on an `Ok` tree.
    /// Returns `(entries, ok)`; the grade oracle is `old_grade`.
    fn expected_check_visits(e: &GeoExpr, ctx: &GradeCtx) -> (usize, bool) {
        match e {
            GeoExpr::Basis(i) if *i >= 16 => return (1, false),
            GeoExpr::GradeLift(k, _) | GeoExpr::GradeProject(k, _) if *k > 4 => return (1, false),
            _ => {}
        }
        let mut entries = 1;
        for c in children_of(e) {
            let (n, ok) = expected_check_visits(c, ctx);
            entries += n;
            if !ok {
                return (entries, false);
            }
        }
        (entries, !old_grade(e, ctx).is_empty())
    }

    /// The grades an `Mv` actually carries, after cleaning to `tol` (the
    /// `r_0010_soundness.rs` notion).
    fn realized(mv: &Mv, tol: f64) -> GradeSet {
        let mut g = GradeSet::EMPTY;
        for k in 0..=4usize {
            if mv.grade(k).cleaned(tol) != Mv::zero() {
                g = g.with(k);
            }
        }
        g
    }

    fn max_abs(mv: &Mv) -> f64 {
        mv.coeffs.iter().fold(0.0, |m, c| m.max(c.abs()))
    }

    // ── T-visit-ladder (AC1, AC2; §4.3) ────────────────────────────────────

    /// AC1 + AC2 — the mechanism, by counter, not clock. Both drivers enter
    /// every node exactly once (`c = 1`, SPEC-0020 §2.3) on all three shapes at
    /// k ∈ {10, 20, 40, 64}, **ascending in one test**: a re-walk fails at
    /// k = 10 (31–51 nodes) and never reaches k = 64, so a regression cannot
    /// hang the suite. (C) alone had a hole — an operand re-walk passes (C)
    /// and is 2^k on (A); the ladder over all three closes it.
    #[test]
    fn t_visit_ladder_ac1_ac2() {
        let ctx = ctx_all();
        for k in [10usize, 20, 40, 64] {
            for (name, t) in [("A", shape_a(k)), ("B", shape_b(k)), ("C", shape_c(k))] {
                let nodes = node_count(&t);
                let via_analyse = visits_during(|| {
                    let _ = analyse(&t, &ctx);
                });
                assert_eq!(
                    via_analyse, nodes,
                    "AC1: `analyse` entered {via_analyse} nodes on shape ({name}) k={k}, \
                     which has {nodes} — a child is being re-walked",
                );
                let via_check = visits_during(|| {
                    let _ = check(&t, &ctx);
                });
                assert_eq!(
                    via_check, nodes,
                    "AC2: `check` entered {via_check} nodes on shape ({name}) k={k}, \
                     which has {nodes} — `typecheck` is re-walking a subtree",
                );
            }
        }
    }

    /// AC1 + AC2 on 1,000 random trees over every variant (the ladder shapes
    /// omit `Wedge`/`Inner`/`GradeLift`). `analyse` enters each node exactly
    /// once; `check` enters exactly the nodes before the first error and
    /// agrees with the oracle on whether there is one.
    #[test]
    fn t_visit_ladder_random_trees_ac1_ac2() {
        let ctx = ctx_all();
        let mut rng = SplitMix64::new(0x0020_1ADD);
        let mut ok_trees = 0;
        for _ in 0..1_000 {
            let t = rand_tree(&mut rng, 7);
            let nodes = node_count(&t);
            let via_analyse = visits_during(|| {
                let _ = analyse(&t, &ctx);
            });
            assert_eq!(
                via_analyse, nodes,
                "AC1: `analyse` entered {via_analyse} of {nodes} nodes on {t:?}",
            );
            let (want, want_ok) = expected_check_visits(&t, &ctx);
            let mut got_ok = false;
            let via_check = visits_during(|| {
                got_ok = check(&t, &ctx).is_ok();
            });
            assert_eq!(got_ok, want_ok, "`check` verdict diverges on {t:?}");
            assert_eq!(
                via_check, want,
                "AC2: `check` entered {via_check} nodes on {t:?}; §2.3 says {want} \
                 ({nodes} nodes, ok={want_ok})",
            );
            ok_trees += usize::from(want_ok);
        }
        assert!(ok_trees >= 100, "corpus has Ok trees to bound: {ok_trees}");
    }

    // ── T-versor-direct (AC3; §4.2) ────────────────────────────────────────

    /// AC3 — `analyse(r).versor` is `old_is_versor(r)` on every subtree of
    /// random trees and on a pinned table of witnesses and non-witnesses.
    /// Through `grade(Sandwich(r, x))` alone a wrong flag is masked wherever
    /// `grade(r) = {0}`, so the predicate is fuzzed directly.
    #[test]
    fn t_versor_direct_ac3() {
        let ctx = ctx_all();
        let param = || bx(GeoExpr::Param(1.0));
        let e1 = || bx(GeoExpr::Basis(1));
        let pinned = [
            (GeoExpr::Basis(1), true),  // a basis vector
            (GeoExpr::Basis(8), true),  // e₀: null, yet a sound witness (§2.1)
            (GeoExpr::Basis(3), false), // a bivector blade
            (GeoExpr::Basis(0), false), // the scalar blade
            (GeoExpr::Basis(16), false),
            (GeoExpr::Param(1.0), false),
            (GeoExpr::Var("v".into()), false),
            (GeoExpr::Exp(bx(GeoExpr::Basis(3))), true),
            (GeoExpr::Exp(param()), false),
            (GeoExpr::Exp(bx(GeoExpr::Var("w".into()))), false),
            // `Exp` of an ∅-graded operand: the vacuous witness (§2.2).
            (GeoExpr::Exp(bx(GeoExpr::Var("z".into()))), true),
            (GeoExpr::Exp(bx(GeoExpr::GradeProject(3, param()))), true),
            (GeoExpr::GeoProduct(e1(), bx(GeoExpr::Basis(2))), true),
            (GeoExpr::GeoProduct(e1(), param()), false),
            (GeoExpr::Reverse(e1()), true),
            (GeoExpr::Reverse(param()), false),
            (GeoExpr::Reverse(bx(rotor())), true),
            (rotor(), true),
            (non_versor_rotor(), false),
            (GeoExpr::Sandwich(e1(), bx(GeoExpr::Basis(2))), false),
            (GeoExpr::Wedge(e1(), bx(GeoExpr::Basis(2))), false),
            (GeoExpr::Inner(e1(), e1()), false),
            (GeoExpr::GradeLift(1, param()), false),
            (GeoExpr::GradeProject(1, e1()), false),
        ];
        for (r, want) in pinned {
            assert_eq!(old_is_versor(&r, &ctx), want, "oracle self-check {r:?}");
            assert_eq!(analyse(&r, &ctx).versor, want, "AC3: versor flag on {r:?}");
        }

        let mut rng = SplitMix64::new(0x0020_5EED);
        let (mut trues, mut falses) = (0usize, 0usize);
        for _ in 0..5_000 {
            let t = rand_tree(&mut rng, 6);
            for r in subtrees(&t) {
                let want = old_is_versor(r, &ctx);
                assert_eq!(
                    analyse(r, &ctx).versor,
                    want,
                    "AC3: versor flag diverges from `old_is_versor` on {r:?}",
                );
                if want {
                    trues += 1;
                } else {
                    falses += 1;
                }
            }
        }
        assert!(
            trues >= 500 && falses >= 500,
            "corpus covers both answers: {trues} true / {falses} false",
        );
    }

    // ── T-versor-sound (§4.4) ──────────────────────────────────────────────

    /// §4.4 — the flag is sound: for every `r` it marks, `R = eval(r)`
    /// satisfies `realized(R ∗ ~R) ⊆ {0}` — a necessary condition for the
    /// sandwich rule to preserve grade (`exp(B)·~exp(B) = 1`; `eᵢeᵢ` is scalar,
    /// `0` for `e₀`, whose ∅ ⊆ {0} is the intended reading of §2.1). Tolerance
    /// scales with `|R|²` so a large but exact rotor is not a false alarm.
    #[test]
    fn t_versor_sound() {
        let ctx = ctx_all();
        let env = env_all();
        let mut rng = SplitMix64::new(0x0020_50FD);
        let (mut checked, mut exp_witnesses) = (0usize, 0usize);
        for _ in 0..5_000 {
            let t = rand_tree(&mut rng, 5);
            for r in subtrees(&t) {
                if !analyse(r, &ctx).versor {
                    continue;
                }
                let Ok(value) = eval(r, &env) else {
                    continue; // an out-of-range leaf: nothing to evaluate
                };
                if !value.coeffs.iter().all(|c| c.is_finite()) {
                    continue;
                }
                let tol = 1e-9 * max_abs(&value).powi(2).max(1.0);
                let got = realized(&(value * value.reverse()), tol);
                assert!(
                    got.iter().all(|k| k == 0),
                    "UNSOUND versor witness {r:?}: R ∗ ~R realizes {got:?} ⊄ {{0}}",
                );
                checked += 1;
                exp_witnesses += usize::from(matches!(r, GeoExpr::Exp(_)));
            }
        }
        assert!(
            checked >= 500 && exp_witnesses >= 50,
            "a healthy witness sample: {checked} checked, {exp_witnesses} `Exp`",
        );
    }
}
