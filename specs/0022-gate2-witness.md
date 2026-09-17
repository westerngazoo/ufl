# SPEC-0022 — Gate 2's witness and the fair comparison, as one reproducible artifact

- **Realizes:** [R-0022](../requirements/0022-gate2-witness.md) (ACs approved 2026-09-16).
- **Status:** **Draft (rev 2)** — all three lens agents **stalled** (six stalls
  this session), so the main session ran their two decisive checks itself: the
  integrity of the exactness claim (§4.1) and the fairness of the MLP baseline
  (§4.2). Both are folded below; one **materially changes how the result must be
  stated**. Verification-substitution precedent: `decisions/0002`.
- **Crates touched:** `ufl-evolve` only — a new `witness.rs` module and one
  acceptance test. `ufl-geo`/`ufl-ga` are **used, not modified** (SPEC-0011 §7's
  2026-06-21 decision: the geometric crate stays frozen).

## 1. The de-risk is done, and both halves are measured

Unlike SPEC-0011 §2.6 — whose run was a throwaway that left nothing behind —
this spec is written **after** the derivation, against numbers reproduced twice:
once by a five-family derivation fan-out with adversarial per-candidate
verification, and once by the main session rebuilding the two leading candidates
from scratch with its own grids, its own node/param counters, and its own
readout. Both agreed.

| witness | nodes | `Param`s | `typecheck` | in-dist RMSE | OOD RMSE | far `[−8,8]` |
|---|---|---|---|---|---|---|
| **motor-sandwich** | **25** | **4** | `Ok({3})` | 2.55e-16 | **2.54e-16** | 2.59e-16 |
| rotor-only-vector | 21 | 4 | `Ok({0,1,2,3,4})` | 1.34e-16 | 2.36e-16 | 1.90e-16 |

3,600 samples per band, **zero** evaluation failures. The `[−8,8]` band is not in
SPEC-0011 §2.5 — it was added to test the claim harder, and the error does not
move at **four times** outside the training range. That is not generalization; it
is the exact map.

**motor-sandwich reproduces SPEC-0011 §2.6's discarded numbers exactly** — 25
nodes, 4 `Param`s, ≈2.5e-16. The claim that had no artifact behind it was right.

## 2. Which witness ships, and why the smaller one does not

**The 25-node motor-sandwich is the witness.** The 21-node form is smaller and
marginally more accurate, and it is *rejected* for a reason R-0022 **AC4** makes
decisive:

`typecheck` returns **`Ok({3})`** for the motor — a tight grade-3 set, i.e. the
grade system *proves* the result is a PGA point and the sandwich is a rigid
motion. The rotor-only form returns `Ok({0,1,2,3,4})` — the **top** element,
which carries no information at all. It is a rotor multiplied by two null-bivector
registers, not a point; its correctness rests entirely on the numerics agreeing.

AC4 requires the structure be verified *independently of its numerics*. Only the
motor form can satisfy that. Choosing 4 extra nodes to buy a machine-checkable
structural claim is the whole point of having a grade type system.

The rotor-only derivation is recorded in §6 anyway, because its mechanism is a
genuine finding about the form set.

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

/// `Sandwich(R(t1)·T(l1)·R(t2)·T(l2), e₁₂₃)` — the rightmost factor acts first,
/// so the origin walks out link 2, swings by t2, walks out link 1, swings by t1.
/// **25 nodes, 4 `Param`s** = `[−0.5, l1/2, −0.5, l2/2]`.
pub fn fk_witness(l1: f64, l2: f64) -> GeoExpr { … }
```

Angles arrive as `Var`-bound grade-0 `Mv`s (SPEC-0011 §2.5). Declaring
`t1`/`t2` as grade `{0}` in the `GradeCtx` is what makes the motor a *provable*
versor — without it `typecheck` cannot tighten to `{3}`.

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
> parameters and holds **everywhere**; a generic function approximator needs 322
> and holds only where it was trained.

That is an **inductive-bias** claim — precisely the claim GATr and CliffordNet
make — with the difference that this one is *exact and machine-checked* rather
than learned. Any write-up that says "no trigonometry" is false and would be
caught on first reading.

### 4.2 The MLP baseline is fair — measured by trying to break it

The obvious attack on the headline is *"the MLP was set up to fail."* Tested
directly (release, `ArmFk { l1: 1.0, l2: 0.7 }`):

| config | H | params | in-dist | OOD | ratio |
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

**Reproducibility: no gap** — correcting rev 2's own claim. An earlier draft of
this paragraph said `train_report_with`/`TrainConfig` "cannot be reproduced from
outside the crate" because they are not re-exported from `lib.rs:18`. That was a
misdiagnosis of an import error of mine: `baseline` is `pub mod` (`lib.rs:15`),
so both are reachable as `ufl_evolve::baseline::{train_report_with, TrainConfig}`.
Verified from a standalone external crate:

```
REACHABLE from outside: H=16 params=82 in-dist=2.92e-3 ood=4.62e-1
```

A root re-export would be a convenience, not a fix. §5's artifact therefore needs
nothing extra for AC3's reproducibility clause beyond printing what it measures.

### 4.3 The readout is the kernel's, not an invention — and it computes nothing

SPEC-0011 §7 permits "readout in the verifier", which is exactly the permission a
dishonest witness would abuse by doing the trigonometry outside the `GeoExpr`.
It does not here, and the spec states the check explicitly:

```rust
pub fn read_xy(mv: &Mv) -> (f64, f64) {
    let c = mv.coeffs.as_slice();
    let w = c[0b0111];                       // e₁₂₃ — the homogeneous weight
    (-c[0b1110] / w, c[0b1101] / w)          // −e₀₂₃/w , e₀₁₃/w
}
```

Two coefficient reads, two sign flips and one divide. **No transcendental, no
arithmetic on the inputs** — `t1` and `t2` appear nowhere. The signs are garust's
point convention (`point(x,y,z) = w·e₁₂₃ − x·e₀₂₃ + y·e₀₁₃ − z·e₀₁₂`), and this
is character-for-character the first two components of garust's own
`Point::to_euclidean`.

The weight divide is a measured no-op (`max|w − 1|` ≈ 4.4e-16…6.7e-16 across all
bands) and is **kept deliberately**: SPEC-0011 §2.3 requires it because an
evolver's intermediates are not rigid and can produce an ideal (zero-weight)
point. `coeffs[11]` (the z coordinate) is exactly `0.0` everywhere — the arm is
planar, and asserting that is a free structural cross-check (§5.4).

## 5. Tests (TDD — written first, red)

1. **T-exact-in-dist (AC1)** — RMSE over a deterministic 60×60 grid on `[−2,2]²`
   against `ArmFk::forward`. Asserted against a bound **recorded from
   measurement** (§7 Q1), not a tolerance picked afterwards.
2. **T-exact-ood (AC2 — the headline)** — the same on `[2,3]²`, asserting the OOD
   RMSE is **within one order of magnitude of the in-dist RMSE**. That is the
   property the MLP cannot match and the reason the requirement exists.
3. **T-structure (AC4)** — `typecheck(&fk_witness(1.0, 0.7), &ctx()) == Ok({3})`.
   Verifies the derivation's *claim* (a rigid motion producing a point)
   independently of any numeric agreement.
4. **T-planar** — `coeffs[11] == 0.0` exactly, on every sample. A structural
   cross-check the numerics cannot fake.
5. **T-readout-is-blind** — the readout never sees the inputs: asserted by
   construction (`read_xy` takes only `&Mv`) and stated in §4. A *test* would be
   theatre; the type signature is the proof. §7 Q3 asks whether that is enough.
6. **T-comparison (AC3)** — the deliverable artifact. One `#[ignore]`d
   release test printing **unconditionally**: the witness's node count, `Param`
   count, in-dist/OOD/far RMSE; the full MLP sweep; the smallest-at-error
   selections; seeds and grid sizes. Asserts only that every row was recorded
   (*Assert the Protocol, Not the Outcome*).
7. **T-far-band** — the `[−8,8]` band from §1. Not required by any AC; kept
   because it is the strongest single number in the result.

## 6. The finding the rejected derivation produced

The rotor-only family solved a problem the form set appears to forbid: **there is
no addition form**, yet FK is a sum of two rotated vectors. Its mechanism:
`Basis(9)` = e₁e₀ is **null**, so `Exp(u·E) = 1 + u·E` *exactly* (the `c == 0`
closed form truncates), and because the two register slots annihilate,
**multiplying two such `Exp` factors adds their arguments**. Addition, obtained
from a multiplicative form set via a null blade.

Recorded because it is a durable fact about what this form set can express — and
because the same trick is what a future `FormFitness`-style requirement or an
evolved program might need. It is **not** a reason to ship that witness (§2).

## 7. Open questions for the three-lens

1. **What pins "machine precision" in T-exact-*?** An absolute `< 1e-14`; a
   multiple of `f64::EPSILON` scaled to the coordinate magnitude (|x| ≤ 1.7 here);
   or record-and-assert-recorded. R-0022 §6 Q2 leans to the third as the
   *Assert the Protocol* form — but a headline claim with no asserted bound is
   weak. Recommendation: assert `< 1e-14` **and** print the measured figure.
2. **Is one MLP seed enough (R-0022 §6 Q3)?** The OOD floor looks
   width-independent, which suggests it is structural rather than a training
   artifact — but "looks" is not measured. A 5-seed sweep costs ~12 s. Given
   SPEC-0011 AC6 demands disclosed run counts, is a single seed defensible?
3. **AC5's "equivariant".** R-0011's headline uses the word. A motor sandwich
   *is* equivariant by construction, but §5 has no test for it — only the
   structural `Ok({3})`. Either a concrete test (rotate the input frame, assert
   the output rotates by the same motor) or the adjective is dropped. Which?
4. **Does the 21-node rejection hold up?** §2 rejects it on AC4 grounds. Is a
   grade-type proof worth 4 nodes and a slightly worse RMSE — or is the
   smaller, more accurate form the honest primary with the motor recorded
   alongside as the structurally-provable one?
