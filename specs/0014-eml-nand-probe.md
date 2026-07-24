# SPEC-0014N — The literal `eml`-NAND tree + the integer-regime probe

> A **companion spec** to the Accepted [SPEC-0014](0014-discovery-framework.md)
> (the pattern SPEC-0011M set): SPEC-0014 realized R-0014 AC1/AC2/AC4 but left
> **AC3 — "the eml prize, discharged"** without a realizing design. This document
> supplies it. It touches no merged lane and adds no public API to a shipped crate.

- **Realizes:** [R-0014](../requirements/0014-discovery-framework.md) **AC3** — the
  literal `eml` tree for NAND evaluated end-to-end through `ufl-core`, **plus** the
  integer-regime probe (one matmul entry as an `eml` tree vs the exact `i64`
  verifier).
- **Status:** **Draft** — three-lens pending.
- **Milestone:** the staircase, **Rung 2** (*the language evaluates itself*) — and
  the closure of the **one "Owed" row** in
  [`theory/universal-computability.md` §6](../theory/universal-computability.md).
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
| 4 | `ln(y)` | `node(0, exp(node(0, y)))` | `node(0,y) = 1 − ln y`; then `1 − exp(…)`… ⇒ `ln y` |
| 5 | `a − b` | `node(ln(a), exp(b))` | `exp(ln a) − ln(exp b) = a − b` |
| 6 | `−b` | `a − b` with `a = 0` | routes through **`ln 0 = −∞`** (R-0001 AC3) |
| 7 | `a + b` | `a − (−b)` | via 5 + 6 |
| 8 | `a × b` | `exp(ln a + ln b)` | the log-domain product (§3 of the theory) |
| 9 | **`NAND(a,b)`** | `1 − (a × b)` | `= 1 − AND(a,b)` |

### 1.1 Measured (the pre-run, before this spec was written)

Built through `Eml::{one,node}` and evaluated by the real `ufl_core::eval`:

```
e = 2.718281828459045   0 = 0        exp(1) = 2.718281828459045
ln(1) = 0               ln(e) = 1    ln(0) = -inf   (no trap — R-0001 AC3)
1-1 = 0    -1 = -1      1+1 = 2      1*1 = 1        0*1 = 0
NAND(0,0)=1  NAND(0,1)=1  NAND(1,0)=1  NAND(1,1)=0     ← the truth table, EXACT
```

**The truth table came out exact, not merely within tolerance** — the `ln`/`exp`
round-trips did not accumulate visible error on bit inputs. §2.3 therefore asserts
exactness with a documented ε as a *safety margin*, not as an admission of drift.

## 2. Design

### 2.1 Where the trees live

A `pub(crate)`-free **test-support module** `crates/ufl-core/tests/nand/mod.rs`
exposing `exp_t/e_t/zero_t/ln_t/sub_t/neg_t/add_t/mul_t/nand_t` as `fn(..) -> Eml`.
**Not** shipped in `src/`: these are *derivations under test*, not a public numeric
API, and putting them in `src/` would add unused public surface (CLAUDE.md §2).
If a later requirement needs them at runtime (an "EML compiler"), it promotes them
with its own spec.

### 2.2 Bit encoding and the `0`-input path

`false ↦ zero_t()`, `true ↦ Eml::one()`. The delicate case is any `0` input:
`ln(0) = −∞` propagates and `exp(−∞) = 0` absorbs it, so `AND(0,b) = 0` **arrives
through the extended-reals path R-0001 AC3 already ships** — measured `-inf`, no
trap, no panic. This is asserted explicitly (§2.5 T-zero-edge), not assumed.

### 2.3 The truth table gate

All four rows evaluated through `ufl_core::eval`, asserted against `{1,1,1,0}` with
`|got − want| ≤ ε`, **ε = 1e-12** (a margin; measured error was 0). The real parts
carry the result; the **imaginary parts must be 0** within ε too — a nonzero
imaginary component would mean the branch cut (R-0001 AC4) leaked into a
real-valued Boolean computation, which is a finding, not a rounding detail.

### 2.4 The integer-regime probe (the honest half)

One entry of a 2×2 matmul — `c[0] = a₀₀·b₀₀ + a₀₁·b₁₀` — built as an `eml` tree from
§1's `add_t`/`mul_t`, evaluated over `Complex<f64>`, and compared to the **exact
`i64`** arithmetic the `ufl-tensor` verifier uses, over the `{−1,0,1}` inputs the
matmul lane actually ranges over (the certified schemes' coefficient set).

**Both outcomes are results** (R-0014 AC3 is explicit):
- **Match** (all entries exact within ε): `eml` carries the discrete regime; record
  it, and the ledger row closes *positively*.
- **Leak** (any entry drifts, or a `−∞`/branch artifact appears): record the exact
  input, the expected `i64`, the observed `Complex<f64>`, and the magnitude — the
  ledger row closes with a **documented negative**. This is the "no silent middle"
  clause: the probe must never be quietly dropped.

*Prediction, to be falsified rather than assumed:* `×` routes through `exp(ln a +
ln b)`, so a **negative** operand takes `ln` of a negative number — landing on the
R-0001 AC4 branch cut and producing an imaginary part. The `{−1,0,1}` regime
therefore likely leaks on negative coefficients while the `{0,1}` Boolean regime
does not. §2.5 T-integer-probe is written to *detect* this, not to encode it.

### 2.5 Tests (TDD — written first, red)

`crates/ufl-core/tests/r_0014_ac3_eml_nand.rs`:

1. **T-primitives** — each §1 row asserted individually (`e`, `0`, `exp`, `ln(1)`,
   `ln(e)`, `sub`, `neg`, `add`, `mul`) against its closed form within ε. Isolates
   *which* encoding broke when the table fails.
2. **T-truth-table** (the AC3 gate) — the 4 rows = `{1,1,1,0}`, real and imaginary
   parts both within ε.
3. **T-zero-edge** (R-0001 AC3) — `ln(0)` evaluates to `−∞` and `AND(0,b) = 0`
   arrives *through* it, with **no panic**: the whole table also runs under
   `catch_unwind` asserting `Ok`, so a future trap is a test failure, not a crash.
4. **T-integer-probe** (§2.4) — the matmul entry vs exact `i64`; asserts **either**
   a match within ε **or** an explicitly-recorded documented leak (the test prints
   the full comparison table unconditionally, so the result is in the PR either way).
5. **T-nand-is-universal-shape** — `NOT a = NAND(a,a)` and `a AND b =
   NAND(NAND(a,b), NAND(a,b))` reproduce their truth tables from the *same* tree
   combinator, evidencing the §3 composition claim rather than restating it.

## 3. Non-goals (the claim discipline — R-0014 AC3 + the brief)

This discharges **discrete/combinational** universality *only*. It does **not**
claim: control universality (branching/recursion/state — "standard theory, but
unbuilt", theory §6), self-hosting, or that `eml` is a programming language in the
control sense. The honest closed class stays **"elementary functions — total,
terminating, no recursion, no branching, no state"** until R-0005/T13. No new
public API; no change to any merged lane.

## 4. Deliverable beyond code

`theory/universal-computability.md` **§6 ledger row and §7.1 close** — "the literal
`eml` tree for NAND, evaluated through `ufl-core`" moves from **Owed** to
**Verified**, citing the committed test; and the §6 table gains a row for the
integer-regime probe with its measured outcome (match or documented leak).

## 5. Open questions for the three-lens

1. **§2.4's prediction:** is the negative-operand branch-cut leak the *right* thing
   to probe, or should the probe restrict to `{0,1}` (where the Boolean result
   already holds) and report the `{−1}` case as a separate, sharper finding?
2. **§2.1 placement:** test-support module vs a `src/` module behind a feature flag —
   does anything foreseeable (an EML compiler, the quote path) need these trees at
   runtime soon enough to justify shipping them now?
3. **ε = 1e-12:** right margin given measured error was 0, or should the table assert
   *bit-exact* equality and let any drift be a loud failure?
4. Does T-nand-is-universal-shape (§2.5.5) overreach toward the §3 composition claim
   the non-goals disclaim, or is it the right bounded evidence?
