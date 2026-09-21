# SPEC-0022 — Gate 2's witness and the fair comparison, as one reproducible artifact

- **Realizes:** [R-0022](../requirements/0022-gate2-witness.md) (ACs approved 2026-09-16).
- **Status:** **Draft (rev 3)** — architect review returned **APPROVE WITH
  CHANGES**, one finding **blocking**. All nine are folded below; the blocking
  one **refutes rev 2's central structural claim** and is repaired with a
  measured three-part guard (§2). The hater and nice-guy lenses stalled (six
  agent stalls this session), so the main session ran their two decisive checks
  itself. Verification-substitution precedent: `decisions/0002`.
- **Crates touched:** `ufl-evolve` only — a new `witness.rs` module, one
  acceptance test, and a new `ufl-ga` path dependency (§4.3). `ufl-geo`/`ufl-ga`
  are **used, not modified** (SPEC-0011 §7's 2026-06-21 decision: the geometric
  crate stays frozen).

## 1. The de-risk is done, and both halves are measured

Unlike SPEC-0011 §2.6 — whose run was a throwaway that left nothing behind —
this spec is written **after** the derivation, against numbers reproduced three
times: by a five-family derivation fan-out with adversarial per-candidate
verification, by the main session rebuilding the leading candidate from scratch
with its own grids and counters, and by the architect review rebuilding it again
independently. All three agreed.

**One metric, stated once.** RMSE here is always **per-component**:

```text
rmse = sqrt( Σᵢ [ (xᵢ − x̂ᵢ)² + (yᵢ − ŷᵢ)² ] / (2n) )
```

This is `baseline.rs:295`'s formula, unchanged, so the witness and the MLP are
divided by the same denominator. Rev 2's table mixed this with a per-point
figure (denominator `n`); the two differ by exactly √2 and must never appear in
one table. Every number below is per-component.

| | nodes | `Param`s | `typecheck` | in-dist `[−2,2]²` | OOD `[2,3]²` | far `[−8,8]²` |
|---|---|---|---|---|---|---|
| **motor-sandwich (the witness)** | **25** | **4** | `Ok({3})` | **1.803e-16** | **1.797e-16** | **1.833e-16** |

3,600 samples per band, **zero** evaluation failures. `max abs(w − 1)` ranges
4.44e-16 … 7.77e-16; the z coordinate is **exactly `0.0`** at every sample.

The `[−8,8]` band is not in SPEC-0011 §2.5 — it was added to test the claim
harder, and the error does not move at **four times** outside the training
range. That is not generalization; it is the exact map.

**motor-sandwich reproduces SPEC-0011 §2.6's discarded numbers exactly** — 25
nodes, 4 `Param`s. The claim that had no artifact behind it was right.

**Withdrawn:** rev 2's second row (a 21-node "rotor-only-vector" form at
1.34e-16) is **removed, not merely demoted**. The derivation fan-out recorded
its counts and RMSE but not its expression, so nothing in this repo or its
artifacts can rebuild it, and an unreproducible row has no place in the
headline. §2's choice of witness does not depend on it; §6 keeps the durable
*mechanism* that family found, which **is** reproducible and ships as a test.

## 2. What the grade proof actually proves — and the guard that closes the gap

Rev 2 said `typecheck` returning `Ok({3})` means "the grade system *proves* the
result is a PGA point and the sandwich is a rigid motion." **The second half is
false.** Measured, every one of these also returns `Ok({3})`:

| expression | what it really is | in-dist RMSE |
|---|---|---|
| rotor plane `Basis(5)` (e₁₃) instead of `Basis(3)` | a rigid motion — the **wrong** one | 7.754e-1 |
| all four `Param`s replaced with garbage | a rigid motion — the wrong one | 3.803e0 |
| `t1`/`t2` swapped | a rigid motion — the wrong one | 8.980e-1 |
| `Sandwich(Basis(8), Basis(7))` — e₀ is **null** | the **zero** multivector; readout is `(NaN, NaN)` | — |
| `Sandwich(Basis(1), Basis(7))` — e₁ is a vector | a **reflection**: improper; collapses every input to `(−0, 0)` | — |

(Measured in §1's per-component metric on the `[−2,2]²` grid, this session. The
architect's independent rebuild found the same *kinds* of counterexample at
comparable magnitudes with its own garbage constants; the figures above are the
ones this repo reproduces.)

`grade.rs:79-82` says so itself, in a doc comment written during R-0020: *"Not
'is a versor': `Basis(8) = e₀` is null and non-invertible, yet the flag is
`true` and sound."* The refutation was already in the repo; rev 2 cited the flag
as proof anyway.

**What `Ok({3})` does prove, and it is worth having:** for *all* inputs,
structurally and without evaluating anything, the output lies in grade-3 — PGA
point space. That is a real machine-checked claim and it satisfies AC4's first
clause. It is simply not a rigid-motion certificate.

### 2.1 The guard: even + unit + `{3}`

The repair is structural, cheap, and measured. Name the motor subtree
`M = R(t₁)·T(l₁)·R(t₂)·T(l₂)` and assert three things:

| assertion | excludes | measured |
|---|---|---|
| `typecheck(M, ctx) == Ok({0,2,4})` — **even** | every reflection and every odd blade | motor `Ok({0,2,4})`; `Basis(1)` and `Basis(8)` are `Ok({1})` |
| `M ∗ M̃ == 1` — **unit** | the null/degenerate case, and any scaled versor | motor scalar part **0.9999999999999999**, non-scalar residue 2.78e-17; `Basis(8)` gives **0.0** |
| `typecheck(Sandwich(M, e₁₂₃), ctx) == Ok({3})` | a result outside point space | `Ok({3})` |

Even **and** unit **and** `{3}` together is *a proper rigid motion applied to
the origin, yielding a point*. Neither check alone suffices: `Basis(1)` is a
reflection whose `M ∗ M̃` is exactly `1.0` (unit passes, even fails), and
`Basis(8)` is odd *and* null (even fails, unit fails — its `M ∗ M̃` is `0.0`).

**The honest division of labour, stated so no reader has to find it:** the three
structural assertions prove the witness is *a* proper rigid motion producing a
point. They do **not** pin *which* rigid motion — the wrong-plane and
garbage-`Param` rows above pass all three. Pinning which one is the numeric
tests' job (§5.1–5.2). Structure and numerics each carry half the claim, and the
spec says which half is which.

### 2.2 What the grade proof buys the search today: nothing yet

`GradeScreen` (`lane.rs`) consumes `typecheck` as a **pass/fail** and discards
the grade set. So `Ok({3})` prunes nothing today that `Ok(⊤)` would not also
pass. The grade result is an **AC4 correctness argument and a presentation
asset**, not a search optimization — and saying otherwise in the write-up would
be the second-easiest thing for a reviewer to check.

### 2.3 The declaration the proof rests on

`Ok({3})` is **conditional on `t1`/`t2` being declared grade `{0}`** in the
`GradeCtx`. With an empty context both are ⊤ and `typecheck` returns
`Ok({0,1,2,3,4})` — confirmed by the architect's independent rebuild. Nothing
in the pipeline checks that the declaration matches what `Env` actually binds;
a caller that declares `{0}` and binds a bivector gets an unsound proof. The
witness test therefore **asserts the binding is grade-0** alongside the
declaration (§5.5), rather than leaving the two silently unlinked. Closing this
properly is a `ufl-geo` concern and is filed, not fixed here — the geometric
crate stays frozen.

## 3. The witness

```rust
/// `R(t) = Exp((−½·t)·e₁₂)` — garust's rotor convention is `exp(−½·θ·plane)`;
/// `Basis(3)` is the e₁₂ blade. 6 nodes, 1 `Param`.
fn rotor(joint: &str) -> GeoExpr { … Exp(GeoProduct(GeoProduct(Param(-0.5), Var(joint)), Basis(3))) }

/// `T(L) = Exp((L/2)·e₁e₀)`. `Basis(9)` is null (`Basis(9)² = 0`, measured), so
/// `exp` takes garust's `c == 0` closed form and **truncates after one term**:
/// `1 + (L/2)·e₁e₀`. The half-angle factor and the link length fold into one
/// `Param`. 4 nodes, 1 `Param`.
fn translator(link: f64) -> GeoExpr { … Exp(GeoProduct(Param(0.5 * link), Basis(9))) }

/// `M = R(t1)·T(l1)·R(t2)·T(l2)` — the motor, **exposed as its own function**
/// so §2.1's even/unit assertions have something to name. 23 nodes, 4 `Param`s
/// (the witness's 25 less the `Sandwich` node and the `Basis(7)` origin).
pub fn fk_motor(l1: f64, l2: f64) -> GeoExpr { … }

/// `Sandwich(M, e₁₂₃)` — the rightmost factor acts first, so the origin walks
/// out link 2, swings by t2, walks out link 1, swings by t1.
/// **25 nodes, 4 `Param`s** = `[−0.5, l1/2, −0.5, l2/2]`.
pub fn fk_witness(l1: f64, l2: f64) -> GeoExpr { Sandwich(fk_motor(l1, l2), Basis(7)) }
```

Angles arrive as `Var`-bound grade-0 `Mv`s (SPEC-0011 §2.5). Declaring
`t1`/`t2` as grade `{0}` in the `GradeCtx` is what makes the motor a *provable*
versor — without it `typecheck` cannot tighten past ⊤ (§2.3).

## 4. Where the transcendental lives — state this before anyone asks

### 4.1 The trigonometry is inside `Exp`, and the claim must say so

`garust-core/src/transform.rs:194-206` — `exp_scalar_square` for `c < 0`:

```rust
let s = (-c).sqrt();
Self::scalar(s.cos()) + *self * (s.sin() / s)
```

A rotor is `Exp(−t/2 · e₁₂)`, and `e₁₂² = −1`, so `c < 0` and **this branch
computes `cos(t/2)` and `sin(t/2)`.** The witness does not avoid trigonometry; it
*relocates* it into the algebra's canonical exponential, which is what a rotation
**is** in GA.

One precision in the result's favour: the two **translators** use `Basis(9)`
(e₁e₀), which is **null**, so `c == 0` and `exp` takes the `1 + self` branch —
**no transcendental at all**. Only the two rotors evaluate `cos`/`sin`.

**So the claim is not "exact FK without trig."** It is:

> With the right algebraic structure, this map is **exactly expressible** in 4
> parameters and stays exact wherever f64 can represent the angle; a generic
> function approximator needs 322 and holds only where it was trained.

And the sentence that carries it, because it is what a GATr reviewer tests
first: **the angle addition is performed by the algebra, not by the harness.**
Nothing outside the `GeoExpr` ever sees `t1 + t2`; the motor product composes the
two rotations, and the readout (§4.3) touches only coefficients.

That is an **inductive-bias** claim — precisely the claim GATr and CliffordNet
make — with the difference that this one is *exact and machine-checked* rather
than learned. Any write-up that says "no trigonometry" is false and would be
caught on first reading.

**It is a claim about expressibility, not about learning.** The witness's 4
`Param`s are *derived from ground-truth constants*; the MLP's 322 are *fitted
from labelled samples*. Stating "4 vs 322" without that sentence invites the
reading that the comparison was rigged, and the reading would be fair. The
learning claim — that a search can *find* these 4 without being told them — is
R-0011 §2.8's, and it is not made here.

#### The boundary, measured — "holds everywhere" was false as written

Error grows linearly in `|θ|`, as O(ε·|θ|):

| band | witness vs `ArmFk::forward` | `ArmFk::forward` vs 80-digit exact |
|---|---|---|
| `[−2,2]` | 1.80e-16 | 0.77e-16 |
| `[−10²,10²]` | 1.38e-15 | 1.39e-15 |
| `[−10³,10³]` | 1.32e-14 | 1.49e-14 |
| `[−10⁶,10⁶]` | 1.40e-11 | 1.58e-11 |
| `[−10⁹,10⁹]` | 1.43e-8 | 1.45e-8 |

The right-hand column is the decisive one and it is new to rev 3: the f64
reference **degrades identically**, tracking the witness within a factor of 1.2
at every decade. The drift is shared floating-point argument-reduction loss in
`sin`/`cos`, not a defect of the GA path — the witness stays within machine
precision *of the reference* everywhere tested, and the two walk away from the
true value together. Reference computed in 80-digit `Decimal` with
Taylor-series `sin`/`cos` after exact mod-2π reduction of the same f64 angles.

Consequence for §7 Q1: an absolute `< 1e-14` bound, which rev 2 recommended,
**would fail** on `[−10³,10³]`. The asserted bound is scoped to the bands the
ACs name.

### 4.2 The MLP baseline is fair — measured by trying to break it

The obvious attack on the headline is *"the MLP was set up to fail."* Tested
directly (release, `ArmFk { l1: 1.0, l2: 0.7 }`; all figures per-component,
§1's metric, which is what `baseline.rs` already reports):

| config | H | params | in-dist | OOD | OOD ÷ in-dist |
|---|---|---|---|---|---|
| default (700 ep) | 64 | 322 | 2.39e-3 | 3.29e-1 | 138× |
| **20,000 epochs** | 64 | 322 | **8.08e-4** | 2.78e-1 | **344×** |
| **5,000 ep + 4× train data** | 64 | 322 | **6.73e-4** | 2.70e-1 | **402×** |
| 20,000 ep | 256 | 1,282 | 3.34e-3 | 3.17e-1 | 95× |
| 20,000 ep | 16 | 82 | 1.52e-3 | 3.17e-1 | 209× |

Seed variance at H=64 / 5,000 epochs (five seeds): OOD **2.61e-1 … 3.00e-1**.

Three things follow, and all three favour the result:

1. **Training 28× harder improves in-distribution ~3× and moves OOD by 16%.**
   Four times the data does the same. The OOD floor is ~2.7e-1 regardless.
2. **More width does not help** — H=256 at 1,282 params is *worse*
   in-distribution than H=64 and flat OOD.
3. **The ratio gets *worse* with better training** (138× → 402×), because
   in-distribution error falls and OOD does not.

The collapse is **structural, not under-training**. On `[2,3]²` the angle sum
`t1+t2` reaches `[4,6]` — phase the network never saw — and a smooth
interpolator cannot know a periodic map outside its samples. That is the honest
mechanism, and it is a stronger statement than "the MLP is bad."

**Reproducibility: no gap.** `baseline` is `pub mod` (`lib.rs:15`), so
`train_report_with`/`TrainConfig` are reachable as
`ufl_evolve::baseline::{…}` without a root re-export. Verified from a
standalone external crate: `H=16 params=82 in-dist=2.92e-3 ood=4.62e-1`. §5's
artifact therefore needs nothing extra for AC3's reproducibility clause beyond
printing what it measures.

### 4.3 The readout is the kernel's — now enforced by the compiler

SPEC-0011 §7 permits "readout in the verifier", which is exactly the permission a
dishonest witness would abuse by doing the trigonometry outside the `GeoExpr`.
Rev 2 hand-wrote the readout and *argued* it matched garust's. Rev 3 stops
arguing and calls garust's:

```rust
pub fn read_xy(mv: &Mv) -> (f64, f64) {
    let (x, y, _z) = Point::from_multivector(*mv).to_euclidean();
    (x, y)
}
```

Measured **bitwise identical** to rev 2's hand-written coefficient reads across
1,600 samples (`to_bits()` equality on both components, max difference `0e0`).
The hand-written version is deleted: a claim the compiler enforces beats a claim
a reader has to check. This is why `ufl-ga` becomes a direct dependency of
`ufl-evolve`.

The readout still computes nothing — `t1` and `t2` appear nowhere in it, and it
takes only `&Mv`, so it *cannot* see them. The homogeneous-weight divide inside
`to_euclidean` is a measured no-op here (`max abs(w − 1)` ≈ 4.4e-16 … 7.8e-16)
and is required by SPEC-0011 §2.3, because an evolver's intermediates are not
rigid and can produce an ideal (zero-weight) point. The z coordinate is exactly
`0.0` everywhere — the arm is planar, and asserting that is a free structural
cross-check (§5.4).

## 5. Tests (TDD — written first, red)

1. **T-exact-in-dist (AC1)** — per-component RMSE over a deterministic 60×60
   grid on `[−2,2]²` against `ArmFk::forward`, asserted `< 1e-15` **and printed**.
   The bound is recorded from measurement (1.80e-16) with an order of headroom,
   and is scoped to this band per §4.1.
2. **T-exact-ood (AC2 — the headline)** — the same on `[2,3]²`, asserting the OOD
   RMSE is **within one order of magnitude of the in-dist RMSE**. That is the
   property the MLP cannot match and the reason the requirement exists.
3. **T-structure (AC4)** — §2.1's three assertions, as three separate
   `assert!`s with distinct messages: `typecheck(fk_motor(..)) == Ok({0,2,4})`
   (even), `M ∗ M̃ == 1` within 1e-15 with non-scalar residue < 1e-15 (unit),
   and `typecheck(fk_witness(..)) == Ok({3})` (point space). Each names what it
   excludes, so a failure says which property broke.
4. **T-structure-is-not-enough** — the counterexample table of §2 as a test:
   `Sandwich(Basis(1), Basis(7))` typechecks `Ok({3})` and **fails** the even
   check. This pins the *limit* of the structural claim so a future reader
   cannot re-derive rev 2's error from rev 3's passing tests.
5. **T-binding-matches-declaration (§2.3)** — the `Env` binds `t1`/`t2` as
   grade-0 `Mv`s, asserted against the `GradeCtx` declaration the proof uses.
6. **T-planar** — z coordinate exactly `0.0`, on every sample.
7. **T-reparametrisation (AC5)** — `FK(t1+a, t2) == R_a · FK(t1,t2) · R_a~`
   over a 40×40 grid with `a = 0.37`: **max deviation 1.11e-15, measured**.
   This is what AC5's "equivariant" means here and the only sense in which the
   word is earned (§7 Q3).
8. **T-null-blade-addition (§6)** — `Exp(a·e₁e₀) ∗ Exp(b·e₁e₀) == Exp((a+b)·e₁e₀)`.
   Measured **bitwise identical** (`to_bits()` equality on all 16 coefficients)
   across 54 `(a,b)` pairs spanning `[−3.25, 17]`; max coefficient difference
   exactly `0e0`. The durable finding, as a regression that asserts equality of
   bits rather than a tolerance.
9. **T-comparison (AC3)** — the deliverable artifact. One `#[ignore]`d release
   test printing **unconditionally**: node count, `Param` count, all three
   `typecheck` results, every band's RMSE, the full MLP sweep, the
   smallest-at-error selections, seeds and grid sizes, and the metric definition
   itself. Asserts only that every row was recorded (*Assert the Protocol, Not
   the Outcome*).
10. **T-far-band** — the `[−8,8]` band from §1. Not required by any AC; kept
    because it is the strongest single number in the result.

## 6. The finding the rejected derivation produced

The rotor-only family solved a problem the form set appears to forbid: **there is
no addition form**, yet FK is a sum of two rotated vectors. Its mechanism:
`Basis(9)` = e₁e₀ is **null**, so `Exp(u·E) = 1 + u·E` *exactly* (the `c == 0`
closed form truncates), and because the two register slots annihilate,
**multiplying two such `Exp` factors adds their arguments**. Addition, obtained
from a multiplicative form set via a null blade.

Measured, and the measurement is stronger than the claim needed to be: the two
sides are **bitwise identical**, not merely equal to within rounding — 54 pairs,
every one of the 16 coefficients matching on `to_bits()`, max difference `0e0`.
`Basis(9)²` evaluates to all-zero, which is the property the truncation rests
on. Addition here is not approximated by the algebra; it *is* the algebra.

Recorded because it is a durable fact about what this form set can express — and
because the same trick is what a future `FormFitness`-style requirement or an
evolved program might need. It ships as T-null-blade-addition (§5.8) so the fact
is checked rather than remembered.

The *witness* that family produced is withdrawn (§1): its expression was not
preserved, so its numbers cannot be rebuilt. The mechanism survives because it
can be.

## 7. The open questions, resolved

1. **What pins "machine precision"?** — `< 1e-15` asserted **and** the measured
   figure printed, scoped to the AC bands. Rev 2's proposed `< 1e-14` is
   rejected: §4.1 measures it failing on `[−10³,10³]`.
2. **Is one MLP seed enough?** — No; five seeds, already measured
   (2.61e-1 … 3.00e-1) and printed by T-comparison. SPEC-0011 AC6 requires the
   disclosed run count, and the floor being seed-stable is load-bearing for
   §4.2's "structural, not under-training".
3. **AC5's "equivariant".** — **Kept, narrowly, and only as discharged by
   T-reparametrisation.** The word is *not* earned in the GATr sense: the inputs
   are grade-0 scalars, and the rotation group acts trivially on them, so a
   referee's first question has no good answer. What is true and measured is
   base-frame reparametrisation: `FK(t1+a, t2) = R_a · FK(t1, t2) · R_a~`, max
   deviation 1.11e-15. Every use of the word must read "equivariant with respect
   to base-frame rotation" and cite that test. Unqualified, it is dropped.
4. **Does the 21-node rejection hold up?** — Moot. The row is withdrawn as
   unreproducible (§1), so there is no competing witness to reject. The motor
   ships because §2.1's guard can be *stated* about it, not because it beat
   something.
