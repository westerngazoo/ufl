# R-0022 — Gate 2's witness, in-repo: the exact equivariant program vs the fair MLP

- **Status:** **Draft** — acceptance criteria await Gustavo's sign-off (CLAUDE.md §4 step 1).
- **Milestone:** M5 — this **is** R-0011's headline (AC5 primary), not a new direction.
- **Realizes:** [R-0011](0011-geometric-neuroevolution.md) **AC5 primary** +
  **AC6** (honest reporting). SPEC-0011 §2.5 defines the gate; §2.6 de-risked the
  expressibility half.
- **Why it exists as its own requirement:** SPEC-0011 §2.6's expressibility
  de-risk was run **as a throwaway on 2026-06-22 and left nothing in-repo**
  ("repo left clean"). So the project's headline claim currently rests on a run
  no one can re-execute. R-0022 makes it an artifact.

## 1. What Gate 2's primary actually needs — and what already exists

SPEC-0011 §2.5's must-deliver claim is **two halves**:

| half | state |
|---|---|
| the **fair-MLP OOD comparison** | **exists and works** — `ufl-evolve/src/baseline.rs` (PR #33): `sweep`, `smallest_at`, `train_report`, and `MlpReport` already carries `ood_rmse` |
| the **expressibility witness** — an exact `GeoExpr` for the 2-link FK | **not in repo.** §2.6 proved it, threw it away |

**The stretch** — that the tree-GA *evolves* the structure rather than it being
hand-built — is explicitly §2.8-gated and **not part of this requirement**.
R-0011's novelty lives there, and SPEC-0011 says so; R-0022 delivers the primary
and leaves the stretch where the spec put it.

## 2. The MLP half, measured now

Run against the merged baseline (release, seed 42, widths 2…64) — **2.35 s**:

| H | params | train RMSE | in-dist RMSE | **OOD RMSE** | OOD / in-dist |
|---|---|---|---|---|---|
| 2 | 12 | 2.53e-1 | 2.37e-1 | 1.37e0 | 5.8× |
| 4 | 22 | 1.51e-1 | 1.54e-1 | 7.90e-1 | 5.1× |
| 8 | 42 | 2.22e-2 | 2.22e-2 | 6.14e-1 | **27.6×** |
| 16 | 82 | 5.55e-3 | 5.35e-3 | 5.30e-1 | **99.2×** |
| 32 | 162 | 3.61e-3 | 3.56e-3 | 1.93e-1 | 54.3× |
| 64 | 322 | 2.47e-3 | 2.39e-3 | 3.29e-1 | **137.6×** |

Smallest-at-error, per SPEC-0011 §2.5's anti-strawman rule: **H=8 (42 params)** at
in-dist ≤ 0.05, **H=16 (82 params)** at ≤ 0.01.

**The shape is the claim:** the MLP's OOD error **never falls below ~1.9e-1** at
any width. Sixteen times the parameters buys 100× better in-distribution fit and
**nothing** out of distribution. That is the fair, non-strawman baseline SPEC-0011
§2.5 demanded, and it is already reproducible from merged code.

## 3. The witness half — what must be built

A `GeoExpr` over the **existing** `ufl-geo` form set (no crate extension —
SPEC-0011 §7's 2026-06-21 decision) computing the 2-link planar forward map

```
x = L1·cos(t1) + L2·cos(t1 + t2)
y = L1·sin(t1) + L2·sin(t1 + t2)
```

as a **PGA motor chain**: angles arrive as `Var`-bound grade-0 `Mv`s; rotations
are `Exp` of an `e₁₂` bivector; translations are `Exp` of a null (`e₀ᵢ`)
bivector; the end-effector is the motor sandwiched onto the origin point, with
the `(x, y)` readout taken in the verifier (SPEC-0011 §7: "readout in the
verifier").

§2.6 reported ~25 nodes, **4 `Param`s**, and RMSE ≈ 2.5e-16. **None of those three
numbers is currently reproducible**, so R-0022 treats all of them as *claims to
re-measure*, not givens. If the derivation lands at a different node or `Param`
count, the measured number is what ships.

## 4. What this must NOT claim

- **Not "evolution discovered it."** The witness is hand-derived. R-0011's §2.8
  stretch is untouched, and any write-up must say "expressible and exact",
  never "evolved".
- **Not a parameter-count headline.** SPEC-0011's 2026-06-22 decision retired
  that framing: the earlier "0.07% of params" rested on a ~30× strawman MLP. The
  claim is the **OOD gap**; params are secondary.
- **Not IK.** The inverse map is inexpressible without the deferred
  `Normalize`/`Log` forms (SPEC-0011 §2.7). This is forward kinematics.
- **Not a benchmark win.** One task, one architecture family, one seed unless
  §5's AC3 says otherwise. The value is *exactness + equivariance*, not a
  leaderboard.

## 5. Proposed acceptance criteria — **for Gustavo's sign-off**

- **AC1 (the witness is in-repo and exact).** A committed `GeoExpr` — a named
  `pub fn` or a const-built value in `ufl-geo` or `ufl-evolve` — whose evaluation
  through the **real `ufl-geo`/`ufl-ga` kernel** matches `ArmFk::forward` to
  machine precision on the in-distribution band `[−2,2]²`. "Machine precision"
  is pinned as a measured RMSE, not a tolerance chosen after the fact.
- **AC2 (the OOD half — the actual headline).** The same witness is equally
  exact on the **OOD band `[2,3]²`**: its OOD RMSE is within the same order of
  magnitude as its in-dist RMSE. This is the claim the MLP cannot match, and it
  is the reason the requirement exists.
- **AC3 (the comparison, honestly reported).** A single committed artifact — a
  test that prints the table **unconditionally** — carrying: the witness's node
  count, `Param` count, in-dist and OOD RMSE; the MLP sweep's full table; the
  smallest-at-error selections; seeds and run counts. A reader must be able to
  re-run one command and get §2's table plus the witness's row.
- **AC4 (the derivation is checked, not asserted).** The witness's *structure*
  is verified independently of its numerics: the grade contract holds
  (`typecheck` is `Ok`), and the motor is a provable versor where the derivation
  claims one. A formulation that happens to fit numerically but is not a
  motor chain does not satisfy this.
- **AC5 (equivariance, if it is claimed).** The word "equivariant" appears in
  R-0011's headline. Either a test demonstrates the property concretely — e.g.
  rotating the input frame rotates the output by the same motor — **or** the
  claim is dropped from the write-up. No unevidenced adjective.
- **AC6 (honest negative is a pass).** If no exact witness is derivable over the
  current form set, that is a **documented result** satisfying R-0011 AC6, and it
  falsifies §2.6's expressibility claim — which would be a more important finding
  than the gate. The requirement is discharged either way.

## 6. Open questions for the three-lens

1. **Where does the witness live?** `ufl-geo` (next to the forms it uses, but it
   is a *task* fixture, not a language feature) or `ufl-evolve` (next to
   `ArmFk`/`baseline.rs`, which is what it is compared against)?
   Recommendation: `ufl-evolve`, beside the thing it beats.
2. **What pins "machine precision"** in AC1/AC2 — an absolute bound (`< 1e-14`),
   a multiple of `f64::EPSILON` scaled to the coordinate magnitude, or a
   measured-and-recorded figure with the assertion being only that it was
   recorded? The third is the *Assert the Protocol* form and the safest.
2. **Is one seed enough for AC3?** The MLP sweep is seeded; §2 used seed 42.
   SPEC-0011 AC6 demands disclosed run counts. Is a multi-seed MLP sweep needed
   for the comparison to be fair, given the OOD floor looks width-independent?
4. **Does AC5's equivariance test exist to be written?** The property is real for
   a motor sandwich, but a *test* of it needs a chosen group action and a
   tolerance. If it is more than a few lines, is dropping the adjective the
   honest cheaper path?
