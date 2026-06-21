//! R-0010 acceptance suite — geometric forms + the grade-type system (`ufl-geo`).
//!
//! Derived from [SPEC-0010 §6](../../../specs/0010-geometric-forms-grade-types.md)
//! (the authoritative restatement of [R-0010 §3](../../../requirements/0010-geometric-forms-grade-types.md))
//! — one or more `#[test]`s per acceptance criterion, each citing its `ACn` id.
//! The tests are the requirement's weight: they exercise the typed geometric
//! layer (`GeoExpr` / `eval` / `grade` / `typecheck`) through `ufl-geo`'s own
//! public surface.
//!
//! # TDD status (loop step 3, RED)
//!
//! `eval`, `grade`, and `typecheck` are `unimplemented!()` until R-0010 step 5.
//! So every test that calls one of them **panics today** — the failing-test-first
//! state CLAUDE.md §4/§5 requires. The two structural AC1/AC5 tests
//! ([`ac1_geoexpr_is_clone_debug_partialeq`], [`ac5_homoiconic_ast_reader_deferred`])
//! are compile-time proofs that do not touch the unimplemented core; they pass
//! immediately and stay green through step 5. Every behavioural test turns GREEN
//! with no edit once the stubs are filled in.
//!
//! # Conventions (mirrors `r_0009_acceptance.rs`)
//!
//! - `τ = std::f64::consts::TAU` (UFL's circle constant — `docs/conventions.md`).
//! - **ε = 1e-10** (SPEC-0009 §2.5). Multivector compares use [`approx_mv`],
//!   the spec's *`cleaned`-then-compare* pattern.
//! - Grade facts are **exact**: `GradeSet` derives `Eq`, so they assert with `==`
//!   against constructed sets (`GradeSet::singleton`, `GradeSet::EMPTY.with(..)`,
//!   `GradeSet::full(4)`); there is no `Display`, only `Debug`.
//!
//! # The keystone form (shared by AC2 / AC4 — matches the R-0009 keystone)
//!
//! The R-0009 keystone is `((e1*e2)*(−τ/8)).exp().sandwich(&e1) ≈ e2`. Its
//! `GeoExpr` form is `Sandwich(Exp(GeoProduct(Param(−τ/8), Basis(e12))), Basis(e1))`
//! (`e12` is blade index `3`), built once by [`keystone`].

use std::f64::consts::TAU;

use ufl_ga::Mv;
use ufl_geo::{eval, grade, typecheck, Env, GeoError, GeoExpr, GradeCtx, GradeError, GradeSet};

/// SPEC-0009 §2.5 — the floating tolerance carried into AC2/AC4 eval checks.
const EPS: f64 = 1e-10;

/// e12 — the grade-2 bivector blade. `grade_of(3) = 3u8.count_ones() = 2`.
const E12: u8 = 3;

/// Floating multivector equality, SPEC-0009 §2.5's *subtract-then-`cleaned`*
/// pattern (reused verbatim from `r_0009_acceptance.rs`).
fn approx_mv(got: &Mv, want: &Mv) {
    let residual = (*got - *want).cleaned(EPS);
    assert_eq!(
        residual,
        Mv::zero(),
        "multivectors differ by more than ε={EPS}: got {got:?}, want {want:?}",
    );
}

/// The keystone form (SPEC-0010 §2.4): `Sandwich(R, e1)` with
/// `R = Exp(GeoProduct(Param(−τ/8), Basis(e12)))` — a statically-known versor
/// (the `exp` of a `{2}` bivector). `eval`s to `e2` (AC2); `grade`s to `{1}` (AC4).
fn keystone() -> GeoExpr {
    let theta = -TAU / 8.0;
    let rotor = GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
        Box::new(GeoExpr::Param(theta)),
        Box::new(GeoExpr::Basis(E12)),
    )));
    GeoExpr::Sandwich(Box::new(rotor), Box::new(GeoExpr::Basis(1)))
}

// ===========================================================================
// AC1 — The geometric AST. `GeoExpr` is `Clone + Debug + PartialEq`, an
// inspectable code-as-data tree (the genotype R-0011 mutates). SPEC-0010 §2.1.
// ===========================================================================

/// AC1 — `GeoExpr` is `Clone + Debug + PartialEq` and structurally comparable: a
/// form clones equal to itself, two *different* forms compare unequal, and the
/// tree is inspectable (`Debug` renders). This is the compile-time proof the AST
/// is the inspectable genotype — it touches none of the unimplemented core.
#[test]
fn ac1_geoexpr_is_clone_debug_partialeq() {
    let form = keystone();

    // Clone — a clone equals the original (derives `Clone + PartialEq`).
    let cloned = form.clone();
    assert_eq!(form, cloned, "a cloned GeoExpr must equal its original");

    // Debug — the tree is inspectable (renders a non-empty representation).
    assert!(
        !format!("{form:?}").is_empty(),
        "GeoExpr must be Debug-inspectable",
    );

    // PartialEq discriminates — two structurally different forms are unequal.
    let other = GeoExpr::Param(1.0);
    assert_ne!(
        form, other,
        "structurally different forms must compare unequal"
    );

    // Leaf-level discrimination — same variant, different payload ⇒ unequal.
    assert_ne!(
        GeoExpr::Param(1.0),
        GeoExpr::Param(2.0),
        "Param(1.0) and Param(2.0) must compare unequal",
    );
    assert_ne!(
        GeoExpr::Basis(1),
        GeoExpr::Basis(2),
        "Basis(1) and Basis(2) must compare unequal",
    );
}

// ===========================================================================
// AC2 — Evaluation onto the kernel, total. `eval(GeoExpr, env) → Result<Mv,
// GeoError>` lowers each form onto its `ufl_ga` op; out-of-range leaves return a
// typed `GeoError`, never a panic. SPEC-0010 §2.2.
// ===========================================================================

/// AC2 (keystone) — the keystone form `eval`s to `e2` within ε: the same
/// sandwich that R-0009 proved sends `e1 → e2`, now expressed as a `GeoExpr` and
/// lowered. Compared on the grade-1 part (a rotated vector is a vector).
#[test]
fn ac2_keystone_evals_to_e2() {
    let value = eval(&keystone(), &Env::new()).expect("keystone must eval to Ok");
    approx_mv(&value.grade(1), &Mv::basis(2));
}

/// AC2 (lowering) — the leaf and product forms lower onto their `ufl_ga` ops:
/// `Param(2)` → scalar 2; `e1 ∗ e1` → scalar 1; `e1 ∧ e2` → e12; a bound `Var`
/// reads its binding back.
#[test]
fn ac2_forms_lower_onto_the_kernel() {
    let env = Env::new();

    // Param(2.0) → Mv::scalar(2.0).
    let s = eval(&GeoExpr::Param(2.0), &env).expect("Param must eval");
    approx_mv(&s, &Mv::scalar(2.0));

    // e1 ∗ e1 = 1 (the geometric product, e1² = +1).
    let sq = eval(
        &GeoExpr::GeoProduct(Box::new(GeoExpr::Basis(1)), Box::new(GeoExpr::Basis(1))),
        &env,
    )
    .expect("GeoProduct must eval");
    approx_mv(&sq, &Mv::scalar(1.0));

    // e1 ∧ e2 = e12 (blade index 3).
    let wedge = eval(
        &GeoExpr::Wedge(Box::new(GeoExpr::Basis(1)), Box::new(GeoExpr::Basis(2))),
        &env,
    )
    .expect("Wedge must eval");
    approx_mv(&wedge, &Mv::basis(E12 as usize));

    // A bound Var reads its binding back.
    let mut bound = Env::new();
    bound.bind("x", Mv::basis(2));
    let read = eval(&GeoExpr::Var("x".into()), &bound).expect("bound Var must eval");
    approx_mv(&read, &Mv::basis(2));
}

/// AC2 (totality) — out-of-range / unbound leaves return a **typed `GeoError`**,
/// never a panic: an unbound `Var`, a `Basis(≥16)`, and a grade `> 4` in both
/// `GradeProject` and `GradeLift`.
#[test]
fn ac2_eval_is_total_typed_errors_not_panics() {
    let env = Env::new();

    // Unbound variable.
    assert_eq!(
        eval(&GeoExpr::Var("x".into()), &env),
        Err(GeoError::Unbound("x".into())),
        "an unbound Var must be GeoError::Unbound, not a panic",
    );

    // Basis index out of the 16-blade algebra.
    assert_eq!(
        eval(&GeoExpr::Basis(16), &env),
        Err(GeoError::BadBlade(16)),
        "Basis(16) must be GeoError::BadBlade, not a panic",
    );

    // GradeProject grade > 4.
    assert_eq!(
        eval(
            &GeoExpr::GradeProject(5, Box::new(GeoExpr::Param(1.0))),
            &env,
        ),
        Err(GeoError::BadGrade(5)),
        "GradeProject(5, ..) must be GeoError::BadGrade, not a panic",
    );

    // GradeLift grade > 4.
    assert_eq!(
        eval(&GeoExpr::GradeLift(5, Box::new(GeoExpr::Param(1.0))), &env),
        Err(GeoError::BadGrade(5)),
        "GradeLift(5, ..) must be GeoError::BadGrade, not a panic",
    );
}

// ===========================================================================
// AC3 — Grade inference (sound, total, delegated). `grade` returns the correct
// sound over-approximation, delegating catalog forms to `Op::output_grades` and
// hand-ruling Sandwich/Exp/GradeLift; out-of-range leaves return ⊤, not an error.
// SPEC-0010 §2.3.
// ===========================================================================

/// AC3 — the leaf grade rules: `Param → {0}`, `Basis(e12) → {2}` (the bivector
/// blade index 3 has two set bits).
#[test]
fn ac3_leaf_grades() {
    let ctx = GradeCtx::new();
    assert_eq!(grade(&GeoExpr::Param(1.0), &ctx), GradeSet::singleton(0));
    assert_eq!(grade(&GeoExpr::Basis(E12), &ctx), GradeSet::singleton(2));
}

/// AC3 — the binary product rules, delegated to `Op::output_grades` (n = 4):
/// wedge **adds** (`e1∧e2 → {2}`); the Hestenes inner **subtracts**
/// (`e1·e2 → {|1−1|} = {0}` for grade-1 operands); the geometric product
/// **mixes** (`e1∗e1 → {0,2}`, the step-2 span `|1−1|..1+1`).
#[test]
fn ac3_product_grade_rules() {
    let ctx = GradeCtx::new();

    // Wedge adds: 1 + 1 = 2.
    assert_eq!(
        grade(
            &GeoExpr::Wedge(Box::new(GeoExpr::Basis(1)), Box::new(GeoExpr::Basis(2))),
            &ctx,
        ),
        GradeSet::singleton(2),
        "wedge adds grades: e1∧e2 ∈ {{2}}",
    );

    // Hestenes inner subtracts: |1 − 1| = 0.
    assert_eq!(
        grade(
            &GeoExpr::Inner(Box::new(GeoExpr::Basis(1)), Box::new(GeoExpr::Basis(2))),
            &ctx,
        ),
        GradeSet::singleton(0),
        "Hestenes inner subtracts grades: e1·e2 ∈ {{0}}",
    );

    // Geometric product mixes: |1−1|, …, 1+1 step 2 = {0, 2}.
    assert_eq!(
        grade(
            &GeoExpr::GeoProduct(Box::new(GeoExpr::Basis(1)), Box::new(GeoExpr::Basis(1))),
            &ctx,
        ),
        GradeSet::EMPTY.with(0).with(2),
        "geo product of two grade-1s spans {{0, 2}}",
    );
}

/// AC3 — `Reverse` preserves grade, `GradeProject(k)` intersects with `{k}`, and
/// `GradeLift(k)` produces `{k}`.
#[test]
fn ac3_reverse_project_lift_grades() {
    let ctx = GradeCtx::new();

    // Reverse preserves: ~e12 stays grade 2.
    assert_eq!(
        grade(&GeoExpr::Reverse(Box::new(GeoExpr::Basis(E12))), &ctx),
        GradeSet::singleton(2),
        "reverse preserves grade",
    );

    // GradeProject(2) of a pure grade-2 wedge → {2} (k ∩ {2}).
    assert_eq!(
        grade(
            &GeoExpr::GradeProject(
                2,
                Box::new(GeoExpr::Wedge(
                    Box::new(GeoExpr::Basis(1)),
                    Box::new(GeoExpr::Basis(2)),
                )),
            ),
            &ctx,
        ),
        GradeSet::singleton(2),
        "grade-project to a present grade keeps it",
    );

    // GradeLift(2) produces {2}, regardless of the lifted scalar.
    assert_eq!(
        grade(&GeoExpr::GradeLift(2, Box::new(GeoExpr::Param(3.0))), &ctx),
        GradeSet::singleton(2),
        "grade-lift k produces {{k}}",
    );
}

/// AC3 (totality) — out-of-range leaves leave `grade` **total**, returning the
/// top set `⊤ = full(4)` rather than erroring (`typecheck` is what errors; AC6):
/// `Basis(≥16)` and `GradeLift(>4)` both yield `full(4)`.
#[test]
fn ac3_out_of_range_leaves_are_top_not_panic() {
    let ctx = GradeCtx::new();
    assert_eq!(
        grade(&GeoExpr::Basis(16), &ctx),
        GradeSet::full(4),
        "Basis(16) must be ⊤ = full(4), not a panic",
    );
    assert_eq!(
        grade(&GeoExpr::GradeLift(5, Box::new(GeoExpr::Param(1.0))), &ctx),
        GradeSet::full(4),
        "GradeLift(5, ..) must be ⊤ = full(4), not a panic",
    );
}

// ===========================================================================
// AC4 — The grade-preservation keystone. For a statically-known versor `R` and a
// grade-1 `v`, `grade(Sandwich(R, v)) == {1}` (vector → vector); the same form
// `eval`s to a rotated vector. A *non*-versor `r` yields the sound product bound,
// a STRICT superset of {1}. SPEC-0010 §2.4.
// ===========================================================================

/// AC4 (keystone) — `grade(Sandwich(R, e1)) == {1}` for the versor
/// `R = Exp(GeoProduct(Param, Basis(e12)))`: a rotated vector is still a vector.
/// The *same* form `eval`s to `e2` (proved in [`ac2_keystone_evals_to_e2`]) — the
/// type says "vector → vector," the eval says "specifically e1 → e2."
#[test]
fn ac4_keystone_grade_is_vector() {
    let g = grade(&keystone(), &GradeCtx::new());
    assert_eq!(
        g,
        GradeSet::singleton(1),
        "a rotated vector is still a vector: grade(Sandwich(versor, e1)) == {{1}}",
    );
}

/// AC4 (non-versor fallback) — when `r` is *not* a statically-known versor the
/// sandwich grade rule falls back to the sound product bound: a **strict
/// superset** of `{1}` (it still contains grade 1 — soundness — *and more*).
///
/// SPEC-0010 §2.4's illustrative non-versor is `r = Param + Basis(e1)` (grades
/// `{0,1}`), but `GeoExpr` has **no `Add` variant** — that form can't be built
/// literally. A faithful, constructible substitute is a `Var` *declared* with
/// grades `{0,1}` (a value that is statically not a single-grade versor), kept as
/// the sandwich's `r`. The over-approximation property is identical.
#[test]
fn ac4_nonversor_fallback_is_strict_superset_of_vector() {
    let mut ctx = GradeCtx::new();
    ctx.declare("m", GradeSet::EMPTY.with(0).with(1)); // grades {0,1} — not a static versor

    let nonversor = GeoExpr::Sandwich(
        Box::new(GeoExpr::Var("m".into())),
        Box::new(GeoExpr::Basis(1)), // a grade-1 operand
    );

    let g = grade(&nonversor, &ctx);
    assert!(
        g.contains(1),
        "soundness: the fallback bound must still contain grade 1, got {g:?}",
    );
    assert!(
        g.len() > 1,
        "the non-versor fallback over-approximates — a STRICT superset of {{1}}, got {g:?}",
    );
}

// ===========================================================================
// AC5 — Homoiconic AST (reader deferred). `GeoExpr` is the code-as-data form
// representation; the textual `Sexpr → GeoExpr` reader is a documented non-goal
// here (SPEC-0010 §2.6 + R-0010 decision log). SPEC-0010 §6 AC5.
// ===========================================================================

/// AC5 — the homoiconic contract: `GeoExpr` is the **code-as-data** AST — a
/// `Clone`-able, inspectable, constructor-built tree — and the textual
/// `Sexpr → GeoExpr` reader is **deferred** (SPEC-0010 §2.6; R-0010 AC5
/// pre-authorized + decision-logged). The real evidence is structural and lives
/// in [`ac1_geoexpr_is_clone_debug_partialeq`] (clone + inspect a
/// constructor-built tree); there is no reader to test here, so this test only
/// pins that a `GeoExpr` is built directly from constructors (no parse step).
#[test]
fn ac5_homoiconic_ast_reader_deferred() {
    // Built purely from constructors — code as data, no textual reader involved.
    let form = keystone();
    assert_eq!(
        form,
        form.clone(),
        "GeoExpr is the constructor-built homoiconic AST (reader deferred, §2.6)",
    );
}

// ===========================================================================
// AC6 — Grade coherence. `typecheck` returns the inferred `GradeSet`, or a typed
// `GradeError` for an incoherent form (`GradeProject(k, a)` with `k ∉ grade(a)` →
// ∅) and the out-of-range `BadBlade`/`BadGrade` leaves. SPEC-0010 §2.5.
// ===========================================================================

/// AC6 — a coherent form typechecks to its inferred grade: the keystone →
/// `Ok({1})` (the same set `grade` infers; `typecheck` and `grade` share one
/// source of truth, so they cannot disagree).
#[test]
fn ac6_coherent_form_typechecks_to_its_grade() {
    assert_eq!(
        typecheck(&keystone(), &GradeCtx::new()),
        Ok(GradeSet::singleton(1)),
        "the coherent keystone typechecks to its inferred grade {{1}}",
    );
}

/// AC6 — a grade-incoherent `GradeProject(k, a)` with `k ∉ grade(a)` is rejected:
/// projecting a pure grade-2 bivector (`e1∧e2`) to grade 3 yields the empty grade
/// set, an unsatisfiable form ⇒ `Err(GradeError::Incoherent)`.
#[test]
fn ac6_incoherent_grade_project_is_rejected() {
    let incoherent = GeoExpr::GradeProject(
        3,
        Box::new(GeoExpr::Wedge(
            Box::new(GeoExpr::Basis(1)),
            Box::new(GeoExpr::Basis(2)),
        )),
    );
    match typecheck(&incoherent, &GradeCtx::new()) {
        Err(GradeError::Incoherent(_)) => {}
        other => panic!(
            "projecting a grade-2 bivector to grade 3 must be GradeError::Incoherent, got {other:?}",
        ),
    }
}

/// AC6 — out-of-range leaves are typed `GradeError`s at `typecheck`: a
/// `Basis(≥16)` is `BadBlade`, a grade `> 4` is `BadGrade`.
#[test]
fn ac6_out_of_range_leaves_are_typed_errors() {
    let ctx = GradeCtx::new();

    assert_eq!(
        typecheck(&GeoExpr::Basis(16), &ctx),
        Err(GradeError::BadBlade(16)),
        "typecheck(Basis(16)) must be GradeError::BadBlade",
    );

    assert_eq!(
        typecheck(
            &GeoExpr::GradeProject(5, Box::new(GeoExpr::Param(1.0))),
            &ctx
        ),
        Err(GradeError::BadGrade(5)),
        "typecheck(GradeProject(5, ..)) must be GradeError::BadGrade",
    );
}
