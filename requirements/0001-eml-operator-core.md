# R-0001 — EML Operator Core

- **Status:** Draft
- **Milestone:** M1
- **Owner:** Gustavo Delgadillo (Goose)
- **Created:** 2026-05-17
- **Pillar / atom:** Numeric substrate — atom `eml` (supersedes proposal Pillar 1 "Logarithmic Arithmetic Core" and atoms ℒ, ⊕)
- **Depends on:** none
- **Realized by:** SPEC-0001 (pending)
- **QA:** `qa` agent run scoped to R-0001

## 1. Statement

UFL's numeric substrate is a single primitive: the **EML operator**

```
eml(x, y) = exp(x) − ln(y)
```

together with the numeric literal `1`. Every elementary numeric value and
function UFL can express is representable as a binary tree over the grammar

```
S → 1 | <var> | eml(S, S)
```

where each leaf is the literal `1` or a named variable, and every internal node
is `eml`. UFL must provide this representation and a reference evaluator that
computes the value of any such tree.

This requirement establishes `eml` as *the* arithmetic atom of UFL. It does not
cover translating conventional formulas into EML form, optimized or
substrate-compiled evaluation, geometric algebra, or predicates — those are
separate requirements.

## 2. Rationale

The proposal's Pillar 1 ([`docs/ufl-first-draft.md`](../docs/ufl-first-draft.md)
§2) posited a "Logarithmic Arithmetic Core" — the intuition that arithmetic
reduces to log-domain operations. The result in
[`docs/AllEle.html`](../docs/AllEle.html) ("All elementary functions from a
single operator") supersedes that intuition with a precise, stronger fact: a
*single* binary operator, `eml(x,y) = exp(x) − ln(y)`, plus the constant `1`,
generates the entire scientific-calculator repertoire — the constants `e`, `π`,
`i`, all arithmetic, and every elementary transcendental and algebraic function.

This is the continuous analogue of the NAND gate: one repeatable element from
which everything is built. It is exactly the "single sufficient primitive"
UFL's core thesis reaches for, and it makes the substrate-agnostic story
concrete — an EML expression is a uniform binary tree, a clean intermediate
representation that a later substrate layer can compile to a stack machine,
FPGA, or analog circuit.

Adopting `eml` collapses the proposal's six atoms to five: ℒ and ⊕ are both
subsumed by `eml`; `𝒢ₖ`, `∗`, `⟦P⟧`, and `⊗` are unchanged.

## 3. Acceptance criteria

- **AC1 — Representation.** An EML expression is representable as a binary tree
  whose leaves are the literal `1` or a named variable and whose every internal
  node is `eml`. The representation admits exactly the grammar
  `S → 1 | <var> | eml(S, S)` — no other node or leaf kind.
- **AC2 — Reference evaluation.** A closed (variable-free) EML tree evaluates to
  a single complex value. A tree containing variables evaluates to a complex
  value given a binding for every variable it mentions.
- **AC3 — Extended reals.** Evaluation of `ln 0`, `exp(−∞)`, and expressions
  producing signed zeros or infinities follows IEEE-754 semantics and never
  traps, panics, or aborts; such values propagate as ordinary results.
- **AC4 — Branch convention.** EML's `ln` uses one documented branch cut, chosen
  so that the derived quantities `i` and `π`, and `ln x` for real `x < 0`, carry
  the sign of the standard principal branch.
- **AC5 — Known identities.** Each of the following evaluates to within a
  documented tolerance of an independently computed reference value, over a
  sample of inputs that includes negative real `x`:
  - `e = eml(1, 1)`
  - `exp(x) = eml(x, 1)`
  - `ln(x) = eml(1, eml(eml(1, x), 1))`

## 4. Constraints & non-goals

**Constraints**

- The numeric substrate is complex: values are ℂ over IEEE-754 `f64`. Real
  results may require complex intermediates (AllEle §5).
- The literal `1` is the only numeric constant terminal. All other constants
  (`e`, `π`, `i`, …) are *derived* EML trees, never terminals.

**Non-goals** (each a separate, later requirement)

- **EML compiler** — translating arbitrary conventional formulas into EML form.
- **Stable / optimized evaluation** and substrate compilation (stack machine,
  FPGA, analog).
- **Geometric algebra** — multivectors composed over EML scalars.
- **Predicates**, UFL surface-syntax parsing, and trainable EML trees.

## 5. Open questions

- **Q-AC4.** Which exact branch convention — correct EML's `ln` at the operator
  level so all derived quantities follow the principal branch automatically, or
  evaluate naively and correct the `i` sign downstream? AllEle §4.1 describes
  both; SPEC-0001 must pick one and justify it.
- **AC5 tolerance.** The numeric tolerance and the input sample set are to be
  fixed in SPEC-0001 and by the `qa` agent, informed by identity-tree depth
  (e.g. `ln` is a depth-3 tree).

## 6. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-17 | EML becomes UFL's foundational numeric primitive, superseding proposal Pillar 1 and atoms ℒ / ⊕. | A single operator generating all elementary functions is the precise form of UFL's "single sufficient primitive" thesis; collapses six atoms to five. |
| 2026-05-17 | Adopt the **EML** variant `exp(x) − ln(y)` with terminal `1`, over the EDL and `-eml` cousins. | Terminal `1` is exact and singularity-free; the subtraction node has no poles (EDL's division has a pole at `y = 1`); EML is AllEle's primary result, so its verified identity corpus and the Rust `rust_verify` are reusable; subtraction is the better hardware/analog compilation target; the node is pole-free and cleanly differentiable for later trainable trees. |
| 2026-05-17 | Variables are grammar terminals in R-0001 — a leaf may be the literal `1` or a named variable. | R-0001 must cover elementary *functions*, not only constants; AC5's `exp(x)` and `ln(x)` identities are functions of a variable. |
| 2026-05-17 | EML is UFL's canonical IR; R-0001 ships a *reference* (correctness-first) evaluator only. | A correctness evaluator is needed to verify the representation; optimized, stable, and substrate-compiled evaluation belongs to a later layer, keeping R-0001 small. |

## Changelog

- 2026-05-17 — created (Draft).
