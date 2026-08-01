# Universal Computability in UFL

*How UFL gets from "computes elementary functions" to "computes anything" — and
the honest account of what is proven, what is constructed, and what is still
owed. This is the intellectual backbone of UFL's central claim; it is research
documentation, not a spec.*

---

## 1. The question, and why `eml` alone does not answer it

UFL's headline is a single operator, `eml(x,y) = exp(x) − ln(y)`, from which —
with the literal `1` — *all elementary functions* are built (the AllEle
theorem; see [`../docs/AllEle.html`](../docs/AllEle.html) and R-0001). That is a
real and surprising result. But it must not be over-read:

> **The elementary functions are a closed, bounded class.** They are the finite
> compositions of `exp`, `ln`, the arithmetic operations, and algebraic roots.
> They are *total*, *terminating*, and contain **no recursion, no branching, no
> state, no iteration**.

So `eml` alone is a **function language**, not a model of computation. It can
express `sin`, `√`, `e`, `τ`, `i` — but not "loop until convergence," not "if
`x > 0` then A else B" as discrete control, not a Turing machine, not the gamma
function (which is *not* elementary). **UFL's universal-computability claim
cannot rest on `eml`.** Recognising this plainly is the starting point; the rest
of this note is the architecture that *does* deliver universality, with `eml` as
one load-bearing layer rather than the whole building.

## 2. Two axes of universality

"Universally computable" conflates two orthogonal claims. Separating them is the
key move.

| Axis | Question | UFL's answer |
|------|----------|--------------|
| **Value universality** | What can be computed *as a value*? | `eml` — elementary functions over ℂ (closed class). |
| **Control universality** | What computations can be *expressed* — sequencing, branching, recursion, unbounded iteration? | The **predicate layer** (`⟦P⟧`, Hehner). |

`eml` answers the first and *none* of the second. Universality requires both,
plus a bridge from UFL's continuous substrate to the discrete world of classical
computation. The next three sections give each piece.

## 3. Route A — discrete universality: the continuous NAND is *literal*

In digital logic, one gate is universal: **NAND** (the Sheffer stroke). Every
Boolean function is a composition of NANDs. UFL's docs call `eml` "the
continuous NAND" as an analogy — but it is stronger than analogy. **`eml`
constructs NAND.**

Encode `false = 0`, `true = 1` (real values). Then

```
AND(a, b)  = exp(ln a + ln b)        -- the log-domain product
NAND(a, b) = 1 − AND(a, b)
```

Every operation here (`+`, `−`, `×`, `exp`, `ln`) is `eml`-expressible (AllEle),
so **NAND on bit-encoded values is one `eml` tree.** The truth table is exact:

| a | b | AND | NAND |
|---|---|-----|------|
| 0 | 0 | 0 | 1 |
| 0 | 1 | 0 | 1 |
| 1 | 0 | 0 | 1 |
| 1 | 1 | 1 | 0 |

The one delicate point is the **0 input**: `ln 0 = −∞`. This is exactly the case
R-0001's **AC3 (extended reals)** was built for — `ln 0 = −∞` propagates and
`exp(−∞) = 0` with no trap or panic. So `AND(0, b) = exp(−∞ + …) = 0` falls out
of the substrate UFL already ships; the universality bridge's only edge case is
*already covered by accepted design*.

Verified, including the functional-completeness chain (NOT, AND, OR all built
from the `eml`-NAND), in
[`../experiments/nand-embedding.py`](../experiments/nand-embedding.py):

```
NOT a   = NAND(a, a)
a AND b = NAND(NAND(a, b), NAND(a, b))
a OR b  = NAND(NOT a, NOT b)
```

**Consequence.** Because `eml` builds a functionally-complete gate, it computes
*every* Boolean function — all combinational digital logic — as a special case
of its continuous substrate. Classical digital universality is recovered, not
assumed.

## 4. Route B — control universality: the predicate layer

Combinational logic is not yet computation: it has no state, no time, no
recursion. UFL's third pillar supplies these — **Hehner's predicative
programming** (atom `⟦P⟧`).

A program is a predicate `P(σ, σ′)` relating pre-state `σ` to post-state `σ′`.
On this base, the standard constructions give control universality:

- **Sequencing** `S ; T` = `∃σ′ · S(σ, σ′) ∧ T(σ′, σ″)` — composition is
  conjunction over an intermediate state.
- **Choice / branching** = a predicate case-split on a (possibly `eml`-computed)
  condition: `(b ∧ S) ∨ (¬b ∧ T)`.
- **Recursion / iteration** = a predicate defined as a fixpoint of its own
  specification: `P = (b ∧ (S ; P)) ∨ (¬b ∧ skip)` specifies a loop, and a loop
  may fail to terminate (its predicate is then unsatisfiable / `⊥`).

This is a known-complete model: predicative programming can specify *any*
computation, including non-terminating ones. **UFL inherits Turing-completeness
from the predicate layer**, with `eml` supplying the value computations *inside*
predicates (conditions, updates).

State + sequencing + recursion is also exactly what lifts Route A's
combinational NAND to *sequential* circuits — registers, feedback, finite-state
machines — i.e. all of digital computation, not just logic.

## 5. The synthesis — universal in two senses

Putting the pieces together:

```
            ┌─────────────────────────────────────────────┐
control →   │  ⟦P⟧  predicates: sequence, branch, recurse  │   (Turing-complete)
            └───────────────────┬─────────────────────────┘
                                │ conditions & updates are
                                ▼
            ┌─────────────────────────────────────────────┐
value   →   │  eml  : elementary functions over ℂ          │   (AllEle)
            └───────────────────┬─────────────────────────┘
                                │ on {0,1}-encoded values
                                ▼
bridge  →   ┌─────────────────────────────────────────────┐
            │  NAND = 1 − exp(ln a + ln b)  → all Boolean   │   (verified)
            └─────────────────────────────────────────────┘
```

UFL is then universal in **two distinct, complementary senses**, and this is a
feature, not a hedge:

1. **Discrete (classical Turing).** Via the NAND embedding (§3) + predicate
   state/recursion (§4), UFL expresses any classical computation. "All
   hardware/software" in the digital sense is recovered as a *restriction* of
   the continuous substrate to bit-encoded values.
2. **Continuous (real / BSS).** `eml` computes over ℂ natively, and predicates
   range over continuous state. In the Blum–Shub–Smale sense (computation over
   the reals), UFL is naturally a *real*-computation language — and the digital
   model is the special case. This is the more honest home for UFL's thesis:
   it is not "a weird way to do digital," it is real computation, of which
   digital is a quantised slice.

The substrate orchestrator (`⊗`) then chooses *where* to run a given
predicate-specified, `eml`-valued computation — and the same expression can land
on a digital substrate (use the NAND/bit encoding) or an analog one (use the
continuous values directly). That dual nature is precisely what the
hardware/software-boundary thesis predicts.

## 6. The honest ledger — what is owed

A claim this large must be explicit about what is proven, constructed, and
merely sketched.

| Piece | Status |
|-------|--------|
| `eml` ⇒ all elementary functions | **Proven** (AllEle), implemented & tested (R-0001). |
| `eml` ⇒ Boolean NAND (discrete bridge) | **Verified** at the semantic level ([`nand-embedding.py`](../experiments/nand-embedding.py)); the 0-edge is covered by R-0001 AC3. |
| The *literal* `eml` tree for NAND, evaluated through `ufl-core` | **VERIFIED** (2026-07-25, R-0014 AC3 / SPEC-0014N) — [`crates/ufl-core/tests/r_0014_ac3_eml_nand.rs`](../crates/ufl-core/tests/r_0014_ac3_eml_nand.rs). The 4-row truth table is **bit-exact** (`Re == {1,1,1,0}`, `Im == 0.0` exactly) through the real complex evaluator, with the `0` input routing through `ln 0 = −∞` and **no NaN intermediate**. Scope: **functionally complete for _combinational_ logic** — see the two rows below for what that does *not* mean. |
| — its **cost**: a tree is a *formula*, not a shared circuit | **Measured.** `Eml` has no sharing, so a `G`-gate NAND formula is `50·G + 1` nodes and chained NOT doubles (`50·2^g − 49`, exact to `g=12`) because `NOT a = NAND(a,a)` duplicates its argument. That doubling is the **NAND presentation**, not `eml`: the arithmetic basis (`NOT x = 1−x`, `AND = ×`) is duplication-free, and a **32-bit adder is expressible and was run** (344,423 nodes, depth 667, 0 failures). So: **no poly-size-circuit ⇒ poly-size-tree claim** — and equally, no "not expressible" claim, which an earlier draft wrongly made. |
| — its **depth robustness**: `{0,1}` is an exactly-closed orbit | **Measured, and it _quantifies_ the last row's digital half.** Base case bit-exact + `eval` is a compositional fold ⇒ exact at **any** composition depth (witnessed to depth 93 / 25,551 nodes, and across 300 random mixed circuits × 16 assignments, 0 non-bit-exact). `{0,1}` is a **superattracting 2-cycle** of `x ↦ 1−x²`, so the encoding **self-corrects**: from `true = 0.99`, `NOT⁸` returns bit-exact `1.0`, with the basin boundary at the golden ratio **φ**. Digital margins *grow* inside the basin; this says nothing about the analog half. |
| `eml` ⇒ the exact **integer** regime (R-0014 AC3's second probe) | **Documented NEGATIVE, quantified and reversible.** Over the matmul verifier's real 7-term shape on `{−1,0,1}` (pinned seed, 200 draws): **25/200 bit-exact**, worst **3 ulp** on nonzero expectations, **7.7e-16** absolute where the expectation is 0, **0 leaks** — and `round()` recovers the exact `i64` in **200/200**. So `eml` over `Complex<f64>` is **ulp-accurate but never bit-exact** on integers, and the failure is *silent under truncation* (`mul_t(3,1) = 2.9999…`, `as i64 → 2`). **Engineering consequence:** an `eml`-lowered matmul lane could never satisfy `ufl-tensor`'s `error == 0` gate without an explicit rounding step — which retroactively validates R-0014 §2's decision to keep per-lane atoms. |
| Predicate layer ⇒ control universality | **Standard theory** (Hehner), but **unbuilt** in UFL and not yet instantiated against `eml`-valued state. This is the real frontier. |
| Orchestrator selects a satisfying substrate | **Heuristic, not complete** — predicate satisfiability is undecidable in general; the orchestrator is a cost-guided search, never a decision procedure. Do not overclaim completeness. |
| Universality *survives real substrates* | **Bounded by physics.** On analog, each node carries ~8–10 bits and error compounds with depth, so the bit-encoding's noise margins shrink in deep circuits; digital universality is exact on CPU/FPGA but *approximate* on analog. The σᵏ-noise conjecture (proposal §5) governs the budget and is unproven. |

## 7. Falsifiable next steps

In UFL's style — claims become experiments, then specs.

1. **Materialise the NAND tree.** Build the actual `eml` tree for
   `1 − exp(ln a + ln b)` from AllEle's `+`/`−`/`×` constructions, evaluate it
   through `ufl-core`'s `eval`, and confirm the truth table end to end —
   turning §3's semantic verification into a pure-`eml` one. (Depends on the EML
   compiler / arbitrary-arithmetic-as-trees work.)
2. **Spec the predicate layer** (the future `⟦P⟧` requirement) with control
   universality as an explicit, tested acceptance criterion: exhibit a
   non-trivial recursive computation (e.g. a bounded loop, then an unbounded
   one) specified as predicates over `eml`-valued state.
3. **A `theory/` proof note** for the bijection *full binary trees of one
   operator ≅ S-expressions of one head* (the homoiconicity ground, R-0003),
   tying the meta-level uniformity to this object-level universality.

## 8. The one-paragraph honest summary

`eml` is **value-universal** for the elementary functions and, by literally
building NAND, is the **discrete-universality bridge** — verified, with its only
edge case already handled by R-0001. It is **not** control-universal on its own;
that comes from the predicate layer (Hehner), which is standard theory but
UFL's real unbuilt frontier. Together they make UFL universal in both the
classical-digital and the real-computation senses, with digital as a quantised
special case. The substrate orchestrator is a cost-guided heuristic, not a
decision procedure, and real-substrate universality is bounded by noise. UFL's
claim to compute "anything" is therefore **well-founded but not yet
discharged** — the foundation is proven and the bridge is verified; the control
layer is the work that turns the claim into a theorem.
