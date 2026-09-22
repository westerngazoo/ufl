//! R-0022 acceptance suite — Gate 2's witness and the fair comparison
//! (SPEC-0022 §5's ten tests, plus two guards this suite adds).
//!
//! # The metric, stated once
//!
//! RMSE here is **per-component** — `sqrt(Σ[(x−x̂)² + (y−ŷ)²] / 2n)`,
//! `baseline.rs:295`'s formula unchanged, so the witness and the MLP are divided
//! by the same denominator (SPEC-0022 §1). A per-*point* figure (denominator `n`)
//! is exactly √2 larger; rev 2 of the spec mixed the two in one table.
//! [`metric_is_per_component`] makes reintroducing that a red build rather than a
//! footnote — it pins `measure_band` against a reference spelled out here, and
//! then proves the two figures are distinguishable at this magnitude.
//!
//! # Acceptance-criteria map
//!
//! | AC | test |
//! |----|------|
//! | AC1 | [`exact_in_dist`], [`witness_shape`], [`metric_is_per_component`] |
//! | AC2 | [`exact_ood`], [`metric_is_per_component`] |
//! | AC3 | [`comparison_artifact`] (the `#[ignore]`d release artifact) |
//! | AC4 | [`structure`], [`structure_is_not_enough`], [`binding_matches_declaration`], [`planar`] |
//! | AC5 | [`reparametrisation`] |
//! | AC6 | discharged whichever way the suite falls: no test asserts a hoped outcome |
//! | — | [`far_band`], [`null_blade_addition`] (SPEC-0022 §5.8/§5.10, no AC) |
//!
//! # Measured before anything here was asserted (release, this branch)
//!
//! | quantity | measured | asserted |
//! |---|---|---|
//! | in-dist RMSE `[−2,2]²`, 60×60 | 1.8033e-16 | `< 1e-15` (5.5× headroom) |
//! | OOD RMSE `[2,3]²`, 60×60 | 1.7967e-16 | `< 1e-15`, within 10× of in-dist |
//! | far RMSE `[−8,8]²`, 60×60 | 1.8329e-16 | `< 1e-15`, within 10× of in-dist |
//! | `fk_motor` / `fk_witness` nodes | 23 / 25 | exact |
//! | `Param`s | 4 = `[−0.5, 0.5, −0.5, 0.35]` | exact |
//! | worst `\|⟨M ∗ M̃⟩₀ − 1\|`, 409 poses | 4.44e-16 (2 ulp) | `< 1e-15` |
//! | worst non-scalar residue of `M ∗ M̃` | 1.11e-16 | `< 1e-15` |
//! | `max abs(w − 1)`, all bands | 6.66e-16 | `< 1e-15` |
//! | `z`, every sample of every band | exactly `0.0` | exactly `0.0` |
//! | reparametrisation max deviation, 40×40, `a = 0.37` | 1.1102e-15 | `< 5e-15` (4.5×) |
//! | null-blade addition | bitwise identical | bitwise identical |
//!
//! An absolute `1e-14` is deliberately **not** used: SPEC-0022 §4.1 measures it
//! failing on `[−10³,10³]` (1.3200e-14, reproduced by
//! [`comparison_artifact`]'s boundary row). Every bound above is scoped to the
//! band the ACs name.
//!
//! Two things the *quoted figures* depend on, neither of which the *claim*
//! depends on — both worth knowing before anyone calls a 2% delta a regression:
//!
//! - **Build profile.** The in-dist band is 1.8033e-16 in release and 1.7971e-16
//!   in debug. The table above is the release run, matching SPEC-0022 §1.
//! - **Product association.** A left-nested `((R·T)·R)·T` chain measures
//!   1.7311e-16 where the committed balanced `(R·T)·(R·T)` measures 1.8033e-16 —
//!   a 4% difference in pure rounding. Both are the exact map to f64; only the
//!   balanced tree reproduces §1's quoted numbers, which is one more reason the
//!   expression is committed rather than described.
//!
//! # Commands
//!
//! ```text
//! cargo test -p ufl-evolve --test r_0022_gate2_witness
//! cargo test -p ufl-evolve --release --test r_0022_gate2_witness -- --ignored --nocapture
//! ```

use ufl_evolve::baseline::{smallest_at, sweep, train_report_with, ArmFk, TrainConfig};
use ufl_evolve::witness::{
    even_grades, fk_motor, fk_witness, measure, measure_witness, motor_unit_check, node_count,
    param_count, read_xy, read_z, sample, witness_ctx, witness_env, Band, BandReport, E12, E1E0,
    ORIGIN,
};
use ufl_geo::{params, typecheck, GeoExpr, GradeCtx, GradeSet, Mv};

/// The pinned arm — the same one `baseline.rs`'s tests and SPEC-0022 §4.2 train
/// the MLP against, so both halves of the comparison describe one task.
const ARM: ArmFk = ArmFk { l1: 1.0, l2: 0.7 };

/// Samples per axis for the RMSE bands: 60×60 = 3,600 per band (SPEC-0022 §1).
const GRID: usize = 60;

/// Samples per axis for the reparametrisation grid (SPEC-0022 §5.7).
const REPARAM_GRID: usize = 40;

/// The base-frame rotation in `FK(t1+a, t2) = R_a · FK(t1,t2) · R_a~`.
const SHIFT: f64 = 0.37;

/// The exactness bound for the AC-named bands. Measured 1.80e-16, so this is an
/// order of headroom and no more; SPEC-0022 §7 Q1 rejects `1e-14`.
const EXACT: f64 = 1e-15;

/// The reparametrisation bound — measured 1.1102e-15, so 4.5× headroom.
const REPARAM_TOL: f64 = 5e-15;

/// The in-distribution band `[−2,2]²` (R-0022 AC1).
const IN_DIST: (f64, f64) = (-2.0, 2.0);

/// The out-of-distribution band `[2,3]²` (R-0022 AC2) — where the angle sum
/// `t1+t2` reaches `[4,6]`, a phase no MLP in the sweep ever saw.
const OOD: (f64, f64) = (2.0, 3.0);

/// The far band `[−8,8]²`, four times outside the training range (SPEC-0022 §1).
const FAR: (f64, f64) = (-8.0, 8.0);

/// The boundary band from SPEC-0022 §4.1, where an absolute `1e-14` bound fails.
const BOUNDARY: (f64, f64) = (-1000.0, 1000.0);

/// The same deterministic closed lattice `measure` sweeps: `n` points per
/// axis, endpoints included, `t_i = lo + (hi − lo)·i/(n − 1)`.
/// [`metric_is_per_component`] holds the two implementations together.
fn grid(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| lo + (hi - lo) * (i as f64) / ((n - 1) as f64))
        .collect()
}

/// Sweep a band, or fail by name. `measure` returns `Err` on any evaluation
/// failure, so `Ok` *is* SPEC-0022 §1's "zero evaluation failures".
fn band(name: &str, (lo, hi): (f64, f64)) -> BandReport {
    let b = Band::new(lo, hi, GRID)
        .unwrap_or_else(|e| panic!("{name} band [{lo}, {hi}] is not a band: {e}"));
    let report = measure_witness(&ARM, b)
        .unwrap_or_else(|e| panic!("{name} band [{lo}, {hi}] failed to evaluate: {e}"));
    assert_eq!(
        report.band.n() * report.band.n(),
        GRID * GRID,
        "{name}: the band must sweep {GRID}×{GRID} = {} samples",
        GRID * GRID,
    );
    assert!(
        report.rmse.is_finite(),
        "{name}: RMSE is {} — a non-finite readout is an ideal (zero-weight) \
         point, not an exact one",
        report.rmse,
    );
    report
}

/// Evaluate an expression at one pose. Test code, so `expect` is fine — and an
/// unbound-variable or bad-blade failure here *is* the thing to shout about.
fn at(e: &GeoExpr, t1: f64, t2: f64) -> Mv {
    sample(e, t1, t2).expect("the expression evaluates at every pose")
}

/// The grades an `Mv` actually carries — the same helper as
/// `ufl-geo/tests/r_0010_soundness.rs`, so "realized" means one thing repo-wide.
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

/// The largest blade-wise difference between two multivectors.
fn max_coeff_dev(a: &Mv, b: &Mv) -> f64 {
    a.coeffs
        .as_slice()
        .iter()
        .zip(b.coeffs.as_slice().iter())
        .fold(0.0_f64, |worst, (x, y)| worst.max((x - y).abs()))
}

/// Per-component RMSE, spelled out here rather than called — the reference
/// [`metric_is_per_component`] pins `measure_band` against.
fn reference_rmse(arm: &ArmFk, (lo, hi): (f64, f64), n: usize) -> f64 {
    let expr = fk_witness(arm.l1, arm.l2);
    let axis = grid(lo, hi, n);
    let mut se = 0.0;
    for &t1 in &axis {
        for &t2 in &axis {
            let (x, y) = read_xy(&at(&expr, t1, t2));
            let (gx, gy) = arm.forward(t1, t2);
            se += (x - gx).powi(2) + (y - gy).powi(2);
        }
    }
    (se / ((n * n) as f64 * 2.0)).sqrt()
}

// ---------------------------------------------------------------------------
// AC1 — the witness is in-repo and exact
// ---------------------------------------------------------------------------

/// **AC1** (SPEC-0022 §5.1) — T-exact-in-dist. Per-component RMSE over a
/// deterministic 60×60 lattice on `[−2,2]²` against `ArmFk::forward`, through the
/// real `ufl-geo`/`ufl-ga` kernel. Measured 1.8033e-16; asserted `< 1e-15` **and
/// printed**, because AC1 pins "machine precision" as a measured figure rather
/// than a tolerance chosen after the fact.
#[test]
fn exact_in_dist() {
    let r = band("in-dist", IN_DIST);
    println!(
        "AC1 in-dist [-2,2]² {GRID}x{GRID} per-component RMSE = {:e}",
        r.rmse
    );
    assert!(
        r.rmse < EXACT,
        "AC1: in-dist RMSE {:e} is not machine precision (bound {EXACT:e}; \
         1.8033e-16 when the bound was chosen)",
        r.rmse,
    );
}

/// **AC1 / AC3** — the witness's shape, asserted in the default suite instead of
/// only printed by the `#[ignore]`d artifact.
///
/// `fk_witness` is 25 nodes / 4 `Param`s, reproducing SPEC-0011 §2.6's discarded
/// count. The motor is the 23 that leaves once `Sandwich` and `Basis(7)` come
/// off: two 6-node rotors, two 4-node translators, three `GeoProduct`s.
/// (`witness.rs`'s `fk_motor` doc comment says 23, corrected from an
/// arithmetic slip; 21 + 2 could not be the triply-reproduced 25.)
#[test]
fn witness_shape() {
    let motor = fk_motor(ARM.l1, ARM.l2);
    let witness = fk_witness(ARM.l1, ARM.l2);
    println!(
        "motor: {} nodes, {} params | witness: {} nodes, {} params = {:?}",
        node_count(&motor),
        param_count(&motor),
        node_count(&witness),
        param_count(&witness),
        params(&witness),
    );

    assert_eq!(
        (E12, ORIGIN, E1E0),
        (3, 7, 9),
        "the blade indices are garust's convention and the derivation rests on \
         them: e₁₂ = 3 (the rotation plane), e₁₂₃ = 7 (the origin point), \
         e₁e₀ = 9 (null, so `Exp` truncates)",
    );
    assert_eq!(
        node_count(&motor),
        23,
        "the motor is R(t1)·T(l1)·R(t2)·T(l2): 6+4+6+4 subtree nodes plus three \
         GeoProducts",
    );
    assert_eq!(
        node_count(&witness),
        25,
        "the witness is the motor plus Sandwich plus Basis(e₁₂₃) — SPEC-0022 §1's \
         25 nodes",
    );
    assert_eq!(param_count(&motor), 4, "one Param per rotor and translator");
    assert_eq!(
        param_count(&witness),
        4,
        "the Sandwich adds structure, not parameters — 4 against the MLP's 322",
    );
    assert_eq!(
        params(&witness),
        vec![-0.5, 0.5 * ARM.l1, -0.5, 0.5 * ARM.l2],
        "the four Params are the two half-angle factors and the two half-link \
         lengths — derived from ground-truth constants, NOT fitted (SPEC-0022 §4.1)",
    );
    assert_eq!(
        witness,
        GeoExpr::Sandwich(Box::new(motor), Box::new(GeoExpr::Basis(ORIGIN))),
        "AC4: the witness IS the motor sandwiched onto the origin point — not a \
         formulation that merely happens to fit",
    );
}

/// **AC1 / AC2, the metric** — `measure_band` is per-component over the
/// documented lattice. The regression for SPEC-0022 §1's named defect: rev 2 put
/// a per-point figure (denominator `n`) in the same table as a per-component one,
/// and the two differ by exactly √2.
///
/// Two assertions, because the first alone would prove nothing: the library must
/// **agree** with a reference spelled out in this file, and the per-component and
/// per-point figures must be **distinguishable** at this magnitude (they are
/// separated by 7.47e-17, measured).
#[test]
fn metric_is_per_component() {
    let library = band("in-dist", IN_DIST).rmse;
    let reference = reference_rmse(&ARM, IN_DIST, GRID);
    let per_point = reference * std::f64::consts::SQRT_2;
    println!(
        "library={library:e}  reference(per-component)={reference:e}  \
         per-point(√2×)={per_point:e}"
    );
    assert!(
        (library - reference).abs() < 1e-18,
        "`measure_band` must be per-component over the closed {GRID}×{GRID} \
         lattice: got {library:e}, reference {reference:e} (measured bitwise \
         equal). A denominator-`n` formula would land near {per_point:e}",
    );
    assert!(
        (library - per_point).abs() > 1e-17,
        "the per-component and per-point figures must be distinguishable here, or \
         the assertion above is vacuous: {library:e} vs {per_point:e}",
    );
}

// ---------------------------------------------------------------------------
// AC2 — the OOD half, the actual headline
// ---------------------------------------------------------------------------

/// **AC2** (SPEC-0022 §5.2) — T-exact-ood, the headline. The same witness on
/// `[2,3]²`, where `t1+t2` reaches `[4,6]`. AC2 asks for the OOD RMSE to be
/// within the same order of magnitude as the in-dist RMSE; that *ratio*, not the
/// absolute, is the property the MLP cannot match — its own ratio runs 27×…138×.
#[test]
fn exact_ood() {
    let in_dist = band("in-dist", IN_DIST).rmse;
    let ood = band("ood", OOD).rmse;
    let ratio = ood / in_dist;
    println!("AC2 in-dist={in_dist:e} ood={ood:e} ood/in-dist={ratio:.3}x");
    assert!(
        ood < EXACT,
        "AC2: OOD RMSE {ood:e} is not machine precision (bound {EXACT:e}; \
         1.7967e-16 when the bound was chosen)",
    );
    assert!(
        (0.1..=10.0).contains(&ratio),
        "AC2: the OOD RMSE must stay within one order of magnitude of in-dist \
         ({in_dist:e} → {ood:e} is {ratio:.3}x). The MLP's same ratio is \
         27x…138x, and that gap is the entire reason R-0022 exists",
    );
}

// ---------------------------------------------------------------------------
// AC4 — the derivation is checked, not asserted
// ---------------------------------------------------------------------------

/// **AC4** (SPEC-0022 §5.3/§2.1) — T-structure. The three-part guard as three
/// separate assertions with distinct messages, so a failure says *which* property
/// broke:
///
/// 1. **even** — `typecheck(fk_motor) == Ok({0,2,4})` excludes every reflection
///    and every odd blade;
/// 2. **unit** — `⟨M ∗ M̃⟩₀ == 1` with no non-scalar residue excludes the
///    null/degenerate case and any scaled versor;
/// 3. **point space** — `typecheck(fk_witness) == Ok({3})` puts the result in PGA
///    point space for *all* inputs, structurally, without evaluating anything.
///
/// Together: *a proper rigid motion applied to the origin, yielding a point.*
/// Not *which* rigid motion — [`structure_is_not_enough`] states that limit, and
/// [`exact_in_dist`]/[`exact_ood`] carry the other half of the claim.
#[test]
fn structure() {
    let motor = fk_motor(ARM.l1, ARM.l2);
    let witness = fk_witness(ARM.l1, ARM.l2);
    let ctx = witness_ctx();

    // (0) THE TIE. Without this, the even/unit clauses below are about an
    // expression that need not be the one `Ok({3})` is proved for, and the
    // three parts of the guard would not compose into a single claim. The
    // equality also lives in `witness_shape`; it is duplicated here on purpose
    // so weakening that test cannot silently untie this one.
    assert_eq!(
        witness,
        GeoExpr::Sandwich(Box::new(motor.clone()), Box::new(GeoExpr::Basis(7))),
        "the witness must BE Sandwich(motor, e₁₂₃) — the three guard clauses \
         below are otherwise about two unrelated expressions (SPEC-0022 §2.1)",
    );

    // (1) EVEN. The literal set is spelled out so the library's `even_grades`
    // helper is checked rather than trusted.
    assert_eq!(
        even_grades(),
        GradeSet::singleton(0).with(2).with(4),
        "`even_grades()` must be {{0,2,4}}, or guard 1/3 asserts nothing",
    );
    assert_eq!(
        typecheck(&motor, &ctx),
        Ok(even_grades()),
        "AC4 structural guard 1/3 (EVEN) broke: the motor must typecheck to \
         {{0,2,4}}. Any odd grade means a reflection or an odd blade — improper, \
         not a rigid motion (SPEC-0022 §2.1)",
    );

    // (2) UNIT. Unlike (1) and (3) this clause is **sampled, not proved** —
    // `ufl-geo` has no "is a versor" judgement to appeal to (`grade.rs:79-82`
    // says the versor flag explicitly is not one), so the honest statement is
    // that it holds at every pose tried. So try hard ones: a 20×20 in-dist
    // lattice plus the extremes, where argument reduction, denormals and
    // cancellation would show up if anything did.
    let axis = grid(IN_DIST.0, IN_DIST.1, 20);
    let extremes = [
        (0.0, 0.0),
        (3.0, 3.0),
        (-8.0, 8.0),
        (1e3, -1e3),
        (1e6, 1e6),
        (-1e9, 1e9),
        (1e15, 1.0),
        (f64::MIN_POSITIVE, f64::MIN_POSITIVE),
        (std::f64::consts::PI, -std::f64::consts::PI),
    ];
    let poses = axis
        .iter()
        .flat_map(|&t1| axis.iter().map(move |&t2| (t1, t2)))
        .chain(extremes);
    let mut worst_scalar = 0.0_f64;
    let mut worst_residue = 0.0_f64;
    let mut checked = 0usize;
    for (t1, t2) in poses {
        let (scalar, residue) = motor_unit_check(&motor, t1, t2)
            .unwrap_or_else(|e| panic!("the motor evaluates at ({t1}, {t2}): {e}"));
        worst_scalar = worst_scalar.max((scalar - 1.0).abs());
        worst_residue = worst_residue.max(residue);
        checked += 1;
    }
    println!(
        "AC4 unit over {checked} poses ({}x{} lattice + {} extremes): worst \
         |⟨M∗M̃⟩₀ − 1| = {worst_scalar:e} ({:.1} ulp of 1.0), worst non-scalar \
         residue = {worst_residue:e}",
        axis.len(),
        axis.len(),
        extremes.len(),
        worst_scalar / f64::EPSILON,
    );
    assert_eq!(
        checked,
        axis.len() * axis.len() + extremes.len(),
        "every pose in the lattice and every extreme must be checked",
    );
    assert!(
        worst_scalar < EXACT,
        "AC4 structural guard 2/3 (UNIT, scalar) broke: ⟨M ∗ M̃⟩₀ deviates from 1 \
         by {worst_scalar:e} (bound {EXACT:e}; 4.44e-16 when the bound was \
         chosen). A null versor gives 0.0 and a scaled one gives ≠ 1",
    );
    assert!(
        worst_residue < EXACT,
        "AC4 structural guard 2/3 (UNIT, residue) broke: M ∗ M̃ carries \
         {worst_residue:e} outside grade 0 (bound {EXACT:e}; 1.11e-16 when the \
         bound was chosen), so M is not a versor",
    );

    // (3) POINT SPACE.
    assert_eq!(
        typecheck(&witness, &ctx),
        Ok(GradeSet::singleton(3)),
        "AC4 structural guard 3/3 (POINT SPACE) broke: the sandwich must \
         typecheck to {{3}} — PGA point space — for all inputs",
    );
}

/// **AC4, the limit** (SPEC-0022 §5.4/§2) — T-structure-is-not-enough.
/// *This test asserts a negative on purpose.*
///
/// Rev 2 of SPEC-0022 claimed `typecheck` returning `Ok({3})` proves the sandwich
/// is a rigid motion. It does not, and the refutation is pinned here so that no
/// future reader can re-derive rev 2's error from rev 3's passing tests:
///
/// - `Sandwich(Basis(1), Basis(7))` typechecks `Ok({3})` — **indistinguishable
///   from the witness on that check alone** — yet `Basis(1) = e₁` is a grade-1
///   vector, so it is a *reflection*: improper, and it collapses every input to
///   the origin.
/// - `Basis(1)` even *passes* the unit check exactly (`e₁ ∗ ~e₁ = (1.0, 0.0)`), so
///   unit alone does not save it. Only the **even** check rejects it.
/// - `Basis(8) = e₀` is null: odd *and* non-unit, and its sandwich is the zero
///   multivector whose readout is `NaN`.
///
/// So each clause of §2.1's guard is load-bearing, and `{3}` is the weakest of
/// the three. `grade.rs:79-82` said so itself, in a doc comment written during
/// R-0020: the versor flag is not "is a versor".
#[test]
fn structure_is_not_enough() {
    let ctx = witness_ctx();
    let reflection = GeoExpr::Sandwich(
        Box::new(GeoExpr::Basis(1)),
        Box::new(GeoExpr::Basis(ORIGIN)),
    );
    let null = GeoExpr::Sandwich(
        Box::new(GeoExpr::Basis(8)),
        Box::new(GeoExpr::Basis(ORIGIN)),
    );

    // The point-space check cannot tell either impostor from the witness.
    assert_eq!(
        typecheck(&reflection, &ctx),
        Ok(GradeSet::singleton(3)),
        "the refutation needs Sandwich(e₁, e₁₂₃) to typecheck EXACTLY as the \
         witness does. If this ever differs, `Ok({{3}})` became a stronger claim \
         and SPEC-0022 §2 should be revisited — not this test patched",
    );
    assert_eq!(
        typecheck(&null, &ctx),
        Ok(GradeSet::singleton(3)),
        "Sandwich(e₀, e₁₂₃) also typechecks {{3}} despite evaluating to zero",
    );

    // …but the EVEN check rejects both, which is why §2.1 asserts it.
    for blade in [1u8, 8] {
        assert_eq!(
            typecheck(&GeoExpr::Basis(blade), &ctx),
            Ok(GradeSet::singleton(1)),
            "Basis({blade}) is grade 1 — odd, so the even check rejects it",
        );
    }
    assert!(
        !subset(GradeSet::singleton(1), even_grades()),
        "grade 1 ⊄ {{0,2,4}}: the even check is exactly what excludes an improper \
         motion, and this is the arithmetic that makes it so",
    );

    // Unit alone does not save it: the reflection is exactly unit.
    assert_eq!(
        motor_unit_check(&GeoExpr::Basis(1), 0.0, 0.0),
        Ok((1.0, 0.0)),
        "e₁ ∗ ~e₁ is exactly (1.0, 0.0) — the reflection PASSES the same unit \
         check the motor passes, so unit is not a rigid-motion certificate either",
    );
    // The null blade fails it.
    assert_eq!(
        motor_unit_check(&GeoExpr::Basis(8), 0.0, 0.0),
        Ok((0.0, 0.0)),
        "e₀ ∗ ~e₀ is (0.0, 0.0), not (1.0, 0.0) — null and non-invertible, and \
         the unit check is what excludes it",
    );

    // And the numerics confirm what the structure excluded.
    let (rx, ry) = read_xy(&at(&reflection, 0.7, -0.4));
    assert_eq!(
        (rx.abs(), ry.abs()),
        (0.0, 0.0),
        "the reflection collapses every input to the origin — it fits nothing, \
         and `Ok({{3}})` alone could not tell",
    );
    let nulled = at(&null, 0.7, -0.4);
    assert_eq!(
        nulled,
        Mv::zero(),
        "e₀ x ~e₀ = 0 — the sandwich annihilates the point entirely",
    );
    assert!(
        read_xy(&nulled).0.is_nan(),
        "and dividing by its zero weight yields NaN. A tolerance-based test would \
         report that as a numeric miss; it is a *structural* failure, which is \
         why §2.1's guard is structural",
    );
}

/// **AC4 / SPEC-0022 §2.3** (§5.5) — T-binding-matches-declaration. `Ok({3})` is
/// conditional on `t1`/`t2` being declared grade `{0}`, and *nothing in the
/// pipeline checks that the declaration matches what `Env` actually binds* — a
/// caller that declares `{0}` and binds a bivector gets an unsound proof. Both
/// halves are therefore asserted together here:
///
/// - every binding `witness_env` produces realizes a grade within `{0}`, and at a
///   non-zero pose realizes exactly `{0}`;
/// - the declaration is what tightens the proof — under an empty `GradeCtx` the
///   same expression is only `Ok({0,1,2,3,4})` (⊤), i.e. no proof at all.
#[test]
fn binding_matches_declaration() {
    let scalar_only = GradeSet::singleton(0);
    for (t1, t2) in [(0.0, 0.0), (0.7, -0.4), (2.5, 2.5), (-8.0, 8.0)] {
        let env = witness_env(t1, t2);
        for name in ["t1", "t2"] {
            let bound = env
                .get(name)
                .unwrap_or_else(|| panic!("`witness_env` must bind `{name}`"));
            assert!(
                subset(realized(&bound), scalar_only),
                "§2.3: `{name}` is declared grade {{0}} but binds {:?} at \
                 ({t1}, {t2}) — the grade proof would be unsound",
                realized(&bound),
            );
        }
    }

    // A binding of 0.0 realizes ∅, which is a sound subset of {0} but would also
    // be satisfied by binding nothing — so the strict check needs a pose where
    // the scalar is actually present.
    let env = witness_env(0.7, -0.4);
    for name in ["t1", "t2"] {
        let bound = env
            .get(name)
            .unwrap_or_else(|| panic!("`witness_env` must bind `{name}`"));
        assert_eq!(
            realized(&bound),
            scalar_only,
            "`{name}` must bind a grade-0 `Mv` — a scalar, which is how SPEC-0011 \
             §2.5 requires the angles to arrive",
        );
    }

    let witness = fk_witness(ARM.l1, ARM.l2);
    assert_eq!(
        typecheck(&witness, &witness_ctx()),
        Ok(GradeSet::singleton(3)),
        "with `t1`/`t2` declared {{0}}, the proof tightens to point space",
    );
    assert_eq!(
        typecheck(&witness, &GradeCtx::new()),
        Ok(GradeSet::full(4)),
        "§2.3: with an empty context both vars are ⊤ and `typecheck` cannot \
         tighten past ⊤. The `Ok({{3}})` headline IS conditional on the \
         declaration, and this is the assertion that says so out loud",
    );
}

/// **AC4, a free cross-check** (SPEC-0022 §5.6) — T-planar. The arm is planar, so
/// the `z` coordinate must be **exactly** `0.0` — not small, zero — at every
/// sample of every band. A formulation leaking into `e₃` would still fit
/// `(x, y)`; this catches it with no tolerance at all.
///
/// The per-sample scan and `BandReport::max_z` are both checked, so the library's
/// own accounting is verified rather than taken on trust. `max abs(w − 1)` comes
/// along for free — SPEC-0022 §4.3 claims the homogeneous divide inside `read_xy`
/// is a measured no-op, and this is where that is measured.
#[test]
fn planar() {
    let expr = fk_witness(ARM.l1, ARM.l2);
    for (name, bounds) in [("in-dist", IN_DIST), ("ood", OOD), ("far", FAR)] {
        let (lo, hi) = bounds;
        let axis = grid(lo, hi, GRID);
        let mut reach = 0.0_f64;
        for &t1 in &axis {
            for &t2 in &axis {
                let mv = at(&expr, t1, t2);
                let z = read_z(&mv);
                assert_eq!(
                    z, 0.0,
                    "{name}: z = {z:e} at ({t1}, {t2}). The 2-link arm is planar, \
                     so any e₃ content means the derivation is not the motor \
                     chain it claims to be",
                );
                let (x, y) = read_xy(&mv);
                reach = reach.max(x.hypot(y));
            }
        }
        let report = band(name, bounds);
        println!(
            "planar {name}: z ≡ 0.0 over {} samples, max |w−1| = {:e}, reach = {reach:.4}",
            axis.len() * axis.len(),
            report.max_weight_error,
        );
        // Reachability pre-check: `z ≡ 0` is vacuous for an output collapsed to
        // the origin (measured — a reflection-collapsed motor passes this test
        // without it). The arm's reach is `l1 + l2 = 1.7`.
        assert!(
            reach > 0.5,
            "{name}: the readout never left the origin (max reach {reach:e}), so \
             `z ≡ 0.0` proves nothing about planarity",
        );
        assert_eq!(
            report.max_z, 0.0,
            "{name}: `BandReport::max_z` must agree with the per-sample scan",
        );
        assert!(
            report.max_weight_error < EXACT,
            "{name}: the homogeneous weight drifted by {:e}, so §4.3's \"the \
             divide is a measured no-op\" no longer holds (bound {EXACT:e}; \
             ≤ 6.66e-16 when the bound was chosen)",
            report.max_weight_error,
        );
    }
}

// ---------------------------------------------------------------------------
// SPEC-0022 §2's counterexample table, rebuilt from the repo
// ---------------------------------------------------------------------------

/// **SPEC-0022 §2's counterexample table, measured through the committed
/// harness.** Every row here typechecks `Ok({3})` — the result rev 2 mistook
/// for a rigid-motion certificate — and every row is wrong.
///
/// This test exists because the table's figures were originally produced
/// off-repo by a harness that hard-coded `fk_witness`. `measure` now takes the
/// expression, so the spec's numbers are rebuilt on every run and the exact
/// constants that produce them live in code rather than in a deleted scratch
/// file.
#[test]
fn the_counterexamples_are_measurable_from_the_repo() {
    let ctx = witness_ctx();
    let b = Band::new(-2.0, 2.0, GRID).expect("the in-dist band");

    // A motor from named parts, so each row differs from the witness in exactly
    // one way. `half` is the rotor's half-angle Param, `plane` its bivector.
    fn motor_from(j1: &str, j2: &str, plane: u8, half: f64, l1: f64, l2: f64) -> GeoExpr {
        let rotor = |j: &str| {
            GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
                Box::new(GeoExpr::GeoProduct(
                    Box::new(GeoExpr::Param(half)),
                    Box::new(GeoExpr::Var(j.to_owned())),
                )),
                Box::new(GeoExpr::Basis(plane)),
            )))
        };
        let tr = |l: f64| {
            GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
                Box::new(GeoExpr::Param(0.5 * l)),
                Box::new(GeoExpr::Basis(E1E0)),
            )))
        };
        let limb = |j: &str, l: f64| GeoExpr::GeoProduct(Box::new(rotor(j)), Box::new(tr(l)));
        GeoExpr::GeoProduct(Box::new(limb(j1, l1)), Box::new(limb(j2, l2)))
    }
    let sandwich = |m: GeoExpr| GeoExpr::Sandwich(Box::new(m), Box::new(GeoExpr::Basis(ORIGIN)));

    // The exact constants SPEC-0022 §2 quotes — recorded here, not in prose.
    let rows: [(&str, GeoExpr); 4] = [
        (
            "witness",
            sandwich(motor_from("t1", "t2", E12, -0.5, ARM.l1, ARM.l2)),
        ),
        (
            "wrong plane e13",
            sandwich(motor_from("t1", "t2", 5, -0.5, ARM.l1, ARM.l2)),
        ),
        (
            "garbage params",
            sandwich(motor_from("t1", "t2", E12, 1.7, 2.9, 4.1)),
        ),
        (
            "swapped angles",
            sandwich(motor_from("t2", "t1", E12, -0.5, ARM.l1, ARM.l2)),
        ),
    ];

    let mut wrong = 0usize;
    for (name, expr) in &rows {
        let report = measure(expr, &ARM, b).expect("every counterexample evaluates");
        let grades = typecheck(expr, &ctx);
        println!(
            "  {name:<16} typecheck={grades:?}  rmse={:.4e}",
            report.rmse
        );
        assert_eq!(
            grades,
            Ok(GradeSet::singleton(3)),
            "{name} must ALSO typecheck Ok({{3}}) — that is the whole point of \
             SPEC-0022 §2: the point-space clause does not distinguish these",
        );
        if *name == "witness" {
            assert!(report.rmse < 1e-15, "the witness must still be exact");
        } else {
            assert!(
                report.rmse > 1e-2,
                "{name} must be measurably wrong (got {:.4e}) — otherwise it is \
                 not a counterexample and §2's table is misleading",
                report.rmse,
            );
            wrong += 1;
        }
    }
    assert_eq!(wrong, 3, "all three wrong-motor rows must be measured");

    // The two rows that are not motors at all: both `Ok({3})`, neither usable.
    let reflection = sandwich(GeoExpr::Basis(1));
    let null = sandwich(GeoExpr::Basis(8));
    for (name, expr) in [("reflection e1", &reflection), ("null e0", &null)] {
        assert_eq!(
            typecheck(expr, &ctx),
            Ok(GradeSet::singleton(3)),
            "{name} must typecheck Ok({{3}}) — the point-space clause admits it",
        );
        assert_ne!(
            typecheck(expr, &ctx),
            Ok(even_grades()),
            "{name} must FAIL the even clause — that is what excludes it",
        );
    }
    // The null row is degenerate in the readout; the even clause catches it
    // before anything divides by a zero weight.
    let mv = sample(&null, 0.9, -0.4).expect("e₀ evaluates");
    let (x, y) = read_xy(&mv);
    assert!(
        x.is_nan() && y.is_nan(),
        "Sandwich(e₀, e₁₂₃) is the zero multivector; its readout must be NaN, \
         got ({x}, {y}) — SPEC-0022 §2",
    );
}

// ---------------------------------------------------------------------------
// AC5 — equivariance, in the only sense the word is earned
// ---------------------------------------------------------------------------

/// **AC5** (SPEC-0022 §5.7/§7 Q3) — T-reparametrisation.
/// `FK(t1 + a, t2) == R_a · FK(t1, t2) · R_a~` over a 40×40 lattice with
/// `a = 0.37`: rotating the base frame rotates the end-effector by the same
/// motor, and the angle addition is performed **by the algebra** — nothing
/// outside the `GeoExpr` ever forms `t1 + t2`.
///
/// This is the *whole* of what AC5's "equivariant" means here. SPEC-0022 §7 Q3 is
/// explicit that the word is not earned in the GATr sense — the inputs are
/// grade-0 scalars and the rotation group acts trivially on them — so every use
/// must read "equivariant with respect to base-frame rotation" and cite this
/// test. Unqualified, the adjective is dropped: R-0022 §4, "no unevidenced
/// adjective".
///
/// The equality §5.7 states is between *multivectors*, so both the readout and
/// the raw blade coefficients are compared.
#[test]
fn reparametrisation() {
    let witness = fk_witness(ARM.l1, ARM.l2);
    // R_a = Exp(−½·a·e₁₂) — built from the same forms the witness's own rotors
    // use, so the group action is not a second implementation of rotation.
    let r_a = at(
        &GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
            Box::new(GeoExpr::Param(-0.5 * SHIFT)),
            Box::new(GeoExpr::Basis(E12)),
        ))),
        0.0,
        0.0,
    );

    let axis = grid(IN_DIST.0, IN_DIST.1, REPARAM_GRID);
    let mut worst_xy = 0.0_f64;
    let mut worst_coeff = 0.0_f64;
    let mut motion = 0.0_f64;
    for &t1 in &axis {
        for &t2 in &axis {
            let base = at(&witness, t1, t2);
            let shifted = at(&witness, t1 + SHIFT, t2);
            let rotated = r_a.sandwich(&base);
            let (lx, ly) = read_xy(&shifted);
            let (rx, ry) = read_xy(&rotated);
            let (bx, by) = read_xy(&base);
            worst_xy = worst_xy.max((lx - rx).abs()).max((ly - ry).abs());
            worst_coeff = worst_coeff.max(max_coeff_dev(&shifted, &rotated));
            motion = motion.max((lx - bx).hypot(ly - by));
        }
    }
    println!(
        "AC5 reparametrisation a={SHIFT}, {REPARAM_GRID}x{REPARAM_GRID}: \
         max |Δ(x,y)| = {worst_xy:e}, max |Δcoeff| = {worst_coeff:e}, \
         max actual displacement = {motion:.4}"
    );
    // The reachability pre-check (`docs/conventions.md` — a negative must be
    // clean, not confounded). `FK ≡ origin` satisfies the identity vacuously:
    // `0 == R_a·0·R_a~`. Measured on a reflection-collapsed motor the two
    // assertions below still pass, so without this the AC5 evidence would be
    // worthless. Measured displacement 0.6252 — the chord `2·r·sin(a/2)` at the
    // arm's reach `r ≤ l1 + l2 = 1.7`.
    assert!(
        motion > 0.5,
        "AC5 is vacuous unless the shift actually moves the end-effector: max \
         displacement was only {motion:e}. A motor that collapses every input to \
         the origin satisfies FK(t1+a,t2) == R_a·FK(t1,t2)·R_a~ trivially",
    );
    assert!(
        worst_xy < REPARAM_TOL,
        "AC5: FK(t1+a, t2) and R_a·FK(t1,t2)·R_a~ disagree by {worst_xy:e} in the \
         readout (bound {REPARAM_TOL:e}; 1.1102e-15 when the bound was chosen). \
         Without this test \"equivariant\" is an unevidenced adjective and \
         R-0022 §4 says to drop it",
    );
    assert!(
        worst_coeff < REPARAM_TOL,
        "AC5: the two multivectors disagree by {worst_coeff:e} blade-wise (bound \
         {REPARAM_TOL:e}; 1.1102e-15 when the bound was chosen) — §5.7's equality \
         is between multivectors, not just between readouts",
    );
}

// ---------------------------------------------------------------------------
// No AC — the durable facts (SPEC-0022 §5.8, §5.10, §6)
// ---------------------------------------------------------------------------

/// **SPEC-0022 §6** (§5.8) — T-null-blade-addition. The form set has **no
/// addition form**, yet FK is a sum of two rotated vectors. The mechanism that
/// resolves it: `Basis(9) = e₁e₀` is null, so `Exp(u·E)` truncates to `1 + u·E`
/// exactly, and multiplying two such factors **adds their arguments**. Addition,
/// obtained from a multiplicative form set via a null blade.
///
/// Asserted **bitwise**, because "exactly" here is measured, not approximate —
/// including at `(1e3, −1e3)`, where the sum cancels to zero.
#[test]
fn null_blade_addition() {
    let null_blade = || Box::new(GeoExpr::Basis(E1E0));

    // The premise: e₁e₀ squares to exactly zero, which is why `exp` truncates.
    assert_eq!(
        at(&GeoExpr::GeoProduct(null_blade(), null_blade()), 0.0, 0.0),
        Mv::zero(),
        "the mechanism rests on Basis({E1E0})² = 0; without it `exp` would not \
         truncate and the translators would not be exact",
    );

    let exp_of = |u: f64| {
        GeoExpr::Exp(Box::new(GeoExpr::GeoProduct(
            Box::new(GeoExpr::Param(u)),
            null_blade(),
        )))
    };
    for (a, b) in [
        (0.5, 0.35),
        (1.0, 0.7),
        (-0.25, 3.0),
        (1e3, -1e3),
        (0.0, 0.0),
    ] {
        let product = at(
            &GeoExpr::GeoProduct(Box::new(exp_of(a)), Box::new(exp_of(b))),
            0.0,
            0.0,
        );
        let summed = at(&exp_of(a + b), 0.0, 0.0);
        for (i, (p, s)) in product
            .coeffs
            .as_slice()
            .iter()
            .zip(summed.coeffs.as_slice().iter())
            .enumerate()
        {
            assert_eq!(
                p.to_bits(),
                s.to_bits(),
                "Exp({a}·e₁e₀) ∗ Exp({b}·e₁e₀) must be BITWISE Exp({}·e₁e₀); \
                 blade {i} differs ({p:e} vs {s:e})",
                a + b,
            );
        }
    }
    println!("§6 null-blade addition: bitwise exact on all five argument pairs");
}

/// **No AC** (SPEC-0022 §5.10/§1) — T-far-band. `[−8,8]²` is four times outside
/// the training range and was added to test the claim harder. The error does not
/// move: that is not generalization, it is the exact map. Kept because it is the
/// strongest single number in the result.
#[test]
fn far_band() {
    let in_dist = band("in-dist", IN_DIST).rmse;
    let far = band("far", FAR).rmse;
    let ratio = far / in_dist;
    println!("far [-8,8]² RMSE = {far:e} (in-dist {in_dist:e}, {ratio:.3}x)");
    assert!(
        far < EXACT,
        "far-band RMSE {far:e} exceeds {EXACT:e} (1.8329e-16 when the bound was \
         chosen). Note SPEC-0022 §4.1: this bound is scoped to the AC bands \
         is scoped to |θ| ≤ 8 and would legitimately fail on [−10³,10³]",
    );
    assert!(
        (0.1..=10.0).contains(&ratio),
        "at four times outside the training band the error must not move: \
         {in_dist:e} → {far:e} is {ratio:.3}x",
    );
}

// ---------------------------------------------------------------------------
// AC3 — the comparison, honestly reported. The deliverable artifact.
// ---------------------------------------------------------------------------

/// **AC3** (SPEC-0022 §5.9) — T-comparison. *The* deliverable: one command that
/// prints R-0022 §2's MLP table plus the witness's row, unconditionally.
///
/// Per `docs/conventions.md` *Assert the Protocol, Not the Outcome*, the
/// committed assertion is that every pre-registered row was **recorded** — never
/// that the witness won or that the MLP collapsed. A documented negative is a
/// first-class green (R-0022 AC6), and CI must not create pressure to keep tuning
/// until the answer is the pleasant one.
///
/// **Pre-registered before the run**, nothing below chosen after seeing a result:
/// arm `{l1: 1.0, l2: 0.7}`; MLP widths `{2,4,8,16,32,64}` at seed 42 with
/// `TrainConfig::default()` (700 epochs, 4,000 train samples); the stability probe
/// at `H=64`, `epochs=5000`, seeds `0..5`; witness bands `[−2,2]²`, `[2,3]²`,
/// `[−8,8]²` and the `[−10³,10³]²` boundary row, each on a closed 60×60 lattice.
/// Measured wall clock ≈ 75 s in release (4 s sweep + 67 s five seeds).
#[test]
#[ignore = "release e2e: cargo test -p ufl-evolve --release --test r_0022_gate2_witness -- --ignored --nocapture"]
fn comparison_artifact() {
    let motor = fk_motor(ARM.l1, ARM.l2);
    let witness = fk_witness(ARM.l1, ARM.l2);
    let ctx = witness_ctx();

    println!("=== R-0022 / SPEC-0022 — Gate 2: the exact witness vs the fair MLP ===");
    println!("arm: l1 = {}, l2 = {}", ARM.l1, ARM.l2);
    println!("metric: per-component RMSE = sqrt(Σ[(x−x̂)² + (y−ŷ)²] / 2n) — baseline.rs:295,");
    println!("        the SAME denominator for both sides. A per-POINT figure (denominator n)");
    println!("        is exactly √2 larger and never shares a table with this one.");
    println!(
        "grid:   deterministic closed {GRID}×{GRID} lattice per band = {} samples",
        GRID * GRID
    );

    println!("\n-- the witness, structurally --");
    println!("  nodes(motor)   = {}", node_count(&motor));
    println!("  nodes(witness) = {}", node_count(&witness));
    println!(
        "  Params         = {} = {:?}  (derived from ground truth, NOT fitted)",
        param_count(&witness),
        params(&witness),
    );
    println!(
        "  typecheck(motor,   ctx)       = {:?}   [EVEN: excludes reflections and odd blades]",
        typecheck(&motor, &ctx),
    );
    println!(
        "  typecheck(witness, ctx)       = {:?}      [POINT SPACE: PGA grade 3, all inputs]",
        typecheck(&witness, &ctx),
    );
    println!(
        "  typecheck(witness, EMPTY ctx) = {:?}  [the proof is conditional on the {{0}} declaration]",
        typecheck(&witness, &GradeCtx::new()),
    );
    match motor_unit_check(&motor, 0.7, -0.4) {
        Ok((scalar, residue)) => println!(
            "  unit: ⟨M∗M̃⟩₀ = {scalar:.17}, non-scalar residue = {residue:e}   \
             [excludes null and scaled versors]"
        ),
        Err(e) => println!("  unit: FAILED to evaluate — {e}"),
    }
    println!(
        "  LIMIT of the structural claim: Sandwich(e₁, e₁₂₃) also typechecks {:?} and is a",
        typecheck(
            &GeoExpr::Sandwich(
                Box::new(GeoExpr::Basis(1)),
                Box::new(GeoExpr::Basis(ORIGIN))
            ),
            &ctx,
        ),
    );
    println!("  REFLECTION. Structure proves *a* proper rigid motion yielding a point; WHICH one");
    println!("  is the numerics' job. Each half carries half the claim (SPEC-0022 §2.1).");

    println!("\n-- the witness, numerically (0 evaluation failures per band) --");
    let mut witness_rows = Vec::new();
    for (name, bounds) in [
        ("in-dist [-2,2]²  ", IN_DIST),
        ("ood     [2,3]²   ", OOD),
        ("far     [-8,8]²  ", FAR),
        ("boundary [-1e3,1e3]²", BOUNDARY),
    ] {
        let r = band(name.trim(), bounds);
        println!(
            "  {name}  RMSE = {:e}   max|w−1| = {:e}   max|z| = {:e}",
            r.rmse, r.max_weight_error, r.max_z,
        );
        witness_rows.push((name, r));
    }
    let (in_dist, ood) = (witness_rows[0].1.rmse, witness_rows[1].1.rmse);
    println!(
        "  witness OOD ÷ in-dist = {:.3}x      <-- THE HEADLINE",
        ood / in_dist
    );
    println!("  the boundary row is why no absolute 1e-14 bound is asserted — and it is");
    println!("  the f64 REFERENCE drifting, not the witness: measured against an 80-digit");
    println!("  reference the witness is flat at ~1.4e-16 across nine orders of magnitude of |θ|,");
    println!("  while ArmFk::forward reaches 1.43e-8 at 1e9. The whole growth is the f64 rounding");
    println!("  of t1+t2 at baseline.rs:27 — the one operation the witness never performs.");
    println!("  Rebuild: cargo run -p ufl-evolve --release --example boundary_samples \\");
    println!("           | python3 experiments/0022-exact-fk-reference.py");

    println!("\n-- the fair MLP: seed 42, TrainConfig::default() (700 epochs, 4000 train) --");
    let widths = [2usize, 4, 8, 16, 32, 64];
    let reports = sweep(&ARM, &widths, 42);
    println!("   H  params   train RMSE   in-dist RMSE     OOD RMSE   OOD/in-dist");
    for r in &reports {
        println!(
            "  {:>2}  {:>6}     {:.3e}      {:.3e}    {:.3e}      {:>6.1}x",
            r.hidden,
            r.params,
            r.train_rmse,
            r.test_rmse,
            r.ood_rmse,
            r.ood_rmse / r.test_rmse,
        );
    }

    println!("  ONE SEED. Read no floor off this table: at H=32 seed 42 draws OOD 1.935e-1,");
    println!("  about 4 SD below that width's 8-seed mean — an outlier, not a capability.");
    println!("  The across-seed block below is what pins the floor.");

    println!("\n-- the floor, across seeds (widths × seeds 0..7, default config) --");
    println!("   H  params     OOD min      OOD mean      OOD max");
    let mut floor_rows = Vec::new();
    for h in [16usize, 32, 64] {
        let oods: Vec<f64> = (0..8u64)
            .map(|seed| train_report_with(&ARM, h, seed, &TrainConfig::default()).ood_rmse)
            .collect();
        let lo = oods.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = oods.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mean = oods.iter().sum::<f64>() / oods.len() as f64;
        println!(
            "  {h:>2}  {:>6}   {lo:.3e}     {mean:.3e}    {hi:.3e}",
            2 + h * 5
        );
        floor_rows.push((h, lo, mean, hi));
    }
    println!("  No width's 8-seed minimum goes below 2.9e-1 at the default budget; the");
    println!("  best OOD measured in ANY config or seed is 1.9e-1 — fifteen orders of");
    println!("  magnitude above the witness, so the headline does not depend on the floor's");
    println!("  exact value. It is stated as a range because a single draw is not a floor.");

    println!("\n-- smallest-at-error (SPEC-0011 §2.5's anti-strawman rule) --");
    let mut selections = Vec::new();
    for target in [0.05_f64, 0.01] {
        let pick = smallest_at(&reports, target);
        match pick {
            Some(r) => println!(
                "  in-dist ≤ {target}: H={} ({} params), OOD {:.3e}",
                r.hidden, r.params, r.ood_rmse,
            ),
            None => println!("  in-dist ≤ {target}: no width in the sweep reaches it"),
        }
        selections.push((target, pick.map(|r| (r.hidden, r.params))));
    }

    println!("\n-- seed stability at H=64, 5000 epochs, seeds 0..5 (SPEC-0022 §7 Q2) --");
    let long = TrainConfig {
        epochs: 5000,
        ..TrainConfig::default()
    };
    let mut seed_oods = Vec::new();
    for seed in 0..5u64 {
        let r = train_report_with(&ARM, 64, seed, &long);
        println!(
            "  seed {seed}: in-dist {:.3e}  OOD {:.3e}  ({:.1}x)",
            r.test_rmse,
            r.ood_rmse,
            r.ood_rmse / r.test_rmse,
        );
        seed_oods.push(r.ood_rmse);
    }
    let lo = seed_oods.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = seed_oods.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!("  OOD floor across 5 seeds: {lo:.3e} … {hi:.3e} — seed-stable, which is what");
    println!("  makes SPEC-0022 §4.2's \"structural, not under-training\" load-bearing.");

    println!("\n-- what this does NOT claim (R-0022 §4, SPEC-0022 §4.1) --");
    println!("  * NOT evolved. The witness is hand-derived; R-0011 §2.8's stretch is untouched.");
    println!("  * NOT trig-free. The rotors' Exp computes cos/sin internally. The claim is that");
    println!("    the ANGLE ADDITION is done by the algebra and the map is exactly expressible.");
    println!("  * NOT a learning claim. The 4 Params are derived from ground-truth constants;");
    println!("    the MLP's 322 are fitted from labelled samples.");
    println!("  * NOT equivariant unqualified. Only w.r.t. base-frame rotation (see AC5's test).");
    println!("  * NOT exact everywhere — see the boundary row above.");

    // *The protocol, not the outcome.* Every pre-registered row ran and was
    // recorded. Nothing here asserts that the witness beat the MLP.
    assert_eq!(
        witness_rows.len(),
        4,
        "all three AC bands plus the boundary row must be recorded",
    );
    assert!(
        witness_rows.iter().all(|(_, r)| r.rmse.is_finite()),
        "each witness band must record a finite RMSE",
    );
    assert_eq!(
        reports.len(),
        widths.len(),
        "every pre-registered MLP width must train and report",
    );
    assert!(
        reports
            .iter()
            .zip(widths.iter())
            .all(|(r, &h)| r.hidden == h && r.params == 5 * h + 2),
        "each MLP row must report its own width and param count",
    );
    assert_eq!(
        selections.len(),
        2,
        "both smallest-at-error targets must be decided (Some or None)",
    );
    assert_eq!(seed_oods.len(), 5, "all five stability seeds must report");
    assert_eq!(
        floor_rows.len(),
        3,
        "all three across-seed floor widths must report",
    );
    assert!(
        floor_rows
            .iter()
            .all(|&(_, lo, mean, hi)| lo <= mean && mean <= hi && lo.is_finite()),
        "each floor row must record an ordered, finite min/mean/max",
    );
}
