# SPEC-0021 — `FormFitness`: the acceptance verdict from a discharged UFL form

- **Realizes:** [R-0021](../requirements/0021-form-fitness.md) (ACs approved 2026-09-16).
- **Status:** **Draft** — awaiting the three-lens (CLAUDE.md §4 step 2).
- **Crates touched:** `ufl-discovery` (the new instance + its tests). `ufl-search`,
  `ufl-predicate`, `ufl-syntax`, `ufl-core` are **used, not modified**.

## 1. The shape

`MatmulFitness` (`generic.rs:37-49`) is four lines of Rust:

```rust
fn score(&self, g: &Genome) -> Result<i64, EngineError> { Ok(self.predicate.residual(&express(g)?)?) }
fn solved(&self, score: &i64) -> bool { *score == 0 }
```

`FormFitness` keeps `score` **identical** — the same `RankDecomposition::residual`,
so cost and trajectory are bit-identical — and replaces `solved` with a
**discharge of a verifier-held `Sexpr`**:

```rust
/// The acceptance property as a UFL form, discharged per candidate.
///
/// `score` is [`MatmulFitness`]'s, unchanged — the *cost* is the verifier's
/// residual either way, so a run's trajectory is unaffected (§4.2). Only the
/// **accept decision** moves: from `*score == 0` in Rust to
/// `(= residual zero)` discharged through [`Predicate for Sexpr`].
pub struct FormFitness<'a> {
    predicate: &'a RankDecomposition,
    /// The acceptance form. **Verifier-held**: constructed from [`ACCEPT_FORM`]
    /// at `new`, never supplied, mutated, or read by a proposer (§2.3).
    form: Sexpr,
}
```

## 2. Design

### 2.1 The form, and why its constant is a binding

```rust
/// The acceptance property: the candidate's residual against the verifier's
/// target is exactly zero.
///
/// `zero` is a **variable the verifier binds**, not the literal `0` — R-0001's
/// grammar is `S → 1 | var | eml(S, S)`, so `1` is the only numeric literal that
/// lowers and `(= residual 0)` is `Err(UnsupportedLiteral)` (R-0021 §2, measured).
/// Binding it is the better design, not a workaround: the verifier then supplies
/// the form *and every constant it compares against*, so an acceptance form
/// cannot be written without the verifier's own numbers (§2.3).
const ACCEPT_FORM: &str = "(= residual zero)";
```

Parsed **once** in `FormFitness::new`, not per candidate — `read` is fallible and
a malformed form is a construction error, not a per-score error (§2.4).

### 2.2 `solved` — the discharge

```rust
fn solved(&self, score: &i64) -> bool {
    self.discharge(*score).unwrap_or(false)   // ← NOT this; see §2.4
}
```

**That signature is the problem this spec has to solve.** `Fitness::solved`
returns `bool` (`ufl-search/src/lib.rs:53`) — it has **no error channel**. A
discharge can fail (unbound variable, non-boolean form, `UnsupportedLiteral`), and
`unwrap_or(false)` would turn a broken predicate into "not solved" — a search that
silently never accepts anything. R-0021 **AC4** forbids exactly this.

Three resolutions, and §5 Q1 asks the lens to choose:

1. **Validate at construction, so `solved` cannot fail.** `FormFitness::new`
   discharges the form against a **probe state** (`residual = 0, zero = 0` ⇒ must
   be `Ok(true)`; `residual = 1, zero = 0` ⇒ must be `Ok(false)`). If either
   probe does not hold, `new` returns `Err`. Then in `solved` the only remaining
   failure is a residual outside the §3 envelope, which is checked in `score`.
   A residual discharge that still fails is genuinely unreachable and gets an
   `unreachable!` with a justifying message — CLAUDE.md §6's first case.
2. **Widen the trait** with `fn solved(&self, s: &S) -> Result<bool, Self::Error>`.
   Honest, but it changes `ufl-search`'s public seam for every lane and every
   existing `Fitness` impl, for one instance's benefit. Rejected unless the lens
   disagrees.
3. **Move the verdict into `score`** — return a cost that already encodes
   acceptance. Rejected: it conflates cost with verdict, which is the exact
   coupling R-0008 AC2 established (`residual` *is* both, computed once) and
   would make the form's failure invisible.

**This spec takes (1).** It is the only option that neither widens a shared seam
nor hides a failure: the fallible part happens once, at construction, where an
error can be returned.

### 2.3 C1/C3 by construction (AC3)

- `form` is a **private field**; `FormFitness` exposes no setter and no getter.
- `FormFitness::new(predicate)` takes **only the verifier's predicate** — there
  is no parameter through which a caller could supply a form.
- `ACCEPT_FORM` is a private `const` in this module, not `pub`.
- The `Proposer` trait (`ufl-search/src/lib.rs:27-32`) has `seed` and `vary`, both
  taking only `&self`, `rng`, and `&[(G, S)]` — so a proposer's method signatures
  cannot even *name* a `Fitness`, let alone reach its form. AC3's test asserts
  this negative at the type level: a compile-fail test (or a documented
  type-level argument, §5 Q3) that no path from `Proposer` reaches `form`.

The transparency is one-directional by design: a reader can see the acceptance
property as UFL source; the search cannot touch it.

### 2.4 The failure channel (AC4)

| failure | where it surfaces |
|---|---|
| `ACCEPT_FORM` does not parse | `new` → `Err` (a `const`, so this is a programming error caught by the first test to construct one) |
| the form is not boolean-shaped | `new`'s probe → `Err` |
| the form does not mean "residual == 0" | `new`'s two probes → `Err` (this is the load-bearing one: it catches a form that parses and discharges but decides the *wrong property*) |
| `residual` outside the §3 envelope | `score` → `Err(Fitness::Error)` |
| anything else at `solved` time | **unreachable** by §2.2(1), with a justifying message |

A `false` verdict and a broken predicate are therefore distinguishable at every
point — which R-0021 §4 names as the one way a form-based fitness could silently
corrupt a search.

### 2.5 `Fitness::Error`

`MatmulFitness::Error = EngineError`. `FormFitness` needs to surface
`CheckError`/`PredError` too. Options: add a variant to `EngineError`
(`FormDischarge(CheckError)`), or give `FormFitness` its own error type that
`From`-converts. §5 Q2 asks which — the first keeps the lane's error channel
single, the second keeps `EngineError` free of a concern only one instance has.

## 3. The envelope (AC5)

`residual` is `i64`; `State` binds `ufl_core::Value` (`Complex<f64>`). The
conversion is exact only for |residual| < 2⁵³ (R-0021 §3, measured:
`9007199254740993` binds as `…992`). `FormFitness::score` **rejects** an
out-of-envelope residual as a typed error rather than documenting it, because a
silently-collapsed comparison is indistinguishable from a correct one, and the
check is a single `i64::abs() < 1 << 53`.

Unreachable for matmul at the ranks under test; the guard is cheap and the
alternative is a wrong verdict nobody can see.

## 4. Tests (TDD — written first, red)

1. **T-parity (AC2)** — the deliverable. On the full `r_0014_generic_seam.rs`
   sweep (the four `(n, rank, gens, pop)` cases × seeds `0..6`, plus the solvable
   run), `FormFitness::solved` and `MatmulFitness::solved` return **the same
   verdict for every candidate scored** — not merely the same final `Outcome`.
   Implemented by a wrapper that calls both and asserts agreement per call, so a
   divergence names the candidate.
2. **T-form-is-the-property** — the two construction probes as an explicit test:
   the form is `Ok(true)` at `residual = 0` and `Ok(false)` at `residual ∈
   {1, −1, 7}`. This is what distinguishes "a form that discharges" from "a form
   that decides the right property".
3. **T-brief-form-fails** — `(= residual 0)` is `Err(UnsupportedLiteral)`,
   pinned. R-0021 §2's measurement as a regression guard: if `lower` ever accepts
   arbitrary literals, this test fails and the §2.1 rationale must be revisited.
4. **T-failure-channel (AC4)** — a deliberately broken form (`(= residual
   missing)` with `missing` unbound; a non-boolean `(eml 1 1)`) is an `Err` from
   `new`, never a `false` verdict.
5. **T-envelope (AC5)** — `residual = 2⁵³` is a typed `Err` from `score`;
   `2⁵³ − 1` is `Ok`.
6. **T-verifier-held (AC3)** — the type-level assertion of §2.3.
7. **T-cost (AC6)** — per-candidate overhead of the discharge vs `*score == 0`,
   measured on the sweep, printed **unconditionally**, no threshold asserted
   (*Assert the Protocol, Not the Outcome*). Release-gated like
   `r_0019_cap_probe.rs`.

## 5. Open questions for the three-lens

1. **§2.2 — is "validate at construction" sound, or is it hiding the error
   channel a `bool` return should have?** The alternative widens
   `Fitness::solved` to `Result` across a shared seam. Which is the better
   contract? Is there a fourth option?
2. **§2.5 — `EngineError::FormDischarge` or a `FormFitness`-owned error type?**
3. **§2.3's AC3 test — can "no proposer path reaches the form" be asserted
   mechanically**, or is it only a type-level argument in prose? A
   `compile_fail` doctest is possible but brittle; a `trybuild` dev-dependency is
   heavier than the claim.
4. **Is this worth building at all?** It adds a second acceptance path to a
   verifier whose trustworthiness is the project's foundation, to gain a
   *transparency window* — the acceptance property becomes readable as UFL source
   — and to make C3 structural for the Rung-4 loop that R-0015 measured as
   having no headroom. The honest case is that it is the **first** UFL form in
   the search loop and the cheapest possible one; the honest risk is that it is
   machinery on the path that must stay trustworthy. §1 keeps `score` untouched
   precisely to bound that risk. Hater: is that bound real?
