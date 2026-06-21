# R-0012 — Boolean Deduction via Equality Saturation over 𝔽₂

- **Status:** Draft (2026-06-20 — owner + Claude, discuss phase)
- **Milestone:** M5 — Discovery (the **discrete-logic / reasoning** lane — a third
  lane, orthogonal to the EML-continuous and Clifford-geometric threads; see §2)
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-06-20
- **Pillar / atom:** Pillar 3 — the **boolean/logic substrate**. Where R-0004
  (`ufl-predicate`) checks Hehner predicates over a continuous value, this is
  *deduction* over discrete 𝔽₂: "does B follow from A₁…Aₙ?" as ideal membership.
- **Depends on:** *nothing in the EML or geometric lanes.* New **external**
  dependencies only — an e-graph/equality-saturation crate (`egg`) and a SAT
  solver binding (`splr` or `varisat`) for the baseline. See the §6 premise
  correction.
- **Realized by:** SPEC-0012 (pending — three-lens, then test plan)
- **QA:** `qa` agent run scoped to R-0012
- **Source:** owner's spec *"UFL Spec: Boolean Deduction via Equality Saturation
  over 𝔽₂"* (Google Doc, provided 2026-06-20). This requirement transcribes its
  load-bearing content so the repo stays the source of truth (CLAUDE.md §8), and
  **corrects one factual premise** (§6).

## 1. Statement

UFL gains a **discrete deduction engine**: a new module (crate-vs-module is a §5
open question) `ufl_discovery::reasoning::f2` that decides propositional
entailment by **algebra, not search over truth assignments**.

Boolean logic over **𝔽₂** is the polynomial ring `𝔽₂[x₁,…,xₙ] / (xᵢ² − xᵢ)`:
XOR is `+`, AND is `·`, and Boolean idempotence *is* the ring axiom `x² = x`.
Under this encoding, **"B is deducible from A₁…Aₙ" becomes "poly(B) reduces to 0
modulo the ideal ⟨A₁…Aₙ⟩"** — and **equality saturation (egg) and Gröbner-basis
(Buchberger) reduction are the same Knuth–Bendix completion in two languages**.
The deduction engine *is* an e-graph saturated under the ring rules plus the
axioms-as-ground-equalities; inconsistency is the **𝔽₂ Nullstellensatz collapse**
(`0`'s e-class merges with `1`'s).

Three coupled pieces:

1. **The encoding** — `Expr` (boolean AST: `Zero/One/Var/Not/And/Or/Xor/Implies`)
   and `to_f2: &Expr → RecExpr` lowering it to its 𝔽₂ polynomial per the §1.1
   dictionary.
2. **The deduction engine** — `entails(axioms, goal) → Entailment`
   (`Provable / NotProvable / Inconclusive`), implemented as the **refutation**
   form: `axioms ⊢ goal ⟺ axioms ∪ {¬goal}` is inconsistent (the `0 = 1` merge),
   which is **extractor-independent** (avoids egg's cost-model hazard).
3. **The falsification harness** — `entails` checked against a **SAT-solver
   baseline** on a parameter sweep, from day one (§3 / AC2, AC4).

### 1.1 The canonical encoding (the dictionary the engine must implement)

| Logic | 𝔽₂ polynomial |
|---|---|
| `false / ⊥` | `0` |
| `true / ⊤` | `1` |
| `¬a` | `1 + a` |
| `a ∧ b` | `a·b` |
| `a ⊕ b` | `a + b` |
| `a ∨ b` | `a + b + a·b` |
| `a → b` | `1 + a + a·b` |

An axiom `P` asserted true is the ideal generator `P − 1` (assert `poly(P) ⇒ 1`).
Ring rewrite rules (the engine's rule set): `add_comm`, `add_assoc`, `add_id`
(`a+0⇒a`), **`add_self` (`a+a⇒0`, char 2)**, `mul_comm`, `mul_assoc`, `mul_id`
(`a·1⇒a`), `mul_zero` (`a·0⇒0`), **`idemp` (`a·a⇒a`, Booleanity)**, `distrib`.
`add_self` and `idemp` are what make this 𝔽₂ and not generic polynomial
arithmetic — and the source of the §4 termination subtlety.

## 2. Rationale

R-0012 is a **falsifiable bridge experiment**: does the egg↔Buchberger
correspondence *buy* anything inside UFL's own codebase, or is it real-but-inert?
The point is to **get that answer in days, on a laptop**, with a SAT baseline as
the honesty gate — the same discipline as [R-0008](0008-discovery-engine.md)
(blind-GA Strassen falsified and documented, not hidden).

**This is a deliberate third lane.** The owner's spec is emphatic about scope:
*no EML* (the continuous regime — `ufl-core`), *no Clifford/garust/rotors*
("logic has no metric" — the geometric regime, `ufl-ga`/`ufl-geo`/R-0011). UFL
thereby holds three separately-motivated theses:

| Lane | Crates | Regime |
|---|---|---|
| Continuous / algorithm-discovery | `ufl-core`, `ufl-tensor`, `ufl-discovery` | EML over ℝ (Strassen) |
| Geometric / metric | `ufl-ga`, `ufl-geo`, R-0011 | `Cl(3,0,1)` neuroevolution |
| **Discrete logic (this)** | new `reasoning::f2` | 𝔽₂ / Gröbner-egg |

The latent unification is acknowledged but **deferred**: the continuous `[0,1]`
Hehner regime ([`ufl-predicate`](../crates/ufl-predicate)) is the *temperature→0*
parent of this discrete one; the temperature-parametrized bridge is a *later*
requirement, attempted only if R-0012 returns a positive result.

**Honest prior (for planning):** modern CDCL SAT is extremely strong, and
`distrib + add_self` reproduces Buchberger's doubly-exponential (Mayr–Meyer)
worst case. The likely outcome is **"theoretically real, computationally inert"**
— competitive only, if anywhere, on *structured / sparse-ideal* problems
(implication chains). The spec treats that as a **valid, recordable result**; the
deliverable's value is the clean falsification + the conceptual unification, not a
speed record. R-0012 succeeds by *answering the question*, win or lose.

## 3. Acceptance criteria

- **AC1 — The minimal deductions (the smallest thing that must pass).** With the
  §1.1 encoding and the refutation engine:
  - `{a, a → b} ⊢ b` → `Provable` (modus ponens);
  - `{a → b, b} ⊬ a` → `NotProvable` (affirming the consequent);
  - `{a, ¬a} ⊢ z` (any `z`) → `Provable` (the `0 = 1` / ⊥ collapse).
- **AC2 — SAT-agreement is the correctness gate (wired in from day one).** A
  `tests/f2_vs_sat.rs` harness generates entailment problems parameterized by
  `n ∈ {5,10,15,20,25,30}` and clause density — both **structured** (implication
  chains, sparse ideal) and **random 3-SAT near the phase transition** — and
  asserts `entails` **agrees with the SAT baseline on every answer**. Disagreement
  is a **bug, not a result** (fix before any timing claim).
- **AC3 — Totality / honesty under blowup.** Saturation runs under explicit
  Runner caps (node, iteration, time). Hitting a cap returns `Inconclusive`
  (**never a false `NotProvable`**). `NotProvable` is returned **only** when the
  e-graph genuinely saturated (reached fixpoint). The Mayr–Meyer blowup is
  documented as expected, not papered over.
- **AC4 — The falsification verdict is recorded.** The harness records wall-clock,
  e-graph node count (egg) and conflicts/decisions (SAT), and the result is
  reported against the **decision rule**: *buys-something* (agrees AND
  competitive/winning on structured problems → pursue) / *inert* (agrees but
  crushed everywhere → record the negative result, do **not** build hardware) /
  *bug* (disagrees → fix). A negative result, honestly recorded, **satisfies**
  this AC.
- **AC5 — Encoding correctness + canonicalization.** The §1.1 dictionary is
  implemented and unit-tested; a canonicalization test confirms `a⊕b⊕c` and
  `c⊕b⊕a` share an e-class (associativity/commutativity congruence). The
  refutation `0 = 1`-merge test is used in preference to extraction-equals-`1`.

## 4. Constraints & non-goals

**Constraints**
- The ring is **𝔽₂** (`xᵢ² = xᵢ`); the engine is **egg** equality saturation; the
  baseline is a real SAT solver. Correctness is *exact agreement* with SAT.
- Caps are mandatory and `Inconclusive` is a first-class answer (AC3).

**Non-goals (the spec's §5 scope walls)**
- **No EML operator** here (`eˣ − ln y` is analytic over ℝ; this domain is
  discrete). Mixing reintroduces the continuous-where-discrete error.
- **No Clifford / garust / rotors.** Logic has no metric; the geometric path
  stays in the geometric lane.
- **The Hehner / temperature→0 unification is deferred, not denied** — a later
  requirement, only if R-0012 returns positive.
- **No hardware / GAPU** claims unless AC4 returns *buys-something*.

## 5. Open questions (SPEC-0012 decides)

- **Crate vs module.** The source spec says module `ufl_discovery::reasoning::f2`.
  But adding `egg` + a SAT crate would couple the lean matmul-GA
  [`ufl-discovery`](../crates/ufl-discovery) to heavy reasoning deps; "one crate
  per bounded responsibility" (CLAUDE.md §6) argues for a **new `ufl-logic` (or
  `ufl-reasoning`) crate**. Decide with the three-lens.
- **e-graph library.** `egg` (mature, off-the-shelf) vs a hand-rolled completion.
  Lean `egg` for the spike.
- **SAT baseline crate.** `splr` vs `varisat` (pure-Rust, no C build) — pick one.
- **The research surface (§4.4).** Expose a **seeding / rule-ordering hook** —
  Gröbner cost is brutally monomial-order-sensitive, and a *heuristic* there is
  exactly where a positive result, if any, would come from. Confirm whether the
  spike includes this hook or defers it to a follow-up.

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-20 | **Premise corrected: this is a NEW engine + two NEW external deps, not a "reuse."** | The source spec states it *"reuses the existing egg saturation engine in ufl-discovery."* Verified against the tree (2026-06-20): there is **no `egg`, no e-graph, no equality-saturation, no rewriting anywhere in the workspace.** `ufl-discovery` is a **blind genetic-algorithm proposer + verifier-as-discharge** engine (SPEC-0008), deps `ufl-predicate`/`ufl-tensor`/`thiserror`. So R-0012 must *add* `egg` and a SAT crate and build the deduction engine from scratch. The "days on a laptop" estimate can still hold (egg is mature; ~10 rules), but the scope is larger than "reuse." |
| 2026-06-20 | **Accepted as a deliberate THIRD lane** (discrete logic), walled off from the EML-continuous and Clifford-geometric lanes per the source spec's §5. | UFL becomes a three-regime portfolio with separately-stated theses; the Hehner temperature→0 unification ties them but is deferred (§2). Recorded so the broadening is conscious, not drift. |
| 2026-06-20 | **Sequencing: Draft now; does not block R-0011.** | R-0011 (geometric neuroevolution) is the live headline. R-0012 is cheap, orthogonal (touches nothing in the other lanes), and fast to falsify, so it can run **in parallel** or **after** R-0011 at owner discretion — but it does not gate, and is not gated by, the geometric thread. |
| 2026-06-20 | The verdict (AC4) **may be negative** and still satisfy R-0012. | Mirrors R-0008: the value is the falsification + the egg↔Buchberger unification demonstrated in-codebase; a recorded "inert" result is a result, not a failure. |

## Changelog

- 2026-06-20 — created (Draft); transcribed from the owner's 𝔽₂ spec, premise
  corrected (no existing egg engine), framed as the third (discrete-logic) lane.
