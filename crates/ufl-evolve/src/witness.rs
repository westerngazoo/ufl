//! SPEC-0022 — the **Gate-2 witness**: forward kinematics of a planar 2R arm,
//! written once as an exact [`GeoExpr`], plus the measurement harness the
//! comparison artifact prints.
//!
//! The claim this module exists to make checkable:
//!
//! > With the right algebraic structure this map is **exactly expressible** in
//! > 4 parameters and stays exact wherever `f64` can represent the angle; a
//! > generic function approximator needs 322 and holds only where it was
//! > trained.
//!
//! Two things a reader will look for first, so both are settled here:
//!
//! - **The trigonometry is inside `Exp`, not avoided.** A rotor is
//!   `Exp(−t/2 · e₁₂)` and `e₁₂² = −1`, so garust's `exp` takes its `c < 0`
//!   branch and computes `cos(t/2)`/`sin(t/2)`. What is *not* done anywhere
//!   outside the expression is the **angle addition**: the motor product
//!   composes the two rotations, so nothing in this module ever forms `t1 + t2`.
//! - **The readout is the kernel's.** [`read_xy`] delegates to garust's own
//!   `Point::from_multivector(..).to_euclidean()` (SPEC-0022 §4.3) rather than
//!   reading coefficients by hand, so "the verifier does no geometry" is
//!   enforced by the compiler instead of argued in prose.
//!
//! What the grade system does and does not prove is [`witness_ctx`]'s doc and
//! SPEC-0022 §2: `typecheck(fk_witness(..)) == Ok({3})` proves the output lies
//! in point space, **not** that the sandwich is a rigid motion. The rigid-motion
//! half needs [`fk_motor`] to be provably *even* and *unit* as well.

use ufl_ga::Point;
use ufl_geo::{eval, Env, GeoError, GeoExpr, GradeCtx, GradeSet, Mv};

/// What can go wrong while measuring a band.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum WitnessError {
    /// A grid with fewer than two points per axis. Rejected rather than
    /// averaged: `n = 0` divided by a clamped denominator and reported
    /// **`rmse: 0.0`** — a silent perfect score from the one function the
    /// headline rests on — while `n = 1` samples a single corner but looks
    /// like a sweep.
    #[error("a band needs at least 2 grid points per axis, got {0}")]
    DegenerateGrid(usize),
    /// A band whose bounds do not strictly increase, or are incomparable.
    /// `lo == hi` would report a
    /// healthy-looking `n × n` sweep of a **single pose** — the same
    /// silent-perfect-score class as `n = 0`, which is why both are rejected
    /// at the same seam rather than documented as preconditions.
    #[error("a band needs lo < hi, got lo = {lo}, hi = {hi}")]
    DegenerateBand {
        /// The rejected lower bound.
        lo: f64,
        /// The rejected upper bound.
        hi: f64,
    },
    /// A sample failed to evaluate. An evaluation failure is a result, not
    /// something to average away.
    #[error(transparent)]
    Eval(#[from] GeoError),
}

/// `e₁₂` — the rotation plane of a planar arm (garust blade index 3).
pub const E12: u8 = 3;
/// `e₁₂₃` — the PGA origin point, the blade the motor sandwiches (index 7).
pub const ORIGIN: u8 = 7;
/// `e₁e₀` — a **null** bivector (`Basis(9)² = 0`), so `Exp` truncates to
/// `1 + u·e₁e₀` with no transcendental at all (SPEC-0022 §4.1).
pub const E1E0: u8 = 9;

/// The grade set a proper (even) versor must have: `{0, 2, 4}`.
///
/// Asserting this on [`fk_motor`] is what excludes a **reflection** — an
/// improper motion the `Ok({3})` check alone happily accepts (SPEC-0022 §2).
pub fn even_grades() -> GradeSet {
    GradeSet::singleton(0).with(2).with(4)
}

fn product(a: GeoExpr, b: GeoExpr) -> GeoExpr {
    GeoExpr::GeoProduct(Box::new(a), Box::new(b))
}

/// `R(t) = Exp((−½·t)·e₁₂)` — garust's rotor convention is `exp(−½·θ·plane)`.
///
/// **6 nodes, 1 `Param`.** The `−0.5` is a free slot, not a constant folded in
/// by the harness: a search tuning it changes the rotation rate.
fn rotor(joint: &str) -> GeoExpr {
    GeoExpr::Exp(Box::new(product(
        product(GeoExpr::Param(-0.5), GeoExpr::Var(joint.to_owned())),
        GeoExpr::Basis(E12),
    )))
}

/// `T(L) = Exp((L/2)·e₁e₀)` — a translator along e₁ by `L`.
///
/// **4 nodes, 1 `Param`.** [`E1E0`] is null, so this evaluates through `exp`'s
/// `c == 0` closed form and truncates exactly after one term. The half-angle
/// factor and the link length fold into the single `Param`.
fn translator(link: f64) -> GeoExpr {
    GeoExpr::Exp(Box::new(product(
        GeoExpr::Param(0.5 * link),
        GeoExpr::Basis(E1E0),
    )))
}

/// The motor `M = R(t1)·T(l1)·R(t2)·T(l2)` — **23 nodes, 4 `Param`s**.
///
/// 23 = two limbs of `rotor` (6) + `translator` (4) + their joining
/// `GeoProduct` (1), plus the product that joins the limbs. The witness adds
/// its `Sandwich` and `Basis(7)` for 25.
///
/// Exposed separately from [`fk_witness`] because SPEC-0022 §2.1's structural
/// guard is stated about *this* subtree: `M` is provably **even**
/// ([`even_grades`]) and measurably **unit** ([`motor_unit_check`]), and those
/// two together are what `Ok({3})` does not give on its own.
pub fn fk_motor(l1: f64, l2: f64) -> GeoExpr {
    let limb = |joint: &str, link: f64| product(rotor(joint), translator(link));
    product(limb("t1", l1), limb("t2", l2))
}

/// The witness: `Sandwich(M, e₁₂₃)` — **25 nodes, 4 `Param`s**
/// = `[−0.5, l1/2, −0.5, l2/2]`.
///
/// The rightmost factor acts first, so the origin walks out link 2, swings by
/// `t2`, walks out link 1, swings by `t1`.
pub fn fk_witness(l1: f64, l2: f64) -> GeoExpr {
    GeoExpr::Sandwich(Box::new(fk_motor(l1, l2)), Box::new(GeoExpr::Basis(ORIGIN)))
}

/// The grade context the structural proof rests on: `t1` and `t2` are grade-0.
///
/// **This declaration is load-bearing and unchecked.** With an empty context
/// both variables are ⊤ and `typecheck` returns `Ok({0,1,2,3,4})` — no proof at
/// all. Nothing in `ufl-geo` verifies that a declaration matches what [`Env`]
/// actually binds, so a caller that declares `{0}` and binds a bivector gets an
/// unsound answer. [`witness_env`] is the matching binder, and the acceptance
/// suite asserts the two agree (SPEC-0022 §2.3).
pub fn witness_ctx() -> GradeCtx {
    let mut ctx = GradeCtx::new();
    ctx.declare("t1", GradeSet::singleton(0));
    ctx.declare("t2", GradeSet::singleton(0));
    ctx
}

/// Bind the two joint angles as grade-0 multivectors — the binding
/// [`witness_ctx`] declares.
pub fn witness_env(t1: f64, t2: f64) -> Env {
    let mut env = Env::new();
    env.bind("t1", Mv::scalar(t1));
    env.bind("t2", Mv::scalar(t2));
    env
}

/// Read the Euclidean `(x, y)` out of a PGA point — **garust's own readout**.
///
/// Takes only `&Mv`, so it *cannot* see `t1`/`t2`: the type signature is the
/// proof that no geometry happens outside the expression. The homogeneous
/// divide inside `to_euclidean` is a measured no-op for the witness
/// (`max abs(w − 1)` ≈ 4.4e-16) and is required by SPEC-0011 §2.3, because an
/// evolver's intermediates are not rigid and can produce an ideal point.
pub fn read_xy(mv: &Mv) -> (f64, f64) {
    let (x, y, _z) = Point::from_multivector(*mv).to_euclidean();
    (x, y)
}

/// The z coordinate — exactly `0.0` for the planar witness at every sample, a
/// structural cross-check the numerics cannot fake (SPEC-0022 §5.6).
pub fn read_z(mv: &Mv) -> f64 {
    let (_x, _y, z) = Point::from_multivector(*mv).to_euclidean();
    z
}

/// Total node count of an expression.
pub fn node_count(e: &GeoExpr) -> usize {
    1 + children(e)
        .into_iter()
        .flatten()
        .map(node_count)
        .sum::<usize>()
}

/// How many `Param` slots an expression carries — the witness's headline "4".
///
/// Delegates to `ufl_geo::params`, the slot enumeration the evolver's refiner
/// already uses, rather than re-walking the tree: one traversal, one answer.
pub fn param_count(e: &GeoExpr) -> usize {
    ufl_geo::params(e).len()
}

/// The children of a node, as the single arity source (the R-0020 convention:
/// one place decides arity, so a new variant cannot silently be walked wrong).
fn children(e: &GeoExpr) -> [Option<&GeoExpr>; 2] {
    match e {
        GeoExpr::Param(_) | GeoExpr::Basis(_) | GeoExpr::Var(_) => [None, None],
        GeoExpr::GradeLift(_, a)
        | GeoExpr::GradeProject(_, a)
        | GeoExpr::Reverse(a)
        | GeoExpr::Exp(a) => [Some(a), None],
        GeoExpr::GeoProduct(a, b)
        | GeoExpr::Wedge(a, b)
        | GeoExpr::Inner(a, b)
        | GeoExpr::Sandwich(a, b) => [Some(a), Some(b)],
    }
}

/// One sample of the witness: the evaluated multivector at `(t1, t2)`.
pub fn sample(expr: &GeoExpr, t1: f64, t2: f64) -> Result<Mv, GeoError> {
    eval(expr, &witness_env(t1, t2))
}

/// A validated angle band: the closed `lo..=hi` lattice with `n` points per
/// axis, swept as `n × n` poses.
///
/// Constructed only through [`Band::new`], so a degenerate grid or band cannot
/// reach [`measure`] at all — both failure modes report a healthy RMSE over
/// nothing, which is the one thing a measurement harness must never do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    lo: f64,
    hi: f64,
    n: usize,
}

impl Band {
    /// A band, or the reason it is not one.
    ///
    /// Rejects `n < 2` ([`WitnessError::DegenerateGrid`]) and `lo >= hi`
    /// ([`WitnessError::DegenerateBand`]).
    pub fn new(lo: f64, hi: f64, n: usize) -> Result<Self, WitnessError> {
        if n < 2 {
            return Err(WitnessError::DegenerateGrid(n));
        }
        if !matches!(lo.partial_cmp(&hi), Some(core::cmp::Ordering::Less)) {
            // `partial_cmp` rather than `!(lo < hi)`: it makes the NaN case
            // explicit rather than incidental. A NaN bound is incomparable, so
            // it is rejected here instead of producing a band of NaN poses.
            return Err(WitnessError::DegenerateBand { lo, hi });
        }
        Ok(Self { lo, hi, n })
    }

    /// Inclusive lower bound of both angles.
    pub fn lo(&self) -> f64 {
        self.lo
    }

    /// Inclusive upper bound of both angles.
    pub fn hi(&self) -> f64 {
        self.hi
    }

    /// Grid resolution per axis.
    pub fn n(&self) -> usize {
        self.n
    }

    /// The `i`th lattice coordinate. `n >= 2` is an invariant, so the divisor
    /// is never zero.
    fn step(&self, i: usize) -> f64 {
        self.lo + (self.hi - self.lo) * i as f64 / (self.n - 1) as f64
    }
}

/// What a deterministic sweep over one angle band measured.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BandReport {
    /// The band swept.
    pub band: Band,
    /// **Per-component** RMSE against [`crate::baseline::ArmFk::forward`] —
    /// `sqrt(Σ[(Δx)² + (Δy)²] / 2n)`, `baseline.rs`'s own formula (SPEC-0022
    /// §1). Never the per-point variant; the two differ by √2 and rev 2 of the
    /// spec mixed them.
    pub rmse: f64,
    /// `max abs(w − 1)` over the band — how close the homogeneous weight stays
    /// to 1, i.e. how nearly the divide in [`read_xy`] is a no-op.
    pub max_weight_error: f64,
    /// `max abs(z)` over the band — exactly `0.0` for a planar arm.
    pub max_z: f64,
}

/// Sweep `expr` over `band` against `arm` and report it. Deterministic: the
/// grid is a closed `lo..=hi` lattice, so a run is reproducible from the
/// `Band` alone.
///
/// **Takes the expression rather than building it.** SPEC-0022 §2's
/// counterexample table — the wrong rotor plane, garbage `Param`s, swapped
/// angles — is measurable through this harness precisely because the
/// expression is a parameter. An earlier version hard-coded [`fk_witness`],
/// which is why those rows had to be produced off-repo and could not be
/// rebuilt from it.
///
/// Returns [`WitnessError::Eval`] if any sample fails to evaluate — an
/// evaluation failure is a result, not something to average away.
pub fn measure(
    expr: &GeoExpr,
    arm: &crate::baseline::ArmFk,
    band: Band,
) -> Result<BandReport, WitnessError> {
    let (mut se, mut max_weight_error, mut max_z) = (0.0f64, 0.0f64, 0.0f64);
    for i in 0..band.n {
        for j in 0..band.n {
            let (t1, t2) = (band.step(i), band.step(j));
            let mv = sample(expr, t1, t2)?;
            max_weight_error = max_weight_error.max((weight(&mv) - 1.0).abs());
            max_z = max_z.max(read_z(&mv).abs());
            let (x, y) = read_xy(&mv);
            let (gx, gy) = arm.forward(t1, t2);
            se += (x - gx).powi(2) + (y - gy).powi(2);
        }
    }
    let count = (band.n * band.n) as f64;
    Ok(BandReport {
        band,
        rmse: (se / (count * 2.0)).sqrt(),
        max_weight_error,
        max_z,
    })
}

/// Sweep the witness for `arm` over `band` — the common case, and the one the
/// acceptance bands use.
pub fn measure_witness(
    arm: &crate::baseline::ArmFk,
    band: Band,
) -> Result<BandReport, WitnessError> {
    measure(&fk_witness(arm.l1, arm.l2), arm, band)
}

/// The homogeneous weight of a PGA point — the `e₁₂₃` coefficient.
fn weight(mv: &Mv) -> f64 {
    mv.coeffs.as_slice()[usize::from(ORIGIN)]
}

/// `(scalar part, summed non-scalar residue)` of `M ∗ M̃` — the **unit** half
/// of SPEC-0022 §2.1's guard.
///
/// The residue is the **sum** of all fifteen non-scalar coefficient
/// magnitudes, not their maximum. A caller comparing it against a per-blade
/// tolerance is therefore applying a bound up to 15× tighter than it looks —
/// the safe direction, but say so rather than let a reader assume otherwise.
///
/// A proper unit versor gives `(1, 0)`. The null `Basis(8)` gives `(0, 0)`, so
/// the scalar check is what excludes it; a reflection gives `(1, 0)` too, so
/// the *even* check is what excludes that. Both are needed.
pub fn motor_unit_check(motor: &GeoExpr, t1: f64, t2: f64) -> Result<(f64, f64), GeoError> {
    let m = sample(motor, t1, t2)?;
    let mm = m * m.reverse();
    let coeffs = mm.coeffs.as_slice();
    let scalar = coeffs[0];
    let residue = coeffs.iter().skip(1).map(|c| c.abs()).sum();
    Ok((scalar, residue))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ufl_geo::typecheck;

    /// The arm the spec's figures are quoted for.
    const L1: f64 = 1.0;
    const L2: f64 = 0.7;
    const ARM: crate::baseline::ArmFk = crate::baseline::ArmFk { l1: L1, l2: L2 };

    #[test]
    fn node_and_param_counts_are_the_headline_numbers() {
        // Leaves.
        assert_eq!(node_count(&GeoExpr::Param(0.0)), 1);
        assert_eq!(node_count(&GeoExpr::Basis(E12)), 1);
        assert_eq!(node_count(&GeoExpr::Var("t1".to_owned())), 1);
        // Unary and binary arities, so `children` is exercised on both shapes.
        assert_eq!(node_count(&GeoExpr::Exp(Box::new(GeoExpr::Param(1.0)))), 2);
        assert_eq!(
            node_count(&product(GeoExpr::Param(1.0), GeoExpr::Param(2.0))),
            3
        );
        // Sub-shapes with DISTINCT counts. Asserting only the witness and the
        // motor left both counters open to a constant: 4 `Param`s is the right
        // answer for *both*, and 25 nodes for the only `Sandwich` in the suite,
        // so `param_count -> 4` and `node_count(Sandwich) -> 25` each survived
        // a mutation battery. Distinct expected values defeat any constant.
        assert_eq!(
            (node_count(&rotor("t1")), param_count(&rotor("t1"))),
            (6, 1)
        );
        assert_eq!(
            (node_count(&translator(L1)), param_count(&translator(L1))),
            (4, 1)
        );
        let limb = product(rotor("t1"), translator(L1));
        assert_eq!((node_count(&limb), param_count(&limb)), (11, 2));
        // A second `Sandwich`, so the witness is not the only one measured.
        let tiny = GeoExpr::Sandwich(Box::new(GeoExpr::Param(1.0)), Box::new(GeoExpr::Param(2.0)));
        assert_eq!((node_count(&tiny), param_count(&tiny)), (3, 2));

        // The witness and its motor.
        assert_eq!(node_count(&fk_motor(L1, L2)), 23, "motor: 2×(6+4+1) + 1");
        assert_eq!(
            node_count(&fk_witness(L1, L2)),
            25,
            "witness: the motor plus Sandwich and Basis(ORIGIN)"
        );
        assert_eq!(param_count(&fk_witness(L1, L2)), 4);
        assert_eq!(
            param_count(&fk_motor(L1, L2)),
            4,
            "the Sandwich adds no free slot"
        );
    }

    #[test]
    fn the_four_params_are_the_two_half_angles_and_two_half_links() {
        // Pinning the *values* catches a limb built with the wrong link.
        fn slots(e: &GeoExpr, out: &mut Vec<f64>) {
            if let GeoExpr::Param(v) = e {
                out.push(*v);
            }
            for kid in children(e).into_iter().flatten() {
                slots(kid, out);
            }
        }
        let mut got = Vec::new();
        slots(&fk_witness(L1, L2), &mut got);
        assert_eq!(got, vec![-0.5, 0.5 * L1, -0.5, 0.5 * L2]);
    }

    #[test]
    fn a_degenerate_grid_or_band_is_an_error_not_a_perfect_score() {
        // The regressions these exist for: `n = 0` once reported `rmse: 0.0`,
        // and `lo == hi` once reported a 3,600-sample sweep of one pose.
        assert_eq!(
            Band::new(-2.0, 2.0, 0),
            Err(WitnessError::DegenerateGrid(0))
        );
        assert_eq!(
            Band::new(-2.0, 2.0, 1),
            Err(WitnessError::DegenerateGrid(1))
        );
        assert_eq!(
            Band::new(2.0, 2.0, 60),
            Err(WitnessError::DegenerateBand { lo: 2.0, hi: 2.0 })
        );
        assert_eq!(
            Band::new(3.0, 1.0, 60),
            Err(WitnessError::DegenerateBand { lo: 3.0, hi: 1.0 })
        );
        // A NaN bound is incomparable, not ordered — rejected, never swept.
        assert!(matches!(
            Band::new(f64::NAN, 2.0, 60),
            Err(WitnessError::DegenerateBand { .. })
        ));
        assert!(matches!(
            Band::new(-2.0, f64::NAN, 60),
            Err(WitnessError::DegenerateBand { .. })
        ));
        assert!(Band::new(-2.0, 2.0, 2).is_ok());
    }

    #[test]
    fn a_band_spans_its_endpoints_inclusively() {
        let band = Band::new(-2.0, 2.0, 5).expect("a valid band");
        assert_eq!((band.step(0), band.step(4)), (-2.0, 2.0), "closed lattice");
        let report = measure_witness(&ARM, band).expect("a 5×5 band evaluates");
        assert_eq!(
            (report.band.n(), report.band.lo(), report.band.hi()),
            (5, -2.0, 2.0)
        );
        // The lattice must *reach* hi, or an "OOD band" would silently omit its
        // far corner — the sample that carries the claim.
        let corner = sample(&fk_witness(L1, L2), 2.0, 2.0).expect("the far corner evaluates");
        let (x, y) = read_xy(&corner);
        assert!(x.is_finite() && y.is_finite());
    }

    #[test]
    fn the_readout_is_the_kernels_and_splits_x_y_z_consistently() {
        let mv = sample(&fk_witness(L1, L2), 0.4, -1.1).expect("evaluates");
        let (x, y) = read_xy(&mv);
        let z = read_z(&mv);
        let (kx, ky, kz) = Point::from_multivector(mv).to_euclidean();
        assert_eq!(
            (x.to_bits(), y.to_bits(), z.to_bits()),
            (kx.to_bits(), ky.to_bits(), kz.to_bits())
        );
        assert_eq!(z, 0.0, "the arm is planar");
    }

    #[test]
    fn the_witness_agrees_with_the_f64_reference_at_one_pose() {
        // A single hand-checkable pose, so a unit failure localises without
        // running a sweep.
        let (t1, t2) = (0.3, 0.9);
        let mv = sample(&fk_witness(L1, L2), t1, t2).expect("evaluates");
        let (x, y) = read_xy(&mv);
        let (gx, gy) = crate::baseline::ArmFk { l1: L1, l2: L2 }.forward(t1, t2);
        assert!(
            (x - gx).abs() < 1e-15 && (y - gy).abs() < 1e-15,
            "got ({x}, {y}), want ({gx}, {gy})"
        );
    }

    #[test]
    fn even_grades_is_exactly_zero_two_four() {
        let even = even_grades();
        for k in [0usize, 2, 4] {
            assert!(even.contains(k), "grade {k} must be in the even set");
        }
        for k in [1usize, 3] {
            assert!(!even.contains(k), "grade {k} must not be");
        }
    }

    #[test]
    fn the_structural_guard_separates_the_motor_from_a_reflection() {
        let ctx = witness_ctx();
        // Even: the motor passes, a grade-1 reflection does not.
        assert_eq!(typecheck(&fk_motor(L1, L2), &ctx), Ok(even_grades()));
        assert_eq!(
            typecheck(&GeoExpr::Basis(1), &ctx),
            Ok(GradeSet::singleton(1)),
            "e₁ is odd — this is what the even check excludes"
        );
        // Unit: the motor is unit; the null e₀ has a zero scalar part.
        let (scalar, residue) =
            motor_unit_check(&fk_motor(L1, L2), 0.9, -0.4).expect("motor evaluates");
        assert!((scalar - 1.0).abs() < 1e-15, "scalar part {scalar}");
        assert!(residue < 1e-15, "summed non-scalar residue {residue}");
        let (null_scalar, _) =
            motor_unit_check(&GeoExpr::Basis(8), 0.0, 0.0).expect("e₀ evaluates");
        assert_eq!(
            null_scalar, 0.0,
            "e₀ is null — this is what the unit check excludes"
        );

        // The residue half of the unit clause needs a fixture whose residue is
        // NOT zero, or `residue = 0.0` passes every assertion — it survived a
        // mutation battery for exactly that reason. `Exp(u·e₁)` exponentiates a
        // EUCLIDEAN vector: `e₁² = +1`, so garust takes its `c > 0` branch and
        // returns `cosh u + sinh u · e₁`, whose `M ∗ M̃ = cosh 2u + sinh 2u · e₁`
        // carries a large grade-1 part.
        let hyperbolic = GeoExpr::Exp(Box::new(product(GeoExpr::Param(0.8), GeoExpr::Basis(1))));
        let (h_scalar, h_residue) =
            motor_unit_check(&hyperbolic, 0.0, 0.0).expect("Exp(0.8·e₁) evaluates");
        assert!(
            (h_scalar - 1.6f64.cosh()).abs() < 1e-12,
            "scalar part must be cosh(1.6) = {}, got {h_scalar}",
            1.6f64.cosh()
        );
        assert!(
            (h_residue - 1.6f64.sinh()).abs() < 1e-12,
            "the residue must be sinh(1.6) = {} — a residue that always reads \
             0.0 leaves half the unit clause unverified, got {h_residue}",
            1.6f64.sinh()
        );
    }

    /// **The two kernel facts §4.1 and §2.1 rest on, pinned.**
    ///
    /// `Basis(E12)² = −1` is what forces garust's `exp` into its `c < 0`
    /// branch, which is the branch that computes `cos`/`sin` — so §4.1's "the
    /// trigonometry lives inside `Exp`" is true only while this holds. And
    /// `sandwich(r, x) == r ∗ x ∗ r̃` is what lets a check on `M ∗ M̃` say
    /// anything at all about the sandwich.
    #[test]
    fn the_kernel_identities_the_guard_depends_on() {
        let env = witness_env(0.0, 0.0);
        let rot_plane =
            eval(&product(GeoExpr::Basis(E12), GeoExpr::Basis(E12)), &env).expect("e₁₂² evaluates");
        assert_eq!(
            rot_plane.coeffs.as_slice()[0],
            -1.0,
            "e₁₂² must be −1, or Exp takes the wrong branch and no cos/sin runs"
        );
        let null_plane = eval(&product(GeoExpr::Basis(E1E0), GeoExpr::Basis(E1E0)), &env)
            .expect("(e₁e₀)² evaluates");
        assert!(
            null_plane.coeffs.as_slice().iter().all(|c| *c == 0.0),
            "e₁e₀ must be null, or Exp does not truncate"
        );

        // The sandwich is the conjugation the unit check reasons about.
        let m = sample(&fk_motor(L1, L2), 0.7, -0.3).expect("motor evaluates");
        let x = sample(&GeoExpr::Basis(ORIGIN), 0.0, 0.0).expect("origin evaluates");
        let by_hand = m * x * m.reverse();
        let by_kernel = sample(&fk_witness(L1, L2), 0.7, -0.3).expect("witness evaluates");
        assert!(
            by_hand
                .coeffs
                .as_slice()
                .iter()
                .zip(by_kernel.coeffs.as_slice())
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "Sandwich(M, e₁₂₃) must be bitwise M ∗ e₁₂₃ ∗ M̃"
        );
    }

    /// **M4 — `read_z` must not be vacuous.** Hard-wiring it to `0.0` once
    /// passed every test in the suite, including the planarity cross-check
    /// whose whole purpose it is. An out-of-plane rotor gives a non-zero z, so
    /// the constant-folded version now fails here.
    #[test]
    fn read_z_reports_a_real_coordinate_not_a_constant() {
        // The origin is a FIXED POINT of any rotor about it, so a bare e₁₃
        // rotor leaves z at −0. The point must be translated off the axis
        // first, then swung out of the plane.
        let out_of_plane = GeoExpr::Sandwich(
            Box::new(product(
                GeoExpr::Exp(Box::new(product(
                    product(GeoExpr::Param(-0.5), GeoExpr::Var("t1".to_owned())),
                    GeoExpr::Basis(5), // e₁₃ — rotates out of the e₁₂ plane
                ))),
                translator(L1),
            )),
            Box::new(GeoExpr::Basis(ORIGIN)),
        );
        let mv = sample(&out_of_plane, 1.1, 0.0).expect("evaluates");
        assert!(
            read_z(&mv).abs() > 0.1,
            "an e₁₃ rotor must move the origin out of the z = 0 plane, got {}",
            read_z(&mv)
        );
        // …while the witness itself stays planar.
        let planar = sample(&fk_witness(L1, L2), 1.1, 0.4).expect("evaluates");
        assert_eq!(read_z(&planar), 0.0);
    }

    #[test]
    fn a_bad_blade_surfaces_as_a_typed_error_not_a_panic() {
        let bad = GeoExpr::Sandwich(
            Box::new(GeoExpr::Basis(200)),
            Box::new(GeoExpr::Basis(ORIGIN)),
        );
        assert!(matches!(
            sample(&bad, 0.0, 0.0),
            Err(GeoError::BadBlade(200))
        ));
    }
}
