# FINDINGS — `ufl-discovery` PRD review (pre-implementation)

*Requested by the PRD §0: read the existing UFL pieces, answer four context
questions against the real code, and flag every place the PRD's assumptions
contradict reality. **The real code wins.** This is a review artifact — no
`ufl-discovery` code exists yet.*

Reviewed at `main` = `7377a9f` (R-0001, R-0003, R-0004 merged; R-0002 garust
contract accepted, built by a separate flow; R-0005 value-conditional in
flight on a branch).

---

## §0 — The four context questions, answered against real code

### Q1 · EML core — representation, scalar field, eval entry point

- **Representation:** a recursive, `Box`-ed enum, **not** const generics —
  `crates/ufl-core/src/eml.rs`:
  ```rust
  pub enum Eml { One, Var(String), Node { exp_arg: Box<Eml>, log_arg: Box<Eml> } }
  ```
  Grammar `S → 1 | <var> | eml(S, S)`. The **only numeric literal is `1`**;
  every other constant is a *derived* tree.
- **Scalar field:** `pub type Value = Complex<f64>;`
  (`crates/ufl-core/src/eval.rs:14`). **Complex, not integer.** Not garust
  multivectors (garust is unrelated to the EML core).
- **Eval entry point:** `ufl_core::eval(&Eml, &Env) -> Result<Value, EvalError>`
  (`eval.rs:62`). There is also a text surface in `ufl-syntax`:
  `read` → `lower` (`Sexpr → Eml`) → `eval`, exposed as
  `ufl_syntax::eval_str(&str, &Env) -> Result<Value, UflError>`.

### Q2 · ufl-predicate (R-0004) — refinement / discharge signature

- **There is no `refines(impl_, spec) -> bool`.** The PRD's central assumption
  here does not match the code. ufl-predicate is a **checker**, not a refinement
  engine. Public surface (`crates/ufl-predicate/src/`):
  ```rust
  pub fn eval_pred(&Sexpr, &Env) -> Result<bool, PredError>          // boolean eval
  pub fn check(&Sexpr, pre: &[(&str, Value)], post: &[(&str, Value)]) -> Result<bool, CheckError>
  pub fn check_str(&str, pre, post) -> Result<bool, CheckError>
  ```
  `check` evaluates a boolean predicate over a **pre/post scalar state** (vars →
  `Complex<f64>`), with post-vars bound primed (`x'`). Forms: `true`/`false`,
  `(= a b)` (exact IEEE `Complex` equality), `(and/or/not …)`, `(pred E)`.
- **Numeric evaluation only — no solver.** It *checks* whether a concrete state
  satisfies a predicate (decidable); it does not *search* for a satisfying
  state, and there is no refinement-of-spec-by-impl relation.
- **It cannot express the PRD's `P_n,R` today.** Three hard blockers:
  1. No tensors / no vectors / no outer products — only scalar `Complex<f64>`.
  2. `=` compares two scalar values, not a tensor sum against a target.
  3. The language **cannot even write `0` or `2`** — only `1` is a literal
     (`lower.rs:36` → `UnsupportedLiteral` for anything else). The discovery
     task's `{-1, 0, +1}` coefficients are not expressible as UFL literals.

### Q3 · garust — multivector API + the Scalar split

- **garust is NOT a workspace dependency.** It is a separate sibling library at
  `~/projects/garust`; no `ufl-*` crate depends on it (`grep garust crates/*/Cargo.toml`
  → none).
- Its API (from its README): `Multivector<T, P, Q, R, DIM>`, aliases `Vga3`
  (= `Cl(3,0,0)`), `Pga3`, `Cga3`; geometric product, `wedge`, `inner`,
  `reverse`, grade projection, `sandwich`, closed-form `exp`. Generic over the
  scalar type via a `Scalar` trait.
- **The Scalar split is PENDING.** garust's `Scalar` requires `PartialOrd` +
  `abs(self) -> Self`, which `Complex<f64>` can't satisfy. Splitting it (a
  `Ring`/`Field` super-trait for the product, ordered ops in a refinement) is
  the R-0002 prerequisite, **being built by a separate GA agent flow** — not
  done.

### Q4 · egg / equality saturation

- **Not integrated. Not present anywhere** (`grep -ri egg crates/ Cargo.toml` →
  none). Phase 2's "search-space quotienting via egg" must be **stubbed**.

---

## Contradictions with the PRD (real code wins)

| # | PRD assumption | Reality | Severity |
|---|----------------|---------|----------|
| C1 | "bridge through ufl-predicate so the verifier IS the Hehner discharge of `P_n,R`"; `refines(impl_, spec)` | ufl-predicate is a scalar **checker** with no `refines`, no tensors, and **no integer/`0`/`2` literals**. It cannot express `P_n,R` at all today. | **Blocking** for the "verifier routes through the predicate layer" claim |
| C2 | Building "on top of the existing UFL substrate (EML core, ufl-predicate, garust)" | For `{-1,0,+1}` integer tensor decomposition, **none of the three carry the task**: EML is `Complex<f64>` (float, wrong field), the predicate layer can't express tensors, garust isn't integrated and isn't needed for plain integer tensors. | **Major** — the substrate-fit framing is thin for v1 |
| C3 | EML core could be reused for tensor reconstruction (§7 Q4) | EML evaluates over `Complex<f64>`; reusing it for exact `{-1,0,+1}` reconstruction buys nothing and risks float drift. | Major — answered below: use a dedicated integer path |
| C4 | egg available for Phase 2 (§7 Q2) | Absent. | Minor — stub it (the PRD allows this) |
| C5 | garust available (§0.3) | Not a dependency; Scalar split pending in another flow. | Minor for this task — garust is not actually needed for integer tensors |

The PRD's own §5 and §7 **anticipated C1/C3** ("If the predicate crate can't yet
express tensor-equality predicates, note the gap and implement reconstruction
directly in `verify.rs` with a TODO"; "is a dedicated integer reconstruction
path cleaner?"). This review confirms: **yes, take the direct integer path; the
predicate bridge is a future aspiration, not available now.**

---

## §7 open questions — answered

1. **Does ufl-predicate support tensor-equality predicates today?** No (C1).
   Phase 0–1 must verify directly in `verify.rs` with exact integer
   reconstruction. Routing through the predicate layer requires first extending
   it with integer literals + a tensor/vector value kind + an equality over
   them — a substantial requirement of its own (see Recommendation).
2. **Is egg available?** No (C4). Stub Phase-2 quotienting.
3. **`{-1,0,+1}` strict vs small integer range?** Recommend **strict** — matches
   AlphaTensor, keeps the verifier exact and the genotype small. The verifier
   should accumulate in `i64` regardless (sums of products can exceed `i8`).
4. **What field does EML evaluate over, and does reuse buy anything?**
   `Complex<f64>` — reuse buys **nothing** for this task. Use a dedicated,
   exact **integer** reconstruction path (`i64` accumulation over `i8` triples).
   (C3.)

---

## Fit with UFL's goal & requirements — the honest assessment

**The mechanism is sound; the "built on the UFL substrate" claim is, for v1,
mostly aspirational.** Phases 0–1 of `ufl-discovery` are — by this review — a
**self-contained integer-tensor GA search with an exact verifier**. The three
substrate pieces the PRD names don't actually carry it:

- The genuine UFL connection is *conceptual* — "a discovery is a Hehner
  predicate discharged exactly" — but the predicate layer (R-0004) can't yet
  express the tensor predicate, so v1 realizes that idea in standalone code with
  a TODO, not through the substrate.
- This is fine **as a research result** (rediscovering Strassen is compelling
  regardless of plumbing), but it should be stated honestly: v1 is "a discovery
  engine that lives in the UFL repo," not "UFL's predicate substrate
  discovering algorithms." The stronger thesis ("more general than AlphaTensor
  *because it rides UFL's verifier*") only becomes true once C1 is closed.

**Process fit (CLAUDE.md).** UFL builds nothing without an accepted requirement
+ spec. This PRD is a large, new milestone — well outside the current roadmap
(M1–M4: build the language). It should become its own requirement (or a small
requirement family) and pass the three-lens review before code. The PRD is
high-quality input for that: falsifiable phase gates, honest non-goals, exact
verifier. Two paths to reconcile the substrate tension:

- **Path A (pragmatic, recommended for the result):** build `ufl-discovery` v1
  as a standalone integer GA + exact verifier (Phases 0–1), predicate bridge as
  a documented TODO. Fastest route to the Strassen-rediscovery result that
  proves the mechanism. Captures the PRD's own §5/§8 intent.
- **Path B (substrate-true):** first extend the language — **integer literals**
  (the deferred "arbitrary literals" thread) and a **tensor/vector value kind +
  equality predicate** in ufl-predicate — so the verifier genuinely *is* the
  Hehner discharge. Slower; makes the headline thesis real.

A clean middle: **Path A for Phases 0–1 to get the result, with a parallel
requirement to close C1** (integer literals + tensor-equality predicate) so
Phase 2+ can route through the substrate and the "more general than AlphaTensor"
claim is earned, not asserted.

## Recommended next steps

1. Decide **priority**: is `ufl-discovery` the new headline direction, a
   parallel track to the language build (R-0005 / GA forms), or a future
   milestone? (Owner call — it's a major scope expansion.)
2. If proceeding, turn the PRD into a **requirement** `R-NNNN` (likely a new
   milestone, e.g. "M5 — Discovery") and run the three-lens review on the
   design. Record C1–C5 as accepted constraints/TODOs.
3. Build Phase 0 first (the Strassen fixture + integer verifier) — its acceptance
   gate ("hardcoded 7-term scheme reconstructs `T_2` with error 0") is the
   cheapest possible proof the verifier is correct, and it needs none of the
   contested substrate.
4. Track C1 (predicate layer can't express the tensor predicate) as a real
   requirement if the substrate-true thesis matters; otherwise document it as a
   permanent v1 boundary.
