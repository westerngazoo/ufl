# SPEC-0014N — The literal `eml`-NAND tree + the integer-regime probe

> A **companion spec** to the Accepted [SPEC-0014](0014-discovery-framework.md)
> (the pattern SPEC-0011M set): SPEC-0014 realized R-0014 AC1/AC2/AC4 but left
> **AC3 — "the eml prize, discharged"** without a realizing design. This document
> supplies it. It touches no merged lane and adds no public API to a shipped crate.

- **Realizes:** [R-0014](../requirements/0014-discovery-framework.md) **AC3** — the
  literal `eml` tree for NAND evaluated end-to-end through `ufl-core`, **plus** the
  integer-regime probe (one matmul entry as an `eml` tree vs the exact `i64`
  verifier).
- **Status:** **Accepted** (2026-07-25) — three review rounds, **every** finding
  folded (§6 round 1, §7 round 2, §8 round 3). Both blocking reviewers closed on
  "the substance now holds / fixes are mechanical", and the mechanical fixes are
  applied and independently re-measured. **Implemented** in
  `crates/ufl-core/tests/{nand/mod.rs, r_0014_ac3_eml_nand.rs}` (9 tests green);
  the §6 ledger row is closed. Gustavo holds final approval on the PR.
  *Six of my own claims were measurably wrong across the three rounds* — the
  branch-cut mechanism, the probe's decision rule (it would have written a **false
  row into the honest ledger**), the 0-path story, the `ln` cliff, the size law,
  and "a 32-bit adder is not expressible". All corrected from measurement.
- **Milestone:** **M5 — Route A / value-universality** and the closure of the one
  **"Owed" row** in
  [`theory/universal-computability.md` §6](../theory/universal-computability.md).
  **NOT a rung on the reflection ladder** — Rung 2 is "a bounded *in-UFL* eval,
  differential-tested against the Rust eval" (`theory/two-language-substrate.md`);
  this builds trees in Rust and evaluates them with the Rust evaluator. Different
  axis; claiming Rung 2 would both overstate this and dilute what it actually does.
- **Crate:** `ufl-core` (a new `tests/` e2e + a small `nand` test-support module).
  No change to `Eml`, `eval`, or any public API.
- **Depends on:** R-0001 (the `eml` core + **AC3 extended reals**, the `ln 0 = −∞`
  machinery this leans on), R-0014 AC3 (accepted), the AllEle derivation
  ([`docs/AllEle.html`](../docs/AllEle.html)).

---

## 0. What this discharges, in one paragraph

`theory/universal-computability.md` §3 argues `eml` **constructs** NAND (not merely
resembles it): with `false = 0`, `true = 1`, `AND(a,b) = exp(ln a + ln b)` and
`NAND = 1 − AND`, and since every operation there is `eml`-expressible (AllEle), the
whole of NAND is **one `eml` tree**. That has been verified *at the semantic level*
(`experiments/nand-embedding.py`) but the **literal tree has never been run through
`ufl-core`** — §6's single "Owed" row. This spec materializes the tree from the
primitive encodings, runs the 4-row truth table through the real complex evaluator,
and then asks the honest follow-up question R-0014 AC3 pairs with it: **does `eml`
over `Complex<f64>` carry the exact discrete/integer regime**, or does it leak?
A documented leak is an accepted, valid result — the ledger row closes either way.

## 1. The primitive encodings (derived, then **measured**)

The only atoms are `Eml::one()`, `Eml::var(..)`, and `Eml::node(exp_arg, log_arg)`,
where `node(x,y) ≡ exp(x) − ln(y)`. Everything below is built from those three.
**Every line was verified against the real `ufl_core::eval` before being written
here** (the measured column is not aspirational — §1.1):

| # | Encoding | Tree | Why |
|---|---|---|---|
| 1 | `exp(z)` | `node(z, 1)` | `exp(z) − ln 1 = exp(z) − 0` |
| 2 | `e` | `node(1, 1)` | `= exp(1)` |
| 3 | `0` | `node(1, exp(e))` | `exp(1) − ln(exp(e)) = e − e` |
| 4 | `ln(y)` | `node(1, node(node(1, y), 1))` | **R-0001 AC5's shipped identity** — `eml(1, eml(eml(1,y),1))`; 7 nodes, *exhaustively the unique minimum* (architect). An earlier 19-node 0-anchored form was equivalent but 3× larger. |
| 5 | `a − b` | `node(ln(a), exp(b))` | `exp(ln a) − ln(exp b) = a − b` |
| 6 | `−b` | `a − b` with `a = 0` | routes through **`ln 0 = −∞`** (R-0001 AC3) |
| 7 | `a + b` | `a − (−b)` | via 5 + 6 |
| 8 | `a × b` | `exp(ln a + ln b)` | the log-domain product (§3 of the theory) |
| 9 | **`NAND(a,b)`** | `1 − (a × b)` | `= 1 − AND(a,b)` |

**Domain cliffs (measured — rows 4–7 are NOT unconditional identities).** The
encodings route through `exp` of an intermediate, so they inherit `f64`'s range:

- `a − b`, `−b`, `a + b` require **`|second operand| ≤ ln(f64::MAX)`, measured finite to `709.782712893`**
  — at `b = 709.79` the inner `exp(b)` overflows and the result is `(±inf, NaN)`
  even though the *answer* (`710.79`) is trivially representable.
- `ln(y)` has a hole for **`0 < y < e^e/f64::MAX ≈ 8.4298e-308`** — the AC5 tree's
  inner intermediate is `exp(e − ln y) = e^e/y`, which overflows: measured finite for
  `y ≥ 8.429838189618891e-308`, `(−inf, NaN)` below. **Not subnormal-only** — the
  threshold sits *above* `f64::MIN_POSITIVE = 2.225e-308`, so it eats normal `f64`s.
  *(An earlier draft said `e/f64::MAX ≈ 1.512e-308`: that was the **old 19-node
  `ln`** encoding's intermediate, carried over unmeasured when row 4 changed — and
  it is itself subnormal, which contradicted the very sentence after it. Re-measured.)*

These are documented and boundary-asserted (§2.5 T-primitives), not hidden. They do
not affect the Boolean gate (inputs are `{0,1}`), which is the AC3 deliverable.

### 1.1 Measured (the pre-run, before this spec was written)

Built through `Eml::{one,node}` and evaluated by the real `ufl_core::eval`:

```
e = 2.718281828459045   0 = 0        exp(1) = 2.718281828459045
ln(1) = 0               ln(e) = 1    ln(0) = -inf   (no trap — R-0001 AC3)
1-1 = 0    -1 = -1      1+1 = 2      1*1 = 1        0*1 = 0
NAND(0,0)=1  NAND(0,1)=1  NAND(1,0)=1  NAND(1,1)=0     ← the truth table, EXACT
```

**The truth table came out exact, not merely within tolerance.** Scope that claim
precisely: it is exact **on bit inputs**, because `{0,1}` are exp/ln round-trip fixed
points — it is *not* a claim about the encoding generally (measured: `exp(ln x) == x`
holds for only **886 of 2000** uniform points on `[0.01, 20]`; `sub_t(1e16, 0)` is off by 34).
§2.3 therefore asserts **bit-exactness** on the gate, and reserves tolerance language
for the probe alone.

### 1.2 Why the exactness survives arbitrary circuit depth (the real result)

`{0,1}` is a **closed exact orbit** of the gate. The airtight form of the argument is
*measured base case + compositional evaluator*, not a list of identities (the four
usually cited — `ln 1 = 0`, `exp 0 = 1`, `ln 0 = −∞`, `exp(−∞) = 0` — are necessary
but not what the literal tree computes; it also needs `e − e = 0` exactly via
`zero_t`, `exp(ln 1) = 1`, and §2.2's 14-infinity cancellation):

> **Base:** `nand_t` is bit-exact on all four `{0,1}` rows (measured, §1.1).
> **Step:** `ufl_core::eval` is a compositional post-order fold with no
> depth-dependent state, so a gate whose inputs are *exactly* `0.0`/`1.0` returns
> exactly `0.0`/`1.0` — the same four rows.
> ∴ exact at **any** composition depth; error has nowhere to accumulate. Measured (nice-guy, independently reproduced):
`NOT^k = NAND(prev,prev)` is **bit-exact at every k up to 9** (depth 93, 25,551
nodes), and a 3-layer **XOR** from 4 NANDs is bit-exact on all 4 rows.

Stronger still, `{0,1}` is a **superattracting 2-cycle** of the induced NOT map
`x ↦ 1 − x²` (NOT swaps the rails; they are *fixed points of double-NOT*, not of
NOT — the earlier "superattracting fixed points" wording was loose). So the encoding
is **self-correcting**: perturbed to `true = 0.99`, six NOTs give
`0.9999999999984273` and eight return **bit-exact `1.0`**. The basin boundary is the
**golden ratio φ = 1.618…** — precisely, `−φ` is the repelling fixed point of
`1 − x²` and `+φ` is its positive preimage, hence the edge: `x₀ = 1.618` converges to
the cycle, `1.6180339888` diverges at step 23 (both measured; the step index is
escape-threshold dependent — 20 at `|x|>2`, 23 at `|x|>50`, 30 at non-finite — so the
test must pin the threshold). This **quantifies** `theory/universal-computability.md` §6 rather than correcting it:
§6 already scopes "noise margins shrink" to the **analog** substrate (~8–10 bits per
node) and already asserts digital exactness. §1.2 is the *digital* half — an IEEE-754
witness at depth 93 plus a named φ margin. (An earlier draft claimed it *corrected*
§6; that misread the scoping.) One caveat: near the `0` rail
there is no *relative* accuracy (`NAND(1−1e-12, 1−1e-12)` returns `2.0001778e-12`
vs the ideal `2.0e-12`) — catastrophic cancellation in `1 − a²`, first gate.

## 2. Design

### 2.1 Where the trees live

A `pub(crate)`-free **test-support module** `crates/ufl-core/tests/nand/mod.rs`
exposing `exp_t/e_t/zero_t/ln_t/sub_t/neg_t/add_t/mul_t/nand_t` as `fn(..) -> Eml`.
**Not** shipped in `src/`: these are *derivations under test*, not a public numeric
API, and putting them in `src/` would add unused public surface (CLAUDE.md §2).
If a later requirement needs them at runtime (an "EML compiler"), it promotes them
with its own spec.

### 2.2 Bit encoding and the `0`-input path

`false ↦ zero_t()`, `true ↦ Eml::one()`. The `0` input is the delicate case, and the
**measured mechanism is not the one the first draft claimed**. It is *not* "`−∞`
propagates and `exp(−∞) = 0` absorbs it": walking every subtree of the literal
`nand_t(zero, zero)` shows **14 infinite intermediates**, including `(−inf, −τ/2)` at
depth 6 and `(+inf, +τ/2)` at depth 4 — so the **branch cut is active on the
supposedly clean 0-path**, and the exact result `1` arrives by **cancellation of
complex infinities**, not absorption.

What actually carries the result is therefore: **no intermediate is `NaN` — on the
Boolean domain** (measured: 0 NaN among the 63 subtrees of `nand_t(zero,zero)`).

**That scope is load-bearing, not decoration (hater M1, measured).** Outside `{0,1}`
the claim is false, and asymmetrically so: `nand_t(1e-310, 0.0) = (NaN, NaN)` with
**7 NaN intermediates**, while `nand_t(0.0, 1e-310) = 1.0` exactly — the operator is
**not NaN-symmetric**. The rule: a **first** operand in `(0, 8.4298e-308)` poisons the
tree, because §1's `ln_t` cliff is inherited by `mul_t`/`nand_t` and never propagated.
So the composed operator's honest domain is **first operand ∈ `{0} ∪ [8.43e-308, ∞)`**;
`nand_t` is total on `{0,1}` (the AC3 deliverable) and *partial* below that. Since
§2.4's "Leak" criterion is *NaN or ±inf*, the probe's verdict is domain-dependent —
stated here rather than discovered by a reader.

That is the property §2.5 T-zero-edge asserts —
`ln(0) = −∞` with no trap (R-0001 AC3) *and* a NaN-free evaluation — rather than a
mechanism story that isn't true.

**What makes the 0-path NaN-free is a dependency, not an IEEE law (hater M2).** The
depth-3 intermediate is exactly `(−inf, +inf)`, and `num_complex` special-cases it —
`exp(−inf + inf·i) = (0,0)` per C99 Annex G — where the textbook
`e^re·(cos im, sin im)` gives `(NaN, NaN)`. `ufl-core` pins `num-complex = "0.4"`, a
**caret range**, so a minor bump could silently change what makes T-zero-edge pass.
SPEC-0001 §2.4 gives `log.rs` exactly this treatment (single point of change + the
AC6 tripwire); `exp` has no equivalent. **Owed: an AC6-style tripwire asserting
`exp(−inf + inf·i) == (0,0)`**, so the dependency fails loudly rather than silently.

**A latent fragility this exposes (worth stating, since R-0014 AC4 is equality
saturation):** `inf − inf = NaN`, so these trees are **not obviously safe under
reassociation**. Measured far harder than the first draft (hater M3 — 7 ℝ-valid rewrite families ×
25 operand pairs = 175 instances), the picture splits:

- **`Re` is invariant: 0/175 changed.** Stronger than the original five-rewrite claim,
  and it is why the `{0,1}` gate is safe — no leak found on Boolean inputs.
- **`Im` is NOT: 40/175 introduced `Im ≠ 0` where the original was exactly `0`** —
  including the *multiplicative identity*: `x → x·1` at `x = −1` moves `Im` from `0`
  to `sin(τ/2) = 1.2246e-16`; likewise `a−b → −(b−a)` and `a·b → (−a)·(−b)`.

This matters because **§2.3 defines `Im ≠ 0` as "the branch cut leaked"** — so the
very first rewrite an e-graph implements (`x·1 = x`) *flips that verdict* on a
mathematically identical tree. So this is **not** merely an "unstated invariant": it
is **demonstrated false outside `{0,1}`**, exactly in the negative-operand region
where R-0012's egg would run over the §2.4 probe. The invariant a simplifier must
preserve is **`Im`-preservation, not ℝ-equality** — and any rewrite pass over these
trees needs that stated as a precondition, not discovered.

### 2.3 The truth table gate

All four rows evaluated through `ufl_core::eval` and asserted **bit-exact**: the real
part `== {1,1,1,0}` exactly and the **imaginary part `== 0.0` exactly** (both measured;
§1.2 explains why exactness is structural, not lucky). Bit-exact rather than
ε-hedged because a tolerance here would hide the one thing worth catching: a nonzero
imaginary part means the branch cut (R-0001 AC4) leaked into a real-valued Boolean
computation — a finding, not a rounding detail. Tolerance language is reserved for
the probe (§2.4), which is a different regime.

### 2.4 The integer-regime probe (the honest half)

The probe asks R-0014's open question: does `eml` over `Complex<f64>` carry the
**exact** integer regime the matmul verifier requires? It is built from §1's
`add_t`/`mul_t` and compared against exact `i64`.

**The first draft's prediction was measurably WRONG, twice over — recorded because
the correction is the finding:**

1. *"Only negative operands hit the branch cut."* **False.** The cut is entered by
   **`add_t` itself** — `add_t(u,v) = node(ln_t(u), exp_t(neg_t(v)))` takes `ln` of
   its **first addend** — so `mul_t(a,b)` lands on the cut whenever `ln a < 0`, i.e.
   for **any `a ∈ (0,1)`**, no negative required. Measured: `NAND(0.5,0.5)` carries
   `Im = −2.12e-17`; `NAND(0.9999,0.9999)` carries `Im = −1.22e-20`. **The entire
   analog / probabilistic-bit regime sits on the cut**, which the "just negatives"
   framing would have missed entirely.
2. *"Absolute ε = 1e-12 decides match-vs-leak."* **Unsound as a rule.** Over the
   full `{−1,0,1}⁴` grid the *branch* residue is real but tiny (worst
   `|ΔIm| = 4.9e-16`), so **ε = 1e-12 scores 81/81 as "match"** while the exact-`i64`
   rule the verifier actually uses (`error == Some(0)`) scores **36/81 bit-exact**.
   The ε-rule would have written **"`eml` carries the discrete regime"** into the
   honest ledger — a false row. Worse, absolute ε is the wrong metric on a
   multiplicative substrate: relative error is pinned at ~1e-15, so the *same tree*
   fails a 1e-12 gate at modest operands (`100 × 101` → 1.64e-11).

**The decision rule (pre-registered, three outcomes — not two):**
- **Exact** — bit-exact on every case ⇒ `eml` carries the discrete regime.
- **Inexact but ulp-bounded** *(the measured answer)* — every case within a stated
  **ulp** bound but not bit-exact. Report the **bit-exact count** as the headline and
  the closed-form residue, not a pass/fail.
- **Leak** — a NaN, an `±inf`, or an unbounded/structural error.

**Measured, so the spec states it rather than predicting it** — and the rule is
**two-part, because a bare ulp bound is undefined where the expected value is 0**
(33 of the 81 cases; `ulp(0) = 5e-324`, so a 2.2e-16 residue there is 4.5e307 ulps —
the *same* "metric doesn't cover its grid" defect as the ε rule it replaced):
- **nonzero expectation (48 cases):** worst error **1 ulp**; 38/48 bit-exact in `Re`.
- **exact-zero expectation (33 cases):** absolute residue ≤ **2.221e-16**.

**These are the 2-term grid's numbers, and they do NOT carry to the 7-term shape the
test actually runs** (hater B2 — measured, 200 draws): nonzero-expectation worst
**5 ulp** (112/141 bit-exact `Re`), exact-zero residue ≤ **6.662e-16**. Pre-register
**per shape**, since a bound measured in one shape and asserted in another is the
same class of error as a metric that doesn't cover its grid:

| shape | nonzero-expectation | exact-zero |
|---|---|---|
| 2-term `c[0] = a₀₀b₀₀ + a₀₁b₁₀` (the grid) | ≤ **1 ulp** | ≤ **2.221e-16** |
| **7-term `Σ_t u_t·v_t·w_t` (what T5 asserts)** | ≤ **5 ulp** | ≤ **6.662e-16** |

Overall **36/81 bit-exact** (`Re` *and* `Im`). The imaginary residue is **`Im/sin(τ/2)` rounds to an integer with `|k| ≤ 4`** (residue <= 1e-31; 18/81 are
not *exact* multiples) — a *winding
number*, not drift, whose quantum **is the R-0001 AC6 tripwire constant**
(`log.rs`); and `round()` recovers the exact `i64` in **81/81**. So the honest
characterisation is **"ulp-accurate and reversible, never exact"**.

**Two design corrections the measurement forces:**
- **Use the verifier's real shape.** `reconstruct` sums **seven triple products**
  (`Σ_t u_t·v_t·w_t`), not two pairwise ones. Measured in that shape (uniform `{-1,0,1}`
  draws — **pin the seed and sample space in the test**, since an unspecified draw is
  not reproducible): **21-32/200 bit-exact** depending on the draw, worst
  `|dRe| ~ 1.1e-15` — roughly double the error, same verdict.
  It costs one loop; use it.
- **Probe beyond `{−1,0,1}`, and name the silently-wrong case.** `mul_t(n,1)` is
  **not bit-exact for 51 of the integers 0..=63** — `mul_t(3,1) =
  2.99999999999999956`, so `as i64` **truncates to 2**. That is the worst failure
  mode (finite and wrong, not obviously broken) and it must be a named test case, not
  a lucky miss. Also named: `mul_t(−1,0) = −0.0` while `mul_t(0,−1) = +0.0` (**not
  bit-commutative**), and — *through the encoding*, measured — `ln_t(−0.0) = (−inf, 3τ/8)` vs `ln_t(+0.0) = (−inf, 0)` — a
  `−0.0` intermediate can inject a spurious `iπ`.

**What each outcome changes in the theory doc** (so "no silent middle" is a decision
rule, not a ritual): the *ulp-bounded* result closes §6's row as **"`eml` reproduces
the integer regime to ~1 ulp, never exactly"**, which means **an `eml`-lowered matmul
lane could never satisfy `ufl-tensor`'s `error == 0` gate without a rounding step** —
a specific, citeable engineering constraint that retroactively validates R-0014 §2's
decision to keep per-lane atoms. It does **not** touch the `{0,1}` Boolean claim
(different domain).

### 2.5 Tests (TDD — written first, red)

`crates/ufl-core/tests/r_0014_ac3_eml_nand.rs`:

1. **T-primitives** — each §1 row asserted individually (`e`, `0`, `exp`, `ln(1)`,
   `ln(e)`, `sub`, `neg`, `add`, `mul`) against its closed form, **plus the §1 domain
   cliffs as boundary cases** (`sub_t` at `b = 709.78` ok / `709.79` overflows;
   `ln_t` at **`8.43e-308` ok / `8.42e-308` holes** — the re-measured AC5 cliff
   (§1); both of the earlier `1.51xe-308` points hole). Isolates *which* encoding broke.
2. **T-truth-table** (the AC3 gate) — the 4 rows **bit-exact**: `Re == {1,1,1,0}`
   and `Im == 0.0` exactly (§2.3).
3. **T-zero-edge** (R-0001 AC3) — `ln(0) == −∞` with no trap, and — the property
   that actually carries the result (§2.2) — **no intermediate of `nand_t(zero,zero)`
   is `NaN`**, asserted by walking every subtree. `eval` returns `Result` and cannot
   panic on numeric edges, so `assert!(eval(..).is_ok())` + explicit
   `is_infinite()`/sign assertions replace the earlier `catch_unwind`, which was
   **theatre**: a stack overflow is an `abort`, not an unwind, so `catch_unwind`
   guarded the empty set while missing the only real failure mode. **That mode is now
   bounded** (hater m5, measured under the real harness, debug): a linear `eml` spine
   survives depth **1,000** and aborts at **5,000**; gate-depth 13 (depth 133) and 16
   (depth 163, 3.28M nodes) both pass, and the 32-bit adder at depth 667 sits inside
   it. Recorded — it is the real ceiling R-0016/R-0017 fixtures will meet.
4. **T-depth-exact** (§1.2 — the strongest result) — `NOT^k` for `k = 1..=8` is
   **bit-exact at every level**, and a 3-layer **XOR** from 4 NANDs is bit-exact on
   all 4 rows. Plus the self-correction witness: from `true = 0.99`,
   **NOT⁸ returns bit-exact `1.0`** (measured; a stronger and more stable assertion
   than a tolerance — NOT⁶ is `0.9999999999984273`, i.e. `1.57e-12` out, so the
   earlier "within 2e-13" would have failed). (No release-only gate needed: measured 1.25 ms at `k = 9` in a debug build.)
5. **T-integer-probe** (§2.4) — the **verifier-shaped** 7-term sum vs exact `i64`
   over `{−1,0,1}`; asserts the pre-registered three-way verdict and **prints the
   bit-exact count + the ulp bound unconditionally**, so the number lands in the PR
   either way. Named cases: `mul_t(3,1) as i64 == 2` (the silently-wrong-and-finite
   one), the `±0.0` non-commutativity, and a magnitude sweep showing absolute error
   scaling (`100×101`, `1000×1001`).
6. **T-functional-completeness** (renamed from "universal-shape" — the old name
   invited exactly the over-read §3 forbids) — `NOT a = NAND(a,a)`,
   `a AND b = NAND(NAND(a,b),NAND(a,b))`, **and `a OR b = NAND(NOT a, NOT b)`** (the
   chain `theory/universal-computability.md` cites and `nand-embedding.py` verifies)
   each reproduce their truth tables from the *same* combinator. Bounded evidence for
   §3's composition claim — **combinational only**.

## 3. Non-goals (the claim discipline — R-0014 AC3 + the brief)

This discharges **discrete/combinational** universality *only*. It does **not**
claim: control universality (branching/recursion/state — "standard theory, but
unbuilt", theory §6), self-hosting, or that `eml` is a programming language in the
control sense.

**And it does NOT claim that a circuit's `eml` tree is the size of the circuit.**
`Eml` has **no sharing** (no DAG, no let-binding), so a gate whose output feeds `k`
places is *substituted* `k` times: a tree is a **formula**, not a shared circuit.
Measured under the AC5 encoding: a `G`-gate fan-in-2 NAND formula is
**`50·G + 1` nodes** (NAND 51, AND 151, OR 151).

**But the exponential blow-up is the NAND *gadget*, not `Eml` (hater M4 — measured,
and this corrects the previous draft too).** Chained NOT is `50·2^g − 49` (25,551 at
`g=9`, 204,751 at `g=12`) *only because* `NOT a = NAND(a,a)` **duplicates its
argument**. `eml`'s own §1 arithmetic basis has **duplication-free** gates —
`NOT x = 1 − x` (`sub_t`), `AND = mul_t`, `OR = 1 − (1−x)(1−y)` — so the doubling is
an artifact of expressing everything through NAND, a *presentation* choice, not a
property of the substrate.

**Measured consequence — a 32-bit adder is not merely expressible, it was built and
run** (hater, through the real `ufl_core::eval`):

| circuit | nodes | eml-depth | check |
|---|---|---|---|
| 8-bit ripple-carry adder | 24,671 | 187 | 500 random pairs, 0 failures |
| 16-bit | 90,295 | 347 | 0 failures |
| **32-bit** | **344,423** | 667 | 32 sum bits + carry-out, `Re` in `{0,1}`, `Im == 0.0` exactly |

That is **smaller than this spec's own NOT-12 chain** (204,751) — and ~10x under the
`3.6e6` estimate an earlier draft gave. *(That estimate also conflated units, applying
the per-**formula-gate** constant `50·G+1` to a *circuit* gate count.)* So the honest
limit is narrow and precise: **no sharing means tree = formula, and the NAND
presentation pays 2x per NOT that the arithmetic basis does not.**

**Two claims an earlier draft made here were WRONG and are withdrawn** (architect,
verified):
- *"`220·2^g − 109`"* — that constant fits **neither** encoding; it was a mangled
  carry-over from the old 19-node `ln` (whose law was `110·2^g − 109`). Re-measured
  above. This is the same defect as N1: **numbers derived from the pre-row-4
  encoding were not re-verified when row 4 changed.**
- *"A 32-bit adder is not expressible as an `eml` tree in this universe"* — **false.**
  Integer addition is in AC⁰ ⊆ NC¹, so it *has* poly-size fan-in-2 formulas; an
  unrolled 32-bit carry-lookahead is ~24k gates ≈ 73k NANDs ≈ **3.6 × 10⁶ nodes**,
  well inside what this spec already evaluates in milliseconds. Likewise
  *"formula size is super-polynomially larger than circuit size"* asserts as fact
  the **open NC¹ vs P/poly question**. The non-goal stands on the *measured*
  substitution blow-up, not on an unproven separation.

The universality claimed is **functional, for combinational logic** — not a
statement about efficient representation. The ledger row must say exactly that.

*(Forward-looking: the substitution blow-up is a concrete, exactly-verifiable
benchmark for R-0012's equality saturation — full hash-consing collapses NOT-9's
25,551 nodes to **125** distinct subtrees, with bit-exact ground truth. Note it is
measured on the **worst** basis (NAND); the arithmetic basis is a fairer baseline.)* The honest closed class stays **"elementary functions — total,
terminating, no recursion, no branching, no state"** until R-0005/T13. No new
public API; no change to any merged lane.

## 4. Deliverable beyond code

`theory/universal-computability.md` **§6's ledger row closes** (there is no §7.1 —
§7 is a flat list; the earlier reference was dangling): "the literal `eml` tree for
NAND, evaluated through `ufl-core`" moves **Owed → Verified**, citing the committed
test path, and stating **"functionally complete for *combinational* logic"** plus the
**formula-size caveat** (§3) — never bare "universality verified".

Two rows are **added**: (a) the **depth-exactness** result (§1.2 — `{0,1}` is an
exactly-closed, superattracting orbit; bit-exact to depth 93; φ noise margin), which
**quantifies** §6's already-scoped *digital* exactness claim — §6's margin clause is
scoped to **analog**, so nothing there is corrected;
and (b) the **integer-regime** result with its **bit-exact count and ulp bound**, not
a bare "match"/"leak".

## 5. Open questions for the three-lens

1. **RESOLVED (measured):** the prediction was wrong — the cut is entered by
   `add_t`, so **any `a ∈ (0,1)`** is affected, not just negatives (§2.4). Keep the
   `{−1,0,1}` regime *and* add the `(0,1)` sweep; do **not** retreat to `{0,1}`,
   where the probe could only confirm.
2. **RESOLVED:** `tests/` — nothing needs these at runtime; R-0016 (`raise ∘ lower`
   with a *known exact value*) and R-0017 (bushy deep fixtures with an exact oracle)
   need them at **test** time, which is where they live. Note the CI consequence:
   `clippy -D warnings` makes any unexercised builder a `dead_code` failure, so
   T-primitives asserting every row is what keeps it green — a dependency, not a
   coincidence.
3. **RESOLVED:** **bit-exact** on the gate (§2.3); tolerance language confined to the
   probe, and expressed in **ulps**, not absolute ε (§2.4).
4. **RESOLVED:** T-functional-completeness (§2.5 item 6, renamed from
   "universal-shape") is the right bounded evidence — R-0014 AC3 explicitly requires
   "verifying functional completeness", so it is not optional; `OR` was added so the
   test matches the chain the theory doc cites.
5. **Sequencing (for the implementer):** the derived-gate trees exceed
   `ufl_core::DEFAULT_MAX_DEPTH = 128` at gate-depth **13** (eml-depth = `10g + 3`),
   and `Sexpr`'s `Display` cap turns that into a **panic** via `to_string()` today.
   **SPEC-0017 (Accepted) removes the cap but is not yet implemented.** Measured
   caveat on my own note: it **binds nothing here** — `ufl_core::eval` never consults
   `get_max_depth()`, and this design never builds an `Sexpr`, so the `Display` cap is
   unreachable from this test path. Keep the note for whoever *does* render these
   trees; drop it as a constraint on T-depth-exact.
   **Resolved 2026-07-26 (R-0017, PR #80):** the cap is gone entirely — `Display`
   no longer panics at any depth, so the note above is history, not a live
   constraint on anyone rendering these trees.

## 6. Three-lens resolutions (2026-07-25)

Every lens ran the real code. **Nice-guy STRONG WORK** (and measured the result that
became §1.2 — the depth-exactness induction). **Architect REQUEST CHANGES.**
**Hater NEEDS WORK.** The corrections are the substance of this revision: three
things I asserted turned out to be false.

| Lens · finding | Resolution |
|---|---|
| **Hater 1 (BLOCKING)** — the embedding is **formula-size**, exponential in fan-out (`220·2^g − 109` nodes; `g=32` → 4.7e11); a 32-bit adder is *not expressible* | §3 non-goal: poly-size circuit ⇒ poly-size tree is **false and disclaimed**; §4 ledger row carries the caveat |
| **Architect (BLOCKING)** — my ε=1e-12 rule scores 81/81 "match" while the verifier's exact rule scores **36/81**; it would have written a **false ledger row** | §2.4: three-way rule (exact / **ulp-bounded** / leak), bit-exact count as the headline, ulps not absolute ε |
| **Hater 3 (BLOCKING)** — the cut is entered by **`add_t`**, so `mul_t` hits it for **any `a ∈ (0,1)`** — the whole analog regime, not just negatives | §2.4 rewritten around the measurement; `(0,1)` sweep added |
| **Hater 8 (MAJOR)** — the 0-path works by **cancellation of complex infinities** (14 infinite intermediates, cut *active*), not "absorption" | §2.2 corrected; the load-bearing property is **no NaN intermediate**, now the assertion. Rewrite-fragility invariant stated (`inf − inf = NaN`; 5 rewrites survived — an *unstated invariant*, not a bug) |
| **Hater 4+5 (MAJOR)** — domain cliffs: `sub/neg/add` need `|b| ≤ 709.78`; `ln_t` holes below `1.512e-308` | §1 documents both; §2.5 T-primitives asserts the boundaries |
| **Hater 9 / Architect (MAJOR)** — `catch_unwind` is **theatre** (overflow aborts, isn't unwindable) | §2.5 T3: `is_ok()` + explicit assertions; the real risk named |
| **Architect / nice-guy (MAJOR)** — my `ln` was 3× the **exhaustively unique minimum** and duplicated R-0001 AC5's shipped identity | §1 row 4 → `node(1, node(node(1,y),1))`, citing AC5 (NAND 111 → ~51 nodes) |
| **Nice-guy (the result)** — exactness survives **arbitrary depth**, and `{0,1}` are **superattracting** with a **φ noise margin** | New §1.2 + §2.5 T-depth-exact; §4 **quantifies** (not corrects) §6 — see round-2 N8 |
| **Nice-guy / architect** — milestone mis-numbered as Rung 2 | Header: **M5 Route A / value-universality**, explicitly *not* a reflection rung |
| **Architect (minor)** — `mul_t(3,1) as i64 == 2` (silently wrong); `±0.0` non-commutative; wrong tree shape; dangling §7.1 | All named test cases in §2.5 T5; verifier's **7-term** shape used; §4 reference fixed |
| **Architect (minor)** — R-0017 sequencing: gate-depth ≥13 panics via the `Display` cap today | §5 OQ4 records it — bound the depth test or land after R-0017 |

**The honest headline, post-correction:** "NAND works" was never in doubt after
`nand-embedding.py`. The results worth having are the two *quantified* bounds —
**exact and self-correcting on `{0,1}` at any depth (φ margin)**, and **ulp-accurate
but never bit-exact on integers**, which says an `eml`-lowered matmul lane could never
satisfy `error == 0` without a rounding step. Both are citeable engineering facts;
neither was in the first draft.


## 7. Round-2 architect findings (2026-07-25) — one defect, four faces

Re-review verdict: **REQUEST CHANGES**, and the diagnosis is worth stating plainly
because it is a *process* failure, not four unrelated slips.

**The defect:** §1 row 4 changed encoding (19-node 0-anchored `ln` → R-0001 AC5's
7-node one) and **four families of derived numbers were carried over unmeasured** —
the very thing §1's preamble ("every line was verified against the real
`ufl_core::eval`") promises never happens. The lesson this repo already banked as
*Measured Before Specified* has a corollary it did not have: **a measured block is
invalidated by any change to what it measured.** Re-measure on revision, not just on
first draft.

| Finding | Was | Now (re-measured) |
|---|---|---|
| **N1 (blocking)** `ln_t` cliff | `e/MAX ≈ 1.512e-308` (old encoding; itself subnormal, contradicting the next sentence) | **`e^e/MAX ≈ 8.4298e-308`** — above `MIN_POSITIVE`, so "eats normal f64s" is now true |
| **N2 (blocking)** size law | `220·2^g − 109` — fits **neither** encoding | **`50·G + 1`** per gate; **`50·2^g − 49`** chained-NOT (exact g=1..12) |
| **N2 (blocking)** "32-bit adder not expressible **in this universe**" | asserted | **FALSE, withdrawn** — addition is AC⁰ ⊆ NC¹; ~3.6e6 nodes. Also withdrew "formula ≫ circuit size", which is the **open NC¹ vs P/poly** question |
| **N4 (major)** the ulp rule | one bound over the whole grid | **two-part** — 1 ulp on the 48 nonzero-expectation cases, ≤2.221e-16 absolute on the 33 exact-zero ones (a bare ulp bound is undefined at 0: the *same* defect as the ε rule it replaced) |
| **N7/N8 (minor)** dynamics + framing | "superattracting fixed points"; "*corrects* §6" | **2-cycle** (fixed points of *double*-NOT); `−φ` repelling, `+φ` its preimage; **"quantifies"** §6, whose margin clause is scoped to *analog* |
| **N3/N5/N6/N9–N12** | tolerance 2e-13; raw-`Complex` `ln(−0.0)`; 709.782713; loose induction; needless release-gate; §5 dup numbering; 1071/2000; 49 integers | NOT⁸ bit-exact `1.0`; **`ln_t(−0.0) = (−inf, 3τ/8)`**; 709.782712893; base-case+fold form; debug-fine (1.25 ms @ k=9); renumbered; **886/2000**; **51** integers |

Corroborated unchanged (independently reproduced by the architect): the 14-infinite /
**0-NaN** subtree census, the 7-term **21/200**, `mul_t(3,1) as i64 == 2`, the `±0.0`
non-commutativity, the `(0,1)` branch residues, the φ divergence step, and
**rewrite-invariance across seven semantics-preserving variants**.


## 8. Round-3 hater findings (2026-07-25) — the diagnosis, applied to itself

Round 2 wrote the corollary *"a measured block is invalidated by any change to what
it measured."* Round 3's opening finding is that **§2.5 still carried the exact
constant §1 had withdrawn four sections earlier** — so the first test in the TDD plan
could never go green. The lesson was diagnosed and then **not propagated**, which is
the same failure one level up. Propagation is now part of the fix, not a follow-up.

| Finding | Resolution |
|---|---|
| **B1 (blocking)** — T-primitives still asserted the withdrawn `1.512e-308`; both sides of that pair hole | → **`8.43e-308` ok / `8.42e-308` holes** |
| **B2 (blocking)** — the two-part ulp bound is the **2-term grid's**, but T5 runs the **7-term** shape, where it is violated (5 ulp vs 1; 6.66e-16 vs 2.22e-16) | §2.4 now states the bound **per shape**, in a table; T5 asserts the 7-term row |
| **B3 (blocking)** — §4/§6 still instructed publishing *"corrects §6"* that §1.2 had withdrawn; two sections disagreed on what reaches the theory doc | Both → **"quantifies"** |
| **M1 (major)** — *"no intermediate is ever NaN"* is **false off `{0,1}`** and not even NaN-symmetric: `nand_t(1e-310, 0)` = `(NaN,NaN)` with 7 NaN intermediates, `nand_t(0, 1e-310)` = `1.0` | §2.2 scopes it to the Boolean domain and states the composed operator's real domain (**first operand in `{0} ∪ [8.43e-308, ∞)`**) |
| **M2 (major)** — the 0-path's NaN-freedom is a **`num-complex` C99-Annex-G corner case** (`exp(−inf+inf·i) = (0,0)`), on a caret-range dep, with no tripwire — while `log.rs` has AC6 | Named as a dependency; **an AC6-style `exp` tripwire is now owed** |
| **M3 (major)** — rewrite fragility understated: `Re` invariant **0/175**, but `Im` broken **40/175** — including `x → x·1` at `x=−1`. Since §2.3 defines `Im ≠ 0` as "the cut leaked", the first rewrite an e-graph writes flips the verdict | Upgraded from "unstated invariant" to **demonstrated false outside `{0,1}`**; the invariant is **`Im`-preservation**, stated as a precondition for R-0012 |
| **M4 (major)** — the blow-up is the **NAND gadget**, not `Eml`; and the hater **built a 32-bit adder**: 344,423 nodes, depth 667, 0 failures — *smaller than this spec's own NOT-12 chain* | §3 rewritten: duplication-free arithmetic gates; the measured adder table; unit conflation fixed |
| **m1–m5** | `Im/sin(τ/2)` *rounds* to an integer (18/81 inexact); 21–32/200 is draw-dependent → pin seed; φ step index is escape-threshold dependent; §5 renumbered 1–5; the stack ceiling **recorded (depth 1,000 ok / 5,000 aborts)** |

**What survived a much harder attack than the spec itself ran — §1.2.** 300 random
mixed circuits (NAND/NOT/AND/OR/XOR, ≤11 gates) × 16 assignments = **4,800
evaluations, 0 non-bit-exact**, plus deep AND/OR chains, `AND(0,0)` self-composition,
XOR, `−0.0` as the false encoding (`nand_t(−0.0,−0.0) = 1.0`, `Im = +0.0` — no iπ
leak), and the 32-bit adder. The **base-case + compositional-fold** argument is now
the strongest claim in the document, and it is the one I did *not* have to weaken.

**Net honest position after three rounds:** the Boolean gate is exact, self-correcting
(φ margin) and depth-robust — well-evidenced. Everything *around* it — domains,
size laws, metrics, rewrite safety — needed correcting, mostly because numbers
outlived the encodings they were measured on.
