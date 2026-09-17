# R-0021 — `FormFitness`: the acceptance property as a discharged UFL form

- **Status:** **SHELVED** (2026-09-16, Gustavo) — not killed. ACs were approved
  and the three-lens then found the spec as drafted should not be built
  ([SPEC-0021 §7](../specs/0021-form-fitness.md#7-three-lens-round-1--what-the-lenses-measured-and-the-recommendation)).
  Decision and what survives: [`decisions/0004`](../decisions/0004-shelve-form-fitness.md).
  **The finding this requirement produced is §1.1 below — read that even if you
  never build this.**
- **Milestone:** M5 · rung 3 of the self-eval staircase (*the language scores itself* —
  in the bounded sense §4 pins, never the autonomous one).
- **Tracks:** [#66](https://github.com/westerngazoo/ufl/issues/66) (T10), brief at
  `docs/tasks/10-add-formfitness-verifier-held-fitness-as-dis.md`.
- **Builds on:** R-0004 (`impl Predicate for Sexpr`), R-0014 (the `ufl-search`
  seam), R-0008 (`RankDecomposition::residual`).

## 1.1 The finding, hoisted — the form language has no ordering relation

This is what #66's KILL clause asked for, and it is the durable output of R-0021
whether or not any code ships.

The predicate language's boolean heads are **exactly**
`{and, or, not, =, eq?, pred}` (`ufl-predicate/src/eval_pred.rs:35-37`).
**There is no ordering relation at any arity.** Measured:

```
(<= rot_err eps)  →  Err(Pred(ExpectedBool { found: "form `<=`" }))
(<  rot_err eps)  →  Err(Pred(ExpectedBool { found: "form `<`"  }))
```

`GeoFitness::solved` is `score.value() <= 1e-6` (`ufl-evolve/src/memetic.rs:112`).
So **form-as-acceptance works exactly for a lane whose criterion is an exact
`== 0`** — matmul — and not for the geometric lane, which is every other lane so
far.

The missing form is **one ordering head**, and it is *not* a drop-in: `Value` is
`Complex<f64>`, which has **no total order**. So the next predicate-layer
requirement must *decide* the semantics rather than add a symbol:

- require `im == 0` and compare `re`, with a new typed `PredError::ExpectedReal`
  (consistent with this project's preference for typed refusal over silent
  projection); or
- compare `norm()` — total, but conflates `+1` and `−1`; or
- `total_cmp` on `re` alone, ignoring `im` silently.

A fourth fact for whoever writes that requirement: `RotErr::new` already
normalises the non-finite class at construction, so the geo form needs no NaN
clause — **the cost type owns the binding's totality; the form only compares.**
That rule is derivable from two lanes written independently, and it is the
cleanest transferable thing R-0021 produced.

## 2. What this is

`ufl-discovery`'s `MatmulFitness` (`generic.rs:41-49`) decides acceptance in Rust:

```rust
fn solved(&self, score: &i64) -> bool { *score == 0 }
```

R-0021 adds a **second, parallel** implementation whose verdict comes from
discharging a **UFL `Sexpr` predicate** through the existing
`impl Predicate for Sexpr` (`ufl-predicate/src/predicate.rs:54-61`) — the
s-expression *is* the Hehner predicate — with the verifier's computed quantities
bound into `State` as variables.

This is the first time a UFL form participates in the search loop at all. It
converts the C3 constraint (*reward is the exact verifier verdict, never a
proxy* — `theory/two-language-substrate.md`) from prose into a typed seam, so
the Rung-4 meta-loop inherits it structurally rather than by discipline.

## 2. The brief's proposed form does not lower — measured

The brief (#66 §Work 1) proposes the form `(= residual 0)`. **It fails.**
Measured through `ufl_predicate::check_str`:

| form | bindings | result |
|---|---|---|
| `(= residual 0)` | `residual = 0` | **`Err(Lower(UnsupportedLiteral(0.0)))`** |
| `(= residual 1)` | `residual = 1` | `Ok(true)` |
| `(= residual zero)` | `residual = 0`, `zero = 0` | `Ok(true)` |
| `(= residual zero)` | `residual = 7`, `zero = 0` | `Ok(false)` |
| `(and (= residual zero) (not (= rank zero)))` | `residual = 0`, `rank = 7`, `zero = 0` | `Ok(true)` |
| `(= residual zero)` | `residual = −3`, `zero = 0` | `Ok(false)` |

The cause is R-0001's grammar, `S → 1 | var | eml(S, S)`: **`1` is the only
numeric literal that lowers.** `lower` rejects every other `Num` as
`UnsupportedLiteral` (`ufl-syntax/src/lower.rs`).

### 2.1 The KILL condition does not fire — and the workaround is the better design

#66's KILL clause says: *if the `{pred, =, and, or, not, true, false}` language
cannot express the acceptance property without ad-hoc new forms, record which
forms are missing rather than widening silently.*

**No form is missing.** The property is expressible as `(= residual zero)` with
**`zero` bound by the verifier as a state variable.** Two other routes exist and
are rejected:

- *Widen `lower` to accept arbitrary literals* — changes R-0001's accepted
  grammar for a convenience. Refused.
- *Write zero as an `eml` tree* — R-0001 AC5's shipped identity encodes `0` in 7
  nodes (`node(1, exp(e))`), so `(= residual (eml 1 …))` would lower. Correct
  but unreadable, and it buries a constant in an expression the reader must
  decode.

Binding the constant is **better than the brief's literal**, not merely
equivalent: the verifier then supplies the form *and every constant it compares
against*. An acceptance form cannot be written at all without the verifier
handing over its own numbers — C1/C3 enforced by what the grammar refuses to
express, rather than by a rule someone must remember.

## 3. The envelope — stated, because `i64 → Value` is not total

`residual` is `i64` (`ufl-discovery/src/predicate.rs:68`); `State` binds
`ufl_core::Value`, a `Complex<f64>`. The conversion is **exact only for
|residual| < 2⁵³**. Measured: `9007199254740993` (2⁵³+1) binds as
`9007199254740992.0`, so two distinct residuals one apart compare **equal**.

This is not reachable for matmul today — the residual is a sum of squared
integer errors over a dim-n² tensor, far below 2⁵³ at the ranks under test — but
it is a real edge of the seam and the spec must state it rather than discover it.
`FormFitness` must either reject an out-of-envelope residual as a typed error or
document the envelope on the public item; the spec decides which.

## 4. What this must NOT claim

- **Not "the language scores itself"** in any autonomous sense. The verdict is
  still the Rust discharge (`eval_pred` over `Env`); the form is **verifier-held
  and never evolved**. A proposer cannot supply, rewrite, or read it.
- **Not a new form.** `{pred, =, and, or, not, true, false}` suffices (§2.1); if
  the spec finds it does not for some lane, that lane's missing forms are
  recorded as the next predicate-layer requirement.
- **Not a replacement** for `MatmulFitness`. Both exist; the parity test is the
  deliverable. Replacing the hand-coded fitness would put the acceptance
  decision behind a lowering pass, which is more machinery on the path that
  must stay trustworthy.
- **Not a performance claim.** Discharging a form per candidate is strictly more
  work than `*score == 0`. The cost is the transparency window's price; the spec
  measures it and states it.

## 5. Acceptance criteria — **approved 2026-09-16**

- **AC1 (the form decides).** A `FormFitness` implements
  `ufl_search::Fitness<Genome, i64>`, with `score` delegating to the same
  `RankDecomposition::residual` `MatmulFitness` uses, and **`solved` produced by
  discharging a verifier-held `Sexpr`** through `impl Predicate for Sexpr`, with
  `residual` and the comparison constant bound into `State`.
- **AC2 (byte-identical verdicts).** On the full `r_0014_generic_seam.rs` sweep —
  the four `(n, rank, gens, pop)` cases × seeds `0..6`, plus the solvable run —
  `FormFitness` and `MatmulFitness` return **the same `solved` verdict for every
  candidate scored**, not merely the same final `Outcome`. The AC2 byte-identity
  discipline of R-0014, reused.
- **AC3 (C1/C3 pinned by type, not prose).** The form is owned by the verifier
  side: no `Proposer` method can reach it, and `FormFitness` exposes no setter.
  A test asserts the negative — the form is not reachable from the proposer's
  API surface.
- **AC4 (the failure channel is typed).** An undischargeable form (unbound
  variable, non-boolean result, `UnsupportedLiteral`) surfaces as a typed
  `Fitness::Error`, never a panic and never a silent `false`. A `false` verdict
  and a *broken predicate* must be distinguishable — this is the one way a form-
  based fitness can silently corrupt a search, so it is an AC rather than a note.
- **AC5 (the envelope).** §3's 2⁵³ limit is either rejected as a typed error or
  documented on the public item, and a test pins whichever is chosen.
- **AC6 (the cost, measured not assumed).** The per-candidate overhead of
  discharging the form versus `*score == 0` is measured on the sweep and reported
  **unconditionally**. No threshold is asserted (*Assert the Protocol, Not the
  Outcome*).

## 6. Open questions for the three-lens

1. **Is a parallel `FormFitness` the right shape, or should `MatmulFitness`
   *delegate* to it?** Parallel keeps the trusted path untouched and makes AC2 a
   real differential; delegation makes the form load-bearing and would prove more
   — but it puts a lowering pass on the acceptance decision. §4 argues parallel;
   the lenses should test that.
2. **Where does `FormFitness` live?** `ufl-discovery` (with the lane) or
   `ufl-search` (with the trait)? The latter would make `ufl-search` depend on
   `ufl-predicate` and `ufl-syntax`, which the seam has so far avoided —
   `ufl-search` is deliberately genome-agnostic. Recommendation:
   `ufl-discovery`.
3. **Does the form belong in a file, a `const &str`, or a built `Sexpr`?** A
   `const &str` parsed at construction is readable and puts the form where a
   reviewer sees it; it also moves a `read` failure to runtime. A built `Sexpr`
   cannot fail to parse but is unreadable as a *form*.
4. **Should the constant be named `zero` or something self-documenting?** `(=
   residual zero)` reads well but `zero` is a binding a reader must trust. Is
   there a naming convention for verifier-supplied constants worth setting here,
   since every future `FormFitness` will need them (§2.1)?
