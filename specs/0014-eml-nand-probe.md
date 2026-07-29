# SPEC-0014N — The literal `eml`-NAND tree + the integer-regime probe

> A **companion spec** to the Accepted [SPEC-0014](0014-discovery-framework.md)
> (the pattern SPEC-0011M set): SPEC-0014 realized R-0014 AC1/AC2/AC4 but left
> **AC3 — "the eml prize, discharged"** without a realizing design. This document
> supplies it. It touches no merged lane and adds no public API to a shipped crate.

- **Realizes:** [R-0014](../requirements/0014-discovery-framework.md) **AC3** — the
  literal `eml` tree for NAND evaluated end-to-end through `ufl-core`, **plus** the
  integer-regime probe (one matmul entry as an `eml` tree vs the exact `i64`
  verifier).
- **Status:** **Draft — revised 2026-07-25, all three-lens findings folded (§6).**
  Nice-guy *STRONG WORK*; architect *REQUEST CHANGES*; hater *NEEDS WORK* — and
  three of my claims were **measurably wrong**: the branch-cut mechanism, the
  probe's decision rule (it would have written a **false row into the honest
  ledger**), and the 0-path story. All corrected from measurement. Ready for
  re-review.
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

- `a − b`, `−b`, `a + b` require **`|second operand| ≤ ln(f64::MAX) = 709.782713`**
  — at `b = 709.79` the inner `exp(b)` overflows and the result is `(±inf, NaN)`
  even though the *answer* (`710.79`) is trivially representable.
- `ln(y)` has a hole for **`0 < y < e/f64::MAX ≈ 1.512e-308`** (the inner `e/y`
  overflows): `ln_t(1.512e-308) = (−inf, NaN)` while the true value `−708.78` is
  finite. Not subnormal-only — it eats normal `f64`s.

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
holds for only 1071/2000 reals on `[0.01, 20]`; `sub_t(1e16, 0)` is off by 34).
§2.3 therefore asserts **bit-exactness** on the gate, and reserves tolerance language
for the probe alone.

### 1.2 Why the exactness survives arbitrary circuit depth (the real result)

`{0,1}` is a **closed exact orbit** of the gate: `ln 1 = 0`, `exp 0 = 1`,
`ln 0 = −∞`, `exp(−∞) = 0` are each exact in IEEE-754, so every gate's output is
again *exactly* `0.0` or `1.0` — a gate's inputs are therefore always exact and
error has nowhere to accumulate. Measured (nice-guy, independently reproduced):
`NOT^k = NAND(prev,prev)` is **bit-exact at every k up to 9** (depth 93, 56,211
nodes), and a 3-layer **XOR** from 4 NANDs is bit-exact on all 4 rows.

Stronger still, `{0,1}` are **superattracting** fixed points of the induced map
`x ↦ 1 − x²`, so the encoding is **self-correcting**: perturbed to `true = 0.99`,
six NOTs restore `0.9999999999984`. The basin boundary is exactly the **golden
ratio φ = 1.618…** (the repelling fixed point of `1 − x²`): `x₀ = 1.618` converges,
`1.6180339888` diverges at step 23. So `theory/universal-computability.md` §6's
"noise margins shrink in deep circuits" is **wrong in the favourable direction** for
the digital case — inside the basin, margins *grow*. One caveat: near the `0` rail
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

What actually carries the result is therefore: **no intermediate is ever `NaN`**
(measured: zero NaN intermediates). That is the property §2.5 T-zero-edge asserts —
`ln(0) = −∞` with no trap (R-0001 AC3) *and* a NaN-free evaluation — rather than a
mechanism story that isn't true.

**A latent fragility this exposes (worth stating, since R-0014 AC4 is equality
saturation):** `inf − inf = NaN`, so these trees are **not obviously safe under
reassociation**. Measured: all four rows survived five semantics-preserving rewrites
(product commuted, log-sum commuted, `1−x` as `1+(−x)`, `(a·b)·1`, left-vs-right
association of a 3-input log-sum with a zero input) — so this is an *unstated
invariant*, not a demonstrated bug. Any future simplifier over these trees must
re-establish it: **ℝ-equality does not imply validity here.**

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

**Measured, so the spec states it rather than predicting it:** real error ≤ **2 ulp**
(81/81); the imaginary residue is **`Im ∈ sin(τ/2)·ℤ` with `|k| ≤ 4`** — a *winding
number*, not drift, whose quantum **is the R-0001 AC6 tripwire constant**
(`log.rs`); and `round()` recovers the exact `i64` in **81/81**. So the honest
characterisation is **"ulp-accurate and reversible, never exact"**.

**Two design corrections the measurement forces:**
- **Use the verifier's real shape.** `reconstruct` sums **seven triple products**
  (`Σ_t u_t·v_t·w_t`), not two pairwise ones. Measured in that shape: **21/200
  bit-exact**, worst `|ΔRe| = 8.9e-16` — roughly double the error, same verdict.
  It costs one loop; use it.
- **Probe beyond `{−1,0,1}`, and name the silently-wrong case.** `mul_t(n,1)` is
  **not bit-exact for 49 of the first 64 integers** — `mul_t(3,1) =
  2.99999999999999956`, so `as i64` **truncates to 2**. That is the worst failure
  mode (finite and wrong, not obviously broken) and it must be a named test case, not
  a lucky miss. Also named: `mul_t(−1,0) = −0.0` while `mul_t(0,−1) = +0.0` (**not
  bit-commutative**), and `ln(−0.0) = (−inf, +π)` vs `ln(+0.0) = (−inf, 0)` — a
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
   `ln_t` at `1.513e-308` ok / `1.512e-308` holes). Isolates *which* encoding broke.
2. **T-truth-table** (the AC3 gate) — the 4 rows **bit-exact**: `Re == {1,1,1,0}`
   and `Im == 0.0` exactly (§2.3).
3. **T-zero-edge** (R-0001 AC3) — `ln(0) == −∞` with no trap, and — the property
   that actually carries the result (§2.2) — **no intermediate of `nand_t(zero,zero)`
   is `NaN`**, asserted by walking every subtree. `eval` returns `Result` and cannot
   panic on numeric edges, so `assert!(eval(..).is_ok())` + explicit
   `is_infinite()`/sign assertions replace the earlier `catch_unwind`, which was
   **theatre**: a stack overflow is an `abort`, not an unwind, so `catch_unwind`
   guarded the empty set while missing the only real failure mode.
4. **T-depth-exact** (§1.2 — the strongest result) — `NOT^k` for `k = 1..=8` is
   **bit-exact at every level**, and a 3-layer **XOR** from 4 NANDs is bit-exact on
   all 4 rows. Plus the self-correction witness: from `true = 0.99`, six NOTs land
   within `2e-13` of `1.0`. (Release-only above `k = 6`: node count is `220·2^k − 109`.)
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

**And it does NOT claim poly-size circuit ⇒ poly-size `eml` tree — that is false**
(the hater's blocking finding, measured). `Eml` has **no sharing** (no DAG, no
let-binding), and `NOT a = NAND(a,a)` duplicates its argument, so chained-NOT tree
size is **`220·2^g − 109`** nodes at gate-depth `g` — verified against real trees to
`g = 12` (450,451 nodes); `g = 32` would need **4.7 × 10¹¹**. What `eml` gets is
**Boolean formulas, not Boolean circuits**, and formula size is super-polynomially
larger than circuit size for fan-out ≥ 2. **A 32-bit adder is not expressible as an
`eml` tree** in this repo, in this universe. The universality is *functional*, not
*efficient* — the ledger row must say so.

*(Forward-looking: this fan-out blow-up is the first concrete, exactly-verifiable
benchmark for R-0012's equality saturation — sharing collapses 56,211 nodes to ~940
with bit-exact ground truth. A good open question this spec creates rather than
answers.)* The honest closed class stays **"elementary functions — total,
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
*corrects* §6's current "noise margins shrink in deep circuits" for the digital case;
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
4. **Sequencing (new, for the implementer):** the derived-gate trees exceed
   `ufl_core::DEFAULT_MAX_DEPTH = 128` at gate-depth **13** (eml-depth = `10g + 3`),
   and `Sexpr`'s `Display` cap turns that into a **panic** via `to_string()` today.
   **SPEC-0017 (Accepted) removes the cap but is not yet implemented**, so either
   bound §2.5 T-depth-exact below 13 gates or land after R-0017. Recorded, not
   discovered later.
4. Does T-nand-is-universal-shape (§2.5.5) overreach toward the §3 composition claim
   the non-goals disclaim, or is it the right bounded evidence?


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
| **Nice-guy (the result)** — exactness survives **arbitrary depth**, and `{0,1}` are **superattracting** with a **φ noise margin** | New §1.2 + §2.5 T-depth-exact; §4 *corrects* theory §6's "margins shrink" for the digital case |
| **Nice-guy / architect** — milestone mis-numbered as Rung 2 | Header: **M5 Route A / value-universality**, explicitly *not* a reflection rung |
| **Architect (minor)** — `mul_t(3,1) as i64 == 2` (silently wrong); `±0.0` non-commutative; wrong tree shape; dangling §7.1 | All named test cases in §2.5 T5; verifier's **7-term** shape used; §4 reference fixed |
| **Architect (minor)** — R-0017 sequencing: gate-depth ≥13 panics via the `Display` cap today | §5 OQ4 records it — bound the depth test or land after R-0017 |

**The honest headline, post-correction:** "NAND works" was never in doubt after
`nand-embedding.py`. The results worth having are the two *quantified* bounds —
**exact and self-correcting on `{0,1}` at any depth (φ margin)**, and **ulp-accurate
but never bit-exact on integers**, which says an `eml`-lowered matmul lane could never
satisfy `error == 0` without a rounding step. Both are citeable engineering facts;
neither was in the first draft.
