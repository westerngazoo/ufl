//! R-0010 soundness + totality regression gate (review stage).
//!
//! These tests pin the two defects the three-lens review + the adversarial
//! soundness audit (4 lenses, ~280k fuzzed trees against the real `Cl(3,0,1)`
//! kernel) found in the first green implementation:
//!
//! 1. **Under-approximation in `grade(GradeLift)`.** `eval` lowered
//!    `GradeLift(k, e)` to the geometric product `eval(e) * blade_k`, but `grade`
//!    returned `{k}` ignoring `e` — so for a non-scalar child the realized grade
//!    escaped the inferred set (`GradeLift(1, e1)` → `e1*e1 = scalar`, grade `{0}`,
//!    but inferred `{1}`). SPEC-0010 §2.2 annotates the child *"(scalar)"*; the
//!    fix projects it to its scalar part so the value is genuinely pure grade `k`
//!    and `grade = {k}` is sound (honouring R-0010 AC3 "grade-lift produces `{k}`").
//! 2. **Totality panic in `grade(GradeProject)`.** The rule handed the raw `u8`
//!    `k` to garust, whose `singleton(k) = 1 << k` overflows `u32` for `k ≥ 32`
//!    (a panic — and `cargo test` green is a hard merge gate). `eval` guarded
//!    `k > 4`; `grade` now guards it too (→ `∅`, projection onto an absent grade).
//!
//! THE SOUNDNESS CONTRACT (the permanent property): for every form,
//! `realized(eval(e)) ⊆ grade(e)`. `grade` is a sound *over*-approximation; the
//! dangerous failure is *under*-approximation — a realized grade missing from the
//! inferred set, since that signal is what R-0011 prunes on (SPEC-0010 §2.5/AC6).

use ufl_ga::Mv;
use ufl_geo::{eval, grade, typecheck, Env, GeoExpr, GradeCtx, GradeError, GradeSet};

/// The grades an `Mv` actually carries (non-negligible after cleaning to ε).
fn realized(mv: &Mv) -> GradeSet {
    let mut g = GradeSet::EMPTY;
    for k in 0..=4usize {
        if mv.grade(k).cleaned(1e-9) != Mv::zero() {
            g = g.with(k);
        }
    }
    g
}

/// `a ⊆ b`.
fn subset(a: GradeSet, b: GradeSet) -> bool {
    a.iter().all(|k| b.contains(k))
}

/// Bug 1 — `grade(GradeLift(k, child))` is sound for a NON-scalar child: the
/// realized grade is always within the inferred set. These closed forms each
/// fed `grade`'s old `{k}` rule a value off grade `k` (the geometric product of
/// the child with `blade_k`); the fix projects the child to its scalar part, so
/// the value is pure grade `k` (or zero) and the inference holds.
#[test]
fn gradelift_grade_is_sound_for_nonscalar_children() {
    let env = Env::new();
    let ctx = GradeCtx::new();
    let cases = [
        GeoExpr::GradeLift(1, Box::new(GeoExpr::Basis(1))), // e1 has no scalar part
        GeoExpr::GradeLift(2, Box::new(GeoExpr::Basis(12))), // e34 (the old e1234 escape)
        GeoExpr::GradeLift(2, Box::new(GeoExpr::Basis(3))),
        GeoExpr::GradeLift(1, Box::new(GeoExpr::Basis(2))),
        GeoExpr::GradeLift(0, Box::new(GeoExpr::Basis(1))), // lowest_blade(0)=scalar unit
        GeoExpr::GradeLift(
            3,
            Box::new(GeoExpr::Wedge(
                Box::new(GeoExpr::Basis(1)),
                Box::new(GeoExpr::Basis(2)),
            )),
        ),
    ];
    for e in cases {
        let mv = eval(&e, &env).expect("closed GradeLift evaluates");
        let g = grade(&e, &ctx);
        assert!(
            subset(realized(&mv), g),
            "UNSOUND: {e:?} realized {:?} ⊄ grade {g:?}",
            realized(&mv),
        );
    }
}

/// Bug 1 (semantics anchor) — `GradeLift(k, scalar)` lifts the scalar to the
/// lowest grade-`k` blade: `GradeLift(2, Param(3))` → `3·e12`, grade `{2}`.
#[test]
fn gradelift_lifts_a_scalar_to_grade_k() {
    let env = Env::new();
    let e = GeoExpr::GradeLift(2, Box::new(GeoExpr::Param(3.0)));
    let mv = eval(&e, &env).expect("GradeLift of a scalar evaluates");
    let want = Mv::scalar(3.0) * Mv::basis(3); // 3·e12
    assert_eq!(
        (mv - want).cleaned(1e-10),
        Mv::zero(),
        "GradeLift(2, 3) must be 3·e12"
    );
    assert_eq!(grade(&e, &GradeCtx::new()), GradeSet::singleton(2));
}

/// Bug 2 — `grade`/`typecheck` are TOTAL on an out-of-range `GradeProject` grade,
/// including `k ≥ 32` (the `1 << k` u32-overflow boundary). Projection onto an
/// absent grade is the empty set; `typecheck` reports `BadGrade`.
#[test]
fn gradeproject_grade_is_total_no_overflow_panic() {
    let ctx = GradeCtx::new();
    let child = Box::new(GeoExpr::Param(1.0));
    for k in [5u8, 6, 31, 32, 64, 99, 255] {
        let e = GeoExpr::GradeProject(k, child.clone());
        let g = grade(&e, &ctx); // must not panic
        assert!(
            g.is_empty(),
            "GradeProject({k}, scalar) projects onto an absent grade → ∅, got {g:?}",
        );
        assert_eq!(
            typecheck(&e, &ctx),
            Err(GradeError::BadGrade(k)),
            "typecheck must reject an out-of-range projection grade",
        );
    }
}

/// The permanent soundness fuzz (the regression gate the audit recommended):
/// over many bounded well-formed random trees, `realized(eval(e)) ⊆ grade(e)`,
/// and neither `grade` nor `typecheck` panics. Deterministic (hand-rolled PRNG,
/// fixed seeds), `Param`s bounded to keep `exp` finite. Exercises `GradeLift`
/// with non-scalar children heavily — the shape that broke soundness.
#[test]
fn soundness_property_holds_over_random_trees() {
    let mut env = Env::new();
    env.bind("a", Mv::basis(1)); // grade 1
    env.bind("b", Mv::basis(3)); // e12, grade 2
    let mut ctx = GradeCtx::new();
    ctx.declare("a", GradeSet::singleton(1));
    ctx.declare("b", GradeSet::singleton(2));

    let mut checked = 0usize;
    for seed in 1u64..=8 {
        let mut rng = seed;
        for _ in 0..400 {
            let e = gen(&mut rng, 4);
            let ge = grade(&e, &ctx); // must not panic
            let _ = typecheck(&e, &ctx); // must not panic
            if let Ok(mv) = eval(&e, &env) {
                assert!(
                    subset(realized(&mv), ge),
                    "UNSOUND: {e:?}\n  realized {:?} ⊄ grade {ge:?}",
                    realized(&mv),
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 1000,
        "expected a healthy sample of Ok evals, got {checked}"
    );
}

/// A bounded random well-formed `GeoExpr` (valid leaves only, `Var`s drawn from
/// the declared/bound set; grades 0..=4). Hand-rolled LCG so the suite is
/// deterministic without a dependency.
fn gen(rng: &mut u64, depth: u8) -> GeoExpr {
    *rng = rng
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let r = (*rng >> 33) as u32;
    let param = || GeoExpr::Param(((r >> 4) % 5) as f64 - 2.0); // {-2,-1,0,1,2}
    let basis = || GeoExpr::Basis(((r >> 4) % 16) as u8);
    let var = || GeoExpr::Var(if r & 1 == 0 { "a" } else { "b" }.into());
    if depth == 0 {
        return match r % 4 {
            0 => param(),
            1 => basis(),
            2 => var(),
            _ => var(),
        };
    }
    let k = ((r >> 4) % 5) as u8;
    match r % 11 {
        0 => param(),
        1 => basis(),
        2 => var(),
        3 => GeoExpr::GradeLift(k, Box::new(gen(rng, depth - 1))),
        4 => GeoExpr::GeoProduct(Box::new(gen(rng, depth - 1)), Box::new(gen(rng, depth - 1))),
        5 => GeoExpr::Wedge(Box::new(gen(rng, depth - 1)), Box::new(gen(rng, depth - 1))),
        6 => GeoExpr::Inner(Box::new(gen(rng, depth - 1)), Box::new(gen(rng, depth - 1))),
        7 => GeoExpr::Reverse(Box::new(gen(rng, depth - 1))),
        8 => GeoExpr::GradeProject(k, Box::new(gen(rng, depth - 1))),
        9 => GeoExpr::Sandwich(Box::new(gen(rng, depth - 1)), Box::new(gen(rng, depth - 1))),
        _ => GeoExpr::Exp(Box::new(gen(rng, depth - 1))),
    }
}
