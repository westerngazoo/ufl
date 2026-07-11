//! SPEC-0011M §2.2/§4 — the typed param-slots and the lane instances:
//! T-slots-1 (enumeration), T-slots-2 (the **scoped** typecheck-invariant),
//! and the refiner's verdict-preservation.

use ufl_geo::{
    params, params_mut, typecheck, GeoExpr, GeoParamRefiner, GradeCtx, GradeScreen, GradeSet,
};
use ufl_prng::SplitMix64;
use ufl_search::{Refiner, Screen};

fn ctx_v1() -> GradeCtx {
    let mut ctx = GradeCtx::new();
    ctx.declare("v", GradeSet::singleton(1));
    ctx
}

/// A mixed tree with three `Param` leaves at known pre-order positions.
fn mixed_tree() -> GeoExpr {
    GeoExpr::Sandwich(
        Box::new(GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
            Box::new(GeoExpr::Param(-0.5)),
            Box::new(GeoExpr::Basis(3)),
        )))),
        Box::new(GeoExpr::GeoProduct(
            Box::new(GeoExpr::GradeLift(2, Box::new(GeoExpr::Param(1.5)))),
            Box::new(GeoExpr::GeoProduct(
                Box::new(GeoExpr::Var("v".into())),
                Box::new(GeoExpr::Param(2.5)),
            )),
        )),
    )
}

/// T-slots-1 — `params`/`params_mut` enumerate the same slots in the same
/// pre-order, and writes through `params_mut` land where `params` reads.
#[test]
fn slots_enumerate_the_same_preorder() {
    let mut e = mixed_tree();
    assert_eq!(params(&e), vec![-0.5, 1.5, 2.5], "pre-order snapshot");
    {
        let slots = params_mut(&mut e);
        assert_eq!(slots.len(), 3, "three Param leaves");
        for (i, s) in slots.into_iter().enumerate() {
            *s = i as f64;
        }
    }
    assert_eq!(params(&e), vec![0.0, 1.0, 2.0], "writes land in order");
}

/// T-slots-2 — the SCOPED typecheck-invariant (SPEC-0011M §2.2, three-lens
/// resolution): across Param poison, `typecheck(..).is_ok()` is unchanged and,
/// when `Ok`, the `GradeSet` is unchanged — on coherent AND incoherent trees.
/// (Deliberately NOT "same `Err`": `Incoherent` embeds the refined Param.)
#[test]
fn param_writes_never_change_the_typecheck_verdict() {
    let ctx = ctx_v1();
    let coherent = mixed_tree();
    let incoherent = GeoExpr::Sandwich(
        // A non-versor rotor slot: Param is grade {0}, not a versor — this tree
        // typechecks to an Err regardless of the Param's value.
        Box::new(GeoExpr::GradeLift(2, Box::new(GeoExpr::Param(0.0)))),
        Box::new(GeoExpr::Var("v".into())),
    );
    let poison = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, 1e300, -1.0];

    for tree in [coherent, incoherent] {
        let before = typecheck(&tree, &ctx);
        for p in poison {
            let mut t = tree.clone();
            for slot in params_mut(&mut t) {
                *slot = p;
            }
            let after = typecheck(&t, &ctx);
            assert_eq!(
                before.is_ok(),
                after.is_ok(),
                "is_ok() must be invariant under Param poison {p}"
            );
            if let (Ok(g0), Ok(g1)) = (&before, &after) {
                assert_eq!(g0, g1, "Ok(GradeSet) must be invariant under {p}");
            }
        }
    }
}

/// The refiner is structure-blind: every neighbor differs from its elite only
/// in `Param` values, so its `typecheck` verdict is identical — an admissible
/// elite can never refine into a screened-out neighbor.
#[test]
fn refiner_neighbors_preserve_the_verdict_and_structure() {
    let ctx = ctx_v1();
    let screen = GradeScreen::new(ctx_v1());
    let elite = mixed_tree();
    assert!(screen.admissible(&elite), "the elite is admissible");

    let refiner = GeoParamRefiner::pinned();
    let mut rng = SplitMix64::new(7);
    let neighbors = refiner.neighbors(&elite, &mut rng);
    assert_eq!(
        neighbors.len(),
        3 * 22,
        "±δ ladder: 3 slots × 11 rungs × 2 signs"
    );
    let elite_verdict = typecheck(&elite, &ctx_v1()).is_ok();
    for n in &neighbors {
        // Same slot count and same structural skeleton (params zeroed compare equal).
        assert_eq!(params(n).len(), params(&elite).len());
        let mut a = n.clone();
        let mut b = elite.clone();
        for s in params_mut(&mut a) {
            *s = 0.0;
        }
        for s in params_mut(&mut b) {
            *s = 0.0;
        }
        assert_eq!(a, b, "structure untouched — only Param values move");
        assert_eq!(
            typecheck(n, &ctx_v1()).is_ok(),
            elite_verdict,
            "verdict preserved"
        );
        assert!(screen.admissible(n), "refined neighbor stays admissible");
    }

    // A slot-free elite has no neighborhood.
    let bare = GeoExpr::Var("v".into());
    assert!(refiner.neighbors(&bare, &mut rng).is_empty());
    let _ = ctx; // the shared input-grade context
}
