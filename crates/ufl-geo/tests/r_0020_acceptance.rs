//! R-0020 acceptance suite — `grade` and `typecheck` visit each node once
//! (`ufl-geo`).
//!
//! Derived from [SPEC-0020 §4](../../../specs/0020-single-visit-grade.md) (the
//! test plan for [R-0020 §5](../../../requirements/0020-single-visit-grade.md)),
//! one section per acceptance criterion, each test citing its `ACn` id.
//! AC1/AC2 (the visit counter) need private items and live in
//! `src/grade.rs`'s `#[cfg(test)]` module; this file covers what the public
//! surface can observe: AC3 (the differential and error precedence), AC4 (the
//! re-measurement), AC5 (no surface change), plus §4.5's typecheck ⇒ eval
//! implication and §4.8's throughput report.
//!
//! # TDD status (loop step 3)
//!
//! Every test here compares the product against the **pre-change** code or
//! pins a property that already holds, so the file is **green today** — it is
//! the regression guard for the rewrite and turns red only if the rewrite
//! diverges from the oracle. The RED state of R-0020 is the in-`src` module.
//!
//! # The oracle
//!
//! `old_grade` / `old_is_versor` / `old_typecheck` are **verbatim**
//! transcriptions of `grade.rs:66-182` at commit `a9cbee9`, with one forced
//! substitution: `GradeCtx::get` is `pub(crate)`, so the oracle's `Var` arm
//! reads the declared set through `grade(e, ctx)` on that one leaf — the only
//! place it touches the code under test, as SPEC-0020 §8's harness did.
//! [`ac3_var_leaf_reads_the_declaration`] pins that leaf independently so the
//! substitution cannot mask a `Var` regression.

use std::hint::black_box;
use std::time::{Duration, Instant};

use ufl_ga::{Mv, Op};
use ufl_geo::{eval, grade, typecheck, Env, GeoError, GeoExpr, GradeCtx, GradeError, GradeSet};
use ufl_prng::SplitMix64;

const RELEASE_ONLY: &str =
    "release e2e: cargo test -p ufl-geo --release --test r_0020_acceptance -- --ignored --nocapture --test-threads=1";

/// The number of `Cl(3,0,1)` generators (verbatim from `grade.rs:14`).
const N: usize = 4;

// ── the pre-change oracle (verbatim, `a9cbee9`) ────────────────────────────

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

/// Verbatim pre-change `grade` (`grade.rs:86-144`), except the `Var` arm
/// (see the module doc).
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
        // Was `ctx.get(name)` — `pub(crate)`; a `Var` leaf is exactly that read.
        GeoExpr::Var(_) => grade(e, ctx),
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
        GeoExpr::Wedge(a, b) => Op::Wedge.output_grades(&[old_grade(a, ctx), old_grade(b, ctx)], N),
        GeoExpr::Inner(a, b) => Op::Inner.output_grades(&[old_grade(a, ctx), old_grade(b, ctx)], N),
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

/// Verbatim pre-change `typecheck` (`grade.rs:151-182`).
fn old_typecheck(e: &GeoExpr, ctx: &GradeCtx) -> Result<GradeSet, GradeError> {
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
            old_typecheck(a, ctx)?;
        }
        GeoExpr::Reverse(a) | GeoExpr::Exp(a) => {
            old_typecheck(a, ctx)?;
        }
        GeoExpr::GeoProduct(a, b)
        | GeoExpr::Wedge(a, b)
        | GeoExpr::Inner(a, b)
        | GeoExpr::Sandwich(a, b) => {
            old_typecheck(a, ctx)?;
            old_typecheck(b, ctx)?;
        }
    }
    let g = old_grade(e, ctx);
    if g.is_empty() {
        Err(GradeError::Incoherent(e.clone()))
    } else {
        Ok(g)
    }
}

// ── fixtures (mirror SPEC-0020 §8's harness; duplicated in `src/grade.rs`'s
//    test module, which this file cannot reach) ────────────────────────────

/// `v: {1}`, `w: {0, 2}`, `z: ∅`; `u` undeclared (⊤).
fn ctx_all() -> GradeCtx {
    let mut ctx = GradeCtx::new();
    ctx.declare("v", GradeSet::singleton(1));
    ctx.declare("w", GradeSet::EMPTY.with(0).with(2));
    ctx.declare("z", GradeSet::EMPTY);
    ctx
}

/// Bindings consistent with [`ctx_all`] for `v`/`w`/`z`; `u` is left unbound
/// so `eval`'s `Unbound` path is exercised.
fn env_without_u() -> Env {
    let mut env = Env::new();
    env.bind("v", Mv::basis(1));
    env.bind("w", Mv::scalar(1.0) + Mv::basis(3) * 0.5);
    env.bind("z", Mv::zero());
    env
}

fn bx(e: GeoExpr) -> Box<GeoExpr> {
    Box::new(e)
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

/// The children of a node, left to right.
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

fn node_count(e: &GeoExpr) -> usize {
    1 + children_of(e).into_iter().map(node_count).sum::<usize>()
}

/// A stable index per `GeoExpr` variant, for coverage accounting.
fn variant_index(e: &GeoExpr) -> usize {
    match e {
        GeoExpr::Param(_) => 0,
        GeoExpr::Basis(_) => 1,
        GeoExpr::Var(_) => 2,
        GeoExpr::GradeLift(..) => 3,
        GeoExpr::GeoProduct(..) => 4,
        GeoExpr::Wedge(..) => 5,
        GeoExpr::Inner(..) => 6,
        GeoExpr::Reverse(_) => 7,
        GeoExpr::GradeProject(..) => 8,
        GeoExpr::Sandwich(..) => 9,
        GeoExpr::Exp(_) => 10,
    }
}

fn mark_variants(e: &GeoExpr, seen: &mut [bool; 11]) {
    seen[variant_index(e)] = true;
    for c in children_of(e) {
        mark_variants(c, seen);
    }
}

/// A stable index per `typecheck` verdict, for coverage accounting.
fn verdict_index(r: &Result<GradeSet, GradeError>) -> usize {
    match r {
        Ok(_) => 0,
        Err(GradeError::Incoherent(_)) => 1,
        Err(GradeError::BadBlade(_)) => 2,
        Err(GradeError::BadGrade(_)) => 3,
    }
}

/// Mean wall-clock per call of `f` after one warm-up call, over at least one
/// timed call and at most a 100 ms budget.
fn per_call<T>(mut f: impl FnMut() -> T) -> Duration {
    black_box(f());
    let budget = Duration::from_millis(100);
    let start = Instant::now();
    let mut calls = 0u32;
    while calls == 0 || (start.elapsed() < budget && calls < 100_000) {
        black_box(f());
        calls += 1;
    }
    start.elapsed() / calls
}

// ── AC3 — semantic equivalence, proven (§4.1, §4.6) ────────────────────────

/// AC3 — `grade` and `typecheck` against the verbatim pre-change oracle on
/// random trees over every variant: identical `GradeSet`, identical `Result`
/// including which error and which subtree `Incoherent` carries. The corpus
/// is asserted to reach every variant and every verdict, so the differential
/// is not vacuous.
#[test]
fn ac3_differential_on_random_trees() {
    let ctx = ctx_all();
    let mut rng = SplitMix64::new(0x0020_D1FF);
    let mut seen = [false; 11];
    let mut verdicts = [0usize; 4];
    for _ in 0..30_000 {
        let t = rand_tree(&mut rng, 7);
        mark_variants(&t, &mut seen);
        assert_eq!(
            grade(&t, &ctx),
            old_grade(&t, &ctx),
            "AC3: `grade` diverges from the pre-change oracle on {t:?}",
        );
        let want = old_typecheck(&t, &ctx);
        assert_eq!(
            typecheck(&t, &ctx),
            want,
            "AC3: `typecheck` diverges from the pre-change oracle on {t:?}",
        );
        verdicts[verdict_index(&want)] += 1;
    }
    assert!(seen.iter().all(|&s| s), "every variant appears: {seen:?}");
    assert!(
        verdicts.iter().all(|&n| n >= 100),
        "every verdict appears [ok, incoherent, badblade, badgrade]: {verdicts:?}",
    );
}

/// AC3 — shape (C) pinned at k ∈ 0..=16 against the exponential oracle.
/// Random fuzz reaches a rotor-nest of 5 at ~2×10⁻⁴ and 7+ never, so without
/// this the differential never exercises the shape the change exists for.
#[test]
fn ac3_differential_shape_c_pinned() {
    let ctx = ctx_all();
    for k in 0..=16 {
        let t = shape_c(k);
        assert_eq!(
            grade(&t, &ctx),
            old_grade(&t, &ctx),
            "AC3: `grade` diverges on shape (C) k={k}",
        );
        assert_eq!(
            typecheck(&t, &ctx),
            old_typecheck(&t, &ctx),
            "AC3: `typecheck` diverges on shape (C) k={k}",
        );
    }
}

/// AC3 — the one leaf the oracle reads through the product: a `Var` grades
/// to its declaration, ⊤ when undeclared, and `typecheck` rejects a declared-∅
/// variable as `Incoherent` on the leaf itself.
#[test]
fn ac3_var_leaf_reads_the_declaration() {
    let ctx = ctx_all();
    let var = |n: &str| GeoExpr::Var(n.into());
    assert_eq!(grade(&var("v"), &ctx), GradeSet::singleton(1));
    assert_eq!(grade(&var("w"), &ctx), GradeSet::EMPTY.with(0).with(2));
    assert_eq!(grade(&var("z"), &ctx), GradeSet::EMPTY);
    assert_eq!(grade(&var("u"), &ctx), GradeSet::full(4));
    assert_eq!(typecheck(&var("u"), &ctx), Ok(GradeSet::full(4)));
    assert_eq!(
        typecheck(&var("z"), &ctx),
        Err(GradeError::Incoherent(var("z")))
    );
}

/// AC3 — error precedence (§4.6), four named cases: own `BadBlade`/`BadGrade`
/// before descent; `Incoherent` post-order carrying the **innermost** subtree;
/// children fully processed left to right, so a left child's post-order
/// `Incoherent` beats a right child's pre-order `BadBlade`.
#[test]
fn ac3_error_precedence_four_named_cases() {
    let ctx = ctx_all();
    let param = || bx(GeoExpr::Param(1.0));
    let z = || GeoExpr::Var("z".into());
    let cases = [
        (
            "own BadBlade before the child's BadGrade",
            GeoExpr::Sandwich(
                bx(GeoExpr::Basis(20)),
                bx(GeoExpr::GradeProject(9, param())),
            ),
            GradeError::BadBlade(20),
        ),
        (
            "own BadGrade before descent into a BadBlade",
            GeoExpr::GradeProject(9, bx(GeoExpr::Basis(20))),
            GradeError::BadGrade(9),
        ),
        (
            "Incoherent carries the INNER projection",
            GeoExpr::GradeProject(2, bx(GeoExpr::GradeProject(3, param()))),
            GradeError::Incoherent(GeoExpr::GradeProject(3, param())),
        ),
        (
            "left child's post-order Incoherent beats right child's BadBlade",
            GeoExpr::GeoProduct(bx(z()), bx(GeoExpr::Basis(20))),
            GradeError::Incoherent(z()),
        ),
    ];
    for (label, e, want) in cases {
        assert_eq!(
            old_typecheck(&e, &ctx),
            Err(want.clone()),
            "oracle: {label}"
        );
        assert_eq!(typecheck(&e, &ctx), Err(want), "AC3 precedence: {label}");
    }
}

// ── §4.5 — typecheck ⇒ eval ────────────────────────────────────────────────

/// §4.5 — `typecheck`'s three strict points are `eval`'s three guards at the
/// same pre-order position, so a tree `typecheck` accepts can fail `eval` only
/// with `Unbound` — never `BadBlade`/`BadGrade`.
#[test]
fn typecheck_ok_implies_eval_fails_only_with_unbound() {
    let ctx = ctx_all();
    let env = env_without_u();
    let mut rng = SplitMix64::new(0x0020_E7A1);
    let (mut accepted, mut evaluated, mut unbound) = (0usize, 0usize, 0usize);
    for _ in 0..30_000 {
        let e = rand_tree(&mut rng, 7);
        if typecheck(&e, &ctx).is_err() {
            continue;
        }
        accepted += 1;
        match eval(&e, &env) {
            Ok(_) => evaluated += 1,
            Err(GeoError::Unbound(_)) => unbound += 1,
            Err(other) => panic!("typecheck accepted {e:?} but eval failed with {other:?}"),
        }
    }
    assert!(
        accepted >= 1_000 && evaluated >= 500 && unbound >= 50,
        "a healthy sample: {accepted} accepted, {evaluated} evaluated, {unbound} unbound",
    );
}

// ── AC4 — the table re-measured (§4.7) ─────────────────────────────────────

/// AC4 — R-0020 §1's table on shape (C) at k ∈ {14, 18, 20, 22}: `typecheck`
/// before (the oracle) and after, printed **unconditionally**. Per
/// `docs/conventions.md` *Assert the Protocol, Not the Outcome*, the assertion
/// is that all four pre-registered rows were recorded — not that any target
/// was hit.
#[test]
#[ignore = "release e2e: cargo test -p ufl-geo --release --test r_0020_acceptance -- --ignored --nocapture --test-threads=1"]
fn ac4_remeasure_the_r0020_table() {
    let ctx = ctx_all();
    let mut rows = Vec::new();
    println!("shape (C)   k  nodes   old typecheck   new typecheck   speed-up");
    for k in [14usize, 18, 20, 22] {
        let t = shape_c(k);
        let nodes = node_count(&t);
        let old = per_call(|| old_typecheck(&t, &ctx));
        let new = per_call(|| typecheck(&t, &ctx));
        let ratio = old.as_secs_f64() / new.as_secs_f64().max(1e-12);
        println!("           {k:>2}  {nodes:>5}   {old:>13.1?}   {new:>13.1?}   {ratio:>8.0}×");
        rows.push((k, nodes, old, new));
    }
    assert_eq!(
        rows.len(),
        4,
        "AC4: all four pre-registered depths must be recorded ({RELEASE_ONLY})",
    );
}

// ── §4.8 — throughput on production-sized trees ────────────────────────────

/// §4.8 — `typecheck` old vs new on 20,000 random trees of ≤ 60 nodes,
/// ns/call, printed unconditionally. A regression here is a finding, not a
/// failure: the assertion is that the corpus was measured.
#[test]
#[ignore = "release e2e: cargo test -p ufl-geo --release --test r_0020_acceptance -- --ignored --nocapture --test-threads=1"]
fn throughput_typecheck_old_vs_new() {
    const TREES: usize = 20_000;
    let ctx = ctx_all();
    let mut rng = SplitMix64::new(0x0020_7480);
    let mut corpus = Vec::with_capacity(TREES);
    while corpus.len() < TREES {
        let t = rand_tree(&mut rng, 7);
        if node_count(&t) <= 60 {
            corpus.push(t);
        }
    }
    let nodes: usize = corpus.iter().map(node_count).sum();
    let sweep = |f: &dyn Fn(&GeoExpr) -> Result<GradeSet, GradeError>| {
        for t in &corpus {
            let _ = black_box(f(t));
        }
        let start = Instant::now();
        for t in &corpus {
            let _ = black_box(f(t));
        }
        start.elapsed().as_nanos() as f64 / TREES as f64
    };
    let old = sweep(&|t| old_typecheck(t, &ctx));
    let new = sweep(&|t| typecheck(t, &ctx));
    println!(
        "typecheck on {TREES} random trees (≤ 60 nodes, avg {:.1}): old {old:.0} ns/call, \
         new {new:.0} ns/call ({:.2}× of old)",
        nodes as f64 / TREES as f64,
        new / old,
    );
    assert_eq!(
        corpus.len(),
        TREES,
        "the corpus was measured ({RELEASE_ONLY})"
    );
}

// ── AC5 — no public surface change ─────────────────────────────────────────

/// AC5 — `grade`, `typecheck`, `GradeCtx`, `GradeError`, and `GradeSet` keep
/// their signatures; `GradeError` keeps exactly three variants and
/// `Incoherent` keeps its `GeoExpr` payload (SPEC-0019 §2.7). A change to any
/// of these fails to compile here.
#[test]
fn ac5_public_surface_is_unchanged() {
    let grade_fn: fn(&GeoExpr, &GradeCtx) -> GradeSet = grade;
    let typecheck_fn: fn(&GeoExpr, &GradeCtx) -> Result<GradeSet, GradeError> = typecheck;
    let same_grade_set: fn(ufl_geo::GradeSet) -> ufl_ga::GradeSet = |g| g;

    fn derives<T: Clone + std::fmt::Debug + Default>() {}
    fn error_derives<T: Clone + std::fmt::Debug + PartialEq + std::error::Error>() {}
    derives::<GradeCtx>();
    error_derives::<GradeError>();

    let mut ctx = GradeCtx::new();
    ctx.declare("v", GradeSet::singleton(1));
    ctx.declare(String::from("z"), GradeSet::EMPTY);
    assert_eq!(
        same_grade_set(grade_fn(&GeoExpr::Var("v".into()), &ctx)),
        GradeSet::singleton(1)
    );

    // The payload types, by construction.
    let variants = [
        GradeError::Incoherent(GeoExpr::Var("z".into())),
        GradeError::BadBlade(20u8),
        GradeError::BadGrade(9u8),
    ];
    // Exactly three variants: no wildcard, so a fourth fails to compile.
    let describe = |err: &GradeError| match err {
        GradeError::Incoherent(sub) => format!("incoherent {}", node_count(sub)),
        GradeError::BadBlade(i) => format!("blade {i}"),
        GradeError::BadGrade(k) => format!("grade {k}"),
    };
    let described: Vec<String> = variants.iter().map(describe).collect();
    assert_eq!(described, ["incoherent 1", "blade 20", "grade 9"]);

    let got = typecheck_fn(&GeoExpr::Var("z".into()), &ctx);
    assert_eq!(
        got.as_ref().map_err(describe),
        Err("incoherent 1".to_string())
    );
}
